//! Alternative index backends driven per-search or by config: IVF (lazy
//! rebuild), SCANN (SQ8), and explicit `search_with_method` routing.

use crate::index::flat::flat_search;
use crate::index::graph::CPIndex;
use crate::index::ivf::{IvfConfig, IvfIndex};
use crate::index::scann::ScannIndex;
use crate::index::IndexType;
use crate::index::VecIndex;
use crate::node::{DistanceMetric, FilterBitset, VectorRepresentations};

impl CPIndex {
    /// Lazy-build and search the IVF index (AUDREP-09). Shared by
    /// `search_nearest` (config `index_type = Ivf`) and per-search overrides
    /// from bindings (`method = "ivf"`).
    pub(crate) fn search_ivf(
        &self,
        query_vec: &[f32],
        query_mask: &FilterBitset,
        top_k: usize,
    ) -> Vec<(u128, f32)> {
        let mut guard = self.ivf_index.lock();
        let node_count = self.nodes.len();
        if guard.as_ref().is_none_or(|_| {
            self.ivf_built_at_node_count
                .load(std::sync::atomic::Ordering::Relaxed)
                != node_count
        }) {
            let ivf_config = IvfConfig {
                nlist: (node_count as f64).sqrt() as usize + 1,
                nprobe: 10,
                distance_metric: self.config.distance_metric,
            };
            *guard = Some(IvfIndex::build(&self.nodes, &ivf_config));
            self.ivf_built_at_node_count
                .store(node_count, std::sync::atomic::Ordering::Relaxed);
        }
        match guard.as_ref() {
            Some(ivf) => ivf.search(query_vec, top_k, query_mask),
            None => Vec::new(),
        }
    }

    /// Lazy-build and search the SCANN (SQ8) index. Mirrors the IVF lazy
    /// cache: rebuilt whenever the node count diverges from the last build.
    pub(crate) fn search_scann(
        &self,
        query_vec: &[f32],
        query_mask: &FilterBitset,
        top_k: usize,
    ) -> Vec<(u128, f32)> {
        let mut guard = self.scann_index.lock();
        let node_count = self.nodes.len();
        if guard.as_ref().is_none_or(|_| {
            self.scann_built_at_node_count
                .load(std::sync::atomic::Ordering::Relaxed)
                != node_count
        }) {
            let scann = ScannIndex::new(self.config.distance_metric);
            for entry in self.nodes.iter() {
                let node = entry.value();
                if let VectorRepresentations::Full(v) = &node.vec_data {
                    if let Err(e) = VecIndex::add(
                        &scann,
                        node.id,
                        node.bitset.clone(),
                        VectorRepresentations::Full(v.clone()),
                        node.storage_offset,
                    ) {
                        tracing::warn!(node_id = node.id, error = %e, "scann rebuild: add rejected");
                    }
                }
            }
            *guard = Some(scann);
            self.scann_built_at_node_count
                .store(node_count, std::sync::atomic::Ordering::Relaxed);
        }
        match guard.as_ref() {
            Some(scann) => VecIndex::search(
                scann,
                query_vec,
                query_mask,
                top_k,
                None,
                self.config.distance_metric,
            ),
            None => Vec::new(),
        }
    }

    /// Run a search through an explicit index backend, ignoring the engine's
    /// configured `index_type`. Used by per-search `method` overrides from
    /// bindings; the shared `config` is never mutated (thread-safe).
    ///
    /// `metric` is the per-request `distance_metric` (MCP-02). HNSW and flat
    /// score exactly with it; IVF and SCANN are metric-bound (their lazy
    /// caches — centroids / codebooks — are built with the configured
    /// metric), so a request metric different from config is rejected with a
    /// clear error instead of silently scoring with the wrong metric.
    pub(crate) fn search_with_method(
        &self,
        method: IndexType,
        query_vec: &[f32],
        query_mask: &FilterBitset,
        top_k: usize,
        metric: DistanceMetric,
    ) -> crate::error::Result<Vec<(u128, f32)>> {
        let metric_bound_error = |backend: &str| {
            crate::error::VantaError::InvalidInput(format!(
                "distance_metric {:?} requested but the {} backend is metric-bound to the \
                 configured metric {:?} (built at lazy-cache time). Use method=hnsw or \
                 method=flat for a per-request metric, or configure the engine with {:?}",
                metric, backend, self.config.distance_metric, metric
            ))
        };
        match method {
            IndexType::Ivf => {
                if metric != self.config.distance_metric {
                    return Err(metric_bound_error("ivf"));
                }
                Ok(self.search_ivf(query_vec, query_mask, top_k))
            }
            IndexType::Scann => {
                if metric != self.config.distance_metric {
                    return Err(metric_bound_error("scann"));
                }
                Ok(self.search_scann(query_vec, query_mask, top_k))
            }
            IndexType::Flat => Ok(flat_search(
                &self.nodes,
                query_vec,
                query_mask,
                top_k,
                metric,
            )),
            IndexType::Hnsw | IndexType::DiskAnn => Ok(self.search_nearest_with_metric(
                query_vec, None, None, query_mask, top_k, None, metric,
            )),
        }
    }
}
