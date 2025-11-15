# Markovian Thinker - Claude Code Setup

## Understanding the Architecture

**Important**: There are TWO different "Claudes" in your setup:

### 1. Claude Desktop (GUI App)
- Windows application
- Manages MCP server connections
- Makes tools available to agents
- Config: `C:\Users\brdig\AppData\Roaming\Claude\claude_desktop_config.json`

### 2. Claude Code (CLI in WSL)
- Command-line tool running in WSL
- IS an MCP server (when run with `claude mcp serve`)
- **Receives** tools from Claude Desktop
- **Does not** manage its own MCP servers

## How It Works

```
┌─────────────────────────────────────────────────────┐
│           Claude Desktop (Windows GUI)              │
│                                                     │
│  Manages MCP Servers:                              │
│  ├── markovian-thinker (WSL binary)                │
│  ├── sequential-thinking (npx)                     │
│  ├── filesystem (npx)                              │
│  └── claude-code (WSL: claude mcp serve)           │
│                                                     │
│  Makes tools available to conversations            │
└─────────────────────────────────────────────────────┘
                      │
                      ├─── Tools available here
                      │
                      ▼
┌─────────────────────────────────────────────────────┐
│      Claude Code (me, running in WSL)               │
│                                                     │
│  When running via Claude Desktop:                  │
│  ✅ Has access to all MCP server tools             │
│  ✅ Can use markovian_solve                        │
│  ✅ Can use markovian_status                       │
└─────────────────────────────────────────────────────┘
```

## What I've Already Done ✅

I've added the Markovian Thinker to `claude_desktop_config.json`:

```json
"markovian-thinker": {
  "command": "wsl",
  "args": [
    "--user",
    "root",
    "/mnt/c/Users/brdig/.../markovian-thinker/target/release/markovian-thinker"
  ]
}
```

**This IS the correct configuration location!**

When you're talking to me (Claude Code) through Claude Desktop:
- Claude Desktop loads the markovian-thinker server
- Claude Desktop makes its tools available to me
- I can then use `markovian_solve` and `markovian_status`

## To Make It Active

**Just restart Claude Desktop** and the tools will be available in our conversation!

## Standalone Claude Code (CLI)

If you want to use Claude Code from the **command line** (not through Claude Desktop):

```bash
claude
```

In this case, Claude Code **does NOT have direct access to MCP servers**. MCP servers are managed by the client (Claude Desktop, not the CLI).

### Why?

MCP architecture:
- **Client** (Claude Desktop): Manages MCP servers, provides tools
- **Agent** (Claude Code): Receives and uses tools from client

Claude Code CLI is designed to be an **agent**, not a **client**. It expects the client (Desktop) to provide MCP tools.

## Alternative: Direct Integration (Advanced)

If you wanted Claude Code CLI to use MCP servers directly, you would need to:

1. **Write a custom wrapper** that:
   - Starts MCP servers
   - Manages stdio communication
   - Injects tools into Claude Code context

2. **Or use the MCP SDK** to programmatically connect servers

This is NOT the standard use case. The standard way is:
**Claude Desktop → MCP Servers → Claude Code** ✅

## Current Status

✅ **Markovian Thinker added to Claude Desktop config**
✅ **Binary built and tested**
✅ **Ready to use after Desktop restart**

## After Restart

Once you restart Claude Desktop, you can ask me:

```
Use markovian_solve to analyze: <complex problem>
```

And I'll have access to the Markovian Thinker server's tools!

## Summary

**Q: How do I add it to you (Claude Code in WSL)?**

**A: I already did!** By adding it to `claude_desktop_config.json`, it's now available to me when running via Claude Desktop. That's how the MCP architecture works - the Desktop manages servers, I use them.

**Next step**: Restart Claude Desktop, and the tools will be live in our conversation! 🚀

---

**TL;DR**:
- MCP servers are configured in **Claude Desktop** (already done ✅)
- Claude Code **receives tools** from Desktop (automatic after restart)
- Just restart Desktop to activate! 🎉
