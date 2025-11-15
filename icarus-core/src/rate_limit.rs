// Markovian Thinker: Rate Limiting for Event Storm Mitigation
// Token bucket algorithm with burst capacity

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Rate limit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum tokens (burst capacity)
    pub max_tokens: f64,

    /// Token refill rate (tokens per second)
    pub refill_rate: f64,

    /// Initial tokens (0.0 means start full)
    pub initial_tokens: Option<f64>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_tokens: 100.0,       // 100 events burst
            refill_rate: 10.0,       // 10 events/second sustained
            initial_tokens: None,    // Start with full bucket
        }
    }
}

impl RateLimitConfig {
    /// Conservative rate limit (low burst, low rate)
    pub fn conservative() -> Self {
        Self {
            max_tokens: 50.0,
            refill_rate: 5.0,
            initial_tokens: None,
        }
    }

    /// Aggressive rate limit (high burst, high rate)
    pub fn aggressive() -> Self {
        Self {
            max_tokens: 200.0,
            refill_rate: 20.0,
            initial_tokens: None,
        }
    }

    /// Per-session limit (moderate settings)
    pub fn per_session() -> Self {
        Self {
            max_tokens: 100.0,
            refill_rate: 10.0,
            initial_tokens: Some(50.0), // Start half-full
        }
    }
}

/// Token bucket rate limiter (thread-safe)
pub struct RateLimiter {
    config: RateLimitConfig,
    state: Arc<Mutex<RateLimiterState>>,
}

struct RateLimiterState {
    tokens: f64,
    last_refill: Instant,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(config: RateLimitConfig) -> Self {
        let initial_tokens = config.initial_tokens.unwrap_or(config.max_tokens);

        Self {
            config,
            state: Arc::new(Mutex::new(RateLimiterState {
                tokens: initial_tokens,
                last_refill: Instant::now(),
            })),
        }
    }

    /// Try to acquire N tokens, returns true if allowed
    pub fn try_acquire(&self, tokens: f64) -> bool {
        let mut state = self.state.lock().unwrap();

        // Refill tokens based on elapsed time
        self.refill(&mut state);

        // Check if enough tokens available
        if state.tokens >= tokens {
            state.tokens -= tokens;
            true
        } else {
            false
        }
    }

    /// Acquire 1 token (convenience method)
    pub fn try_acquire_one(&self) -> bool {
        self.try_acquire(1.0)
    }

    /// Refill tokens based on elapsed time
    fn refill(&self, state: &mut RateLimiterState) {
        let now = Instant::now();
        let elapsed = now.duration_since(state.last_refill);
        let elapsed_secs = elapsed.as_secs_f64();

        // Calculate tokens to add
        let tokens_to_add = elapsed_secs * self.config.refill_rate;

        // Add tokens, but don't exceed max
        state.tokens = (state.tokens + tokens_to_add).min(self.config.max_tokens);
        state.last_refill = now;
    }

    /// Get current token count
    pub fn available_tokens(&self) -> f64 {
        let mut state = self.state.lock().unwrap();
        self.refill(&mut state);
        state.tokens
    }

    /// Wait until tokens are available (blocking)
    pub fn wait_for_tokens(&self, tokens: f64) -> Duration {
        let mut state = self.state.lock().unwrap();
        self.refill(&mut state);

        if state.tokens >= tokens {
            return Duration::ZERO;
        }

        // Calculate wait time
        let tokens_needed = tokens - state.tokens;
        let wait_secs = tokens_needed / self.config.refill_rate;
        Duration::from_secs_f64(wait_secs)
    }

    /// Reset the rate limiter
    pub fn reset(&self) {
        let mut state = self.state.lock().unwrap();
        state.tokens = self.config.initial_tokens.unwrap_or(self.config.max_tokens);
        state.last_refill = Instant::now();
    }

    /// Get statistics
    pub fn stats(&self) -> RateLimiterStats {
        let state = self.state.lock().unwrap();

        RateLimiterStats {
            current_tokens: state.tokens,
            max_tokens: self.config.max_tokens,
            refill_rate: self.config.refill_rate,
            utilization: 1.0 - (state.tokens / self.config.max_tokens),
        }
    }
}

impl Clone for RateLimiter {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            state: Arc::clone(&self.state),
        }
    }
}

/// Rate limiter statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimiterStats {
    pub current_tokens: f64,
    pub max_tokens: f64,
    pub refill_rate: f64,
    pub utilization: f64, // 0.0 = idle, 1.0 = saturated
}

/// Per-session rate limiter manager
pub struct SessionRateLimiter {
    config: RateLimitConfig,
    limiters: Arc<Mutex<std::collections::HashMap<String, RateLimiter>>>,
}

impl SessionRateLimiter {
    /// Create a new session rate limiter manager
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            limiters: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// Get or create rate limiter for session
    pub fn get_limiter(&self, session_id: &str) -> RateLimiter {
        let mut limiters = self.limiters.lock().unwrap();

        limiters
            .entry(session_id.to_string())
            .or_insert_with(|| RateLimiter::new(self.config.clone()))
            .clone()
    }

    /// Try to acquire for a session
    pub fn try_acquire(&self, session_id: &str, tokens: f64) -> bool {
        let limiter = self.get_limiter(session_id);
        limiter.try_acquire(tokens)
    }

    /// Remove session limiter
    pub fn remove_session(&self, session_id: &str) {
        let mut limiters = self.limiters.lock().unwrap();
        limiters.remove(session_id);
    }

    /// Get number of active sessions
    pub fn active_sessions(&self) -> usize {
        let limiters = self.limiters.lock().unwrap();
        limiters.len()
    }

    /// Get all session statistics
    pub fn all_stats(&self) -> Vec<(String, RateLimiterStats)> {
        let limiters = self.limiters.lock().unwrap();
        limiters
            .iter()
            .map(|(session_id, limiter)| (session_id.clone(), limiter.stats()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_rate_limiter_creation() {
        let config = RateLimitConfig::default();
        let limiter = RateLimiter::new(config);

        let stats = limiter.stats();
        assert_eq!(stats.max_tokens, 100.0);
        assert_eq!(stats.refill_rate, 10.0);
    }

    #[test]
    fn test_acquire_tokens() {
        let config = RateLimitConfig {
            max_tokens: 10.0,
            refill_rate: 1.0,
            initial_tokens: Some(10.0),
        };
        let limiter = RateLimiter::new(config);

        // Should allow first 10 acquisitions
        for _ in 0..10 {
            assert!(limiter.try_acquire_one());
        }

        // Should deny 11th
        assert!(!limiter.try_acquire_one());
    }

    #[test]
    fn test_token_refill() {
        let config = RateLimitConfig {
            max_tokens: 10.0,
            refill_rate: 10.0, // 10 tokens/second
            initial_tokens: Some(0.0),
        };
        let limiter = RateLimiter::new(config);

        // Initially no tokens
        assert!(!limiter.try_acquire_one());

        // Wait 200ms (should refill 2 tokens)
        thread::sleep(Duration::from_millis(200));

        // Should allow 2 acquisitions
        assert!(limiter.try_acquire_one());
        assert!(limiter.try_acquire_one());
        assert!(!limiter.try_acquire_one());
    }

    #[test]
    fn test_burst_capacity() {
        let config = RateLimitConfig {
            max_tokens: 100.0,
            refill_rate: 10.0,
            initial_tokens: Some(100.0),
        };
        let limiter = RateLimiter::new(config);

        // Should allow burst of 50 tokens
        assert!(limiter.try_acquire(50.0));

        let stats = limiter.stats();
        assert!((stats.current_tokens - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_max_tokens_cap() {
        let config = RateLimitConfig {
            max_tokens: 10.0,
            refill_rate: 100.0, // Fast refill
            initial_tokens: Some(5.0),
        };
        let limiter = RateLimiter::new(config);

        // Wait for refill
        thread::sleep(Duration::from_millis(200));

        // Should cap at max_tokens
        let tokens = limiter.available_tokens();
        assert!(tokens <= 10.0);
    }

    #[test]
    fn test_reset() {
        let config = RateLimitConfig {
            max_tokens: 10.0,
            refill_rate: 1.0,
            initial_tokens: Some(10.0),
        };
        let limiter = RateLimiter::new(config);

        // Consume all tokens
        assert!(limiter.try_acquire(10.0));
        assert!(!limiter.try_acquire_one());

        // Reset
        limiter.reset();

        // Should allow again
        assert!(limiter.try_acquire_one());
    }

    #[test]
    fn test_wait_for_tokens() {
        let config = RateLimitConfig {
            max_tokens: 10.0,
            refill_rate: 10.0, // 1 token per 100ms
            initial_tokens: Some(5.0),
        };
        let limiter = RateLimiter::new(config);

        // Need 8 tokens (have 5, need 3 more)
        let wait = limiter.wait_for_tokens(8.0);

        // Should wait ~300ms for 3 tokens
        assert!(wait.as_millis() >= 200 && wait.as_millis() <= 400);
    }

    #[test]
    fn test_utilization() {
        let config = RateLimitConfig {
            max_tokens: 100.0,
            refill_rate: 10.0,
            initial_tokens: Some(100.0),
        };
        let limiter = RateLimiter::new(config);

        // Initially idle
        let stats = limiter.stats();
        assert!((stats.utilization - 0.0).abs() < 0.01);

        // Consume 50%
        limiter.try_acquire(50.0);
        let stats = limiter.stats();
        assert!((stats.utilization - 0.5).abs() < 0.01);

        // Consume 100%
        limiter.try_acquire(50.0);
        let stats = limiter.stats();
        assert!((stats.utilization - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_session_rate_limiter() {
        let config = RateLimitConfig {
            max_tokens: 10.0,
            refill_rate: 1.0,
            initial_tokens: Some(10.0),
        };
        let manager = SessionRateLimiter::new(config);

        // Session 1: consume 5 tokens
        assert!(manager.try_acquire("session1", 5.0));

        // Session 2: independent limit
        assert!(manager.try_acquire("session2", 10.0));

        // Session 1: should have 5 tokens left
        assert!(manager.try_acquire("session1", 5.0));
        assert!(!manager.try_acquire("session1", 1.0));

        assert_eq!(manager.active_sessions(), 2);
    }

    #[test]
    fn test_remove_session() {
        let config = RateLimitConfig::default();
        let manager = SessionRateLimiter::new(config);

        manager.try_acquire("session1", 1.0);
        manager.try_acquire("session2", 1.0);

        assert_eq!(manager.active_sessions(), 2);

        manager.remove_session("session1");
        assert_eq!(manager.active_sessions(), 1);
    }

    #[test]
    fn test_config_presets() {
        let conservative = RateLimitConfig::conservative();
        assert_eq!(conservative.max_tokens, 50.0);
        assert_eq!(conservative.refill_rate, 5.0);

        let aggressive = RateLimitConfig::aggressive();
        assert_eq!(aggressive.max_tokens, 200.0);
        assert_eq!(aggressive.refill_rate, 20.0);

        let session = RateLimitConfig::per_session();
        assert_eq!(session.initial_tokens, Some(50.0));
    }
}
