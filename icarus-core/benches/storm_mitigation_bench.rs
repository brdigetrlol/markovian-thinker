// Performance Benchmarks for Storm Mitigation
// Measures overhead and throughput of Phase 6 features

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use markovian_thinker::*;

fn benchmark_rate_limiter(c: &mut Criterion) {
    let mut group = c.benchmark_group("rate_limiter");

    for &refill_rate in &[5.0, 10.0, 20.0, 50.0] {
        group.bench_with_input(
            BenchmarkId::new("try_acquire", format!("{}tok/s", refill_rate)),
            &refill_rate,
            |b, &rate| {
                let limiter = RateLimiter::new(RateLimitConfig {
                    max_tokens: rate * 2.0,
                    refill_rate: rate,
                    initial_tokens: Some(rate * 2.0),
                });

                b.iter(|| {
                    black_box(limiter.try_acquire_one())
                });
            },
        );
    }

    group.finish();
}

fn benchmark_circuit_breaker(c: &mut Criterion) {
    let mut group = c.benchmark_group("circuit_breaker");

    for &threshold in &[3, 5, 10, 20] {
        group.bench_with_input(
            BenchmarkId::new("allow_request", format!("threshold_{}", threshold)),
            &threshold,
            |b, &thresh| {
                let breaker = CircuitBreaker::new(CircuitBreakerConfig {
                    failure_threshold: thresh,
                    success_threshold: 2,
                    timeout: std::time::Duration::from_secs(30),
                });

                b.iter(|| {
                    black_box(breaker.allow_request())
                });
            },
        );
    }

    group.finish();
}

fn benchmark_storm_mitigation_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("storm_mitigation");

    for preset in &["aggressive", "default", "lenient"] {
        group.bench_with_input(
            BenchmarkId::from_parameter(preset),
            preset,
            |b, &preset_name| {
                let config = match preset_name {
                    "aggressive" => StormMitigationConfig::aggressive(),
                    "lenient" => StormMitigationConfig::lenient(),
                    _ => StormMitigationConfig::default(),
                };

                let mut mitigation = StormMitigation::new(config);

                b.iter(|| {
                    let decision = black_box(mitigation.allow_event());
                    if matches!(decision, MitigationDecision::Allowed) {
                        mitigation.record_success();
                    }
                });
            },
        );
    }

    group.finish();
}

fn benchmark_event_fusion(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_fusion");

    for &event_count in &[10, 50, 100, 200] {
        group.bench_with_input(
            BenchmarkId::new("fuse_events", format!("{}_events", event_count)),
            &event_count,
            |b, &count| {
                let fusion = EventFusion::new(EventFusionConfig::default());

                // Create test events
                let events: Vec<EventWithMetadata> = (0..count)
                    .map(|i| create_test_event(&format!("Event {}", i % 5), 1.0))
                    .collect();

                b.iter(|| {
                    black_box(fusion.fuse_events(events.clone()))
                });
            },
        );
    }

    group.finish();
}

fn benchmark_expert_selection(c: &mut Criterion) {
    let mut group = c.benchmark_group("expert_selection");

    let gating = ExpertGating::new(ExpertConfig::default());

    let test_cases = vec![
        ("math", "Solve ∫(2x + 3)dx from 0 to 5"),
        ("code", "fn main() { println!(\"Hello\"); }"),
        ("text", "The quick brown fox jumps over the lazy dog."),
    ];

    for (name, prompt) in test_cases {
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &prompt,
            |b, &p| {
                b.iter(|| {
                    black_box(gating.select_experts(p, None))
                });
            },
        );
    }

    group.finish();
}

fn benchmark_attention_compression(c: &mut Criterion) {
    let mut group = c.benchmark_group("attention_compression");

    let attention = SlidingWindowAttention::new(AttentionConfig::default());

    for &text_length in &[100, 500, 1000, 2000] {
        group.bench_with_input(
            BenchmarkId::new("select_important", format!("{}_chars", text_length)),
            &text_length,
            |b, &length| {
                let text: String = (0..length).map(|i| {
                    if i % 50 == 0 { '\n' } else { 'a' }
                }).collect();

                b.iter(|| {
                    black_box(attention.select_important(&text, length / 2))
                });
            },
        );
    }

    group.finish();
}

fn benchmark_concept_crystallization(c: &mut Criterion) {
    let mut group = c.benchmark_group("concept_crystallization");

    for lattice_type in &["e8", "leech", "hcp-8", "cubic-8"] {
        group.bench_with_input(
            BenchmarkId::from_parameter(lattice_type),
            lattice_type,
            |b, &lat_type| {
                let config = ConceptSpaceConfig {
                    lattice_type: match lat_type {
                        "e8" => LatticeType::E8,
                        "leech" => LatticeType::Leech,
                        "hcp-8" => LatticeType::HCP(8),
                        "cubic-8" => LatticeType::Hypercubic(8),
                        _ => LatticeType::E8,
                    },
                    enable_crystallization: true,
                    similarity_threshold: 0.9,
                };

                let space = ConceptSpace::new(config);
                let embedding = vec![1.0, 0.5, 0.2, 0.1, 0.0, 0.0, 0.0, 0.0];

                b.iter(|| {
                    black_box(space.crystallize(&embedding))
                });
            },
        );
    }

    group.finish();
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

criterion_group!(
    benches,
    benchmark_rate_limiter,
    benchmark_circuit_breaker,
    benchmark_storm_mitigation_check,
    benchmark_event_fusion,
    benchmark_expert_selection,
    benchmark_attention_compression,
    benchmark_concept_crystallization,
);

criterion_main!(benches);
