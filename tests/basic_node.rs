//! Integration tests for IADBMS Fase 1: node CRUD, vector search, graph traversal

use iadbms::{FieldValue, InMemoryEngine, UnifiedNode};
use std::time::Instant;

#[test]
fn test_insert_and_get() {
    let engine = InMemoryEngine::new();
    let node = UnifiedNode::new(100);
    let id = engine.insert(node).unwrap();
    assert_eq!(id, 100);

    let retrieved = engine.get(100).unwrap();
    assert_eq!(retrieved.id, 100);
    assert!(retrieved.is_alive());
}

#[test]
fn test_auto_id() {
    let engine = InMemoryEngine::new();
    let id1 = engine.insert(UnifiedNode::new(0)).unwrap();
    let id2 = engine.insert(UnifiedNode::new(0)).unwrap();
    assert_ne!(id1, id2);
    assert!(id1 > 0);
    assert!(id2 > 0);
}

#[test]
fn test_duplicate_id_error() {
    let engine = InMemoryEngine::new();
    engine.insert(UnifiedNode::new(42)).unwrap();
    let err = engine.insert(UnifiedNode::new(42));
    assert!(err.is_err());
}

#[test]
fn test_delete() {
    let engine = InMemoryEngine::new();
    engine.insert(UnifiedNode::new(1)).unwrap();
    engine.delete(1).unwrap();
    assert!(engine.get(1).is_none());
}

#[test]
fn test_update() {
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
}

#[test]
fn test_bitset_scan() {
    let engine = InMemoryEngine::new();

    // Bit 5 = VZLA, Bit 16 = active
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

    // VZLA only: 50 nodes
    let vzla = engine.scan_bitset(1u128 << 5);
    assert_eq!(vzla.len(), 50);

    // VZLA AND active: divisible by 6 → 16 nodes
    let both = engine.scan_bitset((1u128 << 5) | (1u128 << 16));
    assert_eq!(both.len(), 16);
}

#[test]
fn test_vector_search() {
    let engine = InMemoryEngine::new();

    // Insert 3 nodes with 3D vectors
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
    assert_eq!(result.nodes[0].id, 1); // most similar
    assert_eq!(result.nodes[1].id, 2); // second
    assert!(!result.is_partial);
    assert_eq!(result.exhaustivity, 1.0);
}

#[test]
fn test_graph_traversal() {
    let engine = InMemoryEngine::new();

    // Build: 1 -amigo-> 2 -amigo-> 3 -amigo-> 4
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

    // Traverse 1..2 hops
    let result = engine.traverse(1, "amigo", 1, 2).unwrap();
    assert_eq!(result.len(), 2); // nodes 2 and 3
    assert!(result.iter().any(|(id, depth)| *id == 2 && *depth == 1));
    assert!(result.iter().any(|(id, depth)| *id == 3 && *depth == 2));

    // Traverse 1..3 hops
    let result = engine.traverse(1, "amigo", 1, 3).unwrap();
    assert_eq!(result.len(), 3); // nodes 2, 3, 4
}

#[test]
fn test_hybrid_search() {
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
        Some(1u128 << 5), // only even IDs have bit 5
        &[("pais".to_string(), FieldValue::String("VZLA".into()))],
    );

    assert_eq!(result.nodes.len(), 3);
    // All results should have even IDs (bitset filter)
    for node in &result.nodes {
        assert_eq!(node.id % 2, 0);
    }
}

#[test]
fn test_field_filter() {
    let engine = InMemoryEngine::new();

    let mut n1 = UnifiedNode::new(1);
    n1.set_field("pais", FieldValue::String("VZLA".into()));
    let mut n2 = UnifiedNode::new(2);
    n2.set_field("pais", FieldValue::String("USA".into()));
    let mut n3 = UnifiedNode::new(3);
    n3.set_field("pais", FieldValue::String("VZLA".into()));

    engine.insert(n1).unwrap();
    engine.insert(n2).unwrap();
    engine.insert(n3).unwrap();

    let vzla = engine.filter_field("pais", &FieldValue::String("VZLA".into()));
    assert_eq!(vzla.len(), 2);
}

#[test]
fn test_wal_persistence() {
    let wal_path = std::env::temp_dir().join("iadbms_test_wal_persist.bin");
    let _ = std::fs::remove_file(&wal_path);

    // Write
    {
        let engine = InMemoryEngine::with_wal(&wal_path).unwrap();
        let mut node = UnifiedNode::new(42);
        node.set_field("name", FieldValue::String("test".into()));
        engine.insert(node).unwrap();
        engine.flush_wal().unwrap();
    }

    // Recover
    {
        let engine = InMemoryEngine::with_wal(&wal_path).unwrap();
        let node = engine.get(42).unwrap();
        assert_eq!(
            node.get_field("name"),
            Some(&FieldValue::String("test".into()))
        );
    }

    let _ = std::fs::remove_file(&wal_path);
}

#[test]
fn test_stats() {
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
    assert_eq!(stats.edge_count, 1);
    assert_eq!(stats.total_dimensions, 3);
}

/// Performance: insert 10k nodes in < 1 second (target: <1ms each)
#[test]
fn test_insert_10k_performance() {
    let engine = InMemoryEngine::new();
    let start = Instant::now();

    for i in 1..=10_000u64 {
        let node = UnifiedNode::new(i);
        engine.insert(node).unwrap();
    }

    let elapsed = start.elapsed();
    assert_eq!(engine.node_count(), 10_000);
    // Target: 10k inserts in < 500ms (conservative, no WAL)
    assert!(
        elapsed.as_millis() < 500,
        "10k inserts took {}ms (target: <500ms)",
        elapsed.as_millis()
    );
    eprintln!(
        "BENCH: 10k node inserts in {:?} ({:.1} μs/insert)",
        elapsed,
        elapsed.as_micros() as f64 / 10_000.0
    );
}
