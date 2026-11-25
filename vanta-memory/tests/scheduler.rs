// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! WIRE-15 — D19 contract tests for the scheduler PASS (`run_pass`).
//!
//! The pass is the pull-based reuse of the MEM-16 pieces: expired `l1_idle:`
//! timers → L1 tasks + one worker pass + stale-claim reclaim. All time is
//! driven by a [`FakeClock`] — deterministic, zero sleeps. The Tokio loop
//! helper (feature `http-server`) is covered in `tests/scheduler_loop.rs`.

use vanta_memory::core::abstractions::{LlmError, LlmRunParams, LlmRunner, PersonaTriggerPriority};
use vanta_memory::core::conversation::{L0Capture, L0Message, L0Recorder, L0Role};
use vanta_memory::core::persona::{evaluate_persona_trigger, PersonaTriggerInput};
use vanta_memory::core::record::read_session_records;
use vanta_memory::core::state::{TaskKind, TaskPayload};
use vanta_memory::services::scheduler::{run_pass, PERSONA_TRIGGER_EVERY_N};
use vanta_memory::utils::checkpoint::CheckpointManager;
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
            "l2-scene-extraction" => Ok("[]".to_string()),
            _ => Ok("[]".to_string()),
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

// ═══ pass: L3 persona cadence (MEM-71) ═══

/// The cadence the pass ships, pinned to the number — not to "fires someday".
/// `usize::MAX` (the old bridge sentinel) is the failure this test forbids: the
/// L3 persona would never regenerate and no host could tell.
#[test]
fn persona_cadence_is_fifty_memories_and_p4_fires_exactly_at_it() {
    assert_eq!(
        PERSONA_TRIGGER_EVERY_N, 50,
        "the scheduler cadence is a product decision, not a tuning knob: pin it"
    );

    // A healthy, already-generated persona: cold start / recovery / first scene
    // cannot fire, so only P4 decides.
    let input_at = |memories: usize| PersonaTriggerInput {
        request_persona_update: false,
        request_reason: None,
        scenes_processed: 2,
        memories_since_last_persona: memories,
        has_scene_blocks: true,
        previously_generated: true,
        has_persona_body: true,
    };

    // One memory short: regenerating here is the "garbage axioms" failure mode
    // — a persona rewritten with no new material.
    let before = evaluate_persona_trigger(
        &input_at(PERSONA_TRIGGER_EVERY_N - 1),
        PERSONA_TRIGGER_EVERY_N,
    );
    assert!(!before.should, "{before:?}");
    assert_eq!(before.priority, None);

    // Exactly at the threshold: P4, once.
    let at_n =
        evaluate_persona_trigger(&input_at(PERSONA_TRIGGER_EVERY_N), PERSONA_TRIGGER_EVERY_N);
    assert!(at_n.should, "{at_n:?}");
    assert_eq!(at_n.priority, Some(PersonaTriggerPriority::P4MemoryCount));
}

/// Minimal runner for the L3 cadence wiring test: the queue holds only L3
/// tasks, so `persona-generation` is the only reachable call and it must
/// return the output contract. Any other id is a bug (panics).
struct PersonaOnlyRunner;

impl LlmRunner for PersonaOnlyRunner {
    fn run(&self, params: &LlmRunParams) -> Result<String, LlmError> {
        match params.task_id.as_str() {
            "persona-generation" => {
                Ok(r##"{"persona":"# Profile\n\nNight owl builder."}"##.to_string())
            }
            other => panic!("unexpected LLM call in the L3 cadence test: {other}"),
        }
    }
}

fn l3_task(session: &str) -> TaskPayload {
    TaskPayload {
        id: String::new(),
        kind: TaskKind::L3,
        session_id: session.to_string(),
        priority: 1,
        created_at_ms: 0,
        attempts: 0,
    }
}

/// MEM-71, the wiring half: `run_pass` must hand `PERSONA_TRIGGER_EVERY_N` to
/// the handler. Replacing it with `usize::MAX` (the old bridge sentinel) leaves
/// the pure-trigger test above green but must fail here.
///
/// Observable: a successful generation ends in `mark_persona_generated`, which
/// resets `memories_since_last_persona`. With the persona disabled, the pass
/// returns early (`!trigger.should`) and the counter keeps its value — so
/// 49 → 49 is "nothing regenerated" and 50 → 0 is "P4 fired".
#[test]
fn run_pass_regenerates_persona_exactly_at_the_fifty_memory_cadence() {
    let db = open_db();
    let queue = LocalStateBackend::new(FakeClock::new(1_000));
    let checkpoints = CheckpointManager::new(&db);

    // A persona was generated before (P2 cold start off) and no scene block
    // exists yet (`scenes_processed == 0`), so P1/P2/P3 cannot fire: P4 is the
    // only trigger in play.
    checkpoints
        .mark_persona_generated(0, 1_000, "1970-01-01T00:00:01.000Z")
        .expect("seed checkpoint");
    checkpoints
        .add_memories_extracted(PERSONA_TRIGGER_EVERY_N as u64 - 1)
        .expect("seed memories");
    queue.enqueue_task(l3_task("cadence-below"));

    let below = run_pass(&queue, db.clone(), &PersonaOnlyRunner);
    assert_eq!(below.worker.processed, 1);
    assert_eq!(below.worker.failed, 0);
    assert_eq!(
        checkpoints
            .read()
            .expect("read checkpoint")
            .memories_since_last_persona,
        PERSONA_TRIGGER_EVERY_N as u64 - 1,
        "one memory short: regenerating here is the garbage-axioms failure mode"
    );

    // Exactly at the cadence: P4 fires, the pass regenerates and the counter
    // resets. `usize::MAX` would leave it at 50.
    checkpoints
        .add_memories_extracted(1)
        .expect("seed the last memory");
    queue.enqueue_task(l3_task("cadence-at"));

    let at_n = run_pass(&queue, db.clone(), &PersonaOnlyRunner);
    assert_eq!(at_n.worker.processed, 1);
    assert_eq!(at_n.worker.failed, 0);
    assert_eq!(
        checkpoints
            .read()
            .expect("read checkpoint")
            .memories_since_last_persona,
        0,
        "at PERSONA_TRIGGER_EVERY_N memories the pass must regenerate the persona"
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
