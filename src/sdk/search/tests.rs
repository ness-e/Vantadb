use super::VantaEmbedded;
use crate::cost_estimator::{CostEstimator, FilterStrategy};
use crate::node::DistanceMetric;
use crate::sdk::connect::connect;
use crate::sdk::types::*;

/// Open an in-memory VantaDB for testing.
fn setup() -> VantaEmbedded {
    connect(":memory:").expect("in-memory db open")
}

/// Insert a single record with optional vector and metadata.
fn insert(
    db: &VantaEmbedded,
    namespace: &str,
    key: &str,
    payload: &str,
    vector: Option<Vec<f32>>,
    metadata: VantaMemoryMetadata,
) -> VantaMemoryRecord {
    let input = VantaMemoryInput {
        namespace: namespace.into(),
        key: key.into(),
        payload: payload.into(),
        metadata,
        vector,
        sparse_vector: None,
        ttl_ms: None,
    };
    db.put(input).expect("put should succeed")
}

// ── empty / edge cases ─────────────────────────────────────

#[test]
fn test_search_empty_no_text_no_vector() {
    let db = setup();
    let req = VantaMemorySearchRequest {
        namespace: "test".into(),
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
        VantaMemoryMetadata::new(),
    );

    // Text-only with top_k=0
    let req = VantaMemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("hello".into()),
        top_k: 0,
        ..Default::default()
    };
    assert!(db.search(req).unwrap().is_empty());

    // Vector-only with top_k=0
    let req = VantaMemorySearchRequest {
        namespace: "test".into(),
        query_vector: vec![0.1, 0.2, 0.3],
        top_k: 0,
        ..Default::default()
    };
    assert!(db.search(req).unwrap().is_empty());

    // Hybrid with top_k=0
    let req = VantaMemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("hello".into()),
        query_vector: vec![0.1, 0.2, 0.3],
        top_k: 0,
        ..Default::default()
    };
    assert!(db.search(req).unwrap().is_empty());
}

#[test]
fn test_search_invalid_namespace() {
    let db = setup();
    let req = VantaMemorySearchRequest {
        namespace: "".into(),
        text_query: Some("hello".into()),
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
        VantaMemoryMetadata::new(),
    );
    insert(
        &db,
        "test",
        "k2",
        "hello earth",
        None,
        VantaMemoryMetadata::new(),
    );

    let req = VantaMemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("hello".into()),
        top_k: 10,
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
        VantaMemoryMetadata::new(),
    );

    let req = VantaMemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("goodbye".into()),
        top_k: 10,
        ..Default::default()
    };
    let results = db.search(req).expect("text search");
    assert!(results.is_empty(), "expected no hits for 'goodbye'");
}

#[test]
fn test_search_text_only_with_filters() {
    let db = setup();
    let mut meta_a = VantaMemoryMetadata::new();
    meta_a.insert("lang".into(), VantaValue::String("en".into()));
    insert(&db, "test", "k1", "hello world", None, meta_a);

    let mut meta_b = VantaMemoryMetadata::new();
    meta_b.insert("lang".into(), VantaValue::String("es".into()));
    insert(&db, "test", "k2", "hola mundo", None, meta_b);

    // Search with filter for lang=en
    let mut filters = VantaMemoryMetadata::new();
    filters.insert("lang".into(), VantaValue::String("en".into()));
    let req = VantaMemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("hello".into()),
        filters,
        top_k: 10,
        ..Default::default()
    };
    let results = db.search(req).expect("text search with filter");
    assert_eq!(results.len(), 1, "expected one hit matching lang=en");
    assert_eq!(results[0].record.key, "k1");
}

#[test]
fn test_search_text_only_filter_no_match() {
    let db = setup();
    let mut meta = VantaMemoryMetadata::new();
    meta.insert("lang".into(), VantaValue::String("en".into()));
    insert(&db, "test", "k1", "hello world", None, meta);

    let mut filters = VantaMemoryMetadata::new();
    filters.insert("lang".into(), VantaValue::String("de".into()));
    let req = VantaMemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("hello".into()),
        filters,
        top_k: 10,
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
        VantaMemoryMetadata::new(),
    );

    // Search with exact same vector → cosine similarity = 1.0
    let req = VantaMemorySearchRequest {
        namespace: "test".into(),
        query_vector: vec![0.1, 0.2, 0.3],
        top_k: 10,
        distance_metric: DistanceMetric::Cosine,
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
        VantaMemoryMetadata::new(),
    );

    // Search in a different namespace → no matches
    let req = VantaMemorySearchRequest {
        namespace: "other".into(),
        query_vector: vec![0.1, 0.2, 0.3],
        top_k: 10,
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
    let mut meta_a = VantaMemoryMetadata::new();
    meta_a.insert("type".into(), VantaValue::String("doc".into()));
    insert(
        &db,
        "test",
        "k1",
        "text a",
        Some(vec![0.1, 0.2, 0.3]),
        meta_a,
    );

    let mut meta_b = VantaMemoryMetadata::new();
    meta_b.insert("type".into(), VantaValue::String("image".into()));
    insert(
        &db,
        "test",
        "k2",
        "text b",
        Some(vec![0.1, 0.2, 0.3]),
        meta_b,
    );

    let mut filters = VantaMemoryMetadata::new();
    filters.insert("type".into(), VantaValue::String("doc".into()));
    let req = VantaMemorySearchRequest {
        namespace: "test".into(),
        query_vector: vec![0.1, 0.2, 0.3],
        filters,
        top_k: 10,
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
        VantaMemoryMetadata::new(),
    );

    // Search with a very different vector in an empty namespace
    let req = VantaMemorySearchRequest {
        namespace: "empty_ns".into(),
        query_vector: vec![0.1, 0.2, 0.3],
        top_k: 10,
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
        VantaMemoryMetadata::new(),
    );
    insert(
        &db,
        "test",
        "k2",
        "hello there",
        Some(vec![0.11, 0.21, 0.31]),
        VantaMemoryMetadata::new(),
    );
    insert(
        &db,
        "test",
        "k3",
        "goodbye world",
        Some(vec![0.9, 0.8, 0.7]),
        VantaMemoryMetadata::new(),
    );

    let req = VantaMemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("hello".into()),
        query_vector: vec![0.1, 0.2, 0.3],
        top_k: 5,
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
        VantaMemoryMetadata::new(),
    );

    let req = VantaMemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("hello".into()),
        query_vector: vec![0.1, 0.2, 0.3],
        top_k: 5,
        explain: true,
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
        VantaMemoryMetadata::new(),
    );
    insert(
        &db,
        "test",
        "k2",
        "hello foo bar", // "hello" appears once in k2
        None,
        VantaMemoryMetadata::new(),
    );

    let req = VantaMemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("hello".into()),
        top_k: 10,
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
        VantaMemoryMetadata::new(),
    );

    // Corrupt the text index state so ensure_text_index_query_ready fails
    db.debug_corrupt_text_index_state_for_tests()
        .expect("corrupt state");

    let req = VantaMemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("hello".into()),
        top_k: 10,
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
        VantaMemoryMetadata::new(),
    );

    // Verify text search works before clearing
    let req = VantaMemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("hello".into()),
        top_k: 10,
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
        VantaMemoryMetadata::new(),
    );

    // text_query + empty query_vector → text-only path
    let req = VantaMemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("hello".into()),
        query_vector: vec![], // explicitly empty
        top_k: 10,
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
        VantaMemoryMetadata::new(),
    );
    insert(
        &db,
        "test",
        "k2",
        "text",
        Some(vec![0.9, 0.8, 0.7]),
        VantaMemoryMetadata::new(),
    );

    let req = VantaMemorySearchRequest {
        namespace: "test".into(),
        query_vector: vec![0.1, 0.2, 0.3],
        top_k: 5,
        distance_metric: DistanceMetric::Euclidean,
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
    let filters = VantaMemoryMetadata::new();
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
        VantaMemoryMetadata::from([("color".into(), VantaValue::String("red".into()))]),
    );
    insert(
        &db,
        "test",
        "blue_one",
        "text",
        Some(vec![0.3, 0.4]),
        VantaMemoryMetadata::from([("color".into(), VantaValue::String("blue".into()))]),
    );

    let engine = db.engine_handle().unwrap();
    let mut filters = VantaMemoryMetadata::new();
    // "red" → 1 of 2 = selectivity 0.5.  That's above PREFILTER_THRESHOLD
    // but below HIGH_SELECTIVITY_THRESHOLD (0.1 < 0.5 < 0.1? no).
    // 0.5 is >= HIGH_SELECTIVITY_THRESHOLD (0.1) → PostFilter.
    // For a more selective test let's query a very rare value.
    // With only 2 records, "red" has freq 1 and total_nodes = 2, so sel = 0.5.
    // That's > 0.1 → PostFilter + 0.01.  Let's use a value that doesn't exist.
    // Non-existent value → selectivity 0.0 → PreFilter.
    filters.insert("nonexistent".into(), VantaValue::String("nope".into()));
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
            VantaMemoryMetadata::from([("color".into(), VantaValue::String(color.into()))]),
        );
    }

    let engine = db.engine_handle().unwrap();
    let mut filters = VantaMemoryMetadata::new();
    filters.insert("color".into(), VantaValue::String("red".into()));
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
            VantaMemoryMetadata::from([("color".into(), VantaValue::String(color.into()))]),
        );
    }

    let engine = db.engine_handle().unwrap();
    // Force PreFilter by choosing a highly selective value.
    // "teal" → 1 of 10 → sel = 0.1 (= HIGH_SELECTIVITY_THRESHOLD, not < PREFILTER_THRESHOLD 0.01)
    // To get PreFilter, we need sel < 0.01.  With 10 records, use a nonexistent value → sel 0.0.
    let mut filters = VantaMemoryMetadata::new();
    filters.insert(
        "color".into(),
        VantaValue::String("nonexistent_stuff".into()),
    );

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
            VantaMemoryMetadata::from([("color".into(), VantaValue::String(color.into()))]),
        );
    }

    let engine = db.engine_handle().unwrap();
    let mut filters = VantaMemoryMetadata::new();
    filters.insert("color".into(), VantaValue::String("red".into()));

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
            Some(&VantaValue::String("red".into())),
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
        VantaMemoryMetadata::from([("group".into(), VantaValue::String("alpha".into()))]),
    );
    insert(
        &db,
        "test",
        "b",
        "text",
        None,
        VantaMemoryMetadata::from([("group".into(), VantaValue::String("beta".into()))]),
    );
    insert(
        &db,
        "test",
        "c",
        "text",
        None,
        VantaMemoryMetadata::from([("group".into(), VantaValue::String("alpha".into()))]),
    );

    let mut filters = VantaMemoryMetadata::new();
    filters.insert("group".into(), VantaValue::String("alpha".into()));
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
        VantaMemoryMetadata::from([(
            "department".into(),
            VantaValue::String("engineering".into()),
        )]),
    );
    insert(
        &db,
        "test",
        "doc2",
        "payload2",
        Some(vec![0.5, 0.5]),
        VantaMemoryMetadata::from([("department".into(), VantaValue::String("marketing".into()))]),
    );

    let query = vec![0.5, 0.5];
    let mut filters = VantaMemoryMetadata::new();
    filters.insert(
        "department".into(),
        VantaValue::String("engineering".into()),
    );

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
        VantaMemoryMetadata::new(),
    );
    insert(
        &db,
        "test",
        "b",
        "text",
        Some(vec![0.9, 0.8]),
        VantaMemoryMetadata::new(),
    );

    // No filters → PostFilter (current behavior).
    let hits = db
        .vector_memory_search(
            "test",
            &[0.1, 0.2],
            &VantaMemoryMetadata::new(),
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
    let input = VantaMemoryInput {
        namespace: "sparse".into(),
        key: "sparse-doc".into(),
        payload: "sparse payload".into(),
        metadata: VantaMemoryMetadata::new(),
        vector: None,
        sparse_vector: Some(sparse),
        ttl_ms: None,
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
    let req = VantaMemorySearchRequest {
        namespace: "sparse".into(),
        query_sparse: Some(query),
        top_k: 5,
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
        VantaMemoryMetadata::new(),
    );
    insert(
        &db,
        "test",
        "b",
        "dog sleeps all day",
        Some(vec![0.0, 1.0, 0.0]),
        VantaMemoryMetadata::new(),
    );

    // El vector favorece a "b", pero el modo Keyword ignora el canal vectorial:
    // solo "a" matchea el texto "cat".
    let req = VantaMemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("cat".into()),
        query_vector: vec![0.0, 1.0, 0.0],
        top_k: 10,
        search_profile: Some(SearchProfileConfig {
            mode: SearchProfileMode::Keyword,
            ..Default::default()
        }),
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
        VantaMemoryMetadata::new(),
    );
    insert(
        &db,
        "test",
        "b",
        "dog sleeps all day",
        Some(vec![0.0, 1.0, 0.0]),
        VantaMemoryMetadata::new(),
    );

    // El texto favorece a "a", pero el modo Vector ignora el texto: el orden
    // es puramente vectorial (b mas cercano, luego a) — identico a un search
    // sin text_query. Si el texto influyera (hybrid), "a" subiria por BM25.
    let req = VantaMemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("cat".into()),
        query_vector: vec![0.0, 1.0, 0.0],
        top_k: 10,
        search_profile: Some(SearchProfileConfig {
            mode: SearchProfileMode::Vector,
            ..Default::default()
        }),
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
    let req_control = VantaMemorySearchRequest {
        namespace: "test".into(),
        query_vector: vec![0.0, 1.0, 0.0],
        top_k: 10,
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
        VantaMemoryMetadata::new(),
    );
    insert(
        &db,
        "test",
        "b",
        "dog sleeps all day",
        Some(vec![0.0, 1.0, 0.0]),
        VantaMemoryMetadata::new(),
    );

    // Modo Hybrid (default): ambos canales participan, por lo que ambos keys
    // aparecen (a por texto, b por vector).
    let req = VantaMemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("cat".into()),
        query_vector: vec![0.0, 1.0, 0.0],
        top_k: 10,
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
        VantaMemoryMetadata::new(),
    );

    // candidate_k Some(64) con top_k=5 => budget = max(64, 5) = 64 (vs clamp core = 32).
    let req = VantaMemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("cat".into()),
        query_vector: vec![1.0, 0.0, 0.0],
        top_k: 5,
        search_profile: Some(SearchProfileConfig {
            candidate_k: Some(64),
            ..Default::default()
        }),
        ..Default::default()
    };
    let plan = db
        .debug_memory_search_plan_for_tests(req)
        .expect("debug plan should succeed");
    assert_eq!(plan.budget, 64, "candidate_k del perfil define el budget");

    // Sin profile: clamp core (5*4=20 => 32).
    let req_default = VantaMemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("cat".into()),
        query_vector: vec![1.0, 0.0, 0.0],
        top_k: 5,
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
        VantaMemoryMetadata::new(),
    );

    // explain + profile rrf_k=100 => el fusion report expone rrf_k=100 (D20).
    let req = VantaMemorySearchRequest {
        namespace: "test".into(),
        text_query: Some("cat".into()),
        query_vector: vec![1.0, 0.0, 0.0],
        top_k: 5,
        explain: true,
        search_profile: Some(SearchProfileConfig {
            rrf_k: Some(100),
            ..Default::default()
        }),
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
