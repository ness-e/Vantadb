use serde::{Deserialize, Serialize};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

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

    // ── Vector kind (ADR-032) — bits 10-13 in `DiskNodeHeader.flags`
    // ponytail: 4-bit kind in flags, no header bump; lazy migration for legacy Binary
    /// Mask for the 4-bit vector kind field (bits 10-13).
    pub const VECTOR_KIND_MASK: u32 = 0x3C00; // 0b11_1100_0000_0000
    /// Shift for the vector kind field.
    pub const VECTOR_KIND_SHIFT: u32 = 10;
    /// No vector.
    pub const VECTOR_KIND_NONE: u32 = 0;
    /// `Full(Vec<f32>)` — dense f32.
    pub const VECTOR_KIND_FULL: u32 = 1;
    /// `Binary(Box<[u64]>)` — RaBitQ 1-bit.
    pub const VECTOR_KIND_BINARY: u32 = 2;
    /// `Turbo(Box<[u8]>)` — PolarQuant 4-bit.
    pub const VECTOR_KIND_TURBO: u32 = 3;
    /// `SQ8(Box<[i8]>, f32)` — 8-bit + scale.
    pub const VECTOR_KIND_SQ8: u32 = 4;

    /// Extract the vector kind (ADR-032) from raw flags.
    #[inline]
    pub fn vector_kind(flags: u32) -> u32 {
        (flags & Self::VECTOR_KIND_MASK) >> Self::VECTOR_KIND_SHIFT
    }

    /// Encode a vector kind into raw flags, preserving other bits.
    #[inline]
    pub fn with_vector_kind(flags: u32, kind: u32) -> u32 {
        (flags & !Self::VECTOR_KIND_MASK)
            | ((kind << Self::VECTOR_KIND_SHIFT) & Self::VECTOR_KIND_MASK)
    }

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

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;

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

    #[test]
    fn test_node_tier_default() {
        assert_eq!(NodeTier::default(), NodeTier::Cold);
    }
}
