use crate::index::distance::calculate_similarity;
use crate::node::FilterBitset;

/// ponytail: full DashMap O(n) scan is by design — only called when
/// `flat_threshold` (default 10000) bounds the node count. For larger
/// datasets the HNSW path handles search without iterating all nodes.
pub(crate) fn flat_search(
    nodes: &dashmap::DashMap<u128, super::graph::HnswNode>,
    query_vec: &[f32],
    query_mask: &FilterBitset,
    top_k: usize,
    metric: crate::node::DistanceMetric,
) -> Vec<(u128, f32)> {
    use crate::storage::engine::FLAG_TOMBSTONE;

    let mut results: Vec<(u128, f32)> = nodes
        .iter()
        .filter(|entry| {
            let node = entry.value();
            (node.flags & FLAG_TOMBSTONE) == 0
                && (query_mask.is_all_set() || node.bitset.matches_mask(query_mask))
                && !node.vec_data.is_none()
        })
        .map(|entry| {
            let id = *entry.key();
            let node = entry.value();
            let sim = calculate_similarity(query_vec, None, None, None, &node.vec_data, metric);
            (id, sim)
        })
        .collect();

    results.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(top_k);
    results
}
