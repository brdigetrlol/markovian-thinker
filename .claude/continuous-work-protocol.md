# Continuous Work Protocol - Always-On Agent System

## Core Principle: Never Idle

**The system continuously pulls and executes tasks from cognitive-task-manager. There is always work being done when tasks exist.**

---

## 1. System Architecture

```
┌─────────────────────────────────────────────────────────┐
│                  Orchestrator (Claude Code)              │
│                                                          │
│  ┌────────────────────────────────────────────────┐    │
│  │          Task Queue (task-manager)             │    │
│  │  Priority 1: [task-001, task-002]              │    │
│  │  Priority 2: [task-003, task-004, task-005]    │    │
│  │  Priority 3: [task-006]                        │    │
│  └────────────────────────────────────────────────┘    │
│                        ↓ ↓ ↓                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐    │
│  │   Worker    │  │   Worker    │  │   Worker    │    │
│  │   Agent 1   │  │   Agent 2   │  │   Agent N   │    │
│  │             │  │             │  │             │    │
│  │ PULL → WORK │  │ PULL → WORK │  │ PULL → WORK │    │
│  │    ↻        │  │    ↻        │  │    ↻        │    │
│  └─────────────┘  └─────────────┘  └─────────────┘    │
│                                                          │
│  ┌────────────────────────────────────────────────┐    │
│  │              Cognitive Tools                    │    │
│  │  h2ce | markovian-thinker | icarus | parallel  │    │
│  └────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

---

## 2. Agent Lifecycle & State Machine

### Worker Agent States

```
┌──────────┐
│   IDLE   │ ← Agent spawned
└────┬─────┘
     │
     ▼
┌──────────┐
│ QUERYING │ ← Call task_prioritized()
└────┬─────┘
     │
     ├─ Task found ──────────┐
     │                       ▼
     │                  ┌──────────┐
     │                  │ WORKING  │ ← Execute task
     │                  └────┬─────┘
     │                       │
     │                       ▼
     │                  ┌──────────┐
     │                  │COMPLETING│ ← Update, learn
     │                  └────┬─────┘
     │                       │
     │                       └─────┐
     │                             │
     ├─ No task found ──────┐      │
     │                      ▼      │
     │                 ┌──────────┐│
     │                 │  IDLE+   ││ ← Do maintenance
     │                 └────┬─────┘│
     │                      │      │
     │                      │      │
     └──────────────────────┴──────┘
                            │
                            ▼
                    SLEEP → REPEAT
```

### State Transitions

```javascript
STATE: IDLE
  → Call task_prioritized(limit=1)
  → TRANSITION TO: QUERYING

STATE: QUERYING
  IF task found:
    → Lock task (update status="in_progress")
    → TRANSITION TO: WORKING
  ELSE:
    → TRANSITION TO: IDLE+ (maintenance mode)

STATE: WORKING
  → Start time tracking
  → Load context (h2ce, icarus)
  → Execute task
  → Update progress periodically
  → TRANSITION TO: COMPLETING

STATE: COMPLETING
  → Update task (status="completed")
  → Stop time tracking
  → Add context (results, decisions, code)
  → Teach icarus (learn_from_interaction)
  → TRANSITION TO: IDLE

STATE: IDLE+ (Maintenance Mode)
  → Run health checks
  → Optimize code
  → Update indices
  → Clean up old data
  → SLEEP
  → TRANSITION TO: IDLE
```

---

## 3. Continuous Work Loop (Pseudo-Code)

### Worker Agent Implementation

```javascript
class WorkerAgent {
  constructor(agent_id, orchestrator) {
    this.agent_id = agent_id
    this.orchestrator = orchestrator
    this.state = "IDLE"
    this.current_task = null
    this.session = null
  }

  async run() {
    while (true) {
      switch (this.state) {
        case "IDLE":
          this.state = "QUERYING"
          break

        case "QUERYING":
          const tasks = await this.pullNextTask()
          if (tasks && tasks.length > 0) {
            this.current_task = tasks[0]
            this.state = "WORKING"
          } else {
            this.state = "IDLE+"
          }
          break

        case "WORKING":
          await this.executeTask()
          this.state = "COMPLETING"
          break

        case "COMPLETING":
          await this.completeTask()
          this.current_task = null
          this.state = "IDLE"
          break

        case "IDLE+":
          await this.maintenanceMode()
          this.state = "IDLE"
          await this.sleep(5000)  // 5 second sleep
          break
      }
    }
  }

  async pullNextTask() {
    // Pull highest priority task
    return await mcp__task-manager__task_prioritized({
      limit: 1,
      status: "pending"  // Only pull tasks not yet started
    })
  }

  async executeTask() {
    const task = this.current_task

    // 1. Lock task
    await mcp__task-manager__task_update({
      id: task.id,
      status: "in_progress",
      progress_percent: 0
    })

    // 2. Start time tracking
    this.session = await mcp__task-manager__time_start({
      task_id: task.id,
      notes: `Worker ${this.agent_id} started work`
    })

    // 3. Load context from cognitive tools
    const context = await this.loadContext(task)

    // 4. Execute work
    await this.doWork(task, context)

    // 5. Update progress
    await mcp__task-manager__task_update({
      id: task.id,
      progress_percent: 100
    })
  }

  async loadContext(task) {
    // Parallel context loading
    const [h2ce_results, icarus_memories, blockers] = await Promise.all([
      // Search h2ce for relevant code/docs
      mcp__h2ce__h2ce_search({
        query: task.title,
        level: "all",
        top_k: 5
      }),

      // Query icarus for related experiences
      mcp__icarus__icarus_query_memory({
        level: "episodic",
        query: task.title
      }),

      // Check for blockers
      mcp__task-manager__task_blockers({
        task_id: task.id
      })
    ])

    return { h2ce_results, icarus_memories, blockers }
  }

  async doWork(task, context) {
    // Determine complexity and choose approach
    if (task.estimated_hours > 4) {
      // Complex task - use markovian-thinker
      const session = await mcp__markovian-thinker__markovian_init_session({
        problem: task.description || task.title,
        chunk_size: 8192,
        enable_causal_trace: true
      })

      // Reason through task...
      // Execute based on reasoning...
    } else {
      // Standard task - direct execution
      // Use context from h2ce and icarus
      // Execute work...
    }

    // Add context about work done
    await mcp__task-manager__context_add({
      task_id: task.id,
      context_type: "note",
      content: "Work completed by worker agent"
    })
  }

  async completeTask() {
    const task = this.current_task

    // 1. Mark task complete
    await mcp__task-manager__task_update({
      id: task.id,
      status: "completed",
      progress_percent: 100
    })

    // 2. Stop time tracking
    if (this.session) {
      await mcp__task-manager__time_stop({
        session_id: this.session.id
      })
    }

    // 3. Teach icarus
    await mcp__icarus__icarus_learn_from_interaction({
      problem: task.title,
      reasoning: ["Pulled task from queue", "Loaded context", "Executed work", "Completed successfully"],
      solution: `Completed: ${task.title}`,
      context: {
        task_id: task.id,
        worker_id: this.agent_id,
        tools_used: ["task-manager", "h2ce", "icarus"]
      }
    })
  }

  async maintenanceMode() {
    // No tasks available - do maintenance work
    console.log(`Worker ${this.agent_id}: No tasks, entering maintenance mode`)

    // 1. Query icarus for optimization suggestions
    const status = await mcp__icarus__icarus_query_status()

    // 2. Check for orphaned tasks (stuck in progress >1 hour)
    const all_tasks = await mcp__task-manager__task_list({
      status: "in_progress"
    })

    // 3. Run health checks
    // 4. Optimize indices
    // 5. Clean up data

    console.log(`Worker ${this.agent_id}: Maintenance complete`)
  }

  async sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms))
  }
}
```

---

## 4. Planning Agent (Continuous Task Creation)

### Planning Agent Loop

```javascript
class PlanningAgent {
  constructor(orchestrator) {
    this.orchestrator = orchestrator
  }

  async run() {
    while (true) {
      // 1. Monitor for new user requests
      const new_requests = await this.checkForNewWork()

      if (new_requests.length > 0) {
        // 2. Create tasks for each request
        for (const request of new_requests) {
          await this.createTasksForRequest(request)
        }
      }

      // 3. Monitor task health
      await this.monitorTaskHealth()

      // 4. Rebalance priorities if needed
      await this.rebalancePriorities()

      // 5. Sleep briefly
      await this.sleep(10000)  // 10 seconds
    }
  }

  async createTasksForRequest(request) {
    // Analyze request complexity
    const complexity = this.analyzeComplexity(request)

    if (complexity === "high") {
      // Complex request - break into multiple tasks
      const subtasks = await this.decomposeRequest(request)

      // Create tasks with dependencies
      let parent_task = null
      for (const subtask of subtasks) {
        const task = await mcp__task-manager__task_create({
          title: subtask.title,
          description: subtask.description,
          priority: subtask.priority,
          estimated_hours: subtask.estimated_hours,
          parent_id: parent_task?.id
        })

        if (subtask.depends_on && parent_task) {
          await mcp__task-manager__dependency_add({
            task_id: task.id,
            depends_on_task_id: parent_task.id,
            dependency_type: "blocks"
          })
        }

        parent_task = task
      }
    } else {
      // Simple request - single task
      await mcp__task-manager__task_create({
        title: request.title,
        description: request.description,
        priority: request.priority,
        estimated_hours: request.estimated_hours
      })
    }
  }

  async monitorTaskHealth() {
    // Check for stuck tasks
    const in_progress = await mcp__task-manager__task_list({
      status: "in_progress"
    })

    for (const task of in_progress) {
      const age_hours = this.calculateAgeHours(task.updated_at)

      if (age_hours > 2) {
        // Task stuck - create investigation task
        await mcp__task-manager__task_create({
          title: `Investigate stuck task: ${task.title}`,
          description: `Task has been in progress for ${age_hours} hours`,
          priority: 1
        })
      }
    }
  }

  async rebalancePriorities() {
    // Get all pending tasks
    const tasks = await mcp__task-manager__task_prioritized({
      limit: 100
    })

    // Check if priority distribution is healthy
    // Adjust if needed based on deadlines, dependencies
  }

  async sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms))
  }
}
```

---

## 5. Orchestrator (Manages Agent Pool)

### Orchestrator Implementation

```javascript
class Orchestrator {
  constructor() {
    this.workers = []
    this.planning_agent = null
    this.max_workers = 5
  }

  async start() {
    // 1. Start planning agent
    this.planning_agent = new PlanningAgent(this)
    this.planning_agent.run()  // Runs continuously

    // 2. Start worker pool
    await this.scaleWorkerPool()

    // 3. Monitor and adjust
    while (true) {
      await this.monitorWorkload()
      await this.scaleWorkerPool()
      await this.sleep(30000)  // 30 seconds
    }
  }

  async monitorWorkload() {
    // Check pending task count
    const pending = await mcp__task-manager__task_list({
      status: "pending"
    })

    this.current_workload = pending.length

    // Check priority distribution
    const prioritized = await mcp__task-manager__task_prioritized({
      limit: 100
    })

    this.high_priority_count = prioritized.filter(t => t.priority <= 2).length
  }

  async scaleWorkerPool() {
    // Determine optimal worker count
    let target_workers = 1

    if (this.current_workload > 20) {
      target_workers = this.max_workers
    } else if (this.current_workload > 10) {
      target_workers = 3
    } else if (this.current_workload > 5) {
      target_workers = 2
    }

    // Adjust worker pool
    if (this.workers.length < target_workers) {
      // Spawn more workers
      const to_spawn = target_workers - this.workers.length
      for (let i = 0; i < to_spawn; i++) {
        const worker = new WorkerAgent(`worker-${this.workers.length}`, this)
        this.workers.push(worker)
        worker.run()  // Runs continuously
      }
    } else if (this.workers.length > target_workers) {
      // Reduce workers (gracefully)
      const to_remove = this.workers.length - target_workers
      for (let i = 0; i < to_remove; i++) {
        const worker = this.workers.pop()
        worker.stop()  // Graceful shutdown
      }
    }
  }

  async sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms))
  }
}
```

---

## 6. Coordination & Conflict Prevention

### Task Locking Mechanism

```
Worker A:
  1. task_prioritized() → Returns task-001 (status="pending")
  2. task_update(id=task-001, status="in_progress")
     → Task is now LOCKED

Worker B (querying simultaneously):
  1. task_prioritized() → Returns task-002 (skips task-001, it's in_progress)
  2. task_update(id=task-002, status="in_progress")
     → No conflict!
```

### Dependency Handling

```
Tasks:
  task-001: Design feature (status="pending")
  task-002: Implement feature (status="pending", depends_on=task-001)

Worker A:
  task_prioritized() → Returns task-001 (highest priority, unblocked)

Worker B:
  task_prioritized() → Returns task-003 (skips task-002, it's blocked)

[After task-001 completes]

Worker B:
  task_prioritized() → Now returns task-002 (unblocked)
```

---

## 7. Performance Metrics & Monitoring

### Key Metrics

```javascript
// Tracked continuously by orchestrator

metrics = {
  // Throughput
  tasks_completed_per_hour: 15,
  average_task_duration: "24 minutes",

  // Utilization
  worker_utilization: 85%,  // % of time workers are working vs idle
  task_queue_depth: 12,     // Current pending tasks

  // Quality
  tasks_blocked: 2,          // Tasks waiting on dependencies
  tasks_stuck: 0,            // Tasks in progress >2 hours

  // System health
  active_workers: 3,
  planning_agent_health: "healthy",
  cognitive_tools_health: {
    task_manager: "healthy",
    h2ce: "healthy",
    icarus: "healthy",
    markovian_thinker: "healthy"
  }
}
```

### Health Monitoring

```javascript
async function monitorSystemHealth() {
  // 1. Check orchestrator
  const orchestrator_healthy = orchestrator.isRunning()

  // 2. Check workers
  const workers_healthy = workers.every(w => w.state !== "ERROR")

  // 3. Check planning agent
  const planning_healthy = planning_agent.isRunning()

  // 4. Check cognitive tools
  const tools_healthy = await Promise.all([
    mcp__icarus__icarus_query_status(),
    mcp__task-manager__task_list({ limit: 1 }),
    mcp__h2ce__h2ce_search({ query: "test", level: "L0", top_k: 1 })
  ])

  // 5. Alert if issues
  if (!orchestrator_healthy || !workers_healthy || !planning_healthy) {
    // Initiate recovery procedures
    await recoverSystem()
  }
}
```

---

## 8. Recovery & Fault Tolerance

### Failure Scenarios & Recovery

```javascript
// Scenario 1: Worker crashes mid-task
async function recoverCrashedWorker(worker_id, task_id) {
  // 1. Mark task as failed
  await mcp__task-manager__task_update({
    id: task_id,
    status: "pending"  // Reset to pending
  })

  // 2. Add context about crash
  await mcp__task-manager__context_add({
    task_id: task_id,
    context_type: "blocker",
    content: `Worker ${worker_id} crashed during execution`
  })

  // 3. Spawn replacement worker
  const new_worker = new WorkerAgent(`worker-${worker_id}-recovery`, orchestrator)
  new_worker.run()
}

// Scenario 2: Task stuck
async function unstickTask(task_id) {
  // 1. Get task details
  const task = await mcp__task-manager__task_get({ id: task_id })

  // 2. Check blockers
  const blockers = await mcp__task-manager__task_blockers({ task_id })

  if (blockers.length > 0) {
    // Has dependencies - valid reason for being stuck
    // Do nothing
  } else {
    // No dependencies but still stuck - reset
    await mcp__task-manager__task_update({
      id: task_id,
      status: "pending"
    })

    await mcp__task-manager__context_add({
      task_id: task_id,
      context_type: "note",
      content: "Task was stuck, reset to pending for retry"
    })
  }
}

// Scenario 3: Cognitive tool failure
async function recoverCognitiveTool(tool_name) {
  console.error(`${tool_name} is unhealthy, attempting recovery`)

  switch (tool_name) {
    case "task-manager":
      // Critical - cannot function without it
      // Attempt restart, if fails, halt system
      break

    case "h2ce":
      // Important but not critical
      // Workers can operate without it, alert and continue
      break

    case "icarus":
      // Important but not critical
      // Workers lose learning capability, alert and continue
      break

    case "markovian-thinker":
      // Nice to have
      // Workers use simpler reasoning, alert and continue
      break
  }
}
```

---

## 9. Startup & Shutdown Procedures

### System Startup

```javascript
async function startSystem() {
  console.log("Starting Continuous Work System...")

  // 1. Verify cognitive tools available
  console.log("Checking cognitive tools...")
  await verifyTools()

  // 2. Start orchestrator
  console.log("Starting orchestrator...")
  const orchestrator = new Orchestrator()

  // 3. Start planning agent
  console.log("Starting planning agent...")
  orchestrator.planning_agent = new PlanningAgent(orchestrator)
  orchestrator.planning_agent.run()

  // 4. Start initial worker pool
  console.log("Starting worker pool (2 initial workers)...")
  for (let i = 0; i < 2; i++) {
    const worker = new WorkerAgent(`worker-${i}`, orchestrator)
    orchestrator.workers.push(worker)
    worker.run()
  }

  // 5. Start monitoring
  console.log("Starting health monitoring...")
  setInterval(() => monitorSystemHealth(), 60000)  // Every minute

  console.log("✓ Continuous Work System started successfully")
  console.log(`  Planning Agent: RUNNING`)
  console.log(`  Workers: ${orchestrator.workers.length} active`)
  console.log(`  Monitoring: ACTIVE`)
}
```

### Graceful Shutdown

```javascript
async function shutdownSystem() {
  console.log("Initiating graceful shutdown...")

  // 1. Stop accepting new tasks
  console.log("Stopping planning agent...")
  planning_agent.stop()

  // 2. Let workers finish current tasks
  console.log("Waiting for workers to complete current tasks...")
  await Promise.all(workers.map(w => w.finishCurrentTask()))

  // 3. Save state
  console.log("Saving system state...")
  await saveSystemState()

  // 4. Stop workers
  console.log("Stopping workers...")
  workers.forEach(w => w.stop())

  // 5. Close connections
  console.log("Closing cognitive tool connections...")
  await closeConnections()

  console.log("✓ System shutdown complete")
}
```

---

## 10. Summary: Always-On Operation

### System Guarantees

1. **✅ Never Idle (when work exists)**: Workers continuously query for tasks
2. **✅ Automatic Scaling**: Worker pool scales with workload
3. **✅ Conflict-Free**: Task locking prevents double work
4. **✅ Dependency-Aware**: Blockers prevent premature execution
5. **✅ Self-Healing**: Recovery procedures for failures
6. **✅ Health Monitored**: Continuous metrics and alerting
7. **✅ Graceful Degradation**: Works with reduced cognitive tools
8. **✅ Learning Continuous**: Icarus learns from every completion

### Continuous Operation Flow

```
User Request
     ↓
Planning Agent creates tasks
     ↓
Tasks enter queue (task-manager)
     ↓
Workers continuously pull tasks
     ↓
Workers execute using all cognitive tools
     ↓
Workers complete, update, learn
     ↓
Workers pull next task
     ↓
Repeat indefinitely
```

### The Promise

**This system provides true autonomous operation where:**
- Work is automatically discovered and decomposed
- Tasks are continuously executed in priority order
- All cognitive capabilities are leveraged
- System scales to workload
- Failures are recovered automatically
- Learning happens continuously
- Human intervention is only needed for high-level direction

**Claude Code becomes a truly autonomous development assistant that never stops working when there are tasks to complete.**
