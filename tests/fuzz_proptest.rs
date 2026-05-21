//! 🔍 Fuzzing basado en propiedades (Property-Based Testing) para validación cross-platform.
//!
//! Ejecuta: `cargo test fuzz_proptest`
//!
//! Este módulo valida que el código de deserialización de VantaDB maneja inputs
//! aleatorios/corruptos de forma segura, sin entrar en pánico.

use proptest::prelude::*;
use vantadb::node::UnifiedNode;
use vantadb::wal::WalRecord;

proptest! {
    /// Test 1: WalRecord debe manejar bytes aleatorios sin panic
    #[test]
    fn test_wal_record_deserialize_random_bytes(data: Vec<u8>) {
        // Intentar deserializar bytes aleatorios como WalRecord.
        // Debe fallar limpiamente con Err, nunca entrar en pánico.
        let _: Result<WalRecord, _> = bincode::deserialize(&data);
    }

    /// Test 2: UnifiedNode debe manejar bytes aleatorios sin panic
    #[test]
    fn test_unified_node_deserialize_random_bytes(data: Vec<u8>) {
        // Intentar deserializar bytes aleatorios como UnifiedNode.
        let _: Result<UnifiedNode, _> = bincode::deserialize(&data);
    }

    /// Test 3: Roundtrip con IDs generados aleatoriamente
    #[test]
    fn test_unified_node_roundtrip_with_random_id(id: u64) {
        // Generar un nodo válido con ID aleatorio y verificar roundtrip
        let node = UnifiedNode::new(id);
        let serialized = bincode::serialize(&node).unwrap();
        let deserialized: UnifiedNode = bincode::deserialize(&serialized).unwrap();

        // Verificar que el ID se preserva en el roundtrip
        prop_assert_eq!(node.id, deserialized.id);
    }
}
