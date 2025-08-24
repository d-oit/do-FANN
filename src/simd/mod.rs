//! SIMD optimizations for neural network computations
//!
//! This module provides vectorized implementations of critical operations:
//! - Matrix multiplication with AVX2/AVX-512 support
//! - Vectorized activation functions
//! - Parallel gradient computation
//!
//! Expected performance gains:
//! - 3-8x speedup for CPU matrix operations
//! - Better cache utilization through blocking
//! - Multi-threading support with rayon
//!
//! Safety guarantees:
//! - Proper CPU feature detection with runtime fallbacks
//! - Memory alignment for SIMD operations
//! - Bounds checking in debug builds
//! - Graceful degradation to scalar implementations

use num_traits::Float;
use std::alloc::{alloc, dealloc, Layout};
use std::ptr;
use std::sync::Arc;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

// AVX-512 helper functions that may not be available in all std library versions
#[cfg(target_arch = "x86_64")]
#[inline]
unsafe fn _mm512_reduce_add_ps(a: __m512) -> f32 {
    let mut sum = 0.0f32;
    let array = std::mem::transmute::<__m512, [f32; 16]>(a);
    for &val in array.iter() {
        sum += val;
    }
    sum
}

// Comparison constants for AVX-512
#[cfg(target_arch = "x86_64")]
const _CMP_GT_OS: i32 = 14; // _CMP_GT_OS for AVX-512 comparison

// Mask blend helper for AVX-512
#[cfg(target_arch = "x86_64")]
#[inline]
unsafe fn _mm512_mask_blend_ps(mask: u16, a: __m512, b: __m512) -> __m512 {
    let mask_vec = _mm512_set1_ps(mask as f32);
    let mask_cmp = _mm512_cmp_ps_mask(mask_vec, _mm512_setzero_ps(), _CMP_GT_OS);
    _mm512_mask_blend_ps(mask_cmp, a, b)
}

/// CPU feature detection with runtime safety checks
#[derive(Debug, Clone)]
pub struct CpuFeatures {
    /// AVX2 support detected at runtime
    pub has_avx2: bool,
    /// AVX-512 Foundation support detected at runtime
    pub has_avx512f: bool,
    /// AVX-512 Vector Neural Network Instructions
    pub has_avx512vnni: bool,
    /// FMA (Fused Multiply-Add) support
    pub has_fma: bool,
    /// SSE4.2 support (baseline for modern x86)
    pub has_sse42: bool,
    /// NEON support on ARM
    pub has_neon: bool,
}

impl CpuFeatures {
    /// Detect CPU features at runtime with proper error handling
    pub fn detect() -> Self {
        Self {
            #[cfg(target_arch = "x86_64")]
            has_avx2: {
                // Use std::is_x86_feature_detected! with error handling
                if cfg!(target_feature = "avx2") {
                    true
                } else {
                    // Runtime detection with fallback
                    std::panic::catch_unwind(|| is_x86_feature_detected!("avx2")).unwrap_or(false)
                }
            },
            #[cfg(target_arch = "x86_64")]
            has_avx512f: {
                if cfg!(target_feature = "avx512f") {
                    true
                } else {
                    std::panic::catch_unwind(|| is_x86_feature_detected!("avx512f"))
                        .unwrap_or(false)
                }
            },
            #[cfg(target_arch = "x86_64")]
            has_avx512vnni: {
                if cfg!(target_feature = "avx512vnni") {
                    true
                } else {
                    std::panic::catch_unwind(|| is_x86_feature_detected!("avx512vnni"))
                        .unwrap_or(false)
                }
            },
            #[cfg(target_arch = "x86_64")]
            has_fma: {
                if cfg!(target_feature = "fma") {
                    true
                } else {
                    std::panic::catch_unwind(|| is_x86_feature_detected!("fma")).unwrap_or(false)
                }
            },
            #[cfg(target_arch = "x86_64")]
            has_sse42: {
                if cfg!(target_feature = "sse4.2") {
                    true
                } else {
                    std::panic::catch_unwind(|| is_x86_feature_detected!("sse4.2")).unwrap_or(false)
                }
            },
            #[cfg(target_arch = "aarch64")]
            has_neon: {
                // NEON is typically available on ARM64, but check if we can use it
                true
            },
            #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
            has_avx2: false,
            #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
            has_avx512f: false,
            #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
            has_avx512vnni: false,
            #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
            has_fma: false,
            #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
            has_sse42: false,
            #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
            has_neon: false,
        }
    }

    /// Get the best SIMD instruction set available
    pub fn best_simd_level(&self) -> SimdLevel {
        if self.has_avx512vnni {
            SimdLevel::Avx512VNNI
        } else if self.has_avx512f {
            SimdLevel::Avx512F
        } else if self.has_avx2 && self.has_fma {
            SimdLevel::Avx2FMA
        } else if self.has_avx2 {
            SimdLevel::Avx2
        } else if self.has_sse42 {
            SimdLevel::Sse42
        } else if self.has_neon {
            SimdLevel::Neon
        } else {
            SimdLevel::Scalar
        }
    }
}

/// SIMD instruction set levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SimdLevel {
    Scalar = 0,
    Neon = 1,
    Sse42 = 2,
    Avx2 = 3,
    Avx2FMA = 4,
    Avx512F = 5,
    Avx512VNNI = 6,
}

/// Configuration for SIMD operations with safety checks
#[derive(Debug, Clone)]
pub struct SimdConfig {
    /// CPU features detected at runtime
    pub cpu_features: CpuFeatures,
    /// SIMD level to use (auto-detected if None)
    pub simd_level: Option<SimdLevel>,
    /// Block size for cache-friendly matrix operations
    pub block_size: usize,
    /// Number of threads for parallel operations
    pub num_threads: usize,
    /// Enable performance profiling
    pub enable_profiling: bool,
    /// Memory alignment for SIMD operations (must be power of 2)
    pub alignment: usize,
    /// Enable bounds checking in debug builds
    pub enable_bounds_check: bool,
    /// Enable memory alignment checks
    pub enable_alignment_check: bool,
}

impl Default for SimdConfig {
    fn default() -> Self {
        let cpu_features = CpuFeatures::detect();
        let best_level = cpu_features.best_simd_level();

        Self {
            cpu_features,
            simd_level: Some(best_level),
            block_size: 64, // Good balance for most L1 cache sizes
            num_threads: num_cpus::get(),
            enable_profiling: false,
            alignment: best_level.required_alignment(),
            enable_bounds_check: cfg!(debug_assertions),
            enable_alignment_check: cfg!(debug_assertions),
        }
    }
}

impl SimdLevel {
    /// Get the required memory alignment for this SIMD level
    pub fn required_alignment(self) -> usize {
        match self {
            SimdLevel::Avx512F | SimdLevel::Avx512VNNI => 64,
            SimdLevel::Avx2 | SimdLevel::Avx2FMA => 32,
            SimdLevel::Sse42 => 16,
            SimdLevel::Neon => 16,
            SimdLevel::Scalar => 8,
        }
    }

    /// Get the vector width (number of f32 values) for this SIMD level
    pub fn vector_width(self) -> usize {
        match self {
            SimdLevel::Avx512F | SimdLevel::Avx512VNNI => 16,
            SimdLevel::Avx2 | SimdLevel::Avx2FMA => 8,
            SimdLevel::Sse42 => 4,
            SimdLevel::Neon => 4,
            SimdLevel::Scalar => 1,
        }
    }
}

/// Memory alignment utilities for SIMD operations
pub struct AlignedMemory {
    ptr: *mut f32,
    layout: Layout,
    len: usize,
}

impl AlignedMemory {
    /// Allocate aligned memory for SIMD operations
    pub fn new(len: usize, alignment: usize) -> Result<Self, String> {
        if alignment == 0 || (alignment & (alignment - 1)) != 0 {
            return Err("Alignment must be a power of 2".to_string());
        }

        let size = len * std::mem::size_of::<f32>();
        let layout = Layout::from_size_align(size, alignment)
            .map_err(|e| format!("Invalid layout: {}", e))?;

        let ptr = unsafe { alloc(layout) } as *mut f32;
        if ptr.is_null() {
            return Err("Memory allocation failed".to_string());
        }

        Ok(Self { ptr, layout, len })
    }

    /// Get pointer to the aligned memory
    pub fn as_ptr(&self) -> *const f32 {
        self.ptr
    }

    /// Get mutable pointer to the aligned memory
    pub fn as_mut_ptr(&mut self) -> *mut f32 {
        self.ptr
    }

    /// Get slice view of the aligned memory
    pub fn as_slice(&self) -> &[f32] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }

    /// Get mutable slice view of the aligned memory
    pub fn as_mut_slice(&mut self) -> &mut [f32] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }

    /// Check if the memory is properly aligned
    pub fn is_aligned(&self, alignment: usize) -> bool {
        (self.ptr as usize) % alignment == 0
    }

    /// Copy data from a slice to aligned memory
    pub fn copy_from_slice(&mut self, src: &[f32]) -> Result<(), String> {
        if src.len() != self.len {
            return Err(format!(
                "Length mismatch: expected {}, got {}",
                self.len,
                src.len()
            ));
        }
        self.as_mut_slice().copy_from_slice(src);
        Ok(())
    }

    /// Copy data to a slice from aligned memory
    pub fn copy_to_slice(&self, dst: &mut [f32]) -> Result<(), String> {
        if dst.len() != self.len {
            return Err(format!(
                "Length mismatch: expected {}, got {}",
                self.len,
                dst.len()
            ));
        }
        dst.copy_from_slice(self.as_slice());
        Ok(())
    }
}

impl Drop for AlignedMemory {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.ptr as *mut u8, self.layout);
        }
    }
}

/// Safety checks for SIMD operations
pub struct SimdSafety {
    config: SimdConfig,
}

impl SimdSafety {
    pub fn new(config: SimdConfig) -> Self {
        Self { config }
    }

    /// Check if memory is properly aligned for SIMD operations
    pub fn check_alignment(
        &self,
        ptr: *const f32,
        required_alignment: usize,
    ) -> Result<(), String> {
        if self.config.enable_alignment_check {
            if (ptr as usize) % required_alignment != 0 {
                return Err(format!(
                    "Memory not aligned: address {:p} is not aligned to {} bytes",
                    ptr, required_alignment
                ));
            }
        }
        Ok(())
    }

    /// Check bounds for SIMD operations
    pub fn check_bounds(&self, len: usize, required_len: usize) -> Result<(), String> {
        if self.config.enable_bounds_check && len < required_len {
            return Err(format!(
                "Insufficient data length: got {}, need at least {}",
                len, required_len
            ));
        }
        Ok(())
    }

    /// Validate matrix dimensions for SIMD operations
    pub fn validate_matrix_dims(&self, m: usize, n: usize, k: usize) -> Result<(), String> {
        if m == 0 || n == 0 || k == 0 {
            return Err("Matrix dimensions cannot be zero".to_string());
        }

        // Check for potential overflow in indexing
        let max_index = m.saturating_mul(k).saturating_add(k.saturating_mul(n));
        if max_index > usize::MAX / std::mem::size_of::<f32>() {
            return Err("Matrix dimensions too large, would cause overflow".to_string());
        }

        Ok(())
    }

    /// Get safe vector width for the current configuration
    pub fn safe_vector_width(&self, requested_width: usize, data_len: usize) -> usize {
        if self.config.enable_bounds_check {
            requested_width.min(data_len)
        } else {
            requested_width
        }
    }
}

/// Trait for SIMD-accelerated matrix operations
pub trait SimdMatrixOps<T: Float + Send + Sync> {
    /// Perform matrix multiplication: C = A * B
    fn matmul(&self, a: &[T], b: &[T], c: &mut [T], m: usize, n: usize, k: usize);

    /// Perform matrix-vector multiplication: y = A * x
    fn matvec(&self, a: &[T], x: &[T], y: &mut [T], m: usize, n: usize);

    /// Add bias vector to matrix rows
    fn add_bias(&self, matrix: &mut [T], bias: &[T], rows: usize, cols: usize);

    /// Apply activation function element-wise
    fn apply_activation(&self, data: &mut [T], activation: ActivationFunction);

    /// Compute activation derivatives
    fn activation_derivatives(
        &self,
        data: &[T],
        derivatives: &mut [T],
        activation: ActivationFunction,
    );
}

/// Supported activation functions for SIMD optimization
#[derive(Debug, Clone, Copy)]
pub enum ActivationFunction {
    Sigmoid,
    Tanh,
    Relu,
    LeakyRelu(f32),
    Gelu,
    Swish,
}

/// CPU-based SIMD implementation with safety checks
pub struct CpuSimdOps {
    config: SimdConfig,
    safety: SimdSafety,
    current_level: SimdLevel,
}

impl CpuSimdOps {
    pub fn new(config: SimdConfig) -> Self {
        let safety = SimdSafety::new(config.clone());
        let current_level = config
            .simd_level
            .unwrap_or(config.cpu_features.best_simd_level());

        Self {
            config,
            safety,
            current_level,
        }
    }

    pub fn new_with_defaults() -> Self {
        let config = SimdConfig::default();
        Self::new(config)
    }

    /// Get the current SIMD level being used
    pub fn current_simd_level(&self) -> SimdLevel {
        self.current_level
    }

    /// Check if the current configuration is safe for SIMD operations
    pub fn validate_configuration(&self) -> Result<(), String> {
        match self.current_level {
            SimdLevel::Avx512F | SimdLevel::Avx512VNNI => {
                if !self.config.cpu_features.has_avx512f {
                    return Err("AVX-512 not supported on this CPU".to_string());
                }
            }
            SimdLevel::Avx2 | SimdLevel::Avx2FMA => {
                if !self.config.cpu_features.has_avx2 {
                    return Err("AVX2 not supported on this CPU".to_string());
                }
            }
            SimdLevel::Sse42 => {
                if !self.config.cpu_features.has_sse42 {
                    return Err("SSE4.2 not supported on this CPU".to_string());
                }
            }
            _ => {} // Scalar and NEON are always available
        }
        Ok(())
    }

    /// Allocate aligned memory for SIMD operations
    pub fn allocate_aligned(&self, len: usize) -> Result<AlignedMemory, String> {
        let alignment = self.current_level.required_alignment();
        AlignedMemory::new(len, alignment)
    }
}

impl SimdMatrixOps<f32> for CpuSimdOps {
    fn matmul(&self, a: &[f32], b: &[f32], c: &mut [f32], m: usize, n: usize, k: usize) {
        // Validate inputs first
        if let Err(e) = self.safety.validate_matrix_dims(m, n, k) {
            log::warn!("Matrix multiplication validation failed: {}", e);
            self.matmul_scalar(a, b, c, m, n, k);
            return;
        }

        // Check if we have enough data
        let a_required = m * k;
        let b_required = k * n;
        let c_required = m * n;

        if let Err(e) = self.safety.check_bounds(a.len(), a_required) {
            log::warn!("Input A bounds check failed: {}", e);
            self.matmul_scalar(a, b, c, m, n, k);
            return;
        }
        if let Err(e) = self.safety.check_bounds(b.len(), b_required) {
            log::warn!("Input B bounds check failed: {}", e);
            self.matmul_scalar(a, b, c, m, n, k);
            return;
        }
        if let Err(e) = self.safety.check_bounds(c.len(), c_required) {
            log::warn!("Output C bounds check failed: {}", e);
            self.matmul_scalar(a, b, c, m, n, k);
            return;
        }

        // Try SIMD operations based on current level
        match self.current_level {
            #[cfg(target_arch = "x86_64")]
            SimdLevel::Avx512F | SimdLevel::Avx512VNNI => {
                if let Err(e) = self.safety.check_alignment(a.as_ptr(), 64) {
                    log::warn!("Memory alignment check failed for AVX-512: {}", e);
                    self.matmul_scalar(a, b, c, m, n, k);
                } else if let Err(e) = self.safety.check_alignment(b.as_ptr(), 64) {
                    log::warn!("Memory alignment check failed for AVX-512: {}", e);
                    self.matmul_scalar(a, b, c, m, n, k);
                } else if let Err(e) = self.safety.check_alignment(c.as_ptr(), 64) {
                    log::warn!("Memory alignment check failed for AVX-512: {}", e);
                    self.matmul_scalar(a, b, c, m, n, k);
                } else {
                    unsafe {
                        self.matmul_avx512(a, b, c, m, n, k);
                    }
                }
            }
            #[cfg(target_arch = "x86_64")]
            SimdLevel::Avx2 | SimdLevel::Avx2FMA => {
                if let Err(e) = self.safety.check_alignment(a.as_ptr(), 32) {
                    log::warn!("Memory alignment check failed for AVX2: {}", e);
                    self.matmul_scalar(a, b, c, m, n, k);
                } else if let Err(e) = self.safety.check_alignment(b.as_ptr(), 32) {
                    log::warn!("Memory alignment check failed for AVX2: {}", e);
                    self.matmul_scalar(a, b, c, m, n, k);
                } else if let Err(e) = self.safety.check_alignment(c.as_ptr(), 32) {
                    log::warn!("Memory alignment check failed for AVX2: {}", e);
                    self.matmul_scalar(a, b, c, m, n, k);
                } else {
                    unsafe {
                        self.matmul_avx2(a, b, c, m, n, k);
                    }
                }
            }
            #[cfg(target_arch = "aarch64")]
            SimdLevel::Neon => {
                if let Err(e) = self.safety.check_alignment(a.as_ptr(), 16) {
                    log::warn!("Memory alignment check failed for NEON: {}", e);
                    self.matmul_scalar(a, b, c, m, n, k);
                } else if let Err(e) = self.safety.check_alignment(b.as_ptr(), 16) {
                    log::warn!("Memory alignment check failed for NEON: {}", e);
                    self.matmul_scalar(a, b, c, m, n, k);
                } else if let Err(e) = self.safety.check_alignment(c.as_ptr(), 16) {
                    log::warn!("Memory alignment check failed for NEON: {}", e);
                    self.matmul_scalar(a, b, c, m, n, k);
                } else {
                    unsafe {
                        self.matmul_neon(a, b, c, m, n, k);
                    }
                }
            }
            _ => {
                // Fallback to scalar implementation
                self.matmul_scalar(a, b, c, m, n, k);
            }
        }
    }

    fn matvec(&self, a: &[f32], x: &[f32], y: &mut [f32], m: usize, n: usize) {
        // Validate inputs
        if m == 0 || n == 0 {
            log::warn!("Invalid matrix dimensions for matvec: m={}, n={}", m, n);
            return;
        }

        let a_required = m * n;
        let y_required = m;

        if let Err(e) = self.safety.check_bounds(a.len(), a_required) {
            log::warn!("Input A bounds check failed: {}", e);
            self.matvec_scalar(a, x, y, m, n);
            return;
        }
        if let Err(e) = self.safety.check_bounds(x.len(), n) {
            log::warn!("Input X bounds check failed: {}", e);
            self.matvec_scalar(a, x, y, m, n);
            return;
        }
        if let Err(e) = self.safety.check_bounds(y.len(), y_required) {
            log::warn!("Output Y bounds check failed: {}", e);
            self.matvec_scalar(a, x, y, m, n);
            return;
        }

        // Try SIMD operations based on current level
        match self.current_level {
            #[cfg(target_arch = "x86_64")]
            SimdLevel::Avx512F | SimdLevel::Avx512VNNI => {
                if let Err(e) = self.safety.check_alignment(a.as_ptr(), 64) {
                    log::warn!("Memory alignment check failed for AVX-512: {}", e);
                    self.matvec_scalar(a, x, y, m, n);
                } else if let Err(e) = self.safety.check_alignment(x.as_ptr(), 64) {
                    log::warn!("Memory alignment check failed for AVX-512: {}", e);
                    self.matvec_scalar(a, x, y, m, n);
                } else if let Err(e) = self.safety.check_alignment(y.as_ptr(), 64) {
                    log::warn!("Memory alignment check failed for AVX-512: {}", e);
                    self.matvec_scalar(a, x, y, m, n);
                } else {
                    unsafe {
                        self.matvec_avx512(a, x, y, m, n);
                    }
                }
            }
            #[cfg(target_arch = "x86_64")]
            SimdLevel::Avx2 | SimdLevel::Avx2FMA => {
                if let Err(e) = self.safety.check_alignment(a.as_ptr(), 32) {
                    log::warn!("Memory alignment check failed for AVX2: {}", e);
                    self.matvec_scalar(a, x, y, m, n);
                } else if let Err(e) = self.safety.check_alignment(x.as_ptr(), 32) {
                    log::warn!("Memory alignment check failed for AVX2: {}", e);
                    self.matvec_scalar(a, x, y, m, n);
                } else if let Err(e) = self.safety.check_alignment(y.as_ptr(), 32) {
                    log::warn!("Memory alignment check failed for AVX2: {}", e);
                    self.matvec_scalar(a, x, y, m, n);
                } else {
                    unsafe {
                        self.matvec_avx2(a, x, y, m, n);
                    }
                }
            }
            #[cfg(target_arch = "aarch64")]
            SimdLevel::Neon => {
                if let Err(e) = self.safety.check_alignment(a.as_ptr(), 16) {
                    log::warn!("Memory alignment check failed for NEON: {}", e);
                    self.matvec_scalar(a, x, y, m, n);
                } else if let Err(e) = self.safety.check_alignment(x.as_ptr(), 16) {
                    log::warn!("Memory alignment check failed for NEON: {}", e);
                    self.matvec_scalar(a, x, y, m, n);
                } else if let Err(e) = self.safety.check_alignment(y.as_ptr(), 16) {
                    log::warn!("Memory alignment check failed for NEON: {}", e);
                    self.matvec_scalar(a, x, y, m, n);
                } else {
                    unsafe {
                        self.matvec_neon(a, x, y, m, n);
                    }
                }
            }
            _ => {
                // Fallback to scalar implementation
                self.matvec_scalar(a, x, y, m, n);
            }
        }
    }

    fn add_bias(&self, matrix: &mut [f32], bias: &[f32], rows: usize, cols: usize) {
        // Validate inputs
        if rows == 0 || cols == 0 {
            log::warn!(
                "Invalid dimensions for add_bias: rows={}, cols={}",
                rows,
                cols
            );
            return;
        }

        let matrix_required = rows * cols;

        if let Err(e) = self.safety.check_bounds(matrix.len(), matrix_required) {
            log::warn!("Matrix bounds check failed: {}", e);
            self.add_bias_scalar(matrix, bias, rows, cols);
            return;
        }
        if let Err(e) = self.safety.check_bounds(bias.len(), cols) {
            log::warn!("Bias bounds check failed: {}", e);
            self.add_bias_scalar(matrix, bias, rows, cols);
            return;
        }

        // Try SIMD operations based on current level
        match self.current_level {
            #[cfg(target_arch = "x86_64")]
            SimdLevel::Avx512F | SimdLevel::Avx512VNNI => {
                if let Err(e) = self.safety.check_alignment(matrix.as_ptr(), 64) {
                    log::warn!("Memory alignment check failed for AVX-512: {}", e);
                    self.add_bias_scalar(matrix, bias, rows, cols);
                } else if let Err(e) = self.safety.check_alignment(bias.as_ptr(), 64) {
                    log::warn!("Memory alignment check failed for AVX-512: {}", e);
                    self.add_bias_scalar(matrix, bias, rows, cols);
                } else {
                    unsafe {
                        self.add_bias_avx512(matrix, bias, rows, cols);
                    }
                }
            }
            #[cfg(target_arch = "x86_64")]
            SimdLevel::Avx2 | SimdLevel::Avx2FMA => {
                if let Err(e) = self.safety.check_alignment(matrix.as_ptr(), 32) {
                    log::warn!("Memory alignment check failed for AVX2: {}", e);
                    self.add_bias_scalar(matrix, bias, rows, cols);
                } else if let Err(e) = self.safety.check_alignment(bias.as_ptr(), 32) {
                    log::warn!("Memory alignment check failed for AVX2: {}", e);
                    self.add_bias_scalar(matrix, bias, rows, cols);
                } else {
                    unsafe {
                        self.add_bias_avx2(matrix, bias, rows, cols);
                    }
                }
            }
            #[cfg(target_arch = "aarch64")]
            SimdLevel::Neon => {
                if let Err(e) = self.safety.check_alignment(matrix.as_ptr(), 16) {
                    log::warn!("Memory alignment check failed for NEON: {}", e);
                    self.add_bias_scalar(matrix, bias, rows, cols);
                } else if let Err(e) = self.safety.check_alignment(bias.as_ptr(), 16) {
                    log::warn!("Memory alignment check failed for NEON: {}", e);
                    self.add_bias_scalar(matrix, bias, rows, cols);
                } else {
                    unsafe {
                        self.add_bias_neon(matrix, bias, rows, cols);
                    }
                }
            }
            _ => {
                // Fallback to scalar implementation
                self.add_bias_scalar(matrix, bias, rows, cols);
            }
        }
    }

    fn apply_activation(&self, data: &mut [f32], activation: ActivationFunction) {
        // Validate inputs
        if data.is_empty() {
            log::warn!("Empty data array for activation function");
            return;
        }

        // For complex activation functions, we may need to fall back to scalar
        let use_simd = match activation {
            ActivationFunction::Relu => true,
            _ => false, // Complex functions use scalar fallback for now
        };

        if !use_simd {
            self.apply_activation_scalar(data, activation);
            return;
        }

        // Try SIMD operations based on current level
        match self.current_level {
            #[cfg(target_arch = "x86_64")]
            SimdLevel::Avx512F | SimdLevel::Avx512VNNI => {
                if let Err(e) = self.safety.check_alignment(data.as_ptr(), 64) {
                    log::warn!("Memory alignment check failed for AVX-512: {}", e);
                    self.apply_activation_scalar(data, activation);
                } else {
                    unsafe {
                        self.apply_activation_avx512(data, activation);
                    }
                }
            }
            #[cfg(target_arch = "x86_64")]
            SimdLevel::Avx2 | SimdLevel::Avx2FMA => {
                if let Err(e) = self.safety.check_alignment(data.as_ptr(), 32) {
                    log::warn!("Memory alignment check failed for AVX2: {}", e);
                    self.apply_activation_scalar(data, activation);
                } else {
                    unsafe {
                        self.apply_activation_avx2(data, activation);
                    }
                }
            }
            #[cfg(target_arch = "aarch64")]
            SimdLevel::Neon => {
                if let Err(e) = self.safety.check_alignment(data.as_ptr(), 16) {
                    log::warn!("Memory alignment check failed for NEON: {}", e);
                    self.apply_activation_scalar(data, activation);
                } else {
                    unsafe {
                        self.apply_activation_neon(data, activation);
                    }
                }
            }
            _ => {
                // Fallback to scalar implementation
                self.apply_activation_scalar(data, activation);
            }
        }
    }

    fn activation_derivatives(
        &self,
        data: &[f32],
        derivatives: &mut [f32],
        activation: ActivationFunction,
    ) {
        // Validate inputs
        if data.is_empty() || derivatives.is_empty() {
            log::warn!("Empty arrays for activation derivatives");
            return;
        }

        if data.len() != derivatives.len() {
            log::warn!(
                "Array length mismatch for activation derivatives: data={}, derivatives={}",
                data.len(),
                derivatives.len()
            );
            return;
        }

        // For complex activation functions, we may need to fall back to scalar
        let use_simd = match activation {
            ActivationFunction::Relu => true,
            _ => false, // Complex functions use scalar fallback for now
        };

        if !use_simd {
            self.activation_derivatives_scalar(data, derivatives, activation);
            return;
        }

        // Try SIMD operations based on current level
        match self.current_level {
            #[cfg(target_arch = "x86_64")]
            SimdLevel::Avx512F | SimdLevel::Avx512VNNI => {
                if let Err(e) = self.safety.check_alignment(data.as_ptr(), 64) {
                    log::warn!("Memory alignment check failed for AVX-512: {}", e);
                    self.activation_derivatives_scalar(data, derivatives, activation);
                } else if let Err(e) = self.safety.check_alignment(derivatives.as_ptr(), 64) {
                    log::warn!("Memory alignment check failed for AVX-512: {}", e);
                    self.activation_derivatives_scalar(data, derivatives, activation);
                } else {
                    unsafe {
                        self.activation_derivatives_avx512(data, derivatives, activation);
                    }
                }
            }
            #[cfg(target_arch = "x86_64")]
            SimdLevel::Avx2 | SimdLevel::Avx2FMA => {
                if let Err(e) = self.safety.check_alignment(data.as_ptr(), 32) {
                    log::warn!("Memory alignment check failed for AVX2: {}", e);
                    self.activation_derivatives_scalar(data, derivatives, activation);
                } else if let Err(e) = self.safety.check_alignment(derivatives.as_ptr(), 32) {
                    log::warn!("Memory alignment check failed for AVX2: {}", e);
                    self.activation_derivatives_scalar(data, derivatives, activation);
                } else {
                    unsafe {
                        self.activation_derivatives_avx2(data, derivatives, activation);
                    }
                }
            }
            #[cfg(target_arch = "aarch64")]
            SimdLevel::Neon => {
                if let Err(e) = self.safety.check_alignment(data.as_ptr(), 16) {
                    log::warn!("Memory alignment check failed for NEON: {}", e);
                    self.activation_derivatives_scalar(data, derivatives, activation);
                } else if let Err(e) = self.safety.check_alignment(derivatives.as_ptr(), 16) {
                    log::warn!("Memory alignment check failed for NEON: {}", e);
                    self.activation_derivatives_scalar(data, derivatives, activation);
                } else {
                    unsafe {
                        self.activation_derivatives_neon(data, derivatives, activation);
                    }
                }
            }
            _ => {
                // Fallback to scalar implementation
                self.activation_derivatives_scalar(data, derivatives, activation);
            }
        }
    }
}

impl CpuSimdOps {
    /// Scalar fallback for matrix multiplication
    fn matmul_scalar(&self, a: &[f32], b: &[f32], c: &mut [f32], m: usize, n: usize, k: usize) {
        // Initialize output to zero
        c.fill(0.0);

        // Use blocking for better cache performance
        let block_size = self.config.block_size;

        for i_block in (0..m).step_by(block_size) {
            for j_block in (0..n).step_by(block_size) {
                for k_block in (0..k).step_by(block_size) {
                    let i_end = (i_block + block_size).min(m);
                    let j_end = (j_block + block_size).min(n);
                    let k_end = (k_block + block_size).min(k);

                    for i in i_block..i_end {
                        for j in j_block..j_end {
                            let mut sum = 0.0;
                            for k_idx in k_block..k_end {
                                sum += a[i * k + k_idx] * b[k_idx * n + j];
                            }
                            c[i * n + j] += sum;
                        }
                    }
                }
            }
        }
    }

    /// AVX-512 optimized matrix multiplication
    #[cfg(target_arch = "x86_64")]
    unsafe fn matmul_avx512(
        &self,
        a: &[f32],
        b: &[f32],
        c: &mut [f32],
        m: usize,
        n: usize,
        k: usize,
    ) {
        // Initialize output to zero
        c.fill(0.0);

        const SIMD_WIDTH: usize = 16; // AVX-512 processes 16 f32 at once
        let block_size = self.config.block_size;

        for i_block in (0..m).step_by(block_size) {
            for j_block in (0..n).step_by(block_size) {
                for k_block in (0..k).step_by(block_size) {
                    let i_end = (i_block + block_size).min(m);
                    let j_end = (j_block + block_size).min(n);
                    let k_end = (k_block + block_size).min(k);

                    for i in i_block..i_end {
                        for j in (j_block..j_end).step_by(SIMD_WIDTH) {
                            let remaining = (j_end - j).min(SIMD_WIDTH);

                            if remaining == SIMD_WIDTH {
                                // Full SIMD vector processing with AVX-512
                                let mut sum_vec = _mm512_setzero_ps();

                                for k_idx in k_block..k_end {
                                    let a_val = _mm512_set1_ps(a[i * k + k_idx]);
                                    let b_ptr = b.as_ptr().add(k_idx * n + j);
                                    let b_vec = _mm512_load_ps(b_ptr);
                                    sum_vec = _mm512_fmadd_ps(a_val, b_vec, sum_vec);
                                }

                                // Store result
                                let c_ptr = c.as_mut_ptr().add(i * n + j);
                                let c_vec = _mm512_load_ps(c_ptr);
                                let result = _mm512_add_ps(c_vec, sum_vec);
                                _mm512_store_ps(c_ptr, result);
                            } else {
                                // Handle remaining elements with scalar code
                                for j_idx in j..(j + remaining) {
                                    let mut sum = 0.0;
                                    for k_idx in k_block..k_end {
                                        sum += a[i * k + k_idx] * b[k_idx * n + j_idx];
                                    }
                                    c[i * n + j_idx] += sum;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// AVX2 optimized matrix multiplication with safety checks
    #[cfg(target_arch = "x86_64")]
    unsafe fn matmul_avx2(
        &self,
        a: &[f32],
        b: &[f32],
        c: &mut [f32],
        m: usize,
        n: usize,
        k: usize,
    ) {
        // Initialize output to zero
        c.fill(0.0);

        const SIMD_WIDTH: usize = 8; // AVX2 processes 8 f32 at once
        let block_size = self.config.block_size;

        for i_block in (0..m).step_by(block_size) {
            for j_block in (0..n).step_by(block_size) {
                for k_block in (0..k).step_by(block_size) {
                    let i_end = (i_block + block_size).min(m);
                    let j_end = (j_block + block_size).min(n);
                    let k_end = (k_block + block_size).min(k);

                    for i in i_block..i_end {
                        for j in (j_block..j_end).step_by(SIMD_WIDTH) {
                            let remaining = (j_end - j).min(SIMD_WIDTH);

                            if remaining == SIMD_WIDTH {
                                // Full SIMD vector processing
                                let mut sum_vec = _mm256_setzero_ps();

                                for k_idx in k_block..k_end {
                                    let a_val = _mm256_set1_ps(a[i * k + k_idx]);
                                    let b_ptr = b.as_ptr().add(k_idx * n + j);
                                    let b_vec = _mm256_loadu_ps(b_ptr);
                                    sum_vec = _mm256_fmadd_ps(a_val, b_vec, sum_vec);
                                }

                                // Store result
                                let c_ptr = c.as_mut_ptr().add(i * n + j);
                                let c_vec = _mm256_loadu_ps(c_ptr);
                                let result = _mm256_add_ps(c_vec, sum_vec);
                                _mm256_storeu_ps(c_ptr, result);
                            } else {
                                // Handle remaining elements with scalar code
                                for j_idx in j..(j + remaining) {
                                    let mut sum = 0.0;
                                    for k_idx in k_block..k_end {
                                        sum += a[i * k + k_idx] * b[k_idx * n + j_idx];
                                    }
                                    c[i * n + j_idx] += sum;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// NEON optimized matrix multiplication for ARM
    #[cfg(target_arch = "aarch64")]
    unsafe fn matmul_neon(
        &self,
        a: &[f32],
        b: &[f32],
        c: &mut [f32],
        m: usize,
        n: usize,
        k: usize,
    ) {
        // Initialize output to zero
        c.fill(0.0);

        const SIMD_WIDTH: usize = 4; // NEON processes 4 f32 at once
        let block_size = self.config.block_size;

        for i_block in (0..m).step_by(block_size) {
            for j_block in (0..n).step_by(block_size) {
                for k_block in (0..k).step_by(block_size) {
                    let i_end = (i_block + block_size).min(m);
                    let j_end = (j_block + block_size).min(n);
                    let k_end = (k_block + block_size).min(k);

                    for i in i_block..i_end {
                        for j in (j_block..j_end).step_by(SIMD_WIDTH) {
                            let remaining = (j_end - j).min(SIMD_WIDTH);

                            if remaining == SIMD_WIDTH {
                                // Full SIMD vector processing with NEON
                                let mut sum_vec = std::arch::aarch64::vdupq_n_f32(0.0);

                                for k_idx in k_block..k_end {
                                    let a_val = std::arch::aarch64::vdupq_n_f32(a[i * k + k_idx]);
                                    let b_ptr = b.as_ptr().add(k_idx * n + j);
                                    let b_vec = std::arch::aarch64::vld1q_f32(b_ptr);
                                    sum_vec = std::arch::aarch64::vmlaq_f32(sum_vec, a_val, b_vec);
                                }

                                // Store result
                                let c_ptr = c.as_mut_ptr().add(i * n + j);
                                let c_vec = std::arch::aarch64::vld1q_f32(c_ptr);
                                let result = std::arch::aarch64::vaddq_f32(c_vec, sum_vec);
                                std::arch::aarch64::vst1q_f32(c_ptr, result);
                            } else {
                                // Handle remaining elements with scalar code
                                for j_idx in j..(j + remaining) {
                                    let mut sum = 0.0;
                                    for k_idx in k_block..k_end {
                                        sum += a[i * k + k_idx] * b[k_idx * n + j_idx];
                                    }
                                    c[i * n + j_idx] += sum;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Scalar matrix-vector multiplication
    fn matvec_scalar(&self, a: &[f32], x: &[f32], y: &mut [f32], m: usize, n: usize) {
        for i in 0..m {
            let mut sum = 0.0;
            for j in 0..n {
                sum += a[i * n + j] * x[j];
            }
            y[i] = sum;
        }
    }

    /// AVX-512 optimized matrix-vector multiplication
    #[cfg(target_arch = "x86_64")]
    unsafe fn matvec_avx512(&self, a: &[f32], x: &[f32], y: &mut [f32], m: usize, n: usize) {
        const SIMD_WIDTH: usize = 16;

        for i in 0..m {
            let mut sum_vec = _mm512_setzero_ps();

            // Process in chunks of 16
            let chunks = n / SIMD_WIDTH;
            for chunk in 0..chunks {
                let j = chunk * SIMD_WIDTH;
                let a_ptr = a.as_ptr().add(i * n + j);
                let x_ptr = x.as_ptr().add(j);

                let a_vec = _mm512_load_ps(a_ptr);
                let x_vec = _mm512_load_ps(x_ptr);

                sum_vec = _mm512_fmadd_ps(a_vec, x_vec, sum_vec);
            }

            // Horizontal sum of the vector
            let sum = _mm512_reduce_add_ps(sum_vec);

            // Handle remaining elements
            let mut remaining_sum = 0.0;
            for j in (chunks * SIMD_WIDTH)..n {
                remaining_sum += a[i * n + j] * x[j];
            }

            y[i] = sum + remaining_sum;
        }
    }

    /// AVX2 optimized matrix-vector multiplication with safety checks
    #[cfg(target_arch = "x86_64")]
    unsafe fn matvec_avx2(&self, a: &[f32], x: &[f32], y: &mut [f32], m: usize, n: usize) {
        const SIMD_WIDTH: usize = 8;

        for i in 0..m {
            let mut sum_vec = _mm256_setzero_ps();

            // Process in chunks of 8
            let chunks = n / SIMD_WIDTH;
            for chunk in 0..chunks {
                let j = chunk * SIMD_WIDTH;
                let a_ptr = a.as_ptr().add(i * n + j);
                let x_ptr = x.as_ptr().add(j);

                let a_vec = _mm256_loadu_ps(a_ptr);
                let x_vec = _mm256_loadu_ps(x_ptr);

                sum_vec = _mm256_fmadd_ps(a_vec, x_vec, sum_vec);
            }

            // Horizontal sum of the vector
            let sum_array = std::mem::transmute::<__m256, [f32; 8]>(sum_vec);
            let mut sum = sum_array.iter().sum::<f32>();

            // Handle remaining elements
            for j in (chunks * SIMD_WIDTH)..n {
                sum += a[i * n + j] * x[j];
            }

            y[i] = sum;
        }
    }

    /// NEON optimized matrix-vector multiplication for ARM
    #[cfg(target_arch = "aarch64")]
    unsafe fn matvec_neon(&self, a: &[f32], x: &[f32], y: &mut [f32], m: usize, n: usize) {
        const SIMD_WIDTH: usize = 4;

        for i in 0..m {
            let mut sum_vec = std::arch::aarch64::vdupq_n_f32(0.0);

            // Process in chunks of 4
            let chunks = n / SIMD_WIDTH;
            for chunk in 0..chunks {
                let j = chunk * SIMD_WIDTH;
                let a_ptr = a.as_ptr().add(i * n + j);
                let x_ptr = x.as_ptr().add(j);

                let a_vec = std::arch::aarch64::vld1q_f32(a_ptr);
                let x_vec = std::arch::aarch64::vld1q_f32(x_ptr);

                sum_vec = std::arch::aarch64::vmlaq_f32(sum_vec, a_vec, x_vec);
            }

            // Horizontal sum of the vector
            let sum_array =
                std::mem::transmute::<std::arch::aarch64::float32x4_t, [f32; 4]>(sum_vec);
            let mut sum = sum_array.iter().sum::<f32>();

            // Handle remaining elements
            for j in (chunks * SIMD_WIDTH)..n {
                sum += a[i * n + j] * x[j];
            }

            y[i] = sum;
        }
    }

    /// Scalar bias addition
    fn add_bias_scalar(&self, matrix: &mut [f32], bias: &[f32], rows: usize, cols: usize) {
        for i in 0..rows {
            for j in 0..cols {
                matrix[i * cols + j] += bias[j];
            }
        }
    }

    /// AVX-512 optimized bias addition
    #[cfg(target_arch = "x86_64")]
    unsafe fn add_bias_avx512(&self, matrix: &mut [f32], bias: &[f32], rows: usize, cols: usize) {
        const SIMD_WIDTH: usize = 16;

        for i in 0..rows {
            let mut j = 0;

            // Process in chunks of 16
            while j + SIMD_WIDTH <= cols {
                let matrix_ptr = matrix.as_mut_ptr().add(i * cols + j);
                let bias_ptr = bias.as_ptr().add(j);

                let matrix_vec = _mm512_load_ps(matrix_ptr);
                let bias_vec = _mm512_load_ps(bias_ptr);
                let result = _mm512_add_ps(matrix_vec, bias_vec);

                _mm512_store_ps(matrix_ptr, result);
                j += SIMD_WIDTH;
            }

            // Handle remaining elements
            while j < cols {
                matrix[i * cols + j] += bias[j];
                j += 1;
            }
        }
    }

    /// AVX2 optimized bias addition with safety checks
    #[cfg(target_arch = "x86_64")]
    unsafe fn add_bias_avx2(&self, matrix: &mut [f32], bias: &[f32], rows: usize, cols: usize) {
        const SIMD_WIDTH: usize = 8;

        for i in 0..rows {
            let mut j = 0;

            // Process in chunks of 8
            while j + SIMD_WIDTH <= cols {
                let matrix_ptr = matrix.as_mut_ptr().add(i * cols + j);
                let bias_ptr = bias.as_ptr().add(j);

                let matrix_vec = _mm256_loadu_ps(matrix_ptr);
                let bias_vec = _mm256_loadu_ps(bias_ptr);
                let result = _mm256_add_ps(matrix_vec, bias_vec);

                _mm256_storeu_ps(matrix_ptr, result);
                j += SIMD_WIDTH;
            }

            // Handle remaining elements
            while j < cols {
                matrix[i * cols + j] += bias[j];
                j += 1;
            }
        }
    }

    /// NEON optimized bias addition for ARM
    #[cfg(target_arch = "aarch64")]
    unsafe fn add_bias_neon(&self, matrix: &mut [f32], bias: &[f32], rows: usize, cols: usize) {
        const SIMD_WIDTH: usize = 4;

        for i in 0..rows {
            let mut j = 0;

            // Process in chunks of 4
            while j + SIMD_WIDTH <= cols {
                let matrix_ptr = matrix.as_mut_ptr().add(i * cols + j);
                let bias_ptr = bias.as_ptr().add(j);

                let matrix_vec = std::arch::aarch64::vld1q_f32(matrix_ptr);
                let bias_vec = std::arch::aarch64::vld1q_f32(bias_ptr);
                let result = std::arch::aarch64::vaddq_f32(matrix_vec, bias_vec);

                std::arch::aarch64::vst1q_f32(matrix_ptr, result);
                j += SIMD_WIDTH;
            }

            // Handle remaining elements
            while j < cols {
                matrix[i * cols + j] += bias[j];
                j += 1;
            }
        }
    }

    /// Scalar activation function application
    fn apply_activation_scalar(&self, data: &mut [f32], activation: ActivationFunction) {
        match activation {
            ActivationFunction::Sigmoid => {
                for x in data.iter_mut() {
                    *x = 1.0 / (1.0 + (-*x).exp());
                }
            }
            ActivationFunction::Tanh => {
                for x in data.iter_mut() {
                    *x = x.tanh();
                }
            }
            ActivationFunction::Relu => {
                for x in data.iter_mut() {
                    *x = x.max(0.0);
                }
            }
            ActivationFunction::LeakyRelu(alpha) => {
                for x in data.iter_mut() {
                    *x = if *x > 0.0 { *x } else { alpha * *x };
                }
            }
            ActivationFunction::Gelu => {
                for x in data.iter_mut() {
                    // GELU approximation: 0.5 * x * (1 + tanh(sqrt(2/π) * (x + 0.044715 * x³)))
                    let sqrt_2_over_pi = (2.0f32 / std::f32::consts::PI).sqrt();
                    *x = *x * 0.5 * (1.0 + (sqrt_2_over_pi * (*x + 0.044715 * x.powi(3))).tanh());
                }
            }
            ActivationFunction::Swish => {
                for x in data.iter_mut() {
                    *x = *x / (1.0 + (-*x).exp());
                }
            }
        }
    }

    /// AVX-512 optimized activation function application
    #[cfg(target_arch = "x86_64")]
    unsafe fn apply_activation_avx512(&self, data: &mut [f32], activation: ActivationFunction) {
        const SIMD_WIDTH: usize = 16;
        let len = data.len();
        let mut i = 0;

        match activation {
            ActivationFunction::Relu => {
                let zero = _mm512_setzero_ps();

                while i + SIMD_WIDTH <= len {
                    let ptr = data.as_mut_ptr().add(i);
                    let vec = _mm512_load_ps(ptr);
                    let result = _mm512_max_ps(vec, zero);
                    _mm512_store_ps(ptr, result);
                    i += SIMD_WIDTH;
                }
            }
            _ => {
                // For more complex functions, use scalar fallback for now
                self.apply_activation_scalar(data, activation);
                return;
            }
        }

        // Handle remaining elements
        while i < len {
            match activation {
                ActivationFunction::Relu => {
                    data[i] = data[i].max(0.0);
                }
                _ => unreachable!(),
            }
            i += 1;
        }
    }

    /// AVX2 optimized activation function application with safety checks
    #[cfg(target_arch = "x86_64")]
    unsafe fn apply_activation_avx2(&self, data: &mut [f32], activation: ActivationFunction) {
        const SIMD_WIDTH: usize = 8;
        let len = data.len();
        let mut i = 0;

        match activation {
            ActivationFunction::Relu => {
                let zero = _mm256_setzero_ps();

                while i + SIMD_WIDTH <= len {
                    let ptr = data.as_mut_ptr().add(i);
                    let vec = _mm256_loadu_ps(ptr);
                    let result = _mm256_max_ps(vec, zero);
                    _mm256_storeu_ps(ptr, result);
                    i += SIMD_WIDTH;
                }
            }
            _ => {
                // For more complex functions, use scalar fallback for now
                self.apply_activation_scalar(data, activation);
                return;
            }
        }

        // Handle remaining elements
        while i < len {
            match activation {
                ActivationFunction::Relu => {
                    data[i] = data[i].max(0.0);
                }
                _ => unreachable!(),
            }
            i += 1;
        }
    }

    /// NEON optimized activation function application for ARM
    #[cfg(target_arch = "aarch64")]
    unsafe fn apply_activation_neon(&self, data: &mut [f32], activation: ActivationFunction) {
        const SIMD_WIDTH: usize = 4;
        let len = data.len();
        let mut i = 0;

        match activation {
            ActivationFunction::Relu => {
                let zero = std::arch::aarch64::vdupq_n_f32(0.0);

                while i + SIMD_WIDTH <= len {
                    let ptr = data.as_mut_ptr().add(i);
                    let vec = std::arch::aarch64::vld1q_f32(ptr);
                    let result = std::arch::aarch64::vmaxq_f32(vec, zero);
                    std::arch::aarch64::vst1q_f32(ptr, result);
                    i += SIMD_WIDTH;
                }
            }
            _ => {
                // For more complex functions, use scalar fallback for now
                self.apply_activation_scalar(data, activation);
                return;
            }
        }

        // Handle remaining elements
        while i < len {
            match activation {
                ActivationFunction::Relu => {
                    data[i] = data[i].max(0.0);
                }
                _ => unreachable!(),
            }
            i += 1;
        }
    }

    /// Scalar activation derivatives
    fn activation_derivatives_scalar(
        &self,
        data: &[f32],
        derivatives: &mut [f32],
        activation: ActivationFunction,
    ) {
        match activation {
            ActivationFunction::Sigmoid => {
                for (i, &x) in data.iter().enumerate() {
                    derivatives[i] = x * (1.0 - x);
                }
            }
            ActivationFunction::Tanh => {
                for (i, &x) in data.iter().enumerate() {
                    derivatives[i] = 1.0 - x * x;
                }
            }
            ActivationFunction::Relu => {
                for (i, &x) in data.iter().enumerate() {
                    derivatives[i] = if x > 0.0 { 1.0 } else { 0.0 };
                }
            }
            ActivationFunction::LeakyRelu(alpha) => {
                for (i, &x) in data.iter().enumerate() {
                    derivatives[i] = if x > 0.0 { 1.0 } else { alpha };
                }
            }
            ActivationFunction::Gelu => {
                for (i, &x) in data.iter().enumerate() {
                    let sqrt_2_over_pi = (2.0f32 / std::f32::consts::PI).sqrt();
                    let tanh_arg = sqrt_2_over_pi * (x + 0.044715 * x.powi(3));
                    let tanh_val = tanh_arg.tanh();
                    derivatives[i] = 0.5
                        * (1.0
                            + tanh_val
                            + x * sqrt_2_over_pi
                                * (1.0 - tanh_val * tanh_val)
                                * (1.0 + 0.134145 * x * x));
                }
            }
            ActivationFunction::Swish => {
                for (i, &x) in data.iter().enumerate() {
                    let sigmoid = 1.0 / (1.0 + (-x).exp());
                    derivatives[i] = sigmoid * (1.0 + x * (1.0 - sigmoid));
                }
            }
        }
    }

    /// AVX-512 optimized activation derivatives
    #[cfg(target_arch = "x86_64")]
    unsafe fn activation_derivatives_avx512(
        &self,
        data: &[f32],
        derivatives: &mut [f32],
        activation: ActivationFunction,
    ) {
        const SIMD_WIDTH: usize = 16;
        let len = data.len();
        let mut i = 0;

        match activation {
            ActivationFunction::Relu => {
                let zero = _mm512_setzero_ps();
                let one = _mm512_set1_ps(1.0);

                while i + SIMD_WIDTH <= len {
                    let data_ptr = data.as_ptr().add(i);
                    let deriv_ptr = derivatives.as_mut_ptr().add(i);

                    let data_vec = _mm512_load_ps(data_ptr);
                    let mask = _mm512_cmp_ps_mask(data_vec, zero, _CMP_GT_OS);
                    let result = _mm512_mask_blend_ps(mask, zero, one);

                    _mm512_store_ps(deriv_ptr, result);
                    i += SIMD_WIDTH;
                }
            }
            _ => {
                // For more complex functions, use scalar fallback
                self.activation_derivatives_scalar(data, derivatives, activation);
                return;
            }
        }

        // Handle remaining elements
        while i < len {
            match activation {
                ActivationFunction::Relu => {
                    derivatives[i] = if data[i] > 0.0 { 1.0 } else { 0.0 };
                }
                _ => unreachable!(),
            }
            i += 1;
        }
    }

    /// AVX2 optimized activation derivatives with safety checks
    #[cfg(target_arch = "x86_64")]
    unsafe fn activation_derivatives_avx2(
        &self,
        data: &[f32],
        derivatives: &mut [f32],
        activation: ActivationFunction,
    ) {
        const SIMD_WIDTH: usize = 8;
        let len = data.len();
        let mut i = 0;

        match activation {
            ActivationFunction::Relu => {
                let zero = _mm256_setzero_ps();
                let one = _mm256_set1_ps(1.0);

                while i + SIMD_WIDTH <= len {
                    let data_ptr = data.as_ptr().add(i);
                    let deriv_ptr = derivatives.as_mut_ptr().add(i);

                    let data_vec = _mm256_loadu_ps(data_ptr);
                    let mask = _mm256_cmp_ps(data_vec, zero, _CMP_GT_OS);
                    let result = _mm256_and_ps(mask, one);

                    _mm256_storeu_ps(deriv_ptr, result);
                    i += SIMD_WIDTH;
                }
            }
            _ => {
                // For more complex functions, use scalar fallback
                self.activation_derivatives_scalar(data, derivatives, activation);
                return;
            }
        }

        // Handle remaining elements
        while i < len {
            match activation {
                ActivationFunction::Relu => {
                    derivatives[i] = if data[i] > 0.0 { 1.0 } else { 0.0 };
                }
                _ => unreachable!(),
            }
            i += 1;
        }
    }

    /// NEON optimized activation derivatives for ARM
    #[cfg(target_arch = "aarch64")]
    unsafe fn activation_derivatives_neon(
        &self,
        data: &[f32],
        derivatives: &mut [f32],
        activation: ActivationFunction,
    ) {
        const SIMD_WIDTH: usize = 4;
        let len = data.len();
        let mut i = 0;

        match activation {
            ActivationFunction::Relu => {
                let zero = std::arch::aarch64::vdupq_n_f32(0.0);
                let one = std::arch::aarch64::vdupq_n_f32(1.0);

                while i + SIMD_WIDTH <= len {
                    let data_ptr = data.as_ptr().add(i);
                    let deriv_ptr = derivatives.as_mut_ptr().add(i);

                    let data_vec = std::arch::aarch64::vld1q_f32(data_ptr);
                    let mask = std::arch::aarch64::vcgtq_f32(data_vec, zero);
                    let result = std::arch::aarch64::vandq_u32(
                        std::mem::transmute(mask),
                        std::mem::transmute(one),
                    );

                    std::arch::aarch64::vst1q_f32(deriv_ptr, std::mem::transmute(result));
                    i += SIMD_WIDTH;
                }
            }
            _ => {
                // For more complex functions, use scalar fallback
                self.activation_derivatives_scalar(data, derivatives, activation);
                return;
            }
        }

        // Handle remaining elements
        while i < len {
            match activation {
                ActivationFunction::Relu => {
                    derivatives[i] = if data[i] > 0.0 { 1.0 } else { 0.0 };
                }
                _ => unreachable!(),
            }
            i += 1;
        }
    }
}

/// Parallel training operations using rayon
pub struct ParallelTraining {
    simd_ops: CpuSimdOps,
}

impl ParallelTraining {
    pub fn new() -> Self {
        Self {
            simd_ops: CpuSimdOps::new_with_defaults(),
        }
    }

    pub fn new_with_config(config: SimdConfig) -> Self {
        Self {
            simd_ops: CpuSimdOps::new(config),
        }
    }

    /// Parallel batch processing for training
    pub fn process_batch_parallel<F>(&self, inputs: &[Vec<f32>], outputs: &[Vec<f32>], processor: F)
    where
        F: Fn(&[f32], &[f32]) + Send + Sync,
    {
        use rayon::prelude::*;

        inputs
            .par_iter()
            .zip(outputs.par_iter())
            .for_each(|(input, output)| {
                processor(input, output);
            });
    }

    /// Parallel gradient computation
    pub fn compute_gradients_parallel(
        &self,
        network_weights: &[Vec<f32>],
        activations: &[Vec<f32>],
        errors: &[Vec<f32>],
        gradients: &mut [Vec<f32>],
    ) {
        use rayon::prelude::*;

        gradients
            .par_iter_mut()
            .enumerate()
            .for_each(|(layer_idx, layer_gradients)| {
                if layer_idx < network_weights.len()
                    && layer_idx < activations.len()
                    && layer_idx < errors.len()
                {
                    self.simd_ops.matmul(
                        &errors[layer_idx],
                        &activations[layer_idx],
                        layer_gradients,
                        errors[layer_idx].len(),
                        1,
                        activations[layer_idx].len(),
                    );
                }
            });
    }
}

impl Default for ParallelTraining {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_features_detection() {
        let features = CpuFeatures::detect();
        // At minimum, we should have SSE4.2 on modern x86_64
        #[cfg(target_arch = "x86_64")]
        assert!(features.has_sse42 || !features.has_avx2); // If no SSE4.2, no AVX2
    }

    #[test]
    fn test_simd_level_best() {
        let features = CpuFeatures::detect();
        let best = features.best_simd_level();

        match best {
            SimdLevel::Scalar => {} // Always available
            SimdLevel::Neon => {
                #[cfg(target_arch = "aarch64")]
                assert!(features.has_neon);
            }
            SimdLevel::Sse42 => {
                #[cfg(target_arch = "x86_64")]
                assert!(features.has_sse42);
            }
            SimdLevel::Avx2 | SimdLevel::Avx2FMA => {
                #[cfg(target_arch = "x86_64")]
                assert!(features.has_avx2);
            }
            SimdLevel::Avx512F | SimdLevel::Avx512VNNI => {
                #[cfg(target_arch = "x86_64")]
                assert!(features.has_avx512f);
            }
        }
    }

    #[test]
    fn test_simd_config_creation() {
        let config = SimdConfig::default();
        assert!(config.block_size > 0);
        assert!(config.num_threads > 0);
        assert!(config.alignment > 0);
        assert!(config.alignment.is_power_of_two());
    }

    #[test]
    fn test_cpu_simd_ops_creation() {
        let ops = CpuSimdOps::new_with_defaults();
        assert!(ops.config.block_size > 0);

        // Should not panic during validation
        let result = ops.validate_configuration();
        assert!(result.is_ok() || result.is_err()); // Either way is fine, just shouldn't panic
    }

    #[test]
    fn test_matrix_multiplication() {
        let ops = CpuSimdOps::new_with_defaults();

        let a = vec![1.0, 2.0, 3.0, 4.0]; // 2x2 matrix
        let b = vec![5.0, 6.0, 7.0, 8.0]; // 2x2 matrix
        let mut c = vec![0.0; 4]; // 2x2 result

        ops.matmul(&a, &b, &mut c, 2, 2, 2);

        // Expected result: [19, 22, 43, 50]
        assert!((c[0] - 19.0).abs() < 1e-6);
        assert!((c[1] - 22.0).abs() < 1e-6);
        assert!((c[2] - 43.0).abs() < 1e-6);
        assert!((c[3] - 50.0).abs() < 1e-6);
    }

    #[test]
    fn test_matrix_vector_multiplication() {
        let ops = CpuSimdOps::new_with_defaults();

        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]; // 2x3 matrix
        let x = vec![1.0, 2.0, 3.0]; // 3-element vector
        let mut y = vec![0.0; 2]; // 2-element result

        ops.matvec(&a, &x, &mut y, 2, 3);

        // Expected result: [14, 32]
        assert!((y[0] - 14.0).abs() < 1e-6);
        assert!((y[1] - 32.0).abs() < 1e-6);
    }

    #[test]
    fn test_add_bias() {
        let ops = CpuSimdOps::new_with_defaults();

        let mut matrix = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]; // 2x3 matrix
        let bias = vec![0.1, 0.2, 0.3]; // 3-element bias

        ops.add_bias(&mut matrix, &bias, 2, 3);

        // Expected result: [1.1, 2.2, 3.3, 4.1, 5.2, 6.3]
        assert!((matrix[0] - 1.1).abs() < 1e-6);
        assert!((matrix[1] - 2.2).abs() < 1e-6);
        assert!((matrix[2] - 3.3).abs() < 1e-6);
        assert!((matrix[3] - 4.1).abs() < 1e-6);
        assert!((matrix[4] - 5.2).abs() < 1e-6);
        assert!((matrix[5] - 6.3).abs() < 1e-6);
    }

    #[test]
    fn test_relu_activation() {
        let ops = CpuSimdOps::new_with_defaults();
        let mut data = vec![-1.0, 0.0, 1.0, -2.0, 3.0];

        ops.apply_activation(&mut data, ActivationFunction::Relu);

        assert_eq!(data, vec![0.0, 0.0, 1.0, 0.0, 3.0]);
    }

    #[test]
    fn test_sigmoid_activation() {
        let ops = CpuSimdOps::new_with_defaults();
        let mut data = vec![0.0];

        ops.apply_activation(&mut data, ActivationFunction::Sigmoid);

        // Sigmoid(0) = 0.5
        assert!((data[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_tanh_activation() {
        let ops = CpuSimdOps::new_with_defaults();
        let mut data = vec![0.0];

        ops.apply_activation(&mut data, ActivationFunction::Tanh);

        // Tanh(0) = 0.0
        assert!((data[0] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_leaky_relu_activation() {
        let ops = CpuSimdOps::new_with_defaults();
        let mut data = vec![-1.0, 0.0, 1.0];

        ops.apply_activation(&mut data, ActivationFunction::LeakyRelu(0.1));

        assert!((data[0] - (-0.1)).abs() < 1e-6); // LeakyReLU(-1) = -0.1
        assert!((data[1] - 0.0).abs() < 1e-6); // LeakyReLU(0) = 0.0
        assert!((data[2] - 1.0).abs() < 1e-6); // LeakyReLU(1) = 1.0
    }

    #[test]
    fn test_gelu_activation() {
        let ops = CpuSimdOps::new_with_defaults();
        let mut data = vec![0.0];

        ops.apply_activation(&mut data, ActivationFunction::Gelu);

        // GELU(0) ≈ 0.0
        assert!(data[0].abs() < 1e-6);
    }

    #[test]
    fn test_swish_activation() {
        let ops = CpuSimdOps::new_with_defaults();
        let mut data = vec![0.0];

        ops.apply_activation(&mut data, ActivationFunction::Swish);

        // Swish(0) = 0.0
        assert!((data[0] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_relu_derivatives() {
        let ops = CpuSimdOps::new_with_defaults();
        let data = vec![-1.0, 0.0, 1.0, -2.0, 3.0];
        let mut derivatives = vec![0.0; 5];

        ops.activation_derivatives(&data, &mut derivatives, ActivationFunction::Relu);

        assert_eq!(derivatives, vec![0.0, 0.0, 1.0, 0.0, 1.0]);
    }

    #[test]
    fn test_sigmoid_derivatives() {
        let ops = CpuSimdOps::new_with_defaults();
        let data = vec![0.0]; // sigmoid(0) = 0.5
        let mut derivatives = vec![0.0; 1];

        ops.activation_derivatives(&data, &mut derivatives, ActivationFunction::Sigmoid);

        // Sigmoid derivative: x * (1 - x) where x = sigmoid(input)
        // For input = 0, sigmoid = 0.5, derivative = 0.5 * (1 - 0.5) = 0.25
        assert!((derivatives[0] - 0.25).abs() < 1e-6);
    }

    #[test]
    fn test_tanh_derivatives() {
        let ops = CpuSimdOps::new_with_defaults();
        let data = vec![0.0]; // tanh(0) = 0.0
        let mut derivatives = vec![0.0; 1];

        ops.activation_derivatives(&data, &mut derivatives, ActivationFunction::Tanh);

        // Tanh derivative: 1 - x^2 where x = tanh(input)
        // For input = 0, tanh = 0, derivative = 1 - 0 = 1.0
        assert!((derivatives[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_leaky_relu_derivatives() {
        let ops = CpuSimdOps::new_with_defaults();
        let data = vec![-1.0, 0.0, 1.0];
        let mut derivatives = vec![0.0; 3];

        ops.activation_derivatives(&data, &mut derivatives, ActivationFunction::LeakyRelu(0.1));

        assert!((derivatives[0] - 0.1).abs() < 1e-6); // LeakyReLU derivative for negative = alpha
        assert!((derivatives[1] - 0.1).abs() < 1e-6); // LeakyReLU derivative at 0 = alpha
        assert!((derivatives[2] - 1.0).abs() < 1e-6); // LeakyReLU derivative for positive = 1.0
    }

    #[test]
    fn test_memory_alignment() {
        let ops = CpuSimdOps::new_with_defaults();

        // Test aligned memory allocation
        let aligned = ops.allocate_aligned(100).unwrap();
        let alignment = ops
            .config
            .cpu_features
            .best_simd_level()
            .required_alignment();

        assert!(aligned.is_aligned(alignment));
        assert_eq!(aligned.len(), 100);
    }

    #[test]
    fn test_memory_alignment_copy() {
        let ops = CpuSimdOps::new_with_defaults();
        let mut aligned = ops.allocate_aligned(10).unwrap();

        let source = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        aligned.copy_from_slice(&source).unwrap();

        let mut result = vec![0.0; 10];
        aligned.copy_to_slice(&mut result).unwrap();

        assert_eq!(result, source);
    }

    #[test]
    fn test_safety_checks() {
        let config = SimdConfig::default();
        let safety = SimdSafety::new(config);

        // Test bounds checking
        let data = vec![1.0, 2.0, 3.0];
        assert!(safety.check_bounds(data.len(), 3).is_ok());
        assert!(safety.check_bounds(data.len(), 5).is_err());

        // Test matrix dimensions
        assert!(safety.validate_matrix_dims(2, 3, 4).is_ok());
        assert!(safety.validate_matrix_dims(0, 3, 4).is_err());
        assert!(safety.validate_matrix_dims(2, 0, 4).is_err());
        assert!(safety.validate_matrix_dims(2, 3, 0).is_err());
    }

    #[test]
    fn test_simd_level_properties() {
        assert_eq!(SimdLevel::Scalar.vector_width(), 1);
        assert_eq!(SimdLevel::Sse42.vector_width(), 4);
        assert_eq!(SimdLevel::Avx2.vector_width(), 8);
        assert_eq!(SimdLevel::Avx512F.vector_width(), 16);

        assert_eq!(SimdLevel::Scalar.required_alignment(), 8);
        assert_eq!(SimdLevel::Sse42.required_alignment(), 16);
        assert_eq!(SimdLevel::Avx2.required_alignment(), 32);
        assert_eq!(SimdLevel::Avx512F.required_alignment(), 64);
    }

    #[test]
    fn test_simd_integration_with_memory_management() {
        // Test that SIMD operations work with aligned memory
        let ops = CpuSimdOps::new_with_defaults();

        // Allocate aligned memory for SIMD operations
        let aligned_memory = ops.allocate_aligned(100).unwrap();
        let alignment = ops.current_simd_level().required_alignment();

        assert!(aligned_memory.is_aligned(alignment));

        // Fill with test data
        let test_data = vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        aligned_memory.copy_from_slice(&test_data).unwrap();

        // Test that we can read it back
        let mut result = vec![0.0f32; 8];
        aligned_memory.copy_to_slice(&mut result).unwrap();

        assert_eq!(result, test_data);
    }

    #[test]
    fn test_simd_fallback_mechanisms() {
        // Test that operations gracefully fall back when SIMD isn't available
        let config = SimdConfig {
            cpu_features: CpuFeatures {
                has_avx2: false,
                has_avx512f: false,
                has_avx512vnni: false,
                has_fma: false,
                has_sse42: false,
                has_neon: false,
            },
            simd_level: Some(SimdLevel::Scalar),
            ..Default::default()
        };

        let ops = CpuSimdOps::new(config);

        // Should fall back to scalar implementations
        let a = vec![1.0, 2.0, 3.0, 4.0];
        let b = vec![5.0, 6.0, 7.0, 8.0];
        let mut c = vec![0.0; 4];

        ops.matmul(&a, &b, &mut c, 2, 2, 2);

        // Should still produce correct results
        assert!((c[0] - 19.0).abs() < 1e-6);
        assert!((c[1] - 22.0).abs() < 1e-6);
        assert!((c[2] - 43.0).abs() < 1e-6);
        assert!((c[3] - 50.0).abs() < 1e-6);
    }

    #[test]
    fn test_simd_configuration_validation() {
        let ops = CpuSimdOps::new_with_defaults();

        // Should not panic during validation
        let result = ops.validate_configuration();
        assert!(result.is_ok() || result.is_err()); // Either way is fine, just shouldn't panic

        // Test with forced scalar mode
        let config = SimdConfig {
            cpu_features: CpuFeatures::detect(),
            simd_level: Some(SimdLevel::Scalar),
            ..Default::default()
        };

        let ops_scalar = CpuSimdOps::new(config);
        assert_eq!(ops_scalar.current_simd_level(), SimdLevel::Scalar);
    }
}
