// MCP Server Implementation
// Stdio-based MCP server for Markovian reasoning (stateful mode)

use super::protocol::*;
use super::stdio::StdioHandler;
use crate::lattice::LatticeType;
use crate::prompts;
use crate::session_manager::{SessionInfo, SessionManager};
use crate::state::{StateConfig, TerminationInfo};
use crate::storm_mitigation::StormMitigationConfig;
use crate::todo_bridge::{TodoBridge, TodoItem, TodoStatus};
use crate::trace::TerminationReason;
use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;
use uuid::Uuid;

/// MCP Server for Markovian reasoning
pub struct MarkovianMCPServer {
    /// Server info
    server_info: Implementation,
    /// Session manager
    session_manager: Arc<SessionManager>,
    /// Todo bridge for task tracking
    todo_bridge: TodoBridge,
    /// Whether server is initialized
    initialized: Arc<Mutex<bool>>,
}

impl MarkovianMCPServer {
    /// Create new MCP server
    pub fn new() -> Self {
        Self {
            server_info: Implementation {
                name: "markovian-thinker".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            session_manager: Arc::new(SessionManager::new()),
            todo_bridge: TodoBridge::new(),
            initialized: Arc::new(Mutex::new(false)),
        }
    }

    /// Create server wired up with stdio
    /// Returns (server, stdio_handler, reader_task_handle)
    pub fn with_stdio() -> (Self, Arc<StdioHandler>, tokio::task::JoinHandle<()>) {
        let (stdio, reader_handle) = StdioHandler::new();
        let stdio_arc = Arc::new(stdio);
        let server = Self::new();

        (server, stdio_arc, reader_handle)
    }

    /// Run server with stdio
    pub async fn run_with_stdio(self, stdio: Arc<StdioHandler>) -> Result<()> {
        tracing::info!("🧠 Markovian Thinker MCP Server (Stateful Mode)");
        tracing::info!("Version: {}", self.server_info.version);
        tracing::info!("Protocol: MCP over stdio");
        tracing::info!("Waiting for initialize...");

        loop {
            // Receive next request
            let request = match stdio.recv_request().await {
                Some(req) => req,
                None => {
                    tracing::info!("Stdio closed, server exiting");
                    break;
                }
            };

            tracing::debug!("Received request: {} (id: {:?})", request.method, request.id);

            // Detect notifications (requests without id)
            let is_notification = request.id.is_none();

            // Handle request
            let response = self.handle_request(request).await;

            // Send response only if not a notification
            // Per JSON-RPC 2.0 spec: servers MUST NOT reply to notifications
            if !is_notification {
                stdio.send_response(response)?;
            }
        }

        Ok(())
    }

    /// Run server (stdio event loop - simple mode)
    pub async fn run(&self) -> Result<()> {
        tracing::info!("Starting Markovian Thinker MCP Server");
        tracing::info!("Version: {}", self.server_info.version);
        tracing::info!("Protocol: stdio (JSON-RPC 2.0)");

        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        loop {
            line.clear();

            // Read next JSON-RPC message from stdin
            let bytes_read = reader
                .read_line(&mut line)
                .await
                .context("Failed to read from stdin")?;

            if bytes_read == 0 {
                // EOF - client disconnected
                tracing::info!("Client disconnected (EOF)");
                break;
            }

            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            tracing::trace!("Received: {}", line);

            // Parse JSON-RPC request
            let request: JsonRpcRequest = match serde_json::from_str(line) {
                Ok(req) => req,
                Err(e) => {
                    tracing::error!("Failed to parse request: {}", e);
                    let response = JsonRpcResponse::error(None, JsonRpcError::parse_error());
                    self.send_response(&mut stdout, &response).await?;
                    continue;
                }
            };

            // Handle request
            let response = self.handle_request(request).await;

            // Send response
            self.send_response(&mut stdout, &response).await?;
        }

        tracing::info!("Server shutting down");
        Ok(())
    }

    /// Handle incoming JSON-RPC request
    async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        tracing::debug!("Handling request: {}", request.method);

        // Route to appropriate handler
        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(&request).await,
            "initialized" => self.handle_initialized(&request).await,
            "tools/list" => self.handle_tools_list(&request).await,
            "tools/call" => self.handle_tools_call(&request).await,
            "resources/list" => self.handle_resources_list(&request).await,
            "resources/read" => self.handle_resources_read(&request).await,
            _ => Err(JsonRpcError::method_not_found(&request.method)),
        };

        match result {
            Ok(value) => JsonRpcResponse::success(request.id, value),
            Err(error) => JsonRpcResponse::error(request.id, error),
        }
    }

    /// Send JSON-RPC response to stdout
    async fn send_response(
        &self,
        stdout: &mut tokio::io::Stdout,
        response: &JsonRpcResponse,
    ) -> Result<()> {
        let json = serde_json::to_string(response)?;
        tracing::trace!("Sending: {}", json);
        stdout.write_all(json.as_bytes()).await?;
        stdout.write_all(b"\n").await?;
        stdout.flush().await?;
        Ok(())
    }

    // ========================================================================
    // Request Handlers
    // ========================================================================

    async fn handle_initialize(&self, request: &JsonRpcRequest) -> Result<serde_json::Value, JsonRpcError> {
        let _params: InitializeParams = serde_json::from_value(
            request.params.clone().unwrap_or(serde_json::json!({})),
        )
        .map_err(|e| JsonRpcError::invalid_params(&e.to_string()))?;

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

        *self.initialized.lock().await = true;
        tracing::info!("Server initialized");

        serde_json::to_value(&result).map_err(|e| JsonRpcError::internal_error(&e.to_string()))
    }

    async fn handle_initialized(&self, _request: &JsonRpcRequest) -> Result<serde_json::Value, JsonRpcError> {
        // Notification that client has finished initialization
        tracing::debug!("Client initialized");
        Ok(serde_json::json!({}))
    }

    async fn handle_tools_list(&self, _request: &JsonRpcRequest) -> Result<serde_json::Value, JsonRpcError> {
        let tools = vec![
            Tool {
                name: "markovian_init_session".to_string(),
                description: "Initialize a new Markovian reasoning session. Returns a session_id for use in subsequent calls.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "problem": {
                            "type": "string",
                            "description": "The problem or question to solve"
                        },
                        "chunk_size": {
                            "type": "integer",
                            "description": "Maximum tokens per chunk (default: 8192)",
                            "default": 8192
                        },
                        "carryover_size": {
                            "type": "integer",
                            "description": "Tokens to carry between chunks (default: chunk_size/2)",
                            "default": 4096
                        },
                        "max_iterations": {
                            "type": "integer",
                            "description": "Maximum number of chunks (default: 5)",
                            "default": 5
                        },
                        "enable_event_driven": {
                            "type": "boolean",
                            "description": "Enable event-driven chunk processing (Icarus TIC) (default: false)",
                            "default": false
                        },
                        "enable_causal_trace": {
                            "type": "boolean",
                            "description": "Enable causal trace tracking for reasoning structure (default: false)",
                            "default": false
                        },
                        "lattice_type": {
                            "type": "string",
                            "description": "Concept lattice type: 'e8' (8D), 'leech' (24D), 'hcp-N', 'cubic-N' (default: 'e8')",
                            "default": "e8",
                            "enum": ["e8", "leech", "hcp-8", "hcp-16", "hcp-24", "cubic-8", "cubic-16", "cubic-32"]
                        },
                        "enable_storm_mitigation": {
                            "type": "boolean",
                            "description": "Enable storm mitigation (rate limiting, circuit breaker, event fusion) (default: true)",
                            "default": true
                        },
                        "storm_mitigation_level": {
                            "type": "string",
                            "description": "Storm mitigation aggressiveness: 'aggressive', 'default', 'lenient', 'disabled' (default: 'default')",
                            "default": "default",
                            "enum": ["aggressive", "default", "lenient", "disabled"]
                        }
                    },
                    "required": ["problem"]
                }),
            },
            Tool {
                name: "markovian_get_prompt".to_string(),
                description: "Get the next prompt to reason about in a session. This returns the original query plus carryover context from the previous chunk.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "session_id": {
                            "type": "string",
                            "description": "Session UUID"
                        }
                    },
                    "required": ["session_id"]
                }),
            },
            Tool {
                name: "markovian_submit_chunk".to_string(),
                description: "Submit your reasoning chunk output to the session. The server will extract carryover context and check for termination conditions.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "session_id": {
                            "type": "string",
                            "description": "Session UUID"
                        },
                        "output": {
                            "type": "string",
                            "description": "Your reasoning output text"
                        },
                        "tokens": {
                            "type": "integer",
                            "description": "Approximate token count (optional, will be estimated if not provided)"
                        }
                    },
                    "required": ["session_id", "output"]
                }),
            },
            Tool {
                name: "markovian_get_trace".to_string(),
                description: "Get the complete reasoning trace for a session, including all chunks, the final solution, and causal trace (if enabled).".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "session_id": {
                            "type": "string",
                            "description": "Session UUID"
                        }
                    },
                    "required": ["session_id"]
                }),
            },
            Tool {
                name: "markovian_list_sessions".to_string(),
                description: "List all active Markovian reasoning sessions with their metadata.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            Tool {
                name: "markovian_get_metrics".to_string(),
                description: "Get storm mitigation metrics and statistics for a session.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "session_id": {
                            "type": "string",
                            "description": "Session UUID"
                        }
                    },
                    "required": ["session_id"]
                }),
            },
            Tool {
                name: "markovian_query_concepts".to_string(),
                description: "Query similar concepts in a session's concept space using an embedding vector.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "session_id": {
                            "type": "string",
                            "description": "Session UUID"
                        },
                        "embedding": {
                            "type": "array",
                            "items": {
                                "type": "number"
                            },
                            "description": "Embedding vector to query (must match lattice dimension)"
                        },
                        "k": {
                            "type": "integer",
                            "description": "Number of similar concepts to return (default: 5)",
                            "default": 5
                        }
                    },
                    "required": ["session_id", "embedding"]
                }),
            },
            Tool {
                name: "markovian_export_graphviz".to_string(),
                description: "Export the causal trace for a session as GraphViz DOT format for visualization.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "session_id": {
                            "type": "string",
                            "description": "Session UUID"
                        }
                    },
                    "required": ["session_id"]
                }),
            },
            Tool {
                name: "markovian_batch_init".to_string(),
                description: "Create multiple reasoning sessions at once with shared configuration.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "problems": {
                            "type": "array",
                            "description": "Array of problem statements",
                            "items": {
                                "type": "string"
                            }
                        },
                        "config": {
                            "type": "object",
                            "description": "Shared StateConfig for all sessions (optional)",
                            "properties": {
                                "chunk_size": {
                                    "type": "integer",
                                    "description": "Tokens per chunk"
                                },
                                "carryover_size": {
                                    "type": "integer",
                                    "description": "Tokens to carry between chunks"
                                },
                                "max_iterations": {
                                    "type": "integer",
                                    "description": "Maximum reasoning iterations"
                                }
                            }
                        }
                    },
                    "required": ["problems"]
                }),
            },
            Tool {
                name: "markovian_search_corpus".to_string(),
                description: "Search semantic corpus using H²CE multi-resolution retrieval during reasoning.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query string"
                        },
                        "max_results": {
                            "type": "integer",
                            "description": "Maximum number of results to return (default: 5)",
                            "default": 5
                        },
                        "resolution_level": {
                            "type": "string",
                            "enum": ["L0", "L1", "L2", "L4", "all"],
                            "description": "Resolution level: L0 (atomic), L1 (paragraph), L2 (summary), L4 (document), or 'all'",
                            "default": "all"
                        },
                        "similarity_threshold": {
                            "type": "number",
                            "description": "Minimum similarity threshold (0.0-1.0, default: 0.5)",
                            "default": 0.5
                        }
                    },
                    "required": ["query"]
                }),
            },
            Tool {
                name: "markovian_todo_set".to_string(),
                description: "Set the complete todo list for a session. This replaces any existing todos.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "session_id": {
                            "type": "string",
                            "description": "Session UUID"
                        },
                        "todos": {
                            "type": "array",
                            "description": "Array of todo items",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "content": {
                                        "type": "string",
                                        "description": "Todo description"
                                    },
                                    "status": {
                                        "type": "string",
                                        "enum": ["pending", "inprogress", "completed"],
                                        "description": "Current status"
                                    },
                                    "active_form": {
                                        "type": "string",
                                        "description": "Present continuous form (e.g., 'Analyzing code')"
                                    }
                                },
                                "required": ["content", "status", "active_form"]
                            }
                        }
                    },
                    "required": ["session_id", "todos"]
                }),
            },
            Tool {
                name: "markovian_todo_get".to_string(),
                description: "Get the todo list for a session with summary statistics.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "session_id": {
                            "type": "string",
                            "description": "Session UUID"
                        }
                    },
                    "required": ["session_id"]
                }),
            },
            Tool {
                name: "markovian_todo_add".to_string(),
                description: "Add a single todo item to a session's todo list.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "session_id": {
                            "type": "string",
                            "description": "Session UUID"
                        },
                        "content": {
                            "type": "string",
                            "description": "Todo description"
                        },
                        "active_form": {
                            "type": "string",
                            "description": "Present continuous form (e.g., 'Analyzing code')"
                        }
                    },
                    "required": ["session_id", "content", "active_form"]
                }),
            },
            Tool {
                name: "markovian_todo_update_status".to_string(),
                description: "Update the status of a specific todo item by index.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "session_id": {
                            "type": "string",
                            "description": "Session UUID"
                        },
                        "index": {
                            "type": "integer",
                            "description": "0-based index of the todo item"
                        },
                        "status": {
                            "type": "string",
                            "enum": ["pending", "inprogress", "completed"],
                            "description": "New status"
                        }
                    },
                    "required": ["session_id", "index", "status"]
                }),
            },
        ];

        let result = ListToolsResult { tools };
        serde_json::to_value(&result).map_err(|e| JsonRpcError::internal_error(&e.to_string()))
    }

    async fn handle_tools_call(&self, request: &JsonRpcRequest) -> Result<serde_json::Value, JsonRpcError> {
        let params: CallToolParams = serde_json::from_value(
            request.params.clone().unwrap_or(serde_json::json!({})),
        )
        .map_err(|e| JsonRpcError::invalid_params(&e.to_string()))?;

        let result = match params.name.as_str() {
            "markovian_init_session" => self.tool_init_session(params.arguments).await?,
            "markovian_get_prompt" => self.tool_get_prompt(params.arguments).await?,
            "markovian_submit_chunk" => self.tool_submit_chunk(params.arguments).await?,
            "markovian_get_trace" => self.tool_get_trace(params.arguments).await?,
            "markovian_list_sessions" => self.tool_list_sessions(params.arguments).await?,
            "markovian_get_metrics" => self.tool_get_metrics(params.arguments).await?,
            "markovian_query_concepts" => self.tool_query_concepts(params.arguments).await?,
            "markovian_export_graphviz" => self.tool_export_graphviz(params.arguments).await?,
            "markovian_batch_init" => self.tool_batch_init(params.arguments).await?,
            "markovian_search_corpus" => self.tool_search_corpus(params.arguments).await?,
            "markovian_todo_set" => self.tool_todo_set(params.arguments).await?,
            "markovian_todo_get" => self.tool_todo_get(params.arguments).await?,
            "markovian_todo_add" => self.tool_todo_add(params.arguments).await?,
            "markovian_todo_update_status" => self.tool_todo_update_status(params.arguments).await?,
            _ => {
                return Err(JsonRpcError::invalid_params(&format!(
                    "Unknown tool: {}",
                    params.name
                )))
            }
        };

        serde_json::to_value(&result).map_err(|e| JsonRpcError::internal_error(&e.to_string()))
    }

    async fn handle_resources_list(&self, _request: &JsonRpcRequest) -> Result<serde_json::Value, JsonRpcError> {
        // List available reasoning traces
        let sessions: Vec<SessionInfo> = self.session_manager.list_sessions().await;
        let resources: Vec<Resource> = sessions
            .iter()
            .map(|session| Resource {
                uri: format!("markovian://trace/{}", session.id),
                name: format!("Reasoning Trace {}", session.id),
                description: Some(format!(
                    "{} chunks, {} tokens{}",
                    session.iteration,
                    session.total_tokens,
                    if session.is_complete { " (complete)" } else { "" }
                )),
                mime_type: Some("application/json".to_string()),
            })
            .collect();

        let result = ListResourcesResult { resources };
        serde_json::to_value(&result).map_err(|e| JsonRpcError::internal_error(&e.to_string()))
    }

    async fn handle_resources_read(&self, request: &JsonRpcRequest) -> Result<serde_json::Value, JsonRpcError> {
        let params: ReadResourceParams = serde_json::from_value(
            request.params.clone().unwrap_or(serde_json::json!({})),
        )
        .map_err(|e| JsonRpcError::invalid_params(&e.to_string()))?;

        // Parse URI: markovian://trace/{id}
        if !params.uri.starts_with("markovian://trace/") {
            return Err(JsonRpcError::invalid_params("Invalid resource URI"));
        }

        let id_str = params.uri.trim_start_matches("markovian://trace/");
        let id = Uuid::parse_str(id_str)
            .map_err(|_| JsonRpcError::invalid_params("Invalid trace ID"))?;

        let session = self
            .session_manager
            .get_session(id)
            .await
            .map_err(|_| JsonRpcError::invalid_params("Trace not found"))?;

        let trace_json = serde_json::to_string_pretty(&session.trace)
            .map_err(|e| JsonRpcError::internal_error(&e.to_string()))?;

        let result = ReadResourceResult {
            contents: vec![ResourceContents {
                uri: params.uri,
                mime_type: Some("application/json".to_string()),
                content: Content::text(trace_json),
            }],
        };

        serde_json::to_value(&result).map_err(|e| JsonRpcError::internal_error(&e.to_string()))
    }

    // ========================================================================
    // Tool Implementations
    // ========================================================================

    async fn tool_init_session(&self, args: serde_json::Value) -> Result<CallToolResult, JsonRpcError> {
        #[derive(serde::Deserialize)]
        struct InitArgs {
            problem: String,
            #[serde(default = "default_chunk_size")]
            chunk_size: usize,
            #[serde(default)]
            carryover_size: Option<usize>,
            #[serde(default = "default_max_iterations")]
            max_iterations: usize,
            #[serde(default)]
            enable_event_driven: bool,
            #[serde(default)]
            enable_causal_trace: bool,
            #[serde(default = "default_lattice_type")]
            lattice_type: String,
            #[serde(default = "default_enable_storm_mitigation")]
            enable_storm_mitigation: bool,
            #[serde(default = "default_storm_mitigation_level")]
            storm_mitigation_level: String,
        }

        fn default_chunk_size() -> usize {
            8192
        }
        fn default_max_iterations() -> usize {
            5
        }
        fn default_lattice_type() -> String {
            "e8".to_string()
        }
        fn default_enable_storm_mitigation() -> bool {
            true
        }
        fn default_storm_mitigation_level() -> String {
            "default".to_string()
        }

        let args: InitArgs = serde_json::from_value(args)
            .map_err(|e| JsonRpcError::invalid_params(&e.to_string()))?;

        let carryover_size = args.carryover_size.unwrap_or(args.chunk_size / 2);

        // Create base configuration
        let mut config = StateConfig::new(args.chunk_size, carryover_size, args.max_iterations)
            .map_err(|e| JsonRpcError::invalid_params(&e))?;

        // Apply Phase 6 configurations
        config.enable_event_driven = args.enable_event_driven;
        config.enable_causal_trace = args.enable_causal_trace;

        // Parse lattice type
        config.concept_space_config.lattice_type = Self::parse_lattice_type(&args.lattice_type)
            .map_err(|e| JsonRpcError::invalid_params(&e.as_str()))?;

        // Configure storm mitigation
        if !args.enable_storm_mitigation || args.storm_mitigation_level == "disabled" {
            config.storm_mitigation_config = StormMitigationConfig::disabled();
        } else {
            config.storm_mitigation_config = match args.storm_mitigation_level.as_str() {
                "aggressive" => StormMitigationConfig::aggressive(),
                "lenient" => StormMitigationConfig::lenient(),
                _ => StormMitigationConfig::default(),
            };
        }

        // Create session
        let session_id = self
            .session_manager
            .create_session(args.problem.clone(), config.clone())
            .await
            .map_err(|e| JsonRpcError::internal_error(&e.to_string()))?;

        // Get session to access detected domain
        let session = self
            .session_manager
            .get_session(session_id)
            .await
            .map_err(|e| JsonRpcError::internal_error(&e.to_string()))?;

        tracing::info!(
            "Created session {} | Problem: {} chars | Domain: {:?} | Config: chunk_size={}, carryover={}, max_iterations={} | Features: event_driven={}, causal_trace={}, lattice={:?}, storm_mitigation={}",
            session_id,
            args.problem.len(),
            session.state.domain,
            config.chunk_size,
            config.carryover_size,
            config.max_iterations,
            config.enable_event_driven,
            config.enable_causal_trace,
            config.concept_space_config.lattice_type,
            args.enable_storm_mitigation
        );

        let response = serde_json::json!({
            "session_id": session_id.to_string(),
            "domain": session.state.domain.as_ref().map(|d| format!("{:?}", d)),
            "config": {
                "chunk_size": config.chunk_size,
                "carryover_size": config.carryover_size,
                "max_iterations": config.max_iterations,
                "token_budget": config.token_budget
            },
            "features": {
                "event_driven": config.enable_event_driven,
                "causal_trace": config.enable_causal_trace,
                "lattice_type": format!("{:?}", config.concept_space_config.lattice_type),
                "storm_mitigation": args.enable_storm_mitigation,
                "storm_mitigation_level": args.storm_mitigation_level
            },
            "message": "Session initialized with hybrid reasoning (GPT-OSS + Icarus TIC). Call markovian_get_prompt to begin reasoning."
        });

        Ok(CallToolResult {
            content: vec![Content::text(serde_json::to_string_pretty(&response).unwrap())],
            is_error: Some(false),
        })
    }

    async fn tool_get_prompt(&self, args: serde_json::Value) -> Result<CallToolResult, JsonRpcError> {
        #[derive(serde::Deserialize)]
        struct GetPromptArgs {
            session_id: String,
        }

        let args: GetPromptArgs = serde_json::from_value(args)
            .map_err(|e| JsonRpcError::invalid_params(&e.to_string()))?;

        let session_id = Uuid::parse_str(&args.session_id)
            .map_err(|_| JsonRpcError::invalid_params("Invalid session ID"))?;

        let session = self
            .session_manager
            .get_session(session_id)
            .await
            .map_err(|e| JsonRpcError::invalid_params(&e.to_string()))?;

        // Check if session is already complete
        if session.trace.is_complete() {
            return Err(JsonRpcError::invalid_params(
                "Session is already complete. Call markovian_get_trace to retrieve results.",
            ));
        }

        // Generate enhanced prompt with verification instructions
        let prompt = prompts::generate_prompt(&session.state, session.state.domain.as_ref());
        let config = session.state.config();

        let response = serde_json::json!({
            "iteration": session.state.iteration,
            "prompt": prompt,
            "max_tokens": config.chunk_size,
            "domain": session.state.domain.as_ref().map(|d| format!("{:?}", d)),
            "instructions": "Use the structured output format with [REASONING], [VERIFICATION], and [CARRYOVER] sections as specified in the prompt. Call markovian_submit_chunk when complete."
        });

        Ok(CallToolResult {
            content: vec![Content::text(serde_json::to_string_pretty(&response).unwrap())],
            is_error: Some(false),
        })
    }

    async fn tool_submit_chunk(&self, args: serde_json::Value) -> Result<CallToolResult, JsonRpcError> {
        #[derive(serde::Deserialize)]
        struct SubmitArgs {
            session_id: String,
            output: String,
            tokens: Option<usize>,
        }

        let args: SubmitArgs = serde_json::from_value(args)
            .map_err(|e| JsonRpcError::invalid_params(&e.to_string()))?;

        let session_id = Uuid::parse_str(&args.session_id)
            .map_err(|_| JsonRpcError::invalid_params("Invalid session ID"))?;

        let mut session = self
            .session_manager
            .get_session(session_id)
            .await
            .map_err(|e| JsonRpcError::invalid_params(&e.to_string()))?;

        // Check storm mitigation before processing
        use crate::storm_mitigation::MitigationDecision;
        match self
            .session_manager
            .check_storm_mitigation(session_id)
            .await
            .map_err(|e| JsonRpcError::internal_error(&e.to_string()))?
        {
            MitigationDecision::Allowed => {
                // Continue processing
            }
            MitigationDecision::Rejected { reason } => {
                let response = serde_json::json!({
                    "continue": false,
                    "error": "Storm mitigation rejected chunk",
                    "reason": reason,
                    "message": "Circuit breaker is open. System needs time to recover."
                });
                return Ok(CallToolResult {
                    content: vec![Content::text(serde_json::to_string_pretty(&response).unwrap())],
                    is_error: Some(true),
                });
            }
            MitigationDecision::RateLimited { retry_after } => {
                let response = serde_json::json!({
                    "continue": false,
                    "error": "Rate limited",
                    "retry_after_ms": retry_after.as_millis(),
                    "message": format!("Rate limit exceeded. Retry after {} ms", retry_after.as_millis())
                });
                return Ok(CallToolResult {
                    content: vec![Content::text(serde_json::to_string_pretty(&response).unwrap())],
                    is_error: Some(true),
                });
            }
        }

        // Estimate tokens if not provided (rough: 4 chars per token)
        let tokens = args.tokens.unwrap_or(args.output.len() / 4);

        // Record chunk
        self.session_manager
            .record_chunk(
                session_id,
                session.state.build_prompt(),
                args.output.clone(),
                tokens,
            )
            .await
            .map_err(|e| JsonRpcError::internal_error(&e.to_string()))?;

        // Check for termination and update state
        match session.state.update(&args.output, tokens) {
            Ok(TerminationInfo {
                should_terminate: true,
                reason,
                solution,
            }) => {
                // Session complete
                self.session_manager
                    .complete_session(session_id, solution.clone(), reason.clone())
                    .await
                    .map_err(|e| JsonRpcError::internal_error(&e.to_string()))?;

                let response = serde_json::json!({
                    "continue": false,
                    "reason": format!("{:?}", reason),
                    "solution": solution.unwrap_or_else(|| "N/A".to_string()),
                    "iteration": session.state.iteration,
                    "total_tokens": session.trace.total_tokens + tokens,
                    "message": "Reasoning complete! Call markovian_get_trace for the full trace."
                });

                tracing::info!(
                    "Session {} complete | Reason: {:?} | {} chunks",
                    session_id,
                    reason,
                    session.state.iteration
                );

                // Record successful chunk processing
                self.session_manager
                    .record_storm_success(session_id)
                    .await
                    .ok(); // Ignore error if storm mitigation not found

                Ok(CallToolResult {
                    content: vec![Content::text(serde_json::to_string_pretty(&response).unwrap())],
                    is_error: Some(false),
                })
            }
            Ok(TerminationInfo {
                should_terminate: false,
                ..
            }) => {
                // Continue reasoning
                let iteration = session.state.iteration;

                // Update the session
                self.session_manager
                    .update_session(session)
                    .await
                    .map_err(|e| JsonRpcError::internal_error(&e.to_string()))?;

                let response = serde_json::json!({
                    "continue": true,
                    "iteration": iteration,
                    "message": "Chunk recorded. Call markovian_get_prompt for the next iteration."
                });

                // Record successful chunk processing
                self.session_manager
                    .record_storm_success(session_id)
                    .await
                    .ok(); // Ignore error if storm mitigation not found

                Ok(CallToolResult {
                    content: vec![Content::text(serde_json::to_string_pretty(&response).unwrap())],
                    is_error: Some(false),
                })
            }
            Err(e) => {
                // Record failed chunk processing
                self.session_manager
                    .record_storm_failure(session_id)
                    .await
                    .ok(); // Ignore error if storm mitigation not found

                // Error during update
                self.session_manager
                    .complete_session(
                        session_id,
                        None,
                        TerminationReason::Error(e.to_string()),
                    )
                    .await
                    .map_err(|e| JsonRpcError::internal_error(&e.to_string()))?;

                Err(JsonRpcError::internal_error(&e))
            }
        }
    }

    async fn tool_get_trace(&self, args: serde_json::Value) -> Result<CallToolResult, JsonRpcError> {
        #[derive(serde::Deserialize)]
        struct GetTraceArgs {
            session_id: String,
        }

        let args: GetTraceArgs = serde_json::from_value(args)
            .map_err(|e| JsonRpcError::invalid_params(&e.to_string()))?;

        let session_id = Uuid::parse_str(&args.session_id)
            .map_err(|_| JsonRpcError::invalid_params("Invalid session ID"))?;

        let session = self
            .session_manager
            .get_session(session_id)
            .await
            .map_err(|e| JsonRpcError::invalid_params(&e.to_string()))?;

        // Try to get causal trace if available
        let causal_trace = self.session_manager.get_causal_trace(session_id).await.ok();

        // Build combined response with both traces
        let response = if let Some(causal) = causal_trace {
            serde_json::json!({
                "session_id": session_id.to_string(),
                "reasoning_trace": session.trace,
                "causal_trace": causal,
            })
        } else {
            serde_json::json!({
                "session_id": session_id.to_string(),
                "reasoning_trace": session.trace,
                "causal_trace": null,
                "note": "Causal trace not enabled for this session. Set enable_causal_trace: true when creating session."
            })
        };

        let trace_json = serde_json::to_string_pretty(&response)
            .map_err(|e| JsonRpcError::internal_error(&e.to_string()))?;

        Ok(CallToolResult {
            content: vec![Content::text(trace_json)],
            is_error: Some(false),
        })
    }

    async fn tool_list_sessions(&self, _args: serde_json::Value) -> Result<CallToolResult, JsonRpcError> {
        let sessions = self.session_manager.list_sessions().await;

        if sessions.is_empty() {
            Ok(CallToolResult {
                content: vec![Content::text("No active Markovian reasoning sessions.".to_string())],
                is_error: Some(false),
            })
        } else {
            let sessions_json = serde_json::to_string_pretty(&sessions)
                .map_err(|e| JsonRpcError::internal_error(&e.to_string()))?;

            Ok(CallToolResult {
                content: vec![Content::text(sessions_json)],
                is_error: Some(false),
            })
        }
    }

    async fn tool_get_metrics(&self, args: serde_json::Value) -> Result<CallToolResult, JsonRpcError> {
        #[derive(serde::Deserialize)]
        struct GetMetricsArgs {
            session_id: String,
        }

        let args: GetMetricsArgs = serde_json::from_value(args)
            .map_err(|e| JsonRpcError::invalid_params(&e.to_string()))?;

        let session_id = Uuid::parse_str(&args.session_id)
            .map_err(|_| JsonRpcError::invalid_params("Invalid session ID"))?;

        // Get storm mitigation stats
        let stats = self
            .session_manager
            .get_storm_stats(session_id)
            .await
            .map_err(|e| JsonRpcError::invalid_params(&e.to_string()))?;

        // Build response with detailed metrics
        let response = serde_json::json!({
            "session_id": session_id.to_string(),
            "storm_mitigation": {
                "circuit_state": format!("{:?}", stats.circuit_state),
                "metrics": {
                    "total_checks": stats.metrics.total_checks,
                    "allowed_events": stats.metrics.allowed_events,
                    "rate_limit_rejections": stats.metrics.rate_limit_rejections,
                    "circuit_breaker_rejections": stats.metrics.circuit_breaker_rejections,
                    "successful_events": stats.metrics.successful_events,
                    "failed_events": stats.metrics.failed_events,
                    "total_fusions": stats.metrics.total_fusions,
                    "events_fused": stats.metrics.events_fused,
                    "emergency_stops": stats.metrics.emergency_stops,
                    "success_rate": stats.metrics.success_rate(),
                    "rejection_rate": stats.metrics.rejection_rate(),
                    "fusion_effectiveness": stats.metrics.fusion_effectiveness(),
                }
            }
        });

        Ok(CallToolResult {
            content: vec![Content::text(serde_json::to_string_pretty(&response).unwrap())],
            is_error: Some(false),
        })
    }

    async fn tool_query_concepts(&self, args: serde_json::Value) -> Result<CallToolResult, JsonRpcError> {
        #[derive(serde::Deserialize)]
        struct QueryConceptsArgs {
            session_id: String,
            embedding: Vec<f32>,
            #[serde(default = "default_k")]
            k: usize,
        }

        fn default_k() -> usize {
            5
        }

        let args: QueryConceptsArgs = serde_json::from_value(args)
            .map_err(|e| JsonRpcError::invalid_params(&e.to_string()))?;

        let session_id = Uuid::parse_str(&args.session_id)
            .map_err(|_| JsonRpcError::invalid_params("Invalid session ID"))?;

        // Query similar concepts
        let concepts = self
            .session_manager
            .query_concepts(session_id, args.embedding, args.k)
            .await
            .map_err(|e| JsonRpcError::invalid_params(&e.to_string()))?;

        // Get concept space statistics
        let stats = self
            .session_manager
            .get_concept_stats(session_id)
            .await
            .map_err(|e| JsonRpcError::invalid_params(&e.to_string()))?;

        // Build response
        let response = serde_json::json!({
            "session_id": session_id.to_string(),
            "similar_concepts": concepts,
            "statistics": {
                "total_concepts": stats.total_concepts,
                "lattice_type": format!("{:?}", stats.lattice_type),
                "dimension": stats.dimension,
                "avg_norm": stats.avg_norm,
                "max_norm": stats.max_norm,
                "min_norm": stats.min_norm,
            }
        });

        Ok(CallToolResult {
            content: vec![Content::text(serde_json::to_string_pretty(&response).unwrap())],
            is_error: Some(false),
        })
    }

    async fn tool_export_graphviz(&self, args: serde_json::Value) -> Result<CallToolResult, JsonRpcError> {
        #[derive(serde::Deserialize)]
        struct ExportGraphvizArgs {
            session_id: String,
        }

        let args: ExportGraphvizArgs = serde_json::from_value(args)
            .map_err(|e| JsonRpcError::invalid_params(&e.to_string()))?;

        let session_id = Uuid::parse_str(&args.session_id)
            .map_err(|_| JsonRpcError::invalid_params("Invalid session ID"))?;

        // Get causal trace
        let causal_trace = self
            .session_manager
            .get_causal_trace(session_id)
            .await
            .map_err(|e| JsonRpcError::invalid_params(&e.to_string()))?;

        // Export to GraphViz DOT format
        let dot = causal_trace.to_graphviz();

        // Build response with DOT format and metadata
        let metadata = causal_trace.metadata();
        let stats = causal_trace.statistics();

        let response = serde_json::json!({
            "session_id": session_id.to_string(),
            "graphviz_dot": dot,
            "metadata": {
                "session_id": metadata.session_id.to_string(),
                "total_events": metadata.total_events,
                "created_at": metadata.created_at.to_rfc3339(),
                "updated_at": metadata.updated_at.to_rfc3339(),
            },
            "statistics": {
                "total_events": stats.total_events,
                "micro_events": stats.micro_events,
                "meso_events": stats.meso_events,
                "macro_events": stats.macro_events,
                "total_branches": stats.total_branches,
                "active_branches": stats.active_branches,
                "total_edges": stats.total_edges,
                "avg_depth": stats.avg_depth,
                "max_depth": stats.max_depth,
                "has_cycles": stats.has_cycles,
            },
            "note": "Use 'dot' command to render: dot -Tpng output.dot -o output.png"
        });

        Ok(CallToolResult {
            content: vec![Content::text(serde_json::to_string_pretty(&response).unwrap())],
            is_error: Some(false),
        })
    }

    async fn tool_batch_init(&self, args: serde_json::Value) -> Result<CallToolResult, JsonRpcError> {
        #[derive(serde::Deserialize)]
        struct BatchInitArgs {
            problems: Vec<String>,
            config: Option<serde_json::Value>,
        }

        let args: BatchInitArgs = serde_json::from_value(args)
            .map_err(|e| JsonRpcError::invalid_params(&e.to_string()))?;

        if args.problems.is_empty() {
            return Err(JsonRpcError::invalid_params("Problems array cannot be empty"));
        }

        // Parse optional config
        let config = if let Some(config_value) = args.config {
            serde_json::from_value::<StateConfig>(config_value)
                .map_err(|e| JsonRpcError::invalid_params(&format!("Invalid config: {}", e)))?
        } else {
            StateConfig::default()
        };

        // Create sessions for all problems
        let mut session_ids = Vec::new();
        for problem in &args.problems {
            let session_id = self
                .session_manager
                .create_session(problem.clone(), config.clone())
                .await
                .map_err(|e| JsonRpcError::internal_error(&e.to_string()))?;
            session_ids.push(session_id);
        }

        tracing::info!(
            "Batch created {} sessions with shared config: {} chunks × {} tokens",
            session_ids.len(),
            config.chunk_size,
            config.max_iterations
        );

        let response = serde_json::json!({
            "session_ids": session_ids
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>(),
            "count": session_ids.len(),
            "config": {
                "chunk_size": config.chunk_size,
                "carryover_size": config.carryover_size,
                "max_iterations": config.max_iterations,
            },
            "message": format!("Successfully created {} reasoning sessions", session_ids.len())
        });

        Ok(CallToolResult {
            content: vec![Content::text(serde_json::to_string_pretty(&response).unwrap())],
            is_error: Some(false),
        })
    }

    async fn tool_search_corpus(&self, args: serde_json::Value) -> Result<CallToolResult, JsonRpcError> {
        #[derive(serde::Deserialize)]
        struct SearchArgs {
            query: String,
            #[serde(default = "default_max_results")]
            max_results: usize,
            #[serde(default = "default_resolution_level")]
            resolution_level: String,
            #[serde(default = "default_similarity_threshold")]
            similarity_threshold: f32,
        }

        fn default_max_results() -> usize {
            5
        }

        fn default_resolution_level() -> String {
            "all".to_string()
        }

        fn default_similarity_threshold() -> f32 {
            0.5
        }

        let args: SearchArgs = serde_json::from_value(args)
            .map_err(|e| JsonRpcError::invalid_params(&e.to_string()))?;

        tracing::info!(
            "H²CE search: query='{}', max={}, level={}, threshold={}",
            args.query,
            args.max_results,
            args.resolution_level,
            args.similarity_threshold
        );

        // TODO: Implement actual H²CE search
        // For now, return placeholder indicating feature is available but not yet connected
        let response = serde_json::json!({
            "query": args.query,
            "results": [],
            "count": 0,
            "config": {
                "max_results": args.max_results,
                "resolution_level": args.resolution_level,
                "similarity_threshold": args.similarity_threshold,
            },
            "note": "H²CE semantic search is available but requires h2ce-integration feature flag and H²CE server configuration"
        });

        Ok(CallToolResult {
            content: vec![Content::text(serde_json::to_string_pretty(&response).unwrap())],
            is_error: Some(false),
        })
    }

    async fn tool_todo_set(&self, args: serde_json::Value) -> Result<CallToolResult, JsonRpcError> {
        #[derive(serde::Deserialize)]
        struct TodoSetArgs {
            session_id: String,
            todos: Vec<TodoItem>,
        }

        let args: TodoSetArgs = serde_json::from_value(args)
            .map_err(|e| JsonRpcError::invalid_params(&e.to_string()))?;

        let session_id = Uuid::parse_str(&args.session_id)
            .map_err(|e| JsonRpcError::invalid_params(&format!("Invalid session_id: {}", e)))?;

        self.todo_bridge.set_todos(session_id, args.todos)
            .map_err(|e| JsonRpcError::internal_error(&e.to_string()))?;

        let response = serde_json::json!({
            "success": true,
            "session_id": session_id.to_string(),
        });

        Ok(CallToolResult {
            content: vec![Content::text(serde_json::to_string_pretty(&response).unwrap())],
            is_error: Some(false),
        })
    }

    async fn tool_todo_get(&self, args: serde_json::Value) -> Result<CallToolResult, JsonRpcError> {
        #[derive(serde::Deserialize)]
        struct TodoGetArgs {
            session_id: String,
        }

        let args: TodoGetArgs = serde_json::from_value(args)
            .map_err(|e| JsonRpcError::invalid_params(&e.to_string()))?;

        let session_id = Uuid::parse_str(&args.session_id)
            .map_err(|e| JsonRpcError::invalid_params(&format!("Invalid session_id: {}", e)))?;

        let todo_list = self.todo_bridge.get_todos(&session_id);
        let summary = self.todo_bridge.get_summary(&session_id);

        let response = if let Some(list) = todo_list {
            serde_json::json!({
                "session_id": session_id.to_string(),
                "todos": list.todos,
                "summary": summary,
                "created_at": list.created_at,
                "updated_at": list.updated_at,
            })
        } else {
            serde_json::json!({
                "session_id": session_id.to_string(),
                "todos": [],
                "summary": {
                    "total": 0,
                    "completed": 0,
                    "in_progress": 0,
                    "pending": 0,
                },
            })
        };

        Ok(CallToolResult {
            content: vec![Content::text(serde_json::to_string_pretty(&response).unwrap())],
            is_error: Some(false),
        })
    }

    async fn tool_todo_add(&self, args: serde_json::Value) -> Result<CallToolResult, JsonRpcError> {
        #[derive(serde::Deserialize)]
        struct TodoAddArgs {
            session_id: String,
            content: String,
            active_form: String,
        }

        let args: TodoAddArgs = serde_json::from_value(args)
            .map_err(|e| JsonRpcError::invalid_params(&e.to_string()))?;

        let session_id = Uuid::parse_str(&args.session_id)
            .map_err(|e| JsonRpcError::invalid_params(&format!("Invalid session_id: {}", e)))?;

        self.todo_bridge.add_todo(session_id, args.content.clone(), args.active_form.clone())
            .map_err(|e| JsonRpcError::internal_error(&e.to_string()))?;

        let response = serde_json::json!({
            "success": true,
            "session_id": session_id.to_string(),
            "added": {
                "content": args.content,
                "active_form": args.active_form,
                "status": "pending",
            },
        });

        Ok(CallToolResult {
            content: vec![Content::text(serde_json::to_string_pretty(&response).unwrap())],
            is_error: Some(false),
        })
    }

    async fn tool_todo_update_status(&self, args: serde_json::Value) -> Result<CallToolResult, JsonRpcError> {
        #[derive(serde::Deserialize)]
        struct TodoUpdateArgs {
            session_id: String,
            index: usize,
            status: String,
        }

        let args: TodoUpdateArgs = serde_json::from_value(args)
            .map_err(|e| JsonRpcError::invalid_params(&e.to_string()))?;

        let session_id = Uuid::parse_str(&args.session_id)
            .map_err(|e| JsonRpcError::invalid_params(&format!("Invalid session_id: {}", e)))?;

        let status = match args.status.as_str() {
            "pending" => TodoStatus::Pending,
            "inprogress" => TodoStatus::InProgress,
            "completed" => TodoStatus::Completed,
            _ => return Err(JsonRpcError::invalid_params(&format!("Invalid status: {}", args.status))),
        };

        self.todo_bridge.update_todo_status(&session_id, args.index, status)
            .map_err(|e| JsonRpcError::internal_error(&e.to_string()))?;

        let response = serde_json::json!({
            "success": true,
            "session_id": session_id.to_string(),
            "updated": {
                "index": args.index,
                "status": args.status,
            },
        });

        Ok(CallToolResult {
            content: vec![Content::text(serde_json::to_string_pretty(&response).unwrap())],
            is_error: Some(false),
        })
    }

    // ========================================================================
    // Helper Functions
    // ========================================================================

    fn parse_lattice_type(s: &str) -> Result<LatticeType, String> {
        match s.to_lowercase().as_str() {
            "e8" => Ok(LatticeType::E8),
            "leech" => Ok(LatticeType::Leech),
            "hcp-8" => Ok(LatticeType::HCP(8)),
            "hcp-16" => Ok(LatticeType::HCP(16)),
            "hcp-24" => Ok(LatticeType::HCP(24)),
            "cubic-8" => Ok(LatticeType::Hypercubic(8)),
            "cubic-16" => Ok(LatticeType::Hypercubic(16)),
            "cubic-32" => Ok(LatticeType::Hypercubic(32)),
            _ => Err(format!(
                "Invalid lattice type '{}'. Valid options: e8, leech, hcp-8, hcp-16, hcp-24, cubic-8, cubic-16, cubic-32",
                s
            )),
        }
    }
}

impl Default for MarkovianMCPServer {
    fn default() -> Self {
        Self::new()
    }
}
