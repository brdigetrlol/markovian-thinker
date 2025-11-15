# Icarus MCP Server - Quick Reference

Quick reference for common deployment and usage commands.

## Quick Start Commands

```bash
# Build and deploy (Windows)
build-and-deploy.bat

# Build manually
cargo build --release --bin icarus-mcp

# Install via cargo
cargo install --path . --bin icarus-mcp

# Run server
icarus-mcp.exe

# Run with config
icarus-mcp.exe --config icarus-config.toml
```

## Installation Paths

| Method | Location | Command |
|--------|----------|---------|
| User Install | `%USERPROFILE%\.icarus\bin\` | `build-and-deploy.bat` |
| Cargo Install | `%USERPROFILE%\.cargo\bin\` | `cargo install --path .` |
| System Install | `C:\Program Files\Icarus\bin\` | Manual (admin) |
| Portable | Any directory | Manual copy |

## PATH Configuration

```powershell
# Add to PATH (PowerShell)
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";$env:USERPROFILE\.icarus\bin", "User")

# Check PATH
echo $env:Path

# Refresh PATH without restart
$env:Path = [System.Environment]::GetEnvironmentVariable("Path","User")
```

```cmd
# Add to PATH (CMD)
setx PATH "%PATH%;%USERPROFILE%\.icarus\bin"

# Check PATH
echo %PATH%
```

## Claude Code MCP Config

**Location:** `%APPDATA%\Claude\claude_desktop_config.json`

**Basic Config:**
```json
{
  "mcpServers": {
    "icarus": {
      "command": "C:\\Users\\<YourName>\\.icarus\\bin\\icarus-mcp.exe"
    }
  }
}
```

**With Args:**
```json
{
  "mcpServers": {
    "icarus": {
      "command": "C:\\Users\\<YourName>\\.icarus\\bin\\icarus-mcp.exe",
      "args": ["--config", "C:\\Users\\<YourName>\\.icarus\\icarus-config.toml"]
    }
  }
}
```

## Nilesoft Shell Config

**Location:** `C:\Program Files\Nilesoft Shell\shell.nss`

**Basic Menu:**
```nss
menu(type="dir" title="Icarus AI")
{
    item(title="Analyze"
         cmd='%USERPROFILE%\.icarus\bin\icarus-mcp.exe'
         args='"@sel.path"')
}
```

**Reload Config:** Right-click desktop → Nilesoft Shell → Reload

## Build Variants

```bash
# Debug build (faster compile, slower runtime)
cargo build --bin icarus-mcp

# Release build (slower compile, faster runtime)
cargo build --release --bin icarus-mcp

# With features
cargo build --release --bin icarus-mcp --features cuda

# All binaries
cargo build --release

# Specific target
cargo build --release --target x86_64-pc-windows-gnu
```

## Testing Commands

```cmd
# Test binary exists
where icarus-mcp

# Test execution
icarus-mcp.exe --version

# Test MCP protocol
echo {"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}} | icarus-mcp.exe
```

## Common File Locations

| File | Path |
|------|------|
| Binary | `%USERPROFILE%\.icarus\bin\icarus-mcp.exe` |
| Config | `%USERPROFILE%\.icarus\icarus-config.toml` |
| Logs | `%USERPROFILE%\.icarus\logs\icarus.log` |
| Claude Config | `%APPDATA%\Claude\claude_desktop_config.json` |
| Nilesoft Config | `C:\Program Files\Nilesoft Shell\shell.nss` |

## Troubleshooting Quick Fixes

```bash
# Cargo not found - Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build errors - Clean and rebuild
cargo clean
cargo build --release

# PATH not working - Refresh
refreshenv  # Or restart terminal

# DLL errors - Install VC++ Redistributable
# Download from: https://aka.ms/vs/17/release/vc_redist.x64.exe
```

## Environment Variables

```cmd
# Set Icarus home
setx ICARUS_HOME "%USERPROFILE%\.icarus"

# Set config path
setx ICARUS_CONFIG "%USERPROFILE%\.icarus\icarus-config.toml"

# Enable debug logging
setx RUST_LOG "icarus_core=debug"

# Check variables
echo %ICARUS_HOME%
echo %ICARUS_CONFIG%
```

## Git Workflow

```bash
# Commit and push deployment files
git add build-and-deploy.bat nilesoft-mcp-integration.nss MCP_DEPLOYMENT_GUIDE.md
git commit -m "Add MCP deployment scripts and documentation"
git push origin main
```

## Distribution

```bash
# Create release directory
mkdir release
copy target\release\icarus-mcp.exe release\
copy icarus-config.toml release\
copy README.md release\

# Create archive
tar -czf icarus-mcp-v0.1.0-windows.tar.gz release/
# or
7z a icarus-mcp-v0.1.0-windows.zip release/
```

## Keyboard Shortcuts

In Nilesoft Shell, add shortcuts with `keys` parameter:

```nss
item(title="Icarus" keys="ctrl+shift+i" ...)  # Ctrl+Shift+I
item(title="Analyze" keys="ctrl+alt+a" ...)   # Ctrl+Alt+A
```

## Useful Links

- **Rust Installation:** https://rustup.rs/
- **Nilesoft Shell:** https://nilesoft.org/
- **Claude Code:** https://claude.ai/
- **MCP Docs:** https://modelcontextprotocol.io/
- **Cargo Book:** https://doc.rust-lang.org/cargo/

## Command Aliases (Optional)

Add to PowerShell profile (`$PROFILE`):

```powershell
# Icarus aliases
function icarus-mcp { & "$env:USERPROFILE\.icarus\bin\icarus-mcp.exe" $args }
function icarus-auto { & "$env:USERPROFILE\.icarus\bin\icarus-autonomous.exe" $args }
function icarus-build { cargo build --release --bin icarus-mcp }
function icarus-deploy { & ".\build-and-deploy.bat" }
```

## Quick Verification Checklist

```cmd
✓ Rust installed?          → cargo --version
✓ Binary built?            → dir target\release\icarus-mcp.exe
✓ Binary deployed?         → dir %USERPROFILE%\.icarus\bin\icarus-mcp.exe
✓ In PATH?                 → where icarus-mcp
✓ Executes?                → icarus-mcp.exe --version
✓ Nilesoft configured?     → Right-click folder, see "Icarus AI"?
✓ Claude configured?       → Check %APPDATA%\Claude\claude_desktop_config.json
✓ MCP working?             → Test in Claude Code
```

---

**Pro Tip:** Bookmark this page for quick reference during deployment! 📌
