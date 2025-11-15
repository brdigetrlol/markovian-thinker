// Event Bus
// Central communication hub for the 6-agent system

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Event types in the Icarus system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IcarusEvent {
    /// Perception agent detected new input
    PerceptionInput {
        id: Uuid,
        timestamp: DateTime<Utc>,
        data: Vec<u8>,
        modality: Modality,
    },

    /// World model updated its state
    WorldModelUpdated {
        id: Uuid,
        timestamp: DateTime<Utc>,
        predictions: Vec<f32>,
    },

    /// Planning agent generated new plan
    PlanGenerated {
        id: Uuid,
        timestamp: DateTime<Utc>,
        plan: String,
        priority: f32,
    },

    /// Memory system stored/retrieved data
    MemoryOperation {
        id: Uuid,
        timestamp: DateTime<Utc>,
        operation: MemoryOp,
        level: String,
    },

    /// Action agent executed action
    ActionExecuted {
        id: Uuid,
        timestamp: DateTime<Utc>,
        action: String,
        success: bool,
    },

    /// Learning agent updated model
    LearningUpdate {
        id: Uuid,
        timestamp: DateTime<Utc>,
        strategy: String,
    },

    /// System control events
    SystemControl {
        id: Uuid,
        timestamp: DateTime<Utc>,
        command: SystemCommand,
    },
}

/// Input modality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Modality {
    Text,
    Code,
    Data,
    Mixed,
}

/// Memory operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryOp {
    Store,
    Retrieve,
    Consolidate,
    Forget,
}

/// System commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemCommand {
    Shutdown,
    Pause,
    Resume,
    Reset,
}

impl IcarusEvent {
    pub fn id(&self) -> Uuid {
        match self {
            IcarusEvent::PerceptionInput { id, .. } => *id,
            IcarusEvent::WorldModelUpdated { id, .. } => *id,
            IcarusEvent::PlanGenerated { id, .. } => *id,
            IcarusEvent::MemoryOperation { id, .. } => *id,
            IcarusEvent::ActionExecuted { id, .. } => *id,
            IcarusEvent::LearningUpdate { id, .. } => *id,
            IcarusEvent::SystemControl { id, .. } => *id,
        }
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            IcarusEvent::PerceptionInput { timestamp, .. } => *timestamp,
            IcarusEvent::WorldModelUpdated { timestamp, .. } => *timestamp,
            IcarusEvent::PlanGenerated { timestamp, .. } => *timestamp,
            IcarusEvent::MemoryOperation { timestamp, .. } => *timestamp,
            IcarusEvent::ActionExecuted { timestamp, .. } => *timestamp,
            IcarusEvent::LearningUpdate { timestamp, .. } => *timestamp,
            IcarusEvent::SystemControl { timestamp, .. } => *timestamp,
        }
    }
}

/// Event Bus - Broadcast channel for agent communication
pub struct EventBus {
    sender: broadcast::Sender<IcarusEvent>,
}

impl EventBus {
    /// Create new event bus
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000);
        Self { sender }
    }

    /// Publish an event to all subscribers
    pub fn publish(&self, event: IcarusEvent) {
        if self.sender.receiver_count() > 0 {
            let _ = self.sender.send(event);
        }
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<IcarusEvent> {
        self.sender.subscribe()
    }

    /// Get number of active subscribers
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_bus_publish_subscribe() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe();

        let event = IcarusEvent::PerceptionInput {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            data: vec![1, 2, 3],
            modality: Modality::Text,
        };

        bus.publish(event.clone());

        let received = rx.recv().await.unwrap();
        assert_eq!(received.id(), event.id());
    }

    #[test]
    fn test_subscriber_count() {
        let bus = EventBus::new();
        assert_eq!(bus.subscriber_count(), 0);

        let _rx1 = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 1);

        let _rx2 = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 2);
    }
}
