//! Graph-related SDK types: nodes, edges, and input/record views.

use super::super::types::{VantaFields, VantaStorageTier};
use crate::node::{FieldValue, UnifiedNode, VectorRepresentations};
use serde::{Deserialize, Serialize};

/// Stable graph edge representation for external SDKs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VantaEdgeRecord {
    /// Target node id this edge points to.
    pub target: u128,
    /// Edge label describing the relationship.
    pub label: String,
    /// Edge weight for weighted graph algorithms.
    pub weight: f32,
}

/// Stable node payload accepted by external SDKs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VantaNodeInput {
    /// Numeric node identifier.
    pub id: u128,
    /// Optional text content stored in the `content` field.
    pub content: Option<String>,
    /// Optional embedding vector.
    pub vector: Option<Vec<f32>>,
    /// Relational fields key-value pairs.
    pub fields: VantaFields,
}

impl VantaNodeInput {
    /// Create a new node input with the given id.
    /// Content, vector, and fields default to empty/None.
    pub fn new(id: u128) -> Self {
        Self {
            id,
            content: None,
            vector: None,
            fields: VantaFields::new(),
        }
    }
}

/// Stable node view returned to external SDKs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VantaNodeRecord {
    /// Numeric node identifier.
    pub id: u128,
    /// Relational fields key-value pairs.
    pub fields: VantaFields,
    /// Optional embedding vector.
    pub vector: Option<Vec<f32>>,
    /// Dimension count of the vector (0 if no vector).
    pub vector_dimensions: usize,
    /// Outgoing graph edges.
    pub edges: Vec<VantaEdgeRecord>,
    /// Telemetry confidence score (0.0–1.0).
    pub confidence_score: f32,
    /// Telemetry importance score.
    pub importance: f32,
    /// Number of access hits recorded.
    pub hits: u32,
    /// Unix-ms timestamp of last access.
    pub last_accessed: u64,
    /// Telemetry epoch counter.
    pub epoch: u32,
    /// Storage tier (hot or cold).
    pub tier: VantaStorageTier,
    /// Whether the node is alive (not tombstoned).
    pub is_alive: bool,
}

impl From<UnifiedNode> for VantaNodeRecord {
    fn from(node: UnifiedNode) -> Self {
        let is_alive = node.is_alive();
        let (vector, vector_dimensions) = match node.vector {
            VectorRepresentations::Full(vector) => {
                let dims = vector.len();
                (Some(vector), dims)
            }
            VectorRepresentations::None => (None, 0),
            other => (None, other.dimensions()),
        };

        let tier = match node.tier {
            crate::node::NodeTier::Hot => VantaStorageTier::Hot,
            crate::node::NodeTier::Cold => VantaStorageTier::Cold,
        };

        let fields = node
            .relational
            .into_iter()
            .map(|(key, value)| (key, value.into()))
            .collect();

        let edges = node
            .edges
            .into_iter()
            .map(|edge| VantaEdgeRecord {
                target: edge.target,
                label: edge.label,
                weight: edge.weight,
            })
            .collect();

        Self {
            id: node.id,
            fields,
            vector,
            vector_dimensions,
            edges,
            confidence_score: node.confidence_score,
            importance: node.importance,
            hits: node.hits,
            last_accessed: node.last_accessed,
            epoch: node.epoch,
            tier,
            is_alive,
        }
    }
}
