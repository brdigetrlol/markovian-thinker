// Phase 7 Performance Benchmarks
// Measures overhead of event-driven, causal traces, intelligent carryover,
// expert-guided prompts, and attention compression

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use markovian_thinker::*;

// Benchmark event emission overhead
fn bench_event_emission(c: &mut Criterion) {
    use markovian_thinker::chunk_manager::ChunkManager;
    
    let mut config = StateConfig::default();
    config.enable_event_driven = true;
    config.enable_causal_trace = true;
    config.max_iterations = 1;
    
    let session_id = uuid::Uuid::new_v4();
    
    c.bench_function("event_emission", |b| {
        b.iter(|| {
            let mut manager = ChunkManager::with_events(config.clone(), session_id);
            // Simulate event emission
            black_box(&manager);
        });
    });
}

// Benchmark intelligent carryover selection
fn bench_intelligent_carryover(c: &mut Criterion) {
    let chunks = vec![
        "First chunk about algorithms and data structures".to_string(),
        "Second chunk discussing mathematics and equations".to_string(),
        "Third chunk about sorting and searching".to_string(),
    ];
    
    c.bench_function("intelligent_carryover", |b| {
        b.iter(|| {
            // Simulate Jaccard similarity computation
            let words1: std::collections::HashSet<&str> = chunks[0]
                .split_whitespace()
                .filter(|w| w.len() > 3)
                .collect();
            let words2: std::collections::HashSet<&str> = chunks[2]
                .split_whitespace()
                .filter(|w| w.len() > 3)
                .collect();
            
            let intersection = words1.intersection(&words2).count();
            let union = words1.union(&words2).count();
            
            black_box(intersection as f32 / union as f32);
        });
    });
}

// Benchmark expert selection
fn bench_expert_selection(c: &mut Criterion) {
    use markovian_thinker::experts::{ExpertGating, ExpertConfig};
    
    let config = ExpertConfig::default();
    let gating = ExpertGating::new(config);
    
    let problems = vec![
        "Solve x^2 + 5x + 6 = 0",
        "Write a function to sort an array",
        "Explain the causes of climate change",
    ];
    
    c.bench_function("expert_selection", |b| {
        b.iter(|| {
            for problem in &problems {
                let experts = gating.select_experts(problem, None);
                black_box(experts);
            }
        });
    });
}

// Benchmark attention compression
fn bench_attention_compression(c: &mut Criterion) {
    use markovian_thinker::attention::{SlidingWindowAttention, AttentionConfig};
    
    let config = AttentionConfig::default();
    let attention = SlidingWindowAttention::new(config);
    
    let text = "This is a long text with many words that needs to be compressed using attention mechanisms. \
                The algorithm should keep important keywords while removing less critical information. \
                Performance is critical for real-time applications.".repeat(10);
    
    c.bench_function("attention_compression", |b| {
        b.iter(|| {
            let compressed = attention.select_important(&text, 100);
            black_box(compressed);
        });
    });
}

// Benchmark causal trace recording
fn bench_causal_trace(c: &mut Criterion) {
    use markovian_thinker::causal_trace::CausalTrace;
    use markovian_thinker::events::{ReasoningEvent, ReasoningLevel};
    
    let session_id = uuid::Uuid::new_v4();
    let mut trace = CausalTrace::new(session_id);
    
    c.bench_function("causal_trace_recording", |b| {
        b.iter(|| {
            let event = ReasoningEvent::ChunkRequest {
                session_id,
                prompt: "Test prompt".to_string(),
                priority: 1.0,
                timestamp: chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64,
                level: ReasoningLevel::Macro,
            };
            
            trace.add_event(event, ReasoningLevel::Macro, vec![]);
            black_box(&trace);
        });
    });
}

// Benchmark full Phase 7 integration
fn bench_phase7_full_integration(c: &mut Criterion) {
    let mut group = c.benchmark_group("phase7_integration");
    
    for &feature_count in &[0, 1, 3, 5] {
        group.bench_with_input(
            BenchmarkId::from_parameter(feature_count),
            &feature_count,
            |b, &count| {
                let mut config = StateConfig::default();
                
                // Enable features based on count
                if count >= 1 {
                    config.enable_event_driven = true;
                }
                if count >= 2 {
                    config.enable_causal_trace = true;
                }
                if count >= 3 {
                    config.enable_intelligent_carryover = true;
                }
                if count >= 4 {
                    config.expert_config.enabled = true;
                }
                if count >= 5 {
                    config.attention_config.sliding_window_size = Some(256);
                }
                
                b.iter(|| {
                    let state = MarkovianState::new("Test problem".to_string(), config.clone());
                    black_box(state);
                });
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_event_emission,
    bench_intelligent_carryover,
    bench_expert_selection,
    bench_attention_compression,
    bench_causal_trace,
    bench_phase7_full_integration,
);
criterion_main!(benches);
