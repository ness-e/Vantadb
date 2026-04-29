//! Integration tests for VantaDB Fase 1: node CRUD, vector search, graph traversal
//! Modernized with Vanta Certification Framework.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use std::time::Instant;
use vantadb::{FieldValue, InMemoryEngine, UnifiedNode};

#[test]
fn core_engine_certification() {
    let mut harness = VantaHarness::new("CORE ENGINE (CRUD & SEARCH)");

    harness.execute("Node CRUD: Insert & Get", || {
        let engine = InMemoryEngine::new();
        let node = UnifiedNode::new(100);
        let id = engine.insert(node).unwrap();
        assert_eq!(id, 100);

        let retrieved = engine.get(100).unwrap();
        assert_eq!(retrieved.id, 100);
        assert!(retrieved.is_alive());
        TerminalReporter::success("Basic Insert/Get verified.");
    });

    harness.execute("Node CRUD: Auto-ID Generation", || {
        let engine = InMemoryEngine::new();
        let id1 = engine.insert(UnifiedNode::new(0)).unwrap();
        let id2 = engine.insert(UnifiedNode::new(0)).unwrap();
        assert_ne!(id1, id2);
        assert!(id1 > 0);
        assert!(id2 > 0);
    });

    harness.execute("Node CRUD: Duplicate ID Protection", || {
        let engine = InMemoryEngine::new();
        engine.insert(UnifiedNode::new(42)).unwrap();
        let err = engine.insert(UnifiedNode::new(42));
        assert!(err.is_err());
    });

    harness.execute("Node CRUD: Delete logic", || {
        let engine = InMemoryEngine::new();
        engine.insert(UnifiedNode::new(1)).unwrap();
        engine.delete(1).unwrap();
        assert!(engine.get(1).is_none());
    });

    harness.execute("Node CRUD: Field Update logic", || {
        let engine = InMemoryEngine::new();
        engine.insert(UnifiedNode::new(1)).unwrap();

        let mut updated = UnifiedNode::new(1);
        updated.set_field("name", FieldValue::String("Eros".into()));
        engine.update(1, updated).unwrap();

        let node = engine.get(1).unwrap();
        assert_eq!(
            node.get_field("name"),
            Some(&FieldValue::String("Eros".into()))
        );
    });

    harness.execute("Bitset: Multidimensional Scan", || {
        let engine = InMemoryEngine::new();
        for i in 1..=100 {
            let mut node = UnifiedNode::new(i);
            if i % 2 == 0 {
                node.set_bit(5);
            } // VZLA
            if i % 3 == 0 {
                node.set_bit(16);
            } // active
            engine.insert(node).unwrap();
        }
        let vzla = engine.scan_bitset(1u128 << 5);
        assert_eq!(vzla.len(), 50);
        let both = engine.scan_bitset((1u128 << 5) | (1u128 << 16));
        assert_eq!(both.len(), 16);
        TerminalReporter::success("Cross-bitset filtering validated.");
    });

    harness.execute("Vector: Exact Top-K Search", || {
        let engine = InMemoryEngine::new();
        engine
            .insert(UnifiedNode::with_vector(1, vec![1.0, 0.0, 0.0]))
            .unwrap();
        engine
            .insert(UnifiedNode::with_vector(2, vec![0.9, 0.1, 0.0]))
            .unwrap();
        engine
            .insert(UnifiedNode::with_vector(3, vec![0.0, 1.0, 0.0]))
            .unwrap();

        let result = engine.vector_search(&[1.0, 0.0, 0.0], 2, 0.5, None);
        assert_eq!(result.nodes.len(), 2);
        assert_eq!(result.nodes[0].id, 1);
        assert_eq!(result.nodes[1].id, 2);
    });

    harness.execute("Graph: Relation Traversal & Hops", || {
        let engine = InMemoryEngine::new();
        let mut n1 = UnifiedNode::new(1);
        n1.add_edge(2, "amigo");
        let mut n2 = UnifiedNode::new(2);
        n2.add_edge(3, "amigo");
        let mut n3 = UnifiedNode::new(3);
        n3.add_edge(4, "amigo");
        let n4 = UnifiedNode::new(4);

        engine.insert(n1).unwrap();
        engine.insert(n2).unwrap();
        engine.insert(n3).unwrap();
        engine.insert(n4).unwrap();

        let result = engine.traverse(1, "amigo", 1, 2).unwrap();
        assert_eq!(result.len(), 2);
        let result_full = engine.traverse(1, "amigo", 1, 3).unwrap();
        assert_eq!(result_full.len(), 3);
    });

    harness.execute("Vector Retrieval: Bitset + Vector + Fields", || {
        let engine = InMemoryEngine::new();
        for i in 1..=10 {
            let mut node = UnifiedNode::with_vector(i, vec![i as f32, 0.0, 0.0]);
            node.set_field("pais", FieldValue::String("VZLA".into()));
            if i % 2 == 0 {
                node.set_bit(5);
            }
            engine.insert(node).unwrap();
        }
        let result = engine.hybrid_search(
            &[10.0, 0.0, 0.0],
            3,
            0.5,
            Some(1u128 << 5),
            &[("pais".to_string(), FieldValue::String("VZLA".into()))],
        );
        assert_eq!(result.nodes.len(), 3);
        for node in &result.nodes {
            assert_eq!(node.id % 2, 0);
        }
    });

    harness.execute("WAL: Persistence & Recovery", || {
        let wal_path = std::env::temp_dir().join("vanta_wal_modern_test.bin");
        let _ = std::fs::remove_file(&wal_path);
        {
            let engine = InMemoryEngine::with_wal(&wal_path).unwrap();
            let mut node = UnifiedNode::new(42);
            node.set_field("name", FieldValue::String("test".into()));
            engine.insert(node).unwrap();
            engine.flush_wal().unwrap();
        }
        {
            let engine = InMemoryEngine::with_wal(&wal_path).unwrap();
            let node = engine.get(42).unwrap();
            assert_eq!(
                node.get_field("name"),
                Some(&FieldValue::String("test".into()))
            );
        }
        let _ = std::fs::remove_file(&wal_path);
    });

    harness.execute("System: Basic Engine Stats", || {
        let engine = InMemoryEngine::new();
        engine
            .insert(UnifiedNode::with_vector(1, vec![1.0, 2.0, 3.0]))
            .unwrap();
        let mut n2 = UnifiedNode::new(2);
        n2.add_edge(1, "knows");
        engine.insert(n2).unwrap();
        let stats = engine.stats();
        assert_eq!(stats.node_count, 2);
        assert_eq!(stats.vector_count, 1);
    });

    harness.execute("Benchmark: 10K Node Throughput", || {
        let engine = InMemoryEngine::new();
        let start = Instant::now();
        for i in 1..=10_000u64 {
            let node = UnifiedNode::new(i);
            engine.insert(node).unwrap();
        }
        let elapsed = start.elapsed();
        assert_eq!(engine.node_count(), 10_000);
        assert!(elapsed.as_millis() < 500);
        TerminalReporter::success(&format!(
            "BENCH: 10k inserts in {:?} ({:.1} μs/insert)",
            elapsed,
            elapsed.as_micros() as f64 / 10_000.0
        ));
    });
}
