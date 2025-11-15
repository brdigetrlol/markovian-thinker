// Agent System
// 6-agent cognitive architecture for Icarus

use crate::config::AgentConfig;
use crate::event_bus::{EventBus, IcarusEvent, Modality, MemoryOp};
use crate::memory::MemoryHierarchy;
use crate::neural::NeuralCore;
use crate::world_model::WorldModel;
use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

/// Agent types in Icarus
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentType {
    /// Perception - Processes incoming data streams
    Perception,
    /// WorldModel - Maintains predictive model of environment
    WorldModel,
    /// Planning - Generates action plans based on goals
    Planning,
    /// Memory - Manages hierarchical memory system
    Memory,
    /// Action - Executes actions in environment
    Action,
    /// Learning - Extracts strategies and updates models
    Learning,
}

/// Agent trait - All agents must implement this
#[async_trait]
pub trait Agent: Send + Sync {
    /// Get agent type
    fn agent_type(&self) -> AgentType;

    /// Initialize agent
    async fn initialize(&mut self) -> Result<()>;

    /// Process one step/cycle
    async fn step(&mut self) -> Result<()>;

    /// Shutdown agent
    async fn shutdown(&mut self) -> Result<()>;

    /// Handle incoming event
    async fn handle_event(&mut self, event: &IcarusEvent) -> Result<()>;
}

/// Agent System - Manages all 6 agents
pub struct AgentSystem {
    config: AgentConfig,
    agents: Vec<Box<dyn Agent>>,
    event_bus: Arc<EventBus>,
    running: Arc<RwLock<bool>>,
}

impl AgentSystem {
    /// Create new agent system
    pub fn new(
        config: &AgentConfig,
        event_bus: Arc<EventBus>,
        memory: Arc<RwLock<MemoryHierarchy>>,
        neural_core: Arc<RwLock<NeuralCore>>,
        world_model: Arc<RwLock<WorldModel>>,
    ) -> Result<Self> {
        let mut agents: Vec<Box<dyn Agent>> = Vec::new();

        // Create all 6 agents
        agents.push(Box::new(PerceptionAgent::new(
            event_bus.clone(),
            memory.clone(),
        )));

        agents.push(Box::new(WorldModelAgent::new(
            event_bus.clone(),
            world_model.clone(),
        )));

        agents.push(Box::new(PlanningAgent::new(
            event_bus.clone(),
            memory.clone(),
            neural_core.clone(),
        )));

        agents.push(Box::new(MemoryAgent::new(
            event_bus.clone(),
            memory.clone(),
        )));

        agents.push(Box::new(ActionAgent::new(
            event_bus.clone(),
        )));

        agents.push(Box::new(LearningAgent::new(
            event_bus.clone(),
            neural_core.clone(),
        )));

        Ok(Self {
            config: config.clone(),
            agents,
            event_bus,
            running: Arc::new(RwLock::new(false)),
        })
    }

    /// Start all agents
    pub async fn start(&mut self) -> Result<()> {
        tracing::info!("Starting agent system ({} agents)", self.agents.len());

        *self.running.write().await = true;

        // Initialize all agents
        for agent in &mut self.agents {
            agent.initialize().await?;
            tracing::debug!("  ✓ {:?} agent initialized", agent.agent_type());
        }

        // Spawn agent update tasks
        for agent_idx in 0..self.agents.len() {
            let running = self.running.clone();
            let event_bus = self.event_bus.clone();
            let update_interval = tokio::time::Duration::from_millis(self.config.update_interval_ms);

            // Note: We can't move agents out, so we'll handle this differently in production
            // For now, this is the scaffolding structure
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(update_interval);
                while *running.read().await {
                    interval.tick().await;
                    // Agent step would happen here
                    // In production, agents would be in separate Arc<RwLock<>> containers
                }
            });
        }

        tracing::info!("✅ All agents started");

        Ok(())
    }

    /// Stop all agents
    pub async fn stop(&mut self) -> Result<()> {
        tracing::info!("Stopping agent system");

        *self.running.write().await = false;

        // Shutdown all agents
        for agent in &mut self.agents {
            agent.shutdown().await?;
        }

        tracing::info!("✅ All agents stopped");

        Ok(())
    }
}

// ============================================================================
// Individual Agent Implementations
// ============================================================================

/// Perception Agent - Processes incoming data streams
struct PerceptionAgent {
    event_bus: Arc<EventBus>,
    memory: Arc<RwLock<MemoryHierarchy>>,
    event_receiver: Option<broadcast::Receiver<IcarusEvent>>,
}

impl PerceptionAgent {
    fn new(event_bus: Arc<EventBus>, memory: Arc<RwLock<MemoryHierarchy>>) -> Self {
        Self {
            event_bus,
            memory,
            event_receiver: None,
        }
    }
}

#[async_trait]
impl Agent for PerceptionAgent {
    fn agent_type(&self) -> AgentType {
        AgentType::Perception
    }

    async fn initialize(&mut self) -> Result<()> {
        tracing::debug!("Perception agent initializing");
        self.event_receiver = Some(self.event_bus.subscribe());
        Ok(())
    }

    async fn step(&mut self) -> Result<()> {
        // Process incoming perceptions from event bus
        if let Some(rx) = &mut self.event_receiver {
            match rx.try_recv() {
                Ok(event) => {
                    self.handle_event(&event).await?;
                }
                Err(broadcast::error::TryRecvError::Empty) => {
                    // No events, continue
                }
                Err(broadcast::error::TryRecvError::Lagged(_)) => {
                    tracing::warn!("Perception agent lagged behind event bus");
                }
                Err(broadcast::error::TryRecvError::Closed) => {
                    tracing::error!("Event bus closed");
                }
            }
        }
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::debug!("Perception agent shutting down");
        self.event_receiver = None;
        Ok(())
    }

    async fn handle_event(&mut self, event: &IcarusEvent) -> Result<()> {
        // Process perception events and store them in working memory
        match event {
            IcarusEvent::PerceptionInput { id, data, modality, .. } => {
                tracing::debug!("Processing perception input: {:?} ({})", modality, data.len());

                // Create memory from perception
                let mut memory = crate::memory::Memory::new(data.clone());
                memory.importance = 0.6; // Moderate importance for new perceptions

                // Store in working memory
                let mut mem_hierarchy = self.memory.write().await;
                mem_hierarchy.store(memory);

                tracing::debug!("Stored perception {} in working memory", id);
            }
            IcarusEvent::SystemControl { command, .. } => {
                tracing::debug!("Perception agent received system command: {:?}", command);
            }
            _ => {
                // Ignore other event types
            }
        }
        Ok(())
    }
}

/// World Model Agent - Maintains predictive model
struct WorldModelAgent {
    event_bus: Arc<EventBus>,
    world_model: Arc<RwLock<WorldModel>>,
    event_receiver: Option<broadcast::Receiver<IcarusEvent>>,
}

impl WorldModelAgent {
    fn new(event_bus: Arc<EventBus>, world_model: Arc<RwLock<WorldModel>>) -> Self {
        Self {
            event_bus,
            world_model,
            event_receiver: None,
        }
    }
}

#[async_trait]
impl Agent for WorldModelAgent {
    fn agent_type(&self) -> AgentType {
        AgentType::WorldModel
    }

    async fn initialize(&mut self) -> Result<()> {
        tracing::debug!("World Model agent initializing");
        self.event_receiver = Some(self.event_bus.subscribe());
        Ok(())
    }

    async fn step(&mut self) -> Result<()> {
        // Update world model predictions periodically
        let mut wm = self.world_model.write().await;
        wm.step().await?;

        // Process any pending events
        if let Some(rx) = &mut self.event_receiver {
            match rx.try_recv() {
                Ok(event) => {
                    drop(wm); // Release lock before handling event
                    self.handle_event(&event).await?;
                }
                Err(broadcast::error::TryRecvError::Empty) => {}
                Err(broadcast::error::TryRecvError::Lagged(_)) => {
                    tracing::warn!("WorldModel agent lagged behind event bus");
                }
                Err(broadcast::error::TryRecvError::Closed) => {
                    tracing::error!("Event bus closed");
                }
            }
        }

        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::debug!("World Model agent shutting down");
        self.event_receiver = None;
        Ok(())
    }

    async fn handle_event(&mut self, event: &IcarusEvent) -> Result<()> {
        match event {
            IcarusEvent::PerceptionInput { data, .. } => {
                // Convert perception data to observation vector
                let observation: Vec<f32> = data
                    .iter()
                    .map(|&b| b as f32 / 255.0)
                    .collect();

                // Update world model with new observation
                let mut wm = self.world_model.write().await;
                wm.observe(observation);

                // Publish world model update event
                let predictions: Vec<f32> = wm
                    .predictions()
                    .get(0)
                    .map(|p| p.state.clone())
                    .unwrap_or_default();

                drop(wm);

                self.event_bus.publish(IcarusEvent::WorldModelUpdated {
                    id: Uuid::new_v4(),
                    timestamp: Utc::now(),
                    predictions,
                });

                tracing::debug!("Updated world model with new observation");
            }
            IcarusEvent::ActionExecuted { action, success, .. } => {
                tracing::debug!("World model learning from action: {} (success: {})", action, success);
                // In production, update model based on action outcomes
            }
            _ => {}
        }
        Ok(())
    }
}

/// Planning Agent - Generates action plans
struct PlanningAgent {
    event_bus: Arc<EventBus>,
    memory: Arc<RwLock<MemoryHierarchy>>,
    neural_core: Arc<RwLock<NeuralCore>>,
    event_receiver: Option<broadcast::Receiver<IcarusEvent>>,
    // TODO: Integrate markovian-thinker SessionManager once it's publicly exported
    // session_manager: Arc<RwLock<markovian_thinker::SessionManager>>,
}

impl PlanningAgent {
    fn new(
        event_bus: Arc<EventBus>,
        memory: Arc<RwLock<MemoryHierarchy>>,
        neural_core: Arc<RwLock<NeuralCore>>,
    ) -> Self {
        // TODO: Create session manager for complex planning/reasoning tasks
        // let session_manager = markovian_thinker::SessionManager::new();

        Self {
            event_bus,
            memory,
            neural_core,
            event_receiver: None,
            // session_manager: Arc::new(RwLock::new(session_manager)),
        }
    }
}

#[async_trait]
impl Agent for PlanningAgent {
    fn agent_type(&self) -> AgentType {
        AgentType::Planning
    }

    async fn initialize(&mut self) -> Result<()> {
        tracing::debug!("Planning agent initializing");
        self.event_receiver = Some(self.event_bus.subscribe());
        Ok(())
    }

    async fn step(&mut self) -> Result<()> {
        // Process events and generate plans as needed
        if let Some(rx) = &mut self.event_receiver {
            match rx.try_recv() {
                Ok(event) => {
                    self.handle_event(&event).await?;
                }
                Err(broadcast::error::TryRecvError::Empty) => {}
                Err(broadcast::error::TryRecvError::Lagged(_)) => {
                    tracing::warn!("Planning agent lagged behind event bus");
                }
                Err(broadcast::error::TryRecvError::Closed) => {
                    tracing::error!("Event bus closed");
                }
            }
        }
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::debug!("Planning agent shutting down");
        self.event_receiver = None;
        Ok(())
    }

    async fn handle_event(&mut self, event: &IcarusEvent) -> Result<()> {
        match event {
            IcarusEvent::WorldModelUpdated { predictions, .. } => {
                // Use world model predictions to inform planning
                tracing::debug!("Planning agent received world model update with {} predictions", predictions.len());

                // For complex planning, we could initiate a markovian-thinker session
                // For now, generate a plan based on predictions
                let plan = if predictions.len() > 0 {
                    format!("Execute action based on {} world model predictions", predictions.len())
                } else {
                    "Observe and wait for more information".to_string()
                };

                self.event_bus.publish(IcarusEvent::PlanGenerated {
                    id: Uuid::new_v4(),
                    timestamp: Utc::now(),
                    plan,
                    priority: 0.8,
                });

                tracing::info!("Planning agent generated plan");
            }
            IcarusEvent::PerceptionInput { data, modality, .. } => {
                // New perception may trigger planning for complex scenarios
                tracing::debug!("Planning agent processing perception: {:?} ({} bytes)", modality, data.len());

                // For complex reasoning, we could use markovian-thinker SessionManager
                // Example: Create reasoning session for multi-step planning
                // let mut sm = self.session_manager.write().await;
                // let config = StateConfig::default();
                // let session_id = sm.create_session("Plan actions based on perception", config)?;
            }
            IcarusEvent::LearningUpdate { strategy, .. } => {
                // Incorporate learning into future plans
                tracing::debug!("Planning agent received learning update: {}", strategy);
            }
            _ => {}
        }
        Ok(())
    }
}

/// Memory Agent - Manages hierarchical memory
struct MemoryAgent {
    event_bus: Arc<EventBus>,
    memory: Arc<RwLock<MemoryHierarchy>>,
    event_receiver: Option<broadcast::Receiver<IcarusEvent>>,
}

impl MemoryAgent {
    fn new(event_bus: Arc<EventBus>, memory: Arc<RwLock<MemoryHierarchy>>) -> Self {
        Self {
            event_bus,
            memory,
            event_receiver: None,
        }
    }
}

#[async_trait]
impl Agent for MemoryAgent {
    fn agent_type(&self) -> AgentType {
        AgentType::Memory
    }

    async fn initialize(&mut self) -> Result<()> {
        tracing::debug!("Memory agent initializing");
        self.event_receiver = Some(self.event_bus.subscribe());
        Ok(())
    }

    async fn step(&mut self) -> Result<()> {
        // Periodically consolidate memories across hierarchy levels
        let mut mem = self.memory.write().await;
        mem.consolidate().await?;
        drop(mem);

        // Process events
        if let Some(rx) = &mut self.event_receiver {
            match rx.try_recv() {
                Ok(event) => {
                    self.handle_event(&event).await?;
                }
                Err(broadcast::error::TryRecvError::Empty) => {}
                Err(broadcast::error::TryRecvError::Lagged(_)) => {
                    tracing::warn!("Memory agent lagged behind event bus");
                }
                Err(broadcast::error::TryRecvError::Closed) => {
                    tracing::error!("Event bus closed");
                }
            }
        }

        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::debug!("Memory agent shutting down");
        self.event_receiver = None;
        Ok(())
    }

    async fn handle_event(&mut self, event: &IcarusEvent) -> Result<()> {
        match event {
            IcarusEvent::PerceptionInput { id, data, .. } => {
                // Store important perceptions
                let memory = crate::memory::Memory::new(data.clone());
                let mut mem = self.memory.write().await;
                mem.store(memory);

                self.event_bus.publish(IcarusEvent::MemoryOperation {
                    id: Uuid::new_v4(),
                    timestamp: Utc::now(),
                    operation: MemoryOp::Store,
                    level: "working".to_string(),
                });

                tracing::debug!("Memory agent stored perception {}", id);
            }
            IcarusEvent::PlanGenerated { plan, .. } => {
                // Store generated plans for future reference
                let memory = crate::memory::Memory::new(plan.as_bytes().to_vec());
                let mut mem = self.memory.write().await;
                mem.store(memory);

                tracing::debug!("Memory agent stored plan");
            }
            IcarusEvent::ActionExecuted { action, success, .. } => {
                // Store action outcomes for learning
                let outcome = format!("action: {}, success: {}", action, success);
                let mut memory = crate::memory::Memory::new(outcome.as_bytes().to_vec());
                memory.importance = if *success { 0.9 } else { 0.7 };

                let mut mem = self.memory.write().await;
                mem.store(memory);

                tracing::debug!("Memory agent stored action outcome");
            }
            _ => {}
        }
        Ok(())
    }
}

/// Action Agent - Executes actions
struct ActionAgent {
    event_bus: Arc<EventBus>,
    event_receiver: Option<broadcast::Receiver<IcarusEvent>>,
    pending_actions: Vec<String>,
}

impl ActionAgent {
    fn new(event_bus: Arc<EventBus>) -> Self {
        Self {
            event_bus,
            event_receiver: None,
            pending_actions: Vec::new(),
        }
    }
}

#[async_trait]
impl Agent for ActionAgent {
    fn agent_type(&self) -> AgentType {
        AgentType::Action
    }

    async fn initialize(&mut self) -> Result<()> {
        tracing::debug!("Action agent initializing");
        self.event_receiver = Some(self.event_bus.subscribe());
        Ok(())
    }

    async fn step(&mut self) -> Result<()> {
        // Process pending actions
        if let Some(action) = self.pending_actions.pop() {
            // Execute action (in production, this would call external systems)
            tracing::info!("Executing action: {}", action);

            // Simulate action execution
            let success = true; // In production: actual execution result

            self.event_bus.publish(IcarusEvent::ActionExecuted {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                action,
                success,
            });
        }

        // Process events
        if let Some(rx) = &mut self.event_receiver {
            match rx.try_recv() {
                Ok(event) => {
                    self.handle_event(&event).await?;
                }
                Err(broadcast::error::TryRecvError::Empty) => {}
                Err(broadcast::error::TryRecvError::Lagged(_)) => {
                    tracing::warn!("Action agent lagged behind event bus");
                }
                Err(broadcast::error::TryRecvError::Closed) => {
                    tracing::error!("Event bus closed");
                }
            }
        }

        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::debug!("Action agent shutting down");
        self.event_receiver = None;
        Ok(())
    }

    async fn handle_event(&mut self, event: &IcarusEvent) -> Result<()> {
        match event {
            IcarusEvent::PlanGenerated { plan, priority, .. } => {
                tracing::debug!("Action agent received plan (priority: {}): {}", priority, plan);

                // Extract actions from plan and queue them
                // For now, treat the entire plan as one action
                self.pending_actions.push(plan.clone());
            }
            _ => {}
        }
        Ok(())
    }
}

/// Learning Agent - Extracts strategies and updates models
struct LearningAgent {
    event_bus: Arc<EventBus>,
    neural_core: Arc<RwLock<NeuralCore>>,
    event_receiver: Option<broadcast::Receiver<IcarusEvent>>,
    experience_buffer: Vec<(String, bool)>, // (action, success) pairs
}

impl LearningAgent {
    fn new(event_bus: Arc<EventBus>, neural_core: Arc<RwLock<NeuralCore>>) -> Self {
        Self {
            event_bus,
            neural_core,
            event_receiver: None,
            experience_buffer: Vec::new(),
        }
    }
}

#[async_trait]
impl Agent for LearningAgent {
    fn agent_type(&self) -> AgentType {
        AgentType::Learning
    }

    async fn initialize(&mut self) -> Result<()> {
        tracing::debug!("Learning agent initializing");
        self.event_receiver = Some(self.event_bus.subscribe());
        Ok(())
    }

    async fn step(&mut self) -> Result<()> {
        // Periodically extract strategies from experience buffer
        if self.experience_buffer.len() >= 10 {
            let successes = self.experience_buffer.iter().filter(|(_, s)| *s).count();
            let total = self.experience_buffer.len();
            let success_rate = successes as f32 / total as f32;

            tracing::info!(
                "Learning agent: {} experiences, {:.1}% success rate",
                total,
                success_rate * 100.0
            );

            // In production: Update neural core based on experience
            let strategy = format!("success_rate: {:.2}", success_rate);

            self.event_bus.publish(IcarusEvent::LearningUpdate {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                strategy,
            });

            // Clear old experiences (keep sliding window)
            if self.experience_buffer.len() > 100 {
                self.experience_buffer.drain(0..50);
            }
        }

        // Process events
        if let Some(rx) = &mut self.event_receiver {
            match rx.try_recv() {
                Ok(event) => {
                    self.handle_event(&event).await?;
                }
                Err(broadcast::error::TryRecvError::Empty) => {}
                Err(broadcast::error::TryRecvError::Lagged(_)) => {
                    tracing::warn!("Learning agent lagged behind event bus");
                }
                Err(broadcast::error::TryRecvError::Closed) => {
                    tracing::error!("Event bus closed");
                }
            }
        }

        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::debug!("Learning agent shutting down");
        self.event_receiver = None;
        Ok(())
    }

    async fn handle_event(&mut self, event: &IcarusEvent) -> Result<()> {
        match event {
            IcarusEvent::ActionExecuted { action, success, .. } => {
                // Record experience for learning
                self.experience_buffer.push((action.clone(), *success));
                tracing::debug!("Learning agent recorded experience: {} -> {}", action, success);
            }
            IcarusEvent::WorldModelUpdated { predictions, .. } => {
                // Learn from prediction accuracy
                tracing::debug!("Learning agent analyzing {} predictions", predictions.len());
            }
            _ => {}
        }
        Ok(())
    }
}
