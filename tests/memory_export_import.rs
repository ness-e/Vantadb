// ponytail: blanket allow — unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]

//! JSONL export/import certification for persistent memory APIs.

use std::fs;
use std::path::Path;
use tempfile::tempdir;
use vantadb::{Embedded, MemoryInput, MemoryListOptions, MemorySearchRequest, Value};

fn str_value(value: &str) -> Value {
    Value::String(value.to_string())
}

fn record(namespace: &str, key: &str, payload: &str, category: &str) -> MemoryInput {
    let mut input = MemoryInput::new(namespace, key, payload);
    input
        .metadata
        .insert("category".to_string(), str_value(category));
    input.vector = Some(vec![1.0, 0.0, 0.0]);
    input
}

fn copy_dir_all(source: &Path, target: &Path) {
    fs::create_dir_all(target).expect("create restore dir");
    for entry in fs::read_dir(source).expect("read source dir") {
        let entry = entry.expect("source entry");
        let file_type = entry.file_type().expect("entry type");
        let target_path = target.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &target_path);
        } else {
            fs::copy(entry.path(), target_path).expect("copy file");
        }
    }
}

#[test]
fn export_import_namespace_round_trip() {
    let source_dir = tempdir().expect("source tempdir");
    let target_dir = tempdir().expect("target tempdir");
    let export_path = source_dir.path().join("agent-main.jsonl");

    let source = Embedded::open(source_dir.path()).expect("open source");
    source
        .put(record("agent/main", "a", "alpha memory", "task"))
        .expect("put a");
    source
        .put(record("agent/main", "b", "beta memory", "note"))
        .expect("put b");
    source
        .put(record("agent/other", "c", "outside namespace", "task"))
        .expect("put c");
    source.flush().expect("flush source");

    let export = source
        .export_namespace(&export_path, "agent/main", None)
        .expect("export namespace");
    assert_eq!(export.records_exported, 2);
    assert_eq!(export.namespaces, vec!["agent/main".to_string()]);

    let target = Embedded::open(target_dir.path()).expect("open target");
    let import = target
        .import_file(&export_path, false)
        .expect("import file");
    assert_eq!(import.inserted, 2);
    assert_eq!(import.updated, 0);
    assert_eq!(import.errors, 0);

    let fetched = target.get("agent/main", "a").expect("get").expect("record");
    assert_eq!(fetched.payload, "alpha memory");
    assert_eq!(fetched.metadata.get("category"), Some(&str_value("task")));

    let page = target
        .list("agent/main", MemoryListOptions::default())
        .expect("list");
    assert_eq!(page.records.len(), 2);

    let hits = target
        .search(MemorySearchRequest {
            namespace: "agent/main".to_string(),
            query_vector: vec![1.0, 0.0, 0.0],
            filters: Default::default(),
            text_query: None,
            top_k: 5,
            range: None,
            group_by: None,
            mmr: None,
            cursor: None,
            ..Default::default()
        })
        .expect("search");
    assert_eq!(hits.len(), 2);
    assert!(target
        .get("agent/other", "c")
        .expect("get outside namespace")
        .is_none());
}

#[test]
fn export_all_import_updates_existing_records() {
    let source_dir = tempdir().expect("source tempdir");
    let target_dir = tempdir().expect("target tempdir");
    let export_path = source_dir.path().join("all.jsonl");

    let source = Embedded::open(source_dir.path()).expect("open source");
    source
        .put(record("agent/main", "a", "alpha memory", "task"))
        .expect("put a");
    source
        .put(record("agent/other", "a", "other alpha", "note"))
        .expect("put other");

    let export = source.export_all(&export_path).expect("export all");
    assert_eq!(export.records_exported, 2);

    let target = Embedded::open(target_dir.path()).expect("open target");
    target
        .put(record("agent/main", "a", "stale alpha", "task"))
        .expect("seed stale");

    let import = target
        .import_file(&export_path, false)
        .expect("import file");
    assert_eq!(import.inserted, 1);
    assert_eq!(import.updated, 1);
    assert_eq!(import.errors, 0);

    let updated = target
        .get("agent/main", "a")
        .expect("get updated")
        .expect("record");
    assert_eq!(updated.payload, "alpha memory");
    assert_eq!(updated.version, 1);
}

#[test]
fn fjall_cold_copy_restore_preserves_memory_text_and_hybrid_search() {
    let source_dir = tempdir().expect("source tempdir");
    let restore_parent = tempdir().expect("restore parent");
    let restore_path = restore_parent.path().join("restored-db");

    {
        let source = Embedded::open(source_dir.path()).expect("open source");
        let mut input = record("agent/main", "restore", "restore alpha phrase", "backup");
        input.vector = Some(vec![1.0, 0.0, 0.0]);
        source.put(input).expect("put restore");
        source.flush().expect("flush source");
        source.close().expect("close source before cold copy");
    }

    copy_dir_all(source_dir.path(), &restore_path);

    let restored = Embedded::open(&restore_path).expect("open restored");
    let fetched = restored
        .get("agent/main", "restore")
        .expect("get restored")
        .expect("restored record");
    assert_eq!(fetched.payload, "restore alpha phrase");

    let audit = restored
        .audit_text_index(Some("agent/main"))
        .expect("audit restored text index");
    assert!(audit.passed);

    let text_hits = restored
        .search(MemorySearchRequest {
            namespace: "agent/main".to_string(),
            query_vector: Vec::new(),
            filters: Default::default(),
            text_query: Some("\"restore alpha\"".to_string()),
            top_k: 5,
            range: None,
            group_by: None,
            mmr: None,
            cursor: None,
            ..Default::default()
        })
        .expect("restored text search");
    assert_eq!(text_hits[0].record.key, "restore");

    let hybrid_hits = restored
        .search(MemorySearchRequest {
            namespace: "agent/main".to_string(),
            query_vector: vec![1.0, 0.0, 0.0],
            filters: Default::default(),
            text_query: Some("restore".to_string()),
            top_k: 5,
            range: None,
            group_by: None,
            mmr: None,
            cursor: None,
            ..Default::default()
        })
        .expect("restored hybrid search");
    assert_eq!(hybrid_hits[0].record.key, "restore");
}

// ─── ADR-046 (SCH-02): v2 schema — v1 import, roundtrip, reopen ─────

use vantadb::sdk::{default_confidence, ConfidenceClass};

#[test]
fn import_v1_jsonl_line_normalizes_to_v2_defaults() {
    let dir = tempdir().expect("tempdir");
    // A real v1 export line (13 fields, schema_version 1) with supersession.
    let v1_line = r#"{"schema_version":1,"namespace":"legacy","key":"a","payload":"old data","metadata":{},"vector":null,"created_at_ms":1000,"updated_at_ms":2000,"version":2,"expires_at_ms":null,"superseded_by":"b","superseded_at_ms":1500}"#;
    let path = dir.path().join("v1.jsonl");
    fs::write(&path, format!("{v1_line}\n")).expect("write v1 fixture");

    let db = Embedded::open(dir.path()).expect("open");
    let report = db.import_file(&path, false).expect("import v1");
    assert_eq!(report.inserted, 1);
    assert_eq!(report.errors, 0);

    let record = db.get("legacy", "a").expect("get").expect("record");
    assert_eq!(
        record.valid_at_ms, 1000,
        "v1 normalization: valid_at := created_at"
    );
    assert_eq!(
        record.invalid_at_ms,
        Some(1500),
        "v1 normalization: invalid_at := superseded_at"
    );
    assert_eq!(record.confidence_class, ConfidenceClass::Asserted);
    assert_eq!(record.confidence, default_confidence());
    assert_eq!(record.quarantined_at_ms, None);
}

#[test]
fn v2_fields_survive_export_import_roundtrip() {
    let source_dir = tempdir().expect("source");
    let target_dir = tempdir().expect("target");
    let export_path = source_dir.path().join("v2.jsonl");

    {
        let source = Embedded::open(source_dir.path()).expect("open source");
        source
            .put(MemoryInput {
                confidence: Some(0.8),
                ..MemoryInput::new("ns/v2", "parent", "parent payload")
            })
            .expect("put parent");
        let derived = source
            .put(MemoryInput {
                confidence_class: Some(ConfidenceClass::Derived),
                derived_from: Some(vec!["parent".to_string()]),
                ..MemoryInput::new("ns/v2", "child", "derived payload")
            })
            .expect("put derived");
        assert_eq!(derived.confidence, 0.8 * 0.9);
        source.flush().expect("flush");
        source.export_all(&export_path).expect("export");
    }

    {
        let target = Embedded::open(target_dir.path()).expect("open target");
        let report = target.import_file(&export_path, false).expect("import v2");
        assert_eq!(report.inserted, 2);
        assert_eq!(report.errors, 0);

        let child = target.get("ns/v2", "child").expect("get").expect("child");
        assert_eq!(child.confidence_class, ConfidenceClass::Derived);
        assert_eq!(child.confidence, 0.8 * 0.9, "v2 score is transported");
        assert_eq!(child.derived_from, vec!["parent".to_string()]);
        assert_eq!(child.valid_at_ms, child.created_at_ms);

        // Re-export → import into a third DB keeps the fields (idempotent wire).
        let reexport = target_dir.path().join("reexport.jsonl");
        target.export_all(&reexport).expect("re-export");
        let third_dir = tempdir().expect("third");
        let third = Embedded::open(third_dir.path()).expect("open third");
        third.import_file(&reexport, false).expect("re-import");
        let again = third.get("ns/v2", "child").expect("get").expect("child");
        assert_eq!(again, child);
    }
}

#[test]
fn reopen_preserves_v2_fields() {
    let dir = tempdir().expect("tempdir");
    {
        let db = Embedded::open(dir.path()).expect("open");
        db.put(MemoryInput {
            confidence: Some(0.9),
            valid_at_ms: Some(1234),
            ..MemoryInput::new("ns/reopen", "a", "payload a")
        })
        .expect("put a");
        db.put(MemoryInput {
            confidence_class: Some(ConfidenceClass::Derived),
            derived_from: Some(vec!["a".to_string()]),
            ..MemoryInput::new("ns/reopen", "b", "payload b")
        })
        .expect("put b");
        db.put(MemoryInput::new("ns/reopen", "c", "payload c"))
            .expect("put c");
        db.supersede("ns/reopen", "c", "a").expect("supersede c");
        db.flush().expect("flush");
        db.close().expect("close");
    }

    let db = Embedded::open(dir.path()).expect("reopen");
    let a = db.get("ns/reopen", "a").expect("get a").expect("a");
    assert_eq!(a.confidence, 0.9);
    assert_eq!(a.valid_at_ms, 1234);
    assert_eq!(a.confidence_class, ConfidenceClass::Asserted);

    let b = db.get("ns/reopen", "b").expect("get b").expect("b");
    assert_eq!(b.confidence_class, ConfidenceClass::Derived);
    assert_eq!(b.confidence, 0.9 * 0.9, "score survives reopen");
    assert_eq!(b.derived_from, vec!["a".to_string()]);

    let c = db.get("ns/reopen", "c").expect("get c").expect("c");
    assert_eq!(
        c.invalid_at_ms, c.superseded_at_ms,
        "D3 alignment survives reopen"
    );
    assert!(c.valid_at_ms <= c.invalid_at_ms.expect("superseded has invalid_at"));

    // Version history mirror roundtrips the v2 fields too (ADR-046 §D7 #2).
    let versions = db.versions("ns/reopen", "a").expect("versions");
    assert_eq!(versions.len(), 1);
    assert_eq!(versions[0].confidence, 0.9);
    assert_eq!(versions[0].valid_at_ms, 1234);
}

#[test]
fn bulk_import_v0x01_payload_still_imports_with_v2_defaults() {
    let dir = tempdir().expect("tempdir");
    // v0x01 bulk payload: MemoryInput JSON WITHOUT any v2 field (pre-SCH-02 wire).
    let inputs = serde_json::json!([
        {"namespace":"bulk","key":"k1","payload":"p1","metadata":{},"vector":null,"ttl_ms":null},
        {"namespace":"bulk","key":"k2","payload":"p2","metadata":{},"vector":null,"ttl_ms":null}
    ]);
    let body = serde_json::to_vec(&inputs).expect("serialize inputs");
    let mut framed = Vec::new();
    framed.extend_from_slice(b"VDBJSON\n");
    framed.push(0x01);
    framed.extend_from_slice(&(2u64).to_le_bytes());
    framed.extend_from_slice(&body);
    let path = dir.path().join("bulk.vdbdump");
    fs::write(&path, &framed).expect("write bulk file");

    let db = Embedded::open(dir.path()).expect("open");
    let report = db
        .bulk_import_file(path.to_str().unwrap())
        .expect("bulk import");
    assert_eq!(report.total_records, 2);

    let record = db.get("bulk", "k1").expect("get").expect("record");
    assert_eq!(
        record.confidence,
        default_confidence(),
        "v0x01 payload ⇒ D_a"
    );
    assert_eq!(record.confidence_class, ConfidenceClass::Asserted);
    assert_eq!(record.valid_at_ms, record.created_at_ms);
    assert_eq!(record.invalid_at_ms, None);
}

#[test]
fn bulk_import_rejects_explicit_zero_valid_at() {
    // N2: boundary alignment — `Some(0)` is rejected on the validated put path
    // (0 = v1 unset sentinel); the raw bulk path must not silently reinterpret
    // it as "default".
    let dir = tempdir().expect("tempdir");
    let inputs = serde_json::json!([
        {"namespace":"bulk","key":"z","payload":"p","metadata":{},"vector":null,"ttl_ms":null,"valid_at_ms":0}
    ]);
    let body = serde_json::to_vec(&inputs).expect("serialize inputs");
    let mut framed = Vec::new();
    framed.extend_from_slice(b"VDBJSON\n");
    framed.push(0x01);
    framed.extend_from_slice(&(1u64).to_le_bytes());
    framed.extend_from_slice(&body);
    let path = dir.path().join("bulk-zero.vdbdump");
    fs::write(&path, &framed).expect("write bulk file");

    let db = Embedded::open(dir.path()).expect("open");
    let err = db
        .bulk_import_file(path.to_str().unwrap())
        .expect_err("explicit valid_at_ms=0 must be rejected");
    assert!(
        err.to_string().contains("valid_at_ms"),
        "error must name the field, got: {err}"
    );
    assert!(db.get("bulk", "z").expect("get").is_none());
}

// ─── SCH-06: roundtrip v1↔v2 (D7) — fixture v1 + wire v2 full-field ─────

#[test]
fn v1_fixture_re_exports_as_normalized_v2_and_reimports_lossless() {
    let source_dir = tempdir().expect("source");
    let out_dir = tempdir().expect("out");
    let third_dir = tempdir().expect("third");
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("export-v1.jsonl");

    // v1 sigue importable (D7) — 2 lines, one superseded.
    let source = Embedded::open(source_dir.path()).expect("open source");
    let report = source.import_file(&fixture, false).expect("import v1");
    assert_eq!(report.inserted, 2);
    assert_eq!(report.errors, 0);

    let alpha = source
        .get("legacy", "v1-alpha")
        .expect("get")
        .expect("alpha");
    assert_eq!(
        alpha.valid_at_ms, 1000,
        "v1 normaliza valid_at := created_at"
    );
    assert_eq!(alpha.confidence, default_confidence());
    let superseded = source
        .get("legacy", "v1-superseded")
        .expect("get")
        .expect("superseded");
    assert_eq!(
        superseded.invalid_at_ms,
        Some(1500),
        "JSONL v1 normaliza invalid_at := superseded_at (D7 #3)"
    );

    // Re-export: always v2, with the normalized values materialized.
    let reexport = out_dir.path().join("reexport.jsonl");
    source.export_all(&reexport).expect("re-export");
    let content = fs::read_to_string(&reexport).expect("read re-export");
    let mut seen_superseded = false;
    for line in content.lines() {
        let parsed: serde_json::Value = serde_json::from_str(line).expect("parse line");
        assert_eq!(
            parsed["schema_version"].as_u64(),
            Some(2),
            "export always v2"
        );
        if parsed["key"] == serde_json::json!("v1-superseded") {
            seen_superseded = true;
            assert_eq!(parsed["invalid_at_ms"].as_u64(), Some(1500));
            assert_eq!(parsed["confidence"].as_f64(), Some(1.0));
            assert_eq!(parsed["valid_at_ms"].as_u64(), Some(1000));
        }
    }
    assert!(seen_superseded, "the superseded fixture line re-exports");

    // ida y vuelta: re-import the v2 export — normalized values survive.
    let third = Embedded::open(third_dir.path()).expect("open third");
    let import = third.import_file(&reexport, false).expect("re-import");
    assert_eq!(import.inserted, 2);
    assert_eq!(import.errors, 0);
    assert_eq!(
        third.get("legacy", "v1-superseded").expect("get").unwrap(),
        superseded
    );
}

#[test]
fn all_new_v2_fields_survive_wire_roundtrip() {
    let source_dir = tempdir().expect("source");
    let mid_dir = tempdir().expect("mid");
    let final_dir = tempdir().expect("final");

    // Hand-written v2 wire lines (schema_version 2). `rich` carries every new
    // field populated; `derived` carries the computed-score transport shape.
    let lines = [
        r#"{"schema_version":2,"namespace":"wire","key":"rich","payload":"wire payload","metadata":{},"vector":[0.5,0.25],"created_at_ms":1000,"updated_at_ms":2000,"version":3,"expires_at_ms":null,"superseded_by":"succ","superseded_at_ms":2222,"valid_at_ms":1111,"invalid_at_ms":2222,"confidence_class":"Asserted","confidence":0.42,"last_validated_at_ms":3333,"derived_from":[],"quarantined_at_ms":4444,"quarantine_reason":"explicit_write","quarantined_by":"system:test","quarantine_review_due_ms":5555}"#,
        r#"{"schema_version":2,"namespace":"wire","key":"derived","payload":"derived payload","metadata":{},"vector":null,"created_at_ms":3000,"updated_at_ms":3000,"version":1,"expires_at_ms":null,"superseded_by":null,"superseded_at_ms":null,"valid_at_ms":3000,"invalid_at_ms":null,"confidence_class":"Derived","confidence":0.378,"last_validated_at_ms":null,"derived_from":["rich"],"quarantined_at_ms":null,"quarantine_reason":null,"quarantined_by":null,"quarantine_review_due_ms":null}"#,
    ];
    let wire_path = source_dir.path().join("wire-v2.jsonl");
    fs::write(&wire_path, lines.join("\n") + "\n").expect("write wire lines");

    let source = Embedded::open(source_dir.path()).expect("open source");
    let report = source.import_file(&wire_path, false).expect("import v2");
    assert_eq!(report.inserted, 2);
    assert_eq!(report.errors, 0);

    let rich = source.get("wire", "rich").expect("get").expect("rich");
    assert_eq!(rich.valid_at_ms, 1111);
    assert_eq!(rich.invalid_at_ms, Some(2222));
    assert_eq!(rich.confidence, 0.42);
    assert_eq!(rich.last_validated_at_ms, Some(3333));
    assert_eq!(rich.quarantined_at_ms, Some(4444));
    assert_eq!(rich.quarantine_reason.as_deref(), Some("explicit_write"));
    assert_eq!(rich.quarantined_by.as_deref(), Some("system:test"));
    assert_eq!(rich.quarantine_review_due_ms, Some(5555));
    let derived = source
        .get("wire", "derived")
        .expect("get")
        .expect("derived");
    assert_eq!(derived.confidence_class, ConfidenceClass::Derived);
    assert_eq!(derived.confidence, 0.378);
    assert_eq!(derived.derived_from, vec!["rich".to_string()]);

    // Quarantine survives the wire and stays visible-with-state on get.
    assert_eq!(
        source
            .get("wire", "rich")
            .expect("get")
            .expect("rich")
            .quarantined_at_ms,
        Some(4444)
    );

    // ida y vuelta ×2: export → import → export → import, full struct equality.
    let export1 = source_dir.path().join("export1.jsonl");
    source.export_all(&export1).expect("export 1");
    let mid = Embedded::open(mid_dir.path()).expect("open mid");
    mid.import_file(&export1, false).expect("import mid");
    assert_eq!(mid.get("wire", "rich").expect("get").unwrap(), rich);
    assert_eq!(mid.get("wire", "derived").expect("get").unwrap(), derived);

    let export2 = mid_dir.path().join("export2.jsonl");
    mid.export_all(&export2).expect("export 2");
    let final_db = Embedded::open(final_dir.path()).expect("open final");
    final_db.import_file(&export2, false).expect("import final");
    assert_eq!(final_db.get("wire", "rich").expect("get").unwrap(), rich);
    assert_eq!(
        final_db.get("wire", "derived").expect("get").unwrap(),
        derived
    );
}
