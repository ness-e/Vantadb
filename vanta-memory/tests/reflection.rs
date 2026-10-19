// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! MEMG-21 — reflection pass over episodic memory (lessons).
//!
//! Seeds a session with episodic records (+ one persona that must be
//! ignored), runs the pull-based reflection pass and pins: deterministic
//! LLM-free lessons with provenance, L1 byte-identity (the pass never mutates
//! the store), the typed precondition and the optional `Reflector` runner.

use vanta_memory::core::abstractions::{MemoryRecord, MemoryType};
use vanta_memory::core::record::{l1_namespace, read_session_records};
use vanta_memory::core::reflection::{
    load_reflection_run, reflect_session, ReflectionConfig, ReflectionContext, ReflectionError,
    Reflector,
};
use vantadb::config::Config;
use vantadb::sdk::Embedded;

/// Fixed instant: 2026-01-31T00:00:00.000Z (same fixture instant as MEMG-07/21).
const NOW_MS: u64 = 1_769_817_600_000;

fn db() -> Embedded {
    Embedded::open_with_config(Config {
        backend_kind: vantadb::storage::BackendKind::InMemory,
        ..Config::default()
    })
    .expect("open in-memory db")
}

fn record(
    id: &str,
    scene: &str,
    content: &str,
    memory_type: MemoryType,
    priority: i32,
    updated_at: &str,
) -> MemoryRecord {
    MemoryRecord {
        id: id.into(),
        content: content.into(),
        memory_type,
        priority,
        scene_name: scene.into(),
        source_message_ids: vec![],
        metadata: serde_json::Value::Null,
        timestamps: vec![updated_at.into()],
        created_at: updated_at.into(),
        updated_at: updated_at.into(),
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

fn episode(id: &str, scene: &str, content: &str, priority: i32) -> MemoryRecord {
    record(
        id,
        scene,
        content,
        MemoryType::Episodic,
        priority,
        "2026-01-20T10:00:00.000Z",
    )
}

/// Persist an L1 record exactly as `read_session_records` expects it.
fn put_l1(db: &Embedded, record: &MemoryRecord) {
    use vantadb::sdk::{MemoryInput, MemoryMetadata};
    db.put(MemoryInput {
        namespace: l1_namespace(&record.session_key),
        key: record.id.clone(),
        payload: serde_json::to_string(record).unwrap(),
        metadata: MemoryMetadata::new(),
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
        ..Default::default()
    })
    .expect("put l1 record");
}

/// 4 episodes in 2 scenes + 1 persona (must be ignored by the pass).
fn seed(db: &Embedded) -> Vec<MemoryRecord> {
    let records = vec![
        episode("e1", "ops", "deploy failed on friday", 40),
        episode("e2", "ops", "deploy retried with rollback", 90),
        episode("e3", "ops", "deploy pipeline needed manual approval", 60),
        episode("e4", "ui", "user prefers dark mode", 70),
        record(
            "p1",
            "ui",
            "stable preference: concise answers",
            MemoryType::Persona,
            80,
            "2026-01-20T10:00:00.000Z",
        ),
    ];
    for r in &records {
        put_l1(db, r);
    }
    records
}

fn config() -> ReflectionConfig {
    ReflectionConfig::default().with_run_id_salt("test")
}

fn l1_payloads(db: &Embedded, records: &[MemoryRecord]) -> Vec<String> {
    records
        .iter()
        .map(|r| {
            db.get(&l1_namespace(&r.session_key), &r.id)
                .expect("read l1")
                .expect("record present")
                .payload
        })
        .collect()
}

// ── pass: lessons + L1 byte-identity ──

#[test]
fn reflection_pass_produces_lessons_and_never_mutates_l1() {
    let db = db();
    let records = seed(&db);
    let before = l1_payloads(&db, &records);

    let run = reflect_session(&db, "sess-1", NOW_MS, &config()).expect("reflect");

    assert_eq!(run.episodes_scanned, 4, "episodic only (persona ignored)");
    assert_eq!(run.runner_label, "none", "LLM-free degradation (P4)");
    assert_eq!(run.lessons.len(), 2, "one lesson per scene");
    assert_eq!(run.run_id.len(), 16);
    assert_eq!(run.source_ids.len(), 4);

    // The L1 store is byte-identical and still complete.
    let after = l1_payloads(&db, &records);
    assert_eq!(before, after, "L1 payloads untouched");
    assert_eq!(read_session_records(&db, "sess-1").unwrap().len(), 5);

    // The run round-trips through its namespace.
    let loaded = load_reflection_run(&db, "sess-1", &run.run_id)
        .expect("load")
        .expect("run persisted");
    assert_eq!(loaded, run);
}

#[test]
fn reflection_below_min_episodic_is_rejected() {
    let db = db();
    put_l1(&db, &episode("e1", "ops", "one", 40));
    put_l1(&db, &episode("e2", "ops", "two", 50));

    let err = reflect_session(&db, "sess-1", NOW_MS, &config()).expect_err("must reject");
    assert!(matches!(
        err,
        ReflectionError::NotEnoughMaterial {
            found: 2,
            required: 3
        }
    ));
}

// ── lessons: form + provenance ──

#[test]
fn reflection_lessons_carry_provenance() {
    let db = db();
    seed(&db);
    let run = reflect_session(&db, "sess-1", NOW_MS, &config()).expect("reflect");

    let ops = run
        .lessons
        .iter()
        .find(|l| l.scene_name == "ops")
        .expect("ops lesson");
    assert_eq!(ops.memory_type, MemoryType::WorkMethod);
    assert_eq!(ops.priority, 90, "lesson inherits the max priority");
    assert_eq!(ops.session_key, "sess-1");
    let cited = ops.metadata["reflection"]["source_ids"]
        .as_array()
        .expect("source_ids array");
    assert_eq!(cited.len(), 3);
    assert!(cited.iter().any(|v| v == "e2"));

    let ui = run
        .lessons
        .iter()
        .find(|l| l.scene_name == "ui")
        .expect("ui lesson");
    assert_eq!(ui.priority, 70);
    assert_eq!(
        ui.metadata["reflection"]["source_ids"]
            .as_array()
            .unwrap()
            .len(),
        1,
        "persona p1 is not cited"
    );
}

#[test]
fn reflection_pass_is_deterministic_for_same_inputs() {
    let db = db();
    seed(&db);
    let first = reflect_session(&db, "sess-1", NOW_MS, &config()).expect("reflect");
    let second = reflect_session(&db, "sess-1", NOW_MS, &config()).expect("reflect");
    assert_eq!(first, second, "same salt + now → identical run");
}

// ── optional LLM runner ──

struct MockReflector;

impl Reflector for MockReflector {
    fn label(&self) -> &str {
        "mock"
    }

    fn reflect(
        &self,
        episodes: Vec<MemoryRecord>,
        ctx: &ReflectionContext,
    ) -> Result<Vec<MemoryRecord>, String> {
        let mut lesson = episodes[0].clone();
        lesson.id = format!("mock-{}", ctx.session_id);
        lesson.memory_type = MemoryType::WorkMethod;
        lesson.content = format!("synthesized {} episodes", episodes.len());
        Ok(vec![lesson])
    }
}

#[test]
fn reflection_runner_overrides_llm_free_path() {
    let db = db();
    seed(&db);
    let config = ReflectionConfig::default()
        .with_reflector(Box::new(MockReflector))
        .with_run_id_salt("test");

    let run = reflect_session(&db, "sess-1", NOW_MS, &config).expect("reflect");

    assert_eq!(run.runner_label, "mock");
    assert_eq!(run.lessons.len(), 1);
    assert_eq!(run.lessons[0].content, "synthesized 4 episodes");
    assert_eq!(run.lessons[0].id, "mock-sess-1");
}
