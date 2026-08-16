use super::super::builder::VantaEmbedded;
use super::super::serialization::impl_sparse_index::{
    decode_sparse_posting, sparse_posting_prefix,
};
use super::super::serialization::{matches_memory_filters, memory_record_from_node};
use super::super::types::*;
use crate::backend::BackendPartition;
use crate::error::Result;
use crate::node::UnifiedNode;
use std::collections::BTreeMap;

impl VantaEmbedded {
    /// Sparse-dot search over the derived sparse-vector inverted index.
    ///
    /// The `SparseIndex` partition stores one posting per (namespace, dim,
    /// record). A query only walks the posting lists for the dims it contains,
    /// so cost is O(sum of matching posting-list lengths) instead of the old
    /// O(records in namespace) brute-force scan (`NUEVO-22`).
    pub(super) fn sparse_memory_search(
        &self,
        namespace: &str,
        query_sparse: &crate::node::SparseVector,
        filters: &VantaMemoryMetadata,
        top_k: usize,
    ) -> Result<Vec<VantaMemorySearchHit>> {
        if query_sparse.is_empty() || top_k == 0 {
            return Ok(Vec::new());
        }
        let engine = self.engine_handle()?;

        let mut scores: BTreeMap<u128, f32> = BTreeMap::new();
        for (dim, query_weight) in query_sparse.0.iter() {
            let prefix = sparse_posting_prefix(namespace, *dim);
            for entry in
                engine.scan_partition_prefix_iter(BackendPartition::SparseIndex, &prefix)?
            {
                let (_posting_key, posting_value) = entry?;
                let posting = decode_sparse_posting(&posting_value)?;
                scores
                    .entry(posting.node_id)
                    .and_modify(|score| *score += query_weight * posting.weight)
                    .or_insert(query_weight * posting.weight);
            }
        }

        let mut hits = Vec::with_capacity(top_k);
        let node_ids: Vec<u128> = scores.keys().copied().collect();
        if !node_ids.is_empty() {
            let mut node_map: std::collections::HashMap<u128, UnifiedNode> =
                std::collections::HashMap::with_capacity(node_ids.len());
            for n in engine.get_many(&node_ids)? {
                node_map.insert(n.id, n);
            }
            for (node_id, score) in scores {
                if let Some(node) = node_map.get(&node_id) {
                    if let Some(record) = memory_record_from_node(node) {
                        if record.namespace == namespace && matches_memory_filters(&record, filters)
                        {
                            hits.push(VantaMemorySearchHit {
                                record,
                                score,
                                explanation: None,
                            });
                        }
                    }
                }
            }
        }

        crate::planner::sort_hits(&mut hits);
        hits.truncate(top_k);
        Ok(hits)
    }
}
