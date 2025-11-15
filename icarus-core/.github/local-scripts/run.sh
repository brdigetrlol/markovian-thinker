#!/bin/bash

# Master script for local automation
# Uses YOUR Claude subscription (no API costs!)

echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║                                                               ║"
echo "║     🚀 Local AI Automation Suite                             ║"
echo "║     Powered by YOUR Claude subscription                      ║"
echo "║                                                               ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""
echo "💡 These scripts generate prompts for Claude.ai"
echo "   No API costs - uses your existing subscription!"
echo ""
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Select an automation:"
echo ""
echo "1) 🤖 AI Feature Generator"
echo "   → Generate complete feature implementations"
echo ""
echo "2) 📸 Upwork Portfolio Content"
echo "   → Create portfolio entries for your projects"
echo ""
echo "3) 💼 Upwork Proposal Generator"
echo "   → Write winning proposals for jobs"
echo ""
echo "4) ⚡ Performance Analysis"
echo "   → Get optimization suggestions"
echo ""
echo "5) 🔍 Market Research"
echo "   → Find trending project opportunities"
echo ""
echo "6) 📚 Documentation Generator"
echo "   → Create README and API docs"
echo ""
echo "0) Exit"
echo ""
read -p "Enter choice (1-6): " choice

case $choice in
    1)
        echo ""
        ./.github/local-scripts/ai-feature-generator.sh
        ;;
    2)
        echo ""
        ./.github/local-scripts/generate-upwork-content.sh
        ;;
    3)
        echo ""
        ./.github/local-scripts/generate-proposal.sh
        ;;
    4)
        echo ""
        echo "⚡ Performance Analysis"
        echo "Running benchmarks..."
        cd portfolio/rustml-sentiment-api
        cargo bench 2>&1 | tee bench_results.txt

        echo ""
        echo "📋 Results saved to: bench_results.txt"
        echo ""
        echo "Copy these results to Claude.ai and ask:"
        echo "\"Analyze these benchmark results and suggest optimizations\""
        ;;
    5)
        echo ""
        echo "🔍 Market Research"
        echo ""
        echo "Generating market research prompt..."

        PROMPT="I'm a freelance developer with these skills:

**Portfolio**:
$(for dir in portfolio/*/; do echo "- $(basename "$dir")"; done)

**Tech Stack**:
- Rust (backend, async, ML)
- TypeScript (frontend, 3D viz)
- Docker, GitHub Actions

Please research and provide:

1. **Trending Technologies** on GitHub (Rust/TypeScript/ML)
2. **Upwork Job Demand** for my skill combinations
3. **Project Ideas** that would:
   - Be in high demand on Upwork
   - Leverage my existing skills
   - Take 20-40 hours to build
   - Have strong ROI (jobs × rate / time)

4. **Suggested Next Project** with:
   - Why it's marketable
   - Expected Upwork job count
   - Suggested hourly rate
   - Implementation roadmap

Make it data-driven and actionable!"

        echo "$PROMPT" > /tmp/market-research-prompt.txt
        cat /tmp/market-research-prompt.txt
        echo ""
        echo "═══════════════════════════════════════════════════════════"
        echo "📋 Copy this to Claude.ai for market insights!"
        ;;
    6)
        echo ""
        echo "📚 Documentation Generator"
        echo ""
        read -p "Enter project path (e.g., portfolio/rustml-sentiment-api): " PROJECT_PATH

        if [ ! -d "$PROJECT_PATH" ]; then
            echo "❌ Project not found!"
            exit 1
        fi

        echo ""
        echo "Analyzing project..."

        PROMPT="Generate comprehensive documentation for this project:

**Project Structure**:
$(find "$PROJECT_PATH" -type f -name "*.rs" -o -name "*.ts" | head -20)

**Code Sample**:
$(find "$PROJECT_PATH" -name "*.rs" -type f | head -1 | xargs head -50)

Please create:

1. **README.md** with:
   - Project overview
   - Features list
   - Quick start guide
   - API documentation
   - Usage examples
   - Contributing guidelines

2. **API.md** with:
   - All endpoints
   - Request/response examples
   - Error codes
   - Rate limits

3. **ARCHITECTURE.md** with:
   - System design
   - Component diagram
   - Data flow
   - Tech stack details

Make it professional and comprehensive!"

        echo "$PROMPT" > /tmp/docs-prompt.txt
        cat /tmp/docs-prompt.txt
        echo ""
        echo "═══════════════════════════════════════════════════════════"
        echo "📋 Copy this to Claude.ai to generate docs!"
        ;;
    0)
        echo "👋 Goodbye!"
        exit 0
        ;;
    *)
        echo "❌ Invalid choice!"
        exit 1
        ;;
esac

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "✅ Done! Remember:"
echo "   • Copy prompts to https://claude.ai"
echo "   • Your subscription = unlimited usage"
echo "   • No API costs!"
echo ""
echo "💡 Pro tip: Save Claude's responses to files for reuse"
echo ""
