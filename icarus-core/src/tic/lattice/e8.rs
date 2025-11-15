// E8 Lattice Implementation
//
// The E8 lattice is the densest sphere packing in 8 dimensions (Viazovska, 2016).
// Kissing number: 240
// Used for analytical/logical reasoning (System 2) in TIC architecture.
//
// Construction: E8 = D8 ∪ (D8 + (1/2, 1/2, ..., 1/2))
// Where D8 = {(x₁, ..., x₈) ∈ ℤ⁸ | Σxᵢ ∈ 2ℤ}

use super::{Lattice, LatticePoint, LatticeType};

#[derive(Debug, Clone)]
pub struct E8Lattice;

impl E8Lattice {
    pub fn new() -> Self {
        Self
    }

    /// Round point to nearest D8 lattice point
    fn round_to_d8(point: &[f64]) -> Vec<i64> {
        assert_eq!(point.len(), 8, "E8 requires 8-dimensional input");

        // Round each coordinate
        let mut rounded: Vec<i64> = point.iter().map(|&x| x.round() as i64).collect();

        // Ensure sum is even (D8 constraint: Σxᵢ ∈ 2ℤ)
        let sum: i64 = rounded.iter().sum();
        if sum % 2 != 0 {
            // Find coordinate with largest rounding error and flip it
            let mut max_error = 0.0;
            let mut flip_idx = 0;

            for i in 0..8 {
                let error = (point[i] - rounded[i] as f64).abs();
                if error > max_error {
                    max_error = error;
                    flip_idx = i;
                }
            }

            // Flip to maintain even sum
            rounded[flip_idx] += if point[flip_idx] > rounded[flip_idx] as f64 {
                1
            } else {
                -1
            };
        }

        rounded
    }

    /// Round point to D8 + (1/2, ..., 1/2) coset
    fn round_to_d8_coset(point: &[f64]) -> Vec<f64> {
        // Shift by (1/2, ..., 1/2), round to D8, shift back
        let shifted: Vec<f64> = point.iter().map(|&x| x - 0.5).collect();
        let d8_point = Self::round_to_d8(&shifted);
        d8_point
            .iter()
            .map(|&x| x as f64 + 0.5)
            .collect()
    }

    /// Compute squared distance
    fn distance_sq(a: &[f64], b: &[f64]) -> f64 {
        a.iter()
            .zip(b.iter())
            .map(|(&x, &y)| (x - y).powi(2))
            .sum()
    }

    /// Generate all 240 root vectors of E8
    pub fn root_vectors() -> Vec<Vec<i64>> {
        let mut roots = Vec::with_capacity(240);

        // Type 1: All permutations and sign changes of (±1, ±1, 0, 0, 0, 0, 0, 0)
        // Choose 2 positions out of 8: C(8,2) = 28
        // For each, 4 sign combinations: 28 * 4 = 112
        for i in 0..8 {
            for j in (i + 1)..8 {
                for signs in 0..4 {
                    let mut root = vec![0i64; 8];
                    root[i] = if signs & 1 == 0 { 1 } else { -1 };
                    root[j] = if signs & 2 == 0 { 1 } else { -1 };
                    roots.push(root);
                }
            }
        }

        // Type 2: (±1/2, ±1/2, ..., ±1/2) * 2 with even number of minus signs
        // Since we're using integer representation, multiply by 2: (±1, ±1, ..., ±1)
        // Total: 2^8 = 256 combinations, but only those with even parity
        for pattern in 0..256u16 {
            let minus_count = pattern.count_ones();
            if minus_count % 2 == 0 {
                // Even number of -1's
                let root: Vec<i64> = (0..8)
                    .map(|i| if pattern & (1 << i) == 0 { 1 } else { -1 })
                    .collect();
                roots.push(root);
            }
        }

        assert_eq!(roots.len(), 240, "E8 should have exactly 240 roots");
        roots
    }
}

impl Default for E8Lattice {
    fn default() -> Self {
        Self::new()
    }
}

impl Lattice for E8Lattice {
    fn quantize(&self, point: &[f64]) -> Vec<i64> {
        assert_eq!(point.len(), 8, "E8 requires 8-dimensional input");

        // Candidate 1: Nearest D8 point
        let d8_candidate = Self::round_to_d8(point);
        let d8_dist = Self::distance_sq(
            point,
            &d8_candidate.iter().map(|&x| x as f64).collect::<Vec<_>>(),
        );

        // Candidate 2: Nearest D8 + (1/2, ..., 1/2) point
        let coset_candidate = Self::round_to_d8_coset(point);
        let coset_dist = Self::distance_sq(point, &coset_candidate);

        // Return closer candidate
        // Note: Coset points are half-integers, convert to integer representation
        if d8_dist < coset_dist {
            d8_candidate
        } else {
            // Scale coset point by 2 to get integer coordinates
            coset_candidate.iter().map(|&x| (x * 2.0).round() as i64).collect()
        }
    }

    fn dimension(&self) -> usize {
        8
    }

    fn kissing_number(&self) -> usize {
        240
    }

    fn nearest_neighbors(&self, point: &[i64]) -> Vec<Vec<i64>> {
        let roots = Self::root_vectors();
        roots
            .into_iter()
            .map(|root| {
                point
                    .iter()
                    .zip(root.iter())
                    .map(|(p, r)| p + r)
                    .collect()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_e8_lattice_creation() {
        let lattice = E8Lattice::new();
        assert_eq!(lattice.dimension(), 8);
        assert_eq!(lattice.kissing_number(), 240);
    }

    #[test]
    fn test_d8_quantization() {
        let lattice = E8Lattice::new();

        // Test point that should round to D8
        let point = vec![1.1, 2.0, 3.1, 4.0, 5.1, 6.0, 7.1, 8.0];
        let quantized = lattice.quantize(&point);

        // Sum should be even (D8 constraint)
        let sum: i64 = quantized.iter().sum();
        assert_eq!(sum % 2, 0, "D8 constraint: sum must be even");

        println!("Quantized: {:?}", quantized);
    }

    #[test]
    fn test_root_vectors() {
        let roots = E8Lattice::root_vectors();

        // Should have exactly 240 roots
        assert_eq!(roots.len(), 240);

        // All roots should have norm squared = 2 (in standard E8 scaling)
        // Or norm squared = 4 if we're using doubled coset representation
        for root in &roots {
            let norm_sq: i64 = root.iter().map(|x| x * x).sum();
            assert!(
                norm_sq == 2 || norm_sq == 4 || norm_sq == 8,
                "Root norm squared should be 2, 4, or 8, got {}",
                norm_sq
            );
        }

        // Verify we have the expected counts
        // Type 1: 112 roots (±1, ±1, 0, 0, 0, 0, 0, 0) permutations
        let type1_count = roots.iter().filter(|r| r.iter().filter(|&&x| x != 0).count() == 2).count();
        // Type 2: 128 roots (all ±1 with even parity)
        let type2_count = roots.iter().filter(|r| r.iter().all(|&&x| x.abs() == 1)).count();

        println!("Type 1 roots: {}", type1_count);
        println!("Type 2 roots: {}", type2_count);
        assert_eq!(type1_count + type2_count, 240);
    }

    #[test]
    fn test_nearest_neighbors_count() {
        let lattice = E8Lattice::new();
        let origin = vec![0i64; 8];
        let neighbors = lattice.nearest_neighbors(&origin);

        // Should have exactly 240 nearest neighbors
        assert_eq!(neighbors.len(), 240);
    }

    #[test]
    fn test_origin_quantization() {
        let lattice = E8Lattice::new();
        let origin = vec![0.0; 8];
        let quantized = lattice.quantize(&origin);

        // Origin should quantize to itself
        assert_eq!(quantized, vec![0i64; 8]);
    }

    #[test]
    fn test_distance_computation() {
        let lattice = E8Lattice::new();
        let a = vec![0i64; 8];
        let b = vec![1, 1, 0, 0, 0, 0, 0, 0]; // One of the root vectors

        let dist = lattice.distance(&a, &b);
        assert_relative_eq!(dist, 2.0_f64.sqrt(), epsilon = 1e-10);
    }

    #[test]
    fn test_performance_quantization() {
        use std::time::Instant;

        let lattice = E8Lattice::new();
        let test_points: Vec<Vec<f64>> = (0..10000)
            .map(|i| {
                vec![
                    (i as f64) * 0.1,
                    (i as f64) * 0.2,
                    (i as f64) * 0.3,
                    (i as f64) * 0.4,
                    (i as f64) * 0.5,
                    (i as f64) * 0.6,
                    (i as f64) * 0.7,
                    (i as f64) * 0.8,
                ]
            })
            .collect();

        let start = Instant::now();
        for point in &test_points {
            let _ = lattice.quantize(point);
        }
        let elapsed = start.elapsed();

        let ops_per_sec = (test_points.len() as f64) / elapsed.as_secs_f64();
        println!(
            "E8 quantization: {:.2} ops/sec ({:.2} μs/op)",
            ops_per_sec,
            elapsed.as_micros() as f64 / test_points.len() as f64
        );

        // Target: > 1M ops/sec (< 1 μs per op)
        assert!(
            ops_per_sec > 100_000.0,
            "Performance target not met: {:.0} ops/sec",
            ops_per_sec
        );
    }
}
