// build.rs - Compile CUDA kernels to PTX

fn main() {
    #[cfg(feature = "gpu")]
    {
        compile_cuda_kernels();
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=cuda/");
}

#[cfg(feature = "gpu")]
fn compile_cuda_kernels() {
    use std::env;
    use std::path::PathBuf;
    use std::process::Command;
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let cuda_file = "cuda/parallel_kernels.cu";

    println!("cargo:rerun-if-changed={}", cuda_file);

    // Find CUDA toolkit
    let cuda_path = find_cuda_path();

    if cuda_path.is_none() {
        println!("cargo:warning=CUDA toolkit not found. GPU features will be disabled.");
        println!("cargo:warning=Please install CUDA toolkit or set CUDA_PATH environment variable.");
        return;
    }

    let cuda_path = cuda_path.unwrap();
    let nvcc = cuda_path.join("bin").join(if cfg!(windows) { "nvcc.exe" } else { "nvcc" });

    println!("cargo:info=Using CUDA at: {}", cuda_path.display());

    // Output PTX file
    let ptx_output = out_dir.join("parallel_kernels.ptx");

    // Determine compute capability
    let compute_cap = get_compute_capability().unwrap_or("75".to_string()); // Default to Turing (7.5)

    println!("cargo:info=Compiling CUDA kernels for compute capability {}", compute_cap);

    // Compile to PTX
    let output = Command::new(&nvcc)
        .arg("--ptx")
        .arg(format!("-arch=sm_{}", compute_cap))
        .arg("--output-file")
        .arg(&ptx_output)
        .arg(cuda_file)
        .arg("-O3") // Optimize
        .arg("--use_fast_math") // Fast math
        .arg("--extra-device-vectorization") // Better vectorization
        .output();

    match output {
        Ok(output) => {
            if !output.status.success() {
                eprintln!("NVCC stderr: {}", String::from_utf8_lossy(&output.stderr));
                eprintln!("NVCC stdout: {}", String::from_utf8_lossy(&output.stdout));
                panic!("Failed to compile CUDA kernels");
            }

            println!("cargo:info=Successfully compiled CUDA kernels to PTX");
            println!("cargo:rustc-env=PTX_PATH={}", ptx_output.display());
        }
        Err(e) => {
            panic!("Failed to run nvcc: {}. Make sure CUDA toolkit is installed.", e);
        }
    }

    // Also tell cargo where to find CUDA libraries
    let cuda_lib_path = cuda_path.join(if cfg!(windows) { "lib/x64" } else { "lib64" });
    println!("cargo:rustc-link-search=native={}", cuda_lib_path.display());
    println!("cargo:rustc-link-lib=cuda");
    println!("cargo:rustc-link-lib=cudart");
}

/// Find CUDA installation path
#[cfg(feature = "gpu")]
fn find_cuda_path() -> Option<PathBuf> {
    // Try environment variable first
    if let Ok(cuda_path) = env::var("CUDA_PATH") {
        let path = PathBuf::from(cuda_path);
        if path.exists() {
            return Some(path);
        }
    }

    // Try common installation paths
    let common_paths = if cfg!(windows) {
        vec![
            r"C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA",
            r"C:\Program Files (x86)\NVIDIA GPU Computing Toolkit\CUDA",
        ]
    } else {
        vec![
            "/usr/local/cuda",
            "/opt/cuda",
            "/usr/lib/cuda",
        ]
    };

    for base_path in common_paths {
        let base = PathBuf::from(base_path);
        if !base.exists() {
            continue;
        }

        // Check if it's a versioned directory
        if base.join("bin").join("nvcc").exists() || base.join("bin").join("nvcc.exe").exists() {
            return Some(base);
        }

        // Check for versioned subdirectories
        if let Ok(entries) = std::fs::read_dir(&base) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let nvcc_path = path.join("bin").join(if cfg!(windows) { "nvcc.exe" } else { "nvcc" });
                    if nvcc_path.exists() {
                        return Some(path);
                    }
                }
            }
        }
    }

    // Try which/where command
    let which_cmd = if cfg!(windows) { "where" } else { "which" };
    if let Ok(output) = Command::new(which_cmd).arg("nvcc").output() {
        if output.status.success() {
            let nvcc_path = String::from_utf8_lossy(&output.stdout);
            let nvcc_path = nvcc_path.trim();
            if let Some(parent) = PathBuf::from(nvcc_path).parent() {
                if let Some(cuda_root) = parent.parent() {
                    return Some(cuda_root.to_path_buf());
                }
            }
        }
    }

    None
}

/// Detect GPU compute capability
#[cfg(feature = "gpu")]
fn get_compute_capability() -> Option<String> {
    // Try to detect from environment
    if let Ok(cap) = env::var("CUDA_COMPUTE_CAP") {
        return Some(cap.replace('.', ""));
    }

    // Try to detect using deviceQuery or nvidia-smi
    // For now, return common defaults
    // Users can override with CUDA_COMPUTE_CAP environment variable

    // Turing/Ampere/Ada (RTX 20xx, 30xx, 40xx): 75, 80, 86, 89
    // Volta (V100): 70
    // Pascal (GTX 10xx): 61

    // Default to Ampere (RTX 3000 series) which is most common
    Some("86".to_string())
}

#[cfg(not(feature = "gpu"))]
#[allow(dead_code)]
fn compile_cuda_kernels() {
    // No-op when GPU feature is disabled
}
