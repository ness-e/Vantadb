//! Storage Garbage Collection Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use common::{VantaHarness, TerminalReporter};
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::tempdir;
use vantadb::gc::GcWorker;
use vantadb::node::UnifiedNode;
use vantadb::storage::StorageEngine;

#[test]
fn storage_gc_certification() {
    let mut harness = VantaHarness::new("STORAGE LAYER (GARBAGE COLLECTION)");

    harness.execute("Sweep Logic: TTL Expiry & Physical Purge", || {
        let dir = tempdir().unwrap();
        let db_path = dir.path().to_str().unwrap();
        let storage = StorageEngine::open(db_path).unwrap();

        TerminalReporter::sub_step("Initializing nodes with TTL (Node 1=Expired, Node 2=Active)...");
        let node1 = UnifiedNode::new(1);
        let node2 = UnifiedNode::new(2);
        storage.insert(&node1).unwrap();
        storage.insert(&node2).unwrap();

        let mut worker = GcWorker::new(&storage);
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        worker.register_ttl(1, now - 10);  // Past
        worker.register_ttl(2, now + 100); // Future

        TerminalReporter::sub_step("Executing sweep cycle...");
        let purged = worker.sweep();

        assert_eq!(purged, 1, "GC failed to purge expired node");
        assert!(storage.get(1).unwrap().is_none(), "Node 1 should be physically deleted");
        assert!(storage.get(2).unwrap().is_some(), "Node 2 should be preserved");
        
        TerminalReporter::success(&format!("Sweep cycle successful. Purged {} expired nodes.", purged));
    });
}
