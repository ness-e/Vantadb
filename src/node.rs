use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::hash::{Hash, Hasher};
use web_time::{SystemTime, UNIX_EPOCH};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

/// Dynamic bitset supporting >128 bits for multi-tenant filtering.
///
/// Backed by `Vec<u64>` — grows on demand as bits are set. The sentinel
/// `all_set()` (single `u64::MAX` word) signals "match everything" for
/// use as an unbounded query mask in HNSW search paths.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FilterBitset(Vec<u64>);

impl FilterBitset {
    /// Create an empty bitset with no allocation.
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Pre-allocate capacity for `bits` entries.
    pub fn with_capacity(bits: usize) -> Self {
        let words = bits.div_ceil(64);
        Self(Vec::with_capacity(words))
    }

    /// Sentinel meaning "match everything" — used as a no-filter query mask.
    /// A single `u64::MAX` word signals unbounded matching in `matches_mask`.
    pub fn all_set() -> Self {
        Self(vec![u64::MAX])
    }

    /// Returns `true` if this is the all-set sentinel.
    pub fn is_all_set(&self) -> bool {
        self.0.len() == 1 && self.0[0] == u64::MAX
    }

    /// Returns `true` if no bits are set (empty bitset).
    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|&w| w == 0)
    }

    /// Number of u64 words backing this bitset.
    pub fn word_count(&self) -> usize {
        self.0.len()
    }

    /// Set bit at position `pos`.
    pub fn set_bit(&mut self, pos: usize) {
        let word = pos / 64;
        let bit = pos % 64;
        if word >= self.0.len() {
            self.0.resize(word + 1, 0);
        }
        self.0[word] |= 1u64 << bit;
    }

    /// Check if bit at position `pos` is set.
    pub fn has_bit(&self, pos: usize) -> bool {
        let word = pos / 64;
        let bit = pos % 64;
        word < self.0.len() && (self.0[word] & (1u64 << bit)) != 0
    }

    /// Check if ALL bits set in `mask` are also set in `self`.
    ///
    /// The all-set sentinel (produced by `FilterBitset::all_set()`) causes
    /// this method to return `true` unconditionally, acting as a no-filter.
    pub fn matches_mask(&self, mask: &FilterBitset) -> bool {
        if mask.is_all_set() {
            return true;
        }
        let min_len = self.0.len().min(mask.0.len());
        for i in 0..min_len {
            if (self.0[i] & mask.0[i]) != mask.0[i] {
                return false;
            }
        }
        // Any bits set in mask words beyond self's length can't be matched
        if self.0.len() < mask.0.len() {
            for &w in mask.0.iter().skip(self.0.len()) {
                if w != 0 {
                    return false;
                }
            }
        }
        true
    }

    /// Convert to a `u128`, truncating if the bitset exceeds 128 bits.
    pub fn to_u128(&self) -> u128 {
        let lo = self.0.first().copied().unwrap_or(0) as u128;
        let hi = self.0.get(1).copied().unwrap_or(0) as u128;
        lo | (hi << 64)
    }

    /// Create from a `u128` (legacy format — max 128 bits).
    pub fn from_u128(v: u128) -> Self {
        let lo = v as u64;
        let hi = (v >> 64) as u64;
        if hi == 0 {
            Self(vec![lo])
        } else {
            Self(vec![lo, hi])
        }
    }

    /// Serialize to length-prefixed bytes: `[word_count: u32 LE][words × u64 LE]`.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(4 + self.0.len() * 8);
        buf.extend_from_slice(&(self.0.len() as u32).to_le_bytes());
        for &w in &self.0 {
            buf.extend_from_slice(&w.to_le_bytes());
        }
        buf
    }

    /// Deserialize from length-prefixed bytes. Returns `(Self, bytes_consumed)`.
    pub fn from_bytes(data: &[u8]) -> std::io::Result<(Self, usize)> {
        use std::io::{Error, ErrorKind};
        if data.len() < 4 {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "FilterBitset: truncated length",
            ));
        }
        let word_count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        let needed = 4 + word_count * 8;
        if data.len() < needed {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "FilterBitset: truncated words",
            ));
        }
        let mut words = Vec::with_capacity(word_count);
        for i in 0..word_count {
            let off = 4 + i * 8;
            let w = u64::from_le_bytes([
                data[off],
                data[off + 1],
                data[off + 2],
                data[off + 3],
                data[off + 4],
                data[off + 5],
                data[off + 6],
                data[off + 7],
            ]);
            words.push(w);
        }
        Ok((Self(words), needed))
    }
}

impl Default for FilterBitset {
    fn default() -> Self {
        Self::new()
    }
}

impl From<u128> for FilterBitset {
    fn from(v: u128) -> Self {
        Self::from_u128(v)
    }
}

impl From<FilterBitset> for u128 {
    fn from(bs: FilterBitset) -> Self {
        bs.to_u128()
    }
}

/// Global sentinel for "match everything" (no filter) in HNSW queries.
pub static ALL_BITSET: std::sync::LazyLock<FilterBitset> =
    std::sync::LazyLock::new(FilterBitset::all_set);

/// Metric type used for vector distance/similarity calculations.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub enum DistanceMetric {
    /// Cosine similarity (default).
    #[default]
    Cosine,
    /// Euclidean distance.
    Euclidean,
}

// ─── Vector Data ───────────────────────────────────────────

/// Wrapper around a raw `*const f32` pointer that implements `Send` + `Sync`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SendPtr(pub *const f32);
unsafe impl Send for SendPtr {}
unsafe impl Sync for SendPtr {}

impl Default for SendPtr {
    fn default() -> Self {
        SendPtr(std::ptr::null())
    }
}

/// Vector storage — supports tiered precision (Hybrid Quantization)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum VectorRepresentations {
    /// L1: Fast binary index in RAM. Hamming distance (XOR + POPCNT).
    Binary(Box<[u64]>),
    /// L2: Re-ranking and initial validation. Memory-mapped from disk (3-bit).
    Turbo(Box<[u8]>),
    /// L2.5: 8-bit scalar quantization. Higher precision than Turbo, half the
    ///       memory of Full. Each dimension stored as `i8` scaled by `max_abs / 127`.
    SQ8(Box<[i8]>, f32),
    /// L3: Full precision float32.
    Full(Vec<f32>),
    /// L3 (MMap): Zero-copy view into the memory-mapped file
    MmapFull(#[serde(skip)] SendPtr, #[serde(skip)] usize),
    /// No vector attached
    None,
}

impl VectorRepresentations {
    /// Returns the number of dimensions in this vector representation.
    pub fn dimensions(&self) -> usize {
        match self {
            VectorRepresentations::Full(v) => v.len(),
            VectorRepresentations::MmapFull(_, len) => *len,
            VectorRepresentations::Binary(data) => data.len() * 64,
            VectorRepresentations::Turbo(data) => data.len() * 2,
            VectorRepresentations::SQ8(data, _) => data.len(),
            VectorRepresentations::None => 0,
        }
    }

    /// Returns `true` if this is the `None` variant.
    pub fn is_none(&self) -> bool {
        matches!(self, VectorRepresentations::None)
    }

    /// Decode to f32 for distance computation (Fallback/Testing)
    pub fn to_f32(&self) -> Option<Vec<f32>> {
        match self {
            VectorRepresentations::Full(v) => Some(v.clone()),
            VectorRepresentations::MmapFull(ptr, len) => {
                debug_assert!(!ptr.0.is_null(), "MmapFull pointer is null in to_f32");
                debug_assert!(*len > 0, "MmapFull len is zero in to_f32");
                let slice = unsafe { std::slice::from_raw_parts(ptr.0, *len) };
                Some(slice.to_vec())
            }
            VectorRepresentations::SQ8(data, scale) => {
                let inv = scale / 127.0;
                Some(data.iter().map(|&q| (q as f32) * inv).collect())
            }
            _ => None,
        }
    }

    /// Zero-copy borrow of the f32 vector data.
    /// Avoids heap allocation for distance computations on Full vectors.
    pub fn as_f32_slice(&self) -> Option<&[f32]> {
        match self {
            VectorRepresentations::Full(v) => Some(v.as_slice()),
            VectorRepresentations::MmapFull(ptr, len) => {
                debug_assert!(!ptr.0.is_null(), "MmapFull pointer is null in as_f32_slice");
                debug_assert!(*len > 0, "MmapFull len is zero in as_f32_slice");
                Some(unsafe { std::slice::from_raw_parts(ptr.0, *len) })
            }
            _ => None,
        }
    }

    /// Computes cosine similarity or quantized dot-product approximation.
    /// Uses zero-copy slice access to avoid heap allocations where possible.
    pub fn cosine_similarity(&self, other: &VectorRepresentations) -> Option<f32> {
        use crate::hardware::{HardwareCapabilities, InstructionSet};

        // SQ8 ↔ SQ8 fast path: avoid full decode
        if let (
            VectorRepresentations::SQ8(a_data, a_scale),
            VectorRepresentations::SQ8(b_data, b_scale),
        ) = (self, other)
        {
            let dot =
                crate::vector::quantization::sq8_similarity(a_data, *a_scale, b_data, *b_scale);
            return Some(dot);
        }

        let a = self.as_f32_slice()?;
        let b = other.as_f32_slice()?;
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
                if denom < f32::EPSILON {
                    None
                } else {
                    Some(dot / denom)
                }
            }
            _ => {
                let mut dot_v = wide::f32x8::ZERO;
                let mut norm_a_v = wide::f32x8::ZERO;
                let mut norm_b_v = wide::f32x8::ZERO;
                let chunks_a = a.chunks_exact(8);
                let chunks_b = b.chunks_exact(8);
                let rem_a = chunks_a.remainder();
                let rem_b = chunks_b.remainder();
                for (a_chunk, b_chunk) in chunks_a.zip(chunks_b) {
                    let va = wide::f32x8::from([
                        a_chunk[0], a_chunk[1], a_chunk[2], a_chunk[3], a_chunk[4], a_chunk[5],
                        a_chunk[6], a_chunk[7],
                    ]);
                    let vb = wide::f32x8::from([
                        b_chunk[0], b_chunk[1], b_chunk[2], b_chunk[3], b_chunk[4], b_chunk[5],
                        b_chunk[6], b_chunk[7],
                    ]);
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
                if denom < f32::EPSILON {
                    None
                } else {
                    Some(dot / denom)
                }
            }
        }
    }

    /// Estimated heap memory in bytes
    pub fn memory_size(&self) -> usize {
        match self {
            VectorRepresentations::Full(v) => v.len() * 4,
            VectorRepresentations::MmapFull(_, _) => 0, // Zero heap allocations for mapped memory
            VectorRepresentations::Binary(data) => data.len() * 8,
            VectorRepresentations::Turbo(data) => data.len(),
            VectorRepresentations::SQ8(data, _) => data.len() + 4,
            VectorRepresentations::None => 0,
        }
    }
}

// ─── Edge ──────────────────────────────────────────────────

/// Labeled directed edge with optional weight
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Edge {
    /// Target node ID.
    pub target: u64,
    /// Edge label string.
    pub label: String,
    /// Edge weight (defaults to 1.0).
    pub weight: f32,
}

/// Weights for computing per-node eviction scores.
/// Used by `StorageEngine::evict_cold_nodes()` to decide which nodes
/// to evict when under memory pressure.
#[derive(Debug, Clone, Copy)]
pub struct EvictionWeights {
    /// Weight for hit count.
    pub hits: f64,
    /// Weight for confidence score.
    pub confidence: f64,
    /// Weight for importance score.
    pub importance: f64,
    /// Weight for recency score.
    pub recency: f64,
}

impl Edge {
    /// Create an edge with default weight (1.0).
    pub fn new(target: u64, label: impl Into<String>) -> Self {
        Self {
            target,
            label: label.into(),
            weight: 1.0,
        }
    }

    /// Create an edge with a custom weight.
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
    /// A UTF-8 string value.
    String(String),
    /// A 64-bit signed integer value.
    Int(i64),
    /// A 64-bit floating point value.
    Float(f64),
    /// A boolean value.
    Bool(bool),
    /// A UTC date-time value.
    DateTime(chrono::DateTime<chrono::Utc>),
    /// A list of UTF-8 string values.
    ListString(Vec<String>),
    /// A list of 64-bit signed integer values.
    ListInt(Vec<i64>),
    /// A list of 64-bit floating point values.
    ListFloat(Vec<f64>),
    /// A list of boolean values.
    ListBool(Vec<bool>),
    /// A list of UTC date-time values.
    ListDateTime(Vec<chrono::DateTime<chrono::Utc>>),
    /// Absent / null value.
    Null,
}

impl Eq for FieldValue {}

impl Hash for FieldValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            FieldValue::String(s) => {
                0u8.hash(state);
                s.hash(state);
            }
            FieldValue::Int(i) => {
                1u8.hash(state);
                i.hash(state);
            }
            FieldValue::Float(f) => {
                2u8.hash(state);
                f.to_bits().hash(state);
            }
            FieldValue::Bool(b) => {
                3u8.hash(state);
                b.hash(state);
            }
            FieldValue::DateTime(dt) => {
                4u8.hash(state);
                dt.timestamp_nanos_opt().unwrap_or(0).hash(state);
            }
            FieldValue::ListString(v) => {
                5u8.hash(state);
                v.hash(state);
            }
            FieldValue::ListInt(v) => {
                6u8.hash(state);
                v.hash(state);
            }
            FieldValue::ListFloat(v) => {
                7u8.hash(state);
                for f in v {
                    f.to_bits().hash(state);
                }
            }
            FieldValue::ListBool(v) => {
                8u8.hash(state);
                v.hash(state);
            }
            FieldValue::ListDateTime(v) => {
                9u8.hash(state);
                for dt in v {
                    dt.timestamp_nanos_opt().unwrap_or(0).hash(state);
                }
            }
            FieldValue::Null => {
                10u8.hash(state);
            }
        }
    }
}

impl FieldValue {
    /// Returns the inner `&str` if this is a `String` variant.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            FieldValue::String(s) => Some(s),
            _ => None,
        }
    }
    /// Returns the inner `i64` if this is an `Int` variant.
    pub fn as_int(&self) -> Option<i64> {
        match self {
            FieldValue::Int(i) => Some(*i),
            _ => None,
        }
    }
    /// Returns the inner `bool` if this is a `Bool` variant.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            FieldValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Returns a list of string representations of the values.
    /// This is used for indexing and cardinality tracking.
    pub fn to_cardinality_keys(&self) -> Vec<String> {
        match self {
            FieldValue::String(s) => vec![s.clone()],
            FieldValue::Int(i) => vec![i.to_string()],
            FieldValue::Float(f) => vec![f.to_string()],
            FieldValue::Bool(b) => vec![b.to_string()],
            FieldValue::DateTime(dt) => vec![dt.to_rfc3339()],
            FieldValue::ListString(vec) => vec.clone(),
            FieldValue::ListInt(vec) => vec.iter().map(|i| i.to_string()).collect(),
            FieldValue::ListFloat(vec) => vec.iter().map(|f| f.to_string()).collect(),
            FieldValue::ListBool(vec) => vec.iter().map(|b| b.to_string()).collect(),
            FieldValue::ListDateTime(vec) => vec.iter().map(|dt| dt.to_rfc3339()).collect(),
            FieldValue::Null => vec!["null".to_string()],
        }
    }
}

/// Relational fields: ordered key-value map
pub type RelFields = BTreeMap<String, FieldValue>;

// ─── Node Flags ────────────────────────────────────────────

/// Bitfield flags stored in a `u32`, each bit representing a node state.
#[repr(transparent)]
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Serialize,
    Deserialize,
    PartialEq,
    IntoBytes,
    FromBytes,
    Immutable,
    KnownLayout,
)]
pub struct NodeFlags(pub u32);

impl NodeFlags {
    /// Node is active (alive).
    pub const ACTIVE: u32 = 1 << 0;
    /// Node is indexed.
    pub const INDEXED: u32 = 1 << 1;
    /// Node has been modified since last checkpoint.
    pub const DIRTY: u32 = 1 << 2;
    /// Node is marked as deleted (tombstone).
    pub const TOMBSTONE: u32 = 1 << 3;
    /// Node has associated vector data.
    pub const HAS_VECTOR: u32 = 1 << 4;
    /// Node has outgoing edges.
    pub const HAS_EDGES: u32 = 1 << 5;
    /// Node is pinned in memory (exempt from eviction).
    pub const PINNED: u32 = 1 << 6;
    /// Node was recovered from WAL replay.
    pub const RECOVERED: u32 = 1 << 7;
    /// Node has been invalidated.
    pub const INVALIDATED: u32 = 1 << 8;
    /// Node has had a conflict resolved.
    pub const CONFLICT_RESOLVED: u32 = 1 << 9;

    /// Create flags with the ACTIVE bit set.
    pub fn new() -> Self {
        Self(Self::ACTIVE)
    }
    /// Check if a specific flag is set.
    pub fn is_set(&self, flag: u32) -> bool {
        self.0 & flag != 0
    }
    /// Set a specific flag.
    pub fn set(&mut self, flag: u32) {
        self.0 |= flag;
    }
    /// Clear a specific flag.
    pub fn clear(&mut self, flag: u32) {
        self.0 &= !flag;
    }
    /// Returns `true` if the ACTIVE flag is set.
    pub fn is_active(&self) -> bool {
        self.is_set(Self::ACTIVE)
    }
    /// Returns `true` if the TOMBSTONE flag is set.
    pub fn is_tombstone(&self) -> bool {
        self.is_set(Self::TOMBSTONE)
    }
}

// ─── Node Tier ─────────────────────────────────────────────

/// Determines storage tier behavior
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
pub enum NodeTier {
    /// Fast volatile memory (RAM cache)
    Hot,
    /// Long-term persistent storage (disk)
    #[default]
    Cold,
}

/// Trait for tracking access patterns.
pub trait AccessTracker {
    /// Returns the confidence score (0.0–1.0).
    fn confidence_score(&self) -> f32;
    /// Returns the number of hits (access count).
    fn hits(&self) -> u32;
    /// Returns the last access time in Unix milliseconds.
    fn last_accessed(&self) -> u64;
    /// Pin the node in memory (exempt from eviction).
    fn pin(&mut self);
    /// Unpin the node, making it eligible for eviction.
    fn unpin(&mut self);
    /// Returns `true` if the node is pinned.
    fn is_pinned(&self) -> bool;
}

// ─── DiskNodeHeader (Zero-Copy) ────────────────────────────

/// Fixed-size header for zero-copy memory mapping.
/// Aligned to 64 bytes for optimal SIMD access and cache line boundary.
/// Uses raw u32 for flags/tier to avoid enums in #[repr(C)].
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug, PartialEq, IntoBytes, FromBytes, Immutable, KnownLayout)]
pub struct DiskNodeHeader {
    /// Globally unique identifier (Offset 0)
    pub id: u64,
    /// Offset 8
    pub confidence_score: f32,
    /// Offset 12
    pub importance: f32,
    /// 128-bit fast filter (Offset 16)
    pub bitset: u128,
    /// Offset to vector data in the MMap file (Offset 32)
    pub vector_offset: u64,
    /// Number of elements in the vector (Offset 40)
    pub vector_len: u32,
    /// Number of outgoing edges (Offset 44)
    pub edge_count: u16,
    /// Explicit padding to align relational_len (Offset 46)
    pub _pad1: [u8; 2],
    /// Length of the relational metadata block (Offset 48)
    pub relational_len: u32,
    /// Storage tier: Hot (0) or Cold (1) (Offset 52)
    pub tier: u8,
    /// Explicit gap padding for u32 field 'flags' alignment (Offset 53)
    pub _pad2: [u8; 3],
    /// Status flags (Offset 56)
    pub flags: u32,
    /// Explicit padding to reach exactly 64 bytes (Offset 60)
    pub _padding: [u8; 4],
}

impl DiskNodeHeader {
    /// Create a new header with default values for the given node ID.
    pub fn new(id: u64) -> Self {
        Self {
            id,
            confidence_score: 0.5,
            importance: 0.1,
            bitset: 0,
            vector_offset: 0,
            vector_len: 0,
            edge_count: 0,
            _pad1: [0; 2],
            relational_len: 0,
            tier: 0,
            _pad2: [0; 3],
            flags: 0,
            _padding: [0; 4],
        }
    }
}

/// Core multimodel node: vector + graph + relational unified.
///
/// Header (id+bitset+cluster+flags = 32B) is cache-friendly.
/// Heavy data (vector, edges, relational) lives on the heap.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UnifiedNode {
    /// Globally unique identifier
    pub id: u64,
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
    pub fn new(id: u64) -> Self {
        Self {
            id,
            bitset: FilterBitset::new(),
            semantic_cluster: 0,
            flags: NodeFlags::new(),
            vector: VectorRepresentations::None,
            epoch: 0,
            edges: Vec::new(),
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
}
