// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! WIRE-15 — D19 contract tests for the scheduler PASS (`run_pass`).
//!
//! The pass is the pull-based reuse of the MEM-16 pieces: expired `l1_idle:`
//! timers → L1 tasks + one worker pass + stale-claim reclaim. All time is
//! driven by a [`FakeClock`] — deterministic, zero sleeps. The Tokio loop
//! helper (feature `http-server`) is covered in `tests/scheduler_loop.rs`.

use vanta_memory::core::abstractions::{LlmError, LlmRunParams, LlmRunner};
use vanta_memory::core::conversation::{L0Capture, L0Message, L0Recorder, L0Role};
use vanta_memory::core::record::read_session_records;
use vanta_memory::core::state::{TaskKind, TaskPayload};
use vanta_memory::services::scheduler::run_pass;
use vanta_memory::utils::{l1_idle_member, FakeClock, LocalStateBackend};
use vantadb::config::Config;
use vantadb::sdk::Embedded;
use vantadb::storage::BackendKind;

/// L1 extraction response shape per `tests/l1_extractor.rs`.
const EXTRACTION_JSON: &str = r#"[
  {"scene_name": "UI Preferences", "message_ids": ["m1"], "memories": [
    {"content": "User prefers dark mode", "type": "preference", "priority": 80, "source_message_ids": ["m1"]}
  ]}
]"#;

/// Fake runner covering the two LLM calls an L1 pass makes (pattern proven in
/// `tests/conversation_hook.rs`).
struct ScriptedRunner;

impl LlmRunner for ScriptedRunner {
    fn run(&self, params: &LlmRunParams) -> Result<String, LlmError> {
        match params.task_id.as_str() {
            "l1-extraction" => Ok(EXTRACTION_JSON.to_string()),
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
                    r#"[{{"record_id": "{record_id}", "action": "store"}}]"#
                ))
            }
            other => panic!("unexpected LLM call: {other}"),
        }
    }
}

fn open_db() -> Embedded {
    Embedded::open_with_config(Config {
        backend_kind: BackendKind::InMemory,
        ..Config::default()
    })
    .expect("open in-memory db")
}

fn capture_turn(db: &Embedded, session: &str) {
    L0Recorder::new(db.clone())
        .record_turn(
            &L0Capture {
                session_id: session.to_string(),
                messages: vec![L0Message {
                    id: Some("m1".into()),
                    role: L0Role::User,
                    content: "I prefer dark mode".into(),
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

// ═══ pass: worker ═══

#[test]
fn pass_processes_queued_l1_task_and_writes_memories() {
    let db = open_db();
    let queue = LocalStateBackend::new(FakeClock::new(1_000));
    capture_turn(&db, "pass");
    queue.enqueue_task(l1_task("pass"));

    let stats = run_pass(&queue, db.clone(), &ScriptedRunner);

    assert_eq!(stats.worker.processed, 1);
    assert_eq!(stats.worker.failed, 0);
    assert_eq!(queue.queue_depth(), (0, 0));
    let records = read_session_records(&db, "pass").expect("read l1 records");
    assert!(
        records.iter().any(|r| r.content.contains("dark mode")),
        "expected extracted memory in l1/pass, got: {records:?}"
    );
}

// ═══ pass: timer dispatch ═══

#[test]
fn pass_dispatches_expired_idle_timer_into_l1_task_once() {
    let db = open_db();
    let queue = LocalStateBackend::new(FakeClock::new(1_000));
    capture_turn(&db, "timer");
    queue.set_timer(&l1_idle_member("timer"), 2_000);

    // Not due yet: the scanner finds nothing and the worker has no task.
    let early = run_pass(&queue, db.clone(), &ScriptedRunner);
    assert_eq!(early.timers_fired, 0);
    assert_eq!(early.worker.processed, 0);

    queue.clock().set_time(2_000);
    let stats = run_pass(&queue, db.clone(), &ScriptedRunner);

    assert_eq!(stats.timers_fired, 1);
    assert_eq!(stats.worker.processed, 1);
    let records = read_session_records(&db, "timer").expect("read l1 records");
    assert!(records.iter().any(|r| r.content.contains("dark mode")));

    // Consumed once: a second pass fires nothing.
    let again = run_pass(&queue, db.clone(), &ScriptedRunner);
    assert_eq!(again.timers_fired, 0);
    assert_eq!(again.worker.processed, 0);
}

#[test]
fn pass_skips_unknown_timer_members_without_enqueuing() {
    let db = open_db();
    let queue = LocalStateBackend::new(FakeClock::new(100));
    queue.set_timer("l2:future", 50);

    let stats = run_pass(&queue, db, &ScriptedRunner);

    assert_eq!(stats.timers_fired, 1, "the scanner consumes the entry");
    assert_eq!(stats.worker.processed, 0, "unknown member must not enqueue");
    assert_eq!(queue.queue_depth(), (0, 0));
}

// ═══ pass: stale-claim reclaim ═══

#[test]
fn pass_reclaims_stale_task_after_lease_expiry() {
    let db = open_db();
    let queue = LocalStateBackend::new(FakeClock::new(1_000));
    capture_turn(&db, "stale");
    queue.enqueue_task(l1_task("stale"));

    // A worker claims the task with a 1ms lease and dies before completing.
    let claimed = queue.claim_task("dead-worker", 1).expect("claim");
    assert_eq!(claimed.session_id, "stale");
    assert_eq!(queue.pending_count(), 1);
    queue.clock().advance(2); // lease expired

    let stats = run_pass(&queue, db.clone(), &ScriptedRunner);

    assert_eq!(
        stats.reclaimed.processed, 1,
        "stale claim must be reclaimed"
    );
    assert_eq!(stats.worker.processed, 0, "nothing was left in the queue");
    assert_eq!(queue.pending_count(), 0);
    let records = read_session_records(&db, "stale").expect("read l1 records");
    assert!(records.iter().any(|r| r.content.contains("dark mode")));
}
