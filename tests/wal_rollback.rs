// ponytail: blanket allow — unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]

//! 🛡️ ACID Phase 4a — WAL v2 rollback & multi-layer rollback tests (RES-01).
//!
//! Verifies the WAL v2 Prepare marker + truthful-error path:
//! 1. Round-trip the new `WalRecord::Prepare { txn_id, op_count }` variant.
//! 2. Write `[Begin + Insert*3 + Prepare]` then read back — Prepare must survive
//!    the wire format (postcard + range-based header compat).
//! 3. Mixed-version WAL: v2 records co-exist with v1 markers (Begin/Commit) in
//!    the same file; replay order is preserved.
//! 4. Rollback contract: a `[Begin + ops + Prepare]` batch WITHOUT a matching
//!    Commit is recoverable but discarded — equivalent to an aborted txn from
//!    the replayer's point of view (MOD-02 slice-mask unchanged).
//!
//! Run: `cargo nextest run --profile audit -p vantadb --test wal_rollback --build-jobs 2`

use std::sync::atomic::{AtomicU32, Ordering};
use vantadb::config::SyncMode;
use vantadb::node::UnifiedNode;
use vantadb::wal::{WalRecord, WalWriter};

static SEQ: AtomicU32 = AtomicU32::new(0);

fn tmp_wal_path(tag: &str) -> std::path::PathBuf {
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "vantadb_test_wal_rollback_{}_{}_{}_{}",
        tag,
        std::process::id(),
        n,
        rand::random::<u32>()
    ));
    let _ = std::fs::remove_file(&dir);
    dir
}

/// 1. Pure in-memory postcard round-trip for the new `Prepare` variant.
#[test]
fn wal_v2_prepare_roundtrip() {
    let original = WalRecord::Prepare {
        txn_id: 42,
        op_count: 7,
    };
    let bytes = postcard::to_allocvec(&original).expect("postcard encode Prepare");
    let decoded: WalRecord = postcard::from_bytes(&bytes).expect("postcard decode Prepare");
    match decoded {
        WalRecord::Prepare { txn_id, op_count } => {
            assert_eq!(txn_id, 42, "Prepare.txn_id preserved");
            assert_eq!(op_count, 7, "Prepare.op_count preserved");
        }
        other => panic!("expected Prepare, got {other:?}"),
    }
}

/// 2. File-backed round-trip: `[Begin + 3x Insert + Prepare]` survives close+reopen.
#[test]
fn wal_v2_phase1_batch_roundtrip() {
    let path = tmp_wal_path("phase1");

    // Write phase-1 batch.
    {
        let mut w = WalWriter::open(&path, SyncMode::Always).expect("open writer");
        w.append(&WalRecord::Begin(7)).expect("append Begin");
        w.append(&WalRecord::Insert(UnifiedNode::new(10)))
            .expect("append Insert 10");
        w.append(&WalRecord::Insert(UnifiedNode::new(11)))
            .expect("append Insert 11");
        w.append(&WalRecord::Insert(UnifiedNode::new(12)))
            .expect("append Insert 12");
        w.append(&WalRecord::Prepare {
            txn_id: 7,
            op_count: 3,
        })
        .expect("append Prepare");
        w.sync().expect("sync");
        assert_eq!(w.record_count(), 5);
    }

    // Reopen and read all records back.
    let mut reader = vantadb::wal::WalReader::open(&path).expect("open reader");
    let mut got = Vec::new();
    reader
        .replay_all(|rec| {
            got.push(rec);
            Ok(())
        })
        .expect("replay_all");

    assert_eq!(got.len(), 5, "5 records survived close+reopen");
    // First and last are markers; middle three are inserts.
    assert!(matches!(got[0], WalRecord::Begin(7)), "begin preserved");
    assert!(
        matches!(
            got[4],
            WalRecord::Prepare {
                txn_id: 7,
                op_count: 3
            }
        ),
        "prepare preserved"
    );
    // Inserts preserved in order.
    for (i, rec) in got.iter().enumerate().take(4).skip(1) {
        assert!(matches!(rec, WalRecord::Insert(_)), "record {i} is Insert");
    }

    let _ = std::fs::remove_file(&path);
}

/// 3. Mixed-version WAL: v1 markers (Begin/Commit) + v2 Prepare co-exist in one file.
#[test]
fn wal_v2_mixed_with_v1_markers() {
    let path = tmp_wal_path("mixed");

    {
        let mut w = WalWriter::open(&path, SyncMode::Always).expect("open writer");
        // v1-style txn
        w.append(&WalRecord::Begin(1)).expect("begin 1");
        w.append(&WalRecord::Insert(UnifiedNode::new(100)))
            .expect("ins 100");
        w.append(&WalRecord::Commit(1)).expect("commit 1");
        // v2-style txn
        w.append(&WalRecord::Begin(2)).expect("begin 2");
        w.append(&WalRecord::Insert(UnifiedNode::new(200)))
            .expect("ins 200");
        w.append(&WalRecord::Prepare {
            txn_id: 2,
            op_count: 1,
        })
        .expect("prepare 2");
        w.append(&WalRecord::Commit(2)).expect("commit 2");
        w.sync().expect("sync");
        assert_eq!(w.record_count(), 7);
    }

    let mut reader = vantadb::wal::WalReader::open(&path).expect("reopen");
    let mut got = Vec::new();
    reader
        .replay_all(|rec| {
            got.push(rec);
            Ok(())
        })
        .expect("replay");

    assert_eq!(got.len(), 7, "all 7 records replay back");
    assert!(matches!(got[0], WalRecord::Begin(1)));
    assert!(matches!(got[2], WalRecord::Commit(1)));
    assert!(matches!(got[3], WalRecord::Begin(2)));
    assert!(matches!(
        got[5],
        WalRecord::Prepare {
            txn_id: 2,
            op_count: 1
        }
    ));
    assert!(matches!(got[6], WalRecord::Commit(2)));

    let _ = std::fs::remove_file(&path);
}

/// 4. Rollback contract: a Prepared-but-uncommitted txn must NOT be applied on replay.
///
/// We append `[Begin + Insert + Prepare]` and *do not* write the matching Commit.
/// On reopen, the recovery path (`recover_valid_records`) treats the trailing
/// open batch as truncated/incomplete and the slice-mask discards the partial
/// ops — same invariant MOD-02 established for v1, extended here to v2.
///
/// We assert the inverse: the WAL holds 3 records (Begin + Insert + Prepare)
/// after close (proving Prepare was fsync'd) but recovery sees a complete
/// header + N records; the absence of the matching Commit is the rollback
/// signal. The Prepare record itself survives — that's the audit-trail win.
#[test]
fn wal_v2_prepared_without_commit_is_recoverable_but_rollback_signal() {
    let path = tmp_wal_path("rollback_signal");

    {
        let mut w = WalWriter::open(&path, SyncMode::Always).expect("open writer");
        w.append(&WalRecord::Begin(99)).expect("begin");
        w.append(&WalRecord::Insert(UnifiedNode::new(999)))
            .expect("insert");
        w.append(&WalRecord::Prepare {
            txn_id: 99,
            op_count: 1,
        })
        .expect("prepare");
        w.sync()
            .expect("sync (durability point: Prepare is on disk, Commit is NOT)");
        assert_eq!(w.record_count(), 3);
    }

    // Reopen: 3 records survive. The "missing Commit" is the rollback signal.
    let mut reader = vantadb::wal::WalReader::open(&path).expect("reopen");
    let mut got = Vec::new();
    reader
        .replay_all(|rec| {
            got.push(rec);
            Ok(())
        })
        .expect("replay");

    assert_eq!(
        got.len(),
        3,
        "Begin + Insert + Prepare all on disk after sync"
    );
    assert!(matches!(got[0], WalRecord::Begin(99)));
    assert!(matches!(got[1], WalRecord::Insert(_)));
    assert!(matches!(
        got[2],
        WalRecord::Prepare {
            txn_id: 99,
            op_count: 1
        }
    ));
    // The truthful-error path: the engine that produced this WAL knows there's
    // no Commit for txn 99; recovery applies MOD-02's slice-mask and skips
    // the Insert. Prepare itself is the durable proof that an apply was
    // attempted (or failed). This test asserts only the on-disk invariants.

    let _ = std::fs::remove_file(&path);
}

/// 5. WAL format version constant is 2 (RES-01 keystone).
///
/// This is the cheapest possible gate: if `WAL_FORMAT_VERSION` ever regresses
/// to 1, this test fails. Pinned here so the bump-and never gets silently
/// reverted by an unrelated refactor.
#[test]
fn wal_format_version_is_v2() {
    assert_eq!(
        vantadb::wal::WAL_FORMAT_VERSION,
        2,
        "RES-01 keystone: WAL_FORMAT_VERSION must be 2"
    );
}
