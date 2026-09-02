#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Query Result Advanced Snapshot Tests
//!
//! Part of the VantaDB insta snapshot testing suite (TBH-06).
//! Tests advanced serialization: search profiles, exclude_superseded, sparse vectors, pagination.

use insta::assert_debug_snapshot;
use std::collections::BTreeMap;
use vantadb::sdk::{SearchProfileConfig, SearchProfileMode};
use vantadb::{
    DistanceMetric, SparseVector, VantaBm25TermContribution, VantaMemoryListPage,
    VantaMemoryMetadata, VantaMemoryRecord, VantaMemorySearchHit, VantaMemorySearchRequest,
    VantaNodeRecord, VantaQueryResult, VantaSearchExplanationHit, VantaStorageTier, VantaValue,
};

#[test]
fn search_request_with_profile_hybrid_snapshot() {
    let req = VantaMemorySearchRequest {
        namespace: "agent/main".into(),
        query_vector: vec![0.1, 0.2, 0.3],
        query_sparse: None,
        filters: VantaMemoryMetadata::new(),
        text_query: Some("rust async patterns".into()),
        top_k: 10,
        distance_metric: DistanceMetric::Cosine,
        explain: false,
        exclude_superseded: false,
        search_profile: Some(SearchProfileConfig {
            mode: SearchProfileMode::Hybrid,
            rrf_k: Some(60),
            candidate_k: Some(100),
        }),
    };
    assert_debug_snapshot!("search_request_with_profile_hybrid", req);
}

#[test]
fn search_request_with_profile_keyword_snapshot() {
    let req = VantaMemorySearchRequest {
        namespace: "docs".into(),
        query_vector: Vec::new(),
        query_sparse: None,
        filters: VantaMemoryMetadata::new(),
        text_query: Some("api reference".into()),
        top_k: 15,
        distance_metric: DistanceMetric::Cosine,
        explain: true,
        exclude_superseded: true,
        search_profile: Some(SearchProfileConfig {
            mode: SearchProfileMode::Keyword,
            rrf_k: None,
            candidate_k: None,
        }),
    };
    assert_debug_snapshot!("search_request_with_profile_keyword", req);
}

#[test]
fn search_request_with_profile_vector_snapshot() {
    let req = VantaMemorySearchRequest {
        namespace: "embeddings".into(),
        query_vector: vec![0.7; 1536],
        query_sparse: None,
        filters: VantaMemoryMetadata::new(),
        text_query: None,
        top_k: 50,
        distance_metric: DistanceMetric::Cosine,
        explain: false,
        exclude_superseded: false,
        search_profile: Some(SearchProfileConfig {
            mode: SearchProfileMode::Vector,
            rrf_k: None,
            candidate_k: Some(200),
        }),
    };
    assert_debug_snapshot!("search_request_with_profile_vector", req);
}

#[test]
fn search_request_exclude_superseded_snapshot() {
    let mut filters = VantaMemoryMetadata::new();
    filters.insert("type".into(), VantaValue::String("note".into()));

    let req = VantaMemorySearchRequest {
        namespace: "notes".into(),
        query_vector: vec![0.3, 0.4],
        query_sparse: None,
        filters,
        text_query: Some("meeting".into()),
        top_k: 25,
        distance_metric: DistanceMetric::Cosine,
        explain: false,
        exclude_superseded: true,
        search_profile: None,
    };
    assert_debug_snapshot!("search_request_exclude_superseded", req);
}

#[test]
fn search_request_sparse_vector_snapshot() {
    let mut sparse = SparseVector::new();
    sparse.insert(10, 1.5);
    sparse.insert(42, 2.0);
    sparse.insert(100, 0.8);

    let req = VantaMemorySearchRequest {
        namespace: "sparse_idx".into(),
        query_vector: Vec::new(),
        query_sparse: Some(sparse),
        filters: VantaMemoryMetadata::new(),
        text_query: None,
        top_k: 30,
        distance_metric: DistanceMetric::Cosine,
        explain: false,
        exclude_superseded: false,
        search_profile: None,
    };
    assert_debug_snapshot!("search_request_sparse_vector", req);
}

#[test]
fn search_request_full_complex_snapshot() {
    let mut filters = VantaMemoryMetadata::new();
    filters.insert("category".into(), VantaValue::String("technical".into()));
    filters.insert("priority".into(), VantaValue::Int(1));

    let mut sparse = SparseVector::new();
    sparse.insert(5, 1.0);
    sparse.insert(15, 0.5);

    let req = VantaMemorySearchRequest {
        namespace: "production".into(),
        query_vector: vec![0.1; 768],
        query_sparse: Some(sparse),
        filters,
        text_query: Some("critical bug fix".into()),
        top_k: 100,
        distance_metric: DistanceMetric::Euclidean,
        explain: true,
        exclude_superseded: true,
        search_profile: Some(SearchProfileConfig {
            mode: SearchProfileMode::Hybrid,
            rrf_k: Some(100),
            candidate_k: Some(500),
        }),
    };
    assert_debug_snapshot!("search_request_full_complex", req);
}

#[test]
fn search_hit_with_explanation_snapshot() {
    let hit = VantaMemorySearchHit {
        record: VantaMemoryRecord {
            namespace: "agent/main".into(),
            key: "memory-42".into(),
            payload: "critical production bug fix".into(),
            metadata: {
                let mut meta = VantaMemoryMetadata::new();
                meta.insert("severity".into(), VantaValue::String("critical".into()));
                meta.insert("component".into(), VantaValue::String("auth".into()));
                meta
            },
            created_at_ms: 1_700_000_000_000,
            updated_at_ms: 1_700_000_010_000,
            version: 3,
            node_id: 999,
            vector: Some(vec![0.1; 768]),
            sparse_vector: None,
            expires_at_ms: None,
            superseded_by: None,
            superseded_at_ms: None,
        },
        score: 0.987,
        explanation: Some(VantaSearchExplanationHit {
            identity: "agent/main\0memory-42".into(),
            score: 0.987,
            snippet: Some("critical production bug fix".into()),
            matched_tokens: vec!["critical".into(), "bug".into(), "fix".into()],
            matched_phrases: vec!["critical bug fix".into()],
            bm25_terms: vec![
                VantaBm25TermContribution {
                    token: "critical".into(),
                    tf: 2,
                    df: 5,
                    doc_len: 100,
                    contribution: 3.5,
                },
                VantaBm25TermContribution {
                    token: "bug".into(),
                    tf: 1,
                    df: 10,
                    doc_len: 100,
                    contribution: 2.1,
                },
            ],
            rrf_text_rank: Some(1),
            rrf_vector_rank: Some(3),
        }),
    };
    assert_debug_snapshot!("search_hit_with_explanation", hit);
}

#[test]
fn search_hit_superseded_chain_snapshot() {
    let hit = VantaMemorySearchHit {
        record: VantaMemoryRecord {
            namespace: "versioned".into(),
            key: "doc_v3".into(),
            payload: "latest version of the document".into(),
            metadata: VantaMemoryMetadata::new(),
            created_at_ms: 1_700_000_020_000,
            updated_at_ms: 1_700_000_025_000,
            version: 3,
            node_id: 1003,
            vector: Some(vec![0.2; 512]),
            sparse_vector: None,
            expires_at_ms: None,
            superseded_by: Some("1004".into()),
            superseded_at_ms: Some(1_700_000_030_000),
        },
        score: 0.85,
        explanation: None,
    };
    assert_debug_snapshot!("search_hit_superseded_chain", hit);
}

#[test]
fn list_page_multi_page_snapshot() {
    let page = VantaMemoryListPage {
        records: (1..=50)
            .map(|i| VantaMemoryRecord {
                namespace: "large_ns".into(),
                key: format!("key_{:04}", i),
                payload: format!("record number {}", i),
                metadata: VantaMemoryMetadata::new(),
                created_at_ms: 1000 * i as u64,
                updated_at_ms: 1000 * i as u64,
                version: 1,
                node_id: i as u128,
                vector: None,
                sparse_vector: None,
                expires_at_ms: None,
                superseded_by: None,
                superseded_at_ms: None,
            })
            .collect(),
        next_cursor: Some(50),
    };
    assert_debug_snapshot!("list_page_multi_page", page);
}

#[test]
fn list_page_last_page_snapshot() {
    let page = VantaMemoryListPage {
        records: vec![VantaMemoryRecord {
            namespace: "small_ns".into(),
            key: "final_key".into(),
            payload: "last record".into(),
            metadata: VantaMemoryMetadata::new(),
            created_at_ms: 999999,
            updated_at_ms: 999999,
            version: 1,
            node_id: 999,
            vector: None,
            sparse_vector: None,
            expires_at_ms: None,
            superseded_by: None,
            superseded_at_ms: None,
        }],
        next_cursor: None,
    };
    assert_debug_snapshot!("list_page_last_page", page);
}

#[test]
fn query_result_read_variant_snapshot() {
    let result = VantaQueryResult::Read(vec![VantaNodeRecord {
        id: 1,
        fields: {
            let mut f = BTreeMap::new();
            f.insert("name".into(), VantaValue::String("test".into()));
            f
        },
        vector: None,
        vector_dimensions: 0,
        edges: vec![],
        confidence_score: 0.9,
        importance: 0.5,
        hits: 10,
        last_accessed: 1000,
        epoch: 0,
        tier: VantaStorageTier::Hot,
        is_alive: true,
    }]);
    assert_debug_snapshot!("query_result_read_variant", result);
}

#[test]
fn query_result_write_variant_snapshot() {
    let result = VantaQueryResult::Write {
        affected_nodes: 5,
        message: "batch insert completed".into(),
        node_id: Some(1005),
    };
    assert_debug_snapshot!("query_result_write_variant", result);
}

#[test]
fn query_result_stale_context_variant_snapshot() {
    let result = VantaQueryResult::StaleContext { node_id: 9999 };
    assert_debug_snapshot!("query_result_stale_context_variant", result);
}
