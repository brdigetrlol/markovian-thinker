// Parser for structured LLM output with verification sections
// Extracts [REASONING], [VERIFICATION], and [CARRYOVER] blocks

use crate::types::*;
use regex::Regex;

/// Parsed chunk with separated sections
pub struct ParsedChunk {
    pub reasoning: String,
    pub verification: VerificationResult,
    pub carryover: String,
    pub is_complete: bool,
}

/// Parse structured output from LLM with verification sections
pub fn parse_chunk_output(output: &str) -> ParsedChunk {
    let reasoning = extract_section(output, "REASONING");
    let verification_text = extract_section(output, "VERIFICATION");
    let carryover = extract_section(output, "CARRYOVER");

    let verification = parse_verification(&verification_text);
    let is_complete = output.contains("[SOLUTION]")
        || output.contains("[DONE]")
        || output.contains("[EOS]");

    ParsedChunk {
        reasoning,
        verification,
        carryover,
        is_complete,
    }
}

/// Extract a named section from output
fn extract_section(output: &str, section_name: &str) -> String {
    let pattern = format!(
        r"(?s)\[{}\](.*?)(?:\[(?:VERIFICATION|CARRYOVER|REASONING|SOLUTION|DONE|EOS)\]|$)",
        section_name
    );
    let re = Regex::new(&pattern).unwrap();

    re.captures(output)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().trim().to_string())
        .unwrap_or_default()
}

/// Parse verification section into VerificationResult
fn parse_verification(text: &str) -> VerificationResult {
    if text.is_empty() {
        return VerificationResult::default();
    }

    let status = if text.to_lowercase().contains("status: pass") {
        VerificationStatus::Pass
    } else if text.to_lowercase().contains("status: fail") {
        VerificationStatus::Fail
    } else {
        VerificationStatus::Uncertain
    };

    let confidence = extract_confidence(text);
    let issues = extract_list(text, "Issues:");
    let key_concepts = extract_list(text, "Key Concepts:");

    VerificationResult {
        status,
        confidence,
        issues,
        key_concepts,
    }
}

/// Extract confidence value (0.0-1.0)
fn extract_confidence(text: &str) -> f32 {
    let re = Regex::new(r"Confidence:\s*([0-9.]+)").unwrap();
    re.captures(text)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse::<f32>().ok())
        .unwrap_or(0.5)
}

/// Extract comma-separated list after a marker
fn extract_list(text: &str, marker: &str) -> Vec<String> {
    if let Some(start) = text.find(marker) {
        let after_marker = &text[start + marker.len()..];
        let until_next = after_marker.split('\n').next().unwrap_or("");

        until_next
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && !s.to_lowercase().contains("none"))
            .collect()
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_complete_output() {
        let output = r#"
[REASONING]
Let me think about this problem...
The answer is 42.

[VERIFICATION]
Status: PASS
Confidence: 0.95
Issues: None
Key Concepts: problem solving, calculation

[CARRYOVER]
The final answer is 42, which was derived by analyzing...

[SOLUTION]
"#;

        let parsed = parse_chunk_output(output);

        assert!(parsed.reasoning.contains("The answer is 42"));
        assert_eq!(parsed.verification.status, VerificationStatus::Pass);
        assert!((parsed.verification.confidence - 0.95).abs() < 0.01);
        assert_eq!(parsed.verification.key_concepts.len(), 2);
        assert!(parsed.carryover.contains("final answer is 42"));
        assert!(parsed.is_complete);
    }

    #[test]
    fn test_parse_fail_verification() {
        let output = r#"
[REASONING]
Let me try this approach...

[VERIFICATION]
Status: FAIL
Confidence: 0.3
Issues: logical inconsistency, missing assumption
Key Concepts: None

[CARRYOVER]
Need to reconsider the approach...
"#;

        let parsed = parse_chunk_output(output);

        assert_eq!(parsed.verification.status, VerificationStatus::Fail);
        assert!((parsed.verification.confidence - 0.3).abs() < 0.01);
        assert_eq!(parsed.verification.issues.len(), 2);
        assert!(parsed
            .verification
            .issues
            .contains(&"logical inconsistency".to_string()));
        assert!(!parsed.is_complete);
    }

    #[test]
    fn test_parse_uncertain_verification() {
        let output = r#"
[REASONING]
This is a complex problem...

[VERIFICATION]
Status: UNCERTAIN
Confidence: 0.6
Issues: need more evidence
Key Concepts: complexity, uncertainty

[CARRYOVER]
Will continue investigating...
"#;

        let parsed = parse_chunk_output(output);

        assert_eq!(parsed.verification.status, VerificationStatus::Uncertain);
        assert!((parsed.verification.confidence - 0.6).abs() < 0.01);
    }

    #[test]
    fn test_parse_missing_sections() {
        let output = "Just some reasoning without structure.";

        let parsed = parse_chunk_output(output);

        // Should have defaults
        assert!(parsed.reasoning.is_empty());
        assert_eq!(parsed.verification.status, VerificationStatus::Uncertain);
        assert_eq!(parsed.verification.confidence, 0.5);
        assert!(parsed.carryover.is_empty());
    }

    #[test]
    fn test_extract_confidence() {
        assert!((extract_confidence("Confidence: 0.85") - 0.85).abs() < 0.01);
        assert!((extract_confidence("Confidence: 1.0") - 1.0).abs() < 0.01);
        assert!((extract_confidence("No confidence here") - 0.5).abs() < 0.01); // Default
    }

    #[test]
    fn test_extract_list() {
        let text = "Issues: error1, error2, error3";
        let list = extract_list(text, "Issues:");
        assert_eq!(list.len(), 3);
        assert_eq!(list[0], "error1");

        let text = "Key Concepts: None";
        let list = extract_list(text, "Key Concepts:");
        assert_eq!(list.len(), 0); // "None" filtered out
    }

    #[test]
    fn test_solution_markers() {
        assert!(parse_chunk_output("[SOLUTION]").is_complete);
        assert!(parse_chunk_output("[DONE]").is_complete);
        assert!(parse_chunk_output("[EOS]").is_complete);
        assert!(!parse_chunk_output("No marker").is_complete);
    }
}
