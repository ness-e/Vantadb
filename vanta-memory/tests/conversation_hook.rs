// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
#![cfg(feature = "http-server")]
//! MEM-55 — D19 contract tests for the `/conversation/add` → memory pipeline
//! bridge.
//!
//! The HTTP half (POST → thread saved → trigger fired → 201 even when the
//! trigger fails) is covered by core tests (`cli_server.rs`); these tests
//! cover the other half of the chain: trigger → L0 capture + queued L1 task →
//! MEM-16 worker pass → memories visible under `l1/<session>`.

use std::sync::Arc;
use vanta_memory::core::abstractions::{LlmError, LlmRunParams, LlmRunner};
use vanta_memory::core::conversation::L0Recorder;
use vanta_memory::core::persona::get_persona;
use vanta_memory::core::record::read_session_records;
use vanta_memory::services::conversation_hook::{run_bridge_pass, HttpCaptureBridge};
use vanta_memory::utils::local_backend::LocalStateBackend;
use vanta_memory::utils::managed_timer::SystemClock;
use vantadb::cli_server::ConversationTrigger;
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
/// `tests/e2e_flow.rs`): extraction + dedup conflict judgment.
struct ScriptedRunner {
    fail_l1: bool,
}

impl LlmRunner for ScriptedRunner {
    fn run(&self, params: &LlmRunParams) -> Result<String, LlmError> {
        match params.task_id.as_str() {
            "l1-extraction" if self.fail_l1 => Err(LlmError::Timeout),
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

/// L1 extraction with enough memories to cross the L1→L2 cascade threshold
/// (`MemoryTaskHandler::auto_l2_threshold`, default 5) — the threshold is part
/// of the contract under test, so it is exercised, not bypassed.
const FIVE_MEMORIES_JSON: &str = r#"[
  {"scene_name": "Ways of working", "message_ids": ["m1"], "memories": [
    {"content": "User prefers dark mode", "type": "preference", "priority": 80, "source_message_ids": ["m1"]},
    {"content": "User edits in vim", "type": "preference", "priority": 70, "source_message_ids": ["m1"]},
    {"content": "User works at night", "type": "preference", "priority": 60, "source_message_ids": ["m1"]},
    {"content": "User writes in Spanish", "type": "preference", "priority": 50, "source_message_ids": ["m1"]},
    {"content": "User reviews on Fridays", "type": "preference", "priority": 40, "source_message_ids": ["m1"]}
  ]}
]"#;

/// L2 scene extraction: one CREATE candidate (shape per `scene_extractor`).
const SCENE_JSON: &str = r#"[{"scene_name": "ways_of_working", "summary": "How the user works", "content": "Night owl who edits in vim.", "merge_sources": []}]"#;

/// L3 persona generation (shape per `pipeline_manager.rs`).
const PERSONA_JSON: &str = "{\"persona\":\"# Profile\\n\\nNight owl builder.\"}";

/// Runner covering every stage the documented hook pattern reaches:
/// L1 extraction + dedup, L2 scene extraction, L3 persona (shapes copied from
/// `tests/e2e_flow.rs`). Any other task id is a bug (panics), so the test also
/// pins *which* LLM calls the pass makes.
struct FullCycleRunner;

impl LlmRunner for FullCycleRunner {
    fn run(&self, params: &LlmRunParams) -> Result<String, LlmError> {
        match params.task_id.as_str() {
            "l1-extraction" => Ok(FIVE_MEMORIES_JSON.to_string()),
            // Every judged record gets an explicit `store`; an absent one
            // defaults to store anyway (`parse_batch_result`).
            "l1-conflict-detection" => {
                let judged = params
                    .prompt
                    .split("NEW MEMORIES TO JUDGE")
                    .nth(1)
                    .unwrap_or(&params.prompt);
                let decisions: Vec<String> = judged
                    .match_indices("\"record_id\": \"")
                    .map(|(i, m)| {
                        judged[i + m.len()..]
                            .split('"')
                            .next()
                            .unwrap_or("m_0")
                            .to_string()
                    })
                    .map(|id| format!(r#"{{"record_id": "{id}", "action": "store"}}"#))
                    .collect();
                Ok(format!("[{}]", decisions.join(",")))
            }
            "l2-scene-extraction" => Ok(SCENE_JSON.to_string()),
            "persona-generation" => Ok(PERSONA_JSON.to_string()),
            other => panic!("unexpected LLM call: {other}"),
        }
    }
}

#[test]
fn bridge_pass_reaches_l3_and_writes_the_persona() {
    let db = open_db();
    let queue = Arc::new(LocalStateBackend::new(SystemClock));
    let bridge = HttpCaptureBridge::new(db.clone(), queue.clone());

    // Exactly the documented pattern (`conversation_hook.rs` module docs):
    // wire the bridge, then drive the pipeline with `run_bridge_pass`.
    bridge.trigger(7, "user", "Here is how I work").unwrap();
    let stats = run_bridge_pass(&queue, db.clone(), &FullCycleRunner);
    assert_eq!(
        stats.worker.processed, 3,
        "one pass must drain L1 → L2 → L3 (batch of 8), got {stats:?}"
    );
    assert_eq!(queue.queue_depth(), (0, 0), "the pass must drain the queue");

    // MEM-71: before the fix this pass built a bare handler (no queue wiring,
    // `trigger_every_n = usize::MAX`), so L2 was never enqueued and the
    // documented pattern produced L1 only. The persona is the observable proof
    // that L1 → L2 → L3 actually ran end to end.
    let persona = get_persona(&db, "7")
        .expect("persona read")
        .expect("the bridge pass must reach L3 and write the persona");
    assert!(
        persona.content.contains("Night owl builder."),
        "{persona:?}"
    );
}

#[test]
fn bridge_enqueues_task_and_worker_writes_l1_memories() {
    let db = open_db();
    let queue = Arc::new(LocalStateBackend::new(SystemClock));
    let bridge = HttpCaptureBridge::new(db.clone(), queue.clone());

    // POST /conversation/add already returned 201 (core-tested); the host
    // wired this bridge as the trigger.
    bridge.trigger(42, "user", "I prefer dark mode").unwrap();
    assert_eq!(
        queue.queue_depth(),
        (0, 1),
        "extraction task must be queued"
    );

    let stats = run_bridge_pass(&queue, db.clone(), &ScriptedRunner { fail_l1: false });
    assert_eq!(stats.worker.processed, 1);
    assert_eq!(queue.queue_depth(), (0, 0));

    // Memories appear in l1/<session> where session == decimal thread id.
    let records = read_session_records(&db, "42").expect("read l1 records");
    assert!(
        records.iter().any(|r| r.content.contains("dark mode")),
        "expected extracted memory in l1/42, got: {records:?}"
    );
}

#[test]
fn extraction_failure_keeps_l0_and_recovers_on_healthy_retry() {
    let db = open_db();
    let queue = Arc::new(LocalStateBackend::new(SystemClock));
    let bridge = HttpCaptureBridge::new(db.clone(), queue.clone());
    bridge.trigger(7, "user", "I prefer dark mode").unwrap();

    // Failing extraction: task goes back to the queue (worker retry), nothing
    // partial written, L0 capture intact — and the HTTP side never sees it
    // (P4, asserted by the core test conversation_add_trigger_failure_...).
    run_bridge_pass(&queue, db.clone(), &ScriptedRunner { fail_l1: true });
    assert!(read_session_records(&db, "7").unwrap().is_empty());
    let messages = L0Recorder::new(db.clone())
        .read_messages("7")
        .expect("read l0");
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].content, "I prefer dark mode");
    assert_eq!(queue.queue_depth(), (0, 1), "task must survive for retry");

    // Healthy runner recovers the full flow on the next pass.
    let stats = run_bridge_pass(&queue, db.clone(), &ScriptedRunner { fail_l1: false });
    assert_eq!(stats.worker.processed, 1);
    let records = read_session_records(&db, "7").expect("read l1 records");
    assert!(records.iter().any(|r| r.content.contains("dark mode")));
}

#[test]
fn invalid_role_is_best_effort_error_without_side_effects() {
    let db = open_db();
    let queue = Arc::new(LocalStateBackend::new(SystemClock));
    let bridge = HttpCaptureBridge::new(db.clone(), queue.clone());

    let err = bridge.trigger(9, "tool", "noise").unwrap_err();
    assert!(err.contains("invalid conversation role"), "got: {err}");
    assert_eq!(queue.queue_depth(), (0, 0), "nothing enqueued");
    assert!(L0Recorder::new(db.clone())
        .read_messages("9")
        .unwrap()
        .is_empty());
}

// CODEX-132: concurrent back-to-back triggers (same-ms race) must lose no
// messages — the L0 cursor+sequence tie-break keeps every capture.
#[test]
fn concurrent_same_ms_triggers_lose_no_messages() {
    use std::sync::Barrier;

    let db = open_db();
    let queue = Arc::new(LocalStateBackend::new(SystemClock));
    let bridge = Arc::new(HttpCaptureBridge::new(db.clone(), queue.clone()));

    const N: usize = 8;
    let barrier = Arc::new(Barrier::new(N));
    let handles: Vec<_> = (0..N)
        .map(|i| {
            let bridge = bridge.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait(); // maximize same-ms collision odds
                bridge
                    .trigger(99, "user", &format!("concurrent message {i}"))
                    .expect("trigger");
            })
        })
        .collect();
    for handle in handles {
        handle.join().expect("worker thread");
    }

    let messages = L0Recorder::new(db).read_messages("99").expect("read l0");
    assert_eq!(
        messages.len(),
        N,
        "0 messages may be lost, got: {messages:?}"
    );
    for i in 0..N {
        assert!(
            messages
                .iter()
                .any(|m| m.content == format!("concurrent message {i}")),
            "missing message {i} in {messages:?}"
        );
    }
}
