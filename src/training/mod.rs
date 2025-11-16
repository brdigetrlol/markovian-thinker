//! Training infrastructure for model learning and weight management

pub mod weight_loader;
pub mod optimizer;
pub mod backprop;
pub mod online_learning;

pub use weight_loader::{WeightLoader, WeightFormat, TensorInfo};
pub use optimizer::{Optimizer, AdamOptimizer, AdamConfig, SGDOptimizer, OptimizerConfig};
pub use backprop::BackpropEngine;
pub use online_learning::{OnlineLearner, LearningConfig, TrainingExample};
