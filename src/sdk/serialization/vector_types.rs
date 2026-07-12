//! Vector-related SDK types: search requests, hits, and search results.

use super::super::types::{VantaMemoryMetadata, VantaMemoryRecord, VantaSearchExplanationHit};
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
