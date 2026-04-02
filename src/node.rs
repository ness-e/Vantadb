use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ─── Vector Data ───────────────────────────────────────────

/// Vector storage — supports tiered precision (Hot/Warm/None)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum VectorData {
    /// Full precision: hot data in RAM
    F32(Vec<f32>),
    /// Quantized: warm data, 4× smaller, ~95% recall
    I8 {
        data: Vec<i8>,
        scale: f32,
        offset: f32,
    },
    /// No vector attached to this node
    None,
}

impl VectorData {
    pub fn dimensions(&self) -> usize {
        match self {
            VectorData::F32(v) => v.len(),
            VectorData::I8 { data, .. } => data.len(),
            VectorData::None => 0,
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, VectorData::None)
    }

    /// Decode to f32 for distance computation
    pub fn to_f32(&self) -> Option<Vec<f32>> {
        match self {
            VectorData::F32(v) => Some(v.clone()),
            VectorData::I8 {
                data,
                scale,
                offset,
            } => Some(data.iter().map(|&x| x as f32 * scale + offset).collect()),
            VectorData::None => None,
        }
    }

    /// Cosine similarity between two vectors
    pub fn cosine_similarity(&self, other: &VectorData) -> Option<f32> {
        let a = self.to_f32()?;
        let b = other.to_f32()?;
        if a.len() != b.len() || a.is_empty() {
            return None;
        }
        let mut dot = 0.0f32;
        let mut norm_a = 0.0f32;
        let mut norm_b = 0.0f32;
        for i in 0..a.len() {
            dot += a[i] * b[i];
            norm_a += a[i] * a[i];
            norm_b += b[i] * b[i];
        }
        let denom = norm_a.sqrt() * norm_b.sqrt();
        if denom < f32::EPSILON {
            None
        } else {
            Some(dot / denom)
        }
    }

    /// Estimated heap memory in bytes
    pub fn memory_size(&self) -> usize {
        match self {
            VectorData::F32(v) => v.len() * 4,
            VectorData::I8 { data, .. } => data.len() + 8,
            VectorData::None => 0,
        }
    }
}

// ─── Edge ──────────────────────────────────────────────────

/// Labeled directed edge with optional weight
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Edge {
    pub target: u64,
    pub label: String,
    pub weight: f32,
}

impl Edge {
    pub fn new(target: u64, label: impl Into<String>) -> Self {
        Self {
            target,
            label: label.into(),
            weight: 1.0,
        }
    }

    pub fn with_weight(target: u64, label: impl Into<String>, weight: f32) -> Self {
        Self {
            target,
            label: label.into(),
            weight,
        }
    }
}

// ─── Field Value ───────────────────────────────────────────

/// Typed relational field value
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum FieldValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
}

impl FieldValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            FieldValue::String(s) => Some(s),
            _ => None,
        }
    }
    pub fn as_int(&self) -> Option<i64> {
        match self {
            FieldValue::Int(i) => Some(*i),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            FieldValue::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

/// Relational fields: ordered key-value map
/// Fase 2: migrate to Arrow RecordBatch for columnar access
pub type RelFields = BTreeMap<String, FieldValue>;

// ─── Node Flags ────────────────────────────────────────────

/// Bitfield metadata flags for node state
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct NodeFlags(pub u32);

impl NodeFlags {
    pub const ACTIVE: u32 = 1 << 0;
    pub const INDEXED: u32 = 1 << 1;
    pub const DIRTY: u32 = 1 << 2;
    pub const TOMBSTONE: u32 = 1 << 3;
    pub const HAS_VECTOR: u32 = 1 << 4;
    pub const HAS_EDGES: u32 = 1 << 5;

    pub fn new() -> Self {
        Self(Self::ACTIVE)
    }
    pub fn is_set(&self, flag: u32) -> bool {
        self.0 & flag != 0
    }
    pub fn set(&mut self, flag: u32) {
        self.0 |= flag;
    }
    pub fn clear(&mut self, flag: u32) {
        self.0 &= !flag;
    }
    pub fn is_active(&self) -> bool {
        self.is_set(Self::ACTIVE)
    }
    pub fn is_tombstone(&self) -> bool {
        self.is_set(Self::TOMBSTONE)
    }
}

// ─── UnifiedNode ───────────────────────────────────────────

/// Core multimodel node: vector + graph + relational unified.
///
/// Header (id+bitset+cluster+flags = 32B) is cache-friendly.
/// Heavy data (vector, edges, relational) lives on the heap.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnifiedNode {
    /// Globally unique identifier
    pub id: u64,
    /// 128-bit fast filter (country, role, active, etc.)
    pub bitset: u128,
    /// Semantic cluster for super-node routing
    pub semantic_cluster: u32,
    /// Status flags
    pub flags: NodeFlags,
    /// Vector embedding (FP32/I8/None)
    pub vector: VectorData,
    /// Outgoing graph edges
    pub edges: Vec<Edge>,
    /// Relational key-value fields
    pub relational: RelFields,
}

impl UnifiedNode {
    /// New empty node with given ID
    pub fn new(id: u64) -> Self {
        Self {
            id,
            bitset: 0,
            semantic_cluster: 0,
            flags: NodeFlags::new(),
            vector: VectorData::None,
            edges: Vec::new(),
            relational: BTreeMap::new(),
        }
    }

    /// New node with vector data
    pub fn with_vector(id: u64, vector: Vec<f32>) -> Self {
        let mut node = Self::new(id);
        node.vector = VectorData::F32(vector);
        node.flags.set(NodeFlags::HAS_VECTOR);
        node
    }

    /// Add a labeled edge
    pub fn add_edge(&mut self, target: u64, label: impl Into<String>) {
        self.edges.push(Edge::new(target, label));
        self.flags.set(NodeFlags::HAS_EDGES);
    }

    /// Add weighted edge
    pub fn add_weighted_edge(&mut self, target: u64, label: impl Into<String>, weight: f32) {
        self.edges.push(Edge::with_weight(target, label, weight));
        self.flags.set(NodeFlags::HAS_EDGES);
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
    pub fn set_bit(&mut self, pos: u8) {
        debug_assert!(pos < 128);
        self.bitset |= 1u128 << pos;
    }

    /// Check if bit is set
    pub fn has_bit(&self, pos: u8) -> bool {
        self.bitset & (1u128 << pos) != 0
    }

    /// Check if ALL bits in mask are set
    pub fn matches_mask(&self, mask: u128) -> bool {
        self.bitset & mask == mask
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
}

impl Default for UnifiedNode {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = UnifiedNode::new(42);
        assert_eq!(node.id, 42);
        assert!(node.is_alive());
        assert!(node.vector.is_none());
        assert!(node.edges.is_empty());
    }

    #[test]
    fn test_bitset_operations() {
        let mut node = UnifiedNode::new(1);
        node.set_bit(5); // country bit
        node.set_bit(16); // active bit

        assert!(node.has_bit(5));
        assert!(node.has_bit(16));
        assert!(!node.has_bit(7));

        let mask: u128 = (1 << 5) | (1 << 16);
        assert!(node.matches_mask(mask));
        assert!(!node.matches_mask(mask | (1 << 7)));
    }

    #[test]
    fn test_cosine_similarity() {
        let a = VectorData::F32(vec![1.0, 0.0, 0.0]);
        let b = VectorData::F32(vec![1.0, 0.0, 0.0]);
        let sim = a.cosine_similarity(&b).unwrap();
        assert!((sim - 1.0).abs() < 1e-6);

        let c = VectorData::F32(vec![0.0, 1.0, 0.0]);
        let sim2 = a.cosine_similarity(&c).unwrap();
        assert!(sim2.abs() < 1e-6); // orthogonal = 0
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
        node.set_field("pais", FieldValue::String("VZLA".into()));
        node.set_field("activo", FieldValue::Bool(true));

        assert_eq!(
            node.get_field("pais"),
            Some(&FieldValue::String("VZLA".into()))
        );
        assert_eq!(node.get_field("activo"), Some(&FieldValue::Bool(true)));
        assert_eq!(node.get_field("missing"), None);
    }
}
