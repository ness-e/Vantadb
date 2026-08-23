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

use vanta_memory::context_engine::{
    assemble_with_recall, load_active, save_active, AssembleConfig, ChatMessage, ChatRole,
    CompactionMode, TaskMemory, TokenEstimator, MMD_CONTEXT_MARKER,
};
use vanta_memory::core::abstractions::{LlmError, LlmRunParams, LlmRunner, SceneMeta};
use vanta_memory::core::conversation::{L0Capture, L0Message, L0Recorder, L0Role};
use vanta_memory::core::hooks::{
    perform_auto_recall, AutoRecallParams, RecallConfig, RecallResult,
};
use vanta_memory::core::persona::get_persona;
use vanta_memory::core::record::read_session_records;
use vanta_memory::core::scene::list_scenes;
use vanta_memory::core::state::{TaskKind, TaskPayload};
use vanta_memory::services::pipeline_worker::{
    load_assembled_context, ContextAssemblyConfig, MemoryTaskHandler, TaskHandler,
};
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
        None,
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

// ═══ 4. MEM-37: capture → extract → compress → inject → recall, un flujo ═══

#[test]
fn compress_then_recall_shares_one_budget_end_to_end() {
    let db = open_db();
    let session = "flow-mem37";
    let runner = E2eRunner { fail_task: None };

    // L0 capture + full pipeline pass → real L1 records and L3 persona.
    capture_turn(&db, session);
    run_full_pass(&db, &runner, session).expect("full pass");

    // Real recall over the stored data (MEM-40).
    let recalled =
        recall(&db, session, "what does the user prefer about dark mode?").expect("recall content");
    let prepend = recalled.prepend_context.expect("prepend");
    assert!(prepend.contains("User prefers dark mode"));
    let append = recalled.append_system_context.expect("append");

    // Active MMD persisted through the real store path (MEM-24).
    save_active(
        &db,
        session,
        &TaskMemory {
            meta: SceneMeta {
                created: "2026-08-21T10:00:00.000Z".into(),
                updated: "2026-08-21T10:05:00.000Z".into(),
                summary: "integration task".into(),
                heat: 1,
            },
            content: "current task: integrate offload with recall".into(),
        },
    )
    .expect("save mmd");
    let active = load_active(&db, session).expect("load mmd");

    // Long synthetic chat history that forces aggressive compression; the
    // shared-budget coordinator assembles everything in one pass (MEM-37).
    // Fat units: mild's 10-stub cap can't reach the budget, so aggressive
    // runs and leaves headroom for the injections.
    //
    // The budget is derived from the estimator's own measurement (MEM-43):
    // ~23% of the raw history preserves the mild-fails / aggressive-fits /
    // headroom-fits-MMD ratio under both the chars/3 branch and
    // `precise-tokens` (BPE), instead of pinning a chars/3 magic number.
    let est = TokenEstimator::default();
    let msgs: Vec<ChatMessage> = (0..30)
        .map(|i| ChatMessage::new(ChatRole::User, format!("m{i:02} {}", "y".repeat(600))))
        .collect();
    let budget = (est.estimate_messages(&msgs) * 23 / 100).max(1);
    let out = assemble_with_recall(
        msgs,
        budget,
        &est,
        0,
        &AssembleConfig::default(),
        active,
        Some(&prepend),
        Some(&append),
        None,
        None,
    )
    .expect("assemble with recall");

    // Compression ran AND the injected blocks are present...
    assert_ne!(out.report.mode, CompactionMode::None);
    assert!(out.report.msgs_conserved < out.report.msgs_before);
    let joined = out
        .messages
        .iter()
        .map(|m| m.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(joined.contains(MMD_CONTEXT_MARKER), "MMD injected");
    assert!(joined.contains("<relevant-memories>"), "recall prepend");
    // ...and the union respects the total shared budget.
    assert!(
        est.estimate_messages(&out.messages) <= budget,
        "total context exceeds the shared budget"
    );
}

// ═══ 5. MEM-43 / D19: the worker runs assemble_with_recall post-L3 ═══

/// Long synthetic history that forces compression inside the worker phase:
/// fat turns (~600-char user + ~600-char assistant each) recorded through the
/// REAL L0 recorder so the assembly reads genuine session data.
fn capture_fat_history(db: &VantaEmbedded, session: &str, rounds: usize) {
    let recorder = L0Recorder::new(db.clone());
    for i in 0..rounds {
        let ts = (i as u64 + 10) * 100;
        recorder
            .record_turn(
                &L0Capture {
                    session_id: session.to_string(),
                    messages: vec![
                        L0Message {
                            id: Some(format!("u{i}")),
                            role: L0Role::User,
                            content: format!("turn{i:02} I prefer dark mode {}", "y".repeat(600)),
                            timestamp_ms: ts,
                        },
                        L0Message {
                            id: Some(format!("a{i}")),
                            role: L0Role::Assistant,
                            content: format!("noted turn{i:02} {}", "z".repeat(600)),
                            timestamp_ms: ts + 50,
                        },
                    ],
                },
                None,
            )
            .expect("record fat turn");
    }
}

#[test]
fn d19_worker_assembles_context_post_l3_with_compression_active() {
    let db = open_db();
    let session = "flow-d19";
    let runner = E2eRunner { fail_task: None };

    // Active MMD persisted through the real store path (MEM-24) BEFORE the
    // pass, so the post-L3 phase injects it.
    save_active(
        &db,
        session,
        &TaskMemory {
            meta: SceneMeta {
                created: "2026-08-22T10:00:00.000Z".into(),
                updated: "2026-08-22T10:05:00.000Z".into(),
                summary: "d19 task".into(),
                heat: 1,
            },
            content: "current task: wire context engine into the pipeline".into(),
        },
    )
    .expect("save mmd");

    capture_fat_history(&db, session, 15);

    // Shared budget tight enough to force compression but roomy enough that
    // post-compression headroom fits all three injections (MMD + recall
    // prepend + persona append are whole-or-skip per MEM-37).
    //
    // Derived from the estimator's own measurement of the same fat turns
    // (MEM-43): ~13% keeps the force-compression / headroom-fits-MMD ratio
    // under both the chars/3 branch and `precise-tokens` (BPE), instead of a
    // chars/3 magic number.
    let probe_est = TokenEstimator::default();
    let fat_total = probe_est.estimate_messages(
        &(0..15u32)
            .flat_map(|i| {
                [
                    ChatMessage::new(
                        ChatRole::User,
                        format!("turn{i:02} I prefer dark mode {}", "y".repeat(600)),
                    ),
                    ChatMessage::new(
                        ChatRole::Assistant,
                        format!("noted turn{i:02} {}", "z".repeat(600)),
                    ),
                ]
            })
            .collect::<Vec<_>>(),
    );
    let mut handler = MemoryTaskHandler::new(
        db.clone(),
        &runner,
        Default::default(),
        Default::default(),
        50,
    )
    .with_context_config(ContextAssemblyConfig {
        enabled: true,
        budget_tokens: (fat_total * 13 / 100).max(1),
    });
    // Asserted order: L0 → L1 → L2 → L3 → compress+recall (post-L3).
    for kind in [TaskKind::L1, TaskKind::L2, TaskKind::L3] {
        handler.handle(&task(kind, session)).expect("pass");
    }

    // Every upstream layer produced data first...
    assert_eq!(
        read_session_records(&db, session).expect("records").len(),
        1
    );
    assert_eq!(list_scenes(&db, session).expect("scenes").len(), 1);
    assert!(get_persona(&db, session).expect("persona read").is_some());

    // ...and THEN the assembled context exists as a persisted record.
    let ctx = load_assembled_context(&db, session)
        .expect("read assembled")
        .expect("post-L3 assembly must persist its output");

    // Compression ran (D19): the report proves the compaction pass.
    assert_ne!(ctx.report.mode, CompactionMode::None);
    assert!(
        ctx.report.msgs_conserved < ctx.report.msgs_before,
        "history must be compressed, not conserved"
    );

    // Injections landed within the same pass. MMD is always injected (small
    // block); recall blocks are whole-or-skip against post-compression
    // headroom (MEM-37), so we assert the flag plus the dynamic block rather
    // than pinning every optional block to estimator arithmetic.
    let joined = ctx
        .messages
        .iter()
        .map(|m| m.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(ctx.mmd_injected);
    assert!(ctx.recall_injected, "at least one recall block injected");
    assert!(joined.contains(MMD_CONTEXT_MARKER), "MMD injected");
    assert!(joined.contains("<relevant-memories>"), "recall prepend");

    // Shared-budget contract holds end to end (same derived budget).
    let est = TokenEstimator::default();
    let budget = (fat_total * 13 / 100).max(1);
    assert!(
        est.estimate_messages(&ctx.messages) <= budget,
        "assembled context exceeds the shared budget"
    );
}

#[test]
fn d19_disabled_flag_skips_the_post_l3_phase() {
    let db = open_db();
    let session = "flow-d19-off";
    let runner = E2eRunner { fail_task: None };
    capture_turn(&db, session);

    let mut handler = MemoryTaskHandler::new(
        db.clone(),
        &runner,
        Default::default(),
        Default::default(),
        50,
    )
    .with_context_config(ContextAssemblyConfig {
        enabled: false,
        ..ContextAssemblyConfig::default()
    });
    for kind in [TaskKind::L1, TaskKind::L2, TaskKind::L3] {
        handler.handle(&task(kind, session)).expect("pass");
    }

    // Pipeline layers ran; only the assembly phase is skipped.
    assert_eq!(
        read_session_records(&db, session).expect("records").len(),
        1
    );
    assert!(load_assembled_context(&db, session)
        .expect("read assembled")
        .is_none());
}
