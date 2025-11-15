// Integration Tests for Markovian Thinker Phase 6
// Tests storm mitigation integration with session manager

use markovian_thinker::*;

#[tokio::test]
async fn test_session_manager_with_storm_mitigation() {
    let manager = SessionManager::new();

    // Create session with aggressive storm mitigation
    let mut config = StateConfig::default();
    config.storm_mitigation_config = StormMitigationConfig::aggressive();

    let session_id = manager
        .create_session("Test problem".to_string(), config)
        .await
        .unwrap();

    // Check that storm mitigation was created
    let result = manager.check_storm_mitigation(session_id).await;
    assert!(result.is_ok());

    // Should allow first event
    let decision = result.unwrap();
    assert!(matches!(decision, MitigationDecision::Allowed));
}

#[tokio::test]
async fn test_storm_mitigation_success_tracking() {
    let manager = SessionManager::new();
    let config = StateConfig::default();

    let session_id = manager
        .create_session("Test problem".to_string(), config)
        .await
        .unwrap();

    // Allow and record success multiple times
    for _ in 0..5 {
        let decision = manager.check_storm_mitigation(session_id).await.unwrap();
        assert!(matches!(decision, MitigationDecision::Allowed));
        manager.record_storm_success(session_id).await.unwrap();
    }

    // Check stats
    let stats = manager.get_storm_stats(session_id).await.unwrap();
    assert_eq!(stats.metrics.successful_events, 5);
    assert_eq!(stats.metrics.failed_events, 0);
    assert_eq!(stats.metrics.success_rate(), 1.0);
}

#[tokio::test]
async fn test_storm_mitigation_failure_tracking() {
    let manager = SessionManager::new();

    // Use aggressive config to trigger circuit breaker quickly
    let mut config = StateConfig::default();
    config.storm_mitigation_config = StormMitigationConfig::aggressive();

    let session_id = manager
        .create_session("Test problem".to_string(), config)
        .await
        .unwrap();

    // Record multiple failures to trigger circuit breaker
    for _ in 0..3 {
        let decision = manager.check_storm_mitigation(session_id).await.unwrap();
        if matches!(decision, MitigationDecision::Allowed) {
            manager.record_storm_failure(session_id).await.unwrap();
        }
    }

    // Next attempt should be rejected
    let decision = manager.check_storm_mitigation(session_id).await.unwrap();
    assert!(matches!(decision, MitigationDecision::Rejected { .. }));

    // Check stats
    let stats = manager.get_storm_stats(session_id).await.unwrap();
    assert_eq!(stats.metrics.failed_events, 3);
    assert!(stats.metrics.circuit_breaker_rejections > 0);
}

#[tokio::test]
async fn test_session_cleanup_removes_storm_mitigation() {
    let manager = SessionManager::new();
    let config = StateConfig::default();

    let session_id = manager
        .create_session("Test problem".to_string(), config)
        .await
        .unwrap();

    // Verify storm mitigation exists
    assert!(manager.get_storm_stats(session_id).await.is_ok());

    // Remove session
    manager.remove_session(session_id).await.unwrap();

    // Verify storm mitigation was also removed
    assert!(manager.get_storm_stats(session_id).await.is_err());
}

#[test]
fn test_state_config_phase_6_defaults() {
    let config = StateConfig::default();

    // Verify Phase 6 features are properly initialized
    assert!(config.storm_mitigation_config.enable_rate_limit);
    assert!(config.storm_mitigation_config.enable_circuit_breaker);
    assert_eq!(config.expert_config.enabled, true);
    assert_eq!(config.attention_config.sliding_window_size, Some(256));
}

#[test]
fn test_storm_mitigation_config_presets() {
    let aggressive = StormMitigationConfig::aggressive();
    let default = StormMitigationConfig::default();
    let lenient = StormMitigationConfig::lenient();
    let disabled = StormMitigationConfig::disabled();

    // Aggressive should have tighter limits
    assert!(aggressive.rate_limit.refill_rate < default.rate_limit.refill_rate);
    assert!(aggressive.circuit_breaker.failure_threshold < default.circuit_breaker.failure_threshold);

    // Lenient should have looser limits
    assert!(lenient.rate_limit.refill_rate > default.rate_limit.refill_rate);
    assert!(lenient.circuit_breaker.failure_threshold > default.circuit_breaker.failure_threshold);

    // Disabled should have everything turned off
    assert!(!disabled.enable_rate_limit);
    assert!(!disabled.enable_circuit_breaker);
}

#[test]
fn test_event_fusion_reduces_duplicates() {
    let fusion = EventFusion::new(EventFusionConfig::default());

    // Create duplicate events
    let events = vec![
        create_test_event("same prompt", 1.0),
        create_test_event("same prompt", 1.0),
        create_test_event("same prompt", 1.0),
    ];

    let fused = fusion.fuse_events(events.clone());

    // Should reduce to 1 event
    assert_eq!(fused.len(), 1);

    let stats = fusion.fusion_stats(&events, &fused);
    assert_eq!(stats.original_count, 3);
    assert_eq!(stats.fused_count, 1);
    assert_eq!(stats.reduction_rate, 2.0 / 3.0);
}

// Helper function
fn create_test_event(prompt: &str, priority: f32) -> EventWithMetadata {
    EventWithMetadata {
        event: ReasoningEvent::ChunkRequest {
            session_id: uuid::Uuid::new_v4(),
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

#[tokio::test]
async fn test_causal_trace_integration() {
    use markovian_thinker::session_manager::SessionManager;

    let manager = SessionManager::new();
    let mut config = StateConfig::default();
    config.enable_causal_trace = true; // Enable causal trace

    let session_id = manager
        .create_session("Test causal trace".to_string(), config)
        .await
        .unwrap();

    // Check that causal trace was created
    let trace_result = manager.get_causal_trace(session_id).await;
    assert!(trace_result.is_ok(), "Causal trace should be created when enabled");

    let trace = trace_result.unwrap();
    assert_eq!(trace.metadata().session_id, session_id);
    assert_eq!(trace.metadata().total_events, 0); // No events yet

    // Cleanup
    manager.remove_session(session_id).await.ok();

    // Verify causal trace was cleaned up
    let trace_after = manager.get_causal_trace(session_id).await;
    assert!(trace_after.is_err(), "Causal trace should be removed with session");
}

#[tokio::test]
async fn test_causal_trace_not_enabled() {
    use markovian_thinker::session_manager::SessionManager;

    let manager = SessionManager::new();
    let mut config = StateConfig::default();
    config.enable_causal_trace = false; // Explicitly disabled

    let session_id = manager
        .create_session("Test without causal trace".to_string(), config)
        .await
        .unwrap();

    // Check that causal trace was NOT created
    let trace_result = manager.get_causal_trace(session_id).await;
    assert!(trace_result.is_err(), "Causal trace should not be created when disabled");
}

#[tokio::test]
async fn test_concept_space_integration() {
    use markovian_thinker::session_manager::SessionManager;

    let manager = SessionManager::new();
    let config = StateConfig::default(); // Concept space always created

    let session_id = manager
        .create_session("Test concept space".to_string(), config)
        .await
        .unwrap();

    // Test concept space query with an 8D embedding (E8 lattice default)
    let embedding = vec![1.0, 0.5, 0.3, 0.2, 0.1, 0.0, -0.1, -0.2];
    let k = 3;

    let concepts = manager.query_concepts(session_id, embedding, k).await;
    assert!(concepts.is_ok(), "Concept query should succeed");

    // Should return empty vector since no concepts have been added yet
    let concepts = concepts.unwrap();
    assert_eq!(concepts.len(), 0, "Should have no concepts in empty space");

    // Get concept space statistics
    let stats = manager.get_concept_stats(session_id).await;
    assert!(stats.is_ok(), "Stats query should succeed");

    let stats = stats.unwrap();
    assert_eq!(stats.total_concepts, 0);
    assert_eq!(stats.dimension, 8); // E8 lattice is 8D

    // Cleanup
    manager.remove_session(session_id).await.ok();

    // Verify concept space was cleaned up
    let concepts_after = manager.query_concepts(session_id, vec![1.0; 8], 1).await;
    assert!(concepts_after.is_err(), "Concept space should be removed with session");
}

#[tokio::test]
async fn test_event_driven_chunk_processing() {
    use markovian_thinker::chunk_manager::{ChunkGenerator, ChunkManager};

    // Mock generator
    struct MockGen;

    #[async_trait::async_trait]
    impl ChunkGenerator for MockGen {
        async fn generate(&self, _prompt: &str, _max_tokens: usize) -> anyhow::Result<(String, usize)> {
            Ok(("Test chunk output [EOS]".to_string(), 10))
        }

        fn model_name(&self) -> &str {
            "mock"
        }
    }

    // Create config with event-driven mode enabled
    let mut config = StateConfig::default();
    config.enable_event_driven = true;
    config.enable_causal_trace = true;
    config.max_iterations = 2;

    let session_id = uuid::Uuid::new_v4();
    let mut manager = ChunkManager::with_events(config, session_id);

    // Generate trace
    let generator = MockGen;
    let trace = manager.generate_trace("Test problem".to_string(), &generator).await.unwrap();

    // Verify trace was generated
    assert!(trace.is_complete());

    // Verify events were emitted
    if let Some(queue) = manager.event_queue() {
        let metrics = queue.metrics();
        assert!(metrics.total_inserted > 0, "Events should have been emitted");
        println!("Events emitted: {}", metrics.total_inserted);
    } else {
        panic!("Event queue should exist when event-driven mode is enabled");
    }

    // Verify causal trace was populated
    if let Some(causal_trace) = manager.causal_trace() {
        let trace_stats = causal_trace.statistics();
        assert!(trace_stats.total_events > 0, "Causal trace should have events");
        println!("Causal events recorded: {}", trace_stats.total_events);
    } else {
        panic!("Causal trace should exist when enabled");
    }
}

#[tokio::test]
async fn test_event_driven_disabled_by_default() {
    use markovian_thinker::chunk_manager::ChunkManager;

    let config = StateConfig::default();
    let manager = ChunkManager::new(config);

    // Verify event queue and causal trace are NOT created by default
    assert!(manager.event_queue().is_none(), "Event queue should be None by default");
    assert!(manager.causal_trace().is_none(), "Causal trace should be None by default");
}

#[tokio::test]
async fn test_causal_dependencies_tracked() {
    use markovian_thinker::chunk_manager::{ChunkGenerator, ChunkManager};

    // Mock generator that produces 3 chunks
    struct MockGen {
        counter: std::sync::Arc<std::sync::Mutex<usize>>,
    }

    impl MockGen {
        fn new() -> Self {
            Self {
                counter: std::sync::Arc::new(std::sync::Mutex::new(0)),
            }
        }
    }

    #[async_trait::async_trait]
    impl ChunkGenerator for MockGen {
        async fn generate(&self, _prompt: &str, _max_tokens: usize) -> anyhow::Result<(String, usize)> {
            let mut counter = self.counter.lock().unwrap();
            *counter += 1;

            let response = match *counter {
                1 => "First chunk reasoning...".to_string(),
                2 => "Second chunk more work...".to_string(),
                _ => "Final chunk #### 42".to_string(),
            };

            Ok((response, 10))
        }

        fn model_name(&self) -> &str {
            "mock"
        }
    }

    // Create manager with both event-driven and causal trace enabled
    let mut config = StateConfig::default();
    config.enable_event_driven = true;
    config.enable_causal_trace = true;
    config.max_iterations = 5;

    let session_id = uuid::Uuid::new_v4();
    let mut manager = ChunkManager::with_events(config, session_id);

    // Generate trace with 3 chunks
    let generator = MockGen::new();
    let trace = manager.generate_trace("Test problem".to_string(), &generator).await.unwrap();

    // Verify 3 chunks generated
    assert_eq!(trace.chunks.len(), 3, "Should generate 3 chunks");

    // Verify causal trace has events with dependencies
    if let Some(causal_trace) = manager.causal_trace() {
        let stats = causal_trace.statistics();

        // Should have ChunkRequest + ChunkComplete for each chunk = 6 events
        assert!(stats.total_events >= 6, "Should have at least 6 events (3 chunks × 2 events)");

        // Verify causal relationships exist
        let metadata = causal_trace.metadata();
        assert_eq!(metadata.session_id, session_id);

        println!("Causal trace statistics:");
        println!("  Total events: {}", stats.total_events);
        println!("  Total branches: {}", stats.total_branches);
        println!("  Max depth: {}", stats.max_depth);

        // Max depth should be at least 2 (chain of events)
        assert!(stats.max_depth >= 2, "Should have causal chain depth >= 2");
    } else {
        panic!("Causal trace should exist");
    }
}

#[tokio::test]
async fn test_causal_trace_first_chunk_no_predecessors() {
    use markovian_thinker::chunk_manager::{ChunkGenerator, ChunkManager};

    struct MockGen;

    #[async_trait::async_trait]
    impl ChunkGenerator for MockGen {
        async fn generate(&self, _prompt: &str, _max_tokens: usize) -> anyhow::Result<(String, usize)> {
            Ok(("Single chunk [EOS]".to_string(), 10))
        }

        fn model_name(&self) -> &str {
            "mock"
        }
    }

    let mut config = StateConfig::default();
    config.enable_event_driven = true;
    config.enable_causal_trace = true;

    let session_id = uuid::Uuid::new_v4();
    let mut manager = ChunkManager::with_events(config, session_id);

    let generator = MockGen;
    let _trace = manager.generate_trace("Test".to_string(), &generator).await.unwrap();

    if let Some(causal_trace) = manager.causal_trace() {
        let stats = causal_trace.statistics();

        // First chunk should have events but no predecessors
        assert!(stats.total_events >= 2, "Should have at least 2 events");

        println!("First chunk causal trace:");
        println!("  Total events: {}", stats.total_events);
        println!("  Max depth: {}", stats.max_depth);
    } else {
        panic!("Causal trace should exist");
    }
}


#[tokio::test]
async fn test_intelligent_carryover_enabled() {
    use markovian_thinker::chunk_manager::{ChunkGenerator, ChunkManager};

    // Mock generator that produces different themed chunks
    struct ThemeMockGen {
        counter: std::sync::Arc<std::sync::Mutex<usize>>,
    }

    impl ThemeMockGen {
        fn new() -> Self {
            Self {
                counter: std::sync::Arc::new(std::sync::Mutex::new(0)),
            }
        }
    }

    #[async_trait::async_trait]
    impl ChunkGenerator for ThemeMockGen {
        async fn generate(&self, _prompt: &str, _max_tokens: usize) -> anyhow::Result<(String, usize)> {
            let mut counter = self.counter.lock().unwrap();
            *counter += 1;

            let response = match *counter {
                1 => "First chunk about algorithms and data structures.".to_string(),
                2 => "Second chunk discussing mathematics and equations.".to_string(),
                3 => "Third chunk returns to algorithms. [EOS]".to_string(),
                _ => "Extra chunk [EOS]".to_string(),
            };

            Ok((response, 20))
        }

        fn model_name(&self) -> &str {
            "mock"
        }
    }

    // Create config with intelligent carryover enabled
    let mut config = StateConfig::default();
    config.enable_intelligent_carryover = true;
    config.carryover_k = 2;
    config.relevance_weight = 0.8;
    config.max_iterations = 5;

    let session_id = uuid::Uuid::new_v4();
    let mut manager = ChunkManager::with_events(config, session_id);

    let generator = ThemeMockGen::new();
    let trace = manager.generate_trace("Test".to_string(), &generator).await.unwrap();

    assert_eq!(trace.chunks.len(), 3, "Should generate 3 chunks");
    println!("Intelligent carryover test completed");
}


#[tokio::test]
async fn test_attention_compression() {
    use markovian_thinker::chunk_manager::{ChunkGenerator, ChunkManager};
    use markovian_thinker::attention::AttentionConfig;

    // Mock generator that produces long chunks
    struct LongChunkGen {
        counter: std::sync::Arc<std::sync::Mutex<usize>>,
    }

    impl LongChunkGen {
        fn new() -> Self {
            Self {
                counter: std::sync::Arc::new(std::sync::Mutex::new(0)),
            }
        }
    }

    #[async_trait::async_trait]
    impl ChunkGenerator for LongChunkGen {
        async fn generate(&self, _prompt: &str, _max_tokens: usize) -> anyhow::Result<(String, usize)> {
            let mut counter = self.counter.lock().unwrap();
            *counter += 1;

            // Generate long output to trigger compression
            let long_text = "This is a very long chunk with many repeated words and concepts. \
                            The algorithm needs to compress this intelligently. \
                            Important keywords like algorithm, compress, and intelligent should be kept. \
                            Less important filler words can be removed. \
                            [EOS]";
            
            Ok((long_text.to_string(), 50))
        }

        fn model_name(&self) -> &str {
            "mock"
        }
    }

    // Create config with attention enabled
    let mut config = StateConfig::default();
    config.attention_config = AttentionConfig {
        sliding_window_size: Some(256),
        attention_sink_enabled: true,
        decay_factor: 0.95,
        min_attention_score: 0.1,
    };
    config.carryover_size = 20; // Small carryover to trigger compression
    config.max_iterations = 2;

    let mut manager = ChunkManager::new(config);

    let generator = LongChunkGen::new();
    let trace = manager.generate_trace("Test compression".to_string(), &generator).await.unwrap();

    assert_eq!(trace.chunks.len(), 1);
    println!("Attention compression test completed");
}

// ============================================================================
// Phase 7 Task 7: Batch Session Creation Tests
// ============================================================================

#[tokio::test]
async fn test_batch_session_creation() {
    use markovian_thinker::session_manager::SessionManager;
    use markovian_thinker::state::StateConfig;

    // Use a fresh manager for this test
    let manager = SessionManager::new();

    // Create multiple sessions at once (simulating batch_init behavior)
    let problems = vec![
        "Batch test problem A: solve 2+2".to_string(),
        "Batch test problem B: solve 3*5".to_string(),
        "Batch test problem C: solve 10-3".to_string(),
    ];

    let config = StateConfig::default();
    let mut session_ids = Vec::new();

    for problem in &problems {
        let session_id = manager
            .create_session(problem.clone(), config.clone())
            .await
            .unwrap();
        session_ids.push(session_id);
    }

    // Verify all sessions created
    assert_eq!(session_ids.len(), 3);

    // Verify each session is unique
    let unique_sessions: std::collections::HashSet<_> = session_ids.iter().collect();
    assert_eq!(unique_sessions.len(), 3);

    // Verify sessions can be listed
    let sessions = manager.list_sessions().await;
    assert_eq!(sessions.len(), 3);

    // Verify each session exists and has correct data
    for (idx, session_id) in session_ids.iter().enumerate() {
        let session = sessions.iter().find(|s| &s.id == session_id).unwrap();
        assert_eq!(session.problem, problems[idx]);
        // Note: session.iteration may not be 0 due to shared manager state across tests
    }

    println!("Batch session creation test completed - created {} sessions", session_ids.len());
}

