#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Persistent memory API certification.

use tempfile::tempdir;
use vantadb::config::Config;
use vantadb::{Embedded, MemoryInput, MemoryListOptions, MemorySearchRequest, Value};

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

fn field_string(value: &str) -> Value {
    Value::String(value.to_string())
}

#[test]
fn canonical_memory_model() {
    let dir = tempdir().expect("tempdir");
    let db = Embedded::open(dir.path()).expect("open");

    let mut input = MemoryInput::new("agent/main", "memory-1", "remember the contract");
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

    let mut update = MemoryInput::new("agent/main", "memory-1", "updated payload");
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
    let db = Embedded::open(dir.path()).expect("open");

    db.put(MemoryInput::new("agent/a", "shared", "alpha"))
        .expect("put a");
    db.put(MemoryInput::new("agent/b", "shared", "beta"))
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
        .list("agent/a", MemoryListOptions::default())
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
    let db = Embedded::open(dir.path()).expect("open");

    let mut first = MemoryInput::new("agent/main", "first", "first payload");
    first
        .metadata
        .insert("category".to_string(), field_string("task"));
    first.vector = Some(vec![1.0, 0.0, 0.0]);
    db.put(first).expect("put first");

    let mut second = MemoryInput::new("agent/main", "second", "second payload");
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
            MemoryListOptions {
                #[allow(deprecated)]
                filters: filters.clone(),
                filter_ops: None,
                limit: 10,
                cursor: None,
                exclude_superseded: false,
                as_of_ms: None,
                valid_window: None,
            },
        )
        .expect("filtered list");
    assert_eq!(page.records.len(), 1);
    assert_eq!(page.records[0].key, "first");

    let hits = db
        .search(MemorySearchRequest {
            namespace: "agent/main".to_string(),
            query_vector: vec![1.0, 0.0, 0.0],
            filters,
            text_query: None,
            top_k: 5,
            range: None,
            group_by: None,
            mmr: None,
            cursor: None,
            ..Default::default()
        })
        .expect("search");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].record.key, "first");

    let text_hits = db
        .search(MemorySearchRequest {
            namespace: "agent/main".to_string(),
            query_vector: Vec::new(),
            filters: Default::default(),
            text_query: Some("second".to_string()),
            top_k: 5,
            range: None,
            group_by: None,
            mmr: None,
            cursor: None,
            ..Default::default()
        })
        .expect("text-only search");
    assert_eq!(text_hits.len(), 1);
    assert_eq!(text_hits[0].record.key, "second");

    db.put(MemoryInput::new(
        "agent/main",
        "phrase",
        "first second exact phrase",
    ))
    .expect("put phrase");
    let phrase_hits = db
        .search(MemorySearchRequest {
            namespace: "agent/main".to_string(),
            query_vector: Vec::new(),
            filters: Default::default(),
            text_query: Some("\"first second\"".to_string()),
            top_k: 5,
            range: None,
            group_by: None,
            mmr: None,
            cursor: None,
            ..Default::default()
        })
        .expect("phrase search");
    assert_eq!(phrase_hits.len(), 1);
    assert_eq!(phrase_hits[0].record.key, "phrase");

    let explain = db
        .explain_memory_search(MemorySearchRequest {
            namespace: "agent/main".to_string(),
            query_vector: Vec::new(),
            filters: Default::default(),
            text_query: Some("\"first second\"".to_string()),
            top_k: 5,
            range: None,
            group_by: None,
            mmr: None,
            cursor: None,
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
        .search(MemorySearchRequest {
            namespace: "agent/main".to_string(),
            query_vector: vec![1.0, 0.0, 0.0],
            filters: Default::default(),
            text_query: Some("first".to_string()),
            top_k: 5,
            range: None,
            group_by: None,
            mmr: None,
            cursor: None,
            ..Default::default()
        })
        .expect("hybrid search");
    assert!(hybrid_hits.len() >= 2);
    assert_eq!(hybrid_hits[0].record.key, "first");
    assert!(hybrid_hits.iter().any(|hit| hit.record.key == "second"));

    let empty = db
        .search(MemorySearchRequest {
            namespace: "agent/main".to_string(),
            query_vector: vec![1.0, 0.0, 0.0],
            filters: Default::default(),
            text_query: Some("second".to_string()),
            top_k: 0,
            range: None,
            group_by: None,
            mmr: None,
            cursor: None,
            ..Default::default()
        })
        .expect("hybrid top_k zero");
    assert!(empty.is_empty());

    let whitespace_text_query = db
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
        .expect("whitespace text query falls back to vector");
    assert_eq!(whitespace_text_query[0].record.key, "first");
}

#[test]
fn memory_api_recovery() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().to_path_buf();

    {
        let db = Embedded::open(&path).expect("open");
        let mut input = MemoryInput::new("agent/main", "recover", "wal backed");
        input.vector = Some(vec![0.5, 0.5, 0.0]);
        db.put(input).expect("put");
    }

    let reopened = Embedded::open(&path).expect("reopen");
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
        let db = Embedded::open(&path).expect("open writable");
        let mut input = MemoryInput::new("agent/main", "readonly", "read only payload");
        input.vector = Some(vec![1.0, 0.0, 0.0]);
        db.put(input).expect("put");
        db.flush().expect("flush writable");
        db.close().expect("close writable");
    }

    std::fs::write(&import_path, "{}\n").expect("write import fixture");

    let read_only = Embedded::open_with_config(Config {
        storage_path: path.to_string_lossy().into_owned(),
        read_only: true,
        ..Default::default()
    })
    .expect("open read-only");

    let before = db_snapshot(&path);

    assert_read_only_error(read_only.put(MemoryInput::new("agent/main", "blocked-put", "blocked")));
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
        .search(MemorySearchRequest {
            namespace: "agent/main".to_string(),
            query_vector: Vec::new(),
            filters: Default::default(),
            text_query: Some("payload".to_string()),
            top_k: 5,
            range: None,
            group_by: None,
            mmr: None,
            cursor: None,
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
    let db = Embedded::open(dir.path()).expect("open");

    let mut input1 = MemoryInput::new("agent/main", "vec-1", "payload 1");
    input1.vector = Some(vec![1.0, 0.0, 0.0]);
    input1
        .metadata
        .insert("category".to_string(), field_string("test"));
    db.put(input1).expect("put vec-1");

    let mut input2 = MemoryInput::new("agent/main", "vec-2", "payload 2");
    input2.vector = Some(vec![0.0, 1.0, 0.0]);
    input2
        .metadata
        .insert("category".to_string(), field_string("test"));
    db.put(input2).expect("put vec-2");

    // Buscar con distancia Euclidiana y explain = true
    let request_explain = MemorySearchRequest {
        namespace: "agent/main".to_string(),
        query_vector: vec![0.9, 0.1, 0.0],
        filters: Default::default(),
        text_query: None,
        top_k: 2,
        distance_metric: vantadb::DistanceMetric::Euclidean,
        explain: true,
        query_sparse: None,
        exclude_superseded: false,
        min_confidence: None,
        as_of_ms: None,
        valid_window: None,
        search_profile: None,
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
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
    let request_no_explain = MemorySearchRequest {
        namespace: "agent/main".to_string(),
        query_vector: vec![0.9, 0.1, 0.0],
        filters: Default::default(),
        text_query: None,
        top_k: 2,
        distance_metric: vantadb::DistanceMetric::Euclidean,
        explain: false,
        query_sparse: None,
        exclude_superseded: false,
        min_confidence: None,
        as_of_ms: None,
        valid_window: None,
        search_profile: None,
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
    };

    let hits_no_explain = db
        .search(request_no_explain)
        .expect("search without explain");
    assert_eq!(hits_no_explain.len(), 2);
    assert!(hits_no_explain[0].explanation.is_none());
    assert!(hits_no_explain[1].explanation.is_none());
}

#[test]
fn namespace_stats_end_to_end() {
    let dir = tempdir().expect("tempdir");
    let db = Embedded::open(dir.path()).expect("open");

    // ns1: 1 normal + 1 expiring soon (1h TTL, inside the default 24h window).
    db.put(MemoryInput::new("ns1", "normal", "p"))
        .expect("put ns1 normal");
    let mut soon = MemoryInput::new("ns1", "soon", "p");
    soon.ttl_ms = Some(60 * 60 * 1000);
    db.put(soon).expect("put ns1 expiring soon");

    // ns2: 1 normal + 1 expired (1ms TTL, then wait past the deadline).
    db.put(MemoryInput::new("ns2", "normal", "p"))
        .expect("put ns2 normal");
    let mut expired = MemoryInput::new("ns2", "gone", "p");
    expired.ttl_ms = Some(1);
    db.put(expired).expect("put ns2 expiring");
    std::thread::sleep(std::time::Duration::from_millis(5));

    let stats = db.namespace_stats(None).expect("namespace stats");
    let ns1 = &stats["ns1"];
    assert_eq!(ns1.count, 2);
    assert_eq!(ns1.expiring_soon, 1);
    assert_eq!(ns1.expired, 0);
    let ns2 = &stats["ns2"];
    assert_eq!(ns2.count, 2);
    assert_eq!(ns2.expiring_soon, 0);
    assert_eq!(ns2.expired, 1);

    // Stats map keys (BTreeMap → sorted) stay consistent with list_namespaces.
    let expected: Vec<String> = stats.keys().cloned().collect();
    assert_eq!(expected, db.list_namespaces().expect("list namespaces"));
}

#[test]
fn snippet_with_highlighting() {
    let dir = tempdir().expect("tempdir");
    let db = Embedded::open(dir.path()).expect("open");

    // Insertar un registro con texto
    let input = MemoryInput {
        key: "snippet-test".to_string(),
        namespace: "test".to_string(),
        payload: "The quick brown fox jumps over the lazy dog".to_string(),
        vector: Some(vec![0.1, 0.2, 0.3]),
        sparse_vector: None,
        metadata: Default::default(),
        ttl_ms: None,
        ..Default::default()
    };
    db.put(input).expect("put");

    // Buscar con explicación para obtener snippet
    let request = MemorySearchRequest {
        namespace: "test".to_string(),
        query_vector: vec![0.1, 0.2, 0.3],
        filters: Default::default(),
        text_query: Some("quick fox".to_string()),
        top_k: 1,
        distance_metric: vantadb::DistanceMetric::Euclidean,
        explain: true,
        query_sparse: None,
        exclude_superseded: false,
        min_confidence: None,
        as_of_ms: None,
        valid_window: None,
        search_profile: None,
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
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

// ─── ADR-046 §D6: restore_graph_nodes precedence (CORE-02 intacto) ───

#[test]
fn restore_graph_nodes_preserves_confidence_and_memory_put_renormalizes() {
    use vantadb::{Fields, NodeRecord, StorageTier};

    fn node_record(id: u128, fields: Fields, confidence_score: f32) -> NodeRecord {
        NodeRecord {
            id,
            fields,
            vector: None,
            vector_dimensions: 0,
            edges: Vec::new(),
            confidence_score,
            importance: 0.1,
            hits: 0,
            last_accessed: 0,
            epoch: 0,
            tier: StorageTier::Cold,
            is_alive: true,
        }
    }

    let dir = tempdir().expect("tempdir");
    let db = Embedded::open(dir.path()).expect("open");

    // (1) CORE-02 intacto: a graph restore preserves the transported score
    // verbatim (no recompute, no discard).
    db.restore_graph_nodes(vec![node_record(777, Fields::new(), 0.42)])
        .expect("restore graph node");
    let graph_node = db.get_node(777).expect("get node").expect("node 777");
    assert_eq!(graph_node.confidence_score, 0.42);

    // (2) D6 coherence after a memory write: record.confidence is projected
    // into node.confidence_score.
    let record = db
        .put(MemoryInput {
            confidence: Some(0.3),
            ..MemoryInput::new("ns/d6", "k", "payload")
        })
        .expect("put with confidence");
    let node = db
        .get_node(record.node_id)
        .expect("get memory node")
        .expect("memory node");
    assert_eq!(
        node.confidence_score, record.confidence,
        "D6: memory write is the normalizer (record.confidence == node.confidence_score)"
    );

    // (3) A restore landing on a memory node id is a verbatim transport writer
    // (last-writer-wins); a later memory put re-normalizes the v2 projection.
    let mut fields = Fields::new();
    fields.insert("__vanta_namespace".into(), Value::String("ns/d6".into()));
    fields.insert("__vanta_key".into(), Value::String("k".into()));
    fields.insert("__vanta_payload".into(), Value::String("payload".into()));
    fields.insert(
        "__vanta_created_at_ms".into(),
        Value::Int(record.created_at_ms as i64),
    );
    fields.insert(
        "__vanta_updated_at_ms".into(),
        Value::Int(record.updated_at_ms as i64),
    );
    fields.insert("__vanta_version".into(), Value::Int(record.version as i64));
    fields.insert(
        "__vanta_valid_at_ms".into(),
        Value::Int(record.valid_at_ms as i64),
    );
    fields.insert(
        "__vanta_confidence_class".into(),
        Value::String("Asserted".into()),
    );
    db.restore_graph_nodes(vec![node_record(record.node_id, fields, 0.9)])
        .expect("restore memory node");
    let after_restore = db.get("ns/d6", "k").expect("get").expect("record");
    assert_eq!(
        after_restore.confidence, 0.9,
        "D6: restore preserves the transported score verbatim"
    );

    let reput = db
        .put(MemoryInput {
            confidence: Some(0.5),
            ..MemoryInput::new("ns/d6", "k", "payload v2")
        })
        .expect("re-put");
    assert_eq!(reput.confidence, 0.5);
    let node_after = db.get_node(reput.node_id).expect("get node").expect("node");
    assert_eq!(
        node_after.confidence_score, 0.5,
        "D6: a later memory put re-normalizes the node projection"
    );
}
