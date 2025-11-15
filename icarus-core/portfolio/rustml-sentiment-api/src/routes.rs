//! API route handlers

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use chrono::Utc;
use std::sync::Arc;
use std::time::Instant;

use crate::{
    error::{ApiError, ApiResult},
    models::*,
    sentiment_analyzer::SentimentAnalyzer,
    text_classifier::TextClassifier,
    AppState,
};

// Constants for validation
const MAX_TEXT_LENGTH: usize = 10_000;
const MAX_BATCH_SIZE: usize = 100;

/// Health check endpoint
pub async fn health_check<'a>(
    State(state): State<Arc<AppState<'a>>>,
) -> Json<HealthResponse> {
    let uptime = state.start_time.elapsed().as_secs();
    let requests = state.request_count.load(std::sync::atomic::Ordering::Relaxed);

    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime,
        requests_processed: requests,
    })
}

/// Statistics endpoint
pub async fn get_stats<'a>(
    State(state): State<Arc<AppState<'a>>>,
) -> Json<StatsResponse> {
    let uptime = state.start_time.elapsed().as_secs();
    let total_requests = state.request_count.load(std::sync::atomic::Ordering::Relaxed);
    let total_texts = state.text_count.load(std::sync::atomic::Ordering::Relaxed);

    // Calculate average processing time (simplified)
    let avg_time = if total_requests > 0 {
        (total_requests as f64 * 5.0) / total_requests as f64  // Placeholder calculation
    } else {
        0.0
    };

    Json(StatsResponse {
        total_requests,
        total_texts_analyzed: total_texts,
        avg_processing_time_ms: avg_time,
        uptime_seconds: uptime,
    })
}

/// Analyze sentiment of a single text
pub async fn analyze_sentiment<'a>(
    State(state): State<Arc<AppState<'a>>>,
    Json(payload): Json<AnalyzeRequest>,
) -> ApiResult<Json<SentimentResponse>> {
    // Increment request counter
    state.request_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    state.text_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    // Validate input
    if payload.text.trim().is_empty() {
        return Err(ApiError::InvalidInput("Text cannot be empty".to_string()));
    }

    if payload.text.len() > MAX_TEXT_LENGTH {
        return Err(ApiError::TextTooLong {
            max: MAX_TEXT_LENGTH,
            actual: payload.text.len(),
        });
    }

    // Perform analysis
    let start = Instant::now();
    let sentiment = state.sentiment_analyzer
        .analyze(&payload.text)
        .map_err(|e| ApiError::AnalysisFailed(e.to_string()))?;
    let processing_time = start.elapsed().as_millis() as u64;

    Ok(Json(SentimentResponse {
        id: generate_id(),
        timestamp: Utc::now(),
        text: payload.text,
        sentiment,
        processing_time_ms: processing_time,
    }))
}

/// Classify text into categories
pub async fn classify_text<'a>(
    State(state): State<Arc<AppState<'a>>>,
    Json(payload): Json<AnalyzeRequest>,
) -> ApiResult<Json<ClassificationResponse>> {
    // Increment request counter
    state.request_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    state.text_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    // Validate input
    if payload.text.trim().is_empty() {
        return Err(ApiError::InvalidInput("Text cannot be empty".to_string()));
    }

    if payload.text.len() > MAX_TEXT_LENGTH {
        return Err(ApiError::TextTooLong {
            max: MAX_TEXT_LENGTH,
            actual: payload.text.len(),
        });
    }

    // Perform classification
    let start = Instant::now();
    let classification = state.text_classifier
        .classify(&payload.text)
        .map_err(|e| ApiError::AnalysisFailed(e.to_string()))?;
    let processing_time = start.elapsed().as_millis() as u64;

    Ok(Json(ClassificationResponse {
        id: generate_id(),
        timestamp: Utc::now(),
        text: payload.text,
        classification,
        processing_time_ms: processing_time,
    }))
}

/// Combined analysis: sentiment + classification
pub async fn analyze_combined<'a>(
    State(state): State<Arc<AppState<'a>>>,
    Json(payload): Json<AnalyzeRequest>,
) -> ApiResult<Json<CombinedAnalysisResponse>> {
    // Increment request counter
    state.request_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    state.text_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    // Validate input
    if payload.text.trim().is_empty() {
        return Err(ApiError::InvalidInput("Text cannot be empty".to_string()));
    }

    if payload.text.len() > MAX_TEXT_LENGTH {
        return Err(ApiError::TextTooLong {
            max: MAX_TEXT_LENGTH,
            actual: payload.text.len(),
        });
    }

    // Perform both analyses
    let start = Instant::now();

    let sentiment = state.sentiment_analyzer
        .analyze(&payload.text)
        .map_err(|e| ApiError::AnalysisFailed(e.to_string()))?;

    let classification = state.text_classifier
        .classify(&payload.text)
        .map_err(|e| ApiError::AnalysisFailed(e.to_string()))?;

    let processing_time = start.elapsed().as_millis() as u64;

    Ok(Json(CombinedAnalysisResponse {
        id: generate_id(),
        timestamp: Utc::now(),
        text: payload.text,
        sentiment,
        classification,
        processing_time_ms: processing_time,
    }))
}

/// Batch analysis
pub async fn analyze_batch<'a>(
    State(state): State<Arc<AppState<'a>>>,
    Json(payload): Json<BatchAnalyzeRequest>,
) -> ApiResult<Json<BatchAnalysisResponse>> {
    // Increment request counter
    state.request_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    state.text_count.fetch_add(payload.texts.len() as u64, std::sync::atomic::Ordering::Relaxed);

    // Validate batch size
    if payload.texts.is_empty() {
        return Err(ApiError::InvalidInput("Batch cannot be empty".to_string()));
    }

    if payload.texts.len() > MAX_BATCH_SIZE {
        return Err(ApiError::BatchTooLarge {
            max: MAX_BATCH_SIZE,
            actual: payload.texts.len(),
        });
    }

    // Validate individual texts
    for (i, text) in payload.texts.iter().enumerate() {
        if text.trim().is_empty() {
            return Err(ApiError::InvalidInput(
                format!("Text at index {} is empty", i)
            ));
        }

        if text.len() > MAX_TEXT_LENGTH {
            return Err(ApiError::TextTooLong {
                max: MAX_TEXT_LENGTH,
                actual: text.len(),
            });
        }
    }

    // Perform batch analysis
    let start = Instant::now();
    let mut results = Vec::new();

    for text in &payload.texts {
        let sentiment = state.sentiment_analyzer
            .analyze(text)
            .map_err(|e| ApiError::AnalysisFailed(e.to_string()))?;

        let classification = state.text_classifier
            .classify(text)
            .map_err(|e| ApiError::AnalysisFailed(e.to_string()))?;

        results.push(CombinedAnalysisResponse {
            id: generate_id(),
            timestamp: Utc::now(),
            text: text.clone(),
            sentiment,
            classification,
            processing_time_ms: 0, // Individual times not tracked in batch
        });
    }

    let total_processing_time = start.elapsed().as_millis() as u64;

    Ok(Json(BatchAnalysisResponse {
        id: generate_id(),
        timestamp: Utc::now(),
        count: results.len(),
        results,
        total_processing_time_ms: total_processing_time,
    }))
}

/// Analyze sentiment trends
pub async fn analyze_trend<'a>(
    State(state): State<Arc<AppState<'a>>>,
    Json(payload): Json<BatchAnalyzeRequest>,
) -> ApiResult<Json<TrendAnalysisResponse>> {
    // Increment request counter
    state.request_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    state.text_count.fetch_add(payload.texts.len() as u64, std::sync::atomic::Ordering::Relaxed);

    // Validate batch
    if payload.texts.is_empty() {
        return Err(ApiError::InvalidInput("Batch cannot be empty".to_string()));
    }

    if payload.texts.len() > MAX_BATCH_SIZE {
        return Err(ApiError::BatchTooLarge {
            max: MAX_BATCH_SIZE,
            actual: payload.texts.len(),
        });
    }

    // Perform trend analysis
    let start = Instant::now();
    let trend = state.sentiment_analyzer
        .analyze_trend(&payload.texts)
        .map_err(|e| ApiError::AnalysisFailed(e.to_string()))?;
    let processing_time = start.elapsed().as_millis() as u64;

    Ok(Json(TrendAnalysisResponse {
        id: generate_id(),
        timestamp: Utc::now(),
        trend,
        processing_time_ms: processing_time,
    }))
}

/// Root endpoint with API info
pub async fn root() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "name": "RustML Sentiment API",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "High-performance Rust-based Sentiment Analysis & Text Classification REST API",
        "endpoints": {
            "health": "GET /health",
            "stats": "GET /stats",
            "sentiment": "POST /api/v1/sentiment",
            "classify": "POST /api/v1/classify",
            "analyze": "POST /api/v1/analyze",
            "batch": "POST /api/v1/batch",
            "trend": "POST /api/v1/trend"
        },
        "documentation": "https://github.com/yourusername/rustml-sentiment-api"
    }))
}
