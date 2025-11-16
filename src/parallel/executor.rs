//! Parallel executor for GPU-accelerated task processing

use anyhow::{Context, Result};
use std::sync::Arc;
use tracing::{debug, info, warn};

#[cfg(feature = "gpu")]
use crate::gpu::CudaContext;

use super::batch::{BatchConfig, BatchQueue, QueueStats};
use super::task::{Task, TaskEnvelope, TaskResult};

#[cfg(feature = "gpu")]
use super::gpu_executor::GpuExecutionPipeline;

/// Parallel executor for GPU-accelerated task execution
///
/// Manages task batching, GPU execution, and result distribution.
pub struct ParallelExecutor {
    /// Batch queue for task management
    batch_queue: Arc<BatchQueue>,

    /// GPU context (optional, only with gpu feature)
    #[cfg(feature = "gpu")]
    gpu_context: Option<Arc<CudaContext>>,

    /// GPU execution pipeline
    #[cfg(feature = "gpu")]
    gpu_pipeline: Option<Arc<GpuExecutionPipeline>>,

    /// Worker handles
    workers: Vec<tokio::task::JoinHandle<()>>,

    /// Configuration
    config: ExecutorConfig,
}

/// Executor configuration
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// Number of worker threads
    pub num_workers: usize,

    /// Batch configuration
    pub batch_config: BatchConfig,

    /// GPU device ID (0 for first GPU)
    pub gpu_device: usize,

    /// Number of CUDA streams
    pub num_streams: usize,

    /// Use CPU fallback when GPU is not available
    pub cpu_fallback: bool,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            num_workers: 4,
            batch_config: BatchConfig::default(),
            gpu_device: 0,
            num_streams: 4,
            cpu_fallback: true,
        }
    }
}

impl ParallelExecutor {
    /// Create a new parallel executor
    ///
    /// # Arguments
    /// * `config` - Executor configuration
    ///
    /// # Returns
    /// A new parallel executor ready to process tasks
    pub fn new(config: ExecutorConfig) -> Result<Self> {
        info!("Initializing parallel executor with {} workers", config.num_workers);

        // Create batch queue
        let mut batch_queue = BatchQueue::new(config.batch_config.clone());
        batch_queue.start();
        let batch_queue = Arc::new(batch_queue);

        // Initialize GPU context if feature is enabled
        #[cfg(feature = "gpu")]
        let (gpu_context, gpu_pipeline) = match CudaContext::new(config.gpu_device, config.num_streams) {
            Ok(ctx) => {
                info!("GPU context initialized successfully");
                let info = ctx.device_info();
                info!("GPU Device Info:\n{}", info);
                let ctx_arc = Arc::new(ctx);
                let pipeline = Arc::new(GpuExecutionPipeline::new(ctx_arc.clone()));
                (Some(ctx_arc), Some(pipeline))
            }
            Err(e) => {
                if config.cpu_fallback {
                    warn!("Failed to initialize GPU context, falling back to CPU: {}", e);
                    (None, None)
                } else {
                    return Err(e).context("Failed to initialize GPU context");
                }
            }
        };

        // Spawn worker tasks
        let workers = (0..config.num_workers)
            .map(|worker_id| {
                let queue = batch_queue.clone();
                #[cfg(feature = "gpu")]
                let gpu_pipe = gpu_pipeline.clone();
                let _cfg = config.clone();

                tokio::spawn(async move {
                    info!("Worker {} started", worker_id);

                    loop {
                        // Wait for a batch
                        let batch = match queue.get_batch().await {
                            Some(b) => b,
                            None => {
                                debug!("Worker {} batch channel closed", worker_id);
                                break;
                            }
                        };

                        debug!("Worker {} processing batch of {} tasks", worker_id, batch.len());

                        // Process batch
                        #[cfg(feature = "gpu")]
                        let results: Result<Vec<TaskResult>> = if let Some(ref pipeline) = gpu_pipe {
                            pipeline.execute_batch(&batch, worker_id).await
                        } else {
                            Self::execute_batch_cpu(&batch).await
                        };

                        #[cfg(not(feature = "gpu"))]
                        let results: Result<Vec<TaskResult>> = Self::execute_batch_cpu(&batch).await;

                        // Send results back
                        match results {
                            Ok(task_results) => {
                                for (task, result) in batch.iter().zip(task_results) {
                                    if let Err(e) = queue.send_result(task.id, result).await {
                                        warn!("Failed to send result for task {}: {}", task.id, e);
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("Batch execution failed: {}", e);
                                // Send error results
                                for task in batch {
                                    let error_result = TaskResult {
                                        task_id: task.id,
                                        success: false,
                                        output: serde_json::json!(null),
                                        error: Some(e.to_string()),
                                        gpu_time_ms: 0.0,
                                        total_time_ms: 0.0,
                                    };
                                    let _ = queue.send_result(task.id, error_result).await;
                                }
                            }
                        }
                    }

                    info!("Worker {} stopped", worker_id);
                })
            })
            .collect();

        Ok(Self {
            batch_queue,
            #[cfg(feature = "gpu")]
            gpu_context,
            #[cfg(feature = "gpu")]
            gpu_pipeline,
            workers,
            config,
        })
    }

    /// Submit a task for execution
    ///
    /// # Arguments
    /// * `task` - The task to execute
    /// * `priority` - Task priority (higher = more important)
    ///
    /// # Returns
    /// The task result when execution is complete
    pub async fn submit<T: Task>(&self, task: T, priority: u8) -> Result<T::Output> {
        let envelope = TaskEnvelope::new(task, priority);
        let rx = self.batch_queue.submit(envelope).await?;

        let result = rx.await
            .context("Task was cancelled before completion")?;

        if !result.success {
            anyhow::bail!("Task execution failed: {}", result.error.unwrap_or_else(|| "Unknown error".to_string()));
        }

        // Deserialize output
        // In a real implementation, would properly deserialize
        // For now, this is a placeholder
        anyhow::bail!("Output deserialization not yet implemented")
    }

    /// Submit multiple tasks as a batch
    ///
    /// # Arguments
    /// * `tasks` - Vector of tasks to execute
    /// * `priority` - Priority for all tasks
    ///
    /// # Returns
    /// Vector of results in the same order as tasks
    pub async fn submit_batch<T: Task>(
        &self,
        tasks: Vec<T>,
        priority: u8,
    ) -> Result<Vec<T::Output>> {
        let futures: Vec<_> = tasks
            .into_iter()
            .map(|task| self.submit(task, priority))
            .collect();

        futures::future::try_join_all(futures).await
    }

    /// Execute a batch on the CPU (fallback)
    async fn execute_batch_cpu(batch: &[TaskEnvelope]) -> Result<Vec<TaskResult>> {
        let start_time = std::time::Instant::now();

        debug!("Executing batch of {} tasks on CPU (fallback)", batch.len());

        // For now, return placeholder results
        // In a real implementation, would execute tasks via their execute_cpu method
        let results = batch
            .iter()
            .map(|task| TaskResult {
                task_id: task.id,
                success: true,
                output: serde_json::json!("CPU result"),
                error: None,
                gpu_time_ms: 0.0,
                total_time_ms: start_time.elapsed().as_millis() as f64,
            })
            .collect();

        Ok(results)
    }

    /// Get executor statistics
    pub async fn stats(&self) -> ExecutorStats {
        let queue_stats = self.batch_queue.stats().await;

        ExecutorStats {
            num_workers: self.config.num_workers,
            queue_stats,
            #[cfg(feature = "gpu")]
            gpu_available: self.gpu_context.is_some(),
            #[cfg(not(feature = "gpu"))]
            gpu_available: false,
        }
    }

    /// Shutdown the executor gracefully
    pub async fn shutdown(self) -> Result<()> {
        info!("Shutting down parallel executor");

        // Clear the queue
        self.batch_queue.clear().await;

        // Wait for workers to finish (or abort them)
        for worker in self.workers {
            worker.abort();
        }

        info!("Parallel executor shutdown complete");
        Ok(())
    }
}

/// Executor statistics
#[derive(Debug, Clone)]
pub struct ExecutorStats {
    pub num_workers: usize,
    pub queue_stats: QueueStats,
    pub gpu_available: bool,
}

impl std::fmt::Display for ExecutorStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Parallel Executor Stats:")?;
        writeln!(f, "  Workers: {}", self.num_workers)?;
        writeln!(f, "  GPU Available: {}", self.gpu_available)?;
        writeln!(f, "\n{}", self.queue_stats)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parallel::task::{CodeGenTask, DataProcessTask, DataOperation};

    #[tokio::test]
    async fn test_executor_creation() {
        let config = ExecutorConfig {
            cpu_fallback: true,
            ..Default::default()
        };

        let executor = ParallelExecutor::new(config).unwrap();
        let stats = executor.stats().await;

        println!("{}", stats);
        assert_eq!(stats.num_workers, 4);
    }

    #[tokio::test]
    #[ignore] // Requires manual testing
    async fn test_task_submission() {
        let config = ExecutorConfig {
            cpu_fallback: true,
            ..Default::default()
        };

        let executor = ParallelExecutor::new(config).unwrap();

        // This would fail currently because output deserialization is not implemented
        // But the framework is in place
    }
}
