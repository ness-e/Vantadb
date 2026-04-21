//! Durability & Crash Recovery Certification Suite
//!
//! This suite validates that VantaDB can recover data after ungraceful shutdowns
//! using its Write-Ahead Log (WAL) mechanism, specifically for the Fjall backend.

#[path = "common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaSession};
use tempfile::tempdir;
use vantadb::node::UnifiedNode;
use vantadb::storage::{BackendKind, EngineConfig, StorageEngine};

// ─── HELPER: Open Engine ──────────────────────────────────────

fn open_fjall(path: &str) -> StorageEngine {
    let config = EngineConfig {
        backend_kind: BackendKind::Fjall,
        ..Default::default()
    };
    StorageEngine::open_with_config(path, Some(config)).unwrap()
}

// ─── TEST: WAL Recovery Validation ────────────────────────────

#[test]
fn test_fjall_durability_after_shutdown() {
    TerminalReporter::suite_banner("DURABILITY & CRASH RECOVERY CERTIFICATION", 2);
    let mut session = VantaSession::begin("Fjall WAL Recovery");

    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    // PHASE 1: Persistent Flush
    session.step("Phase 1: Writing 500 nodes with explicit flush");
    {
        let engine = open_fjall(db_path);
        for i in 0..500 {
            engine.insert(&UnifiedNode::new(i)).unwrap();
        }
        engine.flush().unwrap(); // Garantiza persistencia en SST
        session.step("Phase 1: Flush completed.");
    } // Engine dropped here

    // PHASE 2: WAL Integrity (The risky part)
    session.step("Phase 2: Writing 500 more nodes WITHOUT flush (WAL only)");
    {
        let engine = open_fjall(db_path);
        for i in 500..1000 {
            engine.insert(&UnifiedNode::new(i)).unwrap();
        }
        // No llamamos a flush() - Los datos están solo en MemTable y WAL
        session.step("Phase 2: 500 nodes in WAL, simulating shutdown.");
    } // Engine dropped simulation

    // PHASE 3: Recovery Validation
    session.step("Phase 3: Reopening engine and verifying total recovery");
    {
        let engine = open_fjall(db_path);

        let mut recovered_count = 0;
        for i in 0..1000 {
            if engine.get(i).unwrap().is_some() {
                recovered_count += 1;
            }
        }

        assert_eq!(
            recovered_count, 1000,
            "CRITICAL: Only recovered {}/1000 nodes",
            recovered_count
        );
        session.step(&format!(
            "Successfully recovered {}/1000 nodes from WAL/SST",
            recovered_count
        ));
    }

    session.success("Fjall durability verified: WAL recovery is working correctly.");
    session.finish(true);
}

// ─── TEST: Sequence Integrity After Reopen ────────────────────

#[test]
fn test_sequence_integrity_after_reopen() {
    let mut session = VantaSession::begin("Storage Sequence Persistence");
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    session.step("Establishing base sequence");
    {
        let engine = open_fjall(db_path);
        engine.insert(&UnifiedNode::new(10)).unwrap();
        engine.delete(10, "cleanup").unwrap();
    }

    session.step("Reopening and verifying tombstone persistence");
    {
        let engine = open_fjall(db_path);
        let node = engine.get(10).unwrap();
        assert!(node.is_none(), "Tombstone was lost after reopen!");
    }

    session.success("Topological sequence is consistent across restarts.");
    session.finish(true);
}

// ─── TEST: Vector Index Cold Recovery (HNSW Integrity) ──────────

#[test]
fn test_vector_index_cold_recovery() {
    let mut session = VantaSession::begin("Vector Index Cold Start Recovery");
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    let target_vector = vec![0.1, 0.2, 0.3, 0.4];

    session.step("Phase 1: Inserting nodes with vectors");
    {
        let engine = open_fjall(db_path);

        let mut node = UnifiedNode::new(42);
        node.vector = vantadb::node::VectorRepresentations::Full(target_vector.clone());
        node.flags.set(vantadb::node::NodeFlags::HAS_VECTOR);

        let mut decoy = UnifiedNode::new(99);
        decoy.vector = vantadb::node::VectorRepresentations::Full(vec![0.9, 0.8, 0.7, 0.6]);
        decoy.flags.set(vantadb::node::NodeFlags::HAS_VECTOR);

        engine.insert(&node).unwrap();
        engine.insert(&decoy).unwrap();

        // El flush obligará a serializar el HNSW (que ahora incluye storage_offset)
        engine.flush().unwrap();
    }

    session.step("Phase 2: Reopening engine and querying vector");
    {
        // Al abrir, el HNSW leerá el archivo con los offsets correctos
        let engine = open_fjall(db_path);

        let hnsw = engine.hnsw.read();
        let vs = engine.vector_store.read();

        // Hacemos una búsqueda directamente contra el índice y VantaFile
        let results = hnsw.search_nearest(&target_vector, None, None, 0, 1, Some(&vs));

        assert_eq!(
            results.len(),
            1,
            "HNSW failed to find any neighbors after restart"
        );
        let (found_id, score) = results[0];

        assert_eq!(
            found_id, 42,
            "HNSW found wrong neighbor, expected 42, got {}",
            found_id
        );

        // Score should be exactly 1.0 since it's the exact vector
        assert!(
            score > 0.99,
            "Similarity score is unexpectedly low: {}",
            score
        );
    }

    session.success("Vector index successfully recovered from cold start.");
    session.finish(true);
}

// ─── SUMMARY ──────────────────────────────────────────────────

#[test]
fn zzz_print_summary() {
    TerminalReporter::print_certification_summary();
}
