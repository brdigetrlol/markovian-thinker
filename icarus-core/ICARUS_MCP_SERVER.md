# Icarus MCP Server

## Overview

The Icarus MCP Server exposes the Icarus cognitive AI system via the Model Context Protocol (MCP), making it accessible to Claude Code and other MCP-compatible clients.

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│              Claude Code (MCP Client)                     │
└──────────────┬───────────────────────────────────────────┘
               │ MCP Protocol (stdio/JSON-RPC)
               ↓
┌──────────────────────────────────────────────────────────┐
│            Icarus MCP Server (icarus-mcp)                 │
│  ┌────────────────────────────────────────────────────┐  │
│  │  MCP Layer (protocol, stdio, server)               │  │
│  └──────────────┬─────────────────────────────────────┘  │
│                 ↓                                         │
│  ┌────────────────────────────────────────────────────┐  │
│  │          Icarus Core (lib.rs)                      │  │
│  │                                                    │  │
│  │  • 6-Agent System (Perception, WorldModel,        │  │
│  │    Planning, Memory, Action, Learning)            │  │
│  │  • Hierarchical Memory (Working, Short,           │  │
│  │    Long-term, Episodic)                           │  │
│  │  • Neural Core (SSM/Liquid/RNN hybrid)            │  │
│  │  • World Model (Predictive simulation)            │  │
│  │  • Event Bus (Agent communication)                │  │
│  └────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

## Available MCP Tools

The Icarus MCP Server exposes the following tools:

### 1. **icarus_init**
Initialize Icarus cognitive system with configuration.

**Parameters:**
- `config_path` (optional): Path to Icarus configuration file

**Returns:** Initialization status and system info

### 2. **icarus_start**
Start Icarus autonomous operation.

**Returns:** Status confirmation

### 3. **icarus_stop**
Stop Icarus autonomous operation.

**Returns:** Status confirmation

### 4. **icarus_query_status**
Query overall Icarus system status.

**Returns:** System status including all agents, memory, neural core, and world model states

### 5. **icarus_query_agents**
Get status of all 6 cognitive agents.

**Parameters:**
- `agent_type` (optional): Specific agent to query (perception, world_model, planning, memory, action, learning)

**Returns:** Agent status information

### 6. **icarus_send_event**
Send an event to the Icarus event bus for agent processing.

**Parameters:**
- `event_type` (required): Type of event to send
- `data` (required): Event payload data

**Returns:** Event sending confirmation

### 7. **icarus_query_memory**
Query Icarus hierarchical memory system.

**Parameters:**
- `memory_level` (required): Memory level to query (working, short_term, long_term, episodic)
- `query` (optional): Query string for semantic search

**Returns:** Memory query results

### 8. **icarus_query_world_model**
Get current state of Icarus world model and predictions.

**Parameters:**
- `include_predictions` (optional): Include future state predictions (default: false)

**Returns:** World model state and optional predictions

### 9. **icarus_execute_action**
Request Icarus to execute an action in the environment.

**Parameters:**
- `action_type` (required): Type of action to execute
- `parameters` (required): Action parameters

**Returns:** Action execution status

### 10. **icarus_neural_state**
Get current neural core state (SSM/Liquid/RNN hybrid).

**Parameters:**
- `include_hidden_state` (optional): Include hidden state vectors (default: false)

**Returns:** Neural core state information

## Building

```bash
cd icarus-core
cargo build --release --bin icarus-mcp
```

Binary location: `target/release/icarus-mcp`

## Configuration

Add to Claude Code's MCP server configuration:

```json
{
  "mcpServers": {
    "icarus": {
      "command": "/path/to/icarus-core/target/release/icarus-mcp"
    }
  }
}
```

## Testing

Test the server initialization:

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | ./target/release/icarus-mcp
```

Expected output:
```json
{"jsonrpc":"2.0","id":1,"result":{"capabilities":{"resources":{"listChanged":false,"subscribe":false},"tools":{"listChanged":false}},"protocolVersion":"2024-11-05","serverInfo":{"name":"icarus","version":"0.1.0"}}}
```

## Integration with Other MCP Servers

Icarus integrates with:
- **markovian-thinker**: For extended reasoning via chunk-based processing
- **H2CE**: For semantic search and retrieval-augmented operations

## Project Structure

```
icarus-core/
├── src/
│   ├── lib.rs              # Main Icarus Core library
│   ├── main.rs             # Standalone Icarus binary
│   ├── agents.rs           # 6-agent cognitive system
│   ├── config.rs           # Configuration management
│   ├── memory.rs           # Hierarchical memory system
│   ├── neural.rs           # Neural core (SSM/Liquid/RNN)
│   ├── world_model.rs      # Predictive world model
│   ├── streams.rs          # Continuous stream processing
│   ├── event_bus.rs        # Event-driven communication
│   ├── mcp/
│   │   ├── mod.rs          # MCP module exports
│   │   ├── protocol.rs     # MCP protocol definitions
│   │   ├── stdio.rs        # Stdio handler for MCP
│   │   └── server.rs       # Icarus MCP server implementation
│   └── bin/
│       └── icarus-mcp.rs   # MCP server binary
├── Cargo.toml
└── ICARUS_MCP_SERVER.md    # This file
```

## Usage Examples

### Initialize Icarus via Claude Code

```
Use the icarus_init tool to initialize the Icarus cognitive system
```

### Query System Status

```
Use the icarus_query_status tool to check the current state of Icarus
```

### Send an Event

```
Use the icarus_send_event tool with event_type="user_input" and data={"text": "Hello Icarus"}
```

### Query Memory

```
Use the icarus_query_memory tool with memory_level="working" to see current working memory
```

## Development Roadmap

### Phase 1: Core Infrastructure ✓
- [x] MCP protocol implementation
- [x] Stdio bidirectional communication
- [x] Tool definitions and handlers
- [x] Basic server scaffolding

### Phase 2: Agent Integration (In Progress)
- [ ] Complete agent query implementation
- [ ] Event bus connection to MCP
- [ ] Agent state serialization

### Phase 3: Memory System
- [ ] Memory query implementation
- [ ] Semantic search integration (H2CE)
- [ ] Memory consolidation via MCP

### Phase 4: World Model
- [ ] World model state queries
- [ ] Prediction interface
- [ ] Simulation control via MCP

### Phase 5: Action Execution
- [ ] Action execution framework
- [ ] Tool integration for actions
- [ ] Action feedback loop

### Phase 6: Neural Core
- [ ] Neural state queries
- [ ] Hidden state visualization
- [ ] Training/adaptation interface

## License

MIT

## Authors

- Cody Moore <cody.moore@outlook.com>
