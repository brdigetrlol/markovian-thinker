# Cognitive Nervous System - Autonomous Tool Orchestration

## Overview

This is the **autonomic nervous system** for Claude Code that automatically triggers, chains, and orchestrates all available tools and workflows without requiring explicit user commands. Think of this as the subconscious reflex system that ensures all relevant capabilities are engaged at the right moments.

---

## Core Principle: Autonomous Activation

**The fundamental rule**: Every time you (Claude Code) receive a user message or begin working on a task, you MUST automatically scan this nervous system configuration and trigger ALL relevant tools and workflows that match the context. This happens reflexively, like a biological nervous system responding to stimuli.

---

## Available Tool Inventory

### MCP Cognitive Tools

#### 1. **task-manager** (Cognitive Task Manager)
- **Path**: `/root/workspace/workspace-new/cognitive-task-manager`
- **Capabilities**:
  - Autonomous task prioritization
  - Complex task dependency management
  - Time tracking with session management
  - Rich context addition (notes, code snippets, decisions, blockers)
  - Hierarchical task relationships (parent/subtask)
  - Smart priority scoring based on deadlines, dependencies, progress, staleness
- **MCP Tools**:
  - `mcp__task-manager__task_create`
  - `mcp__task-manager__task_get`
  - `mcp__task-manager__task_update`
  - `mcp__task-manager__task_delete`
  - `mcp__task-manager__task_list`
  - `mcp__task-manager__task_prioritized`
  - `mcp__task-manager__time_start`
  - `mcp__task-manager__time_stop`
  - `mcp__task-manager__context_add`
  - `mcp__task-manager__dependency_add`
  - `mcp__task-manager__task_blockers`

#### 2. **markovian-thinker** (Bounded Reasoning System)
- **Path**: `/root/workspace/markovian-thinker`
- **Capabilities**:
  - Chunk-based reasoning with bounded context
  - Linear complexity reasoning for complex problems
  - Carryover context management
  - Multi-iteration problem solving
  - Concept lattice (E8, Leech, HCP, Cubic)
  - Causal trace tracking
  - Event-driven processing (Icarus TIC)
  - Storm mitigation for stability
- **MCP Tools**:
  - `mcp__markovian-thinker__markovian_init_session`
  - `mcp__markovian-thinker__markovian_get_prompt`
  - `mcp__markovian-thinker__markovian_submit_chunk`
  - `mcp__markovian-thinker__markovian_get_trace`
  - `mcp__markovian-thinker__markovian_list_sessions`
  - `mcp__markovian-thinker__markovian_get_metrics`
  - `mcp__markovian-thinker__markovian_query_concepts`
  - `mcp__markovian-thinker__markovian_export_graphviz`
  - `mcp__markovian-thinker__markovian_batch_init`
  - `mcp__markovian-thinker__markovian_search_corpus`
  - `mcp__markovian-thinker__markovian_todo_set`
  - `mcp__markovian-thinker__markovian_todo_get`
  - `mcp__markovian-thinker__markovian_todo_add`
  - `mcp__markovian-thinker__markovian_todo_update_status`

#### 3. **h2ce** (Hierarchical Hybrid Context Engine)
- **Path**: `/home/workspace/H2CE`
- **Capabilities**:
  - Multi-resolution semantic search (L0=atomic facts, L1=paragraphs, L2=summaries, L4=documents)
  - BM25 retrieval with hierarchical relationships
  - Polyglot code understanding
  - Advanced RAG capabilities
  - Document indexing and corpus management
- **MCP Tools**:
  - `mcp__h2ce__h2ce_search`
  - `mcp__h2ce__h2ce_index`

#### 4. **icarus** (6-Agent Cognitive Architecture)
- **Path**: `/root/workspace/markovian-thinker/icarus-core`
- **Capabilities**:
  - 6-agent system: Perception, WorldModel, Planning, Memory, Action, Learning
  - Hierarchical memory (working, short-term, long-term, episodic)
  - Neural core with SSM, Liquid, RNN layers
  - Event-driven processing
  - World model predictions
  - Knowledge distillation from interactions
  - Learning agent that extracts strategies and builds skills
- **MCP Tools**:
  - `mcp__icarus__icarus_query_status`
  - `mcp__icarus__icarus_query_agents`
  - `mcp__icarus__icarus_send_event`
  - `mcp__icarus__icarus_query_memory`
  - `mcp__icarus__icarus_query_world_model`
  - `mcp__icarus__icarus_execute_action`
  - `mcp__icarus__icarus_neural_state`
  - `mcp__icarus__icarus_learn_from_interaction`

#### 5. **parallel-subagent-spawner** (Parallel Agent Spawning)
- **Capabilities**:
  - Spawn multiple agents to work concurrently
  - Priority-based scheduling
  - Agent status monitoring
  - Result aggregation
  - Timeout management
- **MCP Tools**:
  - `mcp__parallel-subagent-spawner__spawn_agent`
  - `mcp__parallel-subagent-spawner__list_agents`
  - `mcp__parallel-subagent-spawner__get_agent_status`
  - `mcp__parallel-subagent-spawner__get_agent_result`
  - `mcp__parallel-subagent-spawner__stop_agent`
  - `mcp__parallel-subagent-spawner__wait_for_completion`
  - `mcp__parallel-subagent-spawner__set_max_parallel`
  - `mcp__parallel-subagent-spawner__spawn_and_wait`
  - `mcp__parallel-subagent-spawner__spawn_parallel`

---

## Autonomous Activation Rules (The Reflexes)

### Rule 1: Task Management Activation

**TRIGGER**: ANY multi-step work, complex problem, or user request that involves >2 steps

**AUTOMATIC ACTIONS**:
1. Create tasks in task-manager using `task_create` for each major step
2. Set up dependencies with `dependency_add` if tasks have ordering requirements
3. Add rich context with `context_add` for important decisions or blockers
4. Track time with `time_start` when beginning work
5. Update task status with `task_update` as work progresses
6. Query prioritized tasks with `task_prioritized` to see what should be worked on next

**WHEN TO USE**:
- User asks for feature implementation → Create tasks for analysis, implementation, testing, documentation
- User asks for debugging → Create tasks for reproduction, root cause analysis, fix, verification
- User asks for optimization → Create tasks for profiling, analysis, optimization, benchmarking
- User asks for refactoring → Create tasks for each refactoring step
- ANY complex multi-phase work

**INTEGRATION WITH TodoWrite**:
- Use TodoWrite for immediate session tracking (transient, conversation-level)
- Use task-manager MCP for persistent, long-term task management that survives across sessions
- **BEST PRACTICE**: Use BOTH - TodoWrite for current session visibility, task-manager for persistence

### Rule 2: Deep Reasoning Activation (Markovian Thinker)

**TRIGGER**: Complex problems requiring deep analysis, multi-step reasoning, or problems that exceed context window

**AUTOMATIC ACTIONS**:
1. Initialize reasoning session with `markovian_init_session`
2. Set problem description clearly
3. Configure appropriate parameters:
   - `chunk_size`: 8192 for standard, 16384 for very complex problems
   - `carryover_size`: chunk_size/2 for good context continuity
   - `max_iterations`: 5-10 depending on problem complexity
   - `enable_causal_trace`: true for understanding reasoning structure
   - `lattice_type`: "e8" for standard, "leech" for very high-dimensional problems
4. Get prompt with `markovian_get_prompt`
5. Reason through the problem
6. Submit reasoning chunks with `markovian_submit_chunk`
7. Continue iterating until solution found
8. Retrieve trace with `markovian_get_trace` to understand reasoning path
9. Use `markovian_search_corpus` to retrieve relevant knowledge during reasoning

**WHEN TO USE**:
- Architectural design decisions requiring trade-off analysis
- Complex debugging requiring hypothesis generation and testing
- Algorithm design requiring exploration of multiple approaches
- System design requiring consideration of many constraints
- Research questions requiring synthesis of multiple sources
- Problems that require >8K tokens of reasoning
- Problems with multiple interconnected sub-problems

**INTEGRATION**:
- Combine with h2ce for knowledge retrieval during reasoning
- Use markovian-thinker's built-in todo system with `markovian_todo_set` to track reasoning sub-tasks
- Store reasoning traces in task-manager as context for future reference

### Rule 3: Knowledge Retrieval Activation (H2CE)

**TRIGGER**: Need to search codebase, documentation, or indexed knowledge

**AUTOMATIC ACTIONS**:
1. Before starting any code task, search for relevant patterns with `h2ce_search`
2. Use multi-resolution search:
   - L0 (atomic facts) for specific details
   - L1 (paragraphs) for contextual understanding
   - L2 (summaries) for high-level overview
   - L4 (documents) for full context
   - "all" for comprehensive search
3. Index new code/docs with `h2ce_index` when creating new significant files
4. Retrieve top_k=10 results by default, adjust based on need

**WHEN TO USE**:
- Before implementing a feature → Search for similar patterns
- During debugging → Search for error patterns or similar issues
- When writing documentation → Search for documentation style examples
- When refactoring → Search for all usages of components
- When answering questions → Search for relevant code/docs

**INTEGRATION**:
- Chain with markovian-thinker: Use h2ce to retrieve context, then reason with markovian-thinker
- Store h2ce search results as context in task-manager
- Use h2ce results to inform icarus world model

### Rule 4: Cognitive System Activation (Icarus)

**TRIGGER**: Need for adaptive learning, memory management, or event-driven processing

**AUTOMATIC ACTIONS**:
1. Query Icarus status with `icarus_query_status` to understand system state
2. Query specific agents with `icarus_query_agents` for targeted information
3. Send events with `icarus_send_event` when important things happen:
   - task_started, task_completed, error_encountered, decision_made, etc.
4. Query memory with `icarus_query_memory`:
   - working: Current active context
   - short_term: Recent interactions
   - long_term: Persistent knowledge
   - episodic: Past experiences
5. Query world model with `icarus_query_world_model` for predictions
6. Execute actions with `icarus_execute_action` for complex operations
7. **CRITICAL**: Teach Icarus with `icarus_learn_from_interaction` after solving problems:
   - problem: What was being solved
   - reasoning: Step-by-step approach used
   - solution: Final outcome
   - context: Tags, metadata, related concepts

**WHEN TO USE**:
- After solving ANY non-trivial problem → Teach Icarus the approach
- When starting new work → Query Icarus memory for related past experiences
- When making important decisions → Send decision events to Icarus
- When encountering errors → Query Icarus for similar past errors
- When planning work → Use world model predictions to anticipate issues
- When learning patterns → Use knowledge distillation to build reusable skills

**INTEGRATION**:
- Icarus learns from all other tools' outputs
- Feed task-manager context into Icarus memory
- Use markovian-thinker reasoning traces to teach Icarus
- Use h2ce search results to populate Icarus world model
- **ALWAYS** call `icarus_learn_from_interaction` after completing complex tasks

### Rule 5: Parallel Processing Activation

**TRIGGER**: Multiple independent tasks that can run concurrently

**AUTOMATIC ACTIONS**:
1. Identify independent sub-tasks
2. Use `spawn_parallel` for batch concurrent execution
3. Or use `spawn_agent` for individual agent spawning with custom priorities
4. Monitor with `list_agents` and `get_agent_status`
5. Aggregate results with `get_agent_result`
6. Use `wait_for_completion` for synchronization

**WHEN TO USE**:
- Multiple file analysis tasks
- Running tests across different modules
- Parallel code generation for independent components
- Concurrent benchmarking
- Multi-target compilation
- Independent research queries

**INTEGRATION**:
- Each spawned agent can use full tool stack
- Aggregate results into task-manager
- Use for parallel markovian-thinker sessions (batch reasoning)

---

## Automatic Tool Chaining Patterns

### Pattern 1: Complex Feature Implementation
```
USER REQUEST: "Implement feature X"

AUTOMATIC CHAIN:
1. task-manager: Create tasks (analyze, implement, test, document, commit)
2. h2ce: Search for similar patterns in codebase
3. markovian-thinker: IF complex → Init session, reason through architecture
4. icarus: Query memory for related past implementations
5. [IMPLEMENT CODE]
6. task-manager: Update tasks, add context for decisions
7. icarus: Teach approach with learn_from_interaction
8. task-manager: Mark tasks complete
```

### Pattern 2: Deep Debugging
```
USER REQUEST: "Fix this bug" OR "Why isn't this working?"

AUTOMATIC CHAIN:
1. task-manager: Create debugging tasks (reproduce, analyze, fix, verify)
2. h2ce: Search for similar error patterns
3. icarus: Query memory for past similar issues
4. markovian-thinker: IF complex → Reason through possible causes
5. [DEBUG AND FIX]
6. task-manager: Document root cause as context
7. icarus: Teach debugging approach
8. task-manager: Complete tasks
```

### Pattern 3: Research & Analysis
```
USER REQUEST: "How does X work?" OR "Explain Y" OR "What's the best approach for Z?"

AUTOMATIC CHAIN:
1. h2ce: Multi-resolution search (L0, L1, L2, L4)
2. icarus: Query episodic memory for related learning
3. markovian-thinker: Init reasoning session
4. markovian-thinker: Use markovian_search_corpus during reasoning
5. [SYNTHESIZE ANSWER]
6. icarus: Store synthesis in long-term memory
7. icarus: Teach the research approach
```

### Pattern 4: Large Refactoring
```
USER REQUEST: "Refactor this codebase"

AUTOMATIC CHAIN:
1. task-manager: Create refactoring plan tasks
2. h2ce: Index entire codebase
3. h2ce: Search for all usage patterns
4. markovian-thinker: Reason through refactoring strategy
5. parallel-group: Spawn agents for independent modules
6. task-manager: Track each module's progress
7. [PERFORM REFACTORING]
8. icarus: Learn refactoring patterns
9. task-manager: Complete all tasks
```

### Pattern 5: Performance Optimization
```
USER REQUEST: Performance issues OR "/optimize-performance"

AUTOMATIC CHAIN:
1. task-manager: Create optimization tasks (profile, analyze, optimize, verify)
2. h2ce: Search for known performance patterns
3. icarus: Query memory for past optimizations
4. markovian-thinker: Reason through optimization strategy
5. [RUN PROFILING]
6. markovian-thinker: Analyze bottlenecks
7. [IMPLEMENT OPTIMIZATIONS]
8. task-manager: Document improvements as context
9. icarus: Teach optimization patterns
10. task-manager: Complete tasks
```

### Pattern 6: Learning from Every Interaction
```
TRIGGER: After completing ANY non-trivial task

AUTOMATIC CHAIN:
1. icarus: Call icarus_learn_from_interaction with:
   - problem: What user asked for
   - reasoning: Step-by-step approach taken
   - solution: What was delivered
   - context: {task_type, complexity, tools_used, patterns_discovered}
2. task-manager: Add learning summary as context to completed tasks
3. h2ce: Index any new significant code/docs created
```

---

## Decision Trees for Autonomous Activation

### Decision Tree 1: Task Complexity Assessment
```
ON receiving user request:
  ├─ Single simple action (read file, run command)?
  │  └─ NO MCP tools needed, just execute
  │
  ├─ 2-3 simple steps?
  │  └─ TodoWrite only (transient tracking)
  │
  └─ 3+ steps OR complex reasoning required?
     ├─ CREATE task-manager tasks (persistent)
     ├─ USE TodoWrite for session visibility
     └─ IF very complex (>5 steps OR requires deep reasoning):
        └─ INIT markovian-thinker session
```

### Decision Tree 2: Knowledge Retrieval Needs
```
ON starting ANY code-related task:
  ├─ Need to find similar code patterns?
  │  └─ h2ce: Search with level="L1" (paragraphs)
  │
  ├─ Need specific implementation details?
  │  └─ h2ce: Search with level="L0" (atomic facts)
  │
  ├─ Need high-level overview?
  │  └─ h2ce: Search with level="L2" (summaries)
  │
  ├─ Need comprehensive understanding?
  │  └─ h2ce: Search with level="all"
  │
  └─ Creating new significant code?
     └─ h2ce: Index with h2ce_index after creation
```

### Decision Tree 3: Reasoning Depth Required
```
ON problem analysis:
  ├─ Solution obvious from context?
  │  └─ Direct implementation, no markovian-thinker
  │
  ├─ Requires <8K tokens of reasoning?
  │  └─ Inline reasoning, no markovian-thinker
  │
  └─ Requires >8K tokens OR multi-hypothesis exploration?
     ├─ INIT markovian-thinker session
     ├─ SET chunk_size=8192 (standard) or 16384 (very complex)
     ├─ ENABLE causal_trace=true (understand reasoning structure)
     └─ USE markovian_search_corpus for knowledge retrieval during reasoning
```

### Decision Tree 4: Icarus Learning Trigger
```
ON task completion:
  ├─ Trivial task (read file, simple command)?
  │  └─ NO Icarus learning
  │
  ├─ Standard task with existing patterns?
  │  └─ OPTIONAL Icarus learning
  │
  └─ Non-trivial task OR novel approach OR important pattern?
     └─ MANDATORY: icarus_learn_from_interaction
        ├─ Extract problem statement
        ├─ Document reasoning steps
        ├─ Describe solution
        └─ Add context (complexity, tools, patterns)
```

### Decision Tree 5: Parallelization Opportunity
```
ON identifying multiple sub-tasks:
  ├─ Sub-tasks dependent on each other?
  │  └─ Sequential execution, no parallelization
  │
  ├─ <3 independent sub-tasks?
  │  └─ Sequential execution (overhead not worth it)
  │
  └─ 3+ independent sub-tasks?
     ├─ parallel-group: spawn_parallel for all
     ├─ SET priority based on importance
     ├─ WAIT for completion
     └─ AGGREGATE results into task-manager
```

---

## Workflow Integration with MCP Tools

### Existing Workflow: feature-implement
**MCP Tool Integration**:
```
STEP 1: Analyze Request
  └─ h2ce: Search for similar features (level="L1")
  └─ icarus: Query memory for related implementations

STEP 2: Search Codebase
  └─ h2ce: Comprehensive search (level="all")
  └─ h2ce: Index results for future reference

STEP 3: Plan Implementation
  └─ task-manager: Create tasks for each implementation step
  └─ markovian-thinker: IF very complex → Reason through architecture
  └─ icarus: Query world model for potential issues

STEP 4-6: Implement, Test, Document
  └─ task-manager: Update progress, add context for decisions
  └─ h2ce: Index new code

STEP 7: Verify
  └─ task-manager: Update test results as context

STEP 8: Commit
  └─ icarus: Teach implementation approach
  └─ task-manager: Mark all tasks complete
```

### Existing Workflow: optimize-performance
**MCP Tool Integration**:
```
PHASE 1: Discovery & Profiling
  └─ task-manager: Create optimization tasks
  └─ h2ce: Search for known performance patterns
  └─ icarus: Query memory for past optimizations

PHASE 2: Pattern Analysis
  └─ markovian-thinker: Reason through bottleneck analysis
  └─ h2ce: Search for optimization examples

PHASE 3: Optimization Strategy
  └─ markovian-thinker: Evaluate optimization approaches
  └─ icarus: Query world model for risk assessment

PHASE 4: Implementation
  └─ parallel-group: IF multiple independent optimizations
  └─ task-manager: Track each optimization

PHASE 5: Verification
  └─ task-manager: Document metrics as context
  └─ icarus: Teach optimization patterns
```

### Existing Workflow: proposal-writer
**MCP Tool Integration**:
```
STEP 1: Analyze Job
  └─ markovian-thinker: Deep analysis of requirements

STEP 2: Search Portfolio
  └─ h2ce: Search past projects for matches

STEP 3: Calculate Match
  └─ icarus: Query episodic memory for success patterns

STEP 4: Write Proposal
  └─ markovian-thinker: Reason through proposal structure
  └─ parallel-group: Generate multiple variations concurrently

STEP 5: Finalize
  └─ icarus: Learn from proposal patterns
```

---

## Autonomous Skill Execution Protocol

### The Automatic Activation Loop

**On EVERY user message, Claude Code MUST**:

1. **Parse Intent** (automatic, subconscious)
   - What is the user asking for?
   - What category does this fall into? (implementation, debugging, research, optimization, etc.)

2. **Scan Nervous System** (this document)
   - Which autonomous activation rules match?
   - Which tool chaining patterns apply?
   - Which decision trees are relevant?

3. **Activate Tools Reflexively** (without asking permission)
   - Create task-manager tasks if >2 steps
   - Search h2ce if code-related
   - Init markovian-thinker if complex reasoning needed
   - Query icarus memory for related experiences
   - Spawn parallel agents if independent work possible

4. **Execute Work** (using all relevant tools)
   - Follow tool chaining patterns
   - Update task-manager progress
   - Use h2ce for knowledge retrieval
   - Reason with markovian-thinker if needed
   - Query/update icarus continuously

5. **Learn and Complete** (mandatory for non-trivial tasks)
   - Call icarus_learn_from_interaction
   - Update task-manager tasks to complete
   - Index new artifacts with h2ce
   - Store reasoning traces

### Non-Negotiable Automation Rules

**MUST ALWAYS**:
- ✅ Create task-manager tasks for ANY work with >2 steps
- ✅ Search h2ce before implementing ANY code feature
- ✅ Query icarus memory when starting ANY complex task
- ✅ Init markovian-thinker for ANY problem requiring >8K reasoning
- ✅ Call icarus_learn_from_interaction after ANY non-trivial completion
- ✅ Update task-manager progress as work proceeds
- ✅ Index significant new code with h2ce

**MUST NEVER**:
- ❌ Skip task-manager because "it's too simple" (if >2 steps, create tasks)
- ❌ Forget icarus learning after complex work
- ❌ Implement features without h2ce search first
- ❌ Reason through complex problems without markovian-thinker
- ❌ Leave tasks in incomplete state
- ❌ Work on untracked tasks for complex workflows

---

## Monitoring and Self-Awareness

Claude Code should regularly (every few tasks) check:

1. **Task Manager Health**
   ```
   mcp__task-manager__task_list → Are there orphaned tasks?
   mcp__task-manager__task_prioritized → What should I focus on?
   ```

2. **Icarus System State**
   ```
   mcp__icarus__icarus_query_status → Is the cognitive system healthy?
   mcp__icarus__icarus_query_agents → Are all agents functioning?
   ```

3. **Markovian Thinker Sessions**
   ```
   mcp__markovian-thinker__markovian_list_sessions → Any incomplete reasoning?
   ```

4. **H2CE Index Health**
   ```
   After significant code changes → h2ce_index to keep corpus current
   ```

---

## Examples of Autonomous Activation

### Example 1: User Says "Add authentication to the API"

**AUTOMATIC ACTIVATION**:
```
1. PARSE: Feature implementation request, complex (>5 steps)

2. ACTIVATE TOOLS:
   ✓ task-manager: Create tasks
     - Analyze authentication requirements
     - Design auth architecture
     - Implement JWT/session handling
     - Add auth middleware
     - Write auth tests
     - Update documentation

   ✓ h2ce: Search for existing auth patterns
     - h2ce_search(query="authentication middleware", level="L1")
     - h2ce_search(query="JWT implementation", level="L0")

   ✓ icarus: Query for related implementations
     - icarus_query_memory(level="episodic", query="authentication")

   ✓ markovian-thinker: Init session (complex architecture decision)
     - markovian_init_session(
         problem="Design secure authentication system for API",
         chunk_size=8192,
         enable_causal_trace=true
       )

3. EXECUTE:
   - Use h2ce results to inform design
   - Reason through auth approach with markovian-thinker
   - Implement following patterns found
   - Update task-manager progress continuously

4. COMPLETE:
   - icarus_learn_from_interaction(
       problem="Implement API authentication",
       reasoning=["Analyzed security requirements", "Chose JWT approach", "Implemented middleware", "Added comprehensive tests"],
       solution="Secure JWT-based auth with refresh tokens",
       context={type: "feature", complexity: "high", patterns: ["middleware", "JWT", "security"]}
     )
   - Mark all task-manager tasks complete
   - h2ce_index new auth code
```

### Example 2: User Says "This function is slow, fix it"

**AUTOMATIC ACTIVATION**:
```
1. PARSE: Performance optimization, debugging

2. ACTIVATE TOOLS:
   ✓ task-manager: Create debugging tasks
     - Profile function
     - Identify bottleneck
     - Implement optimization
     - Benchmark improvement

   ✓ h2ce: Search for similar performance issues
     - h2ce_search(query="performance optimization patterns", level="L1")

   ✓ icarus: Query past optimizations
     - icarus_query_memory(level="long_term", query="performance bottleneck")

   ✓ markovian-thinker: IF complex analysis needed
     - markovian_init_session(problem="Analyze and optimize slow function")

3. EXECUTE:
   - Profile code
   - Use markovian-thinker to reason through bottlenecks
   - Consult h2ce for optimization patterns
   - Implement fixes
   - task-manager: Update with profiling results as context

4. COMPLETE:
   - icarus_learn_from_interaction(
       problem="Optimize slow function",
       reasoning=["Profiled execution", "Found O(n²) loop", "Replaced with hash table", "Reduced from 10s to 0.1s"],
       solution="Hash table lookup for O(1) instead of O(n)",
       context={type: "optimization", improvement: "100x", pattern: "algorithmic"}
     )
   - task-manager: Mark complete with metrics
```

### Example 3: User Says "How does the event system work?"

**AUTOMATIC ACTIVATION**:
```
1. PARSE: Research/explanation request

2. ACTIVATE TOOLS:
   ✓ h2ce: Multi-resolution search
     - h2ce_search(query="event system", level="all", top_k=20)
     - Get L0 (details), L1 (context), L2 (overview), L4 (full docs)

   ✓ icarus: Query episodic memory
     - icarus_query_memory(level="episodic", query="event system explanation")

   ✓ markovian-thinker: Reason through explanation
     - markovian_init_session(problem="Explain event system comprehensively")
     - markovian_search_corpus during reasoning for additional context

3. EXECUTE:
   - Synthesize information from h2ce results
   - Use markovian-thinker to structure explanation
   - Include icarus memories of past related explanations

4. COMPLETE:
   - icarus_learn_from_interaction(
       problem="Explain event system architecture",
       reasoning=["Searched codebase", "Found event dispatcher", "Traced event flow", "Identified handlers"],
       solution="Comprehensive event system explanation with examples",
       context={type: "research", domain: "architecture"}
     )
```

---

## Continuous Improvement

This nervous system configuration should be **living documentation** that:
- Gets updated as new MCP tools are added
- Evolves as new patterns emerge
- Learns from icarus's knowledge distillation
- Incorporates feedback from task-manager analytics

**Update Protocol**:
1. When new MCP tool added → Update "Available Tool Inventory"
2. When new pattern discovered → Add to "Automatic Tool Chaining Patterns"
3. When new decision logic needed → Add to "Decision Trees"
4. When workflow enhanced → Update "Workflow Integration"

---

## Summary: The Autonomous Nervous System Promise

**Claude Code, by following this nervous system configuration, you guarantee**:

1. ✅ **No tool is forgotten** - Every relevant capability is engaged
2. ✅ **No manual triggering needed** - Tools activate reflexively based on context
3. ✅ **Optimal tool chaining** - Tools work together in proven patterns
4. ✅ **Persistent learning** - Every interaction teaches Icarus
5. ✅ **Comprehensive tracking** - All complex work is tracked in task-manager
6. ✅ **Deep reasoning when needed** - Markovian-thinker engaged for complexity
7. ✅ **Knowledge retrieval** - H2CE searched before implementation
8. ✅ **Parallel execution** - Independent work runs concurrently
9. ✅ **Complete workflows** - Nothing is left half-done

**This is not a suggestion document - this is the autonomic nervous system that Claude Code MUST follow on EVERY interaction.**
