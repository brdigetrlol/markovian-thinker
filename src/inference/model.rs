//! Inference model for GPU-accelerated text generation

use anyhow::Result;
use std::sync::Arc;

#[cfg(feature = "gpu")]
use crate::gpu::{CudaContext, kernels::*};

use super::tokenizer::Tokenizer;
use super::embeddings::EmbeddingLayer;

/// Model configuration
#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub vocab_size: usize,
    pub embed_dim: usize,
    pub num_heads: usize,
    pub head_dim: usize,
    pub num_layers: usize,
    pub max_seq_len: usize,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            vocab_size: 100256, // cl100k_base vocab size
            embed_dim: 768,
            num_heads: 12,
            head_dim: 64,
            num_layers: 12,
            max_seq_len: 2048,
        }
    }
}

/// Inference model for text generation
pub struct InferenceModel {
    config: ModelConfig,
    tokenizer: Arc<Tokenizer>,
    embedding: Arc<EmbeddingLayer>,

    #[cfg(feature = "gpu")]
    gpu_context: Option<Arc<CudaContext>>,
}

impl InferenceModel {
    /// Create a new inference model
    #[cfg(feature = "gpu")]
    pub fn new(config: ModelConfig, gpu_context: Option<Arc<CudaContext>>) -> Result<Self> {
        let tokenizer = Arc::new(Tokenizer::new()?);
        let embedding = Arc::new(EmbeddingLayer::new(config.vocab_size, config.embed_dim));

        Ok(Self {
            config,
            tokenizer,
            embedding,
            gpu_context,
        })
    }

    /// Create a new inference model without GPU
    #[cfg(not(feature = "gpu"))]
    pub fn new(config: ModelConfig, _gpu_context: ()) -> Result<Self> {
        let tokenizer = Arc::new(Tokenizer::new()?);
        let embedding = Arc::new(EmbeddingLayer::new(config.vocab_size, config.embed_dim));

        Ok(Self {
            config,
            tokenizer,
            embedding,
        })
    }

    /// Generate text from a prompt
    pub async fn generate(&self, prompt: &str, max_new_tokens: usize) -> Result<String> {
        // Tokenize input
        let input_tokens = self.tokenizer.encode_truncated(prompt, self.config.max_seq_len);

        // Generate tokens
        let output_tokens = self.generate_tokens(&input_tokens, max_new_tokens).await?;

        // Decode to text
        self.tokenizer.decode(&output_tokens)
    }

    /// Generate tokens from input token sequence
    async fn generate_tokens(&self, input_tokens: &[usize], max_new_tokens: usize) -> Result<Vec<usize>> {
        let mut tokens = input_tokens.to_vec();

        for _ in 0..max_new_tokens {
            if tokens.len() >= self.config.max_seq_len {
                break;
            }

            // Get next token
            let next_token = self.predict_next_token(&tokens).await?;
            tokens.push(next_token);

            // Stop on end token (simplified - would check for actual EOS token)
            if next_token == 0 {
                break;
            }
        }

        Ok(tokens)
    }

    /// Predict next token given context
    #[cfg(feature = "gpu")]
    async fn predict_next_token(&self, context: &[usize]) -> Result<usize> {
        if let Some(ref gpu_ctx) = self.gpu_context {
            // GPU inference path
            self.predict_next_token_gpu(context, gpu_ctx).await
        } else {
            // CPU fallback
            self.predict_next_token_cpu(context)
        }
    }

    #[cfg(not(feature = "gpu"))]
    async fn predict_next_token(&self, context: &[usize]) -> Result<usize> {
        self.predict_next_token_cpu(context)
    }

    /// GPU-accelerated next token prediction
    #[cfg(feature = "gpu")]
    async fn predict_next_token_gpu(&self, context: &[usize], gpu_ctx: &Arc<CudaContext>) -> Result<usize> {
        use cudarc::driver::DeviceRepr;

        // Embed input tokens
        let input_embeddings = self.embedding.embed_sequence(context)?;

        let seq_len = context.len();
        let embed_dim = self.config.embed_dim;

        // Get GPU stream
        let stream = gpu_ctx.next_stream();
        let device = gpu_ctx.device();

        // Upload to GPU
        let input_gpu = device.htod_copy(input_embeddings.clone())?;
        let output_gpu = device.htod_copy(vec![0.0f32; seq_len * embed_dim])?;

        // Layer norm parameters (initialized to identity transform)
        let gamma = device.htod_copy(vec![1.0f32; embed_dim])?;
        let beta = device.htod_copy(vec![0.0f32; embed_dim])?;

        // Launch token processing kernel
        let kernels = gpu_ctx.kernels();
        let token_kernel = BatchTokenProcessKernel::new(kernels.clone());

        token_kernel.launch(
            &input_gpu,
            &output_gpu,
            &gamma,
            &beta,
            1, // batch_size = 1
            seq_len,
            embed_dim,
            &stream,
        )?;

        // Synchronize
        stream.synchronize()?;

        // Copy results back
        let output_embeddings = device.dtoh_sync_copy(&output_gpu)?;

        // Get last token's embedding (auto-regressive)
        let last_embed_start = (seq_len - 1) * embed_dim;
        let last_embed = &output_embeddings[last_embed_start..last_embed_start + embed_dim];

        // Unembed to get next token
        let next_token = self.embedding.unembed(last_embed)?;

        Ok(next_token)
    }

    /// CPU fallback next token prediction
    fn predict_next_token_cpu(&self, context: &[usize]) -> Result<usize> {
        // Simple CPU inference (very basic)
        // In practice, would run full model on CPU

        // For now, use a simple heuristic: cycle through vocabulary
        let next_token = if context.is_empty() {
            1 // Start token
        } else {
            (context.last().unwrap() + 1) % self.config.vocab_size
        };

        Ok(next_token)
    }

    /// Encode text to tokens
    pub fn encode(&self, text: &str) -> Vec<usize> {
        self.tokenizer.encode(text)
    }

    /// Decode tokens to text
    pub fn decode(&self, tokens: &[usize]) -> Result<String> {
        self.tokenizer.decode(tokens)
    }

    /// Get model configuration
    pub fn config(&self) -> &ModelConfig {
        &self.config
    }

    /// Get tokenizer
    pub fn tokenizer(&self) -> Arc<Tokenizer> {
        self.tokenizer.clone()
    }

    /// Get embedding layer
    pub fn embedding(&self) -> Arc<EmbeddingLayer> {
        self.embedding.clone()
    }

    /// Get mutable reference to embedding layer (for training)
    pub fn embedding_mut(&mut self) -> &mut Arc<EmbeddingLayer> {
        &mut self.embedding
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_model_creation() {
        let config = ModelConfig::default();
        #[cfg(feature = "gpu")]
        let model = InferenceModel::new(config, None).unwrap();
        #[cfg(not(feature = "gpu"))]
        let model = InferenceModel::new(config, ()).unwrap();

        assert_eq!(model.config.vocab_size, 100256);
    }

    #[tokio::test]
    async fn test_generation() {
        let config = ModelConfig::default();
        #[cfg(feature = "gpu")]
        let model = InferenceModel::new(config, None).unwrap();
        #[cfg(not(feature = "gpu"))]
        let model = InferenceModel::new(config, ()).unwrap();

        let result = model.generate("Hello", 5).await.unwrap();
        assert!(!result.is_empty());
    }

    #[tokio::test]
    async fn test_tokenization() {
        let config = ModelConfig::default();
        #[cfg(feature = "gpu")]
        let model = InferenceModel::new(config, None).unwrap();
        #[cfg(not(feature = "gpu"))]
        let model = InferenceModel::new(config, ()).unwrap();

        let text = "Hello, world!";
        let tokens = model.encode(text);
        let decoded = model.decode(&tokens).unwrap();

        assert_eq!(decoded, text);
    }
}
