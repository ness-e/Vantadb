//! Score semantics helpers — canonical scoring contract for VantaDB.
//!
//! Thin wrapper over `crate::planner::RRF_K` and `crate::index::distance`
//! kernels (ponytail: delegate, no SIMD duplication). Closes FND-06 H3 gap:
//! adapters duplicated `1.0 - s/2.0` without a single source of truth.
//!
//! All helpers are pure `f32` — no alloc, no unsafe, no trust-boundary input.

use crate::planner::RRF_K;

/// Reciprocal Rank Fusion contribution for a 1-based wire rank.
///
/// Wire exposes 1-based ranks via `rank_map` (sdk/search/debug.rs:23);
/// planner uses 0-based `1/(k+rank+1)`. This helper mirrors the wire form
/// `1/(k + r_wire)` used by `desktop/retrieval-core.ts:rrfContribution`.
#[inline]
pub fn rrf_contribution(rank_1based: Option<usize>, rrf_k: Option<f32>) -> f32 {
    let k = rrf_k.unwrap_or(RRF_K);
    match rank_1based {
        Some(r) if r >= 1 => 1.0 / (k + r as f32),
        _ => 0.0,
    }
}

/// RRF contribution for a 0-based planner rank (internal).
#[inline]
pub fn rrf_contribution_0based(rank_0based: usize, rrf_k: Option<f32>) -> f32 {
    let k = rrf_k.unwrap_or(RRF_K);
    1.0 / (k + rank_0based as f32 + 1.0)
}

/// Convert cosine distance ∈ [0, 2] → similarity ∈ [-1, 1].
///
/// Canonical: `similarity = 1 - distance` (inverse of `cosine_similarity_to_distance`).
/// `distance = 1 - similarity` — core HNSW similarity (higher-is-better) vs wire
/// distance (lower-is-better, MCP `1 - similarity`). For normalized relevance
/// ∈ [0,1] use `1 - distance/2` (adapter MMR).
#[inline]
pub fn cosine_distance_to_similarity(distance: f32) -> f32 {
    1.0 - distance
}

/// Convert cosine similarity ∈ [-1, 1] → distance ∈ [0, 2].
#[inline]
pub fn cosine_similarity_to_distance(similarity: f32) -> f32 {
    // clamped to [0,2] for caller safety — ponytail: minimal branching
    (1.0 - similarity).clamp(0.0, 2.0)
}

/// Clamped cosine distance → similarity with bounds.
#[inline]
pub fn cosine_distance_to_similarity_clamped(distance: f32) -> f32 {
    let d = distance.clamp(0.0, 2.0);
    1.0 - d
}

/// Normalized relevance ∈ [0,1] from cosine distance ∈ [0,2] (adapter helper).
///
/// Adapters duplicate `1.0 - s/2.0` for MMR relevance; centralize here.
#[inline]
pub fn cosine_distance_to_relevance(distance: f32) -> f32 {
    (1.0 - distance / 2.0).clamp(0.0, 1.0)
}

// ponytail: helpers delegate to planner/metrics — no batch SIMD duplication.
// Upgrade to vectorized batch if adapter profiling shows bottleneck.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rrf_contribution_wire_matches_planner() {
        // wire rank 1 with default k=60 → 1/61 ≈ 0.01639
        let wire = rrf_contribution(Some(1), None);
        let planner = rrf_contribution_0based(0, None);
        assert!((wire - planner).abs() < 1e-6);
        assert!((wire - 1.0 / 61.0).abs() < 1e-6);
    }

    #[test]
    fn rrf_contribution_none_zero() {
        assert_eq!(rrf_contribution(None, None), 0.0);
        assert_eq!(rrf_contribution(Some(0), None), 0.0);
    }

    #[test]
    fn cosine_distance_similarity_roundtrip() {
        let cases = [0.0, 0.5, 1.0, 1.5, 2.0];
        for d in cases {
            let s = cosine_distance_to_similarity(d);
            let d2 = cosine_similarity_to_distance(s);
            assert!((d - d2).abs() < 1e-6, "roundtrip {d} → {s} → {d2}");
        }
        // known points: distance 0 → similarity 1, distance 1 → 0, distance 2 → -1
        assert!((cosine_distance_to_similarity(0.0) - 1.0).abs() < 1e-6);
        assert!((cosine_distance_to_similarity(2.0) - (-1.0)).abs() < 1e-6);
        assert!((cosine_distance_to_similarity(1.0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn rrf_contribution_custom_k() {
        // custom k=1 → rank1 → 1/2 =0.5 > default 1/61
        let custom = rrf_contribution(Some(1), Some(1.0));
        let def = rrf_contribution(Some(1), None);
        assert!(custom > def);
        assert!((custom - 0.5).abs() < 1e-6);
    }
}
