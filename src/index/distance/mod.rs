//! Distance metrics, SIMD kernels, and similarity dispatch.
//!
//! Split into three concerns (REVIEW-05):
//! - [`kernels`]: SIMD f32x8/f32x16 kernels + runtime multiversion dispatch.
//! - [`metrics`]: public metric helpers (L2 norm, cosine, Euclidean).
//! - [`mapper`]: metric mapping and similarity dispatch over stored vectors.

mod kernels;
mod mapper;
mod metrics;

pub use mapper::*;
pub use metrics::*;

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    #[cfg(miri)]
    #[test]
    fn miri_distance_public_dispatch_paths() {
        // Exercise the public wrappers that dispatch via function pointers.
        // This tests the entire call chain including select_kernels() init,
        // which involves OnceLock + HardwareCapabilities::global().
        let test_sizes: &[usize] = &[0, 1, 7, 8, 15, 16, 32, 100];
        for &size in test_sizes {
            let a: Vec<f32> = (0..size).map(|i| i as f32).collect();
            let b: Vec<f32> = (0..size).map(|i| (i as f32) * -1.0).collect();

            let d = super::euclidean_distance_squared_f32(&a, &b);
            assert!(
                d.is_finite(),
                "euclidean_distance_squared_f32(size={})",
                size
            );

            let norm = super::f32_l2_norm(&a);
            assert!(norm.is_finite(), "f32_l2_norm(size={})", size);

            let cos = super::cosine_sim_f32(&a, &b);
            assert!(cos.is_finite(), "cosine_sim_f32(size={})", size);

            if !a.is_empty() && norm > f32::EPSILON {
                let inv = 1.0 / norm;
                let bn = super::f32_l2_norm(&b);
                let binv = if bn > f32::EPSILON { 1.0 / bn } else { 0.0 };
                let cached = super::cosine_sim_cached_norms(&a, inv, &b, binv);
                assert!(cached.is_finite(), "cosine_sim_cached_norms(size={})", size);
            }

            let fs = super::f32_slice_similarity(&a, None, &b, DistanceMetric::Cosine);
            assert!(fs.is_finite(), "f32_slice_similarity cosine(size={})", size);

            let fs2 = super::f32_slice_similarity(&a, None, &b, DistanceMetric::Euclidean);
            assert!(
                fs2.is_finite(),
                "f32_slice_similarity euclidean(size={})",
                size
            );
        }
    }
}
