// ponytail: blanket allow — unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Integration tests for VantaDB: node CRUD, vector search, graph traversal
//! Modernized with Vanta Certification Framework using StorageEngine / Embedded.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use std::time::Instant;
use tempfile::TempDir;
use vantadb::config::Config;
use vantadb::node::{DistanceMetric, FilterBitset};
use vantadb::storage::StorageEngine;
use vantadb::{Embedded, FieldValue, MemoryInput, NodeInput, UnifiedNode};

#[test]
fn core_engine_certification() {
    let mut harness = VantaHarness::new("CORE ENGINE (CRUD & SEARCH)");

    harness.execute("Node CRUD: Insert & Get", || {
        let engine =
            StorageEngine::open_with_config(":memory:", Some(Config::in_memory())).unwrap();
        let node = UnifiedNode::new(100);
        engine.insert(&node).unwrap();

        let retrieved = engine.get(100).unwrap().unwrap();
        assert_eq!(retrieved.id, 100);
        assert!(
            retrieved.is_alive(),
            "retrieved node should be alive after insert"
        );
        TerminalReporter::success("Basic Insert/Get verified.");
    });

    harness.execute("Node CRUD: Auto-ID Generation", || {
        let db = Embedded::in_memory().unwrap();
        db.put(MemoryInput::new("test", "key1", "payload1"))
            .unwrap();
        db.put(MemoryInput::new("test", "key2", "payload2"))
            .unwrap();
        let r1 = db.get("test", "key1").unwrap().unwrap();
        let r2 = db.get("test", "key2").unwrap().unwrap();
        assert_ne!(r1.node_id, r2.node_id);
        assert!(
            r1.node_id > 0,
            "auto-generated id should be positive, got {}",
            r1.node_id
        );
        assert!(
            r2.node_id > 0,
            "auto-generated id should be positive, got {}",
            r2.node_id
        );
    });

    harness.execute("Node CRUD: Duplicate ID Protection", || {
        let db = Embedded::in_memory().unwrap();
        db.put(MemoryInput::new("cert", "k", "v1")).unwrap();
        let r1 = db.get("cert", "k").unwrap().unwrap();
        assert_eq!(r1.version, 1);
        db.put(MemoryInput::new("cert", "k", "v2")).unwrap();
        let r2 = db.get("cert", "k").unwrap().unwrap();
        assert_eq!(r2.version, 2, "re-inserting same key updates version");
        assert_eq!(r2.payload, "v2");
    });

    harness.execute("Node CRUD: Delete logic", || {
        let engine =
            StorageEngine::open_with_config(":memory:", Some(Config::in_memory())).unwrap();
        engine.insert(&UnifiedNode::new(1)).unwrap();
        engine.delete(1, "delete").unwrap();
        assert!(
            engine.get(1).unwrap().is_none(),
            "deleted node should not be retrievable"
        );
    });

    harness.execute("Node CRUD: Field Update logic", || {
        let engine =
            StorageEngine::open_with_config(":memory:", Some(Config::in_memory())).unwrap();
        engine.insert(&UnifiedNode::new(1)).unwrap();

        let mut updated = UnifiedNode::new(1);
        updated.set_field("name", FieldValue::String("Eros".into()));
        engine.insert(&updated).unwrap();

        let node = engine.get(1).unwrap().unwrap();
        assert_eq!(
            node.get_field("name"),
            Some(&FieldValue::String("Eros".into()))
        );
    });

    harness.execute("Bitset: Multidimensional Scan", || {
        let engine =
            StorageEngine::open_with_config(":memory:", Some(Config::in_memory())).unwrap();
        for i in 1..=100 {
            let mut node = UnifiedNode::new(i);
            if i % 2 == 0 {
                node.set_bit(5);
            } // VZLA
            if i % 3 == 0 {
                node.set_bit(16);
            } // active
            engine.insert(&node).unwrap();
        }
        let vzla_mask = FilterBitset::from(1u128 << 5);
        let nodes = engine.scan_nodes().unwrap();
        let vzla: Vec<_> = nodes
            .iter()
            .filter(|n| n.bitset.matches_mask(&vzla_mask))
            .collect();
        assert_eq!(vzla.len(), 50);
        let both_mask = FilterBitset::from((1u128 << 5) | (1u128 << 16));
        let both: Vec<_> = nodes
            .iter()
            .filter(|n| n.bitset.matches_mask(&both_mask))
            .collect();
        assert_eq!(both.len(), 16);
        TerminalReporter::success("Cross-bitset filtering validated.");
    });

    harness.execute("Vector: Exact Top-K Search", || {
        let engine =
            StorageEngine::open_with_config(":memory:", Some(Config::in_memory())).unwrap();
        engine
            .insert(&UnifiedNode::with_vector(1, vec![1.0, 0.0, 0.0]))
            .unwrap();
        engine
            .insert(&UnifiedNode::with_vector(2, vec![0.9, 0.1, 0.0]))
            .unwrap();
        engine
            .insert(&UnifiedNode::with_vector(3, vec![0.0, 1.0, 0.0]))
            .unwrap();

        let results = engine.vec_index().search(
            &[1.0, 0.0, 0.0],
            &FilterBitset::all_set(),
            2,
            None,
            DistanceMetric::Cosine,
        );
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, 1);
        assert_eq!(results[1].0, 2);
    });

    harness.execute("Graph: Relation Traversal & Hops", || {
        let db = Embedded::in_memory().unwrap();
        db.insert_node(NodeInput {
            id: 1,
            content: Some("node 1".into()),
            vector: None,
            fields: Default::default(),
        })
        .unwrap();
        db.insert_node(NodeInput {
            id: 2,
            content: Some("node 2".into()),
            vector: None,
            fields: Default::default(),
        })
        .unwrap();
        db.insert_node(NodeInput {
            id: 3,
            content: Some("node 3".into()),
            vector: None,
            fields: Default::default(),
        })
        .unwrap();
        db.insert_node(NodeInput {
            id: 4,
            content: Some("node 4".into()),
            vector: None,
            fields: Default::default(),
        })
        .unwrap();

        db.add_edge(1, 2, "amigo", None, None).unwrap();
        db.add_edge(2, 3, "amigo", None, None).unwrap();
        db.add_edge(3, 4, "amigo", None, None).unwrap();

        let edges1: Vec<_> = db
            .get_node(1)
            .unwrap()
            .unwrap()
            .edges
            .into_iter()
            .filter(|e| !e.reverse)
            .collect();
        assert_eq!(edges1.len(), 1);
        assert_eq!(edges1[0].target, 2);

        let edges2: Vec<_> = db
            .get_node(2)
            .unwrap()
            .unwrap()
            .edges
            .into_iter()
            .filter(|e| !e.reverse)
            .collect();
        assert_eq!(edges2.len(), 1);
        assert_eq!(edges2[0].target, 3);
    });

    harness.execute("Vector Retrieval: Bitset + Vector + Fields", || {
        let engine =
            StorageEngine::open_with_config(":memory:", Some(Config::in_memory())).unwrap();
        for i in 1..=10 {
            let mut node = UnifiedNode::with_vector(i, vec![i as f32, 0.0, 0.0]);
            node.set_field("pais", FieldValue::String("VZLA".into()));
            if i % 2 == 0 {
                node.set_bit(5);
            }
            engine.insert(&node).unwrap();
        }
        let hybrid_mask = FilterBitset::from(1u128 << 5);
        let results = engine.vec_index().search(
            &[10.0, 0.0, 0.0],
            &hybrid_mask,
            3,
            None,
            DistanceMetric::Cosine,
        );
        assert_eq!(results.len(), 3);
        for (id, _) in &results {
            assert_eq!(id % 2, 0);
        }
    });

    harness.execute("WAL: Persistence & Recovery", || {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().to_str().unwrap();
        {
            let engine = StorageEngine::open(db_path).unwrap();
            let mut node = UnifiedNode::new(42);
            node.set_field("name", FieldValue::String("test".into()));
            engine.insert(&node).unwrap();
            engine.flush().unwrap();
        }
        {
            let engine = StorageEngine::open(db_path).unwrap();
            let node = engine.get(42).unwrap().unwrap();
            assert_eq!(
                node.get_field("name"),
                Some(&FieldValue::String("test".into()))
            );
        }
    });

    harness.execute("System: Basic Engine Stats", || {
        let engine =
            StorageEngine::open_with_config(":memory:", Some(Config::in_memory())).unwrap();
        engine
            .insert(&UnifiedNode::with_vector(1, vec![1.0, 2.0, 3.0]))
            .unwrap();
        let mut n2 = UnifiedNode::new(2);
        n2.add_edge(1, 42);
        engine.insert(&n2).unwrap();
        let nodes = engine.scan_nodes().unwrap();
        assert_eq!(nodes.len(), 2);
    });

    harness.execute("Benchmark: 10K Node Throughput", || {
        let engine =
            StorageEngine::open_with_config(":memory:", Some(Config::in_memory())).unwrap();
        let start = Instant::now();
        for i in 1..=10_000u64 {
            let node = UnifiedNode::new(i.into());
            engine.insert(&node).unwrap();
        }
        let elapsed = start.elapsed();
        let count = engine.scan_nodes().unwrap().len();
        assert_eq!(
            count, 10_000,
            "engine should contain exactly 10_000 nodes after bulk insert"
        );
        assert!(
            elapsed.as_millis() < 2500,
            "10K inserts should complete quickly, took {}ms",
            elapsed.as_millis()
        );
        TerminalReporter::success(&format!(
            "BENCH: 10k inserts in {:?} ({:.1} μs/insert)",
            elapsed,
            elapsed.as_micros() as f64 / 10_000.0
        ));
    });
}
