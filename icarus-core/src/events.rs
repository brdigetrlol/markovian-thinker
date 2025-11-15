// Markovian Thinker: Event-Driven Reasoning System
// Inspired by Icarus TIC's event-driven architecture

use crate::state::MarkovianState;
use crate::types::VerificationResult;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Reasoning event types for event-driven chunk processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReasoningEvent {
    /// Request to generate a reasoning chunk
    ChunkRequest {
        session_id: Uuid,
        prompt: String,
        priority: f32,
        timestamp: u64,
        level: ReasoningLevel,
    },

    /// Chunk generation completed
    ChunkComplete {
        session_id: Uuid,
        chunk_id: Uuid,
        output: String,
        tokens: usize,
        spawned_events: Vec<ReasoningEvent>,
        timestamp: u64,
    },

    /// Request to verify reasoning step
    VerificationRequest {
        session_id: Uuid,
        parent_event: Uuid,
        hypothesis: String,
        timestamp: u64,
    },

    /// Verification result
    VerificationComplete {
        session_id: Uuid,
        request_id: Uuid,
        result: VerificationResult,
        timestamp: u64,
    },

    /// Crystallize concept to lattice
    ConceptCrystallization {
        session_id: Uuid,
        concept: String,
        embedding: Option<Vec<f32>>,
        timestamp: u64,
    },

    /// Concept crystallized
    ConceptCrystallized {
        session_id: Uuid,
        concept: String,
        lattice_point: Vec<f32>,
        similar_concepts: Vec<(String, f32)>,
        timestamp: u64,
    },

    /// Termination check
    TerminationCheck {
        session_id: Uuid,
        current_state: Box<MarkovianState>,
        timestamp: u64,
    },

    /// Session terminated
    SessionTerminated {
        session_id: Uuid,
        reason: String,
        final_solution: Option<String>,
        timestamp: u64,
    },
}

impl ReasoningEvent {
    /// Get session ID for this event
    pub fn session_id(&self) -> Uuid {
        match self {
            Self::ChunkRequest { session_id, .. }
            | Self::ChunkComplete { session_id, .. }
            | Self::VerificationRequest { session_id, .. }
            | Self::VerificationComplete { session_id, .. }
            | Self::ConceptCrystallization { session_id, .. }
            | Self::ConceptCrystallized { session_id, .. }
            | Self::TerminationCheck { session_id, .. }
            | Self::SessionTerminated { session_id, .. } => *session_id,
        }
    }

    /// Get timestamp
    pub fn timestamp(&self) -> u64 {
        match self {
            Self::ChunkRequest { timestamp, .. }
            | Self::ChunkComplete { timestamp, .. }
            | Self::VerificationRequest { timestamp, .. }
            | Self::VerificationComplete { timestamp, .. }
            | Self::ConceptCrystallization { timestamp, .. }
            | Self::ConceptCrystallized { timestamp, .. }
            | Self::TerminationCheck { timestamp, .. }
            | Self::SessionTerminated { timestamp, .. } => *timestamp,
        }
    }

    /// Get event ID (generate from content hash)
    pub fn event_id(&self) -> Uuid {
        // Use session_id + timestamp for deterministic ID
        let content = format!("{}:{}", self.session_id(), self.timestamp());
        Uuid::new_v5(&Uuid::NAMESPACE_OID, content.as_bytes())
    }

    /// Get priority for queue ordering
    pub fn priority(&self) -> f32 {
        match self {
            Self::ChunkRequest { priority, .. } => *priority,
            Self::TerminationCheck { .. } => 1.0, // High priority
            Self::VerificationRequest { .. } => 0.8,
            Self::ConceptCrystallization { .. } => 0.5,
            Self::ChunkComplete { .. } => 0.3,
            Self::VerificationComplete { .. } => 0.3,
            Self::ConceptCrystallized { .. } => 0.2,
            Self::SessionTerminated { .. } => 0.1,
        }
    }

    /// Check if this event is terminal (ends processing)
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::SessionTerminated { .. })
    }
}

/// Hierarchical reasoning levels (inspired by Icarus TIC)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReasoningLevel {
    /// Micro: Token-level operations (GPU-only, no CPU communication)
    Micro,

    /// Meso: Sentence/paragraph-level reasoning steps
    Meso,

    /// Macro: Full chunk generation with carryover
    Macro,
}

impl ReasoningLevel {
    /// Get processing complexity estimate
    pub fn complexity(&self) -> usize {
        match self {
            Self::Micro => 1,
            Self::Meso => 10,
            Self::Macro => 100,
        }
    }

    /// Check if this level requires CPU communication
    pub fn needs_cpu(&self) -> bool {
        !matches!(self, Self::Micro)
    }
}

/// Event with metadata for queue management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventWithMetadata {
    /// The reasoning event
    pub event: ReasoningEvent,

    /// Priority (higher = process sooner)
    pub priority: f32,

    /// Momentum: boost priority for frequently triggered events
    pub momentum: f32,

    /// Number of times this event type has been triggered
    pub trigger_count: u32,

    /// Parent event ID (for causal tracking)
    pub parent: Option<Uuid>,

    /// Children event IDs spawned by this event
    pub children: Vec<Uuid>,
}

impl EventWithMetadata {
    /// Create new event with metadata
    pub fn new(event: ReasoningEvent) -> Self {
        let priority = event.priority();
        Self {
            event,
            priority,
            momentum: 1.0,
            trigger_count: 0,
            parent: None,
            children: Vec::new(),
        }
    }

    /// Create with parent relationship
    pub fn with_parent(event: ReasoningEvent, parent: Uuid) -> Self {
        let mut meta = Self::new(event);
        meta.parent = Some(parent);
        meta
    }

    /// Get effective priority (base priority * momentum)
    pub fn effective_priority(&self) -> f32 {
        self.priority * self.momentum
    }

    /// Update momentum based on trigger frequency
    pub fn update_momentum(&mut self) {
        self.trigger_count += 1;

        // Boost momentum for frequently triggered events
        // Uses logarithmic scaling to prevent unbounded growth
        self.momentum = 1.0 + (self.trigger_count as f32).ln() * 0.1;
    }

    /// Add child event
    pub fn add_child(&mut self, child_id: Uuid) {
        self.children.push(child_id);
    }
}

/// Event processing result
#[derive(Debug, Clone)]
pub enum EventResult {
    /// Processing completed successfully
    Success {
        spawned_events: Vec<ReasoningEvent>,
    },

    /// Processing failed with error
    Error {
        error: String,
    },

    /// Processing deferred (backpressure)
    Deferred {
        retry_after_ms: u64,
    },

    /// Event fused with existing event
    Fused {
        fused_with: Uuid,
    },
}

impl EventResult {
    /// Check if result is successful
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    /// Get spawned events
    pub fn spawned_events(&self) -> Vec<ReasoningEvent> {
        match self {
            Self::Success { spawned_events } => spawned_events.clone(),
            _ => Vec::new(),
        }
    }
}

/// Cognitive timestamp for event ordering
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CognitiveTimestamp(u64);

impl CognitiveTimestamp {
    /// Create new timestamp
    pub fn now() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        Self(nanos)
    }

    /// Get raw value
    pub fn value(&self) -> u64 {
        self.0
    }

    /// Create from raw value
    pub fn from_value(value: u64) -> Self {
        Self(value)
    }
}

impl Default for CognitiveTimestamp {
    fn default() -> Self {
        Self::now()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_priority() {
        let session_id = Uuid::new_v4();

        let chunk_req = ReasoningEvent::ChunkRequest {
            session_id,
            prompt: "Test".to_string(),
            priority: 0.5,
            timestamp: 1000,
            level: ReasoningLevel::Macro,
        };

        let term_check = ReasoningEvent::TerminationCheck {
            session_id,
            current_state: Box::new(MarkovianState::new(
                "Test".to_string(),
                crate::state::StateConfig::default(),
            )),
            timestamp: 1001,
        };

        assert_eq!(chunk_req.priority(), 0.5);
        assert_eq!(term_check.priority(), 1.0);
    }

    #[test]
    fn test_reasoning_level() {
        assert_eq!(ReasoningLevel::Micro.complexity(), 1);
        assert_eq!(ReasoningLevel::Meso.complexity(), 10);
        assert_eq!(ReasoningLevel::Macro.complexity(), 100);

        assert!(!ReasoningLevel::Micro.needs_cpu());
        assert!(ReasoningLevel::Meso.needs_cpu());
        assert!(ReasoningLevel::Macro.needs_cpu());
    }

    #[test]
    fn test_event_metadata_momentum() {
        let event = ReasoningEvent::ChunkRequest {
            session_id: Uuid::new_v4(),
            prompt: "Test".to_string(),
            priority: 0.5,
            timestamp: 1000,
            level: ReasoningLevel::Macro,
        };

        let mut meta = EventWithMetadata::new(event);

        assert_eq!(meta.momentum, 1.0);
        assert_eq!(meta.effective_priority(), 0.5);

        // Trigger multiple times
        for _ in 0..10 {
            meta.update_momentum();
        }

        assert!(meta.momentum > 1.0);
        assert!(meta.effective_priority() > 0.5);
    }

    #[test]
    fn test_cognitive_timestamp_ordering() {
        let t1 = CognitiveTimestamp::now();
        std::thread::sleep(std::time::Duration::from_micros(100));
        let t2 = CognitiveTimestamp::now();

        assert!(t2 > t1);
    }

    #[test]
    fn test_event_result() {
        let success = EventResult::Success {
            spawned_events: vec![],
        };
        assert!(success.is_success());

        let error = EventResult::Error {
            error: "Test error".to_string(),
        };
        assert!(!error.is_success());
    }
}
