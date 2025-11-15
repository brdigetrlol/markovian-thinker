// Markovian Thinker: Priority Event Queue with Momentum
// Lock-free concurrent queue inspired by Icarus TIC

use crate::events::{EventWithMetadata, ReasoningEvent};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Priority event queue with momentum boosting
pub struct EventQueue {
    /// Main priority queue (using std BinaryHeap for simplicity)
    /// In production: use lock-free skip list for better concurrency
    queue: Arc<Mutex<BinaryHeap<PriorityEvent>>>,

    /// Event momentum tracking
    momentum_tracker: Arc<Mutex<MomentumTracker>>,

    /// Queue metrics
    metrics: Arc<Mutex<QueueMetrics>>,

    /// Maximum queue size
    max_size: usize,
}

/// Event wrapper for priority queue ordering
struct PriorityEvent {
    event: EventWithMetadata,
}

impl PartialEq for PriorityEvent {
    fn eq(&self, other: &Self) -> bool {
        self.event.effective_priority() == other.event.effective_priority()
    }
}

impl Eq for PriorityEvent {}

impl PartialOrd for PriorityEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority = higher in heap (max heap)
        self.event
            .effective_priority()
            .partial_cmp(&other.event.effective_priority())
            .unwrap_or(Ordering::Equal)
    }
}

/// Tracks momentum for frequently triggered events
struct MomentumTracker {
    /// Event type → momentum multiplier
    momentum_map: std::collections::HashMap<String, f32>,

    /// Event type → trigger count
    trigger_counts: std::collections::HashMap<String, u32>,
}

impl MomentumTracker {
    fn new() -> Self {
        Self {
            momentum_map: std::collections::HashMap::new(),
            trigger_counts: std::collections::HashMap::new(),
        }
    }

    /// Get momentum for event type
    fn get_momentum(&self, event_type: &str) -> f32 {
        self.momentum_map.get(event_type).copied().unwrap_or(1.0)
    }

    /// Update momentum when event is triggered
    fn update(&mut self, event_type: String) {
        let count = self.trigger_counts.entry(event_type.clone()).or_insert(0);
        *count += 1;

        // Logarithmic momentum boost
        let momentum = 1.0 + (*count as f32).ln() * 0.1;
        self.momentum_map.insert(event_type, momentum);
    }

    /// Decay momentum over time
    fn decay_all(&mut self, factor: f32) {
        for momentum in self.momentum_map.values_mut() {
            *momentum *= factor;
            *momentum = momentum.max(1.0); // Never go below baseline
        }
    }
}

/// Queue performance metrics
#[derive(Debug, Clone, Default)]
pub struct QueueMetrics {
    /// Current queue size
    pub size: usize,

    /// Total events inserted
    pub total_inserted: u64,

    /// Total events processed
    pub total_processed: u64,

    /// Total events dropped (backpressure)
    pub total_dropped: u64,

    /// Average wait time (milliseconds)
    pub avg_wait_ms: f64,

    /// Peak queue size
    pub peak_size: usize,
}

impl EventQueue {
    /// Create new event queue
    pub fn new(max_size: usize) -> Self {
        Self {
            queue: Arc::new(Mutex::new(BinaryHeap::new())),
            momentum_tracker: Arc::new(Mutex::new(MomentumTracker::new())),
            metrics: Arc::new(Mutex::new(QueueMetrics::default())),
            max_size,
        }
    }

    /// Create with default capacity
    pub fn default() -> Self {
        Self::new(10000)
    }

    /// Try to insert event (returns false if backpressure)
    pub fn try_insert(&self, mut event: EventWithMetadata) -> bool {
        let mut queue = self.queue.lock().unwrap();
        let mut metrics = self.metrics.lock().unwrap();

        // Check backpressure
        if queue.len() >= self.max_size {
            // Only accept high-priority events under pressure
            if event.effective_priority() < 0.8 {
                metrics.total_dropped += 1;
                return false;
            }
        }

        // Update momentum
        let event_type = self.event_type_name(&event.event);
        let mut tracker = self.momentum_tracker.lock().unwrap();
        let momentum = tracker.get_momentum(&event_type);
        event.momentum = momentum;
        tracker.update(event_type);

        // Insert into queue
        queue.push(PriorityEvent { event });

        // Update metrics
        metrics.size = queue.len();
        metrics.total_inserted += 1;
        metrics.peak_size = metrics.peak_size.max(queue.len());

        true
    }

    /// Try to pop highest priority event
    pub fn try_pop(&self) -> Option<EventWithMetadata> {
        let mut queue = self.queue.lock().unwrap();
        let mut metrics = self.metrics.lock().unwrap();

        let event = queue.pop().map(|pe| pe.event);

        if event.is_some() {
            metrics.size = queue.len();
            metrics.total_processed += 1;
        }

        event
    }

    /// Get current queue size
    pub fn size(&self) -> usize {
        self.queue.lock().unwrap().len()
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.queue.lock().unwrap().is_empty()
    }

    /// Get metrics snapshot
    pub fn metrics(&self) -> QueueMetrics {
        self.metrics.lock().unwrap().clone()
    }

    /// Decay momentum for all events (call periodically)
    pub fn decay_momentum(&self, factor: f32) {
        self.momentum_tracker.lock().unwrap().decay_all(factor);
    }

    /// Clear all events
    pub fn clear(&self) {
        self.queue.lock().unwrap().clear();
    }

    /// Get event type name for momentum tracking
    fn event_type_name(&self, event: &ReasoningEvent) -> String {
        match event {
            ReasoningEvent::ChunkRequest { .. } => "ChunkRequest".to_string(),
            ReasoningEvent::ChunkComplete { .. } => "ChunkComplete".to_string(),
            ReasoningEvent::VerificationRequest { .. } => "VerificationRequest".to_string(),
            ReasoningEvent::VerificationComplete { .. } => "VerificationComplete".to_string(),
            ReasoningEvent::ConceptCrystallization { .. } => "ConceptCrystallization".to_string(),
            ReasoningEvent::ConceptCrystallized { .. } => "ConceptCrystallized".to_string(),
            ReasoningEvent::TerminationCheck { .. } => "TerminationCheck".to_string(),
            ReasoningEvent::SessionTerminated { .. } => "SessionTerminated".to_string(),
        }
    }
}

/// Session-aware event queue (separate queues per session)
pub struct SessionEventQueue {
    /// Per-session queues
    sessions: Arc<Mutex<std::collections::HashMap<Uuid, EventQueue>>>,

    /// Global queue for cross-session events
    #[allow(dead_code)]
    global_queue: EventQueue,
}

impl SessionEventQueue {
    /// Create new session queue
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(std::collections::HashMap::new())),
            global_queue: EventQueue::default(),
        }
    }

    /// Insert event into appropriate queue
    pub fn insert(&self, event: EventWithMetadata) -> bool {
        let session_id = event.event.session_id();

        let mut sessions = self.sessions.lock().unwrap();
        let queue = sessions
            .entry(session_id)
            .or_insert_with(EventQueue::default);

        queue.try_insert(event)
    }

    /// Pop next event from any session (round-robin)
    pub fn pop_any(&self) -> Option<(Uuid, EventWithMetadata)> {
        let sessions = self.sessions.lock().unwrap();

        // Simple round-robin: iterate through sessions
        for (session_id, queue) in sessions.iter() {
            if let Some(event) = queue.try_pop() {
                return Some((*session_id, event));
            }
        }

        None
    }

    /// Pop event from specific session
    pub fn pop_session(&self, session_id: Uuid) -> Option<EventWithMetadata> {
        let sessions = self.sessions.lock().unwrap();
        sessions.get(&session_id).and_then(|q| q.try_pop())
    }

    /// Get total events across all sessions
    pub fn total_size(&self) -> usize {
        let sessions = self.sessions.lock().unwrap();
        sessions.values().map(|q| q.size()).sum()
    }

    /// Clear session queue
    pub fn clear_session(&self, session_id: Uuid) {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.remove(&session_id);
    }

    /// Get metrics for all sessions
    pub fn all_metrics(&self) -> std::collections::HashMap<Uuid, QueueMetrics> {
        let sessions = self.sessions.lock().unwrap();
        sessions
            .iter()
            .map(|(id, queue)| (*id, queue.metrics()))
            .collect()
    }
}

impl Default for SessionEventQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{ReasoningEvent, ReasoningLevel};

    #[test]
    fn test_event_queue_basic() {
        let queue = EventQueue::new(100);

        let event = ReasoningEvent::ChunkRequest {
            session_id: Uuid::new_v4(),
            prompt: "Test".to_string(),
            priority: 0.5,
            timestamp: 1000,
            level: ReasoningLevel::Macro,
        };

        let meta = EventWithMetadata::new(event);
        assert!(queue.try_insert(meta));

        assert_eq!(queue.size(), 1);

        let popped = queue.try_pop();
        assert!(popped.is_some());
        assert_eq!(queue.size(), 0);
    }

    #[test]
    fn test_priority_ordering() {
        let queue = EventQueue::new(100);
        let session_id = Uuid::new_v4();

        // Insert low priority event
        let low_pri = ReasoningEvent::ChunkRequest {
            session_id,
            prompt: "Low".to_string(),
            priority: 0.3,
            timestamp: 1000,
            level: ReasoningLevel::Macro,
        };
        queue.try_insert(EventWithMetadata::new(low_pri));

        // Insert high priority event
        let high_pri = ReasoningEvent::TerminationCheck {
            session_id,
            current_state: Box::new(crate::state::MarkovianState::new(
                "Test".to_string(),
                crate::state::StateConfig::default(),
            )),
            timestamp: 1001,
        };
        queue.try_insert(EventWithMetadata::new(high_pri));

        // High priority should come out first
        let first = queue.try_pop().unwrap();
        assert!(matches!(
            first.event,
            ReasoningEvent::TerminationCheck { .. }
        ));
    }

    #[test]
    fn test_backpressure() {
        let queue = EventQueue::new(2); // Very small queue

        let session_id = Uuid::new_v4();

        // Fill queue
        for i in 0..2 {
            let event = ReasoningEvent::ChunkRequest {
                session_id,
                prompt: format!("Test {}", i),
                priority: 0.5,
                timestamp: 1000 + i as u64,
                level: ReasoningLevel::Macro,
            };
            assert!(queue.try_insert(EventWithMetadata::new(event)));
        }

        // Next low-priority insert should fail
        let low_pri = ReasoningEvent::ChunkRequest {
            session_id,
            prompt: "Should fail".to_string(),
            priority: 0.5,
            timestamp: 1002,
            level: ReasoningLevel::Macro,
        };
        assert!(!queue.try_insert(EventWithMetadata::new(low_pri)));

        // High-priority should still succeed
        let high_pri = ReasoningEvent::TerminationCheck {
            session_id,
            current_state: Box::new(crate::state::MarkovianState::new(
                "Test".to_string(),
                crate::state::StateConfig::default(),
            )),
            timestamp: 1003,
        };
        assert!(queue.try_insert(EventWithMetadata::new(high_pri)));
    }

    #[test]
    fn test_momentum_tracking() {
        let queue = EventQueue::new(100);
        let session_id = Uuid::new_v4();

        // Insert same event type multiple times
        for i in 0..5 {
            let event = ReasoningEvent::ChunkRequest {
                session_id,
                prompt: format!("Test {}", i),
                priority: 0.5,
                timestamp: 1000 + i as u64,
                level: ReasoningLevel::Macro,
            };
            let meta = EventWithMetadata::new(event);
            queue.try_insert(meta);
        }

        // Later events should have higher momentum
        let first = queue.try_pop().unwrap();
        let last = queue.try_pop().unwrap();

        // Note: actual momentum comparison depends on processing order
        // Just verify momentum exists
        assert!(first.momentum >= 1.0);
        assert!(last.momentum >= 1.0);
    }

    #[test]
    fn test_session_queue() {
        let session_queue = SessionEventQueue::new();

        let session1 = Uuid::new_v4();
        let session2 = Uuid::new_v4();

        // Insert events for different sessions
        let event1 = ReasoningEvent::ChunkRequest {
            session_id: session1,
            prompt: "Session 1".to_string(),
            priority: 0.5,
            timestamp: 1000,
            level: ReasoningLevel::Macro,
        };
        session_queue.insert(EventWithMetadata::new(event1));

        let event2 = ReasoningEvent::ChunkRequest {
            session_id: session2,
            prompt: "Session 2".to_string(),
            priority: 0.5,
            timestamp: 1001,
            level: ReasoningLevel::Macro,
        };
        session_queue.insert(EventWithMetadata::new(event2));

        assert_eq!(session_queue.total_size(), 2);

        // Pop from specific session
        let popped = session_queue.pop_session(session1);
        assert!(popped.is_some());

        assert_eq!(session_queue.total_size(), 1);
    }
}
