#[cfg(feature = "roaring")]
use croaring::{Bitmap, Portable};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use web_time::{SystemTime, UNIX_EPOCH};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::storage::vfile::Mmap;

const MAX_VEC_F32_LEN: usize = 10_000_000; // Max ~40MB for a single f32 vector

/// Dynamic bitset supporting >128 bits for multi-tenant filtering.
///
/// Backed by `croaring::Bitmap` (RoaringBitmap) — a compressed bitset that
/// is efficient for both sparse and dense sets. The sentinel `all_set()`
/// uses an internal `all_set` flag to signal "match everything" without
/// allocating a full u32::MAX bitmap.
///
/// Serialization (serde) uses croaring's Portable format wrapped in a
/// `(all_set: bool, bytes: Vec<u8>)` tuple.
#[cfg(feature = "roaring")]
#[derive(Clone, Debug)]
pub struct FilterBitset {
    inner: Bitmap,
    all_set: bool,
}

/// Dynamic bitset supporting >128 bits for multi-tenant filtering.
///
/// Pure-Rust `Vec<u64>` fallback used when croaring's C FFI is unavailable
/// (e.g. `wasm32-unknown-unknown`). The all-set sentinel is a single
/// `u64::MAX` word.
#[cfg(not(feature = "roaring"))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FilterBitset(Vec<u64>);

#[cfg(feature = "roaring")]
impl FilterBitset {
    /// Create an empty bitset.
    pub fn new() -> Self {
        Self {
            inner: Bitmap::new(),
            all_set: false,
        }
    }

    /// Pre-allocate capacity for `bits` entries (hint only).
    pub fn with_capacity(bits: usize) -> Self {
        let containers = bits.div_ceil(1 << 16) as u32;
        Self {
            inner: Bitmap::with_container_capacity(containers),
            all_set: false,
        }
    }

    /// Sentinel meaning "match everything" — used as a no-filter query mask.
    /// Uses an internal flag rather than allocating a full bitmap of all u32 values.
    pub fn all_set() -> Self {
        Self {
            inner: Bitmap::new(),
            all_set: true,
        }
    }

    /// Returns `true` if this is the all-set sentinel.
    pub fn is_all_set(&self) -> bool {
        self.all_set
    }

    /// Returns `true` if no bits are set (empty bitset, non-all-set).
    pub fn is_empty(&self) -> bool {
        !self.all_set && self.inner.is_empty()
    }

    /// Number of u64 words backing this bitset (deprecated — always 0 for RoaringBitmap).
    #[allow(dead_code)]
    pub fn word_count(&self) -> usize {
        0
    }

    /// Set bit at position `pos`.
    pub fn set_bit(&mut self, pos: usize) {
        self.inner.add(pos as u32);
    }

    /// Check if bit at position `pos` is set.
    pub fn has_bit(&self, pos: usize) -> bool {
        self.inner.contains(pos as u32)
    }

    /// Check if ALL bits set in `mask` are also set in `self`.
    ///
    /// The all-set sentinel (produced by `FilterBitset::all_set()`) causes
    /// this method to return `true` unconditionally, acting as a no-filter.
    pub fn matches_mask(&self, mask: &FilterBitset) -> bool {
        if mask.all_set {
            return true;
        }
        // If `self` is all_set, it matches every mask.
        if self.all_set {
            return true;
        }
        // All bits in mask must be present in self → mask is subset of self
        mask.inner.is_subset(&self.inner)
    }

    /// Convert to a `u128`, reading the first 128 bits of the bitmap.
    pub fn to_u128(&self) -> u128 {
        if self.all_set {
            return !0;
        }
        let mut result: u128 = 0;
        for i in 0..128 {
            if self.inner.contains(i as u32) {
                result |= 1u128 << i;
            }
        }
        result
    }

    /// Create from a `u128` (legacy format — max 128 bits).
    pub fn from_u128(v: u128) -> Self {
        let mut bm = Bitmap::new();
        for i in 0..128 {
            if (v & (1u128 << i)) != 0 {
                bm.add(i as u32);
            }
        }
        Self {
            inner: bm,
            all_set: false,
        }
    }

    /// Serialize to croaring Portable format bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.inner.serialize::<Portable>()
    }

    /// Deserialize from croaring Portable format bytes. Returns `(Self, bytes_consumed)`.
    pub fn from_bytes(data: &[u8]) -> std::io::Result<(Self, usize)> {
        let bitmap = Bitmap::try_deserialize::<Portable>(data).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "FilterBitset: invalid croaring bitmap data",
            )
        })?;
        let consumed = bitmap.get_serialized_size_in_bytes::<Portable>();
        Ok((
            Self {
                inner: bitmap,
                all_set: false,
            },
            consumed,
        ))
    }
}

#[cfg(not(feature = "roaring"))]
impl FilterBitset {
    /// Create an empty bitset.
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
        // Match croaring's u32 index space (node_ids are 128-bit hashes).
        let pos = pos as u32 as usize;
        let word = pos / 64;
        let bit = pos % 64;
        if word >= self.0.len() {
            self.0.resize(word + 1, 0);
        }
        self.0[word] |= 1u64 << bit;
    }

    /// Check if bit at position `pos` is set.
    pub fn has_bit(&self, pos: usize) -> bool {
        let pos = pos as u32 as usize;
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

    /// Maximum sane number of filter words (256K bits = 32KB).
    const MAX_WORDS: usize = 4096;

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
        if word_count > Self::MAX_WORDS {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "FilterBitset: word_count exceeds maximum",
            ));
        }
        let needed = word_count
            .checked_mul(8)
            .and_then(|v| v.checked_add(4))
            .ok_or_else(|| {
                Error::new(ErrorKind::InvalidData, "FilterBitset: word_count overflow")
            })?;
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

#[cfg(feature = "roaring")]
impl PartialEq for FilterBitset {
    fn eq(&self, other: &Self) -> bool {
        if self.all_set != other.all_set {
            return false;
        }
        if self.all_set {
            return true; // both are all_set
        }
        self.inner == other.inner
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

#[cfg(feature = "roaring")]
mod filter_bitset_serde {
    use super::*;

    impl Serialize for FilterBitset {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let bytes = self.inner.serialize::<Portable>();
            (self.all_set, bytes).serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for FilterBitset {
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let (all_set, bytes): (bool, Vec<u8>) = Deserialize::deserialize(deserializer)?;
            if all_set {
                return Ok(Self {
                    inner: Bitmap::new(),
                    all_set: true,
                });
            }
            let inner = if bytes.is_empty() {
                Bitmap::new()
            } else {
                Bitmap::try_deserialize::<Portable>(&bytes).ok_or_else(|| {
                    serde::de::Error::custom("FilterBitset: invalid croaring bitmap data")
                })?
            };
            Ok(Self {
                inner,
                all_set: false,
            })
        }
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

/// Vector storage — supports tiered precision (Hybrid Quantization)
#[derive(Clone, Debug, Serialize, Deserialize)]
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
    /// L3 (MMap): Zero-copy view into the memory-mapped file.
    /// The `Arc<Mmap>` keeps the mapping alive, preventing dangling pointers
    /// when the index file is re-mapped during checkpoint/sync.
    MmapFull(#[serde(skip)] Option<Arc<Mmap>>),
    /// No vector attached
    None,
}

// Manual PartialEq: Arc<Mmap> doesn't implement PartialEq, so we
// compare MmapFull by content (same pointer + len is sufficient for tests).
impl PartialEq for VectorRepresentations {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Full(a), Self::Full(b)) => a == b,
            (Self::MmapFull(a), Self::MmapFull(b)) => match (a, b) {
                (Some(am), Some(bm)) => am.as_ptr() == bm.as_ptr() && am.len() == bm.len(),
                (None, None) => true,
                _ => false,
            },
            (Self::Binary(a), Self::Binary(b)) => a == b,
            (Self::Turbo(a), Self::Turbo(b)) => a == b,
            (Self::SQ8(a, sa), Self::SQ8(b, sb)) => a == b && (sa - sb).abs() < f32::EPSILON,
            (Self::None, Self::None) => true,
            _ => false,
        }
    }
}

impl VectorRepresentations {
    /// Returns the number of dimensions in this vector representation.
    pub fn dimensions(&self) -> usize {
        match self {
            VectorRepresentations::Full(v) => v.len(),
            VectorRepresentations::MmapFull(mmap_opt) => {
                mmap_opt.as_ref().map_or(0, |m| m.len() / 4)
            }
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
            VectorRepresentations::MmapFull(_) => {
                let slice = self.as_f32_slice()?;
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
            VectorRepresentations::MmapFull(mmap_opt) => {
                let mmap = mmap_opt.as_ref()?;
                let len = mmap.len() / 4;
                if len == 0 || len > MAX_VEC_F32_LEN {
                    return None;
                }
                // SAFETY: the mmap is kept alive by Arc; length bounds checked above.
                Some(unsafe { std::slice::from_raw_parts(mmap.as_ptr() as *const f32, len) })
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
            VectorRepresentations::MmapFull(_) => 0, // Zero heap allocations for mapped memory
            VectorRepresentations::Binary(data) => data.len() * 8,
            VectorRepresentations::Turbo(data) => data.len(),
            VectorRepresentations::SQ8(data, _) => data.len() + 4,
            VectorRepresentations::None => 0,
        }
    }
}

// ─── Label Intern ──────────────────────────────────────────

/// Bidirectional map: String ↔ u32 for edge labels.
/// Cardinalidad típica: decenas/cientos, no miles. HashMap alcanza.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct LabelIntern {
    map: HashMap<String, u32>,
    strings: Vec<String>,
}

impl LabelIntern {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            strings: Vec::new(),
        }
    }

    /// Get or create a label ID.
    pub fn intern(&mut self, label: &str) -> u32 {
        if let Some(&id) = self.map.get(label) {
            return id;
        }
        let id = self.strings.len() as u32;
        self.map.insert(label.to_string(), id);
        self.strings.push(label.to_string());
        id
    }

    /// Resolve an ID back to a label string.
    pub fn resolve(&self, id: u32) -> Option<&str> {
        self.strings.get(id as usize).map(|s| s.as_str())
    }

    /// Look up a label string without creating a new ID.
    pub fn lookup(&self, label: &str) -> Option<u32> {
        self.map.get(label).copied()
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.strings.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }
}

impl Default for LabelIntern {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Edge ──────────────────────────────────────────────────

/// Labeled directed edge with optional weight and reverse flag.
///
/// Label stored as `label_id: u32` referencing a `LabelIntern` map.
/// Saves ~20-28 bytes per edge vs storing a `String` inline.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Edge {
    /// Target node ID.
    pub target: u128,
    /// Interned edge label (use LabelIntern to resolve).
    pub label_id: u32,
    /// Edge weight (defaults to 1.0).
    pub weight: f32,
    /// Whether this is a reverse edge.
    #[serde(default)]
    pub reverse: bool,
    /// Unix epoch milliseconds when this edge was created.
    /// Postcard records written before this field existed end the buffer
    /// here; the manual `Deserialize` below reads it as `0` for them.
    pub created_at_ms: u64,
}

/// Manual `Deserialize` for [`Edge`].
///
/// postcard's `deserialize_struct` → `deserialize_tuple(fields.len())` fixes
/// the element count to the *current* struct shape (5 fields). When reading a
/// legacy record that predates `created_at_ms` (4 fields), postcard's
/// `SeqAccess::next_element_seed` returns `Err(DeserializeUnexpectedEnd)`
/// instead of `Ok(None)` once the buffer is exhausted, so `#[serde(default)]`
/// is never consulted. We therefore default the trailing field to `0`.
impl<'de> Deserialize<'de> for Edge {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct EdgeVisitor;

        impl<'de> serde::de::Visitor<'de> for EdgeVisitor {
            type Value = Edge;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("struct Edge")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let target = seq.next_element()?.unwrap_or_default();
                let label_id = seq.next_element()?.unwrap_or_default();
                let weight = seq.next_element()?.unwrap_or_default();
                let reverse = seq.next_element()?.unwrap_or_default();
                // Legacy records end here; postcard errors instead of `None`,
                // so swallow that and default to 0.
                let created_at_ms = seq.next_element::<u64>().ok().flatten().unwrap_or(0);
                Ok(Edge {
                    target,
                    label_id,
                    weight,
                    reverse,
                    created_at_ms,
                })
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut target = None;
                let mut label_id = None;
                let mut weight = None;
                let mut reverse = None;
                let mut created_at_ms = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "target" => target = Some(map.next_value()?),
                        "label_id" => label_id = Some(map.next_value()?),
                        "weight" => weight = Some(map.next_value()?),
                        "reverse" => reverse = Some(map.next_value()?),
                        "created_at_ms" => created_at_ms = Some(map.next_value()?),
                        _ => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }
                Ok(Edge {
                    target: target.unwrap_or_default(),
                    label_id: label_id.unwrap_or_default(),
                    weight: weight.unwrap_or_default(),
                    reverse: reverse.unwrap_or_default(),
                    created_at_ms: created_at_ms.unwrap_or(0),
                })
            }
        }

        deserializer.deserialize_struct(
            "Edge",
            &["target", "label_id", "weight", "reverse", "created_at_ms"],
            EdgeVisitor,
        )
    }
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
    /// Create an edge with default weight (1.0) and `reverse: false`.
    pub fn new(target: u128, label_id: u32) -> Self {
        Self {
            target,
            label_id,
            weight: 1.0,
            reverse: false,
            created_at_ms: edge_created_at_now(),
        }
    }

    /// Create an edge with a custom weight.
    pub fn with_weight(target: u128, label_id: u32, weight: f32) -> Self {
        Self {
            target,
            label_id,
            weight,
            reverse: false,
            created_at_ms: edge_created_at_now(),
        }
    }

    /// Create a reverse edge (used for bidirectional traversal).
    pub fn reverse(target: u128, label_id: u32) -> Self {
        Self {
            target,
            label_id,
            weight: 1.0,
            reverse: true,
            created_at_ms: edge_created_at_now(),
        }
    }

    /// Create a forward edge with an explicit creation timestamp (Unix ms).
    pub fn with_timestamp(target: u128, label_id: u32, created_at_ms: u64) -> Self {
        Self {
            target,
            label_id,
            weight: 1.0,
            reverse: false,
            created_at_ms,
        }
    }
}

/// Current Unix epoch time in milliseconds, used as the default edge creation
/// timestamp. Falls back to `0` if the system clock predates the epoch.
fn edge_created_at_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
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
///
/// Fields ordered to eliminate internal padding: both u128 fields first,
/// then u64, group of u32, u16, u8, then final pad to exactly 64 bytes.
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug, PartialEq, IntoBytes, FromBytes, Immutable, KnownLayout)]
pub struct DiskNodeHeader {
    /// Globally unique identifier (Offset 0)
    pub id: u128,
    /// 128-bit fast filter (Offset 16)
    pub bitset: u128,
    /// Offset to vector data in the MMap file (Offset 32)
    pub vector_offset: u64,
    /// Confidence score (Offset 40)
    pub confidence_score: f32,
    /// Importance score (Offset 44)
    pub importance: f32,
    /// Length of the relational metadata block (Offset 48)
    pub relational_len: u32,
    /// Number of elements in the vector (Offset 52)
    pub vector_len: u32,
    /// Status flags (Offset 56)
    pub flags: u32,
    /// Number of outgoing edges (Offset 60)
    pub edge_count: u16,
    /// Storage tier: Hot (0) or Cold (1) (Offset 62)
    pub tier: u8,
    /// Explicit padding to reach exactly 64 bytes (Offset 63)
    pub _pad: [u8; 1],
}

impl DiskNodeHeader {
    /// Create a new header with default values for the given node ID.
    pub fn new(id: u128) -> Self {
        Self {
            id,
            bitset: 0,
            vector_offset: 0,
            confidence_score: 0.5,
            importance: 0.1,
            relational_len: 0,
            vector_len: 0,
            flags: 0,
            edge_count: 0,
            tier: 0,
            _pad: [0; 1],
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

    // ── FilterBitset comprehensive tests ──

    #[test]
    fn test_filter_bitset_with_capacity() {
        let bs = FilterBitset::with_capacity(128);
        assert!(bs.is_empty());
        let mut bs = FilterBitset::with_capacity(128);
        bs.set_bit(0);
        assert!(bs.has_bit(0));
        bs.set_bit(64);
        assert!(bs.has_bit(64));
    }

    #[test]
    fn test_filter_bitset_all_set() {
        let all = FilterBitset::all_set();
        assert!(all.is_all_set());
        assert!(!all.is_empty());
    }

    #[test]
    fn test_filter_bitset_is_empty() {
        assert!(FilterBitset::new().is_empty());
        let mut bs = FilterBitset::new();
        bs.set_bit(0);
        assert!(!bs.is_empty());
    }

    #[cfg(feature = "roaring")]
    #[test]
    fn test_filter_bitset_word_count_deprecated() {
        // word_count is deprecated for croaring — always returns 0
        assert_eq!(FilterBitset::new().word_count(), 0);
        let mut bs = FilterBitset::new();
        bs.set_bit(63);
        assert_eq!(bs.word_count(), 0);
        bs.set_bit(64);
        assert_eq!(bs.word_count(), 0);
    }

    #[cfg(not(feature = "roaring"))]
    #[test]
    fn test_filter_bitset_word_count() {
        assert_eq!(FilterBitset::new().word_count(), 0);
        let mut bs = FilterBitset::new();
        bs.set_bit(63);
        assert_eq!(bs.word_count(), 1);
        bs.set_bit(64);
        assert_eq!(bs.word_count(), 2);
    }

    #[test]
    fn test_filter_bitset_set_bit_growth() {
        let mut bs = FilterBitset::new();
        bs.set_bit(0);
        assert!(bs.has_bit(0));

        bs.set_bit(64);
        assert!(bs.has_bit(64));

        bs.set_bit(128);
        assert!(bs.has_bit(128));
    }

    #[test]
    fn test_filter_bitset_has_bit_out_of_range() {
        assert!(!FilterBitset::new().has_bit(0));
        assert!(!FilterBitset::new().has_bit(1000));
    }

    #[test]
    fn test_filter_bitset_matches_mask_all_set() {
        let mut bs = FilterBitset::new();
        bs.set_bit(5);
        assert!(bs.matches_mask(&FilterBitset::all_set()));
    }

    #[test]
    fn test_filter_bitset_matches_mask_self_longer() {
        let mut bs = FilterBitset::new();
        bs.set_bit(0);
        bs.set_bit(100);
        let mut mask = FilterBitset::new();
        mask.set_bit(0);
        assert!(bs.matches_mask(&mask));
    }

    #[test]
    fn test_filter_bitset_matches_mask_mask_longer_nonzero() {
        let mut bs = FilterBitset::new();
        bs.set_bit(0);
        let mut mask = FilterBitset::new();
        mask.set_bit(0);
        mask.set_bit(64);
        assert!(!bs.matches_mask(&mask));
    }

    #[test]
    fn test_filter_bitset_matches_mask_mask_longer_with_zeros() {
        let mut bs = FilterBitset::new();
        bs.set_bit(0);
        let mut mask = FilterBitset::new();
        mask.set_bit(0);
        assert!(bs.matches_mask(&mask));
    }

    #[test]
    fn test_filter_bitset_matches_mask_empty() {
        let mut bs = FilterBitset::new();
        bs.set_bit(5);
        let mask = FilterBitset::new();
        assert!(bs.matches_mask(&mask));
    }

    #[test]
    fn test_filter_bitset_u128_roundtrip() {
        let val: u128 = 0xDEAD_BEEF_CAFE_F00D;
        let bs = FilterBitset::from(val);
        let back: u128 = bs.into();
        assert_eq!(back, val);
        let bs2 = FilterBitset::from_u128(val);
        assert_eq!(bs2.to_u128(), val);
    }

    #[test]
    fn test_filter_bitset_u128_high_bits() {
        let high = 0xCAFE_F00D_0000_0000_0000_0000_0000_0000u128;
        let bs = FilterBitset::from_u128(high);
        assert_eq!(bs.to_u128(), high);
    }

    #[test]
    fn test_filter_bitset_u128_zero() {
        let bs = FilterBitset::from_u128(0);
        assert!(bs.is_empty());
        assert_eq!(bs.to_u128(), 0);
    }

    #[test]
    fn test_filter_bitset_bytes_roundtrip() {
        let mut bs = FilterBitset::new();
        bs.set_bit(0);
        bs.set_bit(63);
        bs.set_bit(64);
        bs.set_bit(128);
        let bytes = bs.to_bytes();
        let (restored, consumed) = FilterBitset::from_bytes(&bytes).unwrap();
        assert_eq!(consumed, bytes.len());
        assert_eq!(restored, bs);
    }

    #[test]
    fn test_filter_bitset_bytes_truncated() {
        assert!(FilterBitset::from_bytes(&[0x00]).is_err());
        assert!(FilterBitset::from_bytes(&[]).is_err());
    }

    #[test]
    fn test_filter_bitset_bytes_exceeds_max_words() {
        let mut data = Vec::new();
        data.extend_from_slice(&5000u32.to_le_bytes());
        assert!(FilterBitset::from_bytes(&data).is_err());
    }

    #[test]
    fn test_filter_bitset_default() {
        let bs: FilterBitset = Default::default();
        assert!(bs.is_empty());
    }

    #[test]
    fn test_filter_bitset_clone_eq() {
        let mut a = FilterBitset::new();
        a.set_bit(1);
        a.set_bit(2);
        let b = a.clone();
        assert_eq!(a, b);
        let mut c = FilterBitset::new();
        c.set_bit(1);
        assert_ne!(a, c);
    }

    // ── VectorRepresentations tests ──

    #[test]
    fn test_vector_dimensions() {
        assert_eq!(
            VectorRepresentations::Full(vec![1.0, 2.0, 3.0]).dimensions(),
            3
        );
        assert_eq!(
            VectorRepresentations::Binary(vec![0u64; 4].into()).dimensions(),
            256
        );
        assert_eq!(
            VectorRepresentations::Turbo(vec![0u8; 10].into()).dimensions(),
            20
        );
        assert_eq!(
            VectorRepresentations::SQ8(vec![0i8; 8].into(), 1.0).dimensions(),
            8
        );
        assert_eq!(VectorRepresentations::None.dimensions(), 0);
        assert_eq!(VectorRepresentations::MmapFull(None).dimensions(), 0);
    }

    #[test]
    fn test_vector_is_none() {
        assert!(VectorRepresentations::None.is_none());
        assert!(!VectorRepresentations::Full(vec![1.0]).is_none());
        assert!(!VectorRepresentations::Binary(vec![0u64].into()).is_none());
        assert!(!VectorRepresentations::Turbo(vec![0u8; 2].into()).is_none());
        assert!(!VectorRepresentations::SQ8(vec![0i8; 2].into(), 1.0).is_none());
    }

    #[test]
    fn test_vector_to_f32() {
        assert_eq!(
            VectorRepresentations::Full(vec![1.0, 2.0, 3.0]).to_f32(),
            Some(vec![1.0, 2.0, 3.0])
        );
        let inv = 127.0 / 127.0;
        let sq8 = VectorRepresentations::SQ8(vec![0i8, 64, 127].into(), 127.0);
        let decoded = sq8.to_f32().unwrap();
        assert!((decoded[0] - 0.0).abs() < 0.001);
        assert!((decoded[1] - 64.0 * inv).abs() < 0.001);
        assert!((decoded[2] - 127.0 * inv).abs() < 0.001);
        assert!(VectorRepresentations::Binary(vec![0u64].into())
            .to_f32()
            .is_none());
        assert!(VectorRepresentations::None.to_f32().is_none());
    }

    #[test]
    fn test_vector_as_f32_slice() {
        assert_eq!(
            VectorRepresentations::Full(vec![1.0, 2.0, 3.0]).as_f32_slice(),
            Some(&[1.0, 2.0, 3.0][..])
        );
        assert!(VectorRepresentations::None.as_f32_slice().is_none());
        assert!(VectorRepresentations::Binary(vec![0u64].into())
            .as_f32_slice()
            .is_none());
    }

    #[test]
    fn test_vector_memory_size() {
        assert_eq!(VectorRepresentations::Full(vec![0.0; 10]).memory_size(), 40);
        assert_eq!(VectorRepresentations::None.memory_size(), 0);
        assert_eq!(VectorRepresentations::MmapFull(None).memory_size(), 0);
        assert_eq!(
            VectorRepresentations::Binary(vec![0u64; 4].into()).memory_size(),
            32
        );
        assert_eq!(
            VectorRepresentations::Turbo(vec![0u8; 100].into()).memory_size(),
            100
        );
        assert_eq!(
            VectorRepresentations::SQ8(vec![0i8; 16].into(), 2.0).memory_size(),
            20
        );
    }

    #[test]
    fn test_vector_cosine_similarity_basic() {
        let a = VectorRepresentations::Full(vec![1.0, 0.0]);
        let b = VectorRepresentations::Full(vec![0.0, 1.0]);
        let sim = a.cosine_similarity(&b);
        assert!(sim.is_some());
        assert!((sim.unwrap() - 0.0).abs() < 1e-6);
        assert!((a.cosine_similarity(&a).unwrap() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_vector_cosine_similarity_incompatible() {
        let a = VectorRepresentations::Full(vec![1.0, 2.0, 3.0]);
        let b = VectorRepresentations::Full(vec![1.0, 2.0]);
        assert!(a.cosine_similarity(&b).is_none());
        assert!(VectorRepresentations::None.cosine_similarity(&a).is_none());
        assert!(a.cosine_similarity(&VectorRepresentations::None).is_none());
    }

    #[test]
    fn test_vector_cosine_similarity_zero() {
        let zero = VectorRepresentations::Full(vec![0.0, 0.0]);
        let a = VectorRepresentations::Full(vec![1.0, 0.0]);
        assert!(zero.cosine_similarity(&a).is_none());
        assert!(a.cosine_similarity(&zero).is_none());
    }

    #[test]
    fn test_vector_partial_eq() {
        assert_eq!(VectorRepresentations::None, VectorRepresentations::None);
        assert_eq!(
            VectorRepresentations::Full(vec![1.0, 2.0]),
            VectorRepresentations::Full(vec![1.0, 2.0])
        );
        assert_ne!(
            VectorRepresentations::Full(vec![1.0]),
            VectorRepresentations::Full(vec![2.0])
        );
        assert_eq!(
            VectorRepresentations::MmapFull(None),
            VectorRepresentations::MmapFull(None)
        );
    }

    // ── NodeFlags tests ──

    #[test]
    fn test_node_flags_new() {
        let flags = NodeFlags::new();
        assert!(flags.is_active());
        assert!(!flags.is_tombstone());
        assert!(!flags.is_set(NodeFlags::DIRTY));
    }

    #[test]
    fn test_node_flags_set_clear() {
        let mut flags = NodeFlags::new();
        flags.set(NodeFlags::DIRTY);
        assert!(flags.is_set(NodeFlags::DIRTY));
        flags.clear(NodeFlags::DIRTY);
        assert!(!flags.is_set(NodeFlags::DIRTY));
    }

    #[test]
    fn test_node_flags_all_constants() {
        let mut flags = NodeFlags(0);
        flags.set(NodeFlags::INDEXED);
        assert!(flags.is_set(NodeFlags::INDEXED));
        flags.clear(NodeFlags::INDEXED);
        assert!(!flags.is_set(NodeFlags::INDEXED));
        flags.set(NodeFlags::HAS_VECTOR);
        assert!(flags.is_set(NodeFlags::HAS_VECTOR));
        flags.set(NodeFlags::HAS_EDGES);
        assert!(flags.is_set(NodeFlags::HAS_EDGES));
        flags.set(NodeFlags::PINNED);
        assert!(flags.is_set(NodeFlags::PINNED));
        flags.set(NodeFlags::RECOVERED);
        assert!(flags.is_set(NodeFlags::RECOVERED));
        flags.set(NodeFlags::INVALIDATED);
        assert!(flags.is_set(NodeFlags::INVALIDATED));
        flags.set(NodeFlags::CONFLICT_RESOLVED);
        assert!(flags.is_set(NodeFlags::CONFLICT_RESOLVED));
    }

    #[test]
    fn test_node_flags_tombstone() {
        let mut flags = NodeFlags::new();
        assert!(flags.is_active());
        assert!(!flags.is_tombstone());
        flags.clear(NodeFlags::ACTIVE);
        assert!(!flags.is_active());
        flags.set(NodeFlags::TOMBSTONE);
        assert!(flags.is_tombstone());
    }

    // ── Edge tests ──

    #[test]
    fn test_edge_new() {
        let edge = Edge::new(42, 0);
        assert_eq!(edge.target, 42);
        assert_eq!(edge.label_id, 0);
        assert!((edge.weight - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_edge_with_weight() {
        let edge = Edge::with_weight(99, 1, 0.5);
        assert_eq!(edge.target, 99);
        assert_eq!(edge.label_id, 1);
        assert!((edge.weight - 0.5).abs() < f32::EPSILON);
    }

    // ── Temporal edge tests (COMP-021) ──

    /// Legacy 4-field shape persisted before `created_at_ms` existed.
    /// WAL/storage use postcard, so this byte-level round-trip proves
    /// `#[serde(default)]` keeps old datasets readable (`created_at_ms == 0`).
    #[derive(Serialize, Deserialize)]
    struct LegacyEdge {
        target: u128,
        label_id: u32,
        weight: f32,
        reverse: bool,
    }

    #[test]
    fn test_edge_backward_compat_postcard_default() {
        let legacy = LegacyEdge {
            target: 7,
            label_id: 3,
            weight: 0.5,
            reverse: false,
        };
        let bytes = postcard::to_allocvec(&legacy).unwrap();
        let edge: Edge = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(edge.created_at_ms, 0, "missing field must default to 0");
        assert_eq!(edge.target, 7);
        assert_eq!(edge.label_id, 3);
        assert!((edge.weight - 0.5).abs() < f32::EPSILON);
        assert!(!edge.reverse);
    }

    #[test]
    fn test_edge_with_timestamp() {
        let edge = Edge::with_timestamp(5, 2, 1_700_000_000_000);
        assert_eq!(edge.created_at_ms, 1_700_000_000_000);
        assert_eq!(edge.target, 5);
        assert!(!edge.reverse);
    }

    #[test]
    fn test_edge_default_timestamp_is_now() {
        let edge = Edge::new(1, 0);
        assert!(
            edge.created_at_ms > 0,
            "default created_at_ms must be wall-clock now, got {}",
            edge.created_at_ms
        );
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

    // ── UnifiedNode additional tests ──

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

    // ── FieldValue tests ──

    #[test]
    fn test_field_value_as_str() {
        assert_eq!(FieldValue::String("hello".into()).as_str(), Some("hello"));
        assert_eq!(FieldValue::Int(42).as_str(), None);
        assert_eq!(FieldValue::Null.as_str(), None);
    }

    #[test]
    fn test_field_value_as_int() {
        assert_eq!(FieldValue::Int(42).as_int(), Some(42));
        assert_eq!(FieldValue::String("x".into()).as_int(), None);
    }

    #[test]
    fn test_field_value_as_bool() {
        assert_eq!(FieldValue::Bool(true).as_bool(), Some(true));
        assert_eq!(FieldValue::Int(0).as_bool(), None);
    }

    #[test]
    fn test_field_value_cardinality_keys() {
        assert_eq!(
            FieldValue::String("test".into()).to_cardinality_keys(),
            vec!["test"]
        );
        assert_eq!(FieldValue::Int(42).to_cardinality_keys(), vec!["42"]);
        assert_eq!(FieldValue::Float(42.5).to_cardinality_keys(), vec!["42.5"]);
        assert_eq!(FieldValue::Bool(true).to_cardinality_keys(), vec!["true"]);
        assert_eq!(FieldValue::Null.to_cardinality_keys(), vec!["null"]);
        let dt = chrono::DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        assert!(FieldValue::DateTime(dt).to_cardinality_keys()[0].contains("2024"));
        assert_eq!(
            FieldValue::ListString(vec!["a".into()]).to_cardinality_keys(),
            vec!["a"]
        );
        assert_eq!(
            FieldValue::ListInt(vec![1, 2]).to_cardinality_keys(),
            vec!["1", "2"]
        );
        assert_eq!(
            FieldValue::ListBool(vec![true]).to_cardinality_keys(),
            vec!["true"]
        );
    }

    #[test]
    fn test_field_value_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(FieldValue::Int(42));
        assert!(set.contains(&FieldValue::Int(42)));
        assert!(!set.contains(&FieldValue::Int(43)));
        set.insert(FieldValue::String("hello".into()));
        assert!(set.contains(&FieldValue::String("hello".into())));
        set.insert(FieldValue::Bool(true));
        assert!(set.contains(&FieldValue::Bool(true)));
        set.insert(FieldValue::Null);
        assert!(set.contains(&FieldValue::Null));
    }

    // ── DiskNodeHeader tests ──

    #[test]
    fn test_disk_node_header_new() {
        let header = DiskNodeHeader::new(999);
        assert_eq!(header.id, 999);
        assert_eq!(header.bitset, 0);
        assert_eq!(header.vector_offset, 0);
        assert!((header.confidence_score - 0.5).abs() < f32::EPSILON);
        assert!((header.importance - 0.1).abs() < f32::EPSILON);
        assert_eq!(header.relational_len, 0);
        assert_eq!(header.vector_len, 0);
        assert_eq!(header.flags, 0);
        assert_eq!(header.edge_count, 0);
        assert_eq!(header.tier, 0);
    }

    // ── Misc tests ──

    #[test]
    fn test_node_tier_default() {
        assert_eq!(NodeTier::default(), NodeTier::Cold);
    }

    #[test]
    fn test_distance_metric_default() {
        assert_eq!(DistanceMetric::default(), DistanceMetric::Cosine);
    }

    #[test]
    fn test_distance_metric_eq() {
        assert_ne!(DistanceMetric::Cosine, DistanceMetric::Euclidean);
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
