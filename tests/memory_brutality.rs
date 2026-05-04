//! Operational MVP smoke: recovery, rebuild, export/import, and volume KPIs.

use std::time::Instant;
use tempfile::tempdir;
use vantadb::{
    VantaEmbedded, VantaMemoryInput, VantaMemoryListOptions, VantaMemorySearchRequest, VantaValue,
};

fn str_value(value: &str) -> VantaValue {
    VantaValue::String(value.to_string())
}

fn vector_for(i: usize) -> Vec<f32> {
    vec![i as f32 + 1.0, 1.0, 0.0]
}

#[test]
fn recovery_rebuild_export_import_survive_restart_and_index_loss() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().to_path_buf();
    let export_path = dir.path().join("memory.jsonl");

    {
        let db = VantaEmbedded::open(&path).expect("open");
        let mut input = VantaMemoryInput::new("agent/main", "recover", "wal backed");
        input
            .metadata
            .insert("category".to_string(), str_value("task"));
        input.vector = Some(vec![1.0, 0.0, 0.0]);
        db.put(input).expect("put");
    }

    let reopened = VantaEmbedded::open(&path).expect("reopen");
    let recovered = reopened
        .get("agent/main", "recover")
        .expect("get")
        .expect("record");
    assert_eq!(recovered.payload, "wal backed");

    reopened.flush().expect("flush");
    let index_path = path.join("data").join("vector_index.bin");
    if index_path.exists() {
        std::fs::remove_file(&index_path).expect("remove vector index");
    }

    let rebuild = reopened.rebuild_index().expect("manual rebuild");
    assert!(rebuild.success);
    assert!(index_path.exists());

    let hits = reopened
        .search(VantaMemorySearchRequest {
            namespace: "agent/main".to_string(),
            query_vector: vec![1.0, 0.0, 0.0],
            filters: Default::default(),
            text_query: None,
            top_k: 1,
        })
        .expect("search");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].record.key, "recover");

    let export = reopened.export_all(&export_path).expect("export all");
    assert_eq!(export.records_exported, 1);

    let imported_dir = tempdir().expect("imported tempdir");
    let imported = VantaEmbedded::open(imported_dir.path()).expect("open imported");
    let import = imported.import_file(&export_path).expect("import file");
    assert_eq!(import.inserted, 1);
    assert_eq!(import.errors, 0);
    assert_eq!(
        imported
            .get("agent/main", "recover")
            .expect("get imported")
            .expect("record")
            .payload,
        "wal backed"
    );
}

#[test]
fn memory_volume_kpi_10k_records_namespaces_filters_export_import_rebuild() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");
    let started = Instant::now();

    for i in 0..10_000usize {
        let namespace = match i % 3 {
            0 => "agent/a",
            1 => "agent/b",
            _ => "agent/c",
        };
        let mut input =
            VantaMemoryInput::new(namespace, format!("key-{i:05}"), format!("payload {i}"));
        input.metadata.insert(
            "kind".to_string(),
            str_value(if i % 2 == 0 { "even" } else { "odd" }),
        );
        if i % 1000 == 0 {
            input.vector = Some(vector_for(i));
        }
        db.put(input).expect("put volume record");
    }

    db.flush().expect("flush volume");
    let ingest_ms = started.elapsed().as_millis();

    let mut filters = std::collections::BTreeMap::new();
    filters.insert("kind".to_string(), str_value("even"));
    let filtered = db
        .list(
            "agent/a",
            VantaMemoryListOptions {
                filters,
                limit: 50,
                cursor: None,
            },
        )
        .expect("filtered list");
    assert_eq!(filtered.records.len(), 50);
    assert!(filtered
        .records
        .iter()
        .all(|record| record.namespace == "agent/a"));

    let export_path = dir.path().join("volume.jsonl");
    let export = db.export_all(&export_path).expect("export all");
    assert_eq!(export.records_exported, 10_000);

    let rebuild = db.rebuild_index().expect("rebuild");
    assert!(rebuild.success);
    assert!(rebuild.scanned_nodes >= 10_000);

    let target_dir = tempdir().expect("target tempdir");
    let target = VantaEmbedded::open(target_dir.path()).expect("open target");
    let import = target.import_file(&export_path).expect("import volume");
    assert_eq!(import.inserted, 10_000);
    assert_eq!(import.errors, 0);

    let page = target
        .list("agent/b", VantaMemoryListOptions::default())
        .expect("list imported");
    assert_eq!(page.records.len(), 100);

    assert!(
        ingest_ms < 120_000,
        "10K memory KPI smoke exceeded 120s ingest budget: {ingest_ms}ms"
    );
}

#[test]
fn delete_without_explicit_flush_survives_reopen() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().to_path_buf();

    {
        let db = VantaEmbedded::open(&path).expect("open");
        db.put(VantaMemoryInput::new(
            "agent/main",
            "delete-replay",
            "temporary",
        ))
        .expect("put");
        db.flush().expect("flush seed");
        assert!(db
            .delete("agent/main", "delete-replay")
            .expect("delete without explicit flush"));
    }

    let reopened = VantaEmbedded::open(&path).expect("reopen");
    assert!(reopened
        .get("agent/main", "delete-replay")
        .expect("get")
        .is_none());
    let page = reopened
        .list("agent/main", VantaMemoryListOptions::default())
        .expect("list");
    assert!(page.records.is_empty());
}
