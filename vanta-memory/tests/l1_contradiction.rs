// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! MEMG-01 dedicated tests: contradiction detection on L1 ingestion.
//!
//! The ingestion path detects explicit contradictions against current records
//! of the same session (inside the existing dedup judgment — no second LLM
//! call) and flags the old record with `superseded_by` (provenance, never
//! deleted), reusing MEM-60 `lifecycle::mark_contradiction`. Canonical case:
//! "me gusta X" → "ya no me gusta X" flags the old record.

use vanta_memory::core::abstractions::{
    DedupAction, DedupDecision, ExtractedMemory, LlmError, LlmRunParams, LlmRunner, MemoryRecord,
    MemoryType,
};
use vanta_memory::core::prompts::{
    get_conflict_detection_system_prompt, CandidateMatch, PromptMode,
};
use vanta_memory::core::record::{
    l1_namespace, parse_batch_result, read_record, read_session_records, run_l1_dedup,
    write_memory, L1DedupConfig,
};
use vantadb::config::Config;
use vantadb::sdk::{Embedded, MemoryInput, MemoryMetadata};
use vantadb::storage::BackendKind;

fn open_db() -> Embedded {
    let config = Config {
        backend_kind: BackendKind::InMemory,
        ..Default::default()
    };
    Embedded::open_with_config(config).expect("open embedded")
}

fn memory(content: &str) -> ExtractedMemory {
    ExtractedMemory {
        content: content.into(),
        memory_type: MemoryType::Persona,
        priority: 80,
        source_message_ids: vec![],
        scene_name: "prefs".into(),
        metadata: serde_json::Value::Null,
    }
}

fn record(id: &str, content: &str) -> MemoryRecord {
    MemoryRecord {
        id: id.into(),
        content: content.into(),
        memory_type: MemoryType::Persona,
        priority: 80,
        scene_name: "prefs".into(),
        source_message_ids: vec![],
        metadata: serde_json::Value::Null,
        timestamps: vec!["2026-08-20T10:00:00.000Z".into()],
        created_at: "2026-08-20T10:00:00.000Z".into(),
        updated_at: "2026-08-20T10:00:00.000Z".into(),
        version: 1,
        session_key: "sk".into(),
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

fn decision(record_id: &str, action: DedupAction, contradicts: &[&str]) -> DedupDecision {
    DedupDecision {
        record_id: record_id.into(),
        action,
        target_ids: vec![],
        contradicts: contradicts.iter().map(|s| s.to_string()).collect(),
        merged_content: None,
        merged_type: None,
        merged_priority: None,
        merged_timestamps: None,
    }
}

fn put_records(db: &Embedded, session: &str, records: &[MemoryRecord]) {
    let ns = l1_namespace(session);
    for r in records {
        db.put(MemoryInput {
            namespace: ns.clone(),
            key: r.id.clone(),
            payload: serde_json::to_string(r).expect("serialize"),
            metadata: MemoryMetadata::new(),
            vector: None,
            sparse_vector: None,
            ttl_ms: None,
            ..Default::default()
        })
        .expect("put");
    }
}

// ── Canonical case: "me gusta X" → "ya no me gusta X" flags the old record ──

#[test]
fn ingestion_marks_contradicted_record_as_superseded() {
    let db = open_db();
    put_records(&db, "sess-a", &[record("m_old", "me gusta el cafe")]);

    let d = decision("m_new", DedupAction::Store, &["m_old"]);
    let written = write_memory(
        &db,
        "sess-a",
        "si",
        &memory("ya no me gusta el cafe"),
        &d,
        1_700_000_000_000,
        0,
        None,
    )
    .expect("write")
    .expect("stored");

    assert_eq!(written.id, "m_new");
    assert_eq!(written.superseded_by, None, "the new record is not flagged");

    let old = read_record(&db, "sess-a", "m_old")
        .expect("read")
        .expect("old record must be preserved");
    assert_eq!(old.superseded_by.as_deref(), Some("m_new"));
    assert_eq!(old.content, "me gusta el cafe", "old content preserved");
    assert_eq!(read_session_records(&db, "sess-a").unwrap().len(), 2);
}

#[test]
fn non_contradictory_write_leaves_existing_records_untouched() {
    let db = open_db();
    put_records(&db, "sess-b", &[record("m_old", "team uses postgres")]);

    let d = decision("m_new", DedupAction::Store, &[]);
    write_memory(
        &db,
        "sess-b",
        "si",
        &memory("deploy on fridays"),
        &d,
        5,
        0,
        None,
    )
    .expect("write")
    .expect("stored");

    let old = read_record(&db, "sess-b", "m_old").unwrap().unwrap();
    assert_eq!(old.superseded_by, None);
}

#[test]
fn already_superseded_target_is_not_remarked() {
    let db = open_db();
    let mut prior = record("m_old", "me gusta el cafe");
    prior.superseded_by = Some("m_prior".into());
    put_records(&db, "sess-c", &[prior]);

    let d = decision("m_new", DedupAction::Store, &["m_old"]);
    let written = write_memory(
        &db,
        "sess-c",
        "si",
        &memory("ya no me gusta el cafe"),
        &d,
        5,
        0,
        None,
    )
    .expect("write");
    assert!(written.is_some());

    let old = read_record(&db, "sess-c", "m_old").unwrap().unwrap();
    assert_eq!(
        old.superseded_by.as_deref(),
        Some("m_prior"),
        "provenance chain of an already-superseded record is not rewritten"
    );
}

#[test]
fn skip_decision_with_contradicts_does_not_mark() {
    let db = open_db();
    put_records(&db, "sess-d", &[record("m_old", "me gusta el cafe")]);

    let d = decision("m_new", DedupAction::Skip, &["m_old"]);
    let written = write_memory(
        &db,
        "sess-d",
        "si",
        &memory("ya no me gusta el cafe"),
        &d,
        5,
        0,
        None,
    )
    .expect("write");
    assert!(written.is_none(), "skip persists nothing");

    let old = read_record(&db, "sess-d", "m_old").unwrap().unwrap();
    assert_eq!(
        old.superseded_by, None,
        "no persisted record → no dangling pointer"
    );
}

#[test]
fn self_reference_in_contradicts_is_ignored() {
    let db = open_db();
    // Exact self id and a sanitization-equivalent variant ("m/new" maps to
    // the same storage key as "m_new") both resolve to the new record
    // itself — never self-marked.
    for (idx, target) in ["m_new", "m/new"].iter().enumerate() {
        let session = format!("sess-e{idx}");
        let d = decision("m_new", DedupAction::Store, &[target]);
        let written = write_memory(&db, &session, "si", &memory("a fact"), &d, 5, 0, None)
            .expect("write")
            .expect("stored");
        assert_eq!(
            written.superseded_by, None,
            "target {target} must not self-mark"
        );
    }
}

#[test]
fn unknown_contradicts_id_is_skipped_silently() {
    let db = open_db();
    let d = decision("m_new", DedupAction::Store, &["m_missing"]);
    let written = write_memory(&db, "sess-f", "si", &memory("a fact"), &d, 5, 0, None)
        .expect("write must not fail")
        .expect("stored");
    assert_eq!(written.id, "m_new");
}

#[test]
fn update_branch_still_marks_external_contradicted_target() {
    let db = open_db();
    put_records(
        &db,
        "sess-g",
        &[
            record("m_old", "me gusta el cafe"),
            record("m_other", "vivo en caracas"),
        ],
    );

    // Update replaces m_old; m_other is only contradicted — it must survive
    // (flagged, never deleted) even though the branch deletes its targets.
    let mut d = decision("m_new", DedupAction::Update, &["m_other"]);
    d.target_ids = vec!["m_old".into()];
    let written = write_memory(
        &db,
        "sess-g",
        "si",
        &memory("ahora vivo en madrid"),
        &d,
        5,
        0,
        None,
    )
    .expect("write")
    .expect("stored");
    assert_eq!(written.id, "m_new");

    assert!(
        read_record(&db, "sess-g", "m_old").unwrap().is_none(),
        "update target replaced"
    );
    let other = read_record(&db, "sess-g", "m_other").unwrap().unwrap();
    assert_eq!(other.superseded_by.as_deref(), Some("m_new"));
    assert_eq!(
        other.content, "vivo en caracas",
        "contradicted record preserved"
    );
}

#[test]
fn contradiction_mark_preserves_old_record_vector() {
    let db = open_db();
    let old = record("m_old", "me gusta el cafe");
    db.put(MemoryInput {
        namespace: l1_namespace("sess-vec"),
        key: "m_old".into(),
        payload: serde_json::to_string(&old).expect("serialize"),
        metadata: MemoryMetadata::new(),
        vector: Some(vec![0.5, 0.25, 0.125]),
        sparse_vector: None,
        ttl_ms: None,
        ..Default::default()
    })
    .expect("seed");

    let d = decision("m_new", DedupAction::Store, &["m_old"]);
    write_memory(
        &db,
        "sess-vec",
        "si",
        &memory("ya no me gusta el cafe"),
        &d,
        5,
        0,
        None,
    )
    .expect("write")
    .expect("stored");

    let old_after = read_record(&db, "sess-vec", "m_old").unwrap().unwrap();
    assert_eq!(old_after.superseded_by.as_deref(), Some("m_new"));
    assert_eq!(
        old_after.vector,
        Some(vec![0.5, 0.25, 0.125]),
        "old node vector preserved on re-persist"
    );
}

// ── Judgment wire: parse + prompt ──────────────────────────────────────

#[test]
fn parse_reads_contradicts_list() {
    let matches = vec![CandidateMatch {
        record_id: "m_0".into(),
        memory: memory("ya no me gusta el cafe"),
        candidates: vec![record("m_old", "me gusta el cafe")],
    }];
    let raw = r#"[{"record_id":"m_0","action":"store","contradicts":["m_old","m_old2"]}]"#;
    let decisions = parse_batch_result(raw, &matches);
    assert_eq!(decisions[0].contradicts, vec!["m_old", "m_old2"]);

    // Missing / malformed → empty (tolerant, never fails the batch).
    let missing = parse_batch_result(r#"[{"record_id":"m_0","action":"store"}]"#, &matches);
    assert!(missing[0].contradicts.is_empty());
    let malformed = parse_batch_result(
        r#"[{"record_id":"m_0","action":"store","contradicts":"m_old"}]"#,
        &matches,
    );
    assert!(malformed[0].contradicts.is_empty());
}

#[test]
fn conflict_prompt_documents_conservative_contradicts_rule() {
    for mode in [PromptMode::Chat, PromptMode::Code] {
        let p = get_conflict_detection_system_prompt(mode);
        assert!(p.contains("contradicts"), "schema documents the field");
        assert!(
            p.to_lowercase().contains("explicit"),
            "rule is conservative (explicit contradictions only)"
        );
    }
}

// ── E2E: canonical case through the full two-call pipeline ─────────────

#[test]
fn run_l1_dedup_marks_contradiction_end_to_end() {
    let db = open_db();
    put_records(&db, "sess-e2e", &[record("m_old", "me gusta el cafe")]);

    // Dynamic runner: echoes the REAL transient id the pipeline assigned and
    // returns the canonical contradiction judgment (store + contradicts).
    struct ContradictionEcho;
    impl LlmRunner for ContradictionEcho {
        fn run(&self, params: &LlmRunParams) -> Result<String, LlmError> {
            let new_block = params
                .prompt
                .split("NEW MEMORIES TO JUDGE")
                .nth(1)
                .unwrap_or(&params.prompt);
            let record_id = new_block
                .split("\"record_id\": \"")
                .nth(1)
                .and_then(|rest| rest.split('"').next())
                .unwrap_or("m_0")
                .to_string();
            Ok(format!(
                r#"[{{"record_id": "{record_id}", "action": "store", "contradicts": ["m_old"]}}]"#
            ))
        }
    }

    let written = run_l1_dedup(
        &db,
        &ContradictionEcho,
        "sess-e2e",
        "si",
        &[memory("ya no me gusta el cafe")],
        &L1DedupConfig::default(),
    )
    .expect("pipeline");

    assert_eq!(written.len(), 1);
    let old = read_record(&db, "sess-e2e", "m_old").unwrap().unwrap();
    assert_eq!(
        old.superseded_by.as_deref(),
        Some(written[0].id.as_str()),
        "canonical: the old record is flagged as superseded by the new one"
    );
    assert_eq!(read_session_records(&db, "sess-e2e").unwrap().len(), 2);
}
