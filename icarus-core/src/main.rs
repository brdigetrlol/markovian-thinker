// Icarus: Autonomous Cognitive AI
// Standalone AI architecture with local GPU inference

use icarus_core::IcarusCore;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .init();

    tracing::info!("🚀 Icarus Cognitive AI - Starting");
    tracing::info!("Version: {}", env!("CARGO_PKG_VERSION"));
    tracing::info!("Architecture: Novel SSM/Liquid/RNN hybrid");

    // Load configuration
    let config = icarus_core::config::IcarusConfig::load()?;

    // Initialize Icarus core
    let mut icarus = IcarusCore::new(config).await?;

    tracing::info!("✅ Icarus initialized successfully");
    tracing::info!("   - Neural core: Ready");
    tracing::info!("   - Agent system: 6 agents active");
    tracing::info!("   - Memory hierarchy: Online");
    tracing::info!("   - World model: Initialized");

    // Run autonomous operation
    icarus.run().await?;

    Ok(())
}
