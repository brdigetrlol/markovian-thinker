# Phase 6 User Guide: Storm Mitigation & Enhanced Configuration

## Overview

Phase 6 adds production-ready storm mitigation to Markovian Thinker, protecting your reasoning sessions from resource exhaustion, cascading failures, and runaway processing.

## New MCP Parameters

When calling `markovian_init_session`, you can now configure Phase 6 features:

```json
{
  "problem": "Your reasoning problem",
  "chunk_size": 512,
  "carryover_size": 128,
  "max_iterations": 10,

  // NEW Phase 6 Parameters
  "enable_event_driven": false,
  "enable_causal_trace": false,
  "lattice_type": "e8",
  "enable_storm_mitigation": true,
  "storm_mitigation_level": "default"
}
```

### Parameter Descriptions

#### `enable_event_driven` (boolean, default: false)
Enable event-driven chunk processing using the Icarus TIC architecture. Currently in development.

#### `enable_causal_trace` (boolean, default: false)
Enable causal trace tracking for reasoning structure. Currently in development.

#### `lattice_type` (string, default: "e8")
Select the crystallographic lattice for concept quantization:
- `"e8"` - E8 lattice (8-dimensional, optimal sphere packing)
- `"leech"` - Leech lattice (24-dimensional, highest density)
- `"hcp-8"`, `"hcp-16"`, `"hcp-24"` - Hexagonal close-packed variants
- `"cubic-8"`, `"cubic-16"`, `"cubic-32"` - Hypercubic lattices

#### `enable_storm_mitigation` (boolean, default: true)
Enable storm mitigation protection (rate limiting, circuit breakers, event fusion). **Recommended: keep enabled in production.**

#### `storm_mitigation_level` (string, default: "default")
Control storm mitigation aggressiveness:
- `"aggressive"` - Tightest protection (5 tok/sec, 3 failure threshold, 70% fusion similarity)
- `"default"` - Balanced protection (10 tok/sec, 5 failure threshold, 80% fusion similarity)
- `"lenient"` - Looser protection (20 tok/sec, 10 failure threshold, 90% fusion similarity)
- `"disabled"` - No protection (not recommended for production)

## Storm Mitigation Features

### Rate Limiting
Protects against burst requests using a token bucket algorithm:
- **Burst capacity**: Initial tokens available
- **Refill rate**: Tokens added per second
- **Behavior**: Requests blocked when bucket empty

**Example**: With default settings (10 tok/sec), you can process 10 chunks immediately, then 10 more per second thereafter.

### Circuit Breaker
Prevents cascading failures with three states:
- **Closed**: Normal operation, requests allowed
- **Open**: Too many failures, requests blocked
- **Half-Open**: Testing recovery, limited requests

**Example**: After 5 consecutive failures (default), circuit breaker opens for 30 seconds.

### Event Fusion
Deduplicates similar pending events using Jaccard similarity:
- Compares event prompts/hypotheses
- Fuses events above similarity threshold
- Boosts priority of fused events

**Example**: "Solve math problem" + "Solve math equation" → 1 fused event (high similarity)

## New MCP Endpoint: `markovian_get_metrics`

Get real-time storm mitigation statistics for a session:

```json
{
  "name": "markovian_get_metrics",
  "arguments": {
    "session_id": "your-session-uuid"
  }
}
```

**Response**:
```json
{
  "session_id": "...",
  "storm_mitigation": {
    "circuit_state": "Closed",
    "metrics": {
      "total_checks": 10,
      "allowed_events": 8,
      "rate_limit_rejections": 1,
      "circuit_breaker_rejections": 1,
      "successful_events": 8,
      "failed_events": 0,
      "total_fusions": 2,
      "events_fused": 2,
      "emergency_stops": 0,
      "success_rate": 1.0,
      "rejection_rate": 0.2,
      "fusion_effectiveness": 1.0
    }
  }
}
```

## Usage Examples

### Example 1: Conservative Session (High Protection)

```json
{
  "name": "markovian_init_session",
  "arguments": {
    "problem": "Prove the Riemann Hypothesis",
    "chunk_size": 512,
    "max_iterations": 50,
    "enable_storm_mitigation": true,
    "storm_mitigation_level": "aggressive"
  }
}
```

This creates a session with:
- **5 tokens/second** rate limit (tight control)
- **3 failure threshold** (opens quickly)
- **70% fusion similarity** (aggressive deduplication)

**Use when**: Processing untrusted inputs, debugging, or limiting resource usage.

### Example 2: Production Session (Balanced Protection)

```json
{
  "name": "markovian_init_session",
  "arguments": {
    "problem": "Analyze this complex dataset",
    "chunk_size": 1024,
    "max_iterations": 20,
    "enable_storm_mitigation": true,
    "storm_mitigation_level": "default"
  }
}
```

This creates a session with:
- **10 tokens/second** rate limit (balanced throughput)
- **5 failure threshold** (moderate tolerance)
- **80% fusion similarity** (moderate deduplication)

**Use when**: Normal production workloads with reliable inputs.

### Example 3: High-Throughput Session (Light Protection)

```json
{
  "name": "markovian_init_session",
  "arguments": {
    "problem": "Quick estimation task",
    "chunk_size": 256,
    "max_iterations": 5,
    "enable_storm_mitigation": true,
    "storm_mitigation_level": "lenient"
  }
}
```

This creates a session with:
- **20 tokens/second** rate limit (high throughput)
- **10 failure threshold** (high tolerance)
- **90% fusion similarity** (minimal deduplication)

**Use when**: Trusted environments, batch processing, or latency-sensitive tasks.

### Example 4: Monitoring Session Health

```json
// 1. Start session
{
  "name": "markovian_init_session",
  "arguments": {
    "problem": "Long-running analysis",
    "max_iterations": 100,
    "storm_mitigation_level": "default"
  }
}

// 2. Submit chunks...
// (multiple markovian_submit_chunk calls)

// 3. Check metrics
{
  "name": "markovian_get_metrics",
  "arguments": {
    "session_id": "session-uuid-from-step-1"
  }
}
```

Monitor:
- **Circuit state**: Closed = healthy, Open = failing
- **Success rate**: Should be close to 1.0
- **Rejection rate**: High values indicate throttling
- **Fusion effectiveness**: How well deduplication is working

## Troubleshooting

### Circuit Breaker Keeps Opening

**Symptom**: Chunks rejected with "Circuit breaker open" message.

**Solutions**:
1. Check `success_rate` in metrics - should be high
2. Review chunk processing logic for errors
3. Switch to `"lenient"` storm mitigation level
4. Increase `failure_threshold` if failures are transient

### Rate Limiting Too Aggressive

**Symptom**: Chunks rejected with "Rate limit exceeded" message.

**Solutions**:
1. Switch to `"lenient"` storm mitigation level
2. Add delays between chunk submissions
3. Reduce burst size (submit fewer chunks at once)
4. Disable rate limiting: `"storm_mitigation_level": "disabled"` (not recommended)

### Too Many Event Fusions

**Symptom**: `fusion_effectiveness` near 1.0, but chunks are semantically different.

**Solutions**:
1. Lower fusion similarity threshold (use conservative config)
2. Make chunk prompts more distinct
3. Check if event fusion is beneficial for your use case

### Session Running Too Slow

**Symptom**: Low throughput despite no errors.

**Solutions**:
1. Use `"lenient"` storm mitigation level
2. Increase chunk size for fewer iterations
3. Monitor `rate_limit_rejections` - should be low
4. Check network latency (not storm mitigation issue)

## Best Practices

### 1. Always Enable Storm Mitigation in Production
Default: `enable_storm_mitigation: true`

**Why**: Protects against:
- Infinite loops in reasoning logic
- Cascading failures from external services
- Resource exhaustion from malicious inputs
- Unintentional DoS from bugs

### 2. Start with "default" Level, Tune as Needed
Aggressive → Default → Lenient → Disabled

**Why**: Default provides good balance. Monitor metrics and adjust.

### 3. Monitor Circuit Breaker State
Use `markovian_get_metrics` to check `circuit_state`.

**Why**: Open circuit = system under stress. Investigate root cause.

### 4. Track Success Rate Over Time
Aim for `success_rate` > 0.95.

**Why**: Low success rate indicates quality issues in reasoning or inputs.

### 5. Use Appropriate Lattice Types
- E8 (default): Best for most use cases
- Leech: When you need highest-density quantization (24D)
- HCP/Cubic: Experimental, for research

## Performance Impact

Storm mitigation has **minimal overhead**:
- **Rate limiting**: < 1µs per check
- **Circuit breaker**: < 1µs per check
- **Event fusion**: O(n²) within window (configurable max 100-200 events)

**Total overhead**: < 5% of chunk processing time in typical workloads.

## Integration Tests

Run end-to-end tests to verify storm mitigation:

```bash
# Run all tests (164 total: 157 unit + 7 integration)
cargo test --release

# Run integration tests only
cargo test --release --test integration_test

# Run storm mitigation example
cargo run --release --example e2e_storm_mitigation
```

Expected output shows:
- ✅ Normal chunk processing (5/5 allowed)
- ✅ Circuit breaker triggering (opens after 3 failures)
- ✅ Rate limiting burst protection (10/20 burst allowed)
- ✅ Event fusion deduplication (25% reduction)

## Further Reading

- **INTEGRATION_STATUS.md**: Overall Phase 1-6 progress
- **ICARUS_TIC_INTEGRATION.md**: Event-driven architecture design
- **GPTOSS_INTEGRATION.md**: GPT-OSS optimizations
- **examples/full_integration.rs**: Complete Phase 1-5 demo
- **examples/e2e_storm_mitigation.rs**: Phase 6 storm mitigation demo

## Support

For issues or questions:
1. Check this guide first
2. Review integration tests for usage patterns
3. Monitor `markovian_get_metrics` for session health
4. File an issue with reproduction steps

---

**Phase 6 Status**: 67% complete, storm mitigation production-ready! 🚀
