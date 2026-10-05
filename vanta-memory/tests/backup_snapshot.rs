// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! MEMG-13 — memory backup/restore over the core filesystem snapshot surface.
//! Fjall + tempdir (a snapshot needs an on-disk store; InMemory keeps no
//! files).
//!
//! What could break: the restore flow (close → swap → reopen) losing or
//! resurrecting records, the wrapper relaxing the core name validation, and
//! missing-snapshot errors losing their NotFound shape.

use vanta_memory::core::abstractions::{MemoryRecord, MemoryType};
use vanta_memory::core::record::l1_reader::l1_namespace;
use vanta_memory::utils::backup::{create_snapshot, list_snapshots, restore_snapshot};
use vantadb::config::Config;
use vantadb::sdk::{Embedded, MemoryInput, MemoryMetadata};
use vantadb::storage::BackendKind;

const SESSION: &str = "sess-backup";

fn fjall_config(storage_path: &str) -> Config {
    Config {
        storage_path: storage_path.to_string(),
        backend_kind: BackendKind::Fjall,
        ..Config::default()
    }
}

fn record(id: &str, content: &str) -> MemoryRecord {
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
        version: 1,
        session_key: SESSION.into(),
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

fn read_contents(db: &Embedded) -> Vec<String> {
    let mut contents: Vec<String> = vanta_memory::core::record::read_session_records(db, SESSION)
        .expect("read l1")
        .into_iter()
        .map(|r| r.content)
        .collect();
    contents.sort();
    contents
}

#[test]
fn snapshot_round_trips_l1_records() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("db").to_string_lossy().to_string();
    let config = fjall_config(&db_path);

    {
        let db = Embedded::open_with_config(config.clone()).expect("open fjall");
        put_l1(&db, &record("m1", "alpha"));
        put_l1(&db, &record("m2", "beta"));

        let snap = create_snapshot(&db, "snap-1").expect("create snapshot");
        assert!(snap.path.exists(), "snapshot dir materialized");
        assert_eq!(
            list_snapshots(&db).expect("list"),
            vec!["snap-1".to_string()],
            "created snapshot is listed"
        );

        // Post-snapshot addition — must NOT survive the restore (certified
        // core contract: snapshot → mutate → close → restore → reopen).
        put_l1(&db, &record("m3", "gamma"));
        db.close().expect("close before restore");
    }

    let db = restore_snapshot(config, "snap-1").expect("restore");
    assert_eq!(
        read_contents(&db),
        vec!["alpha".to_string(), "beta".to_string()],
        "pre-snapshot records back; post-snapshot addition gone"
    );
}

/// FIND-287 canary: `snapshot_restore` swaps `data/` only — the live backend
/// KV (Fjall LSM) is intentionally left in place (FIND-33 decision, archived
/// plan 2026-08-29 §1308), so post-snapshot DELETE and supersession metadata
/// keep their backend state and are NOT rolled back. If the core ever
/// restores the backend too, this test fails and FIND-287 gets re-evaluated.
#[test]
fn restore_does_not_roll_back_post_snapshot_deletes() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("db").to_string_lossy().to_string();
    let config = fjall_config(&db_path);

    {
        let db = Embedded::open_with_config(config.clone()).expect("open fjall");
        put_l1(&db, &record("m1", "alpha"));
        put_l1(&db, &record("m2", "beta"));
        create_snapshot(&db, "snap-del").expect("create snapshot");

        // Post-snapshot mutations: an addition, a delete, a supersession.
        put_l1(&db, &record("m3", "gamma"));
        db.supersede(&l1_namespace(SESSION), "m2", "m3")
            .expect("supersede m2");
        db.delete(&l1_namespace(SESSION), "m1").expect("delete m1");
        db.close().expect("close before restore");
    }

    let db = restore_snapshot(config, "snap-del").expect("restore");
    assert_eq!(
        read_contents(&db),
        vec!["beta".to_string()],
        "backend tombstone survives the data/-only restore (FIND-287)"
    );
    let m2 = db
        .get(&l1_namespace(SESSION), "m2")
        .expect("get m2")
        .expect("m2 present");
    assert_eq!(
        m2.superseded_by.as_deref(),
        Some("m3"),
        "post-snapshot supersession keeps its backend metadata (FIND-287)"
    );
}

#[test]
fn snapshot_name_validation_is_not_relaxed() {
    let dir = tempfile::tempdir().expect("tempdir");
    let config = fjall_config(&dir.path().join("db").to_string_lossy());
    let db = Embedded::open_with_config(config).expect("open fjall");

    let err = create_snapshot(&db, "../evil").expect_err("traversal name must fail");
    assert!(
        matches!(err, vantadb::error::Error::InvalidInput(_)),
        "core validation surfaces as InvalidInput: {err:?}"
    );
}

#[test]
fn restore_missing_snapshot_is_not_found() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("db").to_string_lossy().to_string();
    let config = fjall_config(&db_path);
    {
        let db = Embedded::open_with_config(config.clone()).expect("open fjall");
        assert!(list_snapshots(&db).expect("list").is_empty());
        db.close().expect("close");
    }

    let err = restore_snapshot(config, "nope").expect_err("missing snapshot must fail");
    assert!(
        matches!(err, vantadb::error::Error::NotFound { .. }),
        "missing snapshot surfaces as NotFound: {err:?}"
    );
}
