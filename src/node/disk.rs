use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

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

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;

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
}
