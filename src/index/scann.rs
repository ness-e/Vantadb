// ponytail: SCANN quantization + partition slot invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]

//! SCANN — Simplified ScaNN (Scalable Nearest Neighbors) with scalar
//! quantization (SQ8) compression.
//!
//! Each dimension of a vector is quantized from f32 to u8 using per-dimension
//! min/max bounds. Search first scores all candidates with approximate SQ8
//! distance, then re-ranks the top candidates with full f32 precision.
//!
//! ## ponytail
//! Simplified SQ8 only — no anisotropic quantization, no PQ, no GPU.
//! Anisotropic would require per-dataset variance profiling.

use crate::index::distance::DIMENSION_MISMATCH_SCORE;
use crate::index::graph::total_cmp_sim;
use crate::node::{DistanceMetric, FilterBitset, VectorRepresentations};
use std::sync::Mutex;

/// Configuration for SCANN index.
#[derive(Clone, Debug)]
pub struct ScannConfig {
    pub distance_metric: DistanceMetric,
    /// Number of candidates to pre-filter before full re-rank.
    pub reorder_ratio: usize,
}

impl Default for ScannConfig {
    fn default() -> Self {
        Self {
            distance_metric: DistanceMetric::Cosine,
            reorder_ratio: 10,
        }
    }
}

/// A quantized entry in the SCANN index.
#[derive(Clone)]
struct ScannEntry {
    id: u128,
    bitset: FilterBitset,
    /// SQ8 compressed vector (1 byte per dimension).
    code: Vec<u8>,
    /// Pre-computed squared norm of the original vector (for cosine).
    norm_sq: f32,
    #[allow(dead_code)]
    storage_offset: u64,
}

/// Internal state of ScannIndex consolidated into a single atomic struct
/// to eliminate multi-mutex deadlock risks (FIND-214).
struct ScannInner {
    entries: Vec<ScannEntry>,
    min_bound: Vec<f32>,
    max_bound: Vec<f32>,
    dim: usize,
    bounds_initialized: bool,
}

impl ScannInner {
    fn update_bounds(&mut self, vec: &[f32]) {
        if !self.bounds_initialized {
            self.min_bound = vec.to_vec();
            self.max_bound = vec.to_vec();
            self.dim = vec.len();
            self.bounds_initialized = true;
            return;
        }

        if vec.len() != self.dim {
            return; // skip mismatched dims (ponytail)
        }

        for (i, &v) in vec.iter().enumerate() {
            if v < self.min_bound[i] {
                self.min_bound[i] = v;
            }
            if v > self.max_bound[i] {
                self.max_bound[i] = v;
            }
        }
    }
}

/// SCANN index with scalar quantization (SQ8).
///
/// Vectors are stored as `u8` after per-dimension min-max normalization.
/// Search uses approximate distance on quantized data, then re-ranks the
/// top `reorder_ratio * top_k` candidates with full precision.
pub struct ScannIndex {
    inner: Mutex<ScannInner>,
    config: ScannConfig,
}

#[allow(dead_code)]
impl ScannIndex {
    pub fn new(distance_metric: DistanceMetric) -> Self {
        Self {
            inner: Mutex::new(ScannInner {
                entries: Vec::new(),
                min_bound: Vec::new(),
                max_bound: Vec::new(),
                dim: 0,
                bounds_initialized: false,
            }),
            config: ScannConfig {
                distance_metric,
                ..Default::default()
            },
        }
    }

    /// Quantize a float vector to u8 using current bounds.
    fn quantize(vec: &[f32], min_bound: &[f32], max_bound: &[f32]) -> Vec<u8> {
        vec.iter()
            .zip(min_bound.iter().zip(max_bound.iter()))
            .map(|(&v, (&mn, &mx))| {
                let range = mx - mn;
                if range < f32::EPSILON {
                    128u8
                } else {
                    let scaled = (v - mn) / range;
                    (scaled * 255.0).round().clamp(0.0, 255.0) as u8
                }
            })
            .collect()
    }

    /// Approximate SQ8 distance between query and a compressed vector.
    /// Returns `(approx_similarity, code_ref)`.
    fn approximate_distance(
        query: &[f32],
        code: &[u8],
        min_bound: &[f32],
        max_bound: &[f32],
        orig_norm_sq: f32,
        metric: DistanceMetric,
    ) -> f32 {
        if query.len() != code.len() || query.is_empty() {
            // FIND-411: a dimension mismatch is "not comparable", not the
            // worst possible score — the sentinel the index drops.
            return DIMENSION_MISMATCH_SCORE;
        }

        // Decompress to approximate f32
        let approx: Vec<f32> = code
            .iter()
            .zip(min_bound.iter().zip(max_bound.iter()))
            .map(|(&c, (&mn, &mx))| {
                let ratio = c as f32 / 255.0;
                mn + ratio * (mx - mn)
            })
            .collect();

        match metric {
            DistanceMetric::Cosine => {
                // Cosine with decompressed vector
                let dot: f32 = query.iter().zip(approx.iter()).map(|(a, b)| a * b).sum();
                let query_norm_sq: f32 = query.iter().map(|v| v * v).sum();
                let query_norm = query_norm_sq.sqrt();
                let approx_norm = orig_norm_sq.sqrt();
                if query_norm < f32::EPSILON || approx_norm < f32::EPSILON {
                    return 0.0;
                }
                dot / (query_norm * approx_norm)
            }
            DistanceMetric::Euclidean => -query
                .iter()
                .zip(approx.iter())
                .map(|(a, b)| (a - b) * (a - b))
                .sum::<f32>(),
            // SparseDot is not a quantized-dense codebook path (own brute-force).
            DistanceMetric::SparseDot => 0.0,
        }
    }

    /// Full-precision distance between query and a stored vector (decompressed).
    fn full_distance(
        query: &[f32],
        code: &[u8],
        min_bound: &[f32],
        max_bound: &[f32],
        metric: DistanceMetric,
    ) -> f32 {
        // FIND-411: a dimension mismatch must not be silently truncated by
        // the `zip` below (it compared only the leading dimensions) — it is
        // "not comparable" and scores the mismatch sentinel.
        if query.len() != code.len() || query.is_empty() {
            return DIMENSION_MISMATCH_SCORE;
        }

        let approx: Vec<f32> = code
            .iter()
            .zip(min_bound.iter().zip(max_bound.iter()))
            .map(|(&c, (&mn, &mx))| {
                let ratio = c as f32 / 255.0;
                mn + ratio * (mx - mn)
            })
            .collect();

        match metric {
            DistanceMetric::Cosine => {
                let dot: f32 = query.iter().zip(approx.iter()).map(|(a, b)| a * b).sum();
                let query_norm_sq: f32 = query.iter().map(|v| v * v).sum();
                let approx_norm_sq: f32 = approx.iter().map(|v| v * v).sum();
                let query_norm = query_norm_sq.sqrt();
                let approx_norm = approx_norm_sq.sqrt();
                if query_norm < f32::EPSILON || approx_norm < f32::EPSILON {
                    return 0.0;
                }
                dot / (query_norm * approx_norm)
            }
            DistanceMetric::Euclidean => -query
                .iter()
                .zip(approx.iter())
                .map(|(a, b)| (a - b) * (a - b))
                .sum::<f32>(),
            // SparseDot is not a quantized-dense codebook path (own brute-force).
            DistanceMetric::SparseDot => 0.0,
        }
    }
}

impl crate::index::VecIndex for ScannIndex {
    fn search(
        &self,
        query_vec: &[f32],
        query_mask: &FilterBitset,
        top_k: usize,
        _vector_store: Option<&dyn crate::index_port::VectorStoreRef>,
        _distance_metric: DistanceMetric,
    ) -> Vec<(u128, f32)> {
        if top_k == 0 {
            return Vec::new();
        }

        // INVARIANT (FIND-214): un único lock sobre ScannInner — elimina riesgo de deadlock
        let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());

        if !inner.bounds_initialized || inner.min_bound.is_empty() || inner.entries.is_empty() {
            return Vec::new();
        }

        let metric = self.config.distance_metric;
        let reorder_k = inner.entries.len().min(top_k * self.config.reorder_ratio);

        // Stage 1: Score all candidates with approximate SQ8 distance
        let mut approx_results: Vec<(u128, f32, &[u8])> = inner
            .entries
            .iter()
            .filter(|e| query_mask.is_all_set() || e.bitset.matches_mask(query_mask))
            .map(|e| {
                let approx_sim = Self::approximate_distance(
                    query_vec,
                    &e.code,
                    &inner.min_bound,
                    &inner.max_bound,
                    e.norm_sq,
                    metric,
                );
                (e.id, approx_sim, e.code.as_slice())
            })
            .collect();

        // Sort by approximate similarity descending. FIND-411 (R3): the
        // comparison is NaN-aware — a candidate whose dimension differs from
        // the query scores NaN and is dropped, not ranked.
        approx_results.retain(|(_, sim, _)| !sim.is_nan());
        approx_results.sort_unstable_by(|a, b| total_cmp_sim(b.1, a.1));

        // Stage 2: Re-rank top candidates with full precision
        let rerank_candidates = &approx_results[..approx_results.len().min(reorder_k)];
        let mut final_results: Vec<(u128, f32)> = rerank_candidates
            .iter()
            .map(|(id, _, code)| {
                let full_sim = Self::full_distance(
                    query_vec,
                    code,
                    &inner.min_bound,
                    &inner.max_bound,
                    metric,
                );
                (*id, full_sim)
            })
            .collect();

        // FIND-411 (R3): drop the candidates that are not comparable and
        // apply the same distance-scale conversion the flat path applies, so
        // the backends return the same score scale. The VALUE stays
        // approximate — SCANN re-ranks over the decompressed codebook
        // vector, which is inherent to the lossy index.
        final_results.retain(|(_, score)| !score.is_nan());
        if metric == DistanceMetric::Euclidean {
            for entry in final_results.iter_mut() {
                entry.1 = crate::index::euclidean_score_from_similarity(entry.1);
            }
        }
        final_results.sort_unstable_by(|a, b| total_cmp_sim(b.1, a.1));
        final_results.truncate(top_k);
        final_results
    }

    fn add(
        &self,
        id: u128,
        bitset: FilterBitset,
        vec_data: VectorRepresentations,
        storage_offset: u64,
    ) -> crate::error::Result<()> {
        let vec = match &vec_data {
            VectorRepresentations::Full(v) => v.clone(),
            _ => {
                return Err(crate::error::Error::Validation {
                    field: "vec_data".into(),
                    reason: "ScannIndex::add only accepts full vectors (ERR-031)".into(),
                })
            }
        };

        let mut inner = self.inner.lock().map_err(|_| {
            crate::error::Error::Runtime(crate::error::ChainedError::msg(
                "ScannIndex inner lock poisoned",
            ))
        })?;

        // Actualizar bounds atómicamente bajo el mismo lock
        inner.update_bounds(&vec);

        if vec.len() != inner.dim && !vec.is_empty() {
            return Err(crate::error::Error::Validation {
                field: "vec_data".into(),
                reason: format!(
                    "ScannIndex::add vector dim {} != index dim {} (ERR-031)",
                    vec.len(),
                    inner.dim
                ),
            });
        }

        // Cuantización atómica directa con bounds ya validados
        let code = Self::quantize(&vec, &inner.min_bound, &inner.max_bound);
        let norm_sq: f32 = vec.iter().map(|v| v * v).sum();

        inner.entries.push(ScannEntry {
            id,
            bitset,
            code,
            norm_sq,
            storage_offset,
        });

        Ok(())
    }

    fn estimate_memory_bytes(&self) -> usize {
        let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let code_bytes: usize = inner.entries.iter().map(|e| e.code.len()).sum();
        let overhead = inner.entries.len() * (16 + std::mem::size_of::<FilterBitset>() + 8 + 4);
        let bounds_bytes = inner.min_bound.len() * 4 * 2;
        code_bytes + overhead + bounds_bytes + std::mem::size_of::<ScannConfig>()
    }

    fn len(&self) -> usize {
        self.inner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .entries
            .len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::VecIndex;
    use crate::node::ALL_BITSET;

    #[test]
    fn test_scann_empty() {
        let idx = ScannIndex::new(DistanceMetric::Cosine);
        let results = idx.search(&[1.0, 0.0], &ALL_BITSET, 5, None, DistanceMetric::Cosine);
        assert!(results.is_empty());
        assert_eq!(idx.len(), 0);
    }

    #[test]
    fn test_scann_sq8_roundtrip() {
        let idx = ScannIndex::new(DistanceMetric::Cosine);
        let _ = idx.add(
            1,
            FilterBitset::new(),
            VectorRepresentations::Full(vec![1.0, 0.0, 0.0]),
            0,
        );
        let _ = idx.add(
            2,
            FilterBitset::new(),
            VectorRepresentations::Full(vec![0.0, 1.0, 0.0]),
            0,
        );

        let results = idx.search(
            &[1.0, 0.0, 0.0],
            &ALL_BITSET,
            5,
            None,
            DistanceMetric::Cosine,
        );
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, 1, "top-1 should be id=1");
        assert!(results[0].1 > results[1].1, "scores descending");
    }

    #[test]
    fn test_scann_topk() {
        let idx = ScannIndex::new(DistanceMetric::Cosine);
        for i in 0u128..10 {
            let _ = idx.add(
                i,
                FilterBitset::new(),
                VectorRepresentations::Full(vec![i as f32, 0.0, 0.0]),
                0,
            );
        }
        let results = idx.search(
            &[0.0, 0.0, 0.0],
            &ALL_BITSET,
            3,
            None,
            DistanceMetric::Cosine,
        );
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_scann_bitset_filter() {
        let idx = ScannIndex::new(DistanceMetric::Cosine);
        let mut bs_a = FilterBitset::new();
        bs_a.set_bit(0);
        let mut bs_b = FilterBitset::new();
        bs_b.set_bit(1);

        let _ = idx.add(1, bs_a, VectorRepresentations::Full(vec![1.0, 0.0]), 0);
        let _ = idx.add(2, bs_b, VectorRepresentations::Full(vec![0.0, 1.0]), 0);

        let mut mask = FilterBitset::new();
        mask.set_bit(0);
        let results = idx.search(&[1.0, 0.0], &mask, 5, None, DistanceMetric::Cosine);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, 1);
    }

    #[test]
    fn test_scann_euclidean() {
        let idx = ScannIndex::new(DistanceMetric::Euclidean);
        let _ = idx.add(
            0,
            FilterBitset::new(),
            VectorRepresentations::Full(vec![0.0, 0.0]),
            0,
        );
        let _ = idx.add(
            1,
            FilterBitset::new(),
            VectorRepresentations::Full(vec![10.0, 10.0]),
            0,
        );

        let results = idx.search(&[0.0, 0.0], &ALL_BITSET, 2, None, DistanceMetric::Euclidean);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, 0);
        for &(_, s) in &results {
            assert!(s <= 0.0, "Euclidean <= 0");
        }
    }

    /// FIND-411 (R3.4): the SCANN must return the same score **scale** as
    /// the flat scan and the HNSW traversal — a negative distance, not a
    /// negative squared distance.
    #[test]
    fn test_scann_euclidean_score_uses_the_distance_scale() {
        let idx = ScannIndex::new(DistanceMetric::Euclidean);
        let _ = idx.add(
            0,
            FilterBitset::new(),
            VectorRepresentations::Full(vec![0.0, 0.0]),
            0,
        );
        let _ = idx.add(
            1,
            FilterBitset::new(),
            VectorRepresentations::Full(vec![3.0, 4.0]),
            0,
        );

        let results = idx.search(&[0.0, 0.0], &ALL_BITSET, 2, None, DistanceMetric::Euclidean);
        assert_eq!(results.len(), 2);
        let far = results
            .iter()
            .find(|(id, _)| *id == 1)
            .expect("node 1 is a candidate");
        assert!(
            (far.1 - (-5.0)).abs() < 1e-3,
            "the SCANN Euclidean score must be the distance scale (-5), not the squared \
             distance (-25), got {}",
            far.1
        );
    }

    /// FIND-411 (R3.4): a query whose dimension differs from the codebook is
    /// not comparable — the candidate must be dropped, not scored over the
    /// leading dimensions only.
    #[test]
    fn test_scann_drops_the_dimension_mismatched_candidate() {
        let idx = ScannIndex::new(DistanceMetric::Euclidean);
        let _ = idx.add(
            1,
            FilterBitset::new(),
            VectorRepresentations::Full(vec![0.0, 1.0, 0.0]),
            0,
        );

        let results = idx.search(
            &[1.0, 0.0, 0.0, 0.0, 0.0],
            &ALL_BITSET,
            5,
            None,
            DistanceMetric::Euclidean,
        );
        assert!(
            results.is_empty(),
            "a dimension-mismatched query has no candidates, got {results:?}"
        );
        for &(_, s) in &results {
            assert!(!s.is_nan(), "the SCANN must never return a NaN score");
        }
    }

    #[test]
    fn test_scann_quantize_identity() {
        // With vectors all in [0,1], quantization should round-trip closely
        let idx = ScannIndex::new(DistanceMetric::Euclidean);
        let _ = idx.add(
            0,
            FilterBitset::new(),
            VectorRepresentations::Full(vec![0.0, 0.5, 1.0]),
            0,
        );
        let _ = idx.add(
            1,
            FilterBitset::new(),
            VectorRepresentations::Full(vec![1.0, 0.5, 0.0]),
            0,
        );

        // Query = [0, 0.5, 1.0]
        let results = idx.search(
            &[0.0, 0.5, 1.0],
            &ALL_BITSET,
            2,
            None,
            DistanceMetric::Euclidean,
        );
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, 0, "closest to itself after SQ8");
    }

    #[test]
    fn test_scann_rejects_non_full_vector() {
        let idx = ScannIndex::new(DistanceMetric::Cosine);
        let result = idx.add(
            1,
            FilterBitset::new(),
            VectorRepresentations::Binary(Box::new([0u64; 2])),
            0,
        );
        assert!(result.is_err(), "non-full vector must be rejected");
        assert_eq!(idx.len(), 0, "rejected insert must not be stored");
    }

    #[test]
    fn add_returns_error_on_poisoned_lock() {
        let idx = ScannIndex::new(DistanceMetric::Cosine);
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = idx.inner.lock().unwrap();
            panic!("poison the scann inner lock");
        }));
        assert!(idx.inner.is_poisoned());
        let err = idx
            .add(
                1,
                FilterBitset::new(),
                VectorRepresentations::Full(vec![1.0, 0.0]),
                0,
            )
            .unwrap_err();
        assert!(
            matches!(err, crate::error::Error::Runtime(_)),
            "expected Runtime, got {err:?}"
        );
    }
}
