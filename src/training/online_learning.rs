//! Online learning system for continuous training during inference

use anyhow::Result;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

use super::backprop::{BackpropEngine, LossFunction};
use super::optimizer::{Optimizer, AdamOptimizer, AdamConfig};
use crate::inference::InferenceModel;

#[cfg(feature = "gpu")]
use crate::gpu::CudaContext;

/// Configuration for online learning
#[derive(Debug, Clone)]
pub struct LearningConfig {
    /// Maximum number of examples to keep in buffer
    pub buffer_size: usize,

    /// Number of examples to accumulate before updating
    pub update_frequency: usize,

    /// Whether to use GPU for training
    pub use_gpu: bool,

    /// Learning rate
    pub learning_rate: f32,

    /// Loss function
    pub loss_fn: LossFunction,

    /// Enable/disable learning
    pub enabled: bool,

    /// Save checkpoint every N updates
    pub checkpoint_frequency: Option<usize>,
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1000,
            update_frequency: 10,
            use_gpu: true,
            learning_rate: 1e-4,
            loss_fn: LossFunction::MSE,
            enabled: false,
            checkpoint_frequency: Some(100),
        }
    }
}

/// Training example
#[derive(Debug, Clone)]
pub struct TrainingExample {
    /// Input text
    pub input: String,

    /// Target output text (for supervised learning)
    pub target: Option<String>,

    /// Target embedding (for embedding-level training)
    pub target_embedding: Option<Vec<f32>>,

    /// Weight for this example
    pub weight: f32,
}

impl TrainingExample {
    pub fn new(input: String, target: Option<String>) -> Self {
        Self {
            input,
            target,
            target_embedding: None,
            weight: 1.0,
        }
    }

    pub fn with_embedding(input: String, target_embedding: Vec<f32>) -> Self {
        Self {
            input,
            target: None,
            target_embedding: Some(target_embedding),
            weight: 1.0,
        }
    }

    pub fn with_weight(mut self, weight: f32) -> Self {
        self.weight = weight;
        self
    }
}

/// Online learner that trains during inference
pub struct OnlineLearner {
    config: LearningConfig,
    buffer: VecDeque<TrainingExample>,

    optimizer: Box<dyn Optimizer>,
    backprop: BackpropEngine,

    model: Arc<RwLock<InferenceModel>>,

    // Statistics
    total_examples: usize,
    total_updates: usize,
    recent_losses: VecDeque<f32>,

    #[cfg(feature = "gpu")]
    gpu_context: Option<Arc<CudaContext>>,
}

impl OnlineLearner {
    /// Create a new online learner
    pub fn new(
        config: LearningConfig,
        model: Arc<RwLock<InferenceModel>>,
    ) -> Self {
        let adam_config = AdamConfig {
            base: super::optimizer::OptimizerConfig {
                learning_rate: config.learning_rate,
                weight_decay: 0.01,
                grad_clip: Some(1.0),
            },
            beta1: 0.9,
            beta2: 0.999,
            epsilon: 1e-8,
        };

        let optimizer = Box::new(AdamOptimizer::new(adam_config));
        let backprop = BackpropEngine::new(config.loss_fn);

        Self {
            config,
            buffer: VecDeque::new(),
            optimizer,
            backprop,
            model,
            total_examples: 0,
            total_updates: 0,
            recent_losses: VecDeque::new(),
            #[cfg(feature = "gpu")]
            gpu_context: None,
        }
    }

    #[cfg(feature = "gpu")]
    /// Create online learner with GPU acceleration
    pub fn new_with_gpu(
        config: LearningConfig,
        model: Arc<RwLock<InferenceModel>>,
        gpu_context: Arc<CudaContext>,
    ) -> Self {
        let adam_config = AdamConfig {
            base: super::optimizer::OptimizerConfig {
                learning_rate: config.learning_rate,
                weight_decay: 0.01,
                grad_clip: Some(1.0),
            },
            beta1: 0.9,
            beta2: 0.999,
            epsilon: 1e-8,
        };

        let optimizer = Box::new(AdamOptimizer::new_with_gpu(adam_config, gpu_context.clone()));
        let backprop = BackpropEngine::new_with_gpu(config.loss_fn, gpu_context.clone());

        Self {
            config,
            buffer: VecDeque::new(),
            optimizer,
            backprop,
            model,
            total_examples: 0,
            total_updates: 0,
            recent_losses: VecDeque::new(),
            gpu_context: Some(gpu_context),
        }
    }

    /// Add a training example
    pub fn add_example(&mut self, example: TrainingExample) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        // Add to buffer
        self.buffer.push_back(example);
        self.total_examples += 1;

        // Remove oldest if buffer is full
        if self.buffer.len() > self.config.buffer_size {
            self.buffer.pop_front();
        }

        // Check if we should update
        if self.total_examples % self.config.update_frequency == 0 {
            self.update_weights()?;
        }

        Ok(())
    }

    /// Perform a weight update using buffered examples
    fn update_weights(&mut self) -> Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        // Sample examples for this update
        let batch_size = self.config.update_frequency.min(self.buffer.len());
        let examples: Vec<_> = self.buffer.iter().take(batch_size).cloned().collect();

        // Compute loss and gradients
        let mut total_loss = 0.0;
        let mut accumulated_grads: Option<Vec<f32>> = None;

        for example in &examples {
            // Forward pass
            let (predictions, target_values) = self.compute_prediction_and_target(example)?;

            // Compute loss
            let loss = self.backprop.compute_loss(&predictions, &target_values)?;
            total_loss += loss * example.weight;

            // Compute gradients
            let grads = self.backprop.compute_gradients(&predictions, &target_values)?;

            // Accumulate gradients
            if let Some(ref mut acc) = accumulated_grads {
                for (acc_g, g) in acc.iter_mut().zip(&grads) {
                    *acc_g += g * example.weight;
                }
            } else {
                accumulated_grads = Some(grads.iter().map(|g| g * example.weight).collect());
            }
        }

        // Average loss
        let avg_loss = total_loss / batch_size as f32;
        self.recent_losses.push_back(avg_loss);
        if self.recent_losses.len() > 100 {
            self.recent_losses.pop_front();
        }

        // Update weights if we have gradients
        if let Some(grads) = accumulated_grads {
            // Average gradients
            let avg_grads: Vec<f32> = grads.iter().map(|g| g / batch_size as f32).collect();

            // Update embedding weights
            let mut model = self.model.write().unwrap();
            if let Some(embedding) = Arc::get_mut(model.embedding_mut()) {
                let weights = embedding.weights_mut();
                self.optimizer.step(weights, &avg_grads)?;
            }
        }

        self.total_updates += 1;

        // Checkpoint if needed
        if let Some(freq) = self.config.checkpoint_frequency {
            if self.total_updates % freq == 0 {
                tracing::info!(
                    "Online learning checkpoint: {} examples, {} updates, avg loss: {:.4}",
                    self.total_examples,
                    self.total_updates,
                    avg_loss
                );
            }
        }

        Ok(())
    }

    /// Compute prediction and target for an example
    fn compute_prediction_and_target(&self, example: &TrainingExample) -> Result<(Vec<f32>, Vec<f32>)> {
        let model = self.model.read().unwrap();

        // Get tokenizer and embedding
        let tokenizer = model.tokenizer();
        let embedding = model.embedding();

        // Tokenize input
        let tokens = tokenizer.encode(&example.input);

        // Get embeddings
        let predictions = embedding.embed_batch(&[tokens.clone()])?;

        // Get target
        let target_values = if let Some(ref target_emb) = example.target_embedding {
            target_emb.clone()
        } else if let Some(ref target_text) = example.target {
            // Tokenize and embed target
            let target_tokens = tokenizer.encode(target_text);
            embedding.embed_batch(&[target_tokens])?
        } else {
            // Self-supervised: use input as target
            predictions.clone()
        };

        Ok((predictions, target_values))
    }

    /// Enable learning
    pub fn enable(&mut self) {
        self.config.enabled = true;
        tracing::info!("Online learning enabled");
    }

    /// Disable learning
    pub fn disable(&mut self) {
        self.config.enabled = false;
        tracing::info!("Online learning disabled");
    }

    /// Check if learning is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get learning statistics
    pub fn get_stats(&self) -> LearningStats {
        let avg_loss = if !self.recent_losses.is_empty() {
            self.recent_losses.iter().sum::<f32>() / self.recent_losses.len() as f32
        } else {
            0.0
        };

        LearningStats {
            total_examples: self.total_examples,
            total_updates: self.total_updates,
            buffer_size: self.buffer.len(),
            average_loss: avg_loss,
            learning_rate: self.optimizer.get_lr(),
            enabled: self.config.enabled,
        }
    }

    /// Clear the buffer
    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
    }

    /// Force an immediate update
    pub fn force_update(&mut self) -> Result<()> {
        self.update_weights()
    }

    /// Set learning rate
    pub fn set_learning_rate(&mut self, lr: f32) {
        self.optimizer.set_lr(lr);
        self.config.learning_rate = lr;
    }
}

/// Learning statistics
#[derive(Debug, Clone)]
pub struct LearningStats {
    pub total_examples: usize,
    pub total_updates: usize,
    pub buffer_size: usize,
    pub average_loss: f32,
    pub learning_rate: f32,
    pub enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_learning_config() {
        let config = LearningConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.buffer_size, 1000);
    }

    #[test]
    fn test_training_example() {
        let example = TrainingExample::new(
            "input".to_string(),
            Some("target".to_string()),
        ).with_weight(2.0);

        assert_eq!(example.weight, 2.0);
        assert_eq!(example.input, "input");
    }
}
