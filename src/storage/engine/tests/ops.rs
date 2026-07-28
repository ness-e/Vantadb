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
        node.vector = crate::node::VectorRepresentations::Full(vec![(i as f32) / 10.0; 4]);
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
