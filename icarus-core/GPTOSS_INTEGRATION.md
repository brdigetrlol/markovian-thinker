# GPT-OSS Integration into Markovian Thinker

## Overview

This document describes the integration of concepts from [GPT-OSS](https://www.projektjoe.com/blog/gptoss) into the Markovian Thinker MCP server to enhance reasoning performance through transformer-inspired optimizations.

## Key Concepts from GPT-OSS

### 1. Mixture of Experts (MoE)
- **Original**: 32 expert networks, top-4 selection via gating
- **Adaptation**: Multiple chunking/carryover strategies, domain-specific experts

### 2. Sliding Window Attention
- **Original**: 128-token lookback for even layers
- **Adaptation**: Attention-weighted carryover selection with configurable window

### 3. Grouped Attention
- **Original**: 8 queries per key-value pair
- **Adaptation**: Group similar reasoning patterns for efficient carryover

### 4. Advanced Sampling
- **Original**: Temperature, top-k, top-p/nucleus sampling
- **Adaptation**: Configurable sampling parameters for chunk generation

### 5. Attention Sink
- **Original**: Special token to absorb excess attention
- **Adaptation**: Context relevance scoring to prevent hallucination

## Architecture Enhancements

### New Module: `experts.rs`
```rust
pub enum ExpertType {
    MathReasoning,
    CodeGeneration,
    TextualReasoning,
    VisualReasoning,
    Mixed,
}

pub trait ReasoningExpert {
    fn score_relevance(&self, problem: &str) -> f32;
    fn extract_carryover(&self, text: &str, size: usize) -> String;
    fn detect_termination(&self, text: &str) -> bool;
}

pub struct ExpertGating {
    experts: Vec<Box<dyn ReasoningExpert>>,
    top_k: usize,
}
```

### New Module: `attention.rs`
```rust
pub struct SlidingWindowAttention {
    window_size: usize,
    decay_factor: f32,
}

impl SlidingWindowAttention {
    pub fn score_tokens(&self, tokens: &[Token]) -> Vec<f32>;
    pub fn select_important(&self, text: &str, target_size: usize) -> String;
}

pub struct AttentionSink {
    sink_threshold: f32,
}

impl AttentionSink {
    pub fn filter_hallucinations(&self, text: &str, scores: &[f32]) -> String;
}
```

### New Module: `sampling.rs`
```rust
#[derive(Debug, Clone)]
pub struct SamplingConfig {
    pub temperature: f32,
    pub top_k: Option<usize>,
    pub top_p: Option<f32>,
    pub repetition_penalty: f32,
}

impl SamplingConfig {
    pub fn conservative() -> Self;  // Low temperature, high quality
    pub fn creative() -> Self;      // Higher temperature, exploration
    pub fn balanced() -> Self;      // Default
}
```

### Enhanced `StateConfig`
```rust
pub struct StateConfig {
    // Existing fields
    pub chunk_size: usize,
    pub carryover_size: usize,
    pub max_iterations: usize,
    pub token_budget: usize,

    // New GPT-OSS inspired fields
    pub expert_config: ExpertConfig,
    pub attention_config: AttentionConfig,
    pub sampling_config: SamplingConfig,
}

pub struct ExpertConfig {
    pub enabled: bool,
    pub top_k_experts: usize,
    pub gating_threshold: f32,
}

pub struct AttentionConfig {
    pub sliding_window_size: Option<usize>,
    pub attention_sink_enabled: bool,
    pub decay_factor: f32,
}
```

## Implementation Plan

### Phase 1: Mixture of Experts Chunking
**File**: `src/experts.rs` (new)

1. Define expert trait and types
2. Implement domain-specific experts:
   - `MathExpert`: LaTeX extraction, equation parsing
   - `CodeExpert`: Syntax-aware chunking, code block detection
   - `TextExpert`: Semantic paragraph boundaries
3. Implement gating mechanism using domain detection
4. Integrate into `ChunkManager`

### Phase 2: Sliding Window Attention for Carryover
**File**: `src/attention.rs` (new)

1. Token importance scoring based on:
   - Recency (exponential decay)
   - Semantic markers (question marks, solutions)
   - Pattern matching (equations, code blocks)
2. Sliding window implementation (128-256 token default)
3. Grouped attention for similar patterns
4. Integration into `MarkovianState::extract_carryover`

### Phase 3: Advanced Sampling Strategies
**File**: `src/sampling_strategies.rs` (new)

1. Temperature scaling
2. Top-k filtering
3. Top-p (nucleus) sampling
4. Repetition penalty
5. Domain-adaptive sampling (math=conservative, creative=exploratory)
6. Add to MCP tool parameters

### Phase 4: Attention Sink
**File**: `src/attention.rs` (extend)

1. Relevance scoring for each segment
2. Hallucination detection patterns
3. Context coherence checking
4. Filter low-relevance content before carryover

### Phase 5: Optimization & Testing
**Files**: All modified files

1. RMSNorm-inspired lightweight normalization
2. BFloat16-aware token budget calculations
3. Comprehensive unit tests
4. Integration tests with various problem domains
5. Performance benchmarks

## API Changes

### Enhanced `markovian_solve` Tool

```typescript
{
  "name": "markovian_solve",
  "parameters": {
    "problem": "string (required)",
    "chunk_size": "number (default: 8192)",
    "carryover_size": "number (default: 4096)",
    "max_iterations": "number (default: 5)",

    // NEW: Expert configuration
    "enable_experts": "boolean (default: true)",
    "expert_mode": "auto|math|code|text (default: auto)",

    // NEW: Attention configuration
    "sliding_window": "number|null (default: 256)",
    "attention_sink": "boolean (default: true)",

    // NEW: Sampling configuration
    "temperature": "number (default: 0.7)",
    "top_k": "number|null (default: null)",
    "top_p": "number|null (default: 0.9)",
  }
}
```

## Performance Expectations

### Carryover Quality
- **Before**: Simple character-based truncation
- **After**: Attention-weighted selection preserving key information
- **Expected improvement**: 30-50% better context retention

### Domain Specialization
- **Before**: Generic chunking for all problems
- **After**: Expert-specific strategies per domain
- **Expected improvement**: 20-40% better termination detection

### Hallucination Reduction
- **Before**: No filtering
- **After**: Attention sink removes low-relevance content
- **Expected improvement**: 15-25% reduction in irrelevant carryover

## Backwards Compatibility

All new features are **opt-in** via configuration:
- Default behavior unchanged
- Existing traces remain compatible
- New features activated via parameters

## Testing Strategy

1. **Unit Tests**: Each expert, attention module independently
2. **Integration Tests**: End-to-end with various problem types
3. **Benchmark Suite**:
   - Math problems (GSM8K subset)
   - Code generation (HumanEval subset)
   - Reasoning (ARC subset)
4. **Comparison Metrics**:
   - Solution accuracy
   - Tokens used
   - Iterations required
   - Carryover relevance score

## Migration Path

### Existing Users
No changes required - defaults maintain current behavior

### Opt-in New Features
```bash
# Via MCP tool call
markovian_solve(
  problem="...",
  enable_experts=true,
  sliding_window=256,
  temperature=0.8
)
```

### Environment Variables
```bash
export MARKOVIAN_ENABLE_EXPERTS=true
export MARKOVIAN_SLIDING_WINDOW=256
export MARKOVIAN_ATTENTION_SINK=true
```

## References

1. **GPT-OSS Blog**: https://www.projektjoe.com/blog/gptoss
2. **Markovian Thinker Paper**: arXiv:2510.06557v1
3. **Mixture of Experts**: Shazeer et al. (2017)
4. **Sliding Window Attention**: Beltagy et al. (2020)
5. **Nucleus Sampling**: Holtzman et al. (2019)

## Implementation Timeline

- **Week 1**: Phase 1 - Mixture of Experts
- **Week 2**: Phase 2 - Sliding Window Attention
- **Week 3**: Phase 3 - Advanced Sampling
- **Week 4**: Phase 4 - Attention Sink
- **Week 5**: Phase 5 - Testing & Optimization

## Success Metrics

- [ ] All existing tests pass
- [ ] 30%+ improvement in carryover relevance
- [ ] 20%+ better termination detection
- [ ] No performance degradation (< 5% overhead)
- [ ] Full documentation and examples
