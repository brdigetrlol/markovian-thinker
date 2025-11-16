//! Weight loading from various model formats

use anyhow::{Context, Result};
use safetensors::SafeTensors;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use memmap2::Mmap;

/// Supported weight formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeightFormat {
    /// SafeTensors format (HuggingFace, PyTorch)
    SafeTensors,
    /// GGUF format (llama.cpp)
    GGUF,
    /// Raw binary format
    Binary,
    /// Custom format (bincode serialized)
    Custom,
}

impl WeightFormat {
    /// Detect format from file extension
    pub fn from_path(path: &Path) -> Result<Self> {
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| anyhow::anyhow!("No file extension"))?;

        match extension {
            "safetensors" | "st" => Ok(WeightFormat::SafeTensors),
            "gguf" => Ok(WeightFormat::GGUF),
            "bin" => Ok(WeightFormat::Binary),
            "weights" | "model" => Ok(WeightFormat::Custom),
            _ => anyhow::bail!("Unknown weight format: {}", extension),
        }
    }
}

/// Weight tensor metadata
#[derive(Debug, Clone)]
pub struct TensorInfo {
    pub name: String,
    pub shape: Vec<usize>,
    pub dtype: String,
    pub offset: usize,
    pub size: usize,
}

/// Weight loader for loading model weights from files
pub struct WeightLoader {
    #[allow(dead_code)]
    format: WeightFormat,
    tensors: HashMap<String, Vec<f32>>,
    metadata: HashMap<String, TensorInfo>,
}

impl WeightLoader {
    /// Create a new weight loader
    pub fn new(format: WeightFormat) -> Self {
        Self {
            format,
            tensors: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Load weights from a file
    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();
        let format = WeightFormat::from_path(path)?;

        match format {
            WeightFormat::SafeTensors => self.load_safetensors(path),
            WeightFormat::GGUF => self.load_gguf(path),
            WeightFormat::Binary => self.load_binary(path),
            WeightFormat::Custom => self.load_custom(path),
        }
    }

    /// Load from SafeTensors format
    fn load_safetensors(&mut self, path: &Path) -> Result<()> {
        let file = File::open(path)
            .context("Failed to open safetensors file")?;

        let mmap = unsafe { Mmap::map(&file)? };
        let tensors = SafeTensors::deserialize(&mmap)?;

        for tensor_name in tensors.names() {
            let tensor_view = tensors.tensor(tensor_name)?;

            // Get tensor metadata
            let shape = tensor_view.shape().to_vec();
            let dtype = format!("{:?}", tensor_view.dtype());

            // Convert to f32
            let data = self.convert_to_f32(tensor_view.data(), &dtype)?;

            // Store tensor
            self.tensors.insert(tensor_name.to_string(), data);

            // Store metadata
            self.metadata.insert(
                tensor_name.to_string(),
                TensorInfo {
                    name: tensor_name.to_string(),
                    shape,
                    dtype,
                    offset: 0,
                    size: tensor_view.data().len(),
                },
            );
        }

        Ok(())
    }

    /// Load from GGUF format (llama.cpp)
    fn load_gguf(&mut self, path: &Path) -> Result<()> {
        // GGUF format parsing
        // This is a simplified version - full GGUF parsing is complex
        let mut file = File::open(path)?;
        let mut header = [0u8; 4];
        file.read_exact(&mut header)?;

        // Verify GGUF magic
        if &header != b"GGUF" {
            anyhow::bail!("Invalid GGUF file: wrong magic bytes");
        }

        // For now, return error suggesting conversion
        anyhow::bail!(
            "GGUF format not yet fully implemented. \
             Please convert to SafeTensors using: \
             python -m safetensors.convert --from gguf {}",
            path.display()
        )
    }

    /// Load from raw binary format
    fn load_binary(&mut self, path: &Path) -> Result<()> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        // Interpret as f32 array
        let float_count = buffer.len() / 4;
        let mut weights = Vec::with_capacity(float_count);

        for chunk in buffer.chunks_exact(4) {
            let value = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            weights.push(value);
        }

        self.tensors.insert("weights".to_string(), weights);

        Ok(())
    }

    /// Load from custom format (bincode)
    fn load_custom(&mut self, path: &Path) -> Result<()> {
        let file = File::open(path)?;
        let loaded: HashMap<String, Vec<f32>> = bincode::deserialize_from(file)?;

        self.tensors = loaded;

        Ok(())
    }

    /// Convert tensor data to f32
    fn convert_to_f32(&self, data: &[u8], dtype: &str) -> Result<Vec<f32>> {
        match dtype {
            "F32" | "Float32" => {
                let mut result = Vec::with_capacity(data.len() / 4);
                for chunk in data.chunks_exact(4) {
                    let value = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                    result.push(value);
                }
                Ok(result)
            }
            "F16" | "Float16" => {
                // Convert from f16 to f32
                let mut result = Vec::with_capacity(data.len() / 2);
                for chunk in data.chunks_exact(2) {
                    let f16_bits = u16::from_le_bytes([chunk[0], chunk[1]]);
                    let f32_value = half::f16::from_bits(f16_bits).to_f32();
                    result.push(f32_value);
                }
                Ok(result)
            }
            "BF16" => {
                // Convert from bf16 to f32
                let mut result = Vec::with_capacity(data.len() / 2);
                for chunk in data.chunks_exact(2) {
                    let bf16_bits = u16::from_le_bytes([chunk[0], chunk[1]]);
                    // BF16 to F32: just add zero bits in lower 16 bits
                    let f32_bits = (bf16_bits as u32) << 16;
                    let f32_value = f32::from_bits(f32_bits);
                    result.push(f32_value);
                }
                Ok(result)
            }
            _ => anyhow::bail!("Unsupported dtype: {}", dtype),
        }
    }

    /// Get a tensor by name
    pub fn get_tensor(&self, name: &str) -> Option<&Vec<f32>> {
        self.tensors.get(name)
    }

    /// Get tensor metadata
    pub fn get_metadata(&self, name: &str) -> Option<&TensorInfo> {
        self.metadata.get(name)
    }

    /// Get all tensor names
    pub fn tensor_names(&self) -> Vec<String> {
        self.tensors.keys().cloned().collect()
    }

    /// Get embedding weights if they exist
    pub fn get_embedding_weights(&self) -> Option<&Vec<f32>> {
        // Try common embedding layer names
        for name in &[
            "transformer.wte.weight",
            "model.embed_tokens.weight",
            "embeddings.word_embeddings.weight",
            "token_embedding.weight",
            "wte",
        ] {
            if let Some(tensor) = self.tensors.get(*name) {
                return Some(tensor);
            }
        }
        None
    }

    /// Save weights to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P, format: WeightFormat) -> Result<()> {
        let path = path.as_ref();

        match format {
            WeightFormat::Custom => {
                let file = File::create(path)?;
                bincode::serialize_into(file, &self.tensors)?;
                Ok(())
            }
            WeightFormat::Binary => {
                let mut file = File::create(path)?;
                if let Some(weights) = self.tensors.get("weights") {
                    for &value in weights {
                        std::io::Write::write_all(&mut file, &value.to_le_bytes())?;
                    }
                }
                Ok(())
            }
            _ => anyhow::bail!("Saving to {} format not yet implemented", format.name()),
        }
    }

    /// Total number of parameters
    pub fn total_params(&self) -> usize {
        self.tensors.values().map(|t| t.len()).sum()
    }

    /// Memory size in bytes
    pub fn memory_size(&self) -> usize {
        self.total_params() * std::mem::size_of::<f32>()
    }

    /// Insert tensor with name (for manual weight construction)
    pub fn insert_tensor(&mut self, name: String, weights: Vec<f32>) {
        self.tensors.insert(name, weights);
    }

    /// Load embedding weights from a flat array
    pub fn load_embedding_weights(&mut self, vocab_size: usize, embed_dim: usize, weights: Vec<f32>) -> Result<()> {
        if weights.len() != vocab_size * embed_dim {
            anyhow::bail!(
                "Weight size mismatch: expected {}, got {}",
                vocab_size * embed_dim,
                weights.len()
            );
        }

        self.tensors.insert("embeddings".to_string(), weights);

        Ok(())
    }
}

impl WeightFormat {
    fn name(&self) -> &'static str {
        match self {
            WeightFormat::SafeTensors => "SafeTensors",
            WeightFormat::GGUF => "GGUF",
            WeightFormat::Binary => "Binary",
            WeightFormat::Custom => "Custom",
        }
    }
}

// Add half dependency for f16 conversion
mod half {
    #[allow(non_camel_case_types)]
    pub struct f16 {
        bits: u16,
    }

    impl f16 {
        pub fn from_bits(bits: u16) -> Self {
            Self { bits }
        }

        pub fn to_f32(self) -> f32 {
            // Simple f16 to f32 conversion
            let sign = (self.bits >> 15) & 0x1;
            let exp = (self.bits >> 10) & 0x1F;
            let frac = self.bits & 0x3FF;

            if exp == 0 {
                // Denormalized or zero
                let value = (frac as f32) / 1024.0 / 16384.0;
                if sign == 1 { -value } else { value }
            } else if exp == 31 {
                // Inf or NaN
                if frac == 0 {
                    if sign == 1 { f32::NEG_INFINITY } else { f32::INFINITY }
                } else {
                    f32::NAN
                }
            } else {
                // Normalized
                let exp_f32 = (exp as i32) - 15 + 127;
                let frac_f32 = (frac as u32) << 13;
                let bits = ((sign as u32) << 31) | ((exp_f32 as u32) << 23) | frac_f32;
                f32::from_bits(bits)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weight_format_detection() {
        assert_eq!(
            WeightFormat::from_path(Path::new("model.safetensors")).unwrap(),
            WeightFormat::SafeTensors
        );
        assert_eq!(
            WeightFormat::from_path(Path::new("model.gguf")).unwrap(),
            WeightFormat::GGUF
        );
    }

    #[test]
    fn test_custom_format_roundtrip() {
        let mut loader = WeightLoader::new(WeightFormat::Custom);
        loader.tensors.insert("test".to_string(), vec![1.0, 2.0, 3.0]);

        let temp_path = "/tmp/test_weights.model";
        loader.save_to_file(temp_path, WeightFormat::Custom).unwrap();

        let mut loader2 = WeightLoader::new(WeightFormat::Custom);
        loader2.load_from_file(temp_path).unwrap();

        assert_eq!(loader2.get_tensor("test").unwrap(), &vec![1.0, 2.0, 3.0]);
    }
}
