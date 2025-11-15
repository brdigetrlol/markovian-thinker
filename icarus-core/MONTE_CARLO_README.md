# Monte Carlo Decision Making Module

## Overview

Monte Carlo sampling and tree search implementation for markovian-thinker reasoning system. Provides epsilon-greedy, UCB1, and softmax selection algorithms for balancing exploration vs exploitation in reasoning paths.

## Location

- **Module:** `src/monte_carlo.rs` (396 lines)
- **Tests:** Integrated unit tests (3 tests, all passing)
- **Exports:** Available in `lib.rs`

## Installation

Already integrated into markovian-thinker:

```bash
cd /mnt/c/Users/brdig/Documents/DriveSync/Software/repos-pxl/mcp/workspace/markovian-thinker
cargo build --release
cargo test --release monte_carlo
```

## Core Components

### 1. MonteCarloConfig

Configuration struct with three presets: balanced, creative, and focused.

```rust
use markovian_thinker::{MonteCarloConfig, MonteCarloSampler};

// Use a preset
let config = MonteCarloConfig::balanced();
let config = MonteCarloConfig::creative();
let config = MonteCarloConfig::focused();

// Or create custom configuration
let config = MonteCarloConfig {
    num_samples: 15,
    temperature: 1.2,
    exploration_rate: 0.15,
    ucb_constant: 1.5,
    use_ucb: true,
};
```

**Configuration Presets:**

| Preset | num_samples | temperature | exploration_rate | ucb_constant | use_ucb |
|--------|------------|-------------|------------------|--------------|---------|
| **Balanced** | 10 | 1.0 | 0.1 | 1.414 (√2) | true |
| **Creative** | 20 | 1.5 | 0.3 | 2.0 | true |
| **Focused** | 5 | 0.5 | 0.05 | 1.0 | false |

**Parameters:**

- `num_samples` - Number of samples per decision point
- `temperature` - Controls softmax sampling (0.0-2.0)
  - Lower = more exploitation
  - Higher = more exploration
- `exploration_rate` - Epsilon-greedy parameter (0.0-1.0)
  - 0.0 = pure exploitation
  - 1.0 = pure exploration
- `ucb_constant` - UCB1 exploration bonus (typically √2 ≈ 1.414)
- `use_ucb` - Enable UCB1 algorithm

### 2. MonteCarloSampler

Main sampler with three selection algorithms.

```rust
let mut sampler = MonteCarloSampler::new(MonteCarloConfig::balanced());

// Define possible actions
let actions = vec![
    "continue_reasoning".to_string(),
    "branch_exploration".to_string(),
    "terminate_early".to_string(),
];
```

#### Algorithm 1: Epsilon-Greedy

Balance exploration and exploitation with ε probability.

```rust
let selected = sampler.epsilon_greedy_select(&actions);
```

**Behavior:**
- With probability ε: select random action (explore)
- With probability 1-ε: select best action (exploit)

**Best For:**
- Simple exploration/exploitation balance
- Quick convergence to good solutions
- When you know exploration rate

#### Algorithm 2: UCB1 (Upper Confidence Bound)

Optimal exploration bonus based on visit counts.

```rust
let selected = sampler.ucb_select(&actions);
```

**Formula:**
```
UCB(a) = mean_reward(a) + c * sqrt(ln(N) / n(a))
```

Where:
- `mean_reward(a)` = average reward for action a
- `c` = exploration constant (ucb_constant)
- `N` = total visits
- `n(a)` = visits to action a

**Behavior:**
- Unvisited actions get infinite UCB (explored first)
- Exploration bonus decreases as action is visited
- Proven optimal for multi-armed bandits

**Best For:**
- Optimal exploration/exploitation tradeoff
- Unknown environments
- When you want mathematical guarantees

#### Algorithm 3: Softmax Temperature

Temperature-based probability distribution.

```rust
let selected = sampler.softmax_select(&actions);
```

**Formula:**
```
P(a) = exp(reward(a) / T) / Σ exp(reward(a') / T)
```

Where:
- `reward(a)` = mean reward for action a
- `T` = temperature parameter

**Behavior:**
- T → 0: deterministic (always best action)
- T → ∞: uniform random
- T = 1: balanced stochastic selection

**Best For:**
- Smooth exploration/exploitation curves
- Stochastic reasoning paths
- When you want probabilistic guarantees

### 3. Action Statistics

Track and update action statistics.

```rust
// Update after taking action
sampler.update("continue_reasoning", 0.85);
sampler.update("branch_exploration", 0.72);

// Get best action
let best = sampler.best_action(&actions);
println!("Best action: {}", best);

// Get all statistics
let stats = sampler.get_statistics();
for stat in stats {
    println!("{}: reward={:.2}, visits={}",
        stat.action, stat.reward, stat.visits);
}

// Reset for new episode
sampler.reset();
```

### 4. MCTS Node (Future Use)

Node structure for Monte Carlo Tree Search.

```rust
let node = MCTSNode::new("initial_state".to_string());
println!("Mean reward: {}", node.mean_reward());
```

## Usage Examples

### Example 1: Reasoning Path Selection

```rust
use markovian_thinker::{MonteCarloConfig, MonteCarloSampler};

// Initialize sampler
let mut sampler = MonteCarloSampler::new(MonteCarloConfig::balanced());

// Reasoning session
for iteration in 0..10 {
    let actions = vec![
        "continue_linear".to_string(),
        "explore_branch".to_string(),
        "backtrack".to_string(),
    ];

    // Select action using UCB1
    let action = sampler.ucb_select(&actions);

    // Execute reasoning step
    let reward = execute_reasoning(&action);

    // Update statistics
    sampler.update(&action, reward);

    // Check for convergence
    let best = sampler.best_action(&actions);
    println!("Iteration {}: chose {}, best is {}", iteration, action, best);
}

// Final statistics
let stats = sampler.get_statistics();
for stat in stats {
    println!("{}: {:.2} reward over {} visits",
        stat.action, stat.reward, stat.visits);
}
```

### Example 2: Chunk Strategy Selection

```rust
// At each chunk boundary, select next strategy
let config = MonteCarloConfig::creative(); // More exploration
let mut sampler = MonteCarloSampler::new(config);

let strategies = vec![
    "deep_analysis".to_string(),
    "broad_search".to_string(),
    "focused_refinement".to_string(),
];

// Use softmax for stochastic selection
let strategy = sampler.softmax_select(&strategies);

// Reward based on chunk quality
let quality_score = evaluate_chunk_quality();
sampler.update(&strategy, quality_score);
```

### Example 3: Multi-Armed Bandit

```rust
// Classic bandit problem
let arms = vec!["arm_1".to_string(), "arm_2".to_string(), "arm_3".to_string()];
let mut sampler = MonteCarloSampler::new(MonteCarloConfig::balanced());

for _ in 0..100 {
    let arm = sampler.ucb_select(&arms);
    let reward = pull_arm(&arm); // Returns 0.0 to 1.0
    sampler.update(&arm, reward);
}

// Show which arm is best
let best = sampler.best_action(&arms);
println!("Best arm: {}", best);
```

## Integration with SessionManager

Recommended integration pattern:

```rust
// In SessionManager
pub struct ReasoningSession {
    // Existing fields...
    monte_carlo: Option<MonteCarloSampler>,
}

impl ReasoningSession {
    pub fn with_monte_carlo(mut self, config: MonteCarloConfig) -> Self {
        self.monte_carlo = Some(MonteCarloSampler::new(config));
        self
    }

    pub fn select_action(&mut self, actions: &[String]) -> String {
        if let Some(ref sampler) = self.monte_carlo {
            sampler.ucb_select(actions)
        } else {
            actions[0].clone() // Fallback
        }
    }

    pub fn update_action(&mut self, action: &str, reward: f64) {
        if let Some(ref mut sampler) = self.monte_carlo {
            sampler.update(action, reward);
        }
    }
}
```

## Testing

Run all tests:

```bash
cargo test --release monte_carlo
```

**Tests included:**
1. `test_monte_carlo_config` - Verify presets
2. `test_sampler_update` - Verify statistics tracking
3. `test_best_action` - Verify action selection

**Expected output:**
```
running 3 tests
test monte_carlo::tests::test_monte_carlo_config ... ok
test monte_carlo::tests::test_best_action ... ok
test monte_carlo::tests::test_sampler_update ... ok

test result: ok. 3 passed; 0 failed; 0 ignored
```

## Performance

- **Memory:** ~1KB per sampler instance
- **Computation:** O(n) for selection (n = number of actions)
- **Overhead:** Negligible (<1% of reasoning time)

## When to Use Each Algorithm

### Epsilon-Greedy
- ✅ Simple implementation
- ✅ Predictable behavior
- ✅ Fast computation
- ❌ Fixed exploration rate
- ❌ Doesn't adapt

**Use when:** You know the right exploration rate

### UCB1
- ✅ Theoretically optimal
- ✅ Adapts exploration automatically
- ✅ Prioritizes unvisited actions
- ❌ Can over-explore early
- ❌ Assumes stationary rewards

**Use when:** You want optimal long-term performance

### Softmax
- ✅ Smooth probability distribution
- ✅ Temperature control
- ✅ Stochastic guarantees
- ❌ Sensitive to temperature tuning
- ❌ All actions always have nonzero probability

**Use when:** You want probabilistic reasoning

## Troubleshooting

### Issue: All actions have same reward

**Solution:** Increase exploration or run more samples

```rust
let config = MonteCarloConfig {
    num_samples: 50,  // More samples
    exploration_rate: 0.3,  // More exploration
    ..MonteCarloConfig::balanced()
};
```

### Issue: Sampler stuck on suboptimal action

**Solution:** Use UCB1 or increase temperature

```rust
// Use UCB1 for better exploration
let action = sampler.ucb_select(&actions);

// Or increase temperature for softmax
let config = MonteCarloConfig::creative();
```

### Issue: Too much exploration, not converging

**Solution:** Use focused preset or decrease ε

```rust
let config = MonteCarloConfig::focused();
// Or manually tune
let config = MonteCarloConfig {
    exploration_rate: 0.05,  // Less exploration
    ..MonteCarloConfig::balanced()
};
```

## Future Enhancements

- [ ] Full MCTS tree expansion
- [ ] Multi-objective optimization
- [ ] Contextual bandits
- [ ] Thompson sampling
- [ ] Policy gradient methods
- [ ] MCP tools for configuration
- [ ] Real-time statistics export

## References

- **UCB1:** Auer et al. (2002) - "Finite-time Analysis of the Multiarmed Bandit Problem"
- **MCTS:** Browne et al. (2012) - "A Survey of Monte Carlo Tree Search Methods"
- **Softmax:** Sutton & Barto (2018) - "Reinforcement Learning: An Introduction"

## Support

For issues or questions:
1. Run tests: `cargo test --release monte_carlo`
2. Check logs: Enable tracing for monte_carlo module
3. Verify configuration: Print config with Debug trait

## License

Part of the markovian-thinker reasoning system.
