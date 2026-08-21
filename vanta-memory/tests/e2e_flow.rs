//! AUD-E2E — full killer-flow integration tests: L0 capture → L1 extraction →
//! dedup → L2 scene write → L3 persona → auto-recall, chained over ONE
//! in-memory VantaDB with a scripted fake `LlmRunner` (trait is not
//! dyn-compatible, so the fake is local — pattern: `tests/pipeline_manager.rs`).
//!
//! Three scenarios:
//! 1. Happy path: every layer produces data and recall injects memories +
//!    persona into prepend/append.
//! 2. Principio 4 degradation: the L1 LLM call fails → no panic, L0 messages
//!    survive untouched, nothing partial is written, recall stays callable,
//!    and a healthy retry recovers the full flow.
//! 3. Flow idempotency: replaying the same turn and re-running L1/L2 does not
//!    duplicate memories or scenes (L0 cursor + dedup skip + scene UPDATE).

use vanta_memory::core::abstractions::{LlmError, LlmRunParams, LlmRunner};
use vanta_memory::core::conversation::{L0Capture, L0Message, L0Recorder, L0Role};
use vanta_memory::core::hooks::{
    perform_auto_recall, AutoRecallParams, RecallConfig, RecallResult,
};
use vanta_memory::core::persona::get_persona;
use vanta_memory::core::record::read_session_records;
use vanta_memory::core::scene::list_scenes;
use vanta_memory::core::state::{TaskKind, TaskPayload};
use vanta_memory::services::pipeline_worker::{MemoryTaskHandler, TaskHandler};
use vantadb::config::VantaConfig;
use vantadb::sdk::VantaEmbedded;
use vantadb::storage::BackendKind;

// ── Scripted LLM responses (JSON shapes copied from the unit-test suites) ──

/// L1 extraction: one scene segment with one memory (shape per
/// `tests/l1_extractor.rs`).
const EXTRACTION_JSON: &str = r#"[
  {"scene_name": "UI Preferences", "message_ids": ["m1"], "memories": [
    {"content": "User prefers dark mode", "type": "preference", "priority": 80, "source_message_ids": ["m1"]}
  ]}
]"#;

/// L2 scene extraction: one CREATE candidate (shape per `scene_extractor`).
const SCENE_JSON: &str = r#"[{"scene_name": "ui_preferences", "summary": "Interface preferences", "content": "The user prefers dark mode across all tools.", "merge_sources": []}]"#;

/// L3 persona generation (shape per `pipeline_manager.rs`).
const PERSONA_JSON: &str = "{\"persona\":\"# Profile\\n\\nNight owl builder.\"}";

/// Fake runner keyed by `task_id`, covering every stage of the pipeline:
/// - `l1-extraction` → canned scene/memories JSON.
/// - `l1-conflict-detection` → echoes a `skip` decision using the REAL
///   transient record_id the pipeline stamped (wall-clock ids are unknowable
///   upfront; pattern proven in `tests/l1_dedup.rs` MergeEcho).
/// - `l2-scene-extraction` / `persona-generation` → canned JSON.
///
/// Any other task id is a bug (panics). `fail_task` simulates an LLM outage
/// at one specific stage.
struct E2eRunner {
    fail_task: Option<&'static str>,
}

impl LlmRunner for E2eRunner {
    fn run(&self, params: &LlmRunParams) -> Result<String, LlmError> {
        if self.fail_task == Some(params.task_id.as_str()) {
            return Err(LlmError::Timeout);
        }
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
                    r#"[{{"record_id": "{record_id}", "action": "skip"}}]"#
                ))
            }
            "l2-scene-extraction" => Ok(SCENE_JSON.to_string()),
            "persona-generation" => Ok(PERSONA_JSON.to_string()),
            other => panic!("unexpected LLM call: {other}"),
        }
    }
}

// ── Fixtures ──

fn open_db() -> VantaEmbedded {
    VantaEmbedded::open_with_config(VantaConfig {
        backend_kind: BackendKind::InMemory,
        ..VantaConfig::default()
    })
    .expect("open in-memory db")
}

fn task(kind: TaskKind, session: &str) -> TaskPayload {
    TaskPayload {
        id: String::new(),
        kind,
        session_id: session.to_string(),
        priority: 1,
        created_at_ms: 0,
        attempts: 0,
    }
}

fn turn(session: &str) -> L0Capture {
    L0Capture {
        session_id: session.to_string(),
        messages: vec![
            L0Message {
                id: Some("m1".into()),
                role: L0Role::User,
                content: "I prefer dark mode".into(),
                timestamp_ms: 100,
            },
            L0Message {
                id: Some("m2".into()),
                role: L0Role::Assistant,
                content: "Noted, switching the theme to dark.".into(),
                timestamp_ms: 200,
            },
        ],
    }
}

/// Record the canonical turn through the real L0 recorder.
fn capture_turn(db: &VantaEmbedded, session: &str) {
    L0Recorder::new(db.clone())
        .record_turn(&turn(session), None)
        .expect("record turn");
}

/// Run L1 → L2 → L3 through the real orchestration handler.
fn run_full_pass(db: &VantaEmbedded, runner: &E2eRunner, session: &str) -> Result<(), String> {
    let mut handler = MemoryTaskHandler::new(
        db.clone(),
        runner,
        Default::default(),
        Default::default(),
        50,
    );
    for kind in [TaskKind::L1, TaskKind::L2, TaskKind::L3] {
        handler.handle(&task(kind, session))?;
    }
    Ok(())
}

/// Run only L1 → L2 (used by the idempotency scenario).
fn run_l1_l2(db: &VantaEmbedded, runner: &E2eRunner, session: &str) -> Result<(), String> {
    let mut handler = MemoryTaskHandler::new(
        db.clone(),
        runner,
        Default::default(),
        Default::default(),
        50,
    );
    for kind in [TaskKind::L1, TaskKind::L2] {
        handler.handle(&task(kind, session))?;
    }
    Ok(())
}

fn recall(db: &VantaEmbedded, session: &str, query: &str) -> Option<RecallResult> {
    perform_auto_recall(
        db,
        AutoRecallParams {
            user_text: query,
            session_key: session,
            isolation: None,
            config: RecallConfig::default(),
        },
    )
    .expect("auto recall must never error")
}

// ═══ 1. Happy path: L0 → L1 → dedup → L2 → L3 → recall ═══

#[test]
fn full_flow_produces_memories_scene_persona_and_recalls_them() {
    let db = open_db();
    let session = "flow-happy";
    let runner = E2eRunner { fail_task: None };

    // L0: capture the conversation turn.
    capture_turn(&db, session);

    // L1 (extract + dedup-store) → L2 (scene write) → L3 (persona).
    run_full_pass(&db, &runner, session).expect("full pass");

    // L1 produced exactly one stored memory.
    let records = read_session_records(&db, session).expect("records");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].content, "User prefers dark mode");

    // L2 wrote the scene block.
    let scenes = list_scenes(&db, session).expect("scenes");
    assert_eq!(scenes.len(), 1);
    assert_eq!(scenes[0].filename, "ui_preferences");

    // L3 generated the persona (cold-start trigger fired after L2).
    let persona = get_persona(&db, session).expect("read").expect("exists");
    assert!(persona.content.contains("Night owl builder."));

    // Recall injects the dynamic memory as prepend and the stable persona
    // as append (same contract as tests/recall.rs).
    let out = recall(&db, session, "what does the user prefer about dark mode?")
        .expect("content to inject");
    let prepend = out.prepend_context.expect("prepend");
    assert!(prepend.starts_with("<relevant-memories>"));
    assert!(prepend.contains("User prefers dark mode"));
    let append = out.append_system_context.expect("append");
    assert!(append.contains("<user-persona>"));
    assert!(append.contains("Night owl builder."));
}

// ═══ 2. Principio 4: L1 LLM failure degrades without losing anything ═══

#[test]
fn l1_failure_degrades_without_data_loss_and_recall_stays_alive() {
    let db = open_db();
    let session = "flow-degraded";
    capture_turn(&db, session);

    // The extraction LLM call times out.
    let failing = E2eRunner {
        fail_task: Some("l1-extraction"),
    };
    let err = run_full_pass(&db, &failing, session).expect_err("L1 must fail, not panic");
    assert!(err.contains("L1 extraction failed"));

    // Nothing was lost and nothing partial was written.
    assert_eq!(
        L0Recorder::new(db.clone())
            .read_messages(session)
            .expect("L0 read")
            .len(),
        2,
        "L0 messages survive the failed pass"
    );
    assert!(read_session_records(&db, session)
        .expect("records")
        .is_empty());
    assert!(list_scenes(&db, session).expect("scenes").is_empty());
    assert!(get_persona(&db, session).expect("read").is_none());

    // Recall still works with what there is: a clean None, never an error.
    assert!(recall(&db, session, "dark mode preferences?").is_none());

    // Recovery: a healthy pass over the SAME L0 data completes the flow.
    let healthy = E2eRunner { fail_task: None };
    run_full_pass(&db, &healthy, session).expect("recovery pass");
    let out = recall(&db, session, "what does the user prefer about dark mode?")
        .expect("recovers after retry");
    assert!(out
        .prepend_context
        .expect("prepend")
        .contains("User prefers dark mode"));
}

// ═══ 3. Idempotency: replaying the turn duplicates nothing ═══

#[test]
fn replayed_turn_does_not_duplicate_memories_or_scenes() {
    let db = open_db();
    let session = "flow-idempotent";
    let runner = E2eRunner { fail_task: None };

    // Same L0 turn twice: the cursor swallows the replay.
    capture_turn(&db, session);
    let replay = L0Recorder::new(db.clone())
        .record_turn(&turn(session), None)
        .expect("replay");
    assert_eq!(replay.recorded_count, 0, "cursor must reject the replay");
    assert_eq!(
        L0Recorder::new(db.clone())
            .read_messages(session)
            .expect("read")
            .len(),
        2
    );

    // First pipeline pass: 1 memory stored, 1 scene created.
    run_l1_l2(&db, &runner, session).expect("first pass");
    assert_eq!(
        read_session_records(&db, session).expect("records").len(),
        1
    );
    assert_eq!(list_scenes(&db, session).expect("scenes").len(), 1);

    // Second pass over the same data: extraction re-runs, dedup judges the
    // near-duplicate as `skip`, and the scene strategy is UPDATE (heat bump),
    // never a second CREATE.
    run_l1_l2(&db, &runner, session).expect("second pass");
    let records = read_session_records(&db, session).expect("records");
    assert_eq!(records.len(), 1, "dedup skip prevents memory duplication");
    let scenes = list_scenes(&db, session).expect("scenes");
    assert_eq!(scenes.len(), 1, "upsert updates in place");
    assert_eq!(
        scenes[0].heat, 2,
        "second pass bumped heat instead of duplicating"
    );
}
