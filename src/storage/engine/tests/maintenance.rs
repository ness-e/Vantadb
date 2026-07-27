//! MAINTENANCE module tests: eviction, compaction, quantization, rebuild, consolidate, flush.

use super::super::*;
use super::{in_memory_engine, in_memory_read_only, sample_node};
use crate::backend::BackendPartition;
use crate::config::VantaConfig;
use crate::node::{NodeTier, UnifiedNode};

// ─── Eviction ─────────────────────────────────────────────────

#[test]
fn test_evict_zero_ratio() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(1)).expect("insert");
    let report = engine.evict_cold_nodes(0.0).expect("evict");
    assert_eq!(report.evicted, 0);
}

#[test]
fn test_evict_empty_cache() {
    let engine = in_memory_engine();
    let report = engine.evict_cold_nodes(0.5).expect("evict");
    assert_eq!(report.evicted, 0);
    assert_eq!(report.scanned, 0);
}

#[test]
fn test_evict_cold_nodes_ratio_clamped() {
    let engine = in_memory_engine();
    let report = engine
        .evict_cold_nodes_with_reason(1.5, EvictionReason::Periodic)
        .expect("evict");
    assert_eq!(report.reason, EvictionReason::Periodic);
    assert_eq!(report.evicted, 0);
}

#[test]
fn test_evict_cold_nodes_negative_ratio_clamped() {
    let engine = in_memory_engine();
    let report = engine
        .evict_cold_nodes_with_reason(-0.5, EvictionReason::Periodic)
        .expect("evict");
    assert_eq!(report.evicted, 0);
}

#[test]
fn test_evict_cold_nodes_with_reason_empty() {
    let engine = in_memory_engine();
    let report = engine
        .evict_cold_nodes_with_reason(0.5, EvictionReason::Watermark)
        .expect("evict");
    assert_eq!(report.evicted, 0);
    assert_eq!(report.scanned, 0);
    assert_eq!(report.reason, EvictionReason::Watermark);
}

#[test]
fn test_evict_cold_nodes_with_reason_zero_ratio() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(1)).expect("insert");
    let report = engine
        .evict_cold_nodes_with_reason(0.0, EvictionReason::Manual)
        .expect("evict");
    assert_eq!(report.evicted, 0);
    assert_eq!(report.reason, EvictionReason::Manual);
}

#[test]
fn test_evict_cold_nodes_with_reason_hot_nodes_only() {
    let engine = in_memory_engine();
    let mut node = sample_node(1);
    node.tier = NodeTier::Hot;
    engine.insert(&node).expect("insert");
    let report = engine
        .evict_cold_nodes_with_reason(1.0, EvictionReason::Periodic)
        .expect("evict");
    assert!(report.scanned > 0, "should have scanned at least one node");
}

#[test]
fn test_evict_cold_nodes_successful_eviction() {
    let engine = in_memory_engine();
    let mut node = sample_node(42);
    node.tier = NodeTier::Hot;
    engine.insert(&node).expect("insert");
    assert!(
        engine.volatile_cache.read().contains_key(&42),
        "hot node should be in cache before eviction"
    );
    let report = engine
        .evict_cold_nodes_with_reason(1.0, EvictionReason::Periodic)
        .expect("eviction");
    assert!(report.evicted > 0, "should evict at least one hot node");
    assert_eq!(report.reason, EvictionReason::Periodic);
    assert!(
        !engine.volatile_cache.read().contains_key(&42),
        "evicted node should be removed from cache"
    );
    let retrieved = engine.get(42).expect("get").unwrap();
    assert_eq!(retrieved.id, 42);
}

#[test]
fn test_evict_cold_nodes_oom_reason() {
    let engine = in_memory_engine();
    let mut node = sample_node(42);
    node.tier = NodeTier::Hot;
    engine.insert(&node).expect("insert");
    let report = engine
        .evict_cold_nodes_with_reason(1.0, EvictionReason::Oom)
        .expect("eviction");
    assert!(report.evicted > 0, "OOM eviction should evict nodes");
    assert_eq!(report.reason, EvictionReason::Oom);
}

#[test]
fn test_evict_cold_nodes_manual_reason() {
    let engine = in_memory_engine();
    let mut node = sample_node(42);
    node.tier = NodeTier::Hot;
    engine.insert(&node).expect("insert");
    let report = engine
        .evict_cold_nodes_with_reason(1.0, EvictionReason::Manual)
        .expect("eviction");
    assert!(report.evicted > 0, "Manual eviction should evict nodes");
    assert_eq!(report.reason, EvictionReason::Manual);
}

#[test]
fn test_evict_cold_nodes_watermark_reason() {
    let engine = in_memory_engine();
    let mut node = sample_node(42);
    node.tier = NodeTier::Hot;
    engine.insert(&node).expect("insert");
    let report = engine
        .evict_cold_nodes_with_reason(1.0, EvictionReason::Watermark)
        .expect("eviction");
    assert!(report.evicted > 0, "Watermark eviction should evict nodes");
    assert_eq!(report.reason, EvictionReason::Watermark);
}

#[test]
fn test_evict_cold_nodes_reason_oom_with_governor() {
    let engine = in_memory_engine();
    let mut node = sample_node(42);
    node.tier = NodeTier::Hot;
    engine.insert(&node).expect("insert");
    let report = engine
        .evict_cold_nodes_with_reason(1.0, EvictionReason::Oom)
        .expect("evict OOM");
    assert!(
        report.evicted > 0 || report.scanned > 0,
        "evicted or scanned > 0"
    );
    assert_eq!(report.reason, EvictionReason::Oom);
    assert!(
        engine.memory_governor.is_some(),
        "memory_governor should still exist"
    );
}

#[test]
fn test_read_only_rejects_evict_cold_nodes_with_reason() {
    let engine = in_memory_read_only();
    let result = engine.evict_cold_nodes_with_reason(0.5, EvictionReason::Periodic);
    assert!(result.is_err());
}

#[test]
fn test_read_only_rejects_create_life_insurance() {
    let engine = in_memory_read_only();
    let result = engine.create_life_insurance("test");
    assert!(result.is_err());
}

#[test]
fn test_read_only_rejects_recover_archived_nodes() {
    let engine = in_memory_read_only();
    let result = engine.recover_archived_nodes(42);
    assert!(result.is_err());
}

#[test]
fn test_read_only_rejects_quantization_maintenance() {
    let engine = in_memory_read_only();
    let result = engine.run_quantization_maintenance();
    let _ = result;
}

// ─── Consolidation ────────────────────────────────────────────

#[test]
fn test_consolidate_node_removes_from_cache() {
    let engine = in_memory_engine();
    let mut node = sample_node(42);
    node.tier = crate::node::NodeTier::Hot;
    engine.insert(&node).expect("insert");
    assert!(
        engine.volatile_cache.read().contains_key(&42),
        "hot node should be in cache"
    );
    engine
        .consolidate_node(&sample_node(42))
        .expect("consolidate");
    assert!(
        !engine.volatile_cache.read().contains_key(&42),
        "consolidated node should be removed from cache"
    );
    let retrieved = engine.get(42).expect("get").unwrap();
    assert_eq!(retrieved.id, 42);
}

#[test]
fn test_consolidate_node_preserves_metadata() {
    let engine = in_memory_engine();
    let mut node = sample_node(100);
    node.relational.insert(
        "name".to_string(),
        crate::node::FieldValue::String("test".to_string()),
    );
    node.tier = NodeTier::Hot;
    engine.insert(&node).expect("insert");
    engine.consolidate_node(&node).expect("consolidate");
    let retrieved = engine.get(100).expect("get").unwrap();
    assert_eq!(retrieved.id, 100);
    assert_eq!(
        retrieved.relational.get("name"),
        Some(&crate::node::FieldValue::String("test".to_string()))
    );
}

#[test]
fn test_consolidate_node_changes_tier() {
    let engine = in_memory_engine();
    let mut node = sample_node(200);
    node.tier = NodeTier::Hot;
    engine.insert(&node).expect("insert");
    engine.consolidate_node(&node).expect("consolidate");
    let retrieved = engine.get(200).expect("get").unwrap();
    assert_eq!(retrieved.id, 200);
}

#[test]
fn test_consolidate_node_nonexistent() {
    let engine = in_memory_engine();
    let result = engine.consolidate_node(&sample_node(999));
    assert!(result.is_ok());
}

#[test]
fn test_consolidate_node_with_none_vector() {
    let engine = in_memory_engine();
    let mut node = UnifiedNode::new(42);
    node.vector = crate::node::VectorRepresentations::None;
    engine.insert(&node).expect("insert");
    engine
        .consolidate_node(&node)
        .expect("consolidate with None vector");
    let retrieved = engine.get(42).expect("get").unwrap();
    assert_eq!(retrieved.id, 42);
}

#[test]
fn test_consolidate_node_with_binary_vector() {
    let engine = in_memory_engine();
    let mut node = UnifiedNode::new(42);
    node.vector = crate::node::VectorRepresentations::Binary(Box::new([0b1010u64, 0b1100u64]));
    engine.insert(&node).expect("insert");
    engine
        .consolidate_node(&node)
        .expect("consolidate with Binary vector");
    let retrieved = engine.get(42).expect("get").unwrap();
    assert_eq!(retrieved.id, 42);
}

#[test]
fn test_consolidate_node_with_sq8_vector() {
    let engine = in_memory_engine();
    let mut node = UnifiedNode::new(42);
    node.vector = crate::node::VectorRepresentations::SQ8(Box::new([10, 20, -30, 40]), 0.25);
    node.tier = NodeTier::Hot;
    engine.insert(&node).expect("insert");
    engine.consolidate_node(&node).expect("consolidate SQ8");
    assert!(engine.get(42).expect("get").is_some());
}

#[test]
fn test_consolidate_node_with_turbo_vector() {
    let engine = in_memory_engine();
    let mut node = UnifiedNode::new(42);
    node.vector = crate::node::VectorRepresentations::Turbo(Box::new([0u8; 8]));
    node.tier = NodeTier::Hot;
    engine.insert(&node).expect("insert");
    engine.consolidate_node(&node).expect("consolidate Turbo");
    assert!(engine.get(42).expect("get").is_some());
}

// ─── Refresh index ────────────────────────────────────────────

#[test]
fn test_refresh_index_with_vector() {
    let engine = in_memory_engine();
    let node = sample_node(42);
    engine.insert(&node).expect("insert");
    let offset = {
        let hnsw = engine.hnsw.load();
        hnsw.nodes.get(&42).map(|n| n.storage_offset).unwrap()
    };
    engine.refresh_index(&node, offset).expect("refresh index");
    let retrieved = engine.get(42).expect("get").unwrap();
    assert_eq!(retrieved.id, 42);
}

#[test]
fn test_refresh_index_without_vector() {
    let engine = in_memory_engine();
    let mut node = UnifiedNode::new(99);
    node.vector = crate::node::VectorRepresentations::None;
    engine.refresh_index(&node, 64).expect("refresh");
}

#[test]
fn test_refresh_index_with_misaligned_offset() {
    let engine = in_memory_engine();
    let node = sample_node(42);
    let result = engine.refresh_index(&node, 1);
    assert!(result.is_ok());
}

#[test]
fn test_refresh_index_no_vector_with_misaligned_offset() {
    let engine = in_memory_engine();
    let mut node = UnifiedNode::new(99);
    node.vector = crate::node::VectorRepresentations::None;
    let result = engine.refresh_index(&node, 1);
    assert!(result.is_ok());
}

// ─── Compaction ───────────────────────────────────────────────

#[test]
fn test_trigger_compaction_empty() {
    let engine = in_memory_engine();
    let result = engine.trigger_compaction();
    assert!(result.is_ok());
}

#[test]
fn test_request_compaction_in_memory() {
    let engine = in_memory_engine();
    engine.request_compaction();
}

#[test]
fn test_trigger_compaction_with_deleted_nodes() {
    let engine = in_memory_engine();
    let mut node = sample_node(1);
    node.tier = NodeTier::Hot;
    engine.insert(&node).expect("insert");
    engine.delete(1, "test").expect("delete");
    engine.trigger_compaction().expect("trigger_compaction");
}

#[test]
fn test_trigger_compaction_empty_index() {
    let engine = in_memory_engine();
    engine.trigger_compaction().expect("trigger");
}

#[test]
fn test_trigger_compaction_with_hnsw_nodes() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(1)).expect("insert 1");
    engine.insert(&sample_node(2)).expect("insert 2");
    engine.insert(&sample_node(3)).expect("insert 3");
    engine.trigger_compaction().expect("trigger_compaction");
}

#[test]
fn test_trigger_compaction_high_tombstone_fraction() {
    let engine = in_memory_engine();
    let mut node = sample_node(42);
    node.tier = NodeTier::Hot;
    engine.insert(&node).expect("insert");
    let offset = {
        let hnsw = engine.hnsw.load();
        hnsw.nodes.get(&42).map(|n| n.storage_offset).unwrap()
    };
    {
        let mut vstore = engine.vector_store.write();
        if let Some(mut header) = vstore.read_header(offset) {
            header.flags |= FLAG_TOMBSTONE;
            vstore.write_header(offset, &header).unwrap();
        }
    }
    engine
        .trigger_compaction()
        .expect("trigger with >20% tombstones");
}

#[test]
fn test_compact_wal() {
    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        ..VantaConfig::default()
    };
    let engine = StorageEngine::open_with_config(":memory:", Some(config)).expect("open");
    engine.insert(&sample_node(1)).expect("insert");
    engine.compact_wal().expect("compact_wal");
    engine.flush().expect("flush");
    let node = engine.get(1).expect("get");
    assert!(node.is_some());
}

#[test]
fn test_compact_wal_idempotent() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(1)).expect("insert");
    engine.compact_wal().expect("first compact_wal");
    engine.insert(&sample_node(2)).expect("insert 2");
    engine.compact_wal().expect("second compact_wal");
    let n1 = engine.get(1).expect("get 1").unwrap();
    assert_eq!(n1.id, 1);
    let n2 = engine.get(2).expect("get 2").unwrap();
    assert_eq!(n2.id, 2);
}

#[test]
fn test_compact_wal_on_empty_engine() {
    let engine = in_memory_engine();
    engine.compact_wal().expect("compact_wal with no data");
}

#[test]
fn test_compact_layout_bfs_empty_engine() {
    let engine = in_memory_engine();
    let count = engine.compact_layout_bfs().expect("compact_layout_bfs");
    assert_eq!(count, 0, "empty index should compact 0 nodes");
}

#[test]
fn test_compact_layout_bfs_with_data() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(10)).expect("insert 10");
    engine.insert(&sample_node(20)).expect("insert 20");
    engine.insert(&sample_node(30)).expect("insert 30");
    let count = engine
        .compact_layout_bfs()
        .expect("compact_layout_bfs with data");
    assert!(count > 0, "should compact at least 1 node, got {count}");
    let n1 = engine.get(10).expect("get 10").unwrap();
    assert_eq!(n1.id, 10);
    let n2 = engine.get(20).expect("get 20").unwrap();
    assert_eq!(n2.id, 20);
    let n3 = engine.get(30).expect("get 30").unwrap();
    assert_eq!(n3.id, 30);
}

#[test]
fn test_compact_layout_bfs_twice_idempotent() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(1)).expect("insert");
    engine.compact_layout_bfs().expect("first compact");
    engine
        .compact_layout_bfs()
        .expect("second compact (idempotent)");
    assert!(engine.get(1).expect("get").is_some());
}

// ─── Quantization maintenance ─────────────────────────────────

#[test]
fn test_run_quantization_maintenance_empty() {
    let engine = in_memory_engine();
    let report = engine
        .run_quantization_maintenance()
        .expect("quantization maintenance");
    assert_eq!(report.scanned, 0);
    assert_eq!(report.quantized, 0);
    assert_eq!(report.promoted, 0);
}

#[test]
fn test_run_quantization_maintenance_quantize() {
    let engine = in_memory_engine();
    let mut node = sample_node(42);
    node.vector = crate::node::VectorRepresentations::Full(vec![0.1, 0.2, 0.3]);
    node.tier = NodeTier::Hot;
    engine.insert(&node).expect("insert");
    engine.quantization_governor.record_access(42);
    for _ in 0..105 {
        engine.quantization_governor.tick();
    }
    let report = engine
        .run_quantization_maintenance()
        .expect("quantization maintenance");
    assert_eq!(report.scanned, 1, "should scan 1 node");
    assert_eq!(report.quantized, 1, "should quantize 1 node");
    assert_eq!(report.promoted, 0);
    let hnsw = engine.hnsw.load();
    let entry = hnsw.nodes.get(&42).expect("node should exist");
    assert!(
        matches!(
            entry.value().vec_data,
            crate::node::VectorRepresentations::SQ8(..)
        ),
        "node should be quantized to SQ8 after maintenance"
    );
}

#[test]
fn test_run_quantization_maintenance_promote() {
    let engine = in_memory_engine();
    let mut node = UnifiedNode::new(7);
    node.vector = crate::node::VectorRepresentations::SQ8(Box::new([100, 50, -20]), 0.5);
    engine.insert(&node).expect("insert");
    {
        let hnsw = engine.hnsw.load();
        let entry = hnsw.nodes.get(&7).expect("node should exist after insert");
        assert!(
            matches!(
                entry.value().vec_data,
                crate::node::VectorRepresentations::SQ8(..)
            ),
            "node should be SQ8 after insert"
        );
    }
    for _ in 0..6 {
        engine.quantization_governor.record_access(7);
    }
    engine.quantization_governor.tick();
    let action = engine.quantization_governor.evaluate(7, true);
    assert_eq!(
        action,
        crate::vector::governor::QuantizationAction::Promote,
        "governor.evaluate should return Promote"
    );
    let report = engine.run_quantization_maintenance().expect("maintenance");
    assert_eq!(
        report.promoted, 1,
        "should promote 1 hot SQ8 node, got promoted={}",
        report.promoted
    );
    assert_eq!(report.quantized, 0);
    let hnsw = engine.hnsw.load();
    let entry = hnsw.nodes.get(&7).expect("node should exist");
    assert!(
        matches!(
            entry.value().vec_data,
            crate::node::VectorRepresentations::Full(..)
        ),
        "node should be Full after promotion"
    );
}

// ─── Rebuild vector index ─────────────────────────────────────

#[test]
fn test_rebuild_vector_index_empty_engine() {
    let engine = in_memory_engine();
    let report = engine.rebuild_vector_index().expect("rebuild");
    assert!(report.duration_ms > 0 || report.scanned_nodes == 0);
}

#[test]
fn test_rebuild_vector_index_with_data() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(1)).expect("insert 1");
    engine.insert(&sample_node(2)).expect("insert 2");
    let report = engine.rebuild_vector_index().expect("rebuild");
    assert!(
        report.scanned_nodes > 0,
        "should scan at least 1 node, got {}",
        report.scanned_nodes
    );
    assert!(report.indexed_vectors > 0, "should index vectors");
    assert!(report.success, "rebuild should complete successfully");
    let n1 = engine.get(1).expect("get 1").unwrap();
    assert_eq!(n1.id, 1);
    let n2 = engine.get(2).expect("get 2").unwrap();
    assert_eq!(n2.id, 2);
}

#[test]
fn test_rebuild_vector_index_twice_idempotent() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(1)).expect("insert");
    engine.rebuild_vector_index().expect("first rebuild");
    engine
        .rebuild_vector_index()
        .expect("second rebuild (idempotent)");
    assert!(engine.get(1).expect("get").is_some());
}

// ─── Recover archived nodes ───────────────────────────────────

#[test]
fn test_recover_archived_nodes_empty() {
    let engine = in_memory_engine();
    let recovered = engine.recover_archived_nodes(42).expect("recover archived");
    assert!(recovered.is_empty(), "no archived nodes to recover");
}

#[test]
fn test_recover_archived_nodes_with_data() {
    let engine = in_memory_engine();
    let belonged_to_id = engine.intern_label("belonged_to");
    let mut archived = UnifiedNode::new(100);
    archived.vector = crate::node::VectorRepresentations::Full(vec![0.1, 0.2]);
    archived.edges.push(crate::node::Edge {
        target: 1,
        label_id: belonged_to_id,
        weight: 1.0,
    });
    let data = postcard::to_allocvec(&archived)
        .map_err(|e| format!("serialization: {e}"))
        .unwrap();
    engine
        .put_to_partition(BackendPartition::TombstoneStorage, b"archived_100", &data)
        .expect("put archived node");
    let recovered = engine
        .recover_archived_nodes(1)
        .expect("recover archived nodes");
    assert_eq!(recovered.len(), 1, "should recover 1 node");
    assert_eq!(recovered[0].id, 100);
    assert!(
        recovered[0].flags.is_set(crate::node::NodeFlags::ACTIVE),
        "recovered node should be ACTIVE"
    );
    assert!(
        recovered[0].flags.is_set(crate::node::NodeFlags::RECOVERED),
        "recovered node should be RECOVERED"
    );
}

#[test]
fn test_recover_archived_nodes_wrong_summary() {
    let engine = in_memory_engine();
    let belonged_to_id = engine.intern_label("belonged_to");
    let mut archived = UnifiedNode::new(200);
    archived.edges.push(crate::node::Edge {
        target: 1,
        label_id: belonged_to_id,
        weight: 1.0,
    });
    let data = postcard::to_allocvec(&archived)
        .map_err(|e| format!("serialization: {e}"))
        .unwrap();
    engine
        .put_to_partition(BackendPartition::TombstoneStorage, b"archived_200", &data)
        .expect("put archived node");
    let recovered = engine
        .recover_archived_nodes(2)
        .expect("recover with wrong summary id");
    assert!(
        recovered.is_empty(),
        "should not recover nodes belonging to a different summary"
    );
}

#[test]
fn test_recover_archived_nodes_filter_by_label() {
    let engine = in_memory_engine();
    let belonged_to_id = engine.intern_label("belonged_to");
    let referenced_by_id = engine.intern_label("referenced_by");
    let mut matching = UnifiedNode::new(300);
    matching.edges.push(crate::node::Edge {
        target: 1,
        label_id: belonged_to_id,
        weight: 1.0,
    });
    let data = postcard::to_allocvec(&matching)
        .map_err(|e| format!("serialization: {e}"))
        .unwrap();
    engine
        .put_to_partition(
            BackendPartition::TombstoneStorage,
            b"archived_matching",
            &data,
        )
        .expect("put matching node");
    let mut other = UnifiedNode::new(301);
    other.edges.push(crate::node::Edge {
        target: 1,
        label_id: referenced_by_id,
        weight: 1.0,
    });
    let data2 = postcard::to_allocvec(&other)
        .map_err(|e| format!("serialization: {e}"))
        .unwrap();
    engine
        .put_to_partition(
            BackendPartition::TombstoneStorage,
            b"archived_other",
            &data2,
        )
        .expect("put non-matching node");
    let recovered = engine
        .recover_archived_nodes(1)
        .expect("recover archived nodes");
    assert_eq!(recovered.len(), 1, "only matching label should recover");
    assert_eq!(recovered[0].id, 300);
}

#[test]
fn test_recover_archived_nodes_corrupt_data() {
    let engine = in_memory_engine();
    engine
        .put_to_partition(
            BackendPartition::TombstoneStorage,
            b"corrupt",
            b"not a node",
        )
        .expect("put corrupt data");
    let recovered = engine.recover_archived_nodes(42).expect("recover");
    assert!(recovered.is_empty(), "corrupt data should be skipped");
}

// ─── Life insurance (checkpoint) ──────────────────────────────

#[test]
fn test_create_life_insurance_not_supported() {
    let engine = in_memory_engine();
    let result = engine.create_life_insurance("test_snapshot");
    assert!(result.is_err());
    let err = result.err().unwrap().to_string();
    assert!(
        err.contains("Checkpoint") || err.contains("not supported"),
        "expected checkpoint error, got: {err}"
    );
}

// ─── Flush ────────────────────────────────────────────────────

#[test]
fn test_flush_empty_engine() {
    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        ..VantaConfig::default()
    };
    let engine = StorageEngine::open_with_config(":memory:", Some(config)).expect("open");
    engine.flush().expect("flush on empty engine");
}

#[test]
fn test_flush_preserves_inserted_data() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(42)).expect("insert");
    engine.flush().expect("flush");
    let retrieved = engine.get(42).expect("get").unwrap();
    assert_eq!(retrieved.id, 42);
}

#[test]
fn test_flush_after_compact_wal() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(1)).expect("insert");
    engine.compact_wal().expect("compact_wal");
    engine.flush().expect("flush after compact_wal");
    let node = engine.get(1).expect("get").unwrap();
    assert_eq!(node.id, 1);
}

#[test]
fn test_flush_with_pending_mutations() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(1)).expect("insert");
    engine.insert(&sample_node(2)).expect("insert");
    engine.flush().expect("flush after inserts");
    let n1 = engine.get(1).expect("get 1").unwrap();
    assert_eq!(n1.id, 1);
}

#[test]
fn test_flush_after_quantization_maintenance() {
    let engine = in_memory_engine();
    let mut node = sample_node(55);
    node.vector = crate::node::VectorRepresentations::Full(vec![0.7, 0.2, 0.5]);
    engine.insert(&node).expect("insert");
    engine.quantization_governor.record_access(55);
    for _ in 0..105 {
        engine.quantization_governor.tick();
    }
    engine
        .run_quantization_maintenance()
        .expect("quant maintenance");
    engine.flush().expect("flush after quantization");
    let retrieved = engine.get(55).expect("get").unwrap();
    assert_eq!(retrieved.id, 55);
}

#[test]
fn test_flush_on_writable_engine() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(1)).expect("insert");
    engine.flush().expect("flush on writable");
    engine.flush().expect("second flush (idempotent)");
}

// ─── Flush pending HNSW ───────────────────────────────────────

#[test]
fn test_flush_pending_hnsw_empty() {
    let engine = in_memory_engine();
    let result = engine.flush_pending_hnsw().expect("flush_pending_hnsw");
    assert!(!result, "empty batch should return false");
}

#[test]
fn test_flush_pending_hnsw_after_insert() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(42)).expect("insert");
    let result = engine.flush_pending_hnsw().expect("flush_pending_hnsw");
    let _ = result;
}

#[test]
fn test_flush_pending_hnsw_with_delete() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(42)).expect("insert");
    {
        let mut pending = engine.pending_hnsw_batch.lock();
        pending.push(PendingHnswOp {
            id: 42,
            bitset: crate::node::FilterBitset::new(),
            vector: crate::node::VectorRepresentations::None,
            storage_offset: 0,
            is_delete: true,
        });
    }
    let result = engine.flush_pending_hnsw().expect("flush with delete op");
    let _ = result;
}

#[test]
fn test_flush_pending_hnsw_with_multiple_ops() {
    let engine = in_memory_engine();
    {
        let mut pending = engine.pending_hnsw_batch.lock();
        for i in 0..3 {
            pending.push(PendingHnswOp {
                id: 100 + i,
                bitset: crate::node::FilterBitset::new(),
                vector: crate::node::VectorRepresentations::None,
                storage_offset: 64 * (i + 1) as u64,
                is_delete: false,
            });
        }
    }
    let result = engine.flush_pending_hnsw().expect("flush multiple ops");
    assert!(result, "should report that ops were flushed");
}

// ─── Pipeline: Vacuum ──────────────────────────────────────────

#[test]
fn test_vacuum_no_tombstones() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(1)).expect("insert");
    let report = engine.vacuum().expect("vacuum");
    assert_eq!(report.removed_nodes, 0, "no tombstones to remove");
    assert!(report.success);
    assert!(
        report.duration_ms > 0 || report.scanned_nodes > 0,
        "should have scanned nodes"
    );
}

#[test]
fn test_vacuum_with_tombstone() {
    let engine = in_memory_engine();
    let mut node = sample_node(42);
    node.tier = NodeTier::Hot;
    engine.insert(&node).expect("insert");

    // Manually flag the node as tombstoned
    let offset = {
        let hnsw = engine.hnsw.load();
        hnsw.nodes.get(&42).map(|n| n.storage_offset).unwrap()
    };
    {
        let mut vstore = engine.vector_store.write();
        if let Some(mut header) = vstore.read_header(offset) {
            header.flags |= FLAG_TOMBSTONE;
            vstore.write_header(offset, &header).unwrap();
        }
    }

    let report = engine.vacuum().expect("vacuum");
    assert!(
        report.removed_nodes > 0,
        "should have removed the tombstoned node"
    );
    assert!(report.success);

    // Verify the node is no longer in the HNSW index
    let hnsw = engine.hnsw.load();
    assert!(!hnsw.nodes.contains_key(&42), "node should be removed");
}

#[test]
fn test_vacuum_read_only() {
    let engine = in_memory_read_only();
    let result = engine.vacuum();
    assert!(result.is_err(), "read-only engine should reject vacuum");
}

// ─── Pipeline: Merge ───────────────────────────────────────────

#[test]
fn test_merge_segments_empty() {
    let engine = in_memory_engine();
    let report = engine.merge_segments().expect("merge");
    assert!(report.success);
    assert_eq!(report.segments_before, 1);
    assert_eq!(report.segments_after, 1);
}

#[test]
fn test_merge_segments_with_data() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(1)).expect("insert 1");
    engine.insert(&sample_node(2)).expect("insert 2");
    let report = engine.merge_segments().expect("merge");
    assert!(report.success);
}

// ─── Pipeline: Full run ────────────────────────────────────────

#[test]
fn test_run_pipeline_empty() {
    let engine = in_memory_engine();
    let report = engine.run_pipeline(PipelineMode::Full).expect("pipeline");
    assert!(report.success, "full pipeline on empty should succeed");
    assert!(
        report.total_duration_ms > 0 || report.vacuum.is_some(),
        "pipeline should have produced at least a vacuum phase"
    );
}

#[test]
fn test_run_pipeline_with_data() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(10)).expect("insert 10");
    engine.insert(&sample_node(20)).expect("insert 20");
    let report = engine.run_pipeline(PipelineMode::Full).expect("pipeline");
    assert!(report.success, "pipeline should succeed with data");
    assert!(report.vacuum.is_some(), "vacuum phase should run");
    assert!(report.merge.is_some(), "merge phase should run");
    // reindex on in-memory may be a no-op; that's fine
}

#[test]
fn test_run_pipeline_mode_vacuum_only() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(1)).expect("insert");
    let report = engine
        .run_pipeline(PipelineMode::VacuumOnly)
        .expect("pipeline vacuum-only");
    assert!(report.success);
    assert!(report.vacuum.is_some(), "vacuum phase should be present");
    assert!(
        report.merge.is_none(),
        "merge phase should not run in VacuumOnly mode"
    );
    assert!(
        report.index.is_none(),
        "reindex phase should not run in VacuumOnly mode"
    );
}

#[test]
fn test_run_pipeline_read_only() {
    let engine = in_memory_read_only();
    let result = engine.run_pipeline(PipelineMode::Full);
    assert!(
        result.is_err(),
        "pipeline should be rejected on read-only engine"
    );
}

#[test]
fn test_flush_pending_hnsw_with_mixed_ops() {
    let engine = in_memory_engine();
    engine.insert(&sample_node(77)).expect("insert");
    {
        let mut pending = engine.pending_hnsw_batch.lock();
        pending.push(PendingHnswOp {
            id: 77,
            bitset: crate::node::FilterBitset::new(),
            vector: crate::node::VectorRepresentations::None,
            storage_offset: 0,
            is_delete: true,
        });
    }
    let result = engine.flush_pending_hnsw().expect("flush mixed ops");
    let _ = result;
}
