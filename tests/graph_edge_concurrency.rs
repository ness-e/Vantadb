// ponytail: blanket allow — unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! FIND-773 — `add_edge` is a read-modify-write over the node's edge list.
//!
//! `Embedded::add_edge` (`src/sdk/api/graph.rs`) reads the whole node, pushes
//! the new edge and rewrites the node. Nothing serializes that sequence, so two
//! writers on the same node both read the same list and the last `insert`
//! wins: the other writer's edge disappears, with no error and no log line —
//! indistinguishable from an edge that was never created.
//!
//! Both tests fail on the unfixed code and pass once the read-modify-write is
//! serialized. `THREADS * EDGES_PER_THREAD` is large enough that the window
//! between `get` and `insert` is hit on every run (measured: 0/800 edges lost
//! is what the fix must produce; the unfixed code loses hundreds).

use std::sync::{Arc, Barrier};
use std::thread;

use vantadb::config::Config;
use vantadb::{BackendKind, Embedded, NodeInput};

const THREADS: usize = 8;
const EDGES_PER_THREAD: usize = 100;
const HUB: u128 = 1;

fn leaf_id(thread_idx: usize, edge_idx: usize) -> u128 {
    100 + (thread_idx * EDGES_PER_THREAD + edge_idx) as u128
}

fn open_in_memory() -> Embedded {
    Embedded::open_with_config(Config {
        storage_path: ":memory:".into(),
        backend_kind: BackendKind::InMemory,
        ..Default::default()
    })
    .expect("open in-memory")
}

/// The bug: N writers adding distinct edges to the SAME node. Every edge is a
/// separate call with its own target, so there is no legitimate reason for the
/// final count to be below `THREADS * EDGES_PER_THREAD`.
#[test]
fn concurrent_add_edge_loses_no_edges_on_one_node() {
    let db = open_in_memory();
    db.insert_node(NodeInput::new(HUB)).expect("insert hub");
    for t in 0..THREADS {
        for i in 0..EDGES_PER_THREAD {
            db.insert_node(NodeInput::new(leaf_id(t, i)))
                .expect("insert leaf");
        }
    }

    let barrier = Arc::new(Barrier::new(THREADS));
    let mut handles = Vec::new();
    for t in 0..THREADS {
        let db = db.clone();
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            barrier.wait();
            for i in 0..EDGES_PER_THREAD {
                db.add_edge(HUB, leaf_id(t, i), "rel", None, None)
                    .expect("add_edge");
            }
        }));
    }
    for handle in handles {
        handle.join().expect("writer thread panicked");
    }

    let hub = db.get_node(HUB).expect("get hub").expect("hub present");
    let expected = THREADS * EDGES_PER_THREAD;
    assert_eq!(
        hub.edges.len(),
        expected,
        "lost {} of {expected} edges on the shared node: the read-modify-write \
         of the edge list is not serialized (FIND-773)",
        expected - hub.edges.len()
    );
}

/// FIND-809: `remove_edge` is the *same* read-modify-write as `add_edge`, on the
/// path a reader actually notices — an edge that cannot be removed. Each thread
/// deletes only its own 100 edges from the shared node, so there is no legitimate
/// reason for any edge to survive: the final count must be **0**, and every leaf
/// must have lost its reverse half too.
#[test]
fn concurrent_remove_edge_removes_every_edge() {
    let db = open_in_memory();
    db.insert_node(NodeInput::new(HUB)).expect("insert hub");
    let expected = THREADS * EDGES_PER_THREAD;
    for t in 0..THREADS {
        for i in 0..EDGES_PER_THREAD {
            db.insert_node(NodeInput::new(leaf_id(t, i)))
                .expect("insert leaf");
            db.add_edge(HUB, leaf_id(t, i), "rel", None, None)
                .expect("add_edge");
        }
    }
    assert_eq!(
        db.get_node(HUB)
            .expect("get hub")
            .expect("hub present")
            .edges
            .len(),
        expected,
        "fixture must start with every edge present"
    );

    let barrier = Arc::new(Barrier::new(THREADS));
    let mut handles = Vec::new();
    for t in 0..THREADS {
        let db = db.clone();
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            barrier.wait();
            for i in 0..EDGES_PER_THREAD {
                db.remove_edge(HUB, leaf_id(t, i), "rel")
                    .expect("remove_edge");
            }
        }));
    }
    for handle in handles {
        handle.join().expect("remover thread panicked");
    }

    let hub = db.get_node(HUB).expect("get hub").expect("hub present");
    assert_eq!(
        hub.edges.len(),
        0,
        "{} of {expected} edges survived the removal: the read-modify-write of \
         remove_edge is not serialized (FIND-809)",
        hub.edges.len()
    );

    // The reverse half is the *other* read-modify-write of the same call. A fix
    // that only guarded the source half would leave every leaf pointing at a hub
    // that no longer points back.
    let mut bad_reverse = 0usize;
    for t in 0..THREADS {
        for i in 0..EDGES_PER_THREAD {
            let leaf = db
                .get_node(leaf_id(t, i))
                .expect("get_node")
                .expect("leaf present");
            if leaf.edges.iter().any(|e| e.target == HUB) {
                bad_reverse += 1;
            }
        }
    }
    assert_eq!(
        bad_reverse, 0,
        "leaf nodes whose reverse half outlived the removal: the target-half of \
         remove_edge is not covered by the serialization (FIND-809)"
    );
}

/// The two halves of one edge are two nodes. Two handles over the same engine
/// (`Embedded::from_engine`, the shape PyO3/WASM/MCP use) must not race either:
/// the fix has to share the serialization across wrappers, not only across
/// clones of one handle.
#[test]
fn add_edge_serializes_across_handles_over_one_engine() {
    let db = open_in_memory();
    let engine = db.engine_handle().expect("engine handle");
    db.insert_node(NodeInput::new(HUB)).expect("insert hub");
    for t in 0..THREADS {
        for i in 0..EDGES_PER_THREAD {
            db.insert_node(NodeInput::new(leaf_id(t, i)))
                .expect("insert leaf");
        }
    }

    let barrier = Arc::new(Barrier::new(THREADS));
    let mut handles = Vec::new();
    for t in 0..THREADS {
        let handle = Embedded::from_engine(engine.clone());
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            barrier.wait();
            for i in 0..EDGES_PER_THREAD {
                handle
                    .add_edge(HUB, leaf_id(t, i), "rel", None, None)
                    .expect("add_edge");
            }
        }));
    }
    for handle in handles {
        handle.join().expect("writer thread panicked");
    }

    let hub = db.get_node(HUB).expect("get hub").expect("hub present");
    let expected = THREADS * EDGES_PER_THREAD;
    assert_eq!(
        hub.edges.len(),
        expected,
        "two handles over one engine lost {} of {expected} edges: the \
         serialization must be engine-owned, not per-wrapper (FIND-773)",
        expected - hub.edges.len()
    );
    // Every leaf was written by exactly one thread, so its reverse half must be
    // there too — and the reverse half is the *same* read-modify-write on the
    // other node. Without this, a regression that dropped the target-half
    // rewrite (`target.edges.push` + `insert`) would still pass the count
    // above, because the hub's forward list would be intact.
    let mut bad_reverse = 0usize;
    for t in 0..THREADS {
        for i in 0..EDGES_PER_THREAD {
            let leaf = db
                .get_node(leaf_id(t, i))
                .expect("get_node")
                .expect("leaf present");
            let reverse = leaf
                .edges
                .iter()
                .filter(|e| e.target == HUB && e.reverse)
                .count();
            if reverse != 1 {
                bad_reverse += 1;
            }
        }
    }
    assert_eq!(
        bad_reverse, 0,
        "leaf nodes whose reverse half is missing or duplicated: the target-half \
         rewrite of add_edge is not covered by the serialization (FIND-773)"
    );
}
