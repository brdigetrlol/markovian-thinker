#!/bin/bash

echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║                                                               ║"
echo "║     🦀 RustML Sentiment API - LOCAL DEMO                     ║"
echo "║                                                               ║"
echo "║     Running Rust API locally (no Docker required)            ║"
echo "║                                                               ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo is not installed!"
    echo "Please install Rust: https://rustup.rs/"
    exit 1
fi

# Navigate to the project root
cd "$(dirname "$0")/.." || exit 1

echo "🔧 Building the Rust API..."
echo ""

# Build in release mode
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi

echo ""
echo "✅ Build successful!"
echo ""
echo "🚀 Starting the API server..."
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  API will be available at:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "  👉 http://localhost:3000"
echo ""
echo "  Health:  http://localhost:3000/health"
echo "  Stats:   http://localhost:3000/stats"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  TO VIEW THE DEMO:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "  1. Keep this terminal open"
echo "  2. Open demo/index.html in your browser"
echo "  3. Start analyzing text!"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Run the API
cargo run --release
