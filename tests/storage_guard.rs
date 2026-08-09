//! AUDREP-45 oversized-write guard restoration (ERR-005).
//!
//! The guard lives in `src/storage/ops.rs`: `deserialize_node_payload` rejects
//! persisted payloads larger than `MAX_PERSISTED_NODE_BYTES` (128 MiB) before
//! `postcard` can act on an untrusted length prefix — converting a corrupt or
//! oversized payload into a clean `VantaError` instead of a panic/OOM.
//!
//! The guard is `pub(crate)`, so these integration tests drive it through the
//! public engine API:
//!   - boundary payload (forces vstore growth) → insert/read round-trips OK
//!   - oversized persisted payload (> cap) → `get` returns VantaError, no panic

use std::collections::BTreeMap;

use tempfile::tempdir;
use vantadb::node::{FieldValue, NodeTier, UnifiedNode, VectorRepresentations};
use vantadb::storage::StorageEngine;

/// Hardcoded mirror of `MAX_PERSISTED_NODE_BYTES` (src/storage/ops.rs).
/// Kept as a plain literal: the const itself is `pub(crate)` and the goal is
/// to exercise the guard, not import its internals.
const PERSISTED_NODE_BYTE_CAP: usize = 128 * 1024 * 1024;

/// A vector large enough to force `write_node_to_vstore` to grow the vfile
/// past its initial size (the oversized-write path) yet comfortably within
/// every guard: 1M f32 = 4 MB.
const BOUNDARY_VEC_LEN: usize = 1_000_000;

/// Boundary payload → OK: inserting a node whose vector forces vstore growth,
/// then flushing and reopening, must round-trip the node exactly — no panic,
/// no corruption.
#[test]
fn oversized_write_guard_boundary_payload_roundtrips() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();

    let storage = StorageEngine::open(path).expect("open engine");

    let mut node = UnifiedNode::new(7);
    let mut vec = vec![0.5f32; BOUNDARY_VEC_LEN];
    vec[0] = 1.25;
    vec[BOUNDARY_VEC_LEN - 1] = -3.5;
    node.vector = VectorRepresentations::Full(vec);
    storage.insert(&node).expect("oversized write must succeed");
    storage.flush().expect("flush");

    // Drop + reopen so reads go through the persisted (backend + vstore) path.
    drop(storage);
    let storage = StorageEngine::open(path).expect("reopen engine");
    let got = storage
        .get(7)
        .expect("get must not panic")
        .expect("node must survive reopen");

    assert_eq!(got.id, 7);
    let dim = match got.vector {
        VectorRepresentations::Full(v) => Some(v.len()),
        VectorRepresentations::SQ8(data, _) => Some(data.len()),
        _ => None,
    };
    assert_eq!(
        dim,
        Some(BOUNDARY_VEC_LEN),
        "vector dimension lost across oversized write + reopen"
    );
}

/// Oversized payload → VantaError, no panic: a node whose persisted metadata
/// exceeds the AUDREP-45 byte cap must surface as an error on read, never a
/// process panic or OOM.
///
/// The node is Cold-tier so the volatile cache is skipped and `get` reads from
/// the backend, where the guard lives. The engine is not reopened: the insert
/// WAL-append carries the full payload and the WAL reader's scan-forward
/// auto-heal would walk the oversized record byte-by-byte (a separate
/// resilience finding, ERR-00X — see report).
#[test]
fn oversized_write_guard_oversized_payload_errors_not_panics() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();

    let storage = StorageEngine::open(path).expect("open engine");

    // A single relational string just past the cap: postcard serializes it as
    // length-prefixed bytes, so the persisted metadata payload exceeds
    // MAX_PERSISTED_NODE_BYTES exactly like a corrupt oversized length prefix.
    let mut node = UnifiedNode::new(9);
    let blob = "x".repeat(PERSISTED_NODE_BYTE_CAP + 1024);
    let mut relational = BTreeMap::new();
    relational.insert("blob".to_string(), FieldValue::ListString(vec![blob]));
    node.relational = relational;
    node.vector = VectorRepresentations::Full(vec![0.1, 0.2, 0.3]);
    node.tier = NodeTier::Cold;

    // The write path accepts the payload (the guard protects the read side,
    // where a hostile/corrupt length prefix would otherwise drive allocation).
    storage
        .insert(&node)
        .expect("insert of oversized payload must not panic");

    // Cold tier → no cache hit → backend read → AUDREP-45 cap guard fires.
    let err = storage
        .get(9)
        .expect_err("oversized payload must error, not panic");
    assert!(
        err.to_string().contains("exceeds"),
        "guard error should mention the byte cap, got: {err}"
    );
}
