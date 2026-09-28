//! Maximal Marginal Relevance reranking (WIRE-08).
//!
//! Core-owned MMR so bindings/adapters stop reimplementing search semantics
//! (api-contract R-8: fusion/selection decisions live in the core).
//!
//! Deterministic: candidates are iterated in their incoming rank order and
//! ties keep the earlier candidate; no randomness, no stored-data mutation.
//!
//! Formula (classic MMR, Carbonell & Goldstein):
//! `mmr(d) = lambda * relevance(d) - (1 - lambda) * max_{s in S} cosine(d, s)`
//! where `S` is the selected set. Relevance is min-max normalized inside the
//! candidate window so `lambda` is meaningful across fusion routes (RRF scores
//! ~1/60 vs cosine ~1.0).

use super::super::serialization::vector_types::{MemorySearchHit, MmrConfig};

/// Default candidate multiplier when `fetch_k` is not set ("fetch_k 5-10x",
/// Backlog WIRE-08).
pub const DEFAULT_FETCH_MULTIPLIER: usize = 5;

/// Hard ceiling for the MMR candidate window (same artificial maximum as
/// Qdrant's `candidates_limit` — avoids pathologically expensive reranks).
pub const MAX_MMR_CANDIDATES: usize = 16_384;

/// Effective candidate window for an MMR request: `fetch_k` or `top_k * 5`,
/// clamped to `[top_k, MAX_MMR_CANDIDATES]`.
pub fn resolve_fetch_k(top_k: usize, cfg: &MmrConfig) -> usize {
    let fetch_k = cfg
        .fetch_k
        .unwrap_or_else(|| top_k.saturating_mul(DEFAULT_FETCH_MULTIPLIER));
    fetch_k.clamp(top_k.max(1), MAX_MMR_CANDIDATES).max(top_k)
}

/// Cosine similarity between two hits' dense vectors. `None` when either
/// record carries no vector or the dimensions differ (treated as no penalty).
fn pair_similarity(a: &MemorySearchHit, b: &MemorySearchHit) -> Option<f32> {
    let (va, vb) = (a.record.vector.as_ref()?, b.record.vector.as_ref()?);
    if va.is_empty() || va.len() != vb.len() {
        return None;
    }
    Some(crate::index::cosine_sim_f32(va, vb))
}

/// Rerank `candidates` (already ranked by relevance) with MMR and select at
/// most `top_k` hits.
///
/// `lambda = 1.0` is identity ordering (pure relevance); `lambda = 0.0` is pure
/// diversity. Records without vectors only compete on relevance.
// ponytail: O(top_k * candidates * top_k) pairwise recompute — fine for the
// documented top_k range (<= ~100); precompute the similarity matrix only if
// large-k MMR ever shows up in a profile.
pub fn rerank(
    candidates: Vec<MemorySearchHit>,
    cfg: &MmrConfig,
    top_k: usize,
) -> Vec<MemorySearchHit> {
    if top_k == 0 || candidates.is_empty() {
        return Vec::new();
    }
    let lambda = cfg.lambda.clamp(0.0, 1.0);

    // Min-max normalized relevance over the candidate window.
    let (mut min, mut max) = (f32::INFINITY, f32::NEG_INFINITY);
    for hit in &candidates {
        if hit.score < min {
            min = hit.score;
        }
        if hit.score > max {
            max = hit.score;
        }
    }
    let span = max - min;
    let relevance = |hit: &MemorySearchHit| {
        if span > 0.0 {
            (hit.score - min) / span
        } else {
            1.0
        }
    };

    let mut remaining = candidates;
    let mut selected: Vec<MemorySearchHit> = Vec::with_capacity(top_k.min(remaining.len()));
    while selected.len() < top_k && !remaining.is_empty() {
        let mut best_idx = 0usize;
        let mut best_score = f32::NEG_INFINITY;
        for (idx, candidate) in remaining.iter().enumerate() {
            let diversity_penalty = selected
                .iter()
                .filter_map(|picked| pair_similarity(candidate, picked))
                .fold(0.0f32, f32::max);
            let score = lambda * relevance(candidate) - (1.0 - lambda) * diversity_penalty;
            if score > best_score {
                best_score = score;
                best_idx = idx;
            }
        }
        selected.push(remaining.remove(best_idx));
    }
    selected
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use crate::sdk::types::{MemoryMetadata, MemoryRecord};

    fn hit(key: &str, score: f32, vector: Option<Vec<f32>>) -> MemorySearchHit {
        MemorySearchHit {
            record: MemoryRecord {
                namespace: "ns".into(),
                key: key.into(),
                payload: String::new(),
                metadata: MemoryMetadata::new(),
                created_at_ms: 0,
                updated_at_ms: 0,
                version: 1,
                node_id: key.bytes().map(u128::from).sum(),
                vector,
                sparse_vector: None,
                expires_at_ms: None,
                superseded_by: None,
                superseded_at_ms: None,
            },
            score,
            explanation: None,
        }
    }

    #[test]
    fn fetch_k_defaults_to_five_times_top_k_and_clamps() {
        let cfg = MmrConfig::default();
        assert_eq!(resolve_fetch_k(10, &cfg), 50);
        assert_eq!(resolve_fetch_k(1, &cfg), 5);
        let cfg = MmrConfig {
            lambda: 0.5,
            fetch_k: Some(2),
        };
        assert_eq!(resolve_fetch_k(10, &cfg), 10, "never below top_k");
        let cfg = MmrConfig {
            lambda: 0.5,
            fetch_k: Some(usize::MAX),
        };
        assert_eq!(resolve_fetch_k(10, &cfg), MAX_MMR_CANDIDATES);
    }

    #[test]
    fn lambda_one_is_identity_ordering() {
        let candidates = vec![
            hit("a", 0.9, Some(vec![1.0, 0.0])),
            hit("b", 0.5, Some(vec![1.0, 0.0])),
            hit("c", 0.1, Some(vec![0.0, 1.0])),
        ];
        let cfg = MmrConfig {
            lambda: 1.0,
            fetch_k: None,
        };
        let out = rerank(candidates, &cfg, 3);
        let keys: Vec<&str> = out.iter().map(|h| h.record.key.as_str()).collect();
        assert_eq!(keys, vec!["a", "b", "c"]);
    }

    #[test]
    fn diversity_lambda_prefers_dissimilar_vector() {
        // `b` is the near-duplicate of `a` (cosine ~1); `c` is orthogonal.
        // With lambda=0.5 the second pick must be `c`: rel(a)=1, rel(b)=0.5,
        // rel(c)=0 → mmr(b) = 0.25 - 0.5 = -0.25 < mmr(c) = 0.
        let candidates = vec![
            hit("a", 0.9, Some(vec![1.0, 0.0])),
            hit("b", 0.5, Some(vec![1.0, 0.0])),
            hit("c", 0.1, Some(vec![0.0, 1.0])),
        ];
        let cfg = MmrConfig {
            lambda: 0.5,
            fetch_k: None,
        };
        let out = rerank(candidates, &cfg, 2);
        let keys: Vec<&str> = out.iter().map(|h| h.record.key.as_str()).collect();
        assert_eq!(keys, vec!["a", "c"], "diverse pick must beat the duplicate");
    }

    #[test]
    fn records_without_vectors_compete_on_relevance_only() {
        let candidates = vec![
            hit("a", 0.9, Some(vec![1.0, 0.0])),
            hit("b", 0.5, None),
            hit("c", 0.1, None),
        ];
        let cfg = MmrConfig {
            lambda: 0.0,
            fetch_k: None,
        };
        // Pure diversity: records with no vector carry zero penalty, so the
        // first remaining candidate (best rank) is picked after `a`.
        let out = rerank(candidates, &cfg, 3);
        assert_eq!(out.len(), 3);
        assert_eq!(out[0].record.key, "a");
    }

    #[test]
    fn empty_candidates_and_zero_top_k_are_empty() {
        let cfg = MmrConfig::default();
        assert!(rerank(Vec::new(), &cfg, 5).is_empty());
        assert!(rerank(vec![hit("a", 1.0, None)], &cfg, 0).is_empty());
    }

    #[test]
    fn rerank_never_returns_more_than_candidates() {
        let candidates = vec![hit("a", 0.9, None), hit("b", 0.5, None)];
        let out = rerank(candidates, &MmrConfig::default(), 10);
        assert_eq!(out.len(), 2);
    }
}
