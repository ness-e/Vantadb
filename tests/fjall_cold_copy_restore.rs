//! Fjall cold-copy backup/restore validation.
//!
//! Validates the recommended Fjall backup strategy: stop → copy directory → reopen.
//! The restored copy must preserve canonical records, BM25 search, phrase search,
//! hybrid retrieval, and HNSW nearest-neighbor results.

use std::collections::BTreeMap;
use tempfile::tempdir;
use vantadb::{VantaEmbedded, VantaMemoryInput, VantaMemorySearchRequest, VantaValue};

fn input(namespace: &str, key: &str, payload: &str) -> VantaMemoryInput {
    VantaMemoryInput::new(namespace, key, payload)
}

fn input_with_vector(namespace: &str, key: &str, payload: &str, vec: Vec<f32>) -> VantaMemoryInput {
    let mut inp = VantaMemoryInput::new(namespace, key, payload);
    inp.vector = Some(vec);
    inp
}

fn input_with_meta(
    namespace: &str,
    key: &str,
    payload: &str,
    vec: Vec<f32>,
    meta: BTreeMap<String, VantaValue>,
) -> VantaMemoryInput {
    let mut inp = VantaMemoryInput::new(namespace, key, payload);
    inp.vector = Some(vec);
    inp.metadata = meta;
    inp
}

fn search_keys(
    db: &VantaEmbedded,
    namespace: &str,
    text_query: Option<&str>,
    query_vector: Vec<f32>,
    top_k: usize,
) -> Vec<String> {
    db.search(VantaMemorySearchRequest {
        namespace: namespace.to_string(),
        query_vector,
        filters: Default::default(),
        text_query: text_query.map(|s| s.to_string()),
        top_k,
        ..Default::default()
    })
    .expect("search")
    .into_iter()
    .map(|hit| hit.record.key)
    .collect()
}

/// Recursively copy a directory tree.
fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) {
    std::fs::create_dir_all(dst).expect("create dst dir");
    for entry in std::fs::read_dir(src).expect("read src dir") {
        let entry = entry.expect("dir entry");
        let ty = entry.file_type().expect("file type");
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst_path);
        } else {
            std::fs::copy(entry.path(), &dst_path).expect("copy file");
        }
    }
}

#[test]
fn fjall_cold_copy_restore_preserves_all_retrieval_paths() {
    let source_dir = tempdir().expect("source tempdir");
    let restore_dir = tempdir().expect("restore tempdir");

    // ── Phase 1: Seed source database ──────────────────────────────
    {
        let db = VantaEmbedded::open(source_dir.path()).expect("open source");

        // Records with vectors + text for hybrid search
        db.put(input_with_vector(
            "agent/main",
            "alpha-vec",
            "alpha beta gamma",
            vec![1.0, 0.0, 0.0, 0.0],
        ))
        .expect("put alpha-vec");

        db.put(input_with_vector(
            "agent/main",
            "beta-vec",
            "beta delta epsilon",
            vec![0.0, 1.0, 0.0, 0.0],
        ))
        .expect("put beta-vec");

        db.put(input_with_vector(
            "agent/main",
            "gamma-vec",
            "alpha fused gamma",
            vec![0.9, 0.1, 0.0, 0.0],
        ))
        .expect("put gamma-vec");

        // Record with metadata for filtered search
        let mut meta = BTreeMap::new();
        meta.insert("category".to_string(), VantaValue::String("task".into()));
        db.put(input_with_meta(
            "agent/main",
            "meta-rec",
            "metadata filtered query",
            vec![0.5, 0.5, 0.0, 0.0],
            meta,
        ))
        .expect("put meta-rec");

        // Text-only records (no vector)
        db.put(input(
            "agent/main",
            "text-only",
            "unique lexical term searchable",
        ))
        .expect("put text-only");

        // Different namespace for isolation check
        db.put(input_with_vector(
            "agent/other",
            "other-rec",
            "alpha should not appear in main",
            vec![1.0, 0.0, 0.0, 0.0],
        ))
        .expect("put other-rec");

        // Ensure everything is persisted
        db.flush().expect("flush source");

        // Verify source is healthy before backup
        let audit = db.audit_text_index(None).expect("audit source");
        assert!(audit.passed, "source audit must pass before backup");

        db.close().expect("close source");
    }

    // ── Phase 2: Cold copy ──────────────────────────────────────────
    let restore_path = restore_dir.path().join("restored_db");
    copy_dir_all(source_dir.path(), &restore_path);

    // ── Phase 3: Open restored copy and validate ────────────────────
    let restored = VantaEmbedded::open(&restore_path).expect("open restored");

    // 3a. Canonical record retrieval
    let alpha = restored
        .get("agent/main", "alpha-vec")
        .expect("get alpha-vec");
    assert!(alpha.is_some(), "alpha-vec must exist in restored DB");
    assert_eq!(alpha.unwrap().payload, "alpha beta gamma");

    let meta_rec = restored
        .get("agent/main", "meta-rec")
        .expect("get meta-rec");
    assert!(meta_rec.is_some(), "meta-rec must exist in restored DB");

    let text_only = restored
        .get("agent/main", "text-only")
        .expect("get text-only");
    assert!(text_only.is_some(), "text-only must exist in restored DB");

    // 3b. BM25 text-only search
    let text_hits = search_keys(&restored, "agent/main", Some("alpha"), vec![], 10);
    assert!(
        text_hits.contains(&"alpha-vec".to_string()),
        "BM25 must find alpha-vec: got {:?}",
        text_hits
    );
    assert!(
        text_hits.contains(&"gamma-vec".to_string()),
        "BM25 must find gamma-vec: got {:?}",
        text_hits
    );
    // Namespace isolation: "other-rec" must NOT appear in agent/main
    assert!(
        !text_hits.contains(&"other-rec".to_string()),
        "namespace isolation must hold: got {:?}",
        text_hits
    );

    // 3c. Phrase search
    let phrase_hits = search_keys(&restored, "agent/main", Some("\"alpha fused\""), vec![], 10);
    assert_eq!(
        phrase_hits,
        vec!["gamma-vec".to_string()],
        "phrase search must find exact match"
    );

    // 3d. Vector-only search (HNSW)
    let vector_hits = search_keys(&restored, "agent/main", None, vec![1.0, 0.0, 0.0, 0.0], 3);
    assert!(
        !vector_hits.is_empty(),
        "HNSW must be queryable after restore"
    );
    assert_eq!(
        vector_hits[0], "alpha-vec",
        "nearest neighbor should be alpha-vec"
    );

    // 3e. Hybrid search (text + vector via RRF)
    let hybrid_hits = search_keys(
        &restored,
        "agent/main",
        Some("alpha"),
        vec![1.0, 0.0, 0.0, 0.0],
        10,
    );
    assert!(
        !hybrid_hits.is_empty(),
        "hybrid search must return results after restore"
    );
    // "alpha-vec" should rank high (matches both text and vector)
    assert_eq!(
        hybrid_hits[0], "alpha-vec",
        "alpha-vec should be top hybrid result"
    );

    // 3f. Text index audit on restored copy
    let audit = restored.audit_text_index(None).expect("audit restored");
    assert!(
        audit.passed,
        "restored DB audit must pass: mismatches={}, missing={}, unexpected={}",
        audit.mismatches, audit.missing_entries, audit.unexpected_entries
    );

    // 3g. Namespace listing
    let namespaces = restored.list_namespaces().expect("list namespaces");
    assert!(
        namespaces.contains(&"agent/main".to_string()),
        "agent/main must be listed"
    );
    assert!(
        namespaces.contains(&"agent/other".to_string()),
        "agent/other must be listed"
    );

    // 3h. Operational metrics are populated
    let metrics = restored.operational_metrics();
    assert!(
        metrics.startup_ms > 0,
        "startup_ms must be recorded after restore"
    );
}

