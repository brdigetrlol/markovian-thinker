# Markovian Thinker

**Chunk-based reasoning with bounded context for linear complexity**

An MCP server implementation of "The Markovian Thinker" (arXiv:2510.06557), enabling Claude Code to perform extended reasoning through fixed-size chunks with bounded carryover.

## 🎯 Core Innovation: Delethink Paradigm

Traditional LLM reasoning scales quadratically with sequence length (O(n²S²)). Markovian Thinker achieves **O(n²S) linear scaling** through:

- **Fixed chunk size** (C=8192 tokens)
- **Bounded carryover** (m=4096 tokens between chunks)
- **Markovian property**: Next chunk depends only on query + last m tokens
- **State transition**: x_{i+1} = query ⊕ y_i[-m:]

### Result: Linear Complexity

By maintaining constant context size, attention complexity becomes O(n²S) instead of O(n²S²), where:
- n: sequence length per chunk (constant)
- S: number of chunks (scales linearly with problem complexity)

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   Claude Code (Client)                  │
│                                                         │
│  ┌─────────────┐         MCP Protocol        ┌────────┐│
│  │   Calls:    │◄─────────────────────────────►  Tool  ││
│  │ markovian_  │       (stdio/JSON-RPC)        │ Server ││
│  │   solve     │                               │        ││
│  └─────────────┘                               └────────┘│
│        ▲                                            │    │
│        │                                            │    │
│        │  Sampling Requests (sampling/createMessage)│    │
│        └────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────┐
│         Markovian Thinker MCP Server (Rust)             │
│                                                         │
│  ┌──────────────────────────────────────────────────┐  │
│  │          ChunkManager Orchestration               │  │
│  │                                                    │  │
│  │  Loop: for i in 1..max_iterations {              │  │
│  │    1. Build prompt (query + carryover)           │  │
│  │    2. Request chunk from Claude via sampling     │  │
│  │    3. Extract last m tokens as carryover         │  │
│  │    4. Check for termination markers              │  │
│  │  }                                                │  │
│  └──────────────────────────────────────────────────┘  │
│                                                         │
│  ┌────────────┐ ┌──────────────┐ ┌─────────────────┐  │
│  │ Markovian  │ │   Reasoning  │ │ Bidirectional   │  │
│  │   State    │ │    Trace     │ │     Stdio       │  │
│  └────────────┘ └──────────────┘ └─────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

## 🚀 Quick Start

### Prerequisites

- Rust toolchain (1.70+)
- Claude Code with MCP support

### Build

```bash
cargo build --release
```

Binary location: `target/release/markovian-thinker`

### Configuration

Add to Claude Code's MCP server configuration:

```json
{
  "mcpServers": {
    "markovian-thinker": {
      "command": "/path/to/markovian-thinker/target/release/markovian-thinker"
    }
  }
}
```

### Usage

Once configured, Claude Code will have access to two tools:

#### 1. `markovian_solve`

Solve problems using chunk-based reasoning:

```
Use the markovian_solve tool to solve: <complex problem>
```

**Parameters**:
- `problem` (required): The problem or question to solve
- `chunk_size` (default: 8192): Maximum tokens per chunk
- `carryover_size` (default: 4096): Tokens to carry between chunks
- `max_iterations` (default: 5): Maximum number of chunks

#### 2. `markovian_status`

Get status of active reasoning sessions:

```
Check markovian_status to see active sessions
```

## 📊 Example Workflow

```
User: Solve the following math problem using Markovian reasoning...

Claude Code:
1. Calls markovian_solve tool with problem
2. Server orchestrates multi-chunk reasoning:
   - Chunk 1: Initial analysis (query only)
   - Chunk 2: Continuation (query + carryover from chunk 1)
   - Chunk 3: Further refinement (query + carryover from chunk 2)
   - ...
   - Chunk N: Final solution detected ([EOS] marker)
3. Returns complete reasoning trace and solution
```

## 🧪 Testing

### Manual Testing

Test the server with JSON-RPC requests:

```bash
# Test initialize
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | ./target/release/markovian-thinker

# Test tools/list
(cat test_simple.json; sleep 1) | ./target/release/markovian-thinker 2>/dev/null
```

### Unit Tests

```bash
cargo test
```

**Test coverage**: 30 passing tests across all modules
- Core state management: 10 tests
- Trace recording: 8 tests
- Chunk orchestration: 4 tests
- MCP protocol: 3 tests
- Sampling client: 3 tests
- Request parsing: 2 tests

## 📁 Project Structure

```
markovian-thinker/
├── src/
│   ├── lib.rs              # Public API
│   ├── state.rs            # Markovian state management (345 lines)
│   ├── trace.rs            # Reasoning trace recording (385 lines)
│   ├── chunk_manager.rs    # Orchestration logic (310 lines)
│   ├── mcp/
│   │   ├── protocol.rs     # MCP message types (450 lines)
│   │   ├── sampling.rs     # Sampling abstraction (200 lines)
│   │   ├── sampling_bridge.rs  # Stdio bridge (80 lines)
│   │   ├── server.rs       # MCP server (465 lines)
│   │   └── stdio.rs        # Bidirectional stdio (250 lines)
│   └── main.rs             # Server entry point
├── Cargo.toml              # Dependencies
├── README.md               # This file
├── mcp_config.json         # MCP configuration template
└── PROGRESS.md             # Development log
```

**Total**: ~2,700 lines of Rust code

## 🔬 Technical Details

### State Management

The `MarkovianState` struct maintains:
- Original query (preserved across iterations)
- Carryover from previous chunk
- Current iteration counter
- Token budget tracking

### Trace Recording

Every reasoning session produces a `ReasoningTrace`:
- Unique UUID identifier
- Array of chunks (prompt, output, tokens, timestamp)
- Termination reason
- Metadata (model, config, timing)

### Termination Detection

The server automatically detects solution completion via:
- `[EOS]` marker
- `\boxed{...}` LaTeX notation
- `####` solution headers

### MCP Resources

Reasoning traces are exposed as MCP resources:
- URI: `markovian://trace/{uuid}`
- Format: JSON
- Content: Complete trace with all chunks

## 🚀 Phase 6: Complete Integration (100% COMPLETE!)

**Storm Mitigation, Causal Traces, and Concept Querying** - All features production-ready!

### Storm Mitigation Protection

Comprehensive protection against resource exhaustion and cascading failures:

- **Rate Limiting**: Token bucket algorithm (19.8M checks/sec, 51ns latency)
- **Circuit Breaker**: Three-state protection (Closed/Open/HalfOpen, 78.7M checks/sec)
- **Event Fusion**: Deduplicates similar requests (1,078 fusions/sec, 50% reduction)
- **Real-time Metrics**: Monitor session health via `markovian_get_metrics`

### New MCP Tools

**1. Enhanced Session Creation** - `markovian_init_session`
```json
{
  "problem": "Your reasoning problem",
  "enable_storm_mitigation": true,
  "storm_mitigation_level": "default",
  "enable_causal_trace": true,
  "lattice_type": "e8"
}
```

**2. Causal Trace Query** - `markovian_get_trace` (enhanced)
```json
{
  "session_id": "uuid"
  // Returns: reasoning_trace + causal_trace
}
```

**3. Concept Space Query** - `markovian_query_concepts` ⭐ NEW
```json
{
  "session_id": "uuid",
  "embedding": [1.0, 0.5, 0.3, ...],
  "k": 5
  // Returns: similar concepts + statistics
}
```

**4. Health Monitoring** - `markovian_get_metrics`
```json
{
  "session_id": "uuid"
  // Returns: circuit state + success rates
}
```

### Performance Impact

**Total Phase 6 overhead**: < 1% of chunk processing time
- Storm mitigation check: 126 ns
- Expert selection: 1.9 µs
- Attention compression: 75 µs
- Concept crystallization: 48 ns

See `PHASE6_USER_GUIDE.md` and `PERFORMANCE_RESULTS.md` for details.

### Test Coverage

- **167 tests passing** (157 unit + 10 integration) ⭐
- Storm mitigation integration tests
- Causal trace integration tests ⭐ NEW
- Concept space integration tests ⭐ NEW
- Performance benchmarks included

### Documentation

- `PHASE6_FINAL_SUMMARY.md` - Complete implementation summary
- `PHASE6_USER_GUIDE.md` - Comprehensive user guide (600+ lines)
- `PERFORMANCE_RESULTS.md` - Detailed performance analysis (400+ lines)
- `QUICKSTART_PHASE6.md` - Quick start guide (300+ lines)

## 🎓 Paper Reference

This implementation is based on:

**"The Markovian Thinker: A Revolution in Large Language Model Reasoning"**
arXiv:2510.06557v1 [cs.CL]
https://arxiv.org/html/2510.06557v1

Key concepts:
- Delethink paradigm
- Markovian chunking
- Linear complexity scaling
- Bounded context windows

**Extended with**:
- GPT-OSS optimizations (Mixture of Experts, Sliding Window Attention)
- Icarus TIC event-driven architecture
- Crystallographic concept spaces
- Storm mitigation system

## 🔧 Development

### Build for Development

```bash
cargo build
cargo test
```

### Logging

Logs are written to **stderr** (stdout is reserved for MCP JSON):

```bash
./target/release/markovian-thinker 2> server.log
```

Log levels: TRACE, DEBUG, INFO, WARN, ERROR

### Adding Features

The codebase is modular:
- Extend `StateConfig` for new parameters
- Add termination conditions in `ChunkManager`
- Implement new tools in `MarkovianMCPServer`

## 📝 License

MIT

## 🤝 Contributing

This is a research implementation. Contributions welcome!

## 🙏 Acknowledgments

- Sakana AI for the Markovian Thinker paper
- Model Context Protocol (MCP) specification
- Claude Code team for MCP support
