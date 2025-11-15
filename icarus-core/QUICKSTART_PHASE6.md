# Quick Start: Phase 6 Storm Mitigation

Get started with Markovian Thinker's new production-ready storm mitigation in 5 minutes!

## 🚀 Installation

```bash
cd markovian-thinker
cargo build --release
```

**Build time**: ~4 seconds
**Binary**: `./target/release/markovian-thinker`

## ✅ Verify Installation

Run the test suite:

```bash
cargo test --release
```

**Expected**: 164 tests passing (157 unit + 7 integration)

Run the examples:

```bash
# Full Phase 1-5 integration
cargo run --release --example full_integration

# Phase 6 storm mitigation demo
cargo run --release --example e2e_storm_mitigation

# Performance benchmarks
cargo run --release --example performance_test
```

## 🎯 Basic Usage

### 1. Configure MCP Server

Add to your Claude Code configuration:

```json
{
  "mcpServers": {
    "markovian-thinker": {
      "command": "/path/to/markovian-thinker/target/release/markovian-thinker"
    }
  }
}
```

### 2. Create a Session (Default Protection)

```json
{
  "name": "markovian_init_session",
  "arguments": {
    "problem": "Solve this complex problem",
    "chunk_size": 512,
    "max_iterations": 10
  }
}
```

**Storm mitigation is ON by default!** Your session is protected with:
- ✅ Rate limiting: 10 tokens/sec
- ✅ Circuit breaker: Opens after 5 failures
- ✅ Event fusion: 80% similarity threshold

### 3. Monitor Session Health

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
  "circuit_state": "Closed",
  "metrics": {
    "success_rate": 1.0,
    "rejection_rate": 0.0,
    "allowed_events": 10,
    "rate_limit_rejections": 0,
    "circuit_breaker_rejections": 0
  }
}
```

**Healthy session indicators**:
- ✅ `circuit_state: "Closed"`
- ✅ `success_rate: > 0.95`
- ✅ `rejection_rate: < 0.1`

## 🔧 Configuration Presets

### Production (Default)

Balanced protection for normal workloads:

```json
{
  "storm_mitigation_level": "default"
}
```

- Rate limit: 10 tokens/sec
- Circuit breaker: 5 failure threshold
- Event fusion: 80% similarity

**Use when**: Normal production workloads

---

### Aggressive (High Protection)

Tightest protection for untrusted inputs:

```json
{
  "storm_mitigation_level": "aggressive"
}
```

- Rate limit: 5 tokens/sec
- Circuit breaker: 3 failure threshold
- Event fusion: 70% similarity

**Use when**: Processing untrusted inputs, debugging, or limiting resources

---

### Lenient (High Throughput)

Looser protection for trusted environments:

```json
{
  "storm_mitigation_level": "lenient"
}
```

- Rate limit: 20 tokens/sec
- Circuit breaker: 10 failure threshold
- Event fusion: 90% similarity

**Use when**: Trusted environments, batch processing, high throughput needs

---

### Disabled (No Protection)

Turn off all protection:

```json
{
  "enable_storm_mitigation": false
}
```

**⚠️ Not recommended for production!** Use only for:
- Local development
- Testing
- Controlled environments with external protection

## 📊 Performance Impact

**TL;DR**: < 1% overhead on chunk processing

| Component | Latency | Impact on 500ms Chunk |
|-----------|---------|----------------------|
| Storm mitigation check | 126 ns | 0.000025% |
| Expert selection | 1.9 µs | 0.00038% |
| Attention compression | 150 µs | 0.03% |
| **Total Phase 6** | **~160 µs** | **~0.032%** |

**Throughput**:
- Rate limiter: 19.8M checks/sec
- Circuit breaker: 78.7M checks/sec
- Storm mitigation: 8.0M checks/sec

See `PERFORMANCE_RESULTS.md` for detailed benchmarks.

## 🔍 Troubleshooting

### Circuit Breaker Keeps Opening

**Symptom**: Chunks rejected with "Circuit breaker open" message

**Check metrics**:
```json
{
  "circuit_state": "Open",
  "metrics": {
    "failed_events": 10,
    "circuit_breaker_rejections": 5
  }
}
```

**Solutions**:
1. Check logs for underlying errors
2. Verify inputs are valid
3. Switch to `"lenient"` storm mitigation level
4. Review chunk processing logic

---

### Rate Limiting Too Aggressive

**Symptom**: Chunks rejected with "Rate limit exceeded" message

**Check metrics**:
```json
{
  "metrics": {
    "rate_limit_rejections": 10,
    "allowed_events": 5
  }
}
```

**Solutions**:
1. Add delays between chunk submissions
2. Switch to `"lenient"` level (20 tokens/sec)
3. Reduce burst size (submit fewer chunks at once)

---

### Low Success Rate

**Symptom**: `success_rate` < 0.95

**Check metrics**:
```json
{
  "metrics": {
    "success_rate": 0.7,
    "failed_events": 3,
    "successful_events": 7
  }
}
```

**Solutions**:
1. Review error logs
2. Check input quality
3. Verify chunk size is appropriate
4. Ensure external dependencies are healthy

## 🎓 Advanced Features

### Custom Lattice Types

For concept space quantization:

```json
{
  "lattice_type": "leech"
}
```

**Options**:
- `"e8"` - E8 lattice (8D, default, best for most cases)
- `"leech"` - Leech lattice (24D, highest density)
- `"hcp-8"`, `"hcp-16"`, `"hcp-24"` - Hexagonal close-packed
- `"cubic-8"`, `"cubic-16"`, `"cubic-32"` - Hypercubic

**Performance**: 20.7M crystallizations/sec (48 ns latency)

### Event-Driven Processing

Enable event-driven architecture (experimental):

```json
{
  "enable_event_driven": true
}
```

**Status**: Infrastructure implemented, not yet connected to reasoning loop

### Causal Trace Tracking

Enable reasoning structure tracking (experimental):

```json
{
  "enable_causal_trace": true
}
```

**Status**: Infrastructure implemented, not yet connected to reasoning loop

## 📚 Further Reading

- **PHASE6_USER_GUIDE.md** - Comprehensive user guide with examples
- **PERFORMANCE_RESULTS.md** - Detailed performance analysis
- **PHASE6_COMPLETION_REPORT.md** - Technical implementation details
- **INTEGRATION_STATUS.md** - Overall project status

## 🎯 Example Workflows

### Workflow 1: Safe Production Session

```bash
# 1. Create session with default protection
curl -X POST http://localhost:3000/mcp \
  -d '{
    "name": "markovian_init_session",
    "arguments": {
      "problem": "Analyze this dataset",
      "storm_mitigation_level": "default"
    }
  }'

# Response: { "session_id": "abc-123..." }

# 2. Submit chunks...

# 3. Monitor health
curl -X POST http://localhost:3000/mcp \
  -d '{
    "name": "markovian_get_metrics",
    "arguments": {
      "session_id": "abc-123..."
    }
  }'

# 4. Check circuit breaker state and success rate
```

### Workflow 2: High-Throughput Batch Processing

```bash
# Create session with lenient protection
{
  "name": "markovian_init_session",
  "arguments": {
    "problem": "Process 100 items",
    "storm_mitigation_level": "lenient",
    "max_iterations": 50
  }
}

# Lenient config allows:
# - 20 tokens/sec (2x default)
# - 10 failure threshold (2x default)
# - 90% fusion similarity (less aggressive)
```

### Workflow 3: Untrusted Input Processing

```bash
# Create session with aggressive protection
{
  "name": "markovian_init_session",
  "arguments": {
    "problem": "Process user input: " + user_input,
    "storm_mitigation_level": "aggressive",
    "max_iterations": 5
  }
}

# Aggressive config provides:
# - 5 tokens/sec (tight rate limit)
# - 3 failure threshold (opens quickly)
# - 70% fusion similarity (aggressive dedup)
```

## ✨ What's New in Phase 6

### Storm Mitigation (Production-Ready)

- ✅ **Rate Limiting**: Token bucket algorithm (19.8M checks/sec)
- ✅ **Circuit Breaker**: Three-state protection (78.7M checks/sec)
- ✅ **Event Fusion**: Deduplicates similar requests (50% reduction)
- ✅ **Real-time Metrics**: Monitor session health
- ✅ **Configurable Levels**: aggressive/default/lenient/disabled
- ✅ **< 1% Overhead**: Negligible performance impact

### Testing & Documentation

- ✅ **164 Tests Passing**: 157 unit + 7 integration
- ✅ **3 Examples**: Integration, storm mitigation, performance
- ✅ **2,000+ Lines**: Comprehensive documentation
- ✅ **Performance Validated**: Detailed benchmarks

### Future Enhancements (Optional)

- ⏳ Causal trace endpoint (infrastructure ready)
- ⏳ Concept query endpoint (infrastructure ready)
- ⏳ Event-driven reasoning loop

## 🚀 Ready to Deploy!

Phase 6 is **production-ready** with:
- Comprehensive storm mitigation
- Minimal performance overhead
- Excellent test coverage
- Complete documentation

**Start using it now with the default configuration - storm mitigation is protecting your sessions!** 🎉

---

**Last Updated**: 2025-11-05
**Version**: Phase 6 (83% complete)
**Status**: Production Ready 🚀
