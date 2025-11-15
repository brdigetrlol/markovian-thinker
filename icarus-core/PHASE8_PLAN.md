# Phase 8: Icarus Integration - H2CE + Cognitive Task Manager

**Status**: Planning
**Goal**: Transform Markovian Thinker into complete Icarus TIC system
**Prerequisites**: Phase 7 complete ✅

## Overview

Phase 8 integrates three critical components to create the full Icarus TIC (Thought-Intensive Cognition) system:

1. **H2CE Integration** - Semantic search over codebase/corpus
2. **Cognitive Task Manager** - Task graph orchestration
3. **TodoWrite Bridge** - Native task management integration

This creates a unified reasoning system that can:
- Search semantic knowledge bases during reasoning
- Orchestrate complex multi-step tasks
- Track dependencies and progress in real-time
- Combine symbolic reasoning with semantic retrieval

---

## Phase 8 Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     ICARUS TIC SYSTEM                       │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐ │
│  │   H2CE       │    │  Markovian   │    │  Cognitive   │ │
│  │  Semantic    │◄──►│   Thinker    │◄──►│     Task     │ │
│  │   Search     │    │  Reasoning   │    │   Manager    │ │
│  └──────────────┘    └──────────────┘    └──────────────┘ │
│         ▲                   ▲                    ▲         │
│         │                   │                    │         │
│         └───────────────────┴────────────────────┘         │
│                      Event Queue                           │
│                   (Icarus TIC Core)                        │
└─────────────────────────────────────────────────────────────┘
```

---

## Task Breakdown

### Task 1: H2CE Client Integration
**Goal**: Enable semantic search during reasoning

**Subtasks**:
1. Add H2CE client to dependencies
2. Create `H2CEAdapter` for semantic search
3. Integrate with `ChunkManager` for mid-reasoning queries
4. Add MCP tool: `markovian_search_corpus`
5. Create integration tests

**New Configuration**:
```rust
pub struct H2CEConfig {
    pub enabled: bool,
    pub server_url: String,
    pub index_path: String,
    pub corpus_path: String,
    pub max_results: usize,
    pub similarity_threshold: f32,
}
```

**Expected Behavior**:
- Reasoning can pause to search semantic corpus
- Search results injected into carryover
- Automatic relevance filtering
- Event emission for search queries

**Files to Create/Modify**:
- `src/h2ce_adapter.rs` (new)
- `src/state.rs` (add H2CEConfig)
- `src/chunk_manager.rs` (integrate search)
- `src/mcp/server.rs` (add search tool)
- `tests/integration_test.rs` (H2CE tests)

**Performance Target**: < 100ms per search query

---

### Task 2: Cognitive Task Manager Integration
**Goal**: Orchestrate reasoning as task graph

**Subtasks**:
1. Add cognitive-task-manager client to dependencies
2. Create `TaskGraphAdapter` for task orchestration
3. Map reasoning chunks to task graph nodes
4. Propagate priorities and dependencies
5. Add MCP tools for task management

**New Configuration**:
```rust
pub struct TaskGraphConfig {
    pub enabled: bool,
    pub auto_decompose: bool,        // Automatically break down complex queries
    pub priority_propagation: bool,   // Propagate priorities through graph
    pub dependency_tracking: bool,    // Track task dependencies
}
```

**Task Graph Mapping**:
```
Problem Query → Root Task
    ├─ Chunk 1 → Subtask 1 (depends on root)
    ├─ Chunk 2 → Subtask 2 (depends on chunk 1)
    ├─ Chunk 3 → Subtask 3 (depends on chunk 2)
    └─ Solution → Complete Task
```

**Files to Create/Modify**:
- `src/task_graph_adapter.rs` (new)
- `src/chunk_manager.rs` (task integration)
- `src/mcp/server.rs` (task tools)
- `tests/integration_test.rs` (task graph tests)

**Performance Target**: < 1% overhead for task tracking

---

### Task 3: TodoWrite → Cognitive Task Manager Bridge
**Goal**: Make TodoWrite native to cognitive-task-manager

**Subtasks**:
1. Create bidirectional sync between TodoWrite and task graph
2. Map todo status to task states
3. Propagate updates in real-time
4. Visualize task graph via MCP
5. Support task priorities and dependencies

**Mapping**:
```
TodoWrite State  →  Task Graph State
────────────────────────────────────
pending          →  Pending
in_progress      →  InProgress
completed        →  Completed
blocked          →  Blocked (new)
failed           →  Failed (new)
```

**New MCP Tools**:
- `markovian_get_task_graph` - Export current task DAG
- `markovian_update_task` - Manually update task state
- `markovian_task_dependencies` - Query dependency chain

**Files to Create/Modify**:
- `src/todo_bridge.rs` (new)
- `src/task_graph_adapter.rs` (extend)
- `src/mcp/server.rs` (task graph tools)

**Performance Target**: < 10ms per todo sync

---

### Task 4: Semantic-Augmented Reasoning
**Goal**: Combine reasoning with semantic search automatically

**Subtasks**:
1. Detect when reasoning needs external knowledge
2. Automatically query H2CE for relevant context
3. Inject search results into carryover
4. Track search provenance in causal trace
5. Add search confidence scoring

**Heuristics for Auto-Search**:
- Query contains "find", "search", "lookup", "what is"
- Reasoning mentions unknown concepts
- Expert system requests domain knowledge
- Carryover lacks relevant context

**Example Flow**:
```
User: "Explain how the authentication middleware works"
  ↓
Chunk 1: "I need to understand the auth middleware..."
  ↓
[AUTO-SEARCH TRIGGERED]
  ↓
H2CE Query: "authentication middleware implementation"
  ↓
Results: [auth.rs:45-120, middleware.rs:200-250]
  ↓
Chunk 2: "Based on the code at auth.rs:45, the middleware..."
```

**Files to Create/Modify**:
- `src/semantic_reasoning.rs` (new)
- `src/chunk_manager.rs` (auto-search integration)
- `src/prompts.rs` (search-augmented prompts)

**Performance Target**: < 5% latency increase with auto-search

---

### Task 5: Event-Driven Task Orchestration
**Goal**: Full event-driven integration across all systems

**Subtasks**:
1. Extend event types for H2CE and task graph
2. Create event handlers for cross-system communication
3. Implement event-driven search triggers
4. Add task state change events
5. Build event replay for debugging

**New Event Types**:
```rust
pub enum ReasoningEvent {
    // Existing events
    ChunkRequest { ... },
    ChunkComplete { ... },

    // New events (Phase 8)
    SemanticSearchRequest { query: String, context: String },
    SemanticSearchComplete { results: Vec<SearchResult> },
    TaskCreated { task_id: Uuid, description: String },
    TaskStateChanged { task_id: Uuid, old_state: TaskState, new_state: TaskState },
    TaskDependencyAdded { task_id: Uuid, depends_on: Uuid },
}
```

**Files to Create/Modify**:
- `src/events.rs` (extend event types)
- `src/event_handlers.rs` (new)
- `src/event_queue.rs` (handler registration)

**Performance Target**: < 1ms per event handling

---

### Task 6: Unified MCP API
**Goal**: Expose all Icarus TIC features via MCP

**New MCP Tools**:

1. **`icarus_search_and_reason`**
   - Combined search + reasoning in one call
   - Auto-inject search results into reasoning

2. **`icarus_task_graph_create`**
   - Create task graph from problem description
   - Auto-decompose into subtasks

3. **`icarus_task_graph_visualize`**
   - Export task graph as GraphViz DOT
   - Show dependencies and states

4. **`icarus_event_stream`**
   - Stream reasoning events in real-time
   - For monitoring and debugging

5. **`icarus_replay_session`**
   - Replay event history
   - For debugging and analysis

**Files to Modify**:
- `src/mcp/server.rs` (add 5 new tools)
- `src/mcp/protocol.rs` (extend if needed)

---

### Task 7: Performance Profiling & Optimization
**Goal**: Ensure Icarus TIC system maintains < 5% total overhead

**Benchmarks to Create**:
- H2CE search latency
- Task graph update overhead
- Todo bridge sync latency
- Event handling throughput
- Full Icarus integration overhead

**Performance Targets**:
| Component              | Target Latency | Target Overhead |
|------------------------|----------------|-----------------|
| H2CE Search            | < 100ms        | N/A             |
| Task Graph Update      | < 1ms          | < 1%            |
| Todo Bridge Sync       | < 10ms         | < 0.5%          |
| Event Handling         | < 1ms          | < 1%            |
| **Total Overhead**     | -              | **< 5%**        |

**Files to Create**:
- `benches/phase8_benchmarks.rs`
- `PHASE8_PERFORMANCE.md`

---

## Integration Testing Strategy

### Test Categories

1. **H2CE Integration Tests**
   - Search during reasoning
   - Result injection into carryover
   - Auto-search triggering
   - Search provenance tracking

2. **Task Graph Tests**
   - Task creation from reasoning
   - Dependency tracking
   - Priority propagation
   - State transitions

3. **Todo Bridge Tests**
   - Bidirectional sync
   - State mapping
   - Real-time updates
   - Conflict resolution

4. **End-to-End Icarus Tests**
   - Full system integration
   - Search + Reasoning + Task tracking
   - Event-driven orchestration
   - Performance under load

---

## API Examples

### Search-Augmented Reasoning
```rust
use markovian_thinker::{ChunkManager, StateConfig, H2CEConfig};

let mut config = StateConfig::default();
config.h2ce_config = H2CEConfig {
    enabled: true,
    server_url: "http://localhost:8080".to_string(),
    index_path: ".h2ce_index".to_string(),
    corpus_path: "./src".to_string(),
    max_results: 5,
    similarity_threshold: 0.7,
};

let manager = ChunkManager::with_h2ce(config);
let trace = manager.generate_trace_with_search(
    "Explain the authentication flow".to_string(),
    &generator
).await?;
```

### Task Graph Orchestration
```rust
let mut config = StateConfig::default();
config.task_graph_config = TaskGraphConfig {
    enabled: true,
    auto_decompose: true,
    priority_propagation: true,
    dependency_tracking: true,
};

let manager = ChunkManager::with_task_graph(config);
let (trace, task_graph) = manager.generate_with_tasks(
    "Implement OAuth2 authentication".to_string(),
    &generator
).await?;

// Task graph automatically created:
// Root: "Implement OAuth2 authentication"
//   ├─ Task 1: "Research OAuth2 spec"
//   ├─ Task 2: "Design token flow" (depends on 1)
//   ├─ Task 3: "Implement endpoints" (depends on 2)
//   └─ Task 4: "Add tests" (depends on 3)
```

### Unified Icarus System
```rust
let config = StateConfig::icarus_default(); // Enables all features

let manager = ChunkManager::icarus(config);
let session = manager.create_icarus_session(
    "Debug the login bug".to_string()
).await?;

// Automatically:
// 1. Creates task graph
// 2. Enables semantic search
// 3. Tracks events
// 4. Syncs with TodoWrite
// 5. Records causal trace
```

---

## Dependencies to Add

### Cargo.toml additions
```toml
[dependencies]
# H2CE client (assuming it's a separate crate)
h2ce-client = { path = "../H2CE/client" }  # or version from crates.io

# Cognitive task manager (assuming it's a separate crate)
cognitive-task-manager = { path = "../cognitive-task-manager" }

# Additional dependencies for streaming
tokio-stream = "0.1"
futures = "0.3"
```

---

## Migration Path

### Backward Compatibility
All Phase 8 features are **optional** and **disabled by default**:

```rust
// Phase 7 code continues to work unchanged
let config = StateConfig::default();
let manager = ChunkManager::new(config);

// Opt-in to Icarus features
let config = StateConfig::icarus_default();
let manager = ChunkManager::icarus(config);
```

### Incremental Adoption
Users can enable features individually:
1. Just H2CE search
2. Just task graph
3. Just todo bridge
4. Full Icarus system

---

## Success Criteria

Phase 8 is complete when:

- ✅ H2CE search works during reasoning (< 100ms latency)
- ✅ Task graph tracks reasoning progress automatically
- ✅ TodoWrite syncs with cognitive-task-manager (< 10ms)
- ✅ Events connect all systems seamlessly
- ✅ 5 new MCP tools expose Icarus features
- ✅ Performance overhead < 5% (all features enabled)
- ✅ All integration tests pass (target: 200+ tests)
- ✅ Documentation complete with examples

---

## Risks & Mitigations

### Risk 1: H2CE/Task Manager Dependencies
**Risk**: External crates may not exist or may need modification
**Mitigation**:
- Check if H2CE and cognitive-task-manager are available
- May need to create client libraries if only servers exist
- Can stub with mock implementations for initial testing

### Risk 2: Performance Degradation
**Risk**: Adding 3 systems could exceed 5% overhead
**Mitigation**:
- Lazy initialization (only create clients when used)
- Async all the things (don't block reasoning)
- Aggressive caching of search results
- Batched task graph updates

### Risk 3: Complexity Explosion
**Risk**: Integration of 4 systems could be unmaintainable
**Mitigation**:
- Clear adapter pattern for each system
- Event-driven decoupling
- Comprehensive integration tests
- Excellent documentation

---

## Timeline Estimate

| Task | Estimated Effort | Dependencies |
|------|------------------|--------------|
| Task 1: H2CE Integration | 4-6 hours | H2CE client library |
| Task 2: Task Graph Integration | 4-6 hours | cognitive-task-manager |
| Task 3: TodoWrite Bridge | 2-3 hours | Task 2 |
| Task 4: Semantic Reasoning | 3-4 hours | Task 1 |
| Task 5: Event Orchestration | 3-4 hours | Tasks 1-2 |
| Task 6: Unified MCP API | 2-3 hours | All above |
| Task 7: Performance & Testing | 3-4 hours | All above |
| **Total** | **21-30 hours** | - |

---

## Next Steps

1. **Investigate Dependencies**
   - Check if H2CE client exists
   - Check if cognitive-task-manager is available
   - Determine if we need to create client libraries

2. **Start with Task 1**
   - Create H2CE adapter interface
   - Implement basic search integration
   - Add MCP tool for manual search

3. **Incremental Integration**
   - Build and test each component independently
   - Integrate via events
   - Comprehensive testing at each stage

---

**Phase 8 will transform Markovian Thinker into the complete Icarus TIC system!** 🚀
