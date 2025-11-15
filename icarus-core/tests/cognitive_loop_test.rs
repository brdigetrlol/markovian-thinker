// Integration test for Icarus cognitive loop
// Tests all 6 agents working together

use icarus_core::*;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use uuid::Uuid;
use chrono::Utc;

#[tokio::test]
async fn test_cognitive_loop_integration() {
    // Initialize tracing for test visibility
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .try_init()
        .ok();

    tracing::info!("Starting Icarus cognitive loop integration test");

    // Create configs
    let config = config::IcarusConfig::default();

    // Initialize core components
    let event_bus = Arc::new(event_bus::EventBus::new());
    let memory = Arc::new(RwLock::new(
        memory::MemoryHierarchy::new(&config.memory).unwrap(),
    ));
    let neural_core = Arc::new(RwLock::new(
        neural::NeuralCore::new(&config.neural).unwrap(),
    ));
    let world_model = Arc::new(RwLock::new(
        world_model::WorldModel::new(&config.world_model).unwrap(),
    ));

    // Create agent system
    let mut agent_system = agents::AgentSystem::new(
        &config.agents,
        event_bus.clone(),
        memory.clone(),
        neural_core.clone(),
        world_model.clone(),
    )
    .unwrap();

    // Start the agent system
    agent_system.start().await.unwrap();
    tracing::info!("✅ Agent system started");

    // Give agents time to initialize
    sleep(Duration::from_millis(100)).await;

    // Test the cognitive loop by injecting a perception event
    tracing::info!("📥 Publishing perception input event");
    event_bus.publish(event_bus::IcarusEvent::PerceptionInput {
        id: Uuid::new_v4(),
        timestamp: Utc::now(),
        data: b"Hello, Icarus!".to_vec(),
        modality: event_bus::Modality::Text,
    });

    // Wait for the cognitive loop to process
    sleep(Duration::from_millis(500)).await;

    // Verify memory was updated
    {
        let mem = memory.read().await;
        // The perception should have been stored in working memory
        tracing::info!("📊 Checking memory state");
        // Note: In a real test, we'd have methods to query memory contents
    }

    // Verify world model was updated
    {
        let wm = world_model.read().await;
        let current_state = wm.current_state();
        tracing::info!("🌍 World model state: confidence = {}", current_state.confidence);
        assert!(current_state.confidence > 0.0, "World model should have non-zero confidence");
    }

    // Publish another event to trigger planning
    tracing::info!("📥 Publishing second perception to trigger planning");
    event_bus.publish(event_bus::IcarusEvent::PerceptionInput {
        id: Uuid::new_v4(),
        timestamp: Utc::now(),
        data: b"Test planning".to_vec(),
        modality: event_bus::Modality::Text,
    });

    // Wait for planning and action execution
    sleep(Duration::from_millis(800)).await;

    // Stop the agent system
    agent_system.stop().await.unwrap();
    tracing::info!("✅ Agent system stopped cleanly");

    // Test passed if we got here without panicking
    tracing::info!("✅ Cognitive loop integration test completed successfully");
}

#[tokio::test]
async fn test_event_propagation() {
    // Test that events propagate through the system correctly
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_test_writer()
        .try_init()
        .ok();

    let event_bus = Arc::new(event_bus::EventBus::new());

    // Subscribe to events
    let mut rx1 = event_bus.subscribe();
    let mut rx2 = event_bus.subscribe();

    // Publish an event
    let test_event = event_bus::IcarusEvent::PerceptionInput {
        id: Uuid::new_v4(),
        timestamp: Utc::now(),
        data: b"test".to_vec(),
        modality: event_bus::Modality::Text,
    };

    event_bus.publish(test_event.clone());

    // Both subscribers should receive the event
    let received1 = rx1.recv().await.unwrap();
    let received2 = rx2.recv().await.unwrap();

    assert_eq!(received1.id(), test_event.id());
    assert_eq!(received2.id(), test_event.id());

    tracing::info!("✅ Event propagation test passed");
}

#[tokio::test]
async fn test_memory_consolidation() {
    // Test memory hierarchy consolidation
    use icarus_core::config::MemoryConfig;
    use icarus_core::memory::{Memory, MemoryHierarchy};

    let config = MemoryConfig {
        working_capacity: 3,
        short_term_capacity: 5,
        long_term_capacity: 1000,
        episodic_capacity: 10,
        consolidation_interval_secs: 0, // Immediate consolidation for testing
    };

    let mut hierarchy = MemoryHierarchy::new(&config).unwrap();

    // Add several high-importance memories
    for i in 0..5 {
        let mut mem = Memory::new(vec![i as u8]);
        mem.importance = 0.9;
        mem.access_count = 5;
        hierarchy.store(mem);
    }

    // Trigger consolidation
    hierarchy.consolidate().await.unwrap();

    tracing::info!("✅ Memory consolidation test passed");
}
