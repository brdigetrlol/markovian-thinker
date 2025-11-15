---
description: "Analyzes and optimizes code performance using language-agnostic procedural analysis for any codebase"
---

# Performance Optimization Command

You are an expert performance optimization engineer using systematic, language-agnostic methodologies.

Your task is to comprehensively analyze and optimize any codebase through procedural analysis, pattern recognition, and abstracted optimization strategies.

## Framework: Universal Performance Optimization Process

### Phase 1: Discovery & Profiling
### Phase 2: Pattern Analysis
### Phase 3: Optimization Strategy
### Phase 4: Implementation
### Phase 5: Verification

---

## Phase 1: Discovery & Profiling

### 1.1: Detect Project Structure

**Procedurally identify environment**:

```python
# Pseudo-code for detection
detected_langs = []
detected_tools = {}

IF exists("Cargo.toml"):
  detected_langs.append("Rust")
  detected_tools["bench"] = "cargo bench"
  detected_tools["profile"] = "cargo-flamegraph"
  detected_tools["bloat"] = "cargo bloat"

IF exists("package.json"):
  detected_langs.append("Node/TypeScript")
  detected_tools["bench"] = "clinic"
  detected_tools["bundle"] = "webpack-bundle-analyzer"

IF exists("go.mod"):
  detected_langs.append("Go")
  detected_tools["bench"] = "go test -bench"
  detected_tools["profile"] = "pprof"

IF exists("setup.py" OR "pyproject.toml"):
  detected_langs.append("Python")
  detected_tools["profile"] = "cProfile"

# ... add more language detection
```

### 1.2: Execute Profiling (Language-Agnostic)

**Abstract profiling workflow**:

```
FOR each detected_lang in detected_langs:

  1. LOCATE hot paths:
     - Find main entry points
     - Identify frequently called functions
     - Map call graph depth

  2. MEASURE baseline:
     - Execution time (total, per-function)
     - Memory usage (allocations, peak, steady-state)
     - I/O operations (count, duration)
     - CPU utilization (user, system, idle)

  3. COLLECT metrics:
     - Throughput (operations/second)
     - Latency (p50, p95, p99)
     - Resource usage (CPU%, memory MB)
     - Concurrency (threads, contention)

  4. GENERATE flamegraph/profile:
     - Use language-specific tool
     - Identify functions consuming >5% time
     - Mark allocation hotspots
```

### 1.3: Baseline Establishment

```
CREATE baseline_report:
  timestamp: NOW
  metrics:
    throughput: X ops/sec
    latency_p95: Y ms
    memory_peak: Z MB
    cpu_avg: A%

  hotspots: [
    {function: "foo", time_percent: 25%, calls: 1M},
    {function: "bar", time_percent: 15%, calls: 500K},
    ...
  ]

  resource_distribution:
    computation: X%
    I/O_wait: Y%
    memory_allocation: Z%
```

---

## Phase 2: Pattern Analysis (Language-Agnostic)

### 2.1: Algorithmic Complexity Patterns

**Search for computational inefficiency**:

```
DEFINE complexity_patterns = {
  "quadratic_loop": {
    pattern: "loop(N) containing loop(N)",
    severity: HIGH,
    fix_strategy: "Use hash_table OR single_pass"
  },

  "repeated_search": {
    pattern: "linear_search repeated M times",
    severity: HIGH,
    fix_strategy: "Pre-build index OR use hash_table"
  },

  "unnecessary_sort": {
    pattern: "sort(N*logN) when only need top_k",
    severity: MEDIUM,
    fix_strategy: "Use heap for top_k OR partial_sort"
  },

  "repeated_computation": {
    pattern: "identical_function_call repeated",
    severity: MEDIUM,
    fix_strategy: "Memoize OR hoist_invariant"
  }
}

FOR each file in codebase:
  FOR each complexity_pattern:
    IF matches(file, pattern):
      RECORD as optimization_opportunity
```

### 2.2: Memory Patterns

**Search for memory inefficiency**:

```
DEFINE memory_patterns = {
  "excessive_allocation": {
    detect: "allocation inside hot_loop",
    impact: "GC pressure OR fragmentation",
    fix: "Pre-allocate OR object_pool"
  },

  "unnecessary_copy": {
    detect: "deep_copy when reference suffices",
    impact: "CPU + memory overhead",
    fix: "Use references OR copy-on-write"
  },

  "memory_leak": {
    detect: "allocation without corresponding free",
    impact: "Growing memory usage",
    fix: "Add cleanup OR use RAII/defer"
  },

  "suboptimal_structure": {
    detect: "array_of_structs when struct_of_arrays better",
    impact: "Cache misses",
    fix: "Data structure transform"
  }
}
```

### 2.3: Concurrency Patterns

**Search for synchronization issues**:

```
DEFINE concurrency_patterns = {
  "lock_contention": {
    detect: "multiple_threads compete for single_lock",
    fix: "Shard locks OR lock-free_structure"
  },

  "false_sharing": {
    detect: "adjacent_data modified by different_threads",
    fix: "Pad to cache_line_size"
  },

  "blocking_in_async": {
    detect: "synchronous_operation in async_context",
    fix: "Make operation async OR offload_to_threadpool"
  },

  "excessive_context_switching": {
    detect: "too_many_threads OR bad_scheduling",
    fix: "Thread pool OR work_stealing_queue"
  }
}
```

### 2.4: I/O Patterns

**Search for I/O inefficiency**:

```
DEFINE io_patterns = {
  "unbuffered_io": {
    detect: "small reads/writes without buffer",
    fix: "Add buffering layer"
  },

  "n_plus_one": {
    detect: "query in loop (N queries instead of 1)",
    fix: "Batch query OR join"
  },

  "synchronous_io_blocking": {
    detect: "blocking I/O in hot_path",
    fix: "Async I/O OR prefetch"
  },

  "repeated_parsing": {
    detect: "parse same data multiple times",
    fix: "Cache parsed result"
  }
}
```

---

## Phase 3: Optimization Strategy (Abstract)

### 3.1: Classification & Prioritization

```
FOR each identified_issue:

  CALCULATE impact_score:
    impact = (time_consumed_percent * 40)
           + (memory_waste_MB / total_memory * 30)
           + (frequency_of_execution / max_frequency * 20)
           + (ease_of_fix * 10)

  CLASSIFY by optimization_type:
    - ALGORITHMIC: Change algorithm (O(n²) → O(n))
    - STRUCTURAL: Change data structure
    - COMPUTATIONAL: Reduce operations
    - MEMORY: Reduce allocations
    - CONCURRENCY: Improve parallelism
    - I/O: Reduce/batch I/O
    - COMPILER: Enable optimizations

  ASSIGN risk_level:
    LOW: Configuration change, no logic change
    MEDIUM: Optimization maintains semantics
    HIGH: Algorithm change, careful testing needed
```

### 3.2: Generate Optimization Plan

```
CREATE optimization_plan:

  quick_wins: [  # Apply immediately
    {
      issue: "Debug symbols in release",
      fix: "Enable strip in release config",
      risk: LOW,
      expected_gain: "5-10% binary size"
    },
    ...
  ]

  medium_effort: [  # Apply with testing
    {
      issue: "Allocation in loop",
      fix: "Pre-allocate with capacity",
      risk: MEDIUM,
      expected_gain: "15-20% speed, less GC"
    },
    ...
  ]

  major_refactors: [  # Requires significant work
    {
      issue: "O(n²) search algorithm",
      fix: "Replace with hash_table (O(1))",
      risk: HIGH,
      expected_gain: "10-100x speedup for large N"
    },
    ...
  ]
```

---

## Phase 4: Implementation (Procedural)

### 4.1: Apply Optimizations Systematically

```
FOR each optimization in sorted(by impact_score, DESC):

  STEP 1: Create backup/branch
    git checkout -b "perf/optimize-{optimization.name}"

  STEP 2: Implement fix
    MATCH optimization.type:
      CASE ALGORITHMIC:
        - Replace algorithm
        - Maintain interface contract
        - Add complexity comments

      CASE STRUCTURAL:
        - Transform data structure
        - Update all access patterns
        - Verify behavior unchanged

      CASE MEMORY:
        - Add pre-allocation
        - Use object pools if needed
        - Check for leaks

      CASE CONCURRENCY:
        - Add/modify synchronization
        - Test under load
        - Verify no data races

      CASE I/O:
        - Add buffering
        - Batch operations
        - Make async if possible

      CASE COMPILER:
        - Update build config
        - Enable LTO, optimization flags
        - Verify no behavior change

  STEP 3: Add benchmarks for this specific optimization
    CREATE benchmark measuring:
      - Before/after performance
      - Resource usage
      - Edge cases

  STEP 4: Run tests
    RUN all_tests()
    RUN new_benchmarks()
    IF tests_pass AND benchmark_improves:
      COMMIT with metrics
    ELSE:
      REVERT and ANALYZE why_failed
```

### 4.2: Language-Specific Optimization Techniques

**Abstracted by category, applied per language**:

```python
optimization_techniques = {

  "reduce_allocations": {
    "Rust": ["use &str instead of String", "with_capacity()", "SmallVec", "Cow"],
    "Go": ["sync.Pool", "pre-allocate slices", "reuse buffers"],
    "Java": ["StringBuilder not +", "ArrayList.ensureCapacity()", "object pools"],
    "Python": ["__slots__", "generator expressions", "array.array"],
    "JS/TS": ["TypedArray", "object pools", "WeakMap for caches"]
  },

  "improve_locality": {
    "any": ["struct of arrays", "cache blocking", "data-oriented design"],
    "specific": ["align to cache line", "pack structs", "reorder fields"]
  },

  "reduce_copying": {
    "Rust": ["use references", "Cow", "Arc for shared", "move semantics"],
    "Go": ["pass pointers", "avoid interface{}", "use unsafe carefully"],
    "Python": ["use views", "avoid copy()", "use iterators"],
    "any": ["copy-on-write", "ref-counting", "borrowing"]
  },

  "better_algorithms": {
    "search": ["hash_table O(1)", "binary_search O(log n)", "index/cache"],
    "sort": ["partial_sort for top_k", "radix_sort for integers", "avoid if possible"],
    "iteration": ["single_pass", "early_termination", "parallel if large"]
  },

  "concurrency": {
    "any": ["minimize lock scope", "lock-free structures", "work stealing"],
    "specific": ["sharded locks", "RwLock", "atomic operations", "message passing"]
  }
}

APPLY_TECHNIQUE(issue, detected_language):
  category = CATEGORIZE(issue)
  generic_solutions = optimization_techniques[category]["any"]
  specific_solutions = optimization_techniques[category][detected_language]
  RETURN generic_solutions + specific_solutions
```

---

## Phase 5: Verification (Universal)

### 5.1: Measure Improvements

```
AFTER applying optimizations:

  1. RE-RUN profiling from Phase 1
  2. COMPARE against baseline:

     improvement_report = {
       throughput: {
         before: baseline.throughput,
         after: current.throughput,
         change: ((after - before) / before * 100) + "%"
       },

       latency_p95: {
         before: baseline.latency_p95,
         after: current.latency_p95,
         improvement: ((before - after) / before * 100) + "%"
       },

       memory: {
         before: baseline.memory_peak,
         after: current.memory_peak,
         reduction: ((before - after) / before * 100) + "%"
       },

       cpu: {
         before: baseline.cpu_avg,
         after: current.cpu_avg,
         change: ((before - after) / before * 100) + "%"
       }
     }

  3. VERIFY no regressions:
     - Run full test suite
     - Check edge cases
     - Verify correctness
     - Load test if applicable
```

### 5.2: Document Changes

```
CREATE performance_report.md:

  ## Optimization Summary
  Date: {date}
  Languages: {detected_langs}

  ## Baseline Metrics
  - Throughput: {X} ops/sec
  - Latency p95: {Y} ms
  - Memory: {Z} MB

  ## Issues Identified
  FOR each issue:
    - {description}
    - Impact: {score}/100
    - Type: {ALGORITHMIC|MEMORY|etc}

  ## Optimizations Applied
  FOR each optimization:
    - {description}
    - Code location: {file:line}
    - Expected improvement: {X}%
    - Actual improvement: {Y}%
    - Risk level: {LOW|MED|HIGH}

  ## Final Metrics
  - Throughput: {X'} ops/sec (+{improvement}%)
  - Latency p95: {Y'} ms (-{improvement}%)
  - Memory: {Z'} MB (-{improvement}%)

  ## Configuration Changes
  {build config updates}

  ## Recommendations
  - Further optimizations possible
  - Monitoring points to watch
  - Next optimization targets
```

---

## Usage

When user requests performance optimization:

1. **Auto-detect project type** using Phase 1
2. **Run profiling** with detected tools
3. **Analyze patterns** procedurally using Phase 2 definitions
4. **Generate optimization plan** with impact scores
5. **Apply optimizations** starting with quick wins
6. **Verify improvements** with benchmarks
7. **Create detailed report** with before/after metrics

**Ask user**: "Would you like me to start performance analysis now?"

Then execute this framework procedurally, adapting to detected languages and available tooling.
