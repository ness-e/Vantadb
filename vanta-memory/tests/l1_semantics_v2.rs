// ponytail: blanket allow — unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! MEMG-12 — L1 v2 write-side semantics (integration test).
//!
//! Contract: `put_record` (the single L1 write point) stamps the real v2
//! fields on **both pipeline write sites** — extraction (`write_memory`) and
//! dream promotion (`promote_dream_run`):
//!   - `valid_at_ms` = content birth (the L1 record's `created_at`; a merge
//!     keeps the earliest target birth — MGR-10 §D7/D8);
//!   - confidence `Asserted` + D_a declared explicitly (MGR-12 §3.4: uniform
//!     default for direct pipeline writes).
//!
//! Round-trip: `export_namespace` → `import_file` preserves the stamped fields.
//!
//! RED baseline (before the fix): the storage `valid_at_ms` was the write-time
//! default (real clock) instead of the record birth, so every assertion below
//! on a synthetic `T0` birth fails.

use serde_json::json;
use vantadb::config::Config;
use vantadb::sdk::{ConfidenceClass, Embedded};
use vantadb::storage::BackendKind;

use vanta_memory::core::abstractions::{
    DedupAction, DedupDecision, ExtractedMemory, MemoryRecord, MemoryType,
};
use vanta_memory::core::dream::{promote_dream_run, write_dream_run, DreamRun, PromotionAction};
use vanta_memory::core::prompts::l1_extraction::epoch_ms_to_rfc3339;
use vanta_memory::core::record::{apply_dedup_batch, l1_namespace, write_memory};

/// Synthetic content-birth timestamps (fixed clock — never the real `now`).
const T0: u64 = 1_700_000_000_000; // 2023-11-14T22:13:20.000Z
const T1: u64 = 1_700_000_100_000;
const T2: u64 = 1_700_000_200_000;
const SESSION: &str = "sess-v2";

fn open_db() -> Embedded {
    let config = Config {
        backend_kind: BackendKind::InMemory,
        read_only: false,
        ..Config::default()
    };
    Embedded::open_with_config(config).expect("open in-memory db")
}

fn memory(content: &str) -> ExtractedMemory {
    ExtractedMemory {
        content: content.to_string(),
        memory_type: MemoryType::Episodic,
        priority: 80,
        source_message_ids: vec![],
        scene_name: "general".to_string(),
        metadata: json!(null),
    }
}

fn store_decision(id: &str) -> DedupDecision {
    DedupDecision {
        record_id: id.to_string(),
        action: DedupAction::Store,
        target_ids: vec![],
        contradicts: vec![],
        merged_content: None,
        merged_type: None,
        merged_priority: None,
        merged_timestamps: None,
    }
}

/// Storage-record read. Record ids use only `[A-Za-z0-9_-]`, and
/// `sanitize_key` (l0_recorder.rs:157) is the identity on that set — the raw
/// id is the storage key `put_record` used.
fn get_l1(db: &Embedded, id: &str) -> Option<vantadb::sdk::MemoryRecord> {
    db.get(&l1_namespace(SESSION), id).expect("get l1")
}

/// Consolidated dream-side record fixture (promotion input).
fn consolidated(id: &str, created_at: &str) -> MemoryRecord {
    MemoryRecord {
        id: id.into(),
        content: "consolidated summary".into(),
        memory_type: MemoryType::Episodic,
        priority: 70,
        scene_name: "general".into(),
        source_message_ids: vec![],
        metadata: json!(null),
        timestamps: vec![],
        created_at: created_at.into(),
        updated_at: created_at.into(),
        version: 1,
        session_key: SESSION.into(),
        session_id: SESSION.into(),
        task_id: None,
        team_id: None,
        user_id: None,
        agent_id: None,
        vector: None,
        heat: 0,
        superseded_by: None,
    }
}

/// L1 extraction (Store): `valid_at_ms` is the record birth — not the
/// write-time default — and confidence is declared `Asserted`/D_a.
#[test]
fn store_write_stamps_valid_at_with_record_birth() {
    let db = open_db();
    let record = write_memory(
        &db,
        SESSION,
        SESSION,
        &memory("user likes rust"),
        &store_decision("m_v2_store"),
        T0,
        0,
        None,
    )
    .expect("write")
    .expect("stored");
    assert_eq!(record.created_at, epoch_ms_to_rfc3339(T0));

    let stored = get_l1(&db, "m_v2_store").expect("storage record present");
    assert_eq!(
        stored.valid_at_ms, T0,
        "valid_at_ms must be the content birth, not the write-time default"
    );
    assert_eq!(stored.confidence_class, ConfidenceClass::Asserted);
    assert_eq!(stored.confidence, 1.0, "D_a declared explicitly");
    assert!(stored.derived_from.is_empty());
    assert!(stored.quarantined_at_ms.is_none());
}

/// L1 extraction (Merge): the merged record keeps the earliest target birth,
/// and the storage validity window follows it (MGR-10 §D7 `valid_at = created_at`).
#[test]
fn merge_write_stamps_valid_at_with_earliest_target_birth() {
    let db = open_db();
    write_memory(
        &db,
        SESSION,
        SESSION,
        &memory("first"),
        &store_decision("m_v2_a"),
        T0,
        0,
        None,
    )
    .expect("write a")
    .expect("stored a");
    write_memory(
        &db,
        SESSION,
        SESSION,
        &memory("second"),
        &store_decision("m_v2_b"),
        T1,
        1,
        None,
    )
    .expect("write b")
    .expect("stored b");

    let merge = DedupDecision {
        record_id: "m_v2_merged".into(),
        action: DedupAction::Merge,
        target_ids: vec!["m_v2_a".into(), "m_v2_b".into()],
        contradicts: vec![],
        merged_content: Some("first and second".into()),
        merged_type: None,
        merged_priority: None,
        merged_timestamps: None,
    };
    let merged = write_memory(
        &db,
        SESSION,
        SESSION,
        &memory("first and second"),
        &merge,
        T2,
        2,
        None,
    )
    .expect("write merge")
    .expect("stored merge");
    assert_eq!(
        merged.created_at,
        epoch_ms_to_rfc3339(T0),
        "merged record keeps the earliest target birth"
    );

    let stored = get_l1(&db, "m_v2_merged").expect("merged storage record");
    assert_eq!(
        stored.valid_at_ms, T0,
        "valid_at_ms follows the merged content birth (earliest target)"
    );
    assert_eq!(stored.confidence_class, ConfidenceClass::Asserted);
    assert_eq!(stored.confidence, 1.0);
    assert!(get_l1(&db, "m_v2_a").is_none(), "target a replaced");
    assert!(get_l1(&db, "m_v2_b").is_none(), "target b replaced");
}

/// Dream promotion: the promoted consolidated record is stamped with the
/// consolidated content birth, not the promotion write time.
#[test]
fn dream_promotion_stamps_valid_at_with_content_birth() {
    let db = open_db();
    let run = DreamRun {
        run_id: "run-v2-promote".into(),
        session_id: SESSION.into(),
        started_at_ms: T1,
        ended_at_ms: T1 + 1,
        inputs_scanned: 0,
        input_ids: vec![],
        merged_ids: vec![],
        contradicted_ids: vec![],
        normalized_count: 0,
        runner_label: "test".into(),
        consolidated: vec![consolidated("m_v2_sum", &epoch_ms_to_rfc3339(T0))],
    };
    write_dream_run(&db, &run).expect("write dream run");

    let plan = promote_dream_run(&db, SESSION, &run.run_id).expect("promote");
    assert!(
        plan.ops.iter().any(|op| op.action == PromotionAction::Add),
        "new consolidated record is an ADD"
    );

    let stored = get_l1(&db, "m_v2_sum").expect("promoted storage record");
    assert_eq!(
        stored.valid_at_ms, T0,
        "promotion stamps the consolidated content birth"
    );
    assert_eq!(stored.confidence_class, ConfidenceClass::Asserted);
    assert_eq!(stored.confidence, 1.0);
}

/// L1 extraction (batch group-commit — MEMG-11): the shared input builder
/// stamps the same v2 fields when the flush goes through `put_batch`.
#[test]
fn batch_write_stamps_v2_fields_via_shared_builder() {
    let db = open_db();
    let memories = vec![memory("batch one"), memory("batch two")];
    let decisions = vec![
        store_decision("m_v2_batch_a"),
        store_decision("m_v2_batch_b"),
    ];
    let written = apply_dedup_batch(&db, SESSION, SESSION, &memories, &decisions, T0, None)
        .expect("batch write");
    assert_eq!(written.len(), 2);

    let stored = get_l1(&db, "m_v2_batch_a").expect("batch storage record");
    assert_eq!(
        stored.valid_at_ms, T0,
        "batch path stamps the content birth via the shared input builder"
    );
    assert_eq!(stored.confidence_class, ConfidenceClass::Asserted);
    assert_eq!(stored.confidence, 1.0);
}

/// Round-trip: the pipeline-written v2 fields survive `export_namespace` →
/// `import_file` byte-faithfully (SCH-02 transport, exercised over a
/// pipeline write instead of a hand-built record).
#[test]
fn pipeline_write_roundtrips_v2_fields_via_export_import() {
    let db = open_db();
    write_memory(
        &db,
        SESSION,
        SESSION,
        &memory("round trip"),
        &store_decision("m_v2_rt"),
        T0,
        0,
        None,
    )
    .expect("write")
    .expect("stored");

    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("l1_v2.jsonl");
    let export = db
        .export_namespace(&path, &l1_namespace(SESSION), None)
        .expect("export");
    assert_eq!(export.records_exported, 1);

    let db2 = open_db();
    let import = db2.import_file(&path, false).expect("import");
    assert_eq!(import.errors, 0);
    assert_eq!(import.inserted, 1);

    let got = db2
        .get(&l1_namespace(SESSION), "m_v2_rt")
        .expect("get imported")
        .expect("imported record present");
    assert_eq!(
        got.valid_at_ms, T0,
        "validity birth survives the round-trip"
    );
    assert_eq!(got.confidence_class, ConfidenceClass::Asserted);
    assert_eq!(got.confidence, 1.0);
    assert!(got.derived_from.is_empty());
    assert!(got.quarantined_at_ms.is_none());
}

/// Safety fallback: an unparseable birth falls back to the core default
/// (`valid_at_ms = created_at_ms`) instead of failing the write or emitting
/// the rejected `Some(0)` sentinel.
#[test]
fn unparseable_birth_falls_back_to_core_default() {
    let db = open_db();
    let run = DreamRun {
        run_id: "run-v2-bad-birth".into(),
        session_id: SESSION.into(),
        started_at_ms: T1,
        ended_at_ms: T1 + 1,
        inputs_scanned: 0,
        input_ids: vec![],
        merged_ids: vec![],
        contradicted_ids: vec![],
        normalized_count: 0,
        runner_label: "test".into(),
        consolidated: vec![consolidated("m_v2_bad", "not-a-timestamp")],
    };
    write_dream_run(&db, &run).expect("write dream run");
    promote_dream_run(&db, SESSION, &run.run_id).expect("promote");

    let stored = get_l1(&db, "m_v2_bad").expect("promoted storage record");
    assert_eq!(
        stored.valid_at_ms, stored.created_at_ms,
        "fallback: write-time created_at_ms (core default)"
    );
    assert!(stored.valid_at_ms > 0, "never the Some(0) sentinel");
}

/// MEMG-01 contradiction re-persist: the old record is re-written with its OWN
/// birth (not the contradicting write's clock), keeping its validity window
/// stable, and carries the supersede pointer in the L1 payload.
#[test]
fn contradiction_re_persist_keeps_old_record_birth() {
    let db = open_db();
    write_memory(
        &db,
        SESSION,
        SESSION,
        &memory("old fact"),
        &store_decision("m_v2_old"),
        T0,
        0,
        None,
    )
    .expect("write old")
    .expect("stored old");

    let mut contradicting = store_decision("m_v2_new");
    contradicting.contradicts = vec!["m_v2_old".into()];
    write_memory(
        &db,
        SESSION,
        SESSION,
        &memory("new fact"),
        &contradicting,
        T2,
        1,
        None,
    )
    .expect("write new")
    .expect("stored new");

    let old = get_l1(&db, "m_v2_old").expect("old record re-persisted");
    assert_eq!(
        old.valid_at_ms, T0,
        "re-persist keeps the old record's own birth"
    );
    let payload: MemoryRecord = serde_json::from_str(&old.payload).expect("l1 payload");
    assert_eq!(payload.superseded_by.as_deref(), Some("m_v2_new"));
}
