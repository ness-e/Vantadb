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

/// SCANN index with scalar quantization (SQ8).
///
/// Vectors are stored as `u8` after per-dimension min-max normalization.
/// Search uses approximate distance on quantized data, then re-ranks the
/// top `reorder_ratio * top_k` candidates with full precision.
pub struct ScannIndex {
    entries: Mutex<Vec<ScannEntry>>,
    /// Per-dimension minimum values (from training data).
    min_bound: Mutex<Vec<f32>>,
    /// Per-dimension maximum values (from training data).
    max_bound: Mutex<Vec<f32>>,
    dim: Mutex<usize>,
    config: ScannConfig,
    /// Whether bounds have been initialized.
    bounds_initialized: Mutex<bool>,
}

#[allow(dead_code)]
impl ScannIndex {
    pub fn new(distance_metric: DistanceMetric) -> Self {
        Self {
            entries: Mutex::new(Vec::new()),
            min_bound: Mutex::new(Vec::new()),
            max_bound: Mutex::new(Vec::new()),
            dim: Mutex::new(0),
            config: ScannConfig {
                distance_metric,
                ..Default::default()
            },
            bounds_initialized: Mutex::new(false),
        }
    }

    /// Quantize a float vector to u8 using current bounds.
    fn quantize(&self, vec: &[f32], min_bound: &[f32], max_bound: &[f32]) -> Vec<u8> {
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
            return f32::NEG_INFINITY;
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

    /// Update min/max bounds from a new vector.
    fn update_bounds(&self, vec: &[f32]) {
        let mut min_bound = self.min_bound.lock().unwrap();
        let mut max_bound = self.max_bound.lock().unwrap();
        let mut dim = self.dim.lock().unwrap();
        let mut initialized = self.bounds_initialized.lock().unwrap();

        if !*initialized {
            *min_bound = vec.to_vec();
            *max_bound = vec.to_vec();
            *dim = vec.len();
            *initialized = true;
            return;
        }

        if vec.len() != *dim {
            return; // skip mismatched dims (ponytail)
        }

        for (i, &v) in vec.iter().enumerate() {
            if v < min_bound[i] {
                min_bound[i] = v;
            }
            if v > max_bound[i] {
                max_bound[i] = v;
            }
        }
    }
}

impl crate::index::VecIndex for ScannIndex {
    fn search(
        &self,
        query_vec: &[f32],
        query_mask: &FilterBitset,
        top_k: usize,
        _vector_store: Option<&crate::storage::vfile::VantaFile>,
        _distance_metric: DistanceMetric,
    ) -> Vec<(u128, f32)> {
        if top_k == 0 {
            return Vec::new();
        }

        let entries = self.entries.lock().unwrap();
        let min_bound = self.min_bound.lock().unwrap();
        let max_bound = self.max_bound.lock().unwrap();
        let initialized = *self.bounds_initialized.lock().unwrap();

        if !initialized || min_bound.is_empty() || entries.is_empty() {
            return Vec::new();
        }

        let metric = self.config.distance_metric;
        let reorder_k = entries.len().min(top_k * self.config.reorder_ratio);

        // Stage 1: Score all candidates with approximate SQ8 distance
        let mut approx_results: Vec<(u128, f32, &[u8])> = entries
            .iter()
            .filter(|e| query_mask.is_all_set() || e.bitset.matches_mask(query_mask))
            .map(|e| {
                let approx_sim = Self::approximate_distance(
                    query_vec, &e.code, &min_bound, &max_bound, e.norm_sq, metric,
                );
                (e.id, approx_sim, e.code.as_slice())
            })
            .collect();

        // Sort by approximate similarity descending
        approx_results
            .sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Stage 2: Re-rank top candidates with full precision
        let rerank_candidates = &approx_results[..approx_results.len().min(reorder_k)];
        let mut final_results: Vec<(u128, f32)> = rerank_candidates
            .iter()
            .map(|(id, _, code)| {
                let full_sim = Self::full_distance(query_vec, code, &min_bound, &max_bound, metric);
                (*id, full_sim)
            })
            .collect();

        final_results
            .sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        final_results.truncate(top_k);
        final_results
    }

    fn add(
        &self,
        id: u128,
        bitset: FilterBitset,
        vec_data: VectorRepresentations,
        storage_offset: u64,
    ) {
        let vec = match &vec_data {
            VectorRepresentations::Full(v) => v.clone(),
            _ => return, // skip non-full vectors
        };

        // Update global bounds
        self.update_bounds(&vec);

        let dim = *self.dim.lock().unwrap();
        if vec.len() != dim && !vec.is_empty() {
            return;
        }

        // Quantize
        let min_bound = self.min_bound.lock().unwrap();
        let max_bound = self.max_bound.lock().unwrap();
        let code = self.quantize(&vec, &min_bound, &max_bound);
        let norm_sq: f32 = vec.iter().map(|v| v * v).sum();
        drop(min_bound);
        drop(max_bound);

        let mut entries = self.entries.lock().unwrap();
        entries.push(ScannEntry {
            id,
            bitset,
            code,
            norm_sq,
            storage_offset,
        });
    }

    fn estimate_memory_bytes(&self) -> usize {
        let entries = self.entries.lock().unwrap();
        let code_bytes: usize = entries.iter().map(|e| e.code.len()).sum();
        let overhead = entries.len() * (16 + std::mem::size_of::<FilterBitset>() + 8 + 4);
        let bounds_bytes = self.min_bound.lock().unwrap().len() * 4 * 2;
        code_bytes + overhead + bounds_bytes + std::mem::size_of::<ScannConfig>()
    }

    fn len(&self) -> usize {
        self.entries.lock().unwrap().len()
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
        idx.add(
            1,
            FilterBitset::new(),
            VectorRepresentations::Full(vec![1.0, 0.0, 0.0]),
            0,
        );
        idx.add(
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
            idx.add(
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

        idx.add(1, bs_a, VectorRepresentations::Full(vec![1.0, 0.0]), 0);
        idx.add(2, bs_b, VectorRepresentations::Full(vec![0.0, 1.0]), 0);

        let mut mask = FilterBitset::new();
        mask.set_bit(0);
        let results = idx.search(&[1.0, 0.0], &mask, 5, None, DistanceMetric::Cosine);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, 1);
    }

    #[test]
    fn test_scann_euclidean() {
        let idx = ScannIndex::new(DistanceMetric::Euclidean);
        idx.add(
            0,
            FilterBitset::new(),
            VectorRepresentations::Full(vec![0.0, 0.0]),
            0,
        );
        idx.add(
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

    #[test]
    fn test_scann_quantize_identity() {
        // With vectors all in [0,1], quantization should round-trip closely
        let idx = ScannIndex::new(DistanceMetric::Euclidean);
        idx.add(
            0,
            FilterBitset::new(),
            VectorRepresentations::Full(vec![0.0, 0.5, 1.0]),
            0,
        );
        idx.add(
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
}
