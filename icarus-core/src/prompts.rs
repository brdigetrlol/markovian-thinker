// Enhanced prompt generation with integrated verification system
// Prompts guide LLM through self-verification and structured output

use crate::experts::ExpertGating;
use crate::state::MarkovianState;
use crate::types::ReasoningDomain;

/// Generate enhanced prompt with verification instructions
pub fn generate_prompt(state: &MarkovianState, domain: Option<&ReasoningDomain>) -> String {
    let base_instructions = format!(
        "Problem: {}\n\nPrevious Context: {}\n\n",
        state.query,
        if state.carryover.is_empty() {
            "(This is the first chunk)"
        } else {
            &state.carryover
        }
    );

    // Get domain-specific strategy if available
    let domain_instructions = domain
        .map(|d| d.strategy_instructions())
        .unwrap_or("");

    // Use expert gating to get specialized instructions
    let expert_instructions = if state.config.expert_config.enabled {
        let gating = ExpertGating::new(state.config.expert_config.clone());
        let experts = gating.select_experts(&state.query, Some(&state.carryover));

        if !experts.is_empty() {
            let expert = experts[0]; // Use top expert
            let expert_guidance = match expert.expert_type() {
                crate::experts::ExpertType::MathReasoning => {
                    "\nEXPERT GUIDANCE (Mathematical):
- Use clear mathematical notation
- Show each calculation step explicitly
- Use LaTeX formatting for equations: \\(...\\) inline, \\[...\\] displayed
- Mark final answer with \\boxed{answer}
- Verify calculations numerically when possible
"
                }
                crate::experts::ExpertType::CodeGeneration => {
                    "\nEXPERT GUIDANCE (Code):
- Use proper syntax highlighting with ```language blocks
- Include comments explaining complex logic
- Test edge cases explicitly
- Provide example inputs/outputs
- Mark solution with // SOLUTION or # SOLUTION comment
"
                }
                crate::experts::ExpertType::TextualReasoning => {
                    "\nEXPERT GUIDANCE (Text Analysis):
- Structure arguments clearly with numbered points
- Use evidence and citations when relevant
- Consider multiple perspectives
- Provide clear topic sentences
- Mark final conclusion with [CONCLUSION]
"
                }
                crate::experts::ExpertType::VisualReasoning => {
                    "\nEXPERT GUIDANCE (Visual):
- Describe visual elements systematically
- Use spatial relationships (left, right, above, below)
- Identify patterns and anomalies
- Provide structured descriptions
"
                }
                crate::experts::ExpertType::Mixed => {
                    "\nEXPERT GUIDANCE (Mixed Domain):
- Integrate multiple reasoning approaches
- Clearly separate different types of reasoning
- Cross-validate conclusions across domains
"
                }
            };
            expert_guidance.to_string()
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    format!(
        "{}{}{}

INSTRUCTIONS:

1. REASONING PHASE (up to {} tokens):
   - Apply the domain strategy above
   - Think step-by-step
   - Make assumptions explicit
   - Consider alternative approaches

2. VERIFICATION PHASE:
   After reasoning, perform rigorous self-verification:

   ✓ Logical Consistency: Are there contradictions?
   ✓ Completeness: Are assumptions stated?
   ✓ Accuracy: Are facts/calculations correct?
   ✓ Relevance: Does this advance toward solution?

   If verification FAILS, regenerate your reasoning with corrections.

3. OUTPUT FORMAT:
   Structure your response exactly as:

   [REASONING]
   ... your detailed reasoning here ...

   [VERIFICATION]
   Status: PASS | FAIL | UNCERTAIN
   Confidence: 0.XX (0.0-1.0)
   Issues: [list any problems, or \"None\"]
   Key Concepts: [main ideas this chunk establishes]

   [CARRYOVER]
   ... essential context for next chunk (top {} most important points) ...

4. TERMINATION:
   If you have reached a complete solution, add [SOLUTION] or [DONE] after verification.

Begin reasoning now:",
        base_instructions,
        domain_instructions,
        expert_instructions,
        state.config.chunk_size,
        state.config.carryover_size / 100 // Rough estimate: ~100 tokens per point
    )
}

/// Generate legacy prompt without verification (for backward compatibility)
pub fn generate_legacy_prompt(state: &MarkovianState) -> String {
    format!(
        "Problem: {}\n\nPrevious Context: {}\n\nReason about this problem. Generate up to {} tokens of reasoning. When you have a solution, mark it with [SOLUTION] or [DONE].",
        state.query,
        if state.carryover.is_empty() {
            "(This is the first chunk)"
        } else {
            &state.carryover
        },
        state.config.chunk_size
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::StateConfig;

    #[test]
    fn test_generate_prompt_includes_verification() {
        let config = StateConfig::default();
        let state = MarkovianState::new("Test problem".to_string(), config);
        let domain = ReasoningDomain::Debugging;

        let prompt = generate_prompt(&state, Some(&domain));

        // Should include verification sections
        assert!(prompt.contains("[REASONING]"));
        assert!(prompt.contains("[VERIFICATION]"));
        assert!(prompt.contains("[CARRYOVER]"));
        assert!(prompt.contains("DEBUGGING STRATEGY"));
    }

    #[test]
    fn test_generate_prompt_no_domain() {
        let config = StateConfig::default();
        let state = MarkovianState::new("Test problem".to_string(), config);

        let prompt = generate_prompt(&state, None);

        // Should still have verification structure
        assert!(prompt.contains("[VERIFICATION]"));
        // But no domain-specific strategy
        assert!(!prompt.contains("DEBUGGING STRATEGY"));
    }

    #[test]
    fn test_legacy_prompt_no_verification() {
        let config = StateConfig::default();
        let state = MarkovianState::new("Test problem".to_string(), config);

        let prompt = generate_legacy_prompt(&state);

        // Should NOT have verification sections
        assert!(!prompt.contains("[VERIFICATION]"));
        assert!(!prompt.contains("CARRYOVER]"));
    }

    #[test]
    fn test_prompt_includes_carryover() {
        let config = StateConfig::default();
        let mut state = MarkovianState::new("Test problem".to_string(), config);
        state.carryover = "Previous result".to_string();

        let prompt = generate_prompt(&state, None);

        assert!(prompt.contains("Previous result"));
        assert!(!prompt.contains("(This is the first chunk)"));
    }

    #[test]
    fn test_prompt_first_chunk() {
        let config = StateConfig::default();
        let state = MarkovianState::new("Test problem".to_string(), config);

        let prompt = generate_prompt(&state, None);

        assert!(prompt.contains("(This is the first chunk)"));
    }
}
