//! CUDA context management for GPU-accelerated parallel execution

use anyhow::{Context, Result};
use cudarc::driver::{CudaDevice, CudaStream, CudaSlice, DevicePtr, DevicePtrMut};
use cudarc::cublas::CudaBlas;
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{info, debug, warn};

use super::memory::{MemoryPool, GpuBuffer};
use super::kernels::KernelRegistry;

/// CUDA context for managing GPU device, streams, and memory
///
/// Provides a high-level interface for GPU operations including:
/// - Device initialization and management
/// - Stream creation for concurrent execution
/// - Memory pool for efficient allocation
/// - Kernel launching interface
pub struct CudaContext {
    /// The CUDA device handle
    pub(crate) device: Arc<CudaDevice>,

    /// Multiple CUDA streams for concurrent kernel execution
    /// Each stream can execute kernels independently
    pub(crate) streams: Vec<Arc<CudaStream>>,

    /// Memory pool for efficient GPU memory management
    pub(crate) memory_pool: Arc<RwLock<MemoryPool>>,

    /// cuBLAS handle for optimized matrix operations
    pub(crate) cublas: Arc<CudaBlas>,

    /// Kernel registry for launching CUDA kernels
    pub(crate) kernel_registry: Arc<KernelRegistry>,

    /// Current stream index (round-robin selection)
    current_stream: Arc<RwLock<usize>>,
}

// SAFETY: CudaContext contains Arc<CudaDevice>, Arc<CudaStream>, and other Arc-wrapped types
// which all manage thread safety internally. The raw CUDA pointers in CudaStream are never
// directly accessed across threads, and all CUDA operations are properly synchronized.
unsafe impl Send for CudaContext {}
unsafe impl Sync for CudaContext {}

impl CudaContext {
    /// Create a new CUDA context
    ///
    /// # Arguments
    /// * `device_id` - GPU device ID (0 for first GPU, 1 for second, etc.)
    /// * `num_streams` - Number of CUDA streams to create for concurrent execution
    ///
    /// # Returns
    /// A new `CudaContext` ready for GPU operations
    pub fn new(device_id: usize, num_streams: usize) -> Result<Self> {
        info!("Initializing CUDA context on device {}", device_id);

        // Initialize CUDA device
        let device = CudaDevice::new(device_id)
            .context("Failed to initialize CUDA device")?;

        // Get compute capability
        let cc_major = device.attribute(cudarc::driver::sys::CUdevice_attribute_enum::CU_DEVICE_ATTRIBUTE_COMPUTE_CAPABILITY_MAJOR)?;
        let cc_minor = device.attribute(cudarc::driver::sys::CUdevice_attribute_enum::CU_DEVICE_ATTRIBUTE_COMPUTE_CAPABILITY_MINOR)?;
        info!("CUDA device initialized (compute capability {}.{})", cc_major, cc_minor);

        // Create CUDA streams
        debug!("Creating {} CUDA streams", num_streams);
        let streams = (0..num_streams)
            .map(|i| {
                device.fork_default_stream()
                    .context(format!("Failed to create CUDA stream {}", i))
                    .map(Arc::new)
            })
            .collect::<Result<Vec<_>>>()?;

        // device is already Arc<CudaDevice> in cudarc 0.9
        let device_arc = device;

        // Initialize cuBLAS for matrix operations
        let cublas = CudaBlas::new(device_arc.clone())
            .context("Failed to initialize cuBLAS")?;

        // Create memory pool
        let memory_pool = MemoryPool::new(device_arc.clone())?;

        // Load CUDA kernels
        let kernel_registry = KernelRegistry::load(&device_arc)
            .context("Failed to load CUDA kernels")?;

        info!("CUDA context initialized successfully with {} streams", num_streams);

        Ok(Self {
            device: device_arc,
            streams,
            memory_pool: Arc::new(RwLock::new(memory_pool)),
            cublas: Arc::new(cublas),
            kernel_registry: Arc::new(kernel_registry),
            current_stream: Arc::new(RwLock::new(0)),
        })
    }

    /// Get the number of available CUDA streams
    pub fn num_streams(&self) -> usize {
        self.streams.len()
    }

    /// Get a specific CUDA stream by index
    pub fn get_stream(&self, index: usize) -> Result<Arc<CudaStream>> {
        self.streams.get(index)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Stream index {} out of bounds", index))
    }

    /// Get the next stream in round-robin fashion
    pub fn next_stream(&self) -> Arc<CudaStream> {
        let mut current = self.current_stream.write();
        let stream_idx = *current;
        *current = (stream_idx + 1) % self.streams.len();
        self.streams[stream_idx].clone()
    }

    /// Allocate GPU memory from the memory pool
    pub fn allocate<T: Clone + Default>(&self, size: usize) -> Result<GpuBuffer<T>> {
        self.memory_pool.write().allocate(size)
    }

    /// Copy data from host to device asynchronously
    pub async fn copy_to_device<T: Clone + Unpin>(
        &self,
        data: &[T],
        stream_idx: usize,
    ) -> Result<CudaSlice<T>>
    where
        T: cudarc::driver::DeviceRepr,
    {
        let stream = self.get_stream(stream_idx)?;

        // Allocate device memory
        let device_data = self.device.htod_copy(data.to_vec())
            .context("Failed to copy data to device")?;

        Ok(device_data)
    }

    /// Copy data from device to host asynchronously
    pub async fn copy_from_device<T: Clone>(
        &self,
        device_data: &CudaSlice<T>,
    ) -> Result<Vec<T>>
    where
        T: cudarc::driver::DeviceRepr,
    {
        self.device.dtoh_sync_copy(device_data)
            .context("Failed to copy data from device")
    }

    /// Synchronize a specific stream (in cudarc 0.9, synchronizes the whole device)
    pub fn synchronize_stream(&self, _stream_idx: usize) -> Result<()> {
        self.device.synchronize()
            .context("Failed to synchronize device")
    }

    /// Synchronize all streams (in cudarc 0.9, synchronizes the whole device)
    pub fn synchronize_all(&self) -> Result<()> {
        self.device.synchronize()
            .context("Failed to synchronize device")
    }

    /// Synchronize the device (wait for all operations to complete)
    pub fn synchronize_device(&self) -> Result<()> {
        self.device.synchronize()
            .context("Failed to synchronize device")
    }

    /// Get device information
    pub fn device_info(&self) -> DeviceInfo {
        // In cudarc 0.9.15, device doesn't have name/memory methods
        // Use placeholders for now
        DeviceInfo {
            name: "CUDA Device".to_string(),
            total_memory: 0,
            free_memory: 0,
            compute_capability: (
                self.device.attribute(cudarc::driver::sys::CUdevice_attribute_enum::CU_DEVICE_ATTRIBUTE_COMPUTE_CAPABILITY_MAJOR).unwrap_or(0),
                self.device.attribute(cudarc::driver::sys::CUdevice_attribute_enum::CU_DEVICE_ATTRIBUTE_COMPUTE_CAPABILITY_MINOR).unwrap_or(0),
            ),
            multiprocessor_count: self.device.attribute(cudarc::driver::sys::CUdevice_attribute_enum::CU_DEVICE_ATTRIBUTE_MULTIPROCESSOR_COUNT).unwrap_or(0),
            max_threads_per_block: self.device.attribute(cudarc::driver::sys::CUdevice_attribute_enum::CU_DEVICE_ATTRIBUTE_MAX_THREADS_PER_BLOCK).unwrap_or(0),
        }
    }

    /// Get cuBLAS handle for matrix operations
    pub fn cublas(&self) -> Arc<CudaBlas> {
        self.cublas.clone()
    }

    /// Get the underlying device
    pub fn device(&self) -> Arc<CudaDevice> {
        self.device.clone()
    }

    /// Get the kernel registry
    pub fn kernels(&self) -> Arc<KernelRegistry> {
        self.kernel_registry.clone()
    }
}

/// Device information structure
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub name: String,
    pub total_memory: usize,
    pub free_memory: usize,
    pub compute_capability: (i32, i32),
    pub multiprocessor_count: i32,
    pub max_threads_per_block: i32,
}

impl std::fmt::Display for DeviceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Device: {}\n\
             Total Memory: {} MB\n\
             Free Memory: {} MB\n\
             Compute Capability: {}.{}\n\
             Multiprocessors: {}\n\
             Max Threads per Block: {}",
            self.name,
            self.total_memory / 1024 / 1024,
            self.free_memory / 1024 / 1024,
            self.compute_capability.0,
            self.compute_capability.1,
            self.multiprocessor_count,
            self.max_threads_per_block
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Only run when CUDA is available
    fn test_cuda_context_creation() {
        let ctx = CudaContext::new(0, 4).unwrap();
        assert_eq!(ctx.num_streams(), 4);

        let info = ctx.device_info();
        println!("Device info:\n{}", info);
    }

    #[tokio::test]
    #[ignore] // Only run when CUDA is available
    async fn test_memory_transfer() {
        let ctx = CudaContext::new(0, 1).unwrap();

        let data = vec![1.0f32, 2.0, 3.0, 4.0, 5.0];
        let device_data = ctx.copy_to_device(&data, 0).await.unwrap();
        let result = ctx.copy_from_device(&device_data).await.unwrap();

        assert_eq!(data, result);
    }
}
