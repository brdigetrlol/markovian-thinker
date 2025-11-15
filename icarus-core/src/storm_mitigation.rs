// Markovian Thinker: Event Storm Mitigation System
// Orchestrates rate limiting, circuit breaking, and event fusion

use crate::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
use crate::event_fusion::{EventFusion, EventFusionConfig};
use crate::events::EventWithMetadata;
use crate::rate_limit::{RateLimiter, RateLimitConfig};
use serde::{Deserialize, Serialize};

/// Storm mitigation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StormMitigationConfig {
    /// Rate limiter configuration
    pub rate_limit: RateLimitConfig,

    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,

    /// Event fusion configuration
    pub event_fusion: EventFusionConfig,

    /// Enable rate limiting
    pub enable_rate_limit: bool,

    /// Enable circuit breaker
    pub enable_circuit_breaker: bool,

    /// Enable event fusion
    pub enable_event_fusion: bool,
}

impl Default for StormMitigationConfig {
    fn default() -> Self {
        Self {
            rate_limit: RateLimitConfig::default(),
            circuit_breaker: CircuitBreakerConfig::default(),
            event_fusion: EventFusionConfig::default(),
            enable_rate_limit: true,
            enable_circuit_breaker: true,
            enable_event_fusion: true,
        }
    }
}

impl StormMitigationConfig {
    /// Aggressive mitigation (strict limits)
    pub fn aggressive() -> Self {
        Self {
            rate_limit: RateLimitConfig::conservative(),
            circuit_breaker: CircuitBreakerConfig::aggressive(),
            event_fusion: EventFusionConfig::aggressive(),
            enable_rate_limit: true,
            enable_circuit_breaker: true,
            enable_event_fusion: true,
        }
    }

    /// Lenient mitigation (loose limits)
    pub fn lenient() -> Self {
        Self {
            rate_limit: RateLimitConfig::aggressive(),
            circuit_breaker: CircuitBreakerConfig::lenient(),
            event_fusion: EventFusionConfig::conservative(),
            enable_rate_limit: true,
            enable_circuit_breaker: true,
            enable_event_fusion: true,
        }
    }

    /// Disabled mitigation (for testing)
    pub fn disabled() -> Self {
        Self {
            rate_limit: RateLimitConfig::default(),
            circuit_breaker: CircuitBreakerConfig::default(),
            event_fusion: EventFusionConfig::default(),
            enable_rate_limit: false,
            enable_circuit_breaker: false,
            enable_event_fusion: false,
        }
    }
}

/// Storm mitigation system
pub struct StormMitigation {
    config: StormMitigationConfig,
    rate_limiter: RateLimiter,
    circuit_breaker: CircuitBreaker,
    event_fusion: EventFusion,
    metrics: StormMetrics,
}

impl StormMitigation {
    /// Create a new storm mitigation system
    pub fn new(config: StormMitigationConfig) -> Self {
        Self {
            rate_limiter: RateLimiter::new(config.rate_limit.clone()),
            circuit_breaker: CircuitBreaker::new(config.circuit_breaker.clone()),
            event_fusion: EventFusion::new(config.event_fusion.clone()),
            config,
            metrics: StormMetrics::default(),
        }
    }

    /// Check if an event should be allowed
    pub fn allow_event(&mut self) -> MitigationDecision {
        self.metrics.total_checks += 1;

        // Check circuit breaker first (fastest)
        if self.config.enable_circuit_breaker {
            if !self.circuit_breaker.allow_request() {
                self.metrics.circuit_breaker_rejections += 1;
                return MitigationDecision::Rejected {
                    reason: "Circuit breaker open".to_string(),
                };
            }
        }

        // Check rate limiter
        if self.config.enable_rate_limit {
            if !self.rate_limiter.try_acquire_one() {
                self.metrics.rate_limit_rejections += 1;
                return MitigationDecision::RateLimited {
                    retry_after: self.rate_limiter.wait_for_tokens(1.0),
                };
            }
        }

        self.metrics.allowed_events += 1;
        MitigationDecision::Allowed
    }

    /// Record a successful event completion
    pub fn record_success(&mut self) {
        if self.config.enable_circuit_breaker {
            self.circuit_breaker.record_success();
        }
        self.metrics.successful_events += 1;
    }

    /// Record a failed event
    pub fn record_failure(&mut self) {
        if self.config.enable_circuit_breaker {
            self.circuit_breaker.record_failure();
        }
        self.metrics.failed_events += 1;
    }

    /// Fuse a batch of pending events
    pub fn fuse_events(&mut self, events: Vec<EventWithMetadata>) -> Vec<EventWithMetadata> {
        if !self.config.enable_event_fusion || events.is_empty() {
            return events;
        }

        let original_count = events.len();
        let fused = self.event_fusion.fuse_events(events);
        let fused_count = fused.len();

        self.metrics.total_fusions += 1;
        self.metrics.events_fused += original_count.saturating_sub(fused_count);

        fused
    }

    /// Emergency stop (trips circuit breaker)
    pub fn emergency_stop(&mut self) {
        self.circuit_breaker.trip();
        self.metrics.emergency_stops += 1;
    }

    /// Reset mitigation (clear state)
    pub fn reset(&mut self) {
        self.rate_limiter.reset();
        self.circuit_breaker.reset();
        self.metrics = StormMetrics::default();
    }

    /// Get current circuit state
    pub fn circuit_state(&self) -> CircuitState {
        self.circuit_breaker.state()
    }

    /// Get metrics
    pub fn metrics(&self) -> &StormMetrics {
        &self.metrics
    }

    /// Get detailed statistics
    pub fn stats(&self) -> StormMitigationStats {
        StormMitigationStats {
            circuit_state: self.circuit_breaker.state(),
            rate_limiter_stats: self.rate_limiter.stats(),
            circuit_breaker_stats: self.circuit_breaker.stats(),
            metrics: self.metrics.clone(),
        }
    }
}

impl Clone for StormMitigation {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            rate_limiter: self.rate_limiter.clone(),
            circuit_breaker: self.circuit_breaker.clone(),
            event_fusion: EventFusion::new(self.config.event_fusion.clone()),
            metrics: self.metrics.clone(),
        }
    }
}

/// Decision on whether to allow an event
#[derive(Debug, Clone, PartialEq)]
pub enum MitigationDecision {
    /// Event is allowed
    Allowed,

    /// Event is rejected (circuit breaker open)
    Rejected { reason: String },

    /// Event is rate limited
    RateLimited {
        retry_after: std::time::Duration,
    },
}

/// Storm mitigation metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StormMetrics {
    pub total_checks: usize,
    pub allowed_events: usize,
    pub rate_limit_rejections: usize,
    pub circuit_breaker_rejections: usize,
    pub successful_events: usize,
    pub failed_events: usize,
    pub total_fusions: usize,
    pub events_fused: usize,
    pub emergency_stops: usize,
}

impl StormMetrics {
    /// Get rejection rate
    pub fn rejection_rate(&self) -> f64 {
        if self.total_checks == 0 {
            0.0
        } else {
            (self.rate_limit_rejections + self.circuit_breaker_rejections) as f64
                / self.total_checks as f64
        }
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        let total_events = self.successful_events + self.failed_events;
        if total_events == 0 {
            0.0
        } else {
            self.successful_events as f64 / total_events as f64
        }
    }

    /// Get fusion effectiveness
    pub fn fusion_effectiveness(&self) -> f64 {
        if self.total_fusions == 0 {
            0.0
        } else {
            self.events_fused as f64 / self.total_fusions as f64
        }
    }
}

/// Combined storm mitigation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StormMitigationStats {
    pub circuit_state: CircuitState,
    pub rate_limiter_stats: crate::rate_limit::RateLimiterStats,
    pub circuit_breaker_stats: crate::circuit_breaker::CircuitBreakerStats,
    pub metrics: StormMetrics,
}

/// Per-session storm mitigation manager
pub struct SessionStormMitigation {
    config: StormMitigationConfig,
    sessions: std::collections::HashMap<String, StormMitigation>,
}

impl SessionStormMitigation {
    /// Create a new session storm mitigation manager
    pub fn new(config: StormMitigationConfig) -> Self {
        Self {
            config,
            sessions: std::collections::HashMap::new(),
        }
    }

    /// Get or create mitigation for session
    pub fn get_mitigation(&mut self, session_id: &str) -> &mut StormMitigation {
        self.sessions
            .entry(session_id.to_string())
            .or_insert_with(|| StormMitigation::new(self.config.clone()))
    }

    /// Remove session
    pub fn remove_session(&mut self, session_id: &str) {
        self.sessions.remove(session_id);
    }

    /// Get all session statistics
    pub fn all_stats(&self) -> Vec<(String, StormMitigationStats)> {
        self.sessions
            .iter()
            .map(|(session_id, mitigation)| (session_id.clone(), mitigation.stats()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storm_mitigation_creation() {
        let config = StormMitigationConfig::default();
        let mitigation = StormMitigation::new(config);

        assert_eq!(mitigation.circuit_state(), CircuitState::Closed);
    }

    #[test]
    fn test_allow_event() {
        let config = StormMitigationConfig::default();
        let mut mitigation = StormMitigation::new(config);

        let decision = mitigation.allow_event();
        assert_eq!(decision, MitigationDecision::Allowed);

        let metrics = mitigation.metrics();
        assert_eq!(metrics.total_checks, 1);
        assert_eq!(metrics.allowed_events, 1);
    }

    #[test]
    fn test_rate_limit_rejection() {
        let config = StormMitigationConfig {
            rate_limit: RateLimitConfig {
                max_tokens: 5.0,
                refill_rate: 1.0,
                initial_tokens: Some(5.0),
            },
            enable_rate_limit: true,
            ..Default::default()
        };
        let mut mitigation = StormMitigation::new(config);

        // Allow first 5
        for _ in 0..5 {
            assert_eq!(mitigation.allow_event(), MitigationDecision::Allowed);
        }

        // 6th should be rate limited
        let decision = mitigation.allow_event();
        assert!(matches!(decision, MitigationDecision::RateLimited { .. }));

        let metrics = mitigation.metrics();
        assert_eq!(metrics.rate_limit_rejections, 1);
    }

    #[test]
    fn test_circuit_breaker_rejection() {
        let config = StormMitigationConfig {
            circuit_breaker: CircuitBreakerConfig {
                failure_threshold: 3,
                success_threshold: 2,
                failure_window: std::time::Duration::from_secs(60),
                recovery_timeout: std::time::Duration::from_secs(10),
                max_consecutive_failures: 10,
            },
            enable_circuit_breaker: true,
            ..Default::default()
        };
        let mut mitigation = StormMitigation::new(config);

        // Record failures to trip circuit
        for _ in 0..3 {
            mitigation.record_failure();
        }

        // Should be rejected now
        let decision = mitigation.allow_event();
        assert!(matches!(decision, MitigationDecision::Rejected { .. }));

        let metrics = mitigation.metrics();
        assert_eq!(metrics.circuit_breaker_rejections, 1);
    }

    #[test]
    fn test_record_success() {
        let config = StormMitigationConfig::default();
        let mut mitigation = StormMitigation::new(config);

        mitigation.record_success();

        let metrics = mitigation.metrics();
        assert_eq!(metrics.successful_events, 1);
    }

    #[test]
    fn test_record_failure() {
        let config = StormMitigationConfig::default();
        let mut mitigation = StormMitigation::new(config);

        mitigation.record_failure();

        let metrics = mitigation.metrics();
        assert_eq!(metrics.failed_events, 1);
    }

    #[test]
    fn test_emergency_stop() {
        let config = StormMitigationConfig::default();
        let mut mitigation = StormMitigation::new(config);

        mitigation.emergency_stop();

        assert_eq!(mitigation.circuit_state(), CircuitState::Open);

        let metrics = mitigation.metrics();
        assert_eq!(metrics.emergency_stops, 1);
    }

    #[test]
    fn test_reset() {
        let config = StormMitigationConfig::default();
        let mut mitigation = StormMitigation::new(config);

        mitigation.allow_event();
        mitigation.record_success();
        mitigation.reset();

        let metrics = mitigation.metrics();
        assert_eq!(metrics.total_checks, 0);
        assert_eq!(metrics.successful_events, 0);
    }

    #[test]
    fn test_metrics_rejection_rate() {
        let mut metrics = StormMetrics::default();
        metrics.total_checks = 10;
        metrics.rate_limit_rejections = 3;
        metrics.circuit_breaker_rejections = 2;

        assert_eq!(metrics.rejection_rate(), 0.5);
    }

    #[test]
    fn test_metrics_success_rate() {
        let mut metrics = StormMetrics::default();
        metrics.successful_events = 8;
        metrics.failed_events = 2;

        assert_eq!(metrics.success_rate(), 0.8);
    }

    #[test]
    fn test_config_presets() {
        let aggressive = StormMitigationConfig::aggressive();
        assert!(aggressive.enable_rate_limit);
        assert!(aggressive.enable_circuit_breaker);
        assert!(aggressive.enable_event_fusion);

        let lenient = StormMitigationConfig::lenient();
        assert!(lenient.enable_rate_limit);

        let disabled = StormMitigationConfig::disabled();
        assert!(!disabled.enable_rate_limit);
        assert!(!disabled.enable_circuit_breaker);
        assert!(!disabled.enable_event_fusion);
    }

    #[test]
    fn test_session_storm_mitigation() {
        let config = StormMitigationConfig::default();
        let mut manager = SessionStormMitigation::new(config);

        // Session 1
        let m1 = manager.get_mitigation("session1");
        m1.allow_event();

        // Session 2 (independent)
        let m2 = manager.get_mitigation("session2");
        m2.allow_event();

        let stats = manager.all_stats();
        assert_eq!(stats.len(), 2);
    }
}
