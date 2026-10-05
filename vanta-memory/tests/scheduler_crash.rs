// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! WIRE-18 — adversarial crash-mid-pass + restart verification for the
//! scheduler (ADR-0054 T5).
//!
//! The scheduler queue is ephemeral by design (RAM only — ADR-0054/FIND-113):
//! a crash drops queue + claims + locks, never user data — L0/L1 live in the
//! DB. This suite forces a DETERMINISTIC crash mid-pass (a panicking LLM
//! runner: the sanctioned test hook, not timing — plan pre-mortem 1) and
//! verifies after a restart that:
//!   1. the DB is intact (pre-crash L1 bytes survive reopen);
//!   2. the queue is ephemeral (a fresh queue starts empty — accepted
//!      semantics, never silent);
//!   3. work is re-enqueueable from the persisted L0 captures;
//!   4. re-delivering already-processed work (at-least-once reconstruction)
//!      does not corrupt or duplicate: the conflict judge SKIPS memories that
//!      have candidates.
//!
//! `catch_unwind` only keeps THIS test process alive to observe the crash; the
//! in-process stand-in for process death is "panic mid-pass + drop handles
//! without close()" — an in-process test cannot kill its own process without a
//! flaky subprocess (pre-mortem 1: hook, not timing).
//!
//! A second test covers the in-process failure mode (no restart): a pass that
//! panics leaves its claim in `pending`, and the next pass reclaims it after
//! the lease expires (MEM-66) — no data loss.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::{Duration, Instant};

use tempfile::tempdir;
use vanta_memory::core::abstractions::{LlmError, LlmRunParams, LlmRunner, MemoryRecord};
use vanta_memory::core::conversation::{L0Capture, L0Message, L0Recorder, L0Role};
use vanta_memory::core::record::read_session_records;
use vanta_memory::core::state::{TaskKind, TaskPayload};
use vanta_memory::services::scheduler::run_pass;
use vanta_memory::utils::{FakeClock, LocalStateBackend};
use vantadb::config::Config;
use vantadb::sdk::Embedded;
use vantadb::storage::BackendKind;

/// L1 extraction response shape per `tests/l1_extractor.rs` (session crash-a).
const DARK_JSON: &str = r#"[
  {"scene_name": "UI Preferences", "message_ids": ["m1"], "memories": [
    {"content": "User prefers dark mode", "type": "preference", "priority": 80, "source_message_ids": ["m1"]}
  ]}
]"#;

/// L1 extraction response shape per `tests/l1_extractor.rs` (session crash-b).
const LIGHT_JSON: &str = r#"[
  {"scene_name": "UI Preferences", "message_ids": ["m1"], "memories": [
    {"content": "User prefers light mode", "type": "preference", "priority": 80, "source_message_ids": ["m1"]}
  ]}
]"#;

/// Deterministic crash hook (pre-mortem 1): panics while handling the session
/// whose extraction prompt contains `marker` — no timing, no sleeps. The
/// unwind kills the pass mid-flight at a precise, reproducible spot.
struct PanicOnExtraction(&'static str);

impl LlmRunner for PanicOnExtraction {
    fn run(&self, params: &LlmRunParams) -> Result<String, LlmError> {
        match params.task_id.as_str() {
            "l1-extraction" => {
                if params.prompt.contains(self.0) {
                    panic!(
                        "WIRE-18 deterministic crash hook: panicking mid-pass on {:?}",
                        self.0
                    );
                }
                Ok(DARK_JSON.to_string())
            }
            other => panic!("crash hook only expects l1-extraction before the crash; got {other}"),
        }
    }
}

/// Post-restart runner: extraction works; the conflict judge SKIPS every
/// judged memory. The judge only runs when a candidate pool exists — i.e. the
/// memory already persisted before the crash — so this is the deterministic
/// simulation of the real LLM judging a re-delivered duplicate.
struct RecoveryRunner;

impl LlmRunner for RecoveryRunner {
    fn run(&self, params: &LlmRunParams) -> Result<String, LlmError> {
        match params.task_id.as_str() {
            "l1-extraction" => {
                if params.prompt.contains("beta") {
                    Ok(LIGHT_JSON.to_string())
                } else {
                    Ok(DARK_JSON.to_string())
                }
            }
            "l1-conflict-detection" => {
                let judged = params
                    .prompt
                    .split("NEW MEMORIES TO JUDGE")
                    .nth(1)
                    .unwrap_or(&params.prompt);
                let record_id = judged
                    .split("\"record_id\": \"")
                    .nth(1)
                    .and_then(|rest| rest.split('"').next())
                    .unwrap_or("m_0");
                Ok(format!(
                    r#"[{{"record_id": "{record_id}", "action": "skip"}}]"#
                ))
            }
            other => panic!("unexpected LLM call: {other}"),
        }
    }
}

/// Persistent backend (the crate's real-host-restart precedent:
/// `tests/task_checkpoint.rs`). The core default is Fjall too; explicit keeps
/// the intent visible.
fn fjall_config(storage_path: &str) -> Config {
    Config {
        storage_path: storage_path.to_string(),
        backend_kind: BackendKind::Fjall,
        ..Config::default()
    }
}

/// Reopen after the simulated crash. The writer lock may take a moment to be
/// released once the dropped engine finishes unwinding (WIRE-16 precedent:
/// `vantadb-server/tests/scheduler_e2e.rs::open_storage_with_retry`).
fn reopen_with_retry(config: Config) -> Embedded {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match Embedded::open_with_config(config.clone()) {
            Ok(db) => return db,
            Err(e) => {
                assert!(
                    Instant::now() < deadline,
                    "reopen after the simulated crash failed within 10s: {e}"
                );
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }
}

fn capture_turn(db: &Embedded, session: &str, content: &str) {
    L0Recorder::new(db.clone())
        .record_turn(
            &L0Capture {
                session_id: session.to_string(),
                messages: vec![L0Message {
                    id: Some("m1".into()),
                    role: L0Role::User,
                    content: content.to_string(),
                    timestamp_ms: 100,
                }],
            },
            None,
        )
        .expect("record turn");
}

fn l1_task(session: &str) -> TaskPayload {
    TaskPayload {
        id: String::new(),
        kind: TaskKind::L1,
        session_id: session.to_string(),
        priority: 1,
        created_at_ms: 0,
        attempts: 0,
    }
}

#[test]
fn crash_mid_pass_then_restart_keeps_db_intact_and_reprocesses_from_l0() {
    let dir = tempdir().expect("tempdir");
    let db_path = dir.path().join("db").to_string_lossy().to_string();

    // ── Phase A: a pass crashes mid-flight (process death simulated) ──
    let before_a: Vec<MemoryRecord>;
    {
        let db = Embedded::open_with_config(fjall_config(&db_path)).expect("open fjall");
        capture_turn(&db, "crash-a", "alpha prefers dark mode");
        capture_turn(&db, "crash-b", "beta prefers light mode");

        let queue = LocalStateBackend::new(FakeClock::new(1_000));
        queue.enqueue_task(l1_task("crash-a"));
        queue.enqueue_task(l1_task("crash-b"));

        // Deterministic crash: crash-a completes first, then crash-b is claimed
        // and its extraction call panics — the pass dies mid-flight with
        // partial work done and one claim left in `pending`.
        let crashed = catch_unwind(AssertUnwindSafe(|| {
            run_pass(&queue, db.clone(), &PanicOnExtraction("beta"))
        }));
        assert!(
            crashed.is_err(),
            "the deterministic crash hook must fire mid-pass"
        );

        // Mid-pass evidence: both tasks left the queue, exactly one is claimed
        // (crash-b, interrupted), and only crash-a wrote its memory.
        assert_eq!(queue.queue_depth(), (0, 0), "both tasks left the queue");
        assert_eq!(queue.pending_count(), 1, "crash-b claim survives the crash");
        before_a = read_session_records(&db, "crash-a").expect("read l1");
        assert_eq!(before_a.len(), 1, "crash-a processed before the crash");
        assert!(
            read_session_records(&db, "crash-b")
                .expect("read l1")
                .is_empty(),
            "crash-b was interrupted before writing"
        );
        // No `close()`: the process "dies" here (queue + handles dropped).
    }

    // ── Phase B: restart over the same path ──
    let db = reopen_with_retry(fjall_config(&db_path));

    // DB integrity: the persisted L0 captures and the pre-crash L1 bytes
    // survived the ungraceful stop.
    let l0_a = L0Recorder::new(db.clone())
        .read_messages("crash-a")
        .expect("read l0 a");
    let l0_b = L0Recorder::new(db.clone())
        .read_messages("crash-b")
        .expect("read l0 b");
    assert_eq!(l0_a.len(), 1, "L0 crash-a survives the restart");
    assert_eq!(l0_b.len(), 1, "L0 crash-b survives (reconstruction source)");
    assert!(l0_a[0].content.contains("alpha"));
    assert!(l0_b[0].content.contains("beta"));

    let after_a: Vec<MemoryRecord> = read_session_records(&db, "crash-a").expect("read l1");
    assert_eq!(
        after_a, before_a,
        "pre-crash L1 records must survive the restart record-identical"
    );
    assert!(
        read_session_records(&db, "crash-b")
            .expect("read l1")
            .is_empty(),
        "no partial crash-b write survived"
    );

    // The queue is ephemeral by design: a restart starts empty (ADR-0054).
    let queue = LocalStateBackend::new(FakeClock::new(2_000));
    assert_eq!(queue.queue_depth(), (0, 0), "fresh queue starts empty");
    assert_eq!(queue.pending_count(), 0, "no claims survive the restart");

    // Re-enqueue from the persisted L0 captures — the accepted reconstruction
    // path (FIND-113: "tasks re-enqueue from persisted sessions"; no automatic
    // mechanism exists, the host rebuilds the work).
    queue.enqueue_task(l1_task("crash-a")); // at-least-once redelivery
    queue.enqueue_task(l1_task("crash-b")); // interrupted work

    let stats = run_pass(&queue, db.clone(), &RecoveryRunner);
    assert_eq!(
        stats.worker.processed, 2,
        "both reconstructed tasks processed"
    );
    assert_eq!(stats.worker.failed, 0);
    assert_eq!(queue.queue_depth(), (0, 0));
    assert_eq!(queue.pending_count(), 0);

    // Interrupted work completes; re-delivered work is NOT duplicated (the
    // judge skips memories that already have a candidate pool).
    let final_a = read_session_records(&db, "crash-a").expect("read l1");
    assert_eq!(
        final_a.len(),
        1,
        "re-delivery must not duplicate processed work (skip honored)"
    );
    let final_b = read_session_records(&db, "crash-b").expect("read l1");
    assert_eq!(final_b.len(), 1, "interrupted work completes after restart");
    assert!(final_b[0].content.contains("light mode"));

    // A second reconstruction pass converges to the same state (idempotent:
    // both sessions now have candidates → skip again).
    queue.enqueue_task(l1_task("crash-a"));
    queue.enqueue_task(l1_task("crash-b"));
    let again = run_pass(&queue, db.clone(), &RecoveryRunner);
    assert_eq!(again.worker.processed, 2);
    assert_eq!(read_session_records(&db, "crash-a").expect("l1").len(), 1);
    assert_eq!(read_session_records(&db, "crash-b").expect("l1").len(), 1);

    db.close().expect("close");
}

#[test]
fn panicked_pass_claim_is_reclaimed_after_lease_expiry() {
    let dir = tempdir().expect("tempdir");
    let db_path = dir.path().join("db").to_string_lossy().to_string();
    let db = Embedded::open_with_config(fjall_config(&db_path)).expect("open fjall");
    capture_turn(&db, "reclaim", "gamma prefers dark mode");

    let queue = LocalStateBackend::new(FakeClock::new(1_000));
    queue.enqueue_task(l1_task("reclaim"));

    // The pass panics mid-task: the claim stays in `pending` under a lease.
    let crashed = catch_unwind(AssertUnwindSafe(|| {
        run_pass(&queue, db.clone(), &PanicOnExtraction("gamma"))
    }));
    assert!(crashed.is_err(), "the crash hook must fire mid-pass");
    assert_eq!(
        queue.queue_depth(),
        (0, 0),
        "task left the queue when claimed"
    );
    assert_eq!(
        queue.pending_count(),
        1,
        "claim survives the panicking pass"
    );
    assert!(
        read_session_records(&db, "reclaim")
            .expect("read l1")
            .is_empty(),
        "no partial write from the panicked task"
    );

    // Lease still valid (`WorkerConfig::default().lock_ttl_ms` = 60_000): the
    // next pass must NOT steal the claim.
    let early = run_pass(&queue, db.clone(), &RecoveryRunner);
    assert_eq!(early.reclaimed.processed, 0, "valid lease is respected");
    assert_eq!(queue.pending_count(), 1);

    // Lease expiry: the next pass reclaims the dead pass's claim (MEM-66) and
    // completes the work — the panic lost nothing.
    queue.clock().advance(60_001);
    let recovered = run_pass(&queue, db.clone(), &RecoveryRunner);
    assert_eq!(recovered.reclaimed.processed, 1, "stale claim reclaimed");
    assert_eq!(
        recovered.worker.processed, 0,
        "nothing was left in the queue"
    );
    assert_eq!(queue.pending_count(), 0);
    let records = read_session_records(&db, "reclaim").expect("read l1");
    assert_eq!(records.len(), 1, "no data loss after the panicked pass");
    assert!(records[0].content.contains("dark mode"));

    db.close().expect("close");
}
