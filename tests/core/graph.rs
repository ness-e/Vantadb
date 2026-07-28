//! Graph Traversal Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use vantadb::accumulator::GraphAccumulator;
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
        let relates_id = storage.intern_label("relates_to");
        let mut node1 = UnifiedNode::new(1);
        node1.add_edge(2, relates_id);
        node1.add_edge(4, relates_id);
        let mut node2 = UnifiedNode::new(2);
        node2.add_edge(3, relates_id);
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

    harness.execute("Accumulator-assisted Traversal", || {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().to_str().unwrap();
        let storage = StorageEngine::open(db_path).unwrap();

        // Build 3-node chain: 1 → 2 → 3
        let relates_id = storage.intern_label("relates_to");
        let mut node1 = UnifiedNode::new(1);
        node1.add_edge(2, relates_id);
        let mut node2 = UnifiedNode::new(2);
        node2.add_edge(3, relates_id);
        let node3 = UnifiedNode::new(3);

        storage.insert(&node1).unwrap();
        storage.insert(&node2).unwrap();
        storage.insert(&node3).unwrap();

        let traverser = GraphTraverser::new(&storage);
        let acc = GraphAccumulator::new();

        TerminalReporter::sub_step("Traversing 3-node chain with accumulator...");
        let visited = traverser
            .traverse_with_accumulator(&[1], 10, &acc, |_id, _edges, _acc| {
                // Each discovered node contributes 1.0 per outgoing edge
                _edges.len() as f64
            })
            .unwrap();

        assert_eq!(visited.len(), 3, "should discover all 3 nodes");
        assert!(visited.contains(&1));
        assert!(visited.contains(&2));
        assert!(visited.contains(&3));

        // Verify accumulator: node1 has 1 edge, node2 has 1 edge, node3 has 0
        let snap = acc.snapshot();
        assert_eq!(snap.len(), 3, "accumulator has 3 entries");
        assert!(
            (snap[&1] - 1.0).abs() < 1e-10,
            "node1 should have 1.0 (1 outgoing edge)"
        );
        assert!(
            (snap[&2] - 1.0).abs() < 1e-10,
            "node2 should have 1.0 (1 outgoing edge)"
        );
        assert!(
            (snap[&3] - 0.0).abs() < 1e-10,
            "node3 should have 0.0 (no outgoing edges)"
        );

        TerminalReporter::success("Accumulator traversal axioms satisfied.");
    });
}
