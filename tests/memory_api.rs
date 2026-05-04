//! Persistent memory API certification.

use tempfile::tempdir;
use vantadb::{
    VantaEmbedded, VantaMemoryInput, VantaMemoryListOptions, VantaMemorySearchRequest, VantaValue,
};

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
        })
        .expect("phrase search");
    assert_eq!(phrase_hits.len(), 1);
    assert_eq!(phrase_hits[0].record.key, "phrase");

    let explain = db
        .debug_memory_search_explain_for_tests(VantaMemorySearchRequest {
            namespace: "agent/main".to_string(),
            query_vector: Vec::new(),
            filters: Default::default(),
            text_query: Some("\"first second\"".to_string()),
            top_k: 5,
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
