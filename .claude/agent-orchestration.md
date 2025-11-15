# Autonomous Agent Orchestration System

## Overview

The parallel-subagent-spawner (formerly parallel-subagent-spawner) integrates with ALL MCP cognitive tools to create a self-managing, always-working autonomous system where:

- **Planning agents** create tasks in task-manager
- **Worker agents** continuously pull prioritized tasks
- **All agents** use h2ce, markovian-thinker, and icarus autonomously
- **System never idles** when work exists
- **Full autonomy** - no human intervention needed for task dispatch

---

## Architecture: Multi-Agent Cognitive System

```
┌─────────────────────────────────────────────────────────────┐
│                    Claude Code (Orchestrator)                │
└─────────────────────────────────────────────────────────────┘
                              │
                              ├──────────────┬─────────────┬────────────┐
                              │              │             │            │
                              ▼              ▼             ▼            ▼
                    ┌─────────────┐  ┌────────────┐ ┌──────────┐ ┌──────────┐
                    │  Planning   │  │  Worker    │ │  Worker  │ │  Worker  │
                    │   Agent     │  │  Agent 1   │ │  Agent 2 │ │  Agent N │
                    └─────────────┘  └────────────┘ └──────────┘ └──────────┘
                          │                │             │            │
                          │                │             │            │
                          ▼                ▼             ▼            ▼
                    ┌──────────────────────────────────────────────────┐
                    │         MCP Cognitive Tools Layer                │
                    ├──────────┬────────────┬──────────┬──────────────┤
                    │ task-    │ markovian- │  h2ce    │   icarus     │
                    │ manager  │ thinker    │          │              │
                    └──────────┴────────────┴──────────┴──────────────┘
```

---

## 1. Planning Agent Responsibilities

### Role: Task Creation & Workload Management

**Continuous Activities**:
1. Monitor user interactions and system state
2. Break down complex work into task-manager tasks
3. Assign priorities based on urgency and dependencies
4. Set deadlines and estimated hours
5. Create task hierarchies (parent/subtask relationships)
6. Add dependencies between related tasks

### MCP Tool Usage

```javascript
// Planning agent continuously creates tasks
mcp__task-manager__task_create({
  title: "Implement user authentication",
  description: "Add JWT-based auth with refresh tokens",
  priority: 2,  // HIGH
  estimated_hours: 8,
  deadline: "2025-11-20T00:00:00Z",
  project: "auth-system"
})

// Create subtasks
mcp__task-manager__task_create({
  title: "Design auth architecture",
  parent_id: "parent-task-uuid",
  priority: 1,  // CRITICAL - do first
  estimated_hours: 2
})

// Add dependencies
mcp__task-manager__dependency_add({
  task_id: "implement-auth-task-uuid",
  depends_on_task_id: "design-arch-task-uuid",
  dependency_type: "blocks"
})

// Add rich context
mcp__task-manager__context_add({
  task_id: "auth-task-uuid",
  context_type: "decision",
  content: "Chose JWT over session-based auth because: (1) stateless, (2) scales better, (3) mobile-friendly"
})
```

### Continuous Monitoring Loop

```
Planning Agent Loop (runs continuously):

  WHILE TRUE:
    1. Check for new user requests or system events
    2. IF complex work detected:
         a. Create task-manager tasks
         b. Set priorities using task_prioritized insights
         c. Add context, dependencies, deadlines
         d. Query icarus for similar past work patterns
         e. Use h2ce to find related existing code
    3. Monitor task-manager health:
         a. Check for orphaned tasks (no progress)
         b. Rebalance priorities if needed
         c. Create follow-up tasks if blockers discovered
    4. SLEEP briefly (avoid busy-wait)
```

---

## 2. Worker Agent Responsibilities

### Role: Task Execution & Completion

**Continuous Activities**:
1. Query task-manager for next prioritized task
2. Start time tracking for the task
3. Execute task using ALL available cognitive tools
4. Update progress regularly
5. Add context (decisions, blockers, code snippets)
6. Complete task and stop time tracking
7. Teach icarus the approach used

### Autonomous Task Pulling

```javascript
// Worker agent pulls next prioritized task
const tasks = mcp__task-manager__task_prioritized({
  limit: 1  // Get single highest-priority task
})

const task = tasks[0]

// Start time tracking
const session = mcp__task-manager__time_start({
  task_id: task.id,
  notes: "Starting work on highest priority task"
})

// Execute task (using all cognitive tools)
// ... work happens here ...

// Add context about what was done
mcp__task-manager__context_add({
  task_id: task.id,
  context_type: "code_snippet",
  content: "Implemented auth middleware in src/middleware/auth.rs"
})

// Update progress
mcp__task-manager__task_update({
  id: task.id,
  status: "completed",
  progress_percent: 100
})

// Stop time tracking
mcp__task-manager__time_stop({
  session_id: session.id
})

// Teach icarus
mcp__icarus__icarus_learn_from_interaction({
  problem: task.title,
  reasoning: ["Queried h2ce for patterns", "Used markovian-thinker for architecture", "Implemented following best practices"],
  solution: "Auth middleware complete with comprehensive tests",
  context: { task_id: task.id, tools_used: ["h2ce", "markovian-thinker"] }
})
```

### Continuous Work Loop

```
Worker Agent Loop (runs continuously):

  WHILE TRUE:
    1. Query task_prioritized for next task
    2. IF task exists:
         a. Start time tracking
         b. Load context from h2ce (search for related code)
         c. Query icarus memory for similar past tasks
         d. IF complex: Init markovian-thinker session
         e. Execute work
         f. Update task progress regularly
         g. Add rich context (decisions, code, blockers)
         h. Complete task
         i. Stop time tracking
         j. Teach icarus the approach
    3. ELSE (no tasks):
         a. Query icarus for suggested improvements
         b. Run health checks on cognitive systems
         c. Optimize existing code if idle
    4. SLEEP briefly before next query
```

---

## 3. Full Cognitive Tool Integration

### Worker Agent Using ALL Tools

When a worker agent executes a task, it MUST use all relevant cognitive tools:

#### Step 1: Knowledge Retrieval (h2ce)

```javascript
// Before implementing, search for patterns
const patterns = mcp__h2ce__h2ce_search({
  query: "authentication middleware implementation",
  level: "L1",  // Paragraph-level context
  top_k: 5
})

// Index the codebase if not already indexed
mcp__h2ce__h2ce_index({
  path: "./src"
})
```

#### Step 2: Memory Query (icarus)

```javascript
// Query episodic memory for past similar work
const memories = mcp__icarus__icarus_query_memory({
  level: "episodic",
  query: "authentication implementation"
})

// Query world model for predictions
const predictions = mcp__icarus__icarus_query_world_model({
  include_predictions: true,
  prediction_steps: 5
})
```

#### Step 3: Deep Reasoning (markovian-thinker)

```javascript
// For complex tasks, init reasoning session
const session = mcp__markovian-thinker__markovian_init_session({
  problem: "Design scalable JWT-based authentication system",
  chunk_size: 8192,
  max_iterations: 5,
  enable_causal_trace: true,
  lattice_type: "e8"
})

// Get reasoning prompt
const prompt = mcp__markovian-thinker__markovian_get_prompt({
  session_id: session.session_id
})

// Reason through the problem...
// Submit reasoning chunks...

// Get final trace
const trace = mcp__markovian-thinker__markovian_get_trace({
  session_id: session.session_id
})
```

#### Step 4: Task Management (task-manager)

```javascript
// Update task with progress
mcp__task-manager__task_update({
  id: task.id,
  status: "in_progress",
  progress_percent: 50
})

// Add context about decisions
mcp__task-manager__context_add({
  task_id: task.id,
  context_type: "decision",
  content: "Chose bcrypt for password hashing - industry standard, well-tested"
})

// Add code snippet
mcp__task-manager__context_add({
  task_id: task.id,
  context_type: "code_snippet",
  content: "impl AuthMiddleware for JwtAuth { ... }",
  metadata: { file: "src/middleware/auth.rs", lines: "45-89" }
})

// If blocker encountered
mcp__task-manager__context_add({
  task_id: task.id,
  context_type: "blocker",
  content: "Need to decide between HS256 and RS256 signing algorithm"
})
```

#### Step 5: Learning (icarus)

```javascript
// After completing task, teach icarus
mcp__icarus__icarus_learn_from_interaction({
  problem: "Implement JWT authentication middleware",
  reasoning: [
    "Searched h2ce for existing auth patterns",
    "Queried icarus memory for past auth implementations",
    "Used markovian-thinker to reason through JWT vs session trade-offs",
    "Chose JWT for stateless scaling benefits",
    "Implemented middleware with comprehensive error handling",
    "Added tests for token validation, expiry, refresh"
  ],
  solution: "Fully functional JWT auth middleware with refresh token support",
  context: {
    task_id: task.id,
    complexity: "high",
    tools_used: ["h2ce", "icarus", "markovian-thinker", "task-manager"],
    patterns: ["middleware", "JWT", "auth"],
    domain: "authentication",
    outcome: "success",
    time_spent_hours: 6.5
  }
})
```

---

## 4. Autonomous Workflow Patterns

### Pattern 1: Feature Implementation (Full Autonomous)

```
User: "Add user authentication"

ORCHESTRATOR (Claude Code):
  1. Recognize: Feature implementation request

PLANNING AGENT:
  1. Create task-manager tasks:
     - [CRITICAL] Design auth architecture
     - [HIGH] Implement JWT middleware
     - [HIGH] Add user login endpoint
     - [HIGH] Add user registration endpoint
     - [MEDIUM] Write auth tests
     - [MEDIUM] Update documentation
  2. Set dependencies:
     - Implement depends on Design
     - Tests depend on Implement
  3. Add context:
     - Requirements: JWT-based, refresh tokens, secure

WORKER AGENTS (spawned in parallel):

  Worker 1: Design Task
    1. task_prioritized → Get "Design auth architecture"
    2. h2ce_search → Find existing design patterns
    3. icarus_query_memory → Past architecture decisions
    4. markovian_init_session → Reason through design
    5. [DESIGN ARCHITECTURE]
    6. task_update → Complete, add architecture doc as context
    7. icarus_learn_from_interaction → Store design approach

  Worker 2: Waits for Design to Complete
    1. task_prioritized → Get "Implement JWT middleware" (blocked until Design done)
    2. [Once unblocked...]
    3. h2ce_search → Find JWT implementation examples
    4. icarus_query_memory → Past JWT implementations
    5. [IMPLEMENT MIDDLEWARE]
    6. task_update → Complete, add code as context
    7. icarus_learn_from_interaction → Store implementation approach

  Worker 3-N: Similarly pull and complete tasks in parallel when dependencies allow
```

### Pattern 2: Debugging (Autonomous)

```
User: "Fix the authentication bug"

PLANNING AGENT:
  1. Create tasks:
     - [CRITICAL] Reproduce authentication bug
     - [CRITICAL] Identify root cause
     - [HIGH] Implement fix
     - [HIGH] Verify fix with tests
  2. Add context:
     - Bug description from user
     - Error logs if available

WORKER AGENT:
  1. task_prioritized → Get "Reproduce bug"
  2. h2ce_search → Search for similar bugs
  3. icarus_query_memory(level="episodic", query="authentication bugs")
  4. [REPRODUCE]
  5. task_update → Add reproduction steps as context

  6. task_prioritized → Get "Identify root cause"
  7. markovian_init_session → Deep reasoning through possible causes
  8. [ANALYZE CODE, LOGS]
  9. task_update → Add root cause as context

  10. task_prioritized → Get "Implement fix"
  11. h2ce_search → Find fix patterns
  12. [FIX BUG]
  13. task_update → Complete, add fix as code snippet

  14. icarus_learn_from_interaction → Teach bug fix approach
```

### Pattern 3: Continuous Background Work

```
ORCHESTRATOR:
  Spawns persistent worker agents that run continuously

WORKER AGENTS (always running):

  WHILE TRUE:
    1. Query task_prioritized(limit=1)
    2. IF task exists:
         - Pull task
         - Execute with full cognitive tools
         - Complete and learn
    3. ELSE (no tasks):
         - Query icarus for optimization suggestions
         - Run code quality scans
         - Update h2ce indices
         - Optimize existing implementations
         - Monitor system health
    4. SLEEP 1-5 seconds
    5. REPEAT
```

---

## 5. Coordination & Conflict Avoidance

### Task Locking (Prevents Double Work)

```
Worker Agent Task Pull:

  1. task_prioritized(limit=1) → Get next task
  2. task_update(id=task.id, status="in_progress") → Lock it
  3. [WORK ON TASK]
  4. task_update(id=task.id, status="completed") → Release lock

  Other workers:
    - When they call task_prioritized, they get NEXT task
    - They don't see tasks already "in_progress"
    - No double work
```

### Dependency Coordination

```
Task Dependencies:

  If Task B depends on Task A:
    - Worker pulling Task B sees it's blocked
    - task_blockers(task_id=B) → Returns [Task A]
    - Worker skips Task B, pulls next unblocked task
    - When Task A completes, Task B becomes available
```

### Context Sharing

```
Workers communicate via task-manager context:

  Worker 1:
    context_add(
      task_id=task.id,
      context_type="decision",
      content="Using RS256 for JWT signing"
    )

  Worker 2 (working on related task):
    task_get(id=related_task.id)
    → Sees Worker 1's decision in context
    → Aligns approach accordingly
```

---

## 6. Scaling & Load Balancing

### Dynamic Worker Pool

```
ORCHESTRATOR monitors workload:

  High workload (many pending tasks):
    set_max_parallel(10)  # Spawn more workers

  Medium workload:
    set_max_parallel(5)   # Moderate workers

  Low workload (few tasks):
    set_max_parallel(2)   # Reduce workers

  No workload:
    set_max_parallel(1)   # Single worker for maintenance
```

### Priority-Based Scheduling

```
Workers always pull highest priority:

  task_prioritized(limit=1)
    → Returns task with highest priority score
    → Score considers: deadline, dependencies, progress, staleness

  Critical tasks (priority=1) always processed first
  Low priority tasks (priority=5) wait until critical work done
```

---

## 7. Health Monitoring & Recovery

### System Health Checks

```
Dedicated Health Agent:

  WHILE TRUE:
    1. icarus_query_status()
       → Check cognitive system health

    2. task_list(status="in_progress")
       → Check for stuck tasks (in progress >1 hour)

    3. markovian_list_sessions()
       → Check for incomplete reasoning sessions

    4. list_agents()
       → Check worker agent health

    5. IF issues found:
         - Create recovery tasks
         - Restart stuck agents
         - Alert orchestrator

    6. SLEEP 60 seconds
```

### Error Recovery

```
Worker Agent Error Handling:

  TRY:
    Execute task
  CATCH error:
    1. context_add(
         task_id=task.id,
         context_type="blocker",
         content=f"Error: {error}"
       )
    2. task_update(id=task.id, status="blocked")
    3. Create new task: "Resolve blocker in Task X"
    4. icarus_learn_from_interaction(error_pattern)
    5. Pull next task (don't halt)
```

---

## 8. Complete Autonomous Example

### Scenario: User Requests Feature

```
User: "Add rate limiting to the API"

═══ ORCHESTRATOR (Claude Code) ═══

1. Intent Recognition: Feature implementation
2. Complexity: High (multi-step)
3. Action: Engage autonomous system

═══ PLANNING AGENT ═══

CREATE TASKS:
  task_create(title="Research rate limiting strategies", priority=1)
    └─ ID: task-001

  task_create(title="Design rate limiter architecture", priority=1, parent_id=task-001)
    └─ ID: task-002, depends_on: task-001

  task_create(title="Implement rate limiter middleware", priority=2, parent_id=task-002)
    └─ ID: task-003, depends_on: task-002

  task_create(title="Add rate limit configuration", priority=2)
    └─ ID: task-004

  task_create(title="Write rate limiter tests", priority=3, parent_id=task-003)
    └─ ID: task-005, depends_on: task-003

  task_create(title="Update API documentation", priority=3)
    └─ ID: task-006

ADD CONTEXT:
  context_add(task_id=task-001, type="note", content="Consider: token bucket, leaky bucket, sliding window")

═══ WORKER AGENTS (spawned) ═══

Worker 1:
  ✓ task_prioritized() → task-001 (research)
  ✓ h2ce_search("rate limiting patterns")
  ✓ icarus_query_memory("rate limiting")
  ✓ markovian_init_session("Analyze rate limiting strategies")
  ✓ [RESEARCH]
  ✓ task_update(status="completed")
  ✓ context_add(type="research", content="Token bucket best for burst handling")
  ✓ icarus_learn_from_interaction(...)

Worker 2:
  ✓ task_prioritized() → task-002 (design) - now unblocked
  ✓ h2ce_search("token bucket implementation")
  ✓ markovian_init_session("Design token bucket rate limiter")
  ✓ [DESIGN]
  ✓ task_update(status="completed")
  ✓ context_add(type="code_snippet", content="RateLimiter struct design")
  ✓ icarus_learn_from_interaction(...)

Worker 3 & 4 (in parallel):
  Worker 3:
    ✓ task_prioritized() → task-003 (implement middleware)
    ✓ h2ce_search("middleware examples")
    ✓ [IMPLEMENT]
    ✓ task_update(status="completed")
    ✓ h2ce_index("src/middleware/rate_limiter.rs")
    ✓ icarus_learn_from_interaction(...)

  Worker 4:
    ✓ task_prioritized() → task-004 (config) - parallel with Worker 3
    ✓ [IMPLEMENT CONFIG]
    ✓ task_update(status="completed")
    ✓ icarus_learn_from_interaction(...)

Worker 5:
  ✓ task_prioritized() → task-005 (tests) - unblocked after task-003
  ✓ h2ce_search("rate limiter test patterns")
  ✓ [WRITE TESTS]
  ✓ task_update(status="completed")
  ✓ icarus_learn_from_interaction(...)

Worker 6:
  ✓ task_prioritized() → task-006 (docs)
  ✓ [UPDATE DOCS]
  ✓ task_update(status="completed")
  ✓ icarus_learn_from_interaction(...)

═══ RESULT ═══

All tasks complete, rate limiting fully implemented.
Icarus learned entire pattern.
All context stored in task-manager.
Ready for next work.
```

---

## 9. Summary: Fully Autonomous System

### Key Properties

1. **✅ Self-Managing**: Planning agent creates tasks, workers pull and execute
2. **✅ Always Working**: Workers continuously query for next task, never idle
3. **✅ Full Integration**: Every agent uses h2ce, icarus, markovian-thinker, task-manager
4. **✅ Conflict-Free**: Task locking prevents double work
5. **✅ Coordinated**: Dependencies ensure correct ordering
6. **✅ Learning**: Every completion teaches icarus
7. **✅ Resilient**: Error recovery, health monitoring
8. **✅ Scalable**: Dynamic worker pool based on load

### Agent Lifecycle

```
┌─────────────────────────────────────────────────────┐
│              CONTINUOUS OPERATION                    │
├─────────────────────────────────────────────────────┤
│                                                      │
│  Planning Agent: Creates tasks → task-manager       │
│         ↓                                            │
│  Worker Agents: Pull tasks → Execute → Complete     │
│         ↓                                            │
│  All Agents: Use h2ce + icarus + markovian-thinker  │
│         ↓                                            │
│  Learning: icarus_learn_from_interaction             │
│         ↓                                            │
│  Repeat: Pull next task...                          │
│                                                      │
└─────────────────────────────────────────────────────┘
```

### The Promise

**With this autonomous orchestration system, Claude Code becomes a fully self-managing cognitive system that:**
- Requires minimal human direction (just high-level goals)
- Continuously works on highest-priority tasks
- Learns from every interaction
- Scales to workload
- Never forgets (icarus memory)
- Always has context (h2ce retrieval)
- Reasons deeply when needed (markovian-thinker)
- Coordinates efficiently (task-manager)

**This is true autonomous AI development assistance.**
