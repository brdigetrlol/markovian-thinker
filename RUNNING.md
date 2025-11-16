# How to Run Markovian Thinker

## 🚀 Quick Start

### Option 1: CPU-Only Mode (No GPU Required)

**Build:**
```bash
cd /mnt/c/Users/brdig/Documents/DriveSync/Software/repos-pxl/mcp/workspace/markovian-thinker
cargo build --release
```

**Run:**
```bash
./target/release/markovian-thinker
```

### Option 2: GPU-Accelerated Mode (Requires CUDA)

**Build:**
```bash
cargo build --release --features gpu
```

**Run:**
```bash
./target/release/markovian-thinker
```

---

## 📋 Prerequisites

### For CPU-Only Mode:
- Rust toolchain (1.70+)
- That's it!

### For GPU-Accelerated Mode:
- NVIDIA GPU with Compute Capability 7.5+ (RTX 20xx, 30xx, 40xx, etc.)
- CUDA Toolkit 11.0+ installed
- `nvcc` compiler in PATH
- Rust toolchain (1.70+)

---

## 🔧 Installation for Claude Desktop

### Step 1: Build the Binary

```bash
cd /mnt/c/Users/brdig/Documents/DriveSync/Software/repos-pxl/mcp/workspace/markovian-thinker

# CPU mode
cargo build --release

# OR GPU mode
cargo build --release --features gpu
```

### Step 2: Configure Claude Desktop

On Windows, edit: `%APPDATA%\Claude\claude_desktop_config.json`

On macOS/Linux, edit: `~/Library/Application Support/Claude/claude_desktop_config.json`

**Add this to the `mcpServers` section:**

```json
{
  "mcpServers": {
    "markovian-thinker": {
      "command": "C:\\Users\\brdig\\Documents\\DriveSync\\Software\\repos-pxl\\mcp\\workspace\\markovian-thinker\\target\\release\\markovian-thinker.exe",
      "args": [],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

**For GPU mode, adjust the path if needed or just use the same binary built with `--features gpu`.**

### Step 3: Restart Claude Desktop

Close and reopen Claude Desktop. The MCP server will start automatically.

---

## 🧪 Testing Locally

You can test the MCP server without Claude Desktop using the MCP Inspector:

### Install MCP Inspector:
```bash
npm install -g @modelcontextprotocol/inspector
```

### Run with Inspector:
```bash
# Terminal 1: Start the server
./target/release/markovian-thinker

# Terminal 2: Connect inspector
mcp-inspector
```

Or test directly with stdio:

```bash
# Start the server
./target/release/markovian-thinker

# Send a test request (paste this JSON and press Enter):
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}

# Then send initialized notification:
{"jsonrpc":"2.0","method":"initialized"}

# List available tools:
{"jsonrpc":"2.0","id":2,"method":"tools/list"}
```

---

## 📊 Available MCP Tools

Once running, you'll have access to these tools in Claude:

### Parallel Execution (GPU mode only):
- `parallel_codegen` - Generate multiple code snippets in parallel
- `parallel_analysis` - Analyze multiple texts in parallel
- `parallel_data_process` - Process data arrays on GPU
- `multi_agent_simulation` - Run multi-agent simulations on GPU
- `executor_stats` - Get GPU executor statistics

### Training & Weights:
- `load_weights` - Load pre-trained weights (SafeTensors, GGUF, etc.)
- `save_weights` - Save current model weights
- `enable_learning` - Enable online learning
- `disable_learning` - Disable online learning
- `add_training_example` - Add training example
- `get_learning_stats` - Get learning statistics
- `set_learning_rate` - Adjust learning rate
- `force_update` - Force weight update

### Reasoning:
- `markovian_think` - Chunk-based Markovian reasoning

---

## 🎯 Usage Examples

### In Claude Desktop:

Once configured, you can use the tools like this:

**1. Enable online learning:**
```
Can you enable learning for the markovian-thinker?
```

**2. Add training examples:**
```
Add this training example: input "What is 2+2?" target "2+2=4"
```

**3. Load pre-trained weights:**
```
Load weights from /path/to/model.safetensors
```

**4. Check learning progress:**
```
What are the learning stats?
```

**5. Run parallel code generation (GPU mode):**
```
Generate Python functions for: fibonacci, factorial, and prime checking
```

---

## 🔍 Monitoring & Logs

### View Logs

Logs are written to **STDERR** (stdout is reserved for MCP JSON):

```bash
# Run with verbose logging
RUST_LOG=debug ./target/release/markovian-thinker
```

### Log Levels
- `error` - Only errors
- `warn` - Warnings and errors
- `info` - General info (default)
- `debug` - Detailed debug info
- `trace` - Very verbose

### In Claude Desktop

Check Claude Desktop's logs for MCP server output:

**Windows:** `%APPDATA%\Claude\logs\`
**macOS:** `~/Library/Logs/Claude/`

---

## ⚙️ Configuration

### Environment Variables

- `RUST_LOG` - Log level (error, warn, info, debug, trace)
- `CUDA_VISIBLE_DEVICES` - GPU selection (e.g., "0" for first GPU)

### Runtime Options

Currently, all configuration is done through MCP tool calls. You can:

- Adjust learning rate: `set_learning_rate`
- Enable/disable learning: `enable_learning` / `disable_learning`
- Save/load checkpoints: `save_weights` / `load_weights`

---

## 🐛 Troubleshooting

### "CUDA not found" error

Make sure CUDA toolkit is installed and `nvcc` is in PATH:

```bash
nvcc --version
```

If not found, install CUDA Toolkit from: https://developer.nvidia.com/cuda-downloads

### "No GPU detected" at runtime

Check GPU is visible:

```bash
nvidia-smi
```

Set specific GPU:

```bash
CUDA_VISIBLE_DEVICES=0 ./target/release/markovian-thinker
```

### Build fails with "cudarc" errors

Build without GPU features:

```bash
cargo build --release
```

All CPU functionality still works (just slower).

### Claude Desktop can't find the binary

Use absolute path in `claude_desktop_config.json`:

```json
"command": "C:\\Users\\brdig\\Documents\\DriveSync\\Software\\repos-pxl\\mcp\\workspace\\markovian-thinker\\target\\release\\markovian-thinker.exe"
```

### MCP server not appearing in Claude

1. Check the JSON syntax in `claude_desktop_config.json`
2. Restart Claude Desktop completely
3. Check Claude Desktop logs for errors

---

## 🎓 Advanced Usage

### Loading Pre-Trained Weights

```bash
# Download a model (example: GPT-2 from HuggingFace)
# Then in Claude:

"Load weights from /path/to/gpt2-small.safetensors using safetensors format"
```

### Continuous Learning Workflow

```bash
# 1. Enable learning
"Enable online learning"

# 2. Use the model
"Generate code for a binary search algorithm"

# 3. Add feedback as training example
"Add training example with that query and response"

# 4. Monitor
"Show learning stats"

# 5. Save checkpoint
"Save weights to checkpoint-001.weights"
```

### GPU Multi-Task Execution

```bash
# Process multiple tasks in one call
"Generate Python, Rust, and JavaScript implementations of quicksort in parallel"
```

---

## 📈 Performance Tips

### CPU Mode
- Good for: Small models, testing, development
- Throughput: ~10 tasks/sec

### GPU Mode
- Good for: Production, large batches, training
- Throughput: 100+ tasks/sec
- Memory: Ensure GPU has >4GB VRAM

### Training
- Start with low learning rate (1e-5 to 1e-4)
- Monitor loss with `get_learning_stats`
- Save checkpoints every 100-1000 updates
- Use example weighting for important corrections

---

## 🔄 Updates & Rebuilding

When you make code changes:

```bash
# Rebuild
cargo build --release --features gpu

# Restart Claude Desktop to pick up new binary
```

No need to change `claude_desktop_config.json` unless the path changes.

---

## ✅ Verification

To verify everything is working:

1. **Build succeeds** with 0 errors
2. **Binary exists** at `target/release/markovian-thinker` (or `.exe` on Windows)
3. **Server starts** without crashing
4. **Tools appear** in Claude Desktop
5. **Test tool** works (try `get_learning_stats`)

---

## 🆘 Getting Help

If you encounter issues:

1. Check `RUST_LOG=debug` output
2. Check Claude Desktop logs
3. Verify CUDA installation (GPU mode)
4. Try CPU mode as fallback
5. Check GitHub issues

---

## 🎉 Success!

Once running, you should see these tools available in Claude Desktop:

- ✅ 1 reasoning tool (markovian_think)
- ✅ 5 parallel execution tools (GPU mode)
- ✅ 8 training & weight management tools

**Total: 14 MCP tools** ready to use! 🚀
