//! Parallel task execution framework
//!
//! Provides a GPU-accelerated parallel task execution engine that can process
//! multiple tasks simultaneously using CUDA.

pub mod task;
pub mod executor;
pub mod batch;
pub mod gpu_executor;

pub use task::{Task, TaskType, TaskResult, TaskEnvelope};
pub use executor::{ParallelExecutor, ExecutorConfig};
pub use batch::{BatchQueue, BatchConfig};
pub use gpu_executor::GpuExecutionPipeline;
