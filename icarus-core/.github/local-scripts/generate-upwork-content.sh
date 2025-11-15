#!/bin/bash

# Upwork Portfolio Content Generator (Local)
# Uses your Claude subscription

echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║                                                               ║"
echo "║     📸 Upwork Portfolio Generator (Local)                    ║"
echo "║                                                               ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""

echo "📋 Select project to showcase:"
echo ""
PS3="Enter number: "
select PROJECT in portfolio/*/ ; do
    if [ -n "$PROJECT" ]; then
        PROJECT_NAME=$(basename "$PROJECT")
        break
    fi
done

echo ""
echo "✅ Selected: $PROJECT_NAME"
echo ""

# Analyze project
README_CONTENT=$(cat "$PROJECT/README.md" 2>/dev/null || echo "No README found")
FILE_COUNT=$(find "$PROJECT" -type f | wc -l)
CODE_FILES=$(find "$PROJECT" -name "*.rs" -o -name "*.ts" | wc -l)

# Generate Claude prompt
PROMPT="I need to create Upwork portfolio content for my project:

**Project**: $PROJECT_NAME
**Files**: $FILE_COUNT total, $CODE_FILES code files

**Project README**:
$README_CONTENT

Please generate Upwork portfolio content:

1. **Project Title** (60 characters max)
   - Catchy, professional, includes tech stack

2. **Project Description** (under 600 characters)
   - Focus on business value and results
   - Include impressive metrics
   - Mention technologies
   - Client-focused language

3. **Your Role** (100 characters max)
   - Solo developer or team lead
   - Key responsibilities

4. **Skills** (list 8-10)
   - Technologies used
   - Marketable skills

5. **Key Achievements** (5 bullet points)
   - Quantified results
   - Technical accomplishments
   - Business impact

6. **Suggested Pricing**
   - Project value range (\$2K-\$10K)
   - Justification

7. **Target Client Types**
   - Who would hire for this skill set

Make it compelling and conversion-optimized!"

echo "$PROMPT" > /tmp/upwork-prompt.txt

echo "═══════════════════════════════════════════════════════════════"
echo "📋 PROMPT READY FOR CLAUDE!"
echo "═══════════════════════════════════════════════════════════════"
echo ""
cat /tmp/upwork-prompt.txt
echo ""
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Next steps:"
echo "1. Copy prompt above"
echo "2. Go to https://claude.ai"
echo "3. Paste and get your Upwork content!"
echo "4. Copy results to: portfolio/upwork-submission/"
echo ""

# Also take screenshots locally
echo "📸 Want to take screenshots now? (y/n)"
read -p "> " TAKE_SCREENSHOTS

if [ "$TAKE_SCREENSHOTS" = "y" ]; then
    echo ""
    echo "Starting local server..."
    cd "$PROJECT"

    # Try different ways to serve
    if [ -f "deploy.sh" ]; then
        ./deploy.sh
    elif [ -f "package.json" ]; then
        npm install && npm run serve &
    elif command -v python3 &> /dev/null; then
        python3 -m http.server 8080 &
    fi

    SERVER_PID=$!

    echo "Server started! Opening http://localhost:8080"
    echo ""
    echo "Take screenshots with:"
    echo "  - Windows: Win + Shift + S"
    echo "  - Mac: Cmd + Shift + 4"
    echo "  - Linux: Screenshot app"
    echo ""
    echo "Press Enter when done..."
    read

    kill $SERVER_PID 2>/dev/null
fi

echo ""
echo "✅ Done! Your Upwork content is ready to use."
