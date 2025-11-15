#!/bin/bash

echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║                                                               ║"
echo "║     🧠 Building Sentiment Intelligence Platform             ║"
echo "║                                                               ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo "❌ Node.js is not installed!"
    echo "Please install Node.js: https://nodejs.org/"
    exit 1
fi

# Check if npm is installed
if ! command -v npm &> /dev/null; then
    echo "❌ npm is not installed!"
    echo "Please install npm (comes with Node.js)"
    exit 1
fi

echo "📦 Installing dependencies..."
npm install

echo ""
echo "🔨 Compiling TypeScript..."
npm run build

if [ $? -eq 0 ]; then
    echo ""
    echo "╔═══════════════════════════════════════════════════════════════╗"
    echo "║                                                               ║"
    echo "║  ✅ BUILD SUCCESSFUL!                                        ║"
    echo "║                                                               ║"
    echo "╚═══════════════════════════════════════════════════════════════╝"
    echo ""
    echo "🚀 To run the application:"
    echo ""
    echo "   Option 1 (Local server):"
    echo "   npm run serve"
    echo "   Then open: http://localhost:8080"
    echo ""
    echo "   Option 2 (With backend):"
    echo "   cd ../rustml-sentiment-api/demo"
    echo "   ./run-demo.sh"
    echo ""
else
    echo ""
    echo "❌ Build failed! Check errors above."
    exit 1
fi
