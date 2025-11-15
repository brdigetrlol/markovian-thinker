#!/bin/bash

# Upwork Proposal Generator (Local)
# Uses your Claude subscription

echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║                                                               ║"
echo "║     💼 Upwork Proposal Generator (Local)                     ║"
echo "║                                                               ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""

echo "📋 Paste the Upwork job URL or description:"
read -p "> " JOB_INPUT

echo ""
echo "🔍 Analyzing your portfolio capabilities..."

# Analyze portfolio
RUST_PROJECTS=$(find portfolio -name "Cargo.toml" | wc -l)
TS_PROJECTS=$(find portfolio -name "package.json" | wc -l)

echo "   Rust projects: $RUST_PROJECTS"
echo "   TypeScript projects: $TS_PROJECTS"

# List portfolio projects
echo ""
echo "Your portfolio projects:"
for dir in portfolio/*/; do
    PROJECT=$(basename "$dir")
    if [ -f "$dir/README.md" ]; then
        DESC=$(head -n 5 "$dir/README.md" | tail -n 1)
        echo "   - $PROJECT: $DESC"
    fi
done

# Detect skills
SKILLS=""
[ $(find portfolio -name "*.rs" | wc -l) -gt 0 ] && SKILLS="$SKILLS Rust,"
[ $(find portfolio -name "*.ts" | wc -l) -gt 0 ] && SKILLS="$SKILLS TypeScript,"
grep -r "sentiment" portfolio/ &>/dev/null && SKILLS="$SKILLS Sentiment-Analysis,"
grep -r "ml\|machine" portfolio/ &>/dev/null && SKILLS="$SKILLS Machine-Learning,"
grep -r "three" portfolio/ &>/dev/null && SKILLS="$SKILLS 3D-Visualization,"

echo ""
echo "Detected skills: $SKILLS"
echo ""

# Generate Claude prompt
PROMPT="I need to write a winning Upwork proposal for this job:

**Job Details**:
$JOB_INPUT

**My Portfolio**:
- Rust Projects: $RUST_PROJECTS
- TypeScript Projects: $TS_PROJECTS
- Skills: $SKILLS

**Portfolio Projects**:
$(for dir in portfolio/*/; do
    PROJECT=$(basename "$dir")
    if [ -f "$dir/README.md" ]; then
        echo "- $PROJECT: $(head -n 5 "$dir/README.md" | tail -n 1)"
    fi
done)

Please write a winning proposal that:

1. **Personalized Opening** (2-3 sentences)
   - Reference specific details from their job posting
   - Show you actually read and understood it

2. **Proof of Skills**
   - Mention 1-2 relevant portfolio projects
   - Include a brief code sample or metric
   - Link to GitHub repository

3. **Specific Approach**
   - How you'll solve their problem
   - Timeline estimate (be realistic)
   - Key deliverables

4. **Pricing**
   - Competitive hourly rate or fixed price
   - Justify the value

5. **Strong CTA**
   - Availability
   - Next steps
   - Friendly tone

**Requirements**:
- 200-400 words (not too long!)
- Professional but friendly
- No generic templates
- Focus on THEIR needs, not your experience
- Include relevant GitHub link

Make it feel personal and conversion-optimized!"

echo "$PROMPT" > /tmp/proposal-prompt.txt

echo "═══════════════════════════════════════════════════════════════"
echo "📋 PROPOSAL PROMPT READY!"
echo "═══════════════════════════════════════════════════════════════"
echo ""
cat /tmp/proposal-prompt.txt
echo ""
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Next steps:"
echo "1. Copy prompt above"
echo "2. Go to https://claude.ai"
echo "3. Get your personalized proposal"
echo "4. Review and customize"
echo "5. Send on Upwork within 1 hour!"
echo ""
echo "💡 Pro tip: Claude can also:"
echo "   - Suggest pricing based on job budget"
echo "   - Find relevant code samples from your portfolio"
echo "   - Generate multiple versions for A/B testing"
echo ""
echo "Just ask Claude to do those things in follow-up messages!"
echo ""

# Save to file
mkdir -p proposals
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
cp /tmp/proposal-prompt.txt "proposals/prompt_$TIMESTAMP.txt"
echo "✅ Prompt saved to: proposals/prompt_$TIMESTAMP.txt"
