# Markovian Parallel Executor - Design Document

## Overview

A GPU-accelerated parallel task execution engine built on top of the markovian-thinker framework. Uses CUDA for high-performance parallel processing of multiple reasoning, code generation, analysis, and simulation tasks simultaneously.

## Architecture

### Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                    MCP Server Interface                      │
│  (parallel_think, batch_execute, multi_agent_sim)           │
└──────────────────────┬──────────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────────┐
│                  Parallel Executor                           │
│  - Task Queue Management                                     │
│  - Batching Logic (group tasks for GPU)                      │
│  - Worker Pool (tokio tasks)                                 │
│  - Result Distribution                                       │
└──────────────────────┬──────────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────────┐
│                   GPU Context                                │
│  - CUDA Device Management                                    │
│  - Memory Allocator (pinned host, device memory)             │
│  - Stream Pool (concurrent kernel execution)                 │
│  - Kernel Launcher                                           │
└──────────────────────┬──────────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────────┐
│                  CUDA Kernels                                │
│  - Batch Token Processing                                    │
│  - Parallel Attention (multi-head, multi-task)               │
│  - SSM Forward Pass (Mamba-style selective scan)             │
│  - Matrix Operations (batched GEMM, softmax)                 │
└──────────────────────────────────────────────────────────────┘
```

### Data Flow

1. **Task Submission** → Tasks arrive via MCP (or Rust API)
2. **Queue Management** → Tasks stored in priority queue
3. **Batching** → Collect N tasks (e.g., 32-128) for GPU batch
4. **Memory Transfer** → Copy task inputs to GPU (async via streams)
5. **Kernel Execution** → Launch CUDA kernels on batch
6. **Result Transfer** → Copy outputs back to host
7. **Distribution** → Return results to requesters

### Memory Management

```
CPU (Host)                           GPU (Device)
┌──────────────┐                    ┌──────────────┐
│ Task Queue   │                    │ Input Buffer │
│ (pinned mem) │ ──── Stream 1 ───▶ │ (device mem) │
└──────────────┘                    └──────────────┘
                                            │
┌──────────────┐                    ┌──────┴───────┐
│ Result Queue │                    │ Compute Core │
│ (pinned mem) │ ◀─── Stream 2 ──── │ (kernels)    │
└──────────────┘                    └──────┬───────┘
                                            │
                                    ┌───────▼──────┐
                                    │ Output Buffer│
                                    │ (device mem) │
                                    └──────────────┘
```

**Key Optimizations:**
- Pinned (page-locked) host memory for faster transfers
- Multiple CUDA streams for overlapping compute and transfer
- Memory pool reuse (avoid repeated alloc/free)
- Unified memory for small objects (automatic migration)

## Task Types

### 1. Code Generation Tasks
```rust
pub struct CodeGenTask {
    pub id: Uuid,
    pub prompt: String,
    pub language: String,
    pub max_tokens: usize,
    pub temperature: f32,
}
```

**GPU Optimization:** Batch multiple prompts, share vocabulary embeddings

### 2. Analysis/Reasoning Tasks
```rust
pub struct AnalysisTask {
    pub id: Uuid,
    pub input_text: String,
    pub analysis_type: AnalysisType, // Summarize, Extract, Classify
    pub config: StateConfig,
}
```

**GPU Optimization:** Parallel chunk processing across multiple documents

### 3. Data Processing Tasks
```rust
pub struct DataProcessTask {
    pub id: Uuid,
    pub data: Vec<u8>,
    pub operation: DataOp, // Transform, Filter, Aggregate
    pub params: HashMap<String, Value>,
}
```

**GPU Optimization:** SIMD-style parallel operations on data vectors

### 4. Multi-Agent Simulation
```rust
pub struct SimulationTask {
    pub id: Uuid,
    pub num_agents: usize,
    pub environment: Environment,
    pub steps: usize,
    pub agent_configs: Vec<AgentConfig>,
}
```

**GPU Optimization:** Parallel agent updates, shared world model on GPU

## CUDA Implementation

### Dependencies
```toml
[dependencies]
cudarc = "0.9"           # Safe CUDA bindings
cuda-sys = "0.3"         # Low-level CUDA
cuda-runtime-sys = "0.3" # Runtime API
tokio = { version = "1", features = ["full"] }
```

### Kernel Structure

**File:** `cuda/parallel_executor.cu`

```cuda
// Batch token processing kernel
__global__ void batch_process_tokens(
    const float* input_embeddings,  // [batch_size, seq_len, embed_dim]
    float* output_embeddings,       // [batch_size, seq_len, embed_dim]
    const int batch_size,
    const int seq_len,
    const int embed_dim
) {
    int batch_idx = blockIdx.x;
    int token_idx = blockIdx.y;
    int dim_idx = threadIdx.x;

    if (batch_idx < batch_size && token_idx < seq_len && dim_idx < embed_dim) {
        int idx = batch_idx * seq_len * embed_dim +
                  token_idx * embed_dim +
                  dim_idx;

        // Process token (placeholder - actual logic depends on model)
        output_embeddings[idx] = input_embeddings[idx];
    }
}

// Parallel multi-head attention (batched)
__global__ void batch_attention(
    const float* queries,   // [batch, heads, seq_len, head_dim]
    const float* keys,      // [batch, heads, seq_len, head_dim]
    const float* values,    // [batch, heads, seq_len, head_dim]
    float* output,          // [batch, heads, seq_len, head_dim]
    const int batch_size,
    const int num_heads,
    const int seq_len,
    const int head_dim
) {
    // Implement batched attention
    // Each block handles one (batch, head) pair
    // Threads compute attention scores and weighted sum
}

// SSM forward pass (Mamba-style selective scan)
__global__ void ssm_selective_scan(
    const float* input,     // [batch, seq_len, d_model]
    const float* delta,     // [batch, seq_len, d_state]
    const float* A,         // [d_model, d_state]
    const float* B,         // [batch, seq_len, d_state]
    const float* C,         // [batch, seq_len, d_state]
    float* output,          // [batch, seq_len, d_model]
    const int batch_size,
    const int seq_len,
    const int d_model,
    const int d_state
) {
    // Parallel selective scan across batch
    // Each block handles one sequence in batch
}
```

### Rust CUDA Interface

**File:** `src/gpu/cuda_context.rs`

```rust
use cudarc::driver::*;
use std::sync::Arc;

pub struct CudaContext {
    device: Arc<CudaDevice>,
    streams: Vec<CudaStream>,
    memory_pool: MemoryPool,
    kernels: KernelRegistry,
}

impl CudaContext {
    pub fn new(device_id: usize, num_streams: usize) -> Result<Self> {
        let device = CudaDevice::new(device_id)?;
        let streams = (0..num_streams)
            .map(|_| device.fork_default_stream())
            .collect::<Result<Vec<_>, _>>()?;

        let memory_pool = MemoryPool::new(&device)?;
        let kernels = KernelRegistry::load(&device)?;

        Ok(Self { device, streams, memory_pool, kernels })
    }

    pub async fn execute_batch<T: Task>(
        &self,
        tasks: Vec<T>,
        stream_idx: usize,
    ) -> Result<Vec<T::Output>> {
        let stream = &self.streams[stream_idx];

        // Allocate GPU memory from pool
        let input_buf = self.memory_pool.allocate::<f32>(tasks.len() * T::input_size())?;
        let output_buf = self.memory_pool.allocate::<f32>(tasks.len() * T::output_size())?;

        // Copy inputs to GPU (async)
        let input_data = tasks.iter().flat_map(|t| t.to_gpu_buffer()).collect::<Vec<_>>();
        stream.copy_host_to_device(&input_data, &input_buf)?;

        // Launch kernel
        let grid_dim = (tasks.len() as u32, 1, 1);
        let block_dim = (256, 1, 1);
        self.kernels.launch(
            T::kernel_name(),
            grid_dim,
            block_dim,
            stream,
            &[&input_buf, &output_buf],
        )?;

        // Copy results back (async)
        let mut output_data = vec![0.0f32; tasks.len() * T::output_size()];
        stream.copy_device_to_host(&output_buf, &mut output_data)?;

        // Synchronize stream
        stream.synchronize()?;

        // Parse results
        Ok(tasks.iter().enumerate().map(|(i, task)| {
            let start = i * T::output_size();
            let end = start + T::output_size();
            T::Output::from_gpu_buffer(&output_data[start..end])
        }).collect())
    }
}
```

## Parallel Executor

**File:** `src/parallel/executor.rs`

```rust
use tokio::sync::{mpsc, Mutex};
use std::collections::VecDeque;
use std::sync::Arc;

pub struct ParallelExecutor {
    task_queue: Arc<Mutex<VecDeque<TaskEnvelope>>>,
    gpu_context: Arc<CudaContext>,
    batch_size: usize,
    workers: Vec<tokio::task::JoinHandle<()>>,
    result_senders: Arc<Mutex<HashMap<Uuid, oneshot::Sender<TaskResult>>>>,
}

impl ParallelExecutor {
    pub fn new(gpu_device: usize, batch_size: usize, num_workers: usize) -> Result<Self> {
        let gpu_context = Arc::new(CudaContext::new(gpu_device, num_workers)?);
        let task_queue = Arc::new(Mutex::new(VecDeque::new()));
        let result_senders = Arc::new(Mutex::new(HashMap::new()));

        let workers = (0..num_workers)
            .map(|worker_id| {
                let queue = task_queue.clone();
                let gpu = gpu_context.clone();
                let senders = result_senders.clone();

                tokio::spawn(async move {
                    loop {
                        // Collect batch from queue
                        let batch = {
                            let mut q = queue.lock().await;
                            (0..batch_size)
                                .filter_map(|_| q.pop_front())
                                .collect::<Vec<_>>()
                        };

                        if batch.is_empty() {
                            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                            continue;
                        }

                        // Execute on GPU
                        match gpu.execute_batch(batch.clone(), worker_id).await {
                            Ok(results) => {
                                // Send results back
                                let mut senders_map = senders.lock().await;
                                for (task, result) in batch.iter().zip(results) {
                                    if let Some(sender) = senders_map.remove(&task.id) {
                                        let _ = sender.send(result);
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("GPU batch execution failed: {}", e);
                            }
                        }
                    }
                })
            })
            .collect();

        Ok(Self {
            task_queue,
            gpu_context,
            batch_size,
            workers,
            result_senders,
        })
    }

    pub async fn submit_task<T: Task>(&self, task: T) -> Result<T::Output> {
        let (tx, rx) = oneshot::channel();
        let task_id = Uuid::new_v4();

        // Store result sender
        self.result_senders.lock().await.insert(task_id, tx);

        // Enqueue task
        self.task_queue.lock().await.push_back(TaskEnvelope {
            id: task_id,
            task: Box::new(task),
        });

        // Wait for result
        rx.await.map_err(|_| anyhow::anyhow!("Task cancelled"))
    }

    pub async fn submit_batch<T: Task>(&self, tasks: Vec<T>) -> Result<Vec<T::Output>> {
        let futures = tasks.into_iter().map(|t| self.submit_task(t));
        futures::future::try_join_all(futures).await
    }
}
```

## MCP Integration

**File:** `src/mcp/parallel_tools.rs`

```rust
pub async fn handle_parallel_think(
    executor: Arc<ParallelExecutor>,
    params: ParallelThinkParams,
) -> Result<Value> {
    let tasks = params.prompts.into_iter().map(|prompt| {
        AnalysisTask {
            id: Uuid::new_v4(),
            input_text: prompt,
            analysis_type: AnalysisType::Reason,
            config: params.config.clone(),
        }
    }).collect();

    let results = executor.submit_batch(tasks).await?;

    Ok(json!({
        "results": results,
        "batch_size": results.len(),
        "gpu_time_ms": results[0].gpu_time_ms,
    }))
}

pub async fn handle_multi_agent_sim(
    executor: Arc<ParallelExecutor>,
    params: SimulationParams,
) -> Result<Value> {
    let task = SimulationTask {
        id: Uuid::new_v4(),
        num_agents: params.num_agents,
        environment: params.environment,
        steps: params.steps,
        agent_configs: params.agent_configs,
    };

    let result = executor.submit_task(task).await?;

    Ok(json!({
        "simulation_result": result,
        "agents": params.num_agents,
        "steps_completed": result.steps_completed,
    }))
}
```

## Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Batch Size | 32-128 tasks | Sweet spot for GPU utilization |
| Throughput | 1000+ tasks/sec | With batch processing |
| Latency | <50ms per task | Amortized over batch |
| GPU Utilization | >80% | Avoid idle time |
| Memory Efficiency | <8GB VRAM | For RTX 3080-class GPUs |

## Implementation Phases

### Phase 1: CUDA Foundation (Week 1-2)
- [ ] CUDA context management
- [ ] Memory pool implementation
- [ ] Basic kernel loading
- [ ] Stream management

### Phase 2: Task System (Week 2-3)
- [ ] Task trait and types
- [ ] Queue and batching logic
- [ ] Worker pool
- [ ] Result distribution

### Phase 3: CUDA Kernels (Week 3-5)
- [ ] Token processing kernels
- [ ] Attention kernels
- [ ] SSM selective scan
- [ ] Matrix operations (cuBLAS integration)

### Phase 4: Integration (Week 5-6)
- [ ] MCP server interface
- [ ] Integration with existing markovian-thinker
- [ ] Error handling and recovery
- [ ] Monitoring and metrics

### Phase 5: Optimization (Week 6-8)
- [ ] Kernel tuning (grid/block sizes)
- [ ] Memory access patterns
- [ ] Stream overlap optimization
- [ ] Benchmarking suite

### Phase 6: Advanced Features (Week 8+)
- [ ] Multi-GPU support
- [ ] Dynamic batching
- [ ] Priority queues
- [ ] Checkpointing and recovery

## Testing Strategy

1. **Unit Tests:** Each component (CUDA, queue, workers)
2. **Integration Tests:** End-to-end task execution
3. **Performance Tests:** Throughput, latency benchmarks
4. **Stress Tests:** High load, error conditions
5. **GPU Tests:** Memory leaks, stream synchronization

## Dependencies

```toml
[dependencies]
cudarc = "0.9"
cuda-sys = "0.3"
cuda-runtime-sys = "0.3"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
uuid = { version = "1", features = ["v4"] }
futures = "0.3"
```

## Success Metrics

- 10x throughput vs sequential processing
- Linear scaling with batch size (up to GPU limit)
- <100ms end-to-end latency for batch of 32 tasks
- Stable GPU memory usage (no leaks)
- 95%+ GPU utilization during batch processing

## References

- [CUDA Programming Guide](https://docs.nvidia.com/cuda/cuda-c-programming-guide/)
- [cudarc Documentation](https://docs.rs/cudarc/)
- [Mamba SSM Paper](https://arxiv.org/abs/2312.00752)
- [markovian-thinker Research](https://arxiv.org/abs/2510.06557)
