//! Operational metrics certification for replay, rebuild, export, and import.

use tempfile::tempdir;
use vantadb::{VantaEmbedded, VantaMemoryInput, VantaMemorySearchRequest};

#[test]
fn metrics_track_rebuild_export_import_and_replay() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().to_path_buf();
    let export_path = dir.path().join("metrics.jsonl");

    {
        let db = VantaEmbedded::open(&path).expect("open");
        let mut input = VantaMemoryInput::new("agent/main", "metric", "payload");
        input.vector = Some(vec![1.0, 0.0, 0.0]);
        db.put(input).expect("put");
    }

    let reopened = VantaEmbedded::open(&path).expect("reopen");
    let replay_metrics = reopened.operational_metrics();
    assert!(replay_metrics.wal_records_replayed >= 1);

    let rebuild = reopened.rebuild_index().expect("rebuild");
    assert!(rebuild.success);
    assert!(rebuild.scanned_nodes >= 1);

    let after_rebuild = reopened.operational_metrics();
    assert!(after_rebuild.ann_rebuild_scanned_nodes >= rebuild.scanned_nodes);
    assert!(after_rebuild.text_postings_written >= 1);
    assert_eq!(after_rebuild.text_consistency_audit_failures, 0);

    let before_text = reopened.operational_metrics();
    let text_hits = reopened
        .search(VantaMemorySearchRequest {
            namespace: "agent/main".to_string(),
            query_vector: Vec::new(),
            filters: Default::default(),
            text_query: Some("payload".to_string()),
            top_k: 5,
        })
        .expect("text search");
    assert_eq!(text_hits.len(), 1);
    let after_text = reopened.operational_metrics();
    assert!(after_text.text_lexical_queries >= before_text.text_lexical_queries + 1);
    assert!(after_text.text_candidates_scored >= before_text.text_candidates_scored + 1);

    let before_export = reopened.operational_metrics();
    let export = reopened.export_all(&export_path).expect("export");
    assert_eq!(export.records_exported, 1);
    let after_export = reopened.operational_metrics();
    assert!(after_export.records_exported >= before_export.records_exported + 1);

    let import_dir = tempdir().expect("import tempdir");
    let imported = VantaEmbedded::open(import_dir.path()).expect("open imported");
    let before_import = imported.operational_metrics();
    let import = imported.import_file(&export_path).expect("import");
    assert_eq!(import.inserted, 1);
    let after_import = imported.operational_metrics();
    assert!(after_import.records_imported >= before_import.records_imported + 1);
}

#[test]
fn metrics_track_import_errors() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");
    let import_path = dir.path().join("invalid.jsonl");
    std::fs::write(&import_path, "{not valid json}\n").expect("write invalid import");

    let before = db.operational_metrics();
    let report = db.import_file(&import_path).expect("import invalid file");
    assert_eq!(report.errors, 1);

    let after = db.operational_metrics();
    assert!(after.import_errors >= before.import_errors + 1);
}
