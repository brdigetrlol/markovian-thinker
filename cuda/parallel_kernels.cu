/**
 * CUDA Kernels for Parallel Task Execution
 *
 * Implements GPU-accelerated kernels for:
 * - Batch token processing
 * - Parallel multi-head attention
 * - SSM (State Space Model) selective scan
 * - Data processing operations
 */

#include <cuda_runtime.h>
#include <math.h>
#include <stdio.h>

// ============================================================================
// Utility Functions and Device Code
// ============================================================================

__device__ inline float sigmoid(float x) {
    return 1.0f / (1.0f + expf(-x));
}

__device__ inline float tanh_activation(float x) {
    return tanhf(x);
}

__device__ inline float gelu(float x) {
    // GELU approximation: 0.5 * x * (1 + tanh(sqrt(2/pi) * (x + 0.044715 * x^3)))
    const float sqrt_2_over_pi = 0.7978845608f;
    const float coeff = 0.044715f;
    float x_cubed = x * x * x;
    return 0.5f * x * (1.0f + tanhf(sqrt_2_over_pi * (x + coeff * x_cubed)));
}

__device__ inline float softmax_warp_reduce(float val) {
    // Warp-level reduction for softmax
    for (int offset = 16; offset > 0; offset /= 2) {
        val += __shfl_down_sync(0xffffffff, val, offset);
    }
    return val;
}

// ============================================================================
// Kernel 1: Batch Token Processing
// ============================================================================

/**
 * Process token embeddings in batches
 * Applies layer normalization and activation functions
 *
 * Input:  [batch_size, seq_len, embed_dim]
 * Output: [batch_size, seq_len, embed_dim]
 */
__global__ void batch_token_process(
    const float* __restrict__ input,
    float* __restrict__ output,
    const float* __restrict__ gamma,  // Layer norm scale
    const float* __restrict__ beta,   // Layer norm bias
    const int batch_size,
    const int seq_len,
    const int embed_dim
) {
    int batch_idx = blockIdx.x;
    int token_idx = blockIdx.y;
    int dim_idx = threadIdx.x;

    if (batch_idx >= batch_size || token_idx >= seq_len || dim_idx >= embed_dim) {
        return;
    }

    int idx = batch_idx * seq_len * embed_dim +
              token_idx * embed_dim +
              dim_idx;

    // Step 1: Compute mean and variance for layer norm (per token)
    __shared__ float mean;
    __shared__ float var;

    if (dim_idx == 0) {
        float sum = 0.0f;
        float sum_sq = 0.0f;
        int base_idx = batch_idx * seq_len * embed_dim + token_idx * embed_dim;

        for (int i = 0; i < embed_dim; i++) {
            float val = input[base_idx + i];
            sum += val;
            sum_sq += val * val;
        }

        mean = sum / embed_dim;
        var = (sum_sq / embed_dim) - (mean * mean);
    }
    __syncthreads();

    // Step 2: Apply layer normalization
    float val = input[idx];
    float normalized = (val - mean) / sqrtf(var + 1e-5f);

    // Apply learned scale and bias
    float scaled = normalized * gamma[dim_idx] + beta[dim_idx];

    // Step 3: Apply GELU activation
    output[idx] = gelu(scaled);
}

// ============================================================================
// Kernel 2: Parallel Multi-Head Attention
// ============================================================================

/**
 * Compute multi-head attention across batches in parallel
 *
 * Q, K, V: [batch, heads, seq_len, head_dim]
 * Output:  [batch, heads, seq_len, head_dim]
 */
__global__ void batch_multi_head_attention(
    const float* __restrict__ queries,
    const float* __restrict__ keys,
    const float* __restrict__ values,
    float* __restrict__ output,
    float* __restrict__ attention_scores,  // Workspace for scores
    const int batch_size,
    const int num_heads,
    const int seq_len,
    const int head_dim
) {
    // Each block handles one (batch, head) pair
    int batch_idx = blockIdx.x;
    int head_idx = blockIdx.y;
    int token_idx = threadIdx.x;  // Query token

    if (batch_idx >= batch_size || head_idx >= num_heads || token_idx >= seq_len) {
        return;
    }

    const float scale = 1.0f / sqrtf((float)head_dim);

    // Base indices
    int qkv_base = batch_idx * num_heads * seq_len * head_dim +
                   head_idx * seq_len * head_dim;
    int q_base = qkv_base + token_idx * head_dim;

    // Shared memory for attention scores
    extern __shared__ float smem[];
    float* scores = smem;

    // Step 1: Compute attention scores (Q * K^T)
    for (int key_idx = 0; key_idx < seq_len; key_idx++) {
        int k_base = qkv_base + key_idx * head_dim;

        float score = 0.0f;
        for (int d = 0; d < head_dim; d++) {
            score += queries[q_base + d] * keys[k_base + d];
        }
        scores[key_idx] = score * scale;
    }
    __syncthreads();

    // Step 2: Apply softmax
    // Find max for numerical stability
    float max_score = scores[0];
    for (int i = 1; i < seq_len; i++) {
        max_score = fmaxf(max_score, scores[i]);
    }

    // Compute exp and sum
    float sum_exp = 0.0f;
    for (int i = 0; i < seq_len; i++) {
        scores[i] = expf(scores[i] - max_score);
        sum_exp += scores[i];
    }

    // Normalize
    for (int i = 0; i < seq_len; i++) {
        scores[i] /= sum_exp;
    }
    __syncthreads();

    // Step 3: Weighted sum of values (attention * V)
    for (int d = 0; d < head_dim; d++) {
        float out_val = 0.0f;
        for (int value_idx = 0; value_idx < seq_len; value_idx++) {
            int v_idx = qkv_base + value_idx * head_dim + d;
            out_val += scores[value_idx] * values[v_idx];
        }

        int out_idx = qkv_base + token_idx * head_dim + d;
        output[out_idx] = out_val;
    }
}

// ============================================================================
// Kernel 3: SSM Selective Scan (Mamba-style)
// ============================================================================

/**
 * State Space Model selective scan with parallel batch processing
 * Implements the core Mamba/S4 selective scan operation
 *
 * Input:  [batch, seq_len, d_model]
 * Delta:  [batch, seq_len, d_state] - selection mechanism
 * A:      [d_model, d_state] - state matrix
 * B:      [batch, seq_len, d_state] - input projection
 * C:      [batch, seq_len, d_state] - output projection
 * Output: [batch, seq_len, d_model]
 */
__global__ void ssm_selective_scan(
    const float* __restrict__ input,
    const float* __restrict__ delta,
    const float* __restrict__ A,
    const float* __restrict__ B,
    const float* __restrict__ C,
    float* __restrict__ output,
    const int batch_size,
    const int seq_len,
    const int d_model,
    const int d_state
) {
    // Each block processes one sequence in the batch
    int batch_idx = blockIdx.x;
    int model_idx = threadIdx.x;  // Dimension in d_model

    if (batch_idx >= batch_size || model_idx >= d_model) {
        return;
    }

    // Shared memory for hidden state
    extern __shared__ float state[];  // Size: d_state

    // Initialize state to zero
    if (threadIdx.x < d_state) {
        state[threadIdx.x] = 0.0f;
    }
    __syncthreads();

    // Sequential scan over sequence length
    for (int t = 0; t < seq_len; t++) {
        int input_idx = batch_idx * seq_len * d_model + t * d_model + model_idx;
        float x_t = input[input_idx];

        // Compute discretized A and B based on delta
        // delta acts as the selection mechanism
        int delta_base = batch_idx * seq_len * d_state + t * d_state;
        int bc_base = delta_base;

        // Update state: h_t = A * h_{t-1} + B * x_t
        // With selective mechanism from delta
        float new_state_contribution = 0.0f;

        for (int s = 0; s < d_state; s++) {
            float delta_val = delta[delta_base + s];
            float a_val = A[model_idx * d_state + s];
            float b_val = B[bc_base + s];

            // Discretization: exp(-delta * A)
            float discretized_a = expf(-delta_val * a_val);
            float discretized_b = delta_val * b_val;

            // State update
            if (threadIdx.x == model_idx) {
                state[s] = discretized_a * state[s] + discretized_b * x_t;
            }
        }
        __syncthreads();

        // Compute output: y_t = C * h_t
        float y_t = 0.0f;
        for (int s = 0; s < d_state; s++) {
            float c_val = C[bc_base + s];
            y_t += c_val * state[s];
        }

        // Write output
        output[input_idx] = y_t;
        __syncthreads();
    }
}

// ============================================================================
// Kernel 4: Parallel Data Processing Operations
// ============================================================================

/**
 * Transform: Multiply all values by a factor
 */
__global__ void data_transform(
    const float* __restrict__ input,
    float* __restrict__ output,
    const float factor,
    const int batch_size,
    const int array_size
) {
    int batch_idx = blockIdx.x;
    int elem_idx = threadIdx.x + blockIdx.y * blockDim.x;

    if (batch_idx >= batch_size || elem_idx >= array_size) {
        return;
    }

    int idx = batch_idx * array_size + elem_idx;
    output[idx] = input[idx] * factor;
}

/**
 * Filter: Keep only values above threshold
 */
__global__ void data_filter(
    const float* __restrict__ input,
    float* __restrict__ output,
    int* __restrict__ output_counts,  // Per-batch output count
    const float threshold,
    const int batch_size,
    const int array_size
) {
    int batch_idx = blockIdx.x;
    int elem_idx = threadIdx.x + blockIdx.y * blockDim.x;

    if (batch_idx >= batch_size || elem_idx >= array_size) {
        return;
    }

    int idx = batch_idx * array_size + elem_idx;
    float val = input[idx];

    // Use atomic to count and write filtered values
    if (val > threshold) {
        int pos = atomicAdd(&output_counts[batch_idx], 1);
        output[batch_idx * array_size + pos] = val;
    }
}

/**
 * Aggregate: Compute statistics (mean, sum, etc.)
 */
__global__ void data_aggregate(
    const float* __restrict__ input,
    float* __restrict__ output_mean,
    float* __restrict__ output_sum,
    float* __restrict__ output_max,
    float* __restrict__ output_min,
    const int batch_size,
    const int array_size
) {
    int batch_idx = blockIdx.x;
    int tid = threadIdx.x;

    if (batch_idx >= batch_size) {
        return;
    }

    extern __shared__ float sdata[];
    float* sum_data = sdata;
    float* max_data = sdata + blockDim.x;
    float* min_data = sdata + 2 * blockDim.x;

    // Load data
    int idx = batch_idx * array_size + tid;
    float val = (tid < array_size) ? input[idx] : 0.0f;

    sum_data[tid] = val;
    max_data[tid] = val;
    min_data[tid] = val;
    __syncthreads();

    // Reduction
    for (int s = blockDim.x / 2; s > 0; s >>= 1) {
        if (tid < s && (tid + s) < array_size) {
            sum_data[tid] += sum_data[tid + s];
            max_data[tid] = fmaxf(max_data[tid], max_data[tid + s]);
            min_data[tid] = fminf(min_data[tid], min_data[tid + s]);
        }
        __syncthreads();
    }

    // Write results
    if (tid == 0) {
        output_sum[batch_idx] = sum_data[0];
        output_mean[batch_idx] = sum_data[0] / array_size;
        output_max[batch_idx] = max_data[0];
        output_min[batch_idx] = min_data[0];
    }
}

// ============================================================================
// Kernel 5: Multi-Agent Simulation
// ============================================================================

/**
 * Update agent states in parallel
 * Each agent has: position (x, y), velocity (vx, vy), state
 */
__global__ void agent_simulation_step(
    float* __restrict__ agent_positions,    // [num_agents, 2]
    float* __restrict__ agent_velocities,   // [num_agents, 2]
    int* __restrict__ agent_states,         // [num_agents]
    const float* __restrict__ env_params,   // Environment parameters
    const int num_agents,
    const float dt  // Time step
) {
    int agent_idx = blockIdx.x * blockDim.x + threadIdx.x;

    if (agent_idx >= num_agents) {
        return;
    }

    // Load agent data
    float x = agent_positions[agent_idx * 2];
    float y = agent_positions[agent_idx * 2 + 1];
    float vx = agent_velocities[agent_idx * 2];
    float vy = agent_velocities[agent_idx * 2 + 1];
    int state = agent_states[agent_idx];

    // Environment parameters
    float world_size = env_params[0];
    float interaction_radius = env_params[1];
    float max_speed = env_params[2];

    // Simple behavior: random walk with boundary conditions
    // In a real implementation, this would include agent interactions

    // Update position
    x += vx * dt;
    y += vy * dt;

    // Boundary wrapping
    if (x < 0) x += world_size;
    if (x >= world_size) x -= world_size;
    if (y < 0) y += world_size;
    if (y >= world_size) y -= world_size;

    // Simple velocity damping
    vx *= 0.99f;
    vy *= 0.99f;

    // Write back
    agent_positions[agent_idx * 2] = x;
    agent_positions[agent_idx * 2 + 1] = y;
    agent_velocities[agent_idx * 2] = vx;
    agent_velocities[agent_idx * 2 + 1] = vy;
}

// ============================================================================
// Host Functions for Kernel Launch Configuration
// ============================================================================

extern "C" {

/**
 * Get optimal block size for a given kernel
 */
int get_optimal_block_size(int num_elements) {
    if (num_elements < 128) return 64;
    if (num_elements < 512) return 128;
    return 256;
}

/**
 * Launch batch token processing
 */
void launch_batch_token_process(
    const float* input,
    float* output,
    const float* gamma,
    const float* beta,
    int batch_size,
    int seq_len,
    int embed_dim,
    cudaStream_t stream
) {
    dim3 grid(batch_size, seq_len);
    dim3 block(embed_dim);

    batch_token_process<<<grid, block, 0, stream>>>(
        input, output, gamma, beta,
        batch_size, seq_len, embed_dim
    );
}

/**
 * Launch multi-head attention
 */
void launch_batch_attention(
    const float* queries,
    const float* keys,
    const float* values,
    float* output,
    float* attention_scores,
    int batch_size,
    int num_heads,
    int seq_len,
    int head_dim,
    cudaStream_t stream
) {
    dim3 grid(batch_size, num_heads);
    dim3 block(seq_len);
    size_t smem_size = seq_len * sizeof(float);

    batch_multi_head_attention<<<grid, block, smem_size, stream>>>(
        queries, keys, values, output, attention_scores,
        batch_size, num_heads, seq_len, head_dim
    );
}

/**
 * Launch SSM selective scan
 */
void launch_ssm_scan(
    const float* input,
    const float* delta,
    const float* A,
    const float* B,
    const float* C,
    float* output,
    int batch_size,
    int seq_len,
    int d_model,
    int d_state,
    cudaStream_t stream
) {
    dim3 grid(batch_size);
    dim3 block(d_model);
    size_t smem_size = d_state * sizeof(float);

    ssm_selective_scan<<<grid, block, smem_size, stream>>>(
        input, delta, A, B, C, output,
        batch_size, seq_len, d_model, d_state
    );
}

/**
 * Launch data transform
 */
void launch_data_transform(
    const float* input,
    float* output,
    float factor,
    int batch_size,
    int array_size,
    cudaStream_t stream
) {
    dim3 grid(batch_size, (array_size + 255) / 256);
    dim3 block(256);

    data_transform<<<grid, block, 0, stream>>>(
        input, output, factor, batch_size, array_size
    );
}

/**
 * Launch agent simulation
 */
void launch_agent_simulation(
    float* positions,
    float* velocities,
    int* states,
    const float* env_params,
    int num_agents,
    float dt,
    cudaStream_t stream
) {
    int block_size = 256;
    int grid_size = (num_agents + block_size - 1) / block_size;

    agent_simulation_step<<<grid_size, block_size, 0, stream>>>(
        positions, velocities, states, env_params,
        num_agents, dt
    );
}

/**
 * Adam optimizer step kernel
 * Updates parameters using Adam optimization algorithm
 */
__global__ void adam_optimizer_step(
    float* params,
    const float* grads,
    float* momentum,
    float* velocity,
    int n,
    float lr,
    float beta1,
    float beta2,
    float epsilon,
    float weight_decay,
    float grad_clip
) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx >= n) return;

    // Get gradient
    float g = grads[idx];

    // Gradient clipping
    if (!isinf(grad_clip)) {
        g = fmaxf(-grad_clip, fminf(grad_clip, g));
    }

    // Weight decay
    if (weight_decay > 0.0f) {
        g += weight_decay * params[idx];
    }

    // Update biased first moment estimate (momentum)
    float m = beta1 * momentum[idx] + (1.0f - beta1) * g;
    momentum[idx] = m;

    // Update biased second raw moment estimate (velocity)
    float v = beta2 * velocity[idx] + (1.0f - beta2) * g * g;
    velocity[idx] = v;

    // Update parameters
    params[idx] -= lr * m / (sqrtf(v) + epsilon);
}

/**
 * SGD optimizer step kernel
 * Updates parameters using stochastic gradient descent
 */
__global__ void sgd_optimizer_step(
    float* params,
    const float* grads,
    int n,
    float lr,
    float weight_decay,
    float grad_clip
) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx >= n) return;

    // Get gradient
    float g = grads[idx];

    // Gradient clipping
    if (!isinf(grad_clip)) {
        g = fmaxf(-grad_clip, fminf(grad_clip, g));
    }

    // Weight decay
    if (weight_decay > 0.0f) {
        g += weight_decay * params[idx];
    }

    // Update parameters
    params[idx] -= lr * g;
}

/**
 * Launch Adam optimizer step
 */
void launch_adam_optimizer(
    float* params,
    const float* grads,
    float* momentum,
    float* velocity,
    int n,
    float lr,
    float beta1,
    float beta2,
    float epsilon,
    float weight_decay,
    float grad_clip,
    cudaStream_t stream
) {
    int block_size = 256;
    int grid_size = (n + block_size - 1) / block_size;

    adam_optimizer_step<<<grid_size, block_size, 0, stream>>>(
        params, grads, momentum, velocity,
        n, lr, beta1, beta2, epsilon,
        weight_decay, grad_clip
    );
}

/**
 * Launch SGD optimizer step
 */
void launch_sgd_optimizer(
    float* params,
    const float* grads,
    int n,
    float lr,
    float weight_decay,
    float grad_clip,
    cudaStream_t stream
) {
    int block_size = 256;
    int grid_size = (n + block_size - 1) / block_size;

    sgd_optimizer_step<<<grid_size, block_size, 0, stream>>>(
        params, grads, n, lr, weight_decay, grad_clip
    );
}

} // extern "C"
