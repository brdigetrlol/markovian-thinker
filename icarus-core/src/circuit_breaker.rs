// Markovian Thinker: Circuit Breaker for Runaway Reasoning Detection
// Prevents cascade failures and reasoning loops

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitState {
    /// Normal operation, all requests allowed
    Closed,

    /// Circuit tripped, all requests rejected
    Open,

    /// Testing recovery, limited requests allowed
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Failure threshold to open circuit
    pub failure_threshold: usize,

    /// Success threshold to close circuit (from HalfOpen)
    pub success_threshold: usize,

    /// Time window for failure counting
    pub failure_window: Duration,

    /// Timeout before attempting recovery (HalfOpen)
    pub recovery_timeout: Duration,

    /// Maximum consecutive failures before permanent trip
    pub max_consecutive_failures: usize,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,              // 5 failures
            success_threshold: 3,              // 3 successes to recover
            failure_window: Duration::from_secs(60), // 60 second window
            recovery_timeout: Duration::from_secs(30), // 30 second cooldown
            max_consecutive_failures: 10,     // 10 consecutive = permanent
        }
    }
}

impl CircuitBreakerConfig {
    /// Aggressive breaker (trips quickly)
    pub fn aggressive() -> Self {
        Self {
            failure_threshold: 3,
            success_threshold: 5,
            failure_window: Duration::from_secs(30),
            recovery_timeout: Duration::from_secs(60),
            max_consecutive_failures: 5,
        }
    }

    /// Lenient breaker (tolerates more failures)
    pub fn lenient() -> Self {
        Self {
            failure_threshold: 10,
            success_threshold: 2,
            failure_window: Duration::from_secs(120),
            recovery_timeout: Duration::from_secs(15),
            max_consecutive_failures: 20,
        }
    }

    /// Per-session breaker
    pub fn per_session() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            failure_window: Duration::from_secs(60),
            recovery_timeout: Duration::from_secs(30),
            max_consecutive_failures: 10,
        }
    }
}

/// Circuit breaker for detecting runaway reasoning
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<Mutex<CircuitBreakerState>>,
}

struct CircuitBreakerState {
    current_state: CircuitState,
    failures: Vec<Instant>,
    consecutive_failures: usize,
    consecutive_successes: usize,
    last_state_change: Instant,
    total_requests: usize,
    total_failures: usize,
    total_successes: usize,
    trip_count: usize,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(CircuitBreakerState {
                current_state: CircuitState::Closed,
                failures: Vec::new(),
                consecutive_failures: 0,
                consecutive_successes: 0,
                last_state_change: Instant::now(),
                total_requests: 0,
                total_failures: 0,
                total_successes: 0,
                trip_count: 0,
            })),
        }
    }

    /// Check if request should be allowed
    pub fn allow_request(&self) -> bool {
        let mut state = self.state.lock().unwrap();
        state.total_requests += 1;

        match state.current_state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if recovery timeout has passed
                if state.last_state_change.elapsed() >= self.config.recovery_timeout {
                    // Transition to HalfOpen
                    state.current_state = CircuitState::HalfOpen;
                    state.consecutive_successes = 0;
                    state.last_state_change = Instant::now();
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Record a successful operation
    pub fn record_success(&self) {
        let mut state = self.state.lock().unwrap();
        state.total_successes += 1;
        state.consecutive_failures = 0;
        state.consecutive_successes += 1;

        match state.current_state {
            CircuitState::HalfOpen => {
                if state.consecutive_successes >= self.config.success_threshold {
                    // Recover to Closed
                    state.current_state = CircuitState::Closed;
                    state.failures.clear();
                    state.last_state_change = Instant::now();
                }
            }
            _ => {}
        }
    }

    /// Record a failed operation
    pub fn record_failure(&self) {
        let mut state = self.state.lock().unwrap();
        state.total_failures += 1;
        state.consecutive_failures += 1;
        state.consecutive_successes = 0;
        state.failures.push(Instant::now());

        // Check for permanent trip
        if state.consecutive_failures >= self.config.max_consecutive_failures {
            if state.current_state != CircuitState::Open {
                state.trip_count += 1;
            }
            state.current_state = CircuitState::Open;
            state.last_state_change = Instant::now();
            return;
        }

        // Remove old failures outside window
        let cutoff = Instant::now() - self.config.failure_window;
        state.failures.retain(|&t| t > cutoff);

        // Check if threshold exceeded
        if state.failures.len() >= self.config.failure_threshold {
            match state.current_state {
                CircuitState::Closed | CircuitState::HalfOpen => {
                    state.trip_count += 1;
                    state.current_state = CircuitState::Open;
                    state.last_state_change = Instant::now();
                }
                _ => {}
            }
        }
    }

    /// Get current circuit state
    pub fn state(&self) -> CircuitState {
        let state = self.state.lock().unwrap();
        state.current_state
    }

    /// Force circuit open (emergency stop)
    pub fn trip(&self) {
        let mut state = self.state.lock().unwrap();
        if state.current_state != CircuitState::Open {
            state.trip_count += 1;
        }
        state.current_state = CircuitState::Open;
        state.last_state_change = Instant::now();
    }

    /// Force circuit closed (manual reset)
    pub fn reset(&self) {
        let mut state = self.state.lock().unwrap();
        state.current_state = CircuitState::Closed;
        state.failures.clear();
        state.consecutive_failures = 0;
        state.consecutive_successes = 0;
        state.last_state_change = Instant::now();
    }

    /// Get statistics
    pub fn stats(&self) -> CircuitBreakerStats {
        let state = self.state.lock().unwrap();

        CircuitBreakerStats {
            current_state: state.current_state,
            total_requests: state.total_requests,
            total_failures: state.total_failures,
            total_successes: state.total_successes,
            consecutive_failures: state.consecutive_failures,
            consecutive_successes: state.consecutive_successes,
            trip_count: state.trip_count,
            failure_rate: if state.total_requests > 0 {
                state.total_failures as f64 / state.total_requests as f64
            } else {
                0.0
            },
            time_in_state: state.last_state_change.elapsed(),
        }
    }
}

impl Clone for CircuitBreaker {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            state: Arc::clone(&self.state),
        }
    }
}

/// Circuit breaker statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerStats {
    pub current_state: CircuitState,
    pub total_requests: usize,
    pub total_failures: usize,
    pub total_successes: usize,
    pub consecutive_failures: usize,
    pub consecutive_successes: usize,
    pub trip_count: usize,
    pub failure_rate: f64,
    pub time_in_state: Duration,
}

/// Per-session circuit breaker manager
pub struct SessionCircuitBreaker {
    config: CircuitBreakerConfig,
    breakers: Arc<Mutex<std::collections::HashMap<String, CircuitBreaker>>>,
}

impl SessionCircuitBreaker {
    /// Create a new session circuit breaker manager
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            breakers: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// Get or create circuit breaker for session
    pub fn get_breaker(&self, session_id: &str) -> CircuitBreaker {
        let mut breakers = self.breakers.lock().unwrap();

        breakers
            .entry(session_id.to_string())
            .or_insert_with(|| CircuitBreaker::new(self.config.clone()))
            .clone()
    }

    /// Allow request for session
    pub fn allow_request(&self, session_id: &str) -> bool {
        let breaker = self.get_breaker(session_id);
        breaker.allow_request()
    }

    /// Record success for session
    pub fn record_success(&self, session_id: &str) {
        let breaker = self.get_breaker(session_id);
        breaker.record_success();
    }

    /// Record failure for session
    pub fn record_failure(&self, session_id: &str) {
        let breaker = self.get_breaker(session_id);
        breaker.record_failure();
    }

    /// Remove session breaker
    pub fn remove_session(&self, session_id: &str) {
        let mut breakers = self.breakers.lock().unwrap();
        breakers.remove(session_id);
    }

    /// Get all session statistics
    pub fn all_stats(&self) -> Vec<(String, CircuitBreakerStats)> {
        let breakers = self.breakers.lock().unwrap();
        breakers
            .iter()
            .map(|(session_id, breaker)| (session_id.clone(), breaker.stats()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_circuit_breaker_creation() {
        let config = CircuitBreakerConfig::default();
        let breaker = CircuitBreaker::new(config);

        assert_eq!(breaker.state(), CircuitState::Closed);
    }

    #[test]
    fn test_allow_request_closed() {
        let config = CircuitBreakerConfig::default();
        let breaker = CircuitBreaker::new(config);

        assert!(breaker.allow_request());
        assert_eq!(breaker.state(), CircuitState::Closed);
    }

    #[test]
    fn test_trip_on_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            failure_window: Duration::from_secs(60),
            recovery_timeout: Duration::from_secs(10),
            max_consecutive_failures: 10,
        };
        let breaker = CircuitBreaker::new(config);

        // Record 3 failures
        breaker.record_failure();
        breaker.record_failure();
        breaker.record_failure();

        // Circuit should be open
        assert_eq!(breaker.state(), CircuitState::Open);
        assert!(!breaker.allow_request());
    }

    #[test]
    fn test_recovery_after_timeout() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            failure_window: Duration::from_secs(60),
            recovery_timeout: Duration::from_millis(100), // Short timeout
            max_consecutive_failures: 10,
        };
        let breaker = CircuitBreaker::new(config);

        // Trip the circuit
        breaker.record_failure();
        breaker.record_failure();
        assert_eq!(breaker.state(), CircuitState::Open);

        // Wait for recovery timeout
        thread::sleep(Duration::from_millis(150));

        // Should transition to HalfOpen
        assert!(breaker.allow_request());
        assert_eq!(breaker.state(), CircuitState::HalfOpen);
    }

    #[test]
    fn test_half_open_to_closed() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            failure_window: Duration::from_secs(60),
            recovery_timeout: Duration::from_millis(100),
            max_consecutive_failures: 10,
        };
        let breaker = CircuitBreaker::new(config);

        // Trip and recover to HalfOpen
        breaker.record_failure();
        breaker.record_failure();
        thread::sleep(Duration::from_millis(150));
        breaker.allow_request();

        // Record successes
        breaker.record_success();
        assert_eq!(breaker.state(), CircuitState::HalfOpen);

        breaker.record_success();
        // Should transition to Closed
        assert_eq!(breaker.state(), CircuitState::Closed);
    }

    #[test]
    fn test_half_open_to_open() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            failure_window: Duration::from_secs(60),
            recovery_timeout: Duration::from_millis(100),
            max_consecutive_failures: 10,
        };
        let breaker = CircuitBreaker::new(config);

        // Trip and recover to HalfOpen
        breaker.record_failure();
        breaker.record_failure();
        thread::sleep(Duration::from_millis(150));
        breaker.allow_request();

        // Record more failures
        breaker.record_failure();
        breaker.record_failure();

        // Should trip back to Open
        assert_eq!(breaker.state(), CircuitState::Open);
    }

    #[test]
    fn test_consecutive_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 10,
            success_threshold: 2,
            failure_window: Duration::from_secs(60),
            recovery_timeout: Duration::from_secs(10),
            max_consecutive_failures: 5,
        };
        let breaker = CircuitBreaker::new(config);

        // Record 5 consecutive failures (max)
        for _ in 0..5 {
            breaker.record_failure();
        }

        // Should trip on consecutive limit
        assert_eq!(breaker.state(), CircuitState::Open);
    }

    #[test]
    fn test_manual_trip() {
        let config = CircuitBreakerConfig::default();
        let breaker = CircuitBreaker::new(config);

        assert_eq!(breaker.state(), CircuitState::Closed);

        breaker.trip();
        assert_eq!(breaker.state(), CircuitState::Open);
        assert!(!breaker.allow_request());
    }

    #[test]
    fn test_manual_reset() {
        let config = CircuitBreakerConfig::default();
        let breaker = CircuitBreaker::new(config);

        // Trip the circuit
        breaker.trip();
        assert_eq!(breaker.state(), CircuitState::Open);

        // Reset
        breaker.reset();
        assert_eq!(breaker.state(), CircuitState::Closed);
        assert!(breaker.allow_request());
    }

    #[test]
    fn test_statistics() {
        let config = CircuitBreakerConfig::default();
        let breaker = CircuitBreaker::new(config);

        breaker.allow_request();
        breaker.record_success();
        breaker.allow_request();
        breaker.record_failure();

        let stats = breaker.stats();
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.total_successes, 1);
        assert_eq!(stats.total_failures, 1);
        assert_eq!(stats.failure_rate, 0.5);
    }

    #[test]
    fn test_trip_count() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            failure_window: Duration::from_secs(60),
            recovery_timeout: Duration::from_millis(100),
            max_consecutive_failures: 10,
        };
        let breaker = CircuitBreaker::new(config);

        // First trip
        breaker.record_failure();
        breaker.record_failure();

        let stats = breaker.stats();
        assert_eq!(stats.trip_count, 1);

        // Recover and trip again
        thread::sleep(Duration::from_millis(150));
        breaker.allow_request();
        breaker.record_failure();
        breaker.record_failure();

        let stats = breaker.stats();
        assert_eq!(stats.trip_count, 2);
    }

    #[test]
    fn test_session_circuit_breaker() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            failure_window: Duration::from_secs(60),
            recovery_timeout: Duration::from_secs(10),
            max_consecutive_failures: 5,
        };
        let manager = SessionCircuitBreaker::new(config);

        // Session 1: trip circuit
        manager.record_failure("session1");
        manager.record_failure("session1");
        assert!(!manager.allow_request("session1"));

        // Session 2: independent circuit
        assert!(manager.allow_request("session2"));
    }

    #[test]
    fn test_config_presets() {
        let aggressive = CircuitBreakerConfig::aggressive();
        assert_eq!(aggressive.failure_threshold, 3);

        let lenient = CircuitBreakerConfig::lenient();
        assert_eq!(lenient.failure_threshold, 10);

        let session = CircuitBreakerConfig::per_session();
        assert_eq!(session.failure_threshold, 5);
    }
}
