// Icarus Core: Novel Cognitive AI Architecture
//
// Icarus is a standalone AI system implementing 12 paradigms beyond traditional LLMs:
// 1. Streams - Continuous processing instead of turn-based
// 2. Action-Centric - Primary output is actions, not text
// 3. World Models - Internal predictive simulation
// 4. State Space Models - Linear O(n) complexity via SSM/Mamba
// 5. Agentic - Multi-agent cognitive system
// 6. Hierarchical Memory - Working, short-term, long-term, episodic
// 7. Evolutionary/Adaptive - Dynamic composition
// 8. Retrieval-Augmented - H²CE semantic search
// 9. Liquid Neural Networks - Time-continuous adaptive dynamics
// 10. Multimodal-First - Native code/text/data processing
// 11. Self-Improving - Continuous learning via strategy extraction
// 12. Modern RNNs - Recurrent processing for streaming

pub mod agents;
pub mod config;
pub mod memory;
pub mod neural;
pub mod world_model;
pub mod streams;
pub mod event_bus;
pub mod mcp;
pub mod vulkan_renderer;
pub mod tic;  // TIC (Topological Information Crystallography) substrate

// Re-export core types
pub use config::IcarusConfig;
pub use agents::{Agent, AgentType, AgentSystem};
pub use memory::{Memory, MemoryHierarchy, MemoryLevel};
pub use neural::{NeuralCore, NeuralState};
pub use world_model::WorldModel;
pub use event_bus::EventBus;
pub use mcp::IcarusMCPServer;
pub use vulkan_renderer::{VulkanRenderer, CognitiveVisualization};
pub use tic::TICSubstrate;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

/// The Icarus Core - Main cognitive system
pub struct IcarusCore {
    /// Configuration
    config: IcarusConfig,

    /// 6-agent cognitive system
    agent_system: AgentSystem,

    /// Hierarchical memory subsystem
    memory: Arc<RwLock<MemoryHierarchy>>,

    /// Neural core (SSM/Liquid/RNN hybrid)
    neural_core: Arc<RwLock<NeuralCore>>,

    /// World model for predictive simulation
    world_model: Arc<RwLock<WorldModel>>,

    /// Event bus for agent communication
    event_bus: Arc<EventBus>,

    /// Vulkan renderer for cognitive visualization (optional)
    #[allow(dead_code)]
    vulkan_renderer: Option<vulkan_renderer::VulkanRenderer>,

    /// Running state
    running: Arc<RwLock<bool>>,
}

impl IcarusCore {
    /// Create new Icarus instance
    pub async fn new(config: IcarusConfig) -> Result<Self> {
        tracing::info!("Initializing Icarus cognitive system");

        // Initialize event bus
        let event_bus = Arc::new(EventBus::new());

        // Initialize memory hierarchy
        let memory = Arc::new(RwLock::new(MemoryHierarchy::new(&config.memory)?));

        // Initialize neural core
        let neural_core = Arc::new(RwLock::new(NeuralCore::new(&config.neural)?));

        // Initialize world model
        let world_model = Arc::new(RwLock::new(WorldModel::new(&config.world_model)?));

        // Initialize agent system
        let agent_system = AgentSystem::new(
            &config.agents,
            event_bus.clone(),
            memory.clone(),
            neural_core.clone(),
            world_model.clone(),
        )?;

        // Initialize Vulkan renderer (optional, may fail if no GPU/Vulkan)
        let vulkan_renderer = match vulkan_renderer::VulkanRenderer::new() {
            Ok(renderer) => {
                tracing::info!("✅ Vulkan renderer initialized");
                Some(renderer)
            }
            Err(e) => {
                tracing::warn!("⚠️  Vulkan renderer initialization failed: {}", e);
                None
            }
        };

        Ok(Self {
            config,
            agent_system,
            memory,
            neural_core,
            world_model,
            event_bus,
            vulkan_renderer,
            running: Arc::new(RwLock::new(false)),
        })
    }

    /// Run Icarus autonomous operation
    pub async fn run(&mut self) -> Result<()> {
        tracing::info!("Starting Icarus autonomous operation");

        *self.running.write().await = true;

        // Start agent system
        self.agent_system.start().await?;

        // Main event loop
        while *self.running.read().await {
            // Process events from event bus
            self.process_events().await?;

            // Update world model
            self.update_world_model().await?;

            // Consolidate memories
            self.consolidate_memories().await?;

            // Small delay to prevent busy loop
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // Shutdown
        self.agent_system.stop().await?;

        tracing::info!("Icarus shutdown complete");

        Ok(())
    }

    /// Process events from the event bus
    async fn process_events(&self) -> Result<()> {
        // Event processing will be implemented as agents communicate
        // For now, this is a placeholder
        Ok(())
    }

    /// Update world model with latest observations
    async fn update_world_model(&self) -> Result<()> {
        let mut wm = self.world_model.write().await;
        wm.step().await?;
        Ok(())
    }

    /// Consolidate memories across hierarchy levels
    async fn consolidate_memories(&self) -> Result<()> {
        let mut mem = self.memory.write().await;
        mem.consolidate().await?;
        Ok(())
    }

    /// Stop Icarus
    pub async fn stop(&mut self) -> Result<()> {
        tracing::info!("Stopping Icarus");
        *self.running.write().await = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_icarus_creation() {
        let config = IcarusConfig::default();
        let icarus = IcarusCore::new(config).await;
        assert!(icarus.is_ok());
    }
}
