// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
#![cfg(feature = "http-server")]
//! WIRE-15 — contract tests for the feature-gated scheduler LOOP helper.
//!
//! The loop mirrors the core TTL sweeper (watch + join) and runs the
//! pull-based pass through a per-pass runner factory. `None` from the factory
//! skips the pass observably: tasks stay queued (P4 — nothing lost, nothing
//! blocks). The first tick is immediate (Tokio interval semantics), so no
//! test needs to wait a full interval for the first pass.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use vanta_memory::core::abstractions::{LlmError, LlmRunParams, LlmRunner};
use vanta_memory::core::conversation::{L0Capture, L0Message, L0Recorder, L0Role};
use vanta_memory::core::record::read_session_records;
use vanta_memory::core::state::{TaskKind, TaskPayload};
use vanta_memory::services::scheduler::spawn_memory_scheduler;
use vanta_memory::utils::{LocalStateBackend, SystemClock};
use vantadb::cli_server::BackgroundService;
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

/// Bounded wait for an observable condition (no unbounded sleeps).
async fn wait_until(mut cond: impl FnMut() -> bool, what: &str) {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    while !cond() {
        assert!(
            std::time::Instant::now() < deadline,
            "timed out waiting for {what}"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

#[tokio::test]
async fn scheduler_loop_processes_queued_tasks_via_runner_factory() {
    let db = open_db();
    let queue = Arc::new(LocalStateBackend::new(SystemClock));
    capture_turn(&db, "loop");
    queue.enqueue_task(l1_task("loop"));

    let scheduler =
        spawn_memory_scheduler(queue.clone(), db.clone(), Duration::from_millis(50), || {
            Some(ScriptedRunner)
        });

    wait_until(
        || {
            !read_session_records(&db, "loop")
                .unwrap_or_default()
                .is_empty()
        },
        "the loop pass to write l1 records",
    )
    .await;

    tokio::time::timeout(Duration::from_secs(5), scheduler.shutdown())
        .await
        .expect("shutdown must join the loop");

    let records = read_session_records(&db, "loop").expect("read l1 records");
    assert!(records.iter().any(|r| r.content.contains("dark mode")));
}

#[tokio::test]
async fn scheduler_loop_without_runner_skips_pass_and_keeps_tasks_queued() {
    let db = open_db();
    let queue = Arc::new(LocalStateBackend::new(SystemClock));
    capture_turn(&db, "no-runner");
    queue.enqueue_task(l1_task("no-runner"));

    let ticks = Arc::new(AtomicUsize::new(0));
    let ticks_in_factory = ticks.clone();
    let scheduler = spawn_memory_scheduler(
        queue.clone(),
        db.clone(),
        Duration::from_millis(20),
        move || -> Option<ScriptedRunner> {
            ticks_in_factory.fetch_add(1, Ordering::SeqCst);
            None
        },
    );

    wait_until(|| ticks.load(Ordering::SeqCst) > 0, "the loop to tick").await;
    tokio::time::timeout(Duration::from_secs(5), scheduler.shutdown())
        .await
        .expect("shutdown must join the loop");

    assert_eq!(queue.queue_depth(), (0, 1), "task must stay queued (P4)");
    assert!(
        read_session_records(&db, "no-runner")
            .unwrap_or_default()
            .is_empty(),
        "no runner must not process tasks"
    );
}

#[tokio::test]
async fn scheduler_shutdown_joins_loop_and_stops_processing() {
    let db = open_db();
    let queue = Arc::new(LocalStateBackend::new(SystemClock));

    let passes = Arc::new(AtomicUsize::new(0));
    let passes_in_factory = passes.clone();
    let scheduler = spawn_memory_scheduler(
        queue.clone(),
        db.clone(),
        Duration::from_millis(20),
        move || {
            passes_in_factory.fetch_add(1, Ordering::SeqCst);
            Some(ScriptedRunner)
        },
    );

    wait_until(
        || passes.load(Ordering::SeqCst) >= 2,
        "the second pass (factory must be called per pass, not cached)",
    )
    .await;
    tokio::time::timeout(Duration::from_secs(5), scheduler.shutdown())
        .await
        .expect("shutdown must join the loop");

    // Let any in-flight pass drain, then confirm the loop no longer ticks.
    tokio::time::sleep(Duration::from_millis(100)).await;
    let after_shutdown = passes.load(Ordering::SeqCst);
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(
        passes.load(Ordering::SeqCst),
        after_shutdown,
        "no pass may run after shutdown"
    );
}

#[tokio::test]
async fn scheduler_shutdowns_through_background_service_seam() {
    let db = open_db();
    let queue = Arc::new(LocalStateBackend::new(SystemClock));

    let scheduler = spawn_memory_scheduler(
        queue,
        db,
        Duration::from_millis(50),
        || -> Option<ScriptedRunner> { None },
    );
    let service: Box<dyn BackgroundService> = Box::new(scheduler);

    tokio::time::timeout(Duration::from_secs(5), service.shutdown())
        .await
        .expect("seam shutdown must join the loop");
}
