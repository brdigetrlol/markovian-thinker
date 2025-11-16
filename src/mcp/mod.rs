// MCP Module
// Model Context Protocol implementation for Markovian reasoning (stateful mode)

pub mod protocol;
pub mod stdio;
pub mod parallel_tools;
pub mod training_tools;

pub use protocol::*;
pub use stdio::StdioHandler;
use anyhow::Result;
use serde_json::json;
use std::sync::{Arc, RwLock};

use crate::inference::InferenceModel;
use crate::training::OnlineLearner;

#[cfg(feature = "gpu")]
use crate::parallel::ParallelExecutor;

// MCP Server implementation with state
pub struct MarkovianMCPServer {
    model: Arc<RwLock<InferenceModel>>,
    learner: Arc<RwLock<OnlineLearner>>,
    #[cfg(feature = "gpu")]
    executor: Option<Arc<ParallelExecutor>>,
}

impl MarkovianMCPServer {
    pub fn new(
        model: Arc<RwLock<InferenceModel>>,
        learner: Arc<RwLock<OnlineLearner>>,
    ) -> Self {
        Self {
            model,
            learner,
            #[cfg(feature = "gpu")]
            executor: None,
        }
    }

    #[cfg(feature = "gpu")]
    pub fn with_executor(mut self, executor: Arc<ParallelExecutor>) -> Self {
        self.executor = Some(executor);
        self
    }

    pub fn with_stdio(
        model: Arc<RwLock<InferenceModel>>,
        learner: Arc<RwLock<OnlineLearner>>,
    ) -> (Self, StdioHandler, tokio::task::JoinHandle<()>) {
        let (stdio, reader_handle) = StdioHandler::new();
        let server = Self::new(model, learner);
        (server, stdio, reader_handle)
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

        let mut tools = vec![
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

        // Add parallel execution tools when GPU feature is enabled
        #[cfg(feature = "gpu")]
        {
            tools.extend(vec![
                Tool {
                    name: "parallel_codegen".to_string(),
                    description: "Execute multiple code generation tasks in parallel on GPU. Processes multiple prompts simultaneously for high throughput.".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "prompts": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "List of code generation prompts to process in parallel"
                            },
                            "language": {
                                "type": "string",
                                "description": "Target programming language"
                            },
                            "max_tokens": {
                                "type": "number",
                                "description": "Maximum tokens per generation (default: 2048)",
                                "default": 2048
                            },
                            "temperature": {
                                "type": "number",
                                "description": "Temperature for generation (default: 0.7)",
                                "default": 0.7
                            }
                        },
                        "required": ["prompts", "language"]
                    }),
                },
                Tool {
                    name: "parallel_analysis".to_string(),
                    description: "Execute multiple analysis tasks in parallel on GPU. Analyze multiple documents, code snippets, or problems simultaneously.".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "texts": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "List of texts to analyze in parallel"
                            },
                            "analysis_type": {
                                "type": "string",
                                "enum": ["summarize", "extract", "classify", "reason"],
                                "description": "Type of analysis to perform"
                            },
                            "max_output_tokens": {
                                "type": "number",
                                "description": "Maximum output tokens per analysis (default: 1024)",
                                "default": 1024
                            }
                        },
                        "required": ["texts", "analysis_type"]
                    }),
                },
                Tool {
                    name: "parallel_data_process".to_string(),
                    description: "Execute parallel data processing operations on GPU. Transform, filter, or aggregate data arrays with GPU acceleration.".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "data_arrays": {
                                "type": "array",
                                "items": {
                                    "type": "array",
                                    "items": {"type": "number"}
                                },
                                "description": "List of data arrays to process in parallel"
                            },
                            "operation": {
                                "type": "string",
                                "enum": ["transform", "filter", "aggregate"],
                                "description": "Operation to perform on data"
                            },
                            "params": {
                                "type": "object",
                                "description": "Operation-specific parameters (e.g., {factor: 2.0} for transform)"
                            }
                        },
                        "required": ["data_arrays", "operation", "params"]
                    }),
                },
                Tool {
                    name: "multi_agent_simulation".to_string(),
                    description: "Run multi-agent simulation with GPU acceleration. Simulate multiple AI agents working on different parts of a larger problem in parallel.".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "num_agents": {
                                "type": "number",
                                "description": "Number of agents in simulation"
                            },
                            "steps": {
                                "type": "number",
                                "description": "Number of simulation steps"
                            },
                            "environment_params": {
                                "type": "object",
                                "description": "Environment parameters as key-value pairs"
                            }
                        },
                        "required": ["num_agents", "steps"]
                    }),
                },
                Tool {
                    name: "executor_stats".to_string(),
                    description: "Get statistics about the parallel executor including GPU status, queue sizes, and worker information.".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {}
                    }),
                },
            ]);
        }

        // Add training and weight management tools
        tools.extend(vec![
            Tool {
                name: "load_weights".to_string(),
                description: "Load model weights from a file (supports SafeTensors, GGUF, binary, custom formats).".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "file_path": {
                            "type": "string",
                            "description": "Path to the weights file"
                        },
                        "format": {
                            "type": "string",
                            "description": "Weight format: safetensors, gguf, binary, or custom (default: safetensors)",
                            "default": "safetensors"
                        }
                    },
                    "required": ["file_path"]
                }),
            },
            Tool {
                name: "save_weights".to_string(),
                description: "Save current model weights to a file.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "file_path": {
                            "type": "string",
                            "description": "Path to save the weights"
                        },
                        "format": {
                            "type": "string",
                            "description": "Save format: custom or binary (default: custom)",
                            "default": "custom"
                        }
                    },
                    "required": ["file_path"]
                }),
            },
            Tool {
                name: "enable_learning".to_string(),
                description: "Enable online learning - the model will continuously learn from examples during inference.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            Tool {
                name: "disable_learning".to_string(),
                description: "Disable online learning.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            Tool {
                name: "add_training_example".to_string(),
                description: "Add a training example for online learning. The model will learn from this example.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "input": {
                            "type": "string",
                            "description": "Input text"
                        },
                        "target": {
                            "type": "string",
                            "description": "Target output text (optional for self-supervised)"
                        },
                        "weight": {
                            "type": "number",
                            "description": "Example weight (default: 1.0)",
                            "default": 1.0
                        }
                    },
                    "required": ["input"]
                }),
            },
            Tool {
                name: "get_learning_stats".to_string(),
                description: "Get statistics about online learning (examples, updates, loss, learning rate).".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            Tool {
                name: "set_learning_rate".to_string(),
                description: "Set the learning rate for online learning.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "learning_rate": {
                            "type": "number",
                            "description": "New learning rate (e.g., 0.001)"
                        }
                    },
                    "required": ["learning_rate"]
                }),
            },
            Tool {
                name: "force_update".to_string(),
                description: "Force an immediate weight update using buffered training examples.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
        ]);

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

    async fn handle_call_tool(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        tracing::info!("Handling tools/call request");

        // Parse params
        let params: CallToolParams = match request.params.as_ref() {
            Some(p) => match serde_json::from_value(p.clone()) {
                Ok(params) => params,
                Err(e) => {
                    return JsonRpcResponse::error(
                        request.id,
                        JsonRpcError::invalid_params(&e.to_string()),
                    );
                }
            },
            None => {
                return JsonRpcResponse::error(
                    request.id,
                    JsonRpcError::invalid_params("Missing params"),
                );
            }
        };

        tracing::debug!("Calling tool: {}", params.name);

        // Dispatch to appropriate handler
        let result = match params.name.as_str() {
            // Core reasoning tool
            "markovian_think" => {
                self.handle_markovian_think(params.arguments).await
            }

            // Weight management tools
            "load_weights" => {
                self.handle_load_weights_tool(params.arguments)
            }
            "save_weights" => {
                self.handle_save_weights_tool(params.arguments)
            }

            // Training tools
            "enable_learning" => {
                self.handle_enable_learning_tool()
            }
            "disable_learning" => {
                self.handle_disable_learning_tool()
            }
            "add_training_example" => {
                self.handle_add_training_example_tool(params.arguments)
            }
            "get_learning_stats" => {
                self.handle_get_learning_stats_tool()
            }
            "set_learning_rate" => {
                self.handle_set_learning_rate_tool(params.arguments)
            }
            "force_update" => {
                self.handle_force_update_tool()
            }

            // GPU parallel execution tools
            #[cfg(feature = "gpu")]
            "parallel_codegen" => {
                self.handle_parallel_codegen_tool(params.arguments).await
            }
            #[cfg(feature = "gpu")]
            "parallel_analysis" => {
                self.handle_parallel_analysis_tool(params.arguments).await
            }
            #[cfg(feature = "gpu")]
            "parallel_data_process" => {
                self.handle_parallel_data_process_tool(params.arguments).await
            }
            #[cfg(feature = "gpu")]
            "multi_agent_simulation" => {
                self.handle_simulation_tool(params.arguments).await
            }
            #[cfg(feature = "gpu")]
            "executor_stats" => {
                self.handle_executor_stats_tool().await
            }

            _ => {
                Err(anyhow::anyhow!("Unknown tool: {}", params.name))
            }
        };

        // Convert result to MCP response
        match result {
            Ok(value) => {
                let result = CallToolResult {
                    content: vec![Content::text(serde_json::to_string_pretty(&value).unwrap())],
                    is_error: Some(false),
                };
                JsonRpcResponse::success(request.id, serde_json::to_value(result).unwrap())
            }
            Err(e) => {
                let error_result = CallToolResult {
                    content: vec![Content::text(format!("Error: {}", e))],
                    is_error: Some(true),
                };
                JsonRpcResponse::success(request.id, serde_json::to_value(error_result).unwrap())
            }
        }
    }

    // Tool implementation methods

    async fn handle_markovian_think(&self, arguments: serde_json::Value) -> Result<serde_json::Value> {
        #[derive(serde::Deserialize)]
        struct ThinkParams {
            problem: String,
            #[serde(default = "default_max_iterations")]
            max_iterations: usize,
        }
        fn default_max_iterations() -> usize { 5 }

        let params: ThinkParams = serde_json::from_value(arguments)?;

        // For now, return a placeholder response
        // TODO: Implement actual Markovian reasoning
        Ok(json!({
            "status": "success",
            "result": format!("Markovian reasoning on: {}", params.problem),
            "iterations": params.max_iterations,
            "note": "Full implementation requires client-side LLM calls via MCP sampling"
        }))
    }

    fn handle_load_weights_tool(&self, arguments: serde_json::Value) -> Result<serde_json::Value> {
        use crate::mcp::training_tools::{LoadWeightsParams, handle_load_weights};
        let params: LoadWeightsParams = serde_json::from_value(arguments)?;
        handle_load_weights(params, self.model.clone())
    }

    fn handle_save_weights_tool(&self, arguments: serde_json::Value) -> Result<serde_json::Value> {
        use crate::mcp::training_tools::{SaveWeightsParams, handle_save_weights};
        let params: SaveWeightsParams = serde_json::from_value(arguments)?;
        handle_save_weights(params, self.model.clone())
    }

    fn handle_enable_learning_tool(&self) -> Result<serde_json::Value> {
        use crate::mcp::training_tools::handle_enable_learning;
        handle_enable_learning(self.learner.clone())
    }

    fn handle_disable_learning_tool(&self) -> Result<serde_json::Value> {
        use crate::mcp::training_tools::handle_disable_learning;
        handle_disable_learning(self.learner.clone())
    }

    fn handle_add_training_example_tool(&self, arguments: serde_json::Value) -> Result<serde_json::Value> {
        use crate::mcp::training_tools::{AddTrainingExampleParams, handle_add_training_example};
        let params: AddTrainingExampleParams = serde_json::from_value(arguments)?;
        handle_add_training_example(params, self.learner.clone())
    }

    fn handle_get_learning_stats_tool(&self) -> Result<serde_json::Value> {
        use crate::mcp::training_tools::handle_get_learning_stats;
        handle_get_learning_stats(self.learner.clone())
    }

    fn handle_set_learning_rate_tool(&self, arguments: serde_json::Value) -> Result<serde_json::Value> {
        use crate::mcp::training_tools::{SetLearningRateParams, handle_set_learning_rate};
        let params: SetLearningRateParams = serde_json::from_value(arguments)?;
        handle_set_learning_rate(params, self.learner.clone())
    }

    fn handle_force_update_tool(&self) -> Result<serde_json::Value> {
        use crate::mcp::training_tools::handle_force_update;
        handle_force_update(self.learner.clone())
    }

    #[cfg(feature = "gpu")]
    async fn handle_parallel_codegen_tool(&self, arguments: serde_json::Value) -> Result<serde_json::Value> {
        use crate::mcp::parallel_tools::{ParallelCodeGenParams, handle_parallel_codegen};
        let params: ParallelCodeGenParams = serde_json::from_value(arguments)?;
        match &self.executor {
            Some(executor) => handle_parallel_codegen(executor, params).await,
            None => Err(anyhow::anyhow!("GPU executor not initialized")),
        }
    }

    #[cfg(feature = "gpu")]
    async fn handle_parallel_analysis_tool(&self, arguments: serde_json::Value) -> Result<serde_json::Value> {
        use crate::mcp::parallel_tools::{ParallelAnalysisParams, handle_parallel_analysis};
        let params: ParallelAnalysisParams = serde_json::from_value(arguments)?;
        match &self.executor {
            Some(executor) => handle_parallel_analysis(executor, params).await,
            None => Err(anyhow::anyhow!("GPU executor not initialized")),
        }
    }

    #[cfg(feature = "gpu")]
    async fn handle_parallel_data_process_tool(&self, arguments: serde_json::Value) -> Result<serde_json::Value> {
        use crate::mcp::parallel_tools::{ParallelDataProcessParams, handle_parallel_data_process};
        let params: ParallelDataProcessParams = serde_json::from_value(arguments)?;
        match &self.executor {
            Some(executor) => handle_parallel_data_process(executor, params).await,
            None => Err(anyhow::anyhow!("GPU executor not initialized")),
        }
    }

    #[cfg(feature = "gpu")]
    async fn handle_simulation_tool(&self, arguments: serde_json::Value) -> Result<serde_json::Value> {
        use crate::mcp::parallel_tools::{SimulationParams, handle_simulation};
        let params: SimulationParams = serde_json::from_value(arguments)?;
        match &self.executor {
            Some(executor) => handle_simulation(executor, params).await,
            None => Err(anyhow::anyhow!("GPU executor not initialized")),
        }
    }

    #[cfg(feature = "gpu")]
    async fn handle_executor_stats_tool(&self) -> Result<serde_json::Value> {
        use crate::mcp::parallel_tools::handle_executor_stats;
        match &self.executor {
            Some(executor) => handle_executor_stats(executor).await,
            None => Err(anyhow::anyhow!("GPU executor not initialized")),
        }
    }
}
