//! FIND-640 (M2) — batch-wide atomicity of `batch_insert_with_opts`.
//!
//! With WAL durability enabled the batch runs inside an implicit WAL
//! transaction (`Begin → Insert × chunks → Prepare → per-chunk apply →
//! Commit`) under a single `insert_lock` hold; the durable `Commit` is the
//! commit point. This suite exercises the crash matrix of the design:
//!
//! - committed frame → every record visible after reopen, exactly one
//!   Begin/Prepare/Commit triple in the WAL,
//! - uncommitted frame (Err, panic mid-apply, crash before apply, lost
//!   Commit) → recovery discards it (EOF / Abort rule) and the TXN-01 undo
//!   pass purges the applied prefix → zero records after reopen,
//! - the frame surviving segment rotation and multi-shard round-robin,
//! - `checkpoint_seq` never splitting a frame,
//! - the R3 upsert residual (crashed upsert of a checkpointed record does
//!   not restore the previous version — documented trade-off),
//! - a foreign `insert()` never landing inside the frame's global_seq range.
//!
//! All tests exercise the ENGINE entry point (`batch_insert_with_opts`) —
//! `Embedded::put_batch` reaches the same code through Stage B — but keep
//! the design's `put_batch_multi_chunk_atomic_*` names.
//!
//! Crash simulation pattern (same as the existing crash-consistency tests):
//! failpoints for deterministic injection + `catch_unwind`/scoped threads +
//! `drop(engine)` without `flush()` — `WalWriter`'s `BufWriter` flushes on
//! drop, so the WAL prefix written before the "crash" survives on disk.

use super::super::*;

#[cfg(any(feature = "fjall", feature = "rocksdb"))]
use super::sample_node;
#[cfg(any(feature = "fjall", feature = "rocksdb"))]
use crate::config::Config;
#[cfg(all(feature = "failpoints", any(feature = "fjall", feature = "rocksdb")))]
use crate::node::FieldValue;
#[cfg(any(feature = "fjall", feature = "rocksdb"))]
use crate::node::UnifiedNode;
#[cfg(any(feature = "fjall", feature = "rocksdb"))]
use crate::wal::WalRecord;

// ─── Shared helpers ─────────────────────────────────────────

/// Three internal engine chunks (BATCH_CHUNK_SIZE = 1000) — the minimum
/// batch that exercises the multi-chunk atomicity contract.
#[cfg(any(feature = "fjall", feature = "rocksdb"))]
const THREE_CHUNKS: usize = 2500;

/// `n` fresh nodes with ids `1..=n` (each carrying a small vector).
#[cfg(any(feature = "fjall", feature = "rocksdb"))]
fn fresh_nodes(n: usize) -> Vec<UnifiedNode> {
    (1..=n as u128).map(sample_node).collect()
}

/// `n` fresh nodes with ids `first..first+n` — for appending a second batch
/// after a checkpointed one.
#[cfg(all(feature = "failpoints", any(feature = "fjall", feature = "rocksdb")))]
fn fresh_nodes_from(first: u128, n: usize) -> Vec<UnifiedNode> {
    (first..first + n as u128).map(sample_node).collect()
}

/// Engine batch options with `InsertMode::Incremental`: a >1000-node batch
/// under the default `Auto` mode would defer HNSW insertion (Rebuild), and
/// `get()` resolves the vector payload through the index — the test
/// assertions need the incremental path.
#[cfg(any(feature = "fjall", feature = "rocksdb"))]
fn incremental_opts() -> BatchInsertOptions {
    BatchInsertOptions {
        insert_mode: InsertMode::Incremental,
        ..Default::default()
    }
}

/// Disk engine over `dir` with the given `Config` (single-shard tests pass
/// `single_shard_cfg()` so the WAL's global_seq order = file order and one
/// `vanta.wal` file covers the whole frame).
#[cfg(any(feature = "fjall", feature = "rocksdb"))]
fn disk_engine(dir: &std::path::Path, config: Config) -> StorageEngine {
    StorageEngine::open_with_config(dir.to_str().expect("utf8 path"), Some(config))
        .expect("open disk engine")
}

#[cfg(any(feature = "fjall", feature = "rocksdb"))]
fn single_shard_cfg() -> Config {
    Config {
        wal_shards: 1,
        ..Config::default()
    }
}

/// The engine's WAL directory: `<db>/data/` (`init.rs` opens
/// `data_dir.join("vanta.wal")`, shards land beside it as `vanta.shardN.wal`).
#[cfg(any(feature = "fjall", feature = "rocksdb"))]
fn wal_dir(dir: &std::path::Path) -> std::path::PathBuf {
    dir.join("data")
}

/// Read every record of a WAL segment file — call after the engine is
/// dropped (or after `wal.flush_all()`), so the `BufWriter` tail is on disk.
#[cfg(any(feature = "fjall", feature = "rocksdb"))]
fn wal_records(path: &std::path::Path) -> Vec<WalRecord> {
    let mut reader = crate::wal::WalReader::open(path).expect("open wal segment");
    let mut out = Vec::new();
    while let Some(record) = reader.next_record().expect("read wal record") {
        out.push(record);
    }
    out
}

/// Per-frame marker census of a WAL record stream.
#[cfg(any(feature = "fjall", feature = "rocksdb"))]
#[derive(Default)]
struct FrameMarkers {
    begins: Vec<u64>,
    prepares: Vec<(u64, u32)>,
    commits: Vec<u64>,
    aborts: Vec<u64>,
    inserts: u64,
}

#[cfg(any(feature = "fjall", feature = "rocksdb"))]
fn scan_markers(records: &[WalRecord]) -> FrameMarkers {
    let mut m = FrameMarkers::default();
    for r in records {
        match r {
            WalRecord::Begin(t) => m.begins.push(*t),
            WalRecord::Prepare { txn_id, op_count } => m.prepares.push((*txn_id, *op_count)),
            WalRecord::Commit(t) => m.commits.push(*t),
            WalRecord::Abort(t) => m.aborts.push(*t),
            WalRecord::Insert(_) => m.inserts += 1,
            _ => {}
        }
    }
    m
}

/// Bounded spin on a condition — bounded spins only, no sleeps (repo
/// convention for cross-thread coordination in tests).
#[cfg(all(feature = "failpoints", any(feature = "fjall", feature = "rocksdb")))]
fn wait_until(mut cond: impl FnMut() -> bool, what: &str) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !cond() {
        assert!(
            std::time::Instant::now() < deadline,
            "wait_until: {what} not met within the deadline"
        );
        std::thread::yield_now();
    }
}

/// Assert `get` returns `None` for every id in `first..first+n`.
#[cfg(all(feature = "failpoints", any(feature = "fjall", feature = "rocksdb")))]
fn assert_batch_absent(engine: &StorageEngine, first: u128, n: usize) {
    for id in first..first + n as u128 {
        assert!(
            engine.get(id).expect("get").is_none(),
            "node {id} must NOT survive the uncommitted batch"
        );
    }
}

// ─── Committed frame ────────────────────────────────────────

/// 2500 records (3 chunks) committed → all visible after reopen; the WAL
/// holds exactly one Begin/Prepare/Commit triple, all carrying the same
/// `batch_txn` id, and no Abort.
#[cfg(any(feature = "fjall", feature = "rocksdb"))]
#[test]
fn put_batch_multi_chunk_atomic_commit_visible() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path();
    let cfg = single_shard_cfg();

    {
        let engine = disk_engine(path, cfg.clone());
        engine
            .batch_insert_with_opts(&fresh_nodes(THREE_CHUNKS), incremental_opts())
            .expect("batch commit");
        engine
            .wal
            .as_ref()
            .expect("disk wal")
            .flush_all()
            .expect("flush wal");
    }

    let records = wal_records(&wal_dir(path).join("vanta.wal"));
    let markers = scan_markers(&records);
    assert_eq!(markers.begins.len(), 1, "exactly one Begin marker");
    assert_eq!(markers.prepares.len(), 1, "exactly one Prepare marker");
    assert_eq!(markers.commits.len(), 1, "exactly one Commit marker");
    assert!(markers.aborts.is_empty(), "no Abort on a committed batch");
    assert_eq!(
        markers.inserts, THREE_CHUNKS as u64,
        "one Insert WAL record per batch node"
    );
    let txn = markers.begins[0];
    assert_eq!(
        markers.prepares[0].0, txn,
        "Prepare carries the frame txn id"
    );
    assert_eq!(markers.commits[0], txn, "Commit carries the frame txn id");
    assert_eq!(
        markers.prepares[0].1, THREE_CHUNKS as u32,
        "Prepare op_count = batch len"
    );

    let engine = disk_engine(path, cfg);
    for id in 1..=THREE_CHUNKS as u128 {
        assert!(
            engine.get(id).expect("get").is_some(),
            "node {id} must be visible after reopen"
        );
    }
}

// ─── In-process error mid-apply → Err + Abort + 0 records ───

/// Failpoint `put_batch_chunk_commit_fail` armed `1*return` fires at the
/// 2nd apply chunk: `put_batch` returns `Err`, the existing rollback runs,
/// a best-effort `Abort(batch_txn)` lands in the WAL, and nothing survives
/// the reopen.
#[cfg(all(feature = "failpoints", any(feature = "fjall", feature = "rocksdb")))]
#[test]
fn put_batch_multi_chunk_atomic_fail_second_chunk() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path();
    let cfg = single_shard_cfg();

    {
        let engine = disk_engine(path, cfg.clone());
        fail::cfg("put_batch_chunk_commit_fail", "1*return").expect("arm apply failpoint");
        let res = engine.batch_insert_with_opts(&fresh_nodes(THREE_CHUNKS), incremental_opts());
        fail::cfg("put_batch_chunk_commit_fail", "off").expect("disarm apply failpoint");
        assert!(res.is_err(), "apply failure at chunk 2 must surface Err");
    }

    let records = wal_records(&wal_dir(path).join("vanta.wal"));
    let markers = scan_markers(&records);
    assert_eq!(markers.begins.len(), 1);
    assert_eq!(markers.prepares.len(), 1);
    assert!(markers.commits.is_empty(), "no Commit may be written");
    assert_eq!(markers.aborts.len(), 1, "exactly one Abort marker");
    assert_eq!(
        markers.aborts[0], markers.begins[0],
        "Abort carries the frame txn id"
    );

    let engine = disk_engine(path, cfg);
    assert_batch_absent(&engine, 1, THREE_CHUNKS);
}

// ─── Lost Commit marker → EOF-open frame discarded ──────────

/// `wal_append_fail` armed `"5*off->return"` on the single-shard WAL lets
/// the Begin(1) + 3 chunk `batch_append`s(2,3,4) + Prepare(5) through and
/// fails the 6th WAL call — the `Commit` append (and the best-effort
/// `Abort` after it, which is expected to fail silently). The frame stays
/// EOF-open; recovery discards it and the undo pass purges all three
/// applied chunks — the exact crash the `created_by_txn` stamp exists for.
#[cfg(all(feature = "failpoints", any(feature = "fjall", feature = "rocksdb")))]
#[test]
fn put_batch_multi_chunk_atomic_crash_no_commit() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path();
    let cfg = single_shard_cfg();

    {
        let engine = disk_engine(path, cfg.clone());
        fail::cfg("wal_append_fail", "5*off->return").expect("arm wal_append_fail");
        let res = engine.batch_insert_with_opts(&fresh_nodes(THREE_CHUNKS), incremental_opts());
        fail::cfg("wal_append_fail", "off").expect("disarm wal_append_fail");
        assert!(res.is_err(), "lost Commit must surface Err");
    }

    let records = wal_records(&wal_dir(path).join("vanta.wal"));
    let markers = scan_markers(&records);
    assert_eq!(markers.begins.len(), 1);
    assert_eq!(markers.prepares.len(), 1);
    assert!(
        markers.commits.is_empty(),
        "the Commit append was failed — no Commit may be on disk"
    );
    assert!(
        markers.aborts.is_empty(),
        "the best-effort Abort append fails too under the armed failpoint"
    );

    let engine = disk_engine(path, cfg);
    assert_batch_absent(&engine, 1, THREE_CHUNKS);
}

// ─── Panic mid-apply → EOF-open frame + undo pass ───────────

/// `put_batch_chunk_commit_fail` armed `1*panic` unwinds out of the apply
/// loop at the 2nd chunk: chunk-0's 1000 nodes are already in KV/HNSW/vstore
/// but no `Abort` marker is written (panic ≠ error path). Recovery finds an
/// EOF-open frame, discards it, and the undo pass purges the applied prefix.
#[cfg(all(feature = "failpoints", any(feature = "fjall", feature = "rocksdb")))]
#[test]
fn put_batch_multi_chunk_atomic_panic_mid_apply() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path();
    let cfg = single_shard_cfg();

    {
        let engine = disk_engine(path, cfg.clone());
        fail::cfg("put_batch_chunk_commit_fail", "1*panic").expect("arm panic failpoint");
        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            engine.batch_insert_with_opts(&fresh_nodes(THREE_CHUNKS), incremental_opts())
        }));
        fail::cfg("put_batch_chunk_commit_fail", "off").expect("disarm panic failpoint");
        assert!(res.is_err(), "failpoint panic must unwind out of the batch");
        // Persist the applied chunk-0 prefix — a real crash preserves it; the
        // undo pass must have real work to do on reopen.
        engine.backend.flush().expect("flush backend");
    } // drop(engine) = ungraceful shutdown — no Commit, no Abort.

    let records = wal_records(&wal_dir(path).join("vanta.wal"));
    let markers = scan_markers(&records);
    assert_eq!(markers.begins.len(), 1);
    assert_eq!(markers.prepares.len(), 1);
    assert!(markers.commits.is_empty());
    assert!(markers.aborts.is_empty());

    let engine = disk_engine(path, cfg);
    assert_batch_absent(&engine, 1, THREE_CHUNKS);
}

// ─── Crash during the WAL phase → nothing applied ───────────

/// `put_batch_wal_chunk_fail` armed `1*return` fails the 2nd chunk's
/// `batch_append` in the WAL phase: `Begin` + 1000 `Insert` records are
/// durable, `Prepare` never lands and the apply phase never ran → the undo
/// pass has nothing to purge; the Abort rule discards the prefix.
#[cfg(all(feature = "failpoints", any(feature = "fjall", feature = "rocksdb")))]
#[test]
fn put_batch_multi_chunk_atomic_crash_before_apply() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path();
    let cfg = single_shard_cfg();

    {
        let engine = disk_engine(path, cfg.clone());
        fail::cfg("put_batch_wal_chunk_fail", "1*return").expect("arm wal-phase failpoint");
        let res = engine.batch_insert_with_opts(&fresh_nodes(THREE_CHUNKS), incremental_opts());
        fail::cfg("put_batch_wal_chunk_fail", "off").expect("disarm wal-phase failpoint");
        assert!(res.is_err(), "WAL-phase failure must surface Err");
    }

    let records = wal_records(&wal_dir(path).join("vanta.wal"));
    let markers = scan_markers(&records);
    assert_eq!(markers.begins.len(), 1);
    assert_eq!(markers.inserts, 1000, "only chunk 1 reached the WAL");
    assert!(markers.prepares.is_empty(), "Prepare never landed");
    assert!(markers.commits.is_empty());
    assert_eq!(
        markers.aborts.len(),
        1,
        "best-effort Abort closes the frame"
    );

    let engine = disk_engine(path, cfg);
    assert_batch_absent(&engine, 1, THREE_CHUNKS);
}

// ─── Sharded WAL round-robin ────────────────────────────────

/// Same mid-apply crash with `wal_shards = 4`: the frame's records are
/// round-robin distributed (markers take single global_seq slots, each
/// chunk `batch_append` reserves a contiguous range), `verify_shard_counts`
/// holds, and recovery still discards the whole uncommitted frame.
#[cfg(all(feature = "failpoints", any(feature = "fjall", feature = "rocksdb")))]
#[test]
fn put_batch_multi_chunk_atomic_sharded() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path();
    let cfg = Config {
        wal_shards: 4,
        ..Config::default()
    };

    {
        let engine = disk_engine(path, cfg.clone());
        fail::cfg("put_batch_chunk_commit_fail", "1*panic").expect("arm panic failpoint");
        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            engine.batch_insert_with_opts(&fresh_nodes(THREE_CHUNKS), incremental_opts())
        }));
        fail::cfg("put_batch_chunk_commit_fail", "off").expect("disarm panic failpoint");
        assert!(res.is_err());
        engine.backend.flush().expect("flush backend");
    }

    // Per-shard durable counts must satisfy the round-robin invariant —
    // the same check `recover_state` runs (`init.rs`): a shard short by
    // more than one record vs its sibling would mean a truncated frame.
    let counts: Vec<u64> = (0..4)
        .map(|idx| wal_records(&wal_dir(path).join(format!("vanta.shard{idx}.wal"))).len() as u64)
        .collect();
    assert!(
        crate::wal_sharded::verify_shard_counts(&counts).is_none(),
        "shard counts must stay coherent: {counts:?}"
    );

    // Reopen succeeds (the verify_shard_counts gate passed) and the
    // uncommitted frame is discarded even though it spans all four shards.
    let engine = disk_engine(path, cfg);
    assert_batch_absent(&engine, 1, THREE_CHUNKS);
}

// ─── Frame spanning a rotated segment ───────────────────────

/// `put_batch_wal_chunk_fail` armed `pause` parks the WAL phase at the
/// chunk-2 boundary while the worker holds `insert_lock`; the main thread
/// then archives the active segment (`force_rotate_to_archive` only needs
/// the shard locks). Resumed, the frame continues on a fresh segment —
/// `Begin` + chunk-1 records in the rotated archive, chunks 2-3 + `Prepare`
/// in the active file. A panic at the 2nd apply chunk leaves the frame
/// EOF-open; recovery must walk the rotated segments in order and still
/// discard it.
#[cfg(all(feature = "failpoints", any(feature = "fjall", feature = "rocksdb")))]
#[test]
fn put_batch_multi_chunk_atomic_rotation() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path();
    let cfg = single_shard_cfg();

    {
        let engine = disk_engine(path, cfg.clone());
        let wal = engine.wal.as_ref().expect("disk wal");
        fail::cfg("put_batch_wal_chunk_fail", "pause").expect("park wal phase");
        let nodes = fresh_nodes(THREE_CHUNKS);

        std::thread::scope(|s| {
            let worker = s.spawn(|| engine.batch_insert_with_opts(&nodes, incremental_opts()));

            // Begin(1) + chunk-1 `batch_append`(1000) durable → the worker is
            // about to park at the chunk-2 failpoint (or is already parked —
            // either way it cannot make progress until resumed).
            wait_until(
                || wal.total_record_count() >= 1001,
                "Begin + chunk-1 WAL records durable",
            );
            wal.force_rotate_to_archive()
                .expect("mid-frame segment rotation");

            // Arm the apply-phase crash BEFORE resuming the parked worker.
            fail::cfg("put_batch_chunk_commit_fail", "1*panic").expect("arm apply panic");
            fail::cfg("put_batch_wal_chunk_fail", "off").expect("resume worker");
            assert!(
                worker.join().is_err(),
                "worker must panic at the apply-chunk-2 failpoint"
            );
        });
    } // drop(engine) — no Commit, no Abort, frame split across segments.

    assert!(
        !crate::wal_sharded::enumerate_rotated_segments(&wal_dir(path).join("vanta.wal"))
            .is_empty(),
        "the mid-frame rotation must have archived a segment"
    );

    let engine = disk_engine(path, cfg);
    assert_batch_absent(&engine, 1, THREE_CHUNKS);
}

// ─── checkpoint_seq never splits a frame ────────────────────

/// `flush()` between two batches writes a `checkpoint_seq` covering the
/// first batch's *complete* frame (it can never interleave mid-frame — it
/// takes `insert_lock`). A crashed second batch is discarded post-checkpoint
/// while the first batch's checkpointed records stay visible.
#[cfg(all(feature = "failpoints", any(feature = "fjall", feature = "rocksdb")))]
#[test]
fn put_batch_multi_chunk_atomic_checkpoint_boundary() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path();
    let cfg = single_shard_cfg();

    {
        let engine = disk_engine(path, cfg.clone());
        engine
            .batch_insert_with_opts(&fresh_nodes(2000), incremental_opts())
            .expect("batch A commit");
        engine.flush().expect("flush checkpoint after batch A");

        fail::cfg("put_batch_chunk_commit_fail", "1*return").expect("arm apply failpoint");
        let res = engine.batch_insert_with_opts(&fresh_nodes_from(2001, 1500), incremental_opts());
        fail::cfg("put_batch_chunk_commit_fail", "off").expect("disarm apply failpoint");
        assert!(res.is_err(), "batch B must fail mid-apply");
    }

    let engine = disk_engine(path, cfg);
    for id in 1..=2000u128 {
        assert!(
            engine.get(id).expect("get").is_some(),
            "checkpointed batch-A node {id} must survive"
        );
    }
    assert_batch_absent(&engine, 2001, 1500);
}

// ─── R3: upsert residual under crash ────────────────────────

/// A crashed batch that overwrites a *checkpointed* record does not restore
/// the previous version: the undo pass deletes the batch-stamped KV entry
/// (`created_by_txn == batch_txn`) and the WAL frame does not carry the old
/// payload — the record is ABSENT after reopen. Documented R3 residual,
/// same semantics as an aborted explicit transaction (TXN-01).
#[cfg(all(feature = "failpoints", any(feature = "fjall", feature = "rocksdb")))]
#[test]
fn put_batch_multi_chunk_atomic_upsert_residual() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path();
    let cfg = single_shard_cfg();
    const UPSERTED: u128 = 7;

    {
        let engine = disk_engine(path, cfg.clone());
        engine.insert(&sample_node(UPSERTED)).expect("seed record");
        engine.flush().expect("checkpoint the seeded record");

        let mut nodes = fresh_nodes(1500);
        nodes[(UPSERTED - 1) as usize]
            .set_field("v2".to_string(), FieldValue::String("new".into()));

        fail::cfg("put_batch_chunk_commit_fail", "1*panic").expect("arm panic failpoint");
        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            engine.batch_insert_with_opts(&nodes, incremental_opts())
        }));
        fail::cfg("put_batch_chunk_commit_fail", "off").expect("disarm panic failpoint");
        assert!(res.is_err(), "panic mid-apply = crash-sim");
        // Persist chunk-0's applied upsert (v2 stamped batch_txn) — the undo
        // pass must delete it on reopen, NOT restore v1 (R3).
        engine.backend.flush().expect("flush backend");
    }

    let engine = disk_engine(path, cfg);
    assert!(
        engine.get(UPSERTED).expect("get").is_none(),
        "R3 residual: the crashed upsert removed the checkpointed version — \
         the WAL frame does not carry the previous payload"
    );
    assert_batch_absent(&engine, 1, 1500);
}

// ─── Foreign writes never join the frame ────────────────────

/// An `insert()` racing a batch must never land inside the frame's
/// global_seq range (it would be discarded with it) and must never be
/// silently buffered: while the parked batch holds `insert_lock` the insert
/// fails with the typed, retriable `Timeout` (`insert_lock_timeout_ms`),
/// and retried after the commit it lands AFTER the `Commit` marker.
///
/// This also guards the `alloc_implicit_id` contract: had the batch txn id
/// been registered in `txn.active`, `insert()` would have taken the
/// buffering path (`has_active()==true`) and returned `Ok` — with the write
/// dying silently at frame discard.
#[cfg(all(feature = "failpoints", any(feature = "fjall", feature = "rocksdb")))]
#[test]
fn put_batch_multi_chunk_atomic_concurrent_insert() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path();
    let cfg = Config {
        wal_shards: 1,
        insert_lock_timeout_ms: 250,
        ..Config::default()
    };
    const FOREIGN: u128 = 9_999;

    {
        let engine = disk_engine(path, cfg.clone());
        let wal = engine.wal.as_ref().expect("disk wal");
        let nodes = fresh_nodes(THREE_CHUNKS);
        fail::cfg("put_batch_chunk_commit_fail", "pause").expect("park apply phase");

        std::thread::scope(|s| {
            let worker = s.spawn(|| engine.batch_insert_with_opts(&nodes, incremental_opts()));

            // Begin durable ⇒ the worker holds insert_lock inside the frame.
            wait_until(|| wal.total_record_count() >= 1, "batch txn frame open");

            let res = engine.insert(&sample_node(FOREIGN));
            assert!(
                matches!(res, Err(crate::error::Error::Timeout { .. })),
                "foreign insert must time out while the batch holds insert_lock, got {res:?}"
            );
            assert!(
                engine.get(FOREIGN).expect("get").is_none(),
                "a timed-out insert must not have written anything"
            );

            fail::cfg("put_batch_chunk_commit_fail", "off").expect("resume worker");
            worker
                .join()
                .expect("worker join")
                .expect("batch commits after resume");
        });

        // Retried post-commit the foreign write lands AFTER the Commit marker.
        engine
            .insert(&sample_node(FOREIGN))
            .expect("retry insert post-commit");
        wal.flush_all().expect("flush wal");
    }

    let records = wal_records(&wal_dir(path).join("vanta.wal"));
    let commit_idx = records
        .iter()
        .position(|r| matches!(r, WalRecord::Commit(_)))
        .expect("commit marker");
    let foreign_idx = records
        .iter()
        .position(|r| matches!(r, WalRecord::Insert(n) if n.id == FOREIGN))
        .expect("foreign insert record");
    assert!(
        foreign_idx > commit_idx,
        "the foreign Insert must land outside the batch frame"
    );

    let engine = disk_engine(path, cfg);
    assert!(
        engine.get(FOREIGN).expect("get").is_some(),
        "the retried foreign write must be durable"
    );
    for id in 1..=THREE_CHUNKS as u128 {
        assert!(
            engine.get(id).expect("get").is_some(),
            "batch node {id} must be visible after reopen"
        );
    }
}
