//! Engine core tests: CRUD, read-only guards, backend capabilities, partition ops, scan/pagination.

use super::super::*;
use super::{in_memory_engine, in_memory_read_only, sample_node};
use crate::backend::{BackendKind, BackendPartition, BackendWriteOp};
use crate::config::Config;
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
    let config = Config {
        read_only: true,
        ..Config::default()
    };
    let result = StorageEngine::guard_write_allowed(&config);
    assert!(result.is_err());
    assert!(result.err().unwrap().to_string().contains("read-only"));
}

#[test]
fn test_guard_write_allowed_writable() {
    let config = Config::default();
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
    let config = Config {
        backend_kind: BackendKind::InMemory,
        rss_threshold: 0.0001,
        memory_limit: Some(1),
        ..Config::default()
    };
    let engine = StorageEngine::open_with_config(":memory:", Some(config)).unwrap();
    let result = engine.insert(&sample_node(1));
    let _ = result;
}

#[test]
fn test_insert_auto_flush_threshold() {
    let config = Config {
        backend_kind: BackendKind::InMemory,
        flush_threshold: Some(1),
        ..Config::default()
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
    let config = Config {
        backend_kind: BackendKind::InMemory,
        // 8 GiB — generous ceiling so neither node memory_size estimates
        // (~42KB/node on Linux vs ~20KB on Windows) nor the real process RSS
        // (FND-01-F1: the guard now measures real RSS, which includes the whole
        // test binary) ever trip ResourceLimit mid-test. This test verifies
        // insert + retrieve of 50 Hot nodes; pressure/eviction paths are
        // covered by stats.rs tests.
        memory_limit: Some(8 * 1024 * 1024 * 1024),
        ..Config::default()
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

// Regression (ERR-010): an empty cursor must not exclude node id 0 — the
// cursor previously parsed "" as 0 and the `id <= cursor_id` filter dropped
// node 0 from every scan.
#[test]
fn test_scan_nodes_includes_node_zero() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(0)).expect("insert 0");
    engine.insert(&sample_node(1)).expect("insert 1");
    let nodes = engine.scan_nodes().expect("scan");
    assert_eq!(nodes.len(), 2);
    let ids: Vec<u128> = nodes.iter().map(|n| n.id).collect();
    assert!(ids.contains(&0), "node id 0 must be included in scan");
    assert!(ids.contains(&1));
    // Pagination with an explicit cursor still skips already-seen ids.
    let (page, _) = engine
        .scan_nodes_page("0", 10)
        .expect("scan after cursor 0");
    assert!(!page.iter().any(|n| n.id == 0), "cursor 0 must skip id 0");
    assert_eq!(page.len(), 1);
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

// ─── WIRE-09: snapshot name sandbox ───────────────────────────────
// Prove-It (systematic-debugging): `create_snapshot` joined the raw `name`
// under `<data_dir>/snapshots/` with no validation, while `snapshot_restore`
// validates via `validate_snapshot_name`. `name = "../escape"` wrote outside
// the snapshots dir. This test FAILS pre-fix (returns Ok + writes outside)
// and PASSES post-fix (returns Err + nothing escapes).

#[cfg(feature = "fjall")]
#[test]
fn snapshot_traversal_name_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    let engine = StorageEngine::open(path).expect("open disk engine");
    for evil in ["../escape", "..", ".", "", "a/b", "a\\b"] {
        let err = engine.create_snapshot(evil).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("plain identifier"),
            "WIRE-09: name {evil:?} must be rejected as non-identifier, got: {msg}"
        );
    }
    // Nothing escaped: `<data>/escape` must not exist and a legit name
    // still lands inside the snapshots dir.
    assert!(
        !dir.path().join("data").join("escape").exists(),
        "WIRE-09: traversal snapshot escaped the sandbox"
    );
    let snap = engine.create_snapshot("snap-1").expect("legit name");
    assert!(
        snap.path
            .starts_with(dir.path().join("data").join("snapshots")),
        "legit snapshot must live under snapshots/, got: {}",
        snap.path.display()
    );
    assert!(engine
        .list_snapshots()
        .expect("list")
        .contains(&"snap-1".to_string()));
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
        created_at_ms: 1,
    });
    engine.insert(&node1).expect("first insert");

    let colleague_id = engine.intern_label("colleague");
    let mut node2 = sample_node(42);
    node2.edges.push(crate::node::Edge {
        target: 2,
        label_id: colleague_id,
        weight: 1.0,
        reverse: false,
        created_at_ms: 1,
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

// ─── MCP-15: prefetch re-entrancy regression ──────────────────

/// Regression for MCP-15: `get()` (cache miss) → `prefetch_related` → recursive
/// `self.get(warm_id)` used to re-enter `prefetch_related` forever when a
/// co-access pair (A↔B) was registered and BOTH nodes were cache misses —
/// `get()` never inserts the node it materializes, so A and B stayed mutually
/// uncached through the whole chain and the worker thread overflowed its stack.
///
/// Uses Cold-tier nodes so neither A nor B enters `volatile` on insert,
/// reproducing the exact cache-miss trigger of the MCP-15 crash.
#[test]
fn test_get_prefetch_does_not_recurse_forever() {
    let engine = in_memory_engine();
    // Cold tier (the default from UnifiedNode::new) → insert() does NOT
    // populate volatile, so get() below takes the cache-miss path.
    let mut a = sample_node(1);
    a.tier = NodeTier::Cold;
    let mut b = sample_node(2);
    b.tier = NodeTier::Cold;
    engine.insert(&a).expect("insert a");
    engine.insert(&b).expect("insert b");

    // Register the co-access pair A↔B the way get_many does when search
    // returns ≥2 hits. min_accesses is 3 by default, so record 3 times.
    for _ in 0..3 {
        engine.cache.warmer.record_co_access(&[1, 2]);
    }

    // Pre-fix this call recursed get(1)→prefetch→get(2)→prefetch→get(1)→…
    // until the stack overflowed. Post-fix it must terminate and leave the
    // prefetched node in the volatile cache.
    let node = engine.get(1).expect("get(1) should terminate");
    assert!(node.is_some(), "node 1 exists");
    assert!(
        engine.cache.volatile.read().contains_key(&2),
        "co-accessed node 2 should be prefetched into the volatile cache"
    );
}

// ─── D1a Slice 1 (RED): pure vector decoders extracted from `get` ───

fn d1a_f32_le_bytes(values: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(values.len() * 4);
    for v in values {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

#[test]
fn d1a_decode_full_roundtrip() {
    use super::super::get::decode_full_bytes;
    let bytes = d1a_f32_le_bytes(&[1.0, 2.0, 3.0]);
    let got = decode_full_bytes(3, 0, &bytes);
    assert!(
        matches!(got, Some(crate::node::VectorRepresentations::Full(ref v)) if *v == vec![1.0, 2.0, 3.0]),
        "FULL bytes must decode to the same f32 vec"
    );
}

#[test]
fn d1a_decode_full_oob_is_none() {
    use super::super::get::decode_full_bytes;
    let got = decode_full_bytes(u32::MAX, 0, &[0u8; 8]);
    assert!(
        got.is_none(),
        "corrupt FULL header must decode to None (caller skips node)"
    );
}

#[test]
fn d1a_decode_binary_roundtrip() {
    use super::super::get::decode_binary_bytes;
    let words = [0x0102_0304_0506_0708u64, 0xAABB_CCDD_EEFF_0011u64];
    let mut bytes = Vec::new();
    for w in words {
        bytes.extend_from_slice(&w.to_le_bytes());
    }
    let got = decode_binary_bytes(2, 0, &bytes);
    assert!(
        matches!(got, Some(crate::node::VectorRepresentations::Binary(ref b)) if b.as_ref() == words),
        "BINARY bytes must decode to the same u64 words"
    );
}

#[test]
fn d1a_decode_binary_oob_is_none() {
    use super::super::get::decode_binary_bytes;
    let got = decode_binary_bytes(4, 0, &[0u8; 8]);
    assert!(
        got.is_none(),
        "truncated BINARY payload must decode to None"
    );
}

#[test]
fn d1a_decode_turbo_roundtrip() {
    use super::super::get::decode_turbo_bytes;
    let got = decode_turbo_bytes(3, 1, &[0xFF, 9, 8, 7]);
    assert!(
        matches!(got, Some(crate::node::VectorRepresentations::Turbo(ref t)) if t.as_ref() == [9, 8, 7]),
        "TURBO bytes must decode with the offset applied"
    );
}

#[test]
fn d1a_decode_turbo_oob_is_none() {
    use super::super::get::decode_turbo_bytes;
    let got = decode_turbo_bytes(3, 2, &[9, 8]);
    assert!(got.is_none(), "truncated TURBO payload must decode to None");
}

#[test]
fn d1a_decode_sq8_roundtrip() {
    use super::super::get::decode_sq8_bytes;
    let mut bytes = vec![0xFE, 0x01];
    bytes.extend_from_slice(&0.5f32.to_le_bytes());
    let got = decode_sq8_bytes(2, 0, &bytes);
    assert!(
        matches!(got, Some(crate::node::VectorRepresentations::SQ8(ref d, s)) if d.as_ref() == [-2i8, 1i8] && s == 0.5),
        "SQ8 payload must decode to (i8 data, scale)"
    );
}

#[test]
fn d1a_decode_sq8_nonfinite_scale_is_none() {
    use super::super::get::decode_sq8_bytes;
    let mut bytes = vec![0x01, 0x02];
    bytes.extend_from_slice(&f32::NAN.to_le_bytes());
    let got = decode_sq8_bytes(2, 0, &bytes);
    assert!(got.is_none(), "non-finite SQ8 scale must decode to None");
}

#[test]
fn d1a_decode_kind0_empty_yields_none_vector() {
    use super::super::get::decode_vector_by_kind;
    let got = decode_vector_by_kind(0, 0, 0, &[]);
    assert!(
        matches!(got, Some(crate::node::VectorRepresentations::None)),
        "legacy kind==0 with len==0 is a valid node with no vector"
    );
}

#[test]
fn d1a_decode_unknown_kind_yields_none_vector() {
    use super::super::get::decode_vector_by_kind;
    let got = decode_vector_by_kind(0xFFFF, 5, 0, &[0u8; 64]);
    assert!(
        matches!(got, Some(crate::node::VectorRepresentations::None)),
        "unknown kind must degrade to a None vector, not skip the node"
    );
}

#[test]
fn d1a_decode_corrupt_full_yields_none() {
    use super::super::get::decode_vector_by_kind;
    use crate::node::NodeFlags;
    let got = decode_vector_by_kind(NodeFlags::VECTOR_KIND_FULL, u32::MAX, 0, &[]);
    assert!(got.is_none(), "corrupt FULL bounds must skip the node");
}

// ─── D1a Slice 2 (RED): lookup-phase helpers extracted from `get` ───

#[test]
fn d1a_lookup_txn_miss_without_active_txn() {
    let engine = in_memory_engine();
    let hit = engine.lookup_txn_buffer(9).expect("probe must not error");
    assert!(hit.is_none(), "no active txn → miss (caller continues)");
}

#[test]
fn d1a_lookup_txn_insert_hit() {
    let engine = in_memory_engine();
    let txn = engine.begin_transaction().expect("begin");
    engine.insert_in_txn(&sample_node(11), txn).expect("buffer");
    let hit = engine.lookup_txn_buffer(11).expect("probe");
    assert!(
        matches!(hit, Some(Some(ref n)) if n.id == 11),
        "buffered insert must hit with the node"
    );
}

#[test]
fn d1a_lookup_txn_delete_hit() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(12)).expect("insert");
    let txn = engine.begin_transaction().expect("begin");
    engine
        .delete_in_txn(12, "d1a probe", txn)
        .expect("buffer delete");
    let hit = engine.lookup_txn_buffer(12).expect("probe");
    assert!(
        matches!(hit, Some(None)),
        "buffered delete must hit as None"
    );
}

#[test]
fn d1a_lookup_cache_miss_empty() {
    let engine = in_memory_engine();
    assert!(
        engine.lookup_volatile(99).is_none(),
        "empty cache → miss (caller continues)"
    );
}

#[test]
fn d1a_lookup_cache_hit_hot_insert() {
    let engine = in_memory_engine();
    let mut node = sample_node(21);
    node.tier = NodeTier::Hot;
    engine.insert(&node).expect("insert hot");
    let hit = engine.lookup_volatile(21);
    assert!(
        matches!(hit, Some(Some(ref n)) if n.id == 21),
        "hot insert must be cached"
    );
}

#[test]
fn d1a_fetch_metadata_miss_absent() {
    let engine = in_memory_engine();
    let md = engine.fetch_backend_metadata(404).expect("fetch");
    assert!(md.is_none(), "absent id → None (caller returns Ok(None))");
}

#[test]
fn d1a_fetch_metadata_hit_after_insert() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(22)).expect("insert");
    let md = engine
        .fetch_backend_metadata(22)
        .expect("fetch")
        .expect("must hit");
    assert!(
        md.relational.is_empty(),
        "sample node has no relational fields"
    );
}

#[test]
fn d1a_index_offset_miss_empty() {
    let engine = in_memory_engine();
    assert!(engine.lookup_index_offset(77).is_none(), "unindexed → None");
}

#[test]
fn d1a_index_offset_hit_after_insert() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(23)).expect("insert");
    assert!(
        engine.lookup_index_offset(23).is_some(),
        "inserted → indexed"
    );
}

#[test]
fn d1a_read_header_roundtrip() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(24)).expect("insert");
    let off = engine.lookup_index_offset(24).expect("indexed");
    let header = engine
        .read_header_at(24, off)
        .expect("read")
        .expect("header");
    assert_eq!(header.vector_len, 3, "sample node persists a 3-dim vector");
}

#[test]
fn d1a_read_header_bad_segment_errors() {
    use crate::lsm::pack_offset;
    let engine = in_memory_engine();
    let bad = pack_offset(50, 0);
    assert!(
        engine.read_header_at(25, bad).is_err(),
        "unknown segment must error (same message as inline code)"
    );
}

// ─── D1a Slice 3 (RED): decode + rescue + assemble extracted from `get` ───

#[test]
fn d1a_decode_vector_at_roundtrip() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(31)).expect("insert");
    let off = engine.lookup_index_offset(31).expect("indexed");
    let header = engine
        .read_header_at(31, off)
        .expect("read")
        .expect("header");
    let vec = engine
        .decode_vector_at(off, &header)
        .expect("decode")
        .expect("vector");
    assert!(
        matches!(vec, crate::node::VectorRepresentations::Full(ref f) if *f == vec![0.1, 0.2, 0.3]),
        "persisted FULL vector must round-trip bit-identical"
    );
}

#[test]
fn d1a_decode_vector_at_bad_segment_errors() {
    use crate::lsm::pack_offset;
    use crate::node::DiskNodeHeader;
    let engine = in_memory_engine();
    let bad = pack_offset(50, 0);
    let header = DiskNodeHeader::new(32);
    assert!(
        engine.decode_vector_at(bad, &header).is_err(),
        "unknown segment must error, not silently skip"
    );
}

#[test]
fn d1a_materialize_uncached_miss_absent() {
    let engine = in_memory_engine();
    let got = engine.materialize_uncached(400).expect("materialize");
    assert!(got.is_none(), "absent id → None at the first stage");
}

#[test]
fn d1a_materialize_uncached_roundtrip() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(33)).expect("insert");
    let node = engine
        .materialize_uncached(33)
        .expect("materialize")
        .expect("node");
    assert_eq!(node.id, 33, "slow path must return the same node as get()");
    assert!(
        matches!(node.vector, crate::node::VectorRepresentations::Full(ref f) if *f == vec![0.1, 0.2, 0.3]),
        "slow path must carry the persisted vector"
    );
}

#[test]
fn d1a_rescue_nonlegacy_passthrough() {
    use crate::node::{NodeFlags, VectorRepresentations};
    let engine = in_memory_engine();
    let out = engine.rescue_legacy_vector(
        51,
        NodeFlags::VECTOR_KIND_FULL,
        VectorRepresentations::Full(vec![1.0, 2.0]),
    );
    assert!(
        matches!(out, VectorRepresentations::Full(ref f) if *f == vec![1.0, 2.0]),
        "non-legacy kinds must pass through untouched (no index lookup effect)"
    );
}

#[test]
fn d1a_rescue_legacy_empty_without_index_data_unchanged() {
    use crate::node::VectorRepresentations;
    let engine = in_memory_engine();
    let out = engine.rescue_legacy_vector(52, 0, VectorRepresentations::None);
    assert!(
        matches!(out, VectorRepresentations::None),
        "legacy kind with no index payload must keep the decoded vector"
    );
}

#[test]
fn d1a_assemble_node_maps_header_fields() {
    use super::super::get::assemble_node;
    use crate::node::{DiskNodeHeader, NodeTier, VectorRepresentations};
    use crate::storage::ops::NodeMetadata;
    let mut header = DiskNodeHeader::new(41);
    header.flags = crate::node::NodeFlags::VECTOR_KIND_FULL;
    header.tier = 1;
    header.confidence_score = 0.9;
    let md = NodeMetadata {
        relational: Default::default(),
        edges: Vec::new(),
        created_by_txn: 0,
        deleted_by_txn: None,
    };
    let node = assemble_node(41, &header, VectorRepresentations::None, md);
    assert_eq!(node.id, 41, "id threads through");
    assert!(matches!(node.tier, NodeTier::Hot), "tier 1 maps to Hot");
    assert!(
        (node.confidence_score - 0.9).abs() < f32::EPSILON,
        "scores thread through"
    );
}

// ─── D1a Slice 4 (RED): batch-phase helpers for `batch_insert_with_opts` ───

#[test]
fn d1a_batch_hnsw_decision_modes() {
    use super::super::insert::batch_should_insert_hnsw;
    use super::super::{BatchInsertOptions, InsertMode};
    let inc = BatchInsertOptions {
        insert_mode: InsertMode::Incremental,
        ..Default::default()
    };
    assert!(
        batch_should_insert_hnsw(&inc, 5000),
        "incremental always inserts"
    );
    let reb = BatchInsertOptions {
        insert_mode: InsertMode::Rebuild,
        ..Default::default()
    };
    assert!(
        !batch_should_insert_hnsw(&reb, 5),
        "rebuild never inserts inline"
    );
    let auto = BatchInsertOptions {
        insert_mode: InsertMode::Auto,
        incremental_threshold: Some(1000),
        ..Default::default()
    };
    assert!(
        batch_should_insert_hnsw(&auto, 999),
        "auto below threshold inserts"
    );
    assert!(
        !batch_should_insert_hnsw(&auto, 1000),
        "auto at threshold rebuilds"
    );
    let def = BatchInsertOptions::default();
    assert!(
        batch_should_insert_hnsw(&def, 999),
        "default threshold is 1000"
    );
    assert!(
        !batch_should_insert_hnsw(&def, 1001),
        "default threshold is 1000"
    );
}

#[test]
fn d1a_cap_cardinality_keeps_under_cap() {
    use super::super::insert::cap_cardinality;
    let mut stats: std::collections::HashMap<String, std::collections::HashMap<String, usize>> =
        Default::default();
    stats.insert(
        "f".to_string(),
        [("a".to_string(), 2)].into_iter().collect(),
    );
    cap_cardinality(&mut stats);
    assert_eq!(stats["f"]["a"], 2, "under cap → untouched");
}

#[test]
fn d1a_cap_cardinality_drops_min_field_over_cap() {
    use super::super::insert::cap_cardinality;
    use crate::config::MAX_CARDINALITY_PAIRS;
    let mut stats: std::collections::HashMap<String, std::collections::HashMap<String, usize>> =
        Default::default();
    stats.insert(
        "big".to_string(),
        (0..9_000).map(|i| (format!("k{i}"), 1)).collect(),
    );
    stats.insert(
        "small".to_string(),
        (0..1_500).map(|i| (format!("j{i}"), 1)).collect(),
    );
    assert!(
        stats.values().map(|m| m.len()).sum::<usize>() > MAX_CARDINALITY_PAIRS,
        "fixture must exceed the cap"
    );
    cap_cardinality(&mut stats);
    assert!(!stats.contains_key("small"), "min field dropped over cap");
    assert!(stats.contains_key("big"), "max field kept");
}

#[test]
fn d1a_batch_wal_skip_flag_is_noop() {
    use super::super::BatchInsertOptions;
    let engine = in_memory_engine();
    let opts = BatchInsertOptions {
        skip_wal: true,
        ..Default::default()
    };
    engine
        .append_batch_wal(&[], &opts)
        .expect("skip_wal must be a noop Ok");
}

#[test]
fn d1a_batch_cache_cold_needs_no_eviction() {
    let engine = in_memory_engine();
    assert!(
        !engine.cache_batch_hot_nodes(&[sample_node(61)]),
        "cold node → no eviction needed"
    );
}

#[test]
fn d1a_batch_overwrite_keeps_readable() {
    use crate::node::FieldValue;
    let engine = in_memory_engine();
    let n = sample_node(71);
    engine
        .batch_insert(std::slice::from_ref(&n))
        .expect("first");
    let mut n2 = n.clone();
    n2.set_field("k".to_string(), FieldValue::String("v".to_string()));
    engine
        .batch_insert(std::slice::from_ref(&n2))
        .expect("overwrite");
    let got = engine.get(71).expect("get").expect("present");
    assert_eq!(
        got.relational.get("k"),
        Some(&FieldValue::String("v".to_string())),
        "overwrite path (existing probe + stats) must converge"
    );
}

#[test]
fn d1a_pick_rescue_prefers_index_sq8() {
    use super::super::get::pick_rescue_payload;
    use crate::node::VectorRepresentations;
    let out = pick_rescue_payload(
        &VectorRepresentations::SQ8(vec![1i8, -2].into_boxed_slice(), 0.5),
        VectorRepresentations::None,
    );
    assert!(
        matches!(out, VectorRepresentations::SQ8(ref d, s) if d.as_ref() == [1i8, -2] && s == 0.5),
        "legacy index payload wins over the empty decode"
    );
}

#[test]
fn d1a_pick_rescue_fallback_keeps_decoded() {
    use super::super::get::pick_rescue_payload;
    use crate::node::VectorRepresentations;
    let out = pick_rescue_payload(
        &VectorRepresentations::None,
        VectorRepresentations::Full(vec![1.0]),
    );
    assert!(
        matches!(out, VectorRepresentations::Full(_)),
        "no index payload → decoded vector is kept"
    );
}

#[test]
fn d1a_segment_reader_bad_errors() {
    let engine = in_memory_engine();
    assert!(
        engine.vstore_segment_reader(50, 81).is_err(),
        "unknown segment must error (shared guard helper)"
    );
}

// ─── D1a Slice 4-resto (RED): prelude + persist-phase helpers ───

#[test]
fn d1a_batch_prelude_empty_is_none() {
    let engine = in_memory_engine();
    let stamp = engine.batch_prelude(&[]).expect("prelude");
    assert!(
        stamp.is_none(),
        "empty batch → None (caller returns Ok(()))"
    );
}

#[test]
fn d1a_batch_prelude_nonempty_stamps_time() {
    let engine = in_memory_engine();
    let stamp = engine
        .batch_prelude(std::slice::from_ref(&sample_node(91)))
        .expect("prelude");
    assert!(stamp.is_some(), "non-empty batch → Some(now_ms)");
}

#[test]
fn d1a_prealloc_grows_small_store() {
    let mut vs = crate::storage::vfile::File::create_in_memory(64);
    super::super::insert::prealloc_vstore_batch(&mut vs, 10_000).expect("grow");
    assert!(
        vs.size >= 10_000 * 1280,
        "batch estimate must fit after prealloc"
    );
}

#[test]
fn d1a_build_kv_put_op_key_shape() {
    let engine = in_memory_engine();
    let op = engine.build_kv_put_op(&sample_node(95)).expect("build");
    match op {
        crate::backend::BackendWriteOp::Put { key, .. } => {
            assert_eq!(key, 95u128.to_le_bytes().to_vec(), "key is the LE id");
        }
        _ => panic!("expected a Put op"),
    }
}

#[test]
fn d1a_serialize_batch_nodes_fills_vecs() {
    use super::super::insert::BatchStageBufs;
    let engine = in_memory_engine();
    let mut bufs = BatchStageBufs::with_capacity(1);
    engine
        .serialize_batch_nodes(
            std::slice::from_ref(&sample_node(96)),
            123,
            false,
            &mut bufs,
        )
        .expect("serialize");
    assert_eq!(bufs.kv_ops.len(), 1, "one KV op per node");
    assert_eq!(bufs.vstore_offsets.len(), 1, "one offset per node");
    assert!(bufs.hnsw_entries.is_empty(), "no HNSW entry when disabled");
}

#[test]
fn d1a_stage_one_node_returns_offset_and_put() {
    let engine = in_memory_engine();
    let mut guard = engine.vector_store[0].write();
    let (off, op) = engine
        .stage_one_node(&mut guard, &sample_node(101), 7)
        .expect("stage");
    let _ = off;
    assert!(
        matches!(op, crate::backend::BackendWriteOp::Put { .. }),
        "staged node must produce a KV Put op"
    );
}

#[test]
fn d1a_tombstone_offset_flags_header() {
    use super::super::insert::tombstone_offset;
    use crate::lsm::unpack_offset;
    let engine = in_memory_engine();
    let mut bufs = super::super::insert::BatchStageBufs::with_capacity(1);
    engine
        .serialize_batch_nodes(std::slice::from_ref(&sample_node(102)), 0, false, &mut bufs)
        .expect("serialize");
    let (_, local) = unpack_offset(bufs.vstore_offsets[0]);
    let mut guard = engine.vector_store[0].write();
    let before = guard.read_header(local).expect("header").flags;
    let err = crate::error::Error::generic_error("boom".to_string());
    tombstone_offset(&mut guard, bufs.vstore_offsets[0], &err);
    let after = guard.read_header(local).expect("header").flags;
    assert!(after & !before != 0, "tombstone must set new flag bits");
}

#[test]
fn d1a_commit_batch_locked_cold_ok() {
    use super::super::insert::BatchStageBufs;
    let engine = in_memory_engine();
    let node = sample_node(103);
    engine
        .commit_batch_locked(
            std::slice::from_ref(&node),
            &crate::storage::engine::BatchInsertOptions::default(),
            BatchStageBufs::with_capacity(0),
            false,
        )
        .expect("commit with staged-empty bufs must succeed");
}

#[test]
fn d1a_insert_hnsw_leveled_indexes() {
    use crate::node::{FilterBitset, VectorRepresentations};
    let engine = in_memory_engine();
    let entries = vec![(
        97u128,
        FilterBitset::all_set(),
        VectorRepresentations::Full(vec![0.1, 0.2, 0.3]),
        0u64,
    )];
    engine.insert_hnsw_leveled(&entries).expect("index");
    assert!(
        engine.lookup_index_offset(97).is_some(),
        "leveled insert must index the id"
    );
    // H1 (review): levels must VARY across a bulk insert — one fresh RNG per
    // entry collapses to a single constant level (broken stream).
    let bulk: Vec<(u128, FilterBitset, VectorRepresentations, u64)> = (1000..1200u128)
        .map(|id| {
            (
                id,
                FilterBitset::all_set(),
                VectorRepresentations::Full(vec![0.1, 0.2, 0.3]),
                0u64,
            )
        })
        .collect();
    engine.insert_hnsw_leveled(&bulk).expect("bulk index");
    let index = engine.vec_index();
    let mut distinct = std::collections::HashSet::new();
    for id in 1000..1200u128 {
        distinct.insert(index.node_layers(id));
    }
    assert!(
        distinct.len() > 1,
        "bulk leveled insert must spread nodes across layers, got {:?}",
        distinct
    );
}

#[test]
fn d1a_write_batch_kv_empty_ok() {
    let engine = in_memory_engine();
    engine
        .write_batch_kv_or_tombstone(Vec::new(), &[])
        .expect("empty batch write must succeed");
}

#[cfg(not(feature = "rayon"))]
#[test]
fn d1a_apply_stats_serial_tracks_field() {
    use crate::node::FieldValue;
    let engine = in_memory_engine();
    let mut n = sample_node(98);
    n.set_field("d1a_f", FieldValue::String("v".into()));
    engine.apply_batch_stats_serial(
        std::slice::from_ref(&n),
        &crate::storage::engine::BatchInsertOptions::default(),
    );
    assert!(
        engine.cache.cardinality_stats.read().contains_key("d1a_f"),
        "serial stats path must track the new field"
    );
}

#[cfg(feature = "rayon")]
#[test]
fn d1a_probe_existing_fresh_is_none() {
    let engine = in_memory_engine();
    let found = engine.probe_existing_for_batch(std::slice::from_ref(&sample_node(99)));
    assert_eq!(found.len(), 1, "one probe per node");
    assert!(found[0].is_none(), "fresh id → miss");
}

#[cfg(feature = "rayon")]
#[test]
fn d1a_apply_stats_rayon_tracks_field() {
    use crate::node::FieldValue;
    let engine = in_memory_engine();
    let mut n = sample_node(100);
    n.set_field("d1a_g", FieldValue::String("v".into()));
    engine.apply_batch_stats_rayon(
        std::slice::from_ref(&n),
        &crate::storage::engine::BatchInsertOptions::default(),
    );
    assert!(
        engine.cache.cardinality_stats.read().contains_key("d1a_g"),
        "rayon stats path must track the new field"
    );
}

#[test]
fn d1a_hot_or_cold_maps_tier_byte() {
    use super::super::get::hot_or_cold;
    use crate::node::NodeTier;
    assert!(matches!(hot_or_cold(1), NodeTier::Hot), "1 → Hot");
    assert!(matches!(hot_or_cold(0), NodeTier::Cold), "0 → Cold");
    assert!(matches!(hot_or_cold(9), NodeTier::Cold), "other → Cold");
}

#[test]
fn d1a_bump_val_map_increments() {
    use super::super::insert::bump_val_map;
    let mut m = std::collections::HashMap::new();
    bump_val_map(&mut m, "k".to_string());
    bump_val_map(&mut m, "k".to_string());
    assert_eq!(m["k"], 2, "under cap → increments");
}

#[test]
fn d1a_bump_val_map_caps_new_key() {
    use super::super::insert::bump_val_map;
    let mut m: std::collections::HashMap<String, usize> =
        (0..100).map(|i| (format!("k{i}"), 1)).collect();
    bump_val_map(&mut m, "new".to_string());
    assert!(!m.contains_key("new"), "at cap → new key ignored");
    bump_val_map(&mut m, "k0".to_string());
    assert_eq!(m["k0"], 2, "at cap → existing key still increments");
}
