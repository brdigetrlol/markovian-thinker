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

:: Get the directory path from parameter (passed by Nilesoft Shell)
set "REPO_PATH=%~1"

:: If no parameter, use current directory
if "%REPO_PATH%"=="" set "REPO_PATH=%CD%"

:: Display header
cls
echo.
echo %CYAN%========================================================%RESET%
echo %BOLD%%CYAN%        GitHub Repository Creator%RESET%
echo %CYAN%========================================================%RESET%
echo.
echo %CYAN%Target Directory:%RESET% %YELLOW%!REPO_PATH!%RESET%
echo.

:: Check for git command
where git >nul 2>&1
if %errorlevel% neq 0 (
    echo %RED%Error: 'git' command not found.%RESET%
    echo Please install Git and ensure it is in your system PATH.
    echo.
    pause
    exit /b 1
)

:: Check for gh command
where gh >nul 2>&1
if %errorlevel% neq 0 (
    echo %RED%Error: 'gh' (GitHub CLI) command not found.%RESET%
    echo Please install GitHub CLI from: https://cli.github.com/
    echo.
    pause
    exit /b 1
)

:: Check gh authentication
gh auth status >nul 2>&1
if %errorlevel% neq 0 (
    echo %RED%Error: GitHub CLI is not authenticated.%RESET%
    echo Please run: %YELLOW%gh auth login%RESET%
    echo.
    pause
    exit /b 1
)

:: Ask for repository visibility
echo %CYAN%Select repository visibility:%RESET%
echo   %GREEN%[1]%RESET% Public
echo   %BLUE%[2]%RESET% Private
echo.
set /p "choice=%CYAN%Enter choice (1 or 2):%RESET% "

if "%choice%"=="1" (
    set "VISIBILITY=public"
) else if "%choice%"=="2" (
    set "VISIBILITY=private"
) else (
    echo %RED%Invalid choice. Defaulting to private.%RESET%
    set "VISIBILITY=private"
    timeout /t 2 /nobreak >nul
)

:: Change to repository directory
cd /d "!REPO_PATH!"
if %errorlevel% neq 0 (
    echo %RED%Error: Failed to change to directory: !REPO_PATH!%RESET%
    pause
    exit /b 1
)

:: Get repository name from directory
for %%I in (.) do set "REPO_NAME=%%~nxI"

:: Total stages for overall progress
set TOTAL_STAGES=5
set CURRENT_STAGE=0

:: ============================================
:: STAGE 1: Initialize Git Repository
:: ============================================
set /a CURRENT_STAGE=1
call :DrawProgress "Initializing Git Repository" 0 100 !CURRENT_STAGE! %TOTAL_STAGES%

:: Remove existing .git directory if present
if exist ".git" (
    call :DrawProgress "Removing existing .git" 30 100 !CURRENT_STAGE! %TOTAL_STAGES%
    rd /s /q ".git" >nul 2>&1
)

git init -b main -q 2>&1
if %errorlevel% neq 0 (
    echo.
    echo %RED%Failed to initialize git repository.%RESET%
    pause
    exit /b 1
)
call :DrawProgress "Git Repository Initialized" 100 100 !CURRENT_STAGE! %TOTAL_STAGES%
timeout /t 1 /nobreak >nul

:: ============================================
:: STAGE 2: Counting Files
:: ============================================
set /a CURRENT_STAGE=2
call :DrawProgress "Counting Files" 0 100 !CURRENT_STAGE! %TOTAL_STAGES%

set FILE_COUNT=0
for /f "delims=" %%A in ('dir /s /b /a-d 2^>nul ^| findstr /v /i "\.git\\"') do set /a FILE_COUNT+=1

call :DrawProgress "Found !FILE_COUNT! files" 100 100 !CURRENT_STAGE! %TOTAL_STAGES%
timeout /t 1 /nobreak >nul

:: ============================================
:: STAGE 3: Adding Files
:: ============================================
set /a CURRENT_STAGE=3
call :DrawProgress "Adding Files" 25 100 !CURRENT_STAGE! %TOTAL_STAGES%

git add . 2>&1
if %errorlevel% neq 0 (
    echo.
    echo %RED%Failed to add files.%RESET%
    pause
    exit /b 1
)

call :DrawProgress "Files Added to Staging" 100 100 !CURRENT_STAGE! %TOTAL_STAGES%
timeout /t 1 /nobreak >nul

:: ============================================
:: STAGE 4: Creating Initial Commit
:: ============================================
set /a CURRENT_STAGE=4
call :DrawProgress "Creating Initial Commit" 50 100 !CURRENT_STAGE! %TOTAL_STAGES%

:: Check if there are files staged
git diff --cached --quiet
if %errorlevel% equ 0 (
    :: No changes, create README
    echo # !REPO_NAME! > README.md
    echo. >> README.md
    echo Created on %date% at %time% >> README.md
    git add README.md 2>nul
)

git commit -m "Initial commit" -q 2>&1
if %errorlevel% neq 0 (
    :: Try with --allow-empty if commit fails
    git commit -m "Initial commit" --allow-empty -q 2>&1
    if %errorlevel% neq 0 (
        echo.
        echo %RED%Failed to create commit.%RESET%
        echo %YELLOW%Make sure git user.name and user.email are configured.%RESET%
        pause
        exit /b 1
    )
)

call :DrawProgress "Commit Created" 100 100 !CURRENT_STAGE! %TOTAL_STAGES%
timeout /t 1 /nobreak >nul

:: ============================================
:: STAGE 5: Creating GitHub Repo and Pushing
:: ============================================
set /a CURRENT_STAGE=5
call :DrawProgress "Creating GitHub Repository" 25 100 !CURRENT_STAGE! %TOTAL_STAGES%

if /i "!VISIBILITY!"=="public" (
    gh repo create !REPO_NAME! --source=. --public --push >nul 2>&1
) else (
    gh repo create !REPO_NAME! --source=. --private --push >nul 2>&1
)

if %errorlevel% neq 0 (
    echo.
    echo %RED%Failed to create GitHub repository.%RESET%
    echo %YELLOW%Repository may already exist or there's a network issue.%RESET%
    pause
    exit /b 1
)

call :DrawProgress "Repository Created" 100 100 !CURRENT_STAGE! %TOTAL_STAGES%
timeout /t 1 /nobreak >nul

:: ============================================
:: SUCCESS!
:: ============================================
cls
echo.
echo %GREEN%========================================================%RESET%
echo %BOLD%%GREEN%                   SUCCESS!%RESET%
echo %GREEN%========================================================%RESET%
echo.
echo %CYAN%  Repository:%RESET% %YELLOW%!REPO_NAME!%RESET%
echo %CYAN%  Visibility:%RESET% %YELLOW%!VISIBILITY!%RESET%
echo %CYAN%  Files Added:%RESET% %YELLOW%!FILE_COUNT!%RESET%
echo %CYAN%  Location:%RESET% %YELLOW%!REPO_PATH!%RESET%
echo.
echo %GREEN%========================================================%RESET%
echo.
pause
exit /b 0

:: ============================================
:: Function: Draw Progress Bars
:: ============================================
:DrawProgress
set "OPERATION=%~1"
set /a "CURRENT_PERCENT=%~2"
set /a "TOTAL_PERCENT=%~3"
set /a "STAGE=%~4"
set /a "TOTAL_STAGES=%~5"

:: Calculate overall progress
set /a "OVERALL_BASE=(!STAGE!-1)*10000/!TOTAL_STAGES!"
set /a "OVERALL_STAGE=10000/!TOTAL_STAGES!"
set /a "OVERALL_PERCENT_RAW=!OVERALL_BASE!+(!CURRENT_PERCENT!*!OVERALL_STAGE!/100)"
set /a "OVERALL_INT=!OVERALL_PERCENT_RAW!/100"
set /a "OVERALL_DEC=!OVERALL_PERCENT_RAW!%%100"
if !OVERALL_DEC! LSS 10 set "OVERALL_DEC=0!OVERALL_DEC!"

:: Build progress bars
set /a "FILLED_CURRENT=!CURRENT_PERCENT!/2"
set /a "EMPTY_CURRENT=50-!FILLED_CURRENT!"
set "BAR_CURRENT="
set "EMPTY_BAR_CURRENT="

for /l %%i in (1,1,!FILLED_CURRENT!) do set "BAR_CURRENT=!BAR_CURRENT!█"
for /l %%i in (1,1,!EMPTY_CURRENT!) do set "EMPTY_BAR_CURRENT=!EMPTY_BAR_CURRENT!░"

set /a "FILLED_OVERALL=!OVERALL_INT!/2"
set /a "EMPTY_OVERALL=50-!FILLED_OVERALL!"
set "BAR_OVERALL="
set "EMPTY_BAR_OVERALL="

for /l %%i in (1,1,!FILLED_OVERALL!) do set "BAR_OVERALL=!BAR_OVERALL!█"
for /l %%i in (1,1,!EMPTY_OVERALL!) do set "EMPTY_BAR_OVERALL=!EMPTY_BAR_OVERALL!░"

:: Display
cls
echo.
echo %CYAN%========================================================%RESET%
echo %BOLD%%CYAN%        Creating GitHub Repository%RESET%
echo %CYAN%========================================================%RESET%
echo.
echo %YELLOW%  Current Operation:%RESET% !OPERATION!
echo.
echo   %GREEN%!BAR_CURRENT!%RESET%!EMPTY_BAR_CURRENT! %BOLD%%GREEN%!CURRENT_PERCENT!.00%%%RESET%
echo.
echo %MAGENTA%  Overall Progress:%RESET% [Stage !STAGE!/!TOTAL_STAGES!]
echo.
echo   %BLUE%!BAR_OVERALL!%RESET%!EMPTY_BAR_OVERALL! %BOLD%%BLUE%!OVERALL_INT!.!OVERALL_DEC!%%%RESET%
echo.
echo %CYAN%========================================================%RESET%

goto :eof
