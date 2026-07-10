use crate::index::distance::calculate_similarity;
use crate::node::FilterBitset;

pub(crate) fn flat_search(
    nodes: &dashmap::DashMap<
        u128,
        super::graph::HnswNode,
        std::hash::BuildHasherDefault<twox_hash::XxHash64>,
    >,
    query_vec: &[f32],
    query_mask: &FilterBitset,
    top_k: usize,
    metric: crate::node::DistanceMetric,
) -> Vec<(u128, f32)> {
    use crate::storage::engine::FLAG_TOMBSTONE;

    let query_norm = match metric {
        crate::node::DistanceMetric::Cosine => {
            let norm = super::distance::f32_l2_norm(query_vec);
            if norm < f32::EPSILON {
                None
            } else {
                Some(norm)
            }
        }
        crate::node::DistanceMetric::Euclidean => Some(super::distance::f32_l2_norm(query_vec)),
    };

    let mut scored: Vec<(u128, f32)> = Vec::with_capacity(nodes.len().min(10000));
    for item in nodes.iter() {
        let node = item.value();
        if (node.flags & FLAG_TOMBSTONE) != 0 {
            continue;
        }
        if !query_mask.is_all_set() && !node.bitset.matches_mask(query_mask) {
            continue;
        }
        let sim = calculate_similarity(query_vec, query_norm, None, None, &node.vec_data, metric);
        if !sim.is_nan() {
            scored.push((node.id, sim));
        }
    }

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(top_k);

    let mut results = Vec::with_capacity(scored.len());
    for (id, score) in scored {
        let adjusted = match metric {
            crate::node::DistanceMetric::Euclidean => -(-score).max(0.0).sqrt(),
            crate::node::DistanceMetric::Cosine => score,
        };
        results.push((id, adjusted));
    }
    results
}
