//! RustML Sentiment API Library
//!
//! This library provides sentiment analysis and text classification capabilities
//! that can be used either as a standalone API server or integrated into other applications.

pub mod error;
pub mod models;
pub mod sentiment_analyzer;
pub mod text_classifier;

pub use sentiment_analyzer::{SentimentAnalyzer, SentimentClass, SentimentResult, SentimentTrend};
pub use text_classifier::{Category, ClassificationResult, TextClassifier};
