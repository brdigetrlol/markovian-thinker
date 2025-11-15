// Icarus MCP Server Binary
// Exposes Icarus cognitive system via Model Context Protocol

use icarus_core::IcarusMCPServer;
use anyhow::Result;

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

    // Create server with stdio wiring
    let (server, stdio, _reader_handle) = IcarusMCPServer::with_stdio();

    // Run server
    server.run_with_stdio(stdio).await?;

    Ok(())
}
