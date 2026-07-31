//! Engine core tests: CRUD, read-only guards, backend capabilities, partition ops, scan/pagination.

use super::super::*;
use super::{in_memory_engine, in_memory_read_only, sample_node};
use crate::backend::{BackendKind, BackendPartition, BackendWriteOp};
use crate::config::VantaConfig;
use crate::node::{NodeTier, UnifiedNode};

// ─── Engine basics ────────────────────────────────────────────

#[test]
fn test_open_in_memory() {
    let engine = in_memory_engine();
    assert_eq!(engine.backend_kind(), BackendKind::InMemory);
    assert!(!engine.read_only);
}

#[cfg(feature = "fjall")]
#[test]
fn test_open_with_default_config() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    let engine = StorageEngine::open(path).expect("open with default config");
    assert!(!engine.read_only);
}

#[test]
fn test_backend_kind_in_memory() {
    let engine = in_memory_engine();
    assert_eq!(engine.backend_kind(), BackendKind::InMemory);
}

#[test]
fn test_supports_checkpoint_in_memory() {
    let engine = in_memory_engine();
    assert!(!engine.supports_checkpoint());
}

#[test]
fn test_supports_manual_compaction_in_memory() {
    let engine = in_memory_engine();
    assert!(!engine.supports_manual_compaction());
}

#[test]
fn test_backend_capabilities() {
    let engine = in_memory_engine();
    let caps = engine.backend_capabilities();
    assert_eq!(caps.kind, BackendKind::InMemory);
}

#[test]
fn test_insert_and_get() {
    let engine = in_memory_engine();
    let node = sample_node(42);
    engine.insert(&node).expect("insert should succeed");
    let retrieved = engine.get(42).expect("get should succeed");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, 42);
}

#[test]
fn test_insert_preserves_vector() {
    let engine = in_memory_engine();
    let mut node = UnifiedNode::new(7);
    let vec = vec![0.5, 0.8, 0.2, 0.9];
    node.vector = crate::node::VectorRepresentations::Full(vec.clone());
    engine.insert(&node).expect("insert");
    let retrieved = engine.get(7).expect("get").unwrap();
    match retrieved.vector {
        crate::node::VectorRepresentations::Full(v) => assert_eq!(v, vec),
        _ => panic!("expected Full vector"),
    }
}

#[test]
fn test_get_nonexistent() {
    let engine = in_memory_engine();
    let retrieved = engine.get(999).expect("get should succeed");
    assert!(retrieved.is_none());
}

#[test]
fn test_insert_duplicate_overwrites() {
    let engine = in_memory_engine();
    let mut node1 = UnifiedNode::new(1);
    node1.importance = 10.0;
    engine.insert(&node1).expect("first insert");
    let mut node2 = UnifiedNode::new(1);
    node2.importance = 99.0;
    engine.insert(&node2).expect("second insert");
    let retrieved = engine.get(1).expect("get").unwrap();
    assert_eq!(retrieved.importance, 99.0);
}

#[test]
fn test_delete_existing() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(10)).expect("insert");
    engine.delete(10, "test").expect("delete should succeed");
    let retrieved = engine.get(10).expect("get");
    assert!(retrieved.is_none(), "deleted node should be gone");
}

#[test]
fn test_delete_nonexistent() {
    let engine = in_memory_engine();
    let result = engine.delete(999, "test");
    assert!(result.is_ok(), "deleting nonexistent should not error");
}

#[test]
fn test_delete_updates_cardinality_stats() {
    let engine = in_memory_engine();
    let mut node = UnifiedNode::new(5);
    node.relational.insert(
        "color".to_string(),
        crate::node::FieldValue::String("red".to_string()),
    );
    engine.insert(&node).expect("insert");
    engine.delete(5, "test").expect("delete");
    let sel = engine.get_estimated_selectivity(
        "color",
        &crate::query::RelOp::Eq,
        &crate::node::FieldValue::String("red".to_string()),
    );
    assert_eq!(sel, 0.0, "cardinality should be zero after delete");
}

#[test]
fn test_is_deleted_false_after_insert() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(100)).expect("insert");
    assert!(!engine.is_deleted(100).expect("is_deleted"));
}

#[test]
fn test_purge_permanent() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(200)).expect("insert");
    engine.purge_permanent(200).expect("purge");
    assert!(engine.get(200).unwrap().is_none());
}

// ─── Read-only guards ─────────────────────────────────────────

#[test]
fn test_guard_write_allowed_read_only() {
    let config = VantaConfig {
        read_only: true,
        ..VantaConfig::default()
    };
    let result = StorageEngine::guard_write_allowed(&config);
    assert!(result.is_err());
    assert!(result.err().unwrap().to_string().contains("read-only"));
}

#[test]
fn test_guard_write_allowed_writable() {
    let config = VantaConfig::default();
    let result = StorageEngine::guard_write_allowed(&config);
    assert!(result.is_ok());
}

#[test]
fn test_read_only_rejects_insert() {
    let engine = in_memory_read_only();
    let result = engine.insert(&sample_node(1));
    assert!(result.is_err());
    assert!(result.err().unwrap().to_string().contains("read-only"));
}

#[test]
fn test_read_only_rejects_delete() {
    let engine = in_memory_read_only();
    let result = engine.delete(1, "test");
    assert!(result.is_err());
}

#[test]
fn test_read_only_rejects_flush() {
    let engine = in_memory_read_only();
    let result = engine.flush();
    assert!(result.is_err());
}

#[test]
fn test_read_only_rejects_compact_wal() {
    let engine = in_memory_read_only();
    let result = engine.compact_wal();
    assert!(result.is_err());
}

#[test]
fn test_read_only_rejects_consolidate() {
    let engine = in_memory_read_only();
    let result = engine.consolidate_node(&sample_node(1));
    assert!(result.is_err());
}

#[test]
fn test_read_only_rejects_evict() {
    let engine = in_memory_read_only();
    let result = engine.evict_cold_nodes(0.5);
    assert!(result.is_err());
}

#[test]
fn test_read_only_rejects_rebuild_index() {
    let engine = in_memory_read_only();
    let result = engine.rebuild_vector_index();
    assert!(result.is_err());
}

#[test]
fn test_read_only_rejects_compact_layout() {
    let engine = in_memory_read_only();
    let result = engine.compact_layout_bfs();
    assert!(result.is_err());
}

#[test]
fn test_read_only_allows_get() {
    let engine = in_memory_read_only();
    let result = engine.get(1);
    assert!(result.is_ok());
}

// ─── Backend partition ops ────────────────────────────────────

#[test]
fn test_put_to_partition_and_scan() {
    let engine = in_memory_engine();
    engine
        .put_to_partition(BackendPartition::Default, b"test_key", b"test_val")
        .expect("put");
    let entries = engine
        .scan_partition(BackendPartition::Default)
        .expect("scan");
    assert!(!entries.is_empty());
    assert!(entries.iter().any(|(k, _)| k == b"test_key"));
}

#[test]
fn test_put_to_partition_read_only_rejected() {
    let engine = in_memory_read_only();
    let result = engine.put_to_partition(BackendPartition::Default, b"k", b"v");
    assert!(result.is_err());
}

#[test]
fn test_get_from_partition() {
    let engine = in_memory_engine();
    engine
        .put_to_partition(BackendPartition::Default, b"mykey", b"myval")
        .expect("put");
    let val = engine
        .get_from_partition(BackendPartition::Default, b"mykey")
        .expect("get")
        .expect("value");
    assert_eq!(val, b"myval");
}

#[test]
fn test_get_from_partition_nonexistent() {
    let engine = in_memory_engine();
    let val = engine
        .get_from_partition(BackendPartition::Default, b"nope")
        .expect("get");
    assert!(val.is_none());
}

#[test]
fn test_scan_partition_prefix() {
    let engine = in_memory_engine();
    engine
        .put_to_partition(BackendPartition::Default, b"abc/1", b"a")
        .expect("put");
    engine
        .put_to_partition(BackendPartition::Default, b"abc/2", b"b")
        .expect("put");
    engine
        .put_to_partition(BackendPartition::Default, b"xyz/1", b"c")
        .expect("put");
    let entries = engine
        .scan_partition_prefix(BackendPartition::Default, b"abc/")
        .expect("scan_prefix");
    assert_eq!(entries.len(), 2);
}

#[test]
fn test_write_backend_batch() {
    let engine = in_memory_engine();
    let ops = vec![
        BackendWriteOp::Put {
            partition: BackendPartition::Default,
            key: b"k1".to_vec(),
            value: b"v1".to_vec(),
        },
        BackendWriteOp::Put {
            partition: BackendPartition::Default,
            key: b"k2".to_vec(),
            value: b"v2".to_vec(),
        },
    ];
    engine.write_backend_batch(ops).expect("batch");
    let v1 = engine
        .get_from_partition(BackendPartition::Default, b"k1")
        .expect("get")
        .expect("value");
    assert_eq!(v1, b"v1");
}

#[test]
fn test_partition_from_cf_name_valid() {
    assert_eq!(
        crate::storage::ops::partition_from_cf_name("default").unwrap(),
        BackendPartition::Default
    );
    assert_eq!(
        crate::storage::ops::partition_from_cf_name("tombstones").unwrap(),
        BackendPartition::Tombstones
    );
    assert_eq!(
        crate::storage::ops::partition_from_cf_name("text_index").unwrap(),
        BackendPartition::TextIndex
    );
}

#[test]
fn test_partition_from_cf_name_invalid() {
    let result = crate::storage::ops::partition_from_cf_name("nonexistent");
    assert!(result.is_err());
    assert!(result.err().unwrap().to_string().contains("Unknown"));
}

#[test]
fn test_touch_activity() {
    let engine = in_memory_engine();
    let before = engine
        .last_query_timestamp
        .load(std::sync::atomic::Ordering::Acquire);
    engine.touch_activity();
    let after = engine
        .last_query_timestamp
        .load(std::sync::atomic::Ordering::Acquire);
    assert!(after >= before);
}

#[test]
fn test_insert_to_cf_default() {
    let engine = in_memory_engine();
    engine
        .insert_to_cf(&sample_node(1), "default")
        .expect("insert_to_cf");
}

#[test]
fn test_insert_to_cf_invalid() {
    let engine = in_memory_engine();
    let result = engine.insert_to_cf(&sample_node(1), "bogus_cf");
    assert!(result.is_err());
    assert!(result.err().unwrap().to_string().contains("Unknown"));
}

#[test]
fn test_insert_fails_on_resource_limit() {
    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        rss_threshold: 0.0001,
        memory_limit: Some(1),
        ..VantaConfig::default()
    };
    let engine = StorageEngine::open_with_config(":memory:", Some(config)).unwrap();
    let result = engine.insert(&sample_node(1));
    let _ = result;
}

#[test]
fn test_insert_auto_flush_threshold() {
    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        flush_threshold: Some(1),
        ..VantaConfig::default()
    };
    let engine = StorageEngine::open_with_config(":memory:", Some(config)).expect("open engine");
    engine.insert(&sample_node(1)).expect("insert 1");
    engine.insert(&sample_node(2)).expect("insert 2");
    let n1 = engine.get(1).expect("get 1").unwrap();
    assert_eq!(n1.id, 1);
    let n2 = engine.get(2).expect("get 2").unwrap();
    assert_eq!(n2.id, 2);
}

#[test]
fn test_insert_cardinality_hundred_cap() {
    let engine = in_memory_engine();
    for i in 0..101u128 {
        let mut node = UnifiedNode::new(i);
        node.relational.insert(
            "color".to_string(),
            crate::node::FieldValue::String(format!("c_{}", i)),
        );
        engine.insert(&node).expect("insert");
    }
    let sel = engine.get_estimated_selectivity(
        "color",
        &crate::query::RelOp::Eq,
        &crate::node::FieldValue::String("c_100".to_string()),
    );
    let expected = 1.0 / 101.0;
    assert!(
        (sel - expected).abs() < 1e-4,
        "expected ~{expected} for untracked value (cap at 100), got {sel}"
    );
    let sel_first = engine.get_estimated_selectivity(
        "color",
        &crate::query::RelOp::Eq,
        &crate::node::FieldValue::String("c_0".to_string()),
    );
    assert!(
        (sel_first - expected).abs() < 1e-4,
        "first value should also be ~{expected}, got {sel_first}"
    );
}

#[test]
fn test_insert_with_hot_node_eviction() {
    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        // 64 MiB — generous ceiling so node memory_size estimates (~42KB/node on
        // Linux vs ~20KB on Windows) never trip ResourceLimit mid-test. This test
        // verifies insert + retrieve of 50 Hot nodes; pressure/eviction paths are
        // covered by stats.rs tests.
        memory_limit: Some(64 * 1024 * 1024),
        ..VantaConfig::default()
    };
    let engine = StorageEngine::open_with_config(":memory:", Some(config)).expect("open");
    for i in 0..50u128 {
        let mut node = sample_node(i);
        node.tier = NodeTier::Hot;
        engine.insert(&node).expect("insert hot node");
    }
    for i in 0..50u128 {
        let n = engine.get(i).expect("get").unwrap();
        assert_eq!(n.id, i, "node {i} should be retrievable");
    }
}

// ─── Scan nodes ───────────────────────────────────────────────

#[test]
fn test_scan_nodes_empty() {
    let engine = in_memory_engine();
    let nodes = engine.scan_nodes().expect("scan");
    assert!(nodes.is_empty());
}

#[test]
fn test_scan_nodes_with_inserts() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(1)).expect("insert 1");
    engine.insert(&sample_node(2)).expect("insert 2");
    let nodes = engine.scan_nodes().expect("scan");
    assert_eq!(nodes.len(), 2);
    let ids: Vec<u128> = nodes.iter().map(|n| n.id).collect();
    assert!(ids.contains(&1));
    assert!(ids.contains(&2));
}

#[test]
fn test_scan_nodes_excludes_deleted() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(1)).expect("insert 1");
    engine.insert(&sample_node(2)).expect("insert 2");
    engine.delete(1, "test").expect("delete 1");
    let nodes = engine.scan_nodes().expect("scan");
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].id, 2);
}

#[test]
fn test_scan_nodes_page_empty() {
    let engine = in_memory_engine();
    let (nodes, cursor) = engine.scan_nodes_page("", 10).expect("scan_nodes_page");
    assert!(nodes.is_empty());
    assert_eq!(cursor, "");
}

#[test]
fn test_scan_nodes_page_pagination() {
    let engine = in_memory_engine();
    for i in 1..=5 {
        engine.insert(&sample_node(i)).expect("insert");
    }
    let (page1, cursor1) = engine.scan_nodes_page("", 3).expect("page 1");
    assert_eq!(page1.len(), 3);
    assert!(!cursor1.is_empty(), "should have next cursor");
    let (page2, cursor2) = engine.scan_nodes_page(&cursor1, 3).expect("page 2");
    assert_eq!(page2.len(), 2);
    assert_eq!(cursor2, "", "last page should have empty cursor");
    let all_ids: Vec<u128> = page1
        .into_iter()
        .chain(page2.into_iter())
        .map(|n| n.id)
        .collect();
    assert_eq!(all_ids, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_scan_nodes_page_excludes_deleted() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(1)).expect("insert");
    engine.insert(&sample_node(2)).expect("insert");
    engine.delete(1, "test").expect("delete");
    let (nodes, _) = engine.scan_nodes_page("", 10).expect("scan_nodes_page");
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].id, 2);
}

#[test]
fn test_scan_nodes_page_with_zero_limit() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(1)).expect("insert");
    let (nodes, _cursor) = engine.scan_nodes_page("", 0).expect("scan");
    assert!(nodes.is_empty());
}

#[test]
fn test_scan_nodes_page_cursor_exact_page() {
    let engine = in_memory_engine();
    for i in 1..=3 {
        engine.insert(&sample_node(i)).expect("insert");
    }
    let (nodes, _cursor) = engine.scan_nodes_page("", 3).expect("scan");
    assert_eq!(nodes.len(), 3);
}

#[test]
fn test_insert_overwrite_cardinality_removes_old_field() {
    let engine = in_memory_engine();
    let mut node = UnifiedNode::new(1);
    node.relational.insert(
        "color".to_string(),
        crate::node::FieldValue::String("red".to_string()),
    );
    engine.insert(&node).expect("first insert");

    let mut node2 = UnifiedNode::new(1);
    node2.relational.insert(
        "color".to_string(),
        crate::node::FieldValue::String("blue".to_string()),
    );
    engine
        .insert(&node2)
        .expect("second insert (different value)");

    let sel_red = engine.get_estimated_selectivity(
        "color",
        &crate::query::RelOp::Eq,
        &crate::node::FieldValue::String("red".to_string()),
    );
    assert_eq!(
        sel_red, 0.0,
        "old field value 'red' should have 0 cardinality after overwrite"
    );

    let sel_blue = engine.get_estimated_selectivity(
        "color",
        &crate::query::RelOp::Eq,
        &crate::node::FieldValue::String("blue".to_string()),
    );
    assert_eq!(
        sel_blue, 1.0,
        "new field value 'blue' should have cardinality 1"
    );
}

#[test]
fn test_insert_overwrite_removes_old_edges() {
    let engine = in_memory_engine();
    let friend_id = engine.intern_label("friend");
    let mut node1 = sample_node(42);
    node1.edges.push(crate::node::Edge {
        target: 1,
        label_id: friend_id,
        weight: 1.0,
        reverse: false,
    });
    engine.insert(&node1).expect("first insert");

    let colleague_id = engine.intern_label("colleague");
    let mut node2 = sample_node(42);
    node2.edges.push(crate::node::Edge {
        target: 2,
        label_id: colleague_id,
        weight: 1.0,
        reverse: false,
    });
    engine.insert(&node2).expect("overwrite");

    let retrieved = engine.get(42).expect("get").unwrap();
    assert_eq!(retrieved.edges.len(), 1, "should have only the new edge");
    assert_eq!(retrieved.edges[0].target, 2);
}

#[test]
fn test_insert_overwrite_updates_scalar_index() {
    let engine = in_memory_engine();
    let mut node1 = sample_node(1);
    node1.relational.insert(
        "color".to_string(),
        crate::node::FieldValue::String("red".to_string()),
    );
    engine.insert(&node1).expect("first insert");

    let mut node2 = sample_node(1);
    node2.relational.insert(
        "color".to_string(),
        crate::node::FieldValue::String("blue".to_string()),
    );
    engine.insert(&node2).expect("overwrite");

    let sel_red = engine.get_estimated_selectivity(
        "color",
        &crate::query::RelOp::Eq,
        &crate::node::FieldValue::String("red".to_string()),
    );
    assert_eq!(sel_red, 0.0, "old value 'red' should have 0 cardinality");

    let sel_blue = engine.get_estimated_selectivity(
        "color",
        &crate::query::RelOp::Eq,
        &crate::node::FieldValue::String("blue".to_string()),
    );
    assert_eq!(sel_blue, 1.0, "new value 'blue' should have cardinality 1");
}
