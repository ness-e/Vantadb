use std::collections::HashSet;

use super::pipeline::{GraphRagEdge, GraphRagNode};

pub fn generate_context(nodes: &[GraphRagNode], edges: &[GraphRagEdge]) -> String {
    if nodes.is_empty() {
        return String::new();
    }

    let mut parts = Vec::new();

    parts.push("## Relevant Nodes".to_string());
    for node in nodes {
        parts.push(format!(
            "- Node {} (score: {:.4}, hop_distance: {})",
            node.id, node.score, node.hop_distance
        ));
        for line in node.content.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                parts.push(format!("  {}", trimmed));
            }
        }
    }

    if !edges.is_empty() {
        parts.push(String::new());
        parts.push("## Graph Relationships".to_string());
        let mut seen: HashSet<(u128, u128)> = HashSet::new();
        for edge in edges {
            if seen.insert((edge.source, edge.target)) {
                parts.push(format!(
                    "- Node {} --[{}]--> Node {}",
                    edge.source, edge.label, edge.target
                ));
            }
        }
    }

    parts.join("\n")
}
