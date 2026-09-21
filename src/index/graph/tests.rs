//! HNSW graph tests (construction, search, repair, layer distribution).
//! Split from graph.rs (FIND-48) — re-exported via graph/mod.rs.

use super::*;
use crate::node::ALL_BITSET;
use std::collections::BinaryHeap;
use std::sync::atomic::Ordering;

// ── Miri tests for unsafe patterns ──────────────────────────────
//
// graph.rs has 3 unsafe patterns: prefetch_mmap_vector (madvise),
// release_mmap_vector (madvise), and mmap_resident_bytes (Mmap::map).
// These all require actual system calls that Miri cannot execute
// (MIRI_NO_HOST_FALLBACK=1).
//
// INSTEAD, these Miri tests exercise HNSW graph construction and
// search. This transitively covers the unsafe in distance.rs
// (chunks_exact + unwrap_unchecked — 14 blocks) through the
// insert_hnsw → search_layer → fast_similarity → distance kernel
// call chain, plus the dispatch via select_kernels().

#[cfg(miri)]
#[test]
#[ignore] // croaring (C FFI) can't run under Miri
fn miri_graph_hnsw_build_and_search() {
    let config = HnswConfig {
        m: 8,
        m_max0: 16,
        ef_construction: 50,
        ef_search: 50,
        ml: 1.0 / (8_f64).ln(),
        distance_metric: DistanceMetric::Cosine,
        flat_threshold: None, // force HNSW path
        index_type: crate::index::IndexType::Hnsw,
        auto_tune: false,
    };
    let index = CPIndex::new_with_config(config);

    // Insert vectors — this calls insert_hnsw → distance kernels
    for i in 0u128..5 {
        let v: Vec<f32> = (0..8).map(|d| ((i * 8 + d) as f32).sin()).collect();
        index
            .add(i, FilterBitset::new(), VectorRepresentations::Full(v), 0)
            .expect("test vectors are non-zero-norm");
    }
    assert_eq!(index.nodes.len(), 5);
    assert!(index.get_entry_point().is_some());

    // Search — this calls search_layer → fast_similarity → distance kernels
    let query: Vec<f32> = (0..8).map(|d| (d as f32).sin()).collect();
    let results = index.search_nearest(&query, None, None, &ALL_BITSET, 3, None);
    assert!(!results.is_empty());
    for &(id, score) in &results {
        assert!(score.is_finite(), "score for id={} should be finite", id);
    }
}

#[cfg(miri)]
#[test]
#[ignore] // croaring (C FFI) can't run under Miri
fn miri_graph_hnsw_euclidean() {
    let config = HnswConfig {
        m: 8,
        m_max0: 16,
        ef_construction: 50,
        ef_search: 50,
        ml: 1.0 / (8_f64).ln(),
        distance_metric: DistanceMetric::Euclidean,
        flat_threshold: None,
        index_type: crate::index::IndexType::Hnsw,
        auto_tune: false,
    };
    let index = CPIndex::new_with_config(config);

    // Insert points in 4D — tests the f32x8 kernels with size < 8 (sub-chunk path)
    let vectors: Vec<Vec<f32>> = vec![
        vec![1.0, 0.0, 0.0, 0.0],
        vec![0.0, 1.0, 0.0, 0.0],
        vec![0.0, 0.0, 1.0, 0.0],
        vec![0.0, 0.0, 0.0, 1.0],
    ];
    for (i, v) in vectors.iter().enumerate() {
        index
            .add(
                i as u128,
                FilterBitset::new(),
                VectorRepresentations::Full(v.clone()),
                0,
            )
            .expect("test vectors are non-zero-norm");
    }

    let query = vec![1.0, 0.0, 0.0, 0.0];
    let results = index.search_nearest(&query, None, None, &ALL_BITSET, 4, None);
    assert!(!results.is_empty());
    assert_eq!(results[0].0, 0, "identical vector should be closest");
    for &(_, score) in &results {
        assert!(score.is_finite(), "Euclidean score should be finite");
    }
}

#[cfg(miri)]
#[test]
#[ignore] // croaring (C FFI) can't run under Miri
fn miri_graph_hnsw_multiple_layers() {
    // Insert enough vectors to trigger multiple HNSW layers
    let config = HnswConfig {
        m: 4,
        m_max0: 8,
        ef_construction: 100,
        ef_search: 100,
        ml: 1.0 / (4_f64).ln(),
        distance_metric: DistanceMetric::Cosine,
        flat_threshold: None,
        index_type: crate::index::IndexType::Hnsw,
        auto_tune: false,
    };
    let index = CPIndex::new_with_config(config);

    for i in 0u128..50 {
        let v: Vec<f32> = (0..16).map(|d| ((i * 16 + d) as f32).cos()).collect();
        index
            .add(i, FilterBitset::new(), VectorRepresentations::Full(v), 0)
            .expect("test vectors are non-zero-norm");
    }
    assert_eq!(index.nodes.len(), 50);

    let query: Vec<f32> = (0..16).map(|d| (d as f32).cos()).collect();
    let results = index.search_nearest(&query, None, None, &ALL_BITSET, 5, None);
    assert_eq!(results.len(), 5);
    for &(_, score) in &results {
        assert!(score.is_finite());
    }
}

#[cfg(miri)]
#[test]
#[ignore] // croaring (C FFI) can't run under Miri
fn miri_graph_entry_point_management() {
    let index = CPIndex::new();
    assert!(index.get_entry_point().is_none());
    assert!(index.find_new_entry_point().is_none());

    // Add a node → entry point should be set
    index
        .add(
            42,
            FilterBitset::new(),
            VectorRepresentations::Full(vec![1.0, 0.0, 0.0, 0.0]),
            0,
        )
        .expect("test vector is non-zero-norm");
    assert_eq!(index.get_entry_point(), Some(42));

    // Check that we can set entry point
    index.set_entry_point(99);
    assert_eq!(index.get_entry_point(), Some(99));
}

/// Euclidean distance invariants: identical vectors → score ≈ 0.0,
/// all scores ≤ 0 (negative distance), descending order.
#[test]
fn test_euclidean_distance_metric() {
    let index = CPIndex::new_with_config(HnswConfig {
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    });

    let vectors: Vec<Vec<f32>> = vec![
        vec![1.0, 0.0, 0.0, 0.0],
        vec![0.0, 1.0, 0.0, 0.0],
        vec![0.0, 0.0, 1.0, 0.0],
        vec![0.0, 0.0, 0.0, 1.0],
    ];
    for (i, v) in vectors.iter().enumerate() {
        index
            .add(
                i as u128,
                FilterBitset::new(),
                VectorRepresentations::Full(v.clone()),
                0,
            )
            .expect("test vectors are non-zero-norm");
    }

    let query = vec![1.0, 0.0, 0.0, 0.0];
    let results = index.search_nearest(&query, None, None, &ALL_BITSET, 4, None);

    assert!(
        !results.is_empty(),
        "Euclidean search should return results"
    );

    let (closest_id, closest_score) = results[0];
    assert_eq!(
        closest_id, 0,
        "identical vector should be closest (id=0), got id={}",
        closest_id
    );
    assert!(
        closest_score.abs() < 0.01,
        "identical vector should have score ~0.0, got {}",
        closest_score
    );

    for (_id, score) in &results {
        assert!(
            *score <= 0.001,
            "Euclidean scores must be <= 0, got {}",
            score
        );
    }

    for window in results.windows(2) {
        assert!(
            window[0].1 >= window[1].1 - f32::EPSILON,
            "Euclidean scores must be descending: {} < {}",
            window[0].1,
            window[1].1
        );
    }
}

// ── AUDREP-27: zero-norm rejection ──────────────────────────────

#[test]
fn test_add_zero_norm_vector_rejected() {
    let index = CPIndex::new_with_config(HnswConfig {
        m: 8,
        m_max0: 16,
        ef_construction: 50,
        ef_search: 50,
        ml: 1.0 / (8_f64).ln(),
        distance_metric: DistanceMetric::Cosine,
        flat_threshold: None,
        index_type: crate::index::IndexType::Hnsw,
        auto_tune: false,
    });

    // Old behaviour: insert_hnsw inserted the node, then silently removed
    // it on zero norm, so `add` returned success while the node vanished.
    let err = index
        .add(
            1,
            FilterBitset::new(),
            VectorRepresentations::Full(vec![0.0, 0.0, 0.0]),
            0,
        )
        .expect_err("zero-norm vector must be rejected under cosine");
    assert!(
        matches!(err, crate::error::Error::InvalidInput(_)),
        "expected InvalidInput, got {err:?}"
    );
    // The rejection happens before any graph mutation: no node survives,
    // no total_nodes increment, no entry point left behind.
    assert_eq!(index.nodes.len(), 0);
    assert_eq!(index.total_nodes.load(Ordering::Relaxed), 0);
    assert!(index.get_entry_point().is_none());

    // A subsequent valid vector inserts normally and becomes the entry point.
    index
        .add(
            2,
            FilterBitset::new(),
            VectorRepresentations::Full(vec![1.0, 0.0, 0.0]),
            0,
        )
        .expect("valid non-zero-norm vector should insert");
    assert_eq!(index.nodes.len(), 1);
    assert_eq!(index.get_entry_point(), Some(2));
}

// ── AUD-29: NaN total ordering / eviction ───────────────────────

#[test]
fn test_nodesim_nan_total_order_evicts_extreme() {
    // NaN is pinned below every finite value in the total order, so it
    // sorts to the extreme low end and is pruned first.
    assert_eq!(total_cmp_sim(f32::NAN, f32::NAN), std::cmp::Ordering::Equal);
    assert_eq!(total_cmp_sim(f32::NAN, 0.5), std::cmp::Ordering::Less);
    assert_eq!(total_cmp_sim(0.5, f32::NAN), std::cmp::Ordering::Greater);
    assert_eq!(
        total_cmp_sim(-f32::MAX, f32::NAN),
        std::cmp::Ordering::Greater
    );
    // Finite ordering is unchanged.
    assert_eq!(total_cmp_sim(0.3, 0.5), std::cmp::Ordering::Less);

    // End-to-end: a NaN neighbour is evicted from the top-M candidate set.
    let index = CPIndex::new();
    let mut heap = BinaryHeap::new();
    heap.push(NodeSimMin(f32::NAN, 99));
    heap.push(NodeSimMin(0.9, 0));
    heap.push(NodeSimMin(0.7, 1));
    let selected = index.select_neighbors(heap, 2, |_| false);
    assert!(!selected.contains(&99), "NaN neighbour must be evicted");
    assert!(selected.contains(&0) && selected.contains(&1));
}

// ── AUD-014: deterministic tie-break / single selection path ───────

#[test]
fn test_select_neighbors_tie_break_deterministic_across_heap_orders() {
    let index = CPIndex::new();
    // Tied similarities — the only differentiator must be node id (asc).
    let candidates = [
        NodeSimMin(0.7, 30),
        NodeSimMin(0.7, 10),
        NodeSimMin(0.7, 20),
        NodeSimMin(0.3, 5),
    ];

    // Different heap push orders simulate different construction paths
    // (insert vs shrink) producing the same candidate set.
    let run = |push_order: &[usize]| {
        let mut heap = BinaryHeap::new();
        for &i in push_order {
            heap.push(candidates[i].clone());
        }
        index.select_neighbors(heap, 2, |_| false)
    };

    let a = run(&[0, 1, 2, 3]);
    let b = run(&[3, 2, 1, 0]);
    assert_eq!(
        a.as_slice(),
        &[10u128, 20u128],
        "tie-break must be by ascending node id"
    );
    assert_eq!(
        a.as_slice(),
        b.as_slice(),
        "identical candidate set must yield identical neighbor lists \
             regardless of heap push order (AUD-014)"
    );
}

#[test]
fn test_shrink_neighbors_keeps_last_inbound_over_capacity() {
    let index = CPIndex::new_with_config(HnswConfig {
        m: 2,
        m_max0: 4,
        ef_construction: 8,
        ef_search: 8,
        ml: 1.0 / (2_f64).ln(),
        distance_metric: DistanceMetric::Cosine,
        flat_threshold: None,
        index_type: crate::index::IndexType::Hnsw,
        auto_tune: false,
    });

    // A (id 0) is the shrink target; B (1) / C (2) are close to A, D (3)
    // is farthest and ranks beyond m_max; E (4) is an unrelated carrier.
    for (id, v) in [
        (0u128, vec![1.0, 0.0, 0.0]),
        (1u128, vec![0.99, 0.01, 0.0]),
        (2u128, vec![0.9, 0.1, 0.0]),
        (3u128, vec![-1.0, 0.0, 0.0]),
        (4u128, vec![0.0, 1.0, 0.0]),
    ] {
        index
            .add(id, FilterBitset::new(), VectorRepresentations::Full(v), 0)
            .expect("test vectors are non-zero-norm");
    }

    // Reset construction topology so the test fully controls inbound state.
    // inbound_count spans ALL layers, so every layer of every node must
    // be emptied before seeding the exact last-inbound scenario.
    for id in 0u128..5 {
        let layers = index.neighbor_index.num_layers(id).unwrap_or(0);
        for layer in 0..layers {
            index
                .neighbor_index
                .set_neighbors(id, layer, NeighborVec::new());
        }
    }
    // A's saturated list [B, C, D]. D's ONLY inbound reference is A's own
    // list (inbound_count == 1) → the shrink must NOT evict it.
    index
        .neighbor_index
        .set_neighbors(0, 0, NeighborVec::from_slice(&[1, 2, 3]));

    index.shrink_neighbors(0, 2, &[1, 2, 3], 0);

    let pruned = index
        .neighbor_index
        .get_neighbors(0, 0)
        .expect("A has layer 0");
    assert_eq!(
        pruned.as_slice(),
        &[1u128, 2u128, 3u128],
        "last-inbound node D must survive the shrink and keep rank order"
    );
    assert_eq!(
        pruned.len(),
        3,
        "over-capacity list is the accepted price for INV-024"
    );
}

#[test]
fn test_shrink_neighbors_evicts_non_last_inbound() {
    let index = CPIndex::new_with_config(HnswConfig {
        m: 2,
        m_max0: 4,
        ef_construction: 8,
        ef_search: 8,
        ml: 1.0 / (2_f64).ln(),
        distance_metric: DistanceMetric::Cosine,
        flat_threshold: None,
        index_type: crate::index::IndexType::Hnsw,
        auto_tune: false,
    });

    for (id, v) in [
        (0u128, vec![1.0, 0.0, 0.0]),
        (1u128, vec![0.99, 0.01, 0.0]),
        (2u128, vec![0.9, 0.1, 0.0]),
        (3u128, vec![-1.0, 0.0, 0.0]),
        (4u128, vec![0.0, 1.0, 0.0]),
    ] {
        index
            .add(id, FilterBitset::new(), VectorRepresentations::Full(v), 0)
            .expect("test vectors are non-zero-norm");
    }

    for id in 0u128..5 {
        let layers = index.neighbor_index.num_layers(id).unwrap_or(0);
        for layer in 0..layers {
            index
                .neighbor_index
                .set_neighbors(id, layer, NeighborVec::new());
        }
    }
    // Same saturated list, but D now has a SECOND inbound reference (E's
    // list) → D is evictable and must be dropped to stay within m_max.
    index
        .neighbor_index
        .set_neighbors(0, 0, NeighborVec::from_slice(&[1, 2, 3]));
    index
        .neighbor_index
        .set_neighbors(4, 0, NeighborVec::from_slice(&[3]));

    index.shrink_neighbors(0, 2, &[1, 2, 3], 0);

    let pruned = index
        .neighbor_index
        .get_neighbors(0, 0)
        .expect("A has layer 0");
    assert_eq!(
        pruned.as_slice(),
        &[1u128, 2u128],
        "non-last-inbound D must be evicted to enforce m_max"
    );
}

// ── repair_orphan_links ─────────────────────────────────────────

#[test]
fn test_repair_orphan_links_empty_index() {
    let index = CPIndex::new();
    let report = index.repair_orphan_links();
    assert_eq!(report.scanned_nodes, 0);
    assert_eq!(report.total_layers, 0);
    assert_eq!(report.repaired_links, 0);
    assert!(report.success);
}

#[test]
fn test_repair_orphan_links_no_orphans() {
    let index = CPIndex::new_with_config(HnswConfig {
        m: 4,
        m_max0: 8,
        ef_construction: 50,
        ef_search: 50,
        ml: 1.0 / (4_f64).ln(),
        distance_metric: DistanceMetric::Cosine,
        flat_threshold: None,
        index_type: crate::index::IndexType::Hnsw,
        auto_tune: false,
    });

    // Insert nodes A, B, C — they form a connected graph with no orphans
    for i in 0u128..5 {
        let v: Vec<f32> = (0..8).map(|d| ((i * 8 + d) as f32).sin()).collect();
        index
            .add(i, FilterBitset::new(), VectorRepresentations::Full(v), 0)
            .expect("test vectors are non-zero-norm");
    }

    let report = index.repair_orphan_links();
    assert!(report.scanned_nodes > 0, "should scan at least one node");
    assert_eq!(report.repaired_links, 0, "no orphans expected");
    assert!(report.success);
}

#[test]
fn test_repair_orphan_links_after_delete() {
    let index = CPIndex::new_with_config(HnswConfig {
        m: 8,
        m_max0: 16,
        ef_construction: 50,
        ef_search: 50,
        ml: 1.0 / (8_f64).ln(),
        distance_metric: DistanceMetric::Cosine,
        flat_threshold: None,
        index_type: crate::index::IndexType::Hnsw,
        auto_tune: false,
    });

    // Insert nodes 0, 1, 2 — they link to each other via HNSW
    for i in 0u128..5 {
        let v: Vec<f32> = (0..8).map(|d| ((i * 8 + d) as f32).sin()).collect();
        index
            .add(i, FilterBitset::new(), VectorRepresentations::Full(v), 0)
            .expect("test vectors are non-zero-norm");
    }
    assert_eq!(index.nodes.len(), 5);

    // Manually add orphan links: give node 0 a link to node 99 (doesn't exist)
    {
        let mut l0 = index.neighbor_index.get_neighbors(0, 0).unwrap_or_default();
        if !l0.contains(&99) {
            l0.push(99);
        }
        index.neighbor_index.set_neighbors(0, 0, l0);
    }
    // Give node 1 a link to node 999 (doesn't exist)
    {
        let mut l0 = index.neighbor_index.get_neighbors(1, 0).unwrap_or_default();
        if !l0.contains(&999) {
            l0.push(999);
        }
        index.neighbor_index.set_neighbors(1, 0, l0);
    }

    // Remove node 2 from the index entirely (simulating delete)
    let removed_node = index.nodes.remove(&2);
    assert!(removed_node.is_some(), "node 2 should exist before removal");

    // Now node 0 and node 1 both have orphan links to deleted/never-existing nodes.
    // Node 2's neighbors (which we removed the node for) can't be checked since
    // the node is gone, but other nodes that linked to node 2 now have orphan links.

    let report = index.repair_orphan_links();
    assert!(report.scanned_nodes > 0, "should scan nodes");
    assert!(
        report.repaired_links >= 2,
        "should repair at least 2 orphan links (99, 999), got {}",
        report.repaired_links
    );
    assert!(report.success);

    // Verify the orphans were actually removed
    if let Some(l0) = index.neighbor_index.get_neighbors(0, 0) {
        assert!(!l0.contains(&99), "node 0 should no longer link to 99");
    };
    if let Some(l0) = index.neighbor_index.get_neighbors(1, 0) {
        assert!(!l0.contains(&999), "node 1 should no longer link to 999");
    };
}

#[test]
fn test_repair_orphan_links_multiple_layers() {
    let index = CPIndex::new_with_config(HnswConfig {
        m: 4,
        m_max0: 8,
        ef_construction: 100,
        ef_search: 100,
        ml: 1.0 / (4_f64).ln(),
        distance_metric: DistanceMetric::Cosine,
        flat_threshold: None,
        index_type: crate::index::IndexType::Hnsw,
        auto_tune: false,
    });

    // Insert enough nodes to create multi-layer graph
    for i in 0u128..30 {
        let v: Vec<f32> = (0..16).map(|d| ((i * 16 + d) as f32).cos()).collect();
        index
            .add(i, FilterBitset::new(), VectorRepresentations::Full(v), 0)
            .expect("test vectors are non-zero-norm");
    }
    assert_eq!(index.nodes.len(), 30);

    // Inject orphan links at layers 0 and 1
    for node_id in [0u128, 5, 10] {
        let num_layers = index.neighbor_index.num_layers(node_id).unwrap_or(0);
        for layer in 0..num_layers {
            let mut l = index
                .neighbor_index
                .get_neighbors(node_id, layer)
                .unwrap_or_default();
            if !l.contains(&100) {
                l.push(100);
            }
            if !l.contains(&200) {
                l.push(200);
            }
            if !l.contains(&300) {
                l.push(300);
            }
            index.neighbor_index.set_neighbors(node_id, layer, l);
        }
    }

    // Remove some nodes to create more orphans
    for id in [15u128, 20, 25] {
        index.nodes.remove(&id);
    }

    let report = index.repair_orphan_links();
    assert!(report.scanned_nodes > 0, "should scan nodes");
    assert!(report.repaired_links > 0, "should repair orphan links");
    assert!(
        report.total_layers >= report.scanned_nodes,
        "total layers >= scanned nodes"
    );
    assert!(report.success);

    // Verify orphans are gone and legit links remain
    for node_id in [0u128, 5, 10] {
        let num_layers = index.neighbor_index.num_layers(node_id).unwrap_or(0);
        for layer in 0..num_layers {
            let l = index
                .neighbor_index
                .get_neighbors(node_id, layer)
                .unwrap_or_default();
            assert!(
                !l.contains(&100),
                "node {node_id} layer {layer} should not link to 100"
            );
            assert!(
                !l.contains(&200),
                "node {node_id} layer {layer} should not link to 200"
            );
        }
    }
}

#[test]
fn test_remove_node_decrements_inbound_and_promotes_entry_point() {
    let index = CPIndex::new_with_config(HnswConfig {
        m: 8,
        m_max0: 16,
        ef_construction: 50,
        ef_search: 50,
        ml: 1.0 / (8_f64).ln(),
        distance_metric: DistanceMetric::Cosine,
        flat_threshold: None,
        index_type: crate::index::IndexType::Hnsw,
        auto_tune: false,
    });

    for i in 0u128..6 {
        let v: Vec<f32> = (0..8).map(|d| ((i * 8 + d) as f32).sin()).collect();
        index
            .add(i, FilterBitset::new(), VectorRepresentations::Full(v), 0)
            .expect("test vectors are non-zero-norm");
    }
    assert_eq!(index.nodes.len(), 6);

    // Make node 2 the entry point so the removal path also exercises
    // entry-point promotion.
    index.set_entry_point(2);
    assert_eq!(index.get_entry_point(), Some(2));

    // ERR-012: engineer-level remove_node — the same call the storage
    // engine must use on delete (vs the old raw `nodes.remove`).
    index.remove_node(2);

    // Node payload is gone.
    assert!(index.nodes.get(&2).is_none(), "node 2 must be removed");
    // Neighbor index: no layers, and no other node's list references 2.
    assert!(
        index.neighbor_index.num_layers(2).is_none(),
        "node 2 meta must be purged from neighbor index"
    );
    let mut stale_refs: Vec<u128> = Vec::new();
    index.neighbor_index.for_each(|owner, layers| {
        for layer in layers {
            if layer.contains(&2) {
                stale_refs.push(owner);
            }
        }
    });
    assert!(
        stale_refs.is_empty(),
        "no neighbor list may reference deleted node 2, found owners {stale_refs:?}"
    );
    // Deleted node's inbound counter is evicted (no leak, bounded map).
    let inbound_after = index.neighbor_index.inbound_count(2);
    assert_eq!(inbound_after, 0, "deleted node's inbound must drop to zero");

    // Entry point must no longer point at the removed node.
    let ep = index.get_entry_point();
    assert_ne!(
        ep,
        Some(2),
        "entry point must be promoted away from deleted node"
    );
    assert!(
        ep.is_some(),
        "remaining nodes still exist, entry point must remain"
    );
}

// ── ERR-018: layer distribution ─────────────────────────────────────
// The old `random_range(0.0001..1.0)` draw truncated the geometric tail:
// with the default ml = 1/ln(32) the max achievable level was
// floor(-ln(0.0001) * ml) = 2, so graphs never grew past layer 2 and
// sparse/low-degree recall degraded. These tests prove the fixed sampler
// follows P(level >= k) = M^-k and that real inserts reach level 3+.

#[test]
fn random_layer_follows_geometric_distribution() {
    // M=4 → ml = 1/ln(4); expect P(level>=2) = 1/16, P(level>=3) = 1/64.
    let config = HnswConfig {
        m: 4,
        m_max0: 8,
        ef_construction: 100,
        ef_search: 100,
        ml: 1.0 / (4_f64).ln(),
        distance_metric: DistanceMetric::Cosine,
        flat_threshold: None,
        index_type: crate::index::IndexType::Hnsw,
        auto_tune: false,
    };
    let index = CPIndex::new_with_config(config);

    const N: usize = 2000;
    let mut hist = [0usize; 4]; // level buckets: 0, 1, 2, >=3
    for _ in 0..N {
        let lvl = index.random_layer();
        assert!(lvl < 200, "runaway level {lvl}");
        hist[lvl.min(3)] += 1;
    }
    // P(level>=2) = 6.25% → expect ~125 of 2000.
    assert!(hist[2] + hist[3] > 40, "too few level-2+ draws: {hist:?}");
    // P(level>=3) = 1.5625% → expect ~31 of 2000. Structurally 0 under
    // the old capped sampler (max level was 2).
    assert!(
        hist[3] > 5,
        "no level >= 3 draws — layer cap regression: {hist:?}"
    );
}

#[test]
fn insert_reach_layer_three() {
    let config = HnswConfig {
        m: 4,
        m_max0: 8,
        ef_construction: 20,
        ef_search: 20,
        ml: 1.0 / (4_f64).ln(),
        distance_metric: DistanceMetric::Cosine,
        flat_threshold: None, // force HNSW (default threshold brute-forces <10k nodes)
        index_type: crate::index::IndexType::Hnsw,
        auto_tune: false,
    };
    let index = CPIndex::new_with_config(config);

    // Deterministic pseudo-random vectors (LCG with fixed seed).
    let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut dims = [0f32; 16];
    let mut ge3 = 0usize; // nodes with level >= 3 (i.e. 4+ allocated layers)

    const N: u128 = 2000;
    for i in 0..N {
        for d in &mut dims {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            *d = (state >> 33) as f32 / (1u64 << 31) as f32;
        }
        index
            .add(
                i,
                FilterBitset::new(),
                VectorRepresentations::Full(dims.to_vec()),
                0,
            )
            .expect("insert should succeed");
        if index.neighbor_index.num_layers(i).unwrap_or(0) >= 4 {
            ge3 += 1;
        }
    }

    assert_eq!(index.nodes.len(), N as usize);
    // With M=4 → P(level>=3) = 1/64 ≈ 1.56% → expect ~31 of 2000.
    assert!(
        ge3 > 5,
        "too few inserts reached layer 3+ — layer cap regression: {ge3}"
    );
}
