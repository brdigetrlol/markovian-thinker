// Resonant Attractor module - Solver for resonant attractor dynamics
//
// Implements the resonant attractor manifold solver for phase field dynamics

use anyhow::Result;
use serde::{Deserialize, Serialize};
use super::phase_field::{PhaseField, Complex, ComplexField};

/// Resonant Attractor Solver
///
/// Solves for attracting states in the phase field dynamics,
/// implementing the resonant attractor manifold theory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResonantAttractorSolver {
    /// Resonance frequency
    pub omega: f32,
    /// Damping coefficient
    pub gamma: f32,
    /// Coupling strength
    pub coupling: f32,
    /// Attractor basin states
    pub attractors: Vec<ComplexField>,
}

impl ResonantAttractorSolver {
    /// Create new resonant attractor solver
    pub fn new() -> Result<Self> {
        Ok(Self {
            omega: 1.0,
            gamma: 0.1,
            coupling: 0.5,
            attractors: Vec::new(),
        })
    }

    /// Find attractors for given phase field
    pub fn find_attractors(&mut self, field: &PhaseField, num_attractors: usize) -> Result<Vec<ComplexField>> {
        let mut attractors = Vec::new();

        for i in 0..num_attractors {
            let mut test_field = field.clone();

            // Initialize with different phase offset
            let phase_offset = (i as f32 / num_attractors as f32) * std::f32::consts::TAU;
            test_field.field.rotate_phase(phase_offset);

            // Relax to attractor
            test_field.relax_to_equilibrium(1000, 1e-6)?;

            attractors.push(test_field.field);
        }

        self.attractors = attractors.clone();
        Ok(attractors)
    }

    /// Compute resonance strength between two fields
    pub fn resonance(&self, field1: &ComplexField, field2: &ComplexField) -> f32 {
        if field1.values.len() != field2.values.len() {
            return 0.0;
        }

        // Compute overlap (inner product)
        let overlap: f32 = field1.values.iter()
            .zip(field2.values.iter())
            .map(|(c1, c2)| {
                let product = *c1 * c2.conj();
                product.magnitude()
            })
            .sum();

        overlap / field1.dimension as f32
    }

    /// Evolve field toward nearest attractor
    pub fn evolve_toward_attractor(&self, field: &mut PhaseField, dt: f32) -> Result<()> {
        if self.attractors.is_empty() {
            return field.evolve(dt);
        }

        // Find nearest attractor
        let mut max_resonance = 0.0;
        let mut nearest_idx = 0;

        for (i, attractor) in self.attractors.iter().enumerate() {
            let res = self.resonance(&field.field, attractor);
            if res > max_resonance {
                max_resonance = res;
                nearest_idx = i;
            }
        }

        // Evolve toward nearest attractor
        let attractor = &self.attractors[nearest_idx];
        for (i, target) in attractor.values.iter().enumerate() {
            let current = field.field.values[i];
            let diff = *target + current * (-1.0);
            let update = diff * (self.coupling * dt);
            field.field.values[i] = current + update;
        }

        field.free_energy = field.compute_free_energy();
        Ok(())
    }

    /// Check if field is in attractor basin
    pub fn is_in_basin(&self, field: &ComplexField, threshold: f32) -> Option<usize> {
        for (i, attractor) in self.attractors.iter().enumerate() {
            if self.resonance(field, attractor) > threshold {
                return Some(i);
            }
        }
        None
    }

    /// Compute attractor landscape energy
    pub fn landscape_energy(&self, field: &ComplexField) -> f32 {
        // Potential energy based on distance from attractors
        let mut min_distance = f32::MAX;

        for attractor in &self.attractors {
            let distance = self.compute_distance(field, attractor);
            if distance < min_distance {
                min_distance = distance;
            }
        }

        min_distance
    }

    fn compute_distance(&self, field1: &ComplexField, field2: &ComplexField) -> f32 {
        if field1.values.len() != field2.values.len() {
            return f32::MAX;
        }

        field1.values.iter()
            .zip(field2.values.iter())
            .map(|(c1, c2)| {
                let diff_re = c1.re - c2.re;
                let diff_im = c1.im - c2.im;
                diff_re * diff_re + diff_im * diff_im
            })
            .sum::<f32>()
            .sqrt()
    }
}

impl Default for ResonantAttractorSolver {
    fn default() -> Self {
        Self::new().expect("Failed to create ResonantAttractorSolver")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resonant_attractor_creation() {
        let solver = ResonantAttractorSolver::new();
        assert!(solver.is_ok());
    }

    #[test]
    fn test_find_attractors() {
        let mut solver = ResonantAttractorSolver::new().unwrap();
        let field = PhaseField::new(8, 1.0);
        let attractors = solver.find_attractors(&field, 3);
        assert!(attractors.is_ok());
        assert_eq!(attractors.unwrap().len(), 3);
    }

    #[test]
    fn test_resonance_computation() {
        let solver = ResonantAttractorSolver::new().unwrap();
        let field1 = ComplexField::new(8);
        let field2 = ComplexField::new(8);
        let resonance = solver.resonance(&field1, &field2);
        assert!(resonance >= 0.0);
    }
}
