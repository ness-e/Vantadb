use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use web_time::{SystemTime, UNIX_EPOCH};

use crate::node::{
    AccessTracker, Edge, EvictionWeights, FieldValue, FilterBitset, NodeFlags, NodeTier, RelFields,
    VectorRepresentations,
};

/// Core multimodel node: vector + graph + relational unified.
///
/// Header (id+bitset+cluster+flags = 32B) is cache-friendly.
/// Heavy data (vector, edges, relational) lives on the heap.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UnifiedNode {
    /// Globally unique identifier
    pub id: u128,
    /// Dynamic bitset for fast multi-tenant category filtering
    pub bitset: FilterBitset,
    /// Semantic cluster for super-node routing
    pub semantic_cluster: u32,
    /// Status flags
    pub flags: NodeFlags,
    /// Vector representations (tiered precision).
    pub vector: VectorRepresentations,
    /// Lineage version
    pub epoch: u32,
    /// Outgoing graph edges
    pub edges: Vec<Edge>,
    /// Secondary index: label_id → target node IDs for O(1) filtered traversal.
    #[serde(default)]
    pub label_index: HashMap<u32, Vec<u128>>,
    /// Relational key-value fields
    pub relational: RelFields,
    /// Storage tier: Hot (RAM) or Cold (disk)
    pub tier: NodeTier,
    /// Access frequency heuristic
    pub hits: u32,
    /// Recency heuristic (Unix MS)
    pub last_accessed: u64,
    /// Confidence score (0.0 - 1.0)
    pub confidence_score: f32,
    /// Importance score (0.0 - 1.0)
    pub importance: f32,
    /// Forward-compatible schema metadata without breaking Bincode
    pub ext_metadata: HashMap<String, Vec<u8>>,
}

impl AccessTracker for UnifiedNode {
    fn confidence_score(&self) -> f32 {
        self.confidence_score
    }
    fn hits(&self) -> u32 {
        self.hits
    }
    fn last_accessed(&self) -> u64 {
        self.last_accessed
    }
    fn pin(&mut self) {
        self.flags.set(NodeFlags::PINNED);
    }
    fn unpin(&mut self) {
        self.flags.clear(NodeFlags::PINNED);
    }
    fn is_pinned(&self) -> bool {
        self.flags.is_set(NodeFlags::PINNED)
    }
}

impl UnifiedNode {
    /// New empty node with given ID
    pub fn new(id: u128) -> Self {
        Self {
            id,
            bitset: FilterBitset::new(),
            semantic_cluster: 0,
            flags: NodeFlags::new(),
            vector: VectorRepresentations::None,
            epoch: 0,
            edges: Vec::new(),
            label_index: HashMap::new(),
            relational: BTreeMap::new(),
            tier: NodeTier::Cold,
            hits: 0,
            last_accessed: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            confidence_score: 0.5,
            importance: 0.1,
            ext_metadata: HashMap::new(),
        }
    }

    /// New node with vector data
    pub fn with_vector(id: u128, vector: Vec<f32>) -> Self {
        let mut node = Self::new(id);
        node.vector = VectorRepresentations::Full(vector);
        node.flags.set(NodeFlags::HAS_VECTOR);
        node
    }

    /// Add a labeled edge with an interned label_id.
    pub fn add_edge(&mut self, target: u128, label_id: u32) {
        self.edges.push(Edge::new(target, label_id));
        self.label_index.entry(label_id).or_default().push(target);
        self.flags.set(NodeFlags::HAS_EDGES);
    }

    /// Add weighted edge with an interned label_id.
    pub fn add_weighted_edge(&mut self, target: u128, label_id: u32, weight: f32) {
        self.edges.push(Edge::with_weight(target, label_id, weight));
        self.label_index.entry(label_id).or_default().push(target);
        self.flags.set(NodeFlags::HAS_EDGES);
    }

    /// Rebuild the label_index from edges (one-time O(n) build cost).
    /// Call after deserializing nodes that have edges but no index.
    pub fn rebuild_label_index(&mut self) {
        self.label_index.clear();
        for edge in &self.edges {
            self.label_index
                .entry(edge.label_id)
                .or_default()
                .push(edge.target);
        }
    }

    /// Returns targets for a specific label_id, or empty slice if none found.
    pub fn targets_by_label(&self, label_id: u32) -> &[u128] {
        self.label_index
            .get(&label_id)
            .map_or(&[], |v| v.as_slice())
    }

    /// Set relational field
    pub fn set_field(&mut self, key: impl Into<String>, value: FieldValue) {
        self.relational.insert(key.into(), value);
    }

    /// Get relational field
    pub fn get_field(&self, key: &str) -> Option<&FieldValue> {
        self.relational.get(key)
    }

    /// Set bit in filter bitset
    pub fn set_bit(&mut self, pos: usize) {
        self.bitset.set_bit(pos);
    }

    /// Check if bit is set
    pub fn has_bit(&self, pos: usize) -> bool {
        self.bitset.has_bit(pos)
    }

    /// Check if ALL bits in mask are set
    pub fn matches_mask(&self, mask: &FilterBitset) -> bool {
        self.bitset.matches_mask(mask)
    }

    /// Estimate total memory usage (bytes)
    pub fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.vector.memory_size()
            + self.edges.capacity() * std::mem::size_of::<Edge>()
            + self.relational.len() * 64 // rough BTreeMap node overhead
    }

    /// Mark as deleted (tombstone)
    pub fn mark_deleted(&mut self) {
        self.flags.clear(NodeFlags::ACTIVE);
        self.flags.set(NodeFlags::TOMBSTONE);
    }

    /// Is this node alive (active and not tombstoned)?
    pub fn is_alive(&self) -> bool {
        self.flags.is_active() && !self.flags.is_tombstone()
    }

    /// Compute a weighted eviction score for memory pressure decisions.
    /// Higher score = more valuable to keep in cache.
    pub fn eviction_score(&self, weights: &EvictionWeights) -> f64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let age_secs = if self.last_accessed > 0 {
            ((now - self.last_accessed) / 1000).max(1)
        } else {
            1
        };
        let recency_score = 1.0 / (age_secs as f64).ln_1p();
        self.hits as f64 * weights.hits
            + self.confidence_score as f64 * weights.confidence
            + self.importance as f64 * weights.importance
            + recency_score * weights.recency
    }
}

impl Default for UnifiedNode {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = UnifiedNode::new(42);
        assert_eq!(node.id, 42);
        assert!(node.is_alive());
        assert!(node.vector.is_none());
        assert_eq!(node.epoch, 0);
        assert!(node.edges.is_empty());
    }

    #[test]
    fn test_bitset_operations() {
        let mut node = UnifiedNode::new(1);
        node.set_bit(5);
        node.set_bit(16);

        assert!(node.has_bit(5));
        assert!(node.has_bit(16));
        assert!(!node.has_bit(7));

        let mut mask = FilterBitset::new();
        mask.set_bit(5);
        mask.set_bit(16);
        assert!(node.matches_mask(&mask));
        let mut bad_mask = mask.clone();
        bad_mask.set_bit(7);
        assert!(!node.matches_mask(&bad_mask));
    }

    #[test]
    fn test_tombstone() {
        let mut node = UnifiedNode::new(1);
        assert!(node.is_alive());
        node.mark_deleted();
        assert!(!node.is_alive());
    }

    #[test]
    fn test_relational_fields() {
        let mut node = UnifiedNode::new(1);
        node.set_field("country", FieldValue::String("US".into()));
        node.set_field("active", FieldValue::Bool(true));

        assert_eq!(
            node.get_field("country"),
            Some(&FieldValue::String("US".into()))
        );
        assert_eq!(node.get_field("active"), Some(&FieldValue::Bool(true)));
        assert_eq!(node.get_field("missing"), None);
    }

    #[test]
    fn test_unified_node_add_edge_sets_timestamp() {
        let mut node = UnifiedNode::new(1);
        node.add_edge(2, 0);
        assert!(node.edges[0].created_at_ms > 0);
        let mut node2 = UnifiedNode::new(1);
        node2.add_weighted_edge(3, 1, 2.5);
        assert!(node2.edges[0].created_at_ms > 0);
    }

    #[test]
    fn test_node_with_vector() {
        let node = UnifiedNode::with_vector(42, vec![1.0, 2.0, 3.0]);
        assert_eq!(node.id, 42);
        assert!(node.flags.is_set(NodeFlags::HAS_VECTOR));
        assert!(!node.vector.is_none());
        assert_eq!(node.vector.dimensions(), 3);
    }

    #[test]
    fn test_node_add_edge() {
        let mut node = UnifiedNode::new(1);
        node.add_edge(2, 0);
        assert_eq!(node.edges.len(), 1);
        assert!(node.flags.is_set(NodeFlags::HAS_EDGES));
        assert_eq!(node.edges[0].target, 2);
        assert_eq!(node.edges[0].label_id, 0);
    }

    #[test]
    fn test_node_add_weighted_edge() {
        let mut node = UnifiedNode::new(1);
        node.add_weighted_edge(3, 1, 2.5);
        assert_eq!(node.edges.len(), 1);
        assert_eq!(node.edges[0].weight, 2.5);
        assert_eq!(node.edges[0].target, 3);
    }

    #[test]
    fn test_node_memory_size() {
        let node = UnifiedNode::new(1);
        assert!(node.memory_size() >= std::mem::size_of::<UnifiedNode>());
        let mut node2 = UnifiedNode::with_vector(2, vec![0.0; 100]);
        node2.add_edge(3, 0);
        node2.set_field("key", FieldValue::String("val".into()));
        assert!(node2.memory_size() > node.memory_size());
    }

    #[test]
    fn test_node_eviction_score() {
        let weights = EvictionWeights {
            hits: 1.0,
            confidence: 1.0,
            importance: 1.0,
            recency: 1.0,
        };
        let node = UnifiedNode::new(1);
        let score = node.eviction_score(&weights);
        assert!(score.is_finite());
        assert!(score > 0.0);
        let mut node2 = UnifiedNode::new(2);
        node2.hits = 100;
        assert!(node2.eviction_score(&weights) > score);
        let zero = EvictionWeights {
            hits: 0.0,
            confidence: 0.0,
            importance: 0.0,
            recency: 0.0,
        };
        assert!((node.eviction_score(&zero) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_node_pin_unpin() {
        let mut node = UnifiedNode::new(1);
        assert!(!node.is_pinned());
        node.pin();
        assert!(node.is_pinned());
        assert!(node.flags.is_set(NodeFlags::PINNED));
        node.unpin();
        assert!(!node.is_pinned());
    }

    #[test]
    fn test_node_default() {
        let node: UnifiedNode = Default::default();
        assert_eq!(node.id, 0);
        assert!(node.is_alive());
        assert_eq!(node.epoch, 0);
    }

    #[test]
    fn test_node_matches_mask_edge_cases() {
        let mut node = UnifiedNode::new(1);
        assert!(node.matches_mask(&FilterBitset::new()));
        node.set_bit(10);
        assert!(node.matches_mask(&FilterBitset::all_set()));
        let mut mask = FilterBitset::new();
        mask.set_bit(10);
        assert!(node.matches_mask(&mask));
        let mut mask2 = FilterBitset::new();
        mask2.set_bit(200);
        assert!(!node.matches_mask(&mask2));
    }

    #[test]
    fn test_node_access_tracker() {
        let mut node = UnifiedNode::new(1);
        assert_eq!(node.hits(), 0);
        assert_eq!(node.confidence_score(), 0.5);
        assert!(node.last_accessed() > 0);
        node.pin();
        assert!(node.is_pinned());
        node.unpin();
        assert!(!node.is_pinned());
    }

    #[test]
    fn test_node_fields_override() {
        let mut node = UnifiedNode::new(1);
        node.set_field("key", FieldValue::Int(1));
        assert_eq!(node.get_field("key"), Some(&FieldValue::Int(1)));
        node.set_field("key", FieldValue::Int(2));
        assert_eq!(node.get_field("key"), Some(&FieldValue::Int(2)));
    }

    #[test]
    fn test_node_has_vector_flag() {
        assert!(!UnifiedNode::new(1).flags.is_set(NodeFlags::HAS_VECTOR));
        assert!(UnifiedNode::with_vector(2, vec![1.0, 2.0])
            .flags
            .is_set(NodeFlags::HAS_VECTOR));
    }

    #[test]
    fn test_eviction_score_recency() {
        let weights = EvictionWeights {
            hits: 0.0,
            confidence: 0.0,
            importance: 0.0,
            recency: 1.0,
        };
        let score = UnifiedNode::new(1).eviction_score(&weights);
        let expected = 1.0 / (2.0f64).ln();
        assert!((score - expected).abs() < 1e-6);
    }
}
