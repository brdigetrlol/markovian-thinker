//! Text classification module for topic and category detection
//!
//! This module implements a keyword-based and pattern-based text classifier
//! that can categorize text into predefined topics. It uses TF-IDF-like
//! scoring and can be extended with more sophisticated ML models.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation;

/// Text classification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationResult {
    /// Primary category
    pub category: Category,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// All category scores
    pub scores: HashMap<String, f64>,
    /// Detected keywords that influenced classification
    pub keywords: Vec<String>,
}

/// Predefined categories for classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Technology,
    Business,
    Sports,
    Entertainment,
    Politics,
    Science,
    Health,
    Finance,
    Education,
    Travel,
    Food,
    Unknown,
}

impl Category {
    /// Get all available categories
    pub fn all() -> Vec<Category> {
        vec![
            Category::Technology,
            Category::Business,
            Category::Sports,
            Category::Entertainment,
            Category::Politics,
            Category::Science,
            Category::Health,
            Category::Finance,
            Category::Education,
            Category::Travel,
            Category::Food,
        ]
    }

    /// Get category name as string
    pub fn as_str(&self) -> &str {
        match self {
            Category::Technology => "technology",
            Category::Business => "business",
            Category::Sports => "sports",
            Category::Entertainment => "entertainment",
            Category::Politics => "politics",
            Category::Science => "science",
            Category::Health => "health",
            Category::Finance => "finance",
            Category::Education => "education",
            Category::Travel => "travel",
            Category::Food => "food",
            Category::Unknown => "unknown",
        }
    }
}

/// Text classifier using keyword-based and pattern-based approaches
pub struct TextClassifier {
    /// Category keywords with weights
    category_keywords: HashMap<Category, Vec<(String, f64)>>,
}

impl TextClassifier {
    /// Create a new text classifier
    pub fn new() -> Self {
        Self {
            category_keywords: Self::build_category_keywords(),
        }
    }

    /// Build keyword dictionary for each category
    fn build_category_keywords() -> HashMap<Category, Vec<(String, f64)>> {
        let mut keywords = HashMap::new();

        // Technology keywords
        keywords.insert(
            Category::Technology,
            vec![
                ("ai".to_string(), 3.0),
                ("artificial intelligence".to_string(), 3.5),
                ("machine learning".to_string(), 3.5),
                ("software".to_string(), 2.5),
                ("programming".to_string(), 2.8),
                ("computer".to_string(), 2.0),
                ("tech".to_string(), 2.5),
                ("digital".to_string(), 2.0),
                ("algorithm".to_string(), 3.0),
                ("data".to_string(), 2.2),
                ("cloud".to_string(), 2.5),
                ("blockchain".to_string(), 3.0),
                ("crypto".to_string(), 2.8),
                ("app".to_string(), 2.0),
                ("api".to_string(), 2.5),
                ("neural".to_string(), 3.0),
                ("robot".to_string(), 2.8),
            ],
        );

        // Business keywords
        keywords.insert(
            Category::Business,
            vec![
                ("business".to_string(), 3.0),
                ("company".to_string(), 2.5),
                ("enterprise".to_string(), 2.8),
                ("management".to_string(), 2.5),
                ("strategy".to_string(), 2.3),
                ("marketing".to_string(), 2.8),
                ("sales".to_string(), 2.5),
                ("profit".to_string(), 2.8),
                ("revenue".to_string(), 2.8),
                ("ceo".to_string(), 2.5),
                ("startup".to_string(), 2.8),
                ("entrepreneur".to_string(), 3.0),
            ],
        );

        // Sports keywords
        keywords.insert(
            Category::Sports,
            vec![
                ("sports".to_string(), 3.0),
                ("game".to_string(), 2.0),
                ("team".to_string(), 2.0),
                ("player".to_string(), 2.5),
                ("football".to_string(), 3.0),
                ("basketball".to_string(), 3.0),
                ("soccer".to_string(), 3.0),
                ("baseball".to_string(), 3.0),
                ("championship".to_string(), 2.8),
                ("olympic".to_string(), 3.0),
                ("athlete".to_string(), 2.8),
                ("coach".to_string(), 2.5),
            ],
        );

        // Entertainment keywords
        keywords.insert(
            Category::Entertainment,
            vec![
                ("movie".to_string(), 3.0),
                ("film".to_string(), 3.0),
                ("music".to_string(), 3.0),
                ("entertainment".to_string(), 3.0),
                ("celebrity".to_string(), 2.8),
                ("actor".to_string(), 2.8),
                ("show".to_string(), 2.2),
                ("concert".to_string(), 3.0),
                ("album".to_string(), 2.8),
                ("streaming".to_string(), 2.5),
                ("netflix".to_string(), 2.5),
            ],
        );

        // Science keywords
        keywords.insert(
            Category::Science,
            vec![
                ("science".to_string(), 3.0),
                ("research".to_string(), 2.8),
                ("study".to_string(), 2.3),
                ("discovery".to_string(), 2.8),
                ("experiment".to_string(), 2.8),
                ("scientist".to_string(), 2.8),
                ("physics".to_string(), 3.0),
                ("chemistry".to_string(), 3.0),
                ("biology".to_string(), 3.0),
                ("quantum".to_string(), 3.0),
                ("dna".to_string(), 3.0),
            ],
        );

        // Health keywords
        keywords.insert(
            Category::Health,
            vec![
                ("health".to_string(), 3.0),
                ("medical".to_string(), 3.0),
                ("doctor".to_string(), 2.8),
                ("hospital".to_string(), 2.8),
                ("medicine".to_string(), 3.0),
                ("disease".to_string(), 2.8),
                ("treatment".to_string(), 2.8),
                ("patient".to_string(), 2.5),
                ("wellness".to_string(), 2.5),
                ("fitness".to_string(), 2.8),
                ("vaccine".to_string(), 3.0),
            ],
        );

        // Finance keywords
        keywords.insert(
            Category::Finance,
            vec![
                ("finance".to_string(), 3.0),
                ("stock".to_string(), 3.0),
                ("market".to_string(), 2.5),
                ("investment".to_string(), 3.0),
                ("bank".to_string(), 2.8),
                ("money".to_string(), 2.5),
                ("currency".to_string(), 2.8),
                ("trading".to_string(), 3.0),
                ("economy".to_string(), 2.8),
                ("bitcoin".to_string(), 3.0),
            ],
        );

        // Education keywords
        keywords.insert(
            Category::Education,
            vec![
                ("education".to_string(), 3.0),
                ("school".to_string(), 2.8),
                ("university".to_string(), 3.0),
                ("student".to_string(), 2.5),
                ("teacher".to_string(), 2.8),
                ("learning".to_string(), 2.5),
                ("course".to_string(), 2.5),
                ("degree".to_string(), 2.8),
            ],
        );

        // Travel keywords
        keywords.insert(
            Category::Travel,
            vec![
                ("travel".to_string(), 3.0),
                ("vacation".to_string(), 3.0),
                ("hotel".to_string(), 2.8),
                ("flight".to_string(), 2.8),
                ("tourism".to_string(), 3.0),
                ("destination".to_string(), 2.5),
                ("trip".to_string(), 2.5),
            ],
        );

        // Food keywords
        keywords.insert(
            Category::Food,
            vec![
                ("food".to_string(), 3.0),
                ("restaurant".to_string(), 3.0),
                ("recipe".to_string(), 3.0),
                ("cooking".to_string(), 3.0),
                ("chef".to_string(), 2.8),
                ("cuisine".to_string(), 2.8),
                ("dish".to_string(), 2.5),
            ],
        );

        keywords
    }

    /// Classify a single text
    pub fn classify(&self, text: &str) -> Result<ClassificationResult> {
        let normalized_text = text.to_lowercase();
        let words: Vec<&str> = normalized_text.unicode_words().collect();

        let mut category_scores: HashMap<Category, f64> = HashMap::new();
        let mut detected_keywords: Vec<String> = Vec::new();

        // Calculate scores for each category
        for (category, keywords) in &self.category_keywords {
            let mut score = 0.0;

            for (keyword, weight) in keywords {
                if normalized_text.contains(keyword) {
                    score += weight;
                    detected_keywords.push(keyword.clone());
                }
            }

            // Normalize by text length (prevent long texts from having unfair advantage)
            let normalized_score = if words.len() > 0 {
                score / (words.len() as f64).sqrt()
            } else {
                0.0
            };

            category_scores.insert(*category, normalized_score);
        }

        // Find the category with the highest score
        let (category, max_score) = category_scores
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(cat, score)| (*cat, *score))
            .unwrap_or((Category::Unknown, 0.0));

        // Calculate confidence based on score separation
        let total_score: f64 = category_scores.values().sum();
        let confidence = if total_score > 0.0 {
            (max_score / total_score).min(1.0)
        } else {
            0.0
        };

        // Convert scores to string keys for serialization
        let scores: HashMap<String, f64> = category_scores
            .iter()
            .map(|(cat, score)| (cat.as_str().to_string(), *score))
            .collect();

        detected_keywords.sort();
        detected_keywords.dedup();

        Ok(ClassificationResult {
            category: if confidence > 0.1 { category } else { Category::Unknown },
            confidence,
            scores,
            keywords: detected_keywords,
        })
    }

    /// Classify multiple texts in batch
    pub fn classify_batch(&self, texts: &[String]) -> Result<Vec<ClassificationResult>> {
        texts.iter()
            .map(|text| self.classify(text))
            .collect()
    }
}

impl Default for TextClassifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_technology_classification() {
        let classifier = TextClassifier::new();
        let result = classifier.classify(
            "Machine learning and artificial intelligence are transforming technology"
        ).unwrap();
        assert_eq!(result.category, Category::Technology);
    }

    #[test]
    fn test_sports_classification() {
        let classifier = TextClassifier::new();
        let result = classifier.classify(
            "The football team won the championship game last night"
        ).unwrap();
        assert_eq!(result.category, Category::Sports);
    }

    #[test]
    fn test_batch_classification() {
        let classifier = TextClassifier::new();
        let texts = vec![
            "AI and machine learning".to_string(),
            "Football championship".to_string(),
        ];
        let results = classifier.classify_batch(&texts).unwrap();
        assert_eq!(results.len(), 2);
    }
}
