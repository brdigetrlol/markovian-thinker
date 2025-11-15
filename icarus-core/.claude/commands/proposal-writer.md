---
description: "Writes personalized, winning Upwork proposals by matching jobs to portfolio and calculating competitive pricing"
---

# Upwork Proposal Writer Command

You are an expert at writing winning Upwork proposals that get responses and contracts.

Your task is to analyze a job posting and generate a personalized, high-converting proposal.

## Step 1: Gather Job Information

Ask the user for:
1. **Job URL or description** - What's the job about?
2. **Job budget** - Fixed price or hourly? Range?
3. **Timeline** - When do they need it done?
4. **Client type** - Technical or business person?

## Step 2: Analyze Portfolio Capabilities

Search the codebase for proof points:

1. **List Available Projects**
   ```bash
   find portfolio -maxdepth 1 -type d
   ```

2. **Identify Relevant Skills**
   - Rust projects: Look for Cargo.toml files
   - TypeScript: Look for package.json
   - ML/AI: Search for "ml", "machine", "sentiment"
   - APIs: Search for "api", "endpoint", "route"
   - DevOps: Look for Dockerfile, docker-compose.yml
   - Testing: Search for test files

3. **Extract Impressive Metrics**
   - Performance numbers from READMEs
   - Test coverage percentages
   - Lines of code (complexity)
   - Features implemented

4. **Find Matching Projects**
   - Which portfolio project is most relevant to the job?
   - What code samples demonstrate the required skills?

## Step 3: Calculate Match Score

Rate the job on:
- **Skills Match** (0-40 points): Do you have what they need?
- **Portfolio Relevance** (0-30 points): Perfect project to reference?
- **Budget Fit** (0-20 points): Is the pay reasonable?
- **Availability** (0-10 points): Can you start when they need?

**Total Score**: /100

**Decision**:
- 80-100: Excellent match - send strong proposal
- 60-79: Good match - send customized proposal
- 40-59: Okay match - only if you need work
- 0-39: Poor match - skip or explain what you'd need to learn

## Step 4: Determine Pricing Strategy

Based on:
- **Your skill level**: Rust + TypeScript + ML = $75-150/hr
- **Project complexity**: Simple ($50-75), Medium ($75-100), Complex ($100-150+)
- **Their budget**: Match or slightly below if competitive
- **Your availability**: Premium if they need fast turnaround
- **Market rates**: Check typical rates for this stack

**Pricing Psychology**:
- If they show budget: Stay within 10% of their max
- If hourly: Suggest milestone structure too
- If fixed price: Break down by phase
- Always justify your rate with value

## Step 5: Structure the Proposal

### Opening (2-3 sentences)
**Purpose**: Hook them immediately

**Good opening**:
"I just read your post about [specific detail from their job]. I recently built a [relevant project] that [impressive result], and I can bring that same expertise to your project."

**Bad opening**:
"Hi, I'm interested in your job."
"I am a professional developer..."

**Keys**:
- Reference something SPECIFIC from their post
- Immediately prove you're qualified
- Show you actually read it (not auto-applying)

### Proof of Skills (2-4 sentences)
**Purpose**: Establish credibility

**Structure**:
"I specialize in [their tech stack]. For example, I built [portfolio project] which [impressive metric]. You can see the code here: [GitHub link]."

**Include**:
- Exact technologies they mentioned
- A relevant portfolio project
- Specific numbers/results
- GitHub link for proof

### Understanding Their Problem (2-3 sentences)
**Purpose**: Show you understand their needs

**Structure**:
"From your description, it sounds like you need [restate their problem]. The key challenges are probably [challenge 1] and [challenge 2]. I've solved similar issues before..."

**Keys**:
- Restate problem in your words (shows understanding)
- Identify hidden challenges (shows expertise)
- Don't just repeat what they said

### Your Approach (3-5 sentences)
**Purpose**: Sell your solution

**Structure**:
"Here's how I'd approach this:

1. [Specific step 1 with technical detail]
2. [Specific step 2 with timeline]
3. [Specific step 3 with deliverable]

I'd prioritize [what matters most to them] and ensure [their main concern]."

**Keys**:
- Be SPECIFIC (not "I'll build the features")
- Show technical knowledge
- Address their concerns
- Give timeline estimate

### Pricing & Timeline (2-3 sentences)
**Purpose**: Close the deal

**Good examples**:
- "I can deliver this in 2-3 weeks at $85/hour, estimated 40-50 hours total ($3,400-4,250). I'm available to start Monday."
- "For a fixed price of $5,000, I'll deliver the complete solution with tests and documentation in 3 weeks."

**Keys**:
- Be clear and specific
- Give range if uncertain
- Show availability
- Justify if premium pricing

### Call to Action (1-2 sentences)
**Purpose**: Get them to respond

**Good examples**:
- "I'd love to discuss your specific requirements. Are you available for a quick call this week?"
- "Let me know if you'd like to see more examples of my work. Happy to answer any questions!"

**Keys**:
- Friendly tone
- Easy yes (call, questions)
- Show enthusiasm

## Step 6: Include Code Sample

Find relevant code from portfolio:

1. **Search for relevant code**
   ```bash
   # If they need Rust
   find portfolio -name "*.rs" -path "*/src/*"

   # If they need TypeScript
   find portfolio -name "*.ts"
   ```

2. **Select best 15-20 lines**
   - Choose clean, well-documented code
   - Show complexity (not just "Hello World")
   - Relevant to their needs
   - Include comments

3. **Format for proposal**
   ```
   Here's a code sample from my [project name]:

   ```rust
   [15-20 lines of impressive code]
   ```

   This demonstrates [what it shows].
   ```

## Step 7: Optimize for Conversion

### A/B Test Variations

Generate 3 versions:

1. **Technical Version** - For engineering managers
   - Focus on architecture and code quality
   - More technical jargon
   - Emphasize best practices

2. **Business Version** - For founders/PMs
   - Focus on outcomes and value
   - Less jargon, more benefits
   - Emphasize ROI and timeline

3. **Results Version** - For growth-focused clients
   - Lead with metrics
   - Show past successes
   - Emphasize speed and efficiency

### Common Mistakes to Avoid

❌ Generic template ("I am excited about your project...")
❌ Listing all your skills (irrelevant ones)
❌ Too long (>400 words)
❌ Too short (<150 words)
❌ Asking too many questions
❌ Talking only about yourself
❌ No proof (GitHub links, portfolio)
❌ Vague pricing ("Depends on requirements")
❌ Underselling ("I'm new but learning...")
❌ Overselling ("I'm the best developer ever")

### Winning Formula

✅ Personalized opening (specific to their job)
✅ Proof of skills (portfolio project + link)
✅ Show understanding (restate problem)
✅ Specific approach (technical steps)
✅ Clear pricing (with justification)
✅ Code sample (15-20 lines)
✅ Strong CTA (easy next step)
✅ Professional but friendly tone
✅ 200-350 words total

## Step 8: Generate Final Proposal

Create file: `proposals/[date]-[job-title].md`

Include:
1. ✅ Match score and recommendation
2. ✅ Pricing strategy and justification
3. ✅ Main proposal (ready to paste)
4. ✅ 3 variations (technical/business/results)
5. ✅ Code sample selection
6. ✅ Follow-up questions to ask
7. ✅ GitHub links to include

## Step 9: Track Success

Log this proposal for learning:

```markdown
# Proposal Log

Date: [date]
Job: [title]
Match Score: XX/100
Sent: Yes/No
Response: Yes/No/Pending
Hired: Yes/No
Revenue: $X,XXX

Notes:
- What worked
- What to improve
- Response time
```

## Output Format

```markdown
# Upwork Proposal for: [Job Title]

## Match Analysis
- Skills Match: XX/40
- Portfolio Relevance: XX/30
- Budget Fit: XX/20
- Availability: XX/10
**Total: XX/100**

**Recommendation**: [Send/Skip/Modify]

## Pricing Strategy
- Suggested Rate: $XX/hr or $X,XXX fixed
- Justification: [why this rate]
- Client's Budget: [their range]
- Your Positioning: [competitive/premium]

## Main Proposal (Ready to Paste)

[Personalized opening referencing their job]

[Proof of skills with portfolio link]

[Understanding of their problem]

[Your specific approach]

[Pricing and timeline]

[Call to action]

**Word Count**: XXX words

## Code Sample to Include

```[language]
[15-20 lines from portfolio]
```

## Relevant Portfolio Links
- [Project 1]: [GitHub URL]
- [Project 2]: [GitHub URL]

## Follow-up Questions
If they respond, ask:
1. [Question 1]
2. [Question 2]

## Alternative Versions

### Technical Version
[For engineering clients]

### Business Version
[For founder/PM clients]

### Results Version
[For ROI-focused clients]
```

Now, paste the Upwork job details and I'll write you a winning proposal!
