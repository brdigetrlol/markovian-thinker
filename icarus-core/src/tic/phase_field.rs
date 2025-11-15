// Phase Field module - Complex-valued fields on TIC substrate
//
// Implements phase field dynamics for resonant attractor manifold

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

/// Complex number representation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Complex {
    pub re: f32,
    pub im: f32,
}

impl Complex {
    pub fn new(re: f32, im: f32) -> Self {
        Self { re, im }
    }

    pub fn from_polar(r: f32, theta: f32) -> Self {
        Self {
            re: r * theta.cos(),
            im: r * theta.sin(),
        }
    }

    pub fn magnitude(&self) -> f32 {
        (self.re * self.re + self.im * self.im).sqrt()
    }

    pub fn phase(&self) -> f32 {
        self.im.atan2(self.re)
    }

    pub fn conj(&self) -> Self {
        Self {
            re: self.re,
            im: -self.im,
        }
    }
}

impl std::ops::Add for Complex {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            re: self.re + other.re,
            im: self.im + other.im,
        }
    }
}

impl std::ops::Mul for Complex {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Self {
            re: self.re * other.re - self.im * other.im,
            im: self.re * other.im + self.im * other.re,
        }
    }
}

impl std::ops::Mul<f32> for Complex {
    type Output = Self;
    fn mul(self, scalar: f32) -> Self {
        Self {
            re: self.re * scalar,
            im: self.im * scalar,
        }
    }
}

/// Complex-valued field over lattice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexField {
    /// Field values at each lattice site
    pub values: Vec<Complex>,
    /// Dimensionality of underlying lattice
    pub dimension: usize,
}

impl ComplexField {
    /// Create new complex field with given dimension
    pub fn new(dimension: usize) -> Self {
        Self {
            values: vec![Complex::new(0.0, 0.0); dimension],
            dimension,
        }
    }

    /// Initialize with random phases
    pub fn initialize_random(&mut self) {
        use std::f32::consts::TAU;
        for i in 0..self.dimension {
            let phase = (i as f32 / self.dimension as f32) * TAU;
            self.values[i] = Complex::from_polar(1.0, phase);
        }
    }

    /// Compute field energy (sum of magnitudes)
    pub fn energy(&self) -> f32 {
        self.values.iter().map(|c| c.magnitude()).sum()
    }

    /// Apply phase rotation
    pub fn rotate_phase(&mut self, angle: f32) {
        let rotation = Complex::from_polar(1.0, angle);
        for value in &mut self.values {
            *value = *value * rotation;
        }
    }
}

/// Phase field with dynamics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseField {
    /// Current field configuration
    pub field: ComplexField,
    /// Free energy of current configuration
    pub free_energy: f32,
    /// Temperature parameter
    pub temperature: f32,
}

impl PhaseField {
    /// Create new phase field
    pub fn new(dimension: usize, temperature: f32) -> Self {
        let field = ComplexField::new(dimension);
        let free_energy = field.energy();
        Self {
            field,
            free_energy,
            temperature,
        }
    }

    /// Evolve phase field dynamics
    pub fn evolve(&mut self, dt: f32) -> Result<()> {
        // Simple gradient descent on free energy
        let gradient = self.compute_gradient();

        for (i, grad) in gradient.iter().enumerate() {
            let update = *grad * (-dt);
            self.field.values[i] = self.field.values[i] + update;
        }

        self.free_energy = self.compute_free_energy();
        Ok(())
    }

    /// Compute gradient of free energy
    fn compute_gradient(&self) -> Vec<Complex> {
        // Simplified gradient computation
        self.field.values
            .iter()
            .map(|c| {
                let mag = c.magnitude();
                if mag > 1e-6 {
                    *c * (-(mag - 1.0) / mag)
                } else {
                    Complex::new(0.0, 0.0)
                }
            })
            .collect()
    }

    /// Compute free energy
    pub fn compute_free_energy(&self) -> f32 {
        // Simple free energy: deviation from unit circle
        self.field.values
            .iter()
            .map(|c| {
                let mag = c.magnitude();
                (mag - 1.0).powi(2)
            })
            .sum()
    }

    /// Find equilibrium configuration
    pub fn relax_to_equilibrium(&mut self, max_steps: usize, tolerance: f32) -> Result<()> {
        for _ in 0..max_steps {
            let old_energy = self.free_energy;
            self.evolve(0.01)?;

            if (self.free_energy - old_energy).abs() < tolerance {
                break;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complex_arithmetic() {
        let c1 = Complex::new(1.0, 2.0);
        let c2 = Complex::new(3.0, 4.0);
        let sum = c1 + c2;
        assert_eq!(sum.re, 4.0);
        assert_eq!(sum.im, 6.0);
    }

    #[test]
    fn test_phase_field_creation() {
        let field = PhaseField::new(8, 1.0);
        assert_eq!(field.field.dimension, 8);
    }

    #[test]
    fn test_phase_field_evolution() {
        let mut field = PhaseField::new(8, 1.0);
        field.field.initialize_random();
        let initial_energy = field.free_energy;
        field.evolve(0.01).unwrap();
        // Energy should change after evolution
        assert_ne!(field.free_energy, initial_energy);
    }
}
