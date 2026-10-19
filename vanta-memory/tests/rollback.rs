// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! MEMG-17 (pieza a) — rollback semántico de L1 a una versión retenida.
//! In-memory store, sin LLM.
//!
//! What could break: la línea de tiempo (el rollback debe ser una versión
//! NUEVA, nunca un overwrite que borre historia), la semántica declarada
//! (supersession restaurada verbatim, bookkeeping avanzado), y los errores
//! de record/versión/registro borrado (NotFound, nunca fabricación).

use vanta_memory::core::abstractions::{MemoryRecord, MemoryType};
use vanta_memory::core::record::l1_reader::{l1_namespace, read_record, read_record_versions};
use vanta_memory::core::record::rollback::rollback_record;
use vanta_memory::core::record::L1Error;
use vanta_memory::utils::backup::{create_snapshot, rollback_snapshot};
use vantadb::config::Config;
use vantadb::error::Error;
use vantadb::sdk::Embedded;
use vantadb::storage::BackendKind;

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

/// Persist an L1 record exactly as `read_record` expects it.
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

const NOW_MS: u64 = 1_700_000_000_000;

#[test]
fn rollback_restores_target_content_as_new_version_with_lineage() {
    let db = db();
    put_l1(&db, &record("m1", "content A", 1));
    let mut v2 = record("m1", "content B", 2);
    v2.updated_at = "2026-08-21T10:00:00Z".into();
    put_l1(&db, &v2);
    let mut v3 = record("m1", "content C", 3);
    v3.updated_at = "2026-08-22T10:00:00Z".into();
    put_l1(&db, &v3);

    let report = rollback_record(&db, "sess-1", "m1", 1, NOW_MS).expect("rollback to v1 succeeds");

    assert_eq!(report.record_id, "m1");
    assert_eq!(report.session_key, "sess-1");
    assert_eq!(report.from_version, 3, "live tip was v3");
    assert_eq!(report.to_version, 1, "target requested was v1");
    assert_eq!(report.new_version, 4, "rollback appends core version v4");
    assert!(
        !report.out_of_scope.is_empty(),
        "declared scope never silent"
    );

    // The store delta the rollback produced: content went C -> A.
    let content = report
        .changes
        .iter()
        .find(|c| c.field == "content")
        .expect("content is part of the reverted delta");
    assert_eq!(content.before, serde_json::json!("content C"));
    assert_eq!(content.after, serde_json::json!("content A"));

    // Live record now reads as the target payload.
    let live = read_record(&db, "sess-1", "m1")
        .expect("read live")
        .expect("live present");
    assert_eq!(live.content, "content A");

    // Lineage is append-only: 4 retained versions, ascending, v4 = restored.
    let versions = read_record_versions(&db, "sess-1", "m1").expect("history");
    assert_eq!(
        versions.iter().map(|v| v.version).collect::<Vec<_>>(),
        vec![1, 2, 3, 4],
        "rollback is itself a new version; nothing is rewritten or dropped"
    );
    assert_eq!(versions[3].record.content, "content A");
    assert_eq!(
        versions[3].record.version, 4,
        "memory-level version advances from the live tip (3 -> 4)"
    );
}

#[test]
fn rollback_missing_version_is_not_found() {
    let db = db();
    put_l1(&db, &record("m1", "content A", 1));

    match rollback_record(&db, "sess-1", "m1", 99, NOW_MS) {
        Err(L1Error::Vanta(Error::NotFound { .. })) => {}
        other => panic!("expected NotFound for v99, got {other:?}"),
    }
}

#[test]
fn rollback_missing_record_is_not_found() {
    let db = db();

    match rollback_record(&db, "sess-1", "nope", 1, NOW_MS) {
        Err(L1Error::Vanta(Error::NotFound { .. })) => {}
        other => panic!("expected NotFound for missing record, got {other:?}"),
    }
}

#[test]
fn rollback_deleted_record_is_not_found() {
    let db = db();
    put_l1(&db, &record("m1", "content A", 1));
    put_l1(&db, &record("m1", "content B", 2));
    // Delete purges the retained version history: the record is not
    // resurrectable from versions (snapshot restore is the data-only path).
    db.delete(&l1_namespace("sess-1"), "m1").expect("delete");

    match rollback_record(&db, "sess-1", "m1", 1, NOW_MS) {
        Err(L1Error::Vanta(Error::NotFound { .. })) => {}
        other => panic!("expected NotFound for deleted record, got {other:?}"),
    }
}

#[test]
fn rollback_to_superseded_target_declares_supersession() {
    let db = db();
    put_l1(&db, &record("m1", "content A", 1));
    let mut v2 = record("m1", "content A", 2);
    v2.superseded_by = Some("m9".into());
    put_l1(&db, &v2);
    let mut v3 = record("m1", "content corrected", 3);
    v3.updated_at = "2026-08-23T10:00:00Z".into();
    put_l1(&db, &v3);

    let report = rollback_record(&db, "sess-1", "m1", 2, NOW_MS).expect("rollback to v2");

    let live = read_record(&db, "sess-1", "m1")
        .expect("read live")
        .expect("live present");
    assert_eq!(
        live.superseded_by.as_deref(),
        Some("m9"),
        "target payload restored verbatim, supersession pointer included"
    );
    assert!(
        report
            .out_of_scope
            .iter()
            .any(|s| s.contains("superseded_by")),
        "the supersession caveat is declared, not silent: {:?}",
        report.out_of_scope
    );
}

// ---------------------------------------------------------------------------
// Step 2 — snapshot rollback declaration (Fjall + tempdir, FIND-287 pinned).
// ---------------------------------------------------------------------------

fn fjall_config(storage_path: &str) -> Config {
    Config {
        storage_path: storage_path.to_string(),
        backend_kind: BackendKind::Fjall,
        ..Config::default()
    }
}

#[test]
fn snapshot_rollback_declares_data_only_scope() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("db").to_string_lossy().to_string();
    let config = fjall_config(&db_path);

    {
        let db = Embedded::open_with_config(config.clone()).expect("open fjall");
        put_l1(&db, &record("m1", "alpha", 1));
        put_l1(&db, &record("m2", "beta", 1));
        create_snapshot(&db, "snap-rb").expect("create snapshot");

        // Post-snapshot mutations: an addition and a delete (both directions
        // of the FIND-287 gap are pinned below through the new surface).
        put_l1(&db, &record("m3", "gamma", 1));
        db.delete(&l1_namespace("sess-1"), "m1").expect("delete m1");
        db.close().expect("close before rollback");
    }

    let (db, report) = rollback_snapshot(config, "snap-rb").expect("rollback snapshot");

    assert_eq!(report.snapshot, "snap-rb");
    assert!(
        report.reverted.iter().any(|s| s.contains("data/")),
        "reverted declares the data-dir swap: {:?}",
        report.reverted
    );
    assert!(
        report.not_reverted.iter().any(|s| s.contains("FIND-287")),
        "not_reverted declares the backend-KV limit (FIND-287): {:?}",
        report.not_reverted
    );
    assert!(
        !report.caveats.is_empty(),
        "flow caveats declared, never silent"
    );

    // Pinned FIND-287 behavior through the new surface: post-snapshot addition
    // disappears with `data/`, and the post-snapshot delete is NOT rolled back
    // (live backend tombstone survives). Same contract as the MEMG-13 canary.
    let mut contents: Vec<String> = vanta_memory::core::record::read_session_records(&db, "sess-1")
        .expect("read session")
        .into_iter()
        .map(|r| r.content)
        .collect();
    contents.sort();
    assert_eq!(
        contents,
        vec!["beta".to_string()],
        "m1 stays deleted, m3 gone, m2 back (data-only restore, FIND-287)"
    );
    db.close().expect("close after rollback");
}

#[test]
fn snapshot_rollback_missing_is_not_found() {
    let dir = tempfile::tempdir().expect("tempdir");
    let config = fjall_config(&dir.path().join("db").to_string_lossy());

    match rollback_snapshot(config, "nope") {
        Err(Error::NotFound { .. }) => {}
        other => panic!("expected NotFound for missing snapshot, got {other:?}"),
    }
}
