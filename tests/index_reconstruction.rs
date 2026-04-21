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
// Vector import removed as it is now internal to NodeRepresentations

// ─── HELPER: Open Engine ──────────────────────────────────────

fn open_engine(path: &str) -> StorageEngine {
    let config = EngineConfig {
        backend_kind: BackendKind::Fjall,
        ..Default::default()
    };
    StorageEngine::open_with_config(path, Some(config)).unwrap()
}

// ─── TEST: Index Cold Boot Reconstruction ─────────────────────

#[test]
fn test_index_reconstruction_from_storage() {
    TerminalReporter::suite_banner("INDEX RECONSTRUCTION & COLD BOOT CERTIFICATION", 2);
    let mut session = VantaSession::begin("HNSW Reconstruction");

    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    // PHASE 1: Populate and Create Index
    session.step("Phase 1: Indexing 1000 vectors");
    {
        let engine = open_engine(db_path);
        for i in 0..1000 {
            let mut node = UnifiedNode::new(i);
            // Create a simple deterministic vector
            node.vector = vantadb::node::VectorRepresentations::Full(vec![i as f32; 32]);
            node.flags.set(vantadb::node::NodeFlags::HAS_VECTOR);
            engine.insert(&node).unwrap();
        }
        engine.flush().unwrap();
        session.step("Phase 1: Data persisted.");
    }

    // PHASE 2: Simulate Index Corruption (Delete HNSW files)
    session.step("Phase 2: Simulating index corruption (Deleting .hnsw files)");
    let hnsw_path = dir.path().join("hnsw_index"); // Assuming standard path
    if hnsw_path.exists() {
        fs::remove_dir_all(&hnsw_path).unwrap();
        session.step("Phase 2: Index files deleted. Storage remains intact.");
    } else {
        session.step(
            "Phase 2: Index path not found, skipping delete (might be in-memory or different path)",
        );
    }

    // PHASE 3: Cold Boot & Rebuild
    session.step("Phase 3: Restarting engine and triggering reconstruction");
    {
        let engine = open_engine(db_path);

        // At this point, the engine should detect missing index and start rebuilding
        // We verify by performing a search
        let query = vec![500.0; 32];
        let results = engine.hnsw.read().search_nearest(
            &query,
            None,
            None,
            u128::MAX,
            5,
            Some(&engine.vector_store.read()),
        );

        assert!(!results.is_empty(), "Search failed after reconstruction!");
        assert_eq!(results[0].0, 500, "Accuracy drift after reconstruction!");

        session.step(&format!(
            "Phase 3: Search verified. Top match ID: {}",
            results[0].0
        ));
    }

    session.success("Index reconstruction successful: Data integrity is the source of truth.");
    session.finish(true);
}

// ─── SUMMARY ──────────────────────────────────────────────────

#[test]
fn zzz_print_summary() {
    TerminalReporter::print_certification_summary();
}
