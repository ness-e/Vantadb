//! Public metric helpers: L2 norm, cosine similarity, and Euclidean distance.
//!
//! These operate on dense f32 vectors and dispatch to the SIMD kernels in
//! `kernels`. Split out of the monolithic `distance` module (REVIEW-05).

use super::kernels::{f32_dot_and_norm_b_sq, f32_dot_product, select_kernels};

/// Compute the L2 norm of a f32 vector.
#[inline(always)]
pub fn f32_l2_norm(v: &[f32]) -> f32 {
    if v.is_empty() {
        return 0.0;
    }
    let (_, norm_sq) = f32_dot_and_norm_b_sq(v, v);
    norm_sq.sqrt()
}

/// Cosine similarity when BOTH inverse norms are pre-cached. Uses pure dot product
/// and multiplications only — eliminates 100% of division and ~50% of SIMD work.
#[inline(always)]
pub fn cosine_sim_cached_norms(a: &[f32], inv_norm_a: f32, b: &[f32], inv_norm_b: f32) -> f32 {
    if inv_norm_a < f32::EPSILON || inv_norm_b < f32::EPSILON || a.len() != b.len() || a.is_empty()
    {
        return 0.0;
    }
    let dot = f32_dot_product(a, b);
    dot * inv_norm_a * inv_norm_b
}

/// Cosine similarity when `||query||` was already computed for the search hot path.
#[inline(always)]
pub fn cosine_sim_with_query_norm(query: &[f32], query_norm: f32, b: &[f32]) -> f32 {
    if query_norm < f32::EPSILON || query.len() != b.len() || query.is_empty() {
        return 0.0;
    }
    let (dot, norm_b_sq) = f32_dot_and_norm_b_sq(query, b);
    let norm_b = norm_b_sq.sqrt();
    if norm_b < f32::EPSILON {
        0.0
    } else {
        dot / (query_norm * norm_b)
    }
}

/// Compute cosine similarity between two f32 vectors without cached norms.
#[inline(always)]
pub fn cosine_sim_f32(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let norm_a = f32_l2_norm(a);
    cosine_sim_with_query_norm(a, norm_a, b)
}

/// Compute squared Euclidean distance between two f32 vectors.
/// Cached dispatch: function pointer selected once at init.
#[inline(always)]
pub fn euclidean_distance_squared_f32(a: &[f32], b: &[f32]) -> f32 {
    (select_kernels().euclidean_sq)(a, b)
}

/// Compute squared Euclidean distance using cached norms to skip the
/// per-element subtraction kernel.
/// euclidean_sq = ||a||² + ||b||² - 2·dot(a,b)
#[inline(always)]
pub fn euclidean_distance_sq_with_norms(
    a: &[f32],
    a_norm_sq: f32,
    b: &[f32],
    b_norm_sq: f32,
) -> f32 {
    let dot = f32_dot_product(a, b);
    // AUDREP-28: for nearly-identical vectors FP rounding can make
    // ||a||² + ||b||² - 2·dot slightly negative. Clamp so negative noise
    // never corrupts HNSW neighbor selection.
    (a_norm_sq + b_norm_sq - 2.0 * dot).max(0.0)
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    #[test]
    fn test_euclidean_distance_sq_with_norms_never_negative() {
        // AUDREP-28: identical vectors must never yield a negative squared
        // distance even under FP rounding of ||a||² + ||b||² - 2·dot.
        let v = vec![1.0_f32; 128];
        let norm_sq = f32_dot_product(&v, &v);
        let d = euclidean_distance_sq_with_norms(&v, norm_sq, &v, norm_sq);
        assert!(
            d >= 0.0,
            "squared distance for identical vectors must be >= 0, got {}",
            d
        );
    }

    // ── Unit tests for distance computation functions ────────────────

    #[test]
    fn test_euclidean_distance_squared_identical() {
        let v = vec![3.0, 4.0, 5.0];
        let d = euclidean_distance_squared_f32(&v, &v);
        assert!(
            d.abs() < 1e-6,
            "identical vectors should have distance 0, got {}",
            d
        );
    }

    #[test]
    fn test_euclidean_distance_squared_known() {
        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];
        let d = euclidean_distance_squared_f32(&a, &b);
        assert!(
            (d - 25.0).abs() < 1e-5,
            "distance between (0,0) and (3,4) should be 25, got {}",
            d
        );
    }

    #[test]
    fn test_euclidean_distance_squared_empty() {
        let d = euclidean_distance_squared_f32(&[], &[]);
        assert!(
            d.abs() < 1e-6,
            "empty vectors should have distance 0, got {}",
            d
        );
    }

    #[test]
    fn test_euclidean_distance_squared_mismatched_length() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0];
        let d = euclidean_distance_squared_f32(&a, &b);
        assert!(
            d.abs() < 1e-6,
            "mismatched lengths should return 0, got {}",
            d
        );
    }

    #[test]
    fn test_euclidean_distance_sq_with_norms_matches_direct() {
        let a = vec![2.0, 3.0, 4.0];
        let b = vec![1.0, -1.0, 2.0];
        let direct = euclidean_distance_squared_f32(&a, &b);
        let a_norm_sq = f32_l2_norm(&a);
        let a_norm_sq = a_norm_sq * a_norm_sq;
        let b_norm_sq = f32_l2_norm(&b);
        let b_norm_sq = b_norm_sq * b_norm_sq;
        let with_norms = euclidean_distance_sq_with_norms(&a, a_norm_sq, &b, b_norm_sq);
        assert!(
            (direct - with_norms).abs() < 1e-5,
            "norms-based formula should match direct: direct={}, with_norms={}",
            direct,
            with_norms
        );
    }

    #[test]
    fn test_f32_l2_norm_known() {
        let v = vec![3.0, 4.0];
        let n = f32_l2_norm(&v);
        assert!(
            (n - 5.0).abs() < 1e-6,
            "norm of (3,4) should be 5, got {}",
            n
        );
    }

    #[test]
    fn test_f32_l2_norm_zero() {
        let v = vec![0.0, 0.0, 0.0];
        let n = f32_l2_norm(&v);
        assert!(n.abs() < 1e-6, "norm of zero vector should be 0, got {}", n);
    }

    #[test]
    fn test_f32_l2_norm_empty() {
        let n = f32_l2_norm(&[]);
        assert!(
            n.abs() < 1e-6,
            "norm of empty vector should be 0, got {}",
            n
        );
    }

    #[test]
    fn test_cosine_sim_parallel() {
        let a = vec![2.0, 0.0, 0.0];
        let b = vec![5.0, 0.0, 0.0];
        let sim = cosine_sim_f32(&a, &b);
        assert!(
            (sim - 1.0).abs() < 1e-6,
            "parallel vectors should have cosine ~1.0, got {}",
            sim
        );
    }

    #[test]
    fn test_cosine_sim_opposite() {
        let a = vec![1.0, 0.0];
        let b = vec![-1.0, 0.0];
        let sim = cosine_sim_f32(&a, &b);
        assert!(
            (sim - (-1.0)).abs() < 1e-6,
            "opposite vectors should have cosine ~-1.0, got {}",
            sim
        );
    }

    #[test]
    fn test_cosine_sim_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let sim = cosine_sim_f32(&a, &b);
        assert!(
            sim.abs() < 1e-6,
            "orthogonal vectors should have cosine ~0.0, got {}",
            sim
        );
    }

    #[test]
    fn test_cosine_sim_zero_norm() {
        let a = vec![0.0, 0.0];
        let b = vec![1.0, 0.0];
        let sim = cosine_sim_f32(&a, &b);
        assert!(
            sim.abs() < 1e-6,
            "cosine with zero-norm query should return 0, got {}",
            sim
        );
    }

    #[test]
    fn test_cosine_sim_mismatched_length() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0];
        let sim = cosine_sim_f32(&a, &b);
        assert!(
            sim.abs() < 1e-6,
            "mismatched lengths should return 0, got {}",
            sim
        );
    }

    #[test]
    fn test_cosine_sim_empty() {
        let sim = cosine_sim_f32(&[], &[]);
        assert!(
            sim.abs() < 1e-6,
            "empty vectors should return 0, got {}",
            sim
        );
    }

    #[test]
    fn test_cosine_sim_cached_norms_matches_direct() {
        let a = vec![3.0, 4.0, 0.0];
        let b = vec![1.0, 2.0, 3.0];
        let direct = cosine_sim_f32(&a, &b);
        let inv_norm_a = 1.0 / f32_l2_norm(&a);
        let inv_norm_b = 1.0 / f32_l2_norm(&b);
        let cached = cosine_sim_cached_norms(&a, inv_norm_a, &b, inv_norm_b);
        assert!(
            (direct - cached).abs() < 1e-5,
            "cached norms path should match direct: direct={}, cached={}",
            direct,
            cached
        );
    }

    #[test]
    fn test_cosine_sim_cached_norms_zero_inv_norm() {
        let a = vec![0.0, 0.0];
        let b = vec![1.0, 2.0];
        let sim = cosine_sim_cached_norms(&a, 0.0, &b, 0.5);
        assert!(
            sim.abs() < 1e-6,
            "zero inv_norm should return 0, got {}",
            sim
        );
    }

    #[test]
    fn test_cosine_sim_with_query_norm_basic() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.5, 0.5, 0.0];
        let norm_a = f32_l2_norm(&a);
        let sim = cosine_sim_with_query_norm(&a, norm_a, &b);
        let expected = cosine_sim_f32(&a, &b);
        assert!(
            (sim - expected).abs() < 1e-6,
            "query-norm path should match: {} vs {}",
            sim,
            expected
        );
    }

    #[test]
    fn test_cosine_sim_with_query_norm_zero_norm() {
        let a = vec![0.0, 0.0];
        let b = vec![1.0, 0.0];
        let sim = cosine_sim_with_query_norm(&a, 0.0, &b);
        assert!(
            sim.abs() < 1e-6,
            "zero query norm should return 0, got {}",
            sim
        );
    }

    // ── Edge cases ───────────────────────────────────────────────────────

    #[test]
    fn test_euclidean_distance_sq_with_norms_known() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        let a_norm_sq = 1.0 + 4.0 + 9.0; // 14
        let b_norm_sq = 16.0 + 25.0 + 36.0; // 77
                                            // euclidean_sq = a_norm_sq + b_norm_sq - 2*dot = 14 + 77 - 2*(4+10+18) = 91 - 64 = 27
        let d = euclidean_distance_sq_with_norms(&a, a_norm_sq, &b, b_norm_sq);
        assert!((d - 27.0).abs() < 1e-5, "expected 27, got {}", d);
    }

    #[test]
    fn test_cosine_sim_cached_norms_zero_target_norm() {
        let a = vec![1.0, 2.0];
        let b = vec![1.0, 2.0];
        let sim = cosine_sim_cached_norms(&a, 0.5, &b, 0.0);
        assert!(
            sim.abs() < 1e-6,
            "zero target inv_norm should return 0, got {}",
            sim
        );
    }

    #[test]
    fn test_cosine_sim_many_dims() {
        let a: Vec<f32> = (0..128).map(|i| (i as f32).sin()).collect();
        let b: Vec<f32> = (0..128).map(|i| (i as f32).cos()).collect();
        let sim = cosine_sim_f32(&a, &b);
        assert!(sim.is_finite(), "similarity should be finite");
        assert!(
            (-1.0..=1.0).contains(&sim),
            "similarity should be in [-1, 1], got {}",
            sim
        );
    }

    #[test]
    fn test_euclidean_distance_sq_many_dims() {
        let a: Vec<f32> = (0..100).map(|i| i as f32).collect();
        let b: Vec<f32> = (0..100).map(|i| (i as f32) * 2.0).collect();
        let d = euclidean_distance_squared_f32(&a, &b);
        assert!(d > 0.0, "distance should be positive, got {}", d);
        assert!(d.is_finite(), "distance should be finite");
    }

    #[test]
    fn test_cosine_sim_negative_values() {
        let a = vec![-1.0, -2.0, -3.0];
        let b = vec![1.0, 2.0, 3.0];
        let sim = cosine_sim_f32(&a, &b);
        assert!(
            (sim - (-1.0)).abs() < 1e-6,
            "opposite vectors should give -1, got {}",
            sim
        );
    }

    #[test]
    fn test_cosine_sim_with_query_norm_zero_target() {
        let q = vec![1.0, 0.0];
        let b = vec![0.0, 0.0];
        let sim = cosine_sim_with_query_norm(&q, 1.0, &b);
        assert!(
            sim.abs() < 1e-6,
            "zero-norm target should return 0, got {}",
            sim
        );
    }

    #[test]
    fn test_f32_l2_norm_single_element() {
        let v = vec![-3.0];
        let n = f32_l2_norm(&v);
        assert!(
            (n - 3.0).abs() < 1e-6,
            "norm of [-3] should be 3, got {}",
            n
        );
    }
}
