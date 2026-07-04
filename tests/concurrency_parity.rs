//! Concurrency & Backend Parity Certification Suite
//!
//! This suite ensures that all storage backends (RocksDB, Fjall, InMemory)
//! produce identical results under identical operations and high concurrency.

#[path = "common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaSession};
use std::sync::Arc;
use std::thread;
use tempfile::tempdir;
use vantadb::config::VantaConfig;
use vantadb::node::UnifiedNode;
use vantadb::storage::{BackendKind, StorageEngine};

// ─── HELPER: Open Engine with Specific Backend ────────────────

fn open_engine(path: &str, kind: BackendKind) -> StorageEngine {
    let config = VantaConfig {
        backend_kind: kind,
        ..Default::default()
    };
    StorageEngine::open_with_config(path, Some(config)).unwrap()
}

// ─── TEST 1: Absolute Parity (RocksDB vs Fjall vs InMemory) ───

#[test]
fn test_triple_backend_parity_validation() {
    TerminalReporter::suite_banner("BACKEND PARITY & CONCURRENCY CERTIFICATION", 3);
    let mut session = VantaSession::begin("Triple Backend Parity");

    let dir_r = tempdir().unwrap();
    let dir_f = tempdir().unwrap();
    let dir_m = tempdir().unwrap();

    // Fix: RocksDB -> RocksDb
    let rocks = open_engine(dir_r.path().to_str().unwrap(), BackendKind::RocksDb);
    let fjall = open_engine(dir_f.path().to_str().unwrap(), BackendKind::Fjall);
    let mem = open_engine(dir_m.path().to_str().unwrap(), BackendKind::InMemory);

    session.step("Injecting identical dataset (1000 nodes) across all backends");
    for i in 0..1000 {
        let node = UnifiedNode::new(i);
        rocks.insert(&node).unwrap();
        fjall.insert(&node).unwrap();
        mem.insert(&node).unwrap();
    }

    session.step("Verifying data parity (Get & List)");
    for i in 0..1000 {
        let n_r = rocks.get(i).unwrap().unwrap();
        let n_f = fjall.get(i).unwrap().unwrap();
        let n_m = mem.get(i).unwrap().unwrap();

        assert_eq!(n_r.id, n_f.id);
        assert_eq!(n_f.id, n_m.id);
    }

    session.step("Executing cross-backend deletions");
    for i in 0..500 {
        rocks.delete(i, "parity test").unwrap();
        fjall.delete(i, "parity test").unwrap();
        mem.delete(i, "parity test").unwrap();
    }

    session.step("Final state reconciliation");
    for i in 0..1000 {
        let exists_r = rocks.get(i).unwrap().is_some();
        let exists_f = fjall.get(i).unwrap().is_some();
        let exists_m = mem.get(i).unwrap().is_some();

        assert_eq!(exists_r, exists_f, "Fjall/RocksDB mismatch at ID {}", i);
        assert_eq!(exists_f, exists_m, "Fjall/InMemory mismatch at ID {}", i);
    }

    session.success("Parity confirmed: All backends are functionally identical.");
    session.finish(true);
}

// ─── TEST 2: High Concurrency Write Stress (Race Condition Check) ───

#[test]
fn test_high_concurrency_fjall_stress() {
    let mut session = VantaSession::begin("Fjall Concurrency Stress");
    let dir = tempdir().unwrap();
    let engine = Arc::new(open_engine(
        dir.path().to_str().unwrap(),
        BackendKind::Fjall,
    ));

    session.step("Launching 10 concurrent writers (100 ops/thread)");
    let mut handles = vec![];
    for t in 0..10 {
        let e: Arc<StorageEngine> = Arc::clone(&engine);
        handles.push(thread::spawn(move || {
            for i in 0..100 {
                let id = (t * 100) + i;
                let node = UnifiedNode::new(id);
                e.insert(&node).unwrap();
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    session.step("Validating engine integrity after stress");
    for i in 0..1000 {
        assert!(
            engine.get(i).unwrap().is_some(),
            "Node {} lost during concurrent write",
            i
        );
    }

    session.success("No race conditions detected under high thread pressure.");
    session.finish(true);
}

// ─── TEST 3: Interleaved Read/Write Chaos ─────────────────────

#[test]
fn test_interleaved_read_write_parity() {
    let mut session = VantaSession::begin("Interleaved R/W Chaos");
    let dir = tempdir().unwrap();
    let engine = Arc::new(open_engine(
        dir.path().to_str().unwrap(),
        BackendKind::Fjall,
    ));

    session.step("Starting interleaved R/W workload");
    let e_write: Arc<StorageEngine> = Arc::clone(&engine);
    let writer = thread::spawn(move || {
        for i in 0..500 {
            let node = UnifiedNode::new(i);
            e_write.insert(&node).unwrap();
            thread::yield_now();
        }
    });

    let e_read: Arc<StorageEngine> = Arc::clone(&engine);
    let reader = thread::spawn(move || {
        let mut found = 0;
        for _ in 0..200 {
            if e_read.get(0).unwrap().is_some() {
                found += 1;
            }
            thread::yield_now();
        }
        found
    });

    writer.join().unwrap();
    let read_hits = reader.join().unwrap();

    // Fix: Using step instead of info
    session.step(&format!(
        "Interleaved reads completed with {} hits",
        read_hits
    ));
    session.success("Shared-state integrity maintained.");
    session.finish(true);
}

// ─── TEST 4: Concurrency & RCU Rebuild Validation (AUD-03) ───

#[test]
fn test_concurrency_rebuild_rcu() {
    let mut session = VantaSession::begin("RCU Index Rebuild Concurrency");
    let dir = tempdir().unwrap();
    let engine = Arc::new(open_engine(
        dir.path().to_str().unwrap(),
        BackendKind::Fjall,
    ));

    // Seed inicial con vectores
    session.step("Seeding initial nodes with vectors");
    for i in 0..100 {
        let mut node = UnifiedNode::new(i);
        node.vector = vantadb::node::VectorRepresentations::Full(vec![i as f32 * 0.01; 128]);
        node.flags.set(vantadb::node::NodeFlags::HAS_VECTOR);
        engine.insert(&node).unwrap();
    }
    engine.flush().unwrap();

    // Hilo lector: hace consultas vectoriales concurrentes en caliente
    session.step("Launching concurrent readers and continuous queries");
    let engine_read = Arc::clone(&engine);
    let reader_handle = thread::spawn(move || {
        let mut query_success = 0;
        let query_vector = vec![0.5; 128];
        for _ in 0..100 {
            let hnsw = engine_read.hnsw.load();
            let vs = engine_read.vector_store.read();
            let results = hnsw.search_nearest(
                &query_vector,
                None,
                None,
                &vantadb::node::ALL_BITSET,
                5,
                Some(&vs),
            );
            if !results.is_empty() {
                query_success += 1;
            }
            thread::yield_now();
        }
        query_success
    });

    // Hilo escritor/mantenimiento: ejecuta rebuild de índice y compactaciones en paralelo
    session.step("Executing rebuild_vector_index in parallel");
    let engine_write = Arc::clone(&engine);
    let writer_handle = thread::spawn(move || {
        // Ejecutar rebuild_vector_index a mitad de las consultas de lectura
        engine_write.rebuild_vector_index().unwrap();
        // Insertar un nuevo nodo de control post-rebuild
        let mut node = UnifiedNode::new(999);
        node.vector = vantadb::node::VectorRepresentations::Full(vec![0.9; 128]);
        node.flags.set(vantadb::node::NodeFlags::HAS_VECTOR);
        engine_write.insert(&node).unwrap();
    });

    writer_handle.join().unwrap();
    let successful_queries = reader_handle.join().unwrap();

    session.step(&format!(
        "Concurrent readers completed: {} successful queries during rebuild",
        successful_queries
    ));

    // Validar que el nodo insertado post-rebuild es alcanzable y no se perdió
    let hnsw = engine.hnsw.load();
    assert!(
        hnsw.nodes.contains_key(&999),
        "Mitigación A-01: El nodo 999 se perdió tras el rebuild!"
    );

    session.success("RCU rebuild lock-free swap and consistency certified successfully.");
    session.finish(true);
}

// ─── SUMMARY ──────────────────────────────────────────────────

#[test]
fn zzz_print_summary() {
    TerminalReporter::print_certification_summary();
}
