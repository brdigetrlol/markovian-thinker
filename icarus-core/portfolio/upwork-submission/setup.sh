#!/bin/bash

# Automated Upwork Portfolio Setup Script
# This script opens all HTML files and guides you through the process

echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║     UPWORK PORTFOLIO - AUTOMATED SETUP                        ║"
echo "║     RustML Sentiment API                                      ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""

# Get the directory where the script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

# Step 1: Open all HTML files in browser
echo "📂 Step 1: Opening HTML files in your browser..."
echo ""

if [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
    # Windows
    start 01-api-demo.html
    sleep 1
    start 02-project-overview.html
    sleep 1
    start 03-test-results.html
    sleep 1
    start 04-architecture.html
elif [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    open 01-api-demo.html
    sleep 1
    open 02-project-overview.html
    sleep 1
    open 03-test-results.html
    sleep 1
    open 04-architecture.html
else
    # Linux
    xdg-open 01-api-demo.html &
    sleep 1
    xdg-open 02-project-overview.html &
    sleep 1
    xdg-open 03-test-results.html &
    sleep 1
    xdg-open 04-architecture.html &
fi

echo "✅ All HTML files are now open in your browser!"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  📸 TAKE SCREENSHOTS NOW"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "For each browser tab:"
echo "  1. Press F11 to enter fullscreen mode"
echo "  2. Take a screenshot:"
echo "     • Windows: Win+Shift+S"
echo "     • Mac: Cmd+Shift+4"
echo "     • Linux: Print Screen"
echo "  3. Save the screenshot"
echo ""
echo "You should have 4 screenshots total."
echo ""

read -p "Press ENTER when you've taken all 4 screenshots..."

# Step 2: Open the form content file
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  📋 OPENING FORM CONTENT"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

if [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
    notepad UPWORK_FORM_CONTENT.txt
elif [[ "$OSTYPE" == "darwin"* ]]; then
    open -a TextEdit UPWORK_FORM_CONTENT.txt
else
    xdg-open UPWORK_FORM_CONTENT.txt || nano UPWORK_FORM_CONTENT.txt
fi

echo "✅ Form content file is now open!"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  🌐 NEXT STEPS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "1. Go to: https://www.upwork.com/freelancers/settings/portfolio"
echo "2. Click 'Add Project'"
echo "3. Copy-paste content from UPWORK_FORM_CONTENT.txt"
echo "4. Upload your 4 screenshots"
echo "5. Add GitHub link (in the text file)"
echo "6. Submit!"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  📊 QUICK COPY-PASTE"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Project Title:"
echo "High-Performance Sentiment Analysis API in Rust"
echo ""
echo "GitHub Link:"
echo "https://github.com/brdigetrlol/icarus-core/tree/main/portfolio/rustml-sentiment-api"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "✅ Everything is ready! Follow the steps above to complete your"
echo "   Upwork portfolio entry."
echo ""
echo "   Estimated time: 3-4 minutes"
echo ""
