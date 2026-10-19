// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! MEMG-13 — L1 version history + field diff: the audit surface over the core
//! `Embedded::versions`/`get_version` (VS-CORE-07). In-memory store, no LLM.
//!
//! What could break: the payload→record mapping (vector filter, key
//! sanitization), the ascending order of the retained history, and the diff
//! contract (changed top-level fields only; absent optional fields read Null).

use vanta_memory::core::abstractions::{MemoryRecord, MemoryType};
use vanta_memory::core::record::l1_reader::{
    diff_records, l1_namespace, read_record_version, read_record_versions,
};
use vantadb::config::Config;
use vantadb::sdk::Embedded;

fn db() -> Embedded {
    Embedded::open_with_config(Config {
        backend_kind: vantadb::storage::BackendKind::InMemory,
        ..Config::default()
    })
    .expect("open in-memory db")
}

fn record(id: &str, content: &str, version: u32) -> MemoryRecord {
    MemoryRecord {
        id: id.into(),
        content: content.into(),
        memory_type: MemoryType::Persona,
        priority: 80,
        scene_name: "ui-setup".into(),
        source_message_ids: vec![],
        metadata: serde_json::Value::Null,
        timestamps: vec![],
        created_at: "2026-08-20T10:00:00Z".into(),
        updated_at: "2026-08-20T10:00:00Z".into(),
        version,
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

#[test]
fn record_versions_history_is_ascending_and_addressable() {
    let db = db();
    put_l1(&db, &record("m1", "content A", 1));
    let mut v2 = record("m1", "content B", 2);
    v2.updated_at = "2026-08-21T10:00:00Z".into();
    put_l1(&db, &v2);
    let mut v3 = record("m1", "content C", 3);
    v3.updated_at = "2026-08-22T10:00:00Z".into();
    put_l1(&db, &v3);

    let versions = read_record_versions(&db, "sess-1", "m1").expect("history");
    assert_eq!(versions.len(), 3, "one retained version per put");
    assert_eq!(
        versions.iter().map(|v| v.version).collect::<Vec<_>>(),
        vec![1, 2, 3],
        "ascending storage versions"
    );
    assert_eq!(versions[0].record.content, "content A");
    assert_eq!(versions[1].record.content, "content B");
    assert_eq!(versions[2].record.content, "content C");

    let v1 = read_record_version(&db, "sess-1", "m1", 1)
        .expect("get v1")
        .expect("v1 present");
    assert_eq!(v1.version, 1);
    assert_eq!(v1.record.content, "content A");

    let v2 = read_record_version(&db, "sess-1", "m1", 2)
        .expect("get v2")
        .expect("v2 present");
    assert_eq!(v2.version, 2);
    assert_eq!(
        v2.record.content, "content B",
        "an interior version is addressable, not just the first/last"
    );

    assert!(
        read_record_version(&db, "sess-1", "m1", 99)
            .expect("missing version")
            .is_none(),
        "unknown version is None, not an error"
    );
    assert!(
        read_record_versions(&db, "sess-1", "missing")
            .expect("missing record")
            .is_empty(),
        "unknown record has no history"
    );
}

#[test]
fn diff_reports_only_changed_fields() {
    let db = db();
    put_l1(&db, &record("m1", "content A", 1));
    let mut v2 = record("m1", "content B", 2);
    v2.superseded_by = Some("m2".into());
    put_l1(&db, &v2);

    let versions = read_record_versions(&db, "sess-1", "m1").expect("history");
    let changes = diff_records(&versions[0].record, &versions[1].record).expect("diff");
    let field = |name: &str| changes.iter().find(|c| c.field == name);

    let content = field("content").expect("content changed");
    assert_eq!(content.before, serde_json::json!("content A"));
    assert_eq!(content.after, serde_json::json!("content B"));
    let version = field("version").expect("payload version changed");
    assert_eq!(version.before, serde_json::json!(1));
    assert_eq!(version.after, serde_json::json!(2));
    let superseded = field("superseded_by").expect("optional field surfaced");
    assert_eq!(
        superseded.before,
        serde_json::Value::Null,
        "absent (skip_serializing_if) reads as Null"
    );
    assert_eq!(superseded.after, serde_json::json!("m2"));

    assert!(
        field("priority").is_none(),
        "unchanged fields are not reported"
    );
    assert!(field("id").is_none());

    assert!(
        diff_records(&versions[0].record, &versions[0].record)
            .expect("self diff")
            .is_empty(),
        "identical records have no changes"
    );
}
