use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::storage::vfile::Mmap;

const MAX_VEC_F32_LEN: usize = 10_000_000; // Max ~40MB for a single f32 vector

/// Metric type used for vector distance/similarity calculations.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub enum DistanceMetric {
    /// Cosine similarity (default).
    #[default]
    Cosine,
    /// Euclidean distance.
    Euclidean,
    /// Sparse vector dot product (higher = more similar). Sparse vectors are
    /// user-provided dim→value maps; no dedicated index topology exists, so
    /// operators under this metric must fall back to a brute-force scan.
    SparseDot,
}

/// A sparse vector: mapping from dimension id → coefficient (f32).
///
/// Stored as a `BTreeMap` for deterministic serialization (JSON/Bincode round
/// trips produce identical byte order) and to make the dot-product walk both
/// maps in sorted order with a single linear merge. Any `u32` dim id is valid;
/// the domain (vocabulary size) is defined by the caller, not this type.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SparseVector(pub BTreeMap<u32, f32>);

impl SparseVector {
    /// Create an empty sparse vector.
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    /// Insert or overwrite the coefficient at `dim`.
    pub fn insert(&mut self, dim: u32, value: f32) {
        self.0.insert(dim, value);
    }

    /// Number of populated dimensions.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether the vector has no populated dimensions.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Sparse dot product between two sparse vectors (linear merge over the
    /// sorted keys of both maps). Dimensions present in only one side
    /// contribute zero, matching the standard sparse-dot definition.
    pub fn dot(&self, other: &SparseVector) -> f32 {
        let mut it_a = self.0.iter().peekable();
        let mut it_b = other.0.iter().peekable();
        let mut acc = 0.0_f32;
        while let (Some((da, va)), Some((db, vb))) = (it_a.peek(), it_b.peek()) {
            match da.cmp(db) {
                std::cmp::Ordering::Less => {
                    it_a.next();
                }
                std::cmp::Ordering::Greater => {
                    it_b.next();
                }
                std::cmp::Ordering::Equal => {
                    acc += *va * *vb;
                    it_a.next();
                    it_b.next();
                }
            }
        }
        acc
    }
}

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

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;

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

    #[test]
    fn test_distance_metric_default() {
        assert_eq!(DistanceMetric::default(), DistanceMetric::Cosine);
    }

    #[test]
    fn test_distance_metric_eq() {
        assert_ne!(DistanceMetric::Cosine, DistanceMetric::Euclidean);
    }
}
