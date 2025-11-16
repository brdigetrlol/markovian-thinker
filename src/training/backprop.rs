//! Backpropagation engine for gradient computation

use anyhow::Result;
use std::collections::HashMap;

#[cfg(feature = "gpu")]
use crate::gpu::CudaContext;

/// Loss function types
#[derive(Debug, Clone, Copy)]
pub enum LossFunction {
    /// Cross-entropy loss for classification
    CrossEntropy,
    /// Mean squared error for regression
    MSE,
    /// Contrastive loss for embeddings
    Contrastive,
}

/// Backpropagation engine for computing gradients
pub struct BackpropEngine {
    loss_fn: LossFunction,
    gradients: HashMap<String, Vec<f32>>,

    #[cfg(feature = "gpu")]
    gpu_context: Option<Arc<CudaContext>>,
}

impl BackpropEngine {
    /// Create a new backpropagation engine
    pub fn new(loss_fn: LossFunction) -> Self {
        Self {
            loss_fn,
            gradients: HashMap::new(),
            #[cfg(feature = "gpu")]
            gpu_context: None,
        }
    }

    #[cfg(feature = "gpu")]
    /// Create backprop engine with GPU acceleration
    pub fn new_with_gpu(loss_fn: LossFunction, gpu_context: Arc<CudaContext>) -> Self {
        Self {
            loss_fn,
            gradients: HashMap::new(),
            gpu_context: Some(gpu_context),
        }
    }

    /// Compute loss and gradients
    pub fn compute_loss(&mut self, predictions: &[f32], targets: &[f32]) -> Result<f32> {
        match self.loss_fn {
            LossFunction::CrossEntropy => self.cross_entropy_loss(predictions, targets),
            LossFunction::MSE => self.mse_loss(predictions, targets),
            LossFunction::Contrastive => self.contrastive_loss(predictions, targets),
        }
    }

    /// Cross-entropy loss
    fn cross_entropy_loss(&self, predictions: &[f32], targets: &[f32]) -> Result<f32> {
        let mut loss = 0.0;

        for i in 0..predictions.len() {
            // Clip to avoid log(0)
            let p = predictions[i].clamp(1e-7, 1.0 - 1e-7);
            loss -= targets[i] * p.ln();
        }

        Ok(loss / predictions.len() as f32)
    }

    /// Mean squared error loss
    fn mse_loss(&self, predictions: &[f32], targets: &[f32]) -> Result<f32> {
        let mut loss = 0.0;

        for i in 0..predictions.len() {
            let diff = predictions[i] - targets[i];
            loss += diff * diff;
        }

        Ok(loss / predictions.len() as f32)
    }

    /// Contrastive loss for embeddings
    fn contrastive_loss(&self, predictions: &[f32], targets: &[f32]) -> Result<f32> {
        // Simple cosine similarity-based loss
        let mut dot = 0.0;
        let mut norm_p = 0.0;
        let mut norm_t = 0.0;

        for i in 0..predictions.len() {
            dot += predictions[i] * targets[i];
            norm_p += predictions[i] * predictions[i];
            norm_t += targets[i] * targets[i];
        }

        let similarity = dot / (norm_p.sqrt() * norm_t.sqrt() + 1e-8);
        Ok(1.0 - similarity) // Loss is 1 - similarity
    }

    /// Compute gradients for predictions
    pub fn compute_gradients(&mut self, predictions: &[f32], targets: &[f32]) -> Result<Vec<f32>> {
        match self.loss_fn {
            LossFunction::CrossEntropy => self.cross_entropy_gradients(predictions, targets),
            LossFunction::MSE => self.mse_gradients(predictions, targets),
            LossFunction::Contrastive => self.contrastive_gradients(predictions, targets),
        }
    }

    /// Cross-entropy gradients
    fn cross_entropy_gradients(&self, predictions: &[f32], targets: &[f32]) -> Result<Vec<f32>> {
        let mut grads = Vec::with_capacity(predictions.len());

        for i in 0..predictions.len() {
            // Gradient: -target/prediction
            let p = predictions[i].clamp(1e-7, 1.0 - 1e-7);
            grads.push(-targets[i] / p);
        }

        Ok(grads)
    }

    /// MSE gradients
    fn mse_gradients(&self, predictions: &[f32], targets: &[f32]) -> Result<Vec<f32>> {
        let mut grads = Vec::with_capacity(predictions.len());

        for i in 0..predictions.len() {
            // Gradient: 2 * (prediction - target)
            grads.push(2.0 * (predictions[i] - targets[i]));
        }

        Ok(grads)
    }

    /// Contrastive gradients
    fn contrastive_gradients(&self, predictions: &[f32], targets: &[f32]) -> Result<Vec<f32>> {
        let mut grads = Vec::with_capacity(predictions.len());

        // Compute similarity
        let mut dot = 0.0;
        let mut norm_p = 0.0;
        let mut norm_t = 0.0;

        for i in 0..predictions.len() {
            dot += predictions[i] * targets[i];
            norm_p += predictions[i] * predictions[i];
            norm_t += targets[i] * targets[i];
        }

        let norm_p_sqrt = (norm_p + 1e-8).sqrt();
        let norm_t_sqrt = (norm_t + 1e-8).sqrt();

        // Gradient of cosine similarity
        for i in 0..predictions.len() {
            let grad = (targets[i] * norm_p_sqrt - predictions[i] * dot / norm_p_sqrt) /
                       (norm_p_sqrt * norm_t_sqrt);
            grads.push(-grad); // Negative because we minimize 1 - similarity
        }

        Ok(grads)
    }

    /// Backward pass through embedding layer
    pub fn backward_embedding(
        &mut self,
        token_ids: &[usize],
        output_grads: &[f32],
        vocab_size: usize,
        embed_dim: usize,
    ) -> Result<Vec<f32>> {
        // Initialize gradient tensor
        let mut embed_grads = vec![0.0; vocab_size * embed_dim];

        // Accumulate gradients for each token
        for (token_idx, &token_id) in token_ids.iter().enumerate() {
            if token_id >= vocab_size {
                continue;
            }

            let offset = token_id * embed_dim;
            let grad_offset = token_idx * embed_dim;

            for d in 0..embed_dim {
                embed_grads[offset + d] += output_grads[grad_offset + d];
            }
        }

        Ok(embed_grads)
    }

    /// Get stored gradients for a parameter
    pub fn get_gradients(&self, name: &str) -> Option<&Vec<f32>> {
        self.gradients.get(name)
    }

    /// Store gradients for a parameter
    pub fn store_gradients(&mut self, name: String, grads: Vec<f32>) {
        self.gradients.insert(name, grads);
    }

    /// Clear all gradients
    pub fn zero_gradients(&mut self) {
        self.gradients.clear();
    }

    /// Clip gradients by norm
    pub fn clip_gradients_by_norm(&mut self, max_norm: f32) -> Result<()> {
        let mut total_norm = 0.0;

        // Compute total norm
        for grads in self.gradients.values() {
            for &g in grads {
                total_norm += g * g;
            }
        }

        total_norm = total_norm.sqrt();

        // Clip if needed
        if total_norm > max_norm {
            let clip_coef = max_norm / (total_norm + 1e-6);

            for grads in self.gradients.values_mut() {
                for g in grads {
                    *g *= clip_coef;
                }
            }
        }

        Ok(())
    }

    /// Clip gradients by value
    pub fn clip_gradients_by_value(&mut self, clip_value: f32) -> Result<()> {
        for grads in self.gradients.values_mut() {
            for g in grads {
                *g = g.clamp(-clip_value, clip_value);
            }
        }

        Ok(())
    }
}

/// Compute gradient for a single layer
pub fn compute_layer_gradient(
    input: &[f32],
    _output: &[f32],
    output_grad: &[f32],
    weights: &[f32],
    input_dim: usize,
    output_dim: usize,
) -> Result<(Vec<f32>, Vec<f32>)> {
    // Gradient w.r.t. weights: input^T * output_grad
    let mut weight_grads = vec![0.0; weights.len()];

    for i in 0..output_dim {
        for j in 0..input_dim {
            weight_grads[i * input_dim + j] = input[j] * output_grad[i];
        }
    }

    // Gradient w.r.t. input: weights^T * output_grad
    let mut input_grads = vec![0.0; input_dim];

    for j in 0..input_dim {
        for i in 0..output_dim {
            input_grads[j] += weights[i * input_dim + j] * output_grad[i];
        }
    }

    Ok((weight_grads, input_grads))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mse_loss() {
        let engine = BackpropEngine::new(LossFunction::MSE);
        let predictions = vec![1.0, 2.0, 3.0];
        let targets = vec![1.0, 2.0, 3.0];

        let loss = engine.mse_loss(&predictions, &targets).unwrap();
        assert!(loss < 1e-6);
    }

    #[test]
    fn test_mse_gradients() {
        let engine = BackpropEngine::new(LossFunction::MSE);
        let predictions = vec![2.0, 3.0, 4.0];
        let targets = vec![1.0, 2.0, 3.0];

        let grads = engine.mse_gradients(&predictions, &targets).unwrap();
        assert_eq!(grads, vec![2.0, 2.0, 2.0]);
    }

    #[test]
    fn test_gradient_clipping() {
        let mut engine = BackpropEngine::new(LossFunction::MSE);
        engine.store_gradients("test".to_string(), vec![10.0, -10.0, 5.0]);

        engine.clip_gradients_by_value(3.0).unwrap();

        let grads = engine.get_gradients("test").unwrap();
        assert_eq!(grads, &vec![3.0, -3.0, 3.0]);
    }
}
