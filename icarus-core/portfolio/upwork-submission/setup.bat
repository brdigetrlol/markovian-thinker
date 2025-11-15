@echo off
REM Automated Upwork Portfolio Setup Script for Windows
REM This script opens all HTML files and guides you through the process

echo ╔═══════════════════════════════════════════════════════════════╗
echo ║     UPWORK PORTFOLIO - AUTOMATED SETUP                        ║
echo ║     RustML Sentiment API                                      ║
echo ╚═══════════════════════════════════════════════════════════════╝
echo.

REM Step 1: Open all HTML files in browser
echo 📂 Step 1: Opening HTML files in your browser...
echo.

start 01-api-demo.html
timeout /t 1 /nobreak >nul
start 02-project-overview.html
timeout /t 1 /nobreak >nul
start 03-test-results.html
timeout /t 1 /nobreak >nul
start 04-architecture.html

echo ✅ All HTML files are now open in your browser!
echo.
echo ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
echo   📸 TAKE SCREENSHOTS NOW
echo ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
echo.
echo For each browser tab:
echo   1. Press F11 to enter fullscreen mode
echo   2. Press Win+Shift+S to take a screenshot
echo   3. Select the area and save
echo.
echo You should have 4 screenshots total.
echo.

pause

REM Step 2: Open the form content file
echo.
echo ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
echo   📋 OPENING FORM CONTENT
echo ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
echo.

start notepad UPWORK_FORM_CONTENT.txt

echo ✅ Form content file is now open!
echo.
echo ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
echo   🌐 NEXT STEPS
echo ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
echo.
echo 1. Go to: https://www.upwork.com/freelancers/settings/portfolio
echo 2. Click 'Add Project'
echo 3. Copy-paste content from UPWORK_FORM_CONTENT.txt
echo 4. Upload your 4 screenshots
echo 5. Add GitHub link (shown below)
echo 6. Submit!
echo.
echo ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
echo   📊 QUICK COPY-PASTE
echo ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
echo.
echo Project Title:
echo High-Performance Sentiment Analysis API in Rust
echo.
echo GitHub Link:
echo https://github.com/brdigetrlol/icarus-core/tree/main/portfolio/rustml-sentiment-api
echo.

REM Copy GitHub link to clipboard
echo https://github.com/brdigetrlol/icarus-core/tree/main/portfolio/rustml-sentiment-api | clip
echo ✅ GitHub link copied to clipboard!
echo.

echo ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
echo.
echo ✅ Everything is ready! Follow the steps above to complete your
echo    Upwork portfolio entry.
echo.
echo    Estimated time: 3-4 minutes
echo.

REM Open Upwork portfolio page
echo Opening Upwork portfolio page in your browser...
start https://www.upwork.com/freelancers/settings/portfolio

echo.
pause
