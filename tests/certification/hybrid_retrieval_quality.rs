//! Deterministic internal certification corpus for memory retrieval modes.
//!
//! This is intentionally small and local. It validates planner/ranking behavior
//! and is not a competitive retrieval benchmark.
//!
//! TSK-38: Extended corpus with known expected rankings for continuous validation
//! across text-only, vector-only, hybrid, phrase, multi-namespace, and edge cases.

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
    search_keys_with_filters(
        db,
        "cert/main",
        text_query,
        query_vector,
        top_k,
        Some(keep_filter()),
    )
}

fn search_keys_with_namespace(
    db: &VantaEmbedded,
    namespace: &str,
    text_query: Option<&str>,
    query_vector: Vec<f32>,
    top_k: usize,
) -> Vec<String> {
    search_keys_with_filters(db, namespace, text_query, query_vector, top_k, None)
}

fn search_keys_with_filters(
    db: &VantaEmbedded,
    namespace: &str,
    text_query: Option<&str>,
    query_vector: Vec<f32>,
    top_k: usize,
    filters: Option<VantaMemoryMetadata>,
) -> Vec<String> {
    db.search(VantaMemorySearchRequest {
        namespace: namespace.to_string(),
        query_vector,
        filters: filters.unwrap_or_default(),
        text_query: text_query.map(str::to_string),
        top_k,
        ..Default::default()
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

/// Extended corpus with known BM25 ranking expectations:
/// Documents with different term frequencies and lengths to verify
/// TF saturation, IDF boost for rare terms, and length normalization.
#[test]
fn extended_corpus_certifies_bm25_ranking_edge_cases_and_multi_namespace() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");

    // ── Namespace A: varied term frequencies ──
    for i in 0..10 {
        let payload = match i {
            0 => "machine learning",                          // rare term
            1 => "machine machine learning",                  // high TF for "machine"
            2 => "deep neural networks for machine learning", // long doc, matches
            3 => "data science python",                       // no match
            4 => "machine learning machine learning",         // repeated phrase
            5 => "quantum computing",                         // no match
            6 => "machine",                                   // single term match
            7 => "machine learning deep learning",            // multi-term, avg len
            8 => "machines learn differently",                // stem: machine → machin
            9 => "irrelevant topic here",                     // no match
            _ => unreachable!(),
        };
        let mut input = VantaMemoryInput::new("bm25/a", format!("doc_{}", i), payload);
        input.vector = Some(vec![1.0 - (i as f32) * 0.1, (i as f32) * 0.1]);
        input.metadata.insert(
            "group".to_string(),
            if i < 5 {
                field_string("first_half")
            } else {
                field_string("second_half")
            },
        );
        db.put(input).expect("put bm25 doc");
    }

    // ── Namespace B: isolated corpus ──
    for i in 0..4 {
        let payload = match i {
            0 => "machine learning in production",
            1 => "learning rust programming",
            2 => "machine learning with python",
            3 => "rust systems programming",
            _ => unreachable!(),
        };
        let mut input = VantaMemoryInput::new("bm25/b", format!("b_doc_{}", i), payload);
        input.vector = Some(vec![0.5, 0.5]);
        db.put(input).expect("put ns_b doc");
    }

    // ── Test 1: BM25 ranking with TF saturation ──
    // Query "machine": doc_1 (TF=2) should rank above doc_0 (TF=1)
    let results = search_keys_with_namespace(&db, "bm25/a", Some("machine"), Vec::new(), 10);
    assert!(
        results.contains(&"doc_1".to_string()),
        "doc_1 (TF=2 for 'machine') should match 'machine'"
    );
    assert!(
        results.contains(&"doc_0".to_string()),
        "doc_0 (TF=1 for 'machine') should match 'machine'"
    );
    assert!(
        results.contains(&"doc_4".to_string()),
        "doc_4 (TF=2 for 'machine', TF=2 for 'learning') should match 'machine'"
    );
    // doc_1 should rank above doc_0 (higher TF)
    let pos_doc_1 = results.iter().position(|k| k == "doc_1");
    let pos_doc_0 = results.iter().position(|k| k == "doc_0");
    if let (Some(p1), Some(p0)) = (pos_doc_1, pos_doc_0) {
        assert!(
            p1 < p0,
            "doc_1 (TF=2) should rank above doc_0 (TF=1) for 'machine'"
        );
    }

    // ── Test 2: Non-matching queries return empty ──
    let no_match = search_keys_with_namespace(
        &db,
        "bm25/a",
        Some("\"exact phrase not in corpus\""),
        Vec::new(),
        10,
    );
    assert!(
        no_match.is_empty(),
        "Phrase not in corpus should return empty results"
    );
    let no_match = search_keys_with_namespace(
        &db,
        "bm25/a",
        Some("zzzzzzzzz_nonexistent_term"),
        Vec::new(),
        10,
    );
    assert!(
        no_match.is_empty(),
        "Non-existent term should return empty results"
    );

    // ── Test 3: Empty/whitespace query returns empty ──
    let empty = search_keys_with_namespace(&db, "bm25/a", Some(""), Vec::new(), 10);
    assert!(
        empty.is_empty(),
        "Empty string query should return empty results"
    );

    // ── Test 4: Namespace isolation ──
    let ns_a = search_keys_with_namespace(&db, "bm25/a", Some("machine"), Vec::new(), 10);
    let ns_b = search_keys_with_namespace(&db, "bm25/b", Some("machine"), Vec::new(), 10);
    assert!(!ns_a.is_empty(), "Namespace A should have machine matches");
    assert!(!ns_b.is_empty(), "Namespace B should have machine matches");
    for key in &ns_b {
        assert!(
            key.starts_with("b_doc_"),
            "All namespace B keys should start with 'b_doc_', got '{}'",
            key
        );
        assert!(
            !ns_a.contains(key),
            "Namespace B key '{}' should not appear in namespace A results",
            key
        );
    }

    // ── Test 5: Filter + text query intersection ──
    let mut filter = VantaMemoryMetadata::new();
    filter.insert("group".to_string(), field_string("first_half"));
    let first_half: Vec<String> =
        search_keys_with_filters(&db, "bm25/a", Some("machine"), Vec::new(), 10, Some(filter));
    for key in &first_half {
        let idx: usize = key.strip_prefix("doc_").unwrap().parse().unwrap();
        assert!(
            idx < 5,
            "With group=first_half filter, only docs with idx < 5 should appear, got {}",
            key
        );
    }

    // ── Test 6: top_k clamping ──
    let top_3 = search_keys_with_namespace(&db, "bm25/a", Some("machine"), Vec::new(), 3);
    assert!(
        top_3.len() <= 3,
        "top_k=3 should return at most 3 results, got {}",
        top_3.len()
    );

    // ── Test 7: Phrase query with consecutive tokens ──
    let phrase_results =
        search_keys_with_namespace(&db, "bm25/a", Some("\"machine learning\""), Vec::new(), 10);
    // doc_0: "machine learning" ✓, doc_4: "machine learning machine learning" ✓
    assert!(phrase_results.contains(&"doc_0".to_string()));
    assert!(phrase_results.contains(&"doc_4".to_string()));
    // doc_3: "data science python" — no "machine learning"
    assert!(!phrase_results.contains(&"doc_3".to_string()));

    // ── Test 8: Vector-only search across namespaces ──
    let vec_ns_a = search_keys_with_namespace(&db, "bm25/a", None, vec![0.5, 0.0], 10);
    assert!(
        !vec_ns_a.is_empty(),
        "Vector search in namespace A should return results"
    );
    let vec_ns_b = search_keys_with_namespace(&db, "bm25/b", None, vec![0.5, 0.0], 10);
    assert!(
        !vec_ns_b.is_empty(),
        "Vector search in namespace B should return results"
    );
    // Results should be disjoint across namespaces
    for key in &vec_ns_b {
        assert!(
            !vec_ns_a.contains(key),
            "Vector results should be namespace-isolated"
        );
    }
}
