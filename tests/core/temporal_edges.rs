#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Temporal edges certification — COMP-021.
//!
//! `Edge.created_at_ms` propagation (forward + reverse), temporal traversal
//! window filtering, and serde backward-compat for the new field.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use vantadb::graph::{GraphTraverser, TraversalDirection};
use vantadb::node::{Edge, UnifiedNode};
use vantadb::storage::StorageEngine;
use vantadb::{VantaEmbedded, VantaNodeInput};

/// Build a 3-node chain with explicit edge timestamps:
///   1 --ts=100--> 2 --ts=200--> 3
fn build_chain(storage: &StorageEngine, relates: u32) {
    let mut n1 = UnifiedNode::new(1);
    n1.edges.push(Edge::with_timestamp(2, relates, 100));
    let mut n2 = UnifiedNode::new(2);
    n2.edges.push(Edge::with_timestamp(3, relates, 200));
    let n3 = UnifiedNode::new(3);

    storage.insert(&n1).unwrap();
    storage.insert(&n2).unwrap();
    storage.insert(&n3).unwrap();
}

/// (1) Backward-compat: an edge serialized without `created_at_ms` deserializes
/// as `0` via `#[serde(default)]`.
///
/// Note: exercised through `serde_json`, a self-describing format that applies
/// serde defaults for missing fields. Postcard (the on-disk storage format,
/// `postcard = 1.1`) does NOT apply defaults for truncated trailing fields —
/// it errors with `DeserializeUnexpectedEnd`. New data round-trips correctly
/// through postcard (covered by `postcard_roundtrip_preserves_ts`).
#[test]
fn backward_compat_old_edge_without_created_at_ms() {
    let mut harness = VantaHarness::new("TEMPORAL EDGES (BACKWARD COMPAT)");

    harness.execute(
        "Old-shape edge JSON deserializes with created_at_ms == 0",
        || {
            let old_edge_json = r#"{"target": 2, "label_id": 7, "weight": 1.0, "reverse": false}"#;
            let edge: Edge = serde_json::from_str(old_edge_json).unwrap();
            assert_eq!(edge.created_at_ms, 0, "missing field must default to 0");
            assert_eq!(edge.target, 2);
            assert!(!edge.reverse);
            TerminalReporter::success("serde default applies for missing created_at_ms.");
        },
    );

    harness.execute("Old-shape node JSON round-trips with edge ts == 0", || {
        // Build a real node, serialize it, strip `created_at_ms` from each edge
        // (simulating data written before the field existed), then deserialize.
        let mut n1 = UnifiedNode::new(1);
        n1.edges.push(Edge::with_timestamp(2, 3, 5000));
        let mut value = serde_json::to_value(&n1).unwrap();
        let edges = value.get_mut("edges").unwrap().as_array_mut().unwrap();
        for edge in edges.iter_mut() {
            edge.as_object_mut().unwrap().remove("created_at_ms");
        }
        let node: UnifiedNode = serde_json::from_value(value).unwrap();
        assert_eq!(node.edges.len(), 1);
        assert_eq!(node.edges[0].created_at_ms, 0);
        TerminalReporter::success("node-level old-shape JSON honors the default.");
    });
}

/// Postcard (the real persistence format) round-trips the NEW shape intact.
#[test]
fn postcard_roundtrip_preserves_created_at_ms() {
    let dir = tempfile::tempdir().unwrap();
    let storage = StorageEngine::open(dir.path().to_str().unwrap()).unwrap();
    let relates = storage.intern_label("relates_to");

    let mut n1 = UnifiedNode::new(1);
    n1.edges.push(Edge::with_timestamp(2, relates, 12345));
    storage.insert(&n1).unwrap();

    let got = storage.get(1).unwrap().unwrap();
    assert_eq!(got.edges.len(), 1);
    assert_eq!(
        got.edges[0].created_at_ms, 12345,
        "timestamp survives postcard round-trip"
    );
}

/// (2) Traversal with a temporal window: edges inside the window are followed,
/// edges outside are not, `None` follows everything.
#[test]
fn temporal_window_filters_traversal() {
    let mut harness = VantaHarness::new("TEMPORAL EDGES (WINDOW FILTER)");
    let dir = tempfile::tempdir().unwrap();
    let storage = StorageEngine::open(dir.path().to_str().unwrap()).unwrap();
    let relates = storage.intern_label("relates_to");
    build_chain(&storage, relates);
    let traverser = GraphTraverser::new(&storage);

    harness.execute("Edge inside window is followed", || {
        // 1→2 (ts=100) in [50,150] → followed; 2→3 (ts=200) out → stopped at 2.
        let res = traverser
            .bfs_traverse_filtered(&[1], 10, TraversalDirection::Forward, &[], Some((50, 150)))
            .unwrap();
        assert!(res.contains(&1));
        assert!(res.contains(&2));
        assert!(!res.contains(&3), "edge 2→3 (ts=200) is outside [50,150]");
        TerminalReporter::success("in-window edge followed, out-of-window edge skipped.");
    });

    harness.execute("Edge outside window is not followed at all", || {
        let res = traverser
            .bfs_traverse_filtered(&[1], 10, TraversalDirection::Forward, &[], Some((250, 400)))
            .unwrap();
        assert!(res.contains(&1));
        assert!(!res.contains(&2), "edge 1→2 (ts=100) is outside [250,400]");
        TerminalReporter::success("window before edge creation blocks traversal.");
    });

    harness.execute(
        "None disables temporal filtering (follows everything)",
        || {
            let res = traverser
                .bfs_traverse_filtered(&[1], 10, TraversalDirection::Forward, &[], None)
                .unwrap();
            assert!(res.contains(&1) && res.contains(&2) && res.contains(&3));
            TerminalReporter::success("None window is behavior-identical to pre-temporal BFS.");
        },
    );

    harness.execute("DFS honors the same window", || {
        let res = traverser
            .dfs_traverse_filtered(&[1], 10, &[], TraversalDirection::Forward, Some((50, 150)))
            .unwrap();
        assert!(res.contains(&2));
        assert!(!res.contains(&3), "DFS must apply the same temporal filter");
        TerminalReporter::success("DFS window filtering matches BFS.");
    });
}

/// (3) SDK `add_edge` with an explicit timestamp persists it on BOTH the
/// forward and reverse edges — verified through temporal traversal in both
/// directions, using the exact window.
#[test]
fn sdk_add_edge_explicit_timestamp_persists_both_directions() {
    let mut harness = VantaHarness::new("TEMPORAL EDGES (SDK ADD_EDGE)");
    let dir = tempfile::tempdir().unwrap();
    let db = VantaEmbedded::open(dir.path()).unwrap();
    db.insert_node(VantaNodeInput::new(1)).unwrap();
    db.insert_node(VantaNodeInput::new(2)).unwrap();

    harness.execute("Forward edge carries the explicit timestamp", || {
        db.add_edge(1, 2, "rel", Some(1.0), Some(5000)).unwrap();
        let fwd = db
            .graph_bfs_filtered(
                &[1],
                10,
                TraversalDirection::Forward,
                &[],
                Some((5000, 5000)),
            )
            .unwrap();
        assert!(
            fwd.contains(&2),
            "forward edge with ts=5000 must be followed in [5000,5000]"
        );

        let fwd_out = db
            .graph_bfs_filtered(
                &[1],
                10,
                TraversalDirection::Forward,
                &[],
                Some((6000, 7000)),
            )
            .unwrap();
        assert!(!fwd_out.contains(&2), "edge ts=5000 is outside [6000,7000]");
        TerminalReporter::success("forward edge timestamp applied and persisted.");
    });

    harness.execute("Reverse edge carries the same explicit timestamp", || {
        let rev = db
            .graph_bfs_filtered(
                &[2],
                10,
                TraversalDirection::Reverse,
                &[],
                Some((5000, 5000)),
            )
            .unwrap();
        assert!(rev.contains(&1), "reverse edge must share the same ts=5000");
        TerminalReporter::success("reverse edge shares the logical creation time.");
    });
}

/// (4) SDK `add_edge` without a timestamp stamps `now()` (created_at_ms > 0).
#[test]
fn sdk_add_edge_without_timestamp_stamps_now() {
    let dir = tempfile::tempdir().unwrap();
    let db = VantaEmbedded::open(dir.path()).unwrap();
    db.insert_node(VantaNodeInput::new(1)).unwrap();
    db.insert_node(VantaNodeInput::new(2)).unwrap();

    db.add_edge(1, 2, "rel", Some(1.0), None).unwrap();

    // A window of (0,0) excludes any edge with ts > 0; (1, u64::MAX) includes it.
    let none_window = db
        .graph_bfs_filtered(&[1], 10, TraversalDirection::Forward, &[], Some((0, 0)))
        .unwrap();
    assert!(!none_window.contains(&2), "default timestamp must be > 0");

    let any_window = db
        .graph_bfs_filtered(
            &[1],
            10,
            TraversalDirection::Forward,
            &[],
            Some((1, u64::MAX)),
        )
        .unwrap();
    assert!(
        any_window.contains(&2),
        "default timestamp must be within (1, u64::MAX)"
    );
}

/// (5) Constructors stamp `now()`.
#[test]
fn edge_constructors_stamp_now() {
    let edge = Edge::new(9, 3);
    assert!(edge.created_at_ms > 0, "Edge::new must stamp now()");

    let rev = Edge::reverse(9, 3);
    assert!(rev.created_at_ms > 0, "Edge::reverse must stamp now()");
    assert!(rev.reverse);

    let ts = Edge::with_timestamp(9, 3, 42);
    assert_eq!(
        ts.created_at_ms, 42,
        "with_timestamp honors the explicit value"
    );
}
