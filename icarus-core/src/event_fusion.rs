// Markovian Thinker: Event Fusion for Storm Mitigation
// Deduplicates and merges similar pending events

use crate::events::{EventWithMetadata, ReasoningEvent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Event fusion configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFusionConfig {
    /// Similarity threshold for fusion (0.0-1.0)
    pub similarity_threshold: f64,

    /// Enable fusion for chunk requests
    pub fuse_chunk_requests: bool,

    /// Enable fusion for verification requests
    pub fuse_verification_requests: bool,

    /// Enable fusion for concept crystallization
    pub fuse_concept_events: bool,

    /// Maximum events in fusion window
    pub max_window_size: usize,
}

impl Default for EventFusionConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.8, // 80% similar = fuse
            fuse_chunk_requests: true,
            fuse_verification_requests: true,
            fuse_concept_events: false, // Concepts need precision
            max_window_size: 100,
        }
    }
}

impl EventFusionConfig {
    /// Aggressive fusion (more deduplication)
    pub fn aggressive() -> Self {
        Self {
            similarity_threshold: 0.7,
            fuse_chunk_requests: true,
            fuse_verification_requests: true,
            fuse_concept_events: true,
            max_window_size: 200,
        }
    }

    /// Conservative fusion (less deduplication)
    pub fn conservative() -> Self {
        Self {
            similarity_threshold: 0.95,
            fuse_chunk_requests: true,
            fuse_verification_requests: false,
            fuse_concept_events: false,
            max_window_size: 50,
        }
    }
}

/// Event fusion engine
pub struct EventFusion {
    config: EventFusionConfig,
}

impl EventFusion {
    /// Create a new event fusion engine
    pub fn new(config: EventFusionConfig) -> Self {
        Self { config }
    }

    /// Fuse similar events in a batch
    pub fn fuse_events(&self, events: Vec<EventWithMetadata>) -> Vec<EventWithMetadata> {
        if events.is_empty() {
            return events;
        }

        // Limit window size
        let events = if events.len() > self.config.max_window_size {
            events.into_iter().take(self.config.max_window_size).collect()
        } else {
            events
        };

        // Group by event type
        let mut groups: HashMap<String, Vec<EventWithMetadata>> = HashMap::new();

        for event in events {
            let type_key = self.event_type_key(&event.event);
            groups.entry(type_key).or_insert_with(Vec::new).push(event);
        }

        // Fuse within each group
        let mut result = Vec::new();
        for (_type_key, group) in groups {
            result.extend(self.fuse_group(group));
        }

        result
    }

    /// Fuse events within a single type group
    fn fuse_group(&self, events: Vec<EventWithMetadata>) -> Vec<EventWithMetadata> {
        if events.len() <= 1 {
            return events;
        }

        let mut result = Vec::new();
        let mut skip_indices = std::collections::HashSet::new();

        for i in 0..events.len() {
            if skip_indices.contains(&i) {
                continue;
            }

            let mut fused_event = events[i].clone();
            let mut fused_with = Vec::new();

            // Try to fuse with remaining events
            for j in (i + 1)..events.len() {
                if skip_indices.contains(&j) {
                    continue;
                }

                let similarity = self.compute_similarity(&events[i].event, &events[j].event);

                if similarity >= self.config.similarity_threshold {
                    // Fuse events
                    fused_event = self.merge_events(fused_event, events[j].clone());
                    fused_with.push(j);
                    skip_indices.insert(j);
                }
            }

            result.push(fused_event);
        }

        result
    }

    /// Compute similarity between two events (0.0-1.0)
    fn compute_similarity(&self, e1: &ReasoningEvent, e2: &ReasoningEvent) -> f64 {
        use ReasoningEvent::*;

        match (e1, e2) {
            (ChunkRequest { prompt: p1, .. }, ChunkRequest { prompt: p2, .. }) => {
                if !self.config.fuse_chunk_requests {
                    return 0.0;
                }
                self.string_similarity(p1, p2)
            }
            (VerificationRequest { hypothesis: h1, .. }, VerificationRequest { hypothesis: h2, .. }) => {
                if !self.config.fuse_verification_requests {
                    return 0.0;
                }
                self.string_similarity(h1, h2)
            }
            (
                ConceptCrystallization { concept: c1, .. },
                ConceptCrystallization { concept: c2, .. },
            ) => {
                if !self.config.fuse_concept_events {
                    return 0.0;
                }
                if c1 == c2 {
                    1.0
                } else {
                    0.0
                }
            }
            _ => 0.0, // Different event types
        }
    }

    /// Compute string similarity (Jaccard similarity of words)
    fn string_similarity(&self, s1: &str, s2: &str) -> f64 {
        let words1: std::collections::HashSet<&str> =
            s1.split_whitespace().collect();
        let words2: std::collections::HashSet<&str> =
            s2.split_whitespace().collect();

        if words1.is_empty() && words2.is_empty() {
            return 1.0;
        }

        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Merge two events (keeps metadata from first, may combine fields)
    fn merge_events(
        &self,
        mut e1: EventWithMetadata,
        e2: EventWithMetadata,
    ) -> EventWithMetadata {
        // Boost priority (take maximum)
        e1.priority = e1.priority.max(e2.priority);

        // Boost momentum
        e1.momentum += e2.momentum / 2.0; // Half of second event's momentum

        e1
    }

    /// Get event type key for grouping
    fn event_type_key(&self, event: &ReasoningEvent) -> String {
        use ReasoningEvent::*;

        match event {
            ChunkRequest { .. } => "chunk_request".to_string(),
            ChunkComplete { .. } => "chunk_complete".to_string(),
            VerificationRequest { .. } => "verification_request".to_string(),
            VerificationComplete { .. } => "verification_complete".to_string(),
            ConceptCrystallization { .. } => "concept_crystallization".to_string(),
            ConceptCrystallized { .. } => "concept_crystallized".to_string(),
            TerminationCheck { .. } => "termination_check".to_string(),
            SessionTerminated { .. } => "session_terminated".to_string(),
        }
    }

    /// Get fusion statistics for a batch
    pub fn fusion_stats(&self, original: &[EventWithMetadata], fused: &[EventWithMetadata]) -> FusionStats {
        FusionStats {
            original_count: original.len(),
            fused_count: fused.len(),
            reduction_count: original.len().saturating_sub(fused.len()),
            reduction_rate: if original.len() > 0 {
                (original.len() - fused.len()) as f64 / original.len() as f64
            } else {
                0.0
            },
        }
    }
}

/// Fusion statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionStats {
    pub original_count: usize,
    pub fused_count: usize,
    pub reduction_count: usize,
    pub reduction_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{CognitiveTimestamp, ReasoningLevel};
    use uuid::Uuid;

    fn make_chunk_request(_session_id: &str, prompt: &str, priority: f32) -> EventWithMetadata {
        EventWithMetadata {
            event: ReasoningEvent::ChunkRequest {
                session_id: Uuid::new_v4(),
                prompt: prompt.to_string(),
                priority,
                timestamp: CognitiveTimestamp::now().value(),
                level: ReasoningLevel::Meso,
            },
            priority,
            momentum: 0.0,
            trigger_count: 0,
            parent: None,
            children: Vec::new(),
        }
    }

    fn make_verification_request(hypothesis: &str, priority: f32) -> EventWithMetadata {
        EventWithMetadata {
            event: ReasoningEvent::VerificationRequest {
                session_id: Uuid::new_v4(),
                parent_event: Uuid::new_v4(),
                hypothesis: hypothesis.to_string(),
                timestamp: CognitiveTimestamp::now().value(),
            },
            priority,
            momentum: 0.0,
            trigger_count: 0,
            parent: None,
            children: Vec::new(),
        }
    }

    #[test]
    fn test_event_fusion_creation() {
        let config = EventFusionConfig::default();
        let fusion = EventFusion::new(config);

        assert_eq!(fusion.config.similarity_threshold, 0.8);
    }

    #[test]
    fn test_string_similarity() {
        let config = EventFusionConfig::default();
        let fusion = EventFusion::new(config);

        // Identical strings
        assert_eq!(fusion.string_similarity("hello world", "hello world"), 1.0);

        // Completely different
        assert_eq!(fusion.string_similarity("hello", "goodbye"), 0.0);

        // Partial overlap
        let sim = fusion.string_similarity("hello world", "hello universe");
        assert!(sim > 0.0 && sim < 1.0);
    }

    #[test]
    fn test_fuse_similar_chunk_requests() {
        let config = EventFusionConfig {
            similarity_threshold: 0.5,
            fuse_chunk_requests: true,
            ..Default::default()
        };
        let fusion = EventFusion::new(config);

        let events = vec![
            make_chunk_request("s1", "solve math problem", 1.0),
            make_chunk_request("s1", "solve math equation", 1.0),
        ];

        let fused = fusion.fuse_events(events.clone());

        // Should fuse into 1 event (high similarity)
        assert!(fused.len() < events.len());
    }

    #[test]
    fn test_no_fusion_different_events() {
        let config = EventFusionConfig::default();
        let fusion = EventFusion::new(config);

        let events = vec![
            make_chunk_request("s1", "completely different prompt", 1.0),
            make_chunk_request("s1", "another unrelated prompt", 1.0),
        ];

        let fused = fusion.fuse_events(events.clone());

        // Should not fuse (low similarity)
        assert_eq!(fused.len(), events.len());
    }

    #[test]
    fn test_priority_boost_on_fusion() {
        let config = EventFusionConfig {
            similarity_threshold: 0.5,
            fuse_chunk_requests: true,
            ..Default::default()
        };
        let fusion = EventFusion::new(config);

        let events = vec![
            make_chunk_request("s1", "solve problem", 1.0),
            make_chunk_request("s1", "solve problem", 2.0),
        ];

        let fused = fusion.fuse_events(events);

        assert_eq!(fused.len(), 1);
        // Priority should be max of the two
        assert_eq!(fused[0].priority, 2.0);
    }

    #[test]
    fn test_momentum_accumulation() {
        let config = EventFusionConfig {
            similarity_threshold: 0.9,
            fuse_chunk_requests: true,
            ..Default::default()
        };
        let fusion = EventFusion::new(config);

        let mut e1 = make_chunk_request("s1", "same prompt", 1.0);
        let mut e2 = make_chunk_request("s1", "same prompt", 1.0);
        e1.momentum = 1.0;
        e2.momentum = 1.0;

        let fused = fusion.fuse_events(vec![e1, e2]);

        assert_eq!(fused.len(), 1);
        // Momentum should accumulate (half of second)
        assert_eq!(fused[0].momentum, 1.5);
    }

    #[test]
    fn test_fusion_disabled_for_type() {
        let config = EventFusionConfig {
            similarity_threshold: 0.8,
            fuse_chunk_requests: false, // Disabled
            ..Default::default()
        };
        let fusion = EventFusion::new(config);

        let events = vec![
            make_chunk_request("s1", "same prompt", 1.0),
            make_chunk_request("s1", "same prompt", 1.0),
        ];

        let fused = fusion.fuse_events(events.clone());

        // Should not fuse (disabled)
        assert_eq!(fused.len(), events.len());
    }

    #[test]
    fn test_verification_request_fusion() {
        let config = EventFusionConfig {
            similarity_threshold: 0.7,
            fuse_verification_requests: true,
            ..Default::default()
        };
        let fusion = EventFusion::new(config);

        let events = vec![
            make_verification_request("hypothesis about math", 1.0),
            make_verification_request("hypothesis about math problem", 1.0),
        ];

        let fused = fusion.fuse_events(events.clone());

        // Should fuse (similar hypotheses)
        assert!(fused.len() < events.len());
    }

    #[test]
    fn test_window_size_limit() {
        let config = EventFusionConfig {
            similarity_threshold: 0.8,
            max_window_size: 5,
            ..Default::default()
        };
        let fusion = EventFusion::new(config);

        // Create 10 events
        let events: Vec<_> = (0..10)
            .map(|i| make_chunk_request("s1", &format!("prompt {}", i), 1.0))
            .collect();

        let fused = fusion.fuse_events(events);

        // Should limit to window size
        assert!(fused.len() <= 5);
    }

    #[test]
    fn test_fusion_stats() {
        let config = EventFusionConfig {
            similarity_threshold: 0.8,
            fuse_chunk_requests: true,
            ..Default::default()
        };
        let fusion = EventFusion::new(config);

        let original = vec![
            make_chunk_request("s1", "same", 1.0),
            make_chunk_request("s1", "same", 1.0),
            make_chunk_request("s1", "same", 1.0),
        ];

        let fused = fusion.fuse_events(original.clone());
        let stats = fusion.fusion_stats(&original, &fused);

        assert_eq!(stats.original_count, 3);
        assert!(stats.fused_count < 3);
        assert_eq!(stats.reduction_count, stats.original_count - stats.fused_count);
    }

    #[test]
    fn test_config_presets() {
        let aggressive = EventFusionConfig::aggressive();
        assert_eq!(aggressive.similarity_threshold, 0.7);
        assert!(aggressive.fuse_concept_events);

        let conservative = EventFusionConfig::conservative();
        assert_eq!(conservative.similarity_threshold, 0.95);
        assert!(!conservative.fuse_concept_events);
    }

    #[test]
    fn test_empty_events() {
        let config = EventFusionConfig::default();
        let fusion = EventFusion::new(config);

        let fused = fusion.fuse_events(vec![]);
        assert_eq!(fused.len(), 0);
    }

    #[test]
    fn test_single_event() {
        let config = EventFusionConfig::default();
        let fusion = EventFusion::new(config);

        let events = vec![make_chunk_request("s1", "prompt", 1.0)];
        let fused = fusion.fuse_events(events.clone());

        assert_eq!(fused.len(), 1);
    }
}
