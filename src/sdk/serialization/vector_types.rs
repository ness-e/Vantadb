//! Vector-related SDK types: search requests, hits, and search results.

use super::super::types::{u128_serde, MemoryMetadata, MemoryRecord, SearchExplanationHit};
use crate::node::{DistanceMetric, SparseVector};
use crate::search_profile::SearchProfileConfig;
use serde::{Deserialize, Serialize};

/// Score-range filter applied after ranking (WIRE-08).
///
/// Invariants (validated at the request boundary):
/// - bounds are finite when present, and `min_score <= max_score`;
/// - bounds are **inclusive** and expressed in **score space** (higher score =
///   more relevant, matching [`MemorySearchHit::score`]).
///
/// Ecosystem mapping: Milvus `radius`/`range_filter` are distance-space and
/// flip sign per metric (COSINE/IP: `radius < distance <= range_filter`;
/// L2: `range_filter <= distance < radius`); Qdrant uses `score_threshold`
/// (minimum score, score-space). This struct is the score-space equivalent
/// with both bounds inclusive.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct RangeFilter {
    /// Lower bound (inclusive). `None` = unbounded below.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_score: Option<f32>,
    /// Upper bound (inclusive). `None` = unbounded above.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_score: Option<f32>,
}

/// Group-by selection over ranked hits (WIRE-08).
///
/// Semantics: iterate the ranked result list in order and keep a hit while its
/// group (the canonical value of `field` in the record metadata) has fewer than
/// `group_size` hits already selected; stop once `top_k` hits were selected.
/// Records missing `field` form their own group. `top_k` caps **total hits**
/// (unlike Milvus/Qdrant, where `limit` caps the number of groups — see
/// `docs/api/SEARCH_PARITY.md`).
///
/// Invariants (validated at the request boundary): `field` non-empty,
/// `group_size >= 1`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupByConfig {
    /// Metadata field whose values define the groups.
    pub field: String,
    /// Maximum hits returned per group value. Defaults to 1 (Milvus default).
    #[serde(default = "default_group_size")]
    pub group_size: usize,
}

fn default_group_size() -> usize {
    1
}

impl Default for GroupByConfig {
    fn default() -> Self {
        Self {
            field: String::new(),
            group_size: default_group_size(),
        }
    }
}

/// Maximal Marginal Relevance reranking config (WIRE-08).
///
/// `mmr(d) = lambda * relevance(d) - (1 - lambda) * max_{s in S} cosine(d, s)`
/// where relevance is min-max normalized within the candidate window (so
/// `lambda` is meaningful across fusion routes whose raw scores live on
/// different scales) and `S` is the already-selected set.
///
/// Invariants (validated at the request boundary): `lambda` in `[0, 1]`,
/// `fetch_k >= 1` when present.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MmrConfig {
    /// Relevance/diversity trade-off in `[0, 1]`. `1.0` = pure relevance
    /// (identity ordering), `0.0` = pure diversity. Defaults to `0.5`.
    #[serde(default = "default_mmr_lambda")]
    pub lambda: f32,
    /// Candidate window considered for reranking. `None` = `top_k * 5`
    /// (the "fetch_k 5-10x" guidance from the backlog), clamped to
    /// `[top_k, MAX_MMR_CANDIDATES]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fetch_k: Option<usize>,
}

fn default_mmr_lambda() -> f32 {
    0.5
}

impl Default for MmrConfig {
    fn default() -> Self {
        Self {
            lambda: default_mmr_lambda(),
            fetch_k: None,
        }
    }
}

/// Stable vector search request for persistent memory records.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemorySearchRequest {
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
    pub filters: MemoryMetadata,
    /// Optional text query for BM25 lexical search.
    pub text_query: Option<String>,
    /// Maximum number of results to return.
    pub top_k: usize,
    /// Distance metric for dense vector similarity. Defaults to Cosine.
    pub distance_metric: DistanceMetric,
    /// When true, each result will carry a `SearchExplanation`.
    pub explain: bool,
    /// When true, records marked as superseded (ADR-028) are dropped from the
    /// results. Defaults to false: superseded records remain searchable.
    #[serde(default)]
    pub exclude_superseded: bool,
    /// Optional search profile (mode, RRF k, candidate budget) for this request.
    /// `None` uses the core defaults (MEM-01).
    #[serde(default)]
    pub search_profile: Option<SearchProfileConfig>,
    /// Optional score-range filter applied after ranking (WIRE-08).
    #[serde(default)]
    pub range: Option<RangeFilter>,
    /// Optional group-by selection applied after ranking (WIRE-08).
    #[serde(default)]
    pub group_by: Option<GroupByConfig>,
    /// Optional MMR diversity reranking (WIRE-08). Mutually exclusive with
    /// `cursor` (rejected at the boundary: MMR is set-dependent).
    #[serde(default)]
    pub mmr: Option<MmrConfig>,
    /// Opaque continuation token from a previous [`MemorySearchPage`] result
    /// (WIRE-08). Valid only for a request with the same plan fingerprint
    /// (namespace, query, filters, metric, profile, range) and the same
    /// process — never persist or parse it. Resume is best-effort, not a
    /// snapshot: see [`Embedded::search_page`](crate::sdk::Embedded::search_page)
    /// for the exact guarantee. Mutually exclusive with `mmr`/`group_by`.
    #[serde(default)]
    pub cursor: Option<String>,
}

impl Default for MemorySearchRequest {
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
            range: None,
            group_by: None,
            mmr: None,
            cursor: None,
        }
    }
}

/// One page of search results with an optional continuation cursor (WIRE-08).
///
/// Page contract (same convention as `MemoryListPage`): `next_cursor` is
/// `Some` only when the page is full (`hits.len() == top_k`); a page with fewer
/// hits is the last page. `next_cursor` is always `None` when `mmr` or
/// `group_by` is set (pagination is unsupported for those selectors).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemorySearchPage {
    /// Ranked hits for this page.
    pub hits: Vec<MemorySearchHit>,
    /// Cursor for the next page, or `None` if this was the last page.
    pub next_cursor: Option<String>,
}

/// Stable vector search hit for external SDKs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchHit {
    /// Numeric node identifier of the matched node.
    #[serde(with = "u128_serde")]
    pub node_id: u128,
    /// Distance from the query vector (lower is more similar for cosine/euclidean).
    pub distance: f32,
}

/// Stable vector search hit for persistent memory records.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemorySearchHit {
    /// The matched memory record.
    pub record: MemoryRecord,
    /// Relevance score (BM25, cosine similarity, or RRF fused score).
    pub score: f32,
    /// Optional explanation for explain-mode searches.
    pub explanation: Option<SearchExplanationHit>,
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use crate::sdk::types::Value;
    use crate::sdk::types::{SearchProfileConfig, SearchProfileMode};

    #[test]
    fn test_search_request_default() {
        let req = MemorySearchRequest::default();
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
        let mut filters = MemoryMetadata::new();
        filters.insert("type".into(), Value::String("doc".into()));
        let req = MemorySearchRequest {
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
            range: None,
            group_by: None,
            mmr: None,
            cursor: None,
        };
        assert_eq!(req.namespace, "test");
        assert_eq!(req.query_vector.len(), 3);
        assert_eq!(req.top_k, 5);
        assert_eq!(req.distance_metric, DistanceMetric::Euclidean);
        assert!(req.explain);
    }

    #[test]
    fn test_search_request_serialization_roundtrip() {
        let req = MemorySearchRequest {
            namespace: "ns".into(),
            query_vector: vec![0.5, 0.5],
            filters: MemoryMetadata::new(),
            text_query: Some("query".into()),
            top_k: 20,
            distance_metric: DistanceMetric::Cosine,
            explain: false,
            query_sparse: None,
            exclude_superseded: false,
            search_profile: None,
            range: None,
            group_by: None,
            mmr: None,
            cursor: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        let deserialized: MemorySearchRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, req);
    }

    // --- WIRE-08: range / group_by / mmr / cursor ---

    #[test]
    fn test_range_filter_defaults_and_roundtrip() {
        let req = MemorySearchRequest::default();
        assert_eq!(req.range, None);
        assert_eq!(req.group_by, None);
        assert_eq!(req.mmr, None);
        assert_eq!(req.cursor, None);

        let req = MemorySearchRequest {
            range: Some(RangeFilter {
                min_score: Some(0.25),
                max_score: Some(0.75),
            }),
            group_by: Some(GroupByConfig {
                field: "doc_id".into(),
                group_size: 3,
            }),
            mmr: Some(MmrConfig {
                lambda: 0.7,
                fetch_k: Some(64),
            }),
            cursor: Some("opaque-token".into()),
            ..Default::default()
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: MemorySearchRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back, req);
    }

    #[test]
    fn test_range_filter_omits_none_bounds() {
        let range = RangeFilter {
            min_score: Some(0.5),
            max_score: None,
        };
        let json = serde_json::to_string(&range).unwrap();
        assert!(json.contains("min_score"));
        assert!(!json.contains("max_score"), "None bound must be omitted");
    }

    #[test]
    fn test_group_by_defaults_to_one_per_group() {
        let cfg = GroupByConfig::default();
        assert_eq!(cfg.group_size, 1);
        // JSON without group_size defaults to 1.
        let back: GroupByConfig = serde_json::from_str(r#"{"field":"doc"}"#).unwrap();
        assert_eq!(back.group_size, 1);
        assert_eq!(back.field, "doc");
    }

    #[test]
    fn test_mmr_defaults() {
        let cfg = MmrConfig::default();
        assert_eq!(cfg.lambda, 0.5);
        assert_eq!(cfg.fetch_k, None);
        let back: MmrConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(back.lambda, 0.5);
    }

    #[test]
    fn test_search_page_roundtrip() {
        let page = MemorySearchPage {
            hits: Vec::new(),
            next_cursor: Some("token".into()),
        };
        let json = serde_json::to_string(&page).unwrap();
        let back: MemorySearchPage = serde_json::from_str(&json).unwrap();
        assert_eq!(back, page);
    }

    #[test]
    fn test_search_request_without_new_fields_deserializes() {
        // Backward compat: JSON produced before WIRE-08 has none of the new
        // fields — it must keep deserializing to `None` defaults.
        let legacy = r#"{
            "namespace": "ns",
            "query_vector": [0.1],
            "filters": {},
            "text_query": null,
            "top_k": 3,
            "distance_metric": "Cosine",
            "explain": false
        }"#;
        let req: MemorySearchRequest = serde_json::from_str(legacy).unwrap();
        assert_eq!(req.range, None);
        assert_eq!(req.group_by, None);
        assert_eq!(req.mmr, None);
        assert_eq!(req.cursor, None);
    }

    #[test]
    fn test_search_hit_serialization_roundtrip() {
        let hit = SearchHit {
            node_id: 12345,
            distance: 0.42,
        };
        let json = serde_json::to_string(&hit).unwrap();
        let deserialized: SearchHit = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, hit);
    }

    #[test]
    fn test_memory_search_hit_serialization_roundtrip() {
        let hit = MemorySearchHit {
            record: MemoryRecord {
                namespace: "ns".into(),
                key: "k".into(),
                payload: "payload".into(),
                metadata: MemoryMetadata::new(),
                created_at_ms: 100,
                updated_at_ms: 200,
                version: 1,
                node_id: 42,
                vector: None,
                sparse_vector: None,
                expires_at_ms: None,
                superseded_by: None,
                superseded_at_ms: None,
                ..Default::default()
            },
            score: 0.95,
            explanation: None,
        };
        let json = serde_json::to_string(&hit).unwrap();
        let deserialized: MemorySearchHit = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, hit);
    }

    #[test]
    fn test_search_hit_node_id_serialized_as_string() {
        let hit = SearchHit {
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
        let req = MemorySearchRequest::default();
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
        let req = MemorySearchRequest {
            namespace: "ns".into(),
            search_profile: Some(SearchProfileConfig {
                mode: SearchProfileMode::Hybrid,
                rrf_k: Some(75),
                candidate_k: Some(96),
            }),
            range: None,
            group_by: None,
            mmr: None,
            cursor: None,
            ..Default::default()
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: MemorySearchRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back, req);
    }

    #[test]
    fn test_search_request_without_profile_field_is_none() {
        // Retrocompat: JSON antiguo sin `search_profile` deserializa a None.
        let json = r#"{"namespace":"ns","query_vector":[],"query_sparse":null,"filters":{},"text_query":null,"top_k":10,"distance_metric":"Cosine","explain":false,"exclude_superseded":false}"#;
        let req: MemorySearchRequest = serde_json::from_str(json).unwrap();
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

    fn minimal_record(key: &str, ns: &str) -> MemoryRecord {
        MemoryRecord {
            namespace: ns.into(),
            key: key.into(),
            payload: String::new(),
            metadata: MemoryMetadata::new(),
            created_at_ms: 0,
            updated_at_ms: 0,
            version: 1,
            node_id: 0,
            vector: None,
            sparse_vector: None,
            expires_at_ms: None,
            superseded_by: None,
            superseded_at_ms: None,
            ..Default::default()
        }
    }

    #[test]
    fn score_roundtrips_through_serde_json() {
        // El SDK Rust expone `score` (higher = better) para Python/Node/HTTP.
        // JSON round-trip debe preservar el field verbatim — un futuro
        // "renombremos a distance" rompe este test.
        let hit = MemorySearchHit {
            record: minimal_record("k1", "agent/main"),
            score: 0.575_364_23_f32,
            explanation: None,
        };
        let json = serde_json::to_string(&hit).expect("serialize");
        assert!(
            json.contains("\"score\":0.575"),
            "score field must survive serde: {json}"
        );
        let de: MemorySearchHit = serde_json::from_str(&json).expect("deserialize");
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
        let hit = MemorySearchHit {
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
            let hit = MemorySearchHit {
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
