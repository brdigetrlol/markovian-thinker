// Markovian Thinker MCP Server
// Stdio-based MCP server for chunk-based reasoning with Claude Code

use markovian_thinker::{MarkovianMCPServer, ModelConfig, InferenceModel, OnlineLearner, LearningConfig};
use anyhow::Result;
use std::sync::{Arc, RwLock};

#[tokio::main]
async fn main() -> Result<()> {
    // Setup logging to STDERR (stdout is for MCP JSON messages!)
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_writer(std::io::stderr)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .init();

    tracing::info!("Initializing Markovian Thinker MCP Server");

    // Initialize model with default configuration
    let model_config = ModelConfig::default();

    #[cfg(feature = "gpu")]
    let model = InferenceModel::new(model_config, None)?;

    #[cfg(not(feature = "gpu"))]
    let model = InferenceModel::new(model_config, ())?;

    let model = Arc::new(RwLock::new(model));

    // Initialize online learner
    let learning_config = LearningConfig::default();
    let learner = OnlineLearner::new(learning_config, model.clone());
    let learner = Arc::new(RwLock::new(learner));

    // Create server with stdio wiring
    let (server, stdio, _reader_handle) = MarkovianMCPServer::with_stdio(model, learner);

    tracing::info!("Server initialized, starting event loop");

    // Run server
    server.run_with_stdio(stdio).await?;

    Ok(())
}
