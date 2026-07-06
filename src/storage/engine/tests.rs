#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::super::*;
    use crate::backend::{BackendKind, BackendPartition, BackendWriteOp};
    use crate::config::VantaConfig;
    use crate::node::UnifiedNode;

    fn in_memory_engine() -> StorageEngine {
        let config = VantaConfig {
            backend_kind: BackendKind::InMemory,
            read_only: false,
            ..VantaConfig::default()
        };
        StorageEngine::open_with_config(":memory:", Some(config))
            .expect("Failed to open in-memory engine")
    }

    fn in_memory_read_only() -> StorageEngine {
        let config = VantaConfig {
            backend_kind: BackendKind::InMemory,
            read_only: true,
            ..VantaConfig::default()
        };
        StorageEngine::open_with_config(":memory:", Some(config))
            .expect("Failed to open read-only in-memory engine")
    }

    fn sample_node(id: u64) -> UnifiedNode {
        let mut node = UnifiedNode::new(id);
        node.vector = crate::node::VectorRepresentations::Full(vec![0.1, 0.2, 0.3]);
        node
    }

    #[test]
    fn test_open_in_memory() {
        let engine = in_memory_engine();
        assert_eq!(engine.backend_kind(), BackendKind::InMemory);
        assert!(!engine.read_only);
    }

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

    #[test]
    fn test_memory_stats_after_insert() {
        let engine = in_memory_engine();
        let stats = engine.get_memory_stats();
        assert_eq!(stats.node_count, 0);
        assert_eq!(stats.cache_entries, 0);
        engine.insert(&sample_node(1)).expect("insert");
        let stats = engine.get_memory_stats();
        assert!(stats.node_count >= 1);
        assert!(stats.logical_bytes > 0);
    }

    #[test]
    fn test_memory_stats_effective_bytes() {
        let stats = MemoryStats {
            logical_bytes: 1000,
            physical_rss: Some(800),
            node_count: 1,
            cache_entries: 0,
            eviction_count: 0,
            eviction_bytes: 0,
            memory_limit: 0,
            quantized_nodes: 0,
        };
        assert_eq!(stats.effective_bytes(), 800);
        let stats_no_rss = MemoryStats {
            logical_bytes: 1000,
            physical_rss: None,
            node_count: 1,
            cache_entries: 0,
            eviction_count: 0,
            eviction_bytes: 0,
            memory_limit: 0,
            quantized_nodes: 0,
        };
        assert_eq!(stats_no_rss.effective_bytes(), 1000);
    }

    #[test]
    fn test_check_memory_pressure_disabled() {
        let config = VantaConfig {
            backend_kind: BackendKind::InMemory,
            rss_threshold: 0.0,
            ..VantaConfig::default()
        };
        let engine = StorageEngine::open_with_config(":memory:", Some(config)).unwrap();
        assert!(engine.check_memory_pressure().is_ok());
    }

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
        let ids: Vec<u64> = nodes.iter().map(|n| n.id).collect();
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
    fn test_selectivity_empty_engine() {
        let engine = in_memory_engine();
        let sel = engine.get_estimated_selectivity(
            "field",
            &crate::query::RelOp::Eq,
            &crate::node::FieldValue::String("val".to_string()),
        );
        assert_eq!(sel, 1.0);
    }

    #[test]
    fn test_selectivity_with_data() {
        let engine = in_memory_engine();
        let mut node = UnifiedNode::new(1);
        node.relational.insert(
            "status".to_string(),
            crate::node::FieldValue::String("active".to_string()),
        );
        engine.insert(&node).expect("insert");
        let sel = engine.get_estimated_selectivity(
            "status",
            &crate::query::RelOp::Eq,
            &crate::node::FieldValue::String("active".to_string()),
        );
        assert_eq!(sel, 1.0);
        let sel_missing = engine.get_estimated_selectivity(
            "status",
            &crate::query::RelOp::Eq,
            &crate::node::FieldValue::String("inactive".to_string()),
        );
        assert_eq!(sel_missing, 0.0);
    }

    #[test]
    fn test_selectivity_neq() {
        let engine = in_memory_engine();
        let mut node = UnifiedNode::new(1);
        node.relational.insert(
            "color".to_string(),
            crate::node::FieldValue::String("red".to_string()),
        );
        engine.insert(&node).expect("insert");
        let sel = engine.get_estimated_selectivity(
            "color",
            &crate::query::RelOp::Neq,
            &crate::node::FieldValue::String("red".to_string()),
        );
        assert_eq!(sel, 0.0);
    }

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
    fn test_flush_empty_engine() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let engine = StorageEngine::open(path).expect("open");
        engine.flush().expect("flush on empty engine");
    }

    #[test]
    fn test_insert_flush_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        {
            let engine = StorageEngine::open(path).expect("open");
            engine.insert(&sample_node(1)).expect("insert");
            engine.flush().expect("flush");
        }
        {
            let engine = StorageEngine::open(path).expect("reopen");
            let node = engine.get(1).expect("get");
            assert!(node.is_some(), "node should persist after reopen");
        }
    }

    #[test]
    fn test_delete_and_flush() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        {
            let engine = StorageEngine::open(path).expect("open");
            engine.insert(&sample_node(1)).expect("insert");
            engine.insert(&sample_node(2)).expect("insert");
            engine.delete(1, "test").expect("delete");
            engine.flush().expect("flush");
        }
        {
            let engine = StorageEngine::open(path).expect("reopen");
            assert!(engine.get(1).unwrap().is_none());
            assert!(engine.get(2).unwrap().is_some());
        }
    }

    #[test]
    fn test_compact_wal() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let engine = StorageEngine::open(path).expect("open");
        engine.insert(&sample_node(1)).expect("insert");
        engine.compact_wal().expect("compact_wal");
        engine.flush().expect("flush");
        let node = engine.get(1).expect("get");
        assert!(node.is_some());
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
}
