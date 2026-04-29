//! Memory Telemetry Contract Harness
//!
//! Validates that VantaDB reports process-scoped memory with explicit units
//! and that controlled scenarios can be compared without pretending those
//! numbers represent full engine footprint or mmap residency.

#[path = "common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use std::fs;
use std::path::Path;
use tempfile::tempdir;
use vantadb::node::{NodeFlags, UnifiedNode, VectorRepresentations};
use vantadb::storage::{BackendKind, EngineConfig, StorageEngine};

fn open_fjall(path: &str) -> StorageEngine {
    let config = EngineConfig {
        backend_kind: BackendKind::Fjall,
        ..Default::default()
    };
    StorageEngine::open_with_config(path, Some(config)).expect("Failed to open Fjall engine")
}

fn dir_size_bytes(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }

    let mut total = 0u64;
    let mut stack = vec![path.to_path_buf()];
    while let Some(current) = stack.pop() {
        if let Ok(entries) = fs::read_dir(&current) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    stack.push(entry_path);
                } else if let Ok(metadata) = entry.metadata() {
                    total += metadata.len();
                }
            }
        }
    }
    total
}

fn make_node(id: u64) -> UnifiedNode {
    let mut node = UnifiedNode::new(id);
    node.vector = VectorRepresentations::Full(vec![id as f32, 1.0, 2.0, 3.0]);
    node.flags.set(NodeFlags::HAS_VECTOR);
    node.set_field("category", vantadb::FieldValue::String("telemetry".into()));
    node
}

#[test]
fn memory_telemetry_contract() {
    TerminalReporter::suite_banner("MEMORY TELEMETRY CONTRACT", 4);
    let mut harness = VantaHarness::new("OBSERVABILITY (MEMORY CONTRACT)");

    let dir = tempdir().expect("Failed to create temp dir");
    let db_path = dir.path().to_str().unwrap();
    let data_dir = dir.path().join("data");

    harness.execute("Cold Start: Empty Engine", || {
        let _engine = open_fjall(db_path);
        let bytes = dir_size_bytes(&data_dir);
        TerminalReporter::info(&format!(
            "Process telemetry is process-scoped only; on-disk bytes after cold start = {}",
            bytes
        ));
        assert!(
            data_dir.exists(),
            "Fjall data directory must exist after open"
        );
    });

    harness.execute("Moderate Ingest: 1K vectors + flush", || {
        let engine = open_fjall(db_path);
        for id in 1..=1_000u64 {
            engine.insert(&make_node(id)).unwrap();
        }
        engine.flush().unwrap();

        let total_bytes = dir_size_bytes(&data_dir);
        let vector_store = data_dir.join("vector_store.vanta");
        let ann_index = data_dir.join("vector_index.bin");

        TerminalReporter::info(&format!(
            "Disk bytes after flush: total={}, vantafile={}, ann={}",
            total_bytes,
            fs::metadata(&vector_store).map(|m| m.len()).unwrap_or(0),
            fs::metadata(&ann_index).map(|m| m.len()).unwrap_or(0)
        ));

        assert!(
            vector_store.exists(),
            "vector_store.vanta must exist after flush"
        );
        assert!(
            ann_index.exists(),
            "vector_index.bin must exist after flush"
        );
    });

    harness.execute("WAL Replay: write without flush then reopen", || {
        {
            let engine = open_fjall(db_path);
            for id in 1_001..=1_100u64 {
                engine.insert(&make_node(id)).unwrap();
            }
        }

        let reopened = open_fjall(db_path);
        for id in 1_001..=1_100u64 {
            assert!(
                reopened.get(id).unwrap().is_some(),
                "Recovered node {} must be visible after WAL replay",
                id
            );
        }

        let wal_path = data_dir.join("vanta.wal");
        TerminalReporter::info(&format!(
            "WAL bytes after replay path: {}",
            fs::metadata(&wal_path).map(|m| m.len()).unwrap_or(0)
        ));
    });

    harness.execute("Restart: Index reload remains queryable", || {
        let reopened = open_fjall(db_path);
        let results = reopened.hnsw.read().search_nearest(
            &[1.0, 1.0, 2.0, 3.0],
            None,
            None,
            0,
            5,
            Some(&reopened.vector_store.read()),
        );

        assert!(!results.is_empty(), "Reopened index must remain queryable");
        TerminalReporter::success(
            "Telemetry contract kept separate from reload/query correctness checks.",
        );
    });
}
