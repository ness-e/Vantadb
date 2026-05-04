//! Deterministic internal certification corpus for memory retrieval modes.
//!
//! This is intentionally small and local. It validates planner/ranking behavior
//! and is not a competitive retrieval benchmark.

use tempfile::tempdir;
use vantadb::{
    VantaEmbedded, VantaMemoryInput, VantaMemoryMetadata, VantaMemorySearchRequest, VantaValue,
};

fn field_string(value: &str) -> VantaValue {
    VantaValue::String(value.to_string())
}

fn put_memory(db: &VantaEmbedded, key: &str, payload: &str, vector: Vec<f32>, category: &str) {
    let mut input = VantaMemoryInput::new("cert/main", key, payload);
    input.vector = Some(vector);
    input
        .metadata
        .insert("category".to_string(), field_string(category));
    db.put(input).expect("put cert memory");
}

fn keep_filter() -> VantaMemoryMetadata {
    let mut filters = VantaMemoryMetadata::new();
    filters.insert("category".to_string(), field_string("keep"));
    filters
}

fn search_keys(
    db: &VantaEmbedded,
    text_query: Option<&str>,
    query_vector: Vec<f32>,
    top_k: usize,
) -> Vec<String> {
    db.search(VantaMemorySearchRequest {
        namespace: "cert/main".to_string(),
        query_vector,
        filters: keep_filter(),
        text_query: text_query.map(str::to_string),
        top_k,
    })
    .expect("search cert corpus")
    .into_iter()
    .map(|hit| hit.record.key)
    .collect()
}

#[test]
fn deterministic_corpus_certifies_text_vector_hybrid_and_phrase_paths() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");

    put_memory(&db, "both", "alpha fused phrase", vec![1.0, 0.0], "keep");
    put_memory(
        &db,
        "text-only",
        "alpha lexical phrase",
        vec![0.0, 1.0],
        "keep",
    );
    put_memory(&db, "vector-only", "background", vec![0.95, 0.05], "keep");
    put_memory(
        &db,
        "filtered",
        "alpha fused phrase",
        vec![1.0, 0.0],
        "drop",
    );

    let text = search_keys(&db, Some("alpha"), Vec::new(), 10);
    assert_eq!(text, vec!["both".to_string(), "text-only".to_string()]);

    let phrase = search_keys(&db, Some("\"fused phrase\""), Vec::new(), 10);
    assert_eq!(phrase, vec!["both".to_string()]);

    let vector = search_keys(&db, None, vec![1.0, 0.0], 10);
    assert_eq!(vector[0], "both");
    assert!(vector.contains(&"vector-only".to_string()));
    assert!(!vector.contains(&"filtered".to_string()));

    let hybrid = search_keys(&db, Some("alpha"), vec![1.0, 0.0], 10);
    assert_eq!(hybrid[0], "both");
    assert!(hybrid.contains(&"text-only".to_string()));
    assert!(hybrid.contains(&"vector-only".to_string()));
    assert!(!hybrid.contains(&"filtered".to_string()));
}
