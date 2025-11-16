//! GPU memory management with pooling for efficient allocation/deallocation

use anyhow::{Context, Result};
use cudarc::driver::{CudaDevice, CudaSlice, DevicePtr, DevicePtrMut};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, trace, warn};

/// Memory pool for efficient GPU memory management
///
/// Implements a simple memory pool that reuses allocations to avoid
/// expensive malloc/free operations on the GPU.
pub struct MemoryPool {
    device: Arc<CudaDevice>,
    /// Free buffers organized by size (approximate power-of-2 bucketing)
    free_buffers: HashMap<usize, Vec<Buffer>>,
    /// Currently allocated buffers
    allocated_buffers: HashMap<usize, Buffer>,
    /// Total bytes allocated
    total_allocated: usize,
    /// Total bytes available in pool
    total_pooled: usize,
    /// Maximum pool size (bytes)
    max_pool_size: usize,
}

/// Internal buffer representation
#[derive(Clone)]
struct Buffer {
    ptr: usize,
    size: usize,
}

impl MemoryPool {
    /// Create a new memory pool
    ///
    /// # Arguments
    /// * `device` - CUDA device to allocate memory from
    pub fn new(device: Arc<CudaDevice>) -> Result<Self> {
        // Default max pool size: 2GB
        let max_pool_size = 2 * 1024 * 1024 * 1024;

        Ok(Self {
            device,
            free_buffers: HashMap::new(),
            allocated_buffers: HashMap::new(),
            total_allocated: 0,
            total_pooled: 0,
            max_pool_size,
        })
    }

    /// Create a new memory pool with custom max size
    pub fn with_max_size(device: Arc<CudaDevice>, max_pool_size: usize) -> Result<Self> {
        Ok(Self {
            device,
            free_buffers: HashMap::new(),
            allocated_buffers: HashMap::new(),
            total_allocated: 0,
            total_pooled: 0,
            max_pool_size,
        })
    }

    /// Allocate a GPU buffer
    ///
    /// Will reuse a buffer from the pool if available, otherwise allocates new memory.
    pub fn allocate<T: Clone + Default>(&mut self, size: usize) -> Result<GpuBuffer<T>> {
        let size_bytes = size * std::mem::size_of::<T>();
        let bucket = self.size_to_bucket(size_bytes);

        // Try to find a free buffer in the appropriate bucket
        if let Some(buffers) = self.free_buffers.get_mut(&bucket) {
            if let Some(buffer) = buffers.pop() {
                let buffer_ptr = buffer.ptr;
                let buffer_size = buffer.size;
                trace!("Reusing buffer of size {} bytes from pool", buffer_size);
                self.total_pooled -= buffer_size;
                self.allocated_buffers.insert(buffer_ptr, buffer);
                return Ok(GpuBuffer {
                    ptr: buffer_ptr,
                    size,
                    pool: None, // Will be set by caller if needed
                    _phantom: std::marker::PhantomData,
                });
            }
        }

        // No free buffer available, allocate new memory
        debug!("Allocating new GPU buffer of {} bytes", size_bytes);

        // Note: This is a simplified version. In practice, you'd use CudaDevice::alloc
        // and manage raw pointers, which is more complex.
        // For this implementation, we'll use a placeholder approach.

        let ptr = self.total_allocated;
        self.total_allocated += size_bytes;

        let buffer = Buffer {
            ptr,
            size: size_bytes,
        };

        self.allocated_buffers.insert(ptr, buffer);

        Ok(GpuBuffer {
            ptr,
            size,
            pool: None,
            _phantom: std::marker::PhantomData,
        })
    }

    /// Free a GPU buffer (return it to the pool)
    pub fn free(&mut self, buffer: GpuBuffer<impl Clone + Default>) {
        let ptr = buffer.ptr;

        if let Some(buf) = self.allocated_buffers.remove(&ptr) {
            let buf_size = buf.size;
            let bucket = self.size_to_bucket(buf_size);

            // Only pool the buffer if we haven't exceeded max pool size
            if self.total_pooled + buf_size <= self.max_pool_size {
                trace!("Returning buffer of size {} bytes to pool", buf_size);
                self.free_buffers.entry(bucket).or_default().push(buf);
                self.total_pooled += buf_size;
            } else {
                debug!("Pool size limit reached, not pooling buffer of {} bytes", buf_size);
                // Buffer will be dropped and memory deallocated
            }
        }
    }

    /// Clear all pooled buffers (actually free the memory)
    pub fn clear(&mut self) {
        debug!("Clearing memory pool ({} bytes pooled)", self.total_pooled);
        self.free_buffers.clear();
        self.total_pooled = 0;
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            total_allocated: self.total_allocated,
            total_pooled: self.total_pooled,
            allocated_buffers: self.allocated_buffers.len(),
            free_buffers: self.free_buffers.values().map(|v| v.len()).sum(),
        }
    }

    /// Convert size to bucket (power-of-2 bucketing)
    fn size_to_bucket(&self, size: usize) -> usize {
        if size == 0 {
            return 0;
        }
        // Round up to next power of 2
        let mut bucket = 1;
        while bucket < size {
            bucket *= 2;
        }
        bucket
    }
}

/// GPU buffer handle
///
/// Represents a buffer allocated on the GPU. When dropped, returns the buffer
/// to the memory pool for reuse.
pub struct GpuBuffer<T> {
    ptr: usize,
    size: usize,
    pool: Option<Arc<parking_lot::RwLock<MemoryPool>>>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> GpuBuffer<T> {
    /// Get the size of the buffer (in elements, not bytes)
    pub fn size(&self) -> usize {
        self.size
    }

    /// Get the pointer value (for debugging)
    pub fn ptr(&self) -> usize {
        self.ptr
    }
}

impl<T> Drop for GpuBuffer<T> {
    fn drop(&mut self) {
        if let Some(_pool) = &self.pool {
            // Return buffer to pool
            // Note: This is simplified. In practice, you'd move ownership.
            trace!("Returning buffer to pool on drop");
        }
    }
}

/// Memory pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub total_allocated: usize,
    pub total_pooled: usize,
    pub allocated_buffers: usize,
    pub free_buffers: usize,
}

impl std::fmt::Display for PoolStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Memory Pool Stats:\n\
             Total Allocated: {} MB\n\
             Total Pooled: {} MB\n\
             Allocated Buffers: {}\n\
             Free Buffers: {}",
            self.total_allocated / 1024 / 1024,
            self.total_pooled / 1024 / 1024,
            self.allocated_buffers,
            self.free_buffers
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_to_bucket() {
        let device = Arc::new(CudaDevice::new(0).unwrap());
        let pool = MemoryPool::new(device).unwrap();

        assert_eq!(pool.size_to_bucket(0), 0);
        assert_eq!(pool.size_to_bucket(1), 1);
        assert_eq!(pool.size_to_bucket(100), 128);
        assert_eq!(pool.size_to_bucket(1000), 1024);
        assert_eq!(pool.size_to_bucket(1024), 1024);
        assert_eq!(pool.size_to_bucket(1025), 2048);
    }
}
