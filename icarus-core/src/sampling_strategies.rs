// Markovian Thinker: Advanced Sampling Strategies
// Temperature, top-k, and top-p sampling inspired by GPT-OSS

use serde::{Deserialize, Serialize};

/// Sampling configuration for chunk generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingConfig {
    /// Temperature for probability distribution (higher = more creative)
    pub temperature: f32,

    /// Top-k sampling: keep only top k tokens (None = disabled)
    pub top_k: Option<usize>,

    /// Top-p (nucleus) sampling: keep tokens until cumulative prob reaches p
    pub top_p: Option<f32>,

    /// Repetition penalty factor (> 1.0 = discourage repetition)
    pub repetition_penalty: f32,

    /// Minimum probability threshold for token consideration
    pub min_p: f32,
}

impl SamplingConfig {
    /// Conservative sampling: low temperature, high quality
    /// Best for: Math problems, code generation requiring correctness
    pub fn conservative() -> Self {
        Self {
            temperature: 0.3,
            top_k: Some(10),
            top_p: Some(0.7),
            repetition_penalty: 1.1,
            min_p: 0.01,
        }
    }

    /// Creative sampling: higher temperature, exploration
    /// Best for: Open-ended reasoning, brainstorming
    pub fn creative() -> Self {
        Self {
            temperature: 0.9,
            top_k: Some(50),
            top_p: Some(0.95),
            repetition_penalty: 1.05,
            min_p: 0.001,
        }
    }

    /// Balanced sampling: default middle ground
    /// Best for: General reasoning tasks
    pub fn balanced() -> Self {
        Self {
            temperature: 0.7,
            top_k: Some(40),
            top_p: Some(0.9),
            repetition_penalty: 1.1,
            min_p: 0.005,
        }
    }

    /// Domain-adaptive sampling based on problem type
    pub fn for_domain(domain: &str) -> Self {
        match domain.to_lowercase().as_str() {
            "math" | "mathematics" | "calculation" => Self::conservative(),
            "code" | "programming" | "algorithm" => Self::conservative(),
            "creative" | "writing" | "brainstorm" => Self::creative(),
            _ => Self::balanced(),
        }
    }
}

impl Default for SamplingConfig {
    fn default() -> Self {
        Self::balanced()
    }
}

/// Token with probability for sampling
#[derive(Debug, Clone, PartialEq)]
pub struct TokenProb {
    pub token_id: usize,
    pub token_text: String,
    pub probability: f32,
    pub log_prob: f32,
}

/// Sampling strategy implementation
pub struct SamplingStrategy {
    config: SamplingConfig,
}

impl SamplingStrategy {
    pub fn new(config: SamplingConfig) -> Self {
        Self { config }
    }

    /// Apply temperature scaling to logits
    pub fn apply_temperature(&self, logits: &[f32]) -> Vec<f32> {
        if (self.config.temperature - 1.0).abs() < 1e-6 {
            return logits.to_vec();
        }

        logits
            .iter()
            .map(|&logit| logit / self.config.temperature)
            .collect()
    }

    /// Convert logits to probabilities via softmax
    pub fn softmax(&self, logits: &[f32]) -> Vec<f32> {
        let max_logit = logits
            .iter()
            .cloned()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        // Subtract max for numerical stability
        let exp_logits: Vec<f32> = logits
            .iter()
            .map(|&logit| (logit - max_logit).exp())
            .collect();

        let sum: f32 = exp_logits.iter().sum();

        exp_logits.iter().map(|&e| e / sum).collect()
    }

    /// Apply top-k filtering
    pub fn apply_top_k(&self, mut probs: Vec<TokenProb>) -> Vec<TokenProb> {
        if let Some(k) = self.config.top_k {
            if k < probs.len() {
                // Sort by probability descending
                probs.sort_by(|a, b| b.probability.partial_cmp(&a.probability).unwrap());

                // Keep top-k
                probs.truncate(k);
            }
        }

        probs
    }

    /// Apply top-p (nucleus) filtering
    pub fn apply_top_p(&self, mut probs: Vec<TokenProb>) -> Vec<TokenProb> {
        if let Some(p) = self.config.top_p {
            // Sort by probability descending
            probs.sort_by(|a, b| b.probability.partial_cmp(&a.probability).unwrap());

            let mut cumulative = 0.0;
            let mut cutoff_index = probs.len();

            for (i, token_prob) in probs.iter().enumerate() {
                cumulative += token_prob.probability;
                if cumulative >= p {
                    cutoff_index = i + 1;
                    break;
                }
            }

            probs.truncate(cutoff_index);
        }

        probs
    }

    /// Apply minimum probability filtering
    pub fn apply_min_p(&self, probs: Vec<TokenProb>) -> Vec<TokenProb> {
        probs
            .into_iter()
            .filter(|tp| tp.probability >= self.config.min_p)
            .collect()
    }

    /// Apply repetition penalty
    pub fn apply_repetition_penalty(&self, logits: &[f32], previous_tokens: &[usize]) -> Vec<f32> {
        if (self.config.repetition_penalty - 1.0).abs() < 1e-6 {
            return logits.to_vec();
        }

        let mut penalized = logits.to_vec();

        for &token_id in previous_tokens {
            if token_id < penalized.len() {
                if penalized[token_id] > 0.0 {
                    penalized[token_id] /= self.config.repetition_penalty;
                } else {
                    penalized[token_id] *= self.config.repetition_penalty;
                }
            }
        }

        penalized
    }

    /// Full sampling pipeline
    pub fn sample(
        &self,
        logits: &[f32],
        previous_tokens: &[usize],
    ) -> Vec<TokenProb> {
        // 1. Apply repetition penalty
        let penalized = self.apply_repetition_penalty(logits, previous_tokens);

        // 2. Apply temperature
        let scaled = self.apply_temperature(&penalized);

        // 3. Convert to probabilities
        let probabilities = self.softmax(&scaled);

        // 4. Create TokenProb structs
        let mut token_probs: Vec<TokenProb> = probabilities
            .into_iter()
            .enumerate()
            .map(|(id, prob)| TokenProb {
                token_id: id,
                token_text: format!("token_{}", id), // Placeholder
                probability: prob,
                log_prob: prob.ln(),
            })
            .collect();

        // 5. Apply top-k filtering
        token_probs = self.apply_top_k(token_probs);

        // 6. Apply top-p filtering
        token_probs = self.apply_top_p(token_probs);

        // 7. Apply min_p filtering
        token_probs = self.apply_min_p(token_probs);

        // 8. Renormalize probabilities
        let total: f32 = token_probs.iter().map(|tp| tp.probability).sum();
        if total > 0.0 {
            for tp in &mut token_probs {
                tp.probability /= total;
                tp.log_prob = tp.probability.ln();
            }
        }

        token_probs
    }

    /// Get configuration
    pub fn config(&self) -> &SamplingConfig {
        &self.config
    }
}

/// Sampling strategy selector based on context
pub struct AdaptiveSampling {
    conservative: SamplingStrategy,
    creative: SamplingStrategy,
    balanced: SamplingStrategy,
}

impl AdaptiveSampling {
    pub fn new() -> Self {
        Self {
            conservative: SamplingStrategy::new(SamplingConfig::conservative()),
            creative: SamplingStrategy::new(SamplingConfig::creative()),
            balanced: SamplingStrategy::new(SamplingConfig::balanced()),
        }
    }

    /// Select appropriate strategy based on problem characteristics
    pub fn select_strategy(&self, problem: &str) -> &SamplingStrategy {
        let problem_lower = problem.to_lowercase();

        // Conservative for math/code
        if problem_lower.contains("calculate")
            || problem_lower.contains("equation")
            || problem_lower.contains("code")
            || problem_lower.contains("algorithm")
            || problem_lower.contains("prove")
        {
            return &self.conservative;
        }

        // Creative for open-ended
        if problem_lower.contains("explain")
            || problem_lower.contains("describe")
            || problem_lower.contains("brainstorm")
            || problem_lower.contains("creative")
        {
            return &self.creative;
        }

        // Balanced for everything else
        &self.balanced
    }
}

impl Default for AdaptiveSampling {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sampling_config_presets() {
        let conservative = SamplingConfig::conservative();
        assert!(conservative.temperature < 0.5);

        let creative = SamplingConfig::creative();
        assert!(creative.temperature > 0.7);

        let balanced = SamplingConfig::balanced();
        assert!(balanced.temperature > 0.5 && balanced.temperature < 0.8);
    }

    #[test]
    fn test_domain_adaptive_config() {
        let math_config = SamplingConfig::for_domain("math");
        assert_eq!(math_config.temperature, 0.3);

        let creative_config = SamplingConfig::for_domain("creative");
        assert!(creative_config.temperature > 0.7);
    }

    #[test]
    fn test_temperature_scaling() {
        let config = SamplingConfig {
            temperature: 0.5,
            ..Default::default()
        };
        let strategy = SamplingStrategy::new(config);

        let logits = vec![1.0, 2.0, 3.0];
        let scaled = strategy.apply_temperature(&logits);

        assert_eq!(scaled[0], 2.0);
        assert_eq!(scaled[1], 4.0);
        assert_eq!(scaled[2], 6.0);
    }

    #[test]
    fn test_softmax() {
        let config = SamplingConfig::default();
        let strategy = SamplingStrategy::new(config);

        let logits = vec![1.0, 2.0, 3.0];
        let probs = strategy.softmax(&logits);

        // Probabilities should sum to ~1.0
        let sum: f32 = probs.iter().sum();
        assert!((sum - 1.0).abs() < 1e-5);

        // Higher logit should have higher probability
        assert!(probs[2] > probs[1]);
        assert!(probs[1] > probs[0]);
    }

    #[test]
    fn test_top_k_filtering() {
        let config = SamplingConfig {
            top_k: Some(2),
            ..Default::default()
        };
        let strategy = SamplingStrategy::new(config);

        let probs = vec![
            TokenProb {
                token_id: 0,
                token_text: "a".to_string(),
                probability: 0.5,
                log_prob: 0.5_f32.ln(),
            },
            TokenProb {
                token_id: 1,
                token_text: "b".to_string(),
                probability: 0.3,
                log_prob: 0.3_f32.ln(),
            },
            TokenProb {
                token_id: 2,
                token_text: "c".to_string(),
                probability: 0.2,
                log_prob: 0.2_f32.ln(),
            },
        ];

        let filtered = strategy.apply_top_k(probs);
        assert_eq!(filtered.len(), 2);
        assert!(filtered[0].probability >= filtered[1].probability);
    }

    #[test]
    fn test_top_p_filtering() {
        let config = SamplingConfig {
            top_p: Some(0.8),
            ..Default::default()
        };
        let strategy = SamplingStrategy::new(config);

        let probs = vec![
            TokenProb {
                token_id: 0,
                token_text: "a".to_string(),
                probability: 0.5,
                log_prob: 0.5_f32.ln(),
            },
            TokenProb {
                token_id: 1,
                token_text: "b".to_string(),
                probability: 0.3,
                log_prob: 0.3_f32.ln(),
            },
            TokenProb {
                token_id: 2,
                token_text: "c".to_string(),
                probability: 0.2,
                log_prob: 0.2_f32.ln(),
            },
        ];

        let filtered = strategy.apply_top_p(probs);

        // Should keep tokens until cumulative >= 0.8
        // 0.5 + 0.3 = 0.8, so should keep first 2 tokens
        assert!(filtered.len() <= 2);
    }

    #[test]
    fn test_repetition_penalty() {
        let config = SamplingConfig {
            repetition_penalty: 2.0,
            ..Default::default()
        };
        let strategy = SamplingStrategy::new(config);

        let logits = vec![1.0, 2.0, 3.0];
        let previous_tokens = vec![1]; // Token 1 was used before

        let penalized = strategy.apply_repetition_penalty(&logits, &previous_tokens);

        // Token 1 should be penalized
        assert_eq!(penalized[1], 1.0); // 2.0 / 2.0
        assert_eq!(penalized[0], 1.0); // Unchanged
        assert_eq!(penalized[2], 3.0); // Unchanged
    }

    #[test]
    fn test_adaptive_sampling() {
        let adaptive = AdaptiveSampling::new();

        let math_problem = "Calculate the derivative";
        let strategy = adaptive.select_strategy(math_problem);
        assert!(strategy.config().temperature < 0.5);

        let creative_problem = "Explain the concept creatively";
        let strategy = adaptive.select_strategy(creative_problem);
        assert!(strategy.config().temperature > 0.7);
    }
}
