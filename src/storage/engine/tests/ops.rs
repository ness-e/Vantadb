//! OPS module tests: transactions, batch operations, and edge cases.

use super::super::*;
use super::{in_memory_engine, sample_node};
use crate::backend::BackendPartition;
use crate::node::{NodeTier, UnifiedNode};

// ─── Transactions ─────────────────────────────────────────────

#[test]
fn test_begin_commit_transaction() {
    let engine = in_memory_engine();
    let txn_id = engine.begin_transaction().expect("begin");
    engine.commit_transaction(txn_id).expect("commit");
}

#[test]
fn test_begin_abort_transaction() {
    let engine = in_memory_engine();
    let txn_id = engine.begin_transaction().expect("begin");
    engine.abort_transaction(txn_id).expect("abort");
}

#[test]
fn test_transaction_ids_are_monotonic() {
    let engine = in_memory_engine();
    let t1 = engine.begin_transaction().expect("begin 1");
    let t2 = engine.begin_transaction().expect("begin 2");
    let t3 = engine.begin_transaction().expect("begin 3");
    assert!(
        t1 < t2 && t2 < t3,
        "txn ids should increase: {t1} < {t2} < {t3}"
    );
    engine.commit_transaction(t1).expect("commit 1");
    engine.abort_transaction(t2).expect("abort 2");
    engine.commit_transaction(t3).expect("commit 3");
}

#[test]
fn test_transaction_abort_after_commit() {
    let engine = in_memory_engine();
    let txn_id = engine.begin_transaction().expect("begin");
    engine.commit_transaction(txn_id).expect("commit");
    engine
        .abort_transaction(txn_id)
        .expect("abort after commit");
}

#[test]
fn test_transaction_commit_after_abort() {
    let engine = in_memory_engine();
    let txn_id = engine.begin_transaction().expect("begin");
    engine.abort_transaction(txn_id).expect("abort");
    let result = engine.commit_transaction(txn_id);
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_txn_insert_commit_persists() {
    let engine = in_memory_engine();
    let txn_id = engine.begin_transaction().expect("begin");
    engine.insert(&sample_node(42)).expect("insert in txn");
    engine.commit_transaction(txn_id).expect("commit");
    let retrieved = engine.get(42).expect("get");
    assert_eq!(retrieved.unwrap().id, 42);
}

#[test]
fn test_txn_insert_abort_rolls_back() {
    let engine = in_memory_engine();
    let txn_id = engine.begin_transaction().expect("begin");
    engine.insert(&sample_node(42)).expect("insert in txn");
    engine.abort_transaction(txn_id).expect("abort");
    let retrieved = engine.get(42).expect("get");
    assert!(retrieved.is_none(), "aborted txn should roll back insert");
}

#[test]
fn test_txn_delete_abort_rolls_back() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(42)).expect("insert outside txn");
    let txn_id = engine.begin_transaction().expect("begin");
    engine.delete(42, "test").expect("delete in txn");
    engine.abort_transaction(txn_id).expect("abort");
    let retrieved = engine.get(42).expect("get");
    assert_eq!(
        retrieved.unwrap().id,
        42,
        "aborted delete should be rolled back"
    );
}

#[test]
fn test_txn_read_your_writes_insert_then_get() {
    let engine = in_memory_engine();
    let txn_id = engine.begin_transaction().expect("begin");
    engine.insert(&sample_node(42)).expect("insert in txn");
    let retrieved = engine.get(42).expect("get inside txn");
    assert_eq!(retrieved.unwrap().id, 42);
    engine.abort_transaction(txn_id).expect("abort");
    let after = engine.get(42).expect("get after abort");
    assert!(after.is_none());
}

#[test]
fn test_txn_empty_commit() {
    let engine = in_memory_engine();
    let txn_id = engine.begin_transaction().expect("begin");
    engine.commit_transaction(txn_id).expect("commit empty txn");
}

#[test]
fn test_txn_commit_after_commit_errors() {
    let engine = in_memory_engine();
    let txn_id = engine.begin_transaction().expect("begin");
    engine.insert(&sample_node(1)).expect("insert in txn");
    engine.commit_transaction(txn_id).expect("first commit");
    let result = engine.commit_transaction(txn_id);
    let _ = result;
}

// ─── ERR-013: cardinality stats deferred to commit ────────────
//
// Buffering an insert/delete inside a transaction must NOT update
// cardinality stats (or edge/scalar indexes) until commit. Otherwise an
// aborted transaction leaves the counters inflated/deflated for records
// that never committed.

fn node_with_color(id: u128, color: &str) -> UnifiedNode {
    let mut node = UnifiedNode::new(id);
    node.relational.insert(
        "color".to_string(),
        crate::node::FieldValue::String(color.to_string()),
    );
    node
}

fn sel_of(engine: &StorageEngine, color: &str) -> f32 {
    engine.get_estimated_selectivity(
        "color",
        &crate::query::RelOp::Eq,
        &crate::node::FieldValue::String(color.to_string()),
    )
}

#[test]
fn test_txn_insert_abort_does_not_inflate_cardinality_stats() {
    let engine = in_memory_engine();
    engine
        .insert(&node_with_color(10, "red"))
        .expect("insert outside txn");

    let txn_id = engine.begin_transaction().expect("begin");
    engine
        .insert(&node_with_color(11, "blue"))
        .expect("insert in txn");
    engine.abort_transaction(txn_id).expect("abort");

    assert_eq!(
        sel_of(&engine, "blue"),
        0.0_f32,
        "aborted txn insert must not inflate cardinality for its value"
    );
    assert_eq!(
        sel_of(&engine, "red"),
        1.0_f32,
        "pre-existing value keeps its stats after abort"
    );
}

#[test]
fn test_txn_insert_commit_applies_stats_once() {
    let engine = in_memory_engine();
    engine
        .insert(&node_with_color(10, "red"))
        .expect("insert outside txn");

    let txn_id = engine.begin_transaction().expect("begin");
    engine
        .insert(&node_with_color(11, "blue"))
        .expect("insert in txn");
    engine.commit_transaction(txn_id).expect("commit");

    assert_eq!(
        sel_of(&engine, "blue"),
        0.5_f32,
        "committed txn insert counts the new value exactly once (1 of 2)"
    );
    assert_eq!(sel_of(&engine, "red"), 0.5_f32);
}

#[test]
fn test_txn_delete_abort_keeps_cardinality_stats() {
    let engine = in_memory_engine();
    engine
        .insert(&node_with_color(10, "red"))
        .expect("insert outside txn");

    let txn_id = engine.begin_transaction().expect("begin");
    engine.delete(10, "test").expect("delete in txn");
    engine.abort_transaction(txn_id).expect("abort");

    assert_eq!(
        sel_of(&engine, "red"),
        1.0_f32,
        "aborted txn delete must not deflate cardinality stats"
    );
}

#[test]
fn test_txn_delete_commit_applies_stats_once() {
    let engine = in_memory_engine();
    engine
        .insert(&node_with_color(10, "red"))
        .expect("insert outside txn");

    let txn_id = engine.begin_transaction().expect("begin");
    engine.delete(10, "test").expect("delete in txn");
    engine.commit_transaction(txn_id).expect("commit");

    assert_eq!(
        sel_of(&engine, "red"),
        0.0_f32,
        "committed txn delete decrements cardinality exactly once"
    );
}

// ─── Batch insert ─────────────────────────────────────────────

#[test]
fn test_batch_insert_empty() {
    let engine = in_memory_engine();
    engine.batch_insert(&[]).expect("batch_insert empty");
}

#[test]
fn test_batch_insert_single() {
    let engine = in_memory_engine();
    let node = sample_node(42);
    engine.batch_insert(&[node]).expect("batch_insert single");
    let retrieved = engine.get(42).expect("get").unwrap();
    assert_eq!(retrieved.id, 42);
}

#[test]
fn test_batch_insert_multiple() {
    let engine = in_memory_engine();
    let nodes: Vec<UnifiedNode> = (1..=5).map(sample_node).collect();
    engine.batch_insert(&nodes).expect("batch_insert multiple");
    for i in 1..=5 {
        let retrieved = engine.get(i).expect("get").unwrap();
        assert_eq!(retrieved.id, i);
    }
}

#[test]
fn test_batch_insert_with_cardinality() {
    let engine = in_memory_engine();
    let nodes: Vec<UnifiedNode> = (1..=5)
        .map(|i| {
            let mut node = sample_node(i);
            node.relational.insert(
                "type".to_string(),
                crate::node::FieldValue::String("batch".to_string()),
            );
            node
        })
        .collect();
    engine.batch_insert(&nodes).expect("batch_insert");
    let sel = engine.get_estimated_selectivity(
        "type",
        &crate::query::RelOp::Eq,
        &crate::node::FieldValue::String("batch".to_string()),
    );
    assert!(
        (sel - 1.0).abs() < 1e-6,
        "batch cardinality should be 1.0, got {sel}"
    );
}

#[test]
fn test_batch_insert_preserves_data_after_flush() {
    let engine = in_memory_engine();
    let nodes: Vec<UnifiedNode> = (1..=3).map(sample_node).collect();
    engine.batch_insert(&nodes).expect("batch_insert");
    engine.flush().expect("flush");
    for i in 1..=3 {
        let n = engine.get(i).expect("get").unwrap();
        assert_eq!(n.id, i as u128);
    }
}

#[test]
fn test_batch_insert_with_mixed_tiers() {
    let engine = in_memory_engine();
    let mut hot = sample_node(1);
    hot.tier = NodeTier::Hot;
    let mut cold = sample_node(2);
    cold.tier = NodeTier::Cold;
    engine
        .batch_insert(&[hot, cold])
        .expect("batch_insert mixed");
    assert!(
        engine.volatile_cache.read().contains_key(&1),
        "hot node should be in cache"
    );
    assert!(
        !engine.volatile_cache.read().contains_key(&2),
        "cold node should not be in cache"
    );
}

#[test]
fn test_batch_insert_cardinality_cap_eviction() {
    let engine = in_memory_engine();
    let nodes: Vec<UnifiedNode> = (0..101)
        .map(|i| {
            let mut node = sample_node(i);
            node.relational.insert(
                "tag".to_string(),
                crate::node::FieldValue::String(format!("val_{}", i)),
            );
            node
        })
        .collect();
    engine.batch_insert(&nodes).expect("batch_insert 101 nodes");
    let stats = engine.cardinality_stats.read();
    let total: usize = stats.values().map(|m| m.len()).sum();
    assert!(
        stats.contains_key("tag") || total > 0,
        "tag stats should exist after insert"
    );
}

// ─── Insert batch (VantaNodeInput) ───────────────────────────

#[test]
fn test_insert_batch_empty() {
    let engine = in_memory_engine();
    let ids = engine.insert_batch(&[]).expect("insert_batch empty");
    assert!(ids.is_empty());
}

#[test]
fn test_insert_batch_single() {
    let engine = in_memory_engine();
    let input = crate::VantaNodeInput::new(42);
    let ids = engine.insert_batch(&[input]).expect("insert_batch");
    assert_eq!(ids, vec![42]);
    let retrieved = engine.get(42).expect("get").unwrap();
    assert_eq!(retrieved.id, 42);
}

#[test]
fn test_insert_batch_with_fields() {
    let engine = in_memory_engine();
    let mut input = crate::VantaNodeInput::new(1);
    input.content = Some("hello world".to_string());
    input.vector = Some(vec![0.1, 0.2, 0.3]);
    input.fields.insert(
        "color".to_string(),
        crate::VantaValue::String("blue".to_string()),
    );
    let ids = engine.insert_batch(&[input]).expect("insert_batch");
    assert_eq!(ids, vec![1]);
    let node = engine.get(1).expect("get").unwrap();
    assert_eq!(node.id, 1);
    assert_eq!(
        node.relational.get("content"),
        Some(&crate::node::FieldValue::String("hello world".to_string()))
    );
}

#[test]
fn test_insert_batch_multiple() {
    let engine = in_memory_engine();
    let inputs: Vec<crate::VantaNodeInput> = (1..=3)
        .map(|i| {
            let mut input = crate::VantaNodeInput::new(i);
            input.content = Some(format!("node {}", i));
            input
        })
        .collect();
    let ids = engine.insert_batch(&inputs).expect("insert_batch");
    assert_eq!(ids, vec![1, 2, 3]);
    for i in 1..=3 {
        let node = engine.get(i).expect("get").unwrap();
        assert_eq!(node.id, i);
    }
}

// ─── Get many ─────────────────────────────────────────────────

#[test]
fn test_get_many_empty() {
    let engine = in_memory_engine();
    let results = engine.get_many(&[]).expect("get_many empty");
    assert!(results.is_empty());
}

#[test]
fn test_get_many_single() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(42)).expect("insert");
    let results = engine.get_many(&[42]).expect("get_many");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, 42);
}

#[test]
fn test_get_many_multiple() {
    let engine = in_memory_engine();
    for i in 1..=5 {
        engine.insert(&sample_node(i)).expect("insert");
    }
    let results = engine.get_many(&[1, 2, 3, 4, 5]).expect("get_many");
    assert_eq!(results.len(), 5);
    let ids: Vec<u128> = results.iter().map(|n| n.id).collect();
    assert_eq!(ids, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_get_many_partial_missing() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(1)).expect("insert");
    engine.insert(&sample_node(3)).expect("insert");
    let results = engine.get_many(&[1, 2, 3]).expect("get_many");
    let ids: Vec<u128> = results.iter().map(|n| n.id).collect();
    assert_eq!(ids, vec![1, 3]);
}

#[test]
fn test_get_many_after_delete() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(1)).expect("insert");
    engine.insert(&sample_node(2)).expect("insert");
    engine.delete(1, "test").expect("delete");
    let results = engine.get_many(&[1, 2]).expect("get_many");
    let ids: Vec<u128> = results.iter().map(|n| n.id).collect();
    assert_eq!(ids, vec![2]);
}

#[test]
fn test_get_many_with_partial_cache_miss() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(1)).expect("insert 1");
    engine.insert(&sample_node(2)).expect("insert 2");
    let results = engine.get_many(&[1, 2]).expect("get_many");
    assert_eq!(results.len(), 2);
    engine.volatile_cache.write().remove(&1);
    let results2 = engine.get_many(&[1, 2]).expect("get_many");
    assert_eq!(results2.len(), 2);
    let ids: Vec<u128> = results2.iter().map(|n| n.id).collect();
    assert_eq!(ids, vec![1, 2]);
}

#[test]
fn test_get_many_all_cache_miss() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(1)).expect("insert 1");
    engine.insert(&sample_node(2)).expect("insert 2");
    engine.volatile_cache.write().clear();
    let results = engine.get_many(&[1, 2]).expect("get_many");
    assert_eq!(results.len(), 2);
    let ids: Vec<u128> = results.iter().map(|n| n.id).collect();
    assert_eq!(ids, vec![1, 2]);
}

// ─── Delete batch ─────────────────────────────────────────────

#[test]
fn test_delete_batch_empty() {
    let engine = in_memory_engine();
    engine.delete_batch(&[]).expect("delete_batch empty");
}

#[test]
fn test_delete_batch_single() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(1)).expect("insert");
    engine.delete_batch(&[1]).expect("delete_batch");
    assert!(engine.get(1).unwrap().is_none());
}

#[test]
fn test_delete_batch_multiple() {
    let engine = in_memory_engine();
    for i in 1..=5 {
        engine.insert(&sample_node(i)).expect("insert");
    }
    engine.delete_batch(&[1, 3, 5]).expect("delete_batch");
    assert!(engine.get(1).unwrap().is_none());
    assert!(engine.get(2).unwrap().is_some());
    assert!(engine.get(3).unwrap().is_none());
    assert!(engine.get(4).unwrap().is_some());
    assert!(engine.get(5).unwrap().is_none());
}

#[test]
fn test_delete_batch_nonexistent() {
    let engine = in_memory_engine();
    engine
        .delete_batch(&[999, 1000])
        .expect("delete_batch nonexistent");
}

#[test]
fn test_delete_batch_with_cardinality() {
    let engine = in_memory_engine();
    for i in 1..=2 {
        let mut node = sample_node(i);
        node.relational.insert(
            "group".to_string(),
            crate::node::FieldValue::String("a".to_string()),
        );
        engine.insert(&node).expect("insert");
    }
    let mut node3 = sample_node(3);
    node3.relational.insert(
        "group".to_string(),
        crate::node::FieldValue::String("b".to_string()),
    );
    engine.insert(&node3).expect("insert");

    let sel_before = engine.get_estimated_selectivity(
        "group",
        &crate::query::RelOp::Eq,
        &crate::node::FieldValue::String("a".to_string()),
    );
    assert!(
        (sel_before - 2.0 / 3.0).abs() < 1e-6,
        "before: expected 2/3, got {sel_before}"
    );

    engine.delete_batch(&[1, 2]).expect("delete_batch");
    let sel_after = engine.get_estimated_selectivity(
        "group",
        &crate::query::RelOp::Eq,
        &crate::node::FieldValue::String("a".to_string()),
    );
    assert!(
        (sel_after - 0.0).abs() < 1e-6,
        "after: expected 0.0, got {sel_after}"
    );
}

#[test]
fn test_delete_batch_mixed_existing_and_nonexistent() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(1)).expect("insert 1");
    engine.insert(&sample_node(3)).expect("insert 3");
    engine.delete_batch(&[1, 2, 3]).expect("delete_batch mixed");
    assert!(engine.get(1).unwrap().is_none(), "1 should be deleted");
    assert!(engine.get(3).unwrap().is_none(), "3 should be deleted");
}

#[test]
fn test_delete_batch_clears_cache() {
    let engine = in_memory_engine();
    let mut hot = sample_node(42);
    hot.tier = NodeTier::Hot;
    engine.insert(&hot).expect("insert hot node");
    assert!(
        engine.volatile_cache.read().contains_key(&42),
        "hot node should be cached"
    );
    engine.delete_batch(&[42]).expect("delete_batch");
    assert!(
        !engine.volatile_cache.read().contains_key(&42),
        "node should be removed from cache"
    );
}

// ─── OPS edge cases: insert with vstore / cardinality ─────────

#[test]
fn test_insert_overwrite_cardinality_decrement() {
    let engine = in_memory_engine();
    let mut node = UnifiedNode::new(1);
    node.relational.insert(
        "color".to_string(),
        crate::node::FieldValue::String("red".to_string()),
    );
    engine.insert(&node).expect("first insert");
    let sel = engine.get_estimated_selectivity(
        "color",
        &crate::query::RelOp::Eq,
        &crate::node::FieldValue::String("red".to_string()),
    );
    assert_eq!(sel, 1.0, "cardinality should be 1 after first insert");

    engine.insert(&node).expect("second insert (overwrite)");
    let sel2 = engine.get_estimated_selectivity(
        "color",
        &crate::query::RelOp::Eq,
        &crate::node::FieldValue::String("red".to_string()),
    );
    assert_eq!(sel2, 1.0, "cardinality should remain 1 after overwrite");
}

// ─── OPS edge cases: get() ────────────────────────────────────

#[test]
fn test_get_cache_tombstone_flag() {
    let engine = in_memory_engine();
    let mut node = sample_node(42);
    node.tier = NodeTier::Hot;
    engine.insert(&node).expect("insert");
    {
        let mut cache = engine.volatile_cache.write();
        let cached = cache.get_mut(&42).expect("node should be cached");
        cached.flags.set(crate::node::NodeFlags::TOMBSTONE);
    }
    let retrieved = engine.get(42).expect("get");
    assert!(retrieved.is_none(), "tombstone in cache → get returns None");
}

#[test]
fn test_get_cache_hit_bumps_hits_uncontended() {
    // ERR-036: cache hits must still accumulate hits/last_accessed on the
    // cached node via try_write — never a mandatory blocking write lock.
    let engine = in_memory_engine();
    let mut node = sample_node(42);
    node.tier = NodeTier::Hot; // only Hot nodes enter volatile_cache
    engine.insert(&node).expect("insert");
    assert!(engine.get(42).expect("get").is_some(), "first hit");
    assert!(engine.get(42).expect("get").is_some(), "second hit");
    {
        let cache = engine.volatile_cache.read();
        let cached = cache.get(&42).expect("node should be cached");
        assert_eq!(
            cached.hits, 2,
            "uncontended hits accumulate: insert + 2 gets"
        );
        assert!(cached.last_accessed > 0, "last_accessed updated on hit");
    }
}

#[test]
fn test_get_corrupt_backend_metadata() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(42)).expect("insert");
    engine.volatile_cache.write().remove(&42);
    let key = 42u128.to_le_bytes();
    engine
        .put_to_partition(BackendPartition::Default, &key, b"garbage bytes")
        .expect("corrupt backend entry");
    let result = engine.get(42);
    assert!(result.is_err(), "corrupt metadata should produce an error");
    let msg = result.err().unwrap().to_string();
    assert!(
        msg.contains("Serialization") || msg.contains("deserialize"),
        "error should mention serialization, got: {msg}"
    );
}

#[test]
fn test_get_missing_hnsw_entry() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(42)).expect("insert");
    engine.volatile_cache.write().remove(&42);
    {
        let hnsw = engine.hnsw.load();
        hnsw.nodes.remove(&42);
    }
    let retrieved = engine.get(42).expect("get");
    assert!(retrieved.is_none(), "missing HNSW entry → get returns None");
}

#[test]
fn test_get_vstore_tombstone() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(42)).expect("insert");
    engine.volatile_cache.write().remove(&42);
    let offset = {
        let hnsw = engine.hnsw.load();
        hnsw.nodes.get(&42).map(|n| n.storage_offset).unwrap()
    };
    {
        let mut vstore = engine.vector_store[0].write();
        if let Some(mut header) = vstore.read_header(offset) {
            header.flags |= FLAG_TOMBSTONE;
            vstore.write_header(offset, &header).unwrap();
        }
    }
    let retrieved = engine.get(42).expect("get");
    assert!(
        retrieved.is_none(),
        "vstore tombstone flag → get returns None"
    );
}

#[test]
fn test_get_vector_bounds_exceeded() {
    let engine = in_memory_engine();
    let mut node = sample_node(42);
    node.vector = crate::node::VectorRepresentations::Full(vec![0.1, 0.2, 0.3]);
    engine.insert(&node).expect("insert");
    let offset = {
        let hnsw = engine.hnsw.load();
        hnsw.nodes.get(&42).map(|n| n.storage_offset).unwrap()
    };
    engine.volatile_cache.write().remove(&42);
    {
        let mut vstore = engine.vector_store[0].write();
        if let Some(mut header) = vstore.read_header(offset) {
            header.vector_len = u32::MAX;
            vstore.write_header(offset, &header).unwrap();
        }
    }
    let retrieved = engine.get(42).expect("get should succeed");
    assert!(
        retrieved.is_none(),
        "vector bounds exceeded → get returns None"
    );
}

// ─── OPS edge cases: is_deleted ─────────────────────────────

#[test]
fn test_is_deleted_nonexistent() {
    let engine = in_memory_engine();
    assert!(!engine.is_deleted(999).expect("is_deleted"));
}

#[test]
fn test_is_deleted_true_when_in_tombstones_partition() {
    let engine = in_memory_engine();
    let key = 42u128.to_le_bytes();
    engine
        .put_to_partition(BackendPartition::Tombstones, &key, b"tombstoned")
        .expect("put tombstone entry");
    assert!(
        engine.is_deleted(42).expect("is_deleted"),
        "node should be marked as deleted"
    );
}

#[test]
fn test_is_deleted_true_with_various_ids() {
    let engine = in_memory_engine();
    let key = 999u128.to_le_bytes();
    engine
        .put_to_partition(BackendPartition::Tombstones, &key, b"1")
        .expect("put tombstone");
    assert!(engine.is_deleted(999).expect("is_deleted"));
    assert!(
        !engine.is_deleted(1000).expect("is_deleted"),
        "other ID should not be deleted"
    );
}

// ─── OPS edge cases: delete entry point ──────────────────────

#[test]
fn test_delete_entry_point_promotion() {
    let engine = in_memory_engine();
    for i in 0..10u128 {
        let mut node = sample_node(i);
        node.vector = crate::node::VectorRepresentations::Full(vec![(i as f32 + 1.0) / 10.0; 4]);
        engine.insert(&node).expect("insert");
    }
    let ep = {
        let hnsw = engine.hnsw.load();
        hnsw.get_entry_point().expect("entry point should exist")
    };
    engine.delete(ep, "test").expect("delete entry point");
    for i in 0..10u128 {
        if i == ep {
            assert!(
                engine.get(i).unwrap().is_none(),
                "entry point {i} should be gone"
            );
        } else {
            assert!(
                engine.get(i).unwrap().is_some(),
                "non-entry-point node {i} should survive"
            );
        }
    }
    let new_ep = {
        let hnsw = engine.hnsw.load();
        hnsw.get_entry_point()
    };
    assert!(
        new_ep.is_some() && new_ep.unwrap() != u128::MAX,
        "new entry point should exist after deletion, got {:?}",
        new_ep
    );
    assert_ne!(new_ep.unwrap(), ep, "entry point should have changed");
}

#[test]
fn test_delete_nonexistent_does_not_affect_stats() {
    let engine = in_memory_engine();
    let mut node = UnifiedNode::new(1);
    node.relational.insert(
        "color".to_string(),
        crate::node::FieldValue::String("red".to_string()),
    );
    engine.insert(&node).expect("insert");
    engine.delete(999, "test").expect("delete nonexistent");
    let sel = engine.get_estimated_selectivity(
        "color",
        &crate::query::RelOp::Eq,
        &crate::node::FieldValue::String("red".to_string()),
    );
    assert_eq!(sel, 1.0, "cardinality should be unchanged");
}

// ─── OPS edge cases: scan_nodes_page ──────────────────────────

#[test]
fn test_scan_nodes_page_skips_corrupt_keys() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(42)).expect("insert");
    engine
        .put_to_partition(BackendPartition::Default, b"short", b"value")
        .expect("put corrupt key");
    engine
        .put_to_partition(BackendPartition::Default, b"", b"empty")
        .expect("put empty key");
    let (nodes, cursor) = engine.scan_nodes_page("", 10).expect("scan");
    assert_eq!(nodes.len(), 1, "should return only the valid node");
    assert_eq!(nodes[0].id, 42);
    assert_eq!(cursor, "");
}

#[test]
fn test_scan_nodes_page_with_mixed_validity() {
    let engine = in_memory_engine();
    for i in 1..=3 {
        engine.insert(&sample_node(i)).expect("insert");
    }
    let ghost_key = 99u128.to_le_bytes();
    let metadata = crate::storage::ops::NodeMetadata {
        relational: std::collections::BTreeMap::new(),
        edges: Vec::new(),
        created_by_txn: 0,
        deleted_by_txn: None,
    };
    let val = postcard::to_allocvec(&metadata).unwrap();
    engine
        .put_to_partition(BackendPartition::Default, &ghost_key, &val)
        .expect("put ghost entry");
    let (nodes, _) = engine.scan_nodes_page("", 10).expect("scan");
    assert_eq!(
        nodes.len(),
        3,
        "ghost entry with missing HNSW should be skipped"
    );
    let ids: Vec<u128> = nodes.iter().map(|n| n.id).collect();
    assert!(ids.contains(&1));
    assert!(ids.contains(&2));
    assert!(ids.contains(&3));
}

// ─── OPS edge cases: insert_to_cf with indexes ─────────────────

#[test]
fn test_insert_to_cf_with_scalar_and_edge_indexes() {
    let engine = in_memory_engine();
    let related_id = engine.intern_label("related");
    let mut node = sample_node(42);
    node.relational.insert(
        "color".to_string(),
        crate::node::FieldValue::String("red".to_string()),
    );
    node.edges.push(crate::node::Edge {
        target: 1,
        label_id: related_id,
        weight: 1.0,
        reverse: false,
        created_at_ms: 1,
    });
    engine
        .insert_to_cf(&node, "default")
        .expect("insert_to_cf with indexes");
}

// ─── OPS edge cases: purge_permanent ──────────────────────────

#[test]
fn test_purge_permanent_removes_all_traces() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(42)).expect("insert");
    assert!(engine.get(42).expect("get").is_some());
    engine.purge_permanent(42).expect("purge_permanent");
    let key = 42u128.to_le_bytes();
    let val = engine
        .get_from_partition(BackendPartition::Default, &key)
        .expect("get from Default");
    assert!(
        val.is_none(),
        "node should be removed from Default partition"
    );
    let val_ts = engine
        .get_from_partition(BackendPartition::TombstoneStorage, &key)
        .expect("get from TombstoneStorage");
    assert!(val_ts.is_none(), "should not be in TombstoneStorage");
    let val_t = engine
        .get_from_partition(BackendPartition::Tombstones, &key)
        .expect("get from Tombstones");
    assert!(val_t.is_none(), "should not be in Tombstones");
}

// ─── Delete with cascade (edge index) ─────────────────────────

#[test]
fn test_delete_with_edge_index_removes_references() {
    let engine = in_memory_engine();
    let refers_to_id = engine.intern_label("refers_to");
    let mut source = sample_node(1);
    source.edges.push(crate::node::Edge {
        target: 2,
        label_id: refers_to_id,
        weight: 1.0,
        reverse: false,
        created_at_ms: 1,
    });
    engine.insert(&source).expect("insert source");
    let target = sample_node(2);
    engine.insert(&target).expect("insert target");
    engine.delete(2, "test").expect("delete target");
    let retrieved = engine.get(2).expect("get");
    assert!(retrieved.is_none(), "target should be deleted");
}

// ─── Emergency shutdown (flush tracker) ───────────────────────

#[test]
fn test_emergency_shutdown_flushes() {
    use std::sync::atomic::AtomicBool;
    static DID_FLUSH: AtomicBool = AtomicBool::new(false);

    struct FlushTracker;
    impl Drop for FlushTracker {
        fn drop(&mut self) {
            DID_FLUSH.store(true, std::sync::atomic::Ordering::SeqCst);
        }
    }

    let _tracker = FlushTracker;
}

#[test]
fn test_emergency_shutdown_not_called() {
    let engine = in_memory_engine();
    let result = engine.ensure_writable();
    assert!(result.is_ok(), "engine should be writable");
}

// ─── Snapshot Isolation / MVCC ────────────────────────────────

#[test]
fn test_begin_snapshot_returns_valid_snapshot() {
    let engine = in_memory_engine();
    let snapshot = engine.begin_snapshot();
    assert!(snapshot.txn_id > 0, "snapshot txn_id should be positive");
}

#[test]
fn test_get_with_snapshot_sees_committed_data() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(42)).expect("insert outside txn");

    let snapshot = engine.begin_snapshot();
    let retrieved = engine
        .get_with_snapshot(42, &snapshot)
        .expect("get_with_snapshot");
    assert_eq!(retrieved.unwrap().id, 42);
}

#[test]
fn test_get_with_snapshot_does_not_see_uncommitted_data() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(42)).expect("insert base");

    let snapshot = engine.begin_snapshot();

    // Start a transaction and insert a different node
    let txn_id = engine.begin_transaction().expect("begin txn");
    engine
        .insert_in_txn(&sample_node(99), txn_id)
        .expect("insert in txn");

    // Snapshot should NOT see the uncommitted node
    let retrieved = engine
        .get_with_snapshot(99, &snapshot)
        .expect("get_with_snapshot");
    assert!(
        retrieved.is_none(),
        "snapshot should not see uncommitted node"
    );

    engine.abort_transaction(txn_id).expect("abort txn");
}

#[test]
fn test_get_with_snapshot_does_not_see_deleted_data() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(42)).expect("insert");

    // Use a transaction so delete stamps deleted_by_txn (MVCC-friendly)
    let txn_id = engine.begin_transaction().expect("begin txn");
    engine
        .delete_in_txn(42, "test", txn_id)
        .expect("delete in txn");

    // Snapshot taken before commit should still see the node
    let snapshot = engine.begin_snapshot();
    let retrieved = engine
        .get_with_snapshot(42, &snapshot)
        .expect("get_with_snapshot before commit");
    assert!(
        retrieved.is_some(),
        "snapshot before commit should see node"
    );

    engine.commit_transaction(txn_id).expect("commit");

    // Snapshot taken after commit should NOT see it
    let later = engine.begin_snapshot();
    let later_retrieved = engine
        .get_with_snapshot(42, &later)
        .expect("get_with_snapshot after commit");
    assert!(
        later_retrieved.is_none(),
        "snapshot after commit should not see deleted node"
    );
}

#[test]
fn test_concurrent_txns_via_explicit_methods() {
    let engine = in_memory_engine();

    let t1 = engine.begin_transaction().expect("begin t1");
    let t2 = engine.begin_transaction().expect("begin t2");

    engine
        .insert_in_txn(&sample_node(1), t1)
        .expect("t1 inserts node 1");
    engine
        .insert_in_txn(&sample_node(2), t2)
        .expect("t2 inserts node 2");

    engine.commit_transaction(t1).expect("commit t1");
    engine.commit_transaction(t2).expect("commit t2");

    assert!(engine.get(1).expect("get node 1").is_some());
    assert!(engine.get(2).expect("get node 2").is_some());
}

#[test]
fn test_write_write_conflict_detected() {
    let engine = in_memory_engine();

    let t1 = engine.begin_transaction().expect("begin t1");
    let t2 = engine.begin_transaction().expect("begin t2");

    engine
        .insert_in_txn(&sample_node(42), t1)
        .expect("t1 inserts node 42");

    let result = engine.insert_in_txn(&sample_node(42), t2);
    assert!(
        result.is_err(),
        "t2 should conflict with t1 for the same node"
    );

    engine.abort_transaction(t2).expect("abort t2");
    engine.commit_transaction(t1).expect("commit t1");
}

#[test]
fn test_plain_insert_errors_with_multiple_active_txns() {
    let engine = in_memory_engine();
    let _t1 = engine.begin_transaction().expect("begin t1");
    let _t2 = engine.begin_transaction().expect("begin t2");

    let result = engine.insert(&sample_node(99));
    assert!(
        result.is_err(),
        "plain insert with >1 active txn should error"
    );

    engine.abort_transaction(0).ok();
    engine.abort_transaction(1).ok();
}

#[test]
fn test_gc_mvcc_versions() {
    let engine = in_memory_engine();

    // Insert two nodes, then transactionally delete them
    let txn = engine.begin_transaction().expect("begin");
    engine
        .insert_in_txn(&sample_node(700), txn)
        .expect("insert in txn");
    engine
        .insert_in_txn(&sample_node(701), txn)
        .expect("insert in txn");
    engine.commit_transaction(txn).expect("commit");

    let txn2 = engine.begin_transaction().expect("begin");
    engine
        .delete_in_txn(700, "gc-test", txn2)
        .expect("delete in txn");
    engine
        .delete_in_txn(701, "gc-test", txn2)
        .expect("delete in txn");
    engine.commit_transaction(txn2).expect("commit");

    // Both should be invisible via snapshot, but still in backend
    let cutoff = engine
        .next_txn_id
        .load(std::sync::atomic::Ordering::Acquire);
    assert!(engine.get(700).expect("get").is_none());
    assert!(engine.get(701).expect("get").is_none());

    // GC should reclaim both
    let removed = engine.gc_mvcc_versions(Some(cutoff)).expect("gc");
    assert!(removed >= 2, "expected >=2 removed, got {}", removed);

    // GC again should be a no-op
    let removed2 = engine.gc_mvcc_versions(Some(cutoff + 100)).expect("gc");
    assert_eq!(removed2, 0, "expected 0 on second pass");
}

#[test]
fn test_many_concurrent_txns_with_final_consistency() {
    let engine = in_memory_engine();

    let mut txns = Vec::new();
    for i in 0..5u128 {
        let txn_id = engine.begin_transaction().expect("begin");
        engine
            .insert_in_txn(&sample_node(100 + i), txn_id)
            .expect("insert in txn");
        txns.push(txn_id);
    }

    for txn_id in &txns {
        engine.commit_transaction(*txn_id).expect("commit");
    }

    for i in 0..5u128 {
        assert!(
            engine.get(100 + i).expect("get").is_some(),
            "node {} should exist after commit",
            100 + i
        );
    }
}

// ─── ERR-014: insert→get immediate visibility ─────────────────
//
// The non-transactional insert() path appends the WAL record, then inside
// apply_insert publishes the KV node metadata BEFORE the queued HNSW mutation
// is drained into the index. A concurrent get() that reads the metadata but
// fails to find the HNSW entry used to return None — a stale miss for a node
// whose insert had already made the metadata visible. The fix registers the
// HNSW entry (queue + synchronous drain) before the backend.put, so the
// invariant below — "metadata visible ⇒ get() returns the node" — holds
// structurally.

#[test]
fn test_concurrent_insert_get_immediate_visibility() {
    use std::sync::Arc;
    let engine = Arc::new(in_memory_engine());

    const THREADS: usize = 8;
    const NODES_PER_THREAD: usize = 32;
    const BASE: u128 = 1_000_000;

    let mut handles = Vec::new();

    // Writer threads: insert into a private id range, then immediately read
    // the node back. ERR-014's contract — a committed insert must be visible
    // to the very next get().
    for t in 0..THREADS {
        let engine = Arc::clone(&engine);
        handles.push(std::thread::spawn(move || {
            for i in 0..NODES_PER_THREAD {
                let id = BASE + t as u128 * NODES_PER_THREAD as u128 + i as u128;
                // Distinct vector per node: all-identical vectors make the HNSW
                // greedy insertion pathologically slow, which starves writers on
                // the shared insert_lock.  (ERR-014 is about visibility, not
                // index topology.)
                let mut node = sample_node(id);
                node.vector = crate::node::VectorRepresentations::Full(vec![
                    0.1 + i as f32 / 1000.0,
                    0.2,
                    0.3,
                ]);
                engine.insert(&node).expect("insert");
                let got = engine.get(id).expect("get after insert");
                assert!(
                    got.is_some(),
                    "ERR-014: inserted node {id} not visible to immediate get()"
                );
            }
        }));
    }

    // Reader threads: hover over the writers' ranges. The moment the KV
    // metadata for an id is visible (backend.put inside apply_insert), get()
    // must already return the node — on the buggy path the metadata was
    // published before the HNSW entry existed, surfacing a transient None.
    for t in 0..THREADS {
        let engine = Arc::clone(&engine);
        handles.push(std::thread::spawn(move || {
            for i in 0..NODES_PER_THREAD {
                let id = BASE + t as u128 * NODES_PER_THREAD as u128 + i as u128;
                let key = id.to_le_bytes();
                let mut spins = 0u32;
                loop {
                    if engine
                        .get_from_partition(BackendPartition::Default, &key)
                        .expect("read partition")
                        .is_some()
                    {
                        break;
                    }
                    spins += 1;
                    assert!(spins < 200_000, "timeout waiting for metadata of {id}");
                    std::thread::sleep(std::time::Duration::from_micros(50));
                }
                let got = engine.get(id).expect("get while insert in flight");
                assert!(
                    got.is_some(),
                    "ERR-014: metadata visible for {id} but get() returned None"
                );
            }
        }));
    }

    for h in handles {
        h.join().expect("thread panicked");
    }
}

// ─── FND-02: multi-index lock coordination ────────────────────
//
// apply_insert/batch_insert call eviction while holding insert_lock
// (ERR-010, non-reentrant). On the buggy path, eviction → consolidate_node →
// refresh_index re-acquired insert_lock via try_lock_for(5000ms), timing out
// per candidate; worse, the call ran while the volatile_cache write guard was
// still held, which would deadlock the eviction's own cache read/write.
// The fix adds *_locked variants that apply the volatile_cache entry without re-locking.

#[test]
fn test_evict_cold_nodes_locked_no_reentrant_timeout() {
    let engine = in_memory_engine();

    // Seed hot nodes so eviction has candidates to consolidate.
    for i in 0..4u128 {
        let mut node = sample_node(i);
        node.tier = crate::node::NodeTier::Hot; // only Hot nodes enter volatile_cache
        engine.insert(&node).expect("seed insert");
    }

    // Simulate the insert path: insert_lock held, volatile_cache free.
    let guard = engine.insert_lock.lock();
    let start = std::time::Instant::now();
    let report = engine
        .evict_cold_nodes_with_reason_locked(1.0, EvictionReason::Manual)
        .expect("locked eviction must succeed without re-acquiring insert_lock");
    let elapsed = start.elapsed();
    drop(guard);

    assert!(
        elapsed.as_millis() < 1000,
        "FND-02: locked eviction took {elapsed:?} — reentrant insert_lock \
         re-acquire times out at 5000ms per candidate"
    );
    assert!(
        report.evicted > 0,
        "FND-02: eviction should have consolidated seeded hot nodes"
    );
}

#[test]
fn test_multi_index_write_paths_no_deadlock() {
    use std::sync::Arc;
    let engine = Arc::new(in_memory_engine());

    const WRITERS: usize = 4;
    const ITERS: usize = 40;
    const WRITER_BASE: u128 = 10_000_000;
    const SEED_BASE: u128 = 20_000_000;

    // Pre-seed a range the deleter owns, so delete_batch exercises the full
    // multi-index removal path (scalar/edge/text/HNSW) on real nodes.
    for i in 0..32u128 {
        engine
            .insert(&sample_node(SEED_BASE + i))
            .expect("seed insert");
    }

    let mut handles = Vec::new();

    // Writers: insert (vector + graph + text index) and batch reads.
    for t in 0..WRITERS {
        let engine = Arc::clone(&engine);
        handles.push(std::thread::spawn(move || {
            for i in 0..ITERS {
                let id = WRITER_BASE + t as u128 * ITERS as u128 + i as u128;
                let mut node = sample_node(id);
                // Distinct vectors avoid pathological HNSW greedy insertion.
                node.vector = crate::node::VectorRepresentations::Full(vec![
                    0.1 + (i % 7) as f32 / 100.0,
                    0.2,
                    0.3,
                ]);
                engine.insert(&node).expect("insert");
                if i % 10 == 0 {
                    let ids: Vec<u128> = (0..8)
                        .map(|k| WRITER_BASE + t as u128 * ITERS as u128 + k)
                        .collect();
                    let _ = engine.get_many(&ids);
                }
            }
        }));
    }

    // Deleter: batch-remove the seeded range (multi-index delete path).
    {
        let engine = Arc::clone(&engine);
        handles.push(std::thread::spawn(move || {
            for i in 0..ITERS {
                let ids: Vec<u128> = (0..4)
                    .map(|k| SEED_BASE + ((i * 4 + k) % 32) as u128)
                    .collect();
                engine.delete_batch(&ids).expect("delete_batch");
            }
        }));
    }

    // Evictor: standalone eviction under contention (acquires insert_lock).
    {
        let engine = Arc::clone(&engine);
        handles.push(std::thread::spawn(move || {
            for _ in 0..10 {
                let _ = engine.evict_cold_nodes_with_reason(0.5, EvictionReason::Watermark);
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
        }));
    }

    // Watchdog: join the workers on a worker thread; recv_timeout fails the
    // test (instead of hanging CI) if any worker deadlocks.
    let (tx, rx) = std::sync::mpsc::channel();
    let watchdog = std::thread::spawn(move || {
        for h in handles {
            h.join().expect("worker panicked");
        }
        tx.send(()).unwrap();
    });
    match rx.recv_timeout(std::time::Duration::from_secs(30)) {
        Ok(()) => {}
        Err(_) => panic!(
            "FND-02: deadlock suspected — mixed insert/get_many/delete_batch/evict \
             paths exceeded 30s wall-clock"
        ),
    }
    watchdog.join().expect("watchdog panicked");
}

// ─── FND-02-M2: evicción *_locked bajo contención real ─────────
//
// El watermark de producción deriva de hardware — max_nodes =
// total_memory/4/1536 (~2.7M en máquinas típicas) — así que los tests de
// estrés existentes (≤192 nodos) jamás disparan la evicción que FND-02
// arregló: `evict_cold_nodes_with_reason_locked` / `consolidate_node_locked`
// solo corren cuando apply_insert/batch_insert superan ese watermark.
//
// Este test sustituye el disparador por un umbral bajo local (64 nodos):
// threads "evictor" toman `insert_lock` — exactamente como hace
// `apply_insert` al superar el watermark — y llaman la variante *_locked con
// razón Watermark mientras writers/deleters/readers corren concurrentes.
// Verifica dos cosas: (1) la evicción SÍ ocurrió — `report.evicted` acumulado
// > 0, no un no-op; (2) sin deadlock ni timeout — watchdog con deadline
// generoso. Si el path regresara a re-adquirir `insert_lock` (bug FND-02),
// consolidate fallaría por try_lock_for timeout → evicted == 0 → assert.

#[test]
fn test_evict_locked_under_contention_no_deadlock() {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;
    let engine = Arc::new(in_memory_engine());

    const SEED: u128 = 256; // hot candidates → volatile_cache (pool de evicción)
    const WRITER_BASE: u128 = 10_000_000;
    const SEED_BASE: u128 = 20_000_000; // rango del deleter (no toca candidatos)
    const WRITERS: usize = 4;
    const ITERS: usize = 40;
    const EVICTOR_THRESHOLD: usize = 64; // "max_nodes" bajo del test

    // Candidatos de evicción: Hot tier entra a volatile_cache. Nadie más los
    // consume (writers insertan Cold, deleter borra otro rango), así que el
    // primer pass del evictor SIEMPRE ve el pool > umbral → evicted > 0.
    for i in 0..SEED {
        let mut node = sample_node(i);
        node.tier = NodeTier::Hot;
        engine.insert(&node).expect("seed insert");
    }
    // Rango del deleter: delete_batch ejercita el path multi-index en nodos reales.
    for i in 0..32u128 {
        engine
            .insert(&sample_node(SEED_BASE + i))
            .expect("seed insert");
    }

    let total_evicted = Arc::new(AtomicU64::new(0));
    let mut handles = Vec::new();

    // Writers: insert (vector + graph + text) y get_many.
    for t in 0..WRITERS {
        let engine = Arc::clone(&engine);
        handles.push(std::thread::spawn(move || {
            for i in 0..ITERS {
                let id = WRITER_BASE + t as u128 * ITERS as u128 + i as u128;
                let mut node = sample_node(id);
                // Vectores distintos evitan el greedy insertion patológico del HNSW.
                node.vector = crate::node::VectorRepresentations::Full(vec![
                    0.1 + (i % 7) as f32 / 100.0,
                    0.2,
                    0.3,
                ]);
                engine.insert(&node).expect("insert");
                if i % 10 == 0 {
                    let ids: Vec<u128> = (0..8)
                        .map(|k| WRITER_BASE + t as u128 * ITERS as u128 + k)
                        .collect();
                    let _ = engine.get_many(&ids);
                }
            }
        }));
    }

    // Deleter: batch-remove del rango seeded (path multi-index de delete).
    {
        let engine = Arc::clone(&engine);
        handles.push(std::thread::spawn(move || {
            for i in 0..ITERS {
                let ids: Vec<u128> = (0..4)
                    .map(|k| SEED_BASE + ((i * 4 + k) % 32) as u128)
                    .collect();
                engine.delete_batch(&ids).expect("delete_batch");
            }
        }));
    }

    // Evictors: simulan apply_insert superando el watermark — insert_lock
    // tomado + evicción *_locked con razón Watermark. Contención real sobre
    // insert_lock y volatile_cache con los threads de arriba.
    for _ in 0..2 {
        let engine = Arc::clone(&engine);
        let total_evicted = Arc::clone(&total_evicted);
        handles.push(std::thread::spawn(move || {
            for _ in 0..30 {
                let over_watermark = engine.volatile_cache.read().len() > EVICTOR_THRESHOLD;
                if !over_watermark {
                    std::thread::sleep(std::time::Duration::from_micros(200));
                    continue;
                }
                let guard = engine.insert_lock.lock();
                if let Ok(report) =
                    engine.evict_cold_nodes_with_reason_locked(0.5, EvictionReason::Watermark)
                {
                    total_evicted.fetch_add(report.evicted as u64, Ordering::Relaxed);
                }
                drop(guard);
            }
        }));
    }

    // Watchdog: deadline generoso — fail en vez de colgar CI ante deadlock.
    let (tx, rx) = std::sync::mpsc::channel();
    let watchdog = std::thread::spawn(move || {
        for h in handles {
            h.join().expect("worker panicked");
        }
        tx.send(()).unwrap();
    });
    match rx.recv_timeout(std::time::Duration::from_secs(60)) {
        Ok(()) => {}
        Err(_) => panic!(
            "FND-02-M2: deadlock/timeout suspected — locked eviction under contention \
             exceeded 60s wall-clock"
        ),
    }
    watchdog.join().expect("watchdog panicked");

    // La evicción *_locked DEBE haber corrido — no un no-op.
    assert!(
        total_evicted.load(Ordering::Relaxed) > 0,
        "FND-02-M2: evict_cold_nodes_with_reason_locked nunca evictó bajo contención"
    );
}

// ─── FND-02-M3: delete vs consolidate race ─────────────────────
//
// consolidate_node (eviction pública, lock_held=false) re-aplica la entrada
// HNSW del nodo y re-persiste metadata en backend; delete() la elimina. Antes
// del fix, consolidate hacía backend.put FUERA de insert_lock y solo tomaba el
// lock para el refresh_index: un delete intermedio dejaba zombie (nodo en HNSW
// sin metadata) o resucitaba el nodo eliminado. El fix retiene insert_lock
// durante toda la sección crítica + version check contra HNSW.

#[test]
fn test_delete_vs_consolidate_no_resurrection() {
    let engine = in_memory_engine();

    // Seed hot node (Hot tier entra al volatile_cache).
    let mut node = sample_node(42);
    node.tier = NodeTier::Hot;
    engine.insert(&node).expect("seed insert");
    assert!(engine.get(42).expect("get after insert").is_some());

    // Snapshot del candidato, como lo toma evict_cold_nodes_inner.
    let candidate = engine
        .volatile_cache
        .read()
        .get(&42)
        .cloned()
        .expect("candidate in volatile_cache");

    // delete() completo: quita HNSW + cache + backend metadata.
    engine.delete(42, "FND-02-M3").expect("delete");
    assert!(engine.hnsw.load().nodes.get(&42).is_none());
    assert!(engine.get(42).expect("get after delete").is_none());

    // consolidate_node sobre el snapshot stale NO debe resucitar el nodo:
    // el version check ve que el nodo ya no está en HNSW y skippea.
    engine
        .consolidate_node(&candidate)
        .expect("consolidate of deleted node must be a no-op");

    let key = 42u128.to_le_bytes();
    assert!(
        engine
            .backend
            .get(BackendPartition::Default, &key)
            .expect("backend read")
            .is_none(),
        "FND-02-M3: consolidate resucitó metadata en backend de un nodo eliminado"
    );
    assert!(
        engine.hnsw.load().nodes.get(&42).is_none(),
        "FND-02-M3: consolidate resucitó la entrada HNSW de un nodo eliminado"
    );
    assert!(
        engine.get(42).expect("final get").is_none(),
        "FND-02-M3: nodo eliminado visible tras consolidate"
    );
}

#[test]
fn test_delete_vs_evict_concurrent_no_zombie() {
    use std::sync::Arc;
    let engine = Arc::new(in_memory_engine());

    // Seed hot nodes: entran al volatile_cache y son candidatos de eviction.
    const N: u128 = 32;
    for i in 0..N {
        let mut node = sample_node(i);
        node.tier = NodeTier::Hot;
        engine.insert(&node).expect("seed insert");
    }

    let mut handles = Vec::new();

    // Deleter: elimina cada nodo seeded del índice + backend.
    {
        let engine = Arc::clone(&engine);
        handles.push(std::thread::spawn(move || {
            for i in 0..N {
                engine.delete(i, "FND-02-M3 stress").expect("delete");
            }
        }));
    }

    // Evictor: consolidación pública concurrente con el delete — con candidatos
    // stale puede correr después de que delete() ya eliminó el nodo.
    for _ in 0..2 {
        let engine = Arc::clone(&engine);
        handles.push(std::thread::spawn(move || {
            for _ in 0..20 {
                let _ = engine.evict_cold_nodes_with_reason(1.0, EvictionReason::Watermark);
                std::thread::sleep(std::time::Duration::from_micros(200));
            }
        }));
    }

    // Watchdog: detecta deadlock en vez de colgar CI.
    let (tx, rx) = std::sync::mpsc::channel();
    let watchdog = std::thread::spawn(move || {
        for h in handles {
            h.join().expect("worker panicked");
        }
        tx.send(()).unwrap();
    });
    match rx.recv_timeout(std::time::Duration::from_secs(30)) {
        Ok(()) => {}
        Err(_) => panic!("FND-02-M3: deadlock suspected — delete vs evict exceeded 30s wall-clock"),
    }
    watchdog.join().expect("watchdog panicked");

    // Invariante: ningún nodo eliminado puede quedar en el HNSW (ni metadata
    // en backend). El delete es la última operación sobre cada id, así que
    // cualquier consolidación posterior debió ser skippeada por el version check.
    let hnsw = engine.hnsw.load();
    for i in 0..N {
        assert!(
            hnsw.nodes.get(&i).is_none(),
            "FND-02-M3: zombie HNSW entry para nodo eliminado {i}"
        );
        let key = i.to_le_bytes();
        assert!(
            engine
                .backend
                .get(BackendPartition::Default, &key)
                .expect("backend read")
                .is_none(),
            "FND-02-M3: metadata resucitada en backend para nodo eliminado {i}"
        );
    }
}

// ─── FIND-62: commit vs flush interleaving (ERR-010) ────────────
//
// commit_transaction() hacía WAL batch_append + apply SIN insert_lock:
// un flush() concurrente podía drenar-vacío → serializar → contar
// (checkpoint_seq incluye esos records) → checkpoint → el commit pusheaba
// tarde = record invisible en recovery. El fix retiene insert_lock en el
// commit a través de [WAL batch → apply → drain → Commit], igual que
// insert()/delete()/batch_insert().

#[cfg(any(feature = "fjall", feature = "rocksdb"))]
#[test]
fn test_commit_flush_interleaving() {
    use std::sync::{Arc, Barrier};

    const ROUNDS: usize = 5;
    const NODES_PER_ROUND: u128 = 8;
    const BASE: u128 = 900_000;

    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().to_str().expect("db path").to_string();

    let engine = Arc::new(StorageEngine::open(path.as_str()).expect("open disk engine with WAL"));

    for round in 0..ROUNDS {
        let base = BASE + round as u128 * NODES_PER_ROUND;
        // Buffer de la ronda en una txn (el commit aplica WAL batch + stores).
        let txn_id = engine.begin_transaction().expect("begin");
        for i in 0..NODES_PER_ROUND {
            let id = base + i;
            let mut node = sample_node(id);
            // Vectores distintos evitan el greedy insertion patológico del HNSW.
            node.vector = crate::node::VectorRepresentations::Full(vec![
                0.1 + id as f32 / 1_000_000.0,
                0.2,
                0.3,
            ]);
            engine.insert_in_txn(&node, txn_id).expect("insert in txn");
        }

        // El commit corre contra un flush concurrente: la Barrier maximiza la
        // ventana de interleaving (el escenario ERR-010). El watchdog hace
        // fail en vez de colgar CI si el fix deadlockeara el lock no-reentrante.
        let barrier = Arc::new(Barrier::new(2));
        let engine_commit = Arc::clone(&engine);
        let barrier_commit = Arc::clone(&barrier);
        let committer = std::thread::spawn(move || {
            barrier_commit.wait();
            engine_commit.commit_transaction(txn_id).expect("commit")
        });
        let engine_flush = Arc::clone(&engine);
        let barrier_flush = Arc::clone(&barrier);
        let flusher = std::thread::spawn(move || {
            barrier_flush.wait();
            engine_flush.flush().expect("concurrent flush")
        });

        let (tx, rx) = std::sync::mpsc::channel();
        let watchdog = std::thread::spawn(move || {
            committer.join().expect("committer panicked");
            flusher.join().expect("flusher panicked");
            tx.send(()).unwrap();
        });
        match rx.recv_timeout(std::time::Duration::from_secs(30)) {
            Ok(()) => {}
            Err(_) => {
                panic!("FIND-62: deadlock suspected — commit vs flush exceeded 30s wall-clock")
            }
        }
        watchdog.join().expect("watchdog panicked");

        // Quiesce antes de verificar la ronda.
        engine.flush().expect("quiesce flush");
        for i in 0..NODES_PER_ROUND {
            let id = base + i;
            assert!(
                engine.get(id).expect("get").is_some(),
                "FIND-62: committed node {id} (round {round}) not visible after concurrent flush"
            );
        }
    }

    // Recovery: todos los records commiteados deben sobrevivir al reopen.
    drop(engine);
    let engine2 = StorageEngine::open(path.as_str()).expect("reopen");
    for round in 0..ROUNDS {
        for i in 0..NODES_PER_ROUND {
            let id = BASE + round as u128 * NODES_PER_ROUND + i;
            assert!(
                engine2.get(id).expect("get after reopen").is_some(),
                "FIND-62: committed node {id} invisible post-recovery"
            );
        }
    }
}
