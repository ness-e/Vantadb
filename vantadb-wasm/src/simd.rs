/// Compute cosine similarity between two f32 vectors.
///
/// Uses WASM SIMD 128-bit (f32x4) when the `simd128` target feature is enabled,
/// processing 4 floats at a time. Falls back to scalar when SIMD is unavailable.
///
/// Returns a value in `[-1.0, 1.0]` where `1.0` means identical direction,
/// `0.0` means orthogonal, and `-1.0` means opposite direction.
/// Zero vectors return `1.0` to avoid division by zero.
///
/// # Panics
/// Panics if `a` and `b` have different lengths.
pub fn cosine_distance_simd(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "vectors must have equal length");

    #[cfg(target_feature = "simd128")]
    {
        simd_impl(a, b)
    }

    #[cfg(not(target_feature = "simd128"))]
    {
        scalar_impl(a, b)
    }
}

#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
fn simd_impl(a: &[f32], b: &[f32]) -> f32 {
    use core::arch::wasm32::*;

    let len = a.len();
    let simd_len = len / 4 * 4;
    let mut i = 0;

    unsafe {
        let mut dot = f32x4_splat(0.0);
        let mut na = f32x4_splat(0.0);
        let mut nb = f32x4_splat(0.0);

        while i < simd_len {
            let va = v128_load(a.as_ptr().add(i) as *const v128);
            let vb = v128_load(b.as_ptr().add(i) as *const v128);

            dot = f32x4_add(dot, f32x4_mul(va, vb));
            na = f32x4_add(na, f32x4_mul(va, va));
            nb = f32x4_add(nb, f32x4_mul(vb, vb));

            i += 4;
        }

        let mut dot_sum = f32x4_extract_lane::<0>(dot)
            + f32x4_extract_lane::<1>(dot)
            + f32x4_extract_lane::<2>(dot)
            + f32x4_extract_lane::<3>(dot);

        let mut na_sum = f32x4_extract_lane::<0>(na)
            + f32x4_extract_lane::<1>(na)
            + f32x4_extract_lane::<2>(na)
            + f32x4_extract_lane::<3>(na);

        let mut nb_sum = f32x4_extract_lane::<0>(nb)
            + f32x4_extract_lane::<1>(nb)
            + f32x4_extract_lane::<2>(nb)
            + f32x4_extract_lane::<3>(nb);

        for j in i..len {
            dot_sum += a[j] * b[j];
            na_sum += a[j] * a[j];
            nb_sum += b[j] * b[j];
        }

        let mag = (na_sum * nb_sum).sqrt();
        if mag == 0.0 {
            1.0
        } else {
            dot_sum / mag
        }
    }
}

fn scalar_impl(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut dot = 0.0f32;
    let mut na = 0.0f32;
    let mut nb = 0.0f32;

    for j in 0..len {
        dot += a[j] * b[j];
        na += a[j] * a[j];
        nb += b[j] * b[j];
    }

    let mag = (na * nb).sqrt();
    if mag == 0.0 {
        1.0
    } else {
        dot / mag
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_vectors() {
        let a = [1.0, 0.0, 0.0, 0.0];
        assert!((cosine_distance_simd(&a, &a) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_orthogonal_vectors() {
        let a = [1.0, 0.0, 0.0, 0.0];
        let b = [0.0, 1.0, 0.0, 0.0];
        assert!(cosine_distance_simd(&a, &b).abs() < 1e-6);
    }

    #[test]
    fn test_opposite_vectors() {
        let a = [1.0, 0.0, 0.0, 0.0];
        let b = [-1.0, 0.0, 0.0, 0.0];
        assert!((cosine_distance_simd(&a, &b) + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_zero_vectors() {
        let a = [0.0, 0.0, 0.0, 0.0];
        assert!((cosine_distance_simd(&a, &a) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_subset_aligned() {
        let a = [2.0, 3.0, 0.0, 0.0];
        let b = [1.0, 1.5, 0.0, 0.0];
        assert!((cosine_distance_simd(&a, &b) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_non_simd_aligned_len() {
        let a = [1.0, 2.0, 3.0];
        let b = [4.0, 5.0, 6.0];
        let r = cosine_distance_simd(&a, &b);
        assert!(!r.is_nan() && r >= -1.0 && r <= 1.0);
    }

    #[test]
    fn test_seven_elements() {
        let a = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        let b = [7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0];
        let r = cosine_distance_simd(&a, &b);
        assert!(!r.is_nan() && r >= -1.0 && r <= 1.0);
    }

    #[test]
    fn test_large_vectors() {
        let a: Vec<f32> = (0..256).map(|i| i as f32).collect();
        let b: Vec<f32> = (0..256).map(|i| (255 - i) as f32).collect();
        let r = cosine_distance_simd(&a, &b);
        assert!(!r.is_nan() && r >= -1.0 && r <= 1.0);
    }

    #[test]
    fn test_partial_identical() {
        let a = [1.0, 2.0, 3.0, 4.0];
        let b = [2.0, 4.0, 6.0, 8.0];
        assert!((cosine_distance_simd(&a, &b) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_single_element() {
        let a = [3.0];
        let b = [6.0];
        assert!((cosine_distance_simd(&a, &b) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_two_elements() {
        let a = [1.0, 0.0];
        let b = [1.0, 1.0];
        let expected = 1.0 / 2.0f32.sqrt();
        assert!((cosine_distance_simd(&a, &b) - expected).abs() < 1e-6);
    }

    #[test]
    fn test_range_bounds() {
        let a: Vec<f32> = (0..128).map(|i| i as f32 * 0.5).collect();
        let b: Vec<f32> = (0..128).map(|i| i as f32 * 0.3 + 1.0).collect();
        let r = cosine_distance_simd(&a, &b);
        assert!(r >= -1.0 && r <= 1.0);
    }

    #[test]
    #[should_panic(expected = "vectors must have equal length")]
    fn test_mismatched_lengths() {
        let a = [1.0, 2.0, 3.0];
        let b = [1.0, 2.0];
        cosine_distance_simd(&a, &b);
    }
}
