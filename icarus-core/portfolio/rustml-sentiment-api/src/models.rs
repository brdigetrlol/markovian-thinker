//! Data models for API requests and responses

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::sentiment_analyzer::{SentimentResult, SentimentTrend};
use crate::text_classifier::ClassificationResult;

/// Request model for single text analysis
#[derive(Debug, Deserialize)]
pub struct AnalyzeRequest {
    /// Text to analyze
    pub text: String,
    /// Optional request identifier
    #[serde(default)]
    pub request_id: Option<String>,
}

/// Request model for batch text analysis
#[derive(Debug, Deserialize)]
pub struct BatchAnalyzeRequest {
    /// List of texts to analyze
    pub texts: Vec<String>,
    /// Optional request identifier
    #[serde(default)]
    pub request_id: Option<String>,
}

/// Response model for sentiment analysis
#[derive(Debug, Serialize)]
pub struct SentimentResponse {
    /// Unique response identifier
    pub id: String,
    /// Timestamp of analysis
    pub timestamp: DateTime<Utc>,
    /// Original text analyzed
    pub text: String,
    /// Sentiment analysis result
    pub sentiment: SentimentResult,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Response model for text classification
#[derive(Debug, Serialize)]
pub struct ClassificationResponse {
    /// Unique response identifier
    pub id: String,
    /// Timestamp of analysis
    pub timestamp: DateTime<Utc>,
    /// Original text analyzed
    pub text: String,
    /// Classification result
    pub classification: ClassificationResult,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Combined analysis response (sentiment + classification)
#[derive(Debug, Serialize)]
pub struct CombinedAnalysisResponse {
    /// Unique response identifier
    pub id: String,
    /// Timestamp of analysis
    pub timestamp: DateTime<Utc>,
    /// Original text analyzed
    pub text: String,
    /// Sentiment analysis result
    pub sentiment: SentimentResult,
    /// Classification result
    pub classification: ClassificationResult,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Response model for batch analysis
#[derive(Debug, Serialize)]
pub struct BatchAnalysisResponse {
    /// Unique response identifier
    pub id: String,
    /// Timestamp of analysis
    pub timestamp: DateTime<Utc>,
    /// Number of texts analyzed
    pub count: usize,
    /// Individual results
    pub results: Vec<CombinedAnalysisResponse>,
    /// Total processing time in milliseconds
    pub total_processing_time_ms: u64,
}

/// Response model for trend analysis
#[derive(Debug, Serialize)]
pub struct TrendAnalysisResponse {
    /// Unique response identifier
    pub id: String,
    /// Timestamp of analysis
    pub timestamp: DateTime<Utc>,
    /// Sentiment trend data
    pub trend: SentimentTrend,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    /// Service status
    pub status: String,
    /// Service version
    pub version: String,
    /// Uptime in seconds
    pub uptime_seconds: u64,
    /// Number of requests processed
    pub requests_processed: u64,
}

/// API statistics response
#[derive(Debug, Serialize)]
pub struct StatsResponse {
    /// Total requests processed
    pub total_requests: u64,
    /// Total texts analyzed
    pub total_texts_analyzed: u64,
    /// Average processing time in milliseconds
    pub avg_processing_time_ms: f64,
    /// Service uptime in seconds
    pub uptime_seconds: u64,
}

/// Error response model
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    /// Error code
    pub error: String,
    /// Human-readable error message
    pub message: String,
    /// Optional request ID for tracing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Timestamp of error
    pub timestamp: DateTime<Utc>,
}

impl ErrorResponse {
    /// Create a new error response
    pub fn new(error: String, message: String, request_id: Option<String>) -> Self {
        Self {
            error,
            message,
            request_id,
            timestamp: Utc::now(),
        }
    }
}

/// Generate a unique ID for responses
pub fn generate_id() -> String {
    Uuid::new_v4().to_string()
}
