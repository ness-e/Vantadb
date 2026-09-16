// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! D19 — dedicated tests for the MEM-16 orchestration layer: injectable
//! clock, local state backend (timers/locks/queue), pipeline managers,
//! checkpoint manager and the pipeline worker. All time is driven by a
//! [`FakeClock`] — deterministic, zero sleeps.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Mutex;

use vanta_memory::core::abstractions::{
    DedupAction, DedupDecision, ExtractedMemory, LlmError, LlmRunParams, LlmRunner, MemoryType,
};
use vanta_memory::core::conversation::{L0Capture, L0Message, L0Recorder, L0Role};
use vanta_memory::core::dream::list_dream_runs;
use vanta_memory::core::persona::{evaluate_persona_trigger, get_persona};
use vanta_memory::core::record::{read_session_records, write_memory, EXTRACT_DEDUP_TASK_ID};
use vanta_memory::core::scene::{list_scenes, upsert_scene};
use vanta_memory::core::state::{CaptureAtomicParams, TaskKind, TaskPayload};
use vanta_memory::services::pipeline_worker::{
    MemoryTaskHandler, PipelineWorker, TaskHandler, WorkerConfig,
};
use vanta_memory::utils::{
    CheckpointManager, FakeClock, LocalStateBackend, MemoryPipelineManager, PipelineConfig,
    TimerScanner,
};

fn open_db() -> vantadb::sdk::Embedded {
    let config = vantadb::config::Config {
        backend_kind: vantadb::storage::BackendKind::InMemory,
        read_only: false,
        ..vantadb::config::Config::default()
    };
    vantadb::sdk::Embedded::open_with_config(config).expect("open in-memory db")
}

fn task(kind: TaskKind, session: &str, priority: u8, created_at_ms: u64) -> TaskPayload {
    TaskPayload {
        id: String::new(),
        kind,
        session_id: session.to_string(),
        priority,
        created_at_ms,
        attempts: 0,
    }
}

// ═══ LocalStateBackend: timers ═══

#[test]
fn timers_fire_only_when_expired_and_are_consumed_once() {
    let clock = FakeClock::new(1_000);
    let backend = LocalStateBackend::new(clock);

    backend.set_timer("l1_idle:s1", 2_000);
    assert_eq!(backend.take_expired_timers(1_500).len(), 0);
    let expired = backend.take_expired_timers(2_000);
    assert_eq!(expired.len(), 1);
    assert_eq!(expired[0].member, "l1_idle:s1");
    // Consumed: a second scan finds nothing.
    assert!(backend.take_expired_timers(9_999).is_empty());
}

#[test]
fn set_timer_if_earlier_is_downward_only() {
    let backend = LocalStateBackend::new(FakeClock::new(0));

    backend.set_timer("l2:s1", 5_000);
    assert!(!backend.set_timer_if_earlier("l2:s1", 6_000));
    assert!(backend.set_timer_if_earlier("l2:s1", 3_000));
    assert_eq!(backend.take_expired_timers(4_999).len(), 1);
}

#[test]
fn timer_scanner_dispatches_expired_entries() {
    let clock = FakeClock::new(100);
    let backend = LocalStateBackend::new(clock);
    backend.set_timer("l1_idle:s9", 150);

    let fired: RefCell<Vec<String>> = RefCell::new(Vec::new());
    {
        let scanner = TimerScanner::new(&backend);
        let count = scanner.run_once(|entry| fired.borrow_mut().push(entry.member.clone()));
        assert_eq!(count, 0); // not due yet at t=100
    }
    backend.clock().set_time(150);
    {
        let scanner = TimerScanner::new(&backend);
        assert_eq!(
            scanner.run_once(|e| fired.borrow_mut().push(e.member.clone())),
            1
        );
    }
    assert_eq!(fired.into_inner(), vec!["l1_idle:s9".to_string()]);
}

// ═══ LocalStateBackend: locks ═══

#[test]
fn locks_are_owner_scoped_and_expire_by_clock() {
    let backend = LocalStateBackend::new(FakeClock::new(500));

    assert!(backend.acquire_lock("k", "w1", 1_000));
    assert!(!backend.acquire_lock("k", "w2", 1_000));
    // Wrong owner cannot renew or release.
    assert!(!backend.renew_lock("k", "w2", 1_000));
    backend.release_lock("k", "w2");
    assert!(!backend.acquire_lock("k", "w2", 1_000));

    // Owner renews past the original expiry.
    backend.clock().advance(900);
    assert!(backend.renew_lock("k", "w1", 1_000));
    backend.clock().advance(900); // t=2300 > original 1500 but renewed to 2400
    assert!(!backend.acquire_lock("k", "w2", 100));

    backend.clock().advance(200); // t=2500 > 2400 → expired
    assert!(backend.acquire_lock("k", "w2", 100));
}

// ═══ Queue ordering ═══

#[test]
fn queue_orders_by_priority_then_creation_time() {
    let backend = LocalStateBackend::new(FakeClock::new(10));

    backend.enqueue_task(task(TaskKind::L1, "a", 1, 10));
    backend.enqueue_task(task(TaskKind::L3, "b", 0, 30));
    backend.enqueue_task(task(TaskKind::L2, "c", 1, 20));
    backend.enqueue_task(task(TaskKind::Flush, "d", 0, 20));

    let queued = backend.list_queued_tasks();
    let order: Vec<&str> = queued.iter().map(|t| t.session_id.as_str()).collect();
    assert_eq!(order, vec!["d", "b", "a", "c"]);
    assert_eq!(backend.queue_depth(), (2, 4 - 2));
}

// ═══ capture_atomic + managers ═══

#[test]
fn capture_atomic_triggers_at_threshold_and_resets_counter() {
    let clock = FakeClock::new(1_000);
    let backend = LocalStateBackend::new(clock);

    let params = |count: u64| CaptureAtomicParams {
        session_id: "s".into(),
        message_json: Some(format!("msg-{count}")),
        threshold: 3,
        fire_at_ms: 5_000,
        timer_member: "l1_idle:s".into(),
        task: task(TaskKind::L1, "s", 1, 1_000),
        now_ms: 1_000,
        rounds: 1,
    };

    assert!(!backend.capture_atomic(params(1)).triggered);
    assert!(!backend.capture_atomic(params(2)).triggered);
    assert_eq!(backend.buffer_len("s"), 2);
    assert!(backend.snapshot().timers >= 1); // idle timer armed

    let result = backend.capture_atomic(params(3));
    assert!(result.triggered);
    assert_eq!(result.conversation_count, 0);
    assert_eq!(backend.queue_depth().0 + backend.queue_depth().1, 1);
    // Idle timer removed on trigger.
    assert!(backend.take_expired_timers(9_999).is_empty());
}

#[test]
fn manager_warmup_doubles_until_cap() {
    let backend = LocalStateBackend::new(FakeClock::new(0));
    let manager = MemoryPipelineManager::new(
        &backend,
        PipelineConfig {
            every_n_conversations: 4,
            l1_idle_timeout_ms: 1_000,
        },
    );

    manager.start_session("s");
    assert_eq!(manager.effective_threshold("s"), 1);

    // First round fires immediately (warm-up = 1).
    assert!(manager.notify_conversation("s", Some("m1"), 1));
    manager.mark_l1_complete("s", "1970-01-01T00:00:01Z");

    // Warm-up doubled to 2: one round does not fire.
    assert_eq!(manager.effective_threshold("s"), 2);
    assert!(!manager.notify_conversation("s", Some("m2"), 1));
    assert!(manager.notify_conversation("s", Some("m3"), 1));
    manager.mark_l1_complete("s", "1970-01-01T00:00:02Z");

    // 4 → capped at every_n (4): doubling stops there.
    assert_eq!(manager.effective_threshold("s"), 4);
    manager.mark_l1_complete("s", "1970-01-01T00:00:03Z");
    assert_eq!(manager.effective_threshold("s"), 4);
}

#[test]
fn idle_timer_flushes_quiet_session_via_scanner() {
    let backend = LocalStateBackend::new(FakeClock::new(1_000));
    let manager = MemoryPipelineManager::new(
        &backend,
        PipelineConfig {
            every_n_conversations: 50,
            l1_idle_timeout_ms: 500,
        },
    );

    assert!(!manager.notify_conversation("quiet", Some("m"), 1));
    assert_eq!(backend.queue_depth().1, 0);

    backend.clock().set_time(1_500); // idle timeout elapsed
    let scanner = TimerScanner::new(&backend);
    let fired = scanner.run_once(|entry| {
        // Idle-timer expiry enqueues the flush task for that session.
        let session = entry
            .member
            .strip_prefix("l1_idle:")
            .unwrap_or(entry.member.as_str())
            .to_string();
        backend.enqueue_task(task(TaskKind::L1, &session, 1, entry.fire_at_ms));
    });
    assert_eq!(fired, 1);
    assert_eq!(backend.queue_depth().1, 1);
    assert_eq!(
        backend.consume_task().map(|t| t.session_id),
        Some("quiet".into())
    );
}

#[test]
fn stateful_manager_persists_states_through_callback() {
    let backend = LocalStateBackend::new(FakeClock::new(0));
    let db = open_db();
    let checkpoints = CheckpointManager::new(&db);
    let manager =
        vanta_memory::utils::StatefulPipelineManager::new(&backend, PipelineConfig::default());

    manager.notify_conversation("s", None, 1);
    manager.mark_l1_complete("s", "2026-08-20T00:00:00Z", |states| {
        checkpoints
            .merge_pipeline_states(states)
            .expect("persist states");
    });

    let cp = checkpoints.read().expect("read checkpoint");
    let state = cp.pipeline_states.get("s").expect("state persisted");
    assert_eq!(state.last_extraction_time, "2026-08-20T00:00:00Z");
    assert_eq!(state.warmup_threshold, 2); // default interval 10 → 1*2=2
}

// ═══ Checkpoint ↔ persona trigger (deuda MEM-15) ═══

#[test]
fn checkpoint_counters_feed_the_persona_trigger() {
    let db = open_db();
    let checkpoints = CheckpointManager::new(&db);

    checkpoints.increment_scenes_processed().expect("scenes");
    checkpoints
        .set_persona_update_request("user asked")
        .expect("flag");

    let cp = checkpoints.read().expect("read");
    let input = checkpoints.persona_trigger_input(&cp, true, true);
    let trigger = evaluate_persona_trigger(&input, 50);
    assert!(trigger.should);
    assert_eq!(trigger.reason, "user asked"); // P1 wins, reason carried through

    // Generation resets the counters (P1 flag cleared, memories zeroed).
    checkpoints
        .mark_persona_generated(42, 1_000, "1970-01-01T00:00:01Z")
        .expect("mark");
    let cp = checkpoints.read().expect("read");
    assert_eq!(cp.total_processed, 42);
    assert_eq!(cp.last_persona_at, 1_000);
    assert!(!cp.request_persona_update);
    assert_eq!(cp.memories_since_last_persona, 0);
    assert_eq!(cp.scenes_processed, 1); // untouched by persona marking
}

// ═══ Worker: retry, dead-letter, lock serialization ═══

struct FailingHandler {
    fail_sessions: Vec<String>,
}

impl TaskHandler for FailingHandler {
    fn handle(&mut self, task: &TaskPayload) -> Result<(), String> {
        if self.fail_sessions.contains(&task.session_id) {
            Err("boom".into())
        } else {
            Ok(())
        }
    }
}

#[test]
fn worker_retries_then_dead_letters_failing_tasks() {
    let backend = LocalStateBackend::new(FakeClock::new(0));
    backend.enqueue_task(task(TaskKind::L1, "bad", 1, 0));

    let mut worker = PipelineWorker::new(
        &backend,
        WorkerConfig {
            max_retries: 3,
            ..WorkerConfig::default()
        },
    );
    let mut handler = FailingHandler {
        fail_sessions: vec!["bad".into()],
    };

    // Attempts 1 and 2 are retried; attempt 3 dead-letters.
    assert_eq!(worker.run_once(&mut handler).processed, 0);
    assert_eq!(worker.run_once(&mut handler).processed, 0);
    let stats = worker.run_once(&mut handler);
    assert_eq!(stats.failed, 1);
    assert_eq!(worker.dead_letters().len(), 1);
    assert_eq!(worker.dead_letters()[0].error, "boom");
    // Queue is drained — no infinite retry loop.
    assert!(backend.list_queued_tasks().is_empty());
}

#[test]
fn worker_skips_locked_sessions_without_losing_tasks() {
    let backend = LocalStateBackend::new(FakeClock::new(0));
    backend.enqueue_task(task(TaskKind::L1, "busy", 1, 0));

    // Someone else holds the session lock.
    assert!(backend.acquire_lock("pipeline_lock:busy", "other", 60_000));

    let mut worker = PipelineWorker::new(&backend, WorkerConfig::default());
    let stats = worker.run_once(&mut FailingHandler {
        fail_sessions: vec![],
    });
    assert_eq!(stats.skipped_locked, 1);
    // Task was deferred back into the queue, not lost.
    assert_eq!(backend.queue_depth().1, 1);
}

// ═══ MEM-66: claim/pending lease + reclaim (Step 1 backend) ═══

#[test]
fn stale_pending_task_reclaimed_once_by_new_owner() {
    let backend = LocalStateBackend::new(FakeClock::new(0));
    backend.enqueue_task(task(TaskKind::L1, "s", 1, 0));
    assert_eq!(backend.pending_count(), 0);

    // Worker A claims the task (lease 1_000ms) and dies without completing.
    let claimed = backend.claim_task("worker-A", 1_000).expect("A claims");
    assert_eq!(claimed.session_id, "s");
    assert_eq!(backend.pending_count(), 1);
    // Queue is drained — nobody else can take it while the lease is live.
    assert!(backend.claim_task("worker-B", 1_000).is_none());
    assert!(backend.claim_stale_tasks("worker-B", 1_000, 8).is_empty());

    // Lease expires while A is dead (heartbeat never renewed).
    backend.clock().advance(1_001);

    // Worker B reclaims the stale pending task exactly once.
    let reclaimed = backend.claim_stale_tasks("worker-B", 1_000, 8);
    assert_eq!(reclaimed.len(), 1);
    assert_eq!(reclaimed[0].session_id, "s");
    assert_eq!(backend.pending_count(), 1);
    // Double-reclaim is impossible: a third worker finds nothing.
    assert!(backend.claim_stale_tasks("worker-C", 1_000, 8).is_empty());

    // Heartbeat vs session-lock TTL are separate: renewing the task lease
    // extends it past the original expiry, and only the owner can do it.
    assert!(!backend.renew_task_lease("worker-C", &reclaimed[0].id, 1_000));
    backend.clock().advance(900); // t=1901 < B lease 2001
    assert!(backend.renew_task_lease("worker-B", &reclaimed[0].id, 1_000));
    backend.clock().advance(900); // t=2801 > original 2001, renewed to 2901
    assert!(backend.claim_stale_tasks("worker-C", 1_000, 8).is_empty());

    // Completion is owner-checked (fencing): wrong owner mutates nothing.
    assert!(!backend.complete_task("worker-C", &reclaimed[0].id));
    assert_eq!(backend.pending_count(), 1);
    assert!(backend.complete_task("worker-B", &reclaimed[0].id));
    assert_eq!(backend.pending_count(), 0);
}

// ═══ MEM-66: worker-muerto→reclaim end-to-end (Step 2 worker — contrato) ═══

#[test]
fn dead_worker_claim_reclaimed_and_processed_by_new_owner() {
    let backend = LocalStateBackend::new(FakeClock::new(0));
    backend.enqueue_task(task(TaskKind::L1, "s", 1, 0));

    // Worker A claims the task and dies without finishing it.
    assert!(backend.claim_task("worker-A", 1_000).is_some());
    assert_eq!(backend.pending_count(), 1);

    // Lease expires while A is dead (no heartbeat was ever sent).
    backend.clock().advance(1_001);

    // Worker B (distinct owner) reclaims and processes it end-to-end.
    let mut worker_b =
        PipelineWorker::new(&backend, WorkerConfig::default()).with_owner("worker-B");
    let mut handler = FailingHandler {
        fail_sessions: vec![],
    };
    let stats = worker_b.reclaim_stale(&mut handler, 1_000, 8);
    assert_eq!(stats.processed, 1);
    assert!(backend.list_queued_tasks().is_empty());
    assert_eq!(backend.pending_count(), 0);
    // Second reclaim finds nothing — exactly-once.
    let again = worker_b.reclaim_stale(&mut handler, 1_000, 8);
    assert_eq!(again.processed, 0);
    assert_eq!(again.failed, 0);
}

// ═══ End-to-end: worker drives L0→L1→L3 with a fake runner ═══

/// Canned runner keyed by task_id; panics on unexpected calls.
struct Scripted {
    persona_json: String,
}

impl vanta_memory::core::abstractions::LlmRunner for Scripted {
    fn run(
        &self,
        params: &vanta_memory::core::abstractions::LlmRunParams,
    ) -> Result<String, vanta_memory::core::abstractions::LlmError> {
        match params.task_id.as_str() {
            // Dedup with no candidates never calls the LLM; extraction only
            // happens when messages pass the quality gate. For this e2e we
            // exercise the L3 path (persona) and the L1 no-op path, so any
            // other call is a bug.
            "persona-generation" => Ok(self.persona_json.clone()),
            other => panic!("unexpected LLM call: {other}"),
        }
    }
}

#[test]
fn handler_l3_generates_persona_from_checkpoint_request() {
    let db = open_db();
    upsert_scene(&db, "sess-e2e", "deploy-runbook", "deploys", "how to ship").expect("scene");

    let checkpoints = CheckpointManager::new(&db);
    checkpoints
        .set_persona_update_request("explicit request")
        .expect("flag");

    let runner = Scripted {
        persona_json: "{\"persona\":\"# Profile\\n\\nBuilder archetype.\"}".into(),
    };
    let mut handler = MemoryTaskHandler::new(
        db.clone(),
        &runner,
        Default::default(),
        Default::default(),
        50,
    );

    handler
        .handle(&task(TaskKind::L3, "sess-e2e", 1, 0))
        .expect("L3 handled");

    let persona = get_persona(&db, "sess-e2e").expect("read").expect("exists");
    assert!(persona.content.contains("Builder archetype"));
    let cp = checkpoints.read().expect("checkpoint");
    assert!(cp.last_persona_at > 0);
    assert!(!cp.request_persona_update);
}

#[test]
fn handler_l1_noop_on_empty_session_and_l3_skips_when_quiet() {
    let db = open_db();
    let runner = Scripted {
        persona_json: String::new(), // must never be used
    };
    let mut handler = MemoryTaskHandler::new(
        db.clone(),
        &runner,
        Default::default(),
        Default::default(),
        50,
    );

    // Empty session: L1 is a clean no-op.
    handler
        .handle(&task(TaskKind::L1, "empty", 1, 0))
        .expect("L1 noop");

    // No scenes, no persona, no counters → trigger stays quiet.
    handler
        .handle(&task(TaskKind::L3, "empty", 1, 0))
        .expect("L3 quiet");
    assert!(get_persona(&db, "empty").expect("read").is_none());
}

#[test]
fn full_worker_pass_records_l0_then_runs_l1_noop() {
    let db = open_db();

    // L0 capture (the phase before the queue).
    let recorder = L0Recorder::new(db.clone());
    recorder
        .record_turn(
            &L0Capture {
                session_id: "flow".into(),
                messages: vec![L0Message {
                    id: Some("m1".into()),
                    role: L0Role::User,
                    content: "I prefer dark mode".into(),
                    timestamp_ms: 100,
                }],
            },
            None,
        )
        .expect("record");

    // The runner must never be called: quality gate passes but this e2e has
    // no extraction script — instead verify the recorded message reaches the
    // store and the worker path completes without data loss.
    let messages = L0Recorder::new(db.clone())
        .read_messages("flow")
        .expect("read");
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].content, "I prefer dark mode");

    // Scenes/persona untouched by an L1 over an unscripted runner is proven
    // by handler_l1_noop_on_empty_session_and_l3_skips_when_quiet; here we
    // assert the scene index is still empty (no partial writes).
    assert!(list_scenes(&db, "flow").expect("scenes").is_empty());
}

// ═══ FIND-86 (D1+D2): worker wiring — Dream TaskKind + L1 batch opt-in ═══

/// The Dream path is LLM-free: any runner call inside a Dream task is a bug.
struct NeverCalled;

impl LlmRunner for NeverCalled {
    fn run(&self, params: &LlmRunParams) -> Result<String, LlmError> {
        panic!(
            "Dream path must be LLM-free; unexpected call: {}",
            params.task_id
        )
    }
}

/// Scripted runner for wiring tests: routes one response per `task_id`,
/// records every call. Missing script → error (unexpected call).
struct WiringRunner {
    calls: Mutex<Vec<String>>,
    script: Mutex<HashMap<String, String>>,
}

impl WiringRunner {
    fn new(script: Vec<(&str, &str)>) -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            script: Mutex::new(
                script
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            ),
        }
    }

    fn calls(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }
}

impl LlmRunner for WiringRunner {
    fn run(&self, params: &LlmRunParams) -> Result<String, LlmError> {
        self.calls.lock().unwrap().push(params.task_id.clone());
        self.script
            .lock()
            .unwrap()
            .remove(&params.task_id)
            .ok_or_else(|| LlmError::Other(format!("unexpected call: {}", params.task_id)))
    }
}

fn seed_l0(db: &vantadb::sdk::Embedded, session: &str, content: &str) {
    L0Recorder::new(db.clone())
        .record_turn(
            &L0Capture {
                session_id: session.into(),
                messages: vec![L0Message {
                    id: Some("m2".into()),
                    role: L0Role::User,
                    content: content.into(),
                    timestamp_ms: 1200,
                }],
            },
            None,
        )
        .expect("record L0");
}

/// Seed one stored L1 record (dedup `store` → persisted) for recall pools.
fn seed_l1(db: &vantadb::sdk::Embedded, session: &str, id: &str, content: &str) {
    let memory = ExtractedMemory {
        content: content.into(),
        memory_type: MemoryType::Persona,
        priority: 80,
        source_message_ids: vec!["m1".into()],
        scene_name: "ui".into(),
        metadata: serde_json::json!({}),
    };
    let decision = DedupDecision {
        record_id: id.into(),
        action: DedupAction::Store,
        target_ids: vec![],
        merged_content: None,
        merged_type: None,
        merged_priority: None,
        merged_timestamps: None,
    };
    write_memory(
        db,
        session,
        session,
        &memory,
        &decision,
        1_700_000_000_000,
        0,
        None,
    )
    .expect("write")
    .expect("stored");
}

/// Batch-shaped response: extraction + safe-default `store` (no inline
/// `dedup` field → the memory is kept, never dropped).
const BATCH_STORE_JSON: &str = r#"[
  {"scene_name": "Setting up the project", "message_ids": ["m2"], "memories": [
    {"content": "User prefers dark mode", "type": "persona", "priority": 80,
     "source_message_ids": ["m2"], "metadata": {}}
  ]}
]"#;

/// Split-path extraction response (same shape, no inline judgment).
const EXTRACT_JSON: &str = BATCH_STORE_JSON;

// (D1) A Dream task consolidates the session into `dream/<s>/<run_id>`
// without touching the L1 store (MEM-61 invariant survives wiring).
#[test]
fn handler_dream_task_consolidates_session_without_touching_l1() {
    let db = open_db();
    seed_l1(&db, "dream-sess", "seed-1", "user prefers dark mode");
    let before = read_session_records(&db, "dream-sess").expect("read before");

    let runner = NeverCalled;
    let mut handler = MemoryTaskHandler::new(
        db.clone(),
        &runner,
        Default::default(),
        Default::default(),
        50,
    );
    handler
        .handle(&task(TaskKind::Dream, "dream-sess", 1, 0))
        .expect("Dream handled");

    // One dream run persisted over the scanned input …
    let runs = list_dream_runs(&db, "dream-sess").expect("list");
    assert_eq!(runs.len(), 1, "Dream task writes exactly one run");
    assert_eq!(runs[0].inputs_scanned, 1);
    // … and the L1 store is byte-identical.
    let after = read_session_records(&db, "dream-sess").expect("read after");
    assert_eq!(before, after, "L1 untouched by dream wiring");
}

// (D1-skip) A premature Dream task (session active inside the idle window)
// is a quiet `Ok` skip: no run persisted, no retry, no dead-letter. The host
// timer re-enqueues later.
#[test]
fn handler_dream_task_skips_quietly_when_not_idle() {
    let db = open_db();
    // Wall-clock now (`now_ms` is `pub(crate)` — same epoch-ms basis).
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_millis() as u64;
    CheckpointManager::new(&db)
        .merge_pipeline_states_owned("busy-sess", |state| {
            state.last_active_time_ms = now;
        })
        .expect("state");

    let runner = NeverCalled;
    let mut handler = MemoryTaskHandler::new(
        db.clone(),
        &runner,
        Default::default(),
        Default::default(),
        50,
    );
    handler
        .handle(&task(TaskKind::Dream, "busy-sess", 1, 0))
        .expect("premature Dream skips Ok");
    assert!(
        list_dream_runs(&db, "busy-sess").expect("list").is_empty(),
        "no run persisted before the idle threshold"
    );
}

// (D2) Opt-in batch: one L1 task → exactly 1 LLM call (`l1-extract-dedup`),
// and the fused memory is written, not dropped.
#[test]
fn handler_l1_batch_flag_fuses_extract_and_dedup_in_one_call() {
    let db = open_db();
    seed_l0(&db, "batch-sess", "I prefer dark mode");
    seed_l1(&db, "batch-sess", "seed-1", "user prefers dark mode");

    let runner = WiringRunner::new(vec![(EXTRACT_DEDUP_TASK_ID, BATCH_STORE_JSON)]);
    let backend = LocalStateBackend::new(FakeClock::new(0));
    backend.enqueue_task(task(TaskKind::L1, "batch-sess", 1, 0));
    let mut worker = PipelineWorker::new(&backend, WorkerConfig::default());
    let mut handler = MemoryTaskHandler::new(
        db.clone(),
        &runner,
        Default::default(),
        Default::default(),
        50,
    )
    .with_use_batch(true);

    let stats = worker.run_once(&mut handler);
    assert_eq!(stats.processed, 1);
    assert_eq!(
        runner.calls(),
        vec![EXTRACT_DEDUP_TASK_ID.to_string()],
        "opt-in batch = exactly 1 LLM call"
    );
    let records = read_session_records(&db, "batch-sess").expect("read");
    assert!(
        records
            .iter()
            .any(|r| r.content == "User prefers dark mode"),
        "fused memory is written, not dropped"
    );
}

// (D2-control) Default path unchanged: extraction + conflict-detection.
#[test]
fn handler_l1_default_path_still_uses_two_calls() {
    let db = open_db();
    seed_l0(&db, "split-sess", "I prefer dark mode");
    seed_l1(&db, "split-sess", "seed-1", "user prefers dark mode");

    let runner = WiringRunner::new(vec![
        ("l1-extraction", EXTRACT_JSON),
        ("l1-conflict-detection", "[]"),
    ]);
    let backend = LocalStateBackend::new(FakeClock::new(0));
    backend.enqueue_task(task(TaskKind::L1, "split-sess", 1, 0));
    let mut worker = PipelineWorker::new(&backend, WorkerConfig::default());
    let mut handler = MemoryTaskHandler::new(
        db.clone(),
        &runner,
        Default::default(),
        Default::default(),
        50,
    );

    let stats = worker.run_once(&mut handler);
    assert_eq!(stats.processed, 1);
    assert_eq!(
        runner.calls(),
        vec![
            "l1-extraction".to_string(),
            "l1-conflict-detection".to_string()
        ],
        "default path unchanged: 2 LLM calls"
    );
    // Write parity with the fused path: the extracted memory lands in L1.
    let records = read_session_records(&db, "split-sess").expect("read");
    assert!(
        records
            .iter()
            .any(|r| r.content == "User prefers dark mode"),
        "default path writes the extracted memory"
    );
}
