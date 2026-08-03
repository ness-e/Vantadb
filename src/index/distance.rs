//! Vector similarity and distance computation functions for HNSW search.
//!
//! Extracted from the monolithic `core.rs` for better maintainability (PERF-05).

use std::sync::OnceLock;

use crate::hardware::{HardwareCapabilities, InstructionSet};
use crate::node::{DistanceMetric, VectorRepresentations};
use crate::vector::quantization::{rabitq_similarity, turbo_quant_similarity};

use super::MAX_VEC_F32_LEN;

// ---------------------------------------------------------------------------
// PERF-38: Runtime multiversion dispatch — kernel function pointers cached
// once so CPU detection + match overhead happens only at init time.
// ---------------------------------------------------------------------------

type KernelEuclideanSq = fn(&[f32], &[f32]) -> f32;
type KernelDotProduct = fn(&[f32], &[f32]) -> f32;
type KernelDotAndNorm = fn(&[f32], &[f32]) -> (f32, f32);

struct DistanceKernels {
    euclidean_sq: KernelEuclideanSq,
    dot_product: KernelDotProduct,
    dot_and_norm_b_sq: KernelDotAndNorm,
}

static KERNELS: OnceLock<DistanceKernels> = OnceLock::new();

fn select_kernels() -> &'static DistanceKernels {
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

// ---------------------------------------------------------------------------
// PERF-29: Cosine ↔ Euclidean mapping cache
//
// For normalized vectors: euclidean_sq = 2 * (1 - cosine).
// MetricCache stores the precomputed conversion factor once.
// ---------------------------------------------------------------------------

/// Cached conversion between cosine similarity and Euclidean squared distance.
pub struct MetricMapper;

impl MetricMapper {
    /// Convert cosine similarity to squared Euclidean distance.
    /// Valid when both vectors are L2-normalized (||a|| = ||b|| = 1).
    #[inline(always)]
    pub fn cosine_to_euclidean_sq(cosine: f32) -> f32 {
        metric_cache().factor * (1.0 - cosine)
    }

    /// Convert cosine similarity to negative Euclidean distance (higher = closer).
    #[inline(always)]
    pub fn cosine_to_euclidean_similarity(cosine: f32) -> f32 {
        -Self::cosine_to_euclidean_sq(cosine)
    }
}

/// Precomputed conversion factor populated once at startup.
struct MetricCache {
    factor: f32,
}

static METRIC_CACHE: OnceLock<MetricCache> = OnceLock::new();

fn metric_cache() -> &'static MetricCache {
    METRIC_CACHE.get_or_init(|| MetricCache { factor: 2.0 })
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

// ---------------------------------------------------------------------------
// PERF-21: Runtime dispatch wrappers
// ---------------------------------------------------------------------------

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
    a_norm_sq + b_norm_sq - 2.0 * dot
}

/// Pure dot product — no norm computation.
/// Cached dispatch: function pointer selected once at init.
#[inline(always)]
fn f32_dot_product(a: &[f32], b: &[f32]) -> f32 {
    (select_kernels().dot_product)(a, b)
}

/// Precomputed dot product + squared norm of `b`. Returns `(dot, norm_b_sq)`.
/// Cached dispatch: function pointer selected once at init.
#[inline(always)]
fn f32_dot_and_norm_b_sq(a: &[f32], b: &[f32]) -> (f32, f32) {
    (select_kernels().dot_and_norm_b_sq)(a, b)
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
                    // SAFETY: chunks_exact(8) guarantees chunk.len() == 8
                    *unsafe { <&[f32; 8]>::try_from(q_chunk).unwrap_unchecked() },
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
                    // SAFETY: chunks_exact(8) guarantees chunk.len() == 8
                    *unsafe { <&[f32; 8]>::try_from(q_chunk).unwrap_unchecked() },
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
        // Sparse search uses its own brute-force path; dense helpers never
        // receive a SparseDot metric.
        DistanceMetric::SparseDot => 0.0,
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
            // Sparse search has its own brute-force path over sparse vectors;
            // dense helpers never receive a SparseDot metric.
            DistanceMetric::SparseDot => 0.0,
        },
        VectorRepresentations::MmapFull(mmap_opt) => {
            let mmap = match mmap_opt {
                Some(m) => m,
                None => return 0.0,
            };
            let len = mmap.len() / 4;
            if len == 0 || len > MAX_VEC_F32_LEN {
                return 0.0;
            }
            // SAFETY: len bounded by MAX_VEC_F32_LEN; mmap kept alive by Arc.
            let slice = unsafe { std::slice::from_raw_parts(mmap.as_ptr() as *const f32, len) };
            match metric {
                DistanceMetric::Cosine => match query_norm {
                    Some(norm) => cosine_sim_with_query_norm(raw_query, norm, slice),
                    None => cosine_sim_f32(raw_query, slice),
                },
                DistanceMetric::Euclidean => -euclidean_distance_squared_f32(raw_query, slice),
                DistanceMetric::SparseDot => 0.0,
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
        DistanceMetric::SparseDot => 0.0,
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
            ..HnswConfig::default()
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

    #[test]
    fn test_metric_mapper_cosine_to_euclidean_sq() {
        // euclidean_sq = 2 * (1 - cosine)
        assert!(
            (MetricMapper::cosine_to_euclidean_sq(1.0) - 0.0).abs() < 1e-6,
            "cosine=1 → euclidean_sq=0"
        );
        assert!(
            (MetricMapper::cosine_to_euclidean_sq(0.0) - 2.0).abs() < 1e-6,
            "cosine=0 → euclidean_sq=2"
        );
        assert!(
            (MetricMapper::cosine_to_euclidean_sq(-1.0) - 4.0).abs() < 1e-6,
            "cosine=-1 → euclidean_sq=4"
        );
        assert!(
            (MetricMapper::cosine_to_euclidean_sq(0.5) - 1.0).abs() < 1e-6,
            "cosine=0.5 → euclidean_sq=1"
        );
    }

    #[test]
    fn test_calculate_similarity_full_cosine() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let sim = calculate_similarity(
            &a,
            None,
            None,
            None,
            &VectorRepresentations::Full(b),
            DistanceMetric::Cosine,
        );
        assert!(
            (sim - 1.0).abs() < 1e-6,
            "identical vectors cosine should be 1, got {}",
            sim
        );
    }

    #[test]
    fn test_calculate_similarity_full_euclidean() {
        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];
        let sim = calculate_similarity(
            &a,
            None,
            None,
            None,
            &VectorRepresentations::Full(b),
            DistanceMetric::Euclidean,
        );
        // Euclidean similarity = -squared_distance → -(9 + 16) = -25
        assert!(
            (sim - (-25.0)).abs() < 1e-5,
            "euclidean similarity should be -25, got {}",
            sim
        );
    }

    #[test]
    fn test_calculate_similarity_none() {
        let a = vec![1.0, 2.0];
        let sim = calculate_similarity(
            &a,
            None,
            None,
            None,
            &VectorRepresentations::None,
            DistanceMetric::Cosine,
        );
        assert!(
            sim.abs() < 1e-6,
            "None representation should return 0, got {}",
            sim
        );
    }

    #[test]
    fn test_f32_slice_similarity_cosine() {
        let q = vec![1.0, 0.0, 0.0];
        let c = vec![0.9, 0.1, 0.0];
        let sim = f32_slice_similarity(&q, None, &c, DistanceMetric::Cosine);
        assert!(
            sim > 0.9,
            "close vectors should have high cosine similarity, got {}",
            sim
        );
    }

    #[test]
    fn test_f32_slice_similarity_euclidean() {
        let q = vec![0.0, 0.0];
        let c = vec![3.0, 4.0];
        let sim = f32_slice_similarity(&q, None, &c, DistanceMetric::Euclidean);
        // Euclidean similarity = -squared_distance → -(9 + 16) = -25
        assert!(
            (sim - (-25.0)).abs() < 1e-5,
            "euclidean similarity should be -25, got {}",
            sim
        );
    }

    // ── SQ8 similarity (via calculate_similarity) ────────────────────────

    fn sq8_encode(v: &[f32]) -> (Box<[i8]>, f32) {
        let max_abs = v
            .iter()
            .map(|x| x.abs())
            .fold(f32::EPSILON.max(0.0_f32), f32::max);
        let scale = max_abs;
        let inv = scale / 127.0;
        let data: Vec<i8> = v
            .iter()
            .map(|&x| (x / inv).round().clamp(-128.0, 127.0) as i8)
            .collect();
        (data.into_boxed_slice(), scale)
    }

    #[test]
    fn test_sq8_similarity_cosine_self() {
        let v = vec![3.0, 4.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let (data, scale) = sq8_encode(&v);
        let sim = calculate_similarity(
            &v,
            Some(5.0),
            None,
            None,
            &VectorRepresentations::SQ8(data, scale),
            DistanceMetric::Cosine,
        );
        assert!(sim > 0.95, "SQ8 self-cosine should be ~1.0, got {}", sim);
    }

    #[test]
    fn test_sq8_similarity_cosine_orthogonal() {
        let q = vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let (data_b, scale_b) = sq8_encode(&b);
        let sim = calculate_similarity(
            &q,
            None,
            None,
            None,
            &VectorRepresentations::SQ8(data_b, scale_b),
            DistanceMetric::Cosine,
        );
        assert!(
            sim.abs() < 0.15,
            "SQ8 orthogonal cosine should be ~0.0, got {}",
            sim
        );
    }

    #[test]
    fn test_sq8_similarity_euclidean_self() {
        let v = vec![3.0, 4.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let (data, scale) = sq8_encode(&v);
        let sim = calculate_similarity(
            &v,
            Some(5.0),
            None,
            None,
            &VectorRepresentations::SQ8(data, scale),
            DistanceMetric::Euclidean,
        );
        assert!(
            sim.abs() < 0.1,
            "SQ8 self-Euclidean should be ~0.0, got {}",
            sim
        );
    }

    #[test]
    fn test_sq8_similarity_euclidean_negative() {
        let q = vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let b = vec![10.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let (data_b, scale_b) = sq8_encode(&b);
        let sim = calculate_similarity(
            &q,
            Some(0.0),
            None,
            None,
            &VectorRepresentations::SQ8(data_b, scale_b),
            DistanceMetric::Euclidean,
        );
        assert!(
            sim < 0.0,
            "SQ8 Euclidean similarity should be negative, got {}",
            sim
        );
    }

    #[test]
    fn test_sq8_zero_query_norm() {
        let q = vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let (data_b, scale_b) = sq8_encode(&b);
        let sim = calculate_similarity(
            &q,
            None,
            None,
            None,
            &VectorRepresentations::SQ8(data_b, scale_b),
            DistanceMetric::Cosine,
        );
        assert!(
            sim.abs() < 1e-6,
            "zero-norm query for SQ8 should return 0, got {}",
            sim
        );
    }

    // ── calculate_similarity: Binary variant ─────────────────────────────

    #[test]
    fn test_calculate_similarity_binary_with_query() {
        use crate::vector::quantization::rabitq_quantize;
        let v = vec![1.0, -2.0, 3.0, -4.0, 5.0, -6.0, 7.0, -8.0];
        let data = rabitq_quantize(&v);
        let sim = calculate_similarity(
            &v,
            None,
            Some(&data),
            None,
            &VectorRepresentations::Binary(data.clone()),
            DistanceMetric::Cosine,
        );
        assert!(
            (sim - 1.0).abs() < 1e-6,
            "identical binary should return 1, got {}",
            sim
        );
    }

    #[test]
    fn test_calculate_similarity_binary_no_quantized_query() {
        let v = vec![1.0, 2.0, 3.0];
        let data: Box<[u64]> = vec![0u64; 1].into_boxed_slice();
        let sim = calculate_similarity(
            &v,
            None,
            None,
            None,
            &VectorRepresentations::Binary(data),
            DistanceMetric::Cosine,
        );
        assert!(
            sim.abs() < 1e-6,
            "no quantized query should return 0 for Binary, got {}",
            sim
        );
    }

    // ── calculate_similarity: Turbo variant ──────────────────────────────

    #[test]
    fn test_calculate_similarity_turbo_with_query() {
        use crate::vector::quantization::turbo_quant_quantize;
        let v = vec![1.0, 2.0, 3.0, 4.0];
        let (data, max_abs) = turbo_quant_quantize(&v);
        let sim = calculate_similarity(
            &v,
            None,
            None,
            Some((&data, max_abs)),
            &VectorRepresentations::Turbo(data.clone()),
            DistanceMetric::Cosine,
        );
        assert!(
            sim > 0.0,
            "turbo self should return positive similarity, got {}",
            sim
        );
    }

    #[test]
    fn test_calculate_similarity_turbo_no_quantized_query() {
        let data: Box<[u8]> = vec![0u8; 4].into_boxed_slice();
        let sim = calculate_similarity(
            &[1.0, 2.0],
            None,
            None,
            None,
            &VectorRepresentations::Turbo(data),
            DistanceMetric::Cosine,
        );
        assert!(
            sim.abs() < 1e-6,
            "no quantized query should return 0 for Turbo, got {}",
            sim
        );
    }

    // ── calculate_similarity: MmapFull(None) branch ──────────────────────

    #[test]
    fn test_calculate_similarity_mmap_none() {
        let sim = calculate_similarity(
            &[1.0, 2.0],
            None,
            None,
            None,
            &VectorRepresentations::MmapFull(None),
            DistanceMetric::Cosine,
        );
        assert!(
            sim.abs() < 1e-6,
            "MmapFull(None) should return 0, got {}",
            sim
        );
    }

    // ── Full + Euclidean + query_norm branches ───────────────────────────

    #[test]
    fn test_calculate_similarity_full_euclidean_zero_norm() {
        let q = vec![0.0, 0.0];
        let node = vec![3.0, 4.0];
        let sim = calculate_similarity(
            &q,
            Some(0.0),
            None,
            None,
            &VectorRepresentations::Full(node),
            DistanceMetric::Euclidean,
        );
        assert!(
            (sim - (-25.0)).abs() < 1e-5,
            "Euclidean similarity should be -25, got {}",
            sim
        );
    }

    #[test]
    fn test_calculate_similarity_full_euclidean_nonzero_norm() {
        let q = vec![1.0, 0.0];
        let node = vec![4.0, 0.0];
        let query_norm = f32_l2_norm(&q);
        let sim = calculate_similarity(
            &q,
            Some(query_norm),
            None,
            None,
            &VectorRepresentations::Full(node),
            DistanceMetric::Euclidean,
        );
        assert!(
            (sim - (-9.0)).abs() < 1e-5,
            "Euclidean similarity should be -9, got {}",
            sim
        );
    }

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
    fn test_f32_slice_similarity_euclidean_with_norm() {
        let q = vec![0.0, 0.0];
        let c = vec![3.0, 4.0];
        let sim = f32_slice_similarity(&q, Some(0.0), &c, DistanceMetric::Euclidean);
        assert!((sim - (-25.0)).abs() < 1e-5, "expected -25, got {}", sim);
    }

    #[test]
    fn test_f32_slice_similarity_cosine_with_norm() {
        let q = vec![1.0, 0.0, 0.0];
        let c = vec![0.9, 0.1, 0.0];
        let norm_q = f32_l2_norm(&q);
        let sim = f32_slice_similarity(&q, Some(norm_q), &c, DistanceMetric::Cosine);
        assert!(
            sim > 0.9,
            "close vectors should have high cosine, got {}",
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

    // ── Miri tests for unsafe patterns (chunks_exact + unwrap_unchecked) ──
    //
    // Each f32x8 kernel has 2 unsafe blocks (a_chunk + b_chunk),
    // each f32x16 kernel has 2 unsafe blocks, and sq8_similarity has 2.
    // Total: 14 unsafe blocks exercised here (the MmapFull from_raw_parts
    // in calculate_similarity is tested separately below).
    //
    // Miri verifies that the SAFETY invariants on chunks_exact(8/16) hold:
    // the unwrap_unchecked is valid because chunks_exact guarantees chunk.len() == N.

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

    #[cfg(miri)]
    #[test]
    fn miri_distance_sq8_kernels() {
        // SQ8 uses 2 unsafe: chunks_exact(8) for q_chunk → unwrap_unchecked.
        // Test sizes that are multiples and non-multiples of 8.
        let test_sizes: &[usize] = &[0, 1, 8, 9, 16, 20, 32];
        for &size in test_sizes {
            let a: Vec<f32> = (0..size).map(|i| (i as f32).sin()).collect();
            if a.is_empty() {
                continue; // skip empty — SQ8 with zero elements is degenerate
            }
            // Encode as SQ8
            let max_abs = a.iter().map(|x| x.abs()).fold(f32::EPSILON, f32::max);
            let scale = max_abs;
            let inv = scale / 127.0;
            let sq8_data: Vec<i8> = a.iter().map(|&x| (x / inv).round() as i8).collect();

            let sim_cos = calculate_similarity(
                &a,
                None,
                None,
                None,
                &VectorRepresentations::SQ8(sq8_data.clone().into_boxed_slice(), scale),
                DistanceMetric::Cosine,
            );
            assert!(sim_cos.is_finite(), "SQ8 cosine(size={})", size);

            let sim_euc = calculate_similarity(
                &a,
                None,
                None,
                None,
                &VectorRepresentations::SQ8(sq8_data.into_boxed_slice(), scale),
                DistanceMetric::Euclidean,
            );
            assert!(sim_euc.is_finite(), "SQ8 euclidean(size={})", size);
        }
    }

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

            let d = euclidean_distance_squared_f32(&a, &b);
            assert!(
                d.is_finite(),
                "euclidean_distance_squared_f32(size={})",
                size
            );

            let norm = f32_l2_norm(&a);
            assert!(norm.is_finite(), "f32_l2_norm(size={})", size);

            let cos = cosine_sim_f32(&a, &b);
            assert!(cos.is_finite(), "cosine_sim_f32(size={})", size);

            if !a.is_empty() && norm > f32::EPSILON {
                let inv = 1.0 / norm;
                let bn = f32_l2_norm(&b);
                let binv = if bn > f32::EPSILON { 1.0 / bn } else { 0.0 };
                let cached = cosine_sim_cached_norms(&a, inv, &b, binv);
                assert!(cached.is_finite(), "cosine_sim_cached_norms(size={})", size);
            }

            let fs = f32_slice_similarity(&a, None, &b, DistanceMetric::Cosine);
            assert!(fs.is_finite(), "f32_slice_similarity cosine(size={})", size);

            let fs2 = f32_slice_similarity(&a, None, &b, DistanceMetric::Euclidean);
            assert!(
                fs2.is_finite(),
                "f32_slice_similarity euclidean(size={})",
                size
            );
        }
    }

    #[cfg(miri)]
    #[test]
    fn miri_distance_calculate_similarity_variants() {
        // Exercise calculate_similarity dispatch for Full, None, and
        // MmapFull(None) variants. The MmapFull(None) path hits the
        // match arm but returns early before the from_raw_parts unsafe.
        let test_sizes: &[usize] = &[0, 1, 8, 16, 32, 100];
        for &size in test_sizes {
            let a: Vec<f32> = (0..size).map(|i| i as f32).collect();

            // Full vectors — exercises the dispatch to f32x8/f32x16 kernels
            if !a.is_empty() {
                let s1 = calculate_similarity(
                    &a,
                    None,
                    None,
                    None,
                    &VectorRepresentations::Full(a.clone()),
                    DistanceMetric::Cosine,
                );
                assert!(s1.is_finite(), "Full Cosine(size={})", size);

                let s2 = calculate_similarity(
                    &a,
                    None,
                    None,
                    None,
                    &VectorRepresentations::Full(a.clone()),
                    DistanceMetric::Euclidean,
                );
                assert!(s2.is_finite(), "Full Euclidean(size={})", size);

                // With query_norm
                let norm = f32_l2_norm(&a);
                let s3 = calculate_similarity(
                    &a,
                    Some(norm),
                    None,
                    None,
                    &VectorRepresentations::Full(a.clone()),
                    DistanceMetric::Cosine,
                );
                assert!(s3.is_finite(), "Full Cosine+norm(size={})", size);

                let s4 = calculate_similarity(
                    &a,
                    Some(norm),
                    None,
                    None,
                    &VectorRepresentations::Full(a.clone()),
                    DistanceMetric::Euclidean,
                );
                assert!(s4.is_finite(), "Full Euclidean+norm(size={})", size);
            }

            // None variant — trivial early-return
            let s5 = calculate_similarity(
                &a,
                None,
                None,
                None,
                &VectorRepresentations::None,
                DistanceMetric::Cosine,
            );
            assert_eq!(s5, 0.0, "None(size={})", size);

            // MmapFull(None) — reaches the MmapFull arm but returns before unsafe
            let s6 = calculate_similarity(
                &a,
                None,
                None,
                None,
                &VectorRepresentations::MmapFull(None),
                DistanceMetric::Cosine,
            );
            assert_eq!(s6, 0.0, "MmapFull(None)(size={})", size);
        }
    }
}
