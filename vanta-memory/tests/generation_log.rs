// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! MEM-41 D19 hook tests: generation-log provenance for L1/L2/L3.
//!
//! (a) successful generations register an entry {layer, status, anchor_id,
//!     session, ts}; (b) LLM failures register status=failed best-effort
//!     (never block the pipeline); (c) query by session/layer is ordered by ts.

use vanta_memory::core::abstractions::{
    DedupAction, DedupDecision, ExtractedMemory, LlmError, LlmRunParams, LlmRunner, MemoryType,
};
use vanta_memory::core::memory_generation_log::{query_session, GenerationLayer, GenerationStatus};
use vanta_memory::core::persona::{generate_persona, PersonaGenerateParams};
use vanta_memory::core::prompts::l1_extraction::PromptMode;
use vanta_memory::core::record::write_memory;
use vanta_memory::core::scene::{extract_scenes_with_llm, SceneMemoryInput};

fn test_db() -> vantadb::sdk::VantaEmbedded {
    use vantadb::config::VantaConfig;
    use vantadb::storage::BackendKind;
    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        read_only: false,
        ..VantaConfig::default()
    };
    vantadb::sdk::VantaEmbedded::open_with_config(config).expect("open in-memory db")
}

struct Failing;

impl LlmRunner for Failing {
    fn run(&self, _params: &LlmRunParams) -> Result<String, LlmError> {
        Err(LlmError::NotConfigured)
    }
}

struct Fixed(&'static str);

impl LlmRunner for Fixed {
    fn run(&self, _params: &LlmRunParams) -> Result<String, LlmError> {
        Ok(self.0.to_string())
    }
}

fn memory(content: &str) -> ExtractedMemory {
    ExtractedMemory {
        content: content.to_string(),
        memory_type: MemoryType::Episodic,
        priority: 80,
        scene_name: "general".to_string(),
        source_message_ids: vec![],
        metadata: serde_json::Value::Null,
    }
}

/// D19(a): a persisted L1 record registers a succeeded entry anchored to it.
#[test]
fn l1_write_registers_succeeded_entry() {
    let db = test_db();
    let decision = DedupDecision {
        record_id: String::new(),
        action: DedupAction::Store,
        target_ids: vec![],
        merged_content: None,
        merged_type: None,
        merged_priority: None,
        merged_timestamps: None,
    };
    let record = write_memory(
        &db,
        "sess-1",
        "sess-1",
        &memory("user likes rust"),
        &decision,
        1_000,
        0,
        None,
    )
    .expect("write")
    .expect("stored");

    let entries = query_session(&db, "sess-1", Some(GenerationLayer::L1)).expect("query");
    assert_eq!(entries.len(), 1);
    let entry = &entries[0];
    assert_eq!(entry.status, GenerationStatus::Succeeded);
    assert_eq!(entry.anchor_id.as_deref(), Some(record.id.as_str()));
    assert_eq!(entry.session_key, "sess-1");
    assert!(entry.error.is_none());
}

/// D19(b): an LLM failure at L2 registers failed — and never blocks.
#[test]
fn l2_llm_failure_registers_failed_entry() {
    let db = test_db();
    let result = extract_scenes_with_llm(
        &db,
        "sess-2",
        &Failing,
        &[SceneMemoryInput {
            id: "m1".into(),
            content: "deployed on friday".into(),
            created_at: "2026-08-21T00:00:00.000Z".into(),
        }],
        None,
    );
    assert!(!result.success, "pipeline degrades but survives");

    let entries = query_session(&db, "sess-2", Some(GenerationLayer::L2)).expect("query");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].status, GenerationStatus::Failed);
    assert!(entries[0].error.is_some());
}

/// D19(a): a real L2 generation (non-empty extraction) registers succeeded.
#[test]
fn l2_generation_registers_succeeded_entry() {
    let db = test_db();
    let runner = Fixed(r#"[{"scene_name":"deploys","summary":"s","content":"shipped v1"}]"#);
    let result = extract_scenes_with_llm(
        &db,
        "sess-3",
        &runner,
        &[SceneMemoryInput {
            id: "m1".into(),
            content: "shipped v1".into(),
            created_at: "2026-08-21T00:00:00.000Z".into(),
        }],
        None,
    );
    assert!(result.success);

    let entries = query_session(&db, "sess-3", Some(GenerationLayer::L2)).expect("query");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].status, GenerationStatus::Succeeded);
}

/// D19(a)+(b): persona generation logs succeeded when it writes, failed when
/// the LLM fails; both consultable per session ordered by ts.
#[test]
fn l3_persona_logs_succeeded_and_failed() {
    let db = test_db();
    // Seed one live scene so the trigger has changes to generate from.
    let seeded = extract_scenes_with_llm(
        &db,
        "sess-4",
        &Fixed(r#"[{"scene_name":"work","summary":"s","content":"team uses postgres"}]"#),
        &[SceneMemoryInput {
            id: "m1".into(),
            content: "team uses postgres".into(),
            created_at: "2026-08-21T00:00:00.000Z".into(),
        }],
        None,
    );
    assert!(seeded.success);

    // Failed run first (ts_1), then a successful generation (ts_2).
    let failed = generate_persona(
        &db,
        &Failing,
        &PersonaGenerateParams {
            session_key: "sess-4",
            total_processed: 10,
            prompt_mode: PromptMode::Chat,
            trigger_info: None,
        },
    );
    assert!(!failed.success);

    let ok = generate_persona(
        &db,
        &Fixed(r#"{"persona":"Calm operator."}"#),
        &PersonaGenerateParams {
            session_key: "sess-4",
            total_processed: 20,
            prompt_mode: PromptMode::Chat,
            trigger_info: None,
        },
    );
    assert!(ok.success && ok.updated);

    let entries = query_session(&db, "sess-4", Some(GenerationLayer::L3)).expect("query");
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].status, GenerationStatus::Failed); // oldest first
    assert_eq!(entries[1].status, GenerationStatus::Succeeded);
    assert!(entries[0].ts_ms <= entries[1].ts_ms);
}
