//! Embedding layer for converting tokens to/from vector representations

use anyhow::Result;
use rand::Rng;

/// Embedding layer that converts token IDs to continuous vectors
pub struct EmbeddingLayer {
    /// Vocabulary size
    vocab_size: usize,
    /// Embedding dimension
    embed_dim: usize,
    /// Embedding weights [vocab_size, embed_dim]
    /// In a real implementation, these would be loaded from a trained model
    weights: Vec<f32>,
}

impl EmbeddingLayer {
    /// Create a new embedding layer with random initialization
    pub fn new(vocab_size: usize, embed_dim: usize) -> Self {
        let mut rng = rand::thread_rng();
        let size = vocab_size * embed_dim;
        let weights: Vec<f32> = (0..size)
            .map(|_| rng.gen_range(-0.1..0.1))
            .collect();

        Self {
            vocab_size,
            embed_dim,
            weights,
        }
    }

    /// Create embedding layer with zeros (for testing)
    pub fn zeros(vocab_size: usize, embed_dim: usize) -> Self {
        Self {
            vocab_size,
            embed_dim,
            weights: vec![0.0; vocab_size * embed_dim],
        }
    }

    /// Embed a single token ID
    pub fn embed_token(&self, token_id: usize) -> Result<&[f32]> {
        if token_id >= self.vocab_size {
            anyhow::bail!("Token ID {} out of range (vocab size: {})", token_id, self.vocab_size);
        }

        let start = token_id * self.embed_dim;
        let end = start + self.embed_dim;
        Ok(&self.weights[start..end])
    }

    /// Embed a sequence of tokens
    /// Returns flat vector [seq_len * embed_dim]
    pub fn embed_sequence(&self, token_ids: &[usize]) -> Result<Vec<f32>> {
        let mut embeddings = Vec::with_capacity(token_ids.len() * self.embed_dim);

        for &token_id in token_ids {
            let embed = self.embed_token(token_id)?;
            embeddings.extend_from_slice(embed);
        }

        Ok(embeddings)
    }

    /// Embed a batch of sequences
    /// Returns flat vector [batch_size * seq_len * embed_dim]
    /// All sequences must have the same length (padded if necessary)
    pub fn embed_batch(&self, batch: &[Vec<usize>]) -> Result<Vec<f32>> {
        if batch.is_empty() {
            return Ok(Vec::new());
        }

        let seq_len = batch[0].len();
        // Verify all sequences have same length
        for seq in batch {
            if seq.len() != seq_len {
                anyhow::bail!("All sequences in batch must have same length. Expected {}, got {}", seq_len, seq.len());
            }
        }

        let mut embeddings = Vec::with_capacity(batch.len() * seq_len * self.embed_dim);

        for sequence in batch {
            let seq_embed = self.embed_sequence(sequence)?;
            embeddings.extend(seq_embed);
        }

        Ok(embeddings)
    }

    /// Unembed: Convert embedding vectors back to token IDs (greedy decoding)
    /// Finds nearest embedding in vocabulary
    pub fn unembed(&self, embedding: &[f32]) -> Result<usize> {
        if embedding.len() != self.embed_dim {
            anyhow::bail!("Embedding size mismatch: expected {}, got {}", self.embed_dim, embedding.len());
        }

        let mut best_token = 0;
        let mut best_distance = f32::INFINITY;

        for token_id in 0..self.vocab_size {
            let vocab_embed = self.embed_token(token_id)?;

            // Compute L2 distance
            let distance: f32 = embedding.iter()
                .zip(vocab_embed.iter())
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f32>()
                .sqrt();

            if distance < best_distance {
                best_distance = distance;
                best_token = token_id;
            }
        }

        Ok(best_token)
    }

    /// Unembed a sequence of embeddings
    pub fn unembed_sequence(&self, embeddings: &[f32]) -> Result<Vec<usize>> {
        if embeddings.len() % self.embed_dim != 0 {
            anyhow::bail!("Embeddings length must be multiple of embed_dim");
        }

        let seq_len = embeddings.len() / self.embed_dim;
        let mut tokens = Vec::with_capacity(seq_len);

        for i in 0..seq_len {
            let start = i * self.embed_dim;
            let end = start + self.embed_dim;
            let token = self.unembed(&embeddings[start..end])?;
            tokens.push(token);
        }

        Ok(tokens)
    }

    /// Get embedding dimension
    pub fn embed_dim(&self) -> usize {
        self.embed_dim
    }

    /// Get vocabulary size
    pub fn vocab_size(&self) -> usize {
        self.vocab_size
    }

    /// Get mutable access to weights (for training)
    pub fn weights_mut(&mut self) -> &mut [f32] {
        &mut self.weights
    }

    /// Get immutable access to weights
    pub fn weights(&self) -> &[f32] {
        &self.weights
    }

    /// Load weights from a flat array
    pub fn load_weights(&mut self, weights: Vec<f32>) -> Result<()> {
        if weights.len() != self.vocab_size * self.embed_dim {
            anyhow::bail!(
                "Weight size mismatch: expected {}, got {}",
                self.vocab_size * self.embed_dim,
                weights.len()
            );
        }
        self.weights = weights;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding() {
        let embed_layer = EmbeddingLayer::new(1000, 128);

        let token_id = 42;
        let embedding = embed_layer.embed_token(token_id).unwrap();
        assert_eq!(embedding.len(), 128);
    }

    #[test]
    fn test_sequence_embedding() {
        let embed_layer = EmbeddingLayer::new(1000, 128);

        let tokens = vec![1, 2, 3, 4, 5];
        let embeddings = embed_layer.embed_sequence(&tokens).unwrap();
        assert_eq!(embeddings.len(), tokens.len() * 128);
    }

    #[test]
    fn test_batch_embedding() {
        let embed_layer = EmbeddingLayer::new(1000, 128);

        let batch = vec![
            vec![1, 2, 3],
            vec![4, 5, 6],
            vec![7, 8, 9],
        ];

        let embeddings = embed_layer.embed_batch(&batch).unwrap();
        assert_eq!(embeddings.len(), 3 * 3 * 128);
    }

    #[test]
    fn test_unembed() {
        let embed_layer = EmbeddingLayer::zeros(1000, 128);

        let token_id = 42;
        let embedding = embed_layer.embed_token(token_id).unwrap();
        let recovered = embed_layer.unembed(embedding).unwrap();

        // With zeros, all tokens have same embedding, so any token is valid
        assert!(recovered < 1000);
    }
}
