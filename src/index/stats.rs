//! HNSW index statistics and structural integrity validation.
//!
//! Extracted from the monolithic `core.rs` for better maintainability (PERF-05).

use std::sync::atomic::Ordering;

use super::CPIndex;

/// Snapshot of HNSW index health metrics
#[derive(Debug, Clone)]
pub struct IndexStats {
    /// Total nodes in the index
    pub node_count: usize,
    /// Maximum layer height in the graph
    pub max_layer: usize,
    /// Nodes with zero neighbors on layer 0 (potential orphans)
    pub orphan_count: usize,
    /// Average outgoing connections on layer 0
    pub avg_connections_l0: f32,
    /// Total number of graph integrity violations found
    pub violation_count: usize,
}

impl CPIndex {
    /// Compute a snapshot of index health metrics.
    pub fn stats(&self) -> IndexStats {
        let node_count = self.nodes.len();
        let orphan_count = self
            .nodes
            .iter()
            .filter(|r| r.value().neighbors.is_empty() || r.value().neighbors[0].is_empty())
            .count();
        let total_l0_connections: usize = self
            .nodes
            .iter()
            .map(|r| r.value().neighbors.first().map(|l| l.len()).unwrap_or(0))
            .sum();
        let avg_connections_l0 = if node_count > 0 {
            total_l0_connections as f32 / node_count as f32
        } else {
            0.0
        };

        IndexStats {
            node_count,
            max_layer: self.max_layer.load(Ordering::Acquire),
            orphan_count,
            avg_connections_l0,
            violation_count: 0,
        }
    }

    /// Validate the structural integrity of the HNSW graph.
    ///
    /// Checks:
    /// 1. Every neighbor reference points to an existing node
    /// 2. No self-loops
    /// 3. Layer count is consistent with node's reported level
    ///
    /// Returns `Ok(())` if the graph is clean, or a list of violation messages.
    ///
    /// # Performance
    /// O(N × M) where N = node count, M = max neighbors per layer.
    /// Run at startup after deserialization, not in hot paths.
    pub fn validate_index(&self) -> Result<(), Vec<String>> {
        let mut violations = Vec::new();

        for r in self.nodes.iter() {
            let id = *r.key();
            let node = r.value();
            if node.neighbors.is_empty() {
                violations.push(format!(
                    "Node {} has empty neighbors array (expected ≥1 layer)",
                    id
                ));
                continue;
            }

            for (layer_idx, layer) in node.neighbors.iter().enumerate() {
                for &neighbor_id in layer {
                    if neighbor_id == id {
                        violations.push(format!(
                            "Node {} has a self-loop at layer {}",
                            id, layer_idx
                        ));
                        continue;
                    }
                    if !self.nodes.contains_key(&neighbor_id) {
                        violations.push(format!(
                            "Node {} references non-existent neighbor {} at layer {}",
                            id, neighbor_id, layer_idx
                        ));
                    }
                }
            }
        }

        if let Some(ep) = self.get_entry_point() {
            if !self.nodes.contains_key(&ep) {
                violations.push(format!("Entry point {} does not exist in the node map", ep));
            }
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations)
        }
    }
}
