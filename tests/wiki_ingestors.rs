#![allow(clippy::expect_used, clippy::unwrap_used)]
//! MEMG-08 (MGR-25): document ingestors end-to-end — scan a directory of
//! txt/json/csv sources into `MemoryInput` chunks with mandatory provenance
//! (`source`/`page`/`chunk`), ingest them into the engine and retrieve them.
//!
//! Contract under test (plan Task 52):
//! - format → chunks `MemoryInput` (trait `Ingestor`, registry `default_ingestors`)
//! - `metadata.source={file,page,chunk}` on every chunk (flat keys — `Value`
//!   has no object variant; the MCP metadata parser rejects nested objects)
//! - `SOURCE_CHAR_BUDGET` (28k) counted on emitted payload, truncated declared
//! - deterministic keys `{rel_path}#{chunk}` → re-scan is idempotent

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use tempfile::tempdir;
use vantadb::config::Config;
use vantadb::storage::BackendKind;
use vantadb::wiki::{default_ingestors, scan_ingestable_sources, SOURCE_CHAR_BUDGET};
use vantadb::{Embedded, MemoryInput, MemorySearchRequest, Value};

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("mkdir parent");
    }
    fs::write(path, content).expect("write source");
}

fn meta_str(input: &MemoryInput, key: &str) -> Option<String> {
    match input.metadata.get(key) {
        Some(Value::String(s)) => Some(s.clone()),
        _ => None,
    }
}

fn meta_int(input: &MemoryInput, key: &str) -> Option<i64> {
    match input.metadata.get(key) {
        Some(Value::Int(i)) => Some(*i),
        _ => None,
    }
}

#[test]
fn scans_supported_formats_into_chunks_with_mandatory_provenance() {
    let root = tempdir().expect("tempdir");
    write(root.path(), "notes/a.txt", "alpha text about needle-topic");
    write(
        root.path(),
        "data/b.json",
        r#"{"name": "needle-topic", "qty": 3}"#,
    );
    write(
        root.path(),
        "tables/c.csv",
        "name,qty\nneedle-topic,3\nother,4\n",
    );
    write(
        root.path(),
        "skip.md",
        "markdown is handled by the wiki path, not here",
    );
    // Non-UTF-8 file with a supported extension: readable text only.
    fs::write(root.path().join("bad.txt"), [0xFFu8, 0xFE, 0xFD]).expect("write binary");

    let inputs = scan_ingestable_sources(root.path(), "docs", &default_ingestors()).expect("scan");

    assert_eq!(
        inputs.len(),
        3,
        "one chunk per small supported file; .md and non-UTF-8 are skipped"
    );
    for input in &inputs {
        assert_eq!(input.namespace, "docs");
        let source = meta_str(input, "source").expect("mandatory `source` metadata");
        assert!(!source.is_empty(), "source must name the origin file");
        assert_eq!(
            meta_int(input, "page"),
            Some(0),
            "page=0 for formats without pagination"
        );
        assert_eq!(meta_int(input, "chunk"), Some(0), "single chunk index 0");
    }
    let keys: BTreeSet<&str> = inputs.iter().map(|i| i.key.as_str()).collect();
    assert!(
        keys.contains("notes/a.txt#0"),
        "keys are `{{rel_path}}#{{chunk}}`: {keys:?}"
    );
    assert!(keys.contains("data/b.json#0"), "keys: {keys:?}");
    assert!(keys.contains("tables/c.csv#0"), "keys: {keys:?}");
}

#[test]
fn ingested_chunks_round_trip_through_the_engine_and_are_searchable() {
    let root = tempdir().expect("tempdir");
    write(
        root.path(),
        "notes/a.txt",
        "the quick brown fox about needle-topic",
    );
    write(
        root.path(),
        "data/b.json",
        r#"{"name": "needle-topic", "qty": 3}"#,
    );

    let db = Embedded::open_with_config(Config {
        backend_kind: BackendKind::InMemory,
        read_only: false,
        ..Config::default()
    })
    .expect("open in-memory engine");

    let inputs = scan_ingestable_sources(root.path(), "docs", &default_ingestors()).expect("scan");
    assert_eq!(inputs.len(), 2);
    let records = db.put_batch(inputs.clone()).expect("put_batch");
    assert_eq!(records.len(), 2);

    // Exact retrieval by deterministic key, provenance intact.
    let got = db
        .get("docs", "notes/a.txt#0")
        .expect("get")
        .expect("record exists");
    assert!(got.payload.contains("needle-topic"));
    assert_eq!(
        got.metadata.get("source"),
        Some(&Value::String("notes/a.txt".to_string()))
    );

    // Text search finds the ingested chunks; provenance rides along. The query
    // is a simple token so the assertion holds under both the basic
    // (`lowercase-alnum`) and the advanced tokenizer feature sets.
    let hits = db
        .search(MemorySearchRequest {
            namespace: "docs".to_string(),
            query_vector: Vec::new(),
            text_query: Some("needle".to_string()),
            top_k: 5,
            ..Default::default()
        })
        .expect("search");
    assert!(
        !hits.is_empty(),
        "text search must find the ingested chunks"
    );
    assert!(
        hits[0].record.metadata.contains_key("source"),
        "provenance survives the engine round-trip"
    );

    // Re-scan is idempotent: identical deterministic keys.
    let rescan =
        scan_ingestable_sources(root.path(), "docs", &default_ingestors()).expect("rescan");
    let keys_before: BTreeSet<String> = inputs.iter().map(|i| i.key.clone()).collect();
    let keys_after: BTreeSet<String> = rescan.iter().map(|i| i.key.clone()).collect();
    assert_eq!(keys_before, keys_after, "re-scan must reuse the same keys");
}

#[test]
fn scan_respects_source_char_budget_and_declares_truncation() {
    let root = tempdir().expect("tempdir");
    // a.txt (20k, single paragraph) chunks as 12_000 + 8_402; b.txt (10k) then
    // crosses the remaining budget and is truncated to fit.
    write(root.path(), "a.txt", &"x".repeat(20_000));
    write(root.path(), "b.txt", &"y".repeat(10_000));
    write(
        root.path(),
        "c.txt",
        "must never be scanned: budget exhausted",
    );

    let inputs = scan_ingestable_sources(root.path(), "docs", &default_ingestors()).expect("scan");

    let total: usize = inputs.iter().map(|i| i.payload.chars().count()).sum();
    assert_eq!(total, SOURCE_CHAR_BUDGET, "budget consumed exactly");
    assert_eq!(
        inputs.len(),
        3,
        "two chunks from a.txt + one truncated chunk from b.txt"
    );
    assert_eq!(
        inputs[0].payload.chars().count(),
        12_000,
        "first chunk at the chunker target size"
    );
    assert_eq!(
        inputs[2].payload.chars().count(),
        7_598,
        "crossing chunk truncated to the remaining budget"
    );
    assert!(
        inputs.iter().all(|i| !i.key.starts_with("c.txt")),
        "scan stops once the budget is exhausted"
    );
}

#[test]
fn nonexistent_root_is_a_clear_error() {
    let err = scan_ingestable_sources(
        Path::new("Z:/definitely/not/here"),
        "docs",
        &default_ingestors(),
    )
    .expect_err("scan must fail for a missing root");
    assert!(
        matches!(err, vantadb::Error::InvalidInput(_)),
        "got {err:?}"
    );
}
