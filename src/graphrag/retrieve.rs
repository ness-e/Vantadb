use std::collections::{HashMap, HashSet};

use crate::error::Result;
use crate::node::FieldValue;
use crate::storage::StorageEngine;

use super::expand::ExpandedNodes;
use super::pipeline::{GraphRagEdge, GraphRagNode};

fn extract_content(node: &crate::node::UnifiedNode) -> String {
    let content_keys = ["content", "payload", "text", "description"];
    for key in &content_keys {
        if let Some(FieldValue::String(s)) = node.relational.get(*key) {
            return s.clone();
        }
    }
    String::new()
}

fn degree_boost(engine: &StorageEngine, id: u128) -> f32 {
    if let Ok(Some(node)) = engine.get(id) {
        node.edges.len() as f32
    } else {
        0.0
    }
}

pub fn retrieve_and_rank(
    engine: &StorageEngine,
    seed_scores: &[(u128, f32)],
    expanded: &ExpandedNodes,
    top_k: usize,
) -> Result<Vec<GraphRagNode>> {
    let seed_score_map: HashMap<u128, f32> = seed_scores.iter().copied().collect();

    let all_ids: Vec<u128> = expanded.visited.clone();
    if all_ids.is_empty() {
        return Ok(Vec::new());
    }

    let nodes = engine.get_many(&all_ids)?;

    let mut candidates: Vec<GraphRagNode> = Vec::with_capacity(nodes.len());

    for node in &nodes {
        let content = extract_content(node);
        let seed_score = seed_score_map.get(&node.id).copied().unwrap_or(0.0);
        let hop_dist = expanded
            .hop_distances
            .get(&node.id)
            .copied()
            .unwrap_or(u32::MAX);
        let hop_boost = 1.0 / (1.0 + hop_dist as f32);
        let degree = degree_boost(engine, node.id);
        let degree_factor = degree / (degree + 10.0);

        let combined = 0.6 * seed_score + 0.3 * hop_boost + 0.1 * degree_factor;

        candidates.push(GraphRagNode {
            id: node.id,
            content,
            score: combined,
            hop_distance: hop_dist,
        });
    }

    candidates.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    candidates.truncate(top_k);

    Ok(candidates)
}

pub fn collect_edges(
    engine: &StorageEngine,
    top_ids: &HashSet<u128>,
    expanded_set: &HashSet<u128>,
) -> Result<Vec<GraphRagEdge>> {
    let all: Vec<u128> = top_ids.iter().copied().collect();
    if all.is_empty() {
        return Ok(Vec::new());
    }

    let nodes = engine.get_many(&all)?;
    let mut edges = Vec::new();

    for node in &nodes {
        for edge in &node.edges {
            if expanded_set.contains(&edge.target) || top_ids.contains(&edge.target) {
                edges.push(GraphRagEdge {
                    source: node.id,
                    target: edge.target,
                    label: engine.resolve_label(edge.label_id).unwrap_or_default(),
                });
            }
        }
    }

    Ok(edges)
}
