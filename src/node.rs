use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::time::{SystemTime, UNIX_EPOCH};

// ─── Vector Data ───────────────────────────────────────────

/// Vector storage — supports tiered precision (Phase 31: Hybrid Quantization)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum VectorRepresentations {
    /// L1: Índice Rápido en RAM. Distancia de Hamming (XOR + POPCNT).
    Binary(Box<[u64]>),
    /// L2: Re-ranking y Validación inicial. Mapeado a memoria desde disco (3-bit).
    Turbo(Box<[u8]>),
    /// L3: Arqueología Semántica y Resolución de Pánico. Precisión absoluta.
    Full(Vec<f32>),
    /// No vector attached
    None,
}

impl VectorRepresentations {
    pub fn dimensions(&self) -> usize {
        match self {
            VectorRepresentations::Full(v) => v.len(),
            VectorRepresentations::Binary(data) => data.len() * 64, // rough dim
            VectorRepresentations::Turbo(data) => data.len() * 2, // depends on PolarQuant packing
            VectorRepresentations::None => 0,
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, VectorRepresentations::None)
    }

    /// Decode to f32 for distance computation (Fallback/Testing)
    pub fn to_f32(&self) -> Option<Vec<f32>> {
        match self {
            VectorRepresentations::Full(v) => Some(v.clone()),
            _ => None, // Only full supports exact to_f32 without decomp
        }
    }

    /// Computes cosine similarity (F32) or delegates to Hamming/PolarQuant logic later
    pub fn cosine_similarity(&self, other: &VectorRepresentations) -> Option<f32> {
        use crate::hardware::{HardwareCapabilities, InstructionSet};

        let a = self.to_f32()?;
        let b = other.to_f32()?;
        if a.len() != b.len() || a.is_empty() {
            return None;
        }

        let caps = HardwareCapabilities::global();
        match caps.instructions {
            InstructionSet::Fallback => {
                let mut dot: f32 = 0.0;
                let mut norm_a: f32 = 0.0;
                let mut norm_b: f32 = 0.0;
                for (va, vb) in a.iter().zip(b.iter()) {
                    dot += va * vb;
                    norm_a += va * va;
                    norm_b += vb * vb;
                }
                let denom = norm_a.sqrt() * norm_b.sqrt();
                if denom < f32::EPSILON { None } else { Some(dot / denom) }
            },
            _ => {
                let mut dot_v = wide::f32x8::ZERO;
                let mut norm_a_v = wide::f32x8::ZERO;
                let mut norm_b_v = wide::f32x8::ZERO;
                let chunks_a = a.chunks_exact(8);
                let chunks_b = b.chunks_exact(8);
                let rem_a = chunks_a.remainder();
                let rem_b = chunks_b.remainder();
                for (a_chunk, b_chunk) in chunks_a.zip(chunks_b) {
                    let va = wide::f32x8::from([a_chunk[0], a_chunk[1], a_chunk[2], a_chunk[3], a_chunk[4], a_chunk[5], a_chunk[6], a_chunk[7]]);
                    let vb = wide::f32x8::from([b_chunk[0], b_chunk[1], b_chunk[2], b_chunk[3], b_chunk[4], b_chunk[5], b_chunk[6], b_chunk[7]]);
                    dot_v += va * vb;
                    norm_a_v += va * va;
                    norm_b_v += vb * vb;
                }
                let mut dot = dot_v.reduce_add();
                let mut norm_a = norm_a_v.reduce_add();
                let mut norm_b = norm_b_v.reduce_add();
                for i in 0..rem_a.len() {
                    dot += rem_a[i] * rem_b[i];
                    norm_a += rem_a[i] * rem_a[i];
                    norm_b += rem_b[i] * rem_b[i];
                }
                let denom = norm_a.sqrt() * norm_b.sqrt();
                if denom < f32::EPSILON { None } else { Some(dot / denom) }
            }
        }
    }

    /// Estimated heap memory in bytes
    pub fn memory_size(&self) -> usize {
        match self {
            VectorRepresentations::Full(v) => v.len() * 4,
            VectorRepresentations::Binary(data) => data.len() * 8,
            VectorRepresentations::Turbo(data) => data.len(),
            VectorRepresentations::None => 0,
        }
    }
}

// ─── Edge ──────────────────────────────────────────────────

/// Labeled directed edge with optional weight
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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
    pub const PINNED: u32 = 1 << 6;
    pub const REHYDRATED: u32 = 1 << 7;
    pub const HALLUCINATION: u32 = 1 << 8; // Phase 31: Previous invalid state

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

// ─── Cognitive Architecture ────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum NeuronType {
    STNeuron, // Fast volatile memory
    LTNeuron, // Long-term persistent graph memory
}

pub trait CognitiveUnit {
    fn trust_score(&self) -> f32;
    fn hits(&self) -> u32;
    fn last_accessed(&self) -> u64; // Unix ms
    fn pin(&mut self);
    fn unpin(&mut self);
    fn is_pinned(&self) -> bool;
}

// ─── UnifiedNode ───────────────────────────────────────────

/// Core multimodel node: vector + graph + relational unified.
///
/// Header (id+bitset+cluster+flags = 32B) is cache-friendly.
/// Heavy data (vector, edges, relational) lives on the heap.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UnifiedNode {
    /// Globally unique identifier
    pub id: u64,
    /// 128-bit fast filter (country, role, active, etc.)
    pub bitset: u128,
    /// Semantic cluster for super-node routing
    pub semantic_cluster: u32,
    /// Status flags
    pub flags: NodeFlags,
    pub vector: VectorRepresentations,
    /// Epoch semantic lineage version (Phase 31)
    pub epoch: u32,
    /// Outgoing graph edges
    pub edges: Vec<Edge>,
    /// Relational key-value fields
    pub relational: RelFields,
    /// Fast volatile vs long-term persistent behavior
    pub neuron_type: NeuronType,
    /// Access frequency heuristic
    pub hits: u32,
    /// Recency heuristic (Unix MS)
    pub last_accessed: u64,
    /// Static Bayesian logic confidence
    pub trust_score: f32,
    /// Biological Amygdala limit: emotional/semantic importance (0.0 - 1.0)
    pub semantic_valence: f32,
    /// Forward-compatible schema metadata without breaking Bincode
    pub ext_metadata: HashMap<String, Vec<u8>>,
}

impl CognitiveUnit for UnifiedNode {
    fn trust_score(&self) -> f32 { self.trust_score }
    fn hits(&self) -> u32 { self.hits }
    fn last_accessed(&self) -> u64 { self.last_accessed }
    fn pin(&mut self) { self.flags.set(NodeFlags::PINNED); }
    fn unpin(&mut self) { self.flags.clear(NodeFlags::PINNED); }
    fn is_pinned(&self) -> bool { self.flags.is_set(NodeFlags::PINNED) }
}

impl UnifiedNode {
    /// New empty node with given ID
    pub fn new(id: u64) -> Self {
        Self {
            id,
            bitset: 0,
            semantic_cluster: 0,
            flags: NodeFlags::new(),
            vector: VectorRepresentations::None,
            epoch: 0,
            edges: Vec::new(),
            relational: BTreeMap::new(),
            neuron_type: NeuronType::LTNeuron,
            hits: 0,
            last_accessed: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            trust_score: 0.5,
            semantic_valence: 0.0,
            ext_metadata: HashMap::new(),
        }
    }

    /// New node with vector data
    pub fn with_vector(id: u64, vector: Vec<f32>) -> Self {
        let mut node = Self::new(id);
        node.vector = VectorRepresentations::Full(vector);
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
        assert_eq!(node.epoch, 0);
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

    // Removed outdated cosine_similarity tests since they moved to quantization / index modules.

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

// ── ConnectomeDB Biological Nomenclature (Type Aliases) ──────────
//
// These aliases allow users to choose between traditional database terms
// and the biologically-inspired ConnectomeDB vocabulary.
// Both names compile identically — the struct definitions remain unchanged.

/// A **Neuron** is the fundamental cognitive unit of ConnectomeDB.
/// Technically identical to `UnifiedNode` — the unified multimodel data structure
/// containing relational fields, graph edges, and vector embeddings.
pub type Neuron = UnifiedNode;

/// A **Synapse** is a weighted, directed connection between two Neurons.
/// Technically identical to `Edge`. The name `Edge` is retained as the primary
/// identifier for compatibility with the Rust graph ecosystem.
pub type Synapse = Edge;
