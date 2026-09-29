// ponytail: blanket allow — unwraps with documented invariants; fixture paths are repo-relative.
#![allow(clippy::expect_used, clippy::unwrap_used)]

//! Importers (Mem0 / Zep / Letta → VantaDB `MemoryExportLine` v2) certification.
//!
//! Each importer is verified at three levels per source:
//! 1. mapping fidelity over a realistic (sanitized) fixture,
//! 2. tolerant-parser paths over minimal inline inputs,
//! 3. stable roundtrip: convert → import → export v2 → diff.

use std::fs;
use std::path::Path;

use tempfile::tempdir;
use vantadb::sdk::importers::{letta, mem0, zep, Conversion};
use vantadb::sdk::MemoryExportLine;
use vantadb::{ConfidenceClass, Embedded, MemoryListOptions, MemorySearchRequest, Value};

// ── helpers ──────────────────────────────────────────────────────────────

fn fixture(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/importers")
        .join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read fixture {}: {e}", path.display()))
}

fn str_value(s: &str) -> Value {
    Value::String(s.to_string())
}

fn ms(rfc3339: &str) -> u64 {
    chrono::DateTime::parse_from_rfc3339(rfc3339)
        .expect("rfc3339 fixture timestamp")
        .timestamp_millis() as u64
}

fn line_for<'a>(lines: &'a [MemoryExportLine], key: &str) -> &'a MemoryExportLine {
    lines
        .iter()
        .find(|l| l.key == key)
        .unwrap_or_else(|| panic!("no converted line with key {key}"))
}

/// Projection of a line whose equality across the export/import roundtrip is
/// required by the contract (wire fields are compared field by field).
fn canonical(line: &MemoryExportLine) -> serde_json::Value {
    serde_json::json!({
        "schema_version": line.schema_version,
        "namespace": line.namespace,
        "key": line.key,
        "payload": line.payload,
        "metadata": line.metadata,
        "created_at_ms": line.created_at_ms,
        "updated_at_ms": line.updated_at_ms,
        "valid_at_ms": line.valid_at_ms,
        "invalid_at_ms": line.invalid_at_ms,
        "confidence_class": line.confidence_class,
        "confidence": line.confidence,
        "derived_from": line.derived_from,
    })
}

/// Import every converted line into a fresh database, export it back as JSONL
/// and return the re-parsed lines sorted by key. Panics on any import error.
fn import_and_export(db_dir: &Path, export_dir: &Path, conv: &Conversion) -> Vec<MemoryExportLine> {
    let records = conv.clone().into_records().expect("into_records");
    assert_eq!(records.len(), conv.lines.len());

    let db = Embedded::open(db_dir).expect("open db");
    let report = db.import_records(records, false).expect("import records");
    assert_eq!(report.errors, 0, "import errors: {report:?}");
    assert_eq!(report.inserted, conv.lines.len() as u64);
    db.flush().expect("flush");

    let out = export_dir.join("roundtrip.jsonl");
    db.export_all(&out).expect("export all");
    let mut lines: Vec<MemoryExportLine> = fs::read_to_string(&out)
        .expect("read export")
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str::<MemoryExportLine>(l).expect("parse exported line"))
        .collect();
    lines.sort_by(|a, b| a.key.cmp(&b.key));
    lines
}

fn assert_roundtrip_stable(before: &[MemoryExportLine], after: &[MemoryExportLine]) {
    assert_eq!(after.len(), before.len(), "line count after roundtrip");
    let mut before_sorted: Vec<serde_json::Value> = before.iter().map(canonical).collect();
    before_sorted.sort_by_key(|v| v["key"].as_str().unwrap_or_default().to_string());
    let after_sorted: Vec<serde_json::Value> = after.iter().map(canonical).collect();
    assert_eq!(before_sorted, after_sorted, "v2 roundtrip diff");
}

// ── Mem0 ─────────────────────────────────────────────────────────────────

#[test]
fn mem0_export_array_maps_memories_with_provenance() {
    // Arrange
    let json = fixture("mem0/memories.json");

    // Act
    let conv = mem0::convert_str(&json).expect("convert mem0");

    // Assert
    assert_eq!(conv.stats.converted, 3);
    assert_eq!(conv.stats.skipped, 0);
    assert!(conv.lines.iter().all(|l| l.schema_version == 2));

    let line = line_for(&conv.lines, "0a1b2c3d-4e5f-4a6b-8c7d-000000000002");
    assert_eq!(line.namespace, "mem0/alice");
    assert_eq!(
        line.payload,
        "Alice is a staff engineer at Acme Corp, working on the billing pipeline."
    );
    assert_eq!(line.created_at_ms, ms("2025-11-03T09:15:30+00:00"));
    assert_eq!(line.updated_at_ms, ms("2025-12-01T10:00:00+00:00"));
    assert_eq!(line.valid_at_ms, line.created_at_ms);
    assert_eq!(line.confidence_class, ConfidenceClass::Asserted);
    assert_eq!(line.metadata.get("source"), Some(&str_value("mem0")));
    assert_eq!(
        line.metadata.get("source_id"),
        Some(&str_value("0a1b2c3d-4e5f-4a6b-8c7d-000000000002"))
    );
    assert_eq!(line.metadata.get("user_id"), Some(&str_value("alice")));
    assert_eq!(line.metadata.get("agent_id"), Some(&str_value("assistant")));
    assert_eq!(
        line.metadata.get("mem0.hash"),
        Some(&str_value("c0ffee1234567890"))
    );
    assert_eq!(
        line.metadata.get("mem0.categories"),
        Some(&Value::ListString(vec![
            "professional".to_string(),
            "work".to_string()
        ]))
    );
    match line.metadata.get("mem0.metadata") {
        Some(Value::String(raw)) => assert!(raw.contains("professional")),
        other => panic!("mem0.metadata should be a JSON string, got {other:?}"),
    }
}

#[test]
fn mem0_date_only_timestamp_is_normalized_and_score_discarded() {
    // Arrange
    let json = fixture("mem0/memories.json");

    // Act
    let conv = mem0::convert_str(&json).expect("convert mem0");

    // Assert: date-only "2026-01-15" → 2026-01-15T00:00:00Z
    let line = line_for(&conv.lines, "0a1b2c3d-4e5f-4a6b-8c7d-000000000003");
    assert_eq!(line.created_at_ms, ms("2026-01-15T00:00:00+00:00"));
    assert_eq!(line.updated_at_ms, line.created_at_ms);
    // The retrieval-time `score` has no v2 destination: discarded, counted.
    assert_eq!(conv.stats.discarded.get("score"), Some(&1));
}

#[test]
fn mem0_get_all_results_envelope_and_minimal_item_are_accepted() {
    // Arrange: the OSS `get_all()` envelope + an item with only the required field.
    let json = r#"{"results":[{"id":"m-1","memory":"User prefers dark mode"}]}"#;

    // Act
    let conv = mem0::convert_str(json).expect("convert mem0 envelope");

    // Assert
    assert_eq!(conv.lines.len(), 1);
    assert_eq!(conv.lines[0].key, "m-1");
    assert_eq!(conv.lines[0].payload, "User prefers dark mode");
    assert_eq!(
        conv.lines[0].created_at_ms, 0,
        "missing timestamp → 0 sentinel"
    );
    assert_eq!(conv.lines[0].valid_at_ms, 0);
}

#[test]
fn mem0_cli_agent_envelope_and_text_content_aliases_are_accepted() {
    // Arrange: CLI `--agent` envelope + `mem0 import` payload aliases.
    let json = r#"{"status":"success","command":"list","data":[
        {"id":"d-1","text":"alias text payload","created_at":"2025-01-02T03:04:05Z"},
        {"content":"alias content payload"}
    ]}"#;

    // Act
    let conv = mem0::convert_str(json).expect("convert mem0 cli envelope");

    // Assert
    assert_eq!(conv.lines.len(), 2);
    assert_eq!(line_for(&conv.lines, "d-1").payload, "alias text payload");
    // Item without id: deterministic content-hash key, prefixed for source.
    let hashed = conv
        .lines
        .iter()
        .find(|l| l.key.starts_with("mem0-"))
        .expect("content-hash key for id-less memory");
    assert_eq!(hashed.payload, "alias content payload");
    assert_eq!(hashed.metadata.get("source_id"), None);
}

#[test]
fn mem0_item_without_payload_is_skipped_and_counted() {
    // Arrange
    let json = r#"[{"id":"x","metadata":{"k":"v"}},{"id":"y","memory":"   "}]"#;

    // Act
    let conv = mem0::convert_str(json).expect("convert mem0");

    // Assert
    assert!(conv.lines.is_empty());
    assert_eq!(conv.stats.skipped, 2);
    assert_eq!(conv.stats.converted, 0);
}

/// Fixture-backed cases shared by the cross-source re-import and determinism
/// tests: (name, converter, fixture, expected namespaces with record counts).
type ConvertFn = fn(&str) -> std::result::Result<Conversion, vantadb::error::Error>;

fn source_cases() -> Vec<(
    &'static str,
    ConvertFn,
    &'static str,
    Vec<(&'static str, usize)>,
)> {
    vec![
        (
            "mem0",
            mem0::convert_str,
            "mem0/memories.json",
            vec![("mem0/alice", 3)],
        ),
        (
            "zep",
            zep::convert_str,
            "zep/graph.json",
            vec![("zep/emily-painter", 4)],
        ),
        (
            "letta",
            letta::convert_str,
            "letta/agent.af",
            vec![("letta/blocks", 2), ("letta/assistant/messages", 3)],
        ),
    ]
}

#[test]
fn every_source_reimport_updates_instead_of_duplicating() {
    for (source, convert, fixture_path, namespaces) in source_cases() {
        // Arrange
        let db_dir = tempdir().expect("db tempdir");
        let conv = convert(&fixture(fixture_path)).expect("convert");
        let db = Embedded::open(db_dir.path()).expect("open db");
        let total = conv.lines.len() as u64;

        // Act: the same conversion imported twice.
        let first = db
            .import_records(conv.clone().into_records().expect("records"), false)
            .expect("first import");
        let second = db
            .import_records(conv.into_records().expect("records"), false)
            .expect("second import");

        // Assert: deterministic keys → update, never duplicate.
        assert_eq!(first.inserted, total, "{source}: first import inserts all");
        assert_eq!(second.inserted, 0, "{source}: re-import must not duplicate");
        assert_eq!(second.updated, total, "{source}: re-import updates all");
        for (namespace, expected) in namespaces {
            let page = db
                .list(namespace, MemoryListOptions::default())
                .expect("list");
            assert_eq!(
                page.records.len(),
                expected,
                "{source}: record count in {namespace}"
            );
        }
    }
}

#[test]
fn every_source_conversion_is_deterministic_across_runs() {
    for (source, convert, fixture_path, _) in source_cases() {
        // Arrange
        let json = fixture(fixture_path);

        // Act
        let a = convert(&json).expect("convert a");
        let b = convert(&json).expect("convert b");

        // Assert
        assert_eq!(
            serde_json::to_string(&a.lines).expect("serialize a"),
            serde_json::to_string(&b.lines).expect("serialize b"),
            "{source}: lines must be byte-identical across runs"
        );
        assert_eq!(a.stats, b.stats, "{source}: stats must be identical");
    }
}

// ── Zep ──────────────────────────────────────────────────────────────────

#[test]
fn zep_fixture_maps_episodes_and_facts_with_validity_and_parents() {
    // Arrange
    let json = fixture("zep/graph.json");

    // Act
    let conv = zep::convert_str(&json).expect("convert zep");

    // Assert: episodes assert with their creation time.
    assert_eq!(conv.stats.converted, 4);
    let ep = line_for(&conv.lines, "ep-11111111-1111-4111-8111-111111111111");
    assert_eq!(ep.namespace, "zep/emily-painter");
    assert_eq!(
        ep.payload,
        "Emily: I just started a new job at Acme Corp as a designer."
    );
    assert_eq!(ep.created_at_ms, ms("2025-09-02T14:00:00+00:00"));
    assert_eq!(ep.confidence_class, ConfidenceClass::Asserted);
    assert_eq!(ep.metadata.get("zep.kind"), Some(&str_value("episode")));
    assert_eq!(ep.metadata.get("zep.role"), Some(&str_value("user")));
    assert_eq!(
        ep.metadata.get("zep.thread_id"),
        Some(&str_value("thread-emily-1"))
    );

    // Assert: facts derive from episodes, preserving the valid-time window.
    let fact = line_for(&conv.lines, "ed-44444444-4444-4444-8444-444444444444");
    assert_eq!(fact.payload, "Emily likes Adidas shoes");
    assert_eq!(fact.confidence_class, ConfidenceClass::Derived);
    assert_eq!(
        fact.derived_from,
        vec![
            "ep-11111111-1111-4111-8111-111111111111".to_string(),
            "ep-22222222-2222-4222-8222-222222222222".to_string()
        ]
    );
    assert_eq!(fact.valid_at_ms, ms("2025-08-01T00:00:00+00:00"));
    assert_eq!(fact.invalid_at_ms, Some(ms("2025-09-10T08:30:00+00:00")));
    assert_eq!(fact.created_at_ms, ms("2025-09-10T08:30:05+00:00"));
    assert_eq!(fact.confidence, 0.9, "derived confidence = D_a × discount");
    assert_eq!(fact.metadata.get("zep.name"), Some(&str_value("LIKES")));
    assert_eq!(
        fact.metadata.get("zep.source_node_name"),
        Some(&str_value("Emily"))
    );
    assert_eq!(
        fact.metadata.get("zep.target_node_name"),
        Some(&str_value("Adidas"))
    );
    // Transaction-time `expired_at` has no v2 destination: discarded, counted.
    assert_eq!(conv.stats.discarded.get("expired_at"), Some(&1));
    // Both source episodes converted → no unresolved parent references.
    assert_eq!(conv.stats.discarded.get("fact.episodes_unresolved"), None);
}

#[test]
fn zep_fact_without_episodes_downgrades_to_asserted() {
    // Arrange: a fact with no source episodes cannot be `Derived`
    // (`derived_from` must be non-empty) → declared downgrade + counter.
    let json = r#"{"user_id":"u1","facts":[{"uuid":"f-1","name":"OWNS","fact":"User owns a bike","created_at":"2025-01-01T00:00:00Z"}]}"#;

    // Act
    let conv = zep::convert_str(json).expect("convert zep");

    // Assert
    let fact = line_for(&conv.lines, "f-1");
    assert_eq!(fact.confidence_class, ConfidenceClass::Asserted);
    assert!(fact.derived_from.is_empty());
    assert_eq!(conv.stats.discarded.get("fact.episodes_missing"), Some(&1));
}

#[test]
fn zep_edges_alias_and_missing_fields_are_handled() {
    // Arrange: `edges` is accepted as an alias for `facts`; episodes without
    // uuid/content are skipped.
    let json = r#"{
        "graph_id": "g-1",
        "episodes": [{"uuid": "e-ok", "content": "kept", "created_at": "2025-02-02T00:00:00Z"}, {"content": "no uuid"}],
        "edges": [{"uuid": "f-ok", "fact": "kept fact", "created_at": "2025-02-02T00:00:01Z", "valid_at": "2025-02-02T00:00:00Z"}]
    }"#;

    // Act
    let conv = zep::convert_str(json).expect("convert zep");

    // Assert
    assert_eq!(conv.lines.len(), 2);
    assert_eq!(conv.stats.skipped, 1);
    assert_eq!(conv.lines[0].namespace, "zep/g-1");
    assert_eq!(line_for(&conv.lines, "e-ok").payload, "kept");
    let fact = line_for(&conv.lines, "f-ok");
    assert_eq!(fact.payload, "kept fact");
    assert_eq!(fact.valid_at_ms, ms("2025-02-02T00:00:00+00:00"));
    assert_eq!(fact.invalid_at_ms, None);
    // `edges` was the chosen array (no `facts` present) → not a discard.
    assert_eq!(conv.stats.discarded.get("edges"), None);
}

#[test]
fn zep_edges_are_ignored_and_counted_when_facts_are_present() {
    // Arrange: both arrays present — `facts` wins, the parallel `edges` data
    // has no destination and is counted, not dropped silently.
    let json = r#"{
        "user_id": "u-1",
        "facts": [{"uuid": "f-1", "fact": "kept", "created_at": "2025-01-01T00:00:00Z"}],
        "edges": [
            {"uuid": "e-1", "fact": "ignored", "created_at": "2025-01-01T00:00:00Z"},
            {"uuid": "e-2", "fact": "ignored too", "created_at": "2025-01-01T00:00:00Z"}
        ]
    }"#;

    // Act
    let conv = zep::convert_str(json).expect("convert zep");

    // Assert
    assert_eq!(conv.lines.len(), 1);
    assert_eq!(conv.lines[0].key, "f-1");
    assert!(
        conv.lines.iter().all(|l| l.key != "e-1" && l.key != "e-2"),
        "edges entries must not convert when facts is present"
    );
    assert_eq!(conv.stats.discarded.get("edges"), Some(&2));
}

#[test]
fn zep_fact_parents_are_filtered_to_converted_episodes() {
    // Arrange: one fact mixes a converted episode with a ghost uuid; another
    // references only ghosts. `derived_from` must never dangle.
    let json = r#"{
        "user_id": "u-1",
        "episodes": [
            {"uuid": "e-1", "content": "kept", "created_at": "2025-01-01T00:00:00Z"},
            {"content": "no uuid — skipped"}
        ],
        "facts": [
            {"uuid": "f-1", "fact": "mixed parents", "created_at": "2025-01-01T00:00:01Z", "episodes": ["e-1", "ghost"]},
            {"uuid": "f-2", "fact": "all ghosts", "created_at": "2025-01-01T00:00:02Z", "episodes": ["ghost-2"]}
        ]
    }"#;

    // Act
    let conv = zep::convert_str(json).expect("convert zep");

    // Assert
    let f1 = line_for(&conv.lines, "f-1");
    assert_eq!(f1.confidence_class, ConfidenceClass::Derived);
    assert_eq!(f1.derived_from, vec!["e-1".to_string()]);
    let f2 = line_for(&conv.lines, "f-2");
    assert_eq!(f2.confidence_class, ConfidenceClass::Asserted);
    assert!(f2.derived_from.is_empty());
    assert_eq!(
        conv.stats.discarded.get("fact.episodes_unresolved"),
        Some(&2)
    );
    // The source list was non-empty for both facts: not `episodes_missing`.
    assert_eq!(conv.stats.discarded.get("fact.episodes_missing"), None);
}

// ── Letta ────────────────────────────────────────────────────────────────

#[test]
fn letta_agent_file_maps_blocks_and_messages() {
    // Arrange
    let json = fixture("letta/agent.af");

    // Act
    let conv = letta::convert_str(&json).expect("convert letta");

    // Assert: 2 blocks + 3 messages.
    assert_eq!(conv.stats.converted, 5);
    assert_eq!(conv.stats.skipped, 0);

    let block = line_for(&conv.lines, "block-00000000-0000-4000-8000-00000000000b");
    assert_eq!(block.namespace, "letta/blocks");
    assert_eq!(
        block.payload,
        "Name: Alice. Preference: short answers. Timezone: UTC-4."
    );
    assert_eq!(block.metadata.get("letta.label"), Some(&str_value("human")));
    assert_eq!(block.metadata.get("letta.limit"), Some(&Value::Int(5000)));
    assert_eq!(
        block.metadata.get("letta.read_only"),
        Some(&Value::Bool(false))
    );
    assert_eq!(
        block.metadata.get("letta.agents"),
        Some(&Value::ListString(vec!["assistant".to_string()]))
    );

    let msg = line_for(&conv.lines, "message-00000000-0000-4000-8000-000000000003");
    assert_eq!(msg.namespace, "letta/assistant/messages");
    assert_eq!(msg.payload, "Also, my timezone is UTC-4.");
    assert_eq!(msg.created_at_ms, ms("2025-10-02T09:30:00+00:00"));
    assert_eq!(msg.metadata.get("letta.role"), Some(&str_value("user")));
    assert_eq!(
        msg.metadata.get("letta.agent"),
        Some(&str_value("assistant"))
    );
    assert_eq!(
        msg.metadata.get("letta.in_context"),
        Some(&Value::Bool(true))
    );
}

#[test]
fn letta_non_text_messages_are_skipped_and_agent_config_is_declared_discarded() {
    // Arrange: a tool-call message without text content + agent config fields
    // that are not memory (system prompt) + non-empty file references.
    let json = r#"{
        "agents": [{
            "id": "agent-1",
            "name": "tool-bot",
            "system": "You are a tool runner.",
            "block_ids": [],
            "messages": [
                {"id": "m-1", "role": "tool", "content": null, "created_at": "2025-03-01T00:00:00Z"},
                {"id": "m-2", "role": "user", "content": "hello", "created_at": "2025-03-01T00:00:01Z"}
            ]
        }],
        "blocks": [],
        "files": [{"id": "file-1", "file_name": "notes.txt"}],
        "sources": [{"id": "src-1", "name": "docs"}]
    }"#;

    // Act
    let conv = letta::convert_str(json).expect("convert letta");

    // Assert
    assert_eq!(conv.lines.len(), 1);
    assert_eq!(conv.stats.converted, 1);
    assert_eq!(conv.stats.skipped, 1);
    assert_eq!(conv.stats.discarded.get("agent.system"), Some(&1));
    assert_eq!(conv.stats.discarded.get("files"), Some(&1));
    assert_eq!(conv.stats.discarded.get("sources"), Some(&1));
}

#[test]
fn letta_empty_and_text_part_content_arrays_are_handled() {
    // Arrange: the real upstream shapes (agent-file repo, `loop.af`):
    // - assistant tool-call messages use `content: []` + `tool_calls`;
    // - structured content uses parts `[{"type":"text","text":…}]`;
    // - non-text parts (images/tool artifacts) have no textual payload.
    let json = r#"{
        "agents": [{
            "id": "agent-1",
            "name": "loop",
            "block_ids": [],
            "messages": [
                {"id": "m-empty", "role": "assistant", "type": "message", "content": [], "tool_calls": [{"id": "t-1"}], "created_at": "2025-04-01T00:00:00Z"},
                {"id": "m-text", "role": "user", "type": "message", "content": [{"type": "text", "text": "You are Loop. You remember."}], "created_at": "2025-04-01T00:00:01Z"},
                {"id": "m-image", "role": "user", "type": "message", "content": [{"type": "image", "source": {"type": "base64"}}]}
            ]
        }],
        "blocks": []
    }"#;

    // Act
    let conv = letta::convert_str(json).expect("convert letta");

    // Assert: only the text part becomes a record; `[]` and text-less arrays
    // are skipped (declared), never stringified into a bogus `"[]"` payload.
    assert_eq!(conv.lines.len(), 1);
    assert_eq!(conv.stats.converted, 1);
    assert_eq!(conv.stats.skipped, 2);
    let msg = line_for(&conv.lines, "m-text");
    assert_eq!(msg.payload, "You are Loop. You remember.");
    assert_eq!(
        conv.stats.discarded.get("message.content_parts"),
        Some(&1),
        "the non-text image part is counted"
    );
}

// ── Roundtrips (import → export v2 → diff) ───────────────────────────────

#[test]
fn mem0_import_export_roundtrip_is_stable() {
    // Arrange
    let db_dir = tempdir().expect("db tempdir");
    let export_dir = tempdir().expect("export tempdir");
    let conv = mem0::convert_str(&fixture("mem0/memories.json")).expect("convert");
    let before = conv.lines.clone();

    // Act
    let after = import_and_export(db_dir.path(), export_dir.path(), &conv);

    // Assert
    assert_roundtrip_stable(&before, &after);
}

#[test]
fn zep_import_export_roundtrip_is_stable() {
    // Arrange
    let db_dir = tempdir().expect("db tempdir");
    let export_dir = tempdir().expect("export tempdir");
    let conv = zep::convert_str(&fixture("zep/graph.json")).expect("convert");
    let before = conv.lines.clone();

    // Act
    let after = import_and_export(db_dir.path(), export_dir.path(), &conv);

    // Assert
    assert_roundtrip_stable(&before, &after);
}

#[test]
fn letta_import_export_roundtrip_is_stable() {
    // Arrange
    let db_dir = tempdir().expect("db tempdir");
    let export_dir = tempdir().expect("export tempdir");
    let conv = letta::convert_str(&fixture("letta/agent.af")).expect("convert");
    let before = conv.lines.clone();

    // Act
    let after = import_and_export(db_dir.path(), export_dir.path(), &conv);

    // Assert
    assert_roundtrip_stable(&before, &after);
}

#[test]
fn zep_imported_facts_search_by_vector_after_embedding_attach() {
    // Arrange: conversion preserves payload/keys; a caller may attach vectors
    // before importing (export files carry no embeddings for any of the 3).
    let db_dir = tempdir().expect("db tempdir");
    let mut conv = zep::convert_str(&fixture("zep/graph.json")).expect("convert");
    for line in conv.lines.iter_mut() {
        line.vector = Some(vec![1.0, 0.0, 0.0]);
    }

    // Act
    let db = Embedded::open(db_dir.path()).expect("open db");
    db.import_records(conv.into_records().expect("records"), false)
        .expect("import");
    db.flush().expect("flush");

    // Assert: list + vector search return the imported records.
    let page = db
        .list("zep/emily-painter", MemoryListOptions::default())
        .expect("list");
    assert_eq!(page.records.len(), 4);
    let hits = db
        .search(MemorySearchRequest {
            namespace: "zep/emily-painter".to_string(),
            query_vector: vec![1.0, 0.0, 0.0],
            filters: Default::default(),
            text_query: None,
            top_k: 10,
            range: None,
            group_by: None,
            mmr: None,
            cursor: None,
            ..Default::default()
        })
        .expect("search");
    assert_eq!(hits.len(), 4);
}
