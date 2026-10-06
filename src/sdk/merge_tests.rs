#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Unit tests for the multi-writer merge policy (MEMG-05).
//!
//! Invariants guarded (see ADR-0055):
//! - the decision depends only on `(updated_at_ms, content bytes)` — never on
//!   arrival order;
//! - equal-time + different content is a *detected conflict* and both
//!   acceptance orders declare the same winner;
//! - identical content with an older/equal clock (even with different
//!   bookkeeping) is a no-op; with a newer clock it advances the order key.

use super::{content_bytes, resolve_merge, MergeDecision};
use crate::sdk::types::MemoryRecord;

fn record(payload: &str, updated_at_ms: u64) -> MemoryRecord {
    MemoryRecord {
        namespace: "ns".into(),
        key: "key".into(),
        payload: payload.into(),
        updated_at_ms,
        version: 1,
        ..Default::default()
    }
}

#[test]
fn resolve_merge_without_existing_stores_incoming() {
    let incoming = record("alpha", 100);
    let decision = resolve_merge(None, &incoming).expect("resolve");
    assert_eq!(decision, MergeDecision::Store { conflict: false });
}

#[test]
fn resolve_merge_newer_incoming_wins_without_conflict() {
    let existing = record("alpha", 100);
    let incoming = record("beta", 200);
    let decision = resolve_merge(Some(&existing), &incoming).expect("resolve");
    assert_eq!(decision, MergeDecision::Store { conflict: false });
}

#[test]
fn resolve_merge_older_incoming_is_rejected_without_conflict() {
    let existing = record("alpha", 200);
    let incoming = record("beta", 100);
    let decision = resolve_merge(Some(&existing), &incoming).expect("resolve");
    assert_eq!(decision, MergeDecision::KeepExisting { conflict: false });
}

#[test]
fn resolve_merge_identical_content_with_newer_clock_advances_the_order_key() {
    // R1 (review P2-01): identical content must still advance the stored
    // `updated_at_ms` to the max of the write set — otherwise arrival order
    // changes the clock and cascades into different winners for later writes.
    let existing = record("alpha", 100);
    let newer_same = record("alpha", 500);
    assert_eq!(
        resolve_merge(Some(&existing), &newer_same).expect("resolve newer"),
        MergeDecision::Store { conflict: false }
    );
}

#[test]
fn resolve_merge_identical_content_with_older_or_equal_clock_is_unchanged() {
    let existing = record("alpha", 100);
    let older_same = record("alpha", 10);
    assert_eq!(
        resolve_merge(Some(&existing), &older_same).expect("resolve older"),
        MergeDecision::Unchanged
    );
    let equal_same = record("alpha", 100);
    assert_eq!(
        resolve_merge(Some(&existing), &equal_same).expect("resolve equal"),
        MergeDecision::Unchanged
    );
}

#[test]
fn resolve_merge_metadata_change_counts_as_content_change() {
    let existing = record("alpha", 100);
    let mut incoming = record("alpha", 200);
    incoming
        .metadata
        .insert("source".into(), crate::sdk::Value::String("agent-b".into()));
    let decision = resolve_merge(Some(&existing), &incoming).expect("resolve");
    assert_eq!(decision, MergeDecision::Store { conflict: false });
}

#[test]
fn resolve_merge_equal_time_different_content_is_a_detected_conflict() {
    let existing = record("alpha", 100);
    let incoming = record("beta", 100);
    let decision = resolve_merge(Some(&existing), &incoming).expect("resolve");
    assert!(
        matches!(
            decision,
            MergeDecision::Store { conflict: true }
                | MergeDecision::KeepExisting { conflict: true }
        ),
        "equal-time differing content must be flagged as a conflict, got {decision:?}"
    );
}

#[test]
fn resolve_merge_equal_time_declares_the_same_winner_in_both_arrival_orders() {
    let a = record("alpha", 100);
    let b = record("beta", 100);
    let a_then_b = resolve_merge(Some(&a), &b).expect("resolve a->b");
    let b_then_a = resolve_merge(Some(&b), &a).expect("resolve b->a");

    // Exactly one order stores the incoming record, and both orders agree on
    // which record wins (the same winner is "Store" in one order and
    // "KeepExisting" in the mirrored one).
    let consistent = match (a_then_b, b_then_a) {
        (
            MergeDecision::Store { conflict: true },
            MergeDecision::KeepExisting { conflict: true },
        ) => true,
        (
            MergeDecision::KeepExisting { conflict: true },
            MergeDecision::Store { conflict: true },
        ) => true,
        _ => false,
    };
    assert!(
        consistent,
        "both arrival orders must declare the same winner: a->b={a_then_b:?} b->a={b_then_a:?}"
    );
}

#[test]
fn resolve_merge_equal_time_stores_the_lexicographically_greater_content() {
    let a = record("alpha", 100);
    let b = record("beta", 100);
    let a_bytes = content_bytes(&a).expect("bytes a");
    let b_bytes = content_bytes(&b).expect("bytes b");

    // Feed the incoming side as whichever record has the greater canonical
    // bytes: the policy must keep it. Mirror: the lesser one must lose.
    let (winner, loser) = if a_bytes > b_bytes {
        (&a, &b)
    } else {
        (&b, &a)
    };
    assert_eq!(
        resolve_merge(Some(loser), winner).expect("resolve winner"),
        MergeDecision::Store { conflict: true }
    );
    assert_eq!(
        resolve_merge(Some(winner), loser).expect("resolve loser"),
        MergeDecision::KeepExisting { conflict: true }
    );
}

#[test]
fn content_bytes_ignores_bookkeeping_fields() {
    let base = record("alpha", 100);
    let mut bookkeeping_changed = record("alpha", 999);
    bookkeeping_changed.version = 42;
    bookkeeping_changed.created_at_ms = 7;
    assert_eq!(
        content_bytes(&base).expect("base"),
        content_bytes(&bookkeeping_changed).expect("changed"),
        "version/created_at_ms are store-local bookkeeping, not content"
    );
}
