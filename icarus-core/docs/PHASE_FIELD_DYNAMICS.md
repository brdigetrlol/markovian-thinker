# Phase Field Dynamics for Resonant Attractor Manifold

## Overview

This document describes the mathematical foundations for the Resonant Attractor Manifold - the unified field theory of cognition in the TIC architecture.

## 1. Complex-Valued Phase Field

### 1.1 Field Definition

The cognitive state at each lattice point r is represented by a complex-valued field:

```
z(r, t) = A(r, t) * e^(iθ(r,t))
```

Where:
- **A(r,t)**: Amplitude - represents certainty/stability of concept at location r
- **θ(r,t)**: Phase - represents temporal dynamics/activation timing

### 1.2 Physical Interpretation

| Component | Meaning | Range |
|-----------|---------|-------|
| A | Concept strength | [0, ∞) |
| θ | Activation phase | [0, 2π) |
| \|z\| | Total activation | [0, ∞) |
| arg(z) | Oscillation state | [0, 2π) |

### 1.3 Spinor Representation (for CUDA)

For computational efficiency:
```rust
struct ComplexField {
    real: f32,  // A * cos(θ)
    imag: f32,  // A * sin(θ)
}
```

Convert to/from amplitude-phase:
```rust
impl ComplexField {
    fn from_polar(amplitude: f32, phase: f32) -> Self {
        Self {
            real: amplitude * phase.cos(),
            imag: amplitude * phase.sin(),
        }
    }

    fn amplitude(&self) -> f32 {
        (self.real * self.real + self.imag * self.imag).sqrt()
    }

    fn phase(&self) -> f32 {
        self.imag.atan2(self.real)
    }
}
```

## 2. Resonant Attractor Equation

### 2.1 Evolution Equation

The field evolves according to:

```
∂z/∂t = -δF/δz* + iωz + ξ(r,t)
```

**Term breakdown:**
1. **-δF/δz\***: Free Energy gradient (content of thought)
2. **iωz**: Intrinsic oscillation (rhythm of thought)
3. **ξ(r,t)**: Noise term (stochasticity/exploration)

### 2.2 Free Energy Functional

```
F[z] = ∫ [ (ε/2)|∇z|² + V(|z|) + U_context(z, z_context) ] d^n r
```

**Components:**

#### Gradient Energy (ε/2)|∇z|²
- Penalizes rapid spatial changes
- Promotes smooth concept transitions
- ε modulated by ACh (acetylcholine analog)

#### Local Potential V(|z|)
Double-well potential (Allen-Cahn style):
```
V(|z|) = (λ/4)(|z|² - 1)²
```

Creates two stable states:
- |z| = 0: Concept inactive
- |z| = 1: Concept active

#### Contextual Coupling U_context
```
U_context(z, z_ctx) = -Σ_i J_i * Re(z * conj(z_i))
```

Where z_i are neighboring/contextually-related fields.
J_i are coupling strengths (learned via Geometrodynamic Learning).

### 2.3 Discretized Evolution (for numerical implementation)

Using Backward Euler (implicit) for stability:

```
z^(n+1) - z^n
------------- = -δF/δz*|_(n+1) + iω z^(n+1) + ξ^n
     Δt
```

Rearrange:
```
(1 + Δt*iω)z^(n+1) + Δt*δF/δz*|_(n+1) = z^n + Δt*ξ^n
```

Solve via Newton-Raphson:
```rust
fn evolve_field(
    z_current: &Array2<Complex<f32>>,
    delta_t: f32,
    omega: f32,
    free_energy_gradient: impl Fn(&Array2<Complex<f32>>) -> Array2<Complex<f32>>,
) -> Array2<Complex<f32>> {
    let mut z_next = z_current.clone();

    // Newton-Raphson iteration
    for _ in 0..MAX_ITERATIONS {
        let grad_f = free_energy_gradient(&z_next);
        let residual = &z_next - z_current - delta_t * (iω * &z_next - &grad_f);

        if residual.norm() < TOLERANCE {
            break;
        }

        // Jacobian (simplified): J ≈ I + Δt*iω
        let correction = &residual / (1.0 + delta_t * omega);
        z_next = &z_next - &correction;
    }

    z_next
}
```

## 3. Emergent Phenomena

### 3.1 Attention as Phase Coherence

**Mechanism:**
- Multiple oscillators phase-lock: θ_i(t) → θ_common(t)
- Constructive interference: A_total = Σ A_i (when phases align)
- High-amplitude coherent region = attentional focus

**Coherence measure:**
```
Φ_coherence = |⟨e^(iθ)⟩| = |Σ_i e^(iθ_i)| / N
```

Range: [0, 1]
- Φ = 1: Perfect coherence (strong attention)
- Φ = 0: Random phases (no attention)

**Implementation:**
```rust
fn compute_coherence(phases: &[f32]) -> f32 {
    let n = phases.len() as f32;
    let sum_cos: f32 = phases.iter().map(|&θ| θ.cos()).sum();
    let sum_sin: f32 = phases.iter().map(|&θ| θ.sin()).sum();
    ((sum_cos*sum_cos + sum_sin*sum_sin).sqrt()) / n
}
```

### 3.2 Surprise as Decoherence

**Mechanism:**
- Unexpected sensory input arrives with phase θ_s
- Existing field has different phase θ_existing
- Destructive interference: Δθ = |θ_s - θ_existing| ~ π
- Decoherence wave propagates through lattice

**Decoherence detector:**
```
D(r,t) = |∂θ/∂t| * A(r,t)
```

High decoherence → Surprise signal → Neuromodulator release (NE analog)

**Implementation:**
```rust
fn detect_decoherence(
    field_current: &Array2<Complex<f32>>,
    field_previous: &Array2<Complex<f32>>,
    delta_t: f32,
) -> Array2<f32> {
    let phase_current = field_current.mapv(|z| z.arg());
    let phase_previous = field_previous.mapv(|z| z.arg());
    let phase_velocity = (&phase_current - &phase_previous) / delta_t;

    let amplitude = field_current.mapv(|z| z.norm());

    &phase_velocity.mapv(|x| x.abs()) * &amplitude
}
```

### 3.3 Learning as Entrainment

**Hebbian-style phase locking:**
```
dJ_ij/dt = η * A_i * A_j * cos(θ_i - θ_j) * (-δF/δJ_ij)
```

Oscillators that repeatedly co-activate with aligned phases strengthen their coupling.

Learning rate modulated by:
```
η_effective = η_base * (DA_phasic / (DA_tonic + k))
```

Where DA = dopamine analog (surprise/reward signal).

### 3.4 Emotion as Physical State

#### Valence (Positive/Negative Emotion)
```
Valence = -dF/dt
```

- Decreasing F (error reduction) → Positive valence
- Increasing F (error increase) → Negative valence

Direct readout of Free Energy gradient!

#### Arousal (Calm/Excited)
```
Arousal = ⟨|∂θ/∂t|⟩_space
```

Average phase decoherence across manifold.

- Low decoherence → Calm
- High decoherence → Excited/Anxious

**Emotional state vector:**
```rust
struct EmotionalState {
    valence: f32,      // [-1, 1]
    arousal: f32,      // [0, 1]
    free_energy: f32,  // [0, ∞)
}

impl EmotionalState {
    fn from_field_dynamics(
        free_energy_current: f32,
        free_energy_previous: f32,
        decoherence: f32,
        delta_t: f32,
    ) -> Self {
        let d_f_dt = (free_energy_current - free_energy_previous) / delta_t;

        Self {
            valence: -d_f_dt.tanh(),  // Map to [-1, 1]
            arousal: decoherence.min(1.0),
            free_energy: free_energy_current,
        }
    }
}
```

## 4. Neuromodulation via Field Parameters

### 4.1 Parameter Space

The physics of the phase field is modulated by:

| Parameter | Biological Analog | Effect |
|-----------|-------------------|--------|
| ε (gradient energy) | Acetylcholine | Attention precision |
| λ (potential depth) | Dopamine | Bistability/commitment |
| T (temperature) | Norepinephrine | Stochasticity/exploration |
| J_ij (coupling) | Synaptic weights | Learned associations |

### 4.2 Acetylcholine (ACh) Analog

**Tonic component:**
```
ACh_tonic = ⟨1 - |Prediction_Error|⟩_time
```

Overall confidence in world model.

**Phasic component:**
```
ACh_phasic = Δ(Attention_Target)
```

Top-down signal indicating attentional shift.

**Modulation:**
```
ε(r,t) = ε_base * (1 + ACh_phasic(r,t)) * ACh_tonic
```

High ACh → Sharper gradients → Focused attention

### 4.3 Dopamine (DA) Analog

**Tonic component:**
```
DA_tonic = ⟨-ΔF⟩_time = average reward rate
```

**Phasic component:**
```
DA_phasic = (-ΔF) - DA_tonic
```

Reward prediction error!

**Modulation:**
```
λ(t) = λ_base * (1 + DA_phasic)
```

High DA → Deeper potential wells → Stronger commitment to activated states

### 4.4 Norepinephrine (NE) Analog

**Tonic component:**
```
NE_tonic = ⟨Decoherence⟩_space
```

Overall arousal level.

**Phasic component:**
```
NE_phasic = Surprise_detector = max(Decoherence) > threshold
```

**Modulation:**
```
T(t) = T_base + NE_tonic + k * NE_phasic
```

High NE → Higher temperature → More exploration/stochasticity

## 5. Cerebellar Forward Model

### 5.1 Architecture

**Purpose:** Predict consequences of planned actions before execution.

**Components:**

#### Granule Cell Analog
High-dimensional sparse projection:
```
x = Sparsify(ReLU(W_gc * [φ_t; Δμ_plan]))
```

Where:
- φ_t: Current field state
- Δμ_plan: Planned neuromodulator change
- W_gc: Random projection matrix

#### Purkinje Cell Analog
Linear readout:
```
ε_predicted = W_pc * x
```

Predicts future error.

### 5.2 Learning Rule

Error-driven Long-Term Depression (LTD):
```
ΔW_pc = -η * (ε_actual - ε_predicted) * x^T
```

Learn to predict actual errors that occur.

### 5.3 Intuition Implementation

**"Gut feeling"**:
```
Intuition_value = -ε_predicted
```

Negative if action will increase error (bad feeling).
Positive if action will decrease error (good feeling).

**Use in planning:**
```rust
fn evaluate_action_intuition(
    current_state: &PhaseField,
    proposed_action: &Action,
    cerebellar_model: &CerebellumForwardModel,
) -> f32 {
    let predicted_error = cerebellar_model.predict(current_state, proposed_action);
    -predicted_error  // Intuition = negative of predicted error
}
```

## 6. Implementation Roadmap

### 6.1 Phase 1: Basic Field Evolution
- [ ] Complex field data structure
- [ ] Free Energy functional (gradient + double-well)
- [ ] Backward Euler solver
- [ ] Newtonian iteration for implicit step

### 6.2 Phase 2: Emergent Phenomena
- [ ] Coherence computation (attention)
- [ ] Decoherence detection (surprise)
- [ ] Entrainment learning (Hebbian)
- [ ] Emotional state readout

### 6.3 Phase 3: Neuromodulation
- [ ] ACh analog (precision modulation)
- [ ] DA analog (commitment modulation)
- [ ] NE analog (temperature modulation)
- [ ] Self-modulating parameter updates

### 6.4 Phase 4: Cerebellar Model
- [ ] Granule cell projection
- [ ] Purkinje cell readout
- [ ] LTD learning rule
- [ ] Intuition integration with planning

## 7. CUDA Optimization Strategy

### 7.1 Memory Layout

Use Structure-of-Arrays (SoA) for coalesced access:
```cuda
struct PhaseFieldGPU {
    float *real;  // N elements
    float *imag;  // N elements
}
```

Not Array-of-Structures (AoS):
```cuda
// AVOID THIS:
struct {
    float real;
    float imag;
} *field;
```

### 7.2 Kernel Launch Configuration

For lattice size N = n^d:
```cuda
dim3 blockSize(16, 16);  // 256 threads/block
dim3 gridSize((n + 15) / 16, (n + 15) / 16);

evolve_field_kernel<<<gridSize, blockSize>>>(d_field, d_params);
```

### 7.3 Shared Memory for Stencil Operations

For gradient computation:
```cuda
__shared__ float s_real[TILE_SIZE + 2][TILE_SIZE + 2];
// Load halo region
// Compute gradient using shared memory
// Write result
```

## 8. Performance Targets

| Operation | Latency | Throughput | Memory |
|-----------|---------|------------|--------|
| Field evolution (E8, 240 points) | < 10 μs | > 100K steps/sec | 7.7 KB |
| Coherence computation | < 1 μs | > 1M/sec | 1 KB |
| Emotional state update | < 0.1 μs | > 10M/sec | 12 bytes |
| CBR forward model | < 50 μs | > 20K predictions/sec | 100 KB |

Target: Full cognitive cycle (all layers) < 1 ms on RTX 3070

## 9. Mathematical References

1. Allen, S. M., & Cahn, J. W. (1979). "A microscopic theory for antiphase boundary motion".
2. Friston, K. (2010). "The free-energy principle: a unified brain theory?"
3. Kuramoto, Y. (1984). *Chemical Oscillations, Waves, and Turbulence*.
4. Hopfield, J. J. (1982). "Neural networks and physical systems with emergent collective computational abilities".

## 10. Next Steps

After mastering this theory, proceed to:
1. Implement `src/tic/phase_field.rs` with ComplexField type
2. Implement `src/tic/resonant_attractor.rs` with PDE solver
3. Integrate with lattice module
4. Build cerebellar forward model
5. Create emotional state tracker
6. Develop CUDA kernels for GPU acceleration
