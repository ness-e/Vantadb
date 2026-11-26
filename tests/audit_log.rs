// ponytail: blanket allow — unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Integration tests for the append-only JSONL audit log (TSK-107b).

use serde_json::Value;
use vantadb::config::Config;
use vantadb::{BackendKind, Embedded, MemoryInput};

fn audit_config(audit_path: &std::path::Path) -> Config {
    Config {
        storage_path: ":memory:".into(),
        backend_kind: BackendKind::InMemory,
        audit_log_path: Some(audit_path.to_path_buf()),
        ..Default::default()
    }
}

#[test]
fn audit_log_records_operations_with_timestamp_op_and_reason() {
    let dir = tempfile::tempdir().unwrap();
    let audit_path = dir.path().join("sub/audit.jsonl");

    let db = Embedded::open_with_config(audit_config(&audit_path)).unwrap();

    db.put(MemoryInput::new("docs", "a", "hello")).unwrap();
    db.put(MemoryInput::new("docs", "b", "world")).unwrap();
    assert!(db.delete("docs", "a").unwrap());
    db.search(vantadb::MemorySearchRequest {
        namespace: "docs".into(),
        text_query: Some("hello".into()),
        top_k: 5,
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
        ..Default::default()
    })
    .unwrap();

    drop(db); // flush + close the audit writer

    let content = std::fs::read_to_string(&audit_path).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    // put ×2 + delete ×1 (search is read-only and not audited)
    assert_eq!(lines.len(), 3, "expected 3 audit lines, got: {content}");

    let mut saw_delete_reason = false;
    for line in &lines {
        let v: Value = serde_json::from_str(line).expect("each audit line must parse as JSON");
        let timestamp = v["timestamp"].as_str().expect("timestamp field");
        assert!(!timestamp.is_empty(), "timestamp must not be empty");
        // ISO 8601 shape: YYYY-MM-DDTHH:MM:SSZ
        assert!(
            timestamp.ends_with('Z'),
            "timestamp not ISO 8601: {timestamp}"
        );
        assert!(v["op"].is_string(), "op field");
        assert!(v["namespace"].is_string(), "namespace field");
        assert!(v["key"].is_string(), "key field");
        assert!(v["outcome"].is_string(), "outcome field");
        match v["op"].as_str().unwrap() {
            "put" => {}
            "delete" => {
                assert_eq!(v["key"], "a");
                assert_eq!(v["reason"], "memory delete", "delete must carry a reason");
                saw_delete_reason = true;
            }
            other => panic!("unexpected op in audit log: {other}"),
        }
    }
    assert!(saw_delete_reason, "delete event with reason not found");
}

#[test]
fn audit_log_not_created_when_unconfigured() {
    let dir = tempfile::tempdir().unwrap();
    let audit_path = dir.path().join("nope/audit.jsonl");

    let db = Embedded::open_with_config(Config {
        storage_path: ":memory:".into(),
        backend_kind: BackendKind::InMemory,
        audit_log_path: None,
        ..Default::default()
    })
    .unwrap();

    db.put(MemoryInput::new("docs", "a", "hello")).unwrap();
    db.delete("docs", "a").unwrap();
    drop(db);

    assert!(
        !audit_path.exists(),
        "no audit file should be created without audit_log_path"
    );
}

#[test]
fn test_audit_merkle_trail_verification_and_tamper_detection() {
    let dir = tempfile::tempdir().unwrap();
    let audit_path = dir.path().join("audit.jsonl");

    let db = Embedded::open_with_config(audit_config(&audit_path)).unwrap();

    // Generate 100 audit events through database operations
    for i in 0..50 {
        db.put(MemoryInput::new("ns", format!("k{i}"), "val"))
            .unwrap();
        assert!(db.delete("ns", &format!("k{i}")).unwrap());
    }

    drop(db); // Flush and close

    // 1. Verify that 100 events were written with valid Merkle chain
    let report = vantadb::audit::verify_merkle_trail(&audit_path).expect("audit trail must verify");
    assert_eq!(report.events_verified, 100);
    assert!(!report.head_hash.is_empty());

    // 2. Tamper with a single byte in line 35
    let content = std::fs::read_to_string(&audit_path).unwrap();
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    assert_eq!(lines.len(), 100);

    // Flip a value in line 35 to test cryptographic tamper detection
    let original = &lines[34];
    let tampered = original.replace("\"outcome\":\"ok\"", "\"outcome\":\"no\"");
    assert_ne!(original, &tampered);
    lines[34] = tampered;

    let tampered_path = dir.path().join("tampered_audit.jsonl");
    std::fs::write(&tampered_path, lines.join("\n") + "\n").unwrap();

    // 3. Verification must fail and report line 35
    let err =
        vantadb::audit::verify_merkle_trail(&tampered_path).expect_err("tampered log must fail");
    assert_eq!(err.line_number, 35);
    assert!(err.reason.contains("tampering") || err.reason.contains("broken chain"));
}
