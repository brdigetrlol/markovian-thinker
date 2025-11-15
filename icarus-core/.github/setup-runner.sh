#!/bin/bash

echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║                                                               ║"
echo "║     🏃 GitHub Self-Hosted Runner Setup                       ║"
echo "║                                                               ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""

# Check if running on Linux
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    echo "❌ This script is for Linux. For Windows/Mac, see GitHub docs."
    exit 1
fi

# Create a folder for the runner
mkdir -p ~/actions-runner && cd ~/actions-runner

echo "📦 Downloading GitHub Actions Runner..."

# Download the latest runner package
curl -o actions-runner-linux-x64-2.311.0.tar.gz -L https://github.com/actions/runner/releases/download/v2.311.0/actions-runner-linux-x64-2.311.0.tar.gz

# Extract the installer
tar xzf ./actions-runner-linux-x64-2.311.0.tar.gz

echo ""
echo "✅ Runner downloaded and extracted!"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  NEXT STEPS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "1. Go to your GitHub repository:"
echo "   https://github.com/YOUR_USERNAME/YOUR_REPO/settings/actions/runners/new"
echo ""
echo "2. Copy the token from the page"
echo ""
echo "3. Run the configuration:"
echo "   ./config.sh --url https://github.com/YOUR_USERNAME/YOUR_REPO --token YOUR_TOKEN"
echo ""
echo "4. Start the runner:"
echo "   ./run.sh"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Optional: Install as a service
echo "To install as a service (runs on boot):"
echo "  sudo ./svc.sh install"
echo "  sudo ./svc.sh start"
echo ""
