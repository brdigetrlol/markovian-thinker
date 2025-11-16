//! GPU-accelerated optimizers for training

use anyhow::Result;
use std::collections::HashMap;

#[cfg(feature = "gpu")]
use crate::gpu::CudaContext;

/// Optimizer configuration
#[derive(Debug, Clone)]
pub struct OptimizerConfig {
    /// Learning rate
    pub learning_rate: f32,
    /// Weight decay (L2 regularization)
    pub weight_decay: f32,
    /// Gradient clipping threshold
    pub grad_clip: Option<f32>,
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self {
            learning_rate: 1e-3,
            weight_decay: 0.01,
            grad_clip: Some(1.0),
        }
    }
}

/// Adam optimizer configuration
#[derive(Debug, Clone)]
pub struct AdamConfig {
    pub base: OptimizerConfig,
    /// Beta1 for momentum
    pub beta1: f32,
    /// Beta2 for RMSProp
    pub beta2: f32,
    /// Epsilon for numerical stability
    pub epsilon: f32,
}

impl Default for AdamConfig {
    fn default() -> Self {
        Self {
            base: OptimizerConfig::default(),
            beta1: 0.9,
            beta2: 0.999,
            epsilon: 1e-8,
        }
    }
}

/// Optimizer trait for different optimization algorithms
pub trait Optimizer: Send + Sync {
    /// Update parameters using gradients
    fn step(&mut self, params: &mut [f32], grads: &[f32]) -> Result<()>;

    /// Zero out gradients
    fn zero_grad(&mut self);

    /// Get current learning rate
    fn get_lr(&self) -> f32;

    /// Set learning rate
    fn set_lr(&mut self, lr: f32);

    /// Get optimization step count
    fn step_count(&self) -> usize;
}

/// Adam optimizer with GPU acceleration
pub struct AdamOptimizer {
    config: AdamConfig,
    step_count: usize,

    // First moment (momentum)
    momentum: HashMap<String, Vec<f32>>,

    // Second moment (RMSProp)
    velocity: HashMap<String, Vec<f32>>,

    #[cfg(feature = "gpu")]
    gpu_context: Option<Arc<CudaContext>>,
}

impl AdamOptimizer {
    /// Create a new Adam optimizer
    pub fn new(config: AdamConfig) -> Self {
        Self {
            config,
            step_count: 0,
            momentum: HashMap::new(),
            velocity: HashMap::new(),
            #[cfg(feature = "gpu")]
            gpu_context: None,
        }
    }

    #[cfg(feature = "gpu")]
    /// Create Adam optimizer with GPU acceleration
    pub fn new_with_gpu(config: AdamConfig, gpu_context: Arc<CudaContext>) -> Self {
        Self {
            config,
            step_count: 0,
            momentum: HashMap::new(),
            velocity: HashMap::new(),
            gpu_context: Some(gpu_context),
        }
    }

    /// Update a named parameter
    pub fn update_param(&mut self, name: &str, params: &mut [f32], grads: &[f32]) -> Result<()> {
        #[cfg(feature = "gpu")]
        if let Some(ref gpu_ctx) = self.gpu_context {
            return self.update_param_gpu(name, params, grads, gpu_ctx);
        }

        // CPU fallback
        self.update_param_cpu(name, params, grads)
    }

    /// CPU implementation of Adam update
    fn update_param_cpu(&mut self, name: &str, params: &mut [f32], grads: &[f32]) -> Result<()> {
        let n = params.len();

        // Initialize momentum and velocity if needed
        let m = self.momentum.entry(name.to_string())
            .or_insert_with(|| vec![0.0; n]);
        let v = self.velocity.entry(name.to_string())
            .or_insert_with(|| vec![0.0; n]);

        self.step_count += 1;
        let t = self.step_count as f32;

        // Bias correction
        let lr = self.config.base.learning_rate *
                 (1.0 - self.config.beta2.powf(t)).sqrt() /
                 (1.0 - self.config.beta1.powf(t));

        for i in 0..n {
            let mut g = grads[i];

            // Gradient clipping
            if let Some(clip) = self.config.base.grad_clip {
                g = g.clamp(-clip, clip);
            }

            // Weight decay
            if self.config.base.weight_decay > 0.0 {
                g += self.config.base.weight_decay * params[i];
            }

            // Update biased first moment estimate
            m[i] = self.config.beta1 * m[i] + (1.0 - self.config.beta1) * g;

            // Update biased second raw moment estimate
            v[i] = self.config.beta2 * v[i] + (1.0 - self.config.beta2) * g * g;

            // Update parameters
            params[i] -= lr * m[i] / (v[i].sqrt() + self.config.epsilon);
        }

        Ok(())
    }

    #[cfg(feature = "gpu")]
    /// GPU implementation of Adam update
    fn update_param_gpu(&mut self, name: &str, params: &mut [f32], grads: &[f32],
                        gpu_ctx: &Arc<CudaContext>) -> Result<()> {
        use cudarc::driver::LaunchAsync;
        use cudarc::driver::DeviceSlice;

        let n = params.len();

        // Initialize momentum and velocity if needed
        let m = self.momentum.entry(name.to_string())
            .or_insert_with(|| vec![0.0; n]);
        let v = self.velocity.entry(name.to_string())
            .or_insert_with(|| vec![0.0; n]);

        self.step_count += 1;
        let t = self.step_count as f32;

        // Bias correction
        let lr = self.config.base.learning_rate *
                 (1.0 - self.config.beta2.powf(t)).sqrt() /
                 (1.0 - self.config.beta1.powf(t));

        // Transfer data to GPU
        let device = gpu_ctx.device();
        let mut d_params = device.htod_copy(params.to_vec())?;
        let d_grads = device.htod_copy(grads.to_vec())?;
        let mut d_momentum = device.htod_copy(m.to_vec())?;
        let mut d_velocity = device.htod_copy(v.to_vec())?;

        // Get kernel
        let kernel_name = "adam_optimizer_step";
        let kernel = gpu_ctx.kernel_registry().get_function(kernel_name)
            .ok_or_else(|| anyhow::anyhow!("Adam kernel not found"))?;

        // Launch kernel
        let threads_per_block = 256;
        let num_blocks = (n + threads_per_block - 1) / threads_per_block;

        let clip = self.config.base.grad_clip.unwrap_or(f32::INFINITY);

        let cfg = cudarc::driver::LaunchConfig {
            grid_dim: (num_blocks as u32, 1, 1),
            block_dim: (threads_per_block as u32, 1, 1),
            shared_mem_bytes: 0,
        };

        unsafe {
            kernel.launch(
                cfg,
                (
                    &mut d_params,
                    &d_grads,
                    &mut d_momentum,
                    &mut d_velocity,
                    n,
                    lr,
                    self.config.beta1,
                    self.config.beta2,
                    self.config.epsilon,
                    self.config.base.weight_decay,
                    clip,
                ),
            )?;
        }

        device.synchronize()?;

        // Copy results back
        device.dtoh_sync_copy_into(&d_params, params)?;
        device.dtoh_sync_copy_into(&d_momentum, m)?;
        device.dtoh_sync_copy_into(&d_velocity, v)?;

        Ok(())
    }
}

impl Optimizer for AdamOptimizer {
    fn step(&mut self, params: &mut [f32], grads: &[f32]) -> Result<()> {
        self.update_param("default", params, grads)
    }

    fn zero_grad(&mut self) {
        // Gradients are passed in, so nothing to zero here
    }

    fn get_lr(&self) -> f32 {
        self.config.base.learning_rate
    }

    fn set_lr(&mut self, lr: f32) {
        self.config.base.learning_rate = lr;
    }

    fn step_count(&self) -> usize {
        self.step_count
    }
}

/// Simple SGD optimizer
pub struct SGDOptimizer {
    config: OptimizerConfig,
    step_count: usize,

    #[cfg(feature = "gpu")]
    gpu_context: Option<Arc<CudaContext>>,
}

impl SGDOptimizer {
    /// Create a new SGD optimizer
    pub fn new(config: OptimizerConfig) -> Self {
        Self {
            config,
            step_count: 0,
            #[cfg(feature = "gpu")]
            gpu_context: None,
        }
    }

    #[cfg(feature = "gpu")]
    /// Create SGD optimizer with GPU acceleration
    pub fn new_with_gpu(config: OptimizerConfig, gpu_context: Arc<CudaContext>) -> Self {
        Self {
            config,
            step_count: 0,
            gpu_context: Some(gpu_context),
        }
    }
}

impl Optimizer for SGDOptimizer {
    fn step(&mut self, params: &mut [f32], grads: &[f32]) -> Result<()> {
        #[cfg(feature = "gpu")]
        if let Some(ref gpu_ctx) = self.gpu_context {
            return self.step_gpu(params, grads, gpu_ctx);
        }

        // CPU fallback
        self.step_cpu(params, grads)
    }

    fn zero_grad(&mut self) {
        // Gradients are passed in, so nothing to zero here
    }

    fn get_lr(&self) -> f32 {
        self.config.learning_rate
    }

    fn set_lr(&mut self, lr: f32) {
        self.config.learning_rate = lr;
    }

    fn step_count(&self) -> usize {
        self.step_count
    }
}

impl SGDOptimizer {
    fn step_cpu(&mut self, params: &mut [f32], grads: &[f32]) -> Result<()> {
        self.step_count += 1;

        for i in 0..params.len() {
            let mut g = grads[i];

            // Gradient clipping
            if let Some(clip) = self.config.grad_clip {
                g = g.clamp(-clip, clip);
            }

            // Weight decay
            if self.config.weight_decay > 0.0 {
                g += self.config.weight_decay * params[i];
            }

            // Update
            params[i] -= self.config.learning_rate * g;
        }

        Ok(())
    }

    #[cfg(feature = "gpu")]
    fn step_gpu(&mut self, params: &mut [f32], grads: &[f32], gpu_ctx: &Arc<CudaContext>) -> Result<()> {
        use cudarc::driver::LaunchAsync;
        use cudarc::driver::DeviceSlice;

        self.step_count += 1;
        let n = params.len();

        // Transfer to GPU
        let device = gpu_ctx.device();
        let mut d_params = device.htod_copy(params.to_vec())?;
        let d_grads = device.htod_copy(grads.to_vec())?;

        // Get kernel
        let kernel_name = "sgd_optimizer_step";
        let kernel = gpu_ctx.kernel_registry().get_function(kernel_name)
            .ok_or_else(|| anyhow::anyhow!("SGD kernel not found"))?;

        // Launch kernel
        let threads_per_block = 256;
        let num_blocks = (n + threads_per_block - 1) / threads_per_block;

        let clip = self.config.grad_clip.unwrap_or(f32::INFINITY);

        let cfg = cudarc::driver::LaunchConfig {
            grid_dim: (num_blocks as u32, 1, 1),
            block_dim: (threads_per_block as u32, 1, 1),
            shared_mem_bytes: 0,
        };

        unsafe {
            kernel.launch(
                cfg,
                (
                    &mut d_params,
                    &d_grads,
                    n,
                    self.config.learning_rate,
                    self.config.weight_decay,
                    clip,
                ),
            )?;
        }

        device.synchronize()?;

        // Copy back
        device.dtoh_sync_copy_into(&d_params, params)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sgd_optimizer() {
        let config = OptimizerConfig {
            learning_rate: 0.1,
            weight_decay: 0.0,
            grad_clip: None,
        };

        let mut optimizer = SGDOptimizer::new(config);
        let mut params = vec![1.0, 2.0, 3.0];
        let grads = vec![0.1, 0.2, 0.3];

        optimizer.step(&mut params, &grads).unwrap();

        assert_eq!(params, vec![0.99, 1.98, 2.97]);
    }

    #[test]
    fn test_adam_optimizer() {
        let config = AdamConfig::default();
        let mut optimizer = AdamOptimizer::new(config);

        let mut params = vec![1.0, 2.0, 3.0];
        let grads = vec![0.1, 0.2, 0.3];

        optimizer.step(&mut params, &grads).unwrap();

        // Parameters should have been updated
        assert!(params[0] < 1.0);
        assert!(params[1] < 2.0);
        assert!(params[2] < 3.0);
    }
}
