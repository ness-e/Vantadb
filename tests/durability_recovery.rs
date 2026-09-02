// ponytail: blanket allow — unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Durability & Crash Recovery Certification Suite
//!
//! This suite validates that VantaDB can recover data after ungraceful shutdowns
//! using its Write-Ahead Log (WAL) mechanism, specifically for the Fjall backend.

#[path = "common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaSession};
use std::sync::Arc;
use tempfile::tempdir;
use vantadb::config::VantaConfig;
use vantadb::node::UnifiedNode;
use vantadb::storage::{BackendKind, StorageEngine};

// ─── HELPER: Open Engine ──────────────────────────────────────

fn open_fjall(path: &str) -> StorageEngine {
    let config = VantaConfig {
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

        let hnsw = engine.hnsw.load();
        let vs = engine.vector_store[0].read();

        // Hacemos una búsqueda directamente contra el índice y VantaFile
        let results = hnsw.search_nearest(
            &target_vector,
            None,
            None,
            &vantadb::node::ALL_BITSET,
            1,
            Some(&vs),
        );

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

// ─── TEST: WAL Replay Idempotence ───────────────────────────────

#[test]
fn test_wal_replay_idempotence() {
    let mut session = VantaSession::begin("WAL Replay Idempotence");
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    session.step("Phase 1: Insert without flush");
    {
        let engine = open_fjall(db_path);
        for i in 1..=50 {
            engine.insert(&UnifiedNode::new(i)).unwrap();
        }
    }

    session.step("Phase 2: First reopen (should recover 50 nodes)");
    {
        let engine = open_fjall(db_path);
        let mut count = 0;
        for i in 1..=100 {
            if engine.get(i).unwrap().is_some() {
                count += 1;
            }
        }
        assert_eq!(count, 50, "First reopen should recover exactly 50 nodes");
    }

    session.step("Phase 3: Second reopen (verifying no duplication or corruption)");
    {
        let engine = open_fjall(db_path);
        let mut count = 0;
        for i in 1..=100 {
            if engine.get(i).unwrap().is_some() {
                count += 1;
            }
        }
        assert_eq!(count, 50, "Second reopen must still have exactly 50 nodes");
    }

    session.success("WAL replay is strictly idempotent.");
    session.finish(true);
}

// ─── TEST: WAL Replay Mixed Mutations ─────────────────────────────

#[test]
fn test_wal_replay_mixed_mutations() {
    let mut session = VantaSession::begin("WAL Replay Mixed Mutations");
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    session.step("Phase 1: Insert, Update, and Delete without flush");
    {
        let engine = open_fjall(db_path);
        // Insert nodes 1, 2, 3
        engine.insert(&UnifiedNode::new(1)).unwrap();
        engine.insert(&UnifiedNode::new(2)).unwrap();
        engine.insert(&UnifiedNode::new(3)).unwrap();

        // Delete node 2
        engine.delete(2, "test delete").unwrap();

        // Update node 3
        let mut updated_node3 = UnifiedNode::new(3);
        updated_node3.importance = 99.0; // Mark a visible change
                                         // En VantaDB usamos insert() para update (upsert semántico en este contexto de test de memoria)
        engine.insert(&updated_node3).unwrap();
    }

    session.step("Phase 2: Reopen and verify exact state");
    {
        let engine = open_fjall(db_path);

        // Node 1 should exist
        assert!(engine.get(1).unwrap().is_some(), "Node 1 should exist");

        // Node 2 should be deleted
        assert!(engine.get(2).unwrap().is_none(), "Node 2 should be deleted");

        // Node 3 should exist with updated importance
        let node3 = engine.get(3).unwrap().expect("Node 3 should exist");
        assert_eq!(
            node3.importance, 99.0,
            "Node 3 should have updated importance"
        );
    }

    session.success("WAL handles mixed mutations correctly during replay.");
    session.finish(true);
}

// ─── TEST: ERR-010 checkpoint_seq ↔ snapshot interleave ─────────────

// Regression for ERR-010: flush() holds the HNSW insert_lock across
// [drain → serialize → checkpoint_seq write], and every mutating path
// (insert/delete) appends its WAL record and queues its HNSW mutation under
// the SAME guard. Without that, a concurrent insert's WAL record can be
// counted into checkpoint_seq while its HNSW mutation misses the serialized
// snapshot — replay then skips the record (invisible node in the vector
// index) or the opposite: a mutation lands in the snapshot whose record is
// NOT counted → replay re-applies it (duplicate entry).
#[test]
fn test_checkpoint_snapshot_interleave_not_lost_or_duplicated() {
    let mut session = VantaSession::begin("Checkpoint/Snapshot Interleave (ERR-010)");
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    const NODES: u128 = 512;

    fn target_for(id: u128) -> Vec<f32> {
        vec![
            (id % 97) as f32 / 97.0,
            (id % 31) as f32 / 31.0,
            0.25,
            0.125,
        ]
    }

    session.step("Phase 1: Concurrent inserts + flushes (forcing checkpoint/snapshot interleaves)");
    let engine = Arc::new(open_fjall(db_path));

    // Writer thread keeps inserting vectors while the main thread flushes,
    // so WAL appends + HNSW queues race the checkpoint critical section.
    let writer_engine = Arc::clone(&engine);
    let writer = std::thread::spawn(move || {
        for i in 0..NODES {
            let mut node = UnifiedNode::new(i as u128);
            node.vector = vantadb::node::VectorRepresentations::Full(target_for(i));
            node.flags.set(vantadb::node::NodeFlags::HAS_VECTOR);
            writer_engine.insert(&node).unwrap();
        }
    });

    for _ in 0..8 {
        engine.flush().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    writer.join().unwrap();
    engine.flush().unwrap();
    drop(engine);

    session.step("Phase 2: Reopen and assert exact index recovery");
    let engine = open_fjall(db_path);
    let hnsw = engine.hnsw.load();
    let vs = engine.vector_store[0].read();

    let indexed_len = hnsw.nodes.len() as u64;
    assert!(
        indexed_len >= NODES as u64,
        "Index lost records: expected >= {NODES} nodes, got {indexed_len} (invisible records)"
    );

    let mut missing = 0;
    for i in 0..NODES {
        // KV data must be durable either way.
        assert!(
            engine.get(i as u128).unwrap().is_some(),
            "node {i} missing from KV after recovery"
        );
        // The HNSW entry must exist (no checkpoint-skipped invisible record).
        if hnsw.nodes.get(&(i as u128)).is_none() {
            missing += 1;
        }
        // And it must not be duplicated inside the index (same id twice in
        // the exact-vector neighborhood).
        let results = hnsw.search_nearest(
            &target_for(i),
            None,
            None,
            &vantadb::node::ALL_BITSET,
            16,
            Some(&vs),
        );
        let dup = results.iter().filter(|(id, _)| *id == i as u128).count();
        assert!(
            dup <= 1,
            "node {i} appears {dup} times in index after recovery (duplicate)"
        );
    }
    assert_eq!(
        missing, 0,
        "{missing} nodes are invisible in the vector index after recovery (checkpoint skipped their WAL record without snapshot capture)"
    );

    session.success("No invisible or duplicate records across checkpoint/snapshot interleaves.");
    session.finish(true);
}

// ─── TEST: ERR-010 snapshot failure must NOT advance checkpoint_seq ──

// The lock makes the [drain → snapshot → count] critical section atomic, but
// the ordering ALSO guarantees durability on the failure path: if
// save_vector_index fails, the count + checkpoint write below it are never
// reached, so the previous checkpoint_seq stays in place and WAL replay on
// reopen covers every record above it (no data loss). This test forces that
// failure with the failpoints feature and asserts the WAL replay fully
// recovers the failed flush.
#[cfg(feature = "failpoints")]
#[test]
fn test_checkpoint_not_advanced_on_snapshot_failure() {
    let mut session = VantaSession::begin("Checkpoint NOT advanced on snapshot failure (ERR-010)");
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    const NODES: u128 = 128;

    fn target_for(id: u128) -> Vec<f32> {
        vec![
            (id % 97) as f32 / 97.0,
            (id % 31) as f32 / 31.0,
            0.25,
            0.125,
        ]
    }

    session.step("Phase 1: Insert nodes, flush cleanly, then flush with a forced snapshot failure");
    {
        let engine = open_fjall(db_path);
        for i in 0..NODES {
            let mut node = UnifiedNode::new(i as u128);
            node.vector = vantadb::node::VectorRepresentations::Full(target_for(i));
            node.flags.set(vantadb::node::NodeFlags::HAS_VECTOR);
            engine.insert(&node).unwrap();
        }
        engine.flush().unwrap();

        // Arm the failpoint: the next save_vector_index errors out, so the
        // checkpoint_seq write must be skipped entirely.
        fail::cfg("snapshot_serialize_fail", "return").unwrap();
        let res = engine.flush();
        fail::cfg("snapshot_serialize_fail", "off").unwrap();
        assert!(
            res.is_err(),
            "snapshot_serialize_fail should make flush() error (bug: checkpoint/drain proceeded after failed snapshot)"
        );
    }

    session.step("Phase 2: Reopen — every node must be recoverable via WAL replay");
    {
        let engine = open_fjall(db_path);
        let hnsw = engine.hnsw.load();
        for i in 0..NODES {
            assert!(
                engine.get(i as u128).unwrap().is_some(),
                "node {i} lost after failed snapshot + replay"
            );
            assert!(
                hnsw.nodes.get(&(i as u128)).is_some(),
                "node {i} invisible in vector index after failed snapshot + replay"
            );
        }
        let indexed_len = hnsw.nodes.len() as u64;
        assert!(
            indexed_len == NODES as u64,
            "expected exactly {NODES} indexed nodes, got {indexed_len}"
        );
    }

    session
        .success("Checkpoint did not advance past the failed snapshot; replay recovered all data.");
    session.finish(true);
}

// ─── SUMMARY ──────────────────────────────────────────────────

#[test]
fn zzz_print_summary() {
    TerminalReporter::print_certification_summary();
}
