//! Graph-related SDK types: nodes, edges, and input/record views.

use super::super::types::{u128_serde, VantaFields, VantaStorageTier};
use crate::node::{LabelIntern, UnifiedNode, VectorRepresentations};
use serde::{Deserialize, Serialize};

/// Stable graph edge representation for external SDKs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VantaEdgeRecord {
    /// Target node id this edge points to.
    #[serde(with = "u128_serde")]
    pub target: u128,
    /// Edge label describing the relationship.
    pub label: String,
    /// Edge weight for weighted graph algorithms.
    pub weight: f32,
    /// Whether this entry is the auto-created reverse half of a bidirectional
    /// edge (`add_edge`). Load-bearing for directional traversal
    /// (`TraversalDirection::Reverse`, src/graph.rs) and preserved so a
    /// serialize→restore cycle keeps topology identical. CORE-02.
    #[serde(default)]
    pub reverse: bool,
    /// Logical creation timestamp (Unix-ms). `0` when unknown (legacy data).
    #[serde(default)]
    pub created_at_ms: u64,
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
    #[serde(with = "u128_serde")]
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

/// Convert a `UnifiedNode` to an SDK `VantaNodeRecord`, resolving edge labels
/// via the provided interner.
pub(crate) fn unified_to_record(node: UnifiedNode, label_intern: &LabelIntern) -> VantaNodeRecord {
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
            label: label_intern
                .resolve(edge.label_id)
                .unwrap_or("<unknown>")
                .to_string(),
            weight: edge.weight,
            reverse: edge.reverse,
            created_at_ms: edge.created_at_ms,
        })
        .collect();

    VantaNodeRecord {
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

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use crate::node::{LabelIntern, UnifiedNode};
    use crate::sdk::types::VantaValue;

    #[allow(dead_code)]
    fn empty_intern() -> LabelIntern {
        LabelIntern::new()
    }

    #[test]
    fn test_node_input_new() {
        let input = VantaNodeInput::new(42);
        assert_eq!(input.id, 42);
        assert!(input.content.is_none());
        assert!(input.vector.is_none());
        assert!(input.fields.is_empty());
    }

    #[test]
    fn test_node_record_from_unified_node_with_vector() {
        let mut intern = LabelIntern::new();
        let knows_id = intern.intern("knows");
        let mut node = UnifiedNode::with_vector(7, vec![0.1, 0.2, 0.3]);
        node.set_field("name", crate::node::FieldValue::String("test".into()));
        node.set_field("count", crate::node::FieldValue::Int(10));
        node.add_weighted_edge(42, knows_id, 0.9);

        let record = unified_to_record(node, &intern);
        assert_eq!(record.id, 7);
        assert_eq!(record.vector, Some(vec![0.1, 0.2, 0.3]));
        assert_eq!(record.vector_dimensions, 3);
        assert_eq!(record.edges.len(), 1);
        assert_eq!(record.edges[0].target, 42);
        assert_eq!(record.edges[0].label, "knows");
        assert_eq!(record.edges[0].weight, 0.9);
        assert_eq!(
            record.fields.get("name"),
            Some(&VantaValue::String("test".into()))
        );
        assert_eq!(record.fields.get("count"), Some(&VantaValue::Int(10)));
        assert!(record.is_alive);
        assert_eq!(record.tier, VantaStorageTier::Cold);
        assert_eq!(record.confidence_score, 0.5);
        assert_eq!(record.importance, 0.1);
    }

    #[test]
    fn test_node_record_from_unified_node_without_vector() {
        let intern = LabelIntern::new();
        let node = UnifiedNode::new(99);
        let record = unified_to_record(node, &intern);
        assert_eq!(record.id, 99);
        assert!(record.vector.is_none());
        assert_eq!(record.vector_dimensions, 0);
        assert!(record.edges.is_empty());
        assert!(record.fields.is_empty());
    }

    #[test]
    fn test_node_record_from_deleted_node() {
        let intern = LabelIntern::new();
        let mut node = UnifiedNode::new(5);
        node.mark_deleted();
        let record = unified_to_record(node, &intern);
        assert!(!record.is_alive);
    }

    #[test]
    fn test_node_record_from_unified_node_with_multiple_edges() {
        let mut intern = LabelIntern::new();
        let friend_id = intern.intern("friend");
        let colleague_id = intern.intern("colleague");
        let mut node = UnifiedNode::new(1);
        node.add_weighted_edge(10, friend_id, 1.0);
        node.add_weighted_edge(20, colleague_id, 0.5);
        let record = unified_to_record(node, &intern);
        assert_eq!(record.edges.len(), 2);
        assert_eq!(record.edges[0].target, 10);
        assert_eq!(record.edges[1].target, 20);
    }

    #[test]
    fn test_edge_record_serialization_roundtrip() {
        let edge = VantaEdgeRecord {
            target: 100,
            label: "connected_to".into(),
            weight: 0.75,
            reverse: false,
            created_at_ms: 1234,
        };
        let json = serde_json::to_string(&edge).unwrap();
        let deserialized: VantaEdgeRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, edge);
        // verify u128 is serialized as string
        assert!(json.contains("\"100\""));
    }

    /// CORE-02: legacy JSON written before `reverse`/`created_at_ms` existed
    /// must keep deserializing (serde defaults) — old snapshots stay readable.
    #[test]
    fn test_edge_record_deserializes_legacy_json_without_new_fields() {
        let legacy = r#"{"target":"7","label":"knows","weight":1.0}"#;
        let edge: VantaEdgeRecord = serde_json::from_str(legacy).expect("legacy json parses");
        assert_eq!(edge.target, 7);
        assert_eq!(edge.label, "knows");
        assert!(!edge.reverse);
        assert_eq!(edge.created_at_ms, 0);
    }

    #[test]
    fn test_node_record_serialization_roundtrip() {
        let record = VantaNodeRecord {
            id: 42,
            fields: {
                let mut f = VantaFields::new();
                f.insert("key".into(), VantaValue::String("val".into()));
                f
            },
            vector: Some(vec![0.5, 0.5]),
            vector_dimensions: 2,
            edges: vec![VantaEdgeRecord {
                target: 1,
                label: "edge".into(),
                weight: 1.0,
                reverse: false,
                created_at_ms: 0,
            }],
            confidence_score: 0.8,
            importance: 0.3,
            hits: 5,
            last_accessed: 1000,
            epoch: 1,
            tier: VantaStorageTier::Hot,
            is_alive: true,
        };
        let json = serde_json::to_string(&record).unwrap();
        let deserialized: VantaNodeRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, record);
    }

    #[test]
    fn test_node_input_serialization_roundtrip() {
        let input = VantaNodeInput {
            id: 100,
            content: Some("hello".into()),
            vector: Some(vec![0.1, 0.2]),
            fields: {
                let mut f = VantaFields::new();
                f.insert("tag".into(), VantaValue::String("important".into()));
                f
            },
        };
        let json = serde_json::to_string(&input).unwrap();
        let deserialized: VantaNodeInput = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, input);
    }
}
