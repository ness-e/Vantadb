use std::collections::{HashMap, HashSet};

use crate::error::Result;
use crate::node::FieldValue;
use crate::storage::StorageEngine;

use super::expand::ExpandedNodes;
use super::pipeline::{GraphRagEdge, GraphRagNode};

fn extract_content(node: &crate::node::UnifiedNode) -> String {
    // GR-03: fallback to FIELD_PAYLOAD ("__vanta_payload") when content/text are absent
    let content_keys = [
        "content",
        "text",
        "payload",
        "description",
        crate::sdk::FIELD_PAYLOAD,
    ];
    for key in &content_keys {
        if let Some(FieldValue::String(s)) = node.relational.get(*key) {
            return s.clone();
        }
    }
    String::new()
}

pub fn retrieve_and_rank(
    engine: &StorageEngine,
    seed_scores: &[(u128, f32)],
    expanded: &ExpandedNodes,
    top_k: usize,
) -> Result<Vec<GraphRagNode>> {
    let seed_score_map: HashMap<u128, f32> = seed_scores.iter().copied().collect();

    // MEM0-02 / FIND-608: Min-Max calibration of seed scores so small RRF values
    // (~0.016) are scaled to [0.0, 1.0] and not crushed by hop_boost [0.33, 1.0].
    let (min_s, max_s) = if seed_scores.is_empty() {
        (0.0_f32, 0.0_f32)
    } else {
        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;
        for &(_, s) in seed_scores {
            if s < min {
                min = s;
            }
            if s > max {
                max = s;
            }
        }
        (min, max)
    };

    let all_ids: Vec<u128> = expanded.visited.clone();
    if all_ids.is_empty() {
        return Ok(Vec::new());
    }

    let nodes = engine.get_many(&all_ids)?;

    let mut candidates: Vec<GraphRagNode> = Vec::with_capacity(nodes.len());

    for node in &nodes {
        let content = extract_content(node);
        let norm_seed = if let Some(&seed_score) = seed_score_map.get(&node.id) {
            if (max_s - min_s) > 1e-6 {
                (seed_score - min_s) / (max_s - min_s)
            } else {
                1.0
            }
        } else {
            0.0
        };
        let hop_dist = expanded
            .hop_distances
            .get(&node.id)
            .copied()
            .unwrap_or(u32::MAX);
        let hop_boost = 1.0 / (1.0 + hop_dist as f32);
        // GR-01: node is already loaded via get_many, read edge count directly to eliminate N+1 engine.get calls.
        let degree = node.edges.len() as f32;
        let degree_factor = degree / (degree + 10.0);

        let combined = 0.6 * norm_seed + 0.3 * hop_boost + 0.1 * degree_factor;

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
    let mut all: Vec<u128> = top_ids.iter().copied().collect();
    if all.is_empty() {
        return Ok(Vec::new());
    }
    // DIST-15: deterministic node iteration. `HashSet` is seeded per instance,
    // so iterating it directly produced differently ordered edges on two runs
    // over the same graph — a non-deterministic `context_text` that broke
    // cross-binding parity. Sort by id before fetching.
    all.sort_unstable();

    let nodes = engine.get_many(&all)?;

    // GR-02 / FIND-607: resolve labels in ONE `label_intern` acquisition
    // instead of one lock per edge (N lock round-trips on the search
    // path). The snapshot is taken after `get_many` returned — no
    // storage/index guard is held while `label_intern` is locked
    // (Regla 8; `label_intern` is outside the `insert_lock` order).
    let label_ids: HashSet<u32> = nodes
        .iter()
        .flat_map(|n| n.edges.iter().map(|e| e.label_id))
        .collect();
    let labels = engine.resolve_labels(&label_ids);
    let mut unresolved = 0usize;

    let mut edges = Vec::new();
    for node in &nodes {
        for edge in &node.edges {
            if expanded_set.contains(&edge.target) || top_ids.contains(&edge.target) {
                let label = match labels.get(&edge.label_id) {
                    Some(label) => label.clone(),
                    None => {
                        // Degraded, not fatal: pre-NOT-01 databases stored
                        // edges before the interner was persisted, so their
                        // label ids legitimately do not resolve.
                        unresolved += 1;
                        String::new()
                    }
                };
                edges.push(GraphRagEdge {
                    source: node.id,
                    target: edge.target,
                    label,
                });
            }
        }
    }
    if unresolved > 0 {
        tracing::warn!(
            unresolved,
            "graphrag: edge label ids missing from the label interner \
             (legacy pre-NOT-01 data or corruption) — emitted as empty labels"
        );
    }

    // Final order is pinned: (source, target, label). Cross-binding parity
    // requires the same `context_text` for the same graph on every surface.
    edges.sort_by(|a, b| {
        (a.source, a.target, a.label.as_str()).cmp(&(b.source, b.target, b.label.as_str()))
    });

    Ok(edges)
}
