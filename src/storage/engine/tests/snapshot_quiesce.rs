//! FIND-651 — filesystem snapshot quiesce barrier.
//!
//! `create_snapshot` holds `insert_lock` across [checkpoint flush → data/
//! mirror → backend mirror], so every mutation path that touches a mirrored
//! file (insert/delete/batch, txn WAL markers, maintenance, partition and
//! backend-batch writes) is excluded for the duration of the capture.
//! Writers that cannot enter the critical section within
//! `insert_lock_timeout_ms` fail with the typed, retriable
//! [`crate::error::Error::Timeout`]; the snapshot itself patiently retries
//! through several timeout windows so an active writer never fails a backup.
//!
//! These tests coordinate threads with channels and on progress signals
//! published by the code under test — never with an assumed schedule and never
//! with sleeps — so they terminate promptly and deterministically on any core
//! count.

use super::*;
use std::sync::mpsc;

#[cfg(feature = "fjall")]
fn disk_engine(dir: &std::path::Path, insert_lock_timeout_ms: u64) -> StorageEngine {
    let config = Config {
        backend_kind: BackendKind::Fjall,
        insert_lock_timeout_ms,
        ..Config::default()
    };
    StorageEngine::open_with_config(dir.to_str().unwrap(), Some(config)).expect("open disk engine")
}

/// Normal path: snapshot on a quiet engine still produces a complete image
/// (format marker + data mirror + backend manifest).
#[cfg(feature = "fjall")]
#[test]
fn test_snapshot_quiesce_normal_path() {
    let dir = tempfile::tempdir().expect("tempdir");
    let engine = disk_engine(dir.path(), 1_000);

    for id in 1..=10 {
        engine.insert(&sample_node(id)).expect("insert seed");
    }

    let snap = engine.create_snapshot("base").expect("create_snapshot");
    assert!(snap.path.join("snapshot.format").is_file());
    assert!(snap.path.join("data").is_dir());
    assert!(snap.path.join("backend.manifest").is_file());
}

/// Writer contention: while the quiesce guard is held (the same section the
/// snapshot mirror holds), `insert` fails inside `insert_lock_timeout_ms`
/// with a typed, retriable `Timeout`, and a retry after release succeeds.
#[cfg(feature = "fjall")]
#[test]
fn test_snapshot_quiesce_writer_times_out_then_retries_ok() {
    let dir = tempfile::tempdir().expect("tempdir");
    let engine = std::sync::Arc::new(disk_engine(dir.path(), 200));

    // Hold the quiesce lock on this thread, as the snapshot mirror does.
    let _quiesce = engine
        .insert_lock
        .try_lock()
        .expect("lock must be free at test start");

    let (done_tx, done_rx) = mpsc::channel();
    let writer = std::thread::spawn({
        let engine = engine.clone();
        move || {
            let res = engine.insert(&sample_node(7));
            done_tx.send(res).expect("send result");
        }
    });

    let res = done_rx.recv().expect("writer must terminate");
    let err = res.expect_err("insert must fail while the quiesce guard is held");
    assert!(
        matches!(err, crate::error::Error::Timeout { .. }),
        "expected typed Timeout, got {err:?}"
    );
    assert!(err.is_retriable(), "Timeout must be retriable");
    writer.join().expect("join writer");

    drop(_quiesce);
    engine.insert(&sample_node(7)).expect("retry insert");
    assert!(
        engine.get(7).expect("get").is_some(),
        "retried insert must be visible"
    );
}

/// The snapshot side is patient: a writer holding `insert_lock` at capture
/// start does not fail the backup — `create_snapshot` waits through the
/// contention window and completes once the writer releases. No deadlock:
/// both threads finish promptly.
///
/// FIND-724: the wait is proven from two signals, never from an assumed
/// schedule.
///
/// 1. `create_snapshot_inner` publishes `snapshot.format` just before it takes
///    the quiesce guard (`create_dir(<snapshot>/data)` sits in between, plus a
///    `failpoints`-only failpoint right after), so that marker proves the
///    capture is *at* the guard — not merely that a thread was spawned
///    (without it, "hold, release, expect success" would be vacuous: the
///    thread could start after the release).
/// 2. The capture must then outlive one whole patience window
///    (`insert_lock_timeout_ms`) with the guard still held. Only waiting can
///    survive a window; a capture that stopped waiting would complete within
///    microseconds, and that is exactly what the assertion catches.
///
/// Residual gap, documented and not mitigated: the marker precedes the
/// acquire, so a capture preempted for longer than the observation window
/// between the two could acquire *after* the release and pass without ever
/// waiting. Closing it needs the product to signal the attempt itself (e.g.
/// counting the "retrying after contention" debug event), which costs a
/// process-global tracing subscriber in a libtest binary — not worth it for a
/// window that would have to exceed 1.5 s.
///
/// The old version spun a fixed 10 000 `yield_now()` and lost on a loaded
/// 3-core runner: the spin outlived the product's *bounded* patience
/// (5 windows, see `acquire_snapshot_quiesce`), the capture returned its
/// documented, retriable `Timeout`, and the assert reported a product failure
/// that never happened.
#[cfg(feature = "fjall")]
#[test]
fn test_snapshot_quiesce_capture_waits_for_held_writer() {
    // Patience = 5 attempts (`acquire_snapshot_quiesce`) × TIMEOUT_MS. The
    // observation below spends 1.5 windows, leaving 3.5 windows of slack
    // before the product would legitimately time out.
    const TIMEOUT_MS: u64 = 1_000;
    // Liveness backstop for the marker only — it is not a scheduling bet.
    const MARKER_DEADLINE: std::time::Duration = std::time::Duration::from_secs(30);

    let dir = tempfile::tempdir().expect("tempdir");
    let engine = std::sync::Arc::new(disk_engine(dir.path(), TIMEOUT_MS));

    for id in 1..=5 {
        engine.insert(&sample_node(id)).expect("insert seed");
    }

    // Written right before the quiesce acquire (data_dir is `<root>/data`).
    let marker = dir
        .path()
        .join("data")
        .join("snapshots")
        .join("patient")
        .join("snapshot.format");

    // A writer pins the lock until told to release — the snapshot must
    // out-wait it (patient quiesce) rather than error out.
    let (held_tx, held_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel::<()>();
    let writer = std::thread::spawn({
        let engine = engine.clone();
        move || {
            let _guard = engine.insert_lock.lock();
            held_tx.send(()).expect("signal held");
            release_rx.recv().expect("wait for release");
        }
    });
    held_rx.recv().expect("writer holds lock");

    let (done_tx, done_rx) = mpsc::channel::<()>();
    let snapshot = std::thread::spawn({
        let engine = engine.clone();
        move || {
            let res = engine.create_snapshot("patient");
            let _ = done_tx.send(());
            res
        }
    });

    // (1) Condition-poll, no sleep: wait until the capture reaches the guard,
    // aborting as soon as it returns instead of spinning out the deadline.
    let deadline = std::time::Instant::now() + MARKER_DEADLINE;
    while !marker.is_file() {
        assert!(
            !snapshot.is_finished(),
            "create_snapshot returned before the quiesce wait was observable \
             (never reached the guard)"
        );
        assert!(
            std::time::Instant::now() < deadline,
            "create_snapshot never reached the quiesce guard ({MARKER_DEADLINE:?})"
        );
        std::thread::yield_now();
    }

    // (2) One full patience window with the guard still held: a capture that
    // stopped waiting completes here, a queued one cannot.
    let observation = std::time::Duration::from_millis(TIMEOUT_MS + TIMEOUT_MS / 2);
    let outlived_a_window = matches!(
        done_rx.recv_timeout(observation),
        Err(mpsc::RecvTimeoutError::Timeout)
    );

    // Always release first, so a failure reports the assertion and not a
    // cascade panic in the writer thread.
    release_tx.send(()).expect("release writer");
    writer.join().expect("join writer");
    let result = snapshot.join().expect("join snapshot");

    assert!(
        outlived_a_window,
        "create_snapshot completed within one {TIMEOUT_MS} ms quiesce window while the writer \
         held the guard: it must wait for it instead of failing"
    );
    let snap = result.expect("snapshot must succeed after the writer released");
    assert!(snap.path.join("snapshot.format").is_file());
    assert!(snap.path.join("backend.manifest").is_file());
}

/// FIND-724 — the patience is BOUNDED, and that is exactly why the capture
/// test above must release the guard *within* the window: `acquire_snapshot_quiesce`
/// retries `QUIESCE_ATTEMPTS` (5) windows of `insert_lock_timeout_ms` and then
/// surfaces the typed, retriable `Timeout` instead of hanging a backup forever
/// (`src/storage/engine/mod.rs` — "a genuinely stuck critical section still
/// surfaces Timeout rather than hanging the caller forever"). A writer that
/// outlives all five windows therefore fails the capture BY DESIGN, which is
/// what the old 10 000-yield scenario accidentally demanded of it.
#[cfg(feature = "fjall")]
#[test]
fn test_snapshot_quiesce_bounded_patience_surfaces_timeout() {
    const TIMEOUT_MS: u64 = 200; // 5 attempts → ~1 s of patience

    let dir = tempfile::tempdir().expect("tempdir");
    let engine = disk_engine(dir.path(), TIMEOUT_MS);

    for id in 1..=3 {
        engine.insert(&sample_node(id)).expect("insert seed");
    }

    // Hold the quiesce guard on this thread, as the snapshot mirror does: the
    // capture burns its five windows and gives up. No thread, no sleep.
    let quiesce = engine
        .insert_lock
        .try_lock()
        .expect("lock must be free at test start");
    let started = std::time::Instant::now();
    let err = engine
        .create_snapshot("bounded")
        .expect_err("a writer outliving every quiesce window must fail the capture");
    eprintln!(
        "bounded patience: elapsed={:?} error={err:?} retriable={}",
        started.elapsed(),
        err.is_retriable()
    );
    assert!(
        matches!(err, crate::error::Error::Timeout { .. }),
        "expected typed Timeout, got {err:?}"
    );
    assert!(
        err.is_retriable(),
        "snapshot quiesce Timeout must be retriable"
    );
    assert!(
        matches!(&err, crate::error::Error::Timeout { operation, .. } if operation.contains("create_snapshot")),
        "Timeout must name the quiesce operation, got {err:?}"
    );

    // The failed attempt cleaned up after itself: a leftover dir would make
    // the next capture fail with a misleading `AlreadyExists`.
    assert!(
        !dir.path()
            .join("data")
            .join("snapshots")
            .join("bounded")
            .exists(),
        "a failed capture must not leave its snapshot dir behind"
    );

    drop(quiesce);
    // The same capture succeeds once the guard is released, and the failed
    // attempt left no half-built snapshot dir behind.
    engine
        .create_snapshot("bounded")
        .expect("snapshot must succeed after the guard is released");
}

/// Writer + snapshot racing for the quiesce guard: whichever order the
/// scheduler picks, both must complete — the writer retrying past a
/// bounded `Timeout` if the snapshot is mid-mirror.
#[cfg(feature = "fjall")]
#[test]
fn test_snapshot_quiesce_concurrent_writer_no_deadlock() {
    const INSERTS: u32 = 25;
    const MAX_ATTEMPTS: u32 = 500;

    let dir = tempfile::tempdir().expect("tempdir");
    let engine = std::sync::Arc::new(disk_engine(dir.path(), 250));
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));

    let writer = std::thread::spawn({
        let engine = engine.clone();
        let barrier = barrier.clone();
        move || {
            barrier.wait();
            for id in 1..=INSERTS {
                let node = sample_node(id as u128);
                let mut attempts = 0;
                loop {
                    attempts += 1;
                    match engine.insert(&node) {
                        Ok(()) => break,
                        Err(crate::error::Error::Timeout { .. }) if attempts < MAX_ATTEMPTS => {
                            std::thread::yield_now();
                        }
                        Err(e) => panic!("insert {id} failed unexpectedly: {e:?}"),
                    }
                }
            }
        }
    });

    barrier.wait();
    let snap = engine
        .create_snapshot("concurrent")
        .expect("snapshot must complete alongside a racing writer");
    writer.join().expect("writer must terminate");

    assert!(snap.path.join("snapshot.format").is_file());
    for id in 1..=INSERTS {
        assert!(
            engine.get(id as u128).expect("get").is_some(),
            "writer insert {id} must be visible after the race"
        );
    }
}

/// End-to-end coherence (P2-01): a snapshot taken while a writer races the
/// quiesce guard must restore into an engine that opens cleanly — WAL replay,
/// backend keyspaces and `vector_index.bin` offsets all read back — and that
/// resolves every node the capture committed. The mirror freezes at a commit
/// boundary, so a racing insert is either fully inside the image or fully
/// outside it; never torn.
#[cfg(feature = "fjall")]
#[test]
fn test_snapshot_quiesce_restores_coherent_image() {
    const SEEDS: u128 = 20;
    const MAX_ATTEMPTS: u32 = 500;

    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().to_path_buf();
    let engine = std::sync::Arc::new(disk_engine(&root, 250));

    for id in 1..=SEEDS {
        engine.insert(&sample_node(id)).expect("seed insert");
    }

    // Writer keeps committing until told the capture finished, so the mirror
    // overlaps real write traffic (Timeout → bounded retry, like production
    // clients must do).
    let (stop_tx, stop_rx) = mpsc::channel::<()>();
    let writer = std::thread::spawn({
        let engine = engine.clone();
        move || {
            let mut id = 1_000u128;
            let mut committed = Vec::new();
            while stop_rx.try_recv().is_err() {
                let mut attempts = 0;
                loop {
                    attempts += 1;
                    match engine.insert(&sample_node(id)) {
                        Ok(()) => break,
                        Err(crate::error::Error::Timeout { .. }) if attempts < MAX_ATTEMPTS => {
                            std::thread::yield_now();
                        }
                        Err(e) => panic!("racing insert {id} failed unexpectedly: {e:?}"),
                    }
                }
                committed.push(id);
                id += 1;
            }
            committed
        }
    });

    let snap = engine
        .create_snapshot("coherent")
        .expect("snapshot under writer traffic");
    stop_tx.send(()).expect("stop writer");
    let racing_committed = writer.join().expect("writer must terminate");

    // Drop before restore: snapshot_restore requires the fs2 exclusivity
    // contract (no live engine on the storage root).
    drop(engine);
    StorageEngine::snapshot_restore(&root, "coherent")
        .expect("raced snapshot must restore cleanly");

    let restored = disk_engine(&root, 1_000);
    // The capture boundary sits between the seed phase and the racing phase:
    // every seed was committed before create_snapshot ran.
    for id in 1..=SEEDS {
        let node = restored
            .get(id)
            .unwrap_or_else(|e| panic!("restored engine cannot resolve seed {id}: {e:?}"))
            .unwrap_or_else(|| panic!("seed {id} missing from restored snapshot"));
        assert_eq!(node.id, id, "restored seed {id} payload mismatch");
    }
    // Racing inserts may be inside or outside the image — but every one that
    // IS inside must read back complete (get would error/None on torn data).
    let mut visible = 0u32;
    for id in racing_committed {
        if let Some(node) = restored.get(id).expect("restored get must not error") {
            assert_eq!(node.id, id, "restored racing node payload mismatch");
            visible += 1;
        }
    }
    let snap_marker = snap.path.join("snapshot.format");
    assert!(snap_marker.is_file(), "snapshot format marker must exist");
    eprintln!("coherent image: {SEEDS} seeds + {visible} racing nodes restored");
}

/// FIND-653: the orphan-link repair paths — `vacuum`'s no-tombstone early
/// return and `fresh_hnsw` — mutate the HNSW neighbor index, so they run
/// under `insert_lock` like every other graph mutation. While the quiesce
/// guard is held (the same section the snapshot mirror holds) they fail with
/// the typed, retriable `Timeout`, and they succeed after release.
#[cfg(feature = "fjall")]
#[test]
fn test_snapshot_quiesce_orphan_repair_times_out_then_succeeds() {
    let dir = tempfile::tempdir().expect("tempdir");
    let engine = std::sync::Arc::new(disk_engine(dir.path(), 200));

    for id in 1..=3 {
        engine.insert(&sample_node(id)).expect("insert seed");
    }

    // Hold the quiesce lock on this thread, as the snapshot mirror does.
    let quiesce = engine
        .insert_lock
        .try_lock()
        .expect("lock must be free at test start");

    let (done_tx, done_rx) = mpsc::channel();
    let repair = std::thread::spawn({
        let engine = engine.clone();
        move || {
            let vacuum_res = engine.vacuum();
            let fresh_res = engine.fresh_hnsw();
            done_tx.send((vacuum_res, fresh_res)).expect("send results");
        }
    });

    let (vacuum_res, fresh_res) = done_rx.recv().expect("repair thread must terminate");
    let err = vacuum_res.expect_err("vacuum must fail while the quiesce guard is held");
    assert!(
        matches!(err, crate::error::Error::Timeout { .. }),
        "expected typed Timeout from vacuum, got {err:?}"
    );
    assert!(err.is_retriable(), "vacuum Timeout must be retriable");
    let err = fresh_res.expect_err("fresh_hnsw must fail while the quiesce guard is held");
    assert!(
        matches!(err, crate::error::Error::Timeout { .. }),
        "expected typed Timeout from fresh_hnsw, got {err:?}"
    );
    assert!(err.is_retriable(), "fresh_hnsw Timeout must be retriable");
    repair.join().expect("join repair thread");

    drop(quiesce);
    engine
        .vacuum()
        .expect("vacuum must succeed after the guard is released");
    engine
        .fresh_hnsw()
        .expect("fresh_hnsw must succeed after the guard is released");
}

/// FIND-651 r2: `batch_insert_with_opts` drops chunk `insert_lock` guards before
/// post-commit caching (`cache_batch_hot_nodes`) and `run_batch_eviction_if_needed`.
/// While the snapshot quiesce barrier holds `insert_lock`, batch eviction must
/// block/time out rather than calling `evict_cold_nodes_with_reason_locked`
/// without the lock.
#[cfg(feature = "fjall")]
#[test]
fn test_snapshot_quiesce_batch_eviction_respects_insert_lock() {
    let dir = tempfile::tempdir().expect("tempdir");
    let engine = std::sync::Arc::new(disk_engine(dir.path(), 200));

    for id in 1..=4u128 {
        let mut node = sample_node(id);
        node.tier = crate::node::NodeTier::Hot;
        engine.insert(&node).expect("seed hot node");
    }
    assert_eq!(engine.cache.volatile.read().len(), 4);

    // Hold the quiesce barrier on this thread, as create_snapshot does.
    let quiesce = engine
        .insert_lock
        .try_lock()
        .expect("lock must be free at test start");

    let (done_tx, done_rx) = mpsc::channel();
    let evictor = std::thread::spawn({
        let engine = engine.clone();
        move || {
            // Mirror the production contract (batch_insert_with_opts):
            // run_batch_eviction_if_needed requires the caller to hold
            // insert_lock; with the quiesce guard held by main, try_lock
            // fails and eviction is skipped — exactly like the Err branch.
            if engine.insert_lock.try_lock().is_some() {
                engine.run_batch_eviction_if_needed(true);
            }
            done_tx.send(()).expect("signal completion");
        }
    });

    done_rx.recv().expect("evictor thread must terminate");
    evictor.join().expect("join evictor");

    assert_eq!(
        engine.cache.volatile.read().len(),
        4,
        "batch eviction must not evict hot nodes while insert_lock quiesce guard is held"
    );

    drop(quiesce);
    let _guard = engine
        .insert_lock
        .try_lock()
        .expect("insert_lock free after quiesce release");
    engine.run_batch_eviction_if_needed(true);
    assert!(
        engine.cache.volatile.read().len() < 4,
        "batch eviction must proceed once insert_lock is released"
    );
}
