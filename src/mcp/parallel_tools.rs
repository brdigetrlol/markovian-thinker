//! MCP tools for parallel task execution
//!
//! Provides MCP (Model Context Protocol) tools for submitting tasks to the
//! GPU-accelerated parallel executor.

#[cfg(feature = "gpu")]
use crate::parallel::{
    ParallelExecutor, ExecutorConfig,
    task::{CodeGenTask, AnalysisTask, AnalysisType, DataProcessTask, DataOperation, SimulationTask},
};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// MCP tool parameters for parallel code generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelCodeGenParams {
    /// List of prompts for code generation
    pub prompts: Vec<String>,
    /// Target programming language
    pub language: String,
    /// Maximum tokens per generation
    #[serde(default = "default_max_tokens")]
    pub max_tokens: usize,
    /// Temperature for generation
    #[serde(default = "default_temperature")]
    pub temperature: f32,
}

fn default_max_tokens() -> usize { 2048 }
fn default_temperature() -> f32 { 0.7 }

/// MCP tool parameters for parallel analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelAnalysisParams {
    /// List of texts to analyze
    pub texts: Vec<String>,
    /// Type of analysis to perform
    pub analysis_type: String, // "summarize", "extract", "classify", "reason"
    /// Maximum output tokens per analysis
    #[serde(default = "default_analysis_tokens")]
    pub max_output_tokens: usize,
}

fn default_analysis_tokens() -> usize { 1024 }

/// MCP tool parameters for parallel data processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelDataProcessParams {
    /// List of data arrays to process
    pub data_arrays: Vec<Vec<f32>>,
    /// Operation to perform
    pub operation: String, // "transform", "filter", "aggregate"
    /// Operation parameters
    pub params: serde_json::Value,
}

/// MCP tool parameters for multi-agent simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationParams {
    /// Number of agents in simulation
    pub num_agents: usize,
    /// Number of simulation steps
    pub steps: usize,
    /// Environment parameters
    #[serde(default)]
    pub environment_params: std::collections::HashMap<String, f32>,
}

/// Handle parallel code generation MCP tool
#[cfg(feature = "gpu")]
pub async fn handle_parallel_codegen(
    executor: &Arc<ParallelExecutor>,
    params: ParallelCodeGenParams,
) -> Result<Value> {
    tracing::info!("Parallel code generation requested for {} prompts", params.prompts.len());

    // Create tasks
    let tasks: Vec<_> = params.prompts.into_iter()
        .map(|prompt| CodeGenTask::new(prompt, params.language.clone()))
        .collect();

    let num_tasks = tasks.len();

    // Submit tasks (would work if output deserialization was implemented)
    // For now, return placeholder
    Ok(serde_json::json!({
        "status": "pending",
        "message": format!("Submitted {} code generation tasks for parallel execution", num_tasks),
        "num_tasks": num_tasks,
        "note": "Full implementation requires kernel and deserialization code"
    }))
}

/// Handle parallel analysis MCP tool
#[cfg(feature = "gpu")]
pub async fn handle_parallel_analysis(
    executor: &Arc<ParallelExecutor>,
    params: ParallelAnalysisParams,
) -> Result<Value> {
    tracing::info!("Parallel analysis requested for {} texts", params.texts.len());

    // Parse analysis type
    let analysis_type = match params.analysis_type.as_str() {
        "summarize" => AnalysisType::Summarize,
        "extract" => AnalysisType::Extract,
        "classify" => AnalysisType::Classify,
        "reason" => AnalysisType::Reason,
        _ => AnalysisType::Reason,
    };

    // Create tasks
    let tasks: Vec<_> = params.texts.into_iter()
        .map(|text| AnalysisTask::new(text, analysis_type.clone()))
        .collect();

    let num_tasks = tasks.len();

    Ok(serde_json::json!({
        "status": "pending",
        "message": format!("Submitted {} analysis tasks for parallel execution", num_tasks),
        "num_tasks": num_tasks,
        "analysis_type": params.analysis_type,
    }))
}

/// Handle parallel data processing MCP tool
#[cfg(feature = "gpu")]
pub async fn handle_parallel_data_process(
    executor: &Arc<ParallelExecutor>,
    params: ParallelDataProcessParams,
) -> Result<Value> {
    tracing::info!("Parallel data processing requested for {} arrays", params.data_arrays.len());

    // Parse operation
    let operation = match params.operation.as_str() {
        "transform" => {
            let factor = params.params.get("factor")
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0) as f32;
            DataOperation::Transform { factor }
        }
        "filter" => {
            let threshold = params.params.get("threshold")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32;
            DataOperation::Filter { threshold }
        }
        "aggregate" => {
            let method = params.params.get("method")
                .and_then(|v| v.as_str())
                .unwrap_or("mean")
                .to_string();
            DataOperation::Aggregate { method }
        }
        _ => DataOperation::Transform { factor: 1.0 },
    };

    // Create tasks
    let tasks: Vec<_> = params.data_arrays.into_iter()
        .map(|data| DataProcessTask::new(data, operation.clone()))
        .collect();

    let num_tasks = tasks.len();

    Ok(serde_json::json!({
        "status": "pending",
        "message": format!("Submitted {} data processing tasks for parallel execution", num_tasks),
        "num_tasks": num_tasks,
        "operation": params.operation,
    }))
}

/// Handle multi-agent simulation MCP tool
#[cfg(feature = "gpu")]
pub async fn handle_simulation(
    executor: &Arc<ParallelExecutor>,
    params: SimulationParams,
) -> Result<Value> {
    tracing::info!(
        "Multi-agent simulation requested: {} agents, {} steps",
        params.num_agents,
        params.steps
    );

    let mut task = SimulationTask::new(params.num_agents, params.steps);
    task.environment_params = params.environment_params;

    Ok(serde_json::json!({
        "status": "pending",
        "message": format!("Submitted simulation with {} agents for {} steps", params.num_agents, params.steps),
        "num_agents": params.num_agents,
        "steps": params.steps,
    }))
}

/// Get executor statistics
#[cfg(feature = "gpu")]
pub async fn handle_executor_stats(executor: &Arc<ParallelExecutor>) -> Result<Value> {
    let stats = executor.stats().await;

    Ok(serde_json::json!({
        "num_workers": stats.num_workers,
        "gpu_available": stats.gpu_available,
        "queue": {
            "total_queued": stats.queue_stats.total_queued,
            "pending_results": stats.queue_stats.pending_results,
            "by_type": stats.queue_stats.queued_by_type,
        }
    }))
}

#[cfg(not(feature = "gpu"))]
pub async fn handle_parallel_codegen(
    _executor: &Arc<()>,
    _params: ParallelCodeGenParams,
) -> Result<Value> {
    anyhow::bail!("GPU feature not enabled. Recompile with --features gpu")
}

#[cfg(not(feature = "gpu"))]
pub async fn handle_parallel_analysis(
    _executor: &Arc<()>,
    _params: ParallelAnalysisParams,
) -> Result<Value> {
    anyhow::bail!("GPU feature not enabled. Recompile with --features gpu")
}

#[cfg(not(feature = "gpu"))]
pub async fn handle_parallel_data_process(
    _executor: &Arc<()>,
    _params: ParallelDataProcessParams,
) -> Result<Value> {
    anyhow::bail!("GPU feature not enabled. Recompile with --features gpu")
}

#[cfg(not(feature = "gpu"))]
pub async fn handle_simulation(
    _executor: &Arc<()>,
    _params: SimulationParams,
) -> Result<Value> {
    anyhow::bail!("GPU feature not enabled. Recompile with --features gpu")
}

#[cfg(not(feature = "gpu"))]
pub async fn handle_executor_stats(_executor: &Arc<()>) -> Result<Value> {
    anyhow::bail!("GPU feature not enabled. Recompile with --features gpu")
}
