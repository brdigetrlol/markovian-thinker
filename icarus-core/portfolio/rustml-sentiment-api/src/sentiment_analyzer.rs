//! Core sentiment analysis engine using VADER algorithm
//!
//! This module implements sentiment analysis using the VADER (Valence Aware Dictionary
//! and sEntiment Reasoner) algorithm, which is specifically tuned for social media text
//! and works well with informal language, emojis, and slang.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Sentiment analysis result with detailed scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentResult {
    /// Overall sentiment classification
    pub sentiment: SentimentClass,
    /// Positive sentiment score (0.0 to 1.0)
    pub positive: f64,
    /// Negative sentiment score (0.0 to 1.0)
    pub negative: f64,
    /// Neutral sentiment score (0.0 to 1.0)
    pub neutral: f64,
    /// Compound score (-1.0 to 1.0) - normalized, weighted composite score
    pub compound: f64,
    /// Confidence level of the analysis
    pub confidence: f64,
}

/// Sentiment classification categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SentimentClass {
    Positive,
    Negative,
    Neutral,
}

/// Main sentiment analyzer using VADER and custom enhancements
pub struct SentimentAnalyzer<'a> {
    /// VADER analyzer instance
    analyzer: vader_sentiment::SentimentIntensityAnalyzer<'a>,
    /// Custom sentiment lexicon for domain-specific terms
    custom_lexicon: HashMap<String, f64>,
}

impl<'a> SentimentAnalyzer<'a> {
    /// Create a new sentiment analyzer with default settings
    pub fn new() -> Result<Self> {
        Ok(Self {
            analyzer: vader_sentiment::SentimentIntensityAnalyzer::new(),
            custom_lexicon: Self::build_custom_lexicon(),
        })
    }

    /// Build custom lexicon for domain-specific sentiment analysis
    /// This can be extended for industry-specific terminology
    fn build_custom_lexicon() -> HashMap<String, f64> {
        let mut lexicon = HashMap::new();

        // Technology/product related terms
        lexicon.insert("innovative".to_string(), 2.5);
        lexicon.insert("breakthrough".to_string(), 2.8);
        lexicon.insert("cutting-edge".to_string(), 2.3);
        lexicon.insert("obsolete".to_string(), -2.1);
        lexicon.insert("buggy".to_string(), -2.0);

        // Business/service related terms
        lexicon.insert("efficient".to_string(), 2.0);
        lexicon.insert("professional".to_string(), 1.8);
        lexicon.insert("responsive".to_string(), 1.9);
        lexicon.insert("unresponsive".to_string(), -2.0);
        lexicon.insert("scam".to_string(), -3.5);

        // Modern social media expressions
        lexicon.insert("lit".to_string(), 2.0);
        lexicon.insert("fire".to_string(), 2.2);
        lexicon.insert("toxic".to_string(), -2.5);

        lexicon
    }

    /// Analyze sentiment of a single text
    pub fn analyze(&self, text: &str) -> Result<SentimentResult> {
        // Preprocess text
        let preprocessed = self.preprocess_text(text);

        // Get VADER scores
        let scores = self.analyzer.polarity_scores(&preprocessed);

        // Apply custom lexicon adjustments
        let adjusted_compound = self.apply_custom_lexicon(&preprocessed, scores["compound"]);

        // Determine sentiment class based on compound score
        let sentiment = self.classify_sentiment(adjusted_compound);

        // Calculate confidence based on score magnitudes
        let confidence = self.calculate_confidence(&scores, adjusted_compound);

        Ok(SentimentResult {
            sentiment,
            positive: scores["pos"],
            negative: scores["neg"],
            neutral: scores["neu"],
            compound: adjusted_compound,
            confidence,
        })
    }

    /// Analyze sentiment of multiple texts in batch
    pub fn analyze_batch(&self, texts: &[String]) -> Result<Vec<SentimentResult>> {
        texts.iter()
            .map(|text| self.analyze(text))
            .collect()
    }

    /// Preprocess text for better sentiment analysis
    fn preprocess_text(&self, text: &str) -> String {
        text.trim()
            .replace("...", " ")
            .replace("!!", "!")
            .to_lowercase()
    }

    /// Apply custom lexicon adjustments to the compound score
    fn apply_custom_lexicon(&self, text: &str, base_score: f64) -> f64 {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut adjustment = 0.0;

        for word in words {
            if let Some(&score) = self.custom_lexicon.get(word) {
                adjustment += score * 0.1; // Scale down custom adjustments
            }
        }

        // Clamp the result to [-1.0, 1.0]
        (base_score + adjustment).max(-1.0).min(1.0)
    }

    /// Classify sentiment based on compound score
    fn classify_sentiment(&self, compound: f64) -> SentimentClass {
        if compound >= 0.05 {
            SentimentClass::Positive
        } else if compound <= -0.05 {
            SentimentClass::Negative
        } else {
            SentimentClass::Neutral
        }
    }

    /// Calculate confidence level based on score distribution
    fn calculate_confidence(&self, scores: &HashMap<&str, f64>, compound: f64) -> f64 {
        // Higher absolute compound score = higher confidence
        let magnitude_confidence = compound.abs();

        // Lower neutral score = higher confidence in positive/negative classification
        let neutral_confidence = 1.0 - scores["neu"];

        // Combine both factors
        ((magnitude_confidence + neutral_confidence) / 2.0).min(1.0)
    }

    /// Analyze sentiment trends over multiple texts
    pub fn analyze_trend(&self, texts: &[String]) -> Result<SentimentTrend> {
        let results = self.analyze_batch(texts)?;

        let positive_count = results.iter().filter(|r| r.sentiment == SentimentClass::Positive).count();
        let negative_count = results.iter().filter(|r| r.sentiment == SentimentClass::Negative).count();
        let neutral_count = results.iter().filter(|r| r.sentiment == SentimentClass::Neutral).count();

        let avg_compound: f64 = results.iter().map(|r| r.compound).sum::<f64>() / results.len() as f64;

        Ok(SentimentTrend {
            total_analyzed: results.len(),
            positive_count,
            negative_count,
            neutral_count,
            average_compound: avg_compound,
            overall_sentiment: self.classify_sentiment(avg_compound),
        })
    }
}

impl<'a> Default for SentimentAnalyzer<'a> {
    fn default() -> Self {
        Self::new().expect("Failed to create default SentimentAnalyzer")
    }
}

/// Sentiment trend analysis result
#[derive(Debug, Serialize, Deserialize)]
pub struct SentimentTrend {
    pub total_analyzed: usize,
    pub positive_count: usize,
    pub negative_count: usize,
    pub neutral_count: usize,
    pub average_compound: f64,
    pub overall_sentiment: SentimentClass,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive_sentiment() {
        let analyzer = SentimentAnalyzer::new().unwrap();
        let result = analyzer.analyze("This is absolutely amazing and wonderful!").unwrap();
        assert_eq!(result.sentiment, SentimentClass::Positive);
        assert!(result.compound > 0.5);
    }

    #[test]
    fn test_negative_sentiment() {
        let analyzer = SentimentAnalyzer::new().unwrap();
        let result = analyzer.analyze("This is terrible and awful!").unwrap();
        assert_eq!(result.sentiment, SentimentClass::Negative);
        assert!(result.compound < -0.5);
    }

    #[test]
    fn test_neutral_sentiment() {
        let analyzer = SentimentAnalyzer::new().unwrap();
        let result = analyzer.analyze("The sky is blue.").unwrap();
        assert_eq!(result.sentiment, SentimentClass::Neutral);
    }

    #[test]
    fn test_batch_analysis() {
        let analyzer = SentimentAnalyzer::new().unwrap();
        let texts = vec![
            "Great product!".to_string(),
            "Terrible experience.".to_string(),
            "It's okay.".to_string(),
        ];
        let results = analyzer.analyze_batch(&texts).unwrap();
        assert_eq!(results.len(), 3);
    }
}
