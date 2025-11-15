// Icarus Configuration
// Central configuration for all Icarus subsystems

use serde::{Deserialize, Serialize};
use anyhow::Result;

/// Main Icarus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IcarusConfig {
    /// Agent system configuration
    pub agents: AgentConfig,

    /// Memory hierarchy configuration
    pub memory: MemoryConfig,

    /// Neural core configuration
    pub neural: NeuralConfig,

    /// World model configuration
    pub world_model: WorldModelConfig,

    /// Event bus configuration
    pub event_bus: EventBusConfig,
}

impl Default for IcarusConfig {
    fn default() -> Self {
        Self {
            agents: AgentConfig::default(),
            memory: MemoryConfig::default(),
            neural: NeuralConfig::default(),
            world_model: WorldModelConfig::default(),
            event_bus: EventBusConfig::default(),
        }
    }
}

impl IcarusConfig {
    /// Load configuration from file or use defaults
    pub fn load() -> Result<Self> {
        // For now, return defaults
        // Later: load from TOML file
        Ok(Self::default())
    }

    /// Load configuration from specific path
    pub fn load_from_path(path: &str) -> Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&contents)?;
        Ok(config)
    }
}

/// Agent system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Enable agent system
    pub enabled: bool,

    /// Update interval in milliseconds
    pub update_interval_ms: u64,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            update_interval_ms: 100, // 10 Hz
        }
    }
}

/// Memory hierarchy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Working memory capacity (items)
    pub working_capacity: usize,

    /// Short-term memory capacity (items)
    pub short_term_capacity: usize,

    /// Long-term memory capacity (items, 0 = unlimited)
    pub long_term_capacity: usize,

    /// Episodic memory capacity (episodes)
    pub episodic_capacity: usize,

    /// Consolidation interval in seconds
    pub consolidation_interval_secs: u64,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            working_capacity: 7,  // Miller's Law: 7±2
            short_term_capacity: 100,
            long_term_capacity: 0,  // Unlimited
            episodic_capacity: 1000,
            consolidation_interval_secs: 60,
        }
    }
}

/// Neural core configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralConfig {
    /// State dimension
    pub state_dim: usize,

    /// Hidden dimension
    pub hidden_dim: usize,

    /// Number of SSM layers
    pub ssm_layers: usize,

    /// Number of Liquid layers
    pub liquid_layers: usize,

    /// Number of RNN layers
    pub rnn_layers: usize,

    /// Enable CUDA acceleration
    #[cfg(feature = "cuda")]
    pub cuda_enabled: bool,

    /// CUDA device ID
    #[cfg(feature = "cuda")]
    pub cuda_device: usize,
}

impl Default for NeuralConfig {
    fn default() -> Self {
        Self {
            state_dim: 256,
            hidden_dim: 512,
            ssm_layers: 4,
            liquid_layers: 2,
            rnn_layers: 2,
            #[cfg(feature = "cuda")]
            cuda_enabled: false,  // Default to CPU for safety
            #[cfg(feature = "cuda")]
            cuda_device: 0,
        }
    }
}

/// World model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldModelConfig {
    /// Enable world model
    pub enabled: bool,

    /// Prediction horizon (steps)
    pub prediction_horizon: usize,

    /// Update frequency (Hz)
    pub update_frequency_hz: f32,
}

impl Default for WorldModelConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            prediction_horizon: 10,
            update_frequency_hz: 10.0,
        }
    }
}

/// Event bus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBusConfig {
    /// Channel capacity
    pub channel_capacity: usize,

    /// Enable event logging
    pub log_events: bool,
}

impl Default for EventBusConfig {
    fn default() -> Self {
        Self {
            channel_capacity: 1000,
            log_events: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = IcarusConfig::default();
        assert!(config.agents.enabled);
        assert_eq!(config.memory.working_capacity, 7);
        assert_eq!(config.neural.state_dim, 256);
    }

    #[test]
    fn test_config_load() {
        let config = IcarusConfig::load();
        assert!(config.is_ok());
    }
}
