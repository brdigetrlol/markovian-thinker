// Markovian Thinker: Efficient Chunk-Based Reasoning for LLMs (Stateful Mode)
// Implementation of Delethink paradigm from "The Markovian Thinker" (arXiv:2510.06557)
//
// Core innovation: Fixed-size reasoning chunks with bounded carryover state,
// achieving linear complexity O(n²S) instead of quadratic O(n²S²) scaling.
//
// This version provides stateful MCP tools for client-orchestrated reasoning,
// eliminating the need for the server to make its own API calls.

pub mod chunk_manager;
pub mod mcp;
pub mod parser;
pub mod prompts;
pub mod session_manager;
pub mod state;
pub mod trace;
pub mod types;

// GPT-OSS inspired enhancements
pub mod attention;
pub mod experts;
pub mod sampling_strategies;

// Icarus TIC inspired enhancements
pub mod causal_trace;
pub mod event_queue;
pub mod events;
pub mod lattice;
pub mod concept_space;

// Event storm mitigation
pub mod rate_limit;
pub mod circuit_breaker;
pub mod event_fusion;
pub mod storm_mitigation;

// Phase 8: Icarus Integration - H2CE semantic search
pub mod h2ce_adapter;

// Phase 8: Icarus Integration - TodoWrite bridge
pub mod todo_bridge;

// Phase 8: Icarus Integration - Monte Carlo decision making
pub mod monte_carlo;

// Re-export core types for convenience
pub use mcp::MarkovianMCPServer;
pub use session_manager::{ReasoningSession, SessionInfo, SessionManager};
pub use state::{ChunkHistory, ChunkRecord, MarkovianState, StateConfig, TerminationInfo};
pub use trace::{
    ReasoningTrace, TerminationReason, TraceChunk, TraceDataset, TraceMetadata,
};

// Re-export GPT-OSS enhancements
pub use attention::{AttentionConfig, AttentionSink, GroupedAttention, SlidingWindowAttention};
pub use experts::{ExpertConfig, ExpertGating, ExpertType, ReasoningExpert};
pub use sampling_strategies::{AdaptiveSampling, SamplingConfig, SamplingStrategy};

// Re-export Icarus TIC enhancements
pub use causal_trace::{
    BranchState, CausalEvent, CausalTrace, CausalTraceMetadata, ReasoningBranch, TraceStatistics,
};
pub use event_queue::{EventQueue, QueueMetrics, SessionEventQueue};
pub use events::{
    CognitiveTimestamp, EventResult, EventWithMetadata, ReasoningEvent, ReasoningLevel,
};
pub use lattice::{
    create_generator, E8Generator, HCPGenerator, HypercubicGenerator, LatticeGenerator,
    LatticePoint, LatticeType, LeechGenerator,
};
pub use concept_space::{Concept, ConceptSpace, ConceptSpaceConfig, ConceptSpaceStatistics};

// Re-export storm mitigation components
pub use rate_limit::{RateLimiter, RateLimitConfig, RateLimiterStats, SessionRateLimiter};
pub use circuit_breaker::{
    CircuitBreaker, CircuitBreakerConfig, CircuitBreakerStats, CircuitState,
    SessionCircuitBreaker,
};
pub use event_fusion::{EventFusion, EventFusionConfig, FusionStats};
pub use storm_mitigation::{
    MitigationDecision, SessionStormMitigation, StormMetrics, StormMitigation,
    StormMitigationConfig, StormMitigationStats,
};

// Re-export Phase 8: Icarus Integration components
pub use h2ce_adapter::{H2CEAdapter, H2CEConfig, SearchResult};
pub use todo_bridge::{TodoBridge, TodoItem, TodoList, TodoStatus, TodoSummary};
pub use monte_carlo::{ActionChoice, MCTSNode, MonteCarloConfig, MonteCarloSampler};
