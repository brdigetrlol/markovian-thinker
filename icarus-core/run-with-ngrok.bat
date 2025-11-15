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
set "RESET=[0m"
set "BOLD=[1m"

cls
echo.
echo %CYAN%========================================================%RESET%
echo %BOLD%%CYAN%   Icarus MCP Server - Ngrok Deployment%RESET%
echo %CYAN%========================================================%RESET%
echo.

:: Check for ngrok
where ngrok >nul 2>&1
if %errorlevel% neq 0 (
    echo %RED%Error: 'ngrok' command not found.%RESET%
    echo.
    echo %YELLOW%Install ngrok:%RESET%
    echo   1. Visit: https://ngrok.com/download
    echo   2. Download and install ngrok
    echo   3. Sign up for free account at https://dashboard.ngrok.com/signup
    echo   4. Get your authtoken and run: ngrok authtoken YOUR_TOKEN
    echo.
    pause
    exit /b 1
)

:: Check if binary exists
set "BINARY_PATH=%USERPROFILE%\.icarus\bin\icarus-mcp-http.exe"

if not exist "!BINARY_PATH!" (
    echo %YELLOW%Binary not found at: !BINARY_PATH!%RESET%
    echo.
    echo %CYAN%Building binary...%RESET%
    cargo build --release --bin icarus-mcp-http

    if %errorlevel% neq 0 (
        echo %RED%Build failed!%RESET%
        pause
        exit /b 1
    )

    set "BINARY_PATH=target\release\icarus-mcp-http.exe"
)

:: Get port
set "PORT=3000"
set /p "custom_port=%CYAN%Enter port (default 3000):%RESET% "
if not "!custom_port!"=="" set "PORT=!custom_port!"

echo.
echo %CYAN%========================================================%RESET%
echo %BOLD%%YELLOW%  Starting Services%RESET%
echo %CYAN%========================================================%RESET%
echo.

:: Start MCP server in background
echo %YELLOW%Starting Icarus MCP HTTP Server on port !PORT!...%RESET%
start "Icarus MCP Server" cmd /c "!BINARY_PATH! !PORT!"

:: Wait for server to start
timeout /t 3 /nobreak >nul

:: Start ngrok
echo %YELLOW%Starting ngrok tunnel...%RESET%
echo.

start "Ngrok Tunnel" cmd /c "ngrok http !PORT!"

:: Wait for ngrok to start
timeout /t 3 /nobreak >nul

:: Get ngrok public URL
echo %CYAN%Fetching ngrok public URL...%RESET%
timeout /t 2 /nobreak >nul

curl -s http://localhost:4040/api/tunnels > ngrok-info.json 2>nul

if exist ngrok-info.json (
    echo.
    echo %GREEN%========================================================%RESET%
    echo %BOLD%%GREEN%              Services Running!%RESET%
    echo %GREEN%========================================================%RESET%
    echo.
    echo %CYAN%  Local Server:%RESET%
    echo     http://localhost:!PORT!
    echo.
    echo %CYAN%  Ngrok Tunnel:%RESET%
    echo     Check the ngrok window for public URL
    echo     Or visit: http://localhost:4040
    echo.
    echo %GREEN%========================================================%RESET%
    echo.
    echo %YELLOW%To add to Claude Code:%RESET%
    echo.
    echo   %BOLD%1. Get your ngrok URL from the ngrok window%RESET%
    echo      (looks like: https://xxxx-xx-xx-xxx-xx.ngrok-free.app)
    echo.
    echo   %BOLD%2. Run this command:%RESET%
    echo      %CYAN%claude mcp add --transport http icarus YOUR_NGROK_URL%RESET%
    echo.
    echo   %BOLD%Example:%RESET%
    echo      %CYAN%claude mcp add --transport http icarus https://abcd-1234.ngrok-free.app%RESET%
    echo.
    echo %GREEN%========================================================%RESET%
    echo.
    echo %YELLOW%Press any key to view ngrok dashboard...%RESET%
    pause >nul
    start http://localhost:4040
    echo.
    echo %YELLOW%Press any key to stop services...%RESET%
    pause >nul

    :: Cleanup
    taskkill /FI "WINDOWTITLE eq Icarus MCP Server*" >nul 2>&1
    taskkill /FI "WINDOWTITLE eq Ngrok Tunnel*" >nul 2>&1
    del ngrok-info.json >nul 2>&1
) else (
    echo.
    echo %RED%Could not get ngrok information.%RESET%
    echo %YELLOW%Please check the ngrok window manually.%RESET%
    echo.
    pause
)

exit /b 0
