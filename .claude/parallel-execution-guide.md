# Maximal Parallel Execution Guide

## Core Principle: Default to Parallel

**FUNDAMENTAL RULE**: If tasks can be done independently, they MUST be done in parallel. Sequential execution is the exception, not the default.

---

## 1. Parallelization Decision Tree

### When to Parallelize (Default: YES)

```
Work identified →
  ├─ Tasks have data dependencies?
  │  └─ NO → ✅ PARALLELIZE (use parallel-subagent-spawner)
  │  └─ YES → Can some be parallelized?
  │     ├─ YES → ✅ PARALLELIZE independent subset
  │     └─ NO → Sequential execution required
  │
  ├─ Only 1-2 tasks?
  │  └─ Parallelization overhead > benefit → Sequential
  │
  └─ 3+ independent tasks?
     └─ ✅ ALWAYS PARALLELIZE
```

### Sequential Execution (Exceptions Only)

**ONLY execute sequentially when**:
- Task B requires output from Task A (data dependency)
- Task B modifies state that Task A reads (race condition risk)
- Only 1-2 very quick tasks (overhead > benefit)
- Explicit user request for sequential execution

**ALL other cases → PARALLELIZE**

---

## 2. Common Parallel Patterns

### Pattern 1: Multi-File Analysis

**Scenario**: Need to analyze multiple files

**Sequential (WRONG)**:
```
❌ Read file1 → analyze → Read file2 → analyze → Read file3 → analyze
```

**Parallel (CORRECT)**:
```
✅ spawn_parallel([
  "Read and analyze file1",
  "Read and analyze file2",
  "Read and analyze file3"
])
```

**MCP Implementation**:
```javascript
mcp__parallel-subagent-spawner__spawn_parallel(
  tasks=[
    "Read /path/to/file1.rs and analyze for performance bottlenecks",
    "Read /path/to/file2.rs and analyze for performance bottlenecks",
    "Read /path/to/file3.rs and analyze for performance bottlenecks"
  ],
  priority="high"
)
```

### Pattern 2: Concurrent Testing

**Scenario**: Run tests across different modules

**Sequential (WRONG)**:
```
❌ cargo test --package module1
   cargo test --package module2
   cargo test --package module3
```

**Parallel (CORRECT)**:
```
✅ spawn_parallel([
  "Run cargo test --package module1 and report results",
  "Run cargo test --package module2 and report results",
  "Run cargo test --package module3 and report results"
])
```

### Pattern 3: Parallel Code Generation

**Scenario**: Implement multiple independent components

**Sequential (WRONG)**:
```
❌ Implement UserService
   Implement OrderService
   Implement PaymentService
```

**Parallel (CORRECT)**:
```
✅ spawn_parallel([
  "Implement UserService with CRUD operations in src/services/user.rs",
  "Implement OrderService with order management in src/services/order.rs",
  "Implement PaymentService with payment processing in src/services/payment.rs"
])
```

### Pattern 4: Concurrent Research

**Scenario**: Research multiple topics

**Sequential (WRONG)**:
```
❌ h2ce_search("authentication patterns")
   h2ce_search("rate limiting approaches")
   h2ce_search("caching strategies")
```

**Parallel (CORRECT)**:
```
✅ spawn_parallel([
  "Search h2ce for authentication patterns (L1 level) and summarize best practices",
  "Search h2ce for rate limiting approaches (L1 level) and summarize strategies",
  "Search h2ce for caching strategies (L1 level) and summarize implementations"
])
```

### Pattern 5: Multi-Target Compilation

**Scenario**: Build for multiple architectures

**Sequential (WRONG)**:
```
❌ cargo build --target x86_64-unknown-linux-gnu
   cargo build --target aarch64-unknown-linux-gnu
   cargo build --target x86_64-apple-darwin
```

**Parallel (CORRECT)**:
```
✅ spawn_parallel([
  "Build for x86_64-unknown-linux-gnu using cargo build --target x86_64-unknown-linux-gnu",
  "Build for aarch64-unknown-linux-gnu using cargo build --target aarch64-unknown-linux-gnu",
  "Build for x86_64-apple-darwin using cargo build --target x86_64-apple-darwin"
])
```

### Pattern 6: Independent Documentation Tasks

**Scenario**: Generate multiple documentation files

**Parallel (CORRECT)**:
```
✅ spawn_parallel([
  "Generate API documentation for UserService module",
  "Generate API documentation for OrderService module",
  "Generate deployment guide based on current infrastructure",
  "Generate architecture diagram description from codebase analysis"
])
```

### Pattern 7: Concurrent Searches (Built-in Parallel Tool Calls)

**Scenario**: Initial context gathering

**Sequential (WRONG)**:
```
❌ h2ce_search("feature X implementation")
   wait for result...
   icarus_query_memory("feature X")
   wait for result...
   task_list(project="current")
```

**Parallel (CORRECT - Single Message, Multiple Tool Calls)**:
```
✅ Call all tools in single message:
   mcp__h2ce__h2ce_search(query="feature X", level="all")
   mcp__icarus__icarus_query_memory(level="episodic", query="feature X")
   mcp__task-manager__task_list(project="current")
```

---

## 3. Two Types of Parallelization

### Type 1: Built-in Parallel Tool Calls

**When**: Multiple MCP tool calls with no dependencies

**How**: Include multiple tool invocations in a single message

**Example**:
```
Starting feature implementation → Call in parallel:
  - h2ce_search (find similar patterns)
  - icarus_query_memory (find past approaches)
  - task_create (create implementation task)
  - task_create (create testing task)
  - task_create (create documentation task)
```

**Benefits**:
- No overhead (built into Claude Code)
- Immediate results
- Efficient for quick MCP tool calls

**Use for**:
- MCP tool calls
- Independent file reads
- Independent searches
- Task creation
- Status queries

### Type 2: Parallel Agent Spawning

**When**: Complex independent work requiring multiple steps

**How**: Use `mcp__parallel-subagent-spawner__spawn_parallel`

**Example**:
```
Refactoring codebase → Spawn agents:
  Agent 1: "Refactor src/services/*.rs to use new error handling pattern"
  Agent 2: "Refactor src/models/*.rs to use new validation pattern"
  Agent 3: "Refactor tests to match new service interfaces"
```

**Benefits**:
- Full agent capability for each task
- Can handle complex multi-step work
- Isolated execution environments
- Result aggregation

**Use for**:
- Complex analysis tasks (each requires multiple steps)
- Independent code generation
- Parallel testing suites
- Concurrent builds
- Multi-file refactoring
- Independent research topics

---

## 4. Parallelization Strategy by Task Type

### Code Implementation

```
Scenario: Implement 5 independent components

Strategy: Spawn parallel agents
✅ spawn_parallel([
  "Implement ComponentA with [specs]",
  "Implement ComponentB with [specs]",
  "Implement ComponentC with [specs]",
  "Implement ComponentD with [specs]",
  "Implement ComponentE with [specs]"
])

Aggregation: Review all implementations, ensure consistency
```

### Codebase Analysis

```
Scenario: Analyze entire codebase for security issues

Strategy: Spawn parallel agents by module
✅ spawn_parallel([
  "Analyze src/auth/*.rs for security vulnerabilities",
  "Analyze src/api/*.rs for security vulnerabilities",
  "Analyze src/db/*.rs for SQL injection risks",
  "Analyze src/services/*.rs for authorization issues"
])

Aggregation: Combine findings, prioritize by severity
```

### Testing

```
Scenario: Run comprehensive test suite

Strategy: Spawn parallel agents by test type
✅ spawn_parallel([
  "Run unit tests with cargo test --lib and report failures",
  "Run integration tests with cargo test --test '*' and report failures",
  "Run benchmark tests with cargo bench and report performance",
  "Run doc tests with cargo test --doc and report failures"
])

Aggregation: Combine test reports, identify failures
```

### Research & Learning

```
Scenario: Research best practices for new technology

Strategy: Spawn parallel research agents
✅ spawn_parallel([
  "Search h2ce for existing implementations of Technology X",
  "Query icarus episodic memory for past Technology X experiences",
  "Research Technology X architecture patterns via web if available",
  "Analyze how Technology X integrates with current codebase patterns"
])

Aggregation: Synthesize findings into comprehensive guide
```

### Documentation

```
Scenario: Create comprehensive project documentation

Strategy: Spawn parallel documentation agents
✅ spawn_parallel([
  "Generate API reference documentation from code comments",
  "Create deployment guide based on infrastructure code",
  "Write user guide based on public API surface",
  "Create architecture documentation from codebase structure",
  "Generate troubleshooting guide from error handling code"
])

Aggregation: Organize into coherent documentation structure
```

---

## 5. Result Aggregation Strategies

### Strategy 1: Summary Aggregation

**Use when**: Combining analysis results

**Pattern**:
```
1. Spawn parallel agents for analysis
2. Each agent returns structured findings
3. Aggregate: Combine, deduplicate, prioritize
4. Present: Unified summary with priority order
```

**Example**:
```
Security analysis across modules →
  Agent 1: Found 3 SQL injection risks in src/db
  Agent 2: Found 2 XSS vulnerabilities in src/api
  Agent 3: Found 1 auth bypass in src/services

Aggregation:
  HIGH: 1 auth bypass (critical)
  MEDIUM: 3 SQL injection (important)
  LOW: 2 XSS (needs fix)
```

### Strategy 2: Merge Aggregation

**Use when**: Combining code/documentation

**Pattern**:
```
1. Spawn parallel agents for generation
2. Each agent creates independent artifact
3. Aggregate: Merge with conflict resolution
4. Verify: Check consistency, no duplicates
```

### Strategy 3: Test Result Aggregation

**Use when**: Running parallel tests

**Pattern**:
```
1. Spawn parallel test agents
2. Each agent returns pass/fail + details
3. Aggregate: Count failures, categorize issues
4. Present: Overall status + failure details
```

### Strategy 4: Metric Aggregation

**Use when**: Collecting performance data

**Pattern**:
```
1. Spawn parallel benchmark agents
2. Each agent returns metrics
3. Aggregate: Calculate averages, identify outliers
4. Present: Performance report with recommendations
```

---

## 6. Coordination Patterns

### Pattern: Leader-Worker

```
Main agent (leader):
  1. Breaks work into independent tasks
  2. Spawns worker agents with spawn_parallel
  3. Waits for completion
  4. Aggregates results
  5. Makes decisions based on combined output

Worker agents:
  - Execute assigned task independently
  - Return structured results
  - No inter-worker communication needed
```

### Pattern: Map-Reduce

```
Map phase:
  - Spawn parallel agents
  - Each processes subset of data
  - Each returns intermediate results

Reduce phase:
  - Main agent aggregates results
  - Combines, filters, summarizes
  - Produces final output
```

### Pattern: Pipeline with Parallel Stages

```
Stage 1 (Sequential): Plan overall approach
  ↓
Stage 2 (Parallel): spawn_parallel([task1, task2, task3])
  ↓
Stage 3 (Sequential): Aggregate and verify
  ↓
Stage 4 (Parallel): spawn_parallel([test1, test2, test3])
  ↓
Stage 5 (Sequential): Final report
```

---

## 7. Performance Optimization Tips

### Tip 1: Batch Similar Tasks

**Instead of**:
```
❌ spawn_agent("Analyze file1")
   spawn_agent("Analyze file2")
   spawn_agent("Analyze file3")
```

**Do**:
```
✅ spawn_parallel([
  "Analyze file1",
  "Analyze file2",
  "Analyze file3"
])
```

### Tip 2: Set Appropriate Priorities

```
spawn_parallel([
  "CRITICAL: Fix security vulnerability",
  "HIGH: Implement core feature",
  "MEDIUM: Update documentation"
], priority="high")  # For critical work

spawn_parallel([
  "Refactor helper functions",
  "Update code comments",
  "Format code style"
], priority="low")  # For non-critical work
```

### Tip 3: Use Timeouts Wisely

```
# Short timeout for quick tasks
spawn_parallel([...], timeout_seconds=60)

# Longer timeout for complex work
spawn_parallel([...], timeout_seconds=300)

# Very long for builds/benchmarks
spawn_parallel([...], timeout_seconds=600)
```

### Tip 4: Monitor and Adjust

```
# After spawning, monitor progress
list_agents() → Check status of all agents
get_agent_status(agent_id) → Check specific agent

# Adjust max parallelism based on system
set_max_parallel(8)  # For powerful systems
set_max_parallel(4)  # For resource-constrained systems
```

---

## 8. Common Mistakes to Avoid

### Mistake 1: Parallelizing Dependent Tasks

**WRONG**:
```
❌ spawn_parallel([
  "Compile code",
  "Run tests"  # Depends on compilation!
])
```

**CORRECT**:
```
✅ Sequential:
   1. Compile code
   2. THEN spawn_parallel([
        "Run unit tests",
        "Run integration tests",
        "Run benchmark tests"
      ])
```

### Mistake 2: Too Fine-Grained Parallelization

**WRONG**:
```
❌ spawn_parallel([
  "Read line 1 of file",
  "Read line 2 of file"
])
# Overhead >> benefit
```

**CORRECT**:
```
✅ spawn_parallel([
  "Read and analyze file1",
  "Read and analyze file2"
])
# Meaningful chunks of work
```

### Mistake 3: Ignoring Race Conditions

**WRONG**:
```
❌ spawn_parallel([
  "Update shared configuration file",
  "Update shared configuration file with different values"
])
# Both write to same file!
```

**CORRECT**:
```
✅ Sequential:
   1. Merge configuration changes
   2. Update configuration file once
```

### Mistake 4: Not Aggregating Results

**WRONG**:
```
❌ spawn_parallel([...])
   # Results scattered, no synthesis
```

**CORRECT**:
```
✅ results = spawn_parallel([...])
   # Aggregate, summarize, present coherently
```

---

## 9. Parallelization Checklist

Before executing work, Claude Code should ask:

### ✅ Pre-Execution Checklist

- [ ] **Independence Check**: Are all tasks truly independent?
- [ ] **Data Dependency Check**: Does any task need output from another?
- [ ] **State Conflict Check**: Do any tasks modify shared state?
- [ ] **Count Check**: Are there 3+ independent tasks? (If yes → parallelize)
- [ ] **Complexity Check**: Is each task complex enough to warrant agent? (If no → built-in parallel tool calls)
- [ ] **Priority Assignment**: Have I assigned appropriate priorities?
- [ ] **Timeout Setting**: Have I set reasonable timeouts?
- [ ] **Aggregation Plan**: Do I have a plan for combining results?

### ✅ Post-Execution Checklist

- [ ] **Completion Check**: Did all agents complete successfully?
- [ ] **Error Check**: Did any agents encounter errors?
- [ ] **Result Aggregation**: Have I combined results coherently?
- [ ] **Consistency Check**: Are results consistent across agents?
- [ ] **Learning**: Should Icarus learn from this parallel execution pattern?

---

## 10. Examples by Complexity

### Simple (Built-in Parallel Tool Calls)

```
User: "Create 5 tasks for the new feature"

✅ Single message, 5 tool calls:
task_create(title="Task 1", ...)
task_create(title="Task 2", ...)
task_create(title="Task 3", ...)
task_create(title="Task 4", ...)
task_create(title="Task 5", ...)
```

### Moderate (spawn_parallel for independent analysis)

```
User: "Analyze the codebase for code quality issues"

✅ spawn_parallel([
  "Analyze src/services for code quality using clippy and complexity metrics",
  "Analyze src/models for code quality using clippy and complexity metrics",
  "Analyze src/api for code quality using clippy and complexity metrics",
  "Analyze tests for code quality and coverage analysis"
])
```

### Complex (Pipeline with multiple parallel stages)

```
User: "Refactor the entire authentication system"

✅ Stage 1 (Sequential): Analyze current auth system
   ↓
   Stage 2 (Parallel): spawn_parallel([
     "Design new JWT-based auth approach",
     "Design new session management approach",
     "Research industry best practices"
   ])
   ↓
   Stage 3 (Sequential): Choose best approach, create plan
   ↓
   Stage 4 (Parallel): spawn_parallel([
     "Implement auth middleware",
     "Implement token management",
     "Implement session storage",
     "Implement auth tests"
   ])
   ↓
   Stage 5 (Parallel): spawn_parallel([
     "Run auth unit tests",
     "Run auth integration tests",
     "Run security audit"
   ])
   ↓
   Stage 6 (Sequential): Aggregate results, finalize
```

---

## 11. Integration with MCP Cognitive Tools

### Parallel Task Management

```
When creating multiple tasks:
✅ Use built-in parallel tool calls (fast, low overhead)

Multiple task_create calls in single message:
  task_create(title="Analyze", priority=2, ...)
  task_create(title="Implement", priority=2, ...)
  task_create(title="Test", priority=3, ...)
```

### Parallel Knowledge Retrieval

```
When gathering context:
✅ Use built-in parallel tool calls

Single message:
  h2ce_search(query="auth patterns", level="L1")
  h2ce_search(query="security best practices", level="L2")
  icarus_query_memory(level="episodic", query="authentication")
  task_list(project="current")
```

### Parallel Deep Reasoning

```
When exploring multiple approaches:
✅ Use spawn_parallel for markovian-thinker sessions

spawn_parallel([
  "Use markovian-thinker to reason through Approach A: [details]",
  "Use markovian-thinker to reason through Approach B: [details]",
  "Use markovian-thinker to reason through Approach C: [details]"
])

Then: Compare reasoning traces, choose best approach
```

### Parallel Learning

```
After completing parallel work:
✅ Teach Icarus about parallelization patterns

icarus_learn_from_interaction(
  problem="Refactor authentication system",
  reasoning=[
    "Broke work into independent modules",
    "Spawned parallel agents for each module",
    "Aggregated results and verified consistency"
  ],
  solution="Parallel refactoring completed 4x faster",
  context={
    pattern: "parallel-refactoring",
    speedup: "4x",
    agents_used: 4
  }
)
```

---

## 12. Summary: Parallelization Principles

### Core Principles

1. ✅ **Default to Parallel**: Unless proven dependency, parallelize
2. ✅ **Two Types**: Built-in parallel tool calls (simple) vs spawn_parallel (complex)
3. ✅ **Independence First**: Verify no data dependencies
4. ✅ **Aggregate Always**: Combine results coherently
5. ✅ **Learn Patterns**: Teach Icarus successful parallelization approaches
6. ✅ **Monitor Health**: Check agent status, handle failures
7. ✅ **Optimize Resources**: Set appropriate priorities and timeouts
8. ✅ **Think Pipeline**: Mix parallel and sequential stages as needed

### Decision Matrix

| Task Type | Count | Complexity | Strategy |
|-----------|-------|------------|----------|
| MCP tool calls | Any | Low | Built-in parallel (single message) |
| Analysis tasks | 3+ | Medium | spawn_parallel |
| Code generation | 3+ | High | spawn_parallel |
| Testing | 3+ | Medium | spawn_parallel |
| Research | 3+ | Medium | spawn_parallel |
| Builds | 3+ | High | spawn_parallel |
| Dependent work | Any | Any | Sequential with parallel stages |

### Performance Expectations

- **Built-in parallel tool calls**: Near-instant (no overhead)
- **spawn_parallel (3 agents)**: ~3x faster than sequential
- **spawn_parallel (5 agents)**: ~5x faster than sequential
- **spawn_parallel (10 agents)**: ~8-10x faster (diminishing returns)

### Remember

**Parallelization is not optional when tasks are independent. It's the default mode of operation for maximal efficiency.**
