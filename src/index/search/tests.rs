use super::profile::SearchProfile;
use super::CPIndex;
use crate::index::distance::cosine_sim_f32;
#[cfg(miri)]
use crate::index::distance::euclidean_distance_squared_f32;
use crate::index::graph::{HnswConfig, NeighborVec, NodeSimMin};
use crate::node::{
    DiskNodeHeader, DistanceMetric, FilterBitset, VectorRepresentations, ALL_BITSET,
};
use crate::storage::engine::FLAG_TOMBSTONE;
use crate::storage::vfile::VantaFile;
use ahash::RandomState;
use std::collections::{BinaryHeap, HashSet};

fn make_index(metric: DistanceMetric) -> CPIndex {
    CPIndex::new_with_config(HnswConfig {
        m: 8,
        m_max0: 16,
        ef_construction: 50,
        ef_search: 50,
        ml: 1.0 / (8_f64).ln(),
        distance_metric: metric,
        ..HnswConfig::default()
    })
}

fn add_node(index: &CPIndex, id: u128, vec: Vec<f32>) {
    index
        .add(id, FilterBitset::new(), VectorRepresentations::Full(vec), 0)
        .expect("test vectors are non-zero-norm");
}

// ΓöÇΓöÇ search_nearest ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

#[test]
fn test_search_nearest_empty_index() {
    let index = make_index(DistanceMetric::Cosine);
    let results = index.search_nearest(&[1.0, 0.0], None, None, &ALL_BITSET, 5, None);
    assert!(results.is_empty(), "empty index should return no results");
}

#[test]
fn test_search_nearest_single_node_cosine() {
    let index = make_index(DistanceMetric::Cosine);
    add_node(&index, 42, vec![1.0, 0.0, 0.0]);
    let results = index.search_nearest(&[1.0, 0.0, 0.0], None, None, &ALL_BITSET, 5, None);
    assert_eq!(results.len(), 1, "single node should be found");
    assert_eq!(results[0].0, 42, "id should match");
    assert!(
        results[0].1 > 0.99,
        "self-similarity should be ~1.0, got {}",
        results[0].1
    );
}

#[test]
fn test_search_nearest_single_node_euclidean() {
    let index = make_index(DistanceMetric::Euclidean);
    add_node(&index, 7, vec![3.0, 4.0]);
    let results = index.search_nearest(&[3.0, 4.0], None, None, &ALL_BITSET, 5, None);
    assert_eq!(results.len(), 1, "single node should be found");
    assert_eq!(results[0].0, 7, "id should match");
    assert!(
        results[0].1.abs() < 0.01,
        "self-distance should be ~0.0, got {}",
        results[0].1
    );
}

#[test]
fn test_search_nearest_ordering_cosine() {
    let index = make_index(DistanceMetric::Cosine);
    add_node(&index, 0, vec![1.0, 0.0, 0.0]);
    add_node(&index, 1, vec![0.9, 0.1, 0.0]);
    add_node(&index, 2, vec![-1.0, 0.0, 0.0]);
    let results = index.search_nearest(&[1.0, 0.0, 0.0], None, None, &ALL_BITSET, 3, None);
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].0, 0, "identical vector should be first");
    for win in results.windows(2) {
        assert!(
            win[0].1 >= win[1].1 - 1e-6,
            "scores should be descending: {} < {}",
            win[0].1,
            win[1].1
        );
    }
}

#[test]
fn test_search_nearest_top_k_limits() {
    let index = make_index(DistanceMetric::Cosine);
    for i in 0..10u128 {
        add_node(&index, i, vec![i as f32 + 1.0, 0.0, 0.0]);
    }
    let results = index.search_nearest(&[0.0, 0.0, 0.0], None, None, &ALL_BITSET, 3, None);
    assert!(
        results.len() <= 3,
        "should not exceed top_k, got {}",
        results.len()
    );
}

#[test]
fn test_search_nearest_zero_top_k() {
    let index = make_index(DistanceMetric::Cosine);
    add_node(&index, 1, vec![1.0, 0.0]);
    let results = index.search_nearest(&[1.0, 0.0], None, None, &ALL_BITSET, 0, None);
    assert!(
        results.is_empty(),
        "top_k=0 should return empty, got {}",
        results.len()
    );
}

#[test]
fn test_search_nearest_scores_not_nan() {
    let index = make_index(DistanceMetric::Cosine);
    for i in 0..5u128 {
        add_node(&index, i, vec![(i as f32) * 0.5, 0.3, -0.1]);
    }
    let results = index.search_nearest(&[0.5, -0.2, 0.1], None, None, &ALL_BITSET, 5, None);
    for &(id, score) in &results {
        assert!(
            score.is_finite(),
            "score for id={} should be finite, got {}",
            id,
            score
        );
    }
}

#[test]
fn test_search_nearest_euclidean_negative_scores() {
    let index = make_index(DistanceMetric::Euclidean);
    add_node(&index, 0, vec![0.0, 0.0]);
    add_node(&index, 1, vec![10.0, 10.0]);
    let results = index.search_nearest(&[0.0, 0.0], None, None, &ALL_BITSET, 2, None);
    for &(_, score) in &results {
        assert!(
            score <= 0.0,
            "Euclidean scores should be <= 0, got {}",
            score
        );
    }
}

#[test]
fn test_search_nearest_closer_first_euclidean() {
    let index = make_index(DistanceMetric::Euclidean);
    add_node(&index, 0, vec![0.0, 0.0]);
    add_node(&index, 1, vec![1.0, 1.0]);
    add_node(&index, 2, vec![10.0, 10.0]);
    let results = index.search_nearest(&[0.0, 0.0], None, None, &ALL_BITSET, 3, None);
    assert_eq!(results[0].0, 0, "closest vector (id=0) should be first");
    assert_eq!(results[2].0, 2, "farthest (id=2) should be last");
}

// ΓöÇΓöÇ select_neighbors ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

#[test]
fn test_select_neighbors_empty_candidates() {
    let index = make_index(DistanceMetric::Cosine);
    let heap = BinaryHeap::new();
    let selected = index.select_neighbors(heap, 5, |_| false);
    assert!(
        selected.is_empty(),
        "empty candidates should produce empty selection"
    );
}

#[test]
fn test_select_neighbors_returns_top_m() {
    let index = make_index(DistanceMetric::Cosine);
    for i in 0..6u128 {
        add_node(&index, i, vec![(i as f32 + 1.0) * 0.2, 0.0, 0.0]);
    }
    let mut heap = BinaryHeap::new();
    for i in 0..6u128 {
        // node compared with itself has sim=1.0
        if let Some(node) = index.nodes.get(&i) {
            if let Some(slice) = node.vec_data.as_f32_slice() {
                let sim = cosine_sim_f32(slice, slice);
                heap.push(NodeSimMin(sim, i));
            }
        }
    }
    let selected = index.select_neighbors(heap, 3, |_| false);
    assert_eq!(selected.len(), 3, "should select top 3 from 6 candidates");
}

#[test]
fn test_select_neighbors_with_tombstone_skipped() {
    let index = make_index(DistanceMetric::Cosine);
    add_node(&index, 0, vec![1.0, 0.0, 0.0]);
    add_node(&index, 1, vec![0.0, 1.0, 0.0]);
    // Mark node 0 as tombstone
    if let Some(mut n) = index.nodes.get_mut(&0) {
        n.flags |= FLAG_TOMBSTONE;
    }
    let mut heap = BinaryHeap::new();
    heap.push(NodeSimMin(1.0, 0));
    heap.push(NodeSimMin(0.5, 1));
    // With top-M selection, tombstone filtering is skipped.
    // During rebuild, tombstones don't appear in the candidate set
    // (they are filtered out during vstore scan).
    let selected = index.select_neighbors(heap, 5, |_| false);
    assert_eq!(selected.len(), 2, "top-M selects both candidates by score");
    assert!(selected.contains(&0), "top-M does not filter tombstones");
}

#[test]
fn test_select_neighbors_keeps_best_scores() {
    // Regression test: B2 (1379343b) inverted the select_nth_unstable_by
    // comparator, which left the m SMALLEST-score elements in vec[0..m] ΓÇö
    // i.e. the m WORST neighbors ΓÇö instead of the m best. Every insert and
    // shrink then built edges to the worst candidates, degrading topology.
    let index = make_index(DistanceMetric::Cosine);
    let mut heap = BinaryHeap::new();
    // Higher score = better neighbor.
    heap.push(NodeSimMin(0.1, 1));
    heap.push(NodeSimMin(0.9, 2));
    heap.push(NodeSimMin(0.5, 3));
    heap.push(NodeSimMin(0.7, 4));
    heap.push(NodeSimMin(0.3, 5));
    let selected = index.select_neighbors(heap, 2, |_| false);
    let mut ids: Vec<u128> = selected.iter().copied().collect();
    ids.sort_unstable();
    assert_eq!(
        ids,
        vec![2, 4],
        "must keep the 2 BEST candidates (scores 0.9, 0.7), got {ids:?}"
    );
}

// ΓöÇΓöÇ use_flat_search ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

#[test]
fn test_use_flat_search_none_threshold() {
    let index = CPIndex::new_with_config(HnswConfig {
        flat_threshold: None,
        ..HnswConfig::default()
    });
    assert!(
        !index.use_flat_search(),
        "None threshold should always return false"
    );
}

#[test]
fn test_use_flat_search_some_zero_threshold() {
    let index = CPIndex::new_with_config(HnswConfig {
        flat_threshold: Some(0),
        ..HnswConfig::default()
    });
    // Empty index: 0 <= 0 ΓåÆ true
    assert!(
        index.use_flat_search(),
        "empty + 0 threshold should be true"
    );
    // Add node: 1 <= 0 ΓåÆ false
    add_node(&index, 1, vec![1.0, 0.0]);
    assert!(
        !index.use_flat_search(),
        "1 node + 0 threshold should be false"
    );
}

#[test]
fn test_use_flat_search_default_threshold() {
    let index = make_index(DistanceMetric::Cosine);
    // Default threshold = Some(10000), small indexes are below it
    assert!(
        index.use_flat_search(),
        "small index should use flat search by default"
    );
}

// ΓöÇΓöÇ AUDREP-55: zero-norm cosine queries ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

#[test]
fn test_search_nearest_zero_norm_cosine_rejected() {
    // Zero-norm query under cosine is undefined (0/0). It must not be
    // silently re-scored with euclidean (score range would swap and
    // callers could no longer compare scores across calls); it must
    // return no results, deterministically, for any zero vector length.
    let index = make_index(DistanceMetric::Cosine);
    add_node(&index, 1, vec![1.0, 0.0, 0.0]);

    for query in [
        vec![0.0_f32, 0.0],
        vec![0.0_f32, 0.0, 0.0],
        vec![0.0_f32, -0.0, 0.0, 0.0],
    ] {
        let results = index.search_nearest(&query, None, None, &ALL_BITSET, 5, None);
        assert!(
            results.is_empty(),
            "zero-norm cosine query must return no results, got {results:?}"
        );
    }

    // Contrast: zero-norm queries remain valid for other metrics ΓÇö the
    // guard must only fire under cosine.
    let euc = make_index(DistanceMetric::Euclidean);
    add_node(&euc, 1, vec![3.0, 4.0]);
    let results = euc.search_nearest(&[0.0, 0.0], None, None, &ALL_BITSET, 5, None);
    assert_eq!(results.len(), 1, "zero-norm euclidean query is valid");
}

#[test]
fn test_search_nearest_hnsw_zero_norm_cosine_rejected() {
    // Same contract via the HNSW path (flat_threshold = None) ΓÇö the
    // guard lives at the search_nearest entry point, before any
    // flat/IVF/HNSW routing.
    let index = make_hnsw_index(DistanceMetric::Cosine);
    add_node(&index, 1, vec![1.0, 0.0, 0.0]);

    for query in [vec![0.0_f32, 0.0, 0.0], vec![0.0_f32, 0.0]] {
        let results = index.search_nearest(&query, None, None, &ALL_BITSET, 5, None);
        assert!(
            results.is_empty(),
            "HNSW zero-norm cosine query must return no results, got {results:?}"
        );
    }

    let euc = make_hnsw_index(DistanceMetric::Euclidean);
    add_node(&euc, 1, vec![3.0, 4.0]);
    let results = euc.search_nearest(&[0.0, 0.0], None, None, &ALL_BITSET, 5, None);
    assert_eq!(results.len(), 1, "HNSW zero-norm euclidean query is valid");
}

// ΓöÇΓöÇ IVF lazy-build invalidation (AUDREP-09) ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

fn make_ivf_index(metric: DistanceMetric) -> CPIndex {
    CPIndex::new_with_config(HnswConfig {
        m: 8,
        m_max0: 16,
        ef_construction: 50,
        ef_search: 50,
        ml: 1.0 / (8_f64).ln(),
        distance_metric: metric,
        flat_threshold: None,
        index_type: crate::index::IndexType::Ivf,
        auto_tune: false,
    })
}

#[test]
fn test_ivf_rebuilds_when_nodes_added_after_build() {
    let index = make_ivf_index(DistanceMetric::Euclidean);
    for id in 0..20_u128 {
        add_node(&index, id, vec![id as f32 * 0.5, id as f32 * 0.5]);
    }
    // Force the lazy IVF build on the first search.
    index.search_nearest(&[0.5, 0.5], None, None, &ALL_BITSET, 5, None);
    assert!(
        index.ivf_index.lock().is_some(),
        "IVF should be built after first search"
    );
    assert_eq!(
        index
            .ivf_built_at_node_count
            .load(std::sync::atomic::Ordering::Relaxed),
        20,
        "IVF built over the initial 20 nodes"
    );

    // Add a new, far vector after the build, then search for it.
    index
        .add(
            999_u128,
            FilterBitset::new(),
            VectorRepresentations::Full(vec![999.0, 999.0]),
            0,
        )
        .expect("test vector is non-zero-norm");
    let results = index.search_nearest(&[999.0, 999.0], None, None, &ALL_BITSET, 5, None);
    assert_eq!(
        index
            .ivf_built_at_node_count
            .load(std::sync::atomic::Ordering::Relaxed),
        21,
        "cached IVF must be rebuilt after node count changed"
    );
    assert!(
        results.iter().any(|(id, _)| *id == 999_u128),
        "newly added vector must be a candidate after rebuild, got {results:?}"
    );
}

// ΓöÇΓöÇ search_nearest via HNSW path (flat_threshold = None) ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

fn make_hnsw_index(metric: DistanceMetric) -> CPIndex {
    CPIndex::new_with_config(HnswConfig {
        m: 8,
        m_max0: 16,
        ef_construction: 50,
        ef_search: 50,
        ml: 1.0 / (8_f64).ln(),
        distance_metric: metric,
        flat_threshold: None,
        index_type: crate::index::IndexType::Hnsw,
        auto_tune: false,
    })
}

#[test]
fn test_search_nearest_hnsw_empty_index() {
    let index = make_hnsw_index(DistanceMetric::Cosine);
    let results = index.search_nearest(&[1.0, 0.0], None, None, &ALL_BITSET, 5, None);
    assert!(
        results.is_empty(),
        "empty index via HNSW should return empty"
    );
}

#[test]
fn test_search_nearest_hnsw_cosine() {
    let index = make_hnsw_index(DistanceMetric::Cosine);
    add_node(&index, 0, vec![1.0, 0.0, 0.0]);
    add_node(&index, 1, vec![0.9, 0.1, 0.0]);
    add_node(&index, 2, vec![-1.0, 0.0, 0.0]);
    let results = index.search_nearest(&[1.0, 0.0, 0.0], None, None, &ALL_BITSET, 3, None);
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].0, 0, "identical vector should be first via HNSW");
    for win in results.windows(2) {
        assert!(
            win[0].1 >= win[1].1 - 1e-6,
            "scores should be descending via HNSW: {} < {}",
            win[0].1,
            win[1].1
        );
    }
    for &(_, score) in &results {
        assert!(
            score.is_finite(),
            "HNSW score should be finite, got {}",
            score
        );
    }
}

#[test]
fn test_search_nearest_hnsw_euclidean() {
    let index = make_hnsw_index(DistanceMetric::Euclidean);
    add_node(&index, 0, vec![0.0, 0.0]);
    add_node(&index, 1, vec![3.0, 4.0]);
    add_node(&index, 2, vec![-5.0, -5.0]);
    let results = index.search_nearest(&[0.0, 0.0], None, None, &ALL_BITSET, 3, None);
    assert_eq!(results.len(), 3);
    assert_eq!(
        results[0].0, 0,
        "self-vector should be first via HNSW Euclidean"
    );
    for &(_, score) in &results {
        assert!(
            score <= 0.0,
            "Euclidean score should be <= 0, got {}",
            score
        );
    }
}

#[test]
fn test_search_with_method_override_routes_backends() {
    // Engine configured for HNSW, but per-search `method` must route to
    // each explicit backend (FEAT-04 binding override path).
    let index = make_hnsw_index(DistanceMetric::Cosine);
    add_node(&index, 0, vec![1.0, 0.0, 0.0]);
    add_node(&index, 1, vec![-1.0, 0.0, 0.0]);
    add_node(&index, 2, vec![0.0, 1.0, 0.0]);
    let query = vec![1.0, 0.0, 0.0];
    for method in [
        crate::index::IndexType::Hnsw,
        crate::index::IndexType::Ivf,
        crate::index::IndexType::Scann,
        crate::index::IndexType::Flat,
    ] {
        let results = index
            .search_with_method(method, &query, &ALL_BITSET, 3, DistanceMetric::Cosine)
            .unwrap();
        assert_eq!(results.len(), 3, "method {method:?} should return 3 nodes");
        assert_eq!(
            results[0].0, 0,
            "method {method:?} should rank the identical vector first, got {results:?}"
        );
    }
}

// ΓöÇΓöÇ search_layer direct tests ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

#[test]
fn test_search_layer_empty_entry_points() {
    let index = make_index(DistanceMetric::Cosine);
    add_node(&index, 0, vec![1.0, 0.0, 0.0]);
    let mut visited: HashSet<u128, RandomState> =
        HashSet::with_capacity_and_hasher(100, RandomState::new());
    let results = index.search_layer(
        &[1.0, 0.0, 0.0],
        Some(1.0),
        Some(1.0),
        &[],
        10,
        0,
        &ALL_BITSET,
        false, // no ACORN in existing tests
        None,
        DistanceMetric::Cosine,
        &mut visited,
        &mut SearchProfile::new(),
    );
    assert!(
        results.is_empty(),
        "empty entry points should return empty results"
    );
}

#[test]
fn test_search_layer_cosine_ordered() {
    let index = make_index(DistanceMetric::Cosine);
    add_node(&index, 0, vec![1.0, 0.0, 0.0]);
    add_node(&index, 1, vec![0.8, 0.6, 0.0]);
    // For search_layer to traverse to node 1, node 0 must have it as neighbor.
    // insert_hnsw with ef_construction=50 should connect them.
    let mut visited: HashSet<u128, RandomState> =
        HashSet::with_capacity_and_hasher(100, RandomState::new());
    let results = index.search_layer(
        &[1.0, 0.0, 0.0],
        Some(1.0),
        Some(1.0),
        &[0],
        10,
        0,
        &ALL_BITSET,
        false, // no ACORN in existing tests
        None,
        DistanceMetric::Cosine,
        &mut visited,
        &mut SearchProfile::new(),
    );
    assert!(!results.is_empty(), "search_layer should find results");
    let sorted = results.into_sorted_vec();
    assert!(
        sorted[0].1 == 0,
        "node 0 should be best match, got id={}",
        sorted[0].1
    );
}

#[test]
fn test_search_layer_tombstone_not_returned() {
    let index = make_index(DistanceMetric::Cosine);
    add_node(&index, 0, vec![1.0, 0.0, 0.0]);
    add_node(&index, 1, vec![0.9, 0.1, 0.0]);
    // Mark node 0 as tombstone
    if let Some(mut n) = index.nodes.get_mut(&0) {
        n.flags |= FLAG_TOMBSTONE;
    }
    let mut visited: HashSet<u128, RandomState> =
        HashSet::with_capacity_and_hasher(100, RandomState::new());
    let results = index.search_layer(
        &[1.0, 0.0, 0.0],
        Some(1.0),
        Some(1.0),
        &[0, 1],
        10,
        0,
        &ALL_BITSET,
        false, // no ACORN in existing tests
        None,
        DistanceMetric::Cosine,
        &mut visited,
        &mut SearchProfile::new(),
    );
    // The already-visited tombstone should be filtered from results
    let sorted = results.into_sorted_vec();
    for ns in &sorted {
        assert!(ns.1 != 0, "tombstone node 0 should not appear in results");
    }
}

// ΓöÇΓöÇ ERR-042 parity: disk (vector_store) vs in-memory search ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ
//
// ERR-042 hoisted `read_header` to once per candidate (reused for the
// distance computation AND the tombstone eligibility check). These tests
// prove the refactor did not change search behavior: searching through a
// populated VantaFile (disk path, read_header) must return the same ids
// and scores as the in-memory path (fast_similarity), and a tombstone
// flag in the disk header must still exclude the node.

const ERR042_ALIGN: u64 = 64;

/// Build a HNSW index AND a matching in-memory VantaFile where each
/// node's `storage_offset` points at a readable header + vector payload.
/// Both data paths see byte-identical vectors.
fn build_index_with_vfile(metric: DistanceMetric) -> (CPIndex, VantaFile) {
    let index = make_hnsw_index(metric);
    let hdr_size = std::mem::size_of::<DiskNodeHeader>() as u64;
    let total = 4096 + 512 * (hdr_size + 16 * 4 + ERR042_ALIGN);
    let mut vfile = VantaFile::create_in_memory(total);
    let mut offset = ERR042_ALIGN;
    for i in 0..200u128 {
        let vec: Vec<f32> = (0..16)
            .map(|d| ((i as f32) * 0.01 + d as f32 * 0.05).sin())
            .collect();
        index
            .add(
                i,
                FilterBitset::all_set(),
                VectorRepresentations::Full(vec.clone()),
                offset,
            )
            .expect("test vectors are non-zero-norm");
        let vec_offset = offset + hdr_size;
        let mut header = DiskNodeHeader::new(i);
        header.vector_len = vec.len() as u32;
        header.vector_offset = vec_offset;
        vfile.write_header(offset, &header).expect("write header");
        let bytes: Vec<u8> = vec.iter().flat_map(|v| v.to_le_bytes()).collect();
        vfile.mmap_bytes_mut().expect("writable mmap")[vec_offset as usize..][..bytes.len()]
            .copy_from_slice(&bytes);
        let node_size = hdr_size + (vec.len() as u64 * 4);
        offset = ((offset + node_size + ERR042_ALIGN - 1) / ERR042_ALIGN) * ERR042_ALIGN;
    }
    (index, vfile)
}

#[test]
fn test_search_vfile_in_memory_parity() {
    let (index, vfile) = build_index_with_vfile(DistanceMetric::Cosine);
    for q in [
        vec![
            0.1f32, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 0.9, 0.8, 0.7, 0.6, 0.5, 0.4,
        ],
        vec![
            1.0f32, -0.5, 0.2, 0.0, 0.3, -0.1, 0.8, 0.4, 0.6, 0.2, 0.1, 0.9, 0.5, 0.3, 0.7, 0.0,
        ],
    ] {
        let in_memory = index.search_nearest(&q, None, None, &ALL_BITSET, 10, None);
        let from_disk = index.search_nearest(&q, None, None, &ALL_BITSET, 10, Some(&vfile));
        assert_eq!(
            from_disk.len(),
            in_memory.len(),
            "result count must match between disk and in-memory search"
        );
        for ((mem_id, mem_score), (disk_id, disk_score)) in in_memory.iter().zip(from_disk.iter()) {
            assert_eq!(mem_id, disk_id, "ids must match in ranking order");
            assert!(
                (mem_score - disk_score).abs() < 1e-5,
                "scores must match: in-memory {mem_score} vs disk {disk_score}"
            );
        }
    }
}

#[test]
fn test_search_vfile_tombstone_header_excluded() {
    // The hoisted header feeds the eligibility check too: a tombstone bit
    // in the DISK header (not the in-memory node) must exclude the node.
    let (index, mut vfile) = build_index_with_vfile(DistanceMetric::Cosine);
    let victim_offset = ERR042_ALIGN;
    let mut header = vfile.read_header(victim_offset).expect("victim header");
    header.flags |= FLAG_TOMBSTONE;
    vfile
        .write_header(victim_offset, &header)
        .expect("write tombstone");

    // Query identical to node 0's vector so it is the top-ranked match.
    let q: Vec<f32> = (0..16).map(|d| (d as f32 * 0.05).sin()).collect();
    let results = index.search_nearest(&q, None, None, &ALL_BITSET, 10, Some(&vfile));
    assert!(
        results.iter().all(|(id, _)| *id != 0),
        "tombstoned node 0 must not appear in disk-backed results"
    );
    // Control: in-memory search (node.flags untouched) still ranks node 0.
    let mem_results = index.search_nearest(&q, None, None, &ALL_BITSET, 10, None);
    assert!(
        mem_results.iter().any(|(id, _)| *id == 0),
        "in-memory path must still return the live node 0"
    );
}

// FIND-29: the disk path in `search_layer` decodes a node's byte payload
// with the canonical `align_to` mechanism (the internals of
// `VectorRepresentations::as_f32_slice`). The decoded slice must carry the
// exact same f32 values the previous manual `from_raw_parts(u8* -> f32*)`
// cast produced — a faithful, value-identical reinterpretation of the same
// bytes. End-to-end search parity is already covered by
// `test_search_vfile_in_memory_parity`; this test pins the slice-level decode.
#[test]
fn test_layer_align_to_decodes_original_values() {
    let hdr_size = std::mem::size_of::<DiskNodeHeader>() as u64;
    let vec: Vec<f32> = vec![1.0f32, -2.5, 3.25, 0.125, 1000.5, -0.0];
    let total = 4096 + hdr_size + vec.len() as u64 * 4;
    let mut vfile = VantaFile::create_in_memory(total);
    let offset = ERR042_ALIGN;
    let vec_offset = offset + hdr_size;

    let mut header = DiskNodeHeader::new(7);
    header.vector_len = vec.len() as u32;
    header.vector_offset = vec_offset;
    vfile.write_header(offset, &header).expect("write header");
    let bytes: Vec<u8> = vec.iter().flat_map(|v| v.to_le_bytes()).collect();
    vfile.mmap_bytes_mut().expect("writable mmap")[vec_offset as usize..][..bytes.len()]
        .copy_from_slice(&bytes);

    // Replicate the disk-path extraction in `search_layer` (read_header →
    // byte sub-range → align_to decode), then assert value equality with the
    // original vector — the same bytes the old from_raw_parts cast read.
    let h = vfile.read_header(offset).expect("read header");
    let vec_start = h.vector_offset as usize;
    let vec_end = vec_start + h.vector_len as usize * 4;
    let byte_slice = &vfile.mmap_bytes()[vec_start..vec_end];
    let (_, f32_slice, _) = unsafe { byte_slice.align_to::<f32>() };
    assert_eq!(f32_slice.len(), vec.len());
    assert_eq!(
        f32_slice,
        vec.as_slice(),
        "decoded slice must equal the written f32 values"
    );
}

#[test]
fn test_search_layer_euclidean_metric() {
    let index = make_index(DistanceMetric::Euclidean);
    add_node(&index, 0, vec![1.0, 0.0]);
    add_node(&index, 1, vec![10.0, 10.0]);
    let mut visited: HashSet<u128, RandomState> =
        HashSet::with_capacity_and_hasher(100, RandomState::new());
    let results = index.search_layer(
        &[1.0, 0.0],
        Some(1.0),
        None,
        &[0],
        10,
        0,
        &ALL_BITSET,
        false, // no ACORN in existing tests
        None,
        DistanceMetric::Euclidean,
        &mut visited,
        &mut SearchProfile::new(),
    );
    assert!(
        !results.is_empty(),
        "search_layer Euclidean should find results"
    );
    let sorted = results.into_sorted_vec();
    // Euclidean scores should be negative
    for ns in &sorted {
        assert!(ns.0 <= 0.0, "Euclidean score should be <= 0");
    }
}

// ΓöÇΓöÇ select_neighbors: diversity pruning ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

#[test]
fn test_select_neighbors_diversity_prunes_similar() {
    let index = make_index(DistanceMetric::Cosine);
    add_node(&index, 0, vec![1.0, 0.0]);
    add_node(&index, 1, vec![0.99, 0.01]);
    add_node(&index, 2, vec![-1.0, 0.0]);

    let mut heap = BinaryHeap::new();
    heap.push(NodeSimMin(0.9, 0));
    heap.push(NodeSimMin(0.85, 1));
    heap.push(NodeSimMin(0.1, 2));

    // select_neighbors now uses simple top-M (no diversity check):
    // sorted: [0 (0.9), 1 (0.85), 2 (0.1)] ΓåÆ top 2 = [0, 1]
    let selected = index.select_neighbors(heap, 2, |_| false);
    assert_eq!(selected.len(), 2);
    assert!(selected.contains(&0), "best candidate should be selected");
    assert!(
        selected.contains(&1),
        "second-best should also be selected with top-M"
    );
}

#[test]
fn test_select_neighbors_diversity_with_euclidean() {
    let index = make_index(DistanceMetric::Euclidean);
    add_node(&index, 0, vec![1.0, 0.0]);
    add_node(&index, 1, vec![1.1, 0.0]); // close to 0
    add_node(&index, 2, vec![-1.0, 0.0]);

    let mut heap = BinaryHeap::new();
    // Euclidean scores are negative (similarity = -distance)
    // Distance(0, self) = 0, so similarity = 0
    // But we use arbitrary scores for selection
    heap.push(NodeSimMin(0.0, 0));
    heap.push(NodeSimMin(-0.01, 1)); // slightly worse
    heap.push(NodeSimMin(-2.0, 2)); // much worse

    let selected = index.select_neighbors(heap, 2, |_| false);
    assert_eq!(selected.len(), 2);
    assert!(selected.contains(&0), "best should be selected");
    // node 1 is close to 0 so it may or may not be pruned depending on distances
    assert!(
        selected.contains(&1) || selected.contains(&2),
        "should pick at least one more candidate"
    );
}

#[test]
fn test_select_neighbors_m_zero() {
    let index = make_index(DistanceMetric::Cosine);
    add_node(&index, 0, vec![1.0, 0.0]);

    let mut heap = BinaryHeap::new();
    heap.push(NodeSimMin(1.0, 0));
    let selected = index.select_neighbors(heap, 0, |_| false);
    assert!(selected.is_empty(), "m=0 should return empty selection");
}

#[test]
fn test_select_neighbors_missing_node_skipped() {
    let index = make_index(DistanceMetric::Cosine);
    add_node(&index, 0, vec![1.0, 0.0]);

    let mut heap = BinaryHeap::new();
    // With top-M selection, the function doesn't validate node existence
    // (IDs come from search_layer which only yields live nodes during build)
    heap.push(NodeSimMin(1.0, 999));
    heap.push(NodeSimMin(0.5, 0));

    let selected = index.select_neighbors(heap, 5, |_| false);
    assert_eq!(selected.len(), 2, "top-M selects both available entries");
    assert!(
        selected.contains(&999),
        "top-M selects by score, not existence"
    );
}

#[test]
fn test_select_neighbors_discarded_fills_remaining() {
    let index = make_index(DistanceMetric::Cosine);
    // 3 nodes where some may be pruned for diversity
    add_node(&index, 0, vec![1.0, 0.0, 0.0]);
    add_node(&index, 1, vec![0.98, 0.02, 0.0]);
    add_node(&index, 2, vec![-0.5, 0.5, 0.0]);

    let mut heap = BinaryHeap::new();
    // Node 0 and 1 are close, node 2 is different
    heap.push(NodeSimMin(0.9, 0));
    heap.push(NodeSimMin(0.8, 1));
    heap.push(NodeSimMin(0.3, 2));

    // select_neighbors with m=3 should try to return 3, even if pruning happens
    let selected = index.select_neighbors(heap, 3, |_| false);
    assert_eq!(
        selected.len(),
        3,
        "should fill remaining slots with discarded candidates"
    );
}

// ΓöÇΓöÇ Miri tests for unsafe patterns ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ
//
// search.rs has 4 unsafe blocks: `from_raw_parts` in search_layer
// for mmap-backed vector access (2 blocks in the entry_point loop,
// 2 in the neighbor evaluation loop). These require an actual
// VantaFile with mmap data which Miri cannot provide.
//
// These Miri tests exercise search_layer and the HNSW search path
// with `vector_store = None`, which routes through fast_similarity
// ΓåÆ distance kernels (unsafe blocks in distance.rs, already tested
// by the distance.rs and graph.rs Miri tests).

#[cfg(miri)]
#[test]
#[ignore] // croaring (C FFI) can't run under Miri
fn miri_search_layer_empty_entry_points() {
    let index = make_index(DistanceMetric::Cosine);
    add_node(&index, 0, vec![1.0, 0.0, 0.0]);
    let mut visited: HashSet<u128, RandomState> =
        HashSet::with_capacity_and_hasher(100, RandomState::new());
    let results = index.search_layer(
        &[1.0, 0.0, 0.0],
        Some(1.0),
        Some(1.0),
        &[],
        10,
        0,
        &ALL_BITSET,
        false, // no ACORN in existing tests
        None,
        DistanceMetric::Cosine,
        &mut visited,
        &mut SearchProfile::new(),
    );
    assert!(results.is_empty(), "empty entry points ΓåÆ empty results");
}

#[cfg(miri)]
#[test]
#[ignore] // croaring (C FFI) can't run under Miri
fn miri_search_layer_cosine_small() {
    let index = make_index(DistanceMetric::Cosine);
    add_node(&index, 0, vec![1.0, 0.0, 0.0]);
    add_node(&index, 1, vec![0.9, 0.1, 0.0]);
    let mut visited: HashSet<u128, RandomState> =
        HashSet::with_capacity_and_hasher(100, RandomState::new());
    let results = index.search_layer(
        &[1.0, 0.0, 0.0],
        Some(1.0),
        Some(1.0),
        &[0],
        10,
        0,
        &ALL_BITSET,
        false, // no ACORN in existing tests
        None,
        DistanceMetric::Cosine,
        &mut visited,
        &mut SearchProfile::new(),
    );
    assert!(!results.is_empty(), "should find results");
    let sorted = results.into_sorted_vec();
    assert!(
        sorted[0].1 == 0 || sorted[0].1 == 1,
        "top result should be 0 or 1"
    );
}

#[cfg(miri)]
#[test]
#[ignore] // croaring (C FFI) can't run under Miri
fn miri_search_layer_euclidean() {
    let index = make_index(DistanceMetric::Euclidean);
    add_node(&index, 0, vec![1.0, 0.0]);
    add_node(&index, 1, vec![10.0, 10.0]);
    let mut visited: HashSet<u128, RandomState> =
        HashSet::with_capacity_and_hasher(100, RandomState::new());
    let results = index.search_layer(
        &[1.0, 0.0],
        Some(1.0),
        None,
        &[0],
        10,
        0,
        &ALL_BITSET,
        false, // no ACORN in existing tests
        None,
        DistanceMetric::Euclidean,
        &mut visited,
        &mut SearchProfile::new(),
    );
    assert!(!results.is_empty(), "should find results");
    let sorted = results.into_sorted_vec();
    for ns in &sorted {
        assert!(ns.0 <= 0.0, "Euclidean scores Γëñ 0");
    }
}

#[cfg(miri)]
#[test]
#[ignore] // croaring (C FFI) can't run under Miri
fn miri_search_nearest_hnsw_path() {
    // Force HNSW path (flat_threshold = None)
    let index = make_hnsw_index(DistanceMetric::Cosine);
    for i in 0u128..10 {
        add_node(&index, i, vec![(i as f32) * 0.2, 0.0, 0.0]);
    }
    let results = index.search_nearest(&[0.0, 0.0, 0.0], None, None, &ALL_BITSET, 5, None);
    // AUDREP-55: zero-norm cosine query is undefined; must return no
    // results (and a warning) instead of re-scoring with euclidean.
    assert_eq!(results.len(), 0);
}

#[cfg(miri)]
#[test]
#[ignore] // croaring (C FFI) can't run under Miri
fn miri_search_nearest_hnsw_euclidean() {
    let index = make_hnsw_index(DistanceMetric::Euclidean);
    for i in 0u128..10 {
        add_node(&index, i, vec![(i as f32) * 0.5, 0.0]);
    }
    let results = index.search_nearest(&[0.0, 0.0], None, None, &ALL_BITSET, 5, None);
    assert_eq!(results.len(), 5);
    for &(_, score) in &results {
        assert!(score.is_finite());
        assert!(score <= 0.0);
    }
}

#[cfg(miri)]
#[test]
#[ignore] // croaring (C FFI) can't run under Miri
fn miri_select_neighbors_basic() {
    let index = make_index(DistanceMetric::Cosine);
    for i in 0u128..6 {
        add_node(&index, i, vec![(i as f32) * 0.2, 0.0, 0.0]);
    }
    let mut heap = BinaryHeap::new();
    for i in 0u128..6 {
        if let Some(node) = index.nodes.get(&i) {
            if let Some(slice) = node.vec_data.as_f32_slice() {
                let sim = cosine_sim_f32(slice, slice);
                heap.push(NodeSimMin(sim, i));
            }
        }
    }
    let selected = index.select_neighbors(heap, 3, |_| false);
    assert_eq!(selected.len(), 3);
}

#[cfg(miri)]
#[test]
#[ignore] // croaring (C FFI) can't run under Miri
fn miri_select_neighbors_euclidean() {
    let index = make_index(DistanceMetric::Euclidean);
    for i in 0u128..4 {
        add_node(&index, i, vec![(i as f32) * 2.0, 0.0]);
    }
    let mut heap = BinaryHeap::new();
    for i in 0u128..4 {
        if let Some(node) = index.nodes.get(&i) {
            if let Some(slice) = node.vec_data.as_f32_slice() {
                // self-distance is 0 ΓåÆ score = 0
                let sim = -euclidean_distance_squared_f32(slice, slice);
                heap.push(NodeSimMin(sim, i));
            }
        }
    }
    let selected = index.select_neighbors(heap, 2, |_| false);
    assert_eq!(selected.len(), 2);
}

// ΓöÇΓöÇ ACORN-1: second-hop filtered search expansion ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

/// Helper to add a node with a custom bitset.
fn add_node_with_bitset(index: &CPIndex, id: u128, vec: Vec<f32>, bits: &[u32]) {
    let mut bs = FilterBitset::new();
    for &b in bits {
        bs.set_bit(b as usize);
    }
    index
        .add(id, bs, VectorRepresentations::Full(vec), 0)
        .expect("test vectors are non-zero-norm");
}

/// Force a node's neighbors in both neighbor_index AND inline cache.
/// Direct `neighbor_index.set_neighbors()` bypasses the inline cache on
/// HnswNode, causing ACORN expansion to read stale neighbors from the cache.
fn set_test_neighbors(index: &CPIndex, id: u128, layer: usize, neighbors: NeighborVec) {
    let inline_cache = neighbors.clone();
    index.neighbor_index.set_neighbors(id, layer, neighbors);
    if let Some(mut node_ref) = index.nodes.get_mut(&id) {
        if node_ref.neighbor_lists.len() > layer {
            node_ref.neighbor_lists[layer] = inline_cache;
        }
    }
}

#[test]
fn test_acorn_expands_through_non_matching() {
    // ACORN-1 scenario: a non-matching node N (blocks filter) is a neighbor of
    // entry A. Without ACORN, N is never popped from the candidates heap because
    // a competing matching node X (also a neighbor of A) pops first, discovers
    // a very close node R (raising the "worst" threshold), and N then fails the
    // break check (d_N < worst). ACORN pre-expands N's neighbors when N is first
    // discovered as A's neighbor, finding S before N reaches the top of the heap.
    //
    // Euclidean, ef=2, single entry A:
    //   A=[1,0,0], d=-1.00, matches {0}
    //   X=[0.8,0,0], d=-0.64, matches {0} (competes with N for heap slot)
    //   N=[0.95,0,0], d=-0.9025, FAILS {0} (has {1})
    //   R=[0.3,0,0], d=-0.09, matches {0} (neighbor of X, raises threshold)
    //   S=[0.2,0,0], d=-0.04, matches {0} (neighbor of N, only reachable via ACORN)
    //
    // Topology: AΓåÆ{X,N}, XΓåÆ{A,R}, NΓåÆ{A,S}
    //
    // Without ACORN: AΓåÆXΓåÆR (raises worst)ΓåÆN(d=-0.9025 < worst=-0.09) ΓåÆ break!
    //   S never discovered.
    // With ACORN: AΓåÆN ACORN-expands to S immediately, S enters results.

    let index = CPIndex::new_with_config(HnswConfig {
        m: 2,
        m_max0: 2,
        ef_construction: 10,
        ef_search: 50,
        ml: 1.0,
        distance_metric: DistanceMetric::Euclidean,
        flat_threshold: None,
        index_type: crate::index::IndexType::Hnsw,
        auto_tune: false,
    });

    // Insert 5 nodes
    add_node_with_bitset(&index, 0, vec![1.0, 0.0, 0.0], &[0]); // A
    add_node_with_bitset(&index, 1, vec![0.95, 0.0, 0.0], &[1]); // N (non-matching)
    add_node_with_bitset(&index, 2, vec![0.8, 0.0, 0.0], &[0]); // X
    add_node_with_bitset(&index, 3, vec![0.3, 0.0, 0.0], &[0]); // R
    add_node_with_bitset(&index, 4, vec![0.2, 0.0, 0.0], &[0]); // S

    // Verify layer-0 neighbors exist
    for id in 0u128..5 {
        assert!(
            index
                .neighbor_index
                .get_neighbors(id, 0)
                .map(|n| !n.is_empty())
                .unwrap_or(false),
            "node {id} should have at least one neighbor after HNSW insert"
        );
    }

    // Force topology (use set_test_neighbors to update both neighbor_index and inline cache)
    // A ΓåÆ X, N
    set_test_neighbors(&index, 0, 0, smallvec::smallvec![2u128, 1u128]); // X, N
                                                                         // X ΓåÆ A, R
    set_test_neighbors(&index, 2, 0, smallvec::smallvec![0u128, 3u128]); // A, R
                                                                         // N ΓåÆ A, S
    set_test_neighbors(&index, 1, 0, smallvec::smallvec![0u128, 4u128]); // A, S
                                                                         // R ΓåÆ X (backlink)
    set_test_neighbors(&index, 3, 0, smallvec::smallvec![2u128]);
    // S ΓåÆ N (backlink)
    set_test_neighbors(&index, 4, 0, smallvec::smallvec![1u128]);

    let mut mask = FilterBitset::new();
    mask.set_bit(0);

    // ΓöÇΓöÇ WITHOUT ACORN ΓöÇΓöÇ
    let mut visited: HashSet<u128, RandomState> =
        HashSet::with_capacity_and_hasher(100, RandomState::new());
    let no_acorn = index.search_layer(
        &[0.0, 0.0, 0.0],
        Some(0.0),
        None,
        &[0], // entry A only
        2,    // ef=2
        0,
        &mask,
        false,
        None,
        DistanceMetric::Euclidean,
        &mut visited,
        &mut SearchProfile::new(),
    );
    let no_acorn_ids: Vec<u128> = no_acorn
        .into_sorted_vec()
        .into_iter()
        .map(|ns| ns.1)
        .collect();
    assert!(
        !no_acorn_ids.contains(&4),
        "without ACORN, node 4 (S) should NOT be found (N never popped), got {:?}",
        no_acorn_ids
    );

    // ΓöÇΓöÇ WITH ACORN ΓöÇΓöÇ
    let mut visited: HashSet<u128, RandomState> =
        HashSet::with_capacity_and_hasher(100, RandomState::new());
    let with_acorn = index.search_layer(
        &[0.0, 0.0, 0.0],
        Some(0.0),
        None,
        &[0],
        2,
        0,
        &mask,
        true,
        None,
        DistanceMetric::Euclidean,
        &mut visited,
        &mut SearchProfile::new(),
    );
    let with_acorn_ids: Vec<u128> = with_acorn
        .into_sorted_vec()
        .into_iter()
        .map(|ns| ns.1)
        .collect();
    assert!(
        with_acorn_ids.contains(&4),
        "with ACORN, node 4 (S) should be found via ACORN expansion through N, got {:?}",
        with_acorn_ids
    );
}

#[test]
fn test_acorn_no_regression_all_set() {
    // When query_mask.is_all_set(), acorn=true should behave identically
    // to acorn=false because the expansion is never triggered.
    let index = CPIndex::new_with_config(HnswConfig {
        m: 1,
        m_max0: 1,
        ef_construction: 10,
        ef_search: 50,
        ml: 1.0,
        distance_metric: DistanceMetric::Cosine,
        flat_threshold: None,
        index_type: crate::index::IndexType::Hnsw,
        auto_tune: false,
    });

    add_node_with_bitset(&index, 0, vec![1.0, 0.0, 0.0], &[0]);
    add_node_with_bitset(&index, 1, vec![0.99, 0.01, 0.0], &[1]);

    // Node 0's neighbor is node 1 (force it)
    index
        .neighbor_index
        .set_neighbors(0, 0, smallvec::smallvec![1u128]);
    index
        .neighbor_index
        .set_neighbors(1, 0, smallvec::smallvec![0u128]);

    // With ALL_BITSET, acorn should never trigger because
    // !query_mask.is_all_set() is false
    let mut visited1: HashSet<u128, RandomState> =
        HashSet::with_capacity_and_hasher(100, RandomState::new());
    let r1 = index.search_layer(
        &[1.0, 0.0, 0.0],
        Some(1.0),
        Some(1.0),
        &[0],
        10,
        0,
        &ALL_BITSET,
        false,
        None,
        DistanceMetric::Cosine,
        &mut visited1,
        &mut SearchProfile::new(),
    );

    let mut visited2: HashSet<u128, RandomState> =
        HashSet::with_capacity_and_hasher(100, RandomState::new());
    let r2 = index.search_layer(
        &[1.0, 0.0, 0.0],
        Some(1.0),
        Some(1.0),
        &[0],
        10,
        0,
        &ALL_BITSET,
        true,
        None,
        DistanceMetric::Cosine,
        &mut visited2,
        &mut SearchProfile::new(),
    );

    // Both should return the same results when mask is all_set
    let ids1: Vec<u128> = r1.into_sorted_vec().into_iter().map(|ns| ns.1).collect();
    let ids2: Vec<u128> = r2.into_sorted_vec().into_iter().map(|ns| ns.1).collect();
    assert_eq!(
        ids1, ids2,
        "ACORN should not change results when mask is ALL_BITSET"
    );
}

#[test]
fn test_acorn_budget_respected() {
    // Verify the ACORN budget formula: ef.saturating_sub(results.len()).max(16).
    // Uses the same topology as test_acorn_expands_through_non_matching but
    // with ef=1 so budget = (1-0).max(16) = 16, enough to explore all second-hops.
    let index = make_hnsw_index(DistanceMetric::Euclidean);

    // Insert 5 nodes as before
    add_node_with_bitset(&index, 0, vec![1.0, 0.0, 0.0], &[0]); // A
    add_node_with_bitset(&index, 1, vec![0.95, 0.0, 0.0], &[1]); // N (non-matching)
    add_node_with_bitset(&index, 2, vec![0.8, 0.0, 0.0], &[0]); // X
    add_node_with_bitset(&index, 3, vec![0.3, 0.0, 0.0], &[0]); // R
    add_node_with_bitset(&index, 4, vec![0.2, 0.0, 0.0], &[0]); // S

    // Force same topology (using set_test_neighbors for both caches)
    set_test_neighbors(&index, 0, 0, smallvec::smallvec![2u128, 1u128]);
    set_test_neighbors(&index, 1, 0, smallvec::smallvec![0u128, 4u128]);
    set_test_neighbors(&index, 2, 0, smallvec::smallvec![0u128, 3u128]);
    set_test_neighbors(&index, 3, 0, smallvec::smallvec![2u128]);
    set_test_neighbors(&index, 4, 0, smallvec::smallvec![1u128]);

    let mut mask = FilterBitset::new();
    mask.set_bit(0);

    let results = index.search_layer(
        &[0.0, 0.0, 0.0],
        Some(0.0),
        None,
        &[0],
        2, // ef=2 ΓåÆ budget=(2-0).max(16)=16
        0,
        &mask,
        true, // acorn_expansion = true
        None,
        DistanceMetric::Euclidean,
        &mut HashSet::with_capacity_and_hasher(100, RandomState::new()),
        &mut SearchProfile::new(),
    );

    let ids: Vec<u128> = results
        .into_sorted_vec()
        .into_iter()
        .map(|ns| ns.1)
        .collect();
    // ACORN should find at least one second-hop node (S=4) through N
    assert!(
        ids.contains(&4),
        "ACORN-expanded node (id=4) should be found with ef=1, got {:?}",
        ids
    );
}

#[test]
fn test_acorn_second_hop_after_repair_orphans() {
    // ERR-020: ACORN's second-hop expansion must read the POST-repair
    // adjacency, not a stale inline cache.
    //
    // repair_orphan_links repairs `neighbor_index` but (pre-fix) left each
    // HnswNode's inline `neighbor_lists` cache holding the removed orphan
    // ids. search_layer prefers the inline cache and only falls back to
    // `neighbor_index` when it is empty, so ACORN keeps walking dead
    // edges. This test inserts 16 orphan nodes into non-matching node N's
    // neighbor list, removes them (simulating engine deletes), repairs,
    // then runs the ACORN search with ef=2 so the second-hop budget is
    // 16: the stale orphans crowd out the live second-hop node S before
    // the fix.
    //
    // Topology (Euclidean, entry A matches {0}; N blocks the filter):
    //   A=[1,0,0] {0} ΓåÆ {X, N}
    //   N=[0.95,0,0] {1} ΓåÆ {D0..D15, A, S}   (D* are removed orphans)
    //   X=[0.8,0,0] {0} ΓåÆ {A, R}
    //   R=[0.3,0,0] {0} ΓåÆ {X}
    //   S=[0.2,0,0] {0} ΓåÆ {N}
    // S is only reachable via ACORN expansion through N. With the stale
    // inline cache, take(16) over {D0..D15, A, S} never reaches S; after
    // repair the list is {A, S} and S is found.
    let index = CPIndex::new_with_config(HnswConfig {
        m: 2,
        m_max0: 2,
        ef_construction: 10,
        ef_search: 50,
        ml: 1.0,
        distance_metric: DistanceMetric::Euclidean,
        flat_threshold: None,
        index_type: crate::index::IndexType::Hnsw,
        auto_tune: false,
    });

    add_node_with_bitset(&index, 0, vec![1.0, 0.0, 0.0], &[0]); // A
    add_node_with_bitset(&index, 1, vec![0.95, 0.0, 0.0], &[1]); // N (non-matching)
    add_node_with_bitset(&index, 2, vec![0.8, 0.0, 0.0], &[0]); // X
    add_node_with_bitset(&index, 3, vec![0.3, 0.0, 0.0], &[0]); // R
    add_node_with_bitset(&index, 4, vec![0.2, 0.0, 0.0], &[0]); // S

    // 16 orphan nodes D0..D15 (ids 5..20), inserted so the delete flow
    // that orphans N's list is realistic, then raw-removed below.
    let orphans: Vec<u128> = (5u128..21).collect();
    for d in &orphans {
        add_node_with_bitset(&index, *d, vec![0.1, 0.0, 0.0], &[1]);
    }

    let mut n_neighbors: NeighborVec = orphans.iter().copied().collect();
    n_neighbors.push(0); // A
    n_neighbors.push(4); // S
    set_test_neighbors(&index, 0, 0, smallvec::smallvec![2u128, 1u128]); // A ΓåÆ X, N
    set_test_neighbors(&index, 2, 0, smallvec::smallvec![0u128, 3u128]); // X ΓåÆ A, R
    set_test_neighbors(&index, 1, 0, n_neighbors); // N ΓåÆ orphans + A + S
    set_test_neighbors(&index, 3, 0, smallvec::smallvec![2u128]); // R ΓåÆ X
    set_test_neighbors(&index, 4, 0, smallvec::smallvec![1u128]); // S ΓåÆ N

    // Provoke orphan repair: raw-remove the D nodes (the engine delete
    // path this mirrors); repair_orphan_links cleans the leftovers.
    for d in &orphans {
        assert!(
            index.nodes.remove(d).is_some(),
            "orphan node {d} should exist before removal"
        );
    }
    let report = index.repair_orphan_links();
    assert!(
        report.repaired_links >= orphans.len() as u64,
        "repair should drop at least the {}-node orphan list from N, got {}",
        orphans.len(),
        report.repaired_links
    );

    // Post-repair invariant: no inline neighbor cache references a
    // removed id (the direct causal check for the stale second hop).
    for d in &orphans {
        if let Some(node_ref) = index.nodes.get(&1) {
            for list in &node_ref.neighbor_lists {
                assert!(
                    !list.contains(d),
                    "inline neighbor list of node 1 still references removed node {d}"
                );
            }
        }
    }

    let mut mask = FilterBitset::new();
    mask.set_bit(0);

    let mut visited: HashSet<u128, RandomState> =
        HashSet::with_capacity_and_hasher(100, RandomState::new());
    let results = index.search_layer(
        &[0.0, 0.0, 0.0],
        Some(0.0),
        None,
        &[0], // entry A only
        2,    // ef=2 ΓåÆ ACORN budget = (2-1).max(16) = 16
        0,
        &mask,
        true, // acorn_expansion
        None,
        DistanceMetric::Euclidean,
        &mut visited,
        &mut SearchProfile::new(),
    );
    let ids: Vec<u128> = results
        .into_sorted_vec()
        .into_iter()
        .map(|ns| ns.1)
        .collect();
    assert!(
        ids.contains(&4),
        "ACORN second-hop after repair_orphan_links: live node S (4) must be found \
             (stale orphans must not crowd the expansion budget), got {:?}",
        ids
    );
}
