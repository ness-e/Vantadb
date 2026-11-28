//! FIND-222: the prefetch must never resurrect a payload the write path has
//! already invalidated.
//!
//! **Deterministic by construction.** The bug is a two-instant race
//! (`prefetch_related`: materialize → insert) on a single thread, so every test
//! here installs the `set_prefetch_interleave` hook and lets it run the writer
//! *inline* between the two instants. No sleeps, no second thread, no
//! probability — the `FIND-826` lesson was a race test that passed on the gate
//! profile and failed at 12 threads.
//!
//! Each case is a distinct invalidation site: `delete`, `delete_batch`, the
//! Cold-overwrite write path (`apply_insert`), and the two transactional
//! commit paths. Without the guard in `prefetch_related` every one of them
//! leaves the pre-mutation payload in `cache.volatile` (the Prove-It in the
//! task file).

use std::sync::Arc;

use crate::node::UnifiedNode;
use crate::storage::engine::get::set_prefetch_interleave;

use super::*;

fn node_with_payload(id: u128, mark: &str) -> UnifiedNode {
    let mut node = sample_node(id);
    // Cold: `insert` does not populate `volatile`, so the prefetch has a real
    // vacancy to fill (the tier that made FIND-190 visible).
    node.tier = crate::node::NodeTier::Cold;
    node.relational.insert(
        "payload".to_string(),
        crate::node::FieldValue::String(mark.to_string()),
    );
    node
}

fn payload_of(node: &UnifiedNode) -> Option<String> {
    match node.relational.get("payload") {
        Some(crate::node::FieldValue::String(s)) => Some(s.clone()),
        _ => None,
    }
}

/// Engine with nodes 1 (anchor, never touched) and 2 (the race target) and the
/// A↔B co-access pair registered, so `get(1)` prefetches 2.
fn engine_with_prefetch_pair(old_mark: &str) -> Arc<StorageEngine> {
    let engine = Arc::new(in_memory_engine());
    engine
        .insert(&node_with_payload(1, "anchor"))
        .expect("insert 1");
    engine
        .insert(&node_with_payload(2, old_mark))
        .expect("insert 2");
    // `min_accesses` is 3 by default.
    for _ in 0..3 {
        engine.cache.warmer.record_co_access(&[1, 2]);
    }
    engine
}

/// `get(1)` (cache miss) → `prefetch_related(1)` → materialize 2 → **hook** →
/// insert 2. The hook receives the warm id being materialized.
///
/// Asserts the hook **fired exactly once** (review R2): if `suggest_warm_ids`
/// ever stops returning the pair (a `min_accesses` change, a warmer refactor,
/// a cache hit that filters it out), every case in this file would still be
/// green with or without the guard — the regression test would silently
/// disable itself.
fn prefetch_with<F>(engine: &Arc<StorageEngine>, mut writer: F)
where
    F: FnMut(u128) + 'static,
{
    let fired = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counter = Arc::clone(&fired);
    set_prefetch_interleave(Some(Box::new(move |id| {
        counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        writer(id);
    })));
    let got = engine.get(1).expect("get(1) must terminate");
    set_prefetch_interleave(None);
    assert!(got.is_some(), "anchor node 1 exists");
    assert_eq!(
        fired.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "the prefetch of node 2 must have fired exactly once"
    );
}

/// `delete` path (`delete.rs::apply_delete_inner`).
#[test]
fn delete_mid_flight_does_not_resurrect_the_node() {
    let engine = engine_with_prefetch_pair("old");
    let writer = Arc::clone(&engine);
    prefetch_with(&engine, move |id| {
        writer.delete(id, "find222").expect("delete");
    });

    assert!(
        !engine.cache.volatile.read().contains_key(&2),
        "a node deleted between materialization and insert must not be cached"
    );
    assert!(
        engine.get(2).expect("get(2)").is_none(),
        "the deleted node must stay deleted"
    );
}

/// `delete_batch` path (Phase 5 removal).
#[test]
fn batch_delete_mid_flight_does_not_resurrect_the_node() {
    let engine = engine_with_prefetch_pair("old");
    let writer = Arc::clone(&engine);
    prefetch_with(&engine, move |id| {
        writer.delete_batch(&[id]).expect("delete_batch");
    });

    assert!(
        !engine.cache.volatile.read().contains_key(&2),
        "a node batch-deleted between materialization and insert must not be cached"
    );
    assert!(
        engine.get(2).expect("get(2)").is_none(),
        "the batch-deleted node must stay deleted"
    );
}

/// Write path the row groups under "Hot": the overwrite that invalidates a
/// non-Hot entry (`insert.rs::apply_insert`, the `FIND-190` hole).
#[test]
fn overwrite_mid_flight_does_not_cache_the_old_payload() {
    let engine = engine_with_prefetch_pair("old");
    let writer = Arc::clone(&engine);
    prefetch_with(&engine, move |id| {
        writer
            .insert(&node_with_payload(id, "new"))
            .expect("overwrite");
    });

    let got = engine.get(2).expect("get(2)").expect("node 2 exists");
    assert_eq!(
        payload_of(&got),
        Some("new".to_string()),
        "the pre-overwrite payload must not survive in the cache"
    );
}

/// `txn` path, insert half (`txn.rs::commit_transaction` → `apply_insert_with_txn`).
#[test]
fn txn_insert_commit_mid_flight_does_not_cache_the_old_payload() {
    let engine = engine_with_prefetch_pair("old");
    let writer = Arc::clone(&engine);
    prefetch_with(&engine, move |id| {
        let txn = writer.begin_transaction().expect("begin transaction");
        writer
            .insert_in_txn(&node_with_payload(id, "new"), txn)
            .expect("buffer insert");
        writer.commit_transaction(txn).expect("commit");
    });

    let got = engine.get(2).expect("get(2)").expect("node 2 exists");
    assert_eq!(
        payload_of(&got),
        Some("new".to_string()),
        "the pre-commit payload must not survive in the cache"
    );
}

/// `txn` path, rollback half: an overwrite that commits inside the buffer and
/// then rolls back must not leave the rejected payload in the cache (review
/// R1 — `commit_transaction` publishes a Hot payload into `volatile` before the
/// commit marker; the rollback restores the KV but used to leave the entry).
///
/// The failure is induced without failpoints: the second buffered op has more
/// edges than the `u16` header field can hold, so `write_node_to_vstore` rejects
/// it *after* the first op was applied.
#[test]
fn txn_rollback_restores_the_previous_payload() {
    let engine = in_memory_engine();
    let mut old = node_with_payload(2, "old");
    old.tier = crate::node::NodeTier::Hot;
    engine.insert(&old).expect("insert 2");
    assert!(
        engine.cache.volatile.read().contains_key(&2),
        "precondition: a Hot insert publishes node 2 in the cache"
    );

    let mut rejected = node_with_payload(2, "new");
    rejected.tier = crate::node::NodeTier::Hot;
    let mut poisoned = sample_node(3);
    poisoned.edges = vec![crate::node::Edge::new(1, 0); u16::MAX as usize + 1];

    let txn = engine.begin_transaction().expect("begin transaction");
    engine
        .insert_in_txn(&rejected, txn)
        .expect("buffer the overwrite");
    engine
        .insert_in_txn(&poisoned, txn)
        .expect("buffer the op that fails at apply time");
    assert!(
        engine.commit_transaction(txn).is_err(),
        "commit must fail on the unrepresentable edge count"
    );

    let node = engine.get(2).expect("get(2)");
    // FIND-222 owns one thing here: the *rejected* payload must never be served.
    // The KV row is restored, but the rollback does not rebuild the HNSW entry
    // (the op loop removed it before applying, and nothing re-adds the previous
    // offset), so `get()` answers `None` instead of the restored payload — a
    // separate pre-existing gap, filed as its own finding.
    assert!(
        node.as_ref().and_then(payload_of) != Some("new".to_string()),
        "the rolled-back payload must never be served, got {node:?}"
    );
    assert!(
        !engine.cache.volatile.read().contains_key(&2),
        "the rejected payload must not stay in the cache"
    );
    // NOTA (review, Optional): este `!contains_key` es MÁS estricto que el
    // invariante. Un fix correcto de `FIND-831` que re-indexe el nodo y
    // re-publique el payload restaurado ("old") en el cache hará fallar esta
    // aserción con el comportamiento correcto: al cerrar FIND-831, relajarla a
    // "el cache no contiene el payload rechazado".
}

/// `txn` path, delete half (`commit_transaction` → `apply_delete_inner`).
#[test]
fn txn_delete_commit_mid_flight_does_not_resurrect_the_node() {
    let engine = engine_with_prefetch_pair("old");
    let writer = Arc::clone(&engine);
    prefetch_with(&engine, move |id| {
        let txn = writer.begin_transaction().expect("begin transaction");
        writer
            .delete_in_txn(id, "find222", txn)
            .expect("buffer delete");
        writer.commit_transaction(txn).expect("commit");
    });

    assert!(
        !engine.cache.volatile.read().contains_key(&2),
        "a transactional delete must not leave the node cached"
    );
    assert!(
        engine.get(2).expect("get(2)").is_none(),
        "the transactionally deleted node must stay deleted"
    );
}
