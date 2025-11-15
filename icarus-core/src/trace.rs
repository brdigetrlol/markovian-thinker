// Markovian Thinker: Trajectory/Trace Management
// Tracks multi-chunk reasoning traces: τ = [(x₁,y₁), ..., (x_L,y_L)]

use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

/// A complete reasoning trajectory across multiple chunks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningTrace {
    /// Unique trace ID
    pub id: Uuid,

    /// Original problem/query
    pub problem: String,

    /// All chunks in the trace
    pub chunks: Vec<TraceChunk>,

    /// Final answer/solution (extracted from last chunk or [EOS])
    pub solution: Option<String>,

    /// Whether the trace completed successfully
    pub completed: bool,

    /// Termination reason
    pub termination_reason: TerminationReason,

    /// Total tokens across all chunks
    pub total_tokens: usize,

    /// Metadata
    pub metadata: TraceMetadata,
}

/// Single chunk within a trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceChunk {
    /// Chunk number (1-indexed)
    pub index: usize,

    /// Input prompt (query + carryover from previous chunk)
    pub prompt: String,

    /// Generated output
    pub output: String,

    /// Tokens in this chunk
    pub tokens: usize,

    /// Timestamp when chunk was generated
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Latency in milliseconds
    pub latency_ms: u64,
}

/// Why the trace terminated
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TerminationReason {
    /// Reached [EOS] or solution marker
    SolutionFound,

    /// Hit max iteration limit
    MaxIterations,

    /// Exceeded token budget
    TokenBudgetExceeded,

    /// Error during generation
    Error(String),

    /// User/system interrupt
    Interrupted,
}

/// Metadata about trace generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceMetadata {
    /// When trace started
    pub start_time: chrono::DateTime<chrono::Utc>,

    /// When trace ended
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,

    /// Total wall-clock time (ms)
    pub total_time_ms: Option<u64>,

    /// Model used (e.g., "claude-sonnet-4-5", "gpt-4")
    pub model: String,

    /// Configuration used
    pub chunk_size: usize,
    pub carryover_size: usize,
    pub max_iterations: usize,

    /// Reward/correctness (for future RL)
    pub reward: Option<f32>,
}

impl ReasoningTrace {
    /// Create a new trace
    pub fn new(problem: String, model: String, chunk_size: usize, carryover_size: usize, max_iterations: usize) -> Self {
        Self {
            id: Uuid::new_v4(),
            problem,
            chunks: Vec::new(),
            solution: None,
            completed: false,
            termination_reason: TerminationReason::Interrupted,
            total_tokens: 0,
            metadata: TraceMetadata {
                start_time: chrono::Utc::now(),
                end_time: None,
                total_time_ms: None,
                model,
                chunk_size,
                carryover_size,
                max_iterations,
                reward: None,
            },
        }
    }

    /// Add a chunk to the trace
    pub fn add_chunk(&mut self, prompt: String, output: String, tokens: usize, latency_ms: u64) {
        let chunk = TraceChunk {
            index: self.chunks.len() + 1,
            prompt,
            output,
            tokens,
            timestamp: chrono::Utc::now(),
            latency_ms,
        };

        self.total_tokens += tokens;
        self.chunks.push(chunk);
    }

    /// Mark trace as complete
    pub fn complete(&mut self, solution: Option<String>, reason: TerminationReason) {
        self.completed = true;
        self.solution = solution;
        self.termination_reason = reason;
        self.metadata.end_time = Some(chrono::Utc::now());

        // Calculate total time
        if let Some(end_time) = self.metadata.end_time {
            let duration = end_time
                .signed_duration_since(self.metadata.start_time)
                .num_milliseconds();
            self.metadata.total_time_ms = Some(duration.max(0) as u64);
        }
    }

    /// Check if trace is complete
    pub fn is_complete(&self) -> bool {
        self.completed
    }

    /// Get full reasoning chain (all chunk outputs concatenated)
    pub fn full_reasoning(&self) -> String {
        self.chunks
            .iter()
            .map(|c| c.output.as_str())
            .collect::<Vec<_>>()
            .join("\n\n--- CHUNK BOUNDARY ---\n\n")
    }

    /// Calculate average tokens per chunk
    pub fn avg_tokens_per_chunk(&self) -> f64 {
        if self.chunks.is_empty() {
            0.0
        } else {
            self.total_tokens as f64 / self.chunks.len() as f64
        }
    }

    /// Calculate tokens per second
    pub fn tokens_per_second(&self) -> f64 {
        if let Some(total_time_ms) = self.metadata.total_time_ms {
            if total_time_ms > 0 {
                return (self.total_tokens as f64 * 1000.0) / total_time_ms as f64;
            }
        }
        0.0
    }

    /// Export trace to JSON file
    pub fn save_json<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load trace from JSON file
    pub fn load_json<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let trace = serde_json::from_str(&json)?;
        Ok(trace)
    }

    /// Get summary string
    pub fn summary(&self) -> String {
        format!(
            "Trace {} | {} chunks | {} tokens | {:.1} tok/sec | {} | Solution: {}",
            self.id,
            self.chunks.len(),
            self.total_tokens,
            self.tokens_per_second(),
            match self.termination_reason {
                TerminationReason::SolutionFound => "✓ SOLVED",
                TerminationReason::MaxIterations => "⏱ MAX_ITER",
                TerminationReason::TokenBudgetExceeded => "⚠ BUDGET",
                TerminationReason::Error(_) => "✗ ERROR",
                TerminationReason::Interrupted => "⏸ INTERRUPTED",
            },
            self.solution.as_deref().unwrap_or("N/A")
        )
    }
}

/// Collection of traces for batch analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceDataset {
    pub traces: Vec<ReasoningTrace>,
    pub metadata: DatasetMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetMetadata {
    pub name: String,
    pub created: chrono::DateTime<chrono::Utc>,
    pub total_traces: usize,
    pub total_tokens: usize,
}

impl TraceDataset {
    pub fn new(name: String) -> Self {
        Self {
            traces: Vec::new(),
            metadata: DatasetMetadata {
                name,
                created: chrono::Utc::now(),
                total_traces: 0,
                total_tokens: 0,
            },
        }
    }

    pub fn add_trace(&mut self, trace: ReasoningTrace) {
        self.metadata.total_tokens += trace.total_tokens;
        self.metadata.total_traces += 1;
        self.traces.push(trace);
    }

    /// Calculate average tokens per trace
    pub fn avg_tokens(&self) -> f64 {
        if self.traces.is_empty() {
            0.0
        } else {
            self.metadata.total_tokens as f64 / self.traces.len() as f64
        }
    }

    /// Count successful solutions
    pub fn success_rate(&self) -> f64 {
        if self.traces.is_empty() {
            0.0
        } else {
            let successes = self.traces.iter()
                .filter(|t| t.termination_reason == TerminationReason::SolutionFound)
                .count();
            successes as f64 / self.traces.len() as f64
        }
    }

    /// Export dataset to JSON
    pub fn save_json<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load dataset from JSON
    pub fn load_json<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let dataset = serde_json::from_str(&json)?;
        Ok(dataset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_creation() {
        let trace = ReasoningTrace::new(
            "What is 2+2?".to_string(),
            "test-model".to_string(),
            8192,
            4096,
            5,
        );

        assert!(!trace.completed);
        assert_eq!(trace.chunks.len(), 0);
        assert_eq!(trace.total_tokens, 0);
        assert_eq!(trace.problem, "What is 2+2?");
    }

    #[test]
    fn test_add_chunks() {
        let mut trace = ReasoningTrace::new(
            "Test".to_string(),
            "model".to_string(),
            100,
            50,
            3,
        );

        trace.add_chunk("Prompt 1".to_string(), "Output 1".to_string(), 10, 100);
        trace.add_chunk("Prompt 2".to_string(), "Output 2".to_string(), 15, 150);

        assert_eq!(trace.chunks.len(), 2);
        assert_eq!(trace.total_tokens, 25);
        assert_eq!(trace.chunks[0].index, 1);
        assert_eq!(trace.chunks[1].index, 2);
    }

    #[test]
    fn test_trace_completion() {
        let mut trace = ReasoningTrace::new(
            "Test".to_string(),
            "model".to_string(),
            100,
            50,
            3,
        );

        trace.add_chunk("P1".to_string(), "O1".to_string(), 10, 100);
        trace.complete(Some("Solution!".to_string()), TerminationReason::SolutionFound);

        assert!(trace.completed);
        assert_eq!(trace.solution, Some("Solution!".to_string()));
        assert_eq!(trace.termination_reason, TerminationReason::SolutionFound);
        assert!(trace.metadata.end_time.is_some());
        assert!(trace.metadata.total_time_ms.is_some());
    }

    #[test]
    fn test_full_reasoning() {
        let mut trace = ReasoningTrace::new(
            "Test".to_string(),
            "model".to_string(),
            100,
            50,
            3,
        );

        trace.add_chunk("P1".to_string(), "First output".to_string(), 10, 100);
        trace.add_chunk("P2".to_string(), "Second output".to_string(), 10, 100);

        let full = trace.full_reasoning();
        assert!(full.contains("First output"));
        assert!(full.contains("Second output"));
        assert!(full.contains("--- CHUNK BOUNDARY ---"));
    }

    #[test]
    fn test_trace_metrics() {
        let mut trace = ReasoningTrace::new(
            "Test".to_string(),
            "model".to_string(),
            100,
            50,
            3,
        );

        trace.add_chunk("P1".to_string(), "O1".to_string(), 100, 100);
        trace.add_chunk("P2".to_string(), "O2".to_string(), 200, 200);

        assert_eq!(trace.avg_tokens_per_chunk(), 150.0);

        trace.metadata.total_time_ms = Some(1000);
        assert_eq!(trace.tokens_per_second(), 300.0); // 300 tokens in 1 second
    }

    #[test]
    fn test_dataset() {
        let mut dataset = TraceDataset::new("Test Dataset".to_string());

        let mut trace1 = ReasoningTrace::new("P1".to_string(), "m".to_string(), 100, 50, 3);
        trace1.add_chunk("".to_string(), "".to_string(), 100, 100);
        trace1.complete(Some("Sol1".to_string()), TerminationReason::SolutionFound);

        let mut trace2 = ReasoningTrace::new("P2".to_string(), "m".to_string(), 100, 50, 3);
        trace2.add_chunk("".to_string(), "".to_string(), 200, 100);
        trace2.complete(None, TerminationReason::MaxIterations);

        dataset.add_trace(trace1);
        dataset.add_trace(trace2);

        assert_eq!(dataset.metadata.total_traces, 2);
        assert_eq!(dataset.metadata.total_tokens, 300);
        assert_eq!(dataset.avg_tokens(), 150.0);
        assert_eq!(dataset.success_rate(), 0.5); // 1 of 2 solved
    }
}
