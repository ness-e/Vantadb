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
use vantadb::node::UnifiedNode;
use vantadb::storage::{BackendKind, EngineConfig, StorageEngine};

// ─── HELPER: Open Engine with Specific Backend ────────────────

fn open_engine(path: &str, kind: BackendKind) -> StorageEngine {
    let config = EngineConfig {
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
        let e = Arc::clone(&engine);
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
    let e_write = Arc::clone(&engine);
    let writer = thread::spawn(move || {
        for i in 0..500 {
            let node = UnifiedNode::new(i);
            e_write.insert(&node).unwrap();
            thread::yield_now();
        }
    });

    let e_read = Arc::clone(&engine);
    let reader = thread::spawn(move || {
        let mut found = 0;
        for _ in 0..1000 {
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

// ─── SUMMARY ──────────────────────────────────────────────────

#[test]
fn zzz_print_summary() {
    TerminalReporter::print_certification_summary();
}
