#![allow(clippy::expect_used, clippy::unwrap_used)]
//! MEMG-05: multi-writer merge certification — deterministic convergence and
//! no silent loss (ADR-0055, declared LWW + conflict detection).
//!
//! The scenario fixed in DISCOVERY: several writers (devices/agents) produce
//! versions of the same `(namespace, key)` record and their write set reaches
//! one store through the merge path. For every write set, the final record must
//! be identical for ANY arrival order (sequential, permuted, or parallel) and
//! every write must receive a non-silent outcome.

use tempfile::tempdir;
use vantadb::{Embedded, MemoryInput, MemoryRecord, MergeOutcome, MergeResult};

const NS: &str = "multi/writer";
const KEY: &str = "shared";

fn record(payload: &str, updated_at_ms: u64) -> MemoryRecord {
    MemoryRecord {
        namespace: NS.into(),
        key: KEY.into(),
        payload: payload.into(),
        updated_at_ms,
        version: 1,
        ..Default::default()
    }
}

/// Applies `writes` in the given order on a fresh store; returns the final
/// `(payload, updated_at_ms)` plus the last merge result.
fn merge_sequence(writes: &[(&str, u64)]) -> (String, u64, MergeResult) {
    let dir = tempdir().expect("tempdir");
    let db = Embedded::open(dir.path()).expect("open");
    let mut last = None;
    for (payload, ts) in writes {
        last = Some(db.merge_record(record(payload, *ts)).expect("merge"));
    }
    let stored = db.get(NS, KEY).expect("get").expect("present");
    (
        stored.payload.clone(),
        stored.updated_at_ms,
        last.expect("at least one write"),
    )
}

#[test]
fn merge_record_inserts_when_absent() {
    let dir = tempdir().expect("tempdir");
    let db = Embedded::open(dir.path()).expect("open");

    let result = db.merge_record(record("alpha", 100)).expect("merge");
    assert_eq!(result.outcome, MergeOutcome::Inserted);
    assert!(!result.conflict);
    assert_eq!(result.winner_updated_at_ms, 100);

    let stored = db.get(NS, KEY).expect("get").expect("present");
    assert_eq!(stored.payload, "alpha");
    assert_eq!(stored.updated_at_ms, 100);
}

#[test]
fn merge_record_newer_incoming_wins() {
    let dir = tempdir().expect("tempdir");
    let db = Embedded::open(dir.path()).expect("open");

    db.merge_record(record("alpha", 100)).expect("seed");
    let result = db.merge_record(record("beta", 200)).expect("merge");

    assert_eq!(result.outcome, MergeOutcome::Updated);
    assert!(!result.conflict);
    assert_eq!(result.winner_updated_at_ms, 200);
    let stored = db.get(NS, KEY).expect("get").expect("present");
    assert_eq!(stored.payload, "beta");
    assert_eq!(stored.updated_at_ms, 200);
}

#[test]
fn merge_record_stale_incoming_is_rejected_not_silent() {
    let dir = tempdir().expect("tempdir");
    let db = Embedded::open(dir.path()).expect("open");

    db.merge_record(record("alpha", 200)).expect("seed");
    let result = db.merge_record(record("beta", 100)).expect("merge");

    // The stale write is explicitly rejected (out-of-order delivery detected),
    // and is NOT a same-time conflict — distinct timestamps are declared LWW.
    assert_eq!(result.outcome, MergeOutcome::StaleRejected);
    assert!(!result.conflict);
    assert_eq!(result.winner_updated_at_ms, 200);

    let stored = db.get(NS, KEY).expect("get").expect("present");
    assert_eq!(stored.payload, "alpha", "stale merge must not overwrite");
    assert_eq!(stored.updated_at_ms, 200);
}

#[test]
fn merge_record_identical_content_is_idempotent() {
    let dir = tempdir().expect("tempdir");
    let db = Embedded::open(dir.path()).expect("open");

    let seed = db.merge_record(record("alpha", 100)).expect("seed");
    let again = db.merge_record(record("alpha", 100)).expect("merge");

    assert_eq!(again.outcome, MergeOutcome::AlreadyCurrent);
    assert!(!again.conflict);

    let stored = db.get(NS, KEY).expect("get").expect("present");
    assert_eq!(stored.updated_at_ms, seed.winner_updated_at_ms);
    assert_eq!(stored.version, 1, "no write means no version bump");
}

#[test]
fn merge_record_equal_time_conflict_converges_across_arrival_orders() {
    // Same write set, opposite arrival orders, fresh stores: the final record
    // must be identical and the collision must be flagged in both directions.
    let (payload_ab, ts_ab, last_ab) = merge_sequence(&[("alpha", 100), ("beta", 100)]);
    let (payload_ba, ts_ba, last_ba) = merge_sequence(&[("beta", 100), ("alpha", 100)]);

    assert_eq!(
        (payload_ab.as_str(), ts_ab),
        (payload_ba.as_str(), ts_ba),
        "equal-time tie must converge to the same winner regardless of arrival order"
    );
    assert!(last_ab.conflict, "collision must be detected (a -> b)");
    assert!(last_ba.conflict, "collision must be detected (b -> a)");
    assert_eq!(last_ab.winner_updated_at_ms, 100);
    assert_eq!(last_ba.winner_updated_at_ms, 100);
}

#[test]
fn merge_record_sequential_permutations_share_final_state() {
    let forward = merge_sequence(&[("one", 100), ("two", 200), ("three", 300)]);
    let reverse = merge_sequence(&[("three", 300), ("two", 200), ("one", 100)]);
    let shuffled = merge_sequence(&[("two", 200), ("one", 100), ("three", 300)]);

    assert_eq!((forward.0.as_str(), forward.1), ("three", 300));
    assert_eq!(
        (forward.0.as_str(), forward.1),
        (reverse.0.as_str(), reverse.1),
        "reverse arrival order must reach the same final record"
    );
    assert_eq!(
        (forward.0.as_str(), forward.1),
        (shuffled.0.as_str(), shuffled.1),
        "shuffled arrival order must reach the same final record"
    );
}

#[test]
fn merge_record_parallel_writers_converge_to_declared_winner() {
    // 8 writers, distinct timestamps, threads racing on the same key. The
    // declared winner is the max `updated_at_ms`; spawning order is varied per
    // round to prove the result is scheduling-independent.
    let mut finals = Vec::new();
    for round in 0..4usize {
        let dir = tempdir().expect("tempdir");
        let db = Embedded::open(dir.path()).expect("open");

        let writers: Vec<(String, u64)> = (0..8u64)
            .map(|i| (format!("writer-{i}"), 100 + i * 10))
            .collect();

        // Rotate the spawn order per round: even rounds ascending, odd rounds
        // descending — the final record must not care.
        let order: Vec<&(String, u64)> = if round % 2 == 1 {
            writers.iter().rev().collect()
        } else {
            writers.iter().collect()
        };
        let db_ref = &db;
        let mut results: Vec<MergeResult> = Vec::with_capacity(writers.len());
        std::thread::scope(|scope| {
            let handles: Vec<_> = order
                .iter()
                .map(|&(payload, ts)| {
                    let payload = payload.clone();
                    let ts = *ts;
                    scope.spawn(move || db_ref.merge_record(record(&payload, ts)).expect("merge"))
                })
                .collect();
            for handle in handles {
                results.push(handle.join().expect("join"));
            }
        });

        assert_eq!(results.len(), writers.len());
        let inserted = results
            .iter()
            .filter(|r| r.outcome == MergeOutcome::Inserted)
            .count();
        assert_eq!(inserted, 1, "exactly one merge inserts the record");

        let stored = db.get(NS, KEY).expect("get").expect("present");
        assert_eq!(stored.payload, "writer-7", "round {round}");
        assert_eq!(stored.updated_at_ms, 170, "round {round}");
        assert_eq!(
            results
                .iter()
                .map(|r| r.winner_updated_at_ms)
                .max()
                .expect("results"),
            170
        );
        finals.push(stored.payload.clone());
    }
    assert!(
        finals.iter().all(|p| p == "writer-7"),
        "all rounds must converge: {finals:?}"
    );
}

#[test]
fn merge_record_parallel_equal_time_writers_converge() {
    // All writers share the same logical time: every version after the first
    // is a detected conflict and the tie-break must converge across rounds.
    let mut finals: Vec<(String, u64)> = Vec::new();
    for round in 0..3usize {
        let dir = tempdir().expect("tempdir");
        let db = Embedded::open(dir.path()).expect("open");

        let writers: Vec<String> = (0..6u64).map(|i| format!("w{i}")).collect();
        // Rotate the spawn order per round (ascending / descending).
        let order: Vec<&String> = if round % 2 == 1 {
            writers.iter().rev().collect()
        } else {
            writers.iter().collect()
        };
        let db_ref = &db;
        let mut results: Vec<MergeResult> = Vec::with_capacity(writers.len());
        std::thread::scope(|scope| {
            let handles: Vec<_> = order
                .iter()
                .map(|payload| {
                    let payload = (*payload).clone();
                    scope.spawn(move || db_ref.merge_record(record(&payload, 100)).expect("merge"))
                })
                .collect();
            for handle in handles {
                results.push(handle.join().expect("join"));
            }
        });

        let inserted = results
            .iter()
            .filter(|r| r.outcome == MergeOutcome::Inserted)
            .count();
        let conflicts = results.iter().filter(|r| r.conflict).count();
        assert_eq!(inserted, 1, "exactly one merge inserts the record");
        assert_eq!(
            conflicts,
            writers.len() - 1,
            "every later same-time version must be flagged as a conflict"
        );

        let stored = db.get(NS, KEY).expect("get").expect("present");
        assert_eq!(stored.updated_at_ms, 100);
        finals.push((stored.payload.clone(), stored.updated_at_ms));
    }
    assert!(
        finals.windows(2).all(|pair| pair[0] == pair[1]),
        "equal-time tie must converge across rounds: {finals:?}"
    );
}

#[test]
fn merge_record_identical_content_advances_the_clock_convergently() {
    // Review P2-01 R1: the same write set {(same,100),(same,500)} must end
    // with the SAME stored clock in both arrival orders (max, not first-seen).
    let forward = merge_sequence(&[("same", 100), ("same", 500)]);
    let reverse = merge_sequence(&[("same", 500), ("same", 100)]);
    assert_eq!((forward.0.as_str(), forward.1), ("same", 500));
    assert_eq!(
        (forward.0.as_str(), forward.1),
        (reverse.0.as_str(), reverse.1),
        "identical content must converge to the max clock, not the first-arrival one"
    );
}

#[test]
fn merge_record_identical_content_does_not_flip_later_winners() {
    // Review P2-01 R1 cascade: with a same-content rewrite in the write set,
    // the declared winner of a later different-content write must not depend
    // on arrival order.
    let write_sets: [&[(&str, u64)]; 3] = [
        &[("c", 100), ("c", 500), ("c2", 300)],
        &[("c", 500), ("c", 100), ("c2", 300)],
        &[("c2", 300), ("c", 100), ("c", 500)],
    ];
    let finals: Vec<(String, u64)> = write_sets
        .iter()
        .map(|writes| {
            let (payload, ts, _) = merge_sequence(writes);
            (payload, ts)
        })
        .collect();
    assert_eq!(
        finals,
        vec![
            ("c".to_string(), 500),
            ("c".to_string(), 500),
            ("c".to_string(), 500)
        ],
        "same-content rewrites must not change the later winner by arrival order"
    );
}

#[test]
fn merge_record_respects_writer_timestamps_against_put_written_records() {
    // Cross-path guard: records written by the local `put` path carry a local
    // `updated_at_ms`; a merge with an older writer clock must NOT clobber them.
    let dir = tempdir().expect("tempdir");
    let db = Embedded::open(dir.path()).expect("open");

    let seeded = db
        .put(MemoryInput::new(NS, KEY, "from-local-put"))
        .expect("put");

    let stale = db
        .merge_record(record(
            "from-remote-old",
            seeded.updated_at_ms.saturating_sub(1),
        ))
        .expect("merge stale");
    assert_eq!(stale.outcome, MergeOutcome::StaleRejected);
    assert_eq!(
        db.get(NS, KEY).expect("get").expect("present").payload,
        "from-local-put"
    );

    let newer = db
        .merge_record(record("from-remote-new", seeded.updated_at_ms + 1000))
        .expect("merge newer");
    assert_eq!(newer.outcome, MergeOutcome::Updated);
    assert_eq!(
        db.get(NS, KEY).expect("get").expect("present").payload,
        "from-remote-new"
    );
}
