//! GPU acceleration module using CUDA
//!
//! Provides GPU-accelerated parallel task execution using NVIDIA CUDA.
//! Only available with the `gpu` feature flag.

#[cfg(feature = "gpu")]
pub mod cuda_context;
#[cfg(feature = "gpu")]
pub mod memory;
#[cfg(feature = "gpu")]
pub mod kernels;

#[cfg(feature = "gpu")]
pub use cuda_context::CudaContext;
#[cfg(feature = "gpu")]
pub use memory::{MemoryPool, GpuBuffer};

/// Placeholder for when GPU feature is disabled
#[cfg(not(feature = "gpu"))]
pub struct CudaContext;

#[cfg(not(feature = "gpu"))]
impl CudaContext {
    pub fn new(_device_id: usize, _num_streams: usize) -> anyhow::Result<Self> {
        anyhow::bail!("GPU support not compiled. Enable the 'gpu' feature.")
    }
}
