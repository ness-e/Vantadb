//! 🔍 Fuzzing basado en propiedades (Property-Based Testing) para validación cross-platform.
//!
//! Ejecuta: `cargo test fuzz_proptest`
//!
//! Este módulo valida que el código de deserialización de VantaDB maneja inputs
//! aleatorios/corruptos de forma segura, sin entrar en pánico.

use proptest::prelude::*;
use std::collections::HashSet;
use vantadb::config::VantaConfig;
use vantadb::node::UnifiedNode;
use vantadb::wal::WalRecord;
use vantadb::BackendKind;
use vantadb::{
    FieldValue, InMemoryEngine, VantaEmbedded, VantaError, VantaMemoryInput, VantaMemoryMetadata,
};

proptest! {
    /// Test 1: WalRecord debe manejar bytes aleatorios sin panic
    #[test]
    fn test_wal_record_deserialize_random_bytes(data: Vec<u8>) {
        let _: Result<WalRecord, _> = bincode::deserialize(&data);
    }

    /// Test 2: UnifiedNode debe manejar bytes aleatorios sin panic
    #[test]
    fn test_unified_node_deserialize_random_bytes(data: Vec<u8>) {
        let _: Result<UnifiedNode, _> = bincode::deserialize(&data);
    }

    /// Test 3: Roundtrip con IDs generados aleatoriamente
    #[test]
    fn test_unified_node_roundtrip_with_random_id(id: u64) {
        let node = UnifiedNode::new(id);
        let serialized = bincode::serialize(&node).unwrap();
        let deserialized: UnifiedNode = bincode::deserialize(&serialized).unwrap();
        prop_assert_eq!(node.id, deserialized.id);
    }

    /// Test 4: Node ID uniqueness — no duplicate IDs collide
    #[test]
    fn test_node_id_uniqueness(ids in proptest::collection::vec(0u64..10000, 1..20)) {
        let engine = InMemoryEngine::new();
        let mut seen = HashSet::new();
        for id in ids {
            if seen.contains(&id) {
                let result = engine.insert(UnifiedNode::new(id));
                prop_assert!(result.is_err(), "Duplicate insert of id {} should fail", id);
            } else {
                let result = engine.insert(UnifiedNode::new(id));
                prop_assert!(result.is_ok(), "First insert of id {} should succeed", id);
                seen.insert(id);
            }
        }
    }

    /// Test 5: Vector roundtrip — insert with random vector, retrieve, verify match
    #[test]
    fn test_vector_roundtrip(vec in proptest::collection::vec(-1.0f32..1.0, 1..=64)) {
        let engine = InMemoryEngine::new();
        let id = 42u64;
        let node = UnifiedNode::with_vector(id, vec.clone());
        let _ = engine.insert(node).unwrap();
        let retrieved = engine.get(id).unwrap();
        prop_assert_eq!(retrieved.vector.to_f32(), Some(vec));
    }

    /// Test 6: Metadata roundtrip — insert with random metadata, retrieve, verify match
    #[test]
    fn test_metadata_roundtrip(
        key in "[a-zA-Z_][a-zA-Z0-9_]{0,15}",
        value in ".{0,50}",
    ) {
        let engine = InMemoryEngine::new();
        let id = 99u64;
        let mut node = UnifiedNode::new(id);
        node.set_field(&key, FieldValue::String(value.clone()));
        let _ = engine.insert(node).unwrap();
        let retrieved = engine.get(id).unwrap();
        let expected = Some(&FieldValue::String(value));
        prop_assert_eq!(retrieved.get_field(&key), expected);
    }

    /// Test 7: Delete idempotency — second delete returns NodeNotFound, never panics
    #[test]
    fn test_delete_idempotency(id: u64) {
        let engine = InMemoryEngine::new();
        prop_assert!(engine.insert(UnifiedNode::new(id)).is_ok());
        prop_assert!(engine.delete(id).is_ok());

        let result = engine.delete(id);
        prop_assert!(result.is_err(), "Second delete should return an error");
        match result {
            Err(VantaError::NodeNotFound(returned_id)) => prop_assert_eq!(returned_id, id),
            _ => panic!("Expected NodeNotFound, got {:?}", result),
        }
    }
}

/// Test 8: TTL boundary — node with expires_at_ms = now+1ms is purged after waiting.
/// This test is outside the proptest! macro because it involves time and I/O.
#[test]
fn test_ttl_boundary_purge_expired() {
    let dir = tempfile::tempdir().unwrap();
    let config = VantaConfig {
        storage_path: dir.path().to_string_lossy().to_string(),
        backend_kind: BackendKind::InMemory,
        ..Default::default()
    };
    let db = VantaEmbedded::open_with_config(config).unwrap();

    let input = VantaMemoryInput {
        namespace: "test_ns".into(),
        key: "ttl_key".into(),
        payload: "ephemeral data".into(),
        metadata: VantaMemoryMetadata::new(),
        vector: None,
        ttl_ms: Some(1),
    };

    let record = db.put(input).unwrap();
    assert!(record.expires_at_ms.is_some());

    std::thread::sleep(std::time::Duration::from_millis(2));

    let purged = db.purge_expired().unwrap();
    assert_eq!(
        purged, 1,
        "purge_expired should remove exactly one expired record"
    );

    let retrieved = db.get("test_ns", "ttl_key").unwrap();
    assert!(
        retrieved.is_none(),
        "Expired record should be gone after purge"
    );
}
