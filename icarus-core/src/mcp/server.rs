// Icarus MCP Server - Exposes Icarus cognitive system via Model Context Protocol

use super::protocol::*;
use super::stdio::StdioHandler;
use crate::{AgentSystem, MemoryHierarchy, NeuralCore, WorldModel, IcarusCore, IcarusConfig};
use anyhow::Result;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

/// MCP Server for Icarus cognitive system
pub struct IcarusMCPServer {
    /// Server info
    server_info: Implementation,
    /// Icarus core instance (optional - created on first use)
    icarus: Arc<RwLock<Option<IcarusCore>>>,
    /// Server initialized flag
    initialized: bool,
}

impl IcarusMCPServer {
    /// Create new Icarus MCP server
    pub fn new() -> Self {
        Self {
            server_info: Implementation {
                name: "icarus".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            icarus: Arc::new(RwLock::new(None)),
            initialized: false,
        }
    }

    /// Create server wired up with stdio
    pub fn with_stdio() -> (Self, Arc<StdioHandler>, tokio::task::JoinHandle<()>) {
        let (stdio, reader_handle) = StdioHandler::new();
        let stdio_arc = Arc::new(stdio);
        let server = Self::new();
        (server, stdio_arc, reader_handle)
    }

    /// Run server with stdio
    pub async fn run_with_stdio(mut self, stdio: Arc<StdioHandler>) -> Result<()> {
        tracing::info!("🧠 Icarus Cognitive System MCP Server");
        tracing::info!("Version: {}", self.server_info.version);
        tracing::info!("Waiting for initialize...");

        loop {
            let request = match stdio.recv_request().await {
                Some(req) => req,
                None => {
                    tracing::info!("Stdio closed, server exiting");
                    break;
                }
            };

            tracing::debug!("Received request: {} (id: {:?})", request.method, request.id);

            let is_notification = request.id.is_none();
            let response = self.handle_request(request).await;

            if !is_notification {
                stdio.send_response(response)?;
            }
        }

        Ok(())
    }

    /// Handle incoming MCP request
    async fn handle_request(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request),
            "initialized" => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: None,
                result: None,
                error: None,
            },
            "tools/list" => self.handle_list_tools(request),
            "tools/call" => self.handle_call_tool(request).await,
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

    /// Handle initialize request
    fn handle_initialize(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        tracing::info!("Handling initialize request");
        self.initialized = true;

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
            server_info: self.server_info.clone(),
        };

        JsonRpcResponse::success(request.id, serde_json::to_value(result).unwrap())
    }

    /// Handle tools/list request
    fn handle_list_tools(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let tools = vec![
            Tool {
                name: "icarus_query_status".to_string(),
                description: "Query the overall status of the Icarus cognitive system including uptime, agent states, memory usage, and event statistics.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            },
            Tool {
                name: "icarus_query_agents".to_string(),
                description: "Query detailed status of Icarus agents (Perception, WorldModel, Planning, Memory, Action, Learning).".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "agent_type": {
                            "type": "string",
                            "description": "Specific agent to query (optional). Options: perception, world_model, planning, memory, action, learning",
                            "enum": ["perception", "world_model", "planning", "memory", "action", "learning"]
                        }
                    },
                    "required": []
                }),
            },
            Tool {
                name: "icarus_send_event".to_string(),
                description: "Send an event to the Icarus event bus for agent processing.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "event_type": {
                            "type": "string",
                            "description": "Type of event to send"
                        },
                        "data": {
                            "type": "object",
                            "description": "Event data payload"
                        }
                    },
                    "required": ["event_type"]
                }),
            },
            Tool {
                name: "icarus_query_memory".to_string(),
                description: "Query Icarus hierarchical memory (working, short-term, long-term, episodic).".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "level": {
                            "type": "string",
                            "description": "Memory level to query",
                            "enum": ["working", "short_term", "long_term", "episodic", "all"]
                        },
                        "query": {
                            "type": "string",
                            "description": "Optional semantic search query"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum results to return",
                            "default": 10
                        }
                    },
                    "required": ["level"]
                }),
            },
            Tool {
                name: "icarus_query_world_model".to_string(),
                description: "Query the current state of Icarus's world model and get predictions.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "include_predictions": {
                            "type": "boolean",
                            "description": "Include future state predictions",
                            "default": false
                        },
                        "prediction_steps": {
                            "type": "integer",
                            "description": "Number of prediction steps (if predictions enabled)",
                            "default": 5
                        }
                    },
                    "required": []
                }),
            },
            Tool {
                name: "icarus_execute_action".to_string(),
                description: "Request Icarus to execute an action via the Action agent.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "action_type": {
                            "type": "string",
                            "description": "Type of action to execute"
                        },
                        "parameters": {
                            "type": "object",
                            "description": "Action parameters"
                        }
                    },
                    "required": ["action_type"]
                }),
            },
            Tool {
                name: "icarus_neural_state".to_string(),
                description: "Query the neural core state (SSM, Liquid, RNN layers).".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "include_hidden_state": {
                            "type": "boolean",
                            "description": "Include detailed hidden state activations",
                            "default": false
                        }
                    },
                    "required": []
                }),
            },
        ];

        let result = ListToolsResult { tools };
        JsonRpcResponse::success(request.id, serde_json::to_value(result).unwrap())
    }

    /// Handle tools/call request
    async fn handle_call_tool(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let params: CallToolParams = match request.params.as_ref().and_then(|p| serde_json::from_value(p.clone()).ok()) {
            Some(p) => p,
            None => {
                return JsonRpcResponse::error(
                    request.id,
                    JsonRpcError::invalid_params("Missing or invalid params"),
                );
            }
        };

        let result = match params.name.as_str() {
            "icarus_query_status" => self.handle_query_status(params.arguments).await,
            "icarus_query_agents" => self.handle_query_agents(params.arguments).await,
            "icarus_send_event" => self.handle_send_event(params.arguments).await,
            "icarus_query_memory" => self.handle_query_memory(params.arguments).await,
            "icarus_query_world_model" => self.handle_query_world_model(params.arguments).await,
            "icarus_execute_action" => self.handle_execute_action(params.arguments).await,
            "icarus_neural_state" => self.handle_neural_state(params.arguments).await,
            _ => CallToolResult {
                content: vec![Content::Text {
                    text: format!("Unknown tool: {}", params.name),
                }],
                is_error: Some(true),
            },
        };

        JsonRpcResponse::success(request.id, serde_json::to_value(result).unwrap())
    }

    /// Handle resources/list request
    fn handle_list_resources(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let result = ListResourcesResult {
            resources: vec![],
        };
        JsonRpcResponse::success(request.id, serde_json::to_value(result).unwrap())
    }

    // ========================================================================
    // Tool Implementations
    // ========================================================================

    async fn handle_query_status(&self, _args: serde_json::Value) -> CallToolResult {
        // TODO: Implement actual status query
        let status = json!({
            "running": false,
            "uptime_seconds": 0,
            "agents": {
                "perception": "not_started",
                "world_model": "not_started",
                "planning": "not_started",
                "memory": "not_started",
                "action": "not_started",
                "learning": "not_started"
            },
            "memory": {
                "working": 0,
                "short_term": 0,
                "long_term": 0,
                "episodic": 0
            },
            "events_processed": 0,
            "note": "Icarus cognitive system is not yet fully initialized. Implementation in progress."
        });

        CallToolResult {
            content: vec![Content::Text {
                text: serde_json::to_string_pretty(&status).unwrap(),
            }],
            is_error: Some(false),
        }
    }

    async fn handle_query_agents(&self, _args: serde_json::Value) -> CallToolResult {
        // TODO: Implement actual agent query
        CallToolResult {
            content: vec![Content::Text {
                text: "Agent query not yet implemented. Building agent intelligence...".to_string(),
            }],
            is_error: Some(false),
        }
    }

    async fn handle_send_event(&self, _args: serde_json::Value) -> CallToolResult {
        // TODO: Implement event sending
        CallToolResult {
            content: vec![Content::Text {
                text: "Event sending not yet implemented.".to_string(),
            }],
            is_error: Some(false),
        }
    }

    async fn handle_query_memory(&self, _args: serde_json::Value) -> CallToolResult {
        // TODO: Implement memory query
        CallToolResult {
            content: vec![Content::Text {
                text: "Memory query not yet implemented. Integrating vector database...".to_string(),
            }],
            is_error: Some(false),
        }
    }

    async fn handle_query_world_model(&self, _args: serde_json::Value) -> CallToolResult {
        // TODO: Implement world model query
        CallToolResult {
            content: vec![Content::Text {
                text: "World model query not yet implemented.".to_string(),
            }],
            is_error: Some(false),
        }
    }

    async fn handle_execute_action(&self, _args: serde_json::Value) -> CallToolResult {
        // TODO: Implement action execution
        CallToolResult {
            content: vec![Content::Text {
                text: "Action execution not yet implemented.".to_string(),
            }],
            is_error: Some(false),
        }
    }

    async fn handle_neural_state(&self, _args: serde_json::Value) -> CallToolResult {
        // TODO: Implement neural state query
        CallToolResult {
            content: vec![Content::Text {
                text: "Neural state query not yet implemented. Building neural core...".to_string(),
            }],
            is_error: Some(false),
        }
    }
}
