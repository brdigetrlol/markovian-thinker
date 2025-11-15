// Markovian Thinker: Crystallographic Lattices
// Simplified lattice structures inspired by Icarus TIC

use serde::{Deserialize, Serialize};

/// Lattice type for concept representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LatticeType {
    /// E8 lattice: 8-dimensional, optimal for mathematical concepts
    E8,

    /// Leech lattice: 24-dimensional, optimal sphere packing
    Leech,

    /// Hexagonal close packing: Variable dimension
    HCP(usize),

    /// Hypercubic lattice: Simple integer grid
    Hypercubic(usize),
}

impl LatticeType {
    /// Get dimension of lattice
    pub fn dimension(&self) -> usize {
        match self {
            Self::E8 => 8,
            Self::Leech => 24,
            Self::HCP(dim) => *dim,
            Self::Hypercubic(dim) => *dim,
        }
    }

    /// Get name
    pub fn name(&self) -> String {
        match self {
            Self::E8 => "E8".to_string(),
            Self::Leech => "Leech".to_string(),
            Self::HCP(dim) => format!("HCP-{}", dim),
            Self::Hypercubic(dim) => format!("Cubic-{}", dim),
        }
    }
}

/// Point in a lattice
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LatticePoint {
    /// Coordinates
    pub coords: Vec<f32>,

    /// Lattice type
    pub lattice_type: LatticeType,

    /// Norm (distance from origin)
    pub norm: f32,
}

impl LatticePoint {
    /// Create new lattice point
    pub fn new(coords: Vec<f32>, lattice_type: LatticeType) -> Self {
        let norm = Self::compute_norm(&coords);
        Self {
            coords,
            lattice_type,
            norm,
        }
    }

    /// Compute Euclidean norm
    fn compute_norm(coords: &[f32]) -> f32 {
        coords.iter().map(|x| x * x).sum::<f32>().sqrt()
    }

    /// Compute distance to another point
    pub fn distance(&self, other: &LatticePoint) -> f32 {
        assert_eq!(
            self.coords.len(),
            other.coords.len(),
            "Points must have same dimension"
        );

        self.coords
            .iter()
            .zip(&other.coords)
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f32>()
            .sqrt()
    }

    /// Add two lattice points
    pub fn add(&self, other: &LatticePoint) -> LatticePoint {
        assert_eq!(
            self.coords.len(),
            other.coords.len(),
            "Points must have same dimension"
        );

        let coords: Vec<f32> = self
            .coords
            .iter()
            .zip(&other.coords)
            .map(|(a, b)| a + b)
            .collect();

        LatticePoint::new(coords, self.lattice_type)
    }

    /// Subtract two lattice points
    pub fn subtract(&self, other: &LatticePoint) -> LatticePoint {
        assert_eq!(
            self.coords.len(),
            other.coords.len(),
            "Points must have same dimension"
        );

        let coords: Vec<f32> = self
            .coords
            .iter()
            .zip(&other.coords)
            .map(|(a, b)| a - b)
            .collect();

        LatticePoint::new(coords, self.lattice_type)
    }

    /// Scalar multiplication
    pub fn scale(&self, scalar: f32) -> LatticePoint {
        let coords: Vec<f32> = self.coords.iter().map(|x| x * scalar).collect();
        LatticePoint::new(coords, self.lattice_type)
    }

    /// Dimension
    pub fn dimension(&self) -> usize {
        self.coords.len()
    }
}

/// Lattice generator trait
pub trait LatticeGenerator: Send + Sync {
    /// Generate lattice point from index
    fn generate(&self, index: usize) -> LatticePoint;

    /// Find closest lattice point (CVP - Closest Vector Problem)
    fn closest_vector(&self, point: &[f32]) -> LatticePoint;

    /// Get lattice type
    fn lattice_type(&self) -> LatticeType;

    /// Get basis vectors
    fn basis(&self) -> Vec<Vec<f32>>;
}

/// E8 lattice generator (simplified)
pub struct E8Generator;

impl E8Generator {
    pub fn new() -> Self {
        Self
    }

    /// E8 root system (240 vectors)
    /// Simplified: returns subset for demonstration
    fn root_vectors() -> Vec<Vec<f32>> {
        let mut roots = Vec::new();

        // Type 1: All permutations of (±1, ±1, 0, 0, 0, 0, 0, 0)
        // Simplified: just a few examples
        roots.push(vec![1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        roots.push(vec![1.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        roots.push(vec![1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0]);

        // Type 2: (±1/2)^8 with even number of minus signs
        roots.push(vec![0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5]);
        roots.push(vec![-0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5]);

        roots
    }
}

impl LatticeGenerator for E8Generator {
    fn generate(&self, index: usize) -> LatticePoint {
        let roots = Self::root_vectors();
        let root = &roots[index % roots.len()];
        LatticePoint::new(root.clone(), LatticeType::E8)
    }

    fn closest_vector(&self, point: &[f32]) -> LatticePoint {
        assert_eq!(point.len(), 8, "E8 requires 8-dimensional vectors");

        // Simplified CVP: quantize each coordinate
        let coords: Vec<f32> = point.iter().map(|x| x.round()).collect();

        LatticePoint::new(coords, LatticeType::E8)
    }

    fn lattice_type(&self) -> LatticeType {
        LatticeType::E8
    }

    fn basis(&self) -> Vec<Vec<f32>> {
        // E8 standard basis
        (0..8)
            .map(|i| {
                let mut v = vec![0.0; 8];
                v[i] = 1.0;
                v
            })
            .collect()
    }
}

impl Default for E8Generator {
    fn default() -> Self {
        Self::new()
    }
}

/// Leech lattice generator (simplified)
pub struct LeechGenerator;

impl LeechGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl LatticeGenerator for LeechGenerator {
    fn generate(&self, index: usize) -> LatticePoint {
        // Simplified: generate point from index
        let mut coords = vec![0.0; 24];
        let mut n = index;

        for i in 0..24 {
            coords[i] = (n % 3) as f32 - 1.0; // -1, 0, 1
            n /= 3;
        }

        LatticePoint::new(coords, LatticeType::Leech)
    }

    fn closest_vector(&self, point: &[f32]) -> LatticePoint {
        assert_eq!(point.len(), 24, "Leech requires 24-dimensional vectors");

        // Simplified CVP: quantize to nearest integer
        let coords: Vec<f32> = point.iter().map(|x| x.round()).collect();

        LatticePoint::new(coords, LatticeType::Leech)
    }

    fn lattice_type(&self) -> LatticeType {
        LatticeType::Leech
    }

    fn basis(&self) -> Vec<Vec<f32>> {
        // Leech standard basis
        (0..24)
            .map(|i| {
                let mut v = vec![0.0; 24];
                v[i] = 1.0;
                v
            })
            .collect()
    }
}

impl Default for LeechGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Hexagonal close packing generator
pub struct HCPGenerator {
    dimension: usize,
}

impl HCPGenerator {
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

impl LatticeGenerator for HCPGenerator {
    fn generate(&self, index: usize) -> LatticePoint {
        let mut coords = vec![0.0; self.dimension];

        // Simple HCP: alternating layers
        let layer = index / self.dimension;
        let pos = index % self.dimension;

        coords[pos] = layer as f32;
        if layer % 2 == 1 && pos < self.dimension - 1 {
            coords[pos + 1] = 0.5;
        }

        LatticePoint::new(coords, LatticeType::HCP(self.dimension))
    }

    fn closest_vector(&self, point: &[f32]) -> LatticePoint {
        assert_eq!(
            point.len(),
            self.dimension,
            "Point dimension must match lattice"
        );

        // HCP CVP: project onto hexagonal layers
        let coords: Vec<f32> = point
            .iter()
            .enumerate()
            .map(|(i, &x)| {
                if i % 2 == 0 {
                    x.round()
                } else {
                    (x * 2.0).round() / 2.0 // Quantize to 0.5 grid
                }
            })
            .collect();

        LatticePoint::new(coords, LatticeType::HCP(self.dimension))
    }

    fn lattice_type(&self) -> LatticeType {
        LatticeType::HCP(self.dimension)
    }

    fn basis(&self) -> Vec<Vec<f32>> {
        (0..self.dimension)
            .map(|i| {
                let mut v = vec![0.0; self.dimension];
                v[i] = 1.0;
                v
            })
            .collect()
    }
}

/// Hypercubic (integer grid) generator
pub struct HypercubicGenerator {
    dimension: usize,
}

impl HypercubicGenerator {
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

impl LatticeGenerator for HypercubicGenerator {
    fn generate(&self, index: usize) -> LatticePoint {
        let mut coords = vec![0.0; self.dimension];
        let mut n = index;

        for i in 0..self.dimension {
            coords[i] = (n % 10) as f32; // 0-9 range
            n /= 10;
        }

        LatticePoint::new(coords, LatticeType::Hypercubic(self.dimension))
    }

    fn closest_vector(&self, point: &[f32]) -> LatticePoint {
        assert_eq!(
            point.len(),
            self.dimension,
            "Point dimension must match lattice"
        );

        // Simple rounding to nearest integer
        let coords: Vec<f32> = point.iter().map(|x| x.round()).collect();

        LatticePoint::new(coords, LatticeType::Hypercubic(self.dimension))
    }

    fn lattice_type(&self) -> LatticeType {
        LatticeType::Hypercubic(self.dimension)
    }

    fn basis(&self) -> Vec<Vec<f32>> {
        (0..self.dimension)
            .map(|i| {
                let mut v = vec![0.0; self.dimension];
                v[i] = 1.0;
                v
            })
            .collect()
    }
}

/// Create lattice generator for given type
pub fn create_generator(lattice_type: LatticeType) -> Box<dyn LatticeGenerator> {
    match lattice_type {
        LatticeType::E8 => Box::new(E8Generator::new()),
        LatticeType::Leech => Box::new(LeechGenerator::new()),
        LatticeType::HCP(dim) => Box::new(HCPGenerator::new(dim)),
        LatticeType::Hypercubic(dim) => Box::new(HypercubicGenerator::new(dim)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lattice_type_dimension() {
        assert_eq!(LatticeType::E8.dimension(), 8);
        assert_eq!(LatticeType::Leech.dimension(), 24);
        assert_eq!(LatticeType::HCP(16).dimension(), 16);
        assert_eq!(LatticeType::Hypercubic(32).dimension(), 32);
    }

    #[test]
    fn test_lattice_point_creation() {
        let point = LatticePoint::new(vec![1.0, 2.0, 3.0], LatticeType::Hypercubic(3));

        assert_eq!(point.dimension(), 3);
        assert_eq!(point.coords, vec![1.0, 2.0, 3.0]);
        assert!(point.norm > 0.0);
    }

    #[test]
    fn test_lattice_point_distance() {
        let p1 = LatticePoint::new(vec![0.0, 0.0], LatticeType::Hypercubic(2));
        let p2 = LatticePoint::new(vec![3.0, 4.0], LatticeType::Hypercubic(2));

        let dist = p1.distance(&p2);
        assert!((dist - 5.0).abs() < 0.001); // 3-4-5 triangle
    }

    #[test]
    fn test_lattice_point_addition() {
        let p1 = LatticePoint::new(vec![1.0, 2.0], LatticeType::Hypercubic(2));
        let p2 = LatticePoint::new(vec![3.0, 4.0], LatticeType::Hypercubic(2));

        let p3 = p1.add(&p2);
        assert_eq!(p3.coords, vec![4.0, 6.0]);
    }

    #[test]
    fn test_lattice_point_subtraction() {
        let p1 = LatticePoint::new(vec![5.0, 7.0], LatticeType::Hypercubic(2));
        let p2 = LatticePoint::new(vec![2.0, 3.0], LatticeType::Hypercubic(2));

        let p3 = p1.subtract(&p2);
        assert_eq!(p3.coords, vec![3.0, 4.0]);
    }

    #[test]
    fn test_lattice_point_scaling() {
        let p1 = LatticePoint::new(vec![1.0, 2.0], LatticeType::Hypercubic(2));
        let p2 = p1.scale(3.0);

        assert_eq!(p2.coords, vec![3.0, 6.0]);
    }

    #[test]
    fn test_e8_generator() {
        let gen = E8Generator::new();

        let point = gen.generate(0);
        assert_eq!(point.dimension(), 8);

        let basis = gen.basis();
        assert_eq!(basis.len(), 8);
    }

    #[test]
    fn test_e8_closest_vector() {
        let gen = E8Generator::new();

        let point = vec![1.4, 2.6, 3.1, 4.9, 5.2, 6.7, 7.3, 8.8];
        let closest = gen.closest_vector(&point);

        assert_eq!(closest.dimension(), 8);
        // Should round to nearest integers
        assert_eq!(closest.coords, vec![1.0, 3.0, 3.0, 5.0, 5.0, 7.0, 7.0, 9.0]);
    }

    #[test]
    fn test_leech_generator() {
        let gen = LeechGenerator::new();

        let point = gen.generate(0);
        assert_eq!(point.dimension(), 24);

        let basis = gen.basis();
        assert_eq!(basis.len(), 24);
    }

    #[test]
    fn test_hcp_generator() {
        let gen = HCPGenerator::new(8);

        let point = gen.generate(0);
        assert_eq!(point.dimension(), 8);
        assert_eq!(gen.lattice_type(), LatticeType::HCP(8));
    }

    #[test]
    fn test_hypercubic_generator() {
        let gen = HypercubicGenerator::new(4);

        let point = gen.generate(0);
        assert_eq!(point.dimension(), 4);

        let test_point = vec![1.4, 2.6, 3.1, 4.9];
        let closest = gen.closest_vector(&test_point);

        assert_eq!(closest.coords, vec![1.0, 3.0, 3.0, 5.0]);
    }

    #[test]
    fn test_create_generator() {
        let gen_e8 = create_generator(LatticeType::E8);
        assert_eq!(gen_e8.lattice_type(), LatticeType::E8);

        let gen_leech = create_generator(LatticeType::Leech);
        assert_eq!(gen_leech.lattice_type(), LatticeType::Leech);

        let gen_hcp = create_generator(LatticeType::HCP(16));
        assert_eq!(gen_hcp.lattice_type(), LatticeType::HCP(16));

        let gen_cubic = create_generator(LatticeType::Hypercubic(32));
        assert_eq!(gen_cubic.lattice_type(), LatticeType::Hypercubic(32));
    }
}
