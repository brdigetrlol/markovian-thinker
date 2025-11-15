// Simple Performance Test for Storm Mitigation
// Measures overhead without external dependencies

use markovian_thinker::*;
use std::time::Instant;

fn main() {
    println!("=== Markovian Thinker Performance Test ===\n");

    // Test 1: Rate Limiter Overhead
    println!("1. Rate Limiter Performance");
    test_rate_limiter();
    println!();

    // Test 2: Circuit Breaker Overhead
    println!("2. Circuit Breaker Performance");
    test_circuit_breaker();
    println!();

    // Test 3: Storm Mitigation Combined
    println!("3. Storm Mitigation Combined Performance");
    test_storm_mitigation();
    println!();

    // Test 4: Event Fusion
    println!("4. Event Fusion Performance");
    test_event_fusion();
    println!();

    // Test 5: Expert Selection
    println!("5. Expert Selection Performance");
    test_expert_selection();
    println!();

    // Test 6: Attention Mechanism
    println!("6. Attention Mechanism Performance");
    test_attention();
    println!();

    // Test 7: Concept Crystallization
    println!("7. Concept Crystallization Performance");
    test_concept_crystallization();
    println!();

    println!("=== Performance Test Complete ===");
}

fn test_rate_limiter() {
    let limiter = RateLimiter::new(RateLimitConfig {
        max_tokens: 100.0,
        refill_rate: 10.0,
        initial_tokens: Some(100.0),
    });

    let iterations = 100_000;
    let start = Instant::now();

    for _ in 0..iterations {
        limiter.try_acquire_one();
    }

    let elapsed = start.elapsed();
    let per_op = elapsed.as_nanos() as f64 / iterations as f64;

    println!("   Iterations: {}", iterations);
    println!("   Total time: {:?}", elapsed);
    println!("   Per operation: {:.0} ns", per_op);
    println!("   Throughput: {:.0} ops/sec", 1_000_000_000.0 / per_op);
}

fn test_circuit_breaker() {
    let breaker = CircuitBreaker::new(CircuitBreakerConfig::default());

    let iterations = 100_000;
    let start = Instant::now();

    for _ in 0..iterations {
        breaker.allow_request();
    }

    let elapsed = start.elapsed();
    let per_op = elapsed.as_nanos() as f64 / iterations as f64;

    println!("   Iterations: {}", iterations);
    println!("   Total time: {:?}", elapsed);
    println!("   Per operation: {:.0} ns", per_op);
    println!("   Throughput: {:.0} ops/sec", 1_000_000_000.0 / per_op);
}

fn test_storm_mitigation() {
    let mut mitigation = StormMitigation::new(StormMitigationConfig::default());

    let iterations = 10_000;
    let start = Instant::now();

    for _ in 0..iterations {
        let decision = mitigation.allow_event();
        if matches!(decision, MitigationDecision::Allowed) {
            mitigation.record_success();
        }
    }

    let elapsed = start.elapsed();
    let per_op = elapsed.as_nanos() as f64 / iterations as f64;

    println!("   Iterations: {}", iterations);
    println!("   Total time: {:?}", elapsed);
    println!("   Per operation: {:.0} ns", per_op);
    println!("   Throughput: {:.0} ops/sec", 1_000_000_000.0 / per_op);

    let stats = mitigation.stats();
    println!("   Success rate: {:.1}%", stats.metrics.success_rate() * 100.0);
}

fn test_event_fusion() {
    let fusion = EventFusion::new(EventFusionConfig::default());

    // Create 100 test events (50% similar)
    let events: Vec<EventWithMetadata> = (0..100)
        .map(|i| create_test_event(&format!("Event group {}", i / 2), 1.0))
        .collect();

    let iterations = 1_000;
    let start = Instant::now();

    for _ in 0..iterations {
        let _ = fusion.fuse_events(events.clone());
    }

    let elapsed = start.elapsed();
    let per_op = elapsed.as_micros() as f64 / iterations as f64;

    println!("   Iterations: {} (100 events each)", iterations);
    println!("   Total time: {:?}", elapsed);
    println!("   Per fusion: {:.0} µs", per_op);
    println!("   Throughput: {:.0} fusions/sec", 1_000_000.0 / per_op);

    // Show fusion effectiveness
    let fused = fusion.fuse_events(events.clone());
    let stats = fusion.fusion_stats(&events, &fused);
    println!("   Reduction: {:.1}%", stats.reduction_rate * 100.0);
}

fn test_expert_selection() {
    let gating = ExpertGating::new(ExpertConfig::default());

    let test_prompts = vec![
        "Solve the integral ∫(2x + 3)dx",
        "Write a function to sort an array",
        "Explain quantum mechanics",
    ];

    let iterations = 10_000;
    let start = Instant::now();

    for _ in 0..iterations {
        for prompt in &test_prompts {
            gating.select_experts(prompt, None);
        }
    }

    let elapsed = start.elapsed();
    let per_op = elapsed.as_nanos() as f64 / (iterations * test_prompts.len()) as f64;

    println!("   Iterations: {} × {} prompts", iterations, test_prompts.len());
    println!("   Total time: {:?}", elapsed);
    println!("   Per selection: {:.0} ns", per_op);
    println!("   Throughput: {:.0} ops/sec", 1_000_000_000.0 / per_op);
}

fn test_attention() {
    let attention = SlidingWindowAttention::new(AttentionConfig::default());

    let text = "The quick brown fox jumps over the lazy dog. ".repeat(20);
    let target_size = text.len() / 2;

    let iterations = 1_000;
    let start = Instant::now();

    for _ in 0..iterations {
        attention.select_important(&text, target_size);
    }

    let elapsed = start.elapsed();
    let per_op = elapsed.as_micros() as f64 / iterations as f64;

    println!("   Iterations: {} (text: {} chars)", iterations, text.len());
    println!("   Total time: {:?}", elapsed);
    println!("   Per compression: {:.0} µs", per_op);
    println!("   Throughput: {:.0} compressions/sec", 1_000_000.0 / per_op);
}

fn test_concept_crystallization() {
    let space = ConceptSpace::new(ConceptSpaceConfig::default());

    let embedding = vec![1.0, 0.5, 0.2, 0.1, 0.0, 0.0, 0.0, 0.0];

    let iterations = 10_000;
    let start = Instant::now();

    for _ in 0..iterations {
        space.crystallize(&embedding);
    }

    let elapsed = start.elapsed();
    let per_op = elapsed.as_nanos() as f64 / iterations as f64;

    println!("   Iterations: {} (E8 lattice)", iterations);
    println!("   Total time: {:?}", elapsed);
    println!("   Per crystallization: {:.0} ns", per_op);
    println!("   Throughput: {:.0} ops/sec", 1_000_000_000.0 / per_op);
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
