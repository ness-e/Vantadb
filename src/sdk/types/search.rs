//! Search-domain SDK types: profiles, explanations, BM25 details, index state/reports.
//!
//! Pure move of the search items from `super` (FIND-49). Public paths
//! `crate::sdk::types::X` are preserved via re-exports in `super`.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub use super::super::serialization::vector_types::{
    MemorySearchHit, MemorySearchRequest, SearchHit,
};
/// Stable report returned by manual ANN rebuild through the SDK boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexRebuildReport {
    /// Number of nodes scanned during the rebuild.
    pub scanned_nodes: u64,
    /// Number of vectors indexed into HNSW.
    pub indexed_vectors: u64,
    /// Number of tombstoned (deleted) nodes skipped.
    pub skipped_tombstones: u64,
    /// Duration of the rebuild in milliseconds.
    pub duration_ms: u64,
    /// Duration of the derived index rebuild in milliseconds.
    pub derived_rebuild_ms: u64,
    /// Filesystem path to the rebuilt index file.
    pub index_path: String,
    /// Whether the rebuild completed successfully.
    pub success: bool,
}

/// Stable report returned by text index repair operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextIndexRepairReport {
    /// Number of memory records indexed.
    pub record_count: u64,
    /// Number of posting list entries written.
    pub posting_entries: u64,
    /// Number of document stats entries written.
    pub doc_stats_entries: u64,
    /// Number of term stats entries written.
    pub term_stats_entries: u64,
    /// Number of namespace stats entries written.
    pub namespace_stats_entries: u64,
    /// Duration of the repair in milliseconds.
    pub duration_ms: u64,
    /// Whether the repair completed successfully.
    pub success: bool,
}

#[cfg(debug_assertions)]
#[derive(Debug, Clone, PartialEq, Eq)]
#[doc(hidden)]
pub struct MemorySearchDebugReport {
    pub route: String,
    pub budget: usize,
    pub text_candidates: usize,
    pub vector_candidates: usize,
    pub fused_candidates: usize,
    pub top_identities: Vec<String>,
}

/// Modo de fusión híbrida para un [`SearchProfileConfig`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SearchProfileMode {
    /// Solo búsqueda BM25 lexical: ignora el vector denso y el sparse.
    Keyword,
    /// Solo búsqueda por similitud vectorial: ignora el texto (mantiene sparse si viene).
    Vector,
    /// Fusión híbrida completa (texto + vector + sparse según los inputs). Default.
    #[default]
    Hybrid,
}

/// Perfil de búsqueda configurable por request/namespace (MEM-01).
///
/// Los campos `None` delegan en las constantes core (`RRF_K`,
/// `hybrid_candidate_budget`) de `planner.rs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SearchProfileConfig {
    /// Modo de búsqueda. Default: `Hybrid`.
    #[serde(default)]
    pub mode: SearchProfileMode,
    /// Parámetro `k` de fusión RRF. `None` usa `RRF_K` (60).
    #[serde(default)]
    pub rrf_k: Option<usize>,
    /// Presupuesto de candidatos por canal. `None` usa `hybrid_candidate_budget`.
    #[serde(default)]
    pub candidate_k: Option<usize>,
}

/// Counts and configuration for a hybrid (text+vector) fusion pass.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HybridFusionReport {
    /// Number of candidates from the BM25 text search.
    pub text_candidates: usize,
    /// Number of candidates from the HNSW vector search.
    pub vector_candidates: usize,
    /// Number of unique candidates after RRF fusion.
    pub fused_candidates: usize,
    /// The k parameter used for reciprocal rank fusion.
    pub rrf_k: usize,
}

/// Explanation of a memory search result, including route, hits, and fusion report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchExplanation {
    /// Route used for the search (hybrid, text-only, vector-only, empty).
    pub route: String,
    /// Explained search hits.
    pub hits: Vec<SearchExplanationHit>,
    /// Fusion report present when the route was hybrid.
    pub fusion_report: Option<HybridFusionReport>,
}

/// Per-hit explanation with score, snippet, matched tokens, and BM25 breakdown.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchExplanationHit {
    /// Unique identity string (`namespace\0key`) of the matched record.
    pub identity: String,
    /// Combined relevance score for this hit.
    pub score: f32,
    /// Text snippet surrounding the matched query terms, if available.
    pub snippet: Option<String>,
    /// Query tokens that matched in this record.
    pub matched_tokens: Vec<String>,
    /// Query phrases that matched in this record.
    pub matched_phrases: Vec<String>,
    /// Per-term BM25 scoring breakdown.
    pub bm25_terms: Vec<Bm25TermContribution>,
    /// Rank of this hit in the text-only result set, if applicable.
    pub rrf_text_rank: Option<usize>,
    /// Rank of this hit in the vector-only result set, if applicable.
    pub rrf_vector_rank: Option<usize>,
}

/// Per-term BM25 scoring decomposition for a single search hit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bm25TermContribution {
    /// The query term token.
    pub token: String,
    /// Term frequency in the matched document.
    pub tf: u32,
    /// Document frequency across the namespace.
    pub df: u64,
    /// Total length (in tokens) of the matched document.
    pub doc_len: u32,
    /// BM25 score contribution for this term.
    pub contribution: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DerivedIndexState {
    pub(crate) schema_version: u32,
    pub(crate) rebuilt_at_ms: u64,
    pub(crate) record_count: u64,
    pub(crate) namespace_entries: u64,
    pub(crate) payload_entries: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DerivedIndexRebuildReport {
    pub(crate) record_count: u64,
    pub(crate) namespace_entries: u64,
    pub(crate) payload_entries: u64,
    pub(crate) duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TextIndexState {
    pub(crate) schema_version: u32,
    pub(crate) tokenizer: String,
    pub(crate) tokenizer_version: u32,
    pub(crate) key_format: String,
    pub(crate) rebuilt_at_ms: u64,
    pub(crate) record_count: u64,
    pub(crate) posting_entries: u64,
    pub(crate) doc_stats_entries: u64,
    pub(crate) term_stats_entries: u64,
    pub(crate) namespace_stats_entries: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextIndexRebuildReport {
    pub(crate) record_count: u64,
    pub(crate) posting_entries: u64,
    pub(crate) doc_stats_entries: u64,
    pub(crate) term_stats_entries: u64,
    pub(crate) namespace_stats_entries: u64,
    pub(crate) duration_ms: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct TextIndexCounts {
    pub(crate) record_count: u64,
    pub(crate) posting_entries: u64,
    pub(crate) doc_stats_entries: u64,
    pub(crate) term_stats_entries: u64,
    pub(crate) namespace_stats_entries: u64,
    pub(crate) unknown_entries: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct TextIndexMutationReport {
    pub(crate) postings_written: u64,
    pub(crate) doc_stats_delta: i64,
    pub(crate) term_stats_delta: i64,
    pub(crate) namespace_stats_delta: i64,
}

/// Persisted state marker for the derived sparse-vector inverted index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SparseIndexState {
    pub(crate) schema_version: u32,
    pub(crate) rebuilt_at_ms: u64,
    pub(crate) record_count: u64,
    pub(crate) posting_entries: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SparseIndexRebuildReport {
    pub(crate) record_count: u64,
    pub(crate) posting_entries: u64,
    pub(crate) duration_ms: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct SparseIndexCounts {
    pub(crate) record_count: u64,
    pub(crate) posting_entries: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ExpectedTextIndexEntries {
    pub(crate) entries: BTreeMap<Vec<u8>, Vec<u8>>,
    pub(crate) counts: TextIndexCounts,
    pub(crate) records_scanned: u64,
    pub(crate) namespaces: BTreeSet<String>,
}

/// Stable structural audit report for the derived persistent text index.
///
/// The audit is read-only. It compares text-index postings and BM25/phrase
/// stats against canonical memory records and reports drift without repairing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextIndexAuditReport {
    /// Schema version of the text index spec.
    pub schema_version: u32,
    /// Tokenizer name used by the index.
    pub tokenizer: String,
    /// Tokenizer version used by the index.
    pub tokenizer_version: u32,
    /// Key format identifier used by the index.
    pub key_format: String,
    /// Optional namespace filter applied during the audit.
    pub namespace_filter: Option<String>,
    /// Namespaces that were audited.
    pub namespaces_audited: Vec<String>,
    /// Number of memory records scanned.
    pub records_scanned: u64,
    /// Number of entries expected from canonical records.
    pub expected_entries: u64,
    /// Number of entries actually present in the text index.
    pub actual_entries: u64,
    /// Entries that exist in canonical records but are missing from the index.
    pub missing_entries: u64,
    /// Entries present in the index but not expected from canonical records.
    pub unexpected_entries: u64,
    /// Entries whose value differs (deep audit only).
    pub value_mismatches: u64,
    /// Entries that could not be decoded.
    pub unreadable_entries: u64,
    /// Total mismatch count (sum of missing, unexpected, value, state).
    pub mismatches: u64,
    /// Whether a deep (value-level) audit was performed.
    pub deep_audit: bool,
    /// Posting position errors detected (deep audit only).
    pub position_errors: u64,
    /// Posting term-frequency errors detected (deep audit only).
    pub tf_errors: u64,
    /// Term-statistics document-frequency errors (deep audit only).
    pub df_errors: u64,
    /// Document-stats length errors (deep audit only).
    pub doc_len_errors: u64,
    /// Logical corruptions where values matched but key category mismatched.
    pub logical_corruptions: u64,
    /// Whether the persisted index state is valid and current.
    pub state_valid: bool,
    /// Human-readable status of the index state check.
    pub state_status: String,
    /// Duration of the audit in milliseconds.
    pub duration_ms: u64,
    /// Whether the audit passed (no mismatches found).
    pub passed: bool,
    /// Machine-readable status string ("ok" or "repair_recommended").
    pub status: String,
}
#[cfg(debug_assertions)]
#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;

    // ΓöÇΓöÇ Reports ΓöÇΓöÇ

    #[test]
    fn test_index_rebuild_report() {
        let r = IndexRebuildReport {
            scanned_nodes: 1000,
            indexed_vectors: 900,
            skipped_tombstones: 50,
            duration_ms: 500,
            derived_rebuild_ms: 100,
            index_path: "/tmp/index".into(),
            success: true,
        };
        assert_eq!(r.scanned_nodes, 1000);
        assert_eq!(r.indexed_vectors, 900);
        assert!(r.success);
    }

    #[test]
    fn test_text_index_repair_report() {
        let r = TextIndexRepairReport {
            record_count: 200,
            posting_entries: 1500,
            doc_stats_entries: 200,
            term_stats_entries: 400,
            namespace_stats_entries: 5,
            duration_ms: 600,
            success: true,
        };
        assert_eq!(r.record_count, 200);
        assert!(r.success);
    }

    // ΓöÇΓöÇ HybridFusionReport ΓöÇΓöÇ

    #[test]
    fn test_hybrid_fusion_report() {
        let r = HybridFusionReport {
            text_candidates: 50,
            vector_candidates: 30,
            fused_candidates: 70,
            rrf_k: 60,
        };
        assert_eq!(r.rrf_k, 60);
        assert_eq!(r.fused_candidates, 70);
    }

    // ΓöÇΓöÇ Bm25TermContribution ΓöÇΓöÇ

    #[test]
    fn test_bm25_term_contribution() {
        let c = Bm25TermContribution {
            token: "rust".into(),
            tf: 3,
            df: 10,
            doc_len: 100,
            contribution: 2.5,
        };
        assert_eq!(c.token, "rust");
        assert_eq!(c.tf, 3);
    }

    // ΓöÇΓöÇ SearchExplanation ΓöÇΓöÇ

    #[test]
    fn test_search_explanation_empty() {
        let expl = SearchExplanation {
            route: "empty".into(),
            hits: vec![],
            fusion_report: None,
        };
        assert!(expl.hits.is_empty());
        assert!(expl.fusion_report.is_none());
    }

    // ΓöÇΓöÇ SearchExplanationHit ΓöÇΓöÇ

    #[test]
    fn test_search_explanation_hit() {
        let hit = SearchExplanationHit {
            identity: "ns\0k".into(),
            score: 0.95,
            snippet: Some("...hello world...".into()),
            matched_tokens: vec!["hello".into()],
            matched_phrases: vec![],
            bm25_terms: vec![],
            rrf_text_rank: Some(1),
            rrf_vector_rank: Some(3),
        };
        assert_eq!(hit.identity, "ns\0k");
        assert_eq!(hit.score, 0.95);
        assert!(hit.rrf_text_rank.is_some());
    }

    // ΓöÇΓöÇ TextIndexAuditReport ΓöÇΓöÇ

    #[test]
    fn test_text_index_audit_report_ok() {
        let r = TextIndexAuditReport {
            schema_version: 1,
            tokenizer: "default".into(),
            tokenizer_version: 1,
            key_format: "v1".into(),
            namespace_filter: None,
            namespaces_audited: vec!["ns".into()],
            records_scanned: 100,
            expected_entries: 500,
            actual_entries: 500,
            missing_entries: 0,
            unexpected_entries: 0,
            value_mismatches: 0,
            unreadable_entries: 0,
            mismatches: 0,
            deep_audit: true,
            position_errors: 0,
            tf_errors: 0,
            df_errors: 0,
            doc_len_errors: 0,
            logical_corruptions: 0,
            state_valid: true,
            state_status: "healthy".into(),
            duration_ms: 100,
            passed: true,
            status: "ok".into(),
        };
        assert!(r.passed);
        assert_eq!(r.status, "ok");
    }

    // ΓöÇΓöÇ HybridFusionReport clone/debug ΓöÇΓöÇ

    #[test]
    fn test_hybrid_fusion_report_clone_debug() {
        let r = HybridFusionReport {
            text_candidates: 10,
            vector_candidates: 20,
            fused_candidates: 25,
            rrf_k: 60,
        };
        let cloned = r.clone();
        assert_eq!(r, cloned);
        let dbg = format!("{:?}", r);
        assert!(dbg.contains("rrf_k"));
    }

    // ΓöÇΓöÇ Bm25TermContribution clone ΓöÇΓöÇ

    #[test]
    fn test_bm25_term_contribution_clone() {
        let c = Bm25TermContribution {
            token: "test".into(),
            tf: 2,
            df: 5,
            doc_len: 50,
            contribution: 1.5,
        };
        let cloned = c.clone();
        assert_eq!(c, cloned);
    }

    // ΓöÇΓöÇ SearchExplanationHit clone ΓöÇΓöÇ

    #[test]
    fn test_search_explanation_hit_clone() {
        let hit = SearchExplanationHit {
            identity: "ns\0k".into(),
            score: 0.9,
            snippet: None,
            matched_tokens: vec!["hi".into()],
            matched_phrases: vec![],
            bm25_terms: vec![],
            rrf_text_rank: None,
            rrf_vector_rank: None,
        };
        let cloned = hit.clone();
        assert_eq!(hit, cloned);
    }

    // ΓöÇΓöÇ SearchExplanation with fusion ΓöÇΓöÇ

    #[test]
    fn test_search_explanation_with_fusion() {
        let expl = SearchExplanation {
            route: "hybrid".into(),
            hits: vec![],
            fusion_report: Some(HybridFusionReport {
                text_candidates: 10,
                vector_candidates: 5,
                fused_candidates: 12,
                rrf_k: 60,
            }),
        };
        assert_eq!(expl.route, "hybrid");
        assert!(expl.fusion_report.is_some());
        assert_eq!(expl.fusion_report.unwrap().fused_candidates, 12);
    }

    // ΓöÇΓöÇ IndexRebuildReport clone ΓöÇΓöÇ

    #[test]
    fn test_index_rebuild_report_clone() {
        let r = IndexRebuildReport {
            scanned_nodes: 100,
            indexed_vectors: 90,
            skipped_tombstones: 5,
            duration_ms: 200,
            derived_rebuild_ms: 50,
            index_path: "/tmp/idx".into(),
            success: true,
        };
        let cloned = r.clone();
        assert_eq!(r, cloned);
    }

    // ΓöÇΓöÇ TextIndexRepairReport clone ΓöÇΓöÇ

    #[test]
    fn test_text_index_repair_report_clone() {
        let r = TextIndexRepairReport {
            record_count: 50,
            posting_entries: 200,
            doc_stats_entries: 50,
            term_stats_entries: 100,
            namespace_stats_entries: 3,
            duration_ms: 150,
            success: true,
        };
        let cloned = r.clone();
        assert_eq!(r, cloned);
    }

    // ΓöÇΓöÇ TextIndexAuditReport failure ΓöÇΓöÇ

    #[test]
    fn test_text_index_audit_report_failure() {
        let r = TextIndexAuditReport {
            schema_version: 1,
            tokenizer: "default".into(),
            tokenizer_version: 1,
            key_format: "v1".into(),
            namespace_filter: Some("ns".into()),
            namespaces_audited: vec!["ns".into()],
            records_scanned: 50,
            expected_entries: 300,
            actual_entries: 280,
            missing_entries: 20,
            unexpected_entries: 5,
            value_mismatches: 3,
            unreadable_entries: 1,
            mismatches: 29,
            deep_audit: true,
            position_errors: 2,
            tf_errors: 1,
            df_errors: 0,
            doc_len_errors: 0,
            logical_corruptions: 0,
            state_valid: true,
            state_status: "healthy".into(),
            duration_ms: 80,
            passed: false,
            status: "repair_recommended".into(),
        };
        assert!(!r.passed);
        assert_eq!(r.missing_entries, 20);
        assert_eq!(r.position_errors, 2);
        assert_eq!(r.status, "repair_recommended");
    }
}
