//! Error handling for the API

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;

use crate::models::ErrorResponse;

/// Application-specific errors
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Text too long: maximum length is {max}, got {actual}")]
    TextTooLong { max: usize, actual: usize },

    #[error("Batch size too large: maximum is {max}, got {actual}")]
    BatchTooLarge { max: usize, actual: usize },

    #[error("Analysis failed: {0}")]
    AnalysisFailed(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match self {
            ApiError::InvalidInput(msg) => (
                StatusCode::BAD_REQUEST,
                "INVALID_INPUT",
                msg,
            ),
            ApiError::TextTooLong { max, actual } => (
                StatusCode::BAD_REQUEST,
                "TEXT_TOO_LONG",
                format!("Text length {} exceeds maximum of {}", actual, max),
            ),
            ApiError::BatchTooLarge { max, actual } => (
                StatusCode::BAD_REQUEST,
                "BATCH_TOO_LARGE",
                format!("Batch size {} exceeds maximum of {}", actual, max),
            ),
            ApiError::AnalysisFailed(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "ANALYSIS_FAILED",
                msg,
            ),
            ApiError::RateLimitExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                "RATE_LIMIT_EXCEEDED",
                "Too many requests. Please try again later.".to_string(),
            ),
            ApiError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                msg,
            ),
        };

        let error_response = ErrorResponse::new(
            error_code.to_string(),
            message,
            None,
        );

        (status, Json(error_response)).into_response()
    }
}

/// Result type alias for API operations
pub type ApiResult<T> = Result<T, ApiError>;
