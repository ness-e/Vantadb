//! Property-based testing para invariantes de durabilidad (TSK-07)
//!
//! Este módulo usa proptest para verificar invariantes críticos de durabilidad:
//! - WAL: Los registros escritos deben ser recuperables después de un crash
//! - Persistencia: Los datos escritos deben persistir después de flush
//! - Atomicidad: Las operaciones deben ser atómicas (todo o nada)
//! - Consistencia: El estado después de recuperación debe ser consistente

use proptest::prelude::*;
use std::fs;
use tempfile::TempDir;
use vantadb::node::{NodeFlags, NodeTier, UnifiedNode, VectorRepresentations};
use vantadb::storage::StorageEngine;

/// Estrategia para generar nodos de prueba
fn node_strategy() -> impl Strategy<Value = UnifiedNode> {
    (0u64..10000u64, 0u32..100u32).prop_map(|(id, cluster)| UnifiedNode {
        id,
        bitset: 0,
        semantic_cluster: cluster,
        tier: NodeTier::Cold,
        flags: NodeFlags::new(),
        vector: VectorRepresentations::None,
        relational: std::collections::BTreeMap::new(),
        edges: Vec::new(),
        epoch: 0,
        ext_metadata: std::collections::HashMap::new(),
        importance: 0.0,
        last_accessed: 0,
        hits: 0,
        confidence_score: 0.0,
    })
}

// Propiedad: Los nodos insertados deben ser recuperables después de flush
proptest! {
    #[test]
    fn prop_insert_persist_after_flush(nodes in proptest::collection::vec(node_strategy(), 0..10)) {
        prop_assume!(!nodes.is_empty());

        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().to_str().unwrap();

        // Insertar nodos
        {
            let engine = StorageEngine::open(db_path).unwrap();
            for node in &nodes {
                engine.insert(node).unwrap();
            }
            engine.flush().unwrap();
        }

        // Recuperar después de cerrar y reabrir
        {
            let engine = StorageEngine::open(db_path).unwrap();
            let all_nodes = engine.scan_nodes().unwrap();
            assert_eq!(all_nodes.len(), nodes.len());
        }
    }
}

// Propiedad: El tamaño del archivo vector_store debe ser monótonamente creciente
// hasta que se haga compactación
proptest! {
    #[test]
    fn prop_vector_store_monotonic_growth(nodes in proptest::collection::vec(node_strategy(), 0..10)) {
        prop_assume!(!nodes.is_empty());

        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().to_str().unwrap();
        let vector_store_path = temp_dir.path().join("data").join("vector_store.vanta");

        let mut last_size = 0;

        // Insertar nodos incrementalmente y verificar crecimiento
        {
            let engine = StorageEngine::open(db_path).unwrap();

            for node in &nodes {
                engine.insert(node).unwrap();
                engine.flush().unwrap();

                if vector_store_path.exists() {
                    let current_size = fs::metadata(&vector_store_path).unwrap().len();
                    assert!(current_size >= last_size, "Vector store should not shrink without compaction");
                    last_size = current_size;
                }
            }
        }
    }
}

// Propiedad: El número de nodos en el índice debe ser consistente con el almacenamiento
proptest! {
    #[test]
    fn prop_index_storage_consistency(nodes in proptest::collection::vec(node_strategy(), 0..10)) {
        prop_assume!(!nodes.is_empty());

        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().to_str().unwrap();

        // Insertar nodos
        {
            let engine = StorageEngine::open(db_path).unwrap();
            for node in &nodes {
                engine.insert(node).unwrap();
            }
            engine.flush().unwrap();
        }

        // Verificar consistencia
        {
            let engine = StorageEngine::open(db_path).unwrap();

            let stats = engine.get_memory_stats();
            let index_count = stats.node_count;

            // Contar nodos desde el almacenamiento
            let storage_nodes = engine.scan_nodes().unwrap();
            let storage_count = storage_nodes.len() as u64;

            assert_eq!(index_count, storage_count, "Index and storage counts must match");
        }
    }
}
