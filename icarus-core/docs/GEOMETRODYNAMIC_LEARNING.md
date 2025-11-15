# Geometrodynamic Learning for TIC Architecture

## Overview

This document describes the learning algorithm for the TIC cognitive substrate - a unified approach based on minimizing Variational Free Energy through Natural Gradient Descent with topological constraints.

## 1. Variational Free Energy

### 1.1 Definition

The cognitive system seeks to minimize:

```
F[φ, μ] = E_q(s|φ)[log q(s|φ) - log p(s, o|μ)]
```

Where:
- φ: Internal model parameters (phase field configuration)
- μ: Generative model parameters (world model)
- s: Hidden states (inferred causes)
- o: Observations (sensory input)
- q(s|φ): Recognition model (inference)
- p(s,o|μ): Generative model (world model)

### 1.2 Decomposition

Free Energy can be decomposed:

```
F = ⟨Energy⟩ - ⟨Entropy⟩
  = Accuracy - Complexity
```

**Accuracy term:** How well does model explain observations?
```
Accuracy = E_q[log p(o|s, μ)]
```

**Complexity term:** KL divergence from prior
```
Complexity = KL[q(s|φ) || p(s|μ)]
```

The system automatically balances:
- **Fitting data** (high accuracy)
- **Simplicity** (low complexity / Occam's razor)

### 1.3 For Phase Fields

In the TIC context:

```
F[z, J] = ∫ [(ε/2)|∇z|² + V(|z|) - z*·o + λ_complexity·||J||²] d^n r
```

Where:
- z: Phase field state
- J: Coupling matrix (learned parameters)
- o: Sensory observations (projected onto lattice)
- ε, λ: Hyperparameters

## 2. Natural Gradient Descent

### 2.1 Why Natural Gradient?

Ordinary gradient descent:
```
Δθ = -η ∇_θ F
```

Problem: Parameter space has non-Euclidean geometry. Equal steps in parameter space ≠ equal steps in distribution space.

Natural gradient uses Fisher Information Metric:
```
Δθ = -η F^(-1) ∇_θ F
```

Where F is the Fisher Information Matrix:
```
F_ij = E[∂log p/∂θ_i · ∂log p/∂θ_j]
```

### 2.2 Local Approximation

For computational tractability, approximate F locally:

```
F ≈ G = metric tensor of the manifold
```

In phase field: G is induced by the L² metric on function space.

Practically:
```
G_ij ≈ ⟨∂z/∂θ_i, ∂z/∂θ_j⟩ + regularization
```

### 2.3 Implementation

**Diagonal approximation (fast):**
```rust
fn natural_gradient_diagonal(
    gradient: &Array1<f32>,
    fisher_diagonal: &Array1<f32>,
    learning_rate: f32,
) -> Array1<f32> {
    let preconditioned = gradient / (fisher_diagonal + 1e-8);
    -learning_rate * preconditioned
}
```

**Full Fisher (accurate but expensive):**
```rust
fn natural_gradient_full(
    gradient: &Array1<f32>,
    fisher_matrix: &Array2<f32>,
    learning_rate: f32,
) -> Array1<f32> {
    // Solve: F * Δθ = -∇F
    let delta_theta = fisher_matrix.solve(gradient).unwrap();
    learning_rate * delta_theta
}
```

## 3. Topological Stabilization

### 3.1 Motivation

During learning, we must preserve topological invariants of the concept space. Otherwise, concepts can "merge" or "split" inappropriately.

### 3.2 Persistent Homology

Compute topological features of the phase field configuration:

```
H_k(X) = k-dimensional homology groups
```

For k=0: Connected components (number of distinct concepts)
For k=1: Loops (circular relationships)
For k=2: Voids (hierarchical structure)

### 3.3 Constraint Enforcement

**Soft constraint (penalty):**
```
F_total = F_energy + λ_topo · ||H_k(z) - H_k(z_target)||²
```

**Hard constraint (projection):**
After each gradient step, project back onto topologically-valid manifold.

```rust
fn topological_projection(
    z_proposed: &PhaseField,
    homology_target: &HomologySignature,
) -> PhaseField {
    // 1. Compute persistent homology of proposed state
    let h_proposed = compute_persistent_homology(z_proposed);

    // 2. Check if it matches target
    if homology_matches(&h_proposed, homology_target) {
        return z_proposed.clone();
    }

    // 3. Iteratively adjust to restore topology
    let mut z_corrected = z_proposed.clone();
    for _ in 0..MAX_TOPO_ITERATIONS {
        let violation = homology_violation(&z_corrected, homology_target);
        if violation < TOLERANCE {
            break;
        }

        // Gradient descent on topological violation
        let topo_gradient = compute_topo_gradient(&z_corrected, homology_target);
        z_corrected = z_corrected - 0.1 * topo_gradient;
    }

    z_corrected
}
```

## 4. Ricci Flow Regularization

### 4.1 Concept

Ricci flow smooths the geometry of the manifold:

```
∂g/∂τ = -2 Ric(g)
```

Where:
- g: Metric tensor
- Ric: Ricci curvature tensor
- τ: "Flow time" (not physical time)

Effect: Regions of high curvature are smoothed.

### 4.2 Discrete Ricci Flow for Lattices

For graph/lattice G = (V, E):

```
∂w_e/∂τ = -K_v
```

Where:
- w_e: Edge weight
- K_v: Discrete curvature at vertex v

Discrete curvature (Ollivier-Ricci):
```
K_v = 1 - W_1(μ_v, μ_neighbors) / d_v
```

W_1 = Wasserstein distance

### 4.3 Integration with Learning

Alternate between:
1. **Free Energy minimization** (content learning)
2. **Ricci flow** (geometry smoothing)

```rust
fn geometrodynamic_learning_step(
    field: &mut PhaseField,
    observations: &Observations,
    parameters: &LearningParams,
) -> f32 {
    // 1. Compute Free Energy gradient
    let grad_f = compute_free_energy_gradient(field, observations);

    // 2. Fisher information (local approximation)
    let fisher_diag = compute_fisher_diagonal(field);

    // 3. Natural gradient
    let natural_grad = grad_f / (fisher_diag + 1e-8);

    // 4. Gradient descent step
    let mut field_proposed = field - parameters.eta * natural_grad;

    // 5. Topological projection
    field_proposed = topological_projection(&field_proposed, &parameters.topology_target);

    // 6. Ricci flow smoothing
    if parameters.enable_ricci_flow {
        field_proposed = ricci_flow_step(&field_proposed, parameters.ricci_tau);
    }

    // 7. Update field
    *field = field_proposed;

    // Return Free Energy for monitoring
    compute_free_energy(field, observations)
}
```

## 5. Neuromodulated Learning Rates

### 5.1 Dopamine-Modulated η

Learning rate adapts based on surprise:

```
η_effective = η_base · DA_phasic / (DA_tonic + k)
```

Where:
```
DA_phasic = Reward - Reward_expected
DA_tonic = ⟨Reward⟩_time
```

**Intuition:**
- Unexpected success (DA_phasic > 0, DA_tonic low) → Learn fast
- Expected success (DA_phasic ≈ 0, DA_tonic high) → Learn slowly
- Unexpected failure (DA_phasic < 0) → Potentially unlearn

### 5.2 Acetylcholine-Modulated Precision

ACh modulates which gradients to trust:

```
∇F_weighted = ACh · ∇F_sensory + (1 - ACh) · ∇F_prior
```

High ACh → Trust sensory gradients (learn from environment)
Low ACh → Trust prior (rely on existing knowledge)

### 5.3 Norepinephrine-Modulated Exploration

NE controls exploration via temperature:

```
z_next = z_current - η·∇F + σ·ξ

where σ = σ_base · (1 + NE)
```

High NE → More noise → More exploration

## 6. Meta-Learning (Learning to Learn)

### 6.1 Homeostatic Meta-Objective

The system learns the learning rate itself:

```
F_total = F_world + λ_h · F_homeostatic
```

Where:
```
F_homeostatic = w_vram · VRAM_usage²
                + w_error · Error_rate²
                + w_temp · Temperature²
```

**Meta-parameter λ_h learned via Q-learning:**
```
Q(λ_h, a) ← Q(λ_h, a) + α[R + γ·max_a' Q(λ_h', a') - Q(λ_h, a)]
```

Where reward R = -F_total.

### 6.2 Adaptive Hyperparameters

All hyperparameters become learnable:

| Parameter | Initial | Adapted via |
|-----------|---------|-------------|
| η (learning rate) | 0.01 | Dopamine modulation + meta-learning |
| ε (gradient energy) | 1.0 | Acetylcholine modulation |
| λ (potential depth) | 1.0 | Dopamine modulation |
| T (temperature) | 0.1 | Norepinephrine modulation |

### 6.3 Implementation

```rust
struct AdaptiveHyperparameters {
    eta: f32,
    epsilon: f32,
    lambda: f32,
    temperature: f32,
    lambda_h: f32,  // Meta-parameter
}

impl AdaptiveHyperparameters {
    fn update_with_neuromodulators(
        &mut self,
        da_tonic: f32,
        da_phasic: f32,
        ach: f32,
        ne: f32,
    ) {
        // Learning rate (DA-modulated)
        self.eta = self.eta * (da_phasic / (da_tonic + 0.1));

        // Gradient energy (ACh-modulated)
        self.epsilon = 1.0 * (1.0 + ach);

        // Potential depth (DA-modulated)
        self.lambda = 1.0 * (1.0 + da_phasic);

        // Temperature (NE-modulated)
        self.temperature = 0.1 + ne;
    }

    fn meta_learn(&mut self, free_energy: f32, homeostatic_cost: f32) {
        // Q-learning update for λ_h
        let total_cost = free_energy + self.lambda_h * homeostatic_cost;
        let reward = -total_cost;

        // Simplified Q-update (full implementation would maintain Q-table)
        let q_target = reward + 0.99 * self.estimate_future_q();
        let q_current = self.lambda_h;  // Simplified
        self.lambda_h += 0.01 * (q_target - q_current);

        // Clamp to reasonable range
        self.lambda_h = self.lambda_h.clamp(0.0, 10.0);
    }

    fn estimate_future_q(&self) -> f32 {
        // Placeholder: estimate expected future reward
        // Full implementation would use learned Q-function
        0.0
    }
}
```

## 7. Crystallographic Batch Learning

### 7.1 Coset-Based Gradient Aggregation

For a batch of observations {o₁, ..., oₙ}, group by symmetry:

1. **Partition into cosets:**
```
{o₁, ..., oₙ} = ⋃_i Coset_i
```

Where Coset_i = {g·o_rep | g ∈ G_i} for some representative o_rep.

2. **Compute gradient on representatives:**
```
∇F_rep = Σ_{representatives} ∇F(o_rep)
```

3. **Apply symmetry to get full batch gradient:**
```
∇F_full = Symmetry_Expand(∇F_rep)
```

**Speedup:** O(N/|G|) instead of O(N)

### 7.2 Implementation

```rust
fn cbr_batch_learning(
    field: &mut PhaseField,
    observations: &[Observation],
    symmetry_group: &SymmetryGroup,
    learning_rate: f32,
) -> f32 {
    // 1. Group observations by cosets
    let cosets = group_by_cosets(observations, symmetry_group);

    // 2. Compute gradient for representatives only
    let mut total_gradient = Array2::zeros(field.shape());

    for coset in &cosets {
        let representative = &coset.representative;
        let grad_rep = compute_gradient(field, representative);

        // 3. Symmetry expansion
        for symmetry in symmetry_group.elements() {
            let grad_expanded = symmetry.apply(&grad_rep);
            total_gradient += &grad_expanded;
        }
    }

    // 4. Natural gradient + update
    let fisher = estimate_fisher_diagonal(field);
    let natural_grad = &total_gradient / (&fisher + 1e-8);

    *field = field - learning_rate * natural_grad;

    // Return Free Energy
    observations.iter()
        .map(|o| compute_free_energy(field, o))
        .sum::<f32>() / observations.len() as f32
}
```

## 8. Continual Learning (Avoiding Catastrophic Forgetting)

### 8.1 Elastic Weight Consolidation (EWC)

Protect important parameters:

```
F_total = F_current + Σ_i (λ_EWC/2) · F_i · (θ_i - θ_i*)²
```

Where:
- θ*: Parameters from previous task
- F_i: Fisher information (importance)

### 8.2 Progressive Neural Networks

For TIC: Add new lattice regions for new concepts, keep old regions frozen.

```rust
struct ContinualLearningState {
    frozen_regions: Vec<LatticeRegion>,
    active_regions: Vec<LatticeRegion>,
    ewc_fisher: HashMap<Parameter, f32>,
}

impl ContinualLearningState {
    fn learn_new_task(&mut self, task_data: &TaskData) {
        // 1. Identify which lattice regions to use
        let new_region = self.allocate_fresh_region();

        // 2. Compute Fisher for current parameters (before learning)
        self.ewc_fisher = compute_fisher_information(&self.active_regions);

        // 3. Learn on new task with EWC penalty
        for batch in task_data {
            let grad = compute_gradient_with_ewc_penalty(batch, &self.ewc_fisher);
            apply_gradient(new_region, grad);
        }

        // 4. Optionally freeze previously learned regions
        if task_data.is_final_task() {
            self.frozen_regions.extend(self.active_regions.clone());
            self.active_regions.clear();
        }
    }
}
```

## 9. Implementation Checklist

### 9.1 Core Components
- [ ] Free Energy functional computation
- [ ] Gradient computation (∇_φ F, ∇_μ F)
- [ ] Fisher Information estimation (diagonal approximation)
- [ ] Natural gradient preconditioner
- [ ] Learning rate scheduler (neuromodulator-driven)

### 9.2 Advanced Features
- [ ] Persistent homology computation
- [ ] Topological projection
- [ ] Discrete Ricci flow for lattice
- [ ] CBR batch learning
- [ ] EWC for continual learning
- [ ] Meta-learning (Q-learning for λ_h)

### 9.3 GPU Optimization
- [ ] CUDA kernel for Free Energy computation
- [ ] CUDA kernel for gradient computation
- [ ] Batched Fisher estimation
- [ ] Parallel Ricci flow
- [ ] Coalesced memory access patterns

## 10. Performance Targets

| Operation | Latency | Memory |
|-----------|---------|--------|
| Free Energy (E8, 240 points) | < 10 μs | 10 KB |
| Gradient computation | < 20 μs | 20 KB |
| Natural gradient | < 5 μs | 5 KB |
| Full learning step | < 100 μs | 50 KB |
| CBR batch (N=10K, |G|=696M) | < 1 ms | 500 KB |

Target hardware: RTX 3070 (7GB VRAM), full system < 7GB total

## 11. References

1. Friston, K. (2010). "The free-energy principle: a unified brain theory?"
2. Amari, S. (1998). "Natural gradient works efficiently in learning"
3. Edelsbrunner, H., & Harer, J. (2010). *Computational Topology: An Introduction*
4. Perelman, G. (2002). "The entropy formula for the Ricci flow"
5. Kirkpatrick, J., et al. (2017). "Overcoming catastrophic forgetting in neural networks"

## 12. Next Steps

After absorbing this theory:
1. Implement Free Energy functional in `src/tic/free_energy.rs`
2. Create natural gradient optimizer in `src/tic/learning.rs`
3. Build topological constraint system
4. Develop Ricci flow regularization
5. Integrate with phase field dynamics
6. Create meta-learning framework
7. Optimize with CUDA kernels
