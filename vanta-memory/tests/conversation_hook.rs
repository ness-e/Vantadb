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
use vanta_memory::core::record::read_session_records;
use vanta_memory::services::conversation_hook::{run_bridge_pass, HttpCaptureBridge};
use vanta_memory::utils::local_backend::LocalStateBackend;
use vanta_memory::utils::managed_timer::SystemClock;
use vantadb::cli_server::ConversationTrigger;
use vantadb::config::VantaConfig;
use vantadb::sdk::VantaEmbedded;
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

fn open_db() -> VantaEmbedded {
    VantaEmbedded::open_with_config(VantaConfig {
        backend_kind: BackendKind::InMemory,
        ..VantaConfig::default()
    })
    .expect("open in-memory db")
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
    assert_eq!(stats.processed, 1);
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
    assert_eq!(stats.processed, 1);
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
