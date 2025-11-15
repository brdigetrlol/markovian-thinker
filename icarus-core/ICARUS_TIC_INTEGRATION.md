# Icarus TIC Integration into Markovian Thinker
## Event-Driven Cognitive Architecture for Chunk-Based Reasoning

## Executive Summary

This document outlines the integration of Icarus TIC (Topological Information Crystallography) concepts into the Markovian Thinker, creating a unified system that combines:

1. **Markovian Thinker**: Linear-complexity chunk-based reasoning with bounded context
2. **GPT-OSS**: Mixture of Experts, sliding window attention, advanced sampling
3. **Icarus TIC**: Event-driven architecture, crystallographic concept spaces, causal traces

The result is a **hybrid cognitive architecture** that processes reasoning as a stream of events with hierarchical structure, maintaining mathematical rigor while achieving unprecedented flexibility and efficiency.

---

## Core Integration Concepts

### 1. Event-Driven Chunk Processing

**Problem**: Current Markovian implementation uses synchronous, sequential chunk generation.

**Solution**: Replace with asynchronous event-driven processing where:
- Chunk requests are **events** that trigger reasoning
- Reasoning chunks can spawn **sub-events** (clarifications, verifications)
- Processing is **non-blocking** and **parallel** where possible
- Events maintain **causal ordering** through timestamps

```rust
pub enum ReasoningEvent {
    ChunkRequest {
        session_id: Uuid,
        prompt: String,
        priority: f32,
        timestamp: u64,
    },
    ChunkComplete {
        session_id: Uuid,
        output: String,
        tokens: usize,
        spawned_events: Vec<ReasoningEvent>,
    },
    VerificationRequest {
        parent_event: Uuid,
        hypothesis: String,
    },
    ConceptCrystallization {
        concept: String,
        lattice_coords: Vec<f32>,
    },
}
```

### 2. Hierarchical Reasoning Levels

**Inspired by**: ICE v4's micro/meso/macro event hierarchy

**Implementation**: Three levels of reasoning granularity:

```rust
pub enum ReasoningLevel {
    /// Micro: Token-level attention and selection
    Micro {
        attention_scores: Vec<f32>,
        selected_tokens: Vec<usize>,
    },

    /// Meso: Sentence/paragraph-level reasoning steps
    Meso {
        reasoning_step: String,
        verification: VerificationResult,
    },

    /// Macro: Full chunk generation with carryover
    Macro {
        chunk: String,
        carryover: String,
        termination_check: bool,
    },
}
```

**Benefits**:
- **Micro** events stay on GPU/fast path (no CPU communication)
- **Meso** events enable mid-chunk verification and correction
- **Macro** events represent complete reasoning units

### 3. Causal Set Trace Management

**Problem**: Current traces are linear arrays; lose causal relationships between reasoning branches.

**Solution**: Maintain a **causal set** (partially ordered set) of reasoning events:

```rust
pub struct CausalTrace {
    /// Events with partial ordering
    events: HashMap<Uuid, ReasoningEvent>,

    /// Causal relationships: event_a → event_b
    causal_edges: Vec<(Uuid, Uuid)>,

    /// Timestamps for ordering
    timestamps: HashMap<Uuid, u64>,

    /// Reasoning branches
    branches: Vec<ReasoningBranch>,
}

impl CausalTrace {
    /// Check if event_a causally precedes event_b
    pub fn precedes(&self, a: Uuid, b: Uuid) -> bool;

    /// Get all events in causal past of event
    pub fn causal_past(&self, event: Uuid) -> Vec<Uuid>;

    /// Get causal future (events that depend on this one)
    pub fn causal_future(&self, event: Uuid) -> Vec<Uuid>;

    /// Visualize causal structure
    pub fn to_graphviz(&self) -> String;
}
```

### 4. Crystallographic Concept Spaces

**Inspired by**: TIC's lattice-based concept representation

**Concept**: Represent semantic concepts as points in high-dimensional lattice spaces:

```rust
pub struct ConceptLattice {
    /// Lattice type (E8, Leech, HCP, etc.)
    lattice_type: LatticeType,

    /// Dimension of the space
    dimension: usize,

    /// Concepts mapped to lattice points
    concepts: HashMap<String, LatticePoint>,

    /// Nearest neighbor index for similarity
    nn_index: KDTree<f32>,
}

pub enum LatticeType {
    /// E8 lattice: 8D, excellent for mathematical concepts
    E8,

    /// Leech lattice: 24D, optimal sphere packing
    Leech,

    /// Hypercubic: Variable dimension, simple structure
    Hypercubic(usize),

    /// HCP: Hexagonal close packing, good for hierarchies
    HCP(usize),
}

impl ConceptLattice {
    /// Crystallize concept to nearest lattice point
    pub fn crystallize(&self, embedding: &[f32]) -> LatticePoint;

    /// Find similar concepts via lattice distance
    pub fn find_similar(&self, concept: &str, k: usize) -> Vec<(String, f32)>;

    /// Compute concept composition via lattice operations
    pub fn compose(&self, a: &str, b: &str) -> Option<String>;
}
```

**Benefits**:
- **Exact distances**: Lattice structure provides precise similarity metrics
- **Composition**: Lattice addition/subtraction for concept algebra
- **Compression**: Quantization to lattice points reduces memory
- **Topology**: Betti numbers capture concept structure

### 5. Event Storm Mitigation

**Problem**: Runaway reasoning can spawn infinite events (e.g., endless verification loops).

**Solution**: Adaptive rate limiting and circuit breakers:

```rust
pub struct EventStormMitigation {
    /// Maximum events per second per session
    rate_limit: RateLimit,

    /// Circuit breaker for recursive patterns
    circuit_breaker: CircuitBreaker,

    /// Event fusion: combine similar pending events
    fusion_engine: EventFusion,

    /// Backpressure metrics
    metrics: StormMetrics,
}

impl EventStormMitigation {
    /// Check if event should be accepted
    pub fn should_accept(&mut self, event: &ReasoningEvent) -> bool {
        // Rate limiting
        if !self.rate_limit.check(event.session_id()) {
            return false;
        }

        // Circuit breaker for loops
        if self.circuit_breaker.is_loop_detected(event) {
            return false;
        }

        // Try to fuse with pending events
        if self.fusion_engine.can_fuse(event) {
            return false; // Will be fused
        }

        true
    }

    /// Detect reasoning loops
    pub fn detect_loop(&self, trace: &CausalTrace) -> Option<Vec<Uuid>>;
}
```

---

## Architectural Design

### System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                MARKOVIAN THINKER + TIC HYBRID                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │           EVENT-DRIVEN REASONING ENGINE                  │  │
│  │                                                          │  │
│  │  ┌────────────────────────────────────────────┐         │  │
│  │  │  Hierarchical Event Queue                  │         │  │
│  │  │  - Micro: Token selection (GPU-only)       │         │  │
│  │  │  - Meso: Reasoning steps (verification)    │         │  │
│  │  │  - Macro: Full chunks (carryover)          │         │  │
│  │  └────────────────────────────────────────────┘         │  │
│  │                                                          │  │
│  │  ┌────────────────────────────────────────────┐         │  │
│  │  │  GPT-OSS Expert Gating                     │         │  │
│  │  │  - MathExpert, CodeExpert, TextExpert      │         │  │
│  │  │  - Sliding window attention                │         │  │
│  │  │  - Adaptive sampling                        │         │  │
│  │  └────────────────────────────────────────────┘         │  │
│  │                                                          │  │
│  │  ┌────────────────────────────────────────────┐         │  │
│  │  │  Markovian State Manager                   │         │  │
│  │  │  - Query + Carryover buffer                │         │  │
│  │  │  - Token budget tracking                   │         │  │
│  │  │  - Termination detection                   │         │  │
│  │  └────────────────────────────────────────────┘         │  │
│  └──────────────────────────────────────────────────────────┘  │
│                           ↓                                     │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │           CRYSTALLOGRAPHIC CONCEPT SPACE                 │  │
│  │  - E8/Leech/HCP lattices for concept mapping            │  │
│  │  - Nearest neighbor search for similarity               │  │
│  │  - Concept composition via lattice algebra              │  │
│  └──────────────────────────────────────────────────────────┘  │
│                           ↓                                     │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │           CAUSAL SET TRACE MANAGEMENT                    │  │
│  │  - Partial ordering of reasoning events                 │  │
│  │  - Branch detection and merging                         │  │
│  │  - Cognitive time emergence                             │  │
│  └──────────────────────────────────────────────────────────┘  │
│                           ↓                                     │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │           EVENT STORM MITIGATION                         │  │
│  │  - Rate limiting per session                            │  │
│  │  - Circuit breakers for loops                           │  │
│  │  - Event fusion and coalescing                          │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Data Flow

```
User Request
     ↓
[ChunkRequest Event]
     ↓
Event Queue (Priority + Momentum)
     ↓
Expert Gating (Select MathExpert/CodeExpert/TextExpert)
     ↓
┌─────────────────────────────────────┐
│  Hierarchical Processing            │
│                                     │
│  Micro: Token attention             │
│    ↓                                │
│  Meso: Reasoning step               │
│    ↓                                │
│  Macro: Full chunk                  │
└─────────────────────────────────────┘
     ↓
Concept Crystallization (Map to Lattice)
     ↓
Causal Trace Update
     ↓
Storm Mitigation Check
     ↓
[ChunkComplete Event] + Spawned Events
     ↓
Termination Check → Continue or Complete
```

---

## Implementation Plan

### Phase 1: Event Infrastructure (Week 1)

**Goal**: Build event-driven foundation

1. **Event Types & Queue**
   - `reasoning_event.rs`: Define all event types
   - `event_queue.rs`: Lock-free priority queue with momentum
   - `event_processor.rs`: Main event loop and dispatcher

2. **Hierarchical Processing**
   - `hierarchical.rs`: Micro/Meso/Macro level definitions
   - Integrate with existing `ChunkManager`

3. **Testing**
   - Unit tests for event queue
   - Integration test: event-driven chunk generation

### Phase 2: Causal Traces (Week 2)

**Goal**: Replace linear traces with causal sets

1. **Causal Set Implementation**
   - `causal_trace.rs`: Core causal set data structure
   - Partial ordering algorithms
   - Branch detection

2. **Visualization**
   - GraphViz export
   - Interactive trace explorer

3. **Integration**
   - Update `ReasoningTrace` to use causal structure
   - MCP endpoint: `markovian_get_causal_trace`

### Phase 3: Concept Lattices (Week 3)

**Goal**: Crystallographic concept representation

1. **Lattice Implementation**
   - `lattice.rs`: E8, Leech, HCP generators (simplified from TIC)
   - Closest vector problem (CVP) solver
   - Lattice point quantization

2. **Concept Space**
   - `concept_space.rs`: Concept → Lattice mapping
   - Similarity search via lattice distance
   - Concept composition

3. **Integration**
   - Map chunk outputs to concept lattice
   - Use for carryover selection (lattice-based similarity)

### Phase 4: Storm Mitigation (Week 4)

**Goal**: Robust event management

1. **Rate Limiting**
   - `rate_limit.rs`: Token bucket algorithm
   - Per-session quotas

2. **Circuit Breakers**
   - `circuit_breaker.rs`: Loop detection
   - Automatic recovery

3. **Event Fusion**
   - `event_fusion.rs`: Combine similar pending events
   - Deduplication

### Phase 5: Integration & Testing (Week 5)

**Goal**: Full system integration

1. **End-to-End Testing**
   - Complex reasoning problems
   - Benchmark vs. baseline

2. **MCP API Updates**
   - New parameters for event-driven mode
   - Causal trace endpoints

3. **Documentation**
   - Architecture guide
   - API examples

---

## Benefits of Integration

### Performance

- **Parallelism**: Event-driven architecture enables concurrent chunk processing
- **Efficiency**: Hierarchical levels avoid unnecessary computation
- **Scalability**: Event queue handles high throughput

### Flexibility

- **Adaptive**: System behavior emerges from event interactions
- **Branching**: Support multiple reasoning paths simultaneously
- **Recovery**: Circuit breakers prevent runaway reasoning

### Mathematical Rigor

- **Causal Ordering**: Maintains logical dependencies
- **Lattice Precision**: Exact concept distances and operations
- **Verification**: Meso-level events enable mid-chunk validation

### Observability

- **Causal Traces**: Visualize reasoning structure
- **Event Metrics**: Monitor system health
- **Concept Maps**: Understand semantic relationships

---

## API Changes

### Enhanced `markovian_solve` Tool

```typescript
{
  "name": "markovian_solve",
  "parameters": {
    // Existing parameters
    "problem": "string (required)",
    "chunk_size": "number (default: 8192)",
    "carryover_size": "number (default: 4096)",
    "max_iterations": "number (default: 5)",

    // GPT-OSS parameters
    "enable_experts": "boolean (default: true)",
    "expert_mode": "auto|math|code|text (default: auto)",
    "sliding_window": "number|null (default: 256)",
    "attention_sink": "boolean (default: true)",
    "temperature": "number (default: 0.7)",
    "top_k": "number|null (default: null)",
    "top_p": "number|null (default: 0.9)",

    // NEW: Icarus TIC parameters
    "event_driven": "boolean (default: true)",
    "hierarchical_levels": "micro|meso|macro|all (default: all)",
    "enable_causal_trace": "boolean (default: true)",
    "lattice_type": "e8|leech|hcp|cubic (default: e8)",
    "enable_storm_mitigation": "boolean (default: true)",
    "max_events_per_second": "number (default: 100)",
  }
}
```

### New MCP Tools

```typescript
// Get causal trace with visualization
{
  "name": "markovian_get_causal_trace",
  "parameters": {
    "session_id": "string (required)",
    "format": "json|graphviz (default: json)"
  }
}

// Query concept lattice
{
  "name": "markovian_query_concepts",
  "parameters": {
    "concept": "string (required)",
    "k": "number (default: 10)",
    "lattice_type": "e8|leech|hcp|cubic (default: e8)"
  }
}

// Get event metrics
{
  "name": "markovian_event_metrics",
  "parameters": {
    "session_id": "string (required)"
  }
}
```

---

## Performance Expectations

### Event Processing

- **Throughput**: 100,000+ events/second
- **Latency**: <1ms for micro events, <10ms for meso, <100ms for macro
- **Concurrency**: 10+ reasoning sessions in parallel

### Memory Efficiency

- **Lattice Compression**: 100-1000x vs. dense embeddings
- **Event Queue**: O(1) amortized operations
- **Causal Trace**: Sparse graph representation

### Reasoning Quality

- **Accuracy**: +10-20% from hierarchical verification
- **Efficiency**: 20-30% fewer tokens via concept lattices
- **Robustness**: 50%+ reduction in runaway reasoning

---

## Migration Path

### Backwards Compatibility

- All existing `markovian_solve` calls work unchanged
- Event-driven mode is opt-in via `event_driven: true`
- Causal traces compatible with existing trace format

### Gradual Adoption

1. **Phase 1**: Enable event-driven processing only
2. **Phase 2**: Add hierarchical levels
3. **Phase 3**: Enable concept lattices
4. **Phase 4**: Full integration with storm mitigation

---

## References

1. **Markovian Thinker**: arXiv:2510.06557v1
2. **GPT-OSS**: https://www.projektjoe.com/blog/gptoss
3. **Icarus TIC**: Internal documentation
4. **Causal Sets**: Sorkin (1987), "Spacetime and Causal Sets"
5. **Lattice Theory**: Conway & Sloane, "Sphere Packings, Lattices and Groups"

---

## Success Metrics

- [x] GPT-OSS integration complete
- [ ] Event-driven architecture functional
- [ ] Causal traces replace linear traces
- [ ] Concept lattices integrated
- [ ] Storm mitigation prevents runaway reasoning
- [ ] Performance meets targets (100K events/sec)
- [ ] Full test coverage
- [ ] Documentation complete
