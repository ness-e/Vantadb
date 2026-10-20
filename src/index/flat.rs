// ponytail: distance invariants / sorted-candidate unwraps; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use crate::index::distance::calculate_similarity;
use crate::node::{DistanceMetric, FilterBitset, NodeFlags, VectorRepresentations};
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
            (node.flags & NodeFlags::TOMBSTONE) == 0
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

    finalize_flat_results(&mut results, metric, top_k);
    results
}

/// Order + trim the flat-scan results so they match the HNSW path exactly.
///
/// FIND-411:
/// - A candidate whose dimension differs from the query scores `NaN`; it is
///   not a candidate and is dropped (it used to score `0.0` — the *best*
///   possible Euclidean score — and outranked every valid candidate).
/// - Euclidean scores are converted from "negative squared distance" to
///   "negative distance", the same scale `search_nearest_with_metric`
///   returns for the HNSW path, so both paths report the same score for the
///   same candidate.
/// - The ordering uses `total_cmp_sim` (NaN-aware) instead of `partial_cmp`
///   so a NaN can never be sorted to the front.
fn finalize_flat_results(
    results: &mut Vec<(u128, f32)>,
    metric: crate::node::DistanceMetric,
    top_k: usize,
) {
    results.retain(|(_, score)| !score.is_nan());
    if metric == crate::node::DistanceMetric::Euclidean {
        for entry in results.iter_mut() {
            entry.1 = crate::index::euclidean_score_from_similarity(entry.1);
        }
    }
    results.sort_unstable_by(|a, b| super::graph::total_cmp_sim(b.1, a.1));
    results.truncate(top_k);
}

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
        _vector_store: Option<&dyn crate::index_port::VectorStoreRef>,
        _distance_metric: DistanceMetric,
    ) -> Vec<(u128, f32)> {
        if top_k == 0 {
            return Vec::new();
        }
        // INVARIANT (B2b): `search` returns `Vec`, not `Result` (trait
        // `VecIndex`), so a poisoned lock recovers via `into_inner` instead of
        // panicking — same policy as `sync_ext`. Hot read path: zero-cost on
        // the happy path, no signature change.
        let nodes = self.nodes.lock().unwrap_or_else(|e| e.into_inner());
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

        finalize_flat_results(&mut results, metric, top_k);
        results
    }

    fn add(
        &self,
        id: u128,
        bitset: FilterBitset,
        vec_data: VectorRepresentations,
        storage_offset: u64,
    ) -> crate::error::Result<()> {
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
        // B2b: poisoned lock is a real error on the write path — fail the
        // insert with `Runtime` so callers propagate with `?` instead of
        // persisting possibly-torn state under a panic.
        let mut nodes = self.nodes.lock().map_err(|_| {
            crate::error::Error::Runtime(crate::error::ChainedError::msg(
                "FlatIndex nodes lock poisoned",
            ))
        })?;
        if vec_data.is_none() {
            nodes.retain(|e| e.id != id);
            return Ok(());
        }
        if let Some(entry) = nodes.iter_mut().find(|e| e.id == id) {
            entry.bitset = bitset;
            entry.vec = vec_data;
            entry.inv_cached_norm = inv_cached_norm;
            entry.storage_offset = storage_offset;
        } else {
            nodes.push(FlatEntry {
                id,
                bitset,
                vec: vec_data,
                inv_cached_norm,
                storage_offset,
            });
        }
        Ok(())
    }

    fn estimate_memory_bytes(&self) -> usize {
        // INVARIANT (B2b): non-`Result` introspection — recover, don't panic.
        let nodes = self.nodes.lock().unwrap_or_else(|e| e.into_inner());
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
        // INVARIANT (B2b): non-`Result` introspection — recover, don't panic.
        self.nodes.lock().unwrap_or_else(|e| e.into_inner()).len()
    }

    fn is_empty(&self) -> bool {
        // INVARIANT (B2b): non-`Result` introspection — recover, don't panic.
        self.nodes
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_empty()
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
            let _ = idx.add(
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

        let _ = idx.add(1, bs_a, VectorRepresentations::Full(vec![1.0, 0.0]), 0);
        let _ = idx.add(2, bs_b, VectorRepresentations::Full(vec![0.0, 1.0]), 0);

        let mut mask = FilterBitset::new();
        mask.set_bit(0);
        let results = idx.search(&[1.0, 0.0], &mask, 5, None, DistanceMetric::Cosine);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, 1);
    }

    #[test]
    fn test_flat_index_euclidean() {
        let idx = FlatIndex::new(DistanceMetric::Euclidean);
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

    // B2b RED: a poisoned `nodes` lock must surface as `Err` on the write
    // path (not a panic), so callers can propagate with `?`.
    #[test]
    fn add_returns_error_on_poisoned_lock() {
        let idx = FlatIndex::new(DistanceMetric::Cosine);
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = idx.nodes.lock().unwrap();
            panic!("poison the flat index lock");
        }));
        assert!(idx.nodes.is_poisoned());
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

    #[test]
    fn test_find374_add_updates_existing_entry_deduplicating_id() {
        let idx = FlatIndex::new(DistanceMetric::Cosine);
        let bs = FilterBitset::new();
        idx.add(
            1,
            bs.clone(),
            VectorRepresentations::Full(vec![1.0, 0.0]),
            0,
        )
        .unwrap();
        assert_eq!(idx.len(), 1);

        // Add again with the same id and a new vector: must update in-place, len() remains 1.
        idx.add(
            1,
            bs.clone(),
            VectorRepresentations::Full(vec![0.0, 1.0]),
            0,
        )
        .unwrap();
        assert_eq!(idx.len(), 1);

        // Searching should find id=1 with high similarity to [0.0, 1.0].
        let results = idx.search(
            &[0.0, 1.0],
            &FilterBitset::all_set(),
            5,
            None,
            DistanceMetric::Cosine,
        );
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, 1);
        assert!((results[0].1 - 1.0).abs() < 1e-4);

        // Adding with VectorRepresentations::None should remove the entry.
        idx.add(1, bs, VectorRepresentations::None, 0).unwrap();
        assert_eq!(idx.len(), 0);
        assert!(idx.is_empty());
    }
}

/// FIND-411 — the vector dimension was never validated.
///
/// The distance kernels returned `0.0` for two vectors of different lengths,
/// which is the **best** possible Euclidean score: the mismatched candidate
/// outranked every valid one. The three index paths (flat scan, HNSW over
/// the vector store, HNSW over the inline vector data) also disagreed on
/// the score of the same candidate.
#[cfg(test)]
mod dimension_mismatch_tests {
    use super::*;
    use crate::index::graph::{CPIndex, HnswConfig};
    use crate::node::{DiskNodeHeader, NodeFlags};

    /// Tolerance for comparing two Euclidean scores.
    ///
    /// FIND-411 (O2): the three index paths score a candidate with
    /// **different** formulas — the flat scan and the HNSW traversal over the
    /// vector store use the per-element difference kernel, the HNSW
    /// traversal over the inline vector data uses the cached-norms identity.
    /// They are algebraically the same distance, so they must agree within
    /// the floating-point rounding, not bit-for-bit.
    const SCORE_TOLERANCE: f32 = 1e-5;

    /// Build an index holding one 3d node (id 1) and one 5d node (id 2).
    fn mixed_dimension_index() -> CPIndex {
        let index = CPIndex::new_with_config(HnswConfig {
            distance_metric: DistanceMetric::Euclidean,
            // force the HNSW path (the flat path is exercised separately)
            flat_threshold: None,
            ..HnswConfig::default()
        });
        index
            .add(
                1,
                FilterBitset::new(),
                VectorRepresentations::Full(vec![0.0, 1.0, 0.0]),
                0,
            )
            .expect("3d node");
        index
            .add(
                2,
                FilterBitset::new(),
                VectorRepresentations::Full(vec![1.0, 0.0, 0.0, 0.0, 0.0]),
                0,
            )
            .expect("5d node");
        index
    }

    #[test]
    fn flat_scan_drops_the_dimension_mismatched_candidate() {
        let index = mixed_dimension_index();
        let query = vec![1.0, 0.0, 0.0];

        let results = flat_search(
            &index.nodes,
            &query,
            &FilterBitset::all_set(),
            5,
            DistanceMetric::Euclidean,
        );

        assert_eq!(
            results.len(),
            1,
            "the 5d node is not a candidate for a 3d query"
        );
        assert_eq!(results[0].0, 1, "the 3d node must win");
        assert!(
            !results[0].1.is_nan(),
            "the surviving score must be a real number"
        );
    }

    #[test]
    fn flat_scan_and_hnsw_agree_on_the_same_corpus() {
        let index = mixed_dimension_index();
        let query = vec![1.0, 0.0, 0.0];
        let mask = FilterBitset::all_set();

        let flat = flat_search(&index.nodes, &query, &mask, 5, DistanceMetric::Euclidean);
        let hnsw = index.search_nearest(&query, None, None, &mask, 5, None);

        assert_eq!(
            flat.iter().map(|r| r.0).collect::<Vec<_>>(),
            hnsw.iter().map(|r| r.0).collect::<Vec<_>>(),
            "the flat scan and the HNSW traversal must rank the same candidates in the same order"
        );
        assert_eq!(flat.len(), hnsw.len());
        for (f, h) in flat.iter().zip(hnsw.iter()) {
            assert!(
                (f.1 - h.1).abs() <= SCORE_TOLERANCE,
                "the flat scan ({}) and the HNSW traversal ({}) must score the same candidate alike",
                f.1,
                h.1
            );
        }
    }

    /// Minimal in-memory `VectorStoreRef`: it serves the FULL vectors of the
    /// index as raw f32 bytes, the way the vector file does.
    struct TestVectorStore {
        bytes: Vec<u8>,
        headers: Vec<(u64, DiskNodeHeader)>,
    }

    impl crate::index_port::sealed::Sealed for TestVectorStore {}

    impl crate::index_port::VectorStoreRef for TestVectorStore {
        fn read_header(&self, offset: u64) -> Option<DiskNodeHeader> {
            self.headers
                .iter()
                .find(|(off, _)| *off == offset)
                .map(|(_, header)| *header)
        }

        fn mmap_bytes(&self) -> &[u8] {
            &self.bytes
        }
    }

    fn test_store(entries: &[(u128, u64, Vec<f32>)]) -> TestVectorStore {
        let mut bytes: Vec<u8> = Vec::new();
        let mut headers = Vec::new();
        for (id, offset, vector) in entries {
            let start = bytes.len();
            for value in vector {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
            let mut header = DiskNodeHeader::new(*id);
            header.vector_offset = start as u64;
            header.vector_len = vector.len() as u32;
            header.flags = NodeFlags::VECTOR_KIND_FULL << NodeFlags::VECTOR_KIND_SHIFT;
            headers.push((*offset, header));
        }
        TestVectorStore { bytes, headers }
    }

    /// FIND-411 (O2): the HNSW traversal over the **vector store** is the
    /// third scoring formula (`src/index/search/layer.rs`) — it must agree
    /// with the flat scan and drop the dimension-mismatched candidate too.
    #[test]
    fn the_over_store_path_agrees_with_the_flat_scan() {
        let index = CPIndex::new_with_config(HnswConfig {
            distance_metric: DistanceMetric::Euclidean,
            flat_threshold: None,
            ..HnswConfig::default()
        });
        index
            .add(
                1,
                FilterBitset::new(),
                VectorRepresentations::Full(vec![0.0, 1.0, 0.0]),
                0,
            )
            .expect("3d node");
        index
            .add(
                2,
                FilterBitset::new(),
                VectorRepresentations::Full(vec![1.0, 0.0, 0.0, 0.0, 0.0]),
                64,
            )
            .expect("5d node");

        let store = test_store(&[
            (1, 0, vec![0.0, 1.0, 0.0]),
            (2, 64, vec![1.0, 0.0, 0.0, 0.0, 0.0]),
        ]);

        let query = vec![1.0, 0.0, 0.0];
        let mask = FilterBitset::all_set();

        let flat = flat_search(&index.nodes, &query, &mask, 5, DistanceMetric::Euclidean);
        let over_store = index.search_nearest(&query, None, None, &mask, 5, Some(&store));

        assert_eq!(
            flat.iter().map(|r| r.0).collect::<Vec<_>>(),
            over_store.iter().map(|r| r.0).collect::<Vec<_>>(),
            "the over-store path must rank the same candidates in the same order"
        );
        assert_eq!(flat.len(), over_store.len());
        for (f, o) in flat.iter().zip(over_store.iter()) {
            assert!(
                (f.1 - o.1).abs() <= SCORE_TOLERANCE,
                "the flat scan ({}) and the over-store path ({}) must score the same candidate alike",
                f.1,
                o.1
            );
        }
    }
}
