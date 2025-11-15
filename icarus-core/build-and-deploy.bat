@echo off
setlocal enabledelayedexpansion

:: Enable ANSI escape codes for colors
reg add HKCU\Console /v VirtualTerminalLevel /t REG_DWORD /d 1 /f >nul 2>&1

:: ANSI Color Codes
set "GREEN=[92m"
set "CYAN=[96m"
set "YELLOW=[93m"
set "RED=[91m"
set "BLUE=[94m"
set "MAGENTA=[95m"
set "RESET=[0m"
set "BOLD=[1m"

:: Configuration
set "PROJECT_NAME=icarus-core"
set "BINARY_NAME=icarus-mcp"
set "INSTALL_DIR=%USERPROFILE%\.icarus"
set "BIN_DIR=%INSTALL_DIR%\bin"

cls
echo.
echo %CYAN%========================================================%RESET%
echo %BOLD%%CYAN%     Icarus MCP Server - Build and Deploy%RESET%
echo %CYAN%========================================================%RESET%
echo.

:: Check for cargo
where cargo >nul 2>&1
if %errorlevel% neq 0 (
    echo %RED%Error: 'cargo' command not found.%RESET%
    echo Please install Rust from: https://rustup.rs/
    echo.
    pause
    exit /b 1
)

:: Display configuration
echo %CYAN%Configuration:%RESET%
echo   Project: %YELLOW%!PROJECT_NAME!%RESET%
echo   Binary: %YELLOW%!BINARY_NAME!%RESET%
echo   Install Dir: %YELLOW%!INSTALL_DIR!%RESET%
echo   Binary Dir: %YELLOW%!BIN_DIR!%RESET%
echo.

:: Ask user what to build
echo %CYAN%Select binary to build:%RESET%
echo   %GREEN%[1]%RESET% icarus-mcp (MCP Server - Stdio)
echo   %BLUE%[2]%RESET% icarus-mcp-http (MCP Server - HTTP for Claude Code Web)
echo   %YELLOW%[3]%RESET% icarus-autonomous (Autonomous Mode)
echo   %MAGENTA%[4]%RESET% All binaries
echo.
set /p "choice=%CYAN%Enter choice (1, 2, 3, or 4):%RESET% "

if "%choice%"=="1" (
    set "BUILD_TARGET=icarus-mcp"
    set "BUILD_ALL=false"
) else if "%choice%"=="2" (
    set "BUILD_TARGET=icarus-mcp-http"
    set "BUILD_ALL=false"
) else if "%choice%"=="3" (
    set "BUILD_TARGET=icarus-autonomous"
    set "BUILD_ALL=false"
) else if "%choice%"=="4" (
    set "BUILD_TARGET=all"
    set "BUILD_ALL=true"
) else (
    echo %RED%Invalid choice. Defaulting to icarus-mcp.%RESET%
    set "BUILD_TARGET=icarus-mcp"
    set "BUILD_ALL=false"
    timeout /t 2 /nobreak >nul
)

echo.
echo %CYAN%========================================================%RESET%
echo %BOLD%%YELLOW%  Building Release Binary%RESET%
echo %CYAN%========================================================%RESET%
echo.

:: Build the binary
if "!BUILD_ALL!"=="true" (
    echo %YELLOW%Building all binaries...%RESET%
    cargo build --release
) else (
    echo %YELLOW%Building !BUILD_TARGET!...%RESET%
    cargo build --release --bin !BUILD_TARGET!
)

if %errorlevel% neq 0 (
    echo.
    echo %RED%Build failed!%RESET%
    pause
    exit /b 1
)

echo.
echo %GREEN%Build completed successfully!%RESET%
echo.

:: Create installation directory
echo %CYAN%========================================================%RESET%
echo %BOLD%%YELLOW%  Creating Installation Directory%RESET%
echo %CYAN%========================================================%RESET%
echo.

if not exist "!INSTALL_DIR!" (
    echo %YELLOW%Creating: !INSTALL_DIR!%RESET%
    mkdir "!INSTALL_DIR!" >nul 2>&1
)

if not exist "!BIN_DIR!" (
    echo %YELLOW%Creating: !BIN_DIR!%RESET%
    mkdir "!BIN_DIR!" >nul 2>&1
)

:: Copy binaries
echo.
echo %CYAN%========================================================%RESET%
echo %BOLD%%YELLOW%  Deploying Binaries%RESET%
echo %CYAN%========================================================%RESET%
echo.

if "!BUILD_ALL!"=="true" (
    echo %YELLOW%Copying icarus-mcp.exe...%RESET%
    copy /Y "target\release\icarus-mcp.exe" "!BIN_DIR!\icarus-mcp.exe" >nul
    echo %YELLOW%Copying icarus-mcp-http.exe...%RESET%
    copy /Y "target\release\icarus-mcp-http.exe" "!BIN_DIR!\icarus-mcp-http.exe" >nul
    echo %YELLOW%Copying icarus-autonomous.exe...%RESET%
    copy /Y "target\release\icarus-autonomous.exe" "!BIN_DIR!\icarus-autonomous.exe" >nul
) else (
    echo %YELLOW%Copying !BUILD_TARGET!.exe...%RESET%
    copy /Y "target\release\!BUILD_TARGET!.exe" "!BIN_DIR!\!BUILD_TARGET!.exe" >nul
)

if %errorlevel% neq 0 (
    echo.
    echo %RED%Failed to copy binary!%RESET%
    pause
    exit /b 1
)

echo.
echo %GREEN%Binaries deployed successfully!%RESET%
echo.

:: Copy configuration files if they exist
if exist "icarus-config.toml" (
    echo %YELLOW%Copying configuration file...%RESET%
    copy /Y "icarus-config.toml" "!INSTALL_DIR!\icarus-config.toml" >nul
)

:: Check if binary directory is in PATH
echo %CYAN%========================================================%RESET%
echo %BOLD%%YELLOW%  Checking PATH Configuration%RESET%
echo %CYAN%========================================================%RESET%
echo.

echo %PATH% | findstr /C:"!BIN_DIR!" >nul
if %errorlevel% neq 0 (
    echo %YELLOW%Warning: !BIN_DIR! is not in your PATH.%RESET%
    echo.
    echo %CYAN%To add it to your PATH:%RESET%
    echo   1. Open System Properties ^> Environment Variables
    echo   2. Add to User PATH: %YELLOW%!BIN_DIR!%RESET%
    echo.
    echo %CYAN%Or run this command in PowerShell (as Administrator):%RESET%
    echo %YELLOW%[Environment]::SetEnvironmentVariable("Path", $env:Path + ";!BIN_DIR!", "User")%RESET%
    echo.
) else (
    echo %GREEN%Binary directory is already in PATH!%RESET%
    echo.
)

:: Display success summary
cls
echo.
echo %GREEN%========================================================%RESET%
echo %BOLD%%GREEN%           Deployment Successful!%RESET%
echo %GREEN%========================================================%RESET%
echo.
echo %CYAN%  Installation Directory:%RESET% %YELLOW%!INSTALL_DIR!%RESET%
echo %CYAN%  Binary Location:%RESET% %YELLOW%!BIN_DIR!%RESET%
echo.
if "!BUILD_ALL!"=="true" (
    echo %CYAN%  Installed Binaries:%RESET%
    echo     - %YELLOW%icarus-mcp.exe%RESET% (Stdio MCP Server)
    echo     - %YELLOW%icarus-mcp-http.exe%RESET% (HTTP MCP Server for Web)
    echo     - %YELLOW%icarus-autonomous.exe%RESET% (Autonomous Mode)
) else (
    echo %CYAN%  Installed Binary:%RESET% %YELLOW%!BUILD_TARGET!.exe%RESET%
)
echo.
echo %CYAN%  Test the installation:%RESET%
echo     %YELLOW%!BIN_DIR!\!BUILD_TARGET!.exe --help%RESET%
echo.
echo %GREEN%========================================================%RESET%
echo.
echo %CYAN%Next Steps:%RESET%
echo   1. Add !BIN_DIR! to your PATH (if not already done)
echo   2. Configure Claude Code MCP settings
echo   3. Use Nilesoft Shell integration for context menu
echo.
echo %YELLOW%See MCP_DEPLOYMENT_GUIDE.md for detailed instructions%RESET%
echo.
pause
exit /b 0
