# GPU Build and Usage Guide

## Prerequisites

### Required Software

1. **NVIDIA GPU** with Compute Capability 7.5+ (Turing, Ampere, or Ada architecture)
   - RTX 20xx series: Compute Capability 7.5
   - RTX 30xx series: Compute Capability 8.6
   - RTX 40xx series: Compute Capability 8.9
   - Check your GPU: `nvidia-smi --query-gpu=compute_cap --format=csv`

2. **CUDA Toolkit 11.0 or later**
   - Download from: https://developer.nvidia.com/cuda-downloads
   - Verify installation: `nvcc --version`

3. **NVIDIA Drivers**
   - Driver version 450+ for CUDA 11.0
   - Driver version 525+ for CUDA 12.0
   - Check version: `nvidia-smi`

4. **Rust 1.70+**
   - Install from: https://rustup.rs/
   - Verify: `rustc --version`

### Environment Setup

#### Linux/WSL2

```bash
# Add CUDA to PATH (add to ~/.bashrc or ~/.zshrc)
export PATH=/usr/local/cuda/bin:$PATH
export LD_LIBRARY_PATH=/usr/local/cuda/lib64:$LD_LIBRARY_PATH

# Verify CUDA
nvcc --version
nvidia-smi
```

#### Windows

```powershell
# CUDA should be added to PATH automatically during installation
# Verify:
nvcc --version
nvidia-smi
```

## Building

### Option 1: Build without GPU (CPU fallback only)

```bash
cargo build --release
```

This will compile the project but GPU features will be disabled.

### Option 2: Build with GPU Acceleration

```bash
# Build with GPU feature enabled
cargo build --release --features gpu

# Optionally set compute capability (if auto-detection fails)
export CUDA_COMPUTE_CAP=86  # For RTX 3000 series
cargo build --release --features gpu
```

### Build Process

When building with `--features gpu`, the build system will:

1. **Locate CUDA toolkit** (from `CUDA_PATH` env var or common install paths)
2. **Compile CUDA kernels** (`cuda/parallel_kernels.cu`) to PTX
3. **Embed PTX** in the binary via `PTX_PATH` environment variable
4. **Link CUDA libraries** (libcuda, libcudart)

### Common Build Issues

#### Issue: "CUDA toolkit not found"

**Solution:**
```bash
# Set CUDA_PATH manually
export CUDA_PATH=/usr/local/cuda-12.0  # Adjust version
cargo build --release --features gpu
```

#### Issue: "nvcc: command not found"

**Solution:**
```bash
# Add CUDA bin to PATH
export PATH=/usr/local/cuda/bin:$PATH
```

#### Issue: "Unsupported GPU architecture"

**Solution:**
```bash
# Set specific compute capability
export CUDA_COMPUTE_CAP=75  # For RTX 2080
cargo build --release --features gpu
```

#### Issue: "PTX compilation failed"

**Solution:**
```bash
# Build with verbose output
RUST_LOG=debug cargo build --release --features gpu 2>&1 | tee build.log

# Check CUDA kernel syntax
nvcc -ptx cuda/parallel_kernels.cu -o /tmp/test.ptx
```

## Running

### Start the MCP Server

```bash
# With GPU
RUST_LOG=info ./target/release/markovian-thinker

# Without GPU (CPU fallback)
RUST_LOG=info ./target/release/markovian-thinker
```

### Check GPU Status

When the server starts with GPU enabled, you'll see:

```
[INFO] Initializing CUDA context on device 0
[INFO] CUDA device initialized: NVIDIA GeForce RTX 3080 (compute capability 8.6)
[INFO] Loading CUDA kernels from PTX
[INFO] Successfully loaded 7 CUDA kernels
[INFO] GPU context initialized successfully with 4 streams
```

### Configure in Claude Desktop

Add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "markovian-gpu": {
      "command": "/path/to/markovian-thinker/target/release/markovian-thinker",
      "env": {
        "RUST_LOG": "info",
        "CUDA_VISIBLE_DEVICES": "0"
      }
    }
  }
}
```

## Usage Examples

### 1. Parallel Code Generation

Generate code for multiple functions simultaneously:

```javascript
// In Claude Desktop
Use the `parallel_codegen` MCP tool with:
{
  "prompts": [
    "Write a quick sort in Rust",
    "Write a binary search in Rust",
    "Write a merge sort in Rust"
  ],
  "language": "rust",
  "max_tokens": 512
}
```

**Expected Output:**
```json
{
  "results": [
    {"code": "// Generated quick sort...", "tokens": 100},
    {"code": "// Generated binary search...", "tokens": 80},
    {"code": "// Generated merge sort...", "tokens": 120}
  ],
  "gpu_time_ms": 15.3,
  "throughput": "196 tasks/sec"
}
```

### 2. Parallel Analysis

Analyze multiple code files simultaneously:

```javascript
{
  "tool": "parallel_analysis",
  "params": {
    "texts": [
      "fn main() { ... }",
      "class Database { ... }",
      "async function handler() { ... }"
    ],
    "analysis_type": "reason"
  }
}
```

### 3. Parallel Data Processing

Process datasets with GPU acceleration:

```javascript
{
  "tool": "parallel_data_process",
  "params": {
    "data_arrays": [
      [1.0, 2.0, 3.0, 4.0],
      [5.0, 6.0, 7.0, 8.0],
      [9.0, 10.0, 11.0, 12.0]
    ],
    "operation": "transform",
    "params": {"factor": 2.0}
  }
}
```

**Output:**
```json
{
  "results": [
    [2.0, 4.0, 6.0, 8.0],
    [10.0, 12.0, 14.0, 16.0],
    [18.0, 20.0, 22.0, 24.0]
  ],
  "gpu_time_ms": 0.5
}
```

### 4. Multi-Agent Simulation

Run agent-based simulations on GPU:

```javascript
{
  "tool": "multi_agent_simulation",
  "params": {
    "num_agents": 1000,
    "steps": 500,
    "environment_params": {
      "world_size": 100.0,
      "interaction_radius": 5.0
    }
  }
}
```

### 5. Executor Statistics

Monitor GPU utilization:

```javascript
{
  "tool": "executor_stats"
}
```

**Output:**
```json
{
  "num_workers": 4,
  "gpu_available": true,
  "queue": {
    "total_queued": 0,
    "pending_results": 0,
    "by_type": {}
  }
}
```

## Performance Tuning

### Batch Size

Adjust batch size for your GPU:

```rust
// In ExecutorConfig
ExecutorConfig {
    batch_config: BatchConfig {
        max_batch_size: 64,  // Increase for more VRAM
        min_batch_size: 8,
        max_wait_time_ms: 50,
        ...
    },
    ...
}
```

**Guidelines:**
- **4GB VRAM**: batch_size = 16-32
- **8GB VRAM**: batch_size = 32-64
- **16GB+ VRAM**: batch_size = 64-128

### Worker Count

```rust
ExecutorConfig {
    num_workers: 4,  // Match number of CUDA streams
    num_streams: 4,
    ...
}
```

### GPU Selection

For multi-GPU systems:

```bash
# Use specific GPU
CUDA_VISIBLE_DEVICES=1 ./target/release/markovian-thinker

# Use multiple GPUs (future feature)
CUDA_VISIBLE_DEVICES=0,1 ./target/release/markovian-thinker
```

## Monitoring

### GPU Utilization

```bash
# Watch GPU usage
watch -n 1 nvidia-smi

# Monitor specific metrics
nvidia-smi --query-gpu=utilization.gpu,memory.used,memory.total --format=csv -l 1
```

### Application Logs

```bash
# Detailed GPU logs
RUST_LOG=debug ./target/release/markovian-thinker

# Only GPU-related logs
RUST_LOG=markovian_thinker::gpu=debug ./target/release/markovian-thinker
```

## Benchmarking

### Built-in Benchmarks

```bash
# Run benchmarks (when implemented)
cargo bench --features gpu

# Specific benchmark
cargo bench --features gpu parallel_execution
```

### Manual Testing

```bash
# Test data processing kernel
cargo test --features gpu --release test_data_transform -- --ignored

# Test agent simulation
cargo test --features gpu --release test_agent_simulation -- --ignored
```

## Troubleshooting

### GPU Not Being Used

**Check:**
1. Built with `--features gpu`: `ldd target/release/markovian-thinker | grep cuda`
2. CUDA visible: `echo $CUDA_VISIBLE_DEVICES`
3. Logs show GPU init: `RUST_LOG=info ./target/release/markovian-thinker`

### Poor Performance

**Solutions:**
1. **Increase batch size** if GPU utilization is low
2. **Reduce batch size** if getting OOM errors
3. **Check thermal throttling**: `nvidia-smi --query-gpu=temperature.gpu --format=csv`
4. **Profile with nsys**:
   ```bash
   nsys profile --trace=cuda,nvtx ./target/release/markovian-thinker
   ```

### Memory Issues

**Out of Memory:**
```bash
# Reduce batch size
# Reduce max_tokens
# Use smaller models
```

**Memory Leaks:**
```bash
# Monitor over time
nvidia-smi --query-gpu=memory.used --format=csv -l 1
```

## Advanced Configuration

### Custom Kernels

To add new CUDA kernels:

1. **Add kernel to `cuda/parallel_kernels.cu`**:
```cuda
__global__ void my_custom_kernel(const float* input, float* output, int size) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < size) {
        output[idx] = my_operation(input[idx]);
    }
}
```

2. **Add Rust launcher in `src/gpu/kernels.rs`**:
```rust
pub struct MyCustomKernel {
    registry: Arc<KernelRegistry>,
}

impl MyCustomKernel {
    pub fn launch(&self, ...) -> Result<()> {
        // Launch implementation
    }
}
```

3. **Rebuild**:
```bash
cargo clean
cargo build --release --features gpu
```

### Debugging Kernels

```bash
# Compile with debug info
nvcc -G -g -ptx cuda/parallel_kernels.cu -o debug.ptx

# Run with cuda-gdb
cuda-gdb ./target/release/markovian-thinker
```

## Docker Deployment

```dockerfile
FROM nvidia/cuda:12.0-devel-ubuntu22.04

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:$PATH"

# Copy source
WORKDIR /app
COPY . .

# Build with GPU
RUN cargo build --release --features gpu

# Run
CMD ["./target/release/markovian-thinker"]
```

**Run:**
```bash
docker build -t markovian-gpu .
docker run --gpus all markovian-gpu
```

## Resources

- [CUDA Programming Guide](https://docs.nvidia.com/cuda/cuda-c-programming-guide/)
- [cudarc Documentation](https://docs.rs/cudarc/)
- [NVIDIA Nsight Systems](https://developer.nvidia.com/nsight-systems)
- [GPU Optimization Best Practices](https://docs.nvidia.com/cuda/cuda-c-best-practices-guide/)

## Support

For issues:
1. Check logs: `RUST_LOG=debug ./target/release/markovian-thinker 2>&1 | tee debug.log`
2. Verify CUDA: `nvidia-smi` and `nvcc --version`
3. Test kernels independently
4. Open GitHub issue with logs and GPU info
