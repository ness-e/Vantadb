//! JSONL export/import certification for persistent memory APIs.

use tempfile::tempdir;
use vantadb::{
    VantaEmbedded, VantaMemoryInput, VantaMemoryListOptions, VantaMemorySearchRequest, VantaValue,
};

fn str_value(value: &str) -> VantaValue {
    VantaValue::String(value.to_string())
}

fn record(namespace: &str, key: &str, payload: &str, category: &str) -> VantaMemoryInput {
    let mut input = VantaMemoryInput::new(namespace, key, payload);
    input
        .metadata
        .insert("category".to_string(), str_value(category));
    input.vector = Some(vec![1.0, 0.0, 0.0]);
    input
}

#[test]
fn export_import_namespace_round_trip() {
    let source_dir = tempdir().expect("source tempdir");
    let target_dir = tempdir().expect("target tempdir");
    let export_path = source_dir.path().join("agent-main.jsonl");

    let source = VantaEmbedded::open(source_dir.path()).expect("open source");
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
        .export_namespace(&export_path, "agent/main")
        .expect("export namespace");
    assert_eq!(export.records_exported, 2);
    assert_eq!(export.namespaces, vec!["agent/main".to_string()]);

    let target = VantaEmbedded::open(target_dir.path()).expect("open target");
    let import = target.import_file(&export_path).expect("import file");
    assert_eq!(import.inserted, 2);
    assert_eq!(import.updated, 0);
    assert_eq!(import.errors, 0);

    let fetched = target.get("agent/main", "a").expect("get").expect("record");
    assert_eq!(fetched.payload, "alpha memory");
    assert_eq!(fetched.metadata.get("category"), Some(&str_value("task")));

    let page = target
        .list("agent/main", VantaMemoryListOptions::default())
        .expect("list");
    assert_eq!(page.records.len(), 2);

    let hits = target
        .search(VantaMemorySearchRequest {
            namespace: "agent/main".to_string(),
            query_vector: vec![1.0, 0.0, 0.0],
            filters: Default::default(),
            text_query: None,
            top_k: 5,
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

    let source = VantaEmbedded::open(source_dir.path()).expect("open source");
    source
        .put(record("agent/main", "a", "alpha memory", "task"))
        .expect("put a");
    source
        .put(record("agent/other", "a", "other alpha", "note"))
        .expect("put other");

    let export = source.export_all(&export_path).expect("export all");
    assert_eq!(export.records_exported, 2);

    let target = VantaEmbedded::open(target_dir.path()).expect("open target");
    target
        .put(record("agent/main", "a", "stale alpha", "task"))
        .expect("seed stale");

    let import = target.import_file(&export_path).expect("import file");
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
