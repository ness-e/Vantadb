//! Vector-related SDK types: search requests, hits, and search results.

use super::super::types::{
    u128_serde, SearchProfileConfig, VantaMemoryMetadata, VantaMemoryRecord,
    VantaSearchExplanationHit,
};
use crate::node::{DistanceMetric, SparseVector};
use serde::{Deserialize, Serialize};

/// Stable vector search request for persistent memory records.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VantaMemorySearchRequest {
    /// Namespace to restrict the search to.
    pub namespace: String,
    /// Query vector for similarity search. Empty means dense vector search is skipped.
    pub query_vector: Vec<f32>,
    /// Optional sparse query vector for sparse-dot similarity. `None` skips
    /// sparse search. Sparse search runs a brute-force dot over the matching
    /// namespace and is fused with any dense/text scores.
    #[serde(default)]
    pub query_sparse: Option<SparseVector>,
    /// Metadata key-value filters to narrow results.
    pub filters: VantaMemoryMetadata,
    /// Optional text query for BM25 lexical search.
    pub text_query: Option<String>,
    /// Maximum number of results to return.
    pub top_k: usize,
    /// Distance metric for dense vector similarity. Defaults to Cosine.
    pub distance_metric: DistanceMetric,
    /// When true, each result will carry a `VantaSearchExplanation`.
    pub explain: bool,
    /// When true, records marked as superseded (ADR-028) are dropped from the
    /// results. Defaults to false: superseded records remain searchable.
    #[serde(default)]
    pub exclude_superseded: bool,
    /// Optional search profile (mode, RRF k, candidate budget) for this request.
    /// `None` uses the core defaults (MEM-01).
    #[serde(default)]
    pub search_profile: Option<SearchProfileConfig>,
}

impl Default for VantaMemorySearchRequest {
    fn default() -> Self {
        Self {
            namespace: String::new(),
            query_vector: Vec::new(),
            query_sparse: None,
            filters: Default::default(),
            text_query: None,
            top_k: 10,
            distance_metric: DistanceMetric::Cosine,
            explain: false,
            exclude_superseded: false,
            search_profile: None,
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
    use crate::sdk::types::{SearchProfileConfig, SearchProfileMode};

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
            query_sparse: None,
            exclude_superseded: false,
            search_profile: None,
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
            query_sparse: None,
            exclude_superseded: false,
            search_profile: None,
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
                sparse_vector: None,
                expires_at_ms: None,
                superseded_by: None,
                superseded_at_ms: None,
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

    // --- SearchProfileConfig (MEM-01) ---

    #[test]
    fn test_search_profile_defaults() {
        let p = SearchProfileConfig::default();
        assert_eq!(p.mode, SearchProfileMode::Hybrid);
        assert_eq!(p.rrf_k, None);
        assert_eq!(p.candidate_k, None);
        let req = VantaMemorySearchRequest::default();
        assert_eq!(req.search_profile, None);
    }

    #[test]
    fn test_search_profile_serialization_roundtrip() {
        let p = SearchProfileConfig {
            mode: SearchProfileMode::Keyword,
            rrf_k: Some(100),
            candidate_k: Some(128),
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: SearchProfileConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back, p);
    }

    #[test]
    fn test_search_profile_serde_lowercase() {
        let json = r#"{"mode":"keyword","rrf_k":100,"candidate_k":128}"#;
        let p: SearchProfileConfig = serde_json::from_str(json).unwrap();
        assert_eq!(p.mode, SearchProfileMode::Keyword);
        assert_eq!(p.rrf_k, Some(100));
        assert_eq!(p.candidate_k, Some(128));
        assert!(serde_json::to_string(&p).unwrap().contains("\"keyword\""));
    }

    #[test]
    fn test_search_profile_partial_json_defaults() {
        // Clientes que solo mandan el modo: los demás campos caen a None.
        let json = r#"{"mode":"vector"}"#;
        let p: SearchProfileConfig = serde_json::from_str(json).unwrap();
        assert_eq!(p.mode, SearchProfileMode::Vector);
        assert_eq!(p.rrf_k, None);
        assert_eq!(p.candidate_k, None);
    }

    #[test]
    fn test_search_request_with_profile_roundtrip() {
        let req = VantaMemorySearchRequest {
            namespace: "ns".into(),
            search_profile: Some(SearchProfileConfig {
                mode: SearchProfileMode::Hybrid,
                rrf_k: Some(75),
                candidate_k: Some(96),
            }),
            ..Default::default()
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: VantaMemorySearchRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back, req);
    }

    #[test]
    fn test_search_request_without_profile_field_is_none() {
        // Retrocompat: JSON antiguo sin `search_profile` deserializa a None.
        let json = r#"{"namespace":"ns","query_vector":[],"query_sparse":null,"filters":{},"text_query":null,"top_k":10,"distance_metric":"Cosine","explain":false,"exclude_superseded":false}"#;
        let req: VantaMemorySearchRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.search_profile, None);
    }

    // ── TS-03: Score/distance semantics pinning ─────────────────────────────
    //
    // Drift histórico entre bindings (ver docs/api/TS_SDK.md CODE-091):
    //
    // | SDK        | Campo expuesto | Convención            |
    // |------------|----------------|-----------------------|
    // | Rust core  | `score`        | higher = better       |
    // | TS SDK     | `distance`     | lower = better        |
    // | Python SDK | `score`        | higher = better       |
    // | Node SDK   | `score`        | higher = better       |
    // | HTTP API   | `score`        | higher = better       |
    //
    // Estos tests fijan los invariantes del score del core para que cualquier
    // cambio futuro (drift zero-norm cosine, redondeo FP, o swap de signo) se
    // detecte en CI. Sources canónicos:
    //   - src/sdk/api.rs:1661        — score: 1.0 - hit.distance (cosine)
    //   - src/sdk/search/vector.rs:30-60 — score formula por DistanceMetric

    fn minimal_record(key: &str, ns: &str) -> VantaMemoryRecord {
        VantaMemoryRecord {
            namespace: ns.into(),
            key: key.into(),
            payload: String::new(),
            metadata: VantaMemoryMetadata::new(),
            created_at_ms: 0,
            updated_at_ms: 0,
            version: 1,
            node_id: 0,
            vector: None,
            sparse_vector: None,
            expires_at_ms: None,
            superseded_by: None,
            superseded_at_ms: None,
        }
    }

    #[test]
    fn score_roundtrips_through_serde_json() {
        // El SDK Rust expone `score` (higher = better) para Python/Node/HTTP.
        // JSON round-trip debe preservar el field verbatim — un futuro
        // "renombremos a distance" rompe este test.
        let hit = VantaMemorySearchHit {
            record: minimal_record("k1", "agent/main"),
            score: 0.575_364_23_f32,
            explanation: None,
        };
        let json = serde_json::to_string(&hit).expect("serialize");
        assert!(
            json.contains("\"score\":0.575"),
            "score field must survive serde: {json}"
        );
        let de: VantaMemorySearchHit = serde_json::from_str(&json).expect("deserialize");
        assert!(
            (de.score - 0.575_364_23).abs() < 1e-6,
            "score round-trip drifted: {}",
            de.score
        );
    }

    #[test]
    fn euclidean_score_supports_negative_values() {
        // Per src/sdk/search/vector.rs:32, Euclidean score = -||a-b||² (negative).
        // Pin this bound so the contract isn't accidentally re-flipped.
        let hit = VantaMemorySearchHit {
            record: minimal_record("k1", "ns"),
            score: -4.0_f32,
            explanation: None,
        };
        assert!(
            hit.score <= 0.0,
            "Euclidean-derived score must be ≤ 0, got {}",
            hit.score
        );
    }

    #[test]
    fn cosine_score_range_matches_documented_contract() {
        // Documented invariant: cosine score ∈ [-1.0, 1.0]. Anything outside
        // indicates a broken normalization step (regression of zero-norm
        // cosine guard).
        for &score in &[-1.0_f32, -0.5, 0.0, 0.5, 1.0] {
            let hit = VantaMemorySearchHit {
                record: minimal_record("k1", "ns"),
                score,
                explanation: None,
            };
            assert!(
                (-1.0..=1.0).contains(&hit.score),
                "cosine score must be in [-1, 1], got {}",
                hit.score
            );
        }
    }

    #[test]
    fn cosine_sim_f32_identical_returns_one() {
        // Pin de la primitiva que alimenta vector_memory_search.
        let v = vec![0.3_f32, 0.4, 0.5, 0.1, 0.7];
        let sim = crate::index::distance::cosine_sim_f32(&v, &v);
        assert!(
            (sim - 1.0).abs() < 1e-5,
            "identical vectors must yield cos≈1.0, got {sim}"
        );
    }

    #[test]
    fn cosine_sim_f32_zero_norm_returns_finite_zero() {
        // TS-03 anti-drift guard: zero-norm vectors used to yield NaN before
        // the cosine_sim_with_query_norm guard. Pinning prevents regression.
        let zero = vec![0.0_f32; 8];
        let some = vec![0.1_f32, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];
        let sim = crate::index::distance::cosine_sim_f32(&zero, &some);
        assert!(
            sim.is_finite(),
            "zero-norm vector must yield finite score, got {sim}"
        );
        assert!(
            sim.abs() < 1e-5,
            "zero-norm vs non-zero must score≈0.0, got {sim}"
        );
    }

    #[test]
    fn euclidean_squared_distance_never_negative_under_fp_rounding() {
        // AUDREP-28 regression guard: ||a||² + ||b||² - 2·a·b can dip slightly
        // below zero from FP rounding; public dispatch must always return ≥ 0.
        let v = vec![1.0_f32; 128];
        let d_sq = crate::index::distance::euclidean_distance_squared_f32(&v, &v);
        assert!(
            d_sq >= 0.0,
            "d² for identical vectors must be ≥ 0, got {d_sq}"
        );
    }
}
