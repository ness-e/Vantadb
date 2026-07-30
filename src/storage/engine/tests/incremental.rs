//! Tests for InsertMode (Incremental / Auto / Rebuild) in batch_insert_with_opts
//! and SDK put_batch.
//!
//! Covers 7 test scenarios:
//!   1–4: engine-level batch_insert_with_opts with InsertMode variants
//!   5–6: SDK-level put_batch with small and large batches
//!   7:   recall@10 parity between Incremental and Rebuild modes

use super::super::*;
use super::in_memory_engine;
use crate::backend::BackendKind;
use crate::config::VantaConfig;
use crate::index::VecIndex;
use crate::node::{DistanceMetric, UnifiedNode, ALL_BITSET};
use crate::sdk::{VantaEmbedded, VantaMemoryInput};
use crate::storage::engine::{BatchInsertOptions, InsertMode};

const DIMS: usize = 8;

// ─── Helpers ─────────────────────────────────────────────────────

fn make_vector(id: u128, dims: usize) -> Vec<f32> {
    (0..dims)
        .map(|d| ((id as f64 * 0.713 + d as f64 * 1.618) % 1.0) as f32)
        .collect()
}

fn make_node(id: u128, dims: usize) -> UnifiedNode {
    let mut node = UnifiedNode::new(id);
    node.vector = crate::node::VectorRepresentations::Full(make_vector(id, dims));
    node
}

fn search_hnsw(engine: &StorageEngine, query: &[f32], top_k: usize) -> Vec<(u128, f32)> {
    let idx = engine.vec_index();
    idx.search(query, &ALL_BITSET, top_k, None, DistanceMetric::Cosine)
}

fn assert_search_finds_any(engine: &StorageEngine, nodes: &[UnifiedNode]) {
    let mid = nodes.len() / 2;
    if let Some(vec) = nodes[mid].vector.as_f32_slice() {
        let results = search_hnsw(engine, vec, 10);
        assert!(
            !results.is_empty(),
            "Search should return results for node {}",
            nodes[mid].id
        );
    }
}

/// Compute recall@10: for each of the first `n` nodes, query with its vector
/// and check whether its own node_id appears in the top 10 results.
fn recall_at_10(engine: &StorageEngine, n: usize) -> f64 {
    let mut hits = 0u64;
    for i in 0..n {
        let vec = make_vector(i as u128, DIMS);
        let results = search_hnsw(engine, &vec, 10);
        if results.iter().any(|(id, _)| *id == i as u128) {
            hits += 1;
        }
    }
    hits as f64 / n.max(1) as f64
}

// ─── Test 1: Small batch, Auto mode → incremental → searchable ──

#[test]
fn test_incremental_small_batch_auto() {
    let engine = in_memory_engine();
    let nodes: Vec<UnifiedNode> = (0..50).map(|i| make_node(i, DIMS)).collect();

    // Auto mode with default threshold (1000): 50 < 1000 → Incremental
    engine
        .batch_insert_with_opts(&nodes, BatchInsertOptions::default())
        .expect("batch_insert_with_opts");

    assert!(
        engine.vec_index().len() > 0,
        "HNSW should have nodes after incremental insert (Auto, batch < threshold)"
    );
    assert_search_finds_any(&engine, &nodes);
}

// ─── Test 2: Large batch, Auto mode → rebuild needed ────────────

#[test]
fn test_incremental_large_batch_auto() {
    let engine = in_memory_engine();
    let nodes: Vec<UnifiedNode> = (0..2000).map(|i| make_node(i, DIMS)).collect();

    // 2000 >= 1000 → Auto chooses Rebuild → no HNSW insertion
    engine
        .batch_insert_with_opts(&nodes, BatchInsertOptions::default())
        .expect("batch_insert_with_opts");

    assert_eq!(
        engine.vec_index().len(),
        0,
        "HNSW should be empty after large batch with Auto mode (no rebuild called)"
    );

    // Rebuild explicitly
    let report = engine.rebuild_vector_index().expect("rebuild_vector_index");
    assert!(
        report.scanned_nodes > 0,
        "Rebuild should scan inserted nodes, got {}",
        report.scanned_nodes
    );

    // Now HNSW should have nodes and search should find them
    assert!(
        engine.vec_index().len() > 0,
        "HNSW should have nodes after rebuild"
    );
    assert_search_finds_any(&engine, &nodes);
}

// ─── Test 3: InsertMode::Incremental explicitly ─────────────────

#[test]
fn test_incremental_explicit_incremental() {
    let engine = in_memory_engine();
    let nodes: Vec<UnifiedNode> = (0..100).map(|i| make_node(i, DIMS)).collect();

    engine
        .batch_insert_with_opts(
            &nodes,
            BatchInsertOptions {
                insert_mode: InsertMode::Incremental,
                ..Default::default()
            },
        )
        .expect("batch_insert_with_opts");

    // Nodes should be searchable immediately (no rebuild needed)
    assert!(
        engine.vec_index().len() > 0,
        "HNSW should have nodes after incremental insert"
    );
    assert_search_finds_any(&engine, &nodes);
}

// ─── Test 4: InsertMode::Rebuild explicitly ─────────────────────

#[test]
fn test_incremental_explicit_rebuild() {
    let engine = in_memory_engine();
    let nodes: Vec<UnifiedNode> = (0..100).map(|i| make_node(i, DIMS)).collect();

    engine
        .batch_insert_with_opts(
            &nodes,
            BatchInsertOptions {
                insert_mode: InsertMode::Rebuild,
                ..Default::default()
            },
        )
        .expect("batch_insert_with_opts");

    // Before rebuild, HNSW should be empty
    assert_eq!(
        engine.vec_index().len(),
        0,
        "HNSW should be empty before rebuild_vector_index"
    );

    let results_before = search_hnsw(&engine, &make_vector(0, DIMS), 10);
    assert!(
        results_before.is_empty(),
        "Search should return nothing before rebuild"
    );

    // Rebuild
    engine.rebuild_vector_index().expect("rebuild_vector_index");

    // After rebuild, HNSW should have nodes and search should find them
    assert!(
        engine.vec_index().len() > 0,
        "HNSW should have nodes after rebuild"
    );
    assert_search_finds_any(&engine, &nodes);
}

// ─── Test 5: put_batch with small number of records ─────────────

#[test]
fn test_incremental_put_batch_small() {
    let db = VantaEmbedded::open_with_config(VantaConfig {
        backend_kind: BackendKind::InMemory,
        ..Default::default()
    })
    .expect("open VantaEmbedded");

    let n = 50;
    let inputs: Vec<VantaMemoryInput> = (0..n)
        .map(|i| {
            let mut input =
                VantaMemoryInput::new("inc_test", format!("key_{}", i), format!("payload_{}", i));
            input.vector = Some(make_vector(i as u128, DIMS));
            input
        })
        .collect();

    let records = db.put_batch(inputs).expect("put_batch");
    assert_eq!(
        records.len(),
        n,
        "All {} records should be returned from put_batch",
        n
    );

    // Search should find results after put_batch
    let hits = db
        .search_vector(&make_vector(0, DIMS), 10)
        .expect("search_vector");
    assert!(
        !hits.is_empty(),
        "Search should return at least one result after put_batch ({} records)",
        n
    );
}

// ─── Test 6: put_batch with large number of records ─────────────

#[test]
fn test_incremental_put_batch_large() {
    let db = VantaEmbedded::open_with_config(VantaConfig {
        backend_kind: BackendKind::InMemory,
        ..Default::default()
    })
    .expect("open VantaEmbedded");

    let n = 1500;
    let inputs: Vec<VantaMemoryInput> = (0..n)
        .map(|i| {
            let mut input =
                VantaMemoryInput::new("inc_test", format!("key_{}", i), format!("payload_{}", i));
            input.vector = Some(make_vector(i as u128, DIMS));
            input
        })
        .collect();

    let records = db.put_batch(inputs).expect("put_batch");
    assert_eq!(
        records.len(),
        n,
        "All {} records should be returned from put_batch",
        n
    );

    // Search should find results
    let hits = db
        .search_vector(&make_vector(0, DIMS), 10)
        .expect("search_vector");
    assert!(
        !hits.is_empty(),
        "Search should return at least one result after put_batch ({} records)",
        n
    );
}

// ─── Test 7: Recall parity between Incremental and Rebuild ──────

#[test]
fn test_incremental_recall_parity() {
    let n = 500usize;

    // ── Build Incremental index ──
    let engine_inc = in_memory_engine();
    let nodes_inc: Vec<UnifiedNode> = (0..n as u128).map(|i| make_node(i, DIMS)).collect();
    engine_inc
        .batch_insert_with_opts(
            &nodes_inc,
            BatchInsertOptions {
                insert_mode: InsertMode::Incremental,
                ..Default::default()
            },
        )
        .expect("incremental insert");
    assert!(
        engine_inc.vec_index().len() > 0,
        "Incremental HNSW should have nodes"
    );

    // ── Build Rebuild index ──
    let engine_rebuild = in_memory_engine();
    let nodes_rebuild: Vec<UnifiedNode> = (0..n as u128).map(|i| make_node(i, DIMS)).collect();
    engine_rebuild
        .batch_insert_with_opts(
            &nodes_rebuild,
            BatchInsertOptions {
                insert_mode: InsertMode::Rebuild,
                ..Default::default()
            },
        )
        .expect("rebuild insert");
    assert_eq!(
        engine_rebuild.vec_index().len(),
        0,
        "Rebuild HNSW should be empty before rebuild_vector_index"
    );
    engine_rebuild
        .rebuild_vector_index()
        .expect("rebuild_vector_index");
    assert!(
        engine_rebuild.vec_index().len() > 0,
        "Rebuild HNSW should have nodes after rebuild_vector_index"
    );

    // ── Compute recall@10 for both indexes ──
    let recall_inc = recall_at_10(&engine_inc, n);
    let recall_rebuild = recall_at_10(&engine_rebuild, n);

    assert!(
        recall_inc > 0.95,
        "Incremental recall@10: {:.3} (expected > 0.95)",
        recall_inc
    );
    assert!(
        recall_rebuild > 0.95,
        "Rebuild recall@10: {:.3} (expected > 0.95)",
        recall_rebuild
    );
}
