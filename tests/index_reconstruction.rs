//! Index Reconstruction Certification Suite
//!
//! This suite validates that VantaDB can rebuild its entire HNSW index
//! purely from the underlying storage (Fjall/RocksDB) if index files are missing.

#[path = "common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaSession};
use std::fs;
use tempfile::tempdir;
use vantadb::node::UnifiedNode;
use vantadb::storage::{BackendKind, EngineConfig, StorageEngine};

// ─── HELPER: Open Engine ──────────────────────────────────────

fn open_engine(path: &str) -> StorageEngine {
    let config = EngineConfig {
        backend_kind: BackendKind::Fjall,
        ..Default::default()
    };
    StorageEngine::open_with_config(path, Some(config)).unwrap()
}

// ─── TEST A: Index Persistence Roundtrip ──────────────────────

#[test]
fn test_index_persistence_roundtrip() {
    TerminalReporter::suite_banner("INDEX PERSISTENCE ROUNDTRIP CERTIFICATION", 2);
    let mut session = VantaSession::begin("HNSW Roundtrip");

    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    // PHASE 1: Populate and Create Index
    session.step("Phase 1: Indexing nodes");
    {
        let engine = open_engine(db_path);
        for i in 0..100 {
            let mut node = UnifiedNode::new(i);
            let mut v = vec![0.0; 32];
            v[0] = i as f32;
            v[1] = 100.0 - i as f32; // Ensure non-collinear
            node.vector = vantadb::node::VectorRepresentations::Full(v);
            node.flags.set(vantadb::node::NodeFlags::HAS_VECTOR);
            engine.insert(&node).unwrap();
        }
        engine.flush().unwrap(); // Force index write to vector_index.bin
        session.step("Phase 1: Data flushed and index saved.");
    }

    // PHASE 2: Reopen and Verify
    session.step("Phase 2: Reopening engine and verifying persistence");
    {
        let engine = open_engine(db_path);
        let mut query = vec![0.0; 32];
        query[0] = 50.0;
        query[1] = 50.0;
        let results = engine.hnsw.read().search_nearest(
            &query,
            None,
            None,
            u128::MAX,
            5,
            Some(&engine.vector_store.read()),
        );

        assert!(!results.is_empty(), "Search failed after reload!");
        assert_eq!(results[0].0, 50, "Accuracy drift after reload!");

        session.step(&format!(
            "Phase 2: Verification successful. Match ID: {}",
            results[0].0
        ));
    }

    session.success("Index persistence roundtrip successful.");
    session.finish(true);
}

// ─── TEST B: Index Reconstruction from Storage ────────────────

#[test]
fn test_index_reconstruction_from_storage() {
    TerminalReporter::suite_banner("INDEX COLD RECONSTRUCTION CERTIFICATION", 2);
    let mut session = VantaSession::begin("HNSW Reconstruction");

    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    // PHASE 1: Populate and Create Index
    session.step("Phase 1: Indexing nodes and flushing");
    {
        let engine = open_engine(db_path);
        for i in 0..100 {
            let mut node = UnifiedNode::new(i);
            let mut v = vec![0.0; 32];
            v[0] = i as f32;
            v[1] = 100.0 - i as f32; // Ensure non-collinear
            node.vector = vantadb::node::VectorRepresentations::Full(v);
            node.flags.set(vantadb::node::NodeFlags::HAS_VECTOR);
            engine.insert(&node).unwrap();
        }
        engine.flush().unwrap();
    }

    // PHASE 2: Simulate Total Index Loss (Delete vector_index.bin)
    session.step("Phase 2: Deleting vector_index.bin (Simulating index loss)");
    let index_file = dir.path().join("data").join("vector_index.bin");
    if index_file.exists() {
        fs::remove_file(&index_file).unwrap();
        session.step("Phase 2: vector_index.bin deleted.");
    } else {
        panic!(
            "vector_index.bin was not created at expected path: {:?}",
            index_file
        );
    }

    // PHASE 3: Reopen and Rebuild
    session.step("Phase 3: Restarting engine (should trigger rebuild from KV/VantaFile)");
    {
        let engine = open_engine(db_path);

        // Verification
        let mut query = vec![0.0; 32];
        query[0] = 50.0;
        query[1] = 50.0;
        let results = engine.hnsw.read().search_nearest(
            &query,
            None,
            None,
            u128::MAX,
            5,
            Some(&engine.vector_store.read()),
        );

        if results.is_empty() {
            session.step("Phase 3 FAILURE: Index is empty. Reconstruction not working.");
            panic!("StorageEngine failed to rebuild index from storage when vector_index.bin was missing.");
        }

        assert_eq!(results[0].0, 50, "Accuracy drift after reconstruction!");
        session.step(&format!(
            "Phase 3: Search verified. Match ID: {}",
            results[0].0
        ));
    }

    session.success("Index reconstruction from storage successful.");
    session.finish(true);
}

// ─── SUMMARY ──────────────────────────────────────────────────

#[test]
fn zzz_print_summary() {
    TerminalReporter::print_certification_summary();
}
