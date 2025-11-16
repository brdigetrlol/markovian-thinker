//! CUDA kernel interfaces and launchers
//!
//! This module provides Rust interfaces to CUDA kernels for parallel task execution.
//! Kernels are compiled from CUDA C++ (.cu files) to PTX and loaded at runtime.

use anyhow::{Context, Result};
use cudarc::driver::{CudaDevice, CudaFunction, CudaSlice, CudaStream, LaunchConfig, LaunchAsync};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

/// Kernel registry for managing compiled CUDA kernels
pub struct KernelRegistry {
    device: Arc<CudaDevice>,
    /// Loaded kernel functions
    functions: HashMap<String, CudaFunction>,
}

impl KernelRegistry {
    /// Load kernels from PTX file
    pub fn load(device: &Arc<CudaDevice>) -> Result<Self> {
        info!("Loading CUDA kernels from PTX");

        // Get PTX path from build environment
        let ptx_path = option_env!("PTX_PATH")
            .context("PTX_PATH not set. Make sure CUDA kernels were compiled during build.")?;

        debug!("Loading PTX from: {}", ptx_path);

        // Load PTX module
        let ptx_bytes = std::fs::read(ptx_path)
            .context(format!("Failed to read PTX file at {}", ptx_path))?;

        let ptx_str = std::str::from_utf8(&ptx_bytes)
            .context("PTX file is not valid UTF-8")?;

        // Load the PTX module into the device
        device.load_ptx(ptx_str.into(), "parallel_kernels", &[
            "batch_token_process",
            "batch_multi_head_attention",
            "ssm_selective_scan",
            "data_transform",
            "data_filter",
            "data_aggregate",
            "agent_simulation_step",
            // Host wrapper functions
            "launch_batch_token_process",
            "launch_batch_attention",
            "launch_ssm_scan",
            "launch_data_transform",
            "launch_agent_simulation",
        ])?;

        // Extract kernel functions
        let mut functions = HashMap::new();

        let kernel_names = vec![
            "batch_token_process",
            "batch_multi_head_attention",
            "ssm_selective_scan",
            "data_transform",
            "data_filter",
            "data_aggregate",
            "agent_simulation_step",
        ];

        for name in kernel_names {
            let func = device.get_func("parallel_kernels", name)
                .context(format!("Failed to get kernel function: {}", name))?;
            functions.insert(name.to_string(), func);
            debug!("Loaded kernel: {}", name);
        }

        info!("Successfully loaded {} CUDA kernels", functions.len());

        Ok(Self {
            device: device.clone(),
            functions,
        })
    }

    /// Get a kernel function by name
    pub fn get_function(&self, name: &str) -> Result<&CudaFunction> {
        self.functions.get(name)
            .ok_or_else(|| anyhow::anyhow!("Kernel function '{}' not found", name))
    }

    // Note: Direct kernel launching via func.launch() with tuples is now done
    // in each kernel launcher struct to properly type the parameters
}

/// Kernel launcher for batch token processing
pub struct BatchTokenProcessKernel {
    registry: Arc<KernelRegistry>,
}

impl BatchTokenProcessKernel {
    pub fn new(registry: Arc<KernelRegistry>) -> Self {
        Self { registry }
    }

    /// Launch batch token processing kernel
    ///
    /// # Arguments
    /// * `input` - Input token embeddings [batch_size, seq_len, embed_dim]
    /// * `output` - Output token embeddings [batch_size, seq_len, embed_dim]
    /// * `gamma` - Layer norm scale [embed_dim]
    /// * `beta` - Layer norm bias [embed_dim]
    /// * `batch_size` - Number of tasks in batch
    /// * `seq_len` - Sequence length
    /// * `embed_dim` - Embedding dimension
    /// * `stream` - CUDA stream for async execution
    pub fn launch(
        &self,
        input: &CudaSlice<f32>,
        output: &CudaSlice<f32>,
        gamma: &CudaSlice<f32>,
        beta: &CudaSlice<f32>,
        batch_size: usize,
        seq_len: usize,
        embed_dim: usize,
        stream: &CudaStream,
    ) -> Result<()> {
        // Calculate grid and block dimensions
        let grid_dim = (batch_size as u32, seq_len as u32, 1);
        let block_dim = (embed_dim as u32, 1, 1);

        debug!(
            "Launching batch_token_process: batch={}, seq_len={}, embed_dim={}",
            batch_size, seq_len, embed_dim
        );

        // Get kernel function and launch with params as tuple
        let func = self.registry.get_function("batch_token_process")?;

        let config = LaunchConfig {
            grid_dim,
            block_dim,
            shared_mem_bytes: 0,
        };

        unsafe {
            func.clone().launch(config, (input, output, gamma, beta, batch_size, seq_len, embed_dim))
                .context("Failed to launch batch_token_process kernel")?;
        }

        Ok(())
    }
}

/// Kernel launcher for parallel multi-head attention
pub struct BatchAttentionKernel {
    registry: Arc<KernelRegistry>,
}

impl BatchAttentionKernel {
    pub fn new(registry: Arc<KernelRegistry>) -> Self {
        Self { registry }
    }

    /// Launch batch attention kernel
    ///
    /// # Arguments
    /// * `queries` - Query tensors [batch, heads, seq_len, head_dim]
    /// * `keys` - Key tensors [batch, heads, seq_len, head_dim]
    /// * `values` - Value tensors [batch, heads, seq_len, head_dim]
    /// * `output` - Output tensor [batch, heads, seq_len, head_dim]
    /// * `attention_scores` - Workspace for attention scores
    pub fn launch(
        &self,
        queries: &CudaSlice<f32>,
        keys: &CudaSlice<f32>,
        values: &CudaSlice<f32>,
        output: &CudaSlice<f32>,
        attention_scores: &CudaSlice<f32>,
        batch_size: usize,
        num_heads: usize,
        seq_len: usize,
        head_dim: usize,
        stream: &CudaStream,
    ) -> Result<()> {
        let grid_dim = (batch_size as u32, num_heads as u32, 1);
        let block_dim = (seq_len as u32, 1, 1);

        debug!(
            "Launching batch_multi_head_attention: batch={}, heads={}, seq_len={}, head_dim={}",
            batch_size, num_heads, seq_len, head_dim
        );

        let func = self.registry.get_function("batch_multi_head_attention")?;

        let config = LaunchConfig {
            grid_dim,
            block_dim,
            shared_mem_bytes: 0,
        };

        unsafe {
            func.clone().launch(config, (queries, keys, values, output, attention_scores, batch_size, num_heads, seq_len, head_dim))
                .context("Failed to launch batch_multi_head_attention kernel")?;
        }

        Ok(())
    }
}

/// Kernel launcher for SSM (State Space Model) selective scan
pub struct SSMSelectiveScanKernel {
    registry: Arc<KernelRegistry>,
}

impl SSMSelectiveScanKernel {
    pub fn new(registry: Arc<KernelRegistry>) -> Self {
        Self { registry }
    }

    /// Launch SSM selective scan kernel (Mamba-style)
    pub fn launch(
        &self,
        input: &CudaSlice<f32>,
        delta: &CudaSlice<f32>,
        a: &CudaSlice<f32>,
        b: &CudaSlice<f32>,
        c: &CudaSlice<f32>,
        output: &CudaSlice<f32>,
        batch_size: usize,
        seq_len: usize,
        d_model: usize,
        d_state: usize,
        stream: &CudaStream,
    ) -> Result<()> {
        let grid_dim = (batch_size as u32, 1, 1);
        let block_dim = (d_model as u32, 1, 1);

        debug!(
            "Launching ssm_selective_scan: batch={}, seq_len={}, d_model={}, d_state={}",
            batch_size, seq_len, d_model, d_state
        );

        let func = self.registry.get_function("ssm_selective_scan")?;

        let config = LaunchConfig {
            grid_dim,
            block_dim,
            shared_mem_bytes: 0,
        };

        unsafe {
            func.clone().launch(config, (input, delta, a, b, c, output, batch_size, seq_len, d_model, d_state))
                .context("Failed to launch ssm_selective_scan kernel")?;
        }

        Ok(())
    }
}

/// Kernel launcher for data transformation
pub struct DataTransformKernel {
    registry: Arc<KernelRegistry>,
}

impl DataTransformKernel {
    pub fn new(registry: Arc<KernelRegistry>) -> Self {
        Self { registry }
    }

    /// Launch data transform kernel
    pub fn launch(
        &self,
        input: &CudaSlice<f32>,
        output: &CudaSlice<f32>,
        factor: f32,
        batch_size: usize,
        array_size: usize,
        stream: &CudaStream,
    ) -> Result<()> {
        let grid_dim = (batch_size as u32, (array_size as u32 + 255) / 256, 1);
        let block_dim = (256, 1, 1);

        debug!(
            "Launching data_transform: batch={}, array_size={}, factor={}",
            batch_size, array_size, factor
        );

        let func = self.registry.get_function("data_transform")?;

        let config = LaunchConfig {
            grid_dim,
            block_dim,
            shared_mem_bytes: 0,
        };

        unsafe {
            func.clone().launch(config, (input, output, factor, batch_size, array_size))
                .context("Failed to launch data_transform kernel")?;
        }

        Ok(())
    }
}

/// Kernel launcher for agent simulation
pub struct AgentSimulationKernel {
    registry: Arc<KernelRegistry>,
}

impl AgentSimulationKernel {
    pub fn new(registry: Arc<KernelRegistry>) -> Self {
        Self { registry }
    }

    /// Launch agent simulation step kernel
    pub fn launch(
        &self,
        positions: &CudaSlice<f32>,
        velocities: &CudaSlice<f32>,
        states: &CudaSlice<i32>,
        env_params: &CudaSlice<f32>,
        num_agents: usize,
        dt: f32,
        stream: &CudaStream,
    ) -> Result<()> {
        let block_size = 256;
        let grid_size = (num_agents as u32 + block_size - 1) / block_size;

        debug!(
            "Launching agent_simulation_step: num_agents={}, dt={}",
            num_agents, dt
        );

        let func = self.registry.get_function("agent_simulation_step")?;

        let config = LaunchConfig {
            grid_dim: (grid_size, 1, 1),
            block_dim: (block_size, 1, 1),
            shared_mem_bytes: 0,
        };

        unsafe {
            func.clone().launch(config, (positions, velocities, states, env_params, num_agents, dt))
                .context("Failed to launch agent_simulation_step kernel")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Only run when CUDA is available
    fn test_kernel_registry_loading() {
        let device = CudaDevice::new(0).unwrap();
        let device = Arc::new(device);
        let registry = KernelRegistry::load(&device).unwrap();

        assert!(registry.functions.len() > 0);
        assert!(registry.get_function("batch_token_process").is_ok());
    }
}
