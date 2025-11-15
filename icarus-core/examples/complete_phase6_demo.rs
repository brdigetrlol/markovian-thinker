// Complete Phase 6 Integration Demo
// Demonstrates all Phase 6 features working together:
// 1. Storm mitigation (rate limiting + circuit breaker + event fusion)
// 2. Causal trace tracking
// 3. Concept space querying
// 4. Real-time metrics monitoring

use markovian_thinker::{
    concept_space::{Concept, ConceptSpace, ConceptSpaceConfig},
    events::{ReasoningEvent, ReasoningLevel},
    lattice::LatticeType,
    session_manager::SessionManager,
    state::StateConfig,
    storm_mitigation::{MitigationDecision, StormMitigationConfig},
};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  Markovian Thinker - Complete Phase 6 Integration Demo");
    println!("═══════════════════════════════════════════════════════════\n");

    // Demo 1: Storm Mitigation with Session Manager
    demo_storm_mitigation().await;

    // Demo 2: Causal Trace Tracking
    demo_causal_trace().await;

    // Demo 3: Concept Space Querying
    demo_concept_space().await;

    // Demo 4: Full Integration (All Features Together)
    demo_full_integration().await;

    println!("\n═══════════════════════════════════════════════════════════");
    println!("  All Phase 6 Features Demonstrated Successfully! ✓");
    println!("═══════════════════════════════════════════════════════════\n");
}

async fn demo_storm_mitigation() {
    println!("───────────────────────────────────────────────────────────");
    println!("Demo 1: Storm Mitigation (Rate Limiting + Circuit Breaker)");
    println!("───────────────────────────────────────────────────────────\n");

    let manager = SessionManager::new();
    let mut config = StateConfig::default();
    config.storm_mitigation_config = StormMitigationConfig::aggressive();

    let session_id = manager
        .create_session("Test storm mitigation".to_string(), config)
        .await
        .unwrap();

    println!("✓ Created session with aggressive storm mitigation");
    println!("  - Rate limit: 5 tokens/sec");
    println!("  - Circuit breaker: 3 failure threshold\n");

    // Simulate normal processing
    println!("Testing normal chunk processing:");
    for i in 1..=5 {
        let decision = manager.check_storm_mitigation(session_id).await.unwrap();
        match decision {
            MitigationDecision::Allowed => {
                manager.record_storm_success(session_id).await.ok();
                println!("  ✓ Chunk {} allowed (success)", i);
            }
            _ => println!("  ✗ Chunk {} rejected", i),
        }
    }

    // Get metrics
    let stats = manager.get_storm_stats(session_id).await.unwrap();
    println!(
        "\n✓ Metrics: {}/{} allowed ({:.1}% success rate)",
        stats.metrics.allowed_events,
        stats.metrics.total_checks,
        stats.metrics.success_rate() * 100.0
    );

    // Simulate failures to trigger circuit breaker
    println!("\nSimulating failures to trigger circuit breaker:");
    for i in 1..=3 {
        let decision = manager.check_storm_mitigation(session_id).await.unwrap();
        if matches!(decision, MitigationDecision::Allowed) {
            manager.record_storm_failure(session_id).await.ok();
            println!("  ✗ Chunk failed ({})", i);
        }
    }

    // Check circuit breaker state
    let decision = manager.check_storm_mitigation(session_id).await.unwrap();
    match decision {
        MitigationDecision::Rejected { reason } => {
            println!("  ⚠ Circuit breaker OPEN: {}", reason);
        }
        _ => println!("  Circuit breaker still closed"),
    }

    let stats = manager.get_storm_stats(session_id).await.unwrap();
    println!(
        "\n✓ Final stats: {} failures, circuit state: {:?}",
        stats.metrics.failed_events, stats.circuit_state
    );

    manager.remove_session(session_id).await.ok();
    println!("✓ Session cleaned up\n");
}

async fn demo_causal_trace() {
    println!("───────────────────────────────────────────────────────────");
    println!("Demo 2: Causal Trace Tracking");
    println!("───────────────────────────────────────────────────────────\n");

    let manager = SessionManager::new();
    let mut config = StateConfig::default();
    config.enable_causal_trace = true;

    let session_id = manager
        .create_session("Test causal trace".to_string(), config)
        .await
        .unwrap();

    println!("✓ Created session with causal trace enabled");

    // Get causal trace
    let trace = manager.get_causal_trace(session_id).await.unwrap();
    println!("✓ Retrieved causal trace");
    println!("  - Session ID: {}", trace.metadata().session_id);
    println!("  - Total events: {}", trace.metadata().total_events);
    println!("  - Created at: {}", trace.metadata().created_at);

    // Demonstrate causal trace operations
    let mut trace = trace;

    // Add some reasoning events to demonstrate causality
    let chunk_id1 = Uuid::new_v4();
    let event1 = ReasoningEvent::ChunkComplete {
        session_id,
        chunk_id: chunk_id1,
        output: "First reasoning step".to_string(),
        tokens: 512,
        spawned_events: vec![],
        timestamp: 1000,
    };
    let event1_id = trace.add_event(event1, ReasoningLevel::Micro, vec![]);
    println!("\n✓ Added root event (ChunkComplete #1)");

    let chunk_id2 = Uuid::new_v4();
    let event2 = ReasoningEvent::ChunkComplete {
        session_id,
        chunk_id: chunk_id2,
        output: "Second reasoning step".to_string(),
        tokens: 512,
        spawned_events: vec![],
        timestamp: 2000,
    };
    let event2_id = trace.add_event(event2, ReasoningLevel::Micro, vec![event1_id]);
    println!("✓ Added event 2 with causal dependency on event 1");

    let chunk_id3 = Uuid::new_v4();
    let event3 = ReasoningEvent::ChunkComplete {
        session_id,
        chunk_id: chunk_id3,
        output: "Third reasoning step".to_string(),
        tokens: 512,
        spawned_events: vec![],
        timestamp: 3000,
    };
    let _event3_id = trace.add_event(event3, ReasoningLevel::Micro, vec![event2_id]);
    println!("✓ Added event 3 with causal dependency on event 2");

    // Check causal relationships
    println!("\nCausal relationships:");
    println!("  - Event 1 precedes Event 2: {}", trace.precedes(event1_id, event2_id));
    println!("  - Event 2 precedes Event 1: {}", trace.precedes(event2_id, event1_id));

    // Get statistics
    let stats = trace.statistics();
    println!("\n✓ Trace statistics:");
    println!("  - Total events: {}", stats.total_events);
    println!("  - Micro events: {}", stats.micro_events);
    println!("  - Meso events: {}", stats.meso_events);
    println!("  - Average depth: {:.1}", stats.avg_depth);
    println!("  - Max depth: {}", stats.max_depth);

    manager.remove_session(session_id).await.ok();
    println!("\n✓ Session cleaned up\n");
}

async fn demo_concept_space() {
    println!("───────────────────────────────────────────────────────────");
    println!("Demo 3: Concept Space Querying");
    println!("───────────────────────────────────────────────────────────\n");

    let manager = SessionManager::new();
    let config = StateConfig::default(); // Concept space always created

    let session_id = manager
        .create_session("Test concept space".to_string(), config)
        .await
        .unwrap();

    println!("✓ Created session with E8 lattice concept space");

    // Get concept space statistics (empty)
    let stats = manager.get_concept_stats(session_id).await.unwrap();
    println!("✓ Initial statistics:");
    println!("  - Lattice type: {:?}", stats.lattice_type);
    println!("  - Dimension: {}", stats.dimension);
    println!("  - Total concepts: {}", stats.total_concepts);

    // Create a concept space directly to demonstrate adding concepts
    let config = ConceptSpaceConfig {
        lattice_type: LatticeType::E8,
        max_concepts: 1000,
        similarity_threshold: 0.999,
    };
    let mut space = ConceptSpace::new(config);

    // Add some concepts
    let embeddings = vec![
        vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], // Pure dimension 1
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], // Pure dimension 2
        vec![0.7, 0.7, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], // Mix of 1 and 2
        vec![0.5, 0.5, 0.5, 0.5, 0.0, 0.0, 0.0, 0.0], // Distributed
    ];

    println!("\nAdding concepts:");
    for (i, embedding) in embeddings.iter().enumerate() {
        let point = space.crystallize(embedding);
        let concept = Concept::new(
            format!("concept_{}", i + 1),
            format!("Test Concept {}", i + 1),
            point,
        );
        space.add_concept(concept).ok();
        println!("  ✓ Added concept {} (dim: {}D)", i + 1, embedding.len());
    }

    // Query for similar concepts
    let query = vec![0.8, 0.6, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    println!("\nQuerying for concepts similar to [0.8, 0.6, 0.0, ...]:");
    let similar = space.find_similar(&query, 3);

    for (i, concept) in similar.iter().enumerate() {
        println!("  {}. {} (similarity: {:.3})",
            i + 1,
            concept.label,
            concept.similarity(&Concept::new("query".to_string(), "Query".to_string(), space.crystallize(&query)))
        );
    }

    // Get final statistics
    let final_stats = space.statistics();
    println!("\n✓ Final statistics:");
    println!("  - Total concepts: {}", final_stats.total_concepts);
    println!("  - Average norm: {:.3}", final_stats.avg_norm);
    println!("  - Min norm: {:.3}", final_stats.min_norm);
    println!("  - Max norm: {:.3}", final_stats.max_norm);

    manager.remove_session(session_id).await.ok();
    println!("\n✓ Session cleaned up\n");
}

async fn demo_full_integration() {
    println!("───────────────────────────────────────────────────────────");
    println!("Demo 4: Full Integration (All Features Together)");
    println!("───────────────────────────────────────────────────────────\n");

    let manager = SessionManager::new();
    let mut config = StateConfig::default();

    // Enable all Phase 6 features
    config.enable_causal_trace = true;
    config.storm_mitigation_config = StormMitigationConfig::default();

    let session_id = manager
        .create_session("Full Phase 6 integration test".to_string(), config)
        .await
        .unwrap();

    println!("✓ Created session with ALL Phase 6 features enabled:");
    println!("  - Storm mitigation: ON (default level)");
    println!("  - Causal trace: ON");
    println!("  - Concept space: ON (E8 lattice)\n");

    // Simulate a reasoning session
    println!("Simulating reasoning session:");

    for i in 1..=5 {
        // Check storm mitigation
        let decision = manager.check_storm_mitigation(session_id).await.unwrap();

        match decision {
            MitigationDecision::Allowed => {
                // Process chunk
                println!("  ✓ Chunk {} allowed by storm mitigation", i);

                // Record success
                manager.record_storm_success(session_id).await.ok();

                // Simulate concept querying (would happen during chunk processing)
                let embedding = vec![0.1 * i as f32; 8];
                let concepts = manager.query_concepts(session_id, embedding.clone(), 3).await.unwrap();
                println!("    - Queried concept space ({} similar concepts found)", concepts.len());
            }
            MitigationDecision::RateLimited { retry_after } => {
                println!("  ⚠ Chunk {} rate limited (retry after {:?})", i, retry_after);
            }
            MitigationDecision::Rejected { reason } => {
                println!("  ✗ Chunk {} rejected: {}", i, reason);
            }
        }
    }

    // Get comprehensive metrics
    println!("\n─────────────────────────────────────────────────────");
    println!("Final Session Metrics:");
    println!("─────────────────────────────────────────────────────\n");

    // Storm mitigation metrics
    let storm_stats = manager.get_storm_stats(session_id).await.unwrap();
    println!("Storm Mitigation:");
    println!("  - Circuit state: {:?}", storm_stats.circuit_state);
    println!("  - Success rate: {:.1}%", storm_stats.metrics.success_rate() * 100.0);
    println!("  - Allowed events: {}", storm_stats.metrics.allowed_events);
    println!("  - Total checks: {}", storm_stats.metrics.total_checks);

    // Causal trace metrics
    let trace = manager.get_causal_trace(session_id).await.unwrap();
    println!("\nCausal Trace:");
    println!("  - Total events: {}", trace.metadata().total_events);
    println!("  - Session ID: {}", trace.metadata().session_id);

    // Concept space metrics
    let concept_stats = manager.get_concept_stats(session_id).await.unwrap();
    println!("\nConcept Space:");
    println!("  - Lattice type: {:?}", concept_stats.lattice_type);
    println!("  - Dimension: {}D", concept_stats.dimension);
    println!("  - Total concepts: {}", concept_stats.total_concepts);

    println!("\n✓ Full integration test completed successfully!");
    println!("✓ All Phase 6 features working together seamlessly!");

    manager.remove_session(session_id).await.ok();
    println!("\n✓ Session cleaned up\n");
}
