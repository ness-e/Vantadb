#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Query Result Basic Snapshot Tests
//!
//! Part of the VantaDB insta snapshot testing suite (TBH-06).
//! Tests basic serialization of search requests and responses.

use insta::assert_debug_snapshot;
use vantadb::{
    DistanceMetric, VantaMemoryListPage, VantaMemoryMetadata, VantaMemoryRecord,
    VantaMemorySearchHit, VantaMemorySearchRequest, VantaSearchHit, VantaValue,
};

#[test]
fn search_request_basic_snapshot() {
    let req = VantaMemorySearchRequest {
        namespace: "agent/main".into(),
        query_vector: vec![0.1, 0.2, 0.3],
        query_sparse: None,
        filters: VantaMemoryMetadata::new(),
        text_query: Some("hello world".into()),
        top_k: 10,
        distance_metric: DistanceMetric::Cosine,
        explain: false,
        exclude_superseded: false,
        search_profile: None,
    };
    assert_debug_snapshot!("search_request_basic", req);
}

#[test]
fn search_request_vector_only_snapshot() {
    let req = VantaMemorySearchRequest {
        namespace: "docs".into(),
        query_vector: vec![0.5, 0.5, 0.5, 0.5],
        query_sparse: None,
        filters: VantaMemoryMetadata::new(),
        text_query: None,
        top_k: 5,
        distance_metric: DistanceMetric::Euclidean,
        explain: false,
        exclude_superseded: false,
        search_profile: None,
    };
    assert_debug_snapshot!("search_request_vector_only", req);
}

#[test]
fn search_request_text_only_snapshot() {
    let mut filters = VantaMemoryMetadata::new();
    filters.insert("category".into(), VantaValue::String("tech".into()));

    let req = VantaMemorySearchRequest {
        namespace: "knowledge".into(),
        query_vector: Vec::new(),
        query_sparse: None,
        filters,
        text_query: Some("rust async".into()),
        top_k: 20,
        distance_metric: DistanceMetric::Cosine,
        explain: true,
        exclude_superseded: false,
        search_profile: None,
    };
    assert_debug_snapshot!("search_request_text_only", req);
}

#[test]
fn search_hit_basic_snapshot() {
    let hit = VantaMemorySearchHit {
        record: VantaMemoryRecord {
            namespace: "agent/main".into(),
            key: "memory-1".into(),
            payload: "remember the contract".into(),
            metadata: VantaMemoryMetadata::new(),
            created_at_ms: 1000,
            updated_at_ms: 2000,
            version: 1,
            node_id: 42,
            vector: Some(vec![0.1, 0.2, 0.3]),
            sparse_vector: None,
            expires_at_ms: None,
            superseded_by: None,
            superseded_at_ms: None,
        },
        score: 0.95,
        explanation: None,
    };
    assert_debug_snapshot!("search_hit_basic", hit);
}

#[test]
fn search_hit_simple_snapshot() {
    let hit = VantaSearchHit {
        node_id: 12345,
        distance: 0.15,
    };
    assert_debug_snapshot!("search_hit_simple", hit);
}

#[test]
fn list_page_empty_snapshot() {
    let page = VantaMemoryListPage {
        records: vec![],
        next_cursor: None,
    };
    assert_debug_snapshot!("list_page_empty", page);
}

#[test]
fn list_page_with_records_snapshot() {
    let page = VantaMemoryListPage {
        records: vec![
            VantaMemoryRecord {
                namespace: "ns1".into(),
                key: "key1".into(),
                payload: "data1".into(),
                metadata: VantaMemoryMetadata::new(),
                created_at_ms: 100,
                updated_at_ms: 200,
                version: 1,
                node_id: 1,
                vector: None,
                sparse_vector: None,
                expires_at_ms: None,
                superseded_by: None,
                superseded_at_ms: None,
            },
            VantaMemoryRecord {
                namespace: "ns1".into(),
                key: "key2".into(),
                payload: "data2".into(),
                metadata: VantaMemoryMetadata::new(),
                created_at_ms: 300,
                updated_at_ms: 400,
                version: 1,
                node_id: 2,
                vector: None,
                sparse_vector: None,
                expires_at_ms: None,
                superseded_by: None,
                superseded_at_ms: None,
            },
        ],
        next_cursor: Some(2),
    };
    assert_debug_snapshot!("list_page_with_records", page);
}
