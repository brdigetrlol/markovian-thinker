# Icarus Knowledge Distillation System - Usage Guide

**Version**: 1.0
**Date**: 2025-11-15

## Overview

The Icarus Knowledge Distillation System enables Icarus to learn from Claude's problem-solving approaches through strategy extraction. Instead of model distillation (compressing a larger model into a smaller one), this implements **knowledge distillation** - learning reusable problem-solving patterns.

## Core Concept

```
Problem → Claude's Reasoning → Extract Strategy → Store as Skill → Reuse on Similar Problems
```

## Architecture

### 1. Skill Structure

```rust
pub struct Skill {
    pub id: String,                    // UUID
    pub name: String,                  // e.g., "Debug Type Errors"
    pub domain: SkillDomain,          // Classification
    pub pattern: String,               // When to apply
    pub steps: Vec<SkillStep>,        // How to execute
    pub heuristics: Vec<String>,      // Decision rules
    pub success_rate: f64,            // Performance (0.0-1.0)
    pub application_count: u32,       // Usage frequency
    pub learned_at: DateTime<Utc>,    // Timestamp
    pub source: String,               // Origin
}
```

### 2. Skill Domains

- **Debugging**: Finding and fixing errors
- **Refactoring**: Code improvement and cleanup
- **Architecture**: System design decisions
- **Testing**: Test creation and validation
- **Performance**: Optimization strategies
- **Documentation**: Writing clear docs
- **CodeReview**: Review heuristics
- **ProblemDecomposition**: Breaking down complex tasks
- **General**: Uncategorized patterns

### 3. SkillLibrary

The library provides:
- **Storage**: HashMap-based skill storage
- **Indexing**: Domain-based categorization
- **Retrieval**: Pattern matching for similar problems
- **Statistics**: Success rates and usage metrics

## Using the MCP Tool

### Tool: `icarus_learn_from_interaction`

**Purpose**: Teach Icarus by demonstrating problem-solving

**Parameters**:
```json
{
  "problem": "string (required)",
  "reasoning": ["array", "of", "strings (required)"],
  "solution": "string (required)",
  "context": {
    "optional": "metadata"
  }
}
```

### Example 1: Debugging Session

```json
{
  "problem": "TypeScript compilation error: Type 'User' is not assignable to type 'AuthUser'",
  "reasoning": [
    "Read the file to understand the type definitions",
    "Identified that User lacks the 'roles' property required by AuthUser",
    "Checked usages to ensure adding 'roles' won't break existing code",
    "Added 'roles?: string[]' to User interface",
    "Ran tsc to verify the fix"
  ],
  "solution": "Extended User interface with optional roles property to satisfy AuthUser requirements",
  "context": {
    "domain": "debugging",
    "file": "src/types/user.ts",
    "error_type": "type_mismatch"
  }
}
```

**Extracted Skill**:
- **Name**: "Resolve TypeScript Type Incompatibility"
- **Domain**: Debugging
- **Pattern**: "type not assignable", "TypeScript", "interface"
- **Steps**:
  1. Read type definitions
  2. Identify missing/conflicting properties
  3. Check usage impact
  4. Extend/modify types
  5. Verify compilation

### Example 2: Refactoring

```json
{
  "problem": "Function 'processUserData' has grown to 150 lines with multiple responsibilities",
  "reasoning": [
    "Identified three distinct concerns: validation, transformation, storage",
    "Extract validation logic into validateUserData()",
    "Extract transformation into transformUserData()",
    "Extract storage into saveUserData()",
    "Compose these in processUserData() for clear flow"
  ],
  "solution": "Broke monolithic function into four smaller, focused functions following Single Responsibility Principle",
  "context": {
    "domain": "refactoring",
    "pattern": "extract_function",
    "principle": "SRP"
  }
}
```

**Extracted Skill**:
- **Name**: "Decompose Large Function by Responsibility"
- **Domain**: Refactoring
- **Pattern**: "large function", "multiple responsibilities"
- **Heuristics**:
  - If function > 50 lines, check for multiple concerns
  - Group by responsibility (validation, transformation, I/O)
  - Extract into cohesive functions
  - Maintain clear data flow

### Example 3: Architecture Decision

```json
{
  "problem": "Need to choose between REST and GraphQL for new API",
  "reasoning": [
    "Analyzed client requirements: mobile app needs flexible queries",
    "Considered team expertise: backend team knows REST well",
    "Evaluated data patterns: nested resources with varying depth",
    "Weighed tradeoffs: GraphQL fits data needs but requires learning",
    "Decision: GraphQL for long-term benefits, plan training period"
  ],
  "solution": "Chose GraphQL with phased rollout and team training plan",
  "context": {
    "domain": "architecture",
    "decision_type": "technology_selection"
  }
}
```

**Extracted Skill**:
- **Name**: "API Technology Selection Framework"
- **Domain**: Architecture
- **Steps**:
  1. Analyze client data access patterns
  2. Assess team expertise
  3. Evaluate long-term maintainability
  4. Create adoption plan
- **Heuristics**:
  - Complex nested data → Consider GraphQL
  - Simple CRUD → REST sufficient
  - Factor in team learning curve

## Response Format

### Success Response

```json
{
  "success": true,
  "message": "Successfully learned from interaction",
  "skill": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Resolve TypeScript Type Incompatibility",
    "domain": "Debugging"
  },
  "library_stats": {
    "total_skills": 15,
    "avg_success_rate": 0.73,
    "domain_counts": {
      "Debugging": 5,
      "Refactoring": 4,
      "Architecture": 3,
      "Testing": 2,
      "General": 1
    }
  }
}
```

### Failure Response

```json
{
  "success": false,
  "message": "Could not extract a meaningful skill from this interaction. Try providing more detailed reasoning steps or a clearer problem statement."
}
```

## Best Practices

### 1. Clear Problem Statements
- Be specific about the issue
- Include relevant context (error messages, symptoms)
- Mention constraints or requirements

### 2. Detailed Reasoning
- Provide 3-7 reasoning steps
- Explain *why* each step was taken
- Include decision points and alternatives considered

### 3. Concrete Solutions
- Describe what was done
- Explain the outcome
- Note any tradeoffs

### 4. Rich Context
- Add domain tags
- Include file paths or system components
- Note patterns or principles applied

## Strategy Extraction Process

The `StrategyExtractor` analyzes interactions to create skills:

1. **Domain Classification**
   - Analyzes problem description for keywords
   - Maps to skill domain (Debugging, Refactoring, etc.)

2. **Pattern Extraction**
   - Identifies key phrases that indicate when skill applies
   - Creates searchable pattern string

3. **Step Generalization**
   - Converts reasoning steps into reusable actions
   - Parameterizes specifics (file names, variable names)

4. **Heuristic Identification**
   - Extracts decision rules from reasoning
   - Creates guidelines for similar problems

## Skill Evolution

### Success Tracking

```rust
// Exponential Moving Average (α = 0.2)
success_rate = 0.2 * (new_result) + 0.8 * (old_success_rate)
```

- Recent outcomes weighted more heavily
- Gradual adaptation to skill effectiveness
- Balances stability with responsiveness

### Continuous Improvement

1. **Apply skill** to similar problem
2. **Measure outcome** (success/failure)
3. **Update success_rate** via EMA
4. **Increment application_count**
5. **Refine heuristics** based on edge cases

## Integration with Other Systems

### With markovian-thinker
- Use markovian reasoning to validate skill applicability
- Chunk-based analysis for complex problem decomposition
- SessionManager for multi-step skill execution

### With H²CE
- Semantic search for similar past problems
- Vector similarity for skill matching
- Index skill descriptions for fast retrieval

### With Icarus Agents
- **Learning Agent**: Manages skill extraction and improvement
- **Planning Agent**: Selects skills for problem-solving
- **Memory Agent**: Stores interactions and outcomes
- **Action Agent**: Executes skill steps

## Future Enhancements

1. **Skill Composition**
   - Combine multiple skills for complex problems
   - Create meta-skills from successful compositions

2. **Validation Framework**
   - Automated testing of learned skills
   - Simulation environments for safe experimentation

3. **Skill Pruning**
   - Remove low-performing skills (success_rate < 0.3)
   - Merge similar/redundant skills

4. **Transfer Learning**
   - Apply skills from one domain to another
   - Generalize patterns across contexts

5. **Skill Persistence**
   - Save library to disk
   - Version control for skill evolution
   - Export/import skill sets

## Troubleshooting

### Skill Not Extracted

**Symptoms**: Returns "Could not extract meaningful skill"

**Causes**:
- Reasoning steps too vague
- Problem description unclear
- Missing context

**Solutions**:
- Add 2-3 more reasoning steps
- Be more specific in problem statement
- Include context metadata

### Low Success Rates

**Symptoms**: Skills consistently fail when applied

**Causes**:
- Skill too specific to original problem
- Pattern matching too broad
- Heuristics incomplete

**Solutions**:
- Review and refine skill steps
- Narrow pattern to reduce false matches
- Add edge case handling to heuristics

### Library Growing Too Large

**Symptoms**: Thousands of skills, slow retrieval

**Solutions**:
- Implement skill pruning (success_rate threshold)
- Merge similar skills
- Create skill hierarchies

## Example Workflow

### Teaching Icarus to Debug

**Session 1: Type Errors**
```bash
# Demonstrate type error fix
icarus_learn_from_interaction(
  problem="Type mismatch in API response",
  reasoning=[...],
  solution="Added type guards"
)
# → Skill: "Handle API Type Mismatches" created
```

**Session 2: Runtime Errors**
```bash
# Demonstrate null handling
icarus_learn_from_interaction(
  problem="Null pointer exception in user profile",
  reasoning=[...],
  solution="Added null checks and defaults"
)
# → Skill: "Defensive Null Handling" created
```

**Session 3: Logic Errors**
```bash
# Demonstrate off-by-one fix
icarus_learn_from_interaction(
  problem="Loop iterating one too many times",
  reasoning=[...],
  solution="Changed < to <= in loop condition"
)
# → Skill: "Fix Off-By-One Errors" created
```

**Result**: Icarus now has 3 debugging skills and can recognize similar patterns in future code.

## Metrics and Monitoring

### Library Statistics

```rust
library.get_statistics().await
```

Returns:
- Total skills
- Average success rate
- Skills per domain
- Most/least used skills

### Individual Skill Performance

```rust
library.get_skill(skill_id).await
```

Check:
- `success_rate`: Current effectiveness
- `application_count`: How often used
- `learned_at`: Age of skill

## Conclusion

The Knowledge Distillation System transforms Claude's expertise into reusable, evolvable skills. By capturing problem-solving patterns and tracking their effectiveness, Icarus can continuously improve its capabilities through structured learning.

**Next Steps**:
1. Start teaching Icarus with real examples
2. Monitor skill performance
3. Refine extraction heuristics
4. Build domain-specific skill libraries
5. Integrate with agent cognitive loops
