// ponytail: integration test unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]

//! MEMG-04 — cuota de records por namespace (multi-tenant, capa core).
//!
//! Locks the contract of `Config::max_records_per_namespace`:
//! - default `None` → unlimited (byte-identical behavior);
//! - fresh inserts over the limit are rejected with `Error::ResourceLimit`
//!   and an explicit message;
//! - updates of an existing key at the limit are allowed;
//! - deletes free quota slots;
//! - batches are pre-checked (all-or-nothing, no partial writes) and count
//!   only fresh keys;
//! - the rejection is audited (`put` err with a quota reason).
//!
//! The quota is a resource control (best-effort under concurrency) — the
//! security barrier is the namespace-scoped RBAC (tests/rbac_namespace.rs).

use std::path::PathBuf;

use vantadb::config::Config;
use vantadb::{BackendKind, Embedded, MemoryInput};

fn db_with_quota(limit: Option<u64>, audit_path: Option<PathBuf>) -> Embedded {
    Embedded::open_with_config(Config {
        storage_path: ":memory:".to_string(),
        backend_kind: BackendKind::InMemory,
        max_records_per_namespace: limit,
        audit_log_path: audit_path,
        ..Default::default()
    })
    .expect("open in-memory database")
}

fn put(db: &Embedded, ns: &str, key: &str) {
    db.put(MemoryInput::new(ns, key, "payload"))
        .unwrap_or_else(|e| panic!("put {ns}/{key} must succeed: {e}"));
}

#[test]
fn quota_disabled_by_default_allows_new_keys() {
    let db = db_with_quota(None, None);
    put(&db, "docs", "k1");
    put(&db, "docs", "k2");
    put(&db, "docs", "k3");
    assert!(db.get("docs", "k2").expect("get").is_some());
}

#[test]
fn quota_rejects_new_key_over_limit_with_explicit_error() {
    let db = db_with_quota(Some(2), None);
    put(&db, "docs", "k1");
    put(&db, "docs", "k2");

    let err = db
        .put(MemoryInput::new("docs", "k3", "payload"))
        .expect_err("third fresh key must be rejected");
    match err {
        vantadb::Error::ResourceLimit(msg) => {
            assert!(
                msg.contains("quota"),
                "quota error must be explicit, got: {msg}"
            );
        }
        other => panic!("expected ResourceLimit, got: {other:?}"),
    }

    // The rejected write must not exist and the limit must hold.
    assert!(db.get("docs", "k3").expect("get").is_none());
    assert!(db.get("docs", "k1").expect("get").is_some());
}

#[test]
fn quota_update_at_limit_is_allowed() {
    let db = db_with_quota(Some(2), None);
    put(&db, "docs", "k1");
    put(&db, "docs", "k2");

    let updated = db
        .put(MemoryInput::new("docs", "k1", "new payload"))
        .expect("updating an existing key must pass at the limit");
    assert_eq!(updated.version, 2);
}

#[test]
fn quota_delete_frees_slot() {
    let db = db_with_quota(Some(1), None);
    put(&db, "docs", "k1");
    assert!(db.delete("docs", "k1").expect("delete"));

    put(&db, "docs", "k2");
    assert!(db.get("docs", "k2").expect("get").is_some());
}

#[test]
fn quota_is_per_namespace() {
    let db = db_with_quota(Some(1), None);
    put(&db, "alpha", "k1");
    put(&db, "beta", "k1");

    let err = db
        .put(MemoryInput::new("alpha", "k2", "payload"))
        .expect_err("alpha is at its limit");
    assert!(matches!(err, vantadb::Error::ResourceLimit(_)));
}

#[test]
fn quota_batch_over_limit_rejected_without_partial_writes() {
    let db = db_with_quota(Some(2), None);
    let batch = vec![
        MemoryInput::new("docs", "k1", "a"),
        MemoryInput::new("docs", "k2", "b"),
        MemoryInput::new("docs", "k3", "c"),
    ];

    let err = db.put_batch(batch).expect_err("batch over quota must fail");
    assert!(matches!(err, vantadb::Error::ResourceLimit(_)));

    // All-or-nothing: none of the three records may exist.
    for key in ["k1", "k2", "k3"] {
        assert!(
            db.get("docs", key).expect("get").is_none(),
            "no partial writes: {key} must be absent"
        );
    }

    // A batch within the limit passes.
    let fitting = vec![
        MemoryInput::new("docs", "k1", "a"),
        MemoryInput::new("docs", "k2", "b"),
    ];
    assert_eq!(db.put_batch(fitting).expect("batch within limit").len(), 2);
}

#[test]
fn quota_batch_of_updates_does_not_count() {
    let db = db_with_quota(Some(2), None);
    put(&db, "docs", "k1");
    put(&db, "docs", "k2");

    let updates = vec![
        MemoryInput::new("docs", "k1", "a2"),
        MemoryInput::new("docs", "k2", "b2"),
    ];
    assert_eq!(
        db.put_batch(updates)
            .expect("updates at the limit must pass")
            .len(),
        2
    );
}

#[test]
fn quota_rejection_is_audited_with_quota_reason() {
    let dir = tempfile::tempdir().expect("tempdir");
    let audit_path = dir.path().join("audit.jsonl");
    let db = db_with_quota(Some(1), Some(audit_path.clone()));

    put(&db, "docs", "k1");
    let err = db
        .put(MemoryInput::new("docs", "k2", "payload"))
        .expect_err("second key must be rejected");
    assert!(matches!(err, vantadb::Error::ResourceLimit(_)));

    let raw = std::fs::read_to_string(&audit_path).unwrap_or_default();
    let denied = raw
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str::<serde_json::Value>(l).expect("audit jsonl line"))
        .find(|r| r["op"] == "put" && r["outcome"] == "err")
        .expect("quota rejection must be audited");
    let reason = denied["reason"].as_str().unwrap_or_default();
    assert!(
        reason.contains("quota"),
        "audit reason must name the quota rejection, got: {reason}"
    );
}
