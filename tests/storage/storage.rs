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
