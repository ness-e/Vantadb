//! Index-level search entry point: `search_nearest` — the public HNSW
//! search over the layered graph, with IVF/SCANN/flat routing.

use ahash::RandomState;

use super::profile::SearchProfile;
use crate::index::distance::f32_l2_norm;
use crate::index::graph::{CPIndex, NodeSimMin};
use crate::index::IndexType;
use crate::node::{DistanceMetric, FilterBitset};

impl CPIndex {
    #[tracing::instrument(skip(self, query_vec, vector_store), level = "debug")]
    pub fn search_nearest(
        &self,
        query_vec: &[f32],
        _q_1bit: Option<&[u64]>,
        _q_3bit: Option<(&[u8], f32)>,
        query_mask: &FilterBitset,
        top_k: usize,
        vector_store: Option<&crate::storage::vfile::VantaFile>,
    ) -> Vec<(u128, f32)> {
        // AUDREP-55: a zero-norm query is undefined under cosine similarity
        // (the cosine of a zero vector is 0/0). Historically this silently
        // fell back to Euclidean scoring, which swaps the score range
        // (cosine ∈ [-1, 1] vs euclidean ∈ (-∞, 0]) so the caller's score
        // thresholds become meaningless across calls. This function (and the
        // VecIndex::search trait) return a plain Vec, so an error cannot be
        // propagated cleanly at this boundary; instead we fail loudly and
        // return no results, keeping the cosine metric pure — consistent with
        // AUDREP-27, which rejects zero-norm inserts under cosine.
        if self.config.distance_metric == DistanceMetric::Cosine
            && f32_l2_norm(query_vec) < f32::EPSILON
        {
            tracing::warn!(
                "zero-norm cosine query is undefined; returning no results \
                 (AUDREP-55, was silently re-scored with euclidean)"
            );
            return Vec::new();
        }

        // IVF path: lazy-build on first search, then search. AUDREP-09:
        // rebuild whenever the node count changed since the last build, so
        // vectors added after a cached IVF was built become candidates.
        if self.config.index_type == IndexType::Ivf {
            return self.search_ivf(query_vec, query_mask, top_k);
        }

        // SCANN (SQ8) path: same lazy-build pattern as IVF. Without this the
        // configured `index_type = Scann` would silently fall through to the
        // HNSW graph, ignoring the selected backend entirely.
        if self.config.index_type == IndexType::Scann {
            return self.search_scann(query_vec, query_mask, top_k);
        }

        if self.use_flat_search() {
            return crate::index::flat::flat_search(
                &self.nodes,
                query_vec,
                query_mask,
                top_k,
                self.config.distance_metric,
            );
        }

        let ep = match self.get_entry_point() {
            Some(id) => id,
            None => return Vec::new(),
        };

        let static_ef = self.config.ef_search;
        let ef_search = if self.config.auto_tune {
            let tuned_ef = crate::index::auto_tune::AutoTune::current_ef();
            static_ef.max(tuned_ef).max(top_k)
        } else {
            static_ef.max(top_k)
        };
        let (effective_metric, query_norm, query_inv_norm) = match self.config.distance_metric {
            DistanceMetric::Cosine => {
                // AUDREP-55 guard at the top of search_nearest guarantees
                // norm >= f32::EPSILON here, so 1/norm is finite and the
                // metric stays Cosine — no silent Euclidean fallback remains.
                let norm = f32_l2_norm(query_vec);
                debug_assert!(
                    norm >= f32::EPSILON,
                    "zero-norm cosine query must be rejected up-front (AUDREP-55)"
                );
                (DistanceMetric::Cosine, Some(norm), Some(1.0 / norm))
            }
            DistanceMetric::Euclidean => {
                let norm = f32_l2_norm(query_vec);
                (DistanceMetric::Euclidean, Some(norm), None)
            }
            DistanceMetric::SparseDot => {
                // Sparse has its own brute-force search path; never routed through
                // the dense HNSW query. Degenerate norm pair if reached.
                (DistanceMetric::SparseDot, None, None)
            }
        };
        let mut curr_entry_points = vec![ep];
        let mut visited: std::collections::HashSet<u128, RandomState> =
            std::collections::HashSet::with_capacity_and_hasher(
                ef_search.max(top_k).saturating_mul(3),
                RandomState::new(),
            );

        let mut profile = SearchProfile::new();
        let max_l = self.max_layer.load(std::sync::atomic::Ordering::Acquire);
        for layer in (1..=max_l).rev() {
            visited.clear();
            let mut w = self.search_layer(
                query_vec,
                query_norm,
                query_inv_norm,
                &curr_entry_points,
                1,
                layer,
                &crate::node::ALL_BITSET,
                false, // no ACORN on coarse layers
                vector_store,
                effective_metric,
                &mut visited,
                &mut profile,
            );
            if let Some(NodeSimMin(_, best_id)) = w.pop() {
                curr_entry_points = vec![best_id];
            }
        }

        visited.clear();
        let w = self.search_layer(
            query_vec,
            query_norm,
            query_inv_norm,
            &curr_entry_points,
            ef_search,
            0,
            query_mask,
            !query_mask.is_all_set(), // ACORN enabled for non-trivial masks
            vector_store,
            effective_metric,
            &mut visited,
            &mut profile,
        );
        profile.log(ef_search, top_k);

        let mut result: Vec<NodeSimMin> = w.into_sorted_vec();
        result.retain(|ns| !ns.0.is_nan());

        result.truncate(top_k);

        let mut final_results = Vec::with_capacity(result.len());
        for NodeSimMin(score, id) in result {
            let adjusted_score = match effective_metric {
                DistanceMetric::Euclidean => -(-score).max(0.0).sqrt(),
                DistanceMetric::Cosine => score,
                DistanceMetric::SparseDot => score,
            };
            final_results.push((id, adjusted_score));
        }
        final_results
    }
}
