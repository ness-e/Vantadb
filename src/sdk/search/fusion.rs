//! Fusion + profile helpers for hybrid retrieval (C2M3).
//!
//! Pure move of the planner-owned helpers that `sdk/search/*` already called
//! (`resolve_search_profile`, `search_mode`, `trimmed_text_query`,
//! `hybrid_candidate_budget`, `fuse_rrf*`, `sort_hits`). They operate purely
//! on SDK types, so `sdk` is their natural owner — this inversion removes the
//! `sdk/search → planner` edge while `planner` no longer imports `sdk` at
//! all. No semantic change: bodies and tests are verbatim moves.

use std::collections::BTreeMap;

use super::super::serialization::vector_types::{MemorySearchHit, MemorySearchRequest};
use super::super::types::HybridFusionReport;
use crate::search_profile::{
    SearchProfileMode, CANDIDATE_MULTIPLIER, MAX_CANDIDATE_BUDGET, MIN_CANDIDATE_BUDGET, RRF_K,
};

/// Derive the per-arm candidate budget for hybrid retrieval.
///
/// The budget is clamped to `[MIN_CANDIDATE_BUDGET, MAX_CANDIDATE_BUDGET]`
/// and never falls below `top_k` so that `fuse_rrf` always has enough
/// candidates to fill the requested result set.
pub fn hybrid_candidate_budget(top_k: usize, candidate_k: Option<usize>) -> usize {
    match candidate_k {
        Some(k) => k.max(top_k),
        None => top_k
            .saturating_mul(CANDIDATE_MULTIPLIER)
            .clamp(MIN_CANDIDATE_BUDGET, MAX_CANDIDATE_BUDGET)
            .max(top_k),
    }
}

/// Valores efectivos de fusión para un request con profile opcional (MEM-01).
///
/// Devuelve `(rrf_k, candidate_k)` efectivos: los valores del
/// [`SearchProfileConfig`](crate::search_profile::SearchProfileConfig) si están presentes,
/// o las constantes core (`RRF_K`, `hybrid_candidate_budget`) en caso contrario.
pub fn resolve_search_profile(request: &MemorySearchRequest) -> (f32, Option<usize>) {
    let profile = request.search_profile;
    let rrf_k = profile
        .and_then(|p| p.rrf_k)
        .map(|k| k as f32)
        .unwrap_or(RRF_K);
    let candidate_k = profile.and_then(|p| p.candidate_k);
    (rrf_k, candidate_k)
}

/// Modo de búsqueda efectivo de un request (default `Hybrid`, MEM-01).
pub fn search_mode(request: &MemorySearchRequest) -> SearchProfileMode {
    request.search_profile.map(|p| p.mode).unwrap_or_default()
}

// ── Normalised request fields ─────────────────────────────────────────────

/// Extract the trimmed, non-empty text query from a search request.
pub fn trimmed_text_query(request: &MemorySearchRequest) -> Option<&str> {
    request
        .text_query
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
}

// ── RRF fusion ────────────────────────────────────────────────────────────

/// Fuse lexical and vector hit lists using Reciprocal Rank Fusion.
///
/// Each ranked hit contributes `1 / (RRF_K + rank + 1)` to its score in
/// the merged result. Hits appearing in both lists receive contributions
/// from both rankings. The returned list is sorted descending by score,
/// with ties broken by `key` then `node_id` for determinism.
pub fn fuse_rrf(
    lexical_hits: Vec<MemorySearchHit>,
    vector_hits: Vec<MemorySearchHit>,
    rrf_k: f32,
) -> Vec<MemorySearchHit> {
    tracing::debug!(
        "Fusing lexical candidates ({}) and vector candidates ({}) with RRF_K = {}",
        lexical_hits.len(),
        vector_hits.len(),
        rrf_k
    );
    let mut fused: BTreeMap<(String, String), MemorySearchHit> = BTreeMap::new();
    apply_rrf_contributions(&mut fused, lexical_hits, rrf_k);
    apply_rrf_contributions(&mut fused, vector_hits, rrf_k);

    let mut hits: Vec<_> = fused.into_values().collect();
    sort_hits(&mut hits);
    tracing::debug!("Fused candidates count: {}", hits.len());
    hits
}

/// Fuse an arbitrary number of ranked candidate lists (lexical, dense, sparse,
/// ...) via reciprocal rank fusion. Each channel contributes RRF score by rank;
/// hits appearing in several channels accumulate contributions. Sorted
/// descending by score with deterministic tie-breaking (see `sort_hits`).
pub fn fuse_rrf_many(channels: Vec<Vec<MemorySearchHit>>, rrf_k: f32) -> Vec<MemorySearchHit> {
    let mut fused: BTreeMap<(String, String), MemorySearchHit> = BTreeMap::new();
    for channel in channels {
        apply_rrf_contributions(&mut fused, channel, rrf_k);
    }
    let mut hits: Vec<_> = fused.into_values().collect();
    sort_hits(&mut hits);
    hits
}
pub fn fuse_rrf_with_report(
    lexical_hits: Vec<MemorySearchHit>,
    vector_hits: Vec<MemorySearchHit>,
    rrf_k: f32,
) -> (Vec<MemorySearchHit>, HybridFusionReport) {
    let text_candidates = lexical_hits.len();
    let vector_candidates = vector_hits.len();
    let fused_hits = fuse_rrf(lexical_hits, vector_hits, rrf_k);
    let report = HybridFusionReport {
        text_candidates,
        vector_candidates,
        fused_candidates: fused_hits.len(),
        rrf_k: rrf_k as usize,
    };
    (fused_hits, report)
}

fn apply_rrf_contributions(
    fused: &mut BTreeMap<(String, String), MemorySearchHit>,
    hits: Vec<MemorySearchHit>,
    rrf_k: f32,
) {
    for (rank, hit) in hits.into_iter().enumerate() {
        let contribution = 1.0 / (rrf_k + rank as f32 + 1.0);
        let identity = (hit.record.namespace.clone(), hit.record.key.clone());
        fused
            .entry(identity)
            .and_modify(|existing| existing.score += contribution)
            .or_insert_with(|| MemorySearchHit {
                record: hit.record,
                score: contribution,
                explanation: None,
            });
    }
}

// ── Sorting ───────────────────────────────────────────────────────────────

/// Sort hits descending by score; ties broken by `key` then `node_id`.
pub fn sort_hits(hits: &mut [MemorySearchHit]) {
    hits.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.record.key.cmp(&b.record.key))
            .then(a.record.node_id.cmp(&b.record.node_id))
    });
}

// ── Unit tests (verbatim move from planner.rs) ────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sdk::{MemoryMetadata, MemoryRecord};
    use crate::search_profile::SearchProfileConfig;

    // ── Candidate budget ─────────────────────────────────────────────────

    #[test]
    fn budget_is_clamped_at_min() {
        assert_eq!(hybrid_candidate_budget(1, None), MIN_CANDIDATE_BUDGET);
    }

    #[test]
    fn budget_is_clamped_at_max_for_mid_range_top_k() {
        // top_k=64 → 64*4=256 = MAX_CANDIDATE_BUDGET; max(256, 64)=256
        let budget = hybrid_candidate_budget(64, None);
        assert_eq!(budget, MAX_CANDIDATE_BUDGET);
    }

    #[test]
    fn budget_returns_top_k_when_top_k_exceeds_max() {
        // top_k=10_000 → 10_000*4 clamped to 256; but max(256, 10_000)=10_000
        // The guardrail ensures we always fetch at least top_k candidates.
        let budget = hybrid_candidate_budget(10_000, None);
        assert!(budget >= 10_000);
    }

    #[test]
    fn budget_is_at_least_top_k() {
        // top_k=50 → 50*4=200 which is within [32,256]
        let budget = hybrid_candidate_budget(50, None);
        assert!(budget >= 50);
        assert_eq!(budget, 200);
    }

    #[test]
    fn budget_never_below_top_k_for_large_top_k() {
        // top_k=200 → 200*4=800 clamped to 256; but max(256, 200)=256 ≥ top_k
        let budget = hybrid_candidate_budget(200, None);
        assert!(budget >= 200);
    }

    // ── RRF fusion ───────────────────────────────────────────────────────

    fn make_hit(ns: &str, key: &str, score: f32, node_id: u128) -> MemorySearchHit {
        MemorySearchHit {
            record: MemoryRecord {
                namespace: ns.to_string(),
                key: key.to_string(),
                payload: String::new(),
                metadata: MemoryMetadata::new(),
                created_at_ms: 0,
                updated_at_ms: 0,
                expires_at_ms: Some(0),
                version: 0,
                node_id,
                vector: None,
                sparse_vector: None,
                superseded_by: None,
                superseded_at_ms: None,
            },
            score,
            explanation: None,
        }
    }

    #[test]
    fn fuse_rrf_returns_deterministic_order() {
        let lex = vec![make_hit("ns", "a", 0.9, 1), make_hit("ns", "b", 0.8, 2)];
        let vec = vec![make_hit("ns", "b", 0.95, 2), make_hit("ns", "c", 0.7, 3)];
        let result = fuse_rrf(lex, vec, RRF_K);
        // "b" appears in both lists → highest combined RRF score
        assert_eq!(result[0].record.key, "b");
    }

    #[test]
    fn fuse_rrf_scores_are_positive() {
        let lex = vec![make_hit("ns", "x", 0.5, 10)];
        let vec = vec![make_hit("ns", "x", 0.5, 10)];
        let result = fuse_rrf(lex, vec, RRF_K);
        assert_eq!(result.len(), 1);
        assert!(result[0].score > 0.0);
    }

    #[test]
    fn fuse_rrf_deduplicates_same_namespace_key() {
        let lex = vec![make_hit("ns", "dup", 0.9, 99)];
        let vec = vec![make_hit("ns", "dup", 0.9, 99)];
        let result = fuse_rrf(lex, vec, RRF_K);
        assert_eq!(result.len(), 1, "same (namespace, key) must be merged");
    }

    #[test]
    fn sort_hits_is_deterministic_on_equal_scores() {
        let mut hits = vec![make_hit("ns", "z", 0.5, 20), make_hit("ns", "a", 0.5, 10)];
        sort_hits(&mut hits);
        assert_eq!(hits[0].record.key, "a", "ties broken alphabetically by key");
    }

    // ── trimmed_text_query ───────────────────────────────────────────────

    #[test]
    fn trimmed_text_query_none() {
        let req = MemorySearchRequest {
            text_query: None,
            ..Default::default()
        };
        assert_eq!(trimmed_text_query(&req), None);
    }

    #[test]
    fn trimmed_text_query_empty() {
        let req = MemorySearchRequest {
            text_query: Some(String::new()),
            ..Default::default()
        };
        assert_eq!(trimmed_text_query(&req), None);
    }

    #[test]
    fn trimmed_text_query_whitespace() {
        let req = MemorySearchRequest {
            text_query: Some("   ".into()),
            ..Default::default()
        };
        assert_eq!(trimmed_text_query(&req), None);
    }

    #[test]
    fn trimmed_text_query_valid() {
        let req = MemorySearchRequest {
            text_query: Some("hello world".into()),
            ..Default::default()
        };
        assert_eq!(trimmed_text_query(&req), Some("hello world"));
    }

    #[test]
    fn trimmed_text_query_trims_input() {
        let req = MemorySearchRequest {
            text_query: Some("  query  ".into()),
            ..Default::default()
        };
        assert_eq!(trimmed_text_query(&req), Some("query"));
    }

    // ── fuse_rrf_with_report ────────────────────────────────────────────

    #[test]
    fn fuse_rrf_with_report_counts() {
        let lex = vec![make_hit("ns", "a", 0.9, 1)];
        let vec = vec![make_hit("ns", "b", 0.8, 2)];
        let (_hits, report) = fuse_rrf_with_report(lex.clone(), vec.clone(), RRF_K);
        assert_eq!(report.text_candidates, 1);
        assert_eq!(report.vector_candidates, 1);
        assert_eq!(report.rrf_k, RRF_K as usize);
    }

    #[test]
    fn fuse_rrf_with_report_fused_results() {
        let lex = vec![make_hit("ns", "a", 0.9, 1)];
        let vec = vec![make_hit("ns", "a", 0.8, 1)];
        let (hits, _report) = fuse_rrf_with_report(lex, vec, RRF_K);
        assert_eq!(hits.len(), 1, "same key merged into one");
        let expected = 2.0 / (RRF_K + 1.0);
        assert!((hits[0].score - expected).abs() < 1e-6);
    }

    #[test]
    fn fuse_rrf_with_report_reports_custom_k() {
        let lex = vec![make_hit("ns", "a", 0.9, 1)];
        let vec = vec![make_hit("ns", "b", 0.8, 2)];
        let (_hits, report) = fuse_rrf_with_report(lex, vec, 100.0);
        assert_eq!(report.rrf_k, 100);
    }

    #[test]
    fn fuse_rrf_custom_k_changes_scores() {
        let lex = vec![make_hit("ns", "b", 0.8, 2)];
        let vec = vec![make_hit("ns", "b", 0.95, 2)];
        let default = fuse_rrf(lex.clone(), vec.clone(), RRF_K);
        let custom = fuse_rrf(lex, vec, 1.0);
        assert_eq!(default[0].record.key, "b");
        assert!(
            custom[0].score > default[0].score,
            "k menor => contribuciones mayores"
        );
    }

    #[test]
    fn resolve_search_profile_defaults_to_core_constants() {
        let req = MemorySearchRequest::default();
        let (rrf_k, candidate_k) = resolve_search_profile(&req);
        assert_eq!(rrf_k, RRF_K);
        assert_eq!(candidate_k, None);
    }

    #[test]
    fn resolve_search_profile_uses_profile_values() {
        let req = MemorySearchRequest {
            search_profile: Some(SearchProfileConfig {
                rrf_k: Some(100),
                candidate_k: Some(128),
                ..Default::default()
            }),
            ..Default::default()
        };
        let (rrf_k, candidate_k) = resolve_search_profile(&req);
        assert_eq!(rrf_k, 100.0);
        assert_eq!(candidate_k, Some(128));
    }

    #[test]
    fn hybrid_candidate_budget_with_explicit_candidate_k() {
        assert_eq!(hybrid_candidate_budget(5, Some(50)), 50);
        assert_eq!(
            hybrid_candidate_budget(100, Some(50)),
            100,
            "nunca por debajo de top_k"
        );
    }

    // ── sort_hits edge cases ────────────────────────────────────────────

    #[test]
    fn sort_hits_descending_order() {
        let mut hits = vec![
            make_hit("ns", "a", 0.3, 1),
            make_hit("ns", "b", 0.9, 2),
            make_hit("ns", "c", 0.5, 3),
        ];
        sort_hits(&mut hits);
        assert_eq!(hits[0].record.key, "b", "highest score first");
        assert_eq!(hits[1].record.key, "c", "middle score second");
        assert_eq!(hits[2].record.key, "a", "lowest score last");
    }

    #[test]
    fn sort_hits_ties_broken_by_key_then_node_id() {
        let mut hits = vec![
            make_hit("ns", "c", 0.5, 3),
            make_hit("ns", "a", 0.5, 1),
            make_hit("ns", "b", 0.5, 2),
        ];
        sort_hits(&mut hits);
        assert_eq!(hits[0].record.key, "a", "ties broken alphabetically");
        assert_eq!(hits[1].record.key, "b");
        assert_eq!(hits[2].record.key, "c");
    }

    #[test]
    fn sort_hits_ties_same_key_different_node_id() {
        let mut hits = vec![make_hit("ns", "x", 0.5, 2), make_hit("ns", "x", 0.5, 1)];
        sort_hits(&mut hits);
        assert_eq!(hits[0].record.node_id, 1, "lower node_id first on tie");
        assert_eq!(hits[1].record.node_id, 2);
    }

    #[test]
    fn sort_hits_empty_list() {
        let mut hits: Vec<MemorySearchHit> = vec![];
        sort_hits(&mut hits);
        assert!(hits.is_empty());
    }
}
