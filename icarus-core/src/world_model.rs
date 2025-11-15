// World Model
// Predictive simulation of environment

use crate::config::WorldModelConfig;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// World state representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    /// State vector
    pub state: Vec<f32>,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Confidence/uncertainty
    pub confidence: f32,
}

impl WorldState {
    pub fn new(dim: usize) -> Self {
        Self {
            state: vec![0.0; dim],
            timestamp: Utc::now(),
            confidence: 0.5,
        }
    }
}

/// World Model - Predictive simulation
pub struct WorldModel {
    config: WorldModelConfig,

    /// Current world state
    current_state: WorldState,

    /// Predicted future states
    predictions: Vec<WorldState>,

    /// History of observations
    history: Vec<WorldState>,

    /// Maximum history size
    max_history: usize,
}

impl WorldModel {
    pub fn new(config: &WorldModelConfig) -> Result<Self> {
        let state_dim = 256;  // TODO: Make configurable

        Ok(Self {
            config: config.clone(),
            current_state: WorldState::new(state_dim),
            predictions: Vec::new(),
            history: Vec::new(),
            max_history: 100,
        })
    }

    /// Update world model with new observation
    pub fn observe(&mut self, observation: Vec<f32>) {
        let mut new_state = WorldState {
            state: observation,
            timestamp: Utc::now(),
            confidence: 1.0,  // Observed state is certain
        };

        // Resize to match state dimension
        new_state.state.resize(self.current_state.state.len(), 0.0);

        // Add current state to history
        self.history.push(self.current_state.clone());
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }

        // Update current state
        self.current_state = new_state;

        // Generate new predictions
        self.generate_predictions();
    }

    /// Generate predictions for future states
    fn generate_predictions(&mut self) {
        self.predictions.clear();

        // Simple prediction: linear extrapolation from history
        if self.history.len() >= 2 {
            let dt = (self.current_state.timestamp - self.history.last().unwrap().timestamp)
                .num_milliseconds() as f32 / 1000.0;

            for step in 1..=self.config.prediction_horizon {
                let mut predicted_state = self.current_state.clone();

                // Simple velocity-based prediction
                if let Some(prev_state) = self.history.last() {
                    for i in 0..predicted_state.state.len() {
                        let velocity = (self.current_state.state[i] - prev_state.state[i]) / dt;
                        predicted_state.state[i] = self.current_state.state[i] + velocity * (step as f32 * dt);
                    }
                }

                // Decrease confidence for further predictions
                predicted_state.confidence = self.current_state.confidence * 0.9_f32.powi(step as i32);
                predicted_state.timestamp = Utc::now() + chrono::Duration::milliseconds((step as i64 * dt as i64 * 1000));

                self.predictions.push(predicted_state);
            }
        }
    }

    /// Step the world model forward (called periodically)
    pub async fn step(&mut self) -> Result<()> {
        // In production, this would:
        // 1. Check if predictions match observations
        // 2. Update model parameters based on prediction error
        // 3. Generate new predictions

        // For now, just refresh predictions
        self.generate_predictions();

        Ok(())
    }

    /// Get current world state
    pub fn current_state(&self) -> &WorldState {
        &self.current_state
    }

    /// Get predictions
    pub fn predictions(&self) -> &[WorldState] {
        &self.predictions
    }

    /// Get prediction for specific time horizon
    pub fn predict(&self, steps_ahead: usize) -> Option<&WorldState> {
        if steps_ahead > 0 && steps_ahead <= self.predictions.len() {
            Some(&self.predictions[steps_ahead - 1])
        } else {
            None
        }
    }

    /// Calculate prediction error (for learning)
    pub fn prediction_error(&self, observed: &WorldState) -> f32 {
        // Mean squared error between predicted and observed
        if self.predictions.is_empty() {
            return 0.0;
        }

        let predicted = &self.predictions[0];

        let mse: f32 = predicted.state.iter()
            .zip(observed.state.iter())
            .map(|(p, o)| (p - o).powi(2))
            .sum::<f32>() / predicted.state.len() as f32;

        mse
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_model_creation() {
        let config = WorldModelConfig::default();
        let wm = WorldModel::new(&config);
        assert!(wm.is_ok());
    }

    #[test]
    fn test_world_model_observe() {
        let config = WorldModelConfig::default();
        let mut wm = WorldModel::new(&config).unwrap();

        let observation = vec![1.0, 2.0, 3.0];
        wm.observe(observation.clone());

        assert_eq!(wm.current_state().state[0..3], [1.0, 2.0, 3.0]);
        assert_eq!(wm.current_state().confidence, 1.0);
    }

    #[test]
    fn test_predictions() {
        let config = WorldModelConfig {
            enabled: true,
            prediction_horizon: 5,
            update_frequency_hz: 10.0,
        };
        let mut wm = WorldModel::new(&config).unwrap();

        // Add some observations
        wm.observe(vec![0.0, 0.0]);
        std::thread::sleep(std::time::Duration::from_millis(100));
        wm.observe(vec![1.0, 1.0]);

        // Should have predictions
        assert!(!wm.predictions().is_empty());
        assert!(wm.predictions().len() <= config.prediction_horizon);
    }

    #[test]
    fn test_prediction_error() {
        let config = WorldModelConfig::default();
        let mut wm = WorldModel::new(&config).unwrap();

        wm.observe(vec![0.0, 0.0]);
        wm.observe(vec![1.0, 1.0]);

        let observed = WorldState {
            state: vec![1.5, 1.5],
            timestamp: Utc::now(),
            confidence: 1.0,
        };

        let error = wm.prediction_error(&observed);
        assert!(error >= 0.0);  // Error should be non-negative
    }
}
