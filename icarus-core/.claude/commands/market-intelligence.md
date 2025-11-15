---
description: "Analyzes market trends, identifies skill gaps, and recommends next portfolio projects with ROI scoring"
---

# Market Intelligence & Project Recommender Command

You are a market research analyst specializing in freelance developer opportunities and portfolio optimization.

Your task is to analyze market trends, identify high-value opportunities, and recommend the next portfolio project to build.

## Step 1: Analyze Current Portfolio

### Inventory What You Have

1. **List All Projects**
   ```bash
   find portfolio -maxdepth 1 -type d -not -path portfolio
   ```

2. **For Each Project, Extract**:
   - Primary technology (Rust, TypeScript, Python, etc.)
   - Domain (ML, visualization, API, SaaS, etc.)
   - Complexity (lines of code, features)
   - State (complete, in-progress, idea)
   - Marketability (0-10 score)

3. **Identify Skill Coverage**:
   - Languages: Rust, TypeScript, Python, Go, etc.
   - Domains: ML/AI, Web3, SaaS, DevOps, Mobile, etc.
   - Frameworks: Actix, Axum, React, Vue, Next.js, etc.
   - Tools: Docker, K8s, GitHub Actions, AWS, etc.

### Calculate Portfolio Strength

For each skill area, rate 0-10:
- **Backend**: Rust score
- **Frontend**: TypeScript/React score
- **ML/AI**: Machine learning score
- **DevOps**: Infrastructure score
- **Mobile**: Mobile development score
- **Web3**: Blockchain score
- **Real-time**: WebSocket/streaming score
- **Visualization**: 3D/charts score

**Total Portfolio Score**: /80

## Step 2: Research Market Demand

### Analyze Job Boards (Manual Research Required)

Since we can't scrape live, provide research framework:

```markdown
## Manual Research Checklist

Visit these sites and record data:

### Upwork (upwork.com)
1. Search: "rust developer" → Count: ___ jobs
2. Search: "typescript developer" → Count: ___ jobs
3. Search: "machine learning" → Count: ___ jobs
4. Search: "blockchain rust" → Count: ___ jobs
5. Search: "saas development" → Count: ___ jobs

Average rates:
- Entry: $___/hr
- Mid: $___/hr
- Senior: $___/hr

### Freelancer.com
[Same searches]

### Toptal
- Required skill level
- Typical project sizes
- Success rates

### LinkedIn Jobs
- Remote Rust jobs: ___ count
- Salary ranges: $___ - $___

### GitHub Jobs (if available)
- Open source opportunities
- Sponsored projects
```

### Analyze Tech Trends

Research framework:

```markdown
## Tech Trend Analysis

### GitHub Trending
Visit: github.com/trending

Rust repos (this week):
1. [Repo name] - [What it does] - [Stars]
2. ...

TypeScript repos:
1. [Repo name] - [What it does] - [Stars]
2. ...

Machine Learning repos:
1. [Repo name] - [What it does] - [Stars]
2. ...

**Patterns Observed**:
- What's hot?
- What's declining?
- Emerging technologies?

### Hacker News
Visit: news.ycombinator.com

Top discussions:
1. [Topic] - [Why it matters]
2. ...

**Key Takeaways**:
- What problems need solving?
- What tech is hyped?

### Dev.to / Reddit r/programming
Hot topics:
1. ...
2. ...
```

## Step 3: Identify Skill Gaps

### Gap Analysis

```markdown
## High-Demand Skills I DON'T Have

Compare market research to portfolio:

| Skill | Upwork Jobs | Current Level | Gap | Priority |
|-------|-------------|---------------|-----|----------|
| Web3/Blockchain | 890 | 0/10 | HIGH | High |
| Mobile (React Native) | 3200 | 0/10 | HIGH | Medium |
| GraphQL | 1200 | 2/10 | Medium | High |
| Kubernetes | 800 | 1/10 | Medium | Medium |
| Real-time (WebRTC) | 650 | 3/10 | Low | High |

**Priority Calculation**:
Priority = (Job Count × Hourly Rate × (10 - Current Level)) / Learning Time
```

### Opportunity Scoring

For each potential project idea:

**Score = (Demand × Rate × Uniqueness) / Build Time**

Where:
- **Demand**: Job count on Upwork (0-5000)
- **Rate**: Average hourly rate ($50-150)
- **Uniqueness**: How rare this combination is (1-10)
- **Build Time**: Hours to complete (20-200)

## Step 4: Generate Project Ideas

### Template for Each Idea:

```markdown
### Project Idea: [Name]

**Elevator Pitch**: [One sentence description]

**Problem It Solves**:
- [Business problem]
- [Who has this problem]
- [Current solutions and their flaws]

**Technical Stack**:
- Backend: [tech]
- Frontend: [tech]
- Infrastructure: [tech]
- Novel/Trendy tech: [what's hot]

**Key Features** (5-7):
1. [Feature] - [Why impressive]
2. ...

**Market Analysis**:
- Upwork jobs matching this: ~XXX
- Average hourly rate: $XX-XX
- Competition level: Low/Medium/High
- Your edge: [What makes you different]

**Skill Development**:
- New skills learned: [list]
- Portfolio gaps filled: [which gaps]
- Career advancement: [how it helps]

**Build Estimate**:
- Time required: XX-XX hours
- Complexity: Low/Medium/High
- MVP timeline: X weeks

**ROI Score**: XXX
(Calculated: (Jobs × Rate × Uniqueness) / Hours)

**Expected Outcomes**:
- Upwork job opportunities: +XX per month
- Rate increase potential: +$XX/hr
- Career positioning: [narrative]
```

## Step 5: Rank & Recommend

### Ranking Criteria:

1. **ROI Score** (40%): Highest income potential per time invested
2. **Skill Gap** (30%): Fills missing portfolio capabilities
3. **Build Difficulty** (20%): Achievable given current skills
4. **Market Timing** (10%): Trending now vs. stable demand

### Generate Top 5 Recommendations:

```markdown
## Top 5 Portfolio Projects to Build

### 🥇 #1: [Project Name]
- **ROI Score**: XXX (Top 1%)
- **Why Build This**: [Compelling reason]
- **Time to Build**: XX hours
- **Expected Impact**: [Specific outcomes]
- **Start Priority**: IMMEDIATE

### 🥈 #2: [Project Name]
...

### 🥉 #3: [Project Name]
...

### 4: [Project Name]
...

### 5: [Project Name]
...
```

## Step 6: Create Implementation Roadmap

For the #1 recommended project:

```markdown
## Implementation Roadmap: [Project Name]

### Phase 1: Foundation (Week 1)
**Goal**: Basic project structure and core functionality

Tasks:
- [ ] Initialize Rust workspace / TypeScript project
- [ ] Set up dependencies and tooling
- [ ] Implement core data structures
- [ ] Basic CLI/API interface
- [ ] Write initial tests

**Deliverable**: Working MVP with 1-2 features

### Phase 2: Feature Development (Week 2-3)
**Goal**: Implement all key features

Tasks:
- [ ] Feature 1: [details]
- [ ] Feature 2: [details]
- [ ] Feature 3: [details]
- [ ] Comprehensive testing
- [ ] Error handling

**Deliverable**: Feature-complete application

### Phase 3: Polish & Documentation (Week 4)
**Goal**: Production-ready portfolio piece

Tasks:
- [ ] Performance optimization
- [ ] UI/UX refinement (if applicable)
- [ ] Comprehensive README
- [ ] API documentation
- [ ] Demo video/GIF
- [ ] Docker deployment

**Deliverable**: Portfolio-ready project

### Phase 4: Portfolio Integration (Week 4)
**Goal**: Market it effectively

Tasks:
- [ ] Create Upwork portfolio entry
- [ ] Write 3 proposal templates
- [ ] Take professional screenshots
- [ ] Deploy live demo (if applicable)
- [ ] Share on LinkedIn/Twitter

**Deliverable**: Ready to win clients
```

## Step 7: Provide Actionable Next Steps

```markdown
## Immediate Action Items

### This Week:
1. [ ] Decide on top project (recommend: [#1])
2. [ ] Create GitHub repository
3. [ ] Set up basic project structure
4. [ ] Implement first core feature

### This Month:
1. [ ] Complete MVP
2. [ ] Add comprehensive tests
3. [ ] Write documentation
4. [ ] Create Upwork portfolio entry

### This Quarter:
1. [ ] Build project #2
2. [ ] Apply to 20+ relevant Upwork jobs
3. [ ] Track response rates
4. [ ] Iterate based on feedback

## Success Metrics

Track these monthly:
- New portfolio projects: X
- Upwork profile views: X (+Y%)
- Job proposals sent: X
- Response rate: X%
- Contracts won: X
- Revenue: $X,XXX
- Average hourly rate: $XX/hr
```

## Step 8: Generate Market Intelligence Report

Create file: `research/market-intelligence-[date].md`

```markdown
# Market Intelligence Report
Date: [date]

## Executive Summary
[3-5 sentences: What you found, top recommendation, why now]

## Current Portfolio Analysis
[Strengths, weaknesses, gaps]

## Market Research
[Demand data, trends, opportunities]

## Project Recommendations
[Top 5 with scores and rationale]

## Implementation Roadmap
[Detailed plan for #1 project]

## Risk Assessment
[What could go wrong, mitigation]

## Competitive Positioning
[How these projects differentiate you]

## Next Steps
[Actionable tasks for next 30/60/90 days]
```

Now, let me analyze your portfolio and recommend your next high-ROI project!
