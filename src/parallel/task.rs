//! Task types and trait definitions for parallel execution

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Trait for tasks that can be executed on the GPU in parallel
pub trait Task: Send + Sync + 'static {
    /// The output type of this task
    type Output: Send + Sync + 'static;

    /// Get the task ID
    fn id(&self) -> Uuid;

    /// Get the task type
    fn task_type(&self) -> TaskType;

    /// Convert task input to GPU buffer format
    fn to_gpu_buffer(&self) -> Vec<f32>;

    /// Parse GPU buffer output into result
    fn from_gpu_buffer(_buffer: &[f32]) -> Self::Output;

    /// Get the input size (number of floats needed)
    fn input_size() -> usize;

    /// Get the output size (number of floats produced)
    fn output_size() -> usize;

    /// Get the CUDA kernel name for this task type
    fn kernel_name() -> &'static str;

    /// Execute the task (CPU fallback when GPU is not available)
    #[allow(async_fn_in_trait)]
    async fn execute_cpu(&self) -> Result<Self::Output>;
}

/// Task types supported by the parallel executor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskType {
    /// Code generation task
    CodeGeneration,
    /// Analysis/reasoning task
    Analysis,
    /// Data processing task
    DataProcessing,
    /// Multi-agent simulation
    Simulation,
}

impl std::fmt::Display for TaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskType::CodeGeneration => write!(f, "code_generation"),
            TaskType::Analysis => write!(f, "analysis"),
            TaskType::DataProcessing => write!(f, "data_processing"),
            TaskType::Simulation => write!(f, "simulation"),
        }
    }
}

/// Task envelope for type-erased task storage
#[derive(Clone)]
pub struct TaskEnvelope {
    pub id: Uuid,
    pub task_type: TaskType,
    pub priority: u8,
    pub created_at: chrono::DateTime<chrono::Utc>,
    // Actual task data (serialized)
    pub data: Vec<u8>,
}

impl TaskEnvelope {
    pub fn new<T: Task>(task: T, priority: u8) -> Self {
        Self {
            id: task.id(),
            task_type: task.task_type(),
            priority,
            created_at: chrono::Utc::now(),
            data: vec![], // Would serialize task here
        }
    }
}

/// Result of a task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: Uuid,
    pub success: bool,
    pub output: serde_json::Value,
    pub error: Option<String>,
    pub gpu_time_ms: f64,
    pub total_time_ms: f64,
}

/// Code generation task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeGenTask {
    pub id: Uuid,
    pub prompt: String,
    pub language: String,
    pub max_tokens: usize,
    pub temperature: f32,
}

impl CodeGenTask {
    pub fn new(prompt: String, language: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            prompt,
            language,
            max_tokens: 2048,
            temperature: 0.7,
        }
    }
}

impl Task for CodeGenTask {
    type Output = String;

    fn id(&self) -> Uuid {
        self.id
    }

    fn task_type(&self) -> TaskType {
        TaskType::CodeGeneration
    }

    fn to_gpu_buffer(&self) -> Vec<f32> {
        // In a real implementation, this would:
        // 1. Tokenize the prompt
        // 2. Convert tokens to embeddings
        // 3. Return as float vector
        vec![0.0; Self::input_size()]
    }

    fn from_gpu_buffer(_buffer: &[f32]) -> Self::Output {
        // In a real implementation, this would:
        // 1. Decode token IDs from buffer
        // 2. Convert to text
        String::from("// Generated code")
    }

    fn input_size() -> usize {
        2048 // Max input tokens * embedding dim
    }

    fn output_size() -> usize {
        2048 // Max output tokens * embedding dim
    }

    fn kernel_name() -> &'static str {
        "code_generation_kernel"
    }

    async fn execute_cpu(&self) -> Result<Self::Output> {
        // CPU fallback implementation
        Ok(format!("// Generated {} code for: {}", self.language, self.prompt))
    }
}

/// Analysis/reasoning task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisTask {
    pub id: Uuid,
    pub input_text: String,
    pub analysis_type: AnalysisType,
    pub max_output_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisType {
    Summarize,
    Extract,
    Classify,
    Reason,
}

impl AnalysisTask {
    pub fn new(input_text: String, analysis_type: AnalysisType) -> Self {
        Self {
            id: Uuid::new_v4(),
            input_text,
            analysis_type,
            max_output_tokens: 1024,
        }
    }
}

impl Task for AnalysisTask {
    type Output = String;

    fn id(&self) -> Uuid {
        self.id
    }

    fn task_type(&self) -> TaskType {
        TaskType::Analysis
    }

    fn to_gpu_buffer(&self) -> Vec<f32> {
        vec![0.0; Self::input_size()]
    }

    fn from_gpu_buffer(_buffer: &[f32]) -> Self::Output {
        String::from("Analysis result")
    }

    fn input_size() -> usize {
        4096
    }

    fn output_size() -> usize {
        1024
    }

    fn kernel_name() -> &'static str {
        "analysis_kernel"
    }

    async fn execute_cpu(&self) -> Result<Self::Output> {
        Ok(format!("Analysis ({:?}): {}", self.analysis_type, &self.input_text[..50.min(self.input_text.len())]))
    }
}

/// Data processing task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataProcessTask {
    pub id: Uuid,
    pub data: Vec<f32>,
    pub operation: DataOperation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataOperation {
    Transform { factor: f32 },
    Filter { threshold: f32 },
    Aggregate { method: String },
}

impl DataProcessTask {
    pub fn new(data: Vec<f32>, operation: DataOperation) -> Self {
        Self {
            id: Uuid::new_v4(),
            data,
            operation,
        }
    }
}

impl Task for DataProcessTask {
    type Output = Vec<f32>;

    fn id(&self) -> Uuid {
        self.id
    }

    fn task_type(&self) -> TaskType {
        TaskType::DataProcessing
    }

    fn to_gpu_buffer(&self) -> Vec<f32> {
        self.data.clone()
    }

    fn from_gpu_buffer(buffer: &[f32]) -> Self::Output {
        buffer.to_vec()
    }

    fn input_size() -> usize {
        1024 // Max data size
    }

    fn output_size() -> usize {
        1024
    }

    fn kernel_name() -> &'static str {
        "data_process_kernel"
    }

    async fn execute_cpu(&self) -> Result<Self::Output> {
        // CPU fallback
        match &self.operation {
            DataOperation::Transform { factor } => {
                Ok(self.data.iter().map(|x| x * factor).collect())
            }
            DataOperation::Filter { threshold } => {
                Ok(self.data.iter().filter(|x| **x > *threshold).copied().collect())
            }
            DataOperation::Aggregate { .. } => {
                Ok(vec![self.data.iter().sum::<f32>() / self.data.len() as f32])
            }
        }
    }
}

/// Multi-agent simulation task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationTask {
    pub id: Uuid,
    pub num_agents: usize,
    pub steps: usize,
    pub environment_params: HashMap<String, f32>,
}

impl SimulationTask {
    pub fn new(num_agents: usize, steps: usize) -> Self {
        Self {
            id: Uuid::new_v4(),
            num_agents,
            steps,
            environment_params: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub steps_completed: usize,
    pub agent_states: Vec<AgentState>,
    pub metrics: HashMap<String, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub id: usize,
    pub position: (f32, f32),
    pub state: String,
}

impl Task for SimulationTask {
    type Output = SimulationResult;

    fn id(&self) -> Uuid {
        self.id
    }

    fn task_type(&self) -> TaskType {
        TaskType::Simulation
    }

    fn to_gpu_buffer(&self) -> Vec<f32> {
        // Encode simulation parameters
        let mut buffer = vec![0.0; Self::input_size()];
        buffer[0] = self.num_agents as f32;
        buffer[1] = self.steps as f32;
        buffer
    }

    fn from_gpu_buffer(buffer: &[f32]) -> Self::Output {
        // Decode simulation results
        SimulationResult {
            steps_completed: buffer[0] as usize,
            agent_states: vec![],
            metrics: HashMap::new(),
        }
    }

    fn input_size() -> usize {
        256 // Simulation parameters
    }

    fn output_size() -> usize {
        1024 // Agent states and metrics
    }

    fn kernel_name() -> &'static str {
        "simulation_kernel"
    }

    async fn execute_cpu(&self) -> Result<Self::Output> {
        // CPU fallback
        Ok(SimulationResult {
            steps_completed: self.steps,
            agent_states: (0..self.num_agents)
                .map(|i| AgentState {
                    id: i,
                    position: (0.0, 0.0),
                    state: "idle".to_string(),
                })
                .collect(),
            metrics: HashMap::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_code_gen_task() {
        let task = CodeGenTask::new("Generate a fibonacci function".to_string(), "rust".to_string());
        let result = task.execute_cpu().await.unwrap();
        assert!(!result.is_empty());
    }

    #[tokio::test]
    async fn test_data_process_task() {
        let task = DataProcessTask::new(
            vec![1.0, 2.0, 3.0, 4.0, 5.0],
            DataOperation::Transform { factor: 2.0 },
        );
        let result = task.execute_cpu().await.unwrap();
        assert_eq!(result, vec![2.0, 4.0, 6.0, 8.0, 10.0]);
    }
}
