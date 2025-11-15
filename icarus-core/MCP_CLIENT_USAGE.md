# Markovian Thinker MCP Server - Usage Guide

## Overview

The Markovian Thinker is a **pure Rust MCP server** that implements the Delethink paradigm (arXiv:2510.06557) for chunk-based reasoning. It runs as a standalone binary and communicates via the Model Context Protocol (MCP) over stdio.

## Architecture

**No wrappers needed!** The Rust binary `target/release/markovian-thinker` IS the MCP server:
- Implements MCP protocol directly in Rust (`src/mcp/` module)
- Communicates via JSON-RPC 2.0 over stdin/stdout
- Spawned directly by Claude Code as a subprocess
- Zero dependencies on Python or Node.js

## Quick Start

### Option 1: Use with Claude Code (Recommended)

The MCP server is already configured in your Claude Code installation. It's automatically available when you use Claude Code.

**Verify it's configured:**
```bash
claude mcp list
```

You should see `markovian-thinker` in the list.

**Use it in Claude Code:**
Just ask Claude Code to use the Markovian Thinker tools:
```
> Use markovian-thinker to analyze the relationship between NEAT and evolutionary algorithms
```

### Option 2: Manual Configuration

If not already configured, add it:

```bash
# Add the MCP server
claude mcp add markovian-thinker /mnt/c/Users/brdig/Documents/DriveSync/Software/repos-pxl/mcp/workspace/markovian-thinker/target/release/markovian-thinker
```

Or edit your Claude Code config manually:

**Location**: `~/.claude/settings.json` or `.claude/settings.local.json`

```json
{
  "mcpServers": {
    "markovian-thinker": {
      "command": "/mnt/c/Users/brdig/Documents/DriveSync/Software/repos-pxl/mcp/workspace/markovian-thinker/target/release/markovian-thinker",
      "args": [],
      "env": {}
    }
  }
}
```

## Available Tools

The server provides two MCP tools:

### 1. markovian_init_session

Initialize a new Markovian reasoning session.

**Parameters:**
- `problem` (required, string) - The problem or question to solve
- `chunk_size` (optional, integer) - Maximum tokens per chunk (default: 8192)
- `carryover_size` (optional, integer) - Tokens to carry between chunks (default: chunk_size/2)
- `max_iterations` (optional, integer) - Maximum number of chunks (default: 5)

**Returns:** Session ID and configuration

### 2. markovian_get_prompt

Get the next reasoning prompt for a session.

**Parameters:**
- `session_id` (required, string) - UUID of the session

**Returns:** Prompt to reason about

### 3. markovian_submit_chunk

Submit a reasoning chunk to a session.

**Parameters:**
- `session_id` (required, string) - UUID of the session
- `output` (required, string) - Your reasoning output
- `tokens` (optional, integer) - Approximate token count

**Returns:** Continuation status and metadata

### 4. markovian_get_trace

Get the complete reasoning trace for a session.

**Parameters:**
- `session_id` (required, string) - UUID of the session

**Returns:** Full trace including all chunks and solution

### 5. markovian_list_sessions

List all active Markovian reasoning sessions.

**Parameters:** None

**Returns:** List of active sessions with metadata

## Testing the Server

### Option 1: Shell Script Testing (Recommended for CI/CD)

Use the included test script for automated testing:

```bash
cd /mnt/c/Users/brdig/Documents/DriveSync/Software/repos-pxl/mcp/workspace/markovian-thinker

# Run integration tests
./test_mcp_integration.sh
```

This script uses FIFOs for bidirectional communication and tests all MCP endpoints.

### Option 2: Direct JSON-RPC Testing

You can manually test the server by piping JSON-RPC requests:

```bash
# Build the server
cargo build --release

# Test initialization (basic health check)
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{"tools":{},"resources":{}},"clientInfo":{"name":"test","version":"1.0"}}}' | ./target/release/markovian-thinker

# List tools
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' | ./target/release/markovian-thinker
```

### Option 3: Use Claude Code Directly

The easiest way to test is just to use it through Claude Code:

```
> List available MCP tools from markovian-thinker

> Use markovian_list_sessions to show active sessions

> Create a session to solve: "What are the key innovations in NEAT?"
```

## Building from Source

```bash
cd /mnt/c/Users/brdig/Documents/DriveSync/Software/repos-pxl/mcp/workspace/markovian-thinker

# Build release binary
cargo build --release

# Binary location
./target/release/markovian-thinker

# Run tests
cargo test
```

## Configuration Reference

Example `mcp_config.json` showing all available options:

```json
{
  "mcpServers": {
    "markovian-thinker": {
      "command": "/path/to/markovian-thinker",
      "args": [],
      "env": {
        "RUST_LOG": "info"
      },
      "description": "Markovian chunk-based reasoning for complex problems",
      "capabilities": {
        "tools": true,
        "resources": true
      }
    }
  }
}
```

## Troubleshooting

### Server won't start

**Check the binary exists and is executable:**
```bash
ls -lh target/release/markovian-thinker
chmod +x target/release/markovian-thinker
```

**Rebuild if necessary:**
```bash
cargo clean
cargo build --release
```

### Server crashes or returns errors

**Check server logs:**
```bash
# Enable debug logging
RUST_LOG=debug ./target/release/markovian-thinker
```

**Verify JSON-RPC format:**
The server expects proper JSON-RPC 2.0 messages via stdin.

### Not showing up in Claude Code

**Verify configuration:**
```bash
claude mcp list
```

**Check config file syntax:**
```bash
cat ~/.claude/settings.json | jq .
```

## Integration Examples

### With Claude Code

```
User: Use the markovian-thinker to analyze this complex algorithm

Claude: I'll use the Markovian Thinker MCP server to break this down...
[Calls markovian_init_session]
[Calls markovian_get_prompt]
[Reasons about the problem]
[Calls markovian_submit_chunk]
...
```

### With Shell Scripts

```bash
#!/bin/bash

# Create a reasoning session
SESSION=$(echo '{
  "jsonrpc":"2.0",
  "id":1,
  "method":"tools/call",
  "params":{
    "name":"markovian_init_session",
    "arguments":{"problem":"Explain NEAT","chunk_size":4096}
  }
}' | ./target/release/markovian-thinker | jq -r '.result.content[0].text' | jq -r '.session_id')

echo "Session: $SESSION"

# Get the reasoning prompt
PROMPT=$(echo "{
  \"jsonrpc\":\"2.0\",
  \"id\":2,
  \"method\":\"tools/call\",
  \"params\":{
    \"name\":\"markovian_get_prompt\",
    \"arguments\":{\"session_id\":\"$SESSION\"}
  }
}" | ./target/release/markovian-thinker | jq -r '.result.content[0].text')

echo "Prompt: $PROMPT"
```

## Performance Notes

- **Chunk Size**: 4096-8192 tokens recommended for good performance
- **Max Iterations**: Typically 3-5 chunks sufficient for most problems
- **Carryover**: Default of chunk_size/2 provides good context continuity
- **Concurrency**: Server handles one session at a time, but manages multiple sessions

## See Also

- **MCP Protocol**: https://modelcontextprotocol.io
- **Delethink Paper**: arXiv:2510.06557
- **Source Code**: `src/mcp/` for protocol implementation
- **Test Scripts**: `test_mcp_integration.sh` for testing patterns

---

**Created:** 2025-10-30
**MCP Protocol Version:** 2024-11-05
**Server Version:** 0.1.0
**Implementation:** Pure Rust (no wrappers)
