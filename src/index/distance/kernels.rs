//! SIMD kernels and runtime multiversion dispatch for distance metrics.
//!
//! PERF-38: kernel function pointers are cached once so CPU detection + match
//! overhead happens only at init time. Split out of the monolithic `distance`
//! module (REVIEW-05).

use std::sync::OnceLock;

use crate::hardware::{HardwareCapabilities, InstructionSet};

pub(crate) type KernelEuclideanSq = fn(&[f32], &[f32]) -> f32;
pub(crate) type KernelDotProduct = fn(&[f32], &[f32]) -> f32;
pub(crate) type KernelDotAndNorm = fn(&[f32], &[f32]) -> (f32, f32);

pub(crate) struct DistanceKernels {
    pub(crate) euclidean_sq: KernelEuclideanSq,
    pub(crate) dot_product: KernelDotProduct,
    pub(crate) dot_and_norm_b_sq: KernelDotAndNorm,
}

static KERNELS: OnceLock<DistanceKernels> = OnceLock::new();

pub(crate) fn select_kernels() -> &'static DistanceKernels {
    KERNELS.get_or_init(|| match HardwareCapabilities::global().instructions {
        InstructionSet::Avx512 => DistanceKernels {
            euclidean_sq: euclidean_distance_sq_f32x16,
            dot_product: f32_dot_product_f32x16,
            dot_and_norm_b_sq: f32_dot_and_norm_b_sq_f32x16,
        },
        _ => DistanceKernels {
            euclidean_sq: euclidean_distance_sq_f32x8,
            dot_product: f32_dot_product_f32x8,
            dot_and_norm_b_sq: f32_dot_and_norm_b_sq_f32x8,
        },
    })
}

/// Precomputed dot product + squared norm of `b`. Returns `(dot, norm_b_sq)`.
/// f32x8 kernel (AVX2 / NEON / scalar fallback).
#[inline(always)]
fn f32_dot_and_norm_b_sq_f32x8(a: &[f32], b: &[f32]) -> (f32, f32) {
    if a.len() != b.len() || a.is_empty() {
        return (0.0, 0.0);
    }
    use wide::f32x8;
    let mut dot_v = f32x8::ZERO;
    let mut norm_b_v = f32x8::ZERO;
    let chunks_a = a.chunks_exact(8);
    let chunks_b = b.chunks_exact(8);
    let rem_a = chunks_a.remainder();
    let rem_b = chunks_b.remainder();
    for (a_chunk, b_chunk) in chunks_a.zip(chunks_b) {
        let va = f32x8::from(
            // SAFETY: chunks_exact(8) guarantees chunk.len() == 8
            *unsafe { <&[f32; 8]>::try_from(a_chunk).unwrap_unchecked() },
        );
        let vb = f32x8::from(
            // SAFETY: chunks_exact(8) guarantees chunk.len() == 8
            *unsafe { <&[f32; 8]>::try_from(b_chunk).unwrap_unchecked() },
        );
        dot_v += va * vb;
        norm_b_v += vb * vb;
    }
    let mut dot = dot_v.reduce_add();
    let mut norm_b = norm_b_v.reduce_add();
    for i in 0..rem_a.len() {
        dot += rem_a[i] * rem_b[i];
        norm_b += rem_b[i] * rem_b[i];
    }
    (dot, norm_b)
}

/// Pure dot product — no norm computation. ~2x faster than `f32_dot_and_norm_b_sq`
/// when norms are already cached. f32x8 kernel (AVX2 / NEON / scalar fallback).
#[inline(always)]
fn f32_dot_product_f32x8(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    use wide::f32x8;
    let mut dot_v = f32x8::ZERO;
    let chunks_a = a.chunks_exact(8);
    let chunks_b = b.chunks_exact(8);
    let rem_a = chunks_a.remainder();
    let rem_b = chunks_b.remainder();
    for (a_chunk, b_chunk) in chunks_a.zip(chunks_b) {
        let va = f32x8::from(
            // SAFETY: chunks_exact(8) guarantees chunk.len() == 8
            *unsafe { <&[f32; 8]>::try_from(a_chunk).unwrap_unchecked() },
        );
        let vb = f32x8::from(
            // SAFETY: chunks_exact(8) guarantees chunk.len() == 8
            *unsafe { <&[f32; 8]>::try_from(b_chunk).unwrap_unchecked() },
        );
        dot_v += va * vb;
    }
    let mut dot = dot_v.reduce_add();
    for i in 0..rem_a.len() {
        dot += rem_a[i] * rem_b[i];
    }
    dot
}

/// f32x8 kernel for squared Euclidean distance (AVX2 / NEON / scalar fallback).
#[inline(always)]
fn euclidean_distance_sq_f32x8(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    use wide::f32x8;
    let mut sum_v = f32x8::ZERO;
    let chunks_a = a.chunks_exact(8);
    let chunks_b = b.chunks_exact(8);
    let rem_a = chunks_a.remainder();
    let rem_b = chunks_b.remainder();
    for (a_chunk, b_chunk) in chunks_a.zip(chunks_b) {
        let va = f32x8::from(
            // SAFETY: chunks_exact(8) guarantees chunk.len() == 8
            *unsafe { <&[f32; 8]>::try_from(a_chunk).unwrap_unchecked() },
        );
        let vb = f32x8::from(
            // SAFETY: chunks_exact(8) guarantees chunk.len() == 8
            *unsafe { <&[f32; 8]>::try_from(b_chunk).unwrap_unchecked() },
        );
        let diff = va - vb;
        sum_v += diff * diff;
    }
    let mut sum = sum_v.reduce_add();
    for i in 0..rem_a.len() {
        let diff = rem_a[i] - rem_b[i];
        sum += diff * diff;
    }
    sum
}

// ---------------------------------------------------------------------------
// PERF-21: f32x16 kernels (AVX-512)
// ---------------------------------------------------------------------------

/// Squared Euclidean distance using f32x16 (AVX-512).
#[inline(always)]
fn euclidean_distance_sq_f32x16(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    use wide::f32x16;
    let mut sum_v = f32x16::ZERO;
    let chunks_a = a.chunks_exact(16);
    let chunks_b = b.chunks_exact(16);
    let rem_a = chunks_a.remainder();
    let rem_b = chunks_b.remainder();
    for (a_chunk, b_chunk) in chunks_a.zip(chunks_b) {
        let va = f32x16::from(
            // SAFETY: chunks_exact(16) guarantees chunk.len() == 16
            *unsafe { <&[f32; 16]>::try_from(a_chunk).unwrap_unchecked() },
        );
        let vb = f32x16::from(
            // SAFETY: chunks_exact(16) guarantees chunk.len() == 16
            *unsafe { <&[f32; 16]>::try_from(b_chunk).unwrap_unchecked() },
        );
        let diff = va - vb;
        sum_v += diff * diff;
    }
    let mut sum = sum_v.reduce_add();
    for i in 0..rem_a.len() {
        let diff = rem_a[i] - rem_b[i];
        sum += diff * diff;
    }
    sum
}

/// Dot product using f32x16 (AVX-512).
#[inline(always)]
fn f32_dot_product_f32x16(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    use wide::f32x16;
    let mut dot_v = f32x16::ZERO;
    let chunks_a = a.chunks_exact(16);
    let chunks_b = b.chunks_exact(16);
    let rem_a = chunks_a.remainder();
    let rem_b = chunks_b.remainder();
    for (a_chunk, b_chunk) in chunks_a.zip(chunks_b) {
        let va = f32x16::from(
            // SAFETY: chunks_exact(16) guarantees chunk.len() == 16
            *unsafe { <&[f32; 16]>::try_from(a_chunk).unwrap_unchecked() },
        );
        let vb = f32x16::from(
            // SAFETY: chunks_exact(16) guarantees chunk.len() == 16
            *unsafe { <&[f32; 16]>::try_from(b_chunk).unwrap_unchecked() },
        );
        dot_v += va * vb;
    }
    let mut dot = dot_v.reduce_add();
    for i in 0..rem_a.len() {
        dot += rem_a[i] * rem_b[i];
    }
    dot
}

/// Combined dot + norm of `b` using f32x16 (AVX-512).
#[inline(always)]
fn f32_dot_and_norm_b_sq_f32x16(a: &[f32], b: &[f32]) -> (f32, f32) {
    if a.len() != b.len() || a.is_empty() {
        return (0.0, 0.0);
    }
    use wide::f32x16;
    let mut dot_v = f32x16::ZERO;
    let mut norm_b_v = f32x16::ZERO;
    let chunks_a = a.chunks_exact(16);
    let chunks_b = b.chunks_exact(16);
    let rem_a = chunks_a.remainder();
    let rem_b = chunks_b.remainder();
    for (a_chunk, b_chunk) in chunks_a.zip(chunks_b) {
        let va = f32x16::from(
            // SAFETY: chunks_exact(16) guarantees chunk.len() == 16
            *unsafe { <&[f32; 16]>::try_from(a_chunk).unwrap_unchecked() },
        );
        let vb = f32x16::from(
            // SAFETY: chunks_exact(16) guarantees chunk.len() == 16
            *unsafe { <&[f32; 16]>::try_from(b_chunk).unwrap_unchecked() },
        );
        dot_v += va * vb;
        norm_b_v += vb * vb;
    }
    let mut dot = dot_v.reduce_add();
    let mut norm_b = norm_b_v.reduce_add();
    for i in 0..rem_a.len() {
        dot += rem_a[i] * rem_b[i];
        norm_b += rem_b[i] * rem_b[i];
    }
    (dot, norm_b)
}

/// Pure dot product — no norm computation.
/// Cached dispatch: function pointer selected once at init.
#[inline(always)]
pub(crate) fn f32_dot_product(a: &[f32], b: &[f32]) -> f32 {
    (select_kernels().dot_product)(a, b)
}

/// Precomputed dot product + squared norm of `b`. Returns `(dot, norm_b_sq)`.
/// Cached dispatch: function pointer selected once at init.
#[inline(always)]
pub(crate) fn f32_dot_and_norm_b_sq(a: &[f32], b: &[f32]) -> (f32, f32) {
    (select_kernels().dot_and_norm_b_sq)(a, b)
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    // ── f32x16 kernels (AVX-512 path) ────────────────────────────────────

    fn vec16(val: f32) -> Vec<f32> {
        vec![val; 16]
    }

    fn vec32(val: f32) -> Vec<f32> {
        vec![val; 32]
    }

    #[test]
    fn test_euclidean_distance_sq_f32x16_identical() {
        let v = vec32(3.0);
        let d = euclidean_distance_sq_f32x16(&v, &v);
        assert!(d.abs() < 1e-6, "identical f32x16 should be 0, got {}", d);
    }

    #[test]
    fn test_euclidean_distance_sq_f32x16_known() {
        let a = vec![0.0_f32; 16];
        let mut b = vec![0.0_f32; 16];
        b[0] = 3.0;
        b[1] = 4.0;
        let d = euclidean_distance_sq_f32x16(&a, &b);
        assert!((d - 25.0).abs() < 1e-5, "expected 25, got {}", d);
    }

    #[test]
    fn test_euclidean_distance_sq_f32x16_mismatched() {
        let a = vec32(1.0);
        let b = vec16(1.0);
        let d = euclidean_distance_sq_f32x16(&a, &b);
        assert!(d.abs() < 1e-6, "mismatched should return 0, got {}", d);
    }

    #[test]
    fn test_euclidean_distance_sq_f32x16_empty() {
        let d = euclidean_distance_sq_f32x16(&[], &[]);
        assert!(d.abs() < 1e-6, "empty should return 0, got {}", d);
    }

    #[test]
    fn test_euclidean_distance_sq_f32x16_multi_chunk() {
        let a = vec![0.0_f32; 32];
        let mut b = vec![0.0_f32; 32];
        b[0] = 3.0;
        b[16] = 4.0;
        let d = euclidean_distance_sq_f32x16(&a, &b);
        assert!((d - 25.0).abs() < 1e-5, "expected 25, got {}", d);
    }

    #[test]
    fn test_f32_dot_product_f32x16_known() {
        let a = vec16(1.0);
        let b = vec16(2.0);
        let dot = f32_dot_product_f32x16(&a, &b);
        assert!((dot - 32.0).abs() < 1e-5, "expected 32, got {}", dot);
    }

    #[test]
    fn test_f32_dot_product_f32x16_mismatched() {
        let a = vec32(1.0);
        let b = vec16(1.0);
        let dot = f32_dot_product_f32x16(&a, &b);
        assert!(dot.abs() < 1e-6, "mismatched should return 0, got {}", dot);
    }

    #[test]
    fn test_f32_dot_product_f32x16_empty() {
        let dot = f32_dot_product_f32x16(&[], &[]);
        assert!(dot.abs() < 1e-6, "empty should return 0, got {}", dot);
    }

    #[test]
    fn test_f32_dot_and_norm_b_sq_f32x16_known() {
        let a = vec16(2.0);
        let b = vec16(3.0);
        let (dot, norm_b_sq) = f32_dot_and_norm_b_sq_f32x16(&a, &b);
        assert!((dot - 96.0).abs() < 1e-5, "expected dot=96, got {}", dot);
        assert!(
            (norm_b_sq - 144.0).abs() < 1e-5,
            "expected norm_b_sq=144, got {}",
            norm_b_sq
        );
    }

    #[test]
    fn test_f32_dot_and_norm_b_sq_f32x16_mismatched() {
        let a = vec32(2.0);
        let b = vec16(3.0);
        let (dot, norm_b_sq) = f32_dot_and_norm_b_sq_f32x16(&a, &b);
        assert!(dot.abs() < 1e-6, "mismatched dot should be 0");
        assert!(norm_b_sq.abs() < 1e-6, "mismatched norm should be 0");
    }

    #[cfg(miri)]
    #[test]
    fn miri_distance_f32x8_kernels() {
        // These sizes exercise: empty (no loop), sub-chunk (no loop),
        // exact-chunk (full SIMD), and multi-chunk paths.
        let test_sizes: &[usize] = &[0, 1, 7, 8, 9, 15, 16, 32, 100];
        for &size in test_sizes {
            let a: Vec<f32> = (0..size).map(|i| i as f32).collect();
            let b: Vec<f32> = (0..size).map(|i| (i as f32) * 2.0 + 1.0).collect();

            // Equal-length calls exercise the chunks_exact(8) loop
            let d1 = euclidean_distance_sq_f32x8(&a, &b);
            let d2 = f32_dot_product_f32x8(&a, &b);
            let (dot, norm) = f32_dot_and_norm_b_sq_f32x8(&a, &b);

            assert!(d1.is_finite(), "euclidean_sq_f32x8(size={})", size);
            assert!(d2.is_finite(), "dot_product_f32x8(size={})", size);
            assert!(dot.is_finite(), "dot_f32x8(size={})", size);
            assert!(norm.is_finite(), "norm_f32x8(size={})", size);

            // Mismatched-length: early-return path, no unsafe executed
            if size >= 2 {
                let short = &a[..size / 2];
                let _ = euclidean_distance_sq_f32x8(&a, short);
                let _ = f32_dot_product_f32x8(&a, short);
                let _ = f32_dot_and_norm_b_sq_f32x8(&a, short);
            }
        }
    }

    #[cfg(miri)]
    #[test]
    fn miri_distance_f32x16_kernels() {
        let test_sizes: &[usize] = &[0, 1, 15, 16, 17, 31, 32, 64, 100];
        for &size in test_sizes {
            let a: Vec<f32> = (0..size).map(|i| i as f32).collect();
            let b: Vec<f32> = (0..size).map(|i| (i as f32) * 2.0 + 1.0).collect();

            let d1 = euclidean_distance_sq_f32x16(&a, &b);
            let d2 = f32_dot_product_f32x16(&a, &b);
            let (dot, norm) = f32_dot_and_norm_b_sq_f32x16(&a, &b);

            assert!(d1.is_finite(), "euclidean_sq_f32x16(size={})", size);
            assert!(d2.is_finite(), "dot_product_f32x16(size={})", size);
            assert!(dot.is_finite(), "dot_f32x16(size={})", size);
            assert!(norm.is_finite(), "norm_f32x16(size={})", size);
        }
    }
}
