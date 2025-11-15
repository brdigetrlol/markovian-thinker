# Markovian Thinker

A Rust workspace containing MCP servers for advanced AI reasoning and cognitive architecture.

## 🎯 Overview

This repository contains two main projects:

1. **Markovian Thinker MCP Server** - Chunk-based reasoning with bounded context for linear complexity
2. **Icarus Core** - Advanced cognitive architecture with event-driven processing and knowledge distillation

## 📦 Projects

### Markovian Thinker MCP Server

An MCP server implementation of "The Markovian Thinker" (arXiv:2510.06557), enabling Claude Code to perform extended reasoning through fixed-size chunks with bounded carryover.

**Key Features:**
- Linear complexity scaling O(n²S) vs traditional O(n²S²)
- Fixed chunk size with bounded carryover (Markovian property)
- Storm mitigation protection (rate limiting, circuit breaker, event fusion)
- Causal trace tracking for reasoning structure
- Concept space querying with crystallographic lattices

**Documentation:** See [icarus-core/README.md](icarus-core/README.md) for comprehensive documentation.

### Icarus Core

An advanced cognitive architecture featuring:
- Event-driven agent system (Perception, WorldModel, Planning, Memory, Action, Learning)
- Hierarchical memory system (working, short-term, long-term, episodic)
- Neural core with SSM, Liquid, and RNN layers
- Knowledge distillation framework for learning from interactions
- Integration with H2CE semantic search

**Documentation:** See [icarus-core/](icarus-core/) for architecture details and implementation guides.

## 🚀 Quick Start

### Prerequisites

- Rust toolchain (1.70+)
- Claude Code with MCP support

### Build

```bash
# Build markovian-thinker MCP server
cargo build --release

# Build icarus-core MCP server
cd icarus-core
cargo build --release
```

### Configuration

Add to Claude Code's MCP server configuration:

```json
{
  "mcpServers": {
    "markovian-thinker": {
      "command": "/path/to/markovian-thinker/target/release/markovian-thinker"
    },
    "icarus": {
      "command": "/path/to/markovian-thinker/icarus-core/target/release/icarus-mcp"
    }
  }
}
```

## 🧪 Testing

```bash
# Test markovian-thinker
cargo test

# Test icarus-core
cd icarus-core
cargo test
```

## 📁 Repository Structure

```
markovian-thinker/
├── src/                    # Markovian Thinker MCP server source
├── icarus-core/           # Icarus cognitive architecture
│   ├── src/              # Source code
│   ├── examples/         # Example programs
│   ├── tests/           # Test suite
│   └── docs/            # Documentation
├── Cargo.toml            # Workspace configuration
└── README.md            # This file
```

## 🎓 References

### Markovian Thinker
- Paper: "The Markovian Thinker: A Revolution in Large Language Model Reasoning"
- arXiv: [2510.06557v1](https://arxiv.org/html/2510.06557v1)

### Extended Features
- GPT-OSS optimizations (Mixture of Experts, Sliding Window Attention)
- Icarus TIC event-driven architecture
- Crystallographic concept spaces (E8, Leech lattices)
- H2CE multi-resolution semantic search

## 📝 License

MIT

## 🤝 Contributing

This is a research implementation. Contributions welcome!

## 🙏 Acknowledgments

- Sakana AI for the Markovian Thinker paper
- Model Context Protocol (MCP) specification
- Claude Code team for MCP support
