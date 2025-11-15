/**
 * Monte Carlo Decision Making Module
 * Provides Monte Carlo Tree Search (MCTS) and sampling strategies
 * for exploration/exploitation balance in reasoning
 */

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Monte Carlo configuration for decision-making
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonteCarloConfig {
    /// Number of samples to take per decision point
    pub num_samples: usize,

    /// Temperature parameter for softmax sampling (0.0-2.0)
    /// Lower = more exploitation, Higher = more exploration
    pub temperature: f64,

    /// Epsilon-greedy exploration rate (0.0-1.0)
    pub exploration_rate: f64,

    /// UCB constant for Upper Confidence Bound (typically sqrt(2))
    pub ucb_constant: f64,

    /// Enable UCB1 algorithm for action selection
    pub use_ucb: bool,
}

impl Default for MonteCarloConfig {
    fn default() -> Self {
        Self::balanced()
    }
}

impl MonteCarloConfig {
    /// Balanced configuration (moderate exploration)
    pub fn balanced() -> Self {
        Self {
            num_samples: 10,
            temperature: 1.0,
            exploration_rate: 0.1,
            ucb_constant: 1.414, // sqrt(2)
            use_ucb: true,
        }
    }

    /// Creative configuration (high exploration)
    pub fn creative() -> Self {
        Self {
            num_samples: 20,
            temperature: 1.5,
            exploration_rate: 0.3,
            ucb_constant: 2.0,
            use_ucb: true,
        }
    }

    /// Focused configuration (low exploration)
    pub fn focused() -> Self {
        Self {
            num_samples: 5,
            temperature: 0.5,
            exploration_rate: 0.05,
            ucb_constant: 1.0,
            use_ucb: false,
        }
    }
}

/// Action choice with associated statistics
#[derive(Debug, Clone)]
pub struct ActionChoice {
    pub action: String,
    pub reward: f64,
    pub visits: usize,
}

/// Monte Carlo sampler for decision-making
#[derive(Debug)]
pub struct MonteCarloSampler {
    config: MonteCarloConfig,
    action_stats: HashMap<String, ActionStats>,
    total_visits: usize,
}

/// Statistics for an action
#[derive(Debug, Clone)]
struct ActionStats {
    total_reward: f64,
    visits: usize,
    mean_reward: f64,
}

impl MonteCarloSampler {
    /// Create a new Monte Carlo sampler
    pub fn new(config: MonteCarloConfig) -> Self {
        Self {
            config,
            action_stats: HashMap::new(),
            total_visits: 0,
        }
    }

    /// Select an action using epsilon-greedy strategy
    pub fn epsilon_greedy_select(&self, actions: &[String]) -> String {
        let mut rng = rand::thread_rng();

        // Explore with probability epsilon
        if rng.gen::<f64>() < self.config.exploration_rate {
            // Random action
            actions[rng.gen_range(0..actions.len())].clone()
        } else {
            // Exploit: choose best action
            self.best_action(actions)
        }
    }

    /// Select an action using UCB1 (Upper Confidence Bound)
    pub fn ucb_select(&self, actions: &[String]) -> String {
        if self.total_visits == 0 {
            // No visits yet, return random action
            let mut rng = rand::thread_rng();
            return actions[rng.gen_range(0..actions.len())].clone();
        }

        let mut best_action = actions[0].clone();
        let mut best_ucb = f64::NEG_INFINITY;

        for action in actions {
            let ucb_value = self.calculate_ucb(action);
            if ucb_value > best_ucb {
                best_ucb = ucb_value;
                best_action = action.clone();
            }
        }

        best_action
    }

    /// Calculate UCB1 value for an action
    fn calculate_ucb(&self, action: &str) -> f64 {
        if let Some(stats) = self.action_stats.get(action) {
            if stats.visits == 0 {
                return f64::INFINITY; // Unvisited actions get priority
            }

            // UCB1 formula: mean + c * sqrt(ln(N) / n)
            let exploitation = stats.mean_reward;
            let exploration = self.config.ucb_constant *
                ((self.total_visits as f64).ln() / stats.visits as f64).sqrt();

            exploitation + exploration
        } else {
            f64::INFINITY // Unvisited actions get priority
        }
    }

    /// Select an action using temperature-based softmax
    pub fn softmax_select(&self, actions: &[String]) -> String {
        if actions.is_empty() {
            panic!("Cannot select from empty action list");
        }

        // Get rewards for all actions
        let rewards: Vec<f64> = actions.iter()
            .map(|a| self.get_mean_reward(a))
            .collect();

        // Apply temperature scaling and compute softmax
        let scaled_rewards: Vec<f64> = rewards.iter()
            .map(|r| (r / self.config.temperature).exp())
            .collect();

        let sum: f64 = scaled_rewards.iter().sum();
        let probabilities: Vec<f64> = scaled_rewards.iter()
            .map(|r| r / sum)
            .collect();

        // Sample from the distribution
        let mut rng = rand::thread_rng();
        let sample = rng.gen::<f64>();

        let mut cumulative = 0.0;
        for (i, prob) in probabilities.iter().enumerate() {
            cumulative += prob;
            if sample <= cumulative {
                return actions[i].clone();
            }
        }

        // Fallback (shouldn't happen)
        actions[actions.len() - 1].clone()
    }

    /// Update statistics after taking an action
    pub fn update(&mut self, action: &str, reward: f64) {
        self.total_visits += 1;

        let stats = self.action_stats
            .entry(action.to_string())
            .or_insert(ActionStats {
                total_reward: 0.0,
                visits: 0,
                mean_reward: 0.0,
            });

        stats.visits += 1;
        stats.total_reward += reward;
        stats.mean_reward = stats.total_reward / stats.visits as f64;
    }

    /// Get the best action based on mean reward
    pub fn best_action(&self, actions: &[String]) -> String {
        let mut best_action = actions[0].clone();
        let mut best_reward = f64::NEG_INFINITY;

        for action in actions {
            let reward = self.get_mean_reward(action);
            if reward > best_reward {
                best_reward = reward;
                best_action = action.clone();
            }
        }

        best_action
    }

    /// Get mean reward for an action
    fn get_mean_reward(&self, action: &str) -> f64 {
        self.action_stats
            .get(action)
            .map(|s| s.mean_reward)
            .unwrap_or(0.0)
    }

    /// Get statistics for all actions
    pub fn get_statistics(&self) -> Vec<ActionChoice> {
        self.action_stats
            .iter()
            .map(|(action, stats)| ActionChoice {
                action: action.clone(),
                reward: stats.mean_reward,
                visits: stats.visits,
            })
            .collect()
    }

    /// Reset all statistics
    pub fn reset(&mut self) {
        self.action_stats.clear();
        self.total_visits = 0;
    }
}

/// MCTS Node for tree search
#[derive(Debug, Clone)]
pub struct MCTSNode {
    pub state: String,
    pub visits: usize,
    pub total_reward: f64,
    pub children: Vec<MCTSNode>,
    pub parent_action: Option<String>,
}

impl MCTSNode {
    /// Create a new MCTS node
    pub fn new(state: String) -> Self {
        Self {
            state,
            visits: 0,
            total_reward: 0.0,
            children: Vec::new(),
            parent_action: None,
        }
    }

    /// Check if node is fully expanded
    pub fn is_fully_expanded(&self, available_actions: &[String]) -> bool {
        self.children.len() >= available_actions.len()
    }

    /// Get mean reward
    pub fn mean_reward(&self) -> f64 {
        if self.visits == 0 {
            0.0
        } else {
            self.total_reward / self.visits as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monte_carlo_config() {
        let config = MonteCarloConfig::balanced();
        assert_eq!(config.num_samples, 10);
        assert_eq!(config.temperature, 1.0);

        let creative = MonteCarloConfig::creative();
        assert!(creative.exploration_rate > config.exploration_rate);

        let focused = MonteCarloConfig::focused();
        assert!(focused.exploration_rate < config.exploration_rate);
    }

    #[test]
    fn test_sampler_update() {
        let mut sampler = MonteCarloSampler::new(MonteCarloConfig::balanced());

        sampler.update("action1", 0.8);
        sampler.update("action1", 0.9);
        sampler.update("action2", 0.5);

        let stats = sampler.get_statistics();
        assert_eq!(stats.len(), 2);

        let action1_stats = stats.iter()
            .find(|s| s.action == "action1")
            .unwrap();
        assert_eq!(action1_stats.visits, 2);
        assert!((action1_stats.reward - 0.85).abs() < 0.01);
    }

    #[test]
    fn test_best_action() {
        let mut sampler = MonteCarloSampler::new(MonteCarloConfig::balanced());

        sampler.update("action1", 0.8);
        sampler.update("action2", 0.9);
        sampler.update("action3", 0.7);

        let actions = vec![
            "action1".to_string(),
            "action2".to_string(),
            "action3".to_string(),
        ];

        let best = sampler.best_action(&actions);
        assert_eq!(best, "action2");
    }
}
