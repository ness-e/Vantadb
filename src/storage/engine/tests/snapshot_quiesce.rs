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
//! These tests coordinate threads with channels / bounded spins only — no
//! sleeps — so they terminate promptly and deterministically.

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
#[cfg(feature = "fjall")]
#[test]
fn test_snapshot_quiesce_capture_waits_for_held_writer() {
    let dir = tempfile::tempdir().expect("tempdir");
    let engine = std::sync::Arc::new(disk_engine(dir.path(), 250));

    for id in 1..=5 {
        engine.insert(&sample_node(id)).expect("insert seed");
    }

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

    let snapshot = std::thread::spawn({
        let engine = engine.clone();
        move || engine.create_snapshot("patient")
    });

    // Bounded spin (no sleep): the snapshot cannot finish while the writer
    // holds the quiesce guard.
    for _ in 0..10_000 {
        if snapshot.is_finished() {
            break;
        }
        std::thread::yield_now();
    }
    assert!(
        !snapshot.is_finished(),
        "snapshot must wait for the quiesce guard instead of failing"
    );

    release_tx.send(()).expect("release writer");
    writer.join().expect("join writer");
    let snap = snapshot
        .join()
        .expect("join snapshot")
        .expect("snapshot must succeed after the writer released");
    assert!(snap.path.join("snapshot.format").is_file());
    assert!(snap.path.join("backend.manifest").is_file());
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
