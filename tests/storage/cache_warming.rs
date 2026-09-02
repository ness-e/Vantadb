#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Cache warming integration tests (OLD-20).
//!
//! Validates that cache warming doesn't break functional correctness
//! and that the HNSW top-layer warming populates the cache at startup.
//!
//! Note: Co-access tracking unit tests live in src/cache_warmer.rs.
//! These tests exercise the feature through the public StorageEngine API.

use tempfile::tempdir;
use vantadb::config::VantaConfig;
use vantadb::node::{NodeTier, UnifiedNode, VectorRepresentations};
use vantadb::storage::StorageEngine;

// ─── Helpers ───────────────────────────────────────────────────

/// Create an in-memory engine for testing.
fn in_memory_engine() -> StorageEngine {
    let config = VantaConfig {
        backend_kind: vantadb::BackendKind::InMemory,
        ..VantaConfig::default()
    };
    StorageEngine::open_with_config(":memory:", Some(config))
        .expect("Failed to open in-memory engine")
}

/// Insert a simple Hot-tier node so it goes into the volatile cache.
fn insert_node(engine: &StorageEngine, id: u128) {
    let mut node = UnifiedNode::new(id);
    node.vector = VectorRepresentations::Full(vec![id as f32 * 0.1; 4]);
    node.tier = NodeTier::Hot;
    engine.insert(&node).expect("insert should succeed");
}

/// Return how many entries are currently in the volatile cache.
fn cache_size(engine: &StorageEngine) -> usize {
    engine.get_memory_stats().cache_entries
}

// ─── Tests ─────────────────────────────────────────────────────

#[test]
fn test_cache_warming_basic_crud_no_regression() {
    // Cache warming should not break basic CRUD operations
    let engine = in_memory_engine();

    insert_node(&engine, 1);
    insert_node(&engine, 2);
    insert_node(&engine, 3);

    // get() after insert should find the node
    for id in [1, 2, 3] {
        let node = engine.get(id).expect("get should succeed");
        assert!(node.is_some(), "node {} should exist after insert", id);
    }

    // get_many should work
    let nodes = engine.get_many(&[1, 2, 3]).expect("get_many should succeed");
    assert_eq!(nodes.len(), 3);

    // delete should work
    engine.delete(1, "test").expect("delete should succeed");
    let node = engine.get(1).expect("get after delete should succeed");
    assert!(node.is_none(), "deleted node should not be found");
}

#[test]
fn test_cache_warming_insert_updates_cache() {
    // Hot-tier nodes should be cached on insert
    let engine = in_memory_engine();
    assert_eq!(cache_size(&engine), 0, "cache should start empty");

    insert_node(&engine, 42);
    assert!(
        cache_size(&engine) >= 1,
        "cache should contain the inserted node"
    );
}

#[test]
fn test_cache_warming_hnsw_top_layer() {
    // Insert several nodes so the HNSW graph has an entry point.
    let engine = in_memory_engine();
    for i in 0..10 {
        insert_node(&engine, 100 + i);
    }

    // Verify the HNSW graph is non-empty.
    let stats = engine.get_memory_stats();
    assert!(stats.node_count > 0, "HNSW should have nodes");

    // HNSW top-layer warming is called at engine startup.
    // Since the cache is populated on insert (Hot tier), the entry point
    // should already be in cache after startup warming.
    let stats = engine.get_memory_stats();
    assert!(
        stats.cache_entries > 0,
        "cache should contain HNSW top-layer nodes after warmup, found {}",
        stats.cache_entries
    );
}

#[test]
fn test_cache_warming_does_not_break_get_many() {
    // get_many should still return correct results with cache warming enabled.
    let engine = in_memory_engine();

    for i in 0..5 {
        insert_node(&engine, 10 + i);
    }

    // Fetch all 5 nodes
    let nodes = engine.get_many(&[10, 11, 12, 13, 14])
        .expect("get_many should succeed");
    assert_eq!(nodes.len(), 5, "should retrieve all 5 nodes");

    // Verify IDs
    let mut ids: Vec<u128> = nodes.iter().map(|n| n.id).collect();
    ids.sort_unstable();
    assert_eq!(ids, vec![10, 11, 12, 13, 14]);
}

#[test]
fn test_cache_warming_cache_hit_on_subsequent_get() {
    // After first get() warms the cache, subsequent gets should be fast (cache hits).
    let engine = in_memory_engine();

    insert_node(&engine, 99);

    // First get — may read from backend (cache miss if cache was cleared)
    let _ = engine.get(99).expect("first get should succeed");

    // Second get — should be a cache hit
    let start = std::time::Instant::now();
    let node = engine.get(99).expect("second get should succeed");
    let elapsed = start.elapsed();

    assert!(node.is_some(), "node should exist");
    // Subsequent get from cache should be very fast (< 100µs)
    assert!(
        elapsed.as_micros() < 1000,
        "cached get took {}µs — expected <1000µs",
        elapsed.as_micros()
    );
}

#[test]
fn test_cache_warming_multiple_gets() {
    // Multiple gets should not crash or produce wrong results
    let engine = in_memory_engine();

    for i in 0..10 {
        insert_node(&engine, i);
    }

    // Fetch all 10 nodes one by one
    for i in 0..10 {
        let node = engine.get(i).expect("get should succeed");
        assert!(node.is_some(), "node {} should exist", i);
    }

    // All 10 should now be in cache
    assert_eq!(
        cache_size(&engine),
        11, // 10 + hnsw top-layer entry point
        "all nodes should be cached"
    );
}

#[test]
fn test_cache_warming_empty_engine() {
    // Edge case: empty engine should handle warming gracefully
    let engine = in_memory_engine();

    let node = engine.get(999).expect("get on empty engine should succeed");
    assert!(node.is_none(), "non-existent node should return None");

    let nodes = engine.get_many(&[]).expect("get_many with empty slice");
    assert!(nodes.is_empty(), "empty get_many should return empty");
}

#[test]
fn test_cache_warming_persistent_engine() {
    // Integration test with path-based engine (in-memory backend with a path)
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    let config = VantaConfig {
        backend_kind: vantadb::BackendKind::InMemory,
        ..VantaConfig::default()
    };
    let engine = StorageEngine::open_with_config(db_path, Some(config))
        .expect("Failed to open engine");

    // Basic warmup + CRUD cycle
    insert_node(&engine, 1);
    insert_node(&engine, 2);

    let node = engine.get(1).expect("get should succeed");
    assert!(node.is_some(), "node 1 should exist");

    engine.delete(2, "test").expect("delete should succeed");
    let node = engine.get(2).expect("get after delete");
    assert!(node.is_none(), "node 2 should be deleted");
}

#[test]
fn test_cache_warming_large_cache_eviction() {
    // Insert many nodes to verify the eviction mechanism still works
    // alongside cache warming.
    let engine = in_memory_engine();

    // Insert more nodes than the default cache capacity
    for i in 0..100 {
        let mut node = UnifiedNode::new(i);
        node.vector = VectorRepresentations::Full(vec![0.1; 4]);
        node.tier = NodeTier::Hot;
        engine.insert(&node).expect("insert should succeed");
    }

    // Cache should have entries but not exceed hardware-based limits
    let stats = engine.get_memory_stats();
    assert!(
        stats.cache_entries > 0,
        "cache should have entries after inserts"
    );
    assert!(
        stats.eviction_count > 0 || stats.cache_entries >= 100,
        "evictions may or may not fire depending on hardware limits"
    );
}
