use super::Embedded;
use crate::cost_estimator::{CostEstimator, FilterStrategy};
use crate::node::DistanceMetric;
use crate::sdk::connect::connect;
use crate::sdk::types::*;

/// Open an in-memory VantaDB for testing.
fn setup() -> Embedded {
    connect(":memory:").expect("in-memory db open")
}

/// Insert a single record with optional vector and metadata.
fn insert(
    db: &Embedded,
    namespace: &str,
    key: &str,
    payload: &str,
    vector: Option<Vec<f32>>,
    metadata: MemoryMetadata,
) -> MemoryRecord {
    let input = MemoryInput {
        namespace: namespace.into(),
        key: key.into(),
        payload: payload.into(),
        metadata,
        vector,
        sparse_vector: None,
        ttl_ms: None,
        ..Default::default()
    };
    db.put(input).expect("put should succeed")
}

// ── empty / edge cases ─────────────────────────────────────

#[test]
fn test_search_empty_no_text_no_vector() {
    let db = setup();
    let req = MemorySearchRequest {
        namespace: "test".into(),
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
        ..Default::default()
    };
    let results = db.search(req).expect("search should succeed");
    assert!(results.is_empty(), "expected empty results");
}

#[test]
fn test_search_top_k_zero() {
    let db = setup();
    // Even with matching data, top_k=0 short-circuits
    insert(
        &db,
        "test",
        "k1",
        "hello world",
        Some(vec![0.1, 0.2, 0.3]),
        MemoryMetadata::new(),
    );

    // Text-only with top_k=0
    let req = MemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("hello".into()),
        top_k: 0,
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
        ..Default::default()
    };
    assert!(db.search(req).unwrap().is_empty());

    // Vector-only with top_k=0
    let req = MemorySearchRequest {
        namespace: "test".into(),
        query_vector: vec![0.1, 0.2, 0.3],
        top_k: 0,
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
        ..Default::default()
    };
    assert!(db.search(req).unwrap().is_empty());

    // Hybrid with top_k=0
    let req = MemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("hello".into()),
        query_vector: vec![0.1, 0.2, 0.3],
        top_k: 0,
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
        ..Default::default()
    };
    assert!(db.search(req).unwrap().is_empty());
}

#[test]
fn test_search_invalid_namespace() {
    let db = setup();
    let req = MemorySearchRequest {
        namespace: "".into(),
        text_query: Some("hello".into()),
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
        ..Default::default()
    };
    let err = db.search(req).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("namespace"),
        "expected namespace error, got: {msg}"
    );
}

// ── text-only lexical search ───────────────────────────────

#[test]
fn test_search_text_only_matching() {
    let db = setup();
    insert(
        &db,
        "test",
        "k1",
        "hello world welcome",
        None,
        MemoryMetadata::new(),
    );
    insert(
        &db,
        "test",
        "k2",
        "hello earth",
        None,
        MemoryMetadata::new(),
    );

    let req = MemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("hello".into()),
        top_k: 10,
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
        ..Default::default()
    };
    let results = db.search(req).expect("text search");
    assert!(!results.is_empty(), "expected hits for 'hello'");
    // Both records contain "hello"
    assert_eq!(results.len(), 2, "both records match 'hello'");
    // BM25 scores should be positive
    for hit in &results {
        assert!(
            hit.score > 0.0,
            "expected positive BM25 score, got {}",
            hit.score
        );
    }
}

#[test]
fn test_search_text_only_no_matches() {
    let db = setup();
    insert(
        &db,
        "test",
        "k1",
        "hello world",
        None,
        MemoryMetadata::new(),
    );

    let req = MemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("goodbye".into()),
        top_k: 10,
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
        ..Default::default()
    };
    let results = db.search(req).expect("text search");
    assert!(results.is_empty(), "expected no hits for 'goodbye'");
}

#[test]
fn test_search_text_only_with_filters() {
    let db = setup();
    let mut meta_a = MemoryMetadata::new();
    meta_a.insert("lang".into(), Value::String("en".into()));
    insert(&db, "test", "k1", "hello world", None, meta_a);

    let mut meta_b = MemoryMetadata::new();
    meta_b.insert("lang".into(), Value::String("es".into()));
    insert(&db, "test", "k2", "hola mundo", None, meta_b);

    // Search with filter for lang=en
    let mut filters = MemoryMetadata::new();
    filters.insert("lang".into(), Value::String("en".into()));
    let req = MemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("hello".into()),
        filters,
        top_k: 10,
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
        ..Default::default()
    };
    let results = db.search(req).expect("text search with filter");
    assert_eq!(results.len(), 1, "expected one hit matching lang=en");
    assert_eq!(results[0].record.key, "k1");
}

#[test]
fn test_search_text_only_filter_no_match() {
    let db = setup();
    let mut meta = MemoryMetadata::new();
    meta.insert("lang".into(), Value::String("en".into()));
    insert(&db, "test", "k1", "hello world", None, meta);

    let mut filters = MemoryMetadata::new();
    filters.insert("lang".into(), Value::String("de".into()));
    let req = MemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("hello".into()),
        filters,
        top_k: 10,
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
        ..Default::default()
    };
    let results = db
        .search(req)
        .expect("text search with non-matching filter");
    assert!(
        results.is_empty(),
        "expected no hits with non-matching filter"
    );
}

// ── vector-only HNSW search ────────────────────────────────

#[test]
fn test_search_vector_only_hnsw() {
    let db = setup();
    insert(
        &db,
        "test",
        "k1",
        "some text",
        Some(vec![0.1, 0.2, 0.3]),
        MemoryMetadata::new(),
    );

    // Search with exact same vector → cosine similarity = 1.0
    let req = MemorySearchRequest {
        namespace: "test".into(),
        query_vector: vec![0.1, 0.2, 0.3],
        top_k: 10,
        distance_metric: DistanceMetric::Cosine,
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
        ..Default::default()
    };
    let results = db.search(req).expect("vector search");
    assert_eq!(results.len(), 1, "expected one hit");
    assert!(
        results[0].score > 0.99,
        "expected near-perfect cosine score, got {}",
        results[0].score
    );
}

#[test]
fn test_search_vector_only_different_ns_no_match() {
    let db = setup();
    insert(
        &db,
        "ns1",
        "k1",
        "some text",
        Some(vec![0.1, 0.2, 0.3]),
        MemoryMetadata::new(),
    );

    // Search in a different namespace → no matches
    let req = MemorySearchRequest {
        namespace: "other".into(),
        query_vector: vec![0.1, 0.2, 0.3],
        top_k: 10,
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
        ..Default::default()
    };
    let results = db.search(req).expect("vector search different ns");
    assert!(
        results.is_empty(),
        "expected no hits in different namespace"
    );
}

#[test]
fn test_search_vector_only_with_filters() {
    let db = setup();
    let mut meta_a = MemoryMetadata::new();
    meta_a.insert("type".into(), Value::String("doc".into()));
    insert(
        &db,
        "test",
        "k1",
        "text a",
        Some(vec![0.1, 0.2, 0.3]),
        meta_a,
    );

    let mut meta_b = MemoryMetadata::new();
    meta_b.insert("type".into(), Value::String("image".into()));
    insert(
        &db,
        "test",
        "k2",
        "text b",
        Some(vec![0.1, 0.2, 0.3]),
        meta_b,
    );

    let mut filters = MemoryMetadata::new();
    filters.insert("type".into(), Value::String("doc".into()));
    let req = MemorySearchRequest {
        namespace: "test".into(),
        query_vector: vec![0.1, 0.2, 0.3],
        filters,
        top_k: 10,
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
        ..Default::default()
    };
    let results = db.search(req).expect("vector search with filter");
    assert_eq!(results.len(), 1, "expected one hit matching type=doc");
    assert_eq!(results[0].record.key, "k1");
}

#[test]
fn test_search_vector_only_no_matches() {
    let db = setup();
    insert(
        &db,
        "test",
        "k1",
        "text",
        Some(vec![0.9, 0.8, 0.7]),
        MemoryMetadata::new(),
    );

    // Search with a very different vector in an empty namespace
    let req = MemorySearchRequest {
        namespace: "empty_ns".into(),
        query_vector: vec![0.1, 0.2, 0.3],
        top_k: 10,
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
        ..Default::default()
    };
    let results = db.search(req).expect("vector search no matches");
    assert!(results.is_empty(), "expected no hits in empty namespace");
}

// ── hybrid search ──────────────────────────────────────────

#[test]
fn test_search_hybrid_both_text_and_vector() {
    let db = setup();
    // Two records, both containing "hello" and having similar vectors
    insert(
        &db,
        "test",
        "k1",
        "hello world",
        Some(vec![0.1, 0.2, 0.3]),
        MemoryMetadata::new(),
    );
    insert(
        &db,
        "test",
        "k2",
        "hello there",
        Some(vec![0.11, 0.21, 0.31]),
        MemoryMetadata::new(),
    );
    insert(
        &db,
        "test",
        "k3",
        "goodbye world",
        Some(vec![0.9, 0.8, 0.7]),
        MemoryMetadata::new(),
    );

    let req = MemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("hello".into()),
        query_vector: vec![0.1, 0.2, 0.3],
        top_k: 5,
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
        ..Default::default()
    };
    let results = db.search(req).expect("hybrid search");
    assert!(!results.is_empty(), "expected hybrid results");
    // k1 and k2 match "hello" AND similar vector; k3 only has similar-ish vector
    assert!(results.len() >= 2, "expected at least 2 hits");
    // Scores should be positive (RRF and BM25 combine)
    for hit in &results {
        assert!(
            hit.score > 0.0,
            "expected positive score, got {}",
            hit.score
        );
    }
    // Top result should be k1 (exact vector match + "hello")
    assert_eq!(results[0].record.key, "k1");
}

// ── explain mode ───────────────────────────────────────────

#[test]
fn test_search_explain_mode() {
    let db = setup();
    insert(
        &db,
        "test",
        "k1",
        "hello world",
        Some(vec![0.1, 0.2, 0.3]),
        MemoryMetadata::new(),
    );

    let req = MemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("hello".into()),
        query_vector: vec![0.1, 0.2, 0.3],
        top_k: 5,
        explain: true,
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
        ..Default::default()
    };
    let results = db.search(req).expect("explain search");
    assert_eq!(results.len(), 1, "expected one hit");
    let hit = &results[0];
    assert!(
        hit.explanation.is_some(),
        "expected explanation field in explain mode"
    );
    if let Some(explanation) = &hit.explanation {
        assert_eq!(explanation.identity, "test\0k1");
        assert!(!explanation.matched_tokens.is_empty());
    }
}

// ── BM25 scoring correctness ───────────────────────────────

/// BM25 scoring follows the standard formula:
///   IDF = ln(1 + (N - df + 0.5) / (df + 0.5))
///   score = IDF * (tf * (k1 + 1)) / (tf + k1 * (1 - b + b * doc_len / avg_doc_len))
#[test]
fn test_search_bm25_scoring_correctness() {
    let db = setup();
    // Insert two records in the same namespace to get N=2
    insert(
        &db,
        "test",
        "k1",
        "hello hello world", // "hello" appears twice in k1
        None,
        MemoryMetadata::new(),
    );
    insert(
        &db,
        "test",
        "k2",
        "hello foo bar", // "hello" appears once in k2
        None,
        MemoryMetadata::new(),
    );

    let req = MemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("hello".into()),
        top_k: 10,
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
        ..Default::default()
    };
    let results = db.search(req).expect("bm25 search");
    assert_eq!(results.len(), 2, "expected both records");

    // Both hits have positive BM25 scores
    for hit in &results {
        assert!(
            hit.score > 0.0,
            "expected positive BM25 score, got {}",
            hit.score
        );
    }

    // k1 has "hello" twice and "world" once (3 tokens), k2 has "hello" once and "foo","bar" (3 tokens)
    // "hello" appears in both documents → df=2 → IDF contributes equally
    // k1 has tf=2, k2 has tf=1 → k1 should score higher
    assert_eq!(
        results[0].record.key, "k1",
        "k1 has higher tf=2, should rank first"
    );
    assert!(
        results[0].score > results[1].score,
        "k1 (tf=2) should score higher than k2 (tf=1): {} vs {}",
        results[0].score,
        results[1].score
    );
}

// ── corrupt text index (debug only) ────────────────────────

#[cfg(debug_assertions)]
#[test]
fn test_search_corrupt_text_index_state() {
    let db = setup();
    insert(
        &db,
        "test",
        "k1",
        "hello world",
        None,
        MemoryMetadata::new(),
    );

    // Corrupt the text index state so ensure_text_index_query_ready fails
    db.debug_corrupt_text_index_state_for_tests()
        .expect("corrupt state");

    let req = MemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("hello".into()),
        top_k: 10,
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
        ..Default::default()
    };
    let err = db.search(req).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("text_index") || msg.contains("rebuild_index") || msg.contains("search"),
        "expected error from corrupt text index, got: {msg}"
    );
}

#[cfg(debug_assertions)]
#[test]
fn test_search_cleared_text_index_returns_empty() {
    let db = setup();
    insert(
        &db,
        "test",
        "k1",
        "hello world",
        None,
        MemoryMetadata::new(),
    );

    // Verify text search works before clearing
    let req = MemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("hello".into()),
        top_k: 10,
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
        ..Default::default()
    };
    let before = db.search(req.clone()).expect("search before clear");
    assert!(
        !before.is_empty(),
        "search should work before clearing index"
    );

    // Clear all text index entries (postings, stats)
    db.debug_clear_text_index_for_tests()
        .expect("clear text index");

    // After clearing, lexical search should return empty (no namespace stats)
    let after = db.search(req).expect("search after clear");
    assert!(after.is_empty(), "expected empty after clearing text index");
}

// ── empty query_vector (vector path, but empty) ────────────

#[test]
fn test_search_empty_query_vector_with_text() {
    let db = setup();
    insert(
        &db,
        "test",
        "k1",
        "hello world",
        None,
        MemoryMetadata::new(),
    );

    // text_query + empty query_vector → text-only path
    let req = MemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("hello".into()),
        query_vector: vec![], // explicitly empty
        top_k: 10,
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
        ..Default::default()
    };
    let results = db.search(req).expect("text-only with empty query vector");
    assert!(!results.is_empty(), "text-only should still work");
}

// ── euclidean distance ─────────────────────────────────────

#[test]
fn test_search_vector_only_euclidean() {
    let db = setup();
    insert(
        &db,
        "test",
        "k1",
        "text",
        Some(vec![0.1, 0.2, 0.3]),
        MemoryMetadata::new(),
    );
    insert(
        &db,
        "test",
        "k2",
        "text",
        Some(vec![0.9, 0.8, 0.7]),
        MemoryMetadata::new(),
    );

    let req = MemorySearchRequest {
        namespace: "test".into(),
        query_vector: vec![0.1, 0.2, 0.3],
        top_k: 5,
        distance_metric: DistanceMetric::Euclidean,
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
        ..Default::default()
    };
    let results = db.search(req).expect("euclidean search");
    // HNSW internally uses Cosine; Euclidean metric conversion only
    // applies in the brute-force fallback path. At minimum verify that
    // results are returned and ordered correctly.
    assert!(!results.is_empty(), "expected hits for euclidean");
    // k1 vector [0.1,0.2,0.3] is identical to query, k2 is further
    assert_eq!(
        results[0].record.key, "k1",
        "k1 has identical vector to query"
    );
}

// ── FilterStrategy ─────────────────────────────────────────

#[test]
fn test_select_filter_strategy_empty() {
    let db = setup();
    let engine = db.engine_handle().unwrap();
    let filters = MemoryMetadata::new();
    let strategy = CostEstimator::new(&engine).select_filter_strategy(&filters);
    assert_eq!(
        strategy,
        FilterStrategy::PostFilter,
        "empty filters → PostFilter"
    );
}

#[test]
fn test_select_filter_strategy_highly_selective() {
    let db = setup();
    // Insert two records with different "color" metadata.
    insert(
        &db,
        "test",
        "red_one",
        "text",
        Some(vec![0.1, 0.2]),
        MemoryMetadata::from([("color".into(), Value::String("red".into()))]),
    );
    insert(
        &db,
        "test",
        "blue_one",
        "text",
        Some(vec![0.3, 0.4]),
        MemoryMetadata::from([("color".into(), Value::String("blue".into()))]),
    );

    let engine = db.engine_handle().unwrap();
    let mut filters = MemoryMetadata::new();
    // "red" → 1 of 2 = selectivity 0.5.  That's above PREFILTER_THRESHOLD
    // but below HIGH_SELECTIVITY_THRESHOLD (0.1 < 0.5 < 0.1? no).
    // 0.5 is >= HIGH_SELECTIVITY_THRESHOLD (0.1) → PostFilter.
    // For a more selective test let's query a very rare value.
    // With only 2 records, "red" has freq 1 and total_nodes = 2, so sel = 0.5.
    // That's > 0.1 → PostFilter + 0.01.  Let's use a value that doesn't exist.
    // Non-existent value → selectivity 0.0 → PreFilter.
    filters.insert("nonexistent".into(), Value::String("nope".into()));
    let strategy = CostEstimator::new(&engine).select_filter_strategy(&filters);
    assert_eq!(
        strategy,
        FilterStrategy::PreFilter,
        "non-existent value → sel 0 → PreFilter"
    );
}

#[test]
fn test_select_filter_strategy_moderate() {
    let db = setup();
    // Insert enough records so that a single "color:red" has selectivity
    // in the InFilter range: 1 / N < 0.1 but >= 0.01.
    // N = 20 → sel = 0.05 → InFilter.
    for i in 0..20 {
        let color = if i == 0 { "red" } else { "blue" };
        insert(
            &db,
            "test",
            &format!("k{i}"),
            "text",
            Some(vec![0.1, 0.2]),
            MemoryMetadata::from([("color".into(), Value::String(color.into()))]),
        );
    }

    let engine = db.engine_handle().unwrap();
    let mut filters = MemoryMetadata::new();
    filters.insert("color".into(), Value::String("red".into()));
    let strategy = CostEstimator::new(&engine).select_filter_strategy(&filters);
    // "red" has freq 1 / 20 = 0.05 → InFilter
    assert_eq!(
        strategy,
        FilterStrategy::InFilter,
        "1 red out of 20 → sel 0.05 → InFilter"
    );
}

#[test]
fn test_vector_memory_search_with_pre_filter() {
    let db = setup();
    // Insert several records; only one has the target metadata.
    for i in 0..10 {
        let color = if i == 0 { "teal" } else { "gray" };
        insert(
            &db,
            "test",
            &format!("k{i}"),
            "text",
            Some(vec![i as f32 * 0.1, (i + 1) as f32 * 0.1]),
            MemoryMetadata::from([("color".into(), Value::String(color.into()))]),
        );
    }

    let engine = db.engine_handle().unwrap();
    // Force PreFilter by choosing a highly selective value.
    // "teal" → 1 of 10 → sel = 0.1 (= HIGH_SELECTIVITY_THRESHOLD, not < PREFILTER_THRESHOLD 0.01)
    // To get PreFilter, we need sel < 0.01.  With 10 records, use a nonexistent value → sel 0.0.
    let mut filters = MemoryMetadata::new();
    filters.insert("color".into(), Value::String("nonexistent_stuff".into()));

    let strategy = CostEstimator::new(&engine).select_filter_strategy(&filters);
    assert_eq!(
        strategy,
        FilterStrategy::PreFilter,
        "nonexistent → PreFilter"
    );

    let hits = db
        .vector_memory_search(
            "test",
            &[0.1, 0.2],
            &filters,
            5,
            DistanceMetric::Cosine,
            None,
        )
        .expect("pre-filter search");
    assert!(hits.is_empty(), "no records match 'nonexistent_stuff'");
}

#[test]
fn test_vector_memory_search_with_in_filter() {
    let db = setup();
    // 20 records, only "color:red" (1 record) → selectivity 0.05 → InFilter
    for i in 0..20 {
        let color = if i == 0 { "red" } else { "blue" };
        insert(
            &db,
            "test",
            &format!("k{i}"),
            "text",
            Some(vec![i as f32 * 0.1, (i + 1) as f32 * 0.1]),
            MemoryMetadata::from([("color".into(), Value::String(color.into()))]),
        );
    }

    let engine = db.engine_handle().unwrap();
    let mut filters = MemoryMetadata::new();
    filters.insert("color".into(), Value::String("red".into()));

    let strategy = CostEstimator::new(&engine).select_filter_strategy(&filters);
    assert_eq!(strategy, FilterStrategy::InFilter, "1/20 → InFilter");

    // Query close to [0.0, 0.1] (k0's vector) so k0 "red" ranks first.
    let hits = db
        .vector_memory_search(
            "test",
            &[0.0, 0.1],
            &filters,
            5,
            DistanceMetric::Cosine,
            None,
        )
        .expect("in-filter search");
    assert!(!hits.is_empty(), "should find k0 (red)");
    assert_eq!(
        hits[0].record.key, "k0",
        "k0 has vector [0.0, 0.1] closest to query"
    );
    for hit in &hits {
        assert_eq!(
            hit.record.metadata.get("color"),
            Some(&Value::String("red".into())),
            "only red records should appear"
        );
    }
}

#[test]
fn test_bitset_from_filters() {
    let db = setup();
    insert(
        &db,
        "test",
        "a",
        "text",
        None,
        MemoryMetadata::from([("group".into(), Value::String("alpha".into()))]),
    );
    insert(
        &db,
        "test",
        "b",
        "text",
        None,
        MemoryMetadata::from([("group".into(), Value::String("beta".into()))]),
    );
    insert(
        &db,
        "test",
        "c",
        "text",
        None,
        MemoryMetadata::from([("group".into(), Value::String("alpha".into()))]),
    );

    let mut filters = MemoryMetadata::new();
    filters.insert("group".into(), Value::String("alpha".into()));
    let bitset = db
        .bitset_from_filters("test", &filters)
        .expect("bitset from filters");
    assert!(!bitset.is_empty(), "bitset should contain alpha records");
    // "a" and "c" have alpha; verify via records
    let records = db.records_for_namespace("test", &filters).unwrap();
    assert_eq!(records.len(), 2, "two alpha records");
    assert!(
        bitset.has_bit(records[0].node_id as usize),
        "bitset has first alpha node_id"
    );
    assert!(
        bitset.has_bit(records[1].node_id as usize),
        "bitset has second alpha node_id"
    );
}

#[test]
fn test_vector_memory_search_with_metadata_filter() {
    let db = setup();
    // Insert two records with different metadata, same vector namespace.
    insert(
        &db,
        "test",
        "doc1",
        "payload1",
        Some(vec![0.5, 0.5]),
        MemoryMetadata::from([("department".into(), Value::String("engineering".into()))]),
    );
    insert(
        &db,
        "test",
        "doc2",
        "payload2",
        Some(vec![0.5, 0.5]),
        MemoryMetadata::from([("department".into(), Value::String("marketing".into()))]),
    );

    let query = vec![0.5, 0.5];
    let mut filters = MemoryMetadata::new();
    filters.insert("department".into(), Value::String("engineering".into()));

    let hits = db
        .vector_memory_search("test", &query, &filters, 10, DistanceMetric::Cosine, None)
        .expect("search with metadata filter");
    assert_eq!(hits.len(), 1, "only engineering doc should match");
    assert_eq!(hits[0].record.key, "doc1");
}

#[test]
fn test_vector_memory_search_no_filters() {
    let db = setup();
    insert(
        &db,
        "test",
        "a",
        "text",
        Some(vec![0.1, 0.2]),
        MemoryMetadata::new(),
    );
    insert(
        &db,
        "test",
        "b",
        "text",
        Some(vec![0.9, 0.8]),
        MemoryMetadata::new(),
    );

    // No filters → PostFilter (current behavior).
    let hits = db
        .vector_memory_search(
            "test",
            &[0.1, 0.2],
            &MemoryMetadata::new(),
            5,
            DistanceMetric::Cosine,
            None,
        )
        .expect("search without filters");
    assert!(!hits.is_empty(), "should find both records");
    assert_eq!(hits[0].record.key, "a", "closest vector is a");
}

// ── sparse vector search (ADR-019 round-trip) ──────────────

#[test]
fn test_sparse_search_roundtrip_recall_identical() {
    let db = setup();

    // Record with a sparse vector — write path now persists ListFloat pairs.
    let mut sparse = crate::node::SparseVector::new();
    sparse.insert(1, 0.5);
    sparse.insert(2, 1.0);
    let input = MemoryInput {
        namespace: "sparse".into(),
        key: "sparse-doc".into(),
        payload: "sparse payload".into(),
        metadata: MemoryMetadata::new(),
        vector: None,
        sparse_vector: Some(sparse),
        ttl_ms: None,
        ..Default::default()
    };
    let record = db.put(input).expect("put should succeed");
    assert!(record.sparse_vector.is_some(), "sparse survives put");

    // Fetch back from store (read path must reconstruct the sparse vector).
    let fetched = db
        .get("sparse", "sparse-doc")
        .expect("get should succeed")
        .expect("record should exist");
    let mut expected = crate::node::SparseVector::new();
    expected.insert(1, 0.5);
    expected.insert(2, 1.0);
    assert_eq!(fetched.sparse_vector, Some(expected));

    // Search by sparse query → recall identical to the stored vector.
    let mut query = crate::node::SparseVector::new();
    query.insert(2, 1.0);
    let req = MemorySearchRequest {
        namespace: "sparse".into(),
        query_sparse: Some(query),
        top_k: 5,
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
        ..Default::default()
    };
    let hits = db.search(req).expect("sparse search should succeed");
    assert!(!hits.is_empty(), "sparse query should hit the record");
    assert_eq!(hits[0].record.key, "sparse-doc");
}

// ── SearchProfileConfig (MEM-01) ──────────────────────────

#[test]
fn test_search_profile_mode_keyword_forces_lexical_only() {
    let db = setup();
    insert(
        &db,
        "test",
        "a",
        "cat chases mouse",
        Some(vec![1.0, 0.0, 0.0]),
        MemoryMetadata::new(),
    );
    insert(
        &db,
        "test",
        "b",
        "dog sleeps all day",
        Some(vec![0.0, 1.0, 0.0]),
        MemoryMetadata::new(),
    );

    // El vector favorece a "b", pero el modo Keyword ignora el canal vectorial:
    // solo "a" matchea el texto "cat".
    let req = MemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("cat".into()),
        query_vector: vec![0.0, 1.0, 0.0],
        top_k: 10,
        search_profile: Some(SearchProfileConfig {
            mode: SearchProfileMode::Keyword,
            ..Default::default()
        }),
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
        ..Default::default()
    };
    let hits = db.search(req).expect("keyword-mode search");
    let keys: Vec<_> = hits.iter().map(|h| h.record.key.as_str()).collect();
    assert_eq!(keys, vec!["a"], "keyword mode debe ignorar el vector");
}

#[test]
fn test_search_profile_mode_vector_ignores_text() {
    let db = setup();
    insert(
        &db,
        "test",
        "a",
        "cat chases mouse",
        Some(vec![1.0, 0.0, 0.0]),
        MemoryMetadata::new(),
    );
    insert(
        &db,
        "test",
        "b",
        "dog sleeps all day",
        Some(vec![0.0, 1.0, 0.0]),
        MemoryMetadata::new(),
    );

    // El texto favorece a "a", pero el modo Vector ignora el texto: el orden
    // es puramente vectorial (b mas cercano, luego a) — identico a un search
    // sin text_query. Si el texto influyera (hybrid), "a" subiria por BM25.
    let req = MemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("cat".into()),
        query_vector: vec![0.0, 1.0, 0.0],
        top_k: 10,
        search_profile: Some(SearchProfileConfig {
            mode: SearchProfileMode::Vector,
            ..Default::default()
        }),
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
        ..Default::default()
    };
    let hits = db.search(req).expect("vector-mode search");
    let keys: Vec<_> = hits.iter().map(|h| h.record.key.as_str()).collect();
    assert_eq!(
        keys,
        vec!["b", "a"],
        "vector mode: orden puramente vectorial"
    );

    // Control: vector-only sin texto produce el mismo orden.
    let req_control = MemorySearchRequest {
        namespace: "test".into(),
        query_vector: vec![0.0, 1.0, 0.0],
        top_k: 10,
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
        ..Default::default()
    };
    let control_hits = db.search(req_control).expect("vector-only control");
    let control_keys: Vec<_> = control_hits.iter().map(|h| h.record.key.as_str()).collect();
    assert_eq!(keys, control_keys, "mode Vector == vector-only puro");
}

#[test]
fn test_search_profile_hybrid_uses_both_channels() {
    let db = setup();
    insert(
        &db,
        "test",
        "a",
        "cat chases mouse",
        Some(vec![1.0, 0.0, 0.0]),
        MemoryMetadata::new(),
    );
    insert(
        &db,
        "test",
        "b",
        "dog sleeps all day",
        Some(vec![0.0, 1.0, 0.0]),
        MemoryMetadata::new(),
    );

    // Modo Hybrid (default): ambos canales participan, por lo que ambos keys
    // aparecen (a por texto, b por vector).
    let req = MemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("cat".into()),
        query_vector: vec![0.0, 1.0, 0.0],
        top_k: 10,
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
        ..Default::default()
    };
    let hits = db.search(req).expect("hybrid search");
    let mut keys: Vec<_> = hits.iter().map(|h| h.record.key.as_str()).collect();
    keys.sort_unstable();
    assert_eq!(keys, vec!["a", "b"], "hybrid mode usa ambos canales");
}

#[test]
fn test_search_profile_candidate_k_affects_budget() {
    let db = setup();
    insert(
        &db,
        "test",
        "a",
        "cat chases mouse",
        Some(vec![1.0, 0.0, 0.0]),
        MemoryMetadata::new(),
    );

    // candidate_k Some(64) con top_k=5 => budget = max(64, 5) = 64 (vs clamp core = 32).
    let req = MemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("cat".into()),
        query_vector: vec![1.0, 0.0, 0.0],
        top_k: 5,
        search_profile: Some(SearchProfileConfig {
            candidate_k: Some(64),
            ..Default::default()
        }),
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
        ..Default::default()
    };
    let plan = db
        .debug_memory_search_plan_for_tests(req)
        .expect("debug plan should succeed");
    assert_eq!(plan.budget, 64, "candidate_k del perfil define el budget");

    // Sin profile: clamp core (5*4=20 => 32).
    let req_default = MemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("cat".into()),
        query_vector: vec![1.0, 0.0, 0.0],
        top_k: 5,
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
        ..Default::default()
    };
    let plan_default = db
        .debug_memory_search_plan_for_tests(req_default)
        .expect("debug plan should succeed");
    assert_eq!(plan_default.budget, 32, "sin profile usa el clamp core");
}

#[test]
fn test_search_profile_rrf_k_reported_in_explain() {
    let db = setup();
    insert(
        &db,
        "test",
        "a",
        "cat chases mouse",
        Some(vec![1.0, 0.0, 0.0]),
        MemoryMetadata::new(),
    );

    // explain + profile rrf_k=100 => el fusion report expone rrf_k=100 (D20).
    let req = MemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("cat".into()),
        query_vector: vec![1.0, 0.0, 0.0],
        top_k: 5,
        explain: true,
        search_profile: Some(SearchProfileConfig {
            rrf_k: Some(100),
            ..Default::default()
        }),
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
        ..Default::default()
    };
    let explanation = db
        .explain_memory_search(req)
        .expect("explain should succeed");
    let report = explanation
        .fusion_report
        .expect("hybrid route debe tener fusion report");
    assert_eq!(report.rrf_k, 100, "rrf_k del perfil llega al report");
}

// ── entity boost (WIRE-05) ─────────────────────────────────

/// Fixture mirroring the fusion unit-test layout: lexical ⇒ [x, z],
/// dense ⇒ [y, z]; `z` leads the OFF ranking by accumulating two channels.
fn seed_entity_boost_fixture(db: &Embedded) -> MemorySearchRequest {
    insert(db, "boost-ns", "x", "alpha", None, MemoryMetadata::new());
    insert(
        db,
        "boost-ns",
        "y",
        "gamma delta",
        Some(vec![1.0, 0.0]),
        MemoryMetadata::new(),
    );
    insert(
        db,
        "boost-ns",
        "z",
        "alpha zeta eta theta iota kappa lambda mu nu xi omicron pi rho",
        Some(vec![0.8, 0.6]),
        MemoryMetadata::new(),
    );

    MemorySearchRequest {
        namespace: "boost-ns".into(),
        text_query: Some("alpha".into()),
        query_vector: vec![1.0, 0.0],
        top_k: 5,
        ..Default::default()
    }
}

fn linked_boost() -> EntityBoost {
    EntityBoost::new()
        .link("boost-ns", "x", "cluster-x")
        .link("boost-ns", "y", "cluster-x")
}

fn keys(hits: &[MemorySearchHit]) -> Vec<&str> {
    hits.iter().map(|hit| hit.record.key.as_str()).collect()
}

#[test]
fn test_search_hybrid_entity_boost_off_is_identical_to_search() {
    let db = setup();
    let request = seed_entity_boost_fixture(&db);

    let plain = db.search(request.clone()).expect("search");
    let boosted = db
        .search_with_entity_boost(request, &EntityBoost::new())
        .expect("boosted search");

    assert_eq!(plain, boosted.hits, "empty boost must be byte-identical");
    assert!(boosted.boost_report.is_empty());
}

#[test]
fn test_search_hybrid_entity_boost_promotes_linked_cluster() {
    let db = setup();
    let request = seed_entity_boost_fixture(&db);

    let plain = db.search(request.clone()).expect("search");
    assert_eq!(keys(&plain), vec!["z", "x", "y"], "baseline ranking");

    let boosted = db
        .search_with_entity_boost(request.clone(), &linked_boost().with_weight(1.0))
        .expect("boosted search");
    assert_eq!(
        keys(&boosted.hits),
        vec!["x", "y", "z"],
        "linked peers overtake z"
    );

    // Provenance: only the linked pair is reported, with peers + reversible delta.
    assert_eq!(boosted.boost_report.len(), 2);
    assert!(boosted.boost_report.get("boost-ns", "z").is_none());
    let x = boosted
        .boost_report
        .get("boost-ns", "x")
        .expect("x provenance");
    assert_eq!(x.cluster, "cluster-x");
    assert_eq!(x.peers, vec!["boost-ns\0y".to_string()]);
    let x_hit = boosted
        .hits
        .iter()
        .find(|hit| hit.record.key == "x")
        .expect("x hit");
    assert!((x_hit.score - x.delta - x.base_score).abs() < 1e-6);
}

#[test]
fn test_search_hybrid_entity_boost_is_deterministic() {
    let db = setup();
    let request = seed_entity_boost_fixture(&db);
    let boost = linked_boost().with_weight(0.5);

    let first = db
        .search_with_entity_boost(request.clone(), &boost)
        .expect("first");
    let second = db
        .search_with_entity_boost(request, &boost)
        .expect("second");
    assert_eq!(first, second, "same inputs → same hits + provenance");
}

#[test]
fn test_search_hybrid_entity_boost_explain_route_applies_boost() {
    let db = setup();
    let mut request = seed_entity_boost_fixture(&db);
    request.explain = true;

    let boosted = db
        .search_with_entity_boost(request, &linked_boost().with_weight(1.0))
        .expect("boosted explain search");
    assert_eq!(
        boosted.boost_report.len(),
        2,
        "boost applies in explain mode"
    );
    assert!(
        boosted.hits.iter().all(|hit| hit.explanation.is_some()),
        "explain mode still attaches per-hit explanations"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// WIRE-08: range / group_by / cursor / MMR
// ═══════════════════════════════════════════════════════════════════════════

fn metadata_with(field: &str, value: &str) -> MemoryMetadata {
    let mut metadata = MemoryMetadata::new();
    metadata.insert(field.into(), Value::String(value.into()));
    metadata
}

/// Five text-matching records whose BM25 scores differ (shorter doc → higher).
fn seed_range_fixture(db: &Embedded) {
    insert(db, "rg", "a", "alpha", None, MemoryMetadata::new());
    insert(db, "rg", "b", "alpha beta", None, MemoryMetadata::new());
    insert(
        db,
        "rg",
        "c",
        "alpha beta gamma",
        None,
        MemoryMetadata::new(),
    );
    insert(
        db,
        "rg",
        "d",
        "alpha beta gamma delta",
        None,
        MemoryMetadata::new(),
    );
    insert(
        db,
        "rg",
        "e",
        "alpha beta gamma delta epsilon",
        None,
        MemoryMetadata::new(),
    );
}

#[test]
fn test_range_filter_keeps_only_hits_inside_bounds() {
    let db = setup();
    seed_range_fixture(&db);

    let all = db
        .search(MemorySearchRequest {
            namespace: "rg".into(),
            text_query: Some("alpha".into()),
            top_k: 10,
            ..Default::default()
        })
        .expect("unfiltered search");
    assert_eq!(all.len(), 5);
    let max = all[0].score;

    // Upper bound only: drop the single best hit.
    let below = db
        .search(MemorySearchRequest {
            namespace: "rg".into(),
            text_query: Some("alpha".into()),
            top_k: 10,
            range: Some(RangeFilter {
                min_score: None,
                max_score: Some(max - 0.001),
            }),
            ..Default::default()
        })
        .expect("max-bound search");
    assert_eq!(below.len(), 4, "max_score must exclude the best hit");
    assert!(below.iter().all(|hit| hit.score <= max - 0.001));

    // Lower bound only: keep the single best hit.
    let top = db
        .search(MemorySearchRequest {
            namespace: "rg".into(),
            text_query: Some("alpha".into()),
            top_k: 10,
            range: Some(RangeFilter {
                min_score: Some(max - 0.001),
                max_score: None,
            }),
            ..Default::default()
        })
        .expect("min-bound search");
    assert_eq!(top.len(), 1);
    assert_eq!(top[0].record.key, all[0].record.key);
}

#[test]
fn test_range_filter_bounds_are_inclusive() {
    let db = setup();
    insert(&db, "rg", "a", "alpha", None, MemoryMetadata::new());
    let hit = db
        .search(MemorySearchRequest {
            namespace: "rg".into(),
            text_query: Some("alpha".into()),
            top_k: 10,
            ..Default::default()
        })
        .expect("baseline")[0]
        .clone();

    let exact = db
        .search(MemorySearchRequest {
            namespace: "rg".into(),
            text_query: Some("alpha".into()),
            top_k: 10,
            range: Some(RangeFilter {
                min_score: Some(hit.score),
                max_score: Some(hit.score),
            }),
            ..Default::default()
        })
        .expect("inclusive bounds");
    assert_eq!(exact.len(), 1, "min == max == score must keep the hit");
}

#[test]
fn test_range_filter_rejects_invalid_bounds() {
    let db = setup();
    let err = db
        .search(MemorySearchRequest {
            namespace: "rg".into(),
            text_query: Some("alpha".into()),
            range: Some(RangeFilter {
                min_score: Some(0.9),
                max_score: Some(0.1),
            }),
            ..Default::default()
        })
        .expect_err("min > max must fail");
    assert!(err.to_string().contains("SEARCH_OPTIONS_INVALID"));

    let err = db
        .search(MemorySearchRequest {
            namespace: "rg".into(),
            text_query: Some("alpha".into()),
            range: Some(RangeFilter {
                min_score: Some(f32::NAN),
                max_score: None,
            }),
            ..Default::default()
        })
        .expect_err("NaN bound must fail");
    assert!(err.to_string().contains("SEARCH_OPTIONS_INVALID"));
}

#[test]
fn test_group_by_limits_hits_per_group() {
    let db = setup();
    insert(&db, "gp", "a1", "alpha", None, metadata_with("doc", "A"));
    insert(
        &db,
        "gp",
        "a2",
        "alpha beta",
        None,
        metadata_with("doc", "A"),
    );
    insert(
        &db,
        "gp",
        "a3",
        "alpha beta gamma",
        None,
        metadata_with("doc", "A"),
    );
    insert(
        &db,
        "gp",
        "b1",
        "alpha beta gamma delta",
        None,
        metadata_with("doc", "B"),
    );

    let one_per_group = db
        .search(MemorySearchRequest {
            namespace: "gp".into(),
            text_query: Some("alpha".into()),
            top_k: 10,
            group_by: Some(GroupByConfig {
                field: "doc".into(),
                group_size: 1,
            }),
            ..Default::default()
        })
        .expect("group_by search");
    assert_eq!(one_per_group.len(), 2, "one hit per group value");
    assert_eq!(one_per_group[0].record.key, "a1", "best of group A first");
    assert_eq!(one_per_group[1].record.key, "b1");

    let two_per_group = db
        .search(MemorySearchRequest {
            namespace: "gp".into(),
            text_query: Some("alpha".into()),
            top_k: 10,
            group_by: Some(GroupByConfig {
                field: "doc".into(),
                group_size: 2,
            }),
            ..Default::default()
        })
        .expect("group_by search");
    assert_eq!(two_per_group.len(), 3, "A×2 + B×1");
    assert_eq!(
        two_per_group
            .iter()
            .filter(|hit| hit.record.metadata.get("doc") == Some(&Value::String("A".into())))
            .count(),
        2
    );
}

#[test]
fn test_group_by_top_k_caps_total_hits() {
    let db = setup();
    insert(&db, "gp", "a1", "alpha", None, metadata_with("doc", "A"));
    insert(
        &db,
        "gp",
        "a2",
        "alpha beta",
        None,
        metadata_with("doc", "A"),
    );
    insert(
        &db,
        "gp",
        "b1",
        "alpha beta gamma",
        None,
        metadata_with("doc", "B"),
    );

    let capped = db
        .search(MemorySearchRequest {
            namespace: "gp".into(),
            text_query: Some("alpha".into()),
            top_k: 2,
            group_by: Some(GroupByConfig {
                field: "doc".into(),
                group_size: 5,
            }),
            ..Default::default()
        })
        .expect("capped group_by search");
    assert_eq!(capped.len(), 2, "top_k caps total hits, not groups");
}

#[test]
fn test_group_by_missing_field_forms_own_group() {
    let db = setup();
    insert(&db, "gp", "a1", "alpha", None, metadata_with("doc", "A"));
    insert(&db, "gp", "a2", "alpha beta", None, MemoryMetadata::new());
    insert(
        &db,
        "gp",
        "a3",
        "alpha beta gamma",
        None,
        MemoryMetadata::new(),
    );

    let hits = db
        .search(MemorySearchRequest {
            namespace: "gp".into(),
            text_query: Some("alpha".into()),
            top_k: 10,
            group_by: Some(GroupByConfig {
                field: "doc".into(),
                group_size: 1,
            }),
            ..Default::default()
        })
        .expect("group_by search");
    assert_eq!(hits.len(), 2, "missing field groups together under one key");
}

#[test]
fn test_group_by_rejects_invalid_config() {
    let db = setup();
    for group_by in [
        GroupByConfig {
            field: "  ".into(),
            group_size: 1,
        },
        GroupByConfig {
            field: "doc".into(),
            group_size: 0,
        },
    ] {
        let err = db
            .search(MemorySearchRequest {
                namespace: "gp".into(),
                text_query: Some("alpha".into()),
                group_by: Some(group_by),
                ..Default::default()
            })
            .expect_err("invalid group_by must fail");
        assert!(err.to_string().contains("SEARCH_OPTIONS_INVALID"));
    }
}

#[test]
fn test_search_page_walks_all_pages_and_marks_last_page() {
    let db = setup();
    for i in 0..5 {
        insert(
            &db,
            "pg",
            &format!("k{i}"),
            &format!("alpha {}", "pad ".repeat(i + 1)),
            None,
            MemoryMetadata::new(),
        );
    }

    let mut seen: Vec<String> = Vec::new();
    let mut cursor: Option<String> = None;
    let mut pages = 0usize;
    loop {
        let page = db
            .search_page(MemorySearchRequest {
                namespace: "pg".into(),
                text_query: Some("alpha".into()),
                top_k: 2,
                cursor: cursor.clone(),
                ..Default::default()
            })
            .expect("page");
        pages += 1;
        assert!(
            page.hits.len() <= 2,
            "page never exceeds top_k: {}",
            page.hits.len()
        );
        for hit in &page.hits {
            assert!(
                !seen.contains(&hit.record.key),
                "duplicate key {} across pages",
                hit.record.key
            );
            seen.push(hit.record.key.clone());
        }
        match page.next_cursor {
            Some(next) => {
                assert_eq!(page.hits.len(), 2, "cursor implies a full page");
                cursor = Some(next);
            }
            None => {
                assert!(page.hits.len() < 2, "short page is the last page");
                break;
            }
        }
        assert!(pages <= 4, "pagination must terminate");
    }
    assert_eq!(pages, 3, "5 hits / page size 2 = 3 pages");
    assert_eq!(seen.len(), 5);
}

#[test]
fn test_search_page_resume_is_stable_with_interleaved_writes() {
    let db = setup();
    for i in 0..4 {
        insert(
            &db,
            "pg",
            &format!("k{i}"),
            &format!("alpha {}", "pad ".repeat(i + 1)),
            None,
            MemoryMetadata::new(),
        );
    }
    let original: Vec<String> = db
        .search(MemorySearchRequest {
            namespace: "pg".into(),
            text_query: Some("alpha".into()),
            top_k: 10,
            ..Default::default()
        })
        .expect("baseline")
        .into_iter()
        .map(|hit| hit.record.key)
        .collect();
    assert_eq!(original.len(), 4);

    let first = db
        .search_page(MemorySearchRequest {
            namespace: "pg".into(),
            text_query: Some("alpha".into()),
            top_k: 2,
            ..Default::default()
        })
        .expect("first page");
    let first_keys: Vec<String> = first
        .hits
        .iter()
        .map(|hit| hit.record.key.clone())
        .collect();
    assert_eq!(first_keys, original[..2].to_vec());

    // Interleaved write that would rank FIRST (shortest doc = best BM25).
    insert(&db, "pg", "new", "alpha", None, MemoryMetadata::new());
    let after_write: Vec<String> = db
        .search(MemorySearchRequest {
            namespace: "pg".into(),
            text_query: Some("alpha".into()),
            top_k: 10,
            ..Default::default()
        })
        .expect("after write")
        .into_iter()
        .map(|hit| hit.record.key)
        .collect();
    assert_eq!(
        after_write[0], "new",
        "fixture sanity: the new record ranks first"
    );

    // Resume: page 2 continues the ORIGINAL order — the new record ranks
    // before the anchor, so it is not visible to this pagination session, and
    // no hit is duplicated. The fetch window grows to compensate for the
    // write that landed before the anchor.
    let second = db
        .search_page(MemorySearchRequest {
            namespace: "pg".into(),
            text_query: Some("alpha".into()),
            top_k: 2,
            cursor: first.next_cursor,
            ..Default::default()
        })
        .expect("second page");
    let second_keys: Vec<String> = second
        .hits
        .iter()
        .map(|hit| hit.record.key.clone())
        .collect();
    assert_eq!(second_keys, original[2..].to_vec());
    assert!(
        !second_keys.contains(&"new".to_string()),
        "writes before the anchor stay invisible to the session"
    );

    // Page 3 is empty: the original corpus is exhausted (the full page 2
    // still emitted a cursor — clients fall out on the empty page).
    let third = db
        .search_page(MemorySearchRequest {
            namespace: "pg".into(),
            text_query: Some("alpha".into()),
            top_k: 2,
            cursor: second.next_cursor,
            ..Default::default()
        })
        .expect("third page");
    assert!(third.hits.is_empty());
    assert!(third.next_cursor.is_none(), "empty page ends the walk");
}

#[test]
fn test_search_page_cursor_is_bound_to_the_plan_fingerprint() {
    let db = setup();
    insert(&db, "pg", "a", "alpha", None, MemoryMetadata::new());
    insert(&db, "pg", "b", "alpha beta", None, MemoryMetadata::new());

    let first = db
        .search_page(MemorySearchRequest {
            namespace: "pg".into(),
            text_query: Some("alpha".into()),
            top_k: 1,
            ..Default::default()
        })
        .expect("first page");
    let cursor = first.next_cursor.expect("full page");

    let err = db
        .search_page(MemorySearchRequest {
            namespace: "pg".into(),
            text_query: Some("beta".into()),
            top_k: 1,
            cursor: Some(cursor.clone()),
            ..Default::default()
        })
        .expect_err("different plan must reject the cursor");
    assert!(err.to_string().contains("SEARCH_CURSOR_INVALID"));

    let err = db
        .search_page(MemorySearchRequest {
            namespace: "pg".into(),
            text_query: Some("alpha".into()),
            top_k: 1,
            cursor: Some("not-a-token".into()),
            ..Default::default()
        })
        .expect_err("garbage cursor must fail");
    assert!(err.to_string().contains("SEARCH_CURSOR_INVALID"));

    // The same plan still resumes.
    let second = db
        .search_page(MemorySearchRequest {
            namespace: "pg".into(),
            text_query: Some("alpha".into()),
            top_k: 1,
            cursor: Some(cursor),
            ..Default::default()
        })
        .expect("resume");
    assert_eq!(second.hits.len(), 1);
}

#[test]
fn test_search_page_rejects_cursor_with_mmr_and_group_by() {
    let db = setup();
    insert(&db, "pg", "a", "alpha", None, MemoryMetadata::new());
    insert(&db, "pg", "b", "alpha beta", None, MemoryMetadata::new());

    let first = db
        .search_page(MemorySearchRequest {
            namespace: "pg".into(),
            text_query: Some("alpha".into()),
            top_k: 1,
            ..Default::default()
        })
        .expect("first page");
    let cursor = first.next_cursor.expect("full page");

    let err = db
        .search_page(MemorySearchRequest {
            namespace: "pg".into(),
            text_query: Some("alpha".into()),
            top_k: 1,
            cursor: Some(cursor.clone()),
            mmr: Some(MmrConfig::default()),
            ..Default::default()
        })
        .expect_err("mmr + cursor must fail");
    assert!(err.to_string().contains("SEARCH_CURSOR_INVALID"));

    let err = db
        .search_page(MemorySearchRequest {
            namespace: "pg".into(),
            text_query: Some("alpha".into()),
            top_k: 1,
            cursor: Some(cursor),
            group_by: Some(GroupByConfig {
                field: "doc".into(),
                group_size: 1,
            }),
            ..Default::default()
        })
        .expect_err("group_by + cursor must fail");
    assert!(err.to_string().contains("SEARCH_CURSOR_INVALID"));
}

#[test]
fn test_search_page_empty_result_has_no_cursor() {
    let db = setup();
    let page = db
        .search_page(MemorySearchRequest {
            namespace: "pg".into(),
            text_query: Some("nothing-matches".into()),
            top_k: 5,
            ..Default::default()
        })
        .expect("empty page");
    assert!(page.hits.is_empty());
    assert!(page.next_cursor.is_none());
}

#[test]
fn test_search_without_cursor_returns_the_same_hits_as_search_page() {
    let db = setup();
    seed_range_fixture(&db);
    let request = MemorySearchRequest {
        namespace: "rg".into(),
        text_query: Some("alpha".into()),
        top_k: 3,
        ..Default::default()
    };
    let direct = db.search(request.clone()).expect("search");
    let page = db.search_page(request).expect("search_page");
    assert_eq!(direct, page.hits);
}

#[test]
fn test_mmr_diversifies_the_candidate_window() {
    let db = setup();
    insert(
        &db,
        "mm",
        "dup1",
        "alpha one",
        Some(vec![1.0, 0.0]),
        MemoryMetadata::new(),
    );
    insert(
        &db,
        "mm",
        "dup2",
        "alpha two",
        Some(vec![0.999, 0.001]),
        MemoryMetadata::new(),
    );
    insert(
        &db,
        "mm",
        "ortho",
        "alpha three",
        Some(vec![0.0, 1.0]),
        MemoryMetadata::new(),
    );

    let query = vec![1.0, 0.0];
    let plain = db
        .search(MemorySearchRequest {
            namespace: "mm".into(),
            query_vector: query.clone(),
            top_k: 2,
            ..Default::default()
        })
        .expect("plain vector search");
    assert_eq!(plain.len(), 2);

    let diversified = db
        .search(MemorySearchRequest {
            namespace: "mm".into(),
            query_vector: query,
            top_k: 2,
            mmr: Some(MmrConfig {
                lambda: 0.5,
                fetch_k: Some(3),
            }),
            ..Default::default()
        })
        .expect("mmr search");
    assert_eq!(diversified.len(), 2);
    let keys: Vec<&str> = diversified
        .iter()
        .map(|hit| hit.record.key.as_str())
        .collect();
    assert!(keys.contains(&"dup1"));
    assert!(
        keys.contains(&"ortho"),
        "MMR must trade the near-duplicate for the orthogonal vector: {keys:?}"
    );

    let pure_relevance = db
        .search(MemorySearchRequest {
            namespace: "mm".into(),
            query_vector: vec![1.0, 0.0],
            top_k: 2,
            mmr: Some(MmrConfig {
                lambda: 1.0,
                fetch_k: Some(3),
            }),
            ..Default::default()
        })
        .expect("lambda=1 search");
    assert_eq!(
        pure_relevance
            .iter()
            .map(|hit| hit.record.key.clone())
            .collect::<Vec<_>>(),
        plain
            .iter()
            .map(|hit| hit.record.key.clone())
            .collect::<Vec<_>>(),
        "lambda=1 is identity ordering"
    );
}

#[test]
fn test_search_page_resume_best_effort_when_writes_reorder_ranks() {
    // KNOWN best-effort behaviour (C1 review, WIRE-08): a resumed page can
    // return a hit that a previous page already returned when interleaved
    // writes reorder its rank *across* the anchor. This test documents that
    // trade-off instead of hiding it; a strong cursor (snapshot / server-side
    // session) is tracked as FIND-183.
    let db = setup();
    // P starts first (short doc, low length penalty); Q has higher tf but a
    // longer doc, so it overtakes P once the corpus average length grows.
    insert(&db, "ro", "P", "alpha", None, MemoryMetadata::new());
    insert(
        &db,
        "ro",
        "Q",
        "alpha alpha alpha pad pad pad pad pad pad",
        None,
        MemoryMetadata::new(),
    );

    let first = db
        .search_page(MemorySearchRequest {
            namespace: "ro".into(),
            text_query: Some("alpha".into()),
            top_k: 2,
            ..Default::default()
        })
        .expect("first page");
    let first_keys: Vec<String> = first
        .hits
        .iter()
        .map(|hit| hit.record.key.clone())
        .collect();
    assert_eq!(first_keys, vec!["P", "Q"], "precondition: P ranks first");

    // ~20 interleaved writes that flip P/Q (long docs raise avg doc length).
    let long_doc = format!("alpha {}", "pad ".repeat(50));
    for i in 0..20 {
        insert(
            &db,
            "ro",
            &format!("W{i}"),
            &long_doc,
            None,
            MemoryMetadata::new(),
        );
    }
    let after: Vec<String> = db
        .search(MemorySearchRequest {
            namespace: "ro".into(),
            text_query: Some("alpha".into()),
            top_k: 5,
            ..Default::default()
        })
        .expect("after writes")
        .into_iter()
        .map(|hit| hit.record.key)
        .collect();
    assert_eq!(
        after[..2],
        ["Q", "P"],
        "precondition: writes inverted P/Q ranks (got {after:?})"
    );

    // Page 2 resumes from the anchor `Q`, which now sits FIRST: skipping past
    // it returns `P` again — the documented best-effort duplicate.
    let second = db
        .search_page(MemorySearchRequest {
            namespace: "ro".into(),
            text_query: Some("alpha".into()),
            top_k: 2,
            cursor: first.next_cursor,
            ..Default::default()
        })
        .expect("second page");
    let second_keys: Vec<String> = second
        .hits
        .iter()
        .map(|hit| hit.record.key.clone())
        .collect();
    assert_eq!(
        second_keys,
        vec!["P", "W0"],
        "KNOWN best-effort: a rank reorder across the anchor may repeat P (FIND-183)"
    );
    assert!(
        !second_keys.contains(&"Q".to_string()),
        "the anchor itself is never repeated"
    );
}

#[test]
fn test_range_max_score_deepens_the_window_until_the_band_fills() {
    // R1 review (WIRE-08): with `max_score` the filter eats the head of the
    // ranking; the fetch window must grow so the page is not silently
    // truncated to fewer than `top_k` hits.
    let db = setup();
    seed_range_fixture(&db); // k0..k4 descending BM25 scores

    let baseline: Vec<MemorySearchHit> = db
        .search(MemorySearchRequest {
            namespace: "rg".into(),
            text_query: Some("alpha".into()),
            top_k: 10,
            ..Default::default()
        })
        .expect("baseline");
    assert_eq!(baseline.len(), 5);

    // Band below the first two hits: [scores[2], -inf).
    let band_max = baseline[2].score;
    let banded = db
        .search(MemorySearchRequest {
            namespace: "rg".into(),
            text_query: Some("alpha".into()),
            top_k: 2,
            range: Some(RangeFilter {
                min_score: None,
                max_score: Some(band_max),
            }),
            ..Default::default()
        })
        .expect("banded search");
    assert_eq!(
        banded.len(),
        2,
        "window deepening must fill the page from inside the band"
    );
    assert_eq!(banded[0].record.key, baseline[2].record.key);
    assert_eq!(banded[1].record.key, baseline[3].record.key);
    assert!(banded.iter().all(|hit| hit.score <= band_max));
}

#[test]
fn test_range_matching_nothing_returns_empty_page_without_cursor() {
    let db = setup();
    seed_range_fixture(&db);
    let page = db
        .search_page(MemorySearchRequest {
            namespace: "rg".into(),
            text_query: Some("alpha".into()),
            top_k: 2,
            range: Some(RangeFilter {
                min_score: Some(f32::MAX),
                max_score: None,
            }),
            ..Default::default()
        })
        .expect("empty band");
    assert!(page.hits.is_empty());
    assert!(page.next_cursor.is_none());
}

#[test]
fn test_search_page_top_k_zero_is_an_empty_page() {
    // O4: `top_k == 0` keeps the early return (ERR-033 convention: limit 0
    // means no records) — no cursor, no scan.
    let db = setup();
    seed_range_fixture(&db);
    let page = db
        .search_page(MemorySearchRequest {
            namespace: "rg".into(),
            text_query: Some("alpha".into()),
            top_k: 0,
            ..Default::default()
        })
        .expect("zero-k page");
    assert!(page.hits.is_empty());
    assert!(page.next_cursor.is_none());
}

#[test]
fn test_mmr_rejects_invalid_lambda_and_fetch_k() {
    let db = setup();
    for mmr in [
        MmrConfig {
            lambda: 1.5,
            fetch_k: None,
        },
        MmrConfig {
            lambda: -0.1,
            fetch_k: None,
        },
        MmrConfig {
            lambda: 0.5,
            fetch_k: Some(0),
        },
    ] {
        let err = db
            .search(MemorySearchRequest {
                namespace: "mm".into(),
                text_query: Some("alpha".into()),
                mmr: Some(mmr),
                ..Default::default()
            })
            .expect_err("invalid mmr must fail");
        assert!(err.to_string().contains("SEARCH_OPTIONS_INVALID"));
    }
}

// ── SCH-03: AS OF / valid-time filters (ADR-046 §D3) ──────────────────

/// Force an exact validity window on an existing record. Deterministic
/// time-travel fixture: no clock involvement anywhere in the assertions.
fn set_window(
    db: &Embedded,
    namespace: &str,
    key: &str,
    valid_at_ms: u64,
    invalid_at_ms: Option<u64>,
) {
    let mut record = db.get(namespace, key).expect("get").expect("record");
    record.valid_at_ms = valid_at_ms;
    record.invalid_at_ms = invalid_at_ms;
    db.put_record_exact(record)
        .expect("put record with exact window");
}

fn search_keys(db: &Embedded, request: MemorySearchRequest) -> Vec<String> {
    let mut keys: Vec<String> = db
        .search(request)
        .expect("search")
        .into_iter()
        .map(|hit| hit.record.key)
        .collect();
    keys.sort();
    keys
}

#[test]
fn test_search_as_of_returns_state_at_known_t() {
    let db = setup();
    insert(
        &db,
        "tt",
        "old",
        "alpha shared term",
        None,
        MemoryMetadata::new(),
    );
    insert(
        &db,
        "tt",
        "new",
        "alpha shared term",
        None,
        MemoryMetadata::new(),
    );
    set_window(&db, "tt", "old", 1000, Some(2000));
    set_window(&db, "tt", "new", 2000, Some(3000));

    let at = |t: u64| {
        search_keys(
            &db,
            MemorySearchRequest {
                namespace: "tt".into(),
                text_query: Some("alpha".into()),
                top_k: 10,
                as_of_ms: Some(t),
                ..Default::default()
            },
        )
    };
    assert!(at(999).is_empty(), "before any window");
    assert_eq!(at(1000), vec!["old"], "start is inclusive");
    assert_eq!(at(1999), vec!["old"]);
    assert_eq!(at(2000), vec!["new"], "end is exclusive");
    assert_eq!(at(2999), vec!["new"]);
    assert!(at(3000).is_empty(), "after both windows");

    // Default (no `as_of_ms`) is unchanged: both records remain searchable.
    let default = search_keys(
        &db,
        MemorySearchRequest {
            namespace: "tt".into(),
            text_query: Some("alpha".into()),
            top_k: 10,
            ..Default::default()
        },
    );
    assert_eq!(default, vec!["new", "old"]);
}

#[test]
fn test_search_valid_window_overlap_filters() {
    let db = setup();
    insert(&db, "tw", "old", "beta shared", None, MemoryMetadata::new());
    insert(&db, "tw", "new", "beta shared", None, MemoryMetadata::new());
    set_window(&db, "tw", "old", 1000, Some(2000));
    set_window(&db, "tw", "new", 2000, Some(3000));

    let in_window = |from: u64, to: u64| {
        search_keys(
            &db,
            MemorySearchRequest {
                namespace: "tw".into(),
                text_query: Some("beta".into()),
                top_k: 10,
                valid_window: Some(ValidWindow {
                    from_ms: from,
                    to_ms: to,
                }),
                ..Default::default()
            },
        )
    };
    assert!(in_window(0, 1000).is_empty(), "query ends at old.start");
    assert_eq!(in_window(0, 1001), vec!["old"]);
    assert_eq!(in_window(1500, 2500), vec!["new", "old"]);
    assert_eq!(in_window(1999, 2000), vec!["old"], "touching old.end");
    assert_eq!(in_window(2000, 3000), vec!["new"]);
    assert!(in_window(3000, 4000).is_empty(), "query starts at new.end");
}

#[test]
fn test_search_rejects_empty_valid_window() {
    let db = setup();
    let err = db
        .search(MemorySearchRequest {
            namespace: "tt".into(),
            text_query: Some("alpha".into()),
            valid_window: Some(ValidWindow {
                from_ms: 10,
                to_ms: 10,
            }),
            ..Default::default()
        })
        .expect_err("empty window must be rejected at the boundary");
    assert!(err.to_string().contains("SEARCH_OPTIONS_INVALID"));
}

#[test]
fn test_exclude_superseded_also_drops_ended_validity_windows() {
    let db = setup();
    insert(
        &db,
        "ex",
        "ended",
        "gamma term",
        None,
        MemoryMetadata::new(),
    );
    insert(&db, "ex", "open", "gamma term", None, MemoryMetadata::new());
    insert(
        &db,
        "ex",
        "future_end",
        "gamma term",
        None,
        MemoryMetadata::new(),
    );
    // Ended in the distant past → dropped under any real clock (deterministic).
    set_window(&db, "ex", "ended", 1, Some(2));
    set_window(&db, "ex", "open", 1, None);
    // Ends far in the future → not "ended yet".
    set_window(&db, "ex", "future_end", 1, Some(u64::MAX / 2));

    let hidden = search_keys(
        &db,
        MemorySearchRequest {
            namespace: "ex".into(),
            text_query: Some("gamma".into()),
            top_k: 10,
            exclude_superseded: true,
            ..Default::default()
        },
    );
    assert_eq!(hidden, vec!["future_end", "open"], "ended window dropped");

    let shown = search_keys(
        &db,
        MemorySearchRequest {
            namespace: "ex".into(),
            text_query: Some("gamma".into()),
            top_k: 10,
            ..Default::default()
        },
    );
    assert_eq!(
        shown,
        vec!["ended", "future_end", "open"],
        "default keeps every record"
    );
}

#[test]
fn test_search_page_cursor_is_bound_to_temporal_params() {
    let db = setup();
    insert(&db, "cp", "a", "delta term", None, MemoryMetadata::new());
    insert(&db, "cp", "b", "delta term", None, MemoryMetadata::new());
    set_window(&db, "cp", "a", 1000, None);
    set_window(&db, "cp", "b", 2000, None);

    let first = db
        .search_page(MemorySearchRequest {
            namespace: "cp".into(),
            text_query: Some("delta".into()),
            top_k: 1,
            as_of_ms: Some(1500),
            ..Default::default()
        })
        .expect("first page");
    assert_eq!(first.hits.len(), 1);
    assert_eq!(first.hits[0].record.key, "a", "only `a` is valid at T=1500");
    let cursor = first.next_cursor.expect("full page yields a cursor");

    // Resuming with a different valid-time param is rejected (fingerprint).
    let err = db
        .search_page(MemorySearchRequest {
            namespace: "cp".into(),
            text_query: Some("delta".into()),
            top_k: 1,
            as_of_ms: Some(2500),
            cursor: Some(cursor.clone()),
            ..Default::default()
        })
        .expect_err("cursor from a different temporal plan must be rejected");
    assert!(err.to_string().contains("SEARCH_CURSOR_INVALID"));

    // Same params resume without duplicates (page is exhausted at T=1500).
    let second = db
        .search_page(MemorySearchRequest {
            namespace: "cp".into(),
            text_query: Some("delta".into()),
            top_k: 1,
            as_of_ms: Some(1500),
            cursor: Some(cursor),
            ..Default::default()
        })
        .expect("resume");
    assert!(second.hits.is_empty(), "no other record is valid at T=1500");
    assert!(second.next_cursor.is_none());
}

#[test]
fn test_search_as_of_page_fills_top_k_when_enough_valid_candidates_exist() {
    let db = setup();
    // 10 records; 8 end before T so the selector eats the ranking head.
    for index in 0..10 {
        let key = format!("k{index}");
        insert(
            &db,
            "gw",
            &key,
            "epsilon shared",
            None,
            MemoryMetadata::new(),
        );
        let window_end = if index < 8 { Some(2) } else { None };
        set_window(&db, "gw", &key, 1, window_end);
    }

    let page = db
        .search_page(MemorySearchRequest {
            namespace: "gw".into(),
            text_query: Some("epsilon".into()),
            top_k: 2,
            as_of_ms: Some(1500),
            ..Default::default()
        })
        .expect("page");
    assert_eq!(
        page.hits.len(),
        2,
        "the selector must not shorten the page below top_k"
    );
    assert!(page.next_cursor.is_some(), "full page yields a cursor");
    for hit in &page.hits {
        assert!(
            hit.record.is_valid_at(1500),
            "every returned hit must be valid at T"
        );
    }
}

// ── SCH-04: confidence threshold filter (ADR-046 §D2) ──────────────────────

/// Put an asserted record with a declared confidence (score kept verbatim).
fn insert_with_confidence(
    db: &Embedded,
    namespace: &str,
    key: &str,
    payload: &str,
    confidence: f32,
) -> MemoryRecord {
    let input = MemoryInput {
        namespace: namespace.into(),
        key: key.into(),
        payload: payload.into(),
        confidence: Some(confidence),
        ..Default::default()
    };
    db.put(input).expect("put with declared confidence")
}

#[test]
fn test_search_min_confidence_filters_below_threshold() {
    let db = setup();
    // Parent asserted at 0.5; derived child = min(0.5) × 0.9 = 0.45 (D4a).
    insert_with_confidence(&db, "conf", "weak-parent", "shared wording", 0.5);
    let derived = MemoryInput {
        namespace: "conf".into(),
        key: "derived-child".into(),
        payload: "shared wording".into(),
        confidence_class: Some(ConfidenceClass::Derived),
        derived_from: Some(vec!["weak-parent".into()]),
        ..Default::default()
    };
    let rec = db.put(derived).expect("derived put");
    assert_eq!(rec.confidence_class, ConfidenceClass::Derived);
    assert!(
        (rec.confidence - 0.45).abs() < 1e-6,
        "derived score = min(parents) × 0.9, got {}",
        rec.confidence
    );

    // Default (`None`) keeps both hits — no behavior change.
    let all = db
        .search(MemorySearchRequest {
            namespace: "conf".into(),
            text_query: Some("shared".into()),
            top_k: 10,
            ..Default::default()
        })
        .expect("search");
    assert_eq!(all.len(), 2, "unfiltered search keeps both records");

    // Threshold 0.5 drops the derived child (0.45) and keeps the parent.
    let filtered = db
        .search(MemorySearchRequest {
            namespace: "conf".into(),
            text_query: Some("shared".into()),
            top_k: 10,
            min_confidence: Some(0.5),
            ..Default::default()
        })
        .expect("filtered search");
    assert_eq!(filtered.len(), 1, "only the parent passes the threshold");
    assert_eq!(filtered[0].record.key, "weak-parent");
    assert!(filtered[0].record.confidence >= 0.5);
}

#[test]
fn test_search_min_confidence_out_of_range_is_rejected() {
    let db = setup();
    insert_with_confidence(&db, "conf", "k1", "alpha", 1.0);

    for bad in [1.5_f32, -0.1, f32::NAN, f32::INFINITY] {
        let err = db
            .search(MemorySearchRequest {
                namespace: "conf".into(),
                text_query: Some("alpha".into()),
                min_confidence: Some(bad),
                ..Default::default()
            })
            .expect_err("out-of-range threshold must be rejected");
        let msg = err.to_string();
        assert!(
            msg.contains("SEARCH_OPTIONS_INVALID"),
            "stable marker expected, got: {msg}"
        );
        assert!(msg.contains("min_confidence"), "field name in error: {msg}");
    }
}

#[test]
fn test_search_min_confidence_is_part_of_the_cursor_fingerprint() {
    let db = setup();
    insert_with_confidence(&db, "conf", "k1", "alpha beta", 1.0);
    insert_with_confidence(&db, "conf", "k2", "alpha gamma", 1.0);

    let first = db
        .search_page(MemorySearchRequest {
            namespace: "conf".into(),
            text_query: Some("alpha".into()),
            top_k: 1,
            ..Default::default()
        })
        .expect("first page");
    assert_eq!(first.hits.len(), 1);
    let cursor = first.next_cursor.expect("full page yields a cursor");

    // Reusing the cursor with a different threshold must be rejected (the
    // threshold is part of the plan fingerprint).
    let err = db
        .search_page(MemorySearchRequest {
            namespace: "conf".into(),
            text_query: Some("alpha".into()),
            top_k: 1,
            min_confidence: Some(0.5),
            cursor: Some(cursor),
            ..Default::default()
        })
        .expect_err("cursor from a different threshold must be rejected");
    let msg = err.to_string();
    assert!(msg.contains("SEARCH_CURSOR_INVALID"), "got: {msg}");
}

#[test]
fn test_search_min_confidence_page_fills_when_enough_candidates_exist() {
    let db = setup();
    // Weak records rank first (higher term frequency) but are filtered by the
    // threshold; the window must grow until the strong ones fill the page.
    for index in 0..6 {
        let key = format!("weak-{index}");
        insert_with_confidence(
            &db,
            "conf",
            &key,
            "theta theta theta theta theta theta theta theta",
            0.4,
        );
    }
    for index in 0..5 {
        let key = format!("strong-{index}");
        insert_with_confidence(&db, "conf", &key, "theta", 1.0);
    }

    let page = db
        .search_page(MemorySearchRequest {
            namespace: "conf".into(),
            text_query: Some("theta".into()),
            top_k: 3,
            min_confidence: Some(0.5),
            ..Default::default()
        })
        .expect("page");
    assert_eq!(
        page.hits.len(),
        3,
        "the confidence selector must not shorten the page below top_k"
    );
    for hit in &page.hits {
        assert!(
            hit.record.confidence >= 0.5,
            "every returned hit must pass the threshold"
        );
    }
}
