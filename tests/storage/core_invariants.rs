//! Core invariants regression tests for StorageEngine.

use tempfile::tempdir;
use vantadb::node::{FieldValue, UnifiedNode};
use vantadb::storage::{BackendKind, EngineConfig, StorageEngine};

#[test]
fn read_only_rejects_mutations() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    let config = EngineConfig {
        backend_kind: BackendKind::Fjall,
        read_only: true,
        ..Default::default()
    };
    let engine = StorageEngine::open_with_config(db_path, Some(config)).unwrap();

    let node = UnifiedNode::new(1);
    assert!(engine.insert(&node).is_err());
    assert!(engine.delete(1, "test").is_err());
    assert!(engine.purge_permanent(1).is_err());
    assert!(engine.consolidate_node(&node).is_err());
    assert!(engine.recover_archived_nodes(1).is_err());
}

#[test]
fn consolidate_node_keeps_metadata_readable() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    let config = EngineConfig {
        backend_kind: BackendKind::Fjall,
        ..Default::default()
    };
    let engine = StorageEngine::open_with_config(db_path, Some(config)).unwrap();

    let mut node = UnifiedNode::with_vector(42, vec![0.1, 0.2, 0.3, 0.4]);
    node.set_field("nombre", FieldValue::String("Eros".to_string()));
    node.add_edge(7, "creo");

    engine.insert(&node).unwrap();
    engine.consolidate_node(&node).unwrap();

    let roundtrip = engine.get(42).unwrap().expect("node must be readable");
    assert_eq!(
        roundtrip.get_field("nombre").and_then(|v| v.as_str()),
        Some("Eros")
    );
    assert_eq!(roundtrip.edges.len(), 1);
    assert_eq!(roundtrip.edges[0].target, 7);
    assert_eq!(roundtrip.edges[0].label, "creo");
}
