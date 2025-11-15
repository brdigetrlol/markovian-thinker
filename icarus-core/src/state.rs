// Markovian Thinker: State Management
// Implements bounded-state reasoning with carryover between chunks

use crate::attention::AttentionConfig;
use crate::concept_space::ConceptSpaceConfig;
use crate::experts::ExpertConfig;
use crate::h2ce_adapter::H2CEConfig;
use crate::parser;
use crate::sampling_strategies::SamplingConfig;
use crate::storm_mitigation::StormMitigationConfig;
use crate::trace::TerminationReason;
use crate::types::{ReasoningDomain, SessionMetadata, VerificationResult};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Information about whether reasoning should terminate
#[derive(Debug, Clone)]
pub struct TerminationInfo {
    pub should_terminate: bool,
    pub reason: TerminationReason,
    pub solution: Option<String>,
}

/// Configuration for Markovian state management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateConfig {
    /// Maximum tokens per chunk (C parameter)
    pub chunk_size: usize,

    /// Tokens to carry forward between chunks (m parameter, typically C/2)
    pub carryover_size: usize,

    /// Maximum number of chunks (iterations)
    pub max_iterations: usize,

    /// Total token budget (C + (max_iterations-1) * (C - carryover_size))
    pub token_budget: usize,

    /// GPT-OSS Enhancement: Expert gating configuration
    #[serde(default)]
    pub expert_config: ExpertConfig,

    /// GPT-OSS Enhancement: Attention mechanism configuration
    #[serde(default)]
    pub attention_config: AttentionConfig,

    /// GPT-OSS Enhancement: Sampling strategy configuration
    #[serde(default)]
    pub sampling_config: SamplingConfig,

    /// Icarus TIC Enhancement: Storm mitigation configuration
    #[serde(default)]
    pub storm_mitigation_config: StormMitigationConfig,

    /// Icarus TIC Enhancement: Concept space configuration
    #[serde(default)]
    pub concept_space_config: ConceptSpaceConfig,

    /// Enable event-driven processing (Icarus TIC)
    #[serde(default)]
    pub enable_event_driven: bool,

    /// Enable causal trace tracking (Icarus TIC)
    #[serde(default)]
    pub enable_causal_trace: bool,

    /// Enable intelligent carryover using concept space (Phase 7)
    #[serde(default)]
    pub enable_intelligent_carryover: bool,

    /// Number of similar chunks to consider for intelligent carryover (Phase 7)
    #[serde(default = "default_carryover_k")]
    pub carryover_k: usize,

    /// Weight for semantic relevance vs recency (0.0-1.0, default 0.8 = 80% semantic)
    #[serde(default = "default_relevance_weight")]
    pub relevance_weight: f32,

    /// Phase 8: H²CE semantic search configuration
    #[serde(default)]
    pub h2ce_config: H2CEConfig,
}

fn default_carryover_k() -> usize {
    3
}

fn default_relevance_weight() -> f32 {
    0.8
}

impl StateConfig {
    /// Create default config (8K chunks, 4K carryover, 5 iterations = 24K budget)
    pub fn default() -> Self {
        Self {
            chunk_size: 8192,
            carryover_size: 4096,
            max_iterations: 5,
            token_budget: 24576, // 8K + 4*(8K-4K) = 24K
            expert_config: ExpertConfig::default(),
            attention_config: AttentionConfig::default(),
            sampling_config: SamplingConfig::default(),
            storm_mitigation_config: StormMitigationConfig::default(),
            concept_space_config: ConceptSpaceConfig::default(),
            enable_event_driven: false, // Opt-in for now
            enable_causal_trace: false, // Opt-in for now
            enable_intelligent_carryover: false, // Opt-in for backward compat
            carryover_k: default_carryover_k(),
            relevance_weight: default_relevance_weight(),
            h2ce_config: H2CEConfig::default(), // Phase 8: Opt-in semantic search
        }
    }

    /// Create custom config with validation
    pub fn new(chunk_size: usize, carryover_size: usize, max_iterations: usize) -> Result<Self, String> {
        if carryover_size >= chunk_size {
            return Err(format!(
                "Carryover size ({}) must be < chunk size ({})",
                carryover_size, chunk_size
            ));
        }

        if max_iterations == 0 {
            return Err("Max iterations must be > 0".to_string());
        }

        // Calculate token budget: C + (I-1)*(C-m)
        let token_budget = chunk_size + (max_iterations - 1) * (chunk_size - carryover_size);

        Ok(Self {
            chunk_size,
            carryover_size,
            max_iterations,
            token_budget,
            expert_config: ExpertConfig::default(),
            attention_config: AttentionConfig::default(),
            sampling_config: SamplingConfig::default(),
            storm_mitigation_config: StormMitigationConfig::default(),
            concept_space_config: ConceptSpaceConfig::default(),
            enable_event_driven: false,
            enable_causal_trace: false,
            enable_intelligent_carryover: false,
            carryover_k: default_carryover_k(),
            relevance_weight: default_relevance_weight(),
            h2ce_config: H2CEConfig::default(),
        })
    }

    /// Calculate actual maximum thinking tokens given config
    pub fn max_thinking_tokens(&self) -> usize {
        self.token_budget
    }
}

/// Markovian state: query + carryover buffer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkovianState {
    /// Original query (preserved across all chunks)
    pub query: String,

    /// Carryover buffer from previous chunk (last m tokens)
    pub carryover: String,

    /// Current iteration number (1-indexed)
    pub iteration: usize,

    /// Total tokens generated so far
    pub tokens_generated: usize,

    /// Configuration
    pub config: StateConfig,

    /// Detected reasoning domain (for specialized strategies)
    pub domain: Option<ReasoningDomain>,

    /// Verification results from each chunk
    pub verifications: Vec<VerificationResult>,

    /// Aggregated session metadata
    pub metadata: SessionMetadata,

    /// History of previous chunks for intelligent carryover (Phase 7)
    pub chunk_history: Vec<String>,
}

impl MarkovianState {
    /// Initialize with query only (iteration 1)
    pub fn new(query: String, config: StateConfig) -> Self {
        // Detect reasoning domain from query
        let domain = Some(ReasoningDomain::detect(&query, None));

        Self {
            query,
            carryover: String::new(),
            iteration: 1,
            tokens_generated: 0,
            config,
            domain,
            verifications: Vec::new(),
            metadata: SessionMetadata::default(),
            chunk_history: Vec::new(),
        }
    }

    /// Build prompt for current iteration: query ⊕ carryover
    pub fn build_prompt(&self) -> String {
        if self.carryover.is_empty() {
            self.query.clone()
        } else {
            format!("{}\n\n{}", self.query, self.carryover)
        }
    }

    /// Update state after generating a chunk
    /// Returns TerminationInfo indicating whether to continue or terminate
    pub fn update(&mut self, chunk_output: &str, chunk_tokens: usize) -> Result<TerminationInfo, String> {
        // Update token count
        self.tokens_generated += chunk_tokens;

        // Try to parse structured output first
        let parsed = parser::parse_chunk_output(chunk_output);

        // Store verification result if present
        if !parsed.verification.issues.is_empty() || !parsed.verification.key_concepts.is_empty() {
            self.verifications.push(parsed.verification.clone());
            // Update metadata
            self.metadata.update_from_verifications(&self.verifications);
            self.metadata.total_tokens = self.tokens_generated;
        }

        // Check for completion (structured or legacy markers)
        let is_complete = parsed.is_complete || Self::extract_solution(chunk_output).is_some();

        if is_complete {
            // Extract solution from structured carryover or reasoning, or use legacy extraction
            let solution = if !parsed.carryover.is_empty() {
                Some(parsed.carryover.clone())
            } else if !parsed.reasoning.is_empty() {
                Some(parsed.reasoning.clone())
            } else {
                Self::extract_solution(chunk_output)
            };

            return Ok(TerminationInfo {
                should_terminate: true,
                reason: TerminationReason::SolutionFound,
                solution,
            });
        }

        // Check if budget exceeded
        if self.tokens_generated > self.config.token_budget {
            return Ok(TerminationInfo {
                should_terminate: true,
                reason: TerminationReason::TokenBudgetExceeded,
                solution: None,
            });
        }

        // Check if max iterations reached
        if self.iteration >= self.config.max_iterations {
            return Ok(TerminationInfo {
                should_terminate: true,
                reason: TerminationReason::MaxIterations,
                solution: None,
            });
        }

        // Store chunk in history for intelligent carryover
        self.chunk_history.push(chunk_output.to_string());

        // Extract carryover: prefer structured [CARRYOVER], then intelligent, then legacy
        let mut carryover = if !parsed.carryover.is_empty() {
            parsed.carryover
        } else if self.config.enable_intelligent_carryover && self.chunk_history.len() > 1 {
            Self::extract_intelligent_carryover(
                chunk_output,
                &self.chunk_history,
                self.config.carryover_size,
                self.config.carryover_k,
                self.config.relevance_weight,
            )
        } else {
            Self::extract_carryover(chunk_output, self.config.carryover_size)
        };

        // Apply attention-based compression if enabled and carryover is too long
        if self.config.attention_config.sliding_window_size.is_some() {
            let approx_tokens = carryover.len() / 4; // Rough estimate
            if approx_tokens > self.config.carryover_size {
                tracing::debug!(
                    "Carryover too long ({} est. tokens), applying attention compression",
                    approx_tokens
                );
                carryover = Self::compress_with_attention(
                    &carryover,
                    self.config.carryover_size,
                    &self.config.attention_config,
                );
            }
        }

        self.carryover = carryover;

        // Increment iteration
        self.iteration += 1;

        // Continue reasoning
        Ok(TerminationInfo {
            should_terminate: false,
            reason: TerminationReason::MaxIterations, // Placeholder, not used when continuing
            solution: None,
        })
    }

    /// Check for termination markers and extract solution if present
    fn extract_solution(text: &str) -> Option<String> {
        // Check for various termination markers
        if text.contains("[EOS]") || text.contains("[DONE]") || text.contains("[SOLUTION]") {
            // Extract solution after the marker
            for marker in &["[SOLUTION]", "[DONE]", "[EOS]"] {
                if let Some(pos) = text.rfind(marker) {
                    let solution_text = text[pos + marker.len()..].trim();
                    if !solution_text.is_empty() {
                        return Some(solution_text.to_string());
                    }
                }
            }
            // If marker exists but no text after, return last paragraph
            return Some(Self::extract_last_paragraph(text));
        }

        // Check for LaTeX boxed answer: \boxed{...}
        if let Some(boxed) = Self::extract_boxed(text) {
            return Some(boxed);
        }

        // Check for markdown answer format: #### Answer
        if let Some(answer) = Self::extract_markdown_answer(text) {
            return Some(answer);
        }

        None
    }

    /// Extract content from \boxed{...}
    fn extract_boxed(text: &str) -> Option<String> {
        if let Some(start) = text.rfind("\\boxed{") {
            let after_boxed = &text[start + 7..];
            let mut depth = 1;
            let mut end = 0;

            for (i, c) in after_boxed.chars().enumerate() {
                match c {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            end = i;
                            break;
                        }
                    }
                    _ => {}
                }
            }

            if end > 0 {
                return Some(after_boxed[..end].trim().to_string());
            }
        }
        None
    }

    /// Extract markdown-style answer (#### Answer or #### Solution)
    fn extract_markdown_answer(text: &str) -> Option<String> {
        for marker in &["#### Answer", "#### Solution", "#### Final Answer"] {
            if let Some(pos) = text.rfind(marker) {
                let answer_text = text[pos + marker.len()..].trim();
                if !answer_text.is_empty() {
                    return Some(answer_text.to_string());
                }
            }
        }
        None
    }

    /// Extract last paragraph/sentence
    fn extract_last_paragraph(text: &str) -> String {
        text.trim()
            .rsplit("\n\n")
            .next()
            .unwrap_or(text.trim())
            .to_string()
    }

    /// Extract last m tokens from text (simplified: use character-based approximation)
    /// Note: Real implementation would use actual tokenizer
    fn extract_carryover(text: &str, carryover_tokens: usize) -> String {
        // Approximate: 1 token ≈ 4 characters (English)
        let approx_chars = carryover_tokens * 4;

        if text.len() <= approx_chars {
            text.to_string()
        } else {
            // Take last N characters, but start at word boundary
            let start_pos = text.len().saturating_sub(approx_chars);

            // Find word boundary
            let boundary = text[start_pos..]
                .find(|c: char| c.is_whitespace())
                .unwrap_or(0);

            text[start_pos + boundary..].trim().to_string()
        }
    }

    /// Intelligent carryover selection using semantic similarity
    /// Selects relevant text from previous chunks based on similarity to current chunk
    fn extract_intelligent_carryover(
        current_chunk: &str,
        chunk_history: &[String],
        carryover_tokens: usize,
        k: usize,
        relevance_weight: f32,
    ) -> String {
        // Simple word-based similarity (Jaccard index)
        fn compute_similarity(text1: &str, text2: &str) -> f32 {
            let words1: std::collections::HashSet<&str> = text1
                .split_whitespace()
                .filter(|w| w.len() > 3) // Filter short words
                .collect();
            let words2: std::collections::HashSet<&str> = text2
                .split_whitespace()
                .filter(|w| w.len() > 3)
                .collect();

            if words1.is_empty() || words2.is_empty() {
                return 0.0;
            }

            let intersection = words1.intersection(&words2).count();
            let union = words1.union(&words2).count();

            if union == 0 {
                0.0
            } else {
                intersection as f32 / union as f32
            }
        }

        // Score each previous chunk
        let mut chunk_scores: Vec<(usize, f32)> = chunk_history
            .iter()
            .enumerate()
            .filter(|(idx, _)| *idx < chunk_history.len() - 1) // Exclude current chunk
            .map(|(idx, chunk)| {
                let semantic_score = compute_similarity(current_chunk, chunk);
                let recency_score = idx as f32 / (chunk_history.len() - 1) as f32;

                // Combined score: weighted average
                let combined_score =
                    relevance_weight * semantic_score + (1.0 - relevance_weight) * recency_score;

                (idx, combined_score)
            })
            .collect();

        // Sort by score descending
        chunk_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Take top-k chunks
        let top_k = chunk_scores.iter().take(k.min(chunk_scores.len()));

        // Extract text from top-k chunks (last part of each)
        let approx_chars_per_chunk = (carryover_tokens * 4) / k.max(1);
        let mut carryover_parts: Vec<String> = Vec::new();

        for (idx, _score) in top_k {
            let chunk = &chunk_history[*idx];
            let extract = Self::extract_carryover(chunk, approx_chars_per_chunk / 4);
            if !extract.is_empty() {
                carryover_parts.push(extract);
            }
        }

        // Combine parts
        let combined = carryover_parts.join("\n...\n");

        // Trim to carryover size
        Self::extract_carryover(&combined, carryover_tokens)
    }

    /// Attention-based carryover compression
    /// Uses attention scoring to keep most important tokens when exceeding limits
    fn compress_with_attention(text: &str, target_tokens: usize, config: &crate::attention::AttentionConfig) -> String {
        use crate::attention::SlidingWindowAttention;

        let attention = SlidingWindowAttention::new(config.clone());
        attention.select_important(text, target_tokens)
    }

    /// Check if reasoning should continue
    pub fn should_continue(&self) -> bool {
        self.iteration < self.config.max_iterations
            && self.tokens_generated < self.config.token_budget
    }

    /// Get current state summary
    pub fn summary(&self) -> String {
        format!(
            "Iteration {}/{} | Tokens: {}/{} | Budget remaining: {}",
            self.iteration,
            self.config.max_iterations,
            self.tokens_generated,
            self.config.token_budget,
            self.config.token_budget.saturating_sub(self.tokens_generated)
        )
    }

    /// Get configuration reference
    pub fn config(&self) -> &StateConfig {
        &self.config
    }
}

/// History buffer for tracking all generated chunks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkHistory {
    /// All generated chunks in order
    chunks: VecDeque<ChunkRecord>,

    /// Maximum chunks to retain
    max_chunks: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkRecord {
    pub iteration: usize,
    pub prompt: String,
    pub output: String,
    pub tokens: usize,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ChunkHistory {
    pub fn new(max_chunks: usize) -> Self {
        Self {
            chunks: VecDeque::with_capacity(max_chunks),
            max_chunks,
        }
    }

    /// Add a chunk to history
    pub fn push(&mut self, record: ChunkRecord) {
        if self.chunks.len() >= self.max_chunks {
            self.chunks.pop_front();
        }
        self.chunks.push_back(record);
    }

    /// Get all chunks
    pub fn chunks(&self) -> &VecDeque<ChunkRecord> {
        &self.chunks
    }

    /// Get total tokens across all chunks
    pub fn total_tokens(&self) -> usize {
        self.chunks.iter().map(|c| c.tokens).sum()
    }

    /// Reconstruct full reasoning chain (all chunks concatenated)
    pub fn full_output(&self) -> String {
        self.chunks
            .iter()
            .map(|c| c.output.as_str())
            .collect::<Vec<_>>()
            .join("\n\n---\n\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_config_default() {
        let config = StateConfig::default();
        assert_eq!(config.chunk_size, 8192);
        assert_eq!(config.carryover_size, 4096);
        assert_eq!(config.max_iterations, 5);
        assert_eq!(config.max_thinking_tokens(), 24576);
    }

    #[test]
    fn test_state_config_custom() {
        let config = StateConfig::new(4096, 2048, 10).unwrap();
        assert_eq!(config.chunk_size, 4096);
        assert_eq!(config.carryover_size, 2048);
        assert_eq!(config.max_iterations, 10);
        // 4096 + 9*(4096-2048) = 4096 + 18432 = 22528
        assert_eq!(config.max_thinking_tokens(), 22528);
    }

    #[test]
    fn test_state_config_validation() {
        // Carryover >= chunk_size should fail
        assert!(StateConfig::new(4096, 4096, 5).is_err());
        assert!(StateConfig::new(4096, 5000, 5).is_err());

        // Zero iterations should fail
        assert!(StateConfig::new(4096, 2048, 0).is_err());
    }

    #[test]
    fn test_markovian_state_init() {
        let config = StateConfig::default();
        let state = MarkovianState::new("What is 2+2?".to_string(), config);

        assert_eq!(state.query, "What is 2+2?");
        assert_eq!(state.carryover, "");
        assert_eq!(state.iteration, 1);
        assert_eq!(state.tokens_generated, 0);
        assert!(state.should_continue());
    }

    #[test]
    fn test_state_build_prompt() {
        let config = StateConfig::default();
        let mut state = MarkovianState::new("Query".to_string(), config);

        // First iteration: just query
        assert_eq!(state.build_prompt(), "Query");

        // After update with carryover
        state.carryover = "Previous result".to_string();
        assert_eq!(state.build_prompt(), "Query\n\nPrevious result");
    }

    #[test]
    fn test_state_update() {
        let config = StateConfig::new(100, 50, 3).unwrap();
        let mut state = MarkovianState::new("Test query".to_string(), config);

        // First update
        let result = state.update("Chunk 1 output with some text", 20);
        assert!(result.is_ok());
        let term_info = result.unwrap();
        assert!(!term_info.should_terminate);
        assert_eq!(state.iteration, 2);
        assert_eq!(state.tokens_generated, 20);
        assert!(!state.carryover.is_empty());

        // Second update
        let result = state.update("Chunk 2 output", 30);
        assert!(result.is_ok());
        let term_info = result.unwrap();
        assert!(!term_info.should_terminate);
        assert_eq!(state.iteration, 3);
        assert_eq!(state.tokens_generated, 50);

        // Third update should terminate (max iterations reached)
        let result = state.update("Chunk 3 output", 40);
        assert!(result.is_ok());
        let term_info = result.unwrap();
        assert!(term_info.should_terminate);
        assert_eq!(term_info.reason, TerminationReason::MaxIterations);
    }

    #[test]
    fn test_extract_carryover() {
        let text = "This is a long piece of text that should be truncated to extract only the last portion as carryover state for the next iteration.";

        // Small carryover (20 tokens ≈ 80 chars)
        let carryover = MarkovianState::extract_carryover(text, 20);
        assert!(carryover.len() < text.len());
        assert!(text.ends_with(&carryover.trim()));
    }

    #[test]
    fn test_chunk_history() {
        let mut history = ChunkHistory::new(3);

        for i in 1..=5 {
            history.push(ChunkRecord {
                iteration: i,
                prompt: format!("Prompt {}", i),
                output: format!("Output {}", i),
                tokens: i * 10,
                timestamp: chrono::Utc::now(),
            });
        }

        // Should only keep last 3
        assert_eq!(history.chunks().len(), 3);
        assert_eq!(history.chunks()[0].iteration, 3);
        assert_eq!(history.chunks()[2].iteration, 5);

        // Total tokens: 30 + 40 + 50 = 120
        assert_eq!(history.total_tokens(), 120);
    }
}
