//! Vector-related SDK types: search requests, hits, and search results.

use super::super::types::{
    u128_serde, VantaMemoryMetadata, VantaMemoryRecord, VantaSearchExplanationHit,
};
use crate::node::DistanceMetric;
use serde::{Deserialize, Serialize};

/// Stable vector search request for persistent memory records.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VantaMemorySearchRequest {
    /// Namespace to restrict the search to.
    pub namespace: String,
    /// Query vector for similarity search. Empty means vector search is skipped.
    pub query_vector: Vec<f32>,
    /// Metadata key-value filters to narrow results.
    pub filters: VantaMemoryMetadata,
    /// Optional text query for BM25 lexical search.
    pub text_query: Option<String>,
    /// Maximum number of results to return.
    pub top_k: usize,
    /// Distance metric for vector similarity. Defaults to Cosine.
    pub distance_metric: DistanceMetric,
    /// When true, each result will carry a `VantaSearchExplanation`.
    pub explain: bool,
}

impl Default for VantaMemorySearchRequest {
    fn default() -> Self {
        Self {
            namespace: String::new(),
            query_vector: Vec::new(),
            filters: Default::default(),
            text_query: None,
            top_k: 10,
            distance_metric: DistanceMetric::Cosine,
            explain: false,
        }
    }
}

/// Stable vector search hit for external SDKs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VantaSearchHit {
    /// Numeric node identifier of the matched node.
    #[serde(with = "u128_serde")]
    pub node_id: u128,
    /// Distance from the query vector (lower is more similar for cosine/euclidean).
    pub distance: f32,
}

/// Stable vector search hit for persistent memory records.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VantaMemorySearchHit {
    /// The matched memory record.
    pub record: VantaMemoryRecord,
    /// Relevance score (BM25, cosine similarity, or RRF fused score).
    pub score: f32,
    /// Optional explanation for explain-mode searches.
    pub explanation: Option<VantaSearchExplanationHit>,
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use crate::sdk::types::VantaValue;

    #[test]
    fn test_search_request_default() {
        let req = VantaMemorySearchRequest::default();
        assert_eq!(req.namespace, "");
        assert!(req.query_vector.is_empty());
        assert!(req.filters.is_empty());
        assert!(req.text_query.is_none());
        assert_eq!(req.top_k, 10);
        assert_eq!(req.distance_metric, DistanceMetric::Cosine);
        assert!(!req.explain);
    }

    #[test]
    fn test_search_request_custom() {
        let mut filters = VantaMemoryMetadata::new();
        filters.insert("type".into(), VantaValue::String("doc".into()));
        let req = VantaMemorySearchRequest {
            namespace: "test".into(),
            query_vector: vec![0.1, 0.2, 0.3],
            filters,
            text_query: Some("hello".into()),
            top_k: 5,
            distance_metric: DistanceMetric::Euclidean,
            explain: true,
        };
        assert_eq!(req.namespace, "test");
        assert_eq!(req.query_vector.len(), 3);
        assert_eq!(req.top_k, 5);
        assert_eq!(req.distance_metric, DistanceMetric::Euclidean);
        assert!(req.explain);
    }

    #[test]
    fn test_search_request_serialization_roundtrip() {
        let req = VantaMemorySearchRequest {
            namespace: "ns".into(),
            query_vector: vec![0.5, 0.5],
            filters: VantaMemoryMetadata::new(),
            text_query: Some("query".into()),
            top_k: 20,
            distance_metric: DistanceMetric::Cosine,
            explain: false,
        };
        let json = serde_json::to_string(&req).unwrap();
        let deserialized: VantaMemorySearchRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, req);
    }

    #[test]
    fn test_search_hit_serialization_roundtrip() {
        let hit = VantaSearchHit {
            node_id: 12345,
            distance: 0.42,
        };
        let json = serde_json::to_string(&hit).unwrap();
        let deserialized: VantaSearchHit = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, hit);
    }

    #[test]
    fn test_memory_search_hit_serialization_roundtrip() {
        let hit = VantaMemorySearchHit {
            record: VantaMemoryRecord {
                namespace: "ns".into(),
                key: "k".into(),
                payload: "payload".into(),
                metadata: VantaMemoryMetadata::new(),
                created_at_ms: 100,
                updated_at_ms: 200,
                version: 1,
                node_id: 42,
                vector: None,
                expires_at_ms: None,
            },
            score: 0.95,
            explanation: None,
        };
        let json = serde_json::to_string(&hit).unwrap();
        let deserialized: VantaMemorySearchHit = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, hit);
    }

    #[test]
    fn test_search_hit_node_id_serialized_as_string() {
        let hit = VantaSearchHit {
            node_id: 999888777666,
            distance: 0.1,
        };
        let json = serde_json::to_string(&hit).unwrap();
        assert!(json.contains("\"999888777666\""));
    }
}
