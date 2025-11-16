//! MCP tools for model training and weight management
//!
//! Provides MCP (Model Context Protocol) tools for training the model,
//! loading/saving weights, and managing online learning.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::{Arc, RwLock};

use crate::training::{WeightLoader, WeightFormat, OnlineLearner, TrainingExample};
use crate::inference::InferenceModel;

/// MCP tool parameters for loading model weights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadWeightsParams {
    /// Path to the weights file
    pub file_path: String,
    /// Weight format (safetensors, gguf, binary, custom)
    #[serde(default = "default_format")]
    pub format: String,
}

fn default_format() -> String { "safetensors".to_string() }

/// MCP tool parameters for saving model weights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveWeightsParams {
    /// Path to save the weights
    pub file_path: String,
    /// Weight format to save as (custom, binary)
    #[serde(default = "default_save_format")]
    pub format: String,
}

fn default_save_format() -> String { "custom".to_string() }

/// MCP tool parameters for adding training examples
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddTrainingExampleParams {
    /// Input text
    pub input: String,
    /// Target output text (optional for self-supervised)
    pub target: Option<String>,
    /// Example weight (default 1.0)
    #[serde(default = "default_weight")]
    pub weight: f32,
}

fn default_weight() -> f32 { 1.0 }

/// MCP tool parameters for setting learning rate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetLearningRateParams {
    /// New learning rate
    pub learning_rate: f32,
}

/// Training tool responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats: Option<LearningStatsJson>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningStatsJson {
    pub total_examples: usize,
    pub total_updates: usize,
    pub buffer_size: usize,
    pub average_loss: f32,
    pub learning_rate: f32,
    pub enabled: bool,
}

/// Handle load_weights MCP tool
pub fn handle_load_weights(
    params: LoadWeightsParams,
    model: Arc<RwLock<InferenceModel>>,
) -> Result<Value> {
    // Parse format
    let format = match params.format.to_lowercase().as_str() {
        "safetensors" | "st" => WeightFormat::SafeTensors,
        "gguf" => WeightFormat::GGUF,
        "binary" | "bin" => WeightFormat::Binary,
        "custom" => WeightFormat::Custom,
        _ => anyhow::bail!("Unknown weight format: {}", params.format),
    };

    // Load weights
    let mut loader = WeightLoader::new(format);
    loader.load_from_file(&params.file_path)?;

    // Get embedding weights
    let embedding_weights = loader.get_embedding_weights()
        .ok_or_else(|| anyhow::anyhow!("No embedding weights found in file"))?;

    // Update model weights
    let mut model = model.write()
        .map_err(|_| anyhow::anyhow!("Failed to acquire write lock on model"))?;
    let embedding = Arc::get_mut(model.embedding_mut())
        .ok_or_else(|| anyhow::anyhow!("Failed to get mutable access to embedding layer"))?;

    embedding.load_weights(embedding_weights.clone())?;

    let response = TrainingResponse {
        success: true,
        message: format!(
            "Loaded {} parameters from {}",
            embedding_weights.len(),
            params.file_path
        ),
        stats: None,
    };

    Ok(serde_json::to_value(response)?)
}

/// Handle save_weights MCP tool
pub fn handle_save_weights(
    params: SaveWeightsParams,
    model: Arc<RwLock<InferenceModel>>,
) -> Result<Value> {
    // Parse format
    let format = match params.format.to_lowercase().as_str() {
        "custom" => WeightFormat::Custom,
        "binary" | "bin" => WeightFormat::Binary,
        _ => anyhow::bail!("Unsupported save format: {} (use 'custom' or 'binary')", params.format),
    };

    // Get model weights
    let model = model.read()
        .map_err(|_| anyhow::anyhow!("Failed to acquire read lock on model"))?;
    let embedding = model.embedding();
    let weights = embedding.weights().to_vec();

    // Create loader with current weights
    let mut loader = WeightLoader::new(format);
    loader.load_embedding_weights(embedding.vocab_size(), embedding.embed_dim(), weights)?;

    // Save to file
    loader.save_to_file(&params.file_path, format)?;

    let response = TrainingResponse {
        success: true,
        message: format!("Saved weights to {}", params.file_path),
        stats: None,
    };

    Ok(serde_json::to_value(response)?)
}

/// Handle enable_learning MCP tool
pub fn handle_enable_learning(learner: Arc<RwLock<OnlineLearner>>) -> Result<Value> {
    let mut learner = learner.write()
        .map_err(|_| anyhow::anyhow!("Failed to acquire write lock on learner"))?;
    learner.enable();

    let stats = learner.get_stats();
    let stats_json = LearningStatsJson {
        total_examples: stats.total_examples,
        total_updates: stats.total_updates,
        buffer_size: stats.buffer_size,
        average_loss: stats.average_loss,
        learning_rate: stats.learning_rate,
        enabled: stats.enabled,
    };

    let response = TrainingResponse {
        success: true,
        message: "Online learning enabled".to_string(),
        stats: Some(stats_json),
    };

    Ok(serde_json::to_value(response)?)
}

/// Handle disable_learning MCP tool
pub fn handle_disable_learning(learner: Arc<RwLock<OnlineLearner>>) -> Result<Value> {
    let mut learner = learner.write()
        .map_err(|_| anyhow::anyhow!("Failed to acquire write lock on learner"))?;
    learner.disable();

    let stats = learner.get_stats();
    let stats_json = LearningStatsJson {
        total_examples: stats.total_examples,
        total_updates: stats.total_updates,
        buffer_size: stats.buffer_size,
        average_loss: stats.average_loss,
        learning_rate: stats.learning_rate,
        enabled: stats.enabled,
    };

    let response = TrainingResponse {
        success: true,
        message: "Online learning disabled".to_string(),
        stats: Some(stats_json),
    };

    Ok(serde_json::to_value(response)?)
}

/// Handle add_training_example MCP tool
pub fn handle_add_training_example(
    params: AddTrainingExampleParams,
    learner: Arc<RwLock<OnlineLearner>>,
) -> Result<Value> {
    let example = TrainingExample::new(params.input, params.target)
        .with_weight(params.weight);

    let mut learner = learner.write()
        .map_err(|_| anyhow::anyhow!("Failed to acquire write lock on learner"))?;
    learner.add_example(example)?;

    let stats = learner.get_stats();
    let stats_json = LearningStatsJson {
        total_examples: stats.total_examples,
        total_updates: stats.total_updates,
        buffer_size: stats.buffer_size,
        average_loss: stats.average_loss,
        learning_rate: stats.learning_rate,
        enabled: stats.enabled,
    };

    let response = TrainingResponse {
        success: true,
        message: "Training example added".to_string(),
        stats: Some(stats_json),
    };

    Ok(serde_json::to_value(response)?)
}

/// Handle get_learning_stats MCP tool
pub fn handle_get_learning_stats(learner: Arc<RwLock<OnlineLearner>>) -> Result<Value> {
    let learner = learner.read()
        .map_err(|_| anyhow::anyhow!("Failed to acquire read lock on learner"))?;
    let stats = learner.get_stats();

    let stats_json = LearningStatsJson {
        total_examples: stats.total_examples,
        total_updates: stats.total_updates,
        buffer_size: stats.buffer_size,
        average_loss: stats.average_loss,
        learning_rate: stats.learning_rate,
        enabled: stats.enabled,
    };

    let response = TrainingResponse {
        success: true,
        message: "Learning statistics retrieved".to_string(),
        stats: Some(stats_json),
    };

    Ok(serde_json::to_value(response)?)
}

/// Handle set_learning_rate MCP tool
pub fn handle_set_learning_rate(
    params: SetLearningRateParams,
    learner: Arc<RwLock<OnlineLearner>>,
) -> Result<Value> {
    let mut learner = learner.write()
        .map_err(|_| anyhow::anyhow!("Failed to acquire write lock on learner"))?;
    learner.set_learning_rate(params.learning_rate);

    let stats = learner.get_stats();
    let stats_json = LearningStatsJson {
        total_examples: stats.total_examples,
        total_updates: stats.total_updates,
        buffer_size: stats.buffer_size,
        average_loss: stats.average_loss,
        learning_rate: stats.learning_rate,
        enabled: stats.enabled,
    };

    let response = TrainingResponse {
        success: true,
        message: format!("Learning rate set to {}", params.learning_rate),
        stats: Some(stats_json),
    };

    Ok(serde_json::to_value(response)?)
}

/// Handle force_update MCP tool
pub fn handle_force_update(learner: Arc<RwLock<OnlineLearner>>) -> Result<Value> {
    let mut learner = learner.write()
        .map_err(|_| anyhow::anyhow!("Failed to acquire write lock on learner"))?;
    learner.force_update()?;

    let stats = learner.get_stats();
    let stats_json = LearningStatsJson {
        total_examples: stats.total_examples,
        total_updates: stats.total_updates,
        buffer_size: stats.buffer_size,
        average_loss: stats.average_loss,
        learning_rate: stats.learning_rate,
        enabled: stats.enabled,
    };

    let response = TrainingResponse {
        success: true,
        message: "Forced training update completed".to_string(),
        stats: Some(stats_json),
    };

    Ok(serde_json::to_value(response)?)
}
