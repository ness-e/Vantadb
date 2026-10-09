// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! MEMG-02: recall → outcome → confidence loop, end-to-end over real
//! components (in-memory VantaDB + auto-recall + `reinforce_recalled`).
//! No LLM involved (recall is LLM-free).

use vanta_memory::core::abstractions::{MemoryRecord, MemoryType};
use vanta_memory::core::hooks::{
    perform_auto_recall, reinforce_recalled, AutoRecallParams, RecallConfig, RecallMode,
    RecallScope, RecalledMemory,
};
use vanta_memory::core::record::l1_reader::l1_namespace;
use vanta_memory::core::record::L1Error;
use vantadb::config::Config;
use vantadb::sdk::{Embedded, MemoryInput};

fn db() -> Embedded {
    Embedded::open_with_config(Config {
        backend_kind: vantadb::storage::BackendKind::InMemory,
        ..Config::default()
    })
    .expect("open in-memory db")
}

fn record(id: &str, content: &str) -> MemoryRecord {
    MemoryRecord {
        id: id.into(),
        content: content.into(),
        memory_type: MemoryType::Persona,
        priority: 80,
        scene_name: "s".into(),
        source_message_ids: vec![],
        metadata: serde_json::Value::Null,
        timestamps: vec!["2026-08-20T10:00:00Z".into()],
        created_at: "2026-08-20T10:00:00Z".into(),
        updated_at: "2026-08-20T10:00:00Z".into(),
        version: 1,
        session_key: "sess-1".into(),
        session_id: "".into(),
        task_id: None,
        team_id: None,
        user_id: None,
        agent_id: None,
        vector: None,
        heat: 0,
        superseded_by: None,
    }
}

/// Seed one L1 record exactly like the L1 writer persists it: payload =
/// serialized vanta-memory record, key = sanitized id, confidence on the core
/// record (declared below D_a so the bump is observable).
fn seed_l1(db: &Embedded, id: &str, content: &str, confidence: f32) {
    db.put(MemoryInput {
        confidence: Some(confidence),
        ..MemoryInput::new(
            l1_namespace("sess-1"),
            id,
            serde_json::to_string(&record(id, content)).expect("serialize l1 record"),
        )
    })
    .expect("seed l1 record");
}

fn recall_session(db: &Embedded, query: &str) -> Vec<RecalledMemory> {
    perform_auto_recall(
        db,
        AutoRecallParams {
            user_text: query,
            session_key: "sess-1",
            isolation: None,
            config: RecallConfig {
                mode: RecallMode::Keyword,
                scope: RecallScope::Session,
                ..RecallConfig::default()
            },
        },
        None,
    )
    .expect("recall")
    .expect("some recall result")
    .recalled_memories
}

#[test]
fn recall_then_reinforce_updates_the_source_record_confidence() {
    let db = db();
    seed_l1(&db, "m1", "user prefers dark mode", 0.5);

    // 1) Recall: the hit must carry the source identity that closes the loop.
    let recalled = recall_session(&db, "dark mode");
    let hit = recalled
        .iter()
        .find(|m| m.source_key == "m1")
        .expect("m1 must be recalled");
    assert_eq!(hit.source_namespace, l1_namespace("sess-1"));

    // 2) Positive outcome → confidence up (+0.05) + validation stamp.
    let updated =
        reinforce_recalled(&db, hit, vantadb::ReinforceOutcome::Used).expect("reinforce used");
    assert!(
        (updated.confidence - 0.55).abs() < 1e-6,
        "used must bump the source record, got {}",
        updated.confidence
    );
    assert!(
        updated.last_validated_at_ms.is_some(),
        "used must stamp last_validated_at_ms"
    );

    // 3) Negative outcome → confidence down (−0.10).
    let updated =
        reinforce_recalled(&db, hit, vantadb::ReinforceOutcome::Corrected).expect("reinforce");
    assert!(
        (updated.confidence - 0.45).abs() < 1e-6,
        "corrected must decay the source record, got {}",
        updated.confidence
    );
}

#[test]
fn reinforce_recalled_rejects_legacy_hits_without_source_identity() {
    let db = db();
    seed_l1(&db, "m1", "fact", 0.5);
    let legacy = RecalledMemory {
        content: "fact".into(),
        score: 1,
        memory_type: "persona".into(),
        source_namespace: String::new(),
        source_key: String::new(),
    };
    let err = reinforce_recalled(&db, &legacy, vantadb::ReinforceOutcome::Used)
        .expect_err("legacy hit without identity must be rejected");
    assert!(
        matches!(err, L1Error::Vanta(vantadb::Error::InvalidInput(_))),
        "expected InvalidInput, got {err:?}"
    );
}
