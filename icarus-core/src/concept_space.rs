// Markovian Thinker: Concept Space with Crystallographic Lattices
// High-level API for concept representation, similarity, and composition

use crate::lattice::{
    create_generator, LatticeGenerator, LatticePoint, LatticeType,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A concept in the concept space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concept {
    /// Unique identifier for the concept
    pub id: String,

    /// Human-readable label
    pub label: String,

    /// Lattice point representing this concept
    pub point: LatticePoint,

    /// Optional metadata
    pub metadata: HashMap<String, String>,
}

impl Concept {
    /// Create a new concept
    pub fn new(id: String, label: String, point: LatticePoint) -> Self {
        Self {
            id,
            label,
            point,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Compute similarity to another concept (cosine similarity)
    pub fn similarity(&self, other: &Concept) -> f32 {
        // Cosine similarity: dot(a, b) / (norm(a) * norm(b))
        let dot: f32 = self
            .point
            .coords
            .iter()
            .zip(&other.point.coords)
            .map(|(a, b)| a * b)
            .sum();

        let norm_product = self.point.norm * other.point.norm;
        if norm_product == 0.0 {
            return 0.0;
        }

        dot / norm_product
    }

    /// Compute Euclidean distance to another concept
    pub fn distance(&self, other: &Concept) -> f32 {
        self.point.distance(&other.point)
    }
}

/// Configuration for concept space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptSpaceConfig {
    /// Type of lattice to use
    pub lattice_type: LatticeType,

    /// Maximum number of concepts to store
    pub max_concepts: usize,

    /// Similarity threshold for deduplication
    pub similarity_threshold: f32,
}

impl Default for ConceptSpaceConfig {
    fn default() -> Self {
        Self {
            lattice_type: LatticeType::E8,
            max_concepts: 10000,
            similarity_threshold: 0.999, // 99.9% similar = duplicate (nearly exact)
        }
    }
}

/// Concept space with lattice-based representation
pub struct ConceptSpace {
    config: ConceptSpaceConfig,
    generator: Box<dyn LatticeGenerator>,
    concepts: HashMap<String, Concept>,
}

impl ConceptSpace {
    /// Create a new concept space
    pub fn new(config: ConceptSpaceConfig) -> Self {
        let generator = create_generator(config.lattice_type);

        Self {
            config,
            generator,
            concepts: HashMap::new(),
        }
    }

    /// Create with default E8 lattice
    pub fn new_e8() -> Self {
        Self::new(ConceptSpaceConfig::default())
    }

    /// Create with Leech lattice (24D)
    pub fn new_leech() -> Self {
        let mut config = ConceptSpaceConfig::default();
        config.lattice_type = LatticeType::Leech;
        Self::new(config)
    }

    /// Create with HCP lattice
    pub fn new_hcp(dimension: usize) -> Self {
        let mut config = ConceptSpaceConfig::default();
        config.lattice_type = LatticeType::HCP(dimension);
        Self::new(config)
    }

    /// Crystallize an embedding into a lattice point
    pub fn crystallize(&self, embedding: &[f32]) -> LatticePoint {
        self.generator.closest_vector(embedding)
    }

    /// Add a concept to the space
    pub fn add_concept(&mut self, concept: Concept) -> Result<(), String> {
        // Check capacity
        if self.concepts.len() >= self.config.max_concepts {
            return Err("Concept space is full".to_string());
        }

        // Check for duplicates
        if let Some(existing) = self.find_most_similar(&concept.point, 1).first() {
            let similarity = concept.similarity(existing);
            if similarity >= self.config.similarity_threshold {
                return Err(format!(
                    "Concept too similar to existing concept '{}' (similarity: {:.2})",
                    existing.label, similarity
                ));
            }
        }

        self.concepts.insert(concept.id.clone(), concept);
        Ok(())
    }

    /// Get a concept by ID
    pub fn get_concept(&self, id: &str) -> Option<&Concept> {
        self.concepts.get(id)
    }

    /// Remove a concept
    pub fn remove_concept(&mut self, id: &str) -> Option<Concept> {
        self.concepts.remove(id)
    }

    /// Find most similar concepts to a lattice point
    pub fn find_most_similar(&self, point: &LatticePoint, k: usize) -> Vec<&Concept> {
        let mut concepts: Vec<&Concept> = self.concepts.values().collect();

        // Sort by distance (ascending)
        concepts.sort_by(|a, b| {
            let dist_a = point.distance(&a.point);
            let dist_b = point.distance(&b.point);
            dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Return top k
        concepts.into_iter().take(k).collect()
    }

    /// Find concepts similar to an embedding
    pub fn find_similar(&self, embedding: &[f32], k: usize) -> Vec<&Concept> {
        let point = self.crystallize(embedding);
        self.find_most_similar(&point, k)
    }

    /// Compose two concepts (vector addition)
    pub fn compose(&self, id1: &str, id2: &str) -> Option<LatticePoint> {
        let c1 = self.concepts.get(id1)?;
        let c2 = self.concepts.get(id2)?;

        Some(c1.point.add(&c2.point))
    }

    /// Find the difference between two concepts
    pub fn difference(&self, id1: &str, id2: &str) -> Option<LatticePoint> {
        let c1 = self.concepts.get(id1)?;
        let c2 = self.concepts.get(id2)?;

        Some(c1.point.subtract(&c2.point))
    }

    /// Scale a concept
    pub fn scale_concept(&self, id: &str, scalar: f32) -> Option<LatticePoint> {
        let concept = self.concepts.get(id)?;
        Some(concept.point.scale(scalar))
    }

    /// Perform concept analogy: a is to b as c is to ?
    /// Returns: b - a + c
    pub fn analogy(&self, a_id: &str, b_id: &str, c_id: &str) -> Option<LatticePoint> {
        let a = self.concepts.get(a_id)?;
        let b = self.concepts.get(b_id)?;
        let c = self.concepts.get(c_id)?;

        // b - a + c
        let diff = b.point.subtract(&a.point);
        Some(c.point.add(&diff))
    }

    /// Get all concepts in a radius around a point
    pub fn concepts_in_radius(&self, point: &LatticePoint, radius: f32) -> Vec<&Concept> {
        self.concepts
            .values()
            .filter(|c| c.point.distance(point) <= radius)
            .collect()
    }

    /// Get concept space statistics
    pub fn statistics(&self) -> ConceptSpaceStatistics {
        let total_concepts = self.concepts.len();

        if total_concepts == 0 {
            return ConceptSpaceStatistics {
                total_concepts: 0,
                lattice_type: self.config.lattice_type,
                dimension: self.generator.lattice_type().dimension(),
                avg_norm: 0.0,
                min_norm: 0.0,
                max_norm: 0.0,
            };
        }

        let norms: Vec<f32> = self.concepts.values().map(|c| c.point.norm).collect();

        let avg_norm = norms.iter().sum::<f32>() / total_concepts as f32;
        let min_norm = norms
            .iter()
            .cloned()
            .fold(f32::INFINITY, f32::min);
        let max_norm = norms
            .iter()
            .cloned()
            .fold(f32::NEG_INFINITY, f32::max);

        ConceptSpaceStatistics {
            total_concepts,
            lattice_type: self.config.lattice_type,
            dimension: self.generator.lattice_type().dimension(),
            avg_norm,
            min_norm,
            max_norm,
        }
    }

    /// Clear all concepts
    pub fn clear(&mut self) {
        self.concepts.clear();
    }

    /// Get number of concepts
    pub fn len(&self) -> usize {
        self.concepts.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.concepts.is_empty()
    }
}

/// Statistics about the concept space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptSpaceStatistics {
    pub total_concepts: usize,
    pub lattice_type: LatticeType,
    pub dimension: usize,
    pub avg_norm: f32,
    pub min_norm: f32,
    pub max_norm: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concept_creation() {
        let point = LatticePoint::new(vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], LatticeType::E8);
        let concept = Concept::new("test".to_string(), "Test Concept".to_string(), point);

        assert_eq!(concept.id, "test");
        assert_eq!(concept.label, "Test Concept");
        assert_eq!(concept.point.dimension(), 8);
    }

    #[test]
    fn test_concept_similarity() {
        let p1 = LatticePoint::new(vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], LatticeType::E8);
        let p2 = LatticePoint::new(vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], LatticeType::E8);
        let p3 = LatticePoint::new(vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], LatticeType::E8);

        let c1 = Concept::new("c1".to_string(), "C1".to_string(), p1);
        let c2 = Concept::new("c2".to_string(), "C2".to_string(), p2);
        let c3 = Concept::new("c3".to_string(), "C3".to_string(), p3);

        // Same concepts should have similarity 1.0
        assert!((c1.similarity(&c2) - 1.0).abs() < 0.001);

        // Orthogonal concepts should have similarity 0.0
        assert!(c1.similarity(&c3).abs() < 0.001);
    }

    #[test]
    fn test_concept_space_creation() {
        let space = ConceptSpace::new_e8();
        assert_eq!(space.len(), 0);
        assert!(space.is_empty());
    }

    #[test]
    fn test_crystallization() {
        let space = ConceptSpace::new_e8();

        // Crystallize a floating-point embedding
        let embedding = vec![1.4, 2.6, 3.1, 4.9, 5.2, 6.7, 7.3, 8.8];
        let point = space.crystallize(&embedding);

        assert_eq!(point.dimension(), 8);
        // Should round to nearest integers
        assert_eq!(point.coords, vec![1.0, 3.0, 3.0, 5.0, 5.0, 7.0, 7.0, 9.0]);
    }

    #[test]
    fn test_add_and_get_concept() {
        let mut space = ConceptSpace::new_e8();

        let point = LatticePoint::new(vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], LatticeType::E8);
        let concept = Concept::new("test".to_string(), "Test".to_string(), point);

        assert!(space.add_concept(concept).is_ok());
        assert_eq!(space.len(), 1);

        let retrieved = space.get_concept("test");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().label, "Test");
    }

    #[test]
    fn test_remove_concept() {
        let mut space = ConceptSpace::new_e8();

        let point = LatticePoint::new(vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], LatticeType::E8);
        let concept = Concept::new("test".to_string(), "Test".to_string(), point);

        space.add_concept(concept).unwrap();
        assert_eq!(space.len(), 1);

        let removed = space.remove_concept("test");
        assert!(removed.is_some());
        assert_eq!(space.len(), 0);
    }

    #[test]
    fn test_duplicate_detection() {
        let mut space = ConceptSpace::new_e8();

        let point1 = LatticePoint::new(vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], LatticeType::E8);
        let concept1 = Concept::new("test1".to_string(), "Test1".to_string(), point1);

        // Almost identical point (very high similarity)
        let point2 = LatticePoint::new(vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], LatticeType::E8);
        let concept2 = Concept::new("test2".to_string(), "Test2".to_string(), point2);

        assert!(space.add_concept(concept1).is_ok());
        let result = space.add_concept(concept2);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("too similar"));
    }

    #[test]
    fn test_find_similar() {
        let mut space = ConceptSpace::new_e8();

        // Add three orthogonal concepts
        let p1 = LatticePoint::new(vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], LatticeType::E8);
        let c1 = Concept::new("c1".to_string(), "C1".to_string(), p1);

        let p2 = LatticePoint::new(vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], LatticeType::E8);
        let c2 = Concept::new("c2".to_string(), "C2".to_string(), p2);

        let p3 = LatticePoint::new(vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0], LatticeType::E8);
        let c3 = Concept::new("c3".to_string(), "C3".to_string(), p3);

        space.add_concept(c1).unwrap();
        space.add_concept(c2).unwrap();
        space.add_concept(c3).unwrap();

        // Find similar to [1.5, 0.5, 0, 0, 0, 0, 0, 0]
        let embedding = vec![1.5, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let similar = space.find_similar(&embedding, 2);

        assert_eq!(similar.len(), 2);
        // c1 and c2 should be closest
        assert!(similar[0].id == "c1" || similar[0].id == "c2");
    }

    #[test]
    fn test_concept_composition() {
        let mut space = ConceptSpace::new_e8();

        let p1 = LatticePoint::new(vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], LatticeType::E8);
        let c1 = Concept::new("c1".to_string(), "C1".to_string(), p1);

        let p2 = LatticePoint::new(vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], LatticeType::E8);
        let c2 = Concept::new("c2".to_string(), "C2".to_string(), p2);

        space.add_concept(c1).unwrap();
        space.add_concept(c2).unwrap();

        // Compose c1 + c2
        let result = space.compose("c1", "c2");
        assert!(result.is_some());

        let composed = result.unwrap();
        assert_eq!(composed.coords, vec![1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_concept_difference() {
        let mut space = ConceptSpace::new_e8();

        let p1 = LatticePoint::new(vec![3.0, 4.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0], LatticeType::E8);
        let c1 = Concept::new("c1".to_string(), "C1".to_string(), p1);

        let p2 = LatticePoint::new(vec![1.0, 2.0, 3.0, 0.0, 0.0, 0.0, 0.0, 0.0], LatticeType::E8);
        let c2 = Concept::new("c2".to_string(), "C2".to_string(), p2);

        space.add_concept(c1).unwrap();
        space.add_concept(c2).unwrap();

        // c1 - c2
        let result = space.difference("c1", "c2");
        assert!(result.is_some());

        let diff = result.unwrap();
        assert_eq!(diff.coords, vec![2.0, 2.0, -2.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_concept_analogy() {
        let mut space = ConceptSpace::new_e8();

        // king - man + woman = queen analogy
        // Use different dimensions to avoid similarity rejection
        let king = LatticePoint::new(vec![1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0], LatticeType::E8);
        let man = LatticePoint::new(vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0], LatticeType::E8);
        let woman = LatticePoint::new(vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], LatticeType::E8);

        space.add_concept(Concept::new("king".to_string(), "King".to_string(), king)).unwrap();
        space.add_concept(Concept::new("man".to_string(), "Man".to_string(), man)).unwrap();
        space.add_concept(Concept::new("woman".to_string(), "Woman".to_string(), woman)).unwrap();

        // king - man + woman = [1,0,1] - [0,0,1] + [0,1,0] = [1,1,0]
        let result = space.analogy("man", "king", "woman");
        assert!(result.is_some());

        let queen = result.unwrap();
        assert_eq!(queen.coords, vec![1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_concepts_in_radius() {
        let mut space = ConceptSpace::new_e8();

        let p1 = LatticePoint::new(vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], LatticeType::E8);
        let c1 = Concept::new("c1".to_string(), "C1".to_string(), p1.clone());

        let p2 = LatticePoint::new(vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], LatticeType::E8);
        let c2 = Concept::new("c2".to_string(), "C2".to_string(), p2);

        let p3 = LatticePoint::new(vec![0.0, 0.0, 10.0, 0.0, 0.0, 0.0, 0.0, 0.0], LatticeType::E8);
        let c3 = Concept::new("c3".to_string(), "C3".to_string(), p3);

        space.add_concept(c1).unwrap();
        space.add_concept(c2).unwrap();
        space.add_concept(c3).unwrap();

        // Find concepts within radius 2.0 of origin
        let concepts = space.concepts_in_radius(&p1, 2.0);

        // Should find c1 and c2, but not c3
        assert_eq!(concepts.len(), 2);
    }

    #[test]
    fn test_statistics() {
        let mut space = ConceptSpace::new_e8();

        let p1 = LatticePoint::new(vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], LatticeType::E8);
        let c1 = Concept::new("c1".to_string(), "C1".to_string(), p1);

        let p2 = LatticePoint::new(vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], LatticeType::E8);
        let c2 = Concept::new("c2".to_string(), "C2".to_string(), p2);

        space.add_concept(c1).unwrap();
        space.add_concept(c2).unwrap();

        let stats = space.statistics();
        assert_eq!(stats.total_concepts, 2);
        assert_eq!(stats.dimension, 8);
        assert_eq!(stats.lattice_type, LatticeType::E8);
        assert!(stats.avg_norm > 0.0);
    }

    #[test]
    fn test_leech_lattice_space() {
        let space = ConceptSpace::new_leech();
        let stats = space.statistics();
        assert_eq!(stats.dimension, 24);
        assert_eq!(stats.lattice_type, LatticeType::Leech);
    }

    #[test]
    fn test_hcp_lattice_space() {
        let space = ConceptSpace::new_hcp(16);
        let stats = space.statistics();
        assert_eq!(stats.dimension, 16);
        assert_eq!(stats.lattice_type, LatticeType::HCP(16));
    }
}
