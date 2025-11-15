// Simplified Stdio Communication for MCP
// Handles basic read/write for server-side request handling

use super::protocol::*;
use anyhow::Result;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;

/// Simple stdio handler for MCP server communication
pub struct StdioHandler {
    /// Channel to send outgoing responses
    tx_outgoing: mpsc::UnboundedSender<JsonRpcResponse>,

    /// Channel to receive incoming requests
    rx_incoming: Arc<Mutex<mpsc::UnboundedReceiver<JsonRpcRequest>>>,
}

impl StdioHandler {
    /// Create new stdio handler
    /// Returns (handler, reader_task_handle)
    pub fn new() -> (Self, JoinHandle<()>) {
        let (tx_out, rx_out) = mpsc::unbounded_channel();
        let (tx_in_req, rx_in_req) = mpsc::unbounded_channel();

        // Spawn writer task (stdout)
        let _writer_handle = tokio::spawn(Self::writer_task(rx_out));

        // Spawn reader task (stdin)
        let reader_handle = tokio::spawn(Self::reader_task(tx_in_req));

        let handler = Self {
            tx_outgoing: tx_out,
            rx_incoming: Arc::new(Mutex::new(rx_in_req)),
        };

        (handler, reader_handle)
    }

    /// Writer task: reads from channel, writes to stdout
    async fn writer_task(mut rx_out: mpsc::UnboundedReceiver<JsonRpcResponse>) {
        let mut stdout = tokio::io::stdout();

        while let Some(response) = rx_out.recv().await {
            let json = match serde_json::to_string(&response) {
                Ok(j) => j,
                Err(e) => {
                    tracing::error!("Failed to serialize response: {}", e);
                    continue;
                }
            };

            tracing::trace!("STDOUT → {}", json);

            if let Err(e) = stdout.write_all(json.as_bytes()).await {
                tracing::error!("Failed to write to stdout: {}", e);
                break;
            }

            if let Err(e) = stdout.write_all(b"\n").await {
                tracing::error!("Failed to write newline: {}", e);
                break;
            }

            if let Err(e) = stdout.flush().await {
                tracing::error!("Failed to flush stdout: {}", e);
                break;
            }
        }

        tracing::info!("Writer task exiting");
    }

    /// Reader task: reads from stdin, forwards requests
    async fn reader_task(tx_in_req: mpsc::UnboundedSender<JsonRpcRequest>) {
        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        loop {
            line.clear();

            let bytes_read = match reader.read_line(&mut line).await {
                Ok(n) => n,
                Err(e) => {
                    tracing::error!("Failed to read from stdin: {}", e);
                    break;
                }
            };

            if bytes_read == 0 {
                // EOF
                tracing::info!("Stdin EOF, reader exiting");
                break;
            }

            let line_trimmed = line.trim();
            if line_trimmed.is_empty() {
                continue;
            }

            tracing::trace!("STDIN ← {}", line_trimmed);

            // Parse as request
            match serde_json::from_str::<JsonRpcRequest>(line_trimmed) {
                Ok(request) => {
                    if let Err(e) = tx_in_req.send(request) {
                        tracing::error!("Failed to forward request: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to parse request: {}", e);
                }
            }
        }

        tracing::info!("Reader task exiting");
    }

    /// Send a response (for incoming requests)
    pub fn send_response(&self, response: JsonRpcResponse) -> Result<()> {
        self.tx_outgoing
            .send(response)
            .map_err(|_| anyhow::anyhow!("Failed to send response (channel closed)"))?;
        Ok(())
    }

    /// Receive the next incoming request
    pub async fn recv_request(&self) -> Option<JsonRpcRequest> {
        self.rx_incoming.lock().await.recv().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stdio_handler_creation() {
        let (handler, _reader_handle) = StdioHandler::new();
        // Just verify we can create the handler
        drop(handler);
    }
}
