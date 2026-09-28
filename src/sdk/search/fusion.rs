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
    fuse_rrf_impl(lexical_hits, vector_hits, rrf_k, None).0
}

/// Same as [`fuse_rrf`] with an opt-in deterministic entity-cluster boost
/// (WIRE-05) and per-hit provenance.
///
/// `boost = None` is byte-identical to [`fuse_rrf`] (the base function
/// delegates here). With `Some(boost)`, fused hits that share an entity
/// cluster with at least one other fused candidate receive an additive
/// `weight × peers × 1/(rrf_k+1)` score delta before the final sort — so
/// co-occurring records of the same entity reinforce each other. The boost is
/// reversible: `score − delta == base_score` for every reported entry, and no
/// stored data is mutated.
pub fn fuse_rrf_with_entity_boost(
    lexical_hits: Vec<MemorySearchHit>,
    vector_hits: Vec<MemorySearchHit>,
    rrf_k: f32,
    boost: Option<&EntityBoost>,
) -> (Vec<MemorySearchHit>, EntityBoostReport) {
    let (hits, _report, boost_report) = fuse_rrf_impl(lexical_hits, vector_hits, rrf_k, boost);
    (hits, boost_report)
}

/// Full RRF fusion (hits + hybrid report + entity boost provenance).
/// Crate-internal entry used by the explain path so counts, boost provenance
/// and the fusion report come from the same pass.
pub(crate) fn fuse_rrf_impl(
    lexical_hits: Vec<MemorySearchHit>,
    vector_hits: Vec<MemorySearchHit>,
    rrf_k: f32,
    boost: Option<&EntityBoost>,
) -> (Vec<MemorySearchHit>, HybridFusionReport, EntityBoostReport) {
    tracing::debug!(
        "Fusing lexical candidates ({}) and vector candidates ({}) with RRF_K = {}",
        lexical_hits.len(),
        vector_hits.len(),
        rrf_k
    );
    let text_candidates = lexical_hits.len();
    let vector_candidates = vector_hits.len();
    let mut fused: BTreeMap<(String, String), MemorySearchHit> = BTreeMap::new();
    apply_rrf_contributions(&mut fused, lexical_hits, rrf_k);
    apply_rrf_contributions(&mut fused, vector_hits, rrf_k);

    let mut hits: Vec<_> = fused.into_values().collect();
    let boost_report = apply_entity_boost(&mut hits, boost, rrf_k);
    sort_hits(&mut hits);
    tracing::debug!("Fused candidates count: {}", hits.len());
    let report = HybridFusionReport {
        text_candidates,
        vector_candidates,
        fused_candidates: hits.len(),
        rrf_k: rrf_k as usize,
    };
    (hits, report, boost_report)
}

/// Fuse an arbitrary number of ranked candidate lists (lexical, dense, sparse,
/// ...) via reciprocal rank fusion. Each channel contributes RRF score by rank;
/// hits appearing in several channels accumulate contributions. Sorted
/// descending by score with deterministic tie-breaking (see `sort_hits`).
pub fn fuse_rrf_many(channels: Vec<Vec<MemorySearchHit>>, rrf_k: f32) -> Vec<MemorySearchHit> {
    fuse_rrf_many_impl(channels, rrf_k, None).0
}

/// Same as [`fuse_rrf_many`] with an opt-in entity-cluster boost (WIRE-05).
/// `boost = None` is byte-identical to [`fuse_rrf_many`].
pub fn fuse_rrf_many_with_entity_boost(
    channels: Vec<Vec<MemorySearchHit>>,
    rrf_k: f32,
    boost: Option<&EntityBoost>,
) -> (Vec<MemorySearchHit>, EntityBoostReport) {
    fuse_rrf_many_impl(channels, rrf_k, boost)
}

fn fuse_rrf_many_impl(
    channels: Vec<Vec<MemorySearchHit>>,
    rrf_k: f32,
    boost: Option<&EntityBoost>,
) -> (Vec<MemorySearchHit>, EntityBoostReport) {
    let mut fused: BTreeMap<(String, String), MemorySearchHit> = BTreeMap::new();
    for channel in channels {
        apply_rrf_contributions(&mut fused, channel, rrf_k);
    }
    let mut hits: Vec<_> = fused.into_values().collect();
    let boost_report = apply_entity_boost(&mut hits, boost, rrf_k);
    sort_hits(&mut hits);
    (hits, boost_report)
}

pub fn fuse_rrf_with_report(
    lexical_hits: Vec<MemorySearchHit>,
    vector_hits: Vec<MemorySearchHit>,
    rrf_k: f32,
) -> (Vec<MemorySearchHit>, HybridFusionReport) {
    let (hits, report, _boost_report) = fuse_rrf_impl(lexical_hits, vector_hits, rrf_k, None);
    (hits, report)
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

// ── Entity boost (WIRE-05) ────────────────────────────────────────────────

/// Opt-in deterministic entity-cluster boost for RRF fusion (WIRE-05).
///
/// Maps fused-record identities `(namespace, key)` to a canonical entity
/// cluster label (typically produced by
/// [`crate::entity::linking`]). When two or more hits of the same cluster
/// co-occur in one fused candidate set, each of them receives an additive
/// `weight × (peers) × 1/(rrf_k+1)` delta — expressed as a fraction of the
/// strongest possible single-channel contribution, so the boost scales with
/// the configured `rrf_k`.
///
/// The boost is opt-in (`None`/empty → no-op) and reversible: it only reorders
/// scores at fusion time and every applied delta is reported in
/// [`EntityBoostReport`] together with the pre-boost score.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityBoost {
    links: BTreeMap<(String, String), String>,
    weight: f32,
}

impl EntityBoost {
    /// Default weight: a quarter of the strongest single-channel RRF
    /// contribution (`0.25 / (rrf_k + 1)`).
    pub const DEFAULT_WEIGHT: f32 = 0.25;

    /// Build an empty boost (no links; [`Self::DEFAULT_WEIGHT`]).
    pub fn new() -> Self {
        Self {
            links: BTreeMap::new(),
            weight: Self::DEFAULT_WEIGHT,
        }
    }

    /// Link one record identity to a cluster label (builder style).
    pub fn link(
        mut self,
        namespace: impl Into<String>,
        key: impl Into<String>,
        cluster: impl Into<String>,
    ) -> Self {
        self.links
            .insert((namespace.into(), key.into()), cluster.into());
        self
    }

    /// Override the boost weight (must be finite and positive to have effect;
    /// other values make the boost a documented no-op).
    pub fn with_weight(mut self, weight: f32) -> Self {
        self.weight = weight;
        self
    }

    /// Effective boost weight.
    pub fn weight(&self) -> f32 {
        self.weight
    }

    /// `true` when no record is linked (the boost is a no-op).
    pub fn is_empty(&self) -> bool {
        self.links.is_empty()
    }

    /// Cluster label of one record identity, if linked.
    pub fn cluster_of(&self, namespace: &str, key: &str) -> Option<&str> {
        self.links
            .get(&(namespace.to_string(), key.to_string()))
            .map(String::as_str)
    }
}

impl Default for EntityBoost {
    fn default() -> Self {
        Self::new()
    }
}

/// Per-hit provenance of an applied entity boost (WIRE-05).
#[derive(Debug, Clone, PartialEq)]
pub struct EntityBoostProvenance {
    /// Canonical entity cluster label shared by the boosted hits.
    pub cluster: String,
    /// Identities (`"namespace\0key"`) of the other fused hits in the same
    /// cluster, sorted (the same convention as `debug::hit_identities`).
    pub peers: Vec<String>,
    /// Score before the boost (exact — recoverable via `base_score`; `score − delta ≈ base_score` within f32 rounding).
    pub base_score: f32,
    /// Additive delta applied to the hit score.
    pub delta: f32,
}

/// Per-hit provenance of one boost application, keyed by record identity.
///
/// Computed over the fused candidate set BEFORE `top_k` truncation and
/// `exclude_superseded` filtering: it may reference identities that are not
/// present in the returned `hits` (their peers still influence the ranking).
///
/// Empty when the boost is `None`, empty, or ineffective (no cluster has two
/// or more co-occurring hits).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EntityBoostReport {
    entries: BTreeMap<(String, String), EntityBoostProvenance>,
}

impl EntityBoostReport {
    /// `true` when no hit was boosted.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Number of boosted hits.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Provenance of one boosted hit, if any.
    pub fn get(&self, namespace: &str, key: &str) -> Option<&EntityBoostProvenance> {
        self.entries.get(&(namespace.to_string(), key.to_string()))
    }

    /// Iterate `((namespace, key), provenance)` pairs in identity order.
    pub fn iter(&self) -> impl Iterator<Item = (&(String, String), &EntityBoostProvenance)> {
        self.entries.iter()
    }
}

/// Result of [`crate::sdk::Embedded::search_with_entity_boost`] (WIRE-05).
///
/// `hits` is the final ranked list (with the opt-in entity boost applied when
/// a cluster had two or more co-occurring candidates); `boost_report` carries
/// the per-hit provenance of every applied delta (empty when the boost was a
/// no-op, e.g. an empty index or no co-occurring cluster members).
#[derive(Debug, Clone, PartialEq)]
pub struct EntityBoostedSearch {
    /// Final ranked hits.
    pub hits: Vec<MemorySearchHit>,
    /// Per-hit provenance of the applied boost.
    pub boost_report: EntityBoostReport,
}

/// Apply the entity-cluster boost to already-fused hits and collect
/// per-hit provenance. A documented no-op (empty report) when the boost is
/// `None`, empty, its weight is not finite-positive, or `rrf_k` is invalid.
fn apply_entity_boost(
    hits: &mut [MemorySearchHit],
    boost: Option<&EntityBoost>,
    rrf_k: f32,
) -> EntityBoostReport {
    let mut report = EntityBoostReport::default();
    let Some(boost) = boost else {
        return report;
    };
    let denominator = rrf_k + 1.0;
    if boost.is_empty()
        || boost.weight <= 0.0
        || !boost.weight.is_finite()
        || !denominator.is_finite()
        || denominator <= 0.0
    {
        return report;
    }

    // Group hit positions by cluster label (BTreeMap → deterministic order).
    let mut groups: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (index, hit) in hits.iter().enumerate() {
        if let Some(cluster) = boost.cluster_of(&hit.record.namespace, &hit.record.key) {
            groups.entry(cluster).or_default().push(index);
        }
    }

    let top_contribution = 1.0 / denominator;
    for (cluster, indices) in groups {
        if indices.len() < 2 {
            continue;
        }
        let delta = boost.weight * (indices.len() - 1) as f32 * top_contribution;
        for &index in &indices {
            let peers = indices
                .iter()
                .filter(|&&other| other != index)
                .map(|&other| {
                    format!(
                        "{}\0{}",
                        hits[other].record.namespace, hits[other].record.key
                    )
                })
                .collect();
            let base_score = hits[index].score;
            hits[index].score = base_score + delta;
            report.entries.insert(
                (
                    hits[index].record.namespace.clone(),
                    hits[index].record.key.clone(),
                ),
                EntityBoostProvenance {
                    cluster: cluster.to_string(),
                    peers,
                    base_score,
                    delta,
                },
            );
        }
    }
    report
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

    // ── Entity boost (WIRE-05) ──────────────────────────────────────────

    /// x and y share a cluster; z is unlinked.
    fn linked_boost() -> EntityBoost {
        EntityBoost::new()
            .link("ns", "x", "cluster-x")
            .link("ns", "y", "cluster-x")
    }

    fn boosted_fixture() -> (Vec<MemorySearchHit>, Vec<MemorySearchHit>) {
        let lexical = vec![make_hit("ns", "x", 0.9, 1), make_hit("ns", "z", 0.8, 3)];
        let vector = vec![make_hit("ns", "y", 0.95, 2), make_hit("ns", "z", 0.7, 3)];
        (lexical, vector)
    }

    fn position(hits: &[MemorySearchHit], key: &str) -> usize {
        hits.iter()
            .position(|hit| hit.record.key == key)
            .expect("hit present")
    }

    #[test]
    fn fuse_rrf_with_entity_boost_off_is_byte_identical() {
        let (lexical, vector) = boosted_fixture();
        let plain = fuse_rrf(lexical.clone(), vector.clone(), RRF_K);
        let (boosted, report) = fuse_rrf_with_entity_boost(lexical, vector, RRF_K, None);
        assert_eq!(plain, boosted, "None boost must be byte-identical");
        assert!(report.is_empty());
    }

    #[test]
    fn fuse_rrf_with_entity_boost_empty_index_is_noop() {
        let (lexical, vector) = boosted_fixture();
        let plain = fuse_rrf(lexical.clone(), vector.clone(), RRF_K);
        let empty = EntityBoost::new();
        let (boosted, report) = fuse_rrf_with_entity_boost(lexical, vector, RRF_K, Some(&empty));
        assert_eq!(plain, boosted, "empty boost index must be a no-op");
        assert!(report.is_empty());
    }

    #[test]
    fn fuse_rrf_with_entity_boost_promotes_cluster_peers() {
        let (lexical, vector) = boosted_fixture();

        // OFF: z accumulates contributions from both channels and leads.
        let plain = fuse_rrf(lexical.clone(), vector.clone(), RRF_K);
        assert_eq!(
            plain
                .iter()
                .map(|h| h.record.key.as_str())
                .collect::<Vec<_>>(),
            vec!["z", "x", "y"],
            "baseline ranking before the boost"
        );

        // ON (weight = 1 top contribution): x and y overtake z.
        let boost = linked_boost().with_weight(1.0);
        let (boosted, report) = fuse_rrf_with_entity_boost(lexical, vector, RRF_K, Some(&boost));
        assert!(
            position(&boosted, "x") < position(&boosted, "z")
                && position(&boosted, "y") < position(&boosted, "z"),
            "cluster peers x and y must overtake unlinked z"
        );

        // Provenance: every boosted hit is reversible and explains itself.
        let x = report.get("ns", "x").expect("x boosted");
        assert_eq!(x.cluster, "cluster-x");
        assert_eq!(x.peers, vec!["ns\0y".to_string()]);
        assert!(
            report.get("ns", "z").is_none(),
            "unlinked hits carry no edge"
        );
        for (key, base) in [("x", 1.0 / (RRF_K + 1.0)), ("y", 1.0 / (RRF_K + 1.0))] {
            let provenance = report.get("ns", key).expect("peers boosted");
            let (score, delta) = boosted
                .iter()
                .find(|h| h.record.key == key)
                .map(|h| (h.score, provenance.delta))
                .expect("hit present");
            assert!((score - delta - base).abs() < 1e-6, "reversible score");
            assert!((provenance.base_score - base).abs() < 1e-6);
        }
    }

    #[test]
    fn fuse_rrf_with_entity_boost_is_deterministic() {
        let boost = linked_boost().with_weight(0.5);
        let (lexical, vector) = boosted_fixture();
        let first =
            fuse_rrf_with_entity_boost(lexical.clone(), vector.clone(), RRF_K, Some(&boost));
        let second = fuse_rrf_with_entity_boost(lexical, vector, RRF_K, Some(&boost));
        assert_eq!(first, second, "same inputs → same ranking + provenance");
    }

    #[test]
    fn fuse_rrf_with_entity_boost_singleton_cluster_is_noop() {
        // x is linked to y, but only x appears among the candidates.
        let lexical = vec![make_hit("ns", "x", 0.9, 1)];
        let vector = vec![make_hit("ns", "z", 0.7, 3)];
        let boost = linked_boost();
        let (boosted, report) =
            fuse_rrf_with_entity_boost(lexical.clone(), vector.clone(), RRF_K, Some(&boost));
        let plain = fuse_rrf(lexical, vector, RRF_K);
        assert_eq!(plain, boosted, "a lone cluster member gets no boost");
        assert!(report.is_empty());
    }

    #[test]
    fn fuse_rrf_with_entity_boost_non_finite_weight_is_noop() {
        let (lexical, vector) = boosted_fixture();
        let plain = fuse_rrf(lexical.clone(), vector.clone(), RRF_K);
        let bad = linked_boost().with_weight(f32::NAN);
        let (boosted, report) = fuse_rrf_with_entity_boost(lexical, vector, RRF_K, Some(&bad));
        assert_eq!(plain, boosted, "non-finite weight must not corrupt scores");
        assert!(report.is_empty());
    }

    #[test]
    fn fuse_rrf_many_with_entity_boost_off_is_byte_identical() {
        let (lexical, vector) = boosted_fixture();
        let plain = fuse_rrf_many(vec![lexical.clone(), vector.clone()], RRF_K);
        let (boosted, report) = fuse_rrf_many_with_entity_boost(vec![lexical, vector], RRF_K, None);
        assert_eq!(plain, boosted, "None boost must be byte-identical");
        assert!(report.is_empty());
    }

    #[test]
    fn fuse_rrf_many_with_entity_boost_promotes_cluster_peers() {
        let (lexical, vector) = boosted_fixture();
        let boost = linked_boost().with_weight(1.0);
        let (boosted, report) =
            fuse_rrf_many_with_entity_boost(vec![lexical, vector], RRF_K, Some(&boost));
        assert!(position(&boosted, "x") < position(&boosted, "z"));
        assert!(position(&boosted, "y") < position(&boosted, "z"));
        assert_eq!(report.len(), 2);
    }
}
