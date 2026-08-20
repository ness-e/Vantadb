//! D19 — dedicated tests for the data contracts (MEM-08b).
//!
//! Exercises serialization round-trips and LLM-wire parsing through the
//! public API, with no features required (LLM-free default).

use vanta_memory::core::abstractions::{
    DedupAction, DedupDecision, ExtractedMemory, L1ExtractionResult, MemoryRecord, MemoryType,
    SceneMeta, SceneSegment,
};

#[test]
fn memory_record_roundtrip() {
    let record = MemoryRecord {
        id: "m_1".into(),
        content: "user prefers dark mode".into(),
        memory_type: MemoryType::Persona,
        priority: 80,
        scene_name: "ui-setup".into(),
        source_message_ids: vec!["msg_1".into()],
        metadata: serde_json::json!({}),
        timestamps: vec!["2026-08-20T10:00:00Z".into()],
        created_at: "2026-08-20T10:00:00Z".into(),
        updated_at: "2026-08-20T10:00:00Z".into(),
        version: 2,
        session_key: "sk".into(),
        session_id: "si".into(),
        task_id: None,
        team_id: None,
        user_id: None,
        agent_id: None,
    };
    let json = serde_json::to_string(&record).unwrap();
    let back: MemoryRecord = serde_json::from_str(&json).unwrap();
    assert_eq!(back, record);
    assert!(
        json.contains("\"type\":\"persona\""),
        "wire uses `type` key: {json}"
    );
}

#[test]
fn dedup_decision_parse_from_llm_wire() {
    // Shape emitted by the L1 conflict-detection prompt (TDAM l1-dedup.ts).
    let wire = r#"[
        {"record_id":"m_2","action":"merge","target_ids":["m_old"],"merged_content":"both","merged_priority":70},
        {"record_id":"m_3","action":"skip","target_ids":[]}
    ]"#;
    let decisions: Vec<DedupDecision> = serde_json::from_str(wire).unwrap();
    assert_eq!(decisions.len(), 2);
    assert_eq!(decisions[0].action, DedupAction::Merge);
    assert_eq!(decisions[0].merged_content.as_deref(), Some("both"));
    assert_eq!(decisions[1].action, DedupAction::Skip);
    assert!(decisions[1].merged_content.is_none());
}

#[test]
fn extracted_memory_parses_llm_wire() {
    let wire = r#"{"content":"ship it","type":"work_task","priority":50,"source_message_ids":["m1"],"scene_name":"release","metadata":{"deadline":"2026-08-25"}}"#;
    let mem: ExtractedMemory = serde_json::from_str(wire).unwrap();
    assert_eq!(mem.memory_type, MemoryType::WorkTask);
    assert_eq!(mem.metadata["deadline"], "2026-08-25");
}

#[test]
fn l1_extraction_result_defaults() {
    // `records`/`scene_names` are optional in the wire — must default.
    let wire = r#"{"success":true,"extracted_count":1,"stored_count":0}"#;
    let result: L1ExtractionResult = serde_json::from_str(wire).unwrap();
    assert!(result.success);
    assert!(result.records.is_empty());
    assert!(result.scene_names.is_empty());
    assert!(result.last_scene_name.is_none());
}

#[test]
fn scene_meta_roundtrip() {
    let meta = SceneMeta {
        created: "2026-08-20T10:00:00Z".into(),
        updated: "2026-08-20T10:00:00Z".into(),
        summary: "deploy runbook".into(),
        heat: 3,
    };
    let json = serde_json::to_string(&meta).unwrap();
    assert!(json.contains("\"heat\":3"));
    let back: SceneMeta = serde_json::from_str(&json).unwrap();
    assert_eq!(back, meta);
}

#[test]
fn scene_segment_grouping_roundtrip() {
    let segment = SceneSegment {
        scene_name: "deploy".into(),
        message_ids: vec!["m1".into(), "m2".into()],
        memories: vec![],
    };
    let json = serde_json::to_string(&segment).unwrap();
    let back: SceneSegment = serde_json::from_str(&json).unwrap();
    assert_eq!(back.scene_name, "deploy");
    assert_eq!(back.message_ids.len(), 2);
}
