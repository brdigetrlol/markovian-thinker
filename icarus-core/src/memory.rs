// Hierarchical Memory System
// 4-level memory hierarchy: Working, Short-term, Long-term, Episodic

use crate::config::MemoryConfig;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Memory levels in the hierarchy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryLevel {
    Working,
    ShortTerm,
    LongTerm,
    Episodic,
}

/// A memory item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: Uuid,
    pub data: Vec<u8>,
    pub embedding: Option<Vec<f32>>,
    pub importance: f32,
    pub access_count: usize,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
}

impl Memory {
    pub fn new(data: Vec<u8>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            data,
            embedding: None,
            importance: 0.5,
            access_count: 0,
            created_at: now,
            last_accessed: now,
        }
    }

    pub fn access(&mut self) {
        self.access_count += 1;
        self.last_accessed = Utc::now();
    }
}

/// Working Memory - Limited capacity, immediate access
pub struct WorkingMemory {
    capacity: usize,
    items: VecDeque<Memory>,
}

impl WorkingMemory {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            items: VecDeque::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, memory: Memory) -> Option<Memory> {
        if self.items.len() >= self.capacity {
            // Evict oldest item
            self.items.pop_front()
        } else {
            self.items.push_back(memory);
            None
        }
    }

    pub fn get(&mut self, id: &Uuid) -> Option<&mut Memory> {
        self.items.iter_mut().find(|m| &m.id == id)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Memory> {
        self.items.iter()
    }
}

/// Short-term Memory - Larger capacity, recent items
pub struct ShortTermMemory {
    capacity: usize,
    items: VecDeque<Memory>,
}

impl ShortTermMemory {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            items: VecDeque::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, memory: Memory) -> Option<Memory> {
        if self.items.len() >= self.capacity {
            self.items.pop_front()
        } else {
            self.items.push_back(memory);
            None
        }
    }

    pub fn get(&mut self, id: &Uuid) -> Option<&mut Memory> {
        self.items.iter_mut().find(|m| &m.id == id)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}

/// Long-term Memory - Unlimited/large capacity, semantic storage
pub struct LongTermMemory {
    items: Vec<Memory>,
}

impl LongTermMemory {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
        }
    }

    pub fn store(&mut self, memory: Memory) {
        self.items.push(memory);
    }

    pub fn get(&mut self, id: &Uuid) -> Option<&mut Memory> {
        self.items.iter_mut().find(|m| &m.id == id)
    }

    pub fn search(&self, query_embedding: &[f32], top_k: usize) -> Vec<&Memory> {
        // Simple cosine similarity search
        // TODO: Use proper vector database for production
        let mut scored: Vec<(usize, f32)> = self.items
            .iter()
            .enumerate()
            .filter_map(|(idx, m)| {
                m.embedding.as_ref().map(|emb| {
                    let similarity = cosine_similarity(query_embedding, emb);
                    (idx, similarity)
                })
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        scored.iter()
            .take(top_k)
            .filter_map(|(idx, _)| self.items.get(*idx))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}

impl Default for LongTermMemory {
    fn default() -> Self {
        Self::new()
    }
}

/// Episodic Memory - Stores episodes/experiences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub id: Uuid,
    pub memories: Vec<Uuid>,
    pub reward: f32,
    pub created_at: DateTime<Utc>,
}

pub struct EpisodicMemory {
    capacity: usize,
    episodes: VecDeque<Episode>,
}

impl EpisodicMemory {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            episodes: VecDeque::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, episode: Episode) {
        if self.episodes.len() >= self.capacity {
            self.episodes.pop_front();
        }
        self.episodes.push_back(episode);
    }

    pub fn get(&self, id: &Uuid) -> Option<&Episode> {
        self.episodes.iter().find(|e| &e.id == id)
    }

    pub fn len(&self) -> usize {
        self.episodes.len()
    }
}

/// Memory Hierarchy - Coordinates all memory levels
pub struct MemoryHierarchy {
    config: MemoryConfig,
    working: WorkingMemory,
    short_term: ShortTermMemory,
    long_term: LongTermMemory,
    episodic: EpisodicMemory,
    last_consolidation: DateTime<Utc>,
}

impl MemoryHierarchy {
    pub fn new(config: &MemoryConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
            working: WorkingMemory::new(config.working_capacity),
            short_term: ShortTermMemory::new(config.short_term_capacity),
            long_term: LongTermMemory::new(),
            episodic: EpisodicMemory::new(config.episodic_capacity),
            last_consolidation: Utc::now(),
        })
    }

    /// Store a memory in working memory
    pub fn store(&mut self, memory: Memory) -> Option<Memory> {
        self.working.push(memory)
    }

    /// Retrieve a memory by ID from any level
    pub fn retrieve(&mut self, id: &Uuid) -> Option<&mut Memory> {
        // Search in order: working -> short-term -> long-term
        if let Some(mem) = self.working.get(id) {
            mem.access();
            return Some(mem);
        }

        if let Some(mem) = self.short_term.get(id) {
            mem.access();
            return Some(mem);
        }

        if let Some(mem) = self.long_term.get(id) {
            mem.access();
            return Some(mem);
        }

        None
    }

    /// Consolidate memories between levels
    pub async fn consolidate(&mut self) -> Result<()> {
        let now = Utc::now();
        let elapsed = (now - self.last_consolidation).num_seconds() as u64;

        if elapsed < self.config.consolidation_interval_secs {
            return Ok(());
        }

        tracing::debug!("Consolidating memories across hierarchy");

        // Move important items from working to short-term
        let to_promote: Vec<Memory> = self.working.iter()
            .filter(|m| m.importance > 0.7 || m.access_count > 3)
            .cloned()
            .collect();

        for memory in to_promote {
            if let Some(evicted) = self.short_term.push(memory) {
                // Evicted from short-term -> consider for long-term
                if evicted.importance > 0.8 {
                    self.long_term.store(evicted);
                }
            }
        }

        self.last_consolidation = now;

        tracing::debug!("Memory consolidation complete");
        tracing::debug!("  Working: {}/{}", self.working.len(), self.config.working_capacity);
        tracing::debug!("  Short-term: {}/{}", self.short_term.len(), self.config.short_term_capacity);
        tracing::debug!("  Long-term: {}", self.long_term.len());
        tracing::debug!("  Episodic: {}/{}", self.episodic.len(), self.config.episodic_capacity);

        Ok(())
    }

    /// Store an episode
    pub fn store_episode(&mut self, episode: Episode) {
        self.episodic.push(episode);
    }

    /// Semantic search in long-term memory
    pub fn search(&self, query_embedding: &[f32], top_k: usize) -> Vec<&Memory> {
        self.long_term.search(query_embedding, top_k)
    }
}

/// Cosine similarity helper
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_working_memory() {
        let mut wm = WorkingMemory::new(3);

        wm.push(Memory::new(vec![1]));
        wm.push(Memory::new(vec![2]));
        wm.push(Memory::new(vec![3]));

        assert_eq!(wm.len(), 3);

        // Should evict oldest
        let evicted = wm.push(Memory::new(vec![4]));
        assert!(evicted.is_some());
        assert_eq!(evicted.unwrap().data, vec![1]);
    }

    #[test]
    fn test_memory_hierarchy() {
        let config = MemoryConfig::default();
        let mut hierarchy = MemoryHierarchy::new(&config).unwrap();

        let mem = Memory::new(vec![1, 2, 3]);
        let id = mem.id;

        hierarchy.store(mem);

        let retrieved = hierarchy.retrieve(&id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().access_count, 1);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![1.0, 0.0, 0.0];
        let d = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&c, &d) - 0.0).abs() < 0.001);
    }
}
