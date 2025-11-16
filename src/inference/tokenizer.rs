//! Tokenizer for text encoding/decoding

use anyhow::Result;
use tiktoken_rs::CoreBPE;

/// Tokenizer for converting text to/from token IDs
pub struct Tokenizer {
    bpe: CoreBPE,
    vocab_size: usize,
}

impl Tokenizer {
    /// Create a new tokenizer using cl100k_base (GPT-4 tokenizer)
    pub fn new() -> Result<Self> {
        let bpe = tiktoken_rs::cl100k_base()?;
        let vocab_size = 100256; // cl100k_base vocab size

        Ok(Self { bpe, vocab_size })
    }

    /// Encode text to token IDs
    pub fn encode(&self, text: &str) -> Vec<usize> {
        self.bpe.encode_with_special_tokens(text)
    }

    /// Decode token IDs to text
    pub fn decode(&self, tokens: &[usize]) -> Result<String> {
        self.bpe.decode(tokens.to_vec())
            .map_err(|e| anyhow::anyhow!("Decode error: {}", e))
    }

    /// Encode with truncation to max length
    pub fn encode_truncated(&self, text: &str, max_length: usize) -> Vec<usize> {
        let tokens = self.encode(text);
        if tokens.len() > max_length {
            tokens[..max_length].to_vec()
        } else {
            tokens
        }
    }

    /// Encode with padding to exact length
    pub fn encode_padded(&self, text: &str, length: usize, pad_token: usize) -> Vec<usize> {
        let mut tokens = self.encode(text);

        if tokens.len() > length {
            tokens.truncate(length);
        } else {
            tokens.resize(length, pad_token);
        }

        tokens
    }

    /// Get vocabulary size
    pub fn vocab_size(&self) -> usize {
        self.vocab_size
    }

    /// Count tokens in text
    pub fn count_tokens(&self, text: &str) -> usize {
        self.encode(text).len()
    }
}

impl Default for Tokenizer {
    fn default() -> Self {
        Self::new().expect("Failed to initialize tokenizer")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer() {
        let tokenizer = Tokenizer::new().unwrap();

        let text = "Hello, world!";
        let tokens = tokenizer.encode(text);
        assert!(tokens.len() > 0);

        let decoded = tokenizer.decode(&tokens).unwrap();
        assert_eq!(decoded, text);
    }

    #[test]
    fn test_truncation() {
        let tokenizer = Tokenizer::new().unwrap();

        let text = "This is a longer text that should be truncated to fit within the maximum token limit.";
        let tokens = tokenizer.encode_truncated(text, 5);
        assert_eq!(tokens.len(), 5);
    }

    #[test]
    fn test_padding() {
        let tokenizer = Tokenizer::new().unwrap();

        let text = "Short";
        let tokens = tokenizer.encode_padded(text, 10, 0);
        assert_eq!(tokens.len(), 10);
    }
}
