//! Vector similarity and distance computation functions for HNSW search.
//!
//! Extracted from the monolithic `core.rs` for better maintainability (PERF-05).

use crate::hardware::{HardwareCapabilities, InstructionSet};
use crate::node::{DistanceMetric, VectorRepresentations};
use crate::vector::quantization::{rabitq_similarity, turbo_quant_similarity};

use super::MAX_VEC_F32_LEN;

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
            *<&[f32; 8]>::try_from(a_chunk).expect("chunks_exact(8) yields 8-element chunks"),
        );
        let vb = f32x8::from(
            *<&[f32; 8]>::try_from(b_chunk).expect("chunks_exact(8) yields 8-element chunks"),
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
            *<&[f32; 8]>::try_from(a_chunk).expect("chunks_exact(8) yields 8-element chunks"),
        );
        let vb = f32x8::from(
            *<&[f32; 8]>::try_from(b_chunk).expect("chunks_exact(8) yields 8-element chunks"),
        );
        dot_v += va * vb;
    }
    let mut dot = dot_v.reduce_add();
    for i in 0..rem_a.len() {
        dot += rem_a[i] * rem_b[i];
    }
    dot
}

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
            *<&[f32; 8]>::try_from(a_chunk).expect("chunks_exact(8) yields 8-element chunks"),
        );
        let vb = f32x8::from(
            *<&[f32; 8]>::try_from(b_chunk).expect("chunks_exact(8) yields 8-element chunks"),
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
            *<&[f32; 16]>::try_from(a_chunk).expect("chunks_exact(16) yields 16-element chunks"),
        );
        let vb = f32x16::from(
            *<&[f32; 16]>::try_from(b_chunk).expect("chunks_exact(16) yields 16-element chunks"),
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
            *<&[f32; 16]>::try_from(a_chunk).expect("chunks_exact(16) yields 16-element chunks"),
        );
        let vb = f32x16::from(
            *<&[f32; 16]>::try_from(b_chunk).expect("chunks_exact(16) yields 16-element chunks"),
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
            *<&[f32; 16]>::try_from(a_chunk).expect("chunks_exact(16) yields 16-element chunks"),
        );
        let vb = f32x16::from(
            *<&[f32; 16]>::try_from(b_chunk).expect("chunks_exact(16) yields 16-element chunks"),
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

// ---------------------------------------------------------------------------
// PERF-21: Runtime dispatch wrappers
// ---------------------------------------------------------------------------

/// Compute squared Euclidean distance between two f32 vectors.
/// Runtime dispatch: Avx512 → f32x16, Avx2/Neon → f32x8, Fallback → scalar.
#[inline(always)]
pub fn euclidean_distance_squared_f32(a: &[f32], b: &[f32]) -> f32 {
    match HardwareCapabilities::global().instructions {
        InstructionSet::Avx512 => euclidean_distance_sq_f32x16(a, b),
        _ => euclidean_distance_sq_f32x8(a, b),
    }
}

/// Pure dot product — no norm computation.
/// Runtime dispatch: Avx512 → f32x16, Avx2/Neon → f32x8, Fallback → scalar.
#[inline(always)]
fn f32_dot_product(a: &[f32], b: &[f32]) -> f32 {
    match HardwareCapabilities::global().instructions {
        InstructionSet::Avx512 => f32_dot_product_f32x16(a, b),
        _ => f32_dot_product_f32x8(a, b),
    }
}

/// Precomputed dot product + squared norm of `b`. Returns `(dot, norm_b_sq)`.
/// Runtime dispatch: Avx512 → f32x16, Avx2/Neon → f32x8, Fallback → scalar.
#[inline(always)]
fn f32_dot_and_norm_b_sq(a: &[f32], b: &[f32]) -> (f32, f32) {
    match HardwareCapabilities::global().instructions {
        InstructionSet::Avx512 => f32_dot_and_norm_b_sq_f32x16(a, b),
        _ => f32_dot_and_norm_b_sq_f32x8(a, b),
    }
}

/// Compute similarity against a raw query when SQ8 is the only available
/// representation for the stored node. Decodes on the fly.
///
/// PERF-22: SIMD-ized with f32x8 (avoids 3-way scalar loop overhead per element).
#[inline(always)]
fn sq8_similarity(
    raw_query: &[f32],
    sq8_data: &[i8],
    sq8_scale: f32,
    metric: DistanceMetric,
    _query_norm: Option<f32>,
) -> f32 {
    let inv_scale = sq8_scale / 127.0;
    match metric {
        DistanceMetric::Cosine => {
            use wide::f32x8;
            let mut dot_v = f32x8::ZERO;
            let mut norm_q_v = f32x8::ZERO;
            let mut norm_sq_v = f32x8::ZERO;
            let chunks_q = raw_query.chunks_exact(8);
            let chunks_s = sq8_data.chunks_exact(8);
            let rem_q = chunks_q.remainder();
            let rem_s = chunks_s.remainder();
            for (q_chunk, s_chunk) in chunks_q.zip(chunks_s) {
                let vq = f32x8::from(
                    *<&[f32; 8]>::try_from(q_chunk).expect("chunks_exact(8) yields 8-element chunks"),
                );
                let decoded = [
                    (s_chunk[0] as f32) * inv_scale,
                    (s_chunk[1] as f32) * inv_scale,
                    (s_chunk[2] as f32) * inv_scale,
                    (s_chunk[3] as f32) * inv_scale,
                    (s_chunk[4] as f32) * inv_scale,
                    (s_chunk[5] as f32) * inv_scale,
                    (s_chunk[6] as f32) * inv_scale,
                    (s_chunk[7] as f32) * inv_scale,
                ];
                let vs = f32x8::from(decoded);
                dot_v += vq * vs;
                norm_q_v += vq * vq;
                norm_sq_v += vs * vs;
            }
            let mut dot = dot_v.reduce_add();
            let mut norm_q = norm_q_v.reduce_add();
            let mut norm_sq = norm_sq_v.reduce_add();
            for i in 0..rem_q.len() {
                let decoded = (rem_s[i] as f32) * inv_scale;
                dot += rem_q[i] * decoded;
                norm_q += rem_q[i] * rem_q[i];
                norm_sq += decoded * decoded;
            }
            if norm_q <= f32::EPSILON || norm_sq <= f32::EPSILON {
                return 0.0;
            }
            dot / (norm_q.sqrt() * norm_sq.sqrt())
        }
        DistanceMetric::Euclidean => {
            use wide::f32x8;
            let mut sum_sq_v = f32x8::ZERO;
            let chunks_q = raw_query.chunks_exact(8);
            let chunks_s = sq8_data.chunks_exact(8);
            let rem_q = chunks_q.remainder();
            let rem_s = chunks_s.remainder();
            for (q_chunk, s_chunk) in chunks_q.zip(chunks_s) {
                let vq = f32x8::from(
                    *<&[f32; 8]>::try_from(q_chunk).expect("chunks_exact(8) yields 8-element chunks"),
                );
                let decoded = [
                    (s_chunk[0] as f32) * inv_scale,
                    (s_chunk[1] as f32) * inv_scale,
                    (s_chunk[2] as f32) * inv_scale,
                    (s_chunk[3] as f32) * inv_scale,
                    (s_chunk[4] as f32) * inv_scale,
                    (s_chunk[5] as f32) * inv_scale,
                    (s_chunk[6] as f32) * inv_scale,
                    (s_chunk[7] as f32) * inv_scale,
                ];
                let vs = f32x8::from(decoded);
                let diff = vq - vs;
                sum_sq_v += diff * diff;
            }
            let mut sum_sq = sum_sq_v.reduce_add();
            for i in 0..rem_q.len() {
                let diff = rem_q[i] - (rem_s[i] as f32) * inv_scale;
                sum_sq += diff * diff;
            }
            -sum_sq
        }
    }
}

/// Compute similarity between a raw query and a node's stored vector representation.
pub fn calculate_similarity(
    raw_query: &[f32],
    query_norm: Option<f32>,
    quantized_query_1bit: Option<&[u64]>,
    quantized_query_3bit: Option<(&[u8], f32)>,
    node_vec: &VectorRepresentations,
    metric: DistanceMetric,
) -> f32 {
    match node_vec {
        VectorRepresentations::Binary(b) => {
            if let Some(q1) = quantized_query_1bit {
                rabitq_similarity(q1, b)
            } else {
                0.0
            }
        }
        VectorRepresentations::Turbo(t) => {
            if let Some((q3, max_abs)) = quantized_query_3bit {
                turbo_quant_similarity(q3, max_abs, t, 1.0)
            } else {
                0.0
            }
        }
        VectorRepresentations::SQ8(data, scale) => {
            sq8_similarity(raw_query, data, *scale, metric, query_norm)
        }
        VectorRepresentations::Full(f) => match metric {
            DistanceMetric::Cosine => match query_norm {
                Some(norm) => cosine_sim_with_query_norm(raw_query, norm, f),
                None => cosine_sim_f32(raw_query, f),
            },
            DistanceMetric::Euclidean => -euclidean_distance_squared_f32(raw_query, f),
        },
        VectorRepresentations::MmapFull(ptr, len) => {
            debug_assert!(
                !ptr.0.is_null(),
                "MmapFull pointer is null in compute_similarity"
            );
            debug_assert!(
                *len > 0 && *len <= MAX_VEC_F32_LEN,
                "MmapFull len out of range in compute_similarity"
            );
            let slice = unsafe { std::slice::from_raw_parts(ptr.0, *len) };
            match metric {
                DistanceMetric::Cosine => match query_norm {
                    Some(norm) => cosine_sim_with_query_norm(raw_query, norm, slice),
                    None => cosine_sim_f32(raw_query, slice),
                },
                DistanceMetric::Euclidean => -euclidean_distance_squared_f32(raw_query, slice),
            }
        }
        VectorRepresentations::None => 0.0,
    }
}

#[inline(always)]
pub(crate) fn f32_slice_similarity(
    query_vec: &[f32],
    query_norm: Option<f32>,
    candidate: &[f32],
    metric: DistanceMetric,
) -> f32 {
    match metric {
        DistanceMetric::Cosine => match query_norm {
            Some(norm) => cosine_sim_with_query_norm(query_vec, norm, candidate),
            None => cosine_sim_f32(query_vec, candidate),
        },
        DistanceMetric::Euclidean => -euclidean_distance_squared_f32(query_vec, candidate),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_euclidean_similarity_is_higher_for_closer() {
        let q = vec![0.0, 0.0];
        let close = vec![1.0, 0.0];
        let far = vec![10.0, 10.0];
        let score_close = calculate_similarity(
            &q,
            None,
            None,
            None,
            &VectorRepresentations::Full(close),
            DistanceMetric::Euclidean,
        );
        let score_far = calculate_similarity(
            &q,
            None,
            None,
            None,
            &VectorRepresentations::Full(far),
            DistanceMetric::Euclidean,
        );
        assert!(
            score_close > score_far,
            "Euclidean similarity must be higher for closer vectors: {} <= {}",
            score_close,
            score_far
        );
        assert!(
            score_close <= 0.0,
            "Euclidean similarity must be <= 0 for non-zero distance: {}",
            score_close
        );
    }

    #[test]
    fn test_cosine_similarity_is_higher_for_closer() {
        let q = vec![1.0, 0.0, 0.0];
        let close = vec![0.9, 0.1, 0.0];
        let far = vec![-1.0, 0.0, 0.0];
        let score_close = calculate_similarity(
            &q,
            None,
            None,
            None,
            &VectorRepresentations::Full(close),
            DistanceMetric::Cosine,
        );
        let score_far = calculate_similarity(
            &q,
            None,
            None,
            None,
            &VectorRepresentations::Full(far),
            DistanceMetric::Cosine,
        );
        assert!(
            score_close > score_far,
            "Cosine similarity must be higher for closer vectors: {} <= {}",
            score_close,
            score_far
        );
    }

    #[test]
    fn test_euclidean_identical_vectors_score_zero() {
        let v = vec![3.0, 4.0, 5.0];
        let score = calculate_similarity(
            &v,
            None,
            None,
            None,
            &VectorRepresentations::Full(v.clone()),
            DistanceMetric::Euclidean,
        );
        assert!(
            (score - 0.0).abs() < 1e-6,
            "Euclidean score for identical vectors should be 0, got {}",
            score
        );
    }

    #[test]
    fn test_search_nearest_euclidean_returns_closest_first() {
        use crate::index::CPIndex;
        use crate::index::HnswConfig;
        use crate::node::FilterBitset;
        let config = HnswConfig {
            m: 8,
            m_max0: 16,
            ef_construction: 50,
            ef_search: 50,
            ml: 1.0 / (8_f64).ln(),
            distance_metric: DistanceMetric::Euclidean,
        };
        let index = CPIndex::new_with_config(config);
        index.add(
            1,
            FilterBitset::all_set(),
            VectorRepresentations::Full(vec![0.0, 0.0]),
            0,
        );
        index.add(
            2,
            FilterBitset::all_set(),
            VectorRepresentations::Full(vec![1.0, 0.0]),
            0,
        );
        index.add(
            3,
            FilterBitset::all_set(),
            VectorRepresentations::Full(vec![10.0, 10.0]),
            0,
        );
        let query = vec![0.0, 0.0];
        let results = index.search_nearest(&query, None, None, &FilterBitset::all_set(), 3, None);
        assert_eq!(results.len(), 3);
        assert_eq!(
            results[0].0, 1,
            "Closest (id=1, distance 1) should be first, got id={}",
            results[0].0
        );
        assert_eq!(
            results[2].0, 3,
            "Farthest (id=3, distance ~14.14) should be last, got id={}",
            results[2].0
        );
        for &(_, score) in &results {
            assert!(!score.is_nan(), "Score should not be NaN");
        }
        assert!(
            results[0].1 > results[1].1,
            "Scores must be descending (higher=better): {} <= {}",
            results[0].1,
            results[1].1
        );
        assert!(
            results[1].1 > results[2].1,
            "Scores must be descending (higher=better): {} <= {}",
            results[1].1,
            results[2].1
        );
    }
}
