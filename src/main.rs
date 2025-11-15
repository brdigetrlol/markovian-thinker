// Markovian Thinker MCP Server
// Stdio-based MCP server for chunk-based reasoning with Claude Code

use markovian_thinker::MarkovianMCPServer;
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
    let (server, stdio, _reader_handle) = MarkovianMCPServer::with_stdio();

    // Run server
    server.run_with_stdio(stdio).await?;

    Ok(())
}
