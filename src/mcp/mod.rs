// MCP Module
// Model Context Protocol implementation for Markovian reasoning (stateful mode)

pub mod protocol;
pub mod stdio;

pub use protocol::*;
pub use stdio::StdioHandler;
use anyhow::Result;
use serde_json::json;

// Minimal MCP Server implementation
pub struct MarkovianMCPServer;

impl MarkovianMCPServer {
    pub fn with_stdio() -> (Self, StdioHandler, tokio::task::JoinHandle<()>) {
        let (stdio, reader_handle) = StdioHandler::new();
        (Self, stdio, reader_handle)
    }

    pub async fn run_with_stdio(self, stdio: StdioHandler) -> Result<()> {
        tracing::info!("Markovian Thinker MCP Server starting...");

        while let Some(request) = stdio.recv_request().await {
            tracing::debug!("Received request: {} (id: {:?})", request.method, request.id);

            let response = self.handle_request(request).await;
            stdio.send_response(response)?;
        }

        tracing::info!("Server shutting down");
        Ok(())
    }

    async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request),
            "initialized" => {
                // Notification - no response needed
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: None,
                };
            }
            "tools/list" => self.handle_list_tools(request),
            "resources/list" => self.handle_list_resources(request),
            method => {
                tracing::warn!("Unknown method: {}", method);
                JsonRpcResponse::error(
                    request.id,
                    JsonRpcError::method_not_found(method),
                )
            }
        }
    }

    fn handle_initialize(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        tracing::info!("Handling initialize request");

        let result = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability {
                    list_changed: Some(false),
                }),
                resources: Some(ResourcesCapability {
                    subscribe: Some(false),
                    list_changed: Some(false),
                }),
                prompts: None,
            },
            server_info: Implementation {
                name: "markovian-thinker".to_string(),
                version: "0.1.0".to_string(),
            },
        };

        JsonRpcResponse::success(request.id, serde_json::to_value(result).unwrap())
    }

    fn handle_list_tools(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        tracing::info!("Handling tools/list request");

        let tools = vec![
            Tool {
                name: "markovian_think".to_string(),
                description: "Perform chunk-based Markovian reasoning on a complex problem. Uses fixed-size reasoning chunks with bounded carryover for linear complexity scaling.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "problem": {
                            "type": "string",
                            "description": "The problem or question to analyze using Markovian reasoning"
                        },
                        "max_iterations": {
                            "type": "number",
                            "description": "Maximum number of reasoning chunks (default: 5)",
                            "default": 5
                        }
                    },
                    "required": ["problem"]
                }),
            },
        ];

        let result = ListToolsResult { tools };
        JsonRpcResponse::success(request.id, serde_json::to_value(result).unwrap())
    }

    fn handle_list_resources(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        tracing::info!("Handling resources/list request");

        let result = ListResourcesResult {
            resources: vec![],
        };
        JsonRpcResponse::success(request.id, serde_json::to_value(result).unwrap())
    }
}
