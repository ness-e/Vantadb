use crate::index::distance::calculate_similarity;
use crate::node::{DistanceMetric, FilterBitset, VectorRepresentations};
use std::sync::Mutex;

/// ponytail: full DashMap O(n) scan is by design — only called when
/// `flat_threshold` (default 10000) bounds the node count. For larger
/// datasets the HNSW path handles search without iterating all nodes.
pub(crate) fn flat_search(
    nodes: &dashmap::DashMap<u128, super::graph::HnswNode>,
    query_vec: &[f32],
    query_mask: &FilterBitset,
    top_k: usize,
    metric: crate::node::DistanceMetric,
) -> Vec<(u128, f32)> {
    use crate::storage::engine::FLAG_TOMBSTONE;

    let query_inv_norm = if metric == DistanceMetric::Cosine {
        let norm = crate::index::f32_l2_norm(query_vec);
        if norm > f32::EPSILON {
            Some(1.0 / norm)
        } else {
            None
        }
    } else {
        None
    };

    let mut results: Vec<(u128, f32)> = nodes
        .iter()
        .filter(|entry| {
            let node = entry.value();
            (node.flags & FLAG_TOMBSTONE) == 0
                && (query_mask.is_all_set() || node.bitset.matches_mask(query_mask))
                && !node.vec_data.is_none()
        })
        .map(|entry| {
            let id = *entry.key();
            let node = entry.value();
            let sim = match (&node.vec_data, metric, query_inv_norm) {
                (VectorRepresentations::Full(v), DistanceMetric::Cosine, Some(q_inv)) => {
                    crate::index::cosine_sim_cached_norms(query_vec, q_inv, v, node.inv_cached_norm)
                }
                _ => calculate_similarity(query_vec, None, None, None, &node.vec_data, metric),
            };
            (id, sim)
        })
        .collect();

    results.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(top_k);
    results
}

// ---------------------------------------------------------------------------
// FlatIndex — standalone brute-force index implementing VecIndex
// ---------------------------------------------------------------------------

/// A simple brute-force (flat) index that stores vectors in a `Vec` and
/// linearly scans all entries on every search.
///
/// Used for small datasets where index construction overhead outweighs
/// the benefit of approximate search.
pub struct FlatIndex {
    nodes: Mutex<Vec<FlatEntry>>,
    config: FlatConfig,
}

#[derive(Clone, Debug)]
pub struct FlatConfig {
    pub distance_metric: DistanceMetric,
}

#[derive(Clone)]
struct FlatEntry {
    id: u128,
    bitset: FilterBitset,
    vec: VectorRepresentations,
    inv_cached_norm: f32,
    #[allow(dead_code)]
    storage_offset: u64,
}

#[allow(dead_code)]
impl FlatIndex {
    pub fn new(distance_metric: DistanceMetric) -> Self {
        Self {
            nodes: Mutex::new(Vec::new()),
            config: FlatConfig { distance_metric },
        }
    }
}

impl crate::index::VecIndex for FlatIndex {
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
        let nodes = self.nodes.lock().unwrap();
        let metric = self.config.distance_metric;

        let query_inv_norm = if metric == DistanceMetric::Cosine {
            let norm = crate::index::f32_l2_norm(query_vec);
            if norm > f32::EPSILON {
                Some(1.0 / norm)
            } else {
                None
            }
        } else {
            None
        };

        let mut results: Vec<(u128, f32)> = nodes
            .iter()
            .filter(|e| query_mask.is_all_set() || e.bitset.matches_mask(query_mask))
            .map(|e| {
                let sim = match (&e.vec, metric, query_inv_norm) {
                    (VectorRepresentations::Full(v), DistanceMetric::Cosine, Some(q_inv)) => {
                        crate::index::cosine_sim_cached_norms(
                            query_vec,
                            q_inv,
                            v,
                            e.inv_cached_norm,
                        )
                    }
                    _ => calculate_similarity(query_vec, None, None, None, &e.vec, metric),
                };
                (e.id, sim)
            })
            .collect();

        results.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(top_k);
        results
    }

    fn add(
        &self,
        id: u128,
        bitset: FilterBitset,
        vec_data: VectorRepresentations,
        storage_offset: u64,
    ) {
        let inv_cached_norm = match &vec_data {
            VectorRepresentations::Full(v) => {
                let norm = crate::index::f32_l2_norm(v);
                if norm > f32::EPSILON {
                    1.0 / norm
                } else {
                    1.0
                }
            }
            _ => 1.0,
        };
        let mut nodes = self.nodes.lock().unwrap();
        nodes.push(FlatEntry {
            id,
            bitset,
            vec: vec_data,
            inv_cached_norm,
            storage_offset,
        });
    }

    fn estimate_memory_bytes(&self) -> usize {
        let nodes = self.nodes.lock().unwrap();
        let vec_bytes: usize = nodes
            .iter()
            .map(|e| match &e.vec {
                VectorRepresentations::Full(v) => v.len() * 4,
                VectorRepresentations::MmapFull(Some(m)) => m.len(),
                VectorRepresentations::Binary(b) => b.len() * 8,
                VectorRepresentations::Turbo(t) => t.len(),
                VectorRepresentations::SQ8(d, _) => d.len() + 4,
                VectorRepresentations::None => 0,
                VectorRepresentations::MmapFull(None) => 0,
            })
            .sum();
        nodes.len() * (16 + std::mem::size_of::<FilterBitset>() + std::mem::size_of::<u64>())
            + vec_bytes
            + std::mem::size_of::<FlatConfig>()
    }

    fn len(&self) -> usize {
        self.nodes.lock().unwrap().len()
    }

    fn is_empty(&self) -> bool {
        self.nodes.lock().unwrap().is_empty()
    }
}

#[cfg(test)]
mod flat_tests {
    use super::*;
    use crate::index::VecIndex;

    #[test]
    fn test_flat_index_empty() {
        let idx = FlatIndex::new(DistanceMetric::Cosine);
        let results = idx.search(
            &[1.0, 0.0],
            &FilterBitset::all_set(),
            5,
            None,
            DistanceMetric::Cosine,
        );
        assert!(results.is_empty());
        assert!(idx.is_empty());
        assert_eq!(idx.len(), 0);
    }

    #[test]
    fn test_flat_index_basic_search() {
        let idx = FlatIndex::new(DistanceMetric::Cosine);
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
            &FilterBitset::all_set(),
            5,
            None,
            DistanceMetric::Cosine,
        );
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, 1, "most similar should be id=1");
        assert!(results[0].1 > results[1].1, "scores descending");
    }

    #[test]
    fn test_flat_index_topk_limits() {
        let idx = FlatIndex::new(DistanceMetric::Cosine);
        for i in 0..10u128 {
            idx.add(
                i,
                FilterBitset::new(),
                VectorRepresentations::Full(vec![i as f32, 0.0, 0.0]),
                0,
            );
        }
        let results = idx.search(
            &[0.0, 0.0, 0.0],
            &FilterBitset::all_set(),
            3,
            None,
            DistanceMetric::Cosine,
        );
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_flat_index_bitset_filter() {
        let idx = FlatIndex::new(DistanceMetric::Cosine);
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
    fn test_flat_index_euclidean() {
        let idx = FlatIndex::new(DistanceMetric::Euclidean);
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

        let results = idx.search(
            &[0.0, 0.0],
            &FilterBitset::all_set(),
            2,
            None,
            DistanceMetric::Euclidean,
        );
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, 0);
        for &(_, s) in &results {
            assert!(s <= 0.0, "Euclidean scores should be <= 0");
        }
    }
}
