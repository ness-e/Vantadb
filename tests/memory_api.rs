//! Persistent memory API certification.

use tempfile::tempdir;
use vantadb::config::VantaConfig;
use vantadb::{
    VantaEmbedded, VantaMemoryInput, VantaMemoryListOptions, VantaMemorySearchRequest, VantaValue,
};

fn db_snapshot(path: &std::path::Path) -> std::collections::BTreeMap<std::path::PathBuf, u64> {
    fn visit(
        root: &std::path::Path,
        current: &std::path::Path,
        out: &mut std::collections::BTreeMap<std::path::PathBuf, u64>,
    ) {
        let Ok(entries) = std::fs::read_dir(current) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                visit(root, &path, out);
            } else if let Ok(metadata) = entry.metadata() {
                let key = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
                out.insert(key, metadata.len());
            }
        }
    }

    let mut snapshot = std::collections::BTreeMap::new();
    visit(path, path, &mut snapshot);
    snapshot
}

fn assert_read_only_error<T: std::fmt::Debug>(result: vantadb::Result<T>) {
    let err = result.expect_err("operation must fail in read-only mode");
    let message = err.to_string();
    assert!(
        message.contains("read-only"),
        "expected read-only error, got: {message}"
    );
}

fn field_string(value: &str) -> VantaValue {
    VantaValue::String(value.to_string())
}

#[test]
fn canonical_memory_model() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");

    let mut input = VantaMemoryInput::new("agent/main", "memory-1", "remember the contract");
    input
        .metadata
        .insert("category".to_string(), field_string("contract"));
    input.vector = Some(vec![1.0, 0.0, 0.0]);

    let record = db.put(input).expect("put");
    assert_eq!(record.namespace, "agent/main");
    assert_eq!(record.key, "memory-1");
    assert_eq!(record.payload, "remember the contract");
    assert_eq!(record.version, 1);
    assert!(record.created_at_ms <= record.updated_at_ms);
    assert_eq!(
        record.metadata.get("category"),
        Some(&field_string("contract"))
    );
    assert_eq!(record.vector.as_ref().map(Vec::len), Some(3));

    let fetched = db
        .get("agent/main", "memory-1")
        .expect("get")
        .expect("record");
    assert_eq!(fetched.node_id, record.node_id);
    assert_eq!(fetched.payload, record.payload);

    let mut update = VantaMemoryInput::new("agent/main", "memory-1", "updated payload");
    update
        .metadata
        .insert("category".to_string(), field_string("contract"));
    let updated = db.put(update).expect("update");
    assert_eq!(updated.node_id, record.node_id);
    assert_eq!(updated.created_at_ms, record.created_at_ms);
    assert_eq!(updated.version, 2);
    assert_eq!(updated.payload, "updated payload");
}

#[test]
fn namespace_isolation() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");

    db.put(VantaMemoryInput::new("agent/a", "shared", "alpha"))
        .expect("put a");
    db.put(VantaMemoryInput::new("agent/b", "shared", "beta"))
        .expect("put b");

    let a = db
        .get("agent/a", "shared")
        .expect("get a")
        .expect("record a");
    let b = db
        .get("agent/b", "shared")
        .expect("get b")
        .expect("record b");

    assert_ne!(a.node_id, b.node_id);
    assert_eq!(a.payload, "alpha");
    assert_eq!(b.payload, "beta");

    let page_a = db
        .list("agent/a", VantaMemoryListOptions::default())
        .expect("list a");
    assert_eq!(page_a.records.len(), 1);
    assert_eq!(page_a.records[0].namespace, "agent/a");

    let namespaces = db.list_namespaces().expect("namespaces");
    assert_eq!(
        namespaces,
        vec!["agent/a".to_string(), "agent/b".to_string()]
    );
}

#[test]
fn memory_api_filters() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");

    let mut first = VantaMemoryInput::new("agent/main", "first", "first payload");
    first
        .metadata
        .insert("category".to_string(), field_string("task"));
    first.vector = Some(vec![1.0, 0.0, 0.0]);
    db.put(first).expect("put first");

    let mut second = VantaMemoryInput::new("agent/main", "second", "second payload");
    second
        .metadata
        .insert("category".to_string(), field_string("note"));
    second.vector = Some(vec![0.0, 1.0, 0.0]);
    db.put(second).expect("put second");

    let mut filters = std::collections::BTreeMap::new();
    filters.insert("category".to_string(), field_string("task"));

    let page = db
        .list(
            "agent/main",
            VantaMemoryListOptions {
                filters: filters.clone(),
                limit: 10,
                cursor: None,
            },
        )
        .expect("filtered list");
    assert_eq!(page.records.len(), 1);
    assert_eq!(page.records[0].key, "first");

    let hits = db
        .search(VantaMemorySearchRequest {
            namespace: "agent/main".to_string(),
            query_vector: vec![1.0, 0.0, 0.0],
            filters,
            text_query: None,
            top_k: 5,
            ..Default::default()
        })
        .expect("search");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].record.key, "first");

    let text_hits = db
        .search(VantaMemorySearchRequest {
            namespace: "agent/main".to_string(),
            query_vector: Vec::new(),
            filters: Default::default(),
            text_query: Some("second".to_string()),
            top_k: 5,
            ..Default::default()
        })
        .expect("text-only search");
    assert_eq!(text_hits.len(), 1);
    assert_eq!(text_hits[0].record.key, "second");

    db.put(VantaMemoryInput::new(
        "agent/main",
        "phrase",
        "first second exact phrase",
    ))
    .expect("put phrase");
    let phrase_hits = db
        .search(VantaMemorySearchRequest {
            namespace: "agent/main".to_string(),
            query_vector: Vec::new(),
            filters: Default::default(),
            text_query: Some("\"first second\"".to_string()),
            top_k: 5,
            ..Default::default()
        })
        .expect("phrase search");
    assert_eq!(phrase_hits.len(), 1);
    assert_eq!(phrase_hits[0].record.key, "phrase");

    let explain = db
        .explain_memory_search(VantaMemorySearchRequest {
            namespace: "agent/main".to_string(),
            query_vector: Vec::new(),
            filters: Default::default(),
            text_query: Some("\"first second\"".to_string()),
            top_k: 5,
            ..Default::default()
        })
        .expect("debug explain");
    assert_eq!(explain.route, "text-only");
    assert_eq!(
        explain.hits[0].matched_phrases,
        vec!["first second".to_string()]
    );
    assert!(explain.hits[0].snippet.is_some());

    let hybrid_hits = db
        .search(VantaMemorySearchRequest {
            namespace: "agent/main".to_string(),
            query_vector: vec![1.0, 0.0, 0.0],
            filters: Default::default(),
            text_query: Some("first".to_string()),
            top_k: 5,
            ..Default::default()
        })
        .expect("hybrid search");
    assert!(hybrid_hits.len() >= 2);
    assert_eq!(hybrid_hits[0].record.key, "first");
    assert!(hybrid_hits.iter().any(|hit| hit.record.key == "second"));

    let empty = db
        .search(VantaMemorySearchRequest {
            namespace: "agent/main".to_string(),
            query_vector: vec![1.0, 0.0, 0.0],
            filters: Default::default(),
            text_query: Some("second".to_string()),
            top_k: 0,
            ..Default::default()
        })
        .expect("hybrid top_k zero");
    assert!(empty.is_empty());

    let whitespace_text_query = db
        .search(VantaMemorySearchRequest {
            namespace: "agent/main".to_string(),
            query_vector: vec![1.0, 0.0, 0.0],
            filters: Default::default(),
            text_query: Some("   ".to_string()),
            top_k: 5,
            ..Default::default()
        })
        .expect("whitespace text query falls back to vector");
    assert_eq!(whitespace_text_query[0].record.key, "first");
}

#[test]
fn memory_api_recovery() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().to_path_buf();

    {
        let db = VantaEmbedded::open(&path).expect("open");
        let mut input = VantaMemoryInput::new("agent/main", "recover", "wal backed");
        input.vector = Some(vec![0.5, 0.5, 0.0]);
        db.put(input).expect("put");
    }

    let reopened = VantaEmbedded::open(&path).expect("reopen");
    let record = reopened
        .get("agent/main", "recover")
        .expect("get")
        .expect("record");
    assert_eq!(record.payload, "wal backed");

    assert!(reopened
        .delete("agent/main", "recover")
        .expect("delete existing"));
    assert!(reopened
        .get("agent/main", "recover")
        .expect("get deleted")
        .is_none());
}

#[test]
fn read_only_rejects_mutations_without_changing_db_files() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().to_path_buf();
    let import_path = dir.path().join("readonly-import.jsonl");

    {
        let db = VantaEmbedded::open(&path).expect("open writable");
        let mut input = VantaMemoryInput::new("agent/main", "readonly", "read only payload");
        input.vector = Some(vec![1.0, 0.0, 0.0]);
        db.put(input).expect("put");
        db.flush().expect("flush writable");
        db.close().expect("close writable");
    }

    std::fs::write(&import_path, "{}\n").expect("write import fixture");

    let read_only = VantaEmbedded::open_with_config(VantaConfig {
        storage_path: path.to_string_lossy().into_owned(),
        read_only: true,
        ..Default::default()
    })
    .expect("open read-only");

    let before = db_snapshot(&path);

    assert_read_only_error(read_only.put(VantaMemoryInput::new(
        "agent/main",
        "blocked-put",
        "blocked",
    )));
    assert_read_only_error(read_only.delete("agent/main", "readonly"));
    assert_read_only_error(read_only.import_file(&import_path));
    assert_read_only_error(read_only.rebuild_index());
    assert_read_only_error(read_only.repair_text_index());
    assert_read_only_error(read_only.flush());

    let fetched = read_only
        .get("agent/main", "readonly")
        .expect("read-only get")
        .expect("record");
    assert_eq!(fetched.payload, "read only payload");

    let audit = read_only
        .audit_text_index_deep(Some("agent/main"))
        .expect("read-only deep audit");
    assert!(audit.passed);

    let hits = read_only
        .search(VantaMemorySearchRequest {
            namespace: "agent/main".to_string(),
            query_vector: Vec::new(),
            filters: Default::default(),
            text_query: Some("payload".to_string()),
            top_k: 5,
            ..Default::default()
        })
        .expect("read-only text search");
    assert_eq!(hits.len(), 1);

    let after = db_snapshot(&path);
    assert_eq!(
        after, before,
        "read-only operations must not change DB files"
    );
}

#[test]
fn memory_euclidean_and_explainable_ranking() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");

    let mut input1 = VantaMemoryInput::new("agent/main", "vec-1", "payload 1");
    input1.vector = Some(vec![1.0, 0.0, 0.0]);
    input1
        .metadata
        .insert("category".to_string(), field_string("test"));
    db.put(input1).expect("put vec-1");

    let mut input2 = VantaMemoryInput::new("agent/main", "vec-2", "payload 2");
    input2.vector = Some(vec![0.0, 1.0, 0.0]);
    input2
        .metadata
        .insert("category".to_string(), field_string("test"));
    db.put(input2).expect("put vec-2");

    // Buscar con distancia Euclidiana y explain = true
    let request_explain = VantaMemorySearchRequest {
        namespace: "agent/main".to_string(),
        query_vector: vec![0.9, 0.1, 0.0],
        filters: Default::default(),
        text_query: None,
        top_k: 2,
        distance_metric: vantadb::DistanceMetric::Euclidean,
        explain: true,
    };

    let hits_explain = db.search(request_explain).expect("search with explain");
    assert_eq!(hits_explain.len(), 2);
    assert_eq!(hits_explain[0].record.key, "vec-1"); // Más cercano
    assert_eq!(hits_explain[1].record.key, "vec-2"); // Más lejano

    // Validar que la explicación de ranking esté presente
    assert!(hits_explain[0].explanation.is_some());
    let explanation = hits_explain[0].explanation.as_ref().unwrap();
    assert_eq!(explanation.identity, "agent/main\0vec-1");

    // Buscar con explain = false para validar que no se devuelvan explicaciones innecesarias
    let request_no_explain = VantaMemorySearchRequest {
        namespace: "agent/main".to_string(),
        query_vector: vec![0.9, 0.1, 0.0],
        filters: Default::default(),
        text_query: None,
        top_k: 2,
        distance_metric: vantadb::DistanceMetric::Euclidean,
        explain: false,
    };

    let hits_no_explain = db
        .search(request_no_explain)
        .expect("search without explain");
    assert_eq!(hits_no_explain.len(), 2);
    assert!(hits_no_explain[0].explanation.is_none());
    assert!(hits_no_explain[1].explanation.is_none());
}

#[test]
fn snippet_with_highlighting() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");

    // Insertar un registro con texto
    let input = VantaMemoryInput {
        key: "snippet-test".to_string(),
        namespace: "test".to_string(),
        payload: "The quick brown fox jumps over the lazy dog".to_string(),
        vector: Some(vec![0.1, 0.2, 0.3]),
        metadata: Default::default(),
    };
    db.put(input).expect("put");

    // Buscar con explicación para obtener snippet
    let request = VantaMemorySearchRequest {
        namespace: "test".to_string(),
        query_vector: vec![0.1, 0.2, 0.3],
        filters: Default::default(),
        text_query: Some("quick fox".to_string()),
        top_k: 1,
        distance_metric: vantadb::DistanceMetric::Euclidean,
        explain: true,
    };

    let hits = db.search(request).expect("search");
    assert_eq!(hits.len(), 1);

    let explanation = hits[0]
        .explanation
        .as_ref()
        .expect("explanation should be present");
    assert!(explanation.snippet.is_some(), "snippet should be present");

    let snippet = explanation.snippet.as_ref().unwrap();
    // El snippet debería contener parte del texto original
    assert!(!snippet.is_empty());
}
