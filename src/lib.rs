// Markovian Thinker: Efficient Chunk-Based Reasoning for LLMs (Stateful Mode)
// Implementation of Delethink paradigm from "The Markovian Thinker" (arXiv:2510.06557)
//
// Core innovation: Fixed-size reasoning chunks with bounded carryover state,
// achieving linear complexity O(n²S) instead of quadratic O(n²S²) scaling.
//
// This version provides stateful MCP tools for client-orchestrated reasoning,
// eliminating the need for the server to make its own API calls.

pub mod mcp;
pub mod gpu;
pub mod parallel;
pub mod inference;
pub mod training;

// Re-export core types for convenience
pub use mcp::MarkovianMCPServer;

#[cfg(feature = "gpu")]
pub use parallel::{ParallelExecutor, Task, TaskType, TaskResult};
#[cfg(feature = "gpu")]
pub use gpu::CudaContext;

pub use inference::{Tokenizer, EmbeddingLayer, InferenceModel, ModelConfig};
pub use training::{
    WeightLoader, WeightFormat,
    Optimizer, AdamOptimizer, SGDOptimizer, OptimizerConfig,
    BackpropEngine, OnlineLearner, LearningConfig,
};
