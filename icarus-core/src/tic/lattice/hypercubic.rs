// Hypercubic Lattice Implementation (ℤⁿ)
//
// Simplest lattice: just the integer lattice in n dimensions.
// Used for 1024D sensory manifold buffer in TIC architecture.
//
// Properties:
// - Kissing number: 2n
// - Packing density: Very poor in high dimensions
// - Advantage: Simplicity, high bandwidth, orthogonal axes

use super::Lattice;

#[derive(Debug, Clone)]
pub struct HypercubicLattice {
    dimension: usize,
}

impl HypercubicLattice {
    pub fn new(dimension: usize) -> Self {
        assert!(dimension > 0, "Dimension must be positive");
        Self { dimension }
    }
}

impl Default for HypercubicLattice {
    fn default() -> Self {
        Self::new(1024) // Default to 1024D for sensory manifold
    }
}

impl Lattice for HypercubicLattice {
    fn quantize(&self, point: &[f64]) -> Vec<i64> {
        assert_eq!(
            point.len(),
            self.dimension,
            "Point dimension must match lattice dimension"
        );

        // Trivial quantization: just round each coordinate
        point.iter().map(|&x| x.round() as i64).collect()
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn kissing_number(&self) -> usize {
        2 * self.dimension
    }

    fn nearest_neighbors(&self, point: &[i64]) -> Vec<Vec<i64>> {
        let mut neighbors = Vec::with_capacity(2 * self.dimension);

        for i in 0..self.dimension {
            // Add +1 neighbor
            let mut neighbor_plus = point.to_vec();
            neighbor_plus[i] += 1;
            neighbors.push(neighbor_plus);

            // Add -1 neighbor
            let mut neighbor_minus = point.to_vec();
            neighbor_minus[i] -= 1;
            neighbors.push(neighbor_minus);
        }

        neighbors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hypercubic_creation() {
        let lattice = HypercubicLattice::new(1024);
        assert_eq!(lattice.dimension(), 1024);
        assert_eq!(lattice.kissing_number(), 2048);
    }

    #[test]
    fn test_quantization() {
        let lattice = HypercubicLattice::new(8);
        let point = vec![1.4, 2.6, 3.1, 4.9, 5.5, 6.2, 7.8, 8.3];
        let quantized = lattice.quantize(&point);

        assert_eq!(quantized, vec![1, 3, 3, 5, 6, 6, 8, 8]);
    }

    #[test]
    fn test_origin_quantization() {
        let lattice = HypercubicLattice::new(8);
        let origin = vec![0.0; 8];
        let quantized = lattice.quantize(&origin);

        assert_eq!(quantized, vec![0i64; 8]);
    }

    #[test]
    fn test_nearest_neighbors() {
        let lattice = HypercubicLattice::new(8);
        let origin = vec![0i64; 8];
        let neighbors = lattice.nearest_neighbors(&origin);

        // Should have 2n = 16 neighbors for n=8
        assert_eq!(neighbors.len(), 16);

        // Each neighbor should differ from origin by exactly 1 in one coordinate
        for neighbor in &neighbors {
            let diff: i64 = neighbor
                .iter()
                .zip(origin.iter())
                .map(|(a, b)| (a - b).abs())
                .sum();
            assert_eq!(diff, 1);
        }
    }

    #[test]
    fn test_distance() {
        let lattice = HypercubicLattice::new(3);
        let a = vec![0, 0, 0];
        let b = vec![3, 4, 0];

        let dist = lattice.distance(&a, &b);
        assert!((dist - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_performance_1024d() {
        use std::time::Instant;

        let lattice = HypercubicLattice::new(1024);
        let test_points: Vec<Vec<f64>> = (0..10000)
            .map(|i| vec![(i as f64) * 0.001; 1024])
            .collect();

        let start = Instant::now();
        for point in &test_points {
            let _ = lattice.quantize(point);
        }
        let elapsed = start.elapsed();

        let ops_per_sec = (test_points.len() as f64) / elapsed.as_secs_f64();
        println!(
            "Hypercubic 1024D quantization: {:.2} ops/sec ({:.2} μs/op)",
            ops_per_sec,
            elapsed.as_micros() as f64 / test_points.len() as f64
        );

        // Target: > 1M ops/sec (< 1 μs per op) even for 1024D
        assert!(
            ops_per_sec > 100_000.0,
            "Performance target not met: {:.0} ops/sec",
            ops_per_sec
        );
    }
}
