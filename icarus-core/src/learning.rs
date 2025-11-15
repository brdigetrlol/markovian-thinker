// Learning Module - Knowledge Distillation & Skill Acquisition
// Enables Icarus to learn from Claude through strategy extraction

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Skill domain classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SkillDomain {
    Debugging,
    Refactoring,
    Architecture,
    Testing,
    Performance,
    Documentation,
    CodeReview,
    ProblemDecomposition,
    General,
}

/// A step in a skill's execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillStep {
    pub description: String,
    pub action_type: String,
    pub parameters: HashMap<String, String>,
}

/// A learned skill/strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// Unique skill identifier
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Skill domain
    pub domain: SkillDomain,

    /// Pattern/condition for when to apply this skill
    pub pattern: String,

    /// Sequence of steps to execute
    pub steps: Vec<SkillStep>,

    /// Decision heuristics
    pub heuristics: Vec<String>,

    /// Success rate (0.0-1.0)
    pub success_rate: f64,

    /// Number of times applied
    pub application_count: u32,

    /// When this skill was learned
    pub learned_at: chrono::DateTime<chrono::Utc>,

    /// Source of learning (e.g., "claude_interaction_2023-11-15")
    pub source: String,
}

impl Skill {
    /// Create new skill
    pub fn new(
        name: String,
        domain: SkillDomain,
        pattern: String,
        steps: Vec<SkillStep>,
        heuristics: Vec<String>,
        source: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            domain,
            pattern,
            steps,
            heuristics,
            success_rate: 0.5, // Start neutral
            application_count: 0,
            learned_at: chrono::Utc::now(),
            source,
        }
    }

    /// Record successful application
    pub fn record_success(&mut self) {
        self.application_count += 1;
        // Update success rate with exponential moving average
        let alpha = 0.2;
        self.success_rate = alpha * 1.0 + (1.0 - alpha) * self.success_rate;
    }

    /// Record failed application
    pub fn record_failure(&mut self) {
        self.application_count += 1;
        // Update success rate with exponential moving average
        let alpha = 0.2;
        self.success_rate = alpha * 0.0 + (1.0 - alpha) * self.success_rate;
    }
}

/// Captured interaction for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub problem: String,
    pub claude_reasoning: Vec<String>,
    pub solution: String,
    pub context: HashMap<String, String>,
}

/// Skill library - stores learned skills
pub struct SkillLibrary {
    skills: Arc<RwLock<HashMap<String, Skill>>>,
    domain_index: Arc<RwLock<HashMap<SkillDomain, Vec<String>>>>,
}

impl SkillLibrary {
    pub fn new() -> Self {
        Self {
            skills: Arc::new(RwLock::new(HashMap::new())),
            domain_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add skill to library
    pub async fn add_skill(&self, skill: Skill) -> Result<()> {
        let skill_id = skill.id.clone();
        let domain = skill.domain.clone();

        // Add to main storage
        self.skills.write().await.insert(skill_id.clone(), skill);

        // Update domain index
        let mut index = self.domain_index.write().await;
        index
            .entry(domain)
            .or_insert_with(Vec::new)
            .push(skill_id);

        Ok(())
    }

    /// Get skill by ID
    pub async fn get_skill(&self, skill_id: &str) -> Option<Skill> {
        self.skills.read().await.get(skill_id).cloned()
    }

    /// Get all skills for a domain
    pub async fn get_skills_by_domain(&self, domain: &SkillDomain) -> Vec<Skill> {
        let index = self.domain_index.read().await;
        let skill_ids = match index.get(domain) {
            Some(ids) => ids.clone(),
            None => return vec![],
        };

        let skills_lock = self.skills.read().await;
        skill_ids
            .iter()
            .filter_map(|id| skills_lock.get(id).cloned())
            .collect()
    }

    /// Find skills matching a pattern
    pub async fn find_matching_skills(&self, problem: &str) -> Vec<Skill> {
        let skills = self.skills.read().await;
        skills
            .values()
            .filter(|skill| {
                // Simple keyword matching (can be improved with embeddings)
                problem.to_lowercase().contains(&skill.pattern.to_lowercase())
            })
            .cloned()
            .collect()
    }

    /// Get skill statistics
    pub async fn get_statistics(&self) -> HashMap<String, serde_json::Value> {
        let skills = self.skills.read().await;

        let total_skills = skills.len();
        let avg_success_rate = if total_skills > 0 {
            skills.values().map(|s| s.success_rate).sum::<f64>() / total_skills as f64
        } else {
            0.0
        };

        let domain_counts: HashMap<String, usize> = self.domain_index
            .read()
            .await
            .iter()
            .map(|(domain, skills)| (format!("{:?}", domain), skills.len()))
            .collect();

        let mut stats = HashMap::new();
        stats.insert("total_skills".to_string(), serde_json::json!(total_skills));
        stats.insert("average_success_rate".to_string(), serde_json::json!(avg_success_rate));
        stats.insert("domain_distribution".to_string(), serde_json::json!(domain_counts));

        stats
    }

    /// Update skill success rate
    pub async fn record_outcome(&self, skill_id: &str, success: bool) -> Result<()> {
        let mut skills = self.skills.write().await;
        if let Some(skill) = skills.get_mut(skill_id) {
            if success {
                skill.record_success();
            } else {
                skill.record_failure();
            }
        }
        Ok(())
    }
}

/// Strategy extractor - parses Claude's reasoning into skills
pub struct StrategyExtractor;

impl StrategyExtractor {
    /// Extract skill from an interaction
    pub fn extract_skill(interaction: &Interaction) -> Result<Option<Skill>> {
        // Parse Claude's reasoning to identify:
        // 1. Problem pattern
        // 2. Solution steps
        // 3. Decision heuristics

        if interaction.claude_reasoning.is_empty() {
            return Ok(None);
        }

        // Classify domain based on problem keywords
        let domain = Self::classify_domain(&interaction.problem);

        // Extract pattern (simplified - use first sentence of problem)
        let pattern = interaction.problem
            .lines()
            .next()
            .unwrap_or(&interaction.problem)
            .to_string();

        // Convert reasoning steps to skill steps
        let mut steps = Vec::new();
        for (idx, reasoning) in interaction.claude_reasoning.iter().enumerate() {
            steps.push(SkillStep {
                description: reasoning.clone(),
                action_type: "reasoning_step".to_string(),
                parameters: HashMap::from([
                    ("step_number".to_string(), (idx + 1).to_string()),
                ]),
            });
        }

        // Extract heuristics (look for patterns like "always", "first", "ensure")
        let heuristics = Self::extract_heuristics(&interaction.claude_reasoning);

        // Generate skill name
        let name = format!("{:?}-Strategy-{}", domain, uuid::Uuid::new_v4().to_string()[..8].to_string());

        let skill = Skill::new(
            name,
            domain,
            pattern,
            steps,
            heuristics,
            format!("interaction-{}", interaction.id),
        );

        Ok(Some(skill))
    }

    /// Classify problem domain
    fn classify_domain(problem: &str) -> SkillDomain {
        let problem_lower = problem.to_lowercase();

        if problem_lower.contains("bug") || problem_lower.contains("error") || problem_lower.contains("fix") {
            SkillDomain::Debugging
        } else if problem_lower.contains("refactor") || problem_lower.contains("clean") || problem_lower.contains("improve") {
            SkillDomain::Refactoring
        } else if problem_lower.contains("architecture") || problem_lower.contains("design") || problem_lower.contains("structure") {
            SkillDomain::Architecture
        } else if problem_lower.contains("test") {
            SkillDomain::Testing
        } else if problem_lower.contains("performance") || problem_lower.contains("optimize") {
            SkillDomain::Performance
        } else if problem_lower.contains("document") || problem_lower.contains("readme") {
            SkillDomain::Documentation
        } else if problem_lower.contains("review") {
            SkillDomain::CodeReview
        } else {
            SkillDomain::General
        }
    }

    /// Extract heuristics from reasoning steps
    fn extract_heuristics(reasoning: &[String]) -> Vec<String> {
        let mut heuristics = Vec::new();

        for step in reasoning {
            // Look for actionable principles
            let step_lower = step.to_lowercase();

            if step_lower.contains("always") || step_lower.contains("never") {
                heuristics.push(step.clone());
            } else if step_lower.contains("first") || step_lower.contains("start by") {
                heuristics.push(step.clone());
            } else if step_lower.contains("ensure") || step_lower.contains("make sure") {
                heuristics.push(step.clone());
            } else if step_lower.contains("avoid") || step_lower.contains("don't") {
                heuristics.push(step.clone());
            }
        }

        heuristics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_skill_library() {
        let library = SkillLibrary::new();

        let skill = Skill::new(
            "Test Skill".to_string(),
            SkillDomain::Testing,
            "write test".to_string(),
            vec![],
            vec!["Always write tests first".to_string()],
            "test".to_string(),
        );

        library.add_skill(skill.clone()).await.unwrap();

        let retrieved = library.get_skill(&skill.id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Skill");
    }

    #[test]
    fn test_domain_classification() {
        assert_eq!(
            StrategyExtractor::classify_domain("Fix the bug in the function"),
            SkillDomain::Debugging
        );

        assert_eq!(
            StrategyExtractor::classify_domain("Refactor this messy code"),
            SkillDomain::Refactoring
        );

        assert_eq!(
            StrategyExtractor::classify_domain("Design the system architecture"),
            SkillDomain::Architecture
        );
    }
}
