//! LSM-tree segment types, offset packing, and multi-level segment registry.
//!
//! Provides packed offsets (segment_id in low 6 bits, 64-aligned offset in upper bits),
//! LSM level identifiers, per-level configuration, segment metadata, and the
//! [`SegmentRegistry`] which manages multi-level VantaFile lifecycle.
//!
//! ponytail: L0 + L1 compaction only — L3 archive tier skipped.

use std::path::PathBuf;
use std::time::Instant;

const SEGMENT_ID_BITS: u64 = 6;
const SEGMENT_ID_MASK: u64 = (1 << SEGMENT_ID_BITS) - 1; // 0x3F

/// Pack segment_id into the low 6 bits of a 64-aligned offset.
/// Precondition: local_offset is a multiple of 64 (guaranteed by STORAGE_ALIGNMENT).
pub fn pack_offset(segment_id: u8, local_offset: u64) -> u64 {
    debug_assert!(local_offset % 64 == 0, "offset must be 64-aligned");
    debug_assert!(
        (segment_id as u64) < SEGMENT_ID_MASK,
        "segment_id out of range"
    );
    (local_offset & !SEGMENT_ID_MASK) | (segment_id as u64 & SEGMENT_ID_MASK)
}

/// Unpack (segment_id, local_offset) from a packed storage_offset.
pub fn unpack_offset(packed: u64) -> (u8, u64) {
    let segment_id = (packed & SEGMENT_ID_MASK) as u8;
    let local_offset = packed & !SEGMENT_ID_MASK;
    (segment_id, local_offset)
}

/// LSM level identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SegmentLevel {
    L0 = 0,
    L1 = 1,
    L2 = 2,
    L3 = 3,
}

impl SegmentLevel {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::L0),
            1 => Some(Self::L1),
            2 => Some(Self::L2),
            3 => Some(Self::L3),
            _ => None,
        }
    }

    /// File name for this level's VantaFile (e.g. "vstore_L0.vanta").
    pub fn file_name(self) -> &'static str {
        match self {
            Self::L0 => "vstore_L0.vanta",
            Self::L1 => "vstore_L1.vanta",
            Self::L2 => "vstore_L2.vanta",
            Self::L3 => "vstore_L3.vanta",
        }
    }
}

/// Info about one segment for SegmentRegistry.
#[derive(Debug, Clone)]
pub(crate) struct SegmentInfo {
    pub segment_id: u8,
    pub level: u8,
    pub path: PathBuf,
    pub size: u64,
    pub last_compacted: Option<Instant>,
    pub tombstone_ratio: f32,
}

/// Multi-level segment registry that manages the lifecycle of level VantaFiles.
///
/// Tracks which segments exist, their levels, and provides a compact
/// `by_id` lookup (64 entries — 6-bit segment_id, more than enough for 4 levels).
#[derive(Debug, Clone)]
pub(crate) struct SegmentRegistry {
    /// Ordered list of known segments (index is the canonical ordering).
    pub segments: Vec<SegmentInfo>,
    /// Fast segment_id → index lookup. `None` means the slot is unused.
    pub by_id: [Option<usize>; 64],
}

impl SegmentRegistry {
    /// Create a new empty registry (no segments).
    pub fn new() -> Self {
        Self {
            segments: Vec::with_capacity(4),
            by_id: [None; 64],
        }
    }

    /// Register a segment by its id, level, and path.
    /// Returns `None` if the slot is already taken.
    pub fn register(&mut self, segment_id: u8, level: u8, path: PathBuf) -> Option<usize> {
        let idx = self.by_id.get(segment_id as usize)?;
        if idx.is_some() {
            return None; // slot taken
        }
        let idx = self.segments.len();
        self.segments.push(SegmentInfo {
            segment_id,
            level,
            path,
            size: 0,
            last_compacted: None,
            tombstone_ratio: 0.0,
        });
        self.by_id[segment_id as usize] = Some(idx);
        Some(idx)
    }

    /// Open or create multi-level VantaFiles for levels L0..=L2.
    ///
    /// Detects legacy `vector_store.vanta` and renames to `vstore_L0.vanta`.
    /// Returns `(Self, Vec<RwLock<VantaFile>>)` with a VantaFile per open/create level.
    pub fn open_or_create(
        data_dir: &std::path::Path,
        config: &crate::storage::engine::SegmentOptimizerConfig,
    ) -> crate::error::Result<(
        Self,
        Vec<parking_lot::RwLock<crate::storage::vfile::VantaFile>>,
    )> {
        let mut registry = Self::new();
        let mut vfiles: Vec<parking_lot::RwLock<crate::storage::vfile::VantaFile>> = Vec::new();

        // Legacy migration: detect vector_store.vanta → rename to vstore_L0.vanta
        let legacy_path = data_dir.join("vector_store.vanta");
        let l0_path = data_dir.join(SegmentLevel::L0.file_name());
        if legacy_path.exists() && !l0_path.exists() {
            std::fs::rename(&legacy_path, &l0_path).map_err(crate::error::VantaError::IoError)?;
            tracing::info!(
                "Migrated legacy vector_store.vanta → {}",
                SegmentLevel::L0.file_name()
            );
        }

        // Open/create L0, L1, L2 — ponytail: L3 archive tier skipped
        for level in &[SegmentLevel::L0, SegmentLevel::L1, SegmentLevel::L2] {
            let path = data_dir.join(level.file_name());
            let vf = crate::storage::vfile::VantaFile::open(path.clone(), 64 * 1024 * 1024)?;
            registry.register(level.as_u8(), level.as_u8(), path);
            vfiles.push(parking_lot::RwLock::new(vf));
        }

        Ok((registry, vfiles))
    }

    /// How many levels are currently tracked (0-4).
    pub fn len(&self) -> usize {
        self.segments.len()
    }

    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Get the segment_id for the given level.
    pub fn segment_id_for_level(&self, level: SegmentLevel) -> Option<u8> {
        self.segments
            .iter()
            .find(|s| s.level == level.as_u8())
            .map(|s| s.segment_id)
    }

    /// Get the level for the given segment_id.
    pub fn level_for_segment(&self, segment_id: u8) -> Option<u8> {
        self.by_id
            .get(segment_id as usize)
            .copied()
            .flatten()
            .and_then(|idx| self.segments.get(idx))
            .map(|s| s.level)
    }
}

impl Default for SegmentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Per-level LSM configuration.
#[derive(Debug, Clone, Copy)]
pub struct LsmConfig {
    pub l0_max_size: u64,
    pub l1_max_size: u64,
    pub l2_max_size: u64,
    pub l0_tombstone_threshold: f32,
    pub l1_tombstone_threshold: f32,
    pub l2_tombstone_threshold: f32,
    pub min_segment_size: u64,
}

impl Default for LsmConfig {
    fn default() -> Self {
        Self {
            l0_max_size: 64 * 1024 * 1024,       // 64 MB
            l1_max_size: 512 * 1024 * 1024,      // 512 MB
            l2_max_size: 4 * 1024 * 1024 * 1024, // 4 GB
            l0_tombstone_threshold: 0.20,
            l1_tombstone_threshold: 0.15,
            l2_tombstone_threshold: 0.10,
            min_segment_size: 64 * 1024, // 64 KB
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_unpack_roundtrip_l0() {
        let offset: u64 = 64; // 64-aligned
        let packed = pack_offset(0, offset);
        assert_eq!(packed, offset); // segment 0 → no bits set → identity
        let (seg, off) = unpack_offset(packed);
        assert_eq!(seg, 0);
        assert_eq!(off, offset);
    }

    #[test]
    fn test_pack_unpack_roundtrip_l1() {
        let offset: u64 = 128; // 64-aligned
        let packed = pack_offset(1, offset);
        assert_ne!(packed, offset); // segment 1 → low bits differ
        let (seg, off) = unpack_offset(packed);
        assert_eq!(seg, 1);
        assert_eq!(off, offset & !0x3F);
    }

    #[test]
    fn test_pack_unpack_all_levels() {
        for seg in 0..=3u8 {
            for off in [64u64, 128, 4096, 1048576] {
                let packed = pack_offset(seg, off);
                let (seg2, off2) = unpack_offset(packed);
                assert_eq!(seg, seg2, "segment mismatch at offset {off}");
                assert_eq!(off & !0x3F, off2, "offset mismatch at seg {seg}");
            }
        }
    }

    #[test]
    fn test_segment_level_conversion() {
        assert_eq!(SegmentLevel::from_u8(0), Some(SegmentLevel::L0));
        assert_eq!(SegmentLevel::from_u8(1), Some(SegmentLevel::L1));
        assert_eq!(SegmentLevel::from_u8(2), Some(SegmentLevel::L2));
        assert_eq!(SegmentLevel::from_u8(3), Some(SegmentLevel::L3));
        assert_eq!(SegmentLevel::from_u8(4), None);
        assert_eq!(SegmentLevel::L0.as_u8(), 0);
        assert_eq!(SegmentLevel::L1.file_name(), "vstore_L1.vanta");
    }
}
