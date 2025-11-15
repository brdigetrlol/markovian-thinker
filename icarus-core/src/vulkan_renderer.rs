// Vulkan Renderer for Icarus Cognitive Visualization
// Renders neural activity, agent states, and memory flow in real-time

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

#[cfg(feature = "vulkan")]
use vulkano::{
    device::{Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo},
    instance::{Instance, InstanceCreateInfo},
    VulkanLibrary,
};

/// Cognitive visualization data
#[derive(Debug, Clone)]
pub struct CognitiveVisualization {
    /// Neural core activation levels (0.0-1.0)
    pub neural_activations: Vec<f32>,
    /// Agent activity levels (6 agents)
    pub agent_activities: [f32; 6],
    /// Memory flow (working -> short -> long -> episodic)
    pub memory_flow: [f32; 4],
    /// Current attention focus (x, y coordinates)
    pub attention_point: (f32, f32),
    /// Event timestamps for animation
    pub event_times: Vec<f64>,
}

impl Default for CognitiveVisualization {
    fn default() -> Self {
        Self {
            neural_activations: vec![0.0; 256],
            agent_activities: [0.0; 6],
            memory_flow: [0.0; 4],
            attention_point: (0.5, 0.5),
            event_times: Vec::new(),
        }
    }
}

/// Vulkan-based renderer for cognitive visualization
pub struct VulkanRenderer {
    #[cfg(feature = "vulkan")]
    instance: Arc<Instance>,
    #[cfg(feature = "vulkan")]
    device: Arc<Device>,

    visualization_data: Arc<RwLock<CognitiveVisualization>>,
    enabled: bool,
}

impl VulkanRenderer {
    /// Create a new Vulkan renderer
    pub fn new() -> Result<Self> {
        #[cfg(feature = "vulkan")]
        {
            tracing::info!("🎨 Initializing Vulkan renderer for cognitive visualization");

            let library = VulkanLibrary::new()
                .map_err(|e| anyhow::anyhow!("Failed to load Vulkan library: {}", e))?;

            let instance = Instance::new(
                library,
                InstanceCreateInfo {
                    application_name: Some("Icarus Cognitive Renderer".into()),
                    application_version: vulkano::Version::V1_3,
                    ..Default::default()
                },
            )
            .map_err(|e| anyhow::anyhow!("Failed to create Vulkan instance: {}", e))?;

            // Get the first physical device
            let physical_device = instance
                .enumerate_physical_devices()
                .map_err(|e| anyhow::anyhow!("Failed to enumerate devices: {}", e))?
                .next()
                .ok_or_else(|| anyhow::anyhow!("No Vulkan-compatible GPU found"))?;

            tracing::info!("🎨 Using GPU: {}", physical_device.properties().device_name);

            // Get the first queue family that supports graphics
            let queue_family_index = physical_device
                .queue_family_properties()
                .iter()
                .position(|q| q.queue_flags.graphics)
                .ok_or_else(|| anyhow::anyhow!("No graphics queue family found"))?
                as u32;

            // Create logical device
            let (device, _queues) = Device::new(
                physical_device,
                DeviceCreateInfo {
                    queue_create_infos: vec![QueueCreateInfo {
                        queue_family_index,
                        ..Default::default()
                    }],
                    enabled_extensions: DeviceExtensions::empty(),
                    ..Default::default()
                },
            )
            .map_err(|e| anyhow::anyhow!("Failed to create device: {}", e))?;

            tracing::info!("✅ Vulkan renderer initialized successfully");

            Ok(Self {
                instance,
                device,
                visualization_data: Arc::new(RwLock::new(CognitiveVisualization::default())),
                enabled: true,
            })
        }

        #[cfg(not(feature = "vulkan"))]
        {
            tracing::warn!("⚠️  Vulkan feature not enabled, rendering disabled");
            Ok(Self {
                visualization_data: Arc::new(RwLock::new(CognitiveVisualization::default())),
                enabled: false,
            })
        }
    }

    /// Update neural activation visualization
    pub async fn update_neural_activity(&self, activations: Vec<f32>) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let mut viz = self.visualization_data.write().await;
        viz.neural_activations = activations;

        tracing::debug!("🎨 Updated neural activation visualization");
        Ok(())
    }

    /// Update agent activity levels
    pub async fn update_agent_activity(&self, agent_idx: usize, activity: f32) -> Result<()> {
        if !self.enabled || agent_idx >= 6 {
            return Ok(());
        }

        let mut viz = self.visualization_data.write().await;
        viz.agent_activities[agent_idx] = activity.clamp(0.0, 1.0);

        tracing::debug!("🎨 Agent {} activity: {:.2}", agent_idx, activity);
        Ok(())
    }

    /// Update memory flow visualization
    pub async fn update_memory_flow(&self, flow: [f32; 4]) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let mut viz = self.visualization_data.write().await;
        viz.memory_flow = flow;

        tracing::debug!("🎨 Updated memory flow visualization");
        Ok(())
    }

    /// Update attention focus point
    pub async fn update_attention(&self, x: f32, y: f32) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let mut viz = self.visualization_data.write().await;
        viz.attention_point = (x.clamp(0.0, 1.0), y.clamp(0.0, 1.0));

        Ok(())
    }

    /// Render a frame (called from main loop)
    pub async fn render_frame(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        #[cfg(feature = "vulkan")]
        {
            let viz = self.visualization_data.read().await;

            // TODO: Implement actual Vulkan rendering pipeline
            // This will render:
            // - Neural network activation patterns
            // - 6 agent activity indicators
            // - Memory hierarchy flow visualization
            // - Attention focus point
            // - Event timeline

            tracing::trace!("🎨 Rendered cognitive visualization frame");
        }

        Ok(())
    }

    /// Get current visualization data (for export/inspection)
    pub async fn get_visualization_data(&self) -> CognitiveVisualization {
        self.visualization_data.read().await.clone()
    }

    /// Check if renderer is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Helper function to map agent name to index
pub fn agent_name_to_index(name: &str) -> Option<usize> {
    match name.to_lowercase().as_str() {
        "perception" => Some(0),
        "memory" => Some(1),
        "worldmodel" | "world_model" => Some(2),
        "planning" => Some(3),
        "action" => Some(4),
        "learning" => Some(5),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_renderer_creation() {
        let renderer = VulkanRenderer::new();
        assert!(renderer.is_ok() || !cfg!(feature = "vulkan"));
    }

    #[tokio::test]
    async fn test_agent_index_mapping() {
        assert_eq!(agent_name_to_index("perception"), Some(0));
        assert_eq!(agent_name_to_index("Memory"), Some(1));
        assert_eq!(agent_name_to_index("world_model"), Some(2));
        assert_eq!(agent_name_to_index("invalid"), None);
    }
}
