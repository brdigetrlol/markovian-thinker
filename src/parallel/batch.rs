//! Batch queue system for collecting and batching tasks for GPU execution

use anyhow::Result;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::{debug, trace};
use uuid::Uuid;

use super::task::{TaskEnvelope, TaskType, TaskResult};

/// Configuration for batch processing
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Maximum number of tasks per batch
    pub max_batch_size: usize,

    /// Minimum number of tasks to trigger batch processing
    pub min_batch_size: usize,

    /// Maximum time to wait for batch to fill (milliseconds)
    pub max_wait_time_ms: u64,

    /// Whether to group tasks by type in batches
    pub group_by_type: bool,

    /// Priority threshold (only process tasks >= this priority)
    pub min_priority: u8,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 32,
            min_batch_size: 4,
            max_wait_time_ms: 100,
            group_by_type: true,
            min_priority: 0,
        }
    }
}

/// Batch queue for managing task batching
pub struct BatchQueue {
    /// Configuration
    config: BatchConfig,

    /// Task queue (priority queue)
    queue: Arc<Mutex<VecDeque<TaskEnvelope>>>,

    /// Task queues by type (when group_by_type is enabled)
    typed_queues: Arc<Mutex<HashMap<TaskType, VecDeque<TaskEnvelope>>>>,

    /// Result senders (one per task)
    result_senders: Arc<Mutex<HashMap<Uuid, oneshot::Sender<TaskResult>>>>,

    /// Channel for batch ready notifications
    batch_ready_tx: mpsc::Sender<Vec<TaskEnvelope>>,
    batch_ready_rx: Arc<Mutex<mpsc::Receiver<Vec<TaskEnvelope>>>>,

    /// Batch collection task handle
    collector_handle: Option<tokio::task::JoinHandle<()>>,
}

impl BatchQueue {
    /// Create a new batch queue
    pub fn new(config: BatchConfig) -> Self {
        let (batch_ready_tx, batch_ready_rx) = mpsc::channel(100);

        Self {
            config,
            queue: Arc::new(Mutex::new(VecDeque::new())),
            typed_queues: Arc::new(Mutex::new(HashMap::new())),
            result_senders: Arc::new(Mutex::new(HashMap::new())),
            batch_ready_tx,
            batch_ready_rx: Arc::new(Mutex::new(batch_ready_rx)),
            collector_handle: None,
        }
    }

    /// Start the batch collector background task
    pub fn start(&mut self) {
        let config = self.config.clone();
        let queue = self.queue.clone();
        let typed_queues = self.typed_queues.clone();
        let batch_ready_tx = self.batch_ready_tx.clone();

        let handle = tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(config.max_wait_time_ms)).await;

                // Collect batches
                if config.group_by_type {
                    // Collect batches by type
                    let mut queues = typed_queues.lock().await;
                    for (task_type, queue) in queues.iter_mut() {
                        if queue.len() >= config.min_batch_size {
                            let batch: Vec<_> = queue
                                .drain(..config.max_batch_size.min(queue.len()))
                                .collect();

                            debug!("Collected batch of {} tasks (type: {})", batch.len(), task_type);

                            if let Err(e) = batch_ready_tx.send(batch).await {
                                debug!("Failed to send batch: {}", e);
                                break;
                            }
                        }
                    }
                } else {
                    // Collect mixed batches
                    let mut q = queue.lock().await;
                    if q.len() >= config.min_batch_size {
                        let q_len = q.len();
                        let batch: Vec<_> = q
                            .drain(..config.max_batch_size.min(q_len))
                            .collect();

                        debug!("Collected mixed batch of {} tasks", batch.len());

                        if let Err(e) = batch_ready_tx.send(batch).await {
                            debug!("Failed to send batch: {}", e);
                            break;
                        }
                    }
                }
            }
        });

        self.collector_handle = Some(handle);
    }

    /// Submit a task to the queue
    ///
    /// Returns a receiver that will receive the task result when it's ready
    pub async fn submit(&self, task: TaskEnvelope) -> Result<oneshot::Receiver<TaskResult>> {
        let (tx, rx) = oneshot::channel();
        let task_id = task.id;

        // Store result sender
        self.result_senders.lock().await.insert(task_id, tx);

        // Add task to appropriate queue
        if self.config.group_by_type {
            let mut queues = self.typed_queues.lock().await;
            queues
                .entry(task.task_type)
                .or_insert_with(VecDeque::new)
                .push_back(task);
        } else {
            self.queue.lock().await.push_back(task);
        }

        trace!("Task {} submitted to queue", task_id);

        Ok(rx)
    }

    /// Get the next ready batch
    pub async fn get_batch(&self) -> Option<Vec<TaskEnvelope>> {
        self.batch_ready_rx.lock().await.recv().await
    }

    /// Send result for a completed task
    pub async fn send_result(&self, task_id: Uuid, result: TaskResult) -> Result<()> {
        if let Some(sender) = self.result_senders.lock().await.remove(&task_id) {
            sender.send(result)
                .map_err(|_| anyhow::anyhow!("Failed to send result for task {}", task_id))?;
        }
        Ok(())
    }

    /// Get queue statistics
    pub async fn stats(&self) -> QueueStats {
        let queue_len = self.queue.lock().await.len();
        let typed_lens: HashMap<TaskType, usize> = self.typed_queues.lock().await
            .iter()
            .map(|(k, v)| (*k, v.len()))
            .collect();
        let pending_results = self.result_senders.lock().await.len();

        QueueStats {
            total_queued: queue_len,
            queued_by_type: typed_lens,
            pending_results,
        }
    }

    /// Clear all queues
    pub async fn clear(&self) {
        self.queue.lock().await.clear();
        self.typed_queues.lock().await.clear();
        self.result_senders.lock().await.clear();
    }
}

impl Drop for BatchQueue {
    fn drop(&mut self) {
        if let Some(handle) = self.collector_handle.take() {
            handle.abort();
        }
    }
}

/// Queue statistics
#[derive(Debug, Clone)]
pub struct QueueStats {
    pub total_queued: usize,
    pub queued_by_type: HashMap<TaskType, usize>,
    pub pending_results: usize,
}

impl std::fmt::Display for QueueStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Queue Statistics:")?;
        writeln!(f, "  Total Queued: {}", self.total_queued)?;
        writeln!(f, "  Pending Results: {}", self.pending_results)?;
        writeln!(f, "  By Type:")?;
        for (task_type, count) in &self.queued_by_type {
            writeln!(f, "    {}: {}", task_type, count)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parallel::task::CodeGenTask;

    #[tokio::test]
    async fn test_batch_queue_creation() {
        let config = BatchConfig::default();
        let queue = BatchQueue::new(config);

        let stats = queue.stats().await;
        assert_eq!(stats.total_queued, 0);
    }

    #[tokio::test]
    async fn test_task_submission() {
        let config = BatchConfig::default();
        let queue = BatchQueue::new(config);

        let task = CodeGenTask::new("test".to_string(), "rust".to_string());
        let envelope = TaskEnvelope::new(task, 1);
        let task_id = envelope.id;

        let rx = queue.submit(envelope).await.unwrap();

        let stats = queue.stats().await;
        if queue.config.group_by_type {
            assert_eq!(stats.queued_by_type.get(&TaskType::CodeGeneration).copied().unwrap_or(0), 1);
        } else {
            assert_eq!(stats.total_queued, 1);
        }

        // Test sending result
        let result = TaskResult {
            task_id,
            success: true,
            output: serde_json::json!("result"),
            error: None,
            gpu_time_ms: 10.0,
            total_time_ms: 15.0,
        };

        queue.send_result(task_id, result).await.unwrap();
    }
}
