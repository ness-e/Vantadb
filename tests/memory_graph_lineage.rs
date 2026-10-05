#![allow(clippy::expect_used, clippy::unwrap_used)]
//! MEMG-03 — memory ↔ graph bridge certification.
//!
//! Covers: edge preservation across record rewrites (`put`/`supersede` must
//! not wipe node-resident graph state), lineage edges created by the memory
//! ops (`superseded_by`, `derived_from`) with idempotent semantics, and the
//! lineage query ("who changed this source and why") answered via BFS +
//! provenance over the existing public API.

use tempfile::tempdir;
use vantadb::graph::TraversalDirection;
use vantadb::{ConfidenceClass, Embedded, MemoryInput};

fn open() -> (tempfile::TempDir, Embedded) {
    let dir = tempdir().expect("tempdir");
    let db = Embedded::open(dir.path()).expect("open");
    (dir, db)
}

/// MEMG-03 (plan risk #1): a plain re-put (payload update) must not wipe the
/// graph edges stored on the record's node. HEAD rebuilds the node from the
/// record (`memory_record_to_node_owned`) without copying `edges` → wipe.
#[test]
fn record_update_preserves_existing_edges() {
    let (_dir, db) = open();
    let a = db
        .put(MemoryInput::new("agent/main", "a", "alpha"))
        .expect("put a");
    let b = db
        .put(MemoryInput::new("agent/main", "b", "beta"))
        .expect("put b");
    db.add_edge(a.node_id, b.node_id, "related", None, None)
        .expect("add_edge");

    let before = db.get_node(a.node_id).expect("get_node").expect("node a");
    assert_eq!(before.edges.len(), 1, "edge must exist before the update");

    // Plain update: same key, new payload — no graph op involved.
    db.put(MemoryInput::new("agent/main", "a", "alpha v2"))
        .expect("update a");

    let after = db.get_node(a.node_id).expect("get_node").expect("node a");
    assert_eq!(
        after.edges.len(),
        1,
        "record update must not wipe the node's edges"
    );
    assert_eq!(after.edges[0].target, b.node_id);
    assert_eq!(after.edges[0].label, "related");
    assert!(!after.edges[0].reverse, "forward edge must survive");

    // The reverse half on the untouched node must also remain intact.
    let other = db.get_node(b.node_id).expect("get_node").expect("node b");
    assert_eq!(other.edges.len(), 1, "reverse edge must survive");
    assert!(other.edges[0].reverse);
}

/// MEMG-03: `supersede` rewrites the old record's node — the rewrite must
/// preserve the edges that were already on that node.
#[test]
fn supersede_preserves_existing_edges() {
    let (_dir, db) = open();
    let a = db
        .put(MemoryInput::new("agent/main", "a", "alpha"))
        .expect("put a");
    let c = db
        .put(MemoryInput::new("agent/main", "c", "gamma"))
        .expect("put c");
    db.add_edge(a.node_id, c.node_id, "related", None, None)
        .expect("add_edge");

    db.supersede("agent/main", "a", "c").expect("supersede");

    let after = db.get_node(a.node_id).expect("get_node").expect("node a");
    assert!(
        after
            .edges
            .iter()
            .any(|e| e.target == c.node_id && e.label == "related"),
        "pre-existing edges must survive the supersede rewrite: {:?}",
        after.edges
    );
}

/// MEMG-03: `supersede` must create the canonical lineage edge
/// old --`superseded_by`--> new (bidirectional), navigable via BFS.
#[test]
fn supersede_creates_superseded_by_edge() {
    let (_dir, db) = open();
    let a = db
        .put(MemoryInput::new("agent/main", "a", "alpha"))
        .expect("put a");
    let c = db
        .put(MemoryInput::new("agent/main", "c", "gamma"))
        .expect("put c");
    db.supersede("agent/main", "a", "c").expect("supersede");

    let old = db.get_node(a.node_id).expect("get_node").expect("node a");
    let edge = old
        .edges
        .iter()
        .find(|e| e.label == "superseded_by" && e.target == c.node_id)
        .expect("superseded_by edge old->new must exist");
    assert!(!edge.reverse, "forward half lives on the old record");
    assert!(edge.created_at_ms > 0, "provenance timestamp recorded");

    let new = db.get_node(c.node_id).expect("get_node").expect("node c");
    assert!(
        new.edges
            .iter()
            .any(|e| e.label == "superseded_by" && e.target == a.node_id && e.reverse),
        "reverse half must live on the new record"
    );

    // Navigable: BFS from the old record reaches the superseder.
    let reached = db
        .graph_bfs(&[a.node_id], 1, TraversalDirection::Both)
        .expect("graph_bfs");
    assert!(reached.contains(&c.node_id));
}

/// MEMG-03: a derived `put` must create the canonical lineage edge
/// child --`derived_from`--> parent (bidirectional).
#[test]
fn derived_put_creates_derived_from_edge() {
    let (_dir, db) = open();
    let parent = db
        .put(MemoryInput::new("agent/main", "p", "parent fact"))
        .expect("put parent");
    let mut input = MemoryInput::new("agent/main", "child", "derived conclusion");
    input.confidence_class = Some(ConfidenceClass::Derived);
    input.derived_from = Some(vec!["p".to_string()]);
    let child = db.put(input).expect("derived put");

    let node = db
        .get_node(child.node_id)
        .expect("get_node")
        .expect("child");
    let edge = node
        .edges
        .iter()
        .find(|e| e.label == "derived_from" && e.target == parent.node_id)
        .expect("derived_from edge child->parent must exist");
    assert!(!edge.reverse, "forward half lives on the child");
    assert!(edge.created_at_ms > 0);

    let pnode = db
        .get_node(parent.node_id)
        .expect("get_node")
        .expect("parent");
    assert!(
        pnode
            .edges
            .iter()
            .any(|e| e.label == "derived_from" && e.target == child.node_id && e.reverse),
        "reverse half must live on the parent"
    );
}

/// MEMG-03: re-putting the same derived record must not duplicate its lineage
/// edge (`ensure_edge` is idempotent — re-idempotencia por op).
#[test]
fn repeated_derived_put_does_not_duplicate_edges() {
    let (_dir, db) = open();
    let parent = db
        .put(MemoryInput::new("agent/main", "p", "parent fact"))
        .expect("put parent");
    let derived = || {
        let mut input = MemoryInput::new("agent/main", "child", "derived conclusion");
        input.confidence_class = Some(ConfidenceClass::Derived);
        input.derived_from = Some(vec!["p".to_string()]);
        input
    };
    let child = db.put(derived()).expect("derived put");
    db.put(derived()).expect("derived re-put");

    let node = db
        .get_node(child.node_id)
        .expect("get_node")
        .expect("child");
    let count = node
        .edges
        .iter()
        .filter(|e| e.label == "derived_from" && e.target == parent.node_id)
        .count();
    assert_eq!(count, 1, "re-put must not duplicate the lineage edge");

    let pnode = db
        .get_node(parent.node_id)
        .expect("get_node")
        .expect("parent");
    let reverse_count = pnode
        .edges
        .iter()
        .filter(|e| e.label == "derived_from" && e.target == child.node_id)
        .count();
    assert_eq!(reverse_count, 1, "reverse half must not duplicate either");
}

/// MEMG-03: changing the declared parents drops the stale lineage edge — the
/// field is canonical data, the edge is derived navigability (pre-mortem 2).
#[test]
fn changing_derived_parents_drops_stale_edge() {
    let (_dir, db) = open();
    let p1 = db
        .put(MemoryInput::new("agent/main", "p1", "first source"))
        .expect("put p1");
    let p2 = db
        .put(MemoryInput::new("agent/main", "p2", "second source"))
        .expect("put p2");
    let derived_from = |parent: &str| {
        let mut input = MemoryInput::new("agent/main", "child", "derived conclusion");
        input.confidence_class = Some(ConfidenceClass::Derived);
        input.derived_from = Some(vec![parent.to_string()]);
        input
    };
    let child = db.put(derived_from("p1")).expect("derived put p1");
    db.put(derived_from("p2")).expect("derived re-put p2");

    let node = db
        .get_node(child.node_id)
        .expect("get_node")
        .expect("child");
    assert!(
        !node
            .edges
            .iter()
            .any(|e| e.label == "derived_from" && e.target == p1.node_id),
        "stale derived_from edge to p1 must be gone: {:?}",
        node.edges
    );
    assert!(
        node.edges
            .iter()
            .any(|e| e.label == "derived_from" && e.target == p2.node_id),
        "edge to the new parent p2 must exist"
    );

    let p1node = db.get_node(p1.node_id).expect("get_node").expect("p1");
    assert!(
        !p1node
            .edges
            .iter()
            .any(|e| e.label == "derived_from" && e.target == child.node_id),
        "reverse half on p1 must be cleaned up"
    );
    let p2node = db.get_node(p2.node_id).expect("get_node").expect("p2");
    assert!(
        p2node
            .edges
            .iter()
            .any(|e| e.label == "derived_from" && e.target == child.node_id && e.reverse),
        "reverse half on p2 must exist"
    );
}

/// MEMG-03 contract (b): "who changed the source of this decision and why?"
/// is answered via BFS + provenance over the existing public API.
#[test]
fn lineage_query_answers_who_changed_source_and_why() {
    let (_dir, db) = open();
    let a = db
        .put(MemoryInput::new("agent/main", "decision", "use provider X"))
        .expect("put decision");
    let c = db
        .put(MemoryInput::new(
            "agent/main",
            "decision-v2",
            "use provider Y because X was deprecated",
        ))
        .expect("put decision-v2");
    db.supersede("agent/main", "decision", "decision-v2")
        .expect("supersede");

    let mut input = MemoryInput::new("agent/main", "impl-note", "migrated to Y");
    input.confidence_class = Some(ConfidenceClass::Derived);
    input.derived_from = Some(vec!["decision-v2".to_string()]);
    let b = db.put(input).expect("derived put");

    // BFS from the original decision reaches the superseder and its derivation.
    let reached = db
        .graph_bfs(&[a.node_id], 2, TraversalDirection::Both)
        .expect("graph_bfs");
    assert!(reached.contains(&c.node_id), "superseder must be reachable");
    assert!(
        reached.contains(&b.node_id),
        "record derived from the new source must be reachable"
    );

    // Provenance: canonical edge label + timestamp on the old node.
    let old = db.get_node(a.node_id).expect("get_node").expect("node a");
    let lineage = old
        .edges
        .iter()
        .find(|e| e.label == "superseded_by" && e.target == c.node_id)
        .expect("lineage edge must carry provenance");
    assert!(lineage.created_at_ms > 0);

    // The "why": the superseder's payload, reachable from the BFS hit.
    let why = db
        .get("agent/main", "decision-v2")
        .expect("get")
        .expect("record");
    assert_eq!(why.payload, "use provider Y because X was deprecated");
}

/// MEMG-03: the batch rewrite path (`put_batch`) must preserve existing edges,
/// create declared `derived_from` edges, and not duplicate them on re-put.
#[test]
fn put_batch_derived_creates_edges_and_preserves_existing() {
    let (_dir, db) = open();
    let parent = db
        .put(MemoryInput::new("agent/main", "p", "parent fact"))
        .expect("put parent");

    let derived = |payload: &str| {
        let mut input = MemoryInput::new("agent/main", "child", payload);
        input.confidence_class = Some(ConfidenceClass::Derived);
        input.derived_from = Some(vec!["p".to_string()]);
        input
    };
    let batch = db
        .put_batch(vec![derived("derived conclusion")])
        .expect("derived batch put");
    let child = &batch[0];

    let node = db
        .get_node(child.node_id)
        .expect("get_node")
        .expect("child");
    assert!(
        node.edges
            .iter()
            .any(|e| e.label == "derived_from" && e.target == parent.node_id),
        "batch derived put must create the lineage edge"
    );

    // A user edge, then a batch UPDATE — both halves must survive and the
    // derived_from edge must not duplicate.
    db.add_edge(child.node_id, parent.node_id, "related", None, None)
        .expect("add_edge");
    db.put_batch(vec![derived("derived conclusion v2")])
        .expect("batch update");

    let node = db
        .get_node(child.node_id)
        .expect("get_node")
        .expect("child");
    assert!(
        node.edges
            .iter()
            .any(|e| e.label == "related" && e.target == parent.node_id),
        "batch update must preserve user edges: {:?}",
        node.edges
    );
    let derived_count = node
        .edges
        .iter()
        .filter(|e| e.label == "derived_from" && e.target == parent.node_id)
        .count();
    assert_eq!(derived_count, 1, "batch re-put must not duplicate the edge");
}

/// MEMG-03: state-only rewrites (`reinforce`) share the record→node
/// projection — they must preserve node-resident edges too.
#[test]
fn reinforce_preserves_existing_edges() {
    let (_dir, db) = open();
    let a = db
        .put(MemoryInput::new("agent/main", "a", "alpha"))
        .expect("put a");
    let b = db
        .put(MemoryInput::new("agent/main", "b", "beta"))
        .expect("put b");
    db.add_edge(a.node_id, b.node_id, "related", None, None)
        .expect("add_edge");

    db.reinforce("agent/main", "a", vantadb::ReinforceOutcome::Used)
        .expect("reinforce");

    let after = db.get_node(a.node_id).expect("get_node").expect("node a");
    assert!(
        after
            .edges
            .iter()
            .any(|e| e.label == "related" && e.target == b.node_id),
        "reinforce must not wipe node-resident edges: {:?}",
        after.edges
    );
}

/// MEMG-03 (review P2-01 H1): re-putting a superseded record revives it
/// (`superseded_by` resets to None) — the stale lineage edge must be dropped,
/// and a later supersede must leave exactly one edge.
#[test]
fn reput_of_superseded_record_drops_stale_superseded_by_edge() {
    let (_dir, db) = open();
    let a = db
        .put(MemoryInput::new("agent/main", "a", "alpha"))
        .expect("put a");
    let c = db
        .put(MemoryInput::new("agent/main", "c", "gamma"))
        .expect("put c");
    db.supersede("agent/main", "a", "c").expect("supersede");
    let c2 = db
        .put(MemoryInput::new("agent/main", "c2", "gamma 2"))
        .expect("put c2");

    // Re-put revives the record: field None ⇒ no superseded_by edge.
    db.put(MemoryInput::new("agent/main", "a", "alpha v2"))
        .expect("re-put a");
    let node = db.get_node(a.node_id).expect("get_node").expect("node a");
    assert!(
        !node.edges.iter().any(|e| e.label == "superseded_by"),
        "stale superseded_by edge must be gone: {:?}",
        node.edges
    );
    let cnode = db.get_node(c.node_id).expect("get_node").expect("node c");
    assert!(
        !cnode.edges.iter().any(|e| e.label == "superseded_by"),
        "stale reverse half must be gone: {:?}",
        cnode.edges
    );

    // A new supersede leaves exactly one lineage edge.
    db.supersede("agent/main", "a", "c2").expect("supersede c2");
    let node = db.get_node(a.node_id).expect("get_node").expect("node a");
    let edges: Vec<_> = node
        .edges
        .iter()
        .filter(|e| e.label == "superseded_by")
        .collect();
    assert_eq!(edges.len(), 1, "exactly one superseded_by edge: {edges:?}");
    assert_eq!(edges[0].target, c2.node_id);
}

/// MEMG-03 (review P2-01 M1): the bulk-import overwrite path must preserve
/// existing edges too (the raw transport bypasses the put path).
#[test]
fn bulk_import_overwrite_preserves_existing_edges() {
    let (_dir, db) = open();
    let a = db
        .put(MemoryInput::new("agent/main", "a", "alpha"))
        .expect("put a");
    let b = db
        .put(MemoryInput::new("agent/main", "b", "beta"))
        .expect("put b");
    db.add_edge(a.node_id, b.node_id, "related", None, None)
        .expect("add_edge");

    let mut body = Vec::new();
    body.extend_from_slice(b"VDBJSON\n");
    body.push(0x01);
    body.extend_from_slice(&1u64.to_le_bytes());
    let records = vec![MemoryInput::new("agent/main", "a", "alpha imported")];
    body.extend_from_slice(&serde_json::to_vec(&records).expect("json"));
    db.bulk_import_stream(&mut std::io::Cursor::new(body))
        .expect("bulk import");

    let node = db.get_node(a.node_id).expect("get_node").expect("node a");
    assert!(
        node.edges
            .iter()
            .any(|e| e.label == "related" && e.target == b.node_id),
        "bulk import must preserve edges: {:?}",
        node.edges
    );
    assert_eq!(
        db.get("agent/main", "a")
            .expect("get")
            .expect("record")
            .payload,
        "alpha imported"
    );
}

/// MEMG-03 (review P2-01 F1): the bulk path declares no lineage fields, so a
/// bulk overwrite of a superseded key must drop the stale lineage edge (and
/// its reverse half) — field = canonical, edge = derived navigability.
#[test]
fn bulk_import_overwrite_drops_stale_lineage_edges() {
    let (_dir, db) = open();
    let a = db
        .put(MemoryInput::new("agent/main", "a", "alpha"))
        .expect("put a");
    let c = db
        .put(MemoryInput::new("agent/main", "c", "gamma"))
        .expect("put c");
    db.supersede("agent/main", "a", "c").expect("supersede");

    let mut body = Vec::new();
    body.extend_from_slice(b"VDBJSON\n");
    body.push(0x01);
    body.extend_from_slice(&1u64.to_le_bytes());
    let records = vec![MemoryInput::new("agent/main", "a", "alpha imported")];
    body.extend_from_slice(&serde_json::to_vec(&records).expect("json"));
    db.bulk_import_stream(&mut std::io::Cursor::new(body))
        .expect("bulk import");

    // The imported record carries no supersession (field None)…
    let record = db.get("agent/main", "a").expect("get").expect("record");
    assert_eq!(record.superseded_by, None);
    // …so no forward lineage edge may survive on the imported node.
    let node = db.get_node(a.node_id).expect("get_node").expect("node a");
    assert!(
        !node.edges.iter().any(|e| e.label == "superseded_by"),
        "stale superseded_by edge must be dropped: {:?}",
        node.edges
    );
    // …and the reverse half on the counterpart must be cleaned up too.
    let cnode = db.get_node(c.node_id).expect("get_node").expect("node c");
    assert!(
        !cnode.edges.iter().any(|e| e.label == "superseded_by"),
        "stale reverse half must be cleaned up: {:?}",
        cnode.edges
    );
}
