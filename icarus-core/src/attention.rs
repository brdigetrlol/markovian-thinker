// Markovian Thinker: Attention Mechanisms
// Sliding window attention and attention sink inspired by GPT-OSS

use serde::{Deserialize, Serialize};

/// Configuration for attention mechanisms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionConfig {
    /// Sliding window size in tokens (None = disabled)
    pub sliding_window_size: Option<usize>,
    /// Enable attention sink for hallucination filtering
    pub attention_sink_enabled: bool,
    /// Decay factor for temporal attention (0.0-1.0)
    pub decay_factor: f32,
    /// Minimum attention score to keep (0.0-1.0)
    pub min_attention_score: f32,
}

impl Default for AttentionConfig {
    fn default() -> Self {
        Self {
            sliding_window_size: Some(256), // 256-token window like GPT-OSS
            attention_sink_enabled: true,
            decay_factor: 0.95, // Slight decay favoring recent content
            min_attention_score: 0.1,
        }
    }
}

/// Token with attention metadata
#[derive(Debug, Clone)]
pub struct AttentionToken {
    pub text: String,
    pub position: usize,
    pub score: f32,
}

impl AttentionToken {
    pub fn new(text: String, position: usize, score: f32) -> Self {
        Self { text, position, score }
    }
}

/// Sliding window attention for carryover selection
pub struct SlidingWindowAttention {
    config: AttentionConfig,
}

impl SlidingWindowAttention {
    pub fn new(config: AttentionConfig) -> Self {
        Self { config }
    }

    /// Score tokens based on recency, semantic importance, and patterns
    pub fn score_tokens(&self, text: &str) -> Vec<AttentionToken> {
        // Approximate tokenization (split on whitespace and punctuation)
        let words: Vec<&str> = text
            .split(|c: char| c.is_whitespace() || c == '.' || c == ',' || c == ';')
            .filter(|w| !w.is_empty())
            .collect();

        let total_tokens = words.len();
        let mut tokens = Vec::new();

        for (i, word) in words.iter().enumerate() {
            // Base score from position (recent = higher)
            let position_score = self.temporal_score(i, total_tokens);

            // Semantic importance boost
            let semantic_score = self.semantic_importance(word);

            // Pattern matching boost
            let pattern_score = self.pattern_importance(word, text);

            // Combine scores
            let combined_score = position_score * 0.4 + semantic_score * 0.3 + pattern_score * 0.3;

            tokens.push(AttentionToken {
                text: word.to_string(),
                position: i,
                score: combined_score.min(1.0),
            });
        }

        tokens
    }

    /// Temporal attention score with exponential decay
    fn temporal_score(&self, position: usize, total: usize) -> f32 {
        if total == 0 {
            return 1.0;
        }

        // Distance from end (0 = most recent)
        let distance = total - position - 1;

        // Exponential decay: score = decay_factor ^ distance
        self.config.decay_factor.powi(distance as i32)
    }

    /// Score semantic importance based on keywords and markers
    fn semantic_importance(&self, word: &str) -> f32 {
        let word_lower = word.to_lowercase();

        // High importance markers
        let high_importance = [
            "therefore", "thus", "hence", "answer", "solution",
            "conclusion", "result", "final", "boxed", "eos"
        ];

        if high_importance.iter().any(|k| word_lower.contains(k)) {
            return 1.0;
        }

        // Medium importance markers
        let medium_importance = [
            "calculate", "equation", "prove", "show", "derive",
            "step", "next", "then", "because"
        ];

        if medium_importance.iter().any(|k| word_lower.contains(k)) {
            return 0.7;
        }

        // Mathematical symbols
        if word.chars().any(|c| "=+-×÷∑∫√".contains(c)) {
            return 0.8;
        }

        // Default importance
        0.3
    }

    /// Score pattern importance (equations, code, etc.)
    fn pattern_importance(&self, word: &str, full_text: &str) -> f32 {
        let mut score: f32 = 0.0;

        // LaTeX patterns
        if word.starts_with('\\') || full_text.contains("\\boxed") {
            score += 0.5;
        }

        // Code patterns
        if word.contains("```") || word.contains("def ") || word.contains("function") {
            score += 0.4;
        }

        // Numbers and equations
        if word.chars().any(|c| c.is_numeric()) {
            score += 0.3;
        }

        score.min(1.0)
    }

    /// Select important tokens within sliding window
    pub fn select_important(&self, text: &str, target_size: usize) -> String {
        let tokens = self.score_tokens(text);

        // Apply sliding window if configured
        let windowed_tokens = if let Some(window_size) = self.config.sliding_window_size {
            let start = tokens.len().saturating_sub(window_size);
            &tokens[start..]
        } else {
            &tokens[..]
        };

        // Sort by score descending
        let mut scored: Vec<_> = windowed_tokens.to_vec();
        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        // Select top tokens up to target size
        let approx_chars = target_size * 4; // 1 token ≈ 4 chars
        let mut selected = Vec::new();
        let mut char_count = 0;

        for token in scored {
            if token.score < self.config.min_attention_score {
                continue;
            }

            if char_count + token.text.len() <= approx_chars || selected.is_empty() {
                char_count += token.text.len() + 1; // +1 for space
                selected.push(token.clone());
            } else {
                break;
            }
        }

        // Re-sort by position to maintain order
        selected.sort_by_key(|t| t.position);

        // Reconstruct text
        selected
            .iter()
            .map(|t| t.text.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Get configuration
    pub fn config(&self) -> &AttentionConfig {
        &self.config
    }
}

/// Attention sink for filtering hallucinations
pub struct AttentionSink {
    config: AttentionConfig,
}

impl AttentionSink {
    pub fn new(config: AttentionConfig) -> Self {
        Self { config }
    }

    /// Filter out low-relevance content that might be hallucinations
    pub fn filter_hallucinations(&self, text: &str, problem: &str) -> String {
        if !self.config.attention_sink_enabled {
            return text.to_string();
        }

        let paragraphs: Vec<&str> = text.split("\n\n").collect();
        let mut filtered = Vec::new();

        for para in &paragraphs {
            let relevance_score = self.compute_relevance(para, problem);

            if relevance_score >= self.config.min_attention_score {
                filtered.push(*para);
            } else {
                tracing::debug!(
                    "Attention sink: filtered paragraph (score: {:.2}): {}...",
                    relevance_score,
                    &para.chars().take(50).collect::<String>()
                );
            }
        }

        if filtered.is_empty() {
            // Keep at least the last paragraph
            if let Some(last) = paragraphs.last() {
                return last.to_string();
            }
        }

        filtered.join("\n\n")
    }

    /// Compute relevance score of content to the original problem
    fn compute_relevance(&self, content: &str, problem: &str) -> f32 {
        let mut score = 0.0;

        // Extract keywords from problem
        let problem_keywords: Vec<&str> = problem
            .split_whitespace()
            .filter(|w| w.len() > 3) // Skip short words
            .collect();

        // Check keyword overlap
        let content_lower = content.to_lowercase();
        let matches = problem_keywords
            .iter()
            .filter(|k| content_lower.contains(&k.to_lowercase()))
            .count();

        score += (matches as f32 / problem_keywords.len().max(1) as f32) * 0.5;

        // Check for continuation markers
        let continuation_markers = [
            "therefore", "thus", "next", "then", "so", "because",
            "step", "calculate", "solve", "answer"
        ];

        let has_continuation = continuation_markers
            .iter()
            .any(|m| content_lower.contains(m));

        if has_continuation {
            score += 0.3;
        }

        // Check for coherence (has numbers, equations, or structured content)
        let has_structure = content.chars().any(|c| c.is_numeric())
            || content.contains("=")
            || content.contains("```")
            || content.contains("\\boxed");

        if has_structure {
            score += 0.2;
        }

        score.min(1.0)
    }

    /// Get configuration
    pub fn config(&self) -> &AttentionConfig {
        &self.config
    }
}

/// Grouped attention for pattern clustering
pub struct GroupedAttention {
    group_size: usize,
}

impl GroupedAttention {
    pub fn new(group_size: usize) -> Self {
        Self { group_size }
    }

    /// Group similar tokens/patterns together for efficient processing
    pub fn group_patterns<'a>(&self, tokens: &'a [AttentionToken]) -> Vec<Vec<&'a AttentionToken>> {
        let mut groups: Vec<Vec<&AttentionToken>> = Vec::new();
        let mut current_group = Vec::new();

        for token in tokens {
            current_group.push(token);

            if current_group.len() >= self.group_size {
                groups.push(current_group);
                current_group = Vec::new();
            }
        }

        if !current_group.is_empty() {
            groups.push(current_group);
        }

        groups
    }

    /// Compute group attention score (max or mean of group)
    pub fn group_score(&self, group: &[&AttentionToken]) -> f32 {
        if group.is_empty() {
            return 0.0;
        }

        // Use max score in group (represents most important token)
        group
            .iter()
            .map(|t| t.score)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sliding_window_scoring() {
        let config = AttentionConfig::default();
        let attention = SlidingWindowAttention::new(config);

        let text = "First step is to solve. Then calculate the answer. Therefore the result is 42.";
        let tokens = attention.score_tokens(text);

        // Check that important words get higher scores
        let answer_token = tokens.iter().find(|t| t.text.contains("answer"));
        let first_token = tokens.iter().find(|t| t.text == "First");

        assert!(answer_token.is_some());
        assert!(first_token.is_some());

        // Recent and semantically important should score higher
        assert!(answer_token.unwrap().score > first_token.unwrap().score);
    }

    #[test]
    fn test_temporal_decay() {
        let config = AttentionConfig {
            decay_factor: 0.9,
            ..Default::default()
        };
        let attention = SlidingWindowAttention::new(config);

        // Recent token should score higher
        let score_recent = attention.temporal_score(99, 100);
        let score_old = attention.temporal_score(0, 100);

        assert!(score_recent > score_old);
        assert!(score_recent > 0.9);
        assert!(score_old < 0.1);
    }

    #[test]
    fn test_semantic_importance() {
        let config = AttentionConfig::default();
        let attention = SlidingWindowAttention::new(config);

        assert_eq!(attention.semantic_importance("therefore"), 1.0);
        assert_eq!(attention.semantic_importance("calculate"), 0.7);
        assert!(attention.semantic_importance("hello") < 0.5);
    }

    #[test]
    fn test_select_important() {
        let config = AttentionConfig::default();
        let attention = SlidingWindowAttention::new(config);

        let text = "Random words here. Calculate the answer. Therefore result is 42.";
        let selected = attention.select_important(text, 10);

        // Should prioritize important keywords
        assert!(selected.contains("answer") || selected.contains("42"));
    }

    #[test]
    fn test_attention_sink_filtering() {
        let config = AttentionConfig::default();
        let sink = AttentionSink::new(config);

        let problem = "Calculate 2+2";
        let text = "First calculate 2+2.\n\nRandom hallucination text here.\n\nThe answer is 4.";

        let filtered = sink.filter_hallucinations(text, problem);

        // Should keep relevant paragraphs
        assert!(filtered.contains("calculate"));
        assert!(filtered.contains("answer"));

        // Might filter out hallucination depending on threshold
        // (this is a weak test, just checking it doesn't crash)
        assert!(!filtered.is_empty());
    }

    #[test]
    fn test_relevance_computation() {
        let config = AttentionConfig::default();
        let sink = AttentionSink::new(config);

        let problem = "Solve the quadratic equation";
        let relevant = "To solve the equation, we use the quadratic formula";
        let irrelevant = "The weather is nice today";

        let score_relevant = sink.compute_relevance(relevant, problem);
        let score_irrelevant = sink.compute_relevance(irrelevant, problem);

        assert!(score_relevant > score_irrelevant);
        assert!(score_relevant > 0.5);
    }

    #[test]
    fn test_grouped_attention() {
        let grouped = GroupedAttention::new(4);

        let tokens = vec![
            AttentionToken {
                text: "a".to_string(),
                position: 0,
                score: 0.5,
            },
            AttentionToken {
                text: "b".to_string(),
                position: 1,
                score: 0.8,
            },
            AttentionToken {
                text: "c".to_string(),
                position: 2,
                score: 0.3,
            },
        ];

        let groups = grouped.group_patterns(&tokens);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].len(), 3);

        let score = grouped.group_score(&groups[0]);
        assert_eq!(score, 0.8); // Max score in group
    }
}
