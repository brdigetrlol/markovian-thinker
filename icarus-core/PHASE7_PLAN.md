# Phase 7: Event-Driven Reasoning & Intelligent Carryover

**Status**: 📋 Planning
**Duration**: 2-3 weeks
**Goal**: Connect event system to stateful reasoning and implement intelligent concept-based carryover

---

## Executive Summary

Phase 7 will complete the final major integrations by:
1. **Event-Driven Chunk Processing** - Connect Icarus TIC event system to ChunkManager
2. **Intelligent Carryover** - Use concept space for semantic carryover selection
3. **Real-Time Event Recording** - Populate causal traces during reasoning
4. **Performance Optimization** - Profile and optimize hot paths
5. **Advanced MCP Features** - Add batch processing and streaming support

---

## Current State Analysis

### ✅ Completed Infrastructure (Phases 1-6)

**Fully Implemented**:
- GPT-OSS: Experts, Attention, Sampling (1,540 lines, 28 tests)
- Events: Types, Queue, Levels (875 lines, 13 tests)
- Causal Traces: DAG, Branches, Queries (671 lines, 11 tests)
- Concept Space: Lattices, Similarity, Composition (912 lines, 21 tests)
- Storm Mitigation: Rate limiting, Circuit breaker (1,460 lines, 49 tests)
- MCP Integration: 7 tools, SessionManager (all working)

**Not Yet Connected**:
1. Event system → ChunkManager (events exist but not fired during reasoning)
2. Causal trace → Chunk processing (trace exists but not populated)
3. Concept space → Carryover selection (space exists but not queried)
4. Expert selection → Prompt generation (experts exist but not used)
5. Attention compression → Context management (attention exists but not applied)

---

## Phase 7 Tasks

### Task 1: Event-Driven Chunk Processing (4 hours)

**Goal**: Fire reasoning events during chunk orchestration and handle them in event queue.

**Subtasks**:
1. Modify `ChunkManager` to emit events:
   - `ChunkRequest` before sampling
   - `ChunkComplete` after sampling
   - `VerificationRequest` for solution checking
   - `VerificationComplete` after verification

2. Add event handler in `ChunkManager`:
   - Process events from queue
   - Update session state based on events
   - Spawn child events for complex tasks

3. Integration points:
   - Create `EventQueue` per session in `SessionManager`
   - Pass queue reference to `ChunkManager`
   - Process events asynchronously

**Deliverables**:
- Modified `ChunkManager` with event emission
- Event processing loop in chunk orchestration
- 3 new integration tests

**Success Criteria**:
- Events fired for every chunk
- Event queue processes events correctly
- No performance regression

---

### Task 2: Causal Trace Recording (3 hours)

**Goal**: Populate causal traces with actual reasoning events during chunk processing.

**Subtasks**:
1. Modify `ChunkManager` to record events:
   - Add events to causal trace after emission
   - Track causal dependencies (chunk N depends on chunk N-1)
   - Set appropriate reasoning levels (Micro/Meso/Macro)

2. Enhance `markovian_get_trace` endpoint:
   - Return causal structure with reasoning trace
   - Include event metadata (timestamps, levels, dependencies)
   - Support GraphViz export format

3. Add causal analysis queries:
   - Find critical path through reasoning
   - Identify failed branches
   - Calculate reasoning depth

**Deliverables**:
- Causal trace populated during reasoning
- Enhanced trace endpoint with causal data
- GraphViz export functionality
- 2 new integration tests

**Success Criteria**:
- Trace contains all chunk events
- Causal dependencies correctly recorded
- GraphViz output is valid DOT format

---

### Task 3: Intelligent Carryover Selection (5 hours)

**Goal**: Use concept space to semantically select most relevant carryover text.

**Current Behavior**:
- Last M tokens carried over (naive positional selection)
- May lose important context from earlier chunks

**New Behavior**:
1. **Crystallize Chunks**: Convert each chunk to concept lattice point
2. **Query Similarity**: Find most similar previous chunks to current problem
3. **Smart Selection**: Carry over text from semantically relevant chunks
4. **Adaptive Window**: Adjust carryover size based on relevance

**Implementation**:
1. Add concept crystallization in `ChunkManager`:
   - Generate embedding for each chunk (simple TF-IDF or count-based)
   - Crystallize to lattice point
   - Store in session's `ConceptSpace`

2. Modify carryover logic:
   - Query concept space for similar chunks
   - Extract relevant sections from top-k similar chunks
   - Combine with recency (80% relevance, 20% recency)

3. Add configuration:
   - `intelligent_carryover: bool` (default: false for backward compat)
   - `carryover_k: usize` (top-k similar chunks, default: 3)
   - `relevance_weight: f32` (0.8 = 80% semantic, 20% temporal)

**Deliverables**:
- Concept-based carryover selection
- Configurable via StateConfig
- Backward compatible (off by default)
- 4 new integration tests
- Performance benchmark

**Success Criteria**:
- Carryover more relevant than naive selection
- Performance overhead < 5%
- Maintains or improves solution quality

---

### Task 4: Expert-Guided Prompting (3 hours)

**Goal**: Use expert gating to customize prompts based on problem domain.

**Implementation**:
1. Modify `prompts::generate_prompt()`:
   - Call `ExpertGating::select()` on problem text
   - Customize prompt based on selected expert
   - Include domain-specific instructions

2. Expert-specific prompt templates:
   - **Math**: Include LaTeX formatting guidelines
   - **Code**: Include syntax highlighting, error checking
   - **Text**: Include structured reasoning format

3. Dynamic prompt adaptation:
   - Adjust based on chunk iteration (early: explore, late: verify)
   - Include previous expert's suggestions

**Deliverables**:
- Expert-guided prompt generation
- Domain-specific templates
- 3 new tests

**Success Criteria**:
- Prompts tailored to domain
- Expert selection < 2µs overhead
- Solution quality improves for math/code problems

---

### Task 5: Attention-Based Context Management (4 hours)

**Goal**: Use sliding window attention to compress context when exceeding limits.

**Implementation**:
1. Add attention scoring in `ChunkManager`:
   - Score each token in carryover for importance
   - Identify attention sinks (critical context)
   - Apply exponential decay for older chunks

2. Compression when needed:
   - If carryover > max size, compress
   - Keep high-importance tokens
   - Summarize low-importance regions

3. Hallucination filtering:
   - Use `AttentionSink` to filter unreliable content
   - Remove repetitive or low-relevance text

**Deliverables**:
- Attention-based compression
- Configurable via `AttentionConfig`
- 3 new tests

**Success Criteria**:
- Context stays within limits
- Critical information preserved
- Compression < 150µs overhead

---

### Task 6: Performance Profiling & Optimization (4 hours)

**Goal**: Profile Phase 7 changes and optimize hot paths.

**Activities**:
1. Benchmark all new features:
   - Event emission overhead
   - Causal trace recording
   - Concept crystallization
   - Expert selection
   - Attention compression

2. Identify bottlenecks:
   - Use criterion for micro-benchmarks
   - Use flamegraph for profiling
   - Target operations > 1ms

3. Optimize hot paths:
   - Cache concept embeddings
   - Batch event processing
   - Lazy causal trace updates

**Deliverables**:
- Performance benchmark suite
- Flamegraph analysis
- Optimization report
- Target: < 5% overhead

---

### Task 7: Advanced MCP Features (3 hours)

**Goal**: Add batch processing and streaming support to MCP API.

**New Tools**:

1. **`markovian_batch_init`** - Create multiple sessions
   ```json
   {
     "problems": ["Problem 1", "Problem 2", ...],
     "config": { /* shared config */ }
   }
   ```

2. **`markovian_stream_chunks`** - Stream chunk generation
   ```json
   {
     "session_id": "uuid",
     "stream": true
   }
   ```
   Returns: SSE (Server-Sent Events) stream of chunks

3. **`markovian_export_graphviz`** - Export causal trace as DOT
   ```json
   {
     "session_id": "uuid"
   }
   ```
   Returns: GraphViz DOT format

**Deliverables**:
- 3 new MCP tools
- 3 new integration tests
- Documentation updates

---

## Integration Testing Plan

### Test Suite Expansion

**New Integration Tests** (15 total):
1. `test_event_driven_chunk_processing` - Events fired during reasoning
2. `test_causal_trace_population` - Trace populated with real events
3. `test_intelligent_carryover_relevance` - Concept-based selection
4. `test_intelligent_carryover_performance` - Overhead < 5%
5. `test_expert_guided_prompts` - Domain-specific prompts
6. `test_attention_compression` - Context within limits
7. `test_batch_session_creation` - Multiple sessions
8. `test_event_queue_integration` - Queue processes events
9. `test_causal_dependencies` - Correct parent/child tracking
10. `test_concept_crystallization` - Chunks → lattice points
11. `test_graphviz_export` - Valid DOT output
12. `test_streaming_chunks` - SSE stream works
13. `test_phase7_full_integration` - All features together
14. `test_phase7_performance` - Overhead < 5%
15. `test_backward_compatibility` - Old configs still work

**Total Tests After Phase 7**: 182 (167 + 15)

---

## Performance Targets

| Feature | Max Overhead | Target Latency |
|---------|--------------|----------------|
| Event emission | < 1% | < 50 ns/event |
| Causal recording | < 1% | < 100 ns/event |
| Concept crystallization | < 3% | < 10 µs/chunk |
| Expert selection | < 0.5% | < 2 µs |
| Attention compression | < 2% | < 150 µs |
| **Total Phase 7** | **< 5%** | **< 200 µs/chunk** |

---

## Documentation Plan

### New Documentation Files

1. **`PHASE7_USER_GUIDE.md`** (500+ lines)
   - Event-driven reasoning guide
   - Intelligent carryover configuration
   - Expert-guided prompting examples
   - Performance tuning

2. **`PHASE7_PERFORMANCE.md`** (300+ lines)
   - Benchmark results
   - Profiling analysis
   - Optimization recommendations

3. **`EVENT_DRIVEN_REASONING.md`** (400+ lines)
   - Event system architecture
   - Event flow diagrams
   - Custom event handlers

4. **`INTELLIGENT_CARRYOVER.md`** (350+ lines)
   - Concept-based selection algorithm
   - Configuration guide
   - Performance comparison

**Total New Documentation**: 1,550+ lines

---

## Risk Assessment

### High Risk
- **Performance regression** from event overhead
  - Mitigation: Benchmark early, optimize hot paths
  - Rollback: Make event-driven optional (flag)

### Medium Risk
- **Complexity** of event-driven integration
  - Mitigation: Incremental implementation, test each step
  - Rollback: Keep synchronous path as fallback

- **Intelligent carryover** may not improve quality
  - Mitigation: A/B test against baseline
  - Rollback: Keep naive carryover as default

### Low Risk
- **MCP API changes** breaking clients
  - Mitigation: Backward compatible (new parameters optional)
  - Rollback: Version API (v1 vs v2)

---

## Success Criteria

### Functional
- [ ] Events fired for all chunk operations
- [ ] Causal traces populated during reasoning
- [ ] Intelligent carryover selects relevant context
- [ ] Expert-guided prompts tailored to domain
- [ ] Attention compression keeps context within limits
- [ ] All 182 tests passing

### Performance
- [ ] Total overhead < 5%
- [ ] Event emission < 50 ns
- [ ] Concept crystallization < 10 µs
- [ ] No memory leaks
- [ ] Throughput within 10% of baseline

### Quality
- [ ] Solution quality maintained or improved
- [ ] Backward compatible with Phase 6
- [ ] Documentation complete
- [ ] Examples updated

---

## Timeline

### Week 1: Core Event Integration
- **Days 1-2**: Task 1 (Event-driven chunk processing)
- **Days 3-4**: Task 2 (Causal trace recording)
- **Day 5**: Integration testing and fixes

### Week 2: Intelligent Features
- **Days 1-3**: Task 3 (Intelligent carryover)
- **Day 4**: Task 4 (Expert-guided prompts)
- **Day 5**: Task 5 (Attention compression)

### Week 3: Polish & Deploy
- **Days 1-2**: Task 6 (Performance profiling)
- **Day 3**: Task 7 (Advanced MCP features)
- **Days 4-5**: Documentation, testing, deployment

---

## Post-Phase 7 Vision

After Phase 7 completion, the Markovian Thinker will have:
- **Full event-driven reasoning** with Icarus TIC architecture
- **Intelligent context selection** using concept spaces
- **Domain-adaptive prompting** via expert gating
- **Efficient context compression** with attention
- **Complete observability** via causal traces
- **Production-grade performance** (< 5% overhead)

**Next phases could explore**:
- Multi-agent collaboration (multiple concurrent sessions)
- Reinforcement learning from reasoning outcomes
- Neural architecture search for optimal configurations
- Cross-session concept transfer learning

---

## Approval & Next Steps

**Ready to begin?**
- [ ] Review and approve Phase 7 plan
- [ ] Allocate 2-3 weeks for implementation
- [ ] Confirm performance targets acceptable
- [ ] Verify backward compatibility requirement

**Once approved, proceed with Task 1: Event-Driven Chunk Processing**

---

**Plan Status**: 📋 Awaiting Approval
**Estimated Duration**: 2-3 weeks
**Estimated LOC**: ~1,200 new lines
**Estimated Tests**: +15 integration tests
**Estimated Docs**: +1,550 lines
