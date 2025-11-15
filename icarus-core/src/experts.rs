// Markovian Thinker: Mixture of Experts
// Domain-specific reasoning strategies inspired by GPT-OSS MoE architecture

use serde::{Deserialize, Serialize};

/// Configuration for the expert gating mechanism
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpertConfig {
    /// Enable mixture of experts
    pub enabled: bool,
    /// Number of experts to activate (top-k selection)
    pub top_k_experts: usize,
    /// Minimum gating score threshold (0.0-1.0)
    pub gating_threshold: f32,
}

impl Default for ExpertConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            top_k_experts: 2,
            gating_threshold: 0.1,
        }
    }
}

/// Expert types for different reasoning domains
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpertType {
    MathReasoning,
    CodeGeneration,
    TextualReasoning,
    VisualReasoning,
    Mixed,
}

/// Trait for domain-specific reasoning experts
pub trait ReasoningExpert: Send + Sync {
    /// Score relevance of this expert to the given problem (0.0-1.0)
    fn score_relevance(&self, problem: &str, context: Option<&str>) -> f32;

    /// Extract carryover from text using domain-specific heuristics
    fn extract_carryover(&self, text: &str, target_size: usize) -> String;

    /// Detect termination markers specific to this domain
    fn detect_termination(&self, text: &str) -> bool;

    /// Extract solution using domain-specific patterns
    fn extract_solution(&self, text: &str) -> Option<String>;

    /// Get expert type
    fn expert_type(&self) -> ExpertType;

    /// Get expert name for logging
    fn name(&self) -> &'static str;
}

/// Mathematical reasoning expert
pub struct MathExpert;

impl MathExpert {
    pub fn new() -> Self {
        Self
    }

    /// Check if text contains mathematical content
    fn has_math_content(text: &str) -> bool {
        // LaTeX math markers
        let latex_markers = ["\\frac", "\\sum", "\\int", "\\sqrt", "\\boxed", "$$", "$"];
        // Math symbols
        let math_symbols = ["=", "+", "-", "×", "÷", "∑", "∫", "√"];

        latex_markers.iter().any(|m| text.contains(m))
            || math_symbols.iter().any(|s| text.contains(s))
            || text.contains("equation")
            || text.contains("calculate")
    }
}

impl ReasoningExpert for MathExpert {
    fn score_relevance(&self, problem: &str, context: Option<&str>) -> f32 {
        let mut score = 0.0;

        // Check problem text
        if Self::has_math_content(problem) {
            score += 0.5;
        }

        // Math keywords
        let math_keywords = [
            "solve", "calculate", "compute", "equation", "formula",
            "sum", "product", "integral", "derivative", "proof"
        ];
        let keyword_count = math_keywords.iter()
            .filter(|k| problem.to_lowercase().contains(*k))
            .count();
        score += (keyword_count as f32 * 0.1).min(0.3);

        // Check context if available
        if let Some(ctx) = context {
            if Self::has_math_content(ctx) {
                score += 0.2;
            }
        }

        score.min(1.0)
    }

    fn extract_carryover(&self, text: &str, target_size: usize) -> String {
        // For math: prioritize equations and final steps
        let lines: Vec<&str> = text.lines().collect();
        let mut important_lines = Vec::new();
        let mut char_count = 0;
        let approx_chars = target_size * 4; // 1 token ≈ 4 chars

        // Scan backwards for equations and important content
        for line in lines.iter().rev() {
            let trimmed = line.trim();

            // Prioritize lines with equations, boxed answers, or markers
            let is_important = trimmed.contains("=")
                || trimmed.starts_with("\\boxed")
                || trimmed.contains("$$")
                || trimmed.contains("Therefore")
                || trimmed.contains("Thus")
                || trimmed.starts_with("####");

            if is_important || char_count < approx_chars {
                important_lines.push(*line);
                char_count += line.len();

                if char_count >= approx_chars && is_important {
                    break;
                }
            }
        }

        important_lines.reverse();
        important_lines.join("\n").trim().to_string()
    }

    fn detect_termination(&self, text: &str) -> bool {
        // Math-specific termination markers
        text.contains("\\boxed{")
            || text.contains("[EOS]")
            || text.contains("[SOLUTION]")
            || text.contains("#### ")
            || (text.contains("Therefore") && text.contains("="))
    }

    fn extract_solution(&self, text: &str) -> Option<String> {
        // Try LaTeX boxed answer first
        if let Some(start) = text.rfind("\\boxed{") {
            let after_boxed = &text[start + 7..];
            let mut depth = 1;
            let mut end = 0;

            for (i, c) in after_boxed.chars().enumerate() {
                match c {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            end = i;
                            break;
                        }
                    }
                    _ => {}
                }
            }

            if end > 0 {
                return Some(after_boxed[..end].trim().to_string());
            }
        }

        // Try #### format
        if let Some(start) = text.rfind("#### ") {
            if let Some(answer) = text[start + 5..].lines().next() {
                return Some(answer.trim().to_string());
            }
        }

        None
    }

    fn expert_type(&self) -> ExpertType {
        ExpertType::MathReasoning
    }

    fn name(&self) -> &'static str {
        "MathExpert"
    }
}

/// Code generation expert
pub struct CodeExpert;

impl CodeExpert {
    pub fn new() -> Self {
        Self
    }

    fn has_code_content(text: &str) -> bool {
        text.contains("```")
            || text.contains("def ")
            || text.contains("function ")
            || text.contains("class ")
            || text.contains("import ")
            || text.contains("const ")
            || text.contains("let ")
    }
}

impl ReasoningExpert for CodeExpert {
    fn score_relevance(&self, problem: &str, context: Option<&str>) -> f32 {
        let mut score = 0.0;

        if Self::has_code_content(problem) {
            score += 0.5;
        }

        let code_keywords = [
            "code", "program", "function", "algorithm", "implement",
            "debug", "syntax", "compile", "runtime"
        ];
        let keyword_count = code_keywords.iter()
            .filter(|k| problem.to_lowercase().contains(*k))
            .count();
        score += (keyword_count as f32 * 0.1).min(0.3);

        if let Some(ctx) = context {
            if Self::has_code_content(ctx) {
                score += 0.2;
            }
        }

        score.min(1.0)
    }

    fn extract_carryover(&self, text: &str, target_size: usize) -> String {
        // For code: preserve complete code blocks and recent context
        let approx_chars = target_size * 4;

        // Find last complete code block if exists
        if let Some(last_code_end) = text.rfind("```") {
            let before_end = &text[..last_code_end];
            if let Some(last_code_start) = before_end.rfind("```") {
                let code_block = &text[last_code_start..=last_code_end + 2];

                // If code block fits, include it plus some context
                if code_block.len() <= approx_chars {
                    let remaining = approx_chars - code_block.len();
                    let context_start = last_code_start.saturating_sub(remaining);
                    return text[context_start..].trim().to_string();
                }
            }
        }

        // Fallback: standard carryover
        if text.len() <= approx_chars {
            text.to_string()
        } else {
            let start_pos = text.len() - approx_chars;
            text[start_pos..].trim().to_string()
        }
    }

    fn detect_termination(&self, text: &str) -> bool {
        text.contains("[EOS]")
            || text.contains("[DONE]")
            || text.contains("[SOLUTION]")
            || (text.contains("```") && text.matches("```").count() >= 2)
    }

    fn extract_solution(&self, text: &str) -> Option<String> {
        // Extract last complete code block
        if let Some(last_end) = text.rfind("```") {
            let before_end = &text[..last_end];
            if let Some(last_start) = before_end.rfind("```") {
                let code = &text[last_start + 3..last_end];
                // Remove language specifier if present
                let code_lines: Vec<&str> = code.lines().collect();
                if code_lines.len() > 1 {
                    return Some(code_lines[1..].join("\n").trim().to_string());
                }
            }
        }

        None
    }

    fn expert_type(&self) -> ExpertType {
        ExpertType::CodeGeneration
    }

    fn name(&self) -> &'static str {
        "CodeExpert"
    }
}

/// General textual reasoning expert
pub struct TextExpert;

impl TextExpert {
    pub fn new() -> Self {
        Self
    }
}

impl ReasoningExpert for TextExpert {
    fn score_relevance(&self, problem: &str, _context: Option<&str>) -> f32 {
        // Text expert is a fallback - scores moderately for all content
        let text_keywords = [
            "explain", "describe", "analyze", "compare", "discuss",
            "what", "why", "how", "when", "where"
        ];

        let keyword_count = text_keywords.iter()
            .filter(|k| problem.to_lowercase().contains(*k))
            .count();

        0.3 + (keyword_count as f32 * 0.05).min(0.3)
    }

    fn extract_carryover(&self, text: &str, target_size: usize) -> String {
        // For text: preserve complete sentences and paragraphs
        let approx_chars = target_size * 4;

        if text.len() <= approx_chars {
            return text.to_string();
        }

        // Find paragraph or sentence boundary
        let start_pos = text.len() - approx_chars;
        let search_slice = &text[start_pos..];

        // Look for paragraph break
        if let Some(para_break) = search_slice.find("\n\n") {
            return text[start_pos + para_break..].trim().to_string();
        }

        // Look for sentence break
        if let Some(sent_break) = search_slice.find(". ") {
            return text[start_pos + sent_break + 2..].trim().to_string();
        }

        // Fallback: word boundary
        if let Some(word_break) = search_slice.find(' ') {
            return text[start_pos + word_break..].trim().to_string();
        }

        search_slice.to_string()
    }

    fn detect_termination(&self, text: &str) -> bool {
        text.contains("[EOS]")
            || text.contains("[DONE]")
            || text.contains("[SOLUTION]")
            || text.contains("In conclusion")
            || text.contains("To summarize")
    }

    fn extract_solution(&self, text: &str) -> Option<String> {
        // Look for conclusion markers
        for marker in &["[SOLUTION]", "In conclusion", "To summarize", "Therefore"] {
            if let Some(pos) = text.rfind(marker) {
                let after_marker = &text[pos + marker.len()..].trim();
                if !after_marker.is_empty() {
                    // Take first paragraph after marker
                    let solution = after_marker
                        .split("\n\n")
                        .next()
                        .unwrap_or(after_marker);
                    return Some(solution.trim().to_string());
                }
            }
        }

        None
    }

    fn expert_type(&self) -> ExpertType {
        ExpertType::TextualReasoning
    }

    fn name(&self) -> &'static str {
        "TextExpert"
    }
}

/// Expert gating mechanism - selects and combines multiple experts
pub struct ExpertGating {
    experts: Vec<Box<dyn ReasoningExpert>>,
    config: ExpertConfig,
}

impl ExpertGating {
    /// Create new gating system with default experts
    pub fn new(config: ExpertConfig) -> Self {
        let experts: Vec<Box<dyn ReasoningExpert>> = vec![
            Box::new(MathExpert::new()),
            Box::new(CodeExpert::new()),
            Box::new(TextExpert::new()),
        ];

        Self { experts, config }
    }

    /// Select top-k experts based on relevance scores
    pub fn select_experts(&self, problem: &str, context: Option<&str>) -> Vec<&dyn ReasoningExpert> {
        if !self.config.enabled {
            // Return text expert as fallback
            return vec![self.experts[2].as_ref()];
        }

        let mut scored_experts: Vec<(f32, &dyn ReasoningExpert)> = self.experts
            .iter()
            .map(|expert| {
                let score = expert.score_relevance(problem, context);
                (score, expert.as_ref())
            })
            .filter(|(score, _)| *score >= self.config.gating_threshold)
            .collect();

        // Sort by score descending
        scored_experts.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        // Take top-k
        scored_experts
            .into_iter()
            .take(self.config.top_k_experts)
            .map(|(_, expert)| expert)
            .collect()
    }

    /// Extract carryover using selected experts (weighted combination)
    pub fn extract_carryover(&self, text: &str, target_size: usize, problem: &str) -> String {
        let experts = self.select_experts(problem, Some(text));

        if experts.is_empty() {
            // Fallback to simple extraction
            return Self::simple_carryover(text, target_size);
        }

        // Use highest-scoring expert's strategy
        experts[0].extract_carryover(text, target_size)
    }

    /// Detect termination using any active expert
    pub fn detect_termination(&self, text: &str, problem: &str) -> bool {
        let experts = self.select_experts(problem, Some(text));
        experts.iter().any(|expert| expert.detect_termination(text))
    }

    /// Extract solution using active experts
    pub fn extract_solution(&self, text: &str, problem: &str) -> Option<String> {
        let experts = self.select_experts(problem, Some(text));

        for expert in experts {
            if let Some(solution) = expert.extract_solution(text) {
                return Some(solution);
            }
        }

        None
    }

    /// Simple carryover fallback
    fn simple_carryover(text: &str, target_size: usize) -> String {
        let approx_chars = target_size * 4;
        if text.len() <= approx_chars {
            text.to_string()
        } else {
            let start_pos = text.len() - approx_chars;
            text[start_pos..].trim().to_string()
        }
    }

    /// Get active expert names for logging
    pub fn get_active_experts(&self, problem: &str, context: Option<&str>) -> Vec<String> {
        self.select_experts(problem, context)
            .iter()
            .map(|e| e.name().to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_math_expert_relevance() {
        let expert = MathExpert::new();

        let math_problem = "Solve the equation: 2x + 5 = 15";
        assert!(expert.score_relevance(math_problem, None) > 0.5);

        let non_math = "What is the capital of France?";
        assert!(expert.score_relevance(non_math, None) < 0.3);
    }

    #[test]
    fn test_math_expert_boxed_extraction() {
        let expert = MathExpert::new();
        let text = "After calculation, the answer is \\boxed{42}.";

        assert_eq!(expert.extract_solution(text), Some("42".to_string()));
    }

    #[test]
    fn test_code_expert_relevance() {
        let expert = CodeExpert::new();

        let code_problem = "Write a function to reverse a string";
        assert!(expert.score_relevance(code_problem, None) > 0.3);

        let code_context = "```python\ndef hello():\n    pass\n```";
        assert!(expert.score_relevance("Test", Some(code_context)) >= 0.2);
    }

    #[test]
    fn test_code_expert_block_extraction() {
        let expert = CodeExpert::new();
        let text = "Here's the solution:\n```python\ndef reverse(s):\n    return s[::-1]\n```";

        let solution = expert.extract_solution(text);
        assert!(solution.is_some());
        assert!(solution.unwrap().contains("return s[::-1]"));
    }

    #[test]
    fn test_expert_gating() {
        let config = ExpertConfig::default();
        let gating = ExpertGating::new(config);

        let math_problem = "Calculate the sum: 1 + 2 + 3 + 4";
        let experts = gating.select_experts(math_problem, None);

        assert!(!experts.is_empty());
        assert_eq!(experts[0].name(), "MathExpert");
    }

    #[test]
    fn test_expert_gating_fallback() {
        let mut config = ExpertConfig::default();
        config.enabled = false;

        let gating = ExpertGating::new(config);
        let experts = gating.select_experts("Any problem", None);

        assert_eq!(experts.len(), 1);
        assert_eq!(experts[0].name(), "TextExpert");
    }

    #[test]
    fn test_text_expert_carryover() {
        let expert = TextExpert::new();
        let text = "First sentence. Second sentence. Third sentence. Fourth sentence.";

        let carryover = expert.extract_carryover(text, 10); // Small size
        assert!(carryover.contains("sentence"));
        assert!(carryover.len() < text.len());
    }
}
