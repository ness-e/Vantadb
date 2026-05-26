//! RocksDB Engine Integration Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use tempfile::tempdir;
use vantadb::node::UnifiedNode;
use vantadb::storage::StorageEngine;

#[test]
fn storage_engine_certification() {
    let mut harness = VantaHarness::new("STORAGE LAYER (ROCKSDB ADAPTER)");

    harness.execute("Integration: Persistent Node IO", || {
        let dir = tempdir().unwrap();
        let db_path = dir.path().to_str().unwrap();

        TerminalReporter::sub_step("Opening StorageEngine (RocksDB backend)...");
        let storage = StorageEngine::open(db_path).expect("Failed to open RocksDB");

        let node = UnifiedNode::new(42);
        storage.insert(&node).unwrap();
        TerminalReporter::sub_step("Node 42 committed to persistent storage.");

        let retrieved = storage
            .get(42)
            .unwrap()
            .expect("Node not found after insertion");
        assert_eq!(retrieved.id, 42);

        TerminalReporter::success("RocksDB roundtrip successful.");
    });
}

#[test]
fn storage_engine_read_only_barrier_test() {
    let mut harness = VantaHarness::new("STORAGE LAYER READ-ONLY BARRIER");

    harness.execute("Integration: Read-Only Rejection", || {
        let dir = tempdir().unwrap();
        let db_path = dir.path();

        TerminalReporter::sub_step("Opening StorageEngine in read-only mode...");
        use vantadb::config::VantaConfig;
        // Creamos una configuración de solo lectura
        // Nota: para evitar que falle porque el directorio no existe, primero creamos un Storage Engine normal
        // para inicializar el directorio, o bien nos aseguramos de que el backend pueda abrirlo.
        // Pero de hecho, el backend in-memory o RocksDB de solo lectura requiere que el directorio exista si es read-only.
        // Primero abrimos y cerramos un StorageEngine de escritura normal para inicializar los archivos:
        {
            let _init = StorageEngine::open(db_path.to_str().unwrap())
                .expect("Failed to init storage directory");
        }

        let config = VantaConfig::default()
            .with_read_only(true)
            .with_storage_path(db_path.to_str().unwrap().to_string());

        let storage = StorageEngine::open_with_config(db_path.to_str().unwrap(), Some(config))
            .expect("Failed to open read-only engine");

        let node = UnifiedNode::new(42);
        let insert_res = storage.insert(&node);
        assert!(
            insert_res.is_err(),
            "Insert should fail on read-only storage"
        );
        let err_msg = insert_res.err().unwrap().to_string();
        assert!(
            err_msg.contains("read-only"),
            "Error should mention read-only: {}",
            err_msg
        );

        let delete_res = storage.delete(42, "read-only validation test");
        assert!(
            delete_res.is_err(),
            "Delete should fail on read-only storage"
        );

        let flush_res = storage.flush();
        assert!(flush_res.is_err(), "Flush should fail on read-only storage");

        TerminalReporter::success("Read-only barrier validated successfully.");
    });
}

#[test]
fn storage_engine_file_locking_test() {
    let mut harness = VantaHarness::new("STORAGE LAYER FILE LOCKING");

    harness.execute("Integration: Multi-Process File Lock Barrier", || {
        let dir = tempdir().unwrap();
        let db_path = dir.path().to_str().unwrap();

        TerminalReporter::sub_step("Opening first StorageEngine (holds the lock)...");
        let _storage1 = StorageEngine::open(db_path).expect("Failed to open first StorageEngine");

        TerminalReporter::sub_step("Opening second StorageEngine (should fail due to lock)...");
        let storage2_res = StorageEngine::open(db_path);
        
        assert!(
            storage2_res.is_err(),
            "Opening the same database concurrently must fail"
        );
        let err_msg = storage2_res.err().unwrap().to_string();
        assert!(
            err_msg.contains("locked by another process") || err_msg.contains("Only one VantaDB instance"),
            "Error message should indicate lock failure, got: {}",
            err_msg
        );

        TerminalReporter::sub_step("Releasing first StorageEngine lock...");
        drop(_storage1);

        TerminalReporter::sub_step("Opening second StorageEngine again (should succeed)...");
        let _storage2 = StorageEngine::open(db_path).expect("Failed to open StorageEngine after lock release");

        TerminalReporter::success("File locking barrier validated successfully.");
    });
}

