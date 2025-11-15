// Lattice module for TIC architecture
//
// Implements multi-resolution lattice hierarchy:
// - E8: 8D, analytical/logical core (System 2)
// - Leech: 24D, creative/analogical reasoning (System 1.5)
// - HCP: 64D, associative/intuitive processing (System 1)
// - Hypercubic: 1024D, sensory manifold buffer

pub mod e8;
pub mod leech;
pub mod hcp;
pub mod hypercubic;

pub use e8::E8Lattice;
pub use leech::LeechLattice;
pub use hcp::HCPLattice;
pub use hypercubic::HypercubicLattice;

use serde::{Deserialize, Serialize};

/// Lattice types for the multi-resolution hierarchy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LatticeType {
    E8,
    Leech24,
    HCP64,
    Sensory1024,
}

/// A point in a lattice
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LatticePoint {
    pub coordinates: Vec<f64>,
    pub lattice_type: LatticeType,
}

impl LatticePoint {
    pub fn new(coordinates: Vec<f64>, lattice_type: LatticeType) -> Self {
        Self {
            coordinates,
            lattice_type,
        }
    }

    pub fn dimension(&self) -> usize {
        self.coordinates.len()
    }

    pub fn distance_to(&self, other: &LatticePoint) -> f64 {
        assert_eq!(self.lattice_type, other.lattice_type);
        assert_eq!(self.dimension(), other.dimension());

        self.coordinates
            .iter()
            .zip(other.coordinates.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt()
    }
}

/// Trait for lattice quantization
pub trait Lattice {
    /// Quantize a point to the nearest lattice point
    fn quantize(&self, point: &[f64]) -> Vec<i64>;

    /// Get the dimension of this lattice
    fn dimension(&self) -> usize;

    /// Get the kissing number (number of nearest neighbors)
    fn kissing_number(&self) -> usize;

    /// Compute distance between two lattice points
    fn distance(&self, a: &[i64], b: &[i64]) -> f64 {
        assert_eq!(a.len(), b.len());
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| ((x - y) as f64).powi(2))
            .sum::<f64>()
            .sqrt()
    }

    /// Get all nearest neighbors of a lattice point
    fn nearest_neighbors(&self, point: &[i64]) -> Vec<Vec<i64>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lattice_point_creation() {
        let point = LatticePoint::new(vec![1.0, 2.0, 3.0], LatticeType::E8);
        assert_eq!(point.dimension(), 3);
        assert_eq!(point.lattice_type, LatticeType::E8);
    }

    #[test]
    fn test_distance_computation() {
        let p1 = LatticePoint::new(vec![0.0, 0.0, 0.0], LatticeType::E8);
        let p2 = LatticePoint::new(vec![3.0, 4.0, 0.0], LatticeType::E8);
        assert!((p1.distance_to(&p2) - 5.0).abs() < 1e-10);
    }
}
