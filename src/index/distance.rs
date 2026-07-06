//! Vector similarity and distance computation functions for HNSW search.
//!
//! Extracted from the monolithic `core.rs` for better maintainability (PERF-05).

use crate::node::{DistanceMetric, VectorRepresentations};
use crate::vector::quantization::{rabitq_similarity, turbo_quant_similarity};

use super::MAX_VEC_F32_LEN;

/// Precomputed dot product + squared norm of `b`. Returns `(dot, norm_b_sq)`.
#[inline(always)]
fn f32_dot_and_norm_b_sq(a: &[f32], b: &[f32]) -> (f32, f32) {
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
/// when norms are already cached.
#[inline(always)]
fn f32_dot_product(a: &[f32], b: &[f32]) -> f32 {
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

/// Compute squared Euclidean distance between two f32 vectors.
#[inline(always)]
pub fn euclidean_distance_squared_f32(a: &[f32], b: &[f32]) -> f32 {
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

/// Compute similarity against a raw query when SQ8 is the only available
/// representation for the stored node. Decodes on the fly.
fn sq8_similarity_fallback(
    raw_query: &[f32],
    sq8_data: &[i8],
    sq8_scale: f32,
    metric: DistanceMetric,
    _query_norm: Option<f32>,
) -> f32 {
    let inv_scale = sq8_scale / 127.0;
    match metric {
        DistanceMetric::Cosine => {
            let mut dot = 0.0_f32;
            let mut norm_q = 0.0_f32;
            for (&q, &s) in raw_query.iter().zip(sq8_data.iter()) {
                let decoded = (s as f32) * inv_scale;
                dot += q * decoded;
                norm_q += q * q;
            }
            let norm_sq = sq8_data.iter().fold(0.0_f32, |acc, &s| {
                let d = (s as f32) * inv_scale;
                acc + d * d
            });
            if norm_q <= f32::EPSILON || norm_sq <= f32::EPSILON {
                return 0.0;
            }
            dot / (norm_q.sqrt() * norm_sq.sqrt())
        }
        DistanceMetric::Euclidean => {
            let mut sum_sq = 0.0_f32;
            for (&q, &s) in raw_query.iter().zip(sq8_data.iter()) {
                let diff = q - (s as f32) * inv_scale;
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
            sq8_similarity_fallback(raw_query, data, *scale, metric, query_norm)
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
