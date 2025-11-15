// Enhanced types for CRV-inspired verification system
// Provides domain detection, verification results, and reasoning metadata

use serde::{Deserialize, Serialize};

/// Reasoning domain detection for specialized strategies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningDomain {
    Debugging,
    Architecture,
    Mathematical,
    Logical,
    General,
}

impl ReasoningDomain {
    /// Detect reasoning domain from problem statement and first chunk content
    pub fn detect(problem: &str, first_chunk: Option<&str>) -> Self {
        let text = format!("{} {}", problem, first_chunk.unwrap_or(""));
        let lower = text.to_lowercase();

        if lower.contains("debug")
            || lower.contains("error")
            || lower.contains("crash")
            || lower.contains("bug")
            || lower.contains("fix")
        {
            Self::Debugging
        } else if lower.contains("design")
            || lower.contains("architecture")
            || lower.contains("scalable")
            || lower.contains("system")
        {
            Self::Architecture
        } else if lower.contains("prove")
            || lower.contains("calculate")
            || lower.contains("derive")
            || lower.contains("theorem")
        {
            Self::Mathematical
        } else if lower.contains("consistent")
            || lower.contains("valid")
            || lower.contains("implies")
            || lower.contains("logic")
        {
            Self::Logical
        } else {
            Self::General
        }
    }

    /// Get domain-specific reasoning strategy instructions
    pub fn strategy_instructions(&self) -> &'static str {
        match self {
            Self::Debugging => "
DEBUGGING STRATEGY:
- Generate multiple hypotheses about root cause
- Systematically gather evidence (logs, traces, measurements)
- Eliminate possibilities through testing
- Distinguish symptoms from root causes
- Verify proposed fix addresses the root issue",

            Self::Architecture => "
ARCHITECTURE STRATEGY:
- Clarify all requirements and constraints explicitly
- Identify and analyze trade-offs between approaches
- Consider component interactions and dependencies
- Evaluate scalability, maintainability, and extensibility
- Validate design against stated requirements",

            Self::Mathematical => "
MATHEMATICAL STRATEGY:
- State all assumptions and constraints clearly
- Validate each algebraic/logical step
- Check boundary conditions and edge cases
- Verify dimensional analysis and units
- Confirm final answer satisfies original problem",

            Self::Logical => "
LOGICAL STRATEGY:
- Check for logical contradictions
- Verify all implications are valid
- Test against potential counterexamples
- Ensure argument completeness
- Validate consistency across claims",

            Self::General => "",
        }
    }
}

/// Verification status for a reasoning chunk
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum VerificationStatus {
    Pass,
    Fail,
    Uncertain,
}

/// Verification result from self-checking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub status: VerificationStatus,
    pub confidence: f32,
    pub issues: Vec<String>,
    pub key_concepts: Vec<String>,
}

impl Default for VerificationResult {
    fn default() -> Self {
        Self {
            status: VerificationStatus::Uncertain,
            confidence: 0.5,
            issues: Vec::new(),
            key_concepts: Vec::new(),
        }
    }
}

impl VerificationResult {
    /// Create a passing verification
    pub fn pass(confidence: f32, key_concepts: Vec<String>) -> Self {
        Self {
            status: VerificationStatus::Pass,
            confidence,
            issues: Vec::new(),
            key_concepts,
        }
    }

    /// Create a failing verification
    pub fn fail(confidence: f32, issues: Vec<String>) -> Self {
        Self {
            status: VerificationStatus::Fail,
            confidence,
            issues,
            key_concepts: Vec::new(),
        }
    }

    /// Create an uncertain verification
    pub fn uncertain(confidence: f32, issues: Vec<String>, key_concepts: Vec<String>) -> Self {
        Self {
            status: VerificationStatus::Uncertain,
            confidence,
            issues,
            key_concepts,
        }
    }
}

/// Metadata aggregated across a reasoning session
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub total_tokens: usize,
    pub avg_confidence: f32,
    pub verification_failures: usize,
}

impl SessionMetadata {
    /// Update metadata from verification results
    pub fn update_from_verifications(&mut self, verifications: &[VerificationResult]) {
        if !verifications.is_empty() {
            let sum: f32 = verifications.iter().map(|v| v.confidence).sum();
            self.avg_confidence = sum / verifications.len() as f32;

            self.verification_failures = verifications
                .iter()
                .filter(|v| matches!(v.status, VerificationStatus::Fail))
                .count();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_detection_debugging() {
        let domain = ReasoningDomain::detect("How do I debug this crash?", None);
        assert_eq!(domain, ReasoningDomain::Debugging);

        let domain = ReasoningDomain::detect("There's a bug in my code", None);
        assert_eq!(domain, ReasoningDomain::Debugging);
    }

    #[test]
    fn test_domain_detection_architecture() {
        let domain = ReasoningDomain::detect("Design a scalable system", None);
        assert_eq!(domain, ReasoningDomain::Architecture);
    }

    #[test]
    fn test_domain_detection_mathematical() {
        let domain = ReasoningDomain::detect("Prove that x² + y² = z²", None);
        assert_eq!(domain, ReasoningDomain::Mathematical);
    }

    #[test]
    fn test_domain_detection_logical() {
        let domain = ReasoningDomain::detect("Is this argument logically valid?", None);
        assert_eq!(domain, ReasoningDomain::Logical);
    }

    #[test]
    fn test_domain_detection_general() {
        let domain = ReasoningDomain::detect("Explain this concept", None);
        assert_eq!(domain, ReasoningDomain::General);
    }

    #[test]
    fn test_verification_result_constructors() {
        let pass = VerificationResult::pass(0.9, vec!["concept1".to_string()]);
        assert_eq!(pass.status, VerificationStatus::Pass);
        assert_eq!(pass.confidence, 0.9);

        let fail = VerificationResult::fail(0.3, vec!["issue1".to_string()]);
        assert_eq!(fail.status, VerificationStatus::Fail);
        assert_eq!(fail.confidence, 0.3);
    }

    #[test]
    fn test_metadata_update() {
        let mut metadata = SessionMetadata::default();
        let verifications = vec![
            VerificationResult::pass(0.9, vec![]),
            VerificationResult::pass(0.8, vec![]),
            VerificationResult::fail(0.4, vec![]),
        ];

        metadata.update_from_verifications(&verifications);
        assert!((metadata.avg_confidence - 0.7).abs() < 0.01); // (0.9 + 0.8 + 0.4) / 3
        assert_eq!(metadata.verification_failures, 1);
    }
}
