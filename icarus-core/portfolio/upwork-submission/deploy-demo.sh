#!/bin/bash

# Quick Deploy Script for Interactive Demo
# Deploys to GitHub Pages automatically

echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║     DEPLOY INTERACTIVE DEMO TO GITHUB PAGES                   ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""

# Check if we're in a git repo
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "❌ Not in a git repository!"
    exit 1
fi

echo "📦 Preparing deployment..."
echo ""

# Create a gh-pages branch
echo "1️⃣  Creating gh-pages branch..."
git checkout -b gh-pages 2>/dev/null || git checkout gh-pages

# Copy the interactive demo to root as index.html
echo "2️⃣  Copying INTERACTIVE-DEMO.html to index.html..."
cp portfolio/upwork-submission/INTERACTIVE-DEMO.html index.html

# Add and commit
echo "3️⃣  Committing deployment..."
git add index.html
git commit -m "Deploy interactive demo to GitHub Pages" 2>/dev/null || echo "   No changes to commit"

# Push to gh-pages
echo "4️⃣  Pushing to GitHub Pages..."
git push -u origin gh-pages 2>&1

echo ""
echo "✅ Deployment complete!"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  📍 YOUR LIVE DEMO URL"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "  https://brdigetrlol.github.io/icarus-core/"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "⏳ Note: It may take 1-2 minutes for GitHub Pages to publish."
echo ""
echo "📋 Next Steps:"
echo "  1. Wait 2 minutes"
echo "  2. Visit the URL above"
echo "  3. Add this URL to your Upwork portfolio!"
echo "  4. Clients can now try your demo live!"
echo ""

# Switch back to original branch
git checkout - 2>/dev/null
