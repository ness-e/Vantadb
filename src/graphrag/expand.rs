use std::collections::{HashMap, VecDeque};

use crate::error::Result;
use crate::storage::StorageEngine;

pub struct ExpandedNodes {
    pub visited: Vec<u128>,
    pub hop_distances: HashMap<u128, u32>,
    pub max_hops_reached: usize,
}

pub fn expand_nodes(
    engine: &StorageEngine,
    seed_ids: &[u128],
    hops: usize,
    max_nodes: usize,
) -> Result<ExpandedNodes> {
    expand_nodes_capped(engine, seed_ids, hops, max_nodes, usize::MAX, None)
}

/// BFS expansion with per-node degree and weight guards (EGO-15).
///
/// Follows at most `max_degree_per_node` edges out of each visited node, in
/// storage order (deterministic per graph). `usize::MAX` disables the cap.
/// Edges below `min_edge_weight` are skipped before the cap applies (`None`
/// keeps all edges). Seeds are always visited; a cap of 0 yields seeds only.
pub fn expand_nodes_capped(
    engine: &StorageEngine,
    seed_ids: &[u128],
    hops: usize,
    max_nodes: usize,
    max_degree_per_node: usize,
    min_edge_weight: Option<f32>,
) -> Result<ExpandedNodes> {
    if hops == 0 || seed_ids.is_empty() {
        let hop_distances: HashMap<u128, u32> = seed_ids.iter().map(|&id| (id, 0)).collect();
        return Ok(ExpandedNodes {
            visited: seed_ids.to_vec(),
            hop_distances,
            max_hops_reached: 0,
        });
    }

    let mut visited: Vec<u128> = Vec::new();
    let mut hop_distances: HashMap<u128, u32> = HashMap::new();
    let mut queue: VecDeque<u128> = VecDeque::new();
    let mut distances: HashMap<u128, u32> = HashMap::new();

    for &id in seed_ids {
        if hop_distances.insert(id, 0).is_none() {
            visited.push(id);
            queue.push_back(id);
            distances.insert(id, 0);
        }
    }

    while let Some(current) = queue.pop_front() {
        let current_dist = distances[&current];
        if current_dist >= hops as u32 {
            continue;
        }
        if visited.len() >= max_nodes {
            break;
        }
        let Ok(Some(node)) = engine.get(current) else {
            continue;
        };
        // EGO-15: cap hub fan-out — a high-degree node otherwise triggers a
        // combinatorial read burst (Ego bounded this defensively client-side).
        // Weak edges are filtered before the cap so the budget is spent on
        // eligible edges.
        let passes_weight = |w: f32| min_edge_weight.is_none_or(|m| w >= m);
        for edge in node
            .edges
            .iter()
            .filter(|e| passes_weight(e.weight))
            .take(max_degree_per_node)
        {
            if visited.len() >= max_nodes {
                break;
            }
            if let std::collections::hash_map::Entry::Vacant(e) = hop_distances.entry(edge.target) {
                let next_dist = current_dist + 1;
                e.insert(next_dist);
                distances.insert(edge.target, next_dist);
                visited.push(edge.target);
                queue.push_back(edge.target);
            }
        }
    }

    let max_hops_reached = distances
        .values()
        .copied()
        .max()
        .map(|d| d as usize)
        .unwrap_or(0);

    Ok(ExpandedNodes {
        visited,
        hop_distances,
        max_hops_reached,
    })
}
