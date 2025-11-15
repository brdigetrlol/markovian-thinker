#!/bin/bash

# Local AI Feature Generator
# Uses YOUR Claude subscription via browser automation or manual interaction

echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║                                                               ║"
echo "║     🤖 AI Feature Generator (Local)                          ║"
echo "║     Uses your Claude subscription                            ║"
echo "║                                                               ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""

# Get feature description from user
echo "📝 Enter the feature you want to implement:"
read -p "> " FEATURE_DESC

# Analyze codebase
echo ""
echo "🔍 Analyzing your codebase..."
RUST_FILES=$(find portfolio -name "*.rs" -type f | wc -l)
TS_FILES=$(find portfolio -name "*.ts" -type f | wc -l)
echo "   Found: $RUST_FILES Rust files, $TS_FILES TypeScript files"

# Get recent code context
echo ""
echo "📋 Recent code patterns:"
grep -r "async fn" portfolio/rustml-sentiment-api/src/ 2>/dev/null | head -5

# Generate prompt for Claude
PROMPT="I need to implement this feature in my Rust/TypeScript project:

**Feature**: $FEATURE_DESC

**Current Project Structure**:
- Rust files: $RUST_FILES
- TypeScript files: $TS_FILES

**Existing Code Patterns**:
$(grep -r "async fn" portfolio/rustml-sentiment-api/src/ 2>/dev/null | head -5)

Please generate:
1. Complete Rust implementation (main code file)
2. Comprehensive test suite
3. TypeScript frontend integration (if needed)
4. Documentation updates

Match the existing code style and use production-ready patterns with proper error handling."

# Save prompt to file
echo "$PROMPT" > /tmp/claude-prompt.txt

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "📋 PROMPT READY!"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Next steps:"
echo ""
echo "1. Open: https://claude.ai"
echo "2. Start new conversation"
echo "3. Paste this prompt:"
echo ""
cat /tmp/claude-prompt.txt
echo ""
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Or copy from: /tmp/claude-prompt.txt"
echo ""

# Open Claude.ai automatically (optional)
read -p "Open Claude.ai in browser? (y/n): " OPEN_BROWSER
if [ "$OPEN_BROWSER" = "y" ]; then
    if command -v xdg-open &> /dev/null; then
        xdg-open "https://claude.ai"
    elif command -v open &> /dev/null; then
        open "https://claude.ai"
    else
        echo "Please open https://claude.ai manually"
    fi
fi

echo ""
echo "✅ When Claude generates the code:"
echo "   1. Copy the Rust code to: portfolio/rustml-sentiment-api/src/features/"
echo "   2. Copy the tests to: portfolio/rustml-sentiment-api/tests/"
echo "   3. Run: cargo test"
echo "   4. Commit when tests pass!"
echo ""
