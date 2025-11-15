// Markovian Thinker: Chunk Manager
// Orchestrates chunk-based reasoning with bounded context

use crate::causal_trace::CausalTrace;
use crate::event_queue::EventQueue;
use crate::events::{EventWithMetadata, ReasoningEvent, ReasoningLevel};
use crate::state::{ChunkHistory, ChunkRecord, MarkovianState, StateConfig};
use crate::trace::{ReasoningTrace, TerminationReason};
use anyhow::Result;
use std::time::Instant;
use uuid::Uuid;

/// Chunk generation interface (abstract over different LLM providers)
#[async_trait::async_trait]
pub trait ChunkGenerator {
    /// Generate text for a single chunk given prompt
    /// Returns (generated_text, token_count)
    async fn generate(&self, prompt: &str, max_tokens: usize) -> Result<(String, usize)>;

    /// Get model name/identifier
    fn model_name(&self) -> &str;
}

/// Main orchestrator for chunk-based Markovian reasoning
pub struct ChunkManager {
    config: StateConfig,
    event_queue: Option<EventQueue>,
    causal_trace: Option<CausalTrace>,
    session_id: Uuid,
    last_chunk_event_id: Option<Uuid>, // Track previous chunk event for causal links
}

impl ChunkManager {
    /// Create new chunk manager with configuration
    pub fn new(config: StateConfig) -> Self {
        let session_id = Uuid::new_v4();
        Self {
            config,
            event_queue: None,
            causal_trace: None,
            session_id,
            last_chunk_event_id: None,
        }
    }

    /// Create with event-driven capabilities
    pub fn with_events(config: StateConfig, session_id: Uuid) -> Self {
        let event_queue = if config.enable_event_driven {
            Some(EventQueue::new(1000)) // Default capacity: 1000 events
        } else {
            None
        };

        let causal_trace = if config.enable_causal_trace {
            Some(CausalTrace::new(session_id))
        } else {
            None
        };

        Self {
            config,
            event_queue,
            causal_trace,
            session_id,
            last_chunk_event_id: None,
        }
    }

    /// Create with default config
    pub fn default() -> Self {
        Self::new(StateConfig::default())
    }

    /// Generate a complete reasoning trace using chunk-based generation
    pub async fn generate_trace<G: ChunkGenerator>(
        &mut self,
        problem: String,
        generator: &G,
    ) -> Result<ReasoningTrace> {
        let mut state = MarkovianState::new(problem.clone(), self.config.clone());
        let mut trace = ReasoningTrace::new(
            problem,
            generator.model_name().to_string(),
            self.config.chunk_size,
            self.config.carryover_size,
            self.config.max_iterations,
        );
        let mut history = ChunkHistory::new(self.config.max_iterations);

        tracing::info!(
            "Starting Markovian reasoning | Config: {} chunks × {} tokens, {} carryover",
            self.config.max_iterations,
            self.config.chunk_size,
            self.config.carryover_size
        );

        // Main generation loop
        while state.should_continue() {
            let chunk_start = Instant::now();

            // Build prompt for this chunk
            let prompt = state.build_prompt();

            tracing::debug!(
                "Chunk {} | Prompt length: {} chars | {}",
                state.iteration,
                prompt.len(),
                state.summary()
            );

            // Emit ChunkRequest event (if event-driven mode enabled)
            self.emit_event(
                ReasoningEvent::ChunkRequest {
                    session_id: self.session_id,
                    prompt: prompt.clone(),
                    priority: 1.0, // Normal priority
                    timestamp: chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64,
                    level: ReasoningLevel::Macro, // Full chunk = Macro level
                },
                1.0,
                ReasoningLevel::Macro,
            );

            // Generate chunk
            let (output, tokens) = match generator.generate(&prompt, self.config.chunk_size).await {
                Ok(result) => result,
                Err(e) => {
                    tracing::error!("Generation error: {}", e);
                    trace.complete(
                        None,
                        TerminationReason::Error(format!("Generation failed: {}", e)),
                    );
                    return Ok(trace);
                }
            };

            let chunk_duration = chunk_start.elapsed().as_millis() as u64;

            // Add to trace and history
            trace.add_chunk(prompt.clone(), output.clone(), tokens, chunk_duration);
            history.push(ChunkRecord {
                iteration: state.iteration,
                prompt,
                output: output.clone(),
                tokens,
                timestamp: chrono::Utc::now(),
            });

            // Emit ChunkComplete event (if event-driven mode enabled)
            let chunk_id = Uuid::new_v4();
            let event_id = self.emit_event(
                ReasoningEvent::ChunkComplete {
                    session_id: self.session_id,
                    chunk_id,
                    output: output.clone(),
                    tokens,
                    spawned_events: vec![],
                    timestamp: chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64,
                },
                1.0,
                ReasoningLevel::Macro,
            );

            // Update last_chunk_event_id for causal tracking
            if event_id.is_some() {
                self.last_chunk_event_id = event_id;
            }

            tracing::info!(
                "Chunk {} complete | {} tokens | {:.1}ms | {:.1} tok/sec",
                state.iteration,
                tokens,
                chunk_duration,
                if chunk_duration > 0 {
                    (tokens as f64 * 1000.0) / chunk_duration as f64
                } else {
                    0.0
                }
            );

            // Check for explicit termination markers
            if Self::has_termination_marker(&output) {
                tracing::info!("Found termination marker, stopping generation");
                let solution = Self::extract_solution(&output);
                trace.complete(solution, TerminationReason::SolutionFound);
                return Ok(trace);
            }

            // Update state for next iteration
            match state.update(&output, tokens) {
                Ok(_) => {
                    // Continue to next chunk
                    continue;
                }
                Err(reason) => {
                    // Termination condition reached
                    tracing::info!("Termination: {}", reason);

                    let termination = if reason.contains("budget") {
                        TerminationReason::TokenBudgetExceeded
                    } else if reason.contains("iterations") {
                        TerminationReason::MaxIterations
                    } else {
                        TerminationReason::Error(reason)
                    };

                    let solution = Self::extract_solution(&output);
                    trace.complete(solution, termination);
                    return Ok(trace);
                }
            }
        }

        // Should not reach here, but handle gracefully
        let solution = if let Some(last_chunk) = history.chunks().back() {
            Self::extract_solution(&last_chunk.output)
        } else {
            None
        };

        trace.complete(solution, TerminationReason::MaxIterations);
        Ok(trace)
    }

    /// Check if output contains termination markers
    fn has_termination_marker(text: &str) -> bool {
        // Common solution markers
        let markers = [
            "[EOS]",
            "[DONE]",
            "[SOLUTION]",
            "\\boxed{",  // LaTeX answer box
            "#### ",     // Many models use this for final answers
        ];

        markers.iter().any(|marker| text.contains(marker))
    }

    /// Extract solution from output (simplified heuristic)
    fn extract_solution(text: &str) -> Option<String> {
        // Try to find boxed answer (LaTeX)
        if let Some(start) = text.find("\\boxed{") {
            let after_boxed = &text[start + 7..];
            if let Some(end) = Self::find_matching_brace(after_boxed) {
                return Some(after_boxed[..end].trim().to_string());
            }
        }

        // Try to find #### answer format
        if let Some(start) = text.rfind("#### ") {
            let answer = text[start + 5..].lines().next()?;
            return Some(answer.trim().to_string());
        }

        // Try to find [SOLUTION] marker
        if let Some(start) = text.find("[SOLUTION]") {
            let after_marker = &text[start + 10..];
            let answer = after_marker.lines().next()?;
            return Some(answer.trim().to_string());
        }

        // Fallback: take last non-empty line
        text.lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .map(|s| s.trim().to_string())
    }

    /// Find matching closing brace for LaTeX boxed answers
    fn find_matching_brace(text: &str) -> Option<usize> {
        let mut depth = 1;
        for (i, c) in text.chars().enumerate() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// Get config
    pub fn config(&self) -> &StateConfig {
        &self.config
    }

    /// Emit a reasoning event (if event-driven mode is enabled)
    /// Returns the event ID if successfully emitted
    fn emit_event(&mut self, event: ReasoningEvent, priority: f32, level: ReasoningLevel) -> Option<Uuid> {
        if let Some(queue) = &mut self.event_queue {
            let event_with_metadata = EventWithMetadata {
                event: event.clone(),
                priority,
                momentum: 0.0,
                trigger_count: 0,
                parent: self.last_chunk_event_id, // Link to previous chunk
                children: Vec::new(),
            };

            if queue.try_insert(event_with_metadata) {
                let event_id = event.event_id();

                // Also record in causal trace if enabled
                if let Some(trace) = &mut self.causal_trace {
                    // Build predecessors: chunk N depends on chunk N-1
                    let predecessors = if let Some(prev_id) = self.last_chunk_event_id {
                        vec![prev_id]
                    } else {
                        vec![] // First chunk has no predecessors
                    };

                    trace.add_event(event.clone(), level, predecessors);

                    tracing::debug!(
                        "Causal trace: event {} added with {} predecessors",
                        event_id,
                        if self.last_chunk_event_id.is_some() { 1 } else { 0 }
                    );
                }

                Some(event_id)
            } else {
                tracing::warn!("Failed to insert event into queue (queue full or backpressure)");
                None
            }
        } else {
            None
        }
    }

    /// Get the event queue (if any)
    pub fn event_queue(&self) -> Option<&EventQueue> {
        self.event_queue.as_ref()
    }

    /// Get the event queue mutably (if any)
    pub fn event_queue_mut(&mut self) -> Option<&mut EventQueue> {
        self.event_queue.as_mut()
    }

    /// Get the causal trace (if any)
    pub fn causal_trace(&self) -> Option<&CausalTrace> {
        self.causal_trace.as_ref()
    }

    /// Get the causal trace mutably (if any)
    pub fn causal_trace_mut(&mut self) -> Option<&mut CausalTrace> {
        self.causal_trace.as_mut()
    }

    /// Get session ID
    pub fn session_id(&self) -> Uuid {
        self.session_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock generator for testing
    struct MockGenerator {
        responses: Vec<(String, usize)>,
        index: std::sync::Arc<std::sync::Mutex<usize>>,
    }

    impl MockGenerator {
        fn new(responses: Vec<(String, usize)>) -> Self {
            Self {
                responses,
                index: std::sync::Arc::new(std::sync::Mutex::new(0)),
            }
        }
    }

    #[async_trait::async_trait]
    impl ChunkGenerator for MockGenerator {
        async fn generate(&self, _prompt: &str, _max_tokens: usize) -> Result<(String, usize)> {
            let mut idx = self.index.lock().unwrap();
            let response = self.responses[*idx].clone();
            *idx = (*idx + 1).min(self.responses.len() - 1);
            Ok(response)
        }

        fn model_name(&self) -> &str {
            "mock-model"
        }
    }

    #[tokio::test]
    async fn test_chunk_manager_single_chunk() {
        let config = StateConfig::new(100, 50, 3).unwrap();
        let mut manager = ChunkManager::new(config);

        let generator = MockGenerator::new(vec![
            ("First chunk with [SOLUTION] 42".to_string(), 10),
        ]);

        let trace = manager
            .generate_trace("What is 6*7?".to_string(), &generator)
            .await
            .unwrap();

        assert_eq!(trace.chunks.len(), 1);
        assert_eq!(trace.termination_reason, TerminationReason::SolutionFound);
        assert_eq!(trace.solution, Some("42".to_string()));
    }

    #[tokio::test]
    async fn test_chunk_manager_multi_chunk() {
        let config = StateConfig::new(100, 50, 5).unwrap(); // Increase max_iterations to ensure all chunks generated
        let mut manager = ChunkManager::new(config);

        let generator = MockGenerator::new(vec![
            ("Chunk 1 reasoning...".to_string(), 20),
            ("Chunk 2 more work...".to_string(), 25),
            ("Chunk 3 final answer #### 123".to_string(), 30),
        ]);

        let trace = manager
            .generate_trace("Problem".to_string(), &generator)
            .await
            .unwrap();

        assert_eq!(trace.chunks.len(), 3);
        assert_eq!(trace.total_tokens, 75);
        assert_eq!(trace.solution, Some("123".to_string()));
    }

    #[tokio::test]
    async fn test_termination_markers() {
        assert!(ChunkManager::has_termination_marker("Answer: [EOS]"));
        assert!(ChunkManager::has_termination_marker("The answer is \\boxed{42}"));
        assert!(ChunkManager::has_termination_marker("Final: #### 99"));
        assert!(!ChunkManager::has_termination_marker("Still thinking..."));
    }

    #[test]
    fn test_solution_extraction() {
        // LaTeX boxed
        let solution = ChunkManager::extract_solution("Answer is \\boxed{42}");
        assert_eq!(solution, Some("42".to_string()));

        // #### format
        let solution = ChunkManager::extract_solution("Therefore #### 99");
        assert_eq!(solution, Some("99".to_string()));

        // [SOLUTION] marker
        let solution = ChunkManager::extract_solution("[SOLUTION] The answer is 7");
        assert_eq!(solution, Some("The answer is 7".to_string()));

        // Fallback to last line
        let solution = ChunkManager::extract_solution("Line 1\nLine 2\nFinal answer");
        assert_eq!(solution, Some("Final answer".to_string()));
    }

    #[test]
    fn test_find_matching_brace() {
        assert_eq!(ChunkManager::find_matching_brace("42}"), Some(2));
        assert_eq!(ChunkManager::find_matching_brace("{nested}}"), Some(8));
        assert_eq!(ChunkManager::find_matching_brace("no brace"), None);
    }
}
