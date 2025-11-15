# Icarus MCP Server - Deployment Guide

Complete guide for deploying the Icarus MCP Server and integrating it with Nilesoft Shell context menus.

## Table of Contents

1. [Overview](#overview)
2. [Deployment Options](#deployment-options)
3. [Quick Start](#quick-start)
4. [Building from Source](#building-from-source)
5. [Deployment Methods](#deployment-methods)
6. [Nilesoft Shell Integration](#nilesoft-shell-integration)
7. [Claude Code Integration](#claude-code-integration)
8. [Advanced Configuration](#advanced-configuration)
9. [Troubleshooting](#troubleshooting)

---

## Overview

The Icarus MCP Server is a Rust-based Model Context Protocol (MCP) server that exposes the Icarus cognitive AI system to compatible clients like Claude Code.

**Key Binaries:**
- `icarus-mcp.exe` - MCP Server for Claude Code integration
- `icarus-autonomous.exe` - Autonomous mode for standalone operation

---

## Deployment Options

### Option 1: Local User Installation (Recommended)
- **Location**: `%USERPROFILE%\.icarus\bin\`
- **Pros**: No admin rights needed, user-specific
- **Cons**: Not available system-wide
- **Best for**: Personal development, testing

### Option 2: System-Wide Installation
- **Location**: `C:\Program Files\Icarus\`
- **Pros**: Available to all users, professional setup
- **Cons**: Requires administrator rights
- **Best for**: Multi-user systems, production

### Option 3: Portable Installation
- **Location**: Any directory (e.g., `D:\Tools\Icarus\`)
- **Pros**: Fully portable, no installation needed
- **Cons**: Must use absolute paths
- **Best for**: USB drives, temporary setups

### Option 4: Cargo Install (Developer)
- **Command**: `cargo install --path . --bin icarus-mcp`
- **Location**: `%USERPROFILE%\.cargo\bin\`
- **Pros**: Standard Rust toolchain integration
- **Cons**: Requires Rust toolchain installed
- **Best for**: Rust developers

### Option 5: Publish to crates.io
- **Command**: `cargo install icarus-core --bin icarus-mcp`
- **Pros**: Easy distribution, automatic updates
- **Cons**: Requires crates.io account, public release
- **Best for**: Public distribution

---

## Quick Start

### Prerequisites

1. **Rust Toolchain** (for building)
   ```bash
   # Install from https://rustup.rs/
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Nilesoft Shell** (for context menu integration)
   - Download: https://nilesoft.org/
   - Install and ensure it's running

3. **Claude Code** (optional, for MCP integration)
   - Claude desktop app or web version

### Build and Deploy (Automated)

1. Open Command Prompt in the `icarus-core` directory

2. Run the deployment script:
   ```cmd
   build-and-deploy.bat
   ```

3. Follow the prompts to:
   - Select which binary to build
   - Complete the build process
   - Deploy to `%USERPROFILE%\.icarus\bin\`

4. Add to PATH (if not already):
   ```powershell
   [Environment]::SetEnvironmentVariable("Path", $env:Path + ";$env:USERPROFILE\.icarus\bin", "User")
   ```

5. Verify installation:
   ```cmd
   icarus-mcp.exe --help
   ```

---

## Building from Source

### Manual Build Process

#### Debug Build (for development)
```bash
cargo build --bin icarus-mcp
# Output: target/debug/icarus-mcp.exe
```

#### Release Build (optimized)
```bash
cargo build --release --bin icarus-mcp
# Output: target/release/icarus-mcp.exe
```

#### Build All Binaries
```bash
cargo build --release
# Builds: icarus, icarus-mcp, icarus-autonomous
```

#### Build with Features
```bash
# With CUDA support
cargo build --release --bin icarus-mcp --features cuda

# With all features
cargo build --release --bin icarus-mcp --features full
```

### Cross-Compilation (Linux to Windows)

If you're building on Linux for Windows deployment:

```bash
# Install Windows target
rustup target add x86_64-pc-windows-gnu

# Build for Windows
cargo build --release --target x86_64-pc-windows-gnu --bin icarus-mcp

# Output: target/x86_64-pc-windows-gnu/release/icarus-mcp.exe
```

---

## Deployment Methods

### Method 1: Automated Script (Recommended)

Use the provided `build-and-deploy.bat`:

```cmd
cd icarus-core
build-and-deploy.bat
```

**What it does:**
1. Checks for Rust/Cargo installation
2. Builds release binaries
3. Creates `%USERPROFILE%\.icarus\bin\`
4. Copies binaries to installation directory
5. Copies configuration files
6. Checks PATH configuration

### Method 2: Manual Installation

#### Step 1: Build the Binary
```cmd
cargo build --release --bin icarus-mcp
```

#### Step 2: Create Installation Directory
```cmd
mkdir %USERPROFILE%\.icarus\bin
```

#### Step 3: Copy Binary
```cmd
copy target\release\icarus-mcp.exe %USERPROFILE%\.icarus\bin\
```

#### Step 4: Add to PATH
```powershell
# PowerShell (as Administrator)
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
[Environment]::SetEnvironmentVariable("Path", "$userPath;$env:USERPROFILE\.icarus\bin", "User")
```

Or manually:
1. Windows Key + R → `sysdm.cpl` → Advanced → Environment Variables
2. Under "User variables", edit "Path"
3. Add: `%USERPROFILE%\.icarus\bin`
4. Click OK, restart terminal

### Method 3: System-Wide Installation (Admin)

```cmd
# Build
cargo build --release --bin icarus-mcp

# Create directory (as Administrator)
mkdir "C:\Program Files\Icarus\bin"

# Copy binary
copy target\release\icarus-mcp.exe "C:\Program Files\Icarus\bin\"

# Add to System PATH
setx /M PATH "%PATH%;C:\Program Files\Icarus\bin"
```

### Method 4: Portable Deployment

```cmd
# Build
cargo build --release --bin icarus-mcp

# Copy to portable location
mkdir D:\Tools\Icarus
copy target\release\icarus-mcp.exe D:\Tools\Icarus\
copy icarus-config.toml D:\Tools\Icarus\

# Use absolute paths in Nilesoft Shell configuration
```

### Method 5: Cargo Install

#### Install from Local Source
```bash
cd icarus-core
cargo install --path . --bin icarus-mcp

# Installs to: %USERPROFILE%\.cargo\bin\icarus-mcp.exe
```

#### Install from Git (after pushing)
```bash
cargo install --git https://github.com/yourusername/icarus-core --bin icarus-mcp
```

#### Install from crates.io (after publishing)
```bash
cargo install icarus-core --bin icarus-mcp
```

---

## Nilesoft Shell Integration

### Basic Setup

1. **Open Nilesoft Shell Configuration**
   - Location: `C:\Program Files\Nilesoft Shell\shell.nss`
   - Or: Right-click desktop → Nilesoft Shell → Edit Config

2. **Add Configuration**

   Copy from `nilesoft-mcp-integration.nss`:

   ```nss
   menu(mode="multiple" type="file|dir" where=sel.count>0 title="Icarus AI" image=icon.ai)
   {
       item(title="Analyze with Icarus MCP"
            cmd='cmd.exe'
            args='/c "start "" cmd /k "%USERPROFILE%\.icarus\bin\icarus-mcp.exe" --context "@sel.path""')

       item(title="Start Autonomous Mode"
            cmd='cmd.exe'
            args='/c "start "" cmd /k "%USERPROFILE%\.icarus\bin\icarus-autonomous.exe" --workspace "@sel.path""')
   }
   ```

3. **Reload Nilesoft Shell**
   - Right-click desktop → Nilesoft Shell → Reload
   - Or restart Windows Explorer

4. **Test Integration**
   - Right-click any folder
   - Look for "Icarus AI" submenu
   - Click to launch

### Advanced Integrations

#### Context-Aware Menu Items

```nss
// Different options for code files
item(where=sel.count>0 type="file"
     where=sel.ext=="rs" || sel.ext=="py" || sel.ext=="js"
     title="Icarus: Code Analysis"
     cmd='cmd.exe'
     args='/c "%USERPROFILE%\.icarus\bin\icarus-mcp.exe" --code-analysis "@sel.path"')

// Project analysis for directories with Cargo.toml
item(where=sel.count>0 type="dir"
     where=io.exists(sel.path + "\Cargo.toml")
     title="Icarus: Analyze Rust Project"
     cmd='cmd.exe'
     args='/c "cd /d @sel.path && %USERPROFILE%\.icarus\bin\icarus-mcp.exe --project-analysis"')
```

#### Background Context Menu

```nss
// Right-click in empty space to launch in current directory
menu(mode="multiple" type="back" title="Icarus AI" image=icon.ai)
{
    item(title="Start MCP Server Here"
         cmd='cmd.exe'
         args='/c "cd /d @sel.dir && %USERPROFILE%\.icarus\bin\icarus-mcp.exe"')
}
```

#### Keyboard Shortcuts

```nss
item(title="Icarus MCP Server"
     keys="ctrl+shift+i"
     cmd='%USERPROFILE%\.icarus\bin\icarus-mcp.exe')
```

### Custom Icons

1. Create icon file: `icarus.ico`
2. Place in: `C:\Program Files\Nilesoft Shell\icons\`
3. Reference in config:
   ```nss
   image="C:\\Program Files\\Nilesoft Shell\\icons\\icarus.ico"
   ```

---

## Claude Code Integration

### Configuration File Location

**Windows:**
- `%APPDATA%\Claude\claude_desktop_config.json`
- Or: `C:\Users\<YourName>\AppData\Roaming\Claude\claude_desktop_config.json`

**macOS:**
- `~/Library/Application Support/Claude/claude_desktop_config.json`

**Linux:**
- `~/.config/Claude/claude_desktop_config.json`

### Basic MCP Configuration

```json
{
  "mcpServers": {
    "icarus": {
      "command": "C:\\Users\\<YourName>\\.icarus\\bin\\icarus-mcp.exe"
    }
  }
}
```

### Advanced Configuration

```json
{
  "mcpServers": {
    "icarus": {
      "command": "C:\\Users\\<YourName>\\.icarus\\bin\\icarus-mcp.exe",
      "args": [
        "--config", "C:\\Users\\<YourName>\\.icarus\\icarus-config.toml",
        "--log-level", "info"
      ],
      "env": {
        "ICARUS_HOME": "C:\\Users\\<YourName>\\.icarus",
        "RUST_LOG": "icarus_core=debug"
      }
    }
  }
}
```

### Multiple MCP Servers

```json
{
  "mcpServers": {
    "icarus": {
      "command": "C:\\Users\\<YourName>\\.icarus\\bin\\icarus-mcp.exe"
    },
    "markovian-thinker": {
      "command": "C:\\Users\\<YourName>\\.markovian\\bin\\markovian-mcp.exe"
    },
    "h2ce": {
      "command": "C:\\Users\\<YourName>\\.h2ce\\bin\\h2ce-mcp.exe"
    }
  }
}
```

### Restart Claude Code

After editing configuration:
1. Close Claude Code completely
2. Restart Claude Code
3. Check MCP servers are loaded: Look for Icarus tools in Claude

---

## Advanced Configuration

### Icarus Configuration File

Create `%USERPROFILE%\.icarus\icarus-config.toml`:

```toml
[system]
name = "Icarus"
version = "0.1.0"

[agents]
enabled = ["perception", "world_model", "planning", "memory", "action", "learning"]

[memory]
working_capacity = 1000
short_term_capacity = 10000
long_term_capacity = 1000000

[neural]
hidden_size = 512
num_layers = 4

[logging]
level = "info"
file = "%USERPROFILE%\\.icarus\\logs\\icarus.log"
```

### Environment Variables

```cmd
# Set via Command Prompt
setx ICARUS_HOME "%USERPROFILE%\.icarus"
setx ICARUS_CONFIG "%USERPROFILE%\.icarus\icarus-config.toml"
setx RUST_LOG "icarus_core=debug"

# Or via PowerShell
[Environment]::SetEnvironmentVariable("ICARUS_HOME", "$env:USERPROFILE\.icarus", "User")
```

### Logging Configuration

```toml
# In icarus-config.toml
[logging]
level = "debug"  # trace, debug, info, warn, error
file = "%USERPROFILE%\\.icarus\\logs\\icarus.log"
max_size = "100MB"
rotate = true
```

---

## Troubleshooting

### Build Issues

#### Error: "cargo: command not found"
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Restart terminal
```

#### Error: "linker 'link.exe' not found"
On Windows, install Visual Studio Build Tools:
- https://visualstudio.microsoft.com/downloads/
- Select "Desktop development with C++"

#### Error: "could not compile"
```bash
# Clean and rebuild
cargo clean
cargo build --release --bin icarus-mcp
```

### Deployment Issues

#### Binary not found in PATH
```cmd
# Check PATH
echo %PATH%

# Add manually
setx PATH "%PATH%;%USERPROFILE%\.icarus\bin"

# Restart terminal
```

#### Permission denied
```cmd
# Run as Administrator or use user directory
```

### Nilesoft Shell Issues

#### Menu items don't appear
1. Verify Nilesoft Shell is installed and running
2. Check configuration file syntax
3. Reload: Right-click desktop → Nilesoft Shell → Reload
4. Check logs: `C:\Program Files\Nilesoft Shell\logs\`

#### Icarus doesn't launch from context menu
1. Test manually: `%USERPROFILE%\.icarus\bin\icarus-mcp.exe`
2. Check paths in Nilesoft config are correct
3. Ensure binary has execute permissions

### MCP Integration Issues

#### Claude Code doesn't recognize Icarus
1. Check config file location and syntax
2. Verify binary path in config is correct
3. Restart Claude Code completely
4. Check logs: `%APPDATA%\Claude\logs\`

#### MCP server fails to start
1. Test manually: `icarus-mcp.exe`
2. Check for error messages
3. Verify dependencies are installed
4. Check configuration file syntax

### Runtime Issues

#### "DLL not found" errors
Install Visual C++ Redistributable:
- https://aka.ms/vs/17/release/vc_redist.x64.exe

#### Out of memory errors
Increase memory limits in `icarus-config.toml`:
```toml
[memory]
working_capacity = 500  # Reduce from 1000
```

#### Slow performance
1. Build with release mode: `cargo build --release`
2. Enable optimizations in `Cargo.toml`
3. Consider GPU acceleration with `--features cuda`

---

## Testing Deployment

### Manual Testing

```cmd
# Test binary exists
dir %USERPROFILE%\.icarus\bin\icarus-mcp.exe

# Test execution
%USERPROFILE%\.icarus\bin\icarus-mcp.exe --version

# Test MCP protocol (basic)
echo {"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}} | %USERPROFILE%\.icarus\bin\icarus-mcp.exe
```

### Integration Testing

1. **Nilesoft Shell Test**
   - Right-click any folder
   - Select "Icarus AI" → "Analyze with Icarus MCP"
   - Verify terminal opens with Icarus running

2. **Claude Code Test**
   - Open Claude Code
   - Check for Icarus tools available
   - Try: "Use icarus_init tool"

3. **Configuration Test**
   - Create test config file
   - Launch with: `icarus-mcp.exe --config test-config.toml`
   - Verify settings are applied

---

## Deployment Checklist

- [ ] Rust toolchain installed
- [ ] Project builds successfully
- [ ] Release binary created
- [ ] Installation directory created
- [ ] Binary copied to installation location
- [ ] PATH environment variable updated
- [ ] Binary executes from command line
- [ ] Configuration file created (if needed)
- [ ] Nilesoft Shell configured
- [ ] Context menu items appear
- [ ] Icarus launches from context menu
- [ ] Claude Code MCP configuration added
- [ ] Claude Code recognizes Icarus MCP server
- [ ] Icarus tools available in Claude Code
- [ ] Test execution successful

---

## Distribution Options

### GitHub Releases

1. Build release binary
2. Create GitHub release
3. Upload binary as asset
4. Users download and extract

### Installer Creation

**WiX Toolset:**
```xml
<!-- icarus-installer.wxs -->
<Wix>
  <Product Id="*" Name="Icarus MCP Server" Version="0.1.0">
    <!-- Installation configuration -->
  </Product>
</Wix>
```

**InnoSetup:**
```ini
[Setup]
AppName=Icarus MCP Server
AppVersion=0.1.0
DefaultDirName={pf}\Icarus

[Files]
Source: "target\release\icarus-mcp.exe"; DestDir: "{app}\bin"
```

### Chocolatey Package

```powershell
# Create package
choco new icarus-mcp

# Edit icarus-mcp.nuspec
# Build and publish
choco pack
choco push icarus-mcp.0.1.0.nupkg
```

---

## Next Steps

1. **Test thoroughly** in your environment
2. **Configure** for your specific use case
3. **Document** any custom configurations
4. **Share** with team or community
5. **Iterate** based on feedback

For support and updates:
- GitHub: https://github.com/yourusername/icarus-core
- Issues: https://github.com/yourusername/icarus-core/issues
- Docs: See `ICARUS_MCP_SERVER.md`

---

**Happy deploying! 🚀**
