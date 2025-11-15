//! RustML Sentiment API - High-performance sentiment analysis and text classification REST API
//!
//! This application provides a production-ready API for:
//! - Sentiment analysis (positive/negative/neutral)
//! - Text classification into categories
//! - Batch processing
//! - Trend analysis
//!
//! Built with Rust, Axum, and machine learning algorithms.

mod error;
mod models;
mod routes;
mod sentiment_analyzer;
mod text_classifier;

use anyhow::Result;
use axum::{
    http::{
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
        HeaderValue, Method,
    },
    routing::{get, post},
    Router,
};
use sentiment_analyzer::SentimentAnalyzer;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use text_classifier::TextClassifier;
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::{info, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Application state shared across all request handlers
pub struct AppState<'a> {
    /// Sentiment analyzer instance
    sentiment_analyzer: SentimentAnalyzer<'a>,
    /// Text classifier instance
    text_classifier: TextClassifier,
    /// Server start time for uptime tracking
    start_time: Instant,
    /// Request counter for statistics
    request_count: AtomicU64,
    /// Text analysis counter
    text_count: AtomicU64,
}

impl<'a> AppState<'a> {
    /// Create new application state
    fn new() -> Result<Self> {
        Ok(Self {
            sentiment_analyzer: SentimentAnalyzer::new()?,
            text_classifier: TextClassifier::new(),
            start_time: Instant::now(),
            request_count: AtomicU64::new(0),
            text_count: AtomicU64::new(0),
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rustml_sentiment_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("🚀 Starting RustML Sentiment API v{}", env!("CARGO_PKG_VERSION"));

    // Create application state
    let state = Arc::new(AppState::new()?);
    info!("✅ Initialized ML models");

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin("*".parse::<HeaderValue>()?)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    // Build API routes
    let app = Router::new()
        // Root endpoint
        .route("/", get(routes::root))
        // Health and stats
        .route("/health", get(routes::health_check))
        .route("/stats", get(routes::get_stats))
        // API v1 endpoints
        .route("/api/v1/sentiment", post(routes::analyze_sentiment))
        .route("/api/v1/classify", post(routes::classify_text))
        .route("/api/v1/analyze", post(routes::analyze_combined))
        .route("/api/v1/batch", post(routes::analyze_batch))
        .route("/api/v1/trend", post(routes::analyze_trend))
        // Add state
        .with_state(state)
        // Add middleware
        .layer(CompressionLayer::new())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(cors);

    // Bind to address
    let addr = std::env::var("BIND_ADDRESS")
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string());

    info!("🎯 Server listening on http://{}", addr);
    info!("📖 API documentation available at http://{}/", addr);
    info!("");
    info!("Available endpoints:");
    info!("  GET  /health           - Health check");
    info!("  GET  /stats            - API statistics");
    info!("  POST /api/v1/sentiment - Analyze sentiment");
    info!("  POST /api/v1/classify  - Classify text");
    info!("  POST /api/v1/analyze   - Combined analysis");
    info!("  POST /api/v1/batch     - Batch analysis");
    info!("  POST /api/v1/trend     - Trend analysis");
    info!("");

    // Start server
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
