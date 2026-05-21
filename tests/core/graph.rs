//! Graph Traversal Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use vantadb::graph::GraphTraverser;
use vantadb::node::UnifiedNode;
use vantadb::storage::StorageEngine;

#[test]
fn graph_traversal_certification() {
    let mut harness = VantaHarness::new("CORE ENGINE (GRAPH TRAVERSAL)");

    harness.execute("BFS Traversal Matrix", || {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().to_str().unwrap();
        let storage = StorageEngine::open(db_path).unwrap();

        TerminalReporter::sub_step("Building system topology (1->2->3, 1->4)...");
        let mut node1 = UnifiedNode::new(1);
        node1.add_edge(2, "relates_to");
        node1.add_edge(4, "relates_to");
        let mut node2 = UnifiedNode::new(2);
        node2.add_edge(3, "relates_to");
        let node3 = UnifiedNode::new(3);
        let node4 = UnifiedNode::new(4);

        storage.insert(&node1).unwrap();
        storage.insert(&node2).unwrap();
        storage.insert(&node3).unwrap();
        storage.insert(&node4).unwrap();

        let traverser = GraphTraverser::new(&storage);

        TerminalReporter::sub_step("Verifying Depth-1 coverage...");
        let res_d1 = traverser.bfs_traverse(&[1], 1).unwrap();
        assert!(res_d1.contains(&1));
        assert!(res_d1.contains(&2));
        assert!(res_d1.contains(&4));
        assert!(!res_d1.contains(&3));

        TerminalReporter::sub_step("Verifying Depth-2 coverage (reaching terminal nodes)...");
        let res_d2 = traverser.bfs_traverse(&[1], 2).unwrap();
        assert_eq!(res_d2.len(), 4);
        assert!(res_d2.contains(&3));

        TerminalReporter::success("BFS Traversal Axioms satisfied.");
    });
}
