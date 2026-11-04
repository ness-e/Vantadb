// ponytail: blanket allow — unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! EU AI Act Art. 50 Provenance Metadata (TEC-12) integration tests.

use vantadb::config::Config;
use vantadb::sdk::Embedded;
use vantadb::storage::BackendKind;

use vanta_memory::core::abstractions::{MemoryRecord, MemoryType, SceneMeta};
use vanta_memory::core::conversation::{L0Capture, L0Message, L0Recorder, L0Role};
use vanta_memory::core::dream::{consolidate_session, DreamConfig};
use vanta_memory::core::scene::scene_format::SceneBlock;
use vanta_memory::core::scene::scene_index::write_scene_block;

fn open_db() -> Embedded {
    let config = Config {
        backend_kind: BackendKind::InMemory,
        read_only: false,
        ..Config::default()
    };
    Embedded::open_with_config(config).expect("open in-memory db")
}

#[test]
fn test_art50_provenance_l0_assistant_messages() {
    let db = open_db();
    let recorder = L0Recorder::new(db.clone());

    let capture = L0Capture {
        session_id: "sess_art50_l0".into(),
        messages: vec![
            L0Message {
                id: None,
                role: L0Role::User,
                content: "Hello from human".into(),
                timestamp_ms: 1000,
            },
            L0Message {
                id: None,
                role: L0Role::Assistant,
                content: "Hello! I am an AI assistant.".into(),
                timestamp_ms: 1001,
            },
        ],
    };

    recorder
        .record_turn(&capture, None)
        .expect("record capture");

    // Read stored records directly from vantadb
    let ns = "l0/sess_art50_l0";
    let list = db
        .list(ns, vantadb::sdk::MemoryListOptions::default())
        .expect("list records");
    assert_eq!(list.records.len(), 2);

    let user_rec = list
        .records
        .iter()
        .find(|r| r.payload == "Hello from human")
        .unwrap();
    assert_eq!(
        user_rec.metadata.get("ai_generated"),
        Some(&vantadb::sdk::Value::Bool(false))
    );

    let ai_rec = list
        .records
        .iter()
        .find(|r| r.payload == "Hello! I am an AI assistant.")
        .unwrap();
    assert_eq!(
        ai_rec.metadata.get("ai_generated"),
        Some(&vantadb::sdk::Value::Bool(true))
    );
    assert_eq!(
        ai_rec.metadata.get("synthesizer_model"),
        Some(&vantadb::sdk::Value::String("assistant".into()))
    );
    assert!(ai_rec.metadata.contains_key("confidence_score"));
    assert!(ai_rec.metadata.contains_key("prompt_hash"));
}

#[test]
fn test_art50_provenance_l2_scene_blocks() {
    let db = open_db();
    let block = SceneBlock::new(
        "scene_intro",
        SceneMeta {
            created: "2026-11-04T00:00:00Z".into(),
            updated: "2026-11-04T00:00:00Z".into(),
            summary: "User introduction scene".into(),
            heat: 1,
        },
        "User discussed rust compiler features",
    );

    write_scene_block(&db, "sess_scene_test", &block).expect("write scene block");

    let rec = db
        .get("scene/sess_scene_test", "scene_intro")
        .expect("get scene")
        .expect("must exist");
    assert_eq!(
        rec.metadata.get("ai_generated"),
        Some(&vantadb::sdk::Value::Bool(true))
    );
    assert_eq!(
        rec.metadata.get("synthesizer_model"),
        Some(&vantadb::sdk::Value::String("scene_extractor".into()))
    );
    assert!(rec.metadata.contains_key("confidence_score"));
    assert!(rec.metadata.contains_key("prompt_hash"));
}

#[test]
fn test_art50_provenance_dream_consolidation() {
    let db = open_db();
    let session = "sess_dream_art50";

    // Write a record to L1
    let rec = MemoryRecord {
        id: "rec_1".into(),
        content: "User prefers async rust".into(),
        memory_type: MemoryType::Persona,
        priority: 90,
        scene_name: "intro".into(),
        source_message_ids: vec!["m1".into(), "m2".into()],
        metadata: serde_json::json!({}),
        timestamps: vec!["2026-11-04T00:00:00Z".into()],
        created_at: "2026-11-04T00:00:00Z".into(),
        updated_at: "2026-11-04T00:00:00Z".into(),
        version: 1,
        session_key: session.into(),
        session_id: session.into(),
        task_id: None,
        team_id: None,
        user_id: None,
        agent_id: None,
        vector: None,
        heat: 0,
        superseded_by: None,
    };

    let payload = serde_json::to_string(&rec).unwrap();
    db.put(vantadb::sdk::MemoryInput::new(
        format!("l1/{session}"),
        "rec_1",
        payload,
    ))
    .expect("put l1 record");

    let config = DreamConfig::default().with_idle_threshold_ms(100);
    let run = consolidate_session(&db, session, 1_000_000, 500_000, &config).expect("consolidate");

    assert!(!run.consolidated.is_empty());
    let cons = &run.consolidated[0];
    let meta = cons
        .metadata
        .as_object()
        .expect("metadata must be json object");

    assert_eq!(
        meta.get("ai_generated"),
        Some(&serde_json::Value::Bool(true))
    );
    assert_eq!(
        meta.get("synthesizer_model"),
        Some(&serde_json::Value::String(
            "heuristic:vanta-dream-v1".into()
        ))
    );
    assert!(meta.get("confidence_score").is_some());
    assert!(meta.get("prompt_hash").is_some());
}
