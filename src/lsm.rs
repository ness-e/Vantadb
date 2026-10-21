//! LSM-tree segment types, offset packing, and multi-level segment registry.
//!
//! Provides packed offsets (segment_id in low 6 bits, 64-aligned offset in upper bits),
//! LSM level identifiers, per-level configuration, segment metadata, and the
//! [`SegmentRegistry`] which manages multi-level File lifecycle.
//!
//! ponytail: tier promotion hot→warm→cold→archive (L0→L3) driven by
//! size/tombstone thresholds; Frequency/Age heuristics are config-only for now.

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

    #[allow(dead_code)]
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::L0),
            1 => Some(Self::L1),
            2 => Some(Self::L2),
            3 => Some(Self::L3),
            _ => None,
        }
    }

    /// File name for this level's File (e.g. "vstore_L0.vanta").
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
#[allow(dead_code)]
pub(crate) struct SegmentInfo {
    pub segment_id: u8,
    pub level: u8,
    pub path: PathBuf,
    pub size: u64,
    pub last_compacted: Option<Instant>,
    pub tombstone_ratio: f32,
}

/// Multi-level segment registry that manages the lifecycle of level Files.
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

    /// Open or create multi-level Files for levels L0..=L3.
    ///
    /// Creates the 4 LSM levels so `compact_level()` never needs to grow
    /// `vector_store` dynamically (no `unsafe` needed).
    /// Returns `(Self, Vec<RwLock<File>>)` with a File per level.
    ///
    /// `crypto` is the encryption context of the `vfile` artefact
    /// (ADR-0057 W3): it is the **same** context for every level, because
    /// the DEK and the nonce space belong to the artefact, not to the file.
    /// `None` keeps every segment plaintext.
    ///
    /// FIND-517: the initial size of a **fresh** segment comes from
    /// `config.lsm.initial_segment_bytes` (1 MiB by default) instead of the
    /// old hardcoded 64 MiB. An empty store used to reserve 4 x 64 MiB of
    /// address space up front. Existing segments keep their size: only a
    /// file that does not exist yet (or is still smaller than the header)
    /// is sized here.
    pub fn open_or_create(
        data_dir: &std::path::Path,
        config: &crate::storage::engine::SegmentOptimizerConfig,
        crypto: crate::storage::vfile_crypto::VFileCryptoSlot,
    ) -> crate::error::Result<(Self, Vec<parking_lot::RwLock<crate::storage::vfile::File>>)> {
        let mut registry = Self::new();
        let mut vfiles: Vec<parking_lot::RwLock<crate::storage::vfile::File>> = Vec::new();

        // Legacy migration: detect vector_store.vanta → rename to vstore_L0.vanta
        let legacy_path = data_dir.join("vector_store.vanta");
        let l0_path = data_dir.join(SegmentLevel::L0.file_name());
        if legacy_path.exists() && !l0_path.exists() {
            std::fs::rename(&legacy_path, &l0_path).map_err(crate::error::Error::Io)?;
            tracing::info!(
                "Migrated legacy vector_store.vanta → {}",
                SegmentLevel::L0.file_name()
            );
        }

        // FIND-517: the floor is the segment header (64 bytes) — a segment
        // smaller than that cannot hold the VFLE header + write cursor.
        let initial_size = config
            .lsm
            .initial_segment_bytes
            .max(crate::storage::engine::STORAGE_ALIGNMENT);

        // Create the 4 levels: L0 (hot), L1 (warm), L2 (cold), L3 (archive).
        // Unused levels cost a file handle + an mmap of `initial_size`.
        for level in &[
            SegmentLevel::L0,
            SegmentLevel::L1,
            SegmentLevel::L2,
            SegmentLevel::L3,
        ] {
            let path = data_dir.join(level.file_name());
            let vf = crate::storage::vfile::File::open_with_crypto(
                path.clone(),
                initial_size,
                false,
                crypto.clone(),
            )?;
            registry.register(level.as_u8(), level.as_u8(), path);
            vfiles.push(parking_lot::RwLock::new(vf));
        }

        Ok((registry, vfiles))
    }

    /// How many levels are currently tracked (0-4).
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.segments.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Get the segment_id for the given level.
    #[allow(dead_code)]
    pub fn segment_id_for_level(&self, level: SegmentLevel) -> Option<u8> {
        self.segments
            .iter()
            .find(|s| s.level == level.as_u8())
            .map(|s| s.segment_id)
    }

    /// Get the level for the given segment_id.
    #[allow(dead_code)]
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

/// Tier promotion heuristic. `compact_level`/`should_compact_level` currently
/// execute `SizeBased`; the frequency/age variants are exposed as a nominated
/// policy for a future per-node access tracker and do not change behavior yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)] // *Based* suffixes are intentional, they mirror STORAGE-TIERS.md
pub enum TierPolicy {
    /// Promote when `write_cursor` reaches the level's max size or the tombstone
    /// ratio crosses the level's threshold.
    SizeBased,
    /// Promote when a level's resident nodes fall below `cold_min_frequency`
    /// accesses per window. Config-only until an access tracker exists.
    FrequencyBased,
    /// Promote when a level's resident nodes idle longer than `cold_age_days`.
    /// Config-only until an access tracker exists.
    AgeBased,
}

/// Tunable knobs for the tier policy.
#[derive(Debug, Clone, Copy)]
pub struct TierPolicyConfig {
    /// Which heuristic drives promotion.
    pub kind: TierPolicy,
    /// Whether the L3 archive level participates in compaction. When `false`,
    /// `should_compact_level` never selects L3 and L2 is the deepest tier.
    pub archive: bool,
    /// Accesses per window under which a node is considered cold (`FrequencyBased`).
    pub cold_min_access: u32,
    /// Days idled before a node is considered cold (`AgeBased`).
    pub cold_age_days: u64,
}

impl Default for TierPolicyConfig {
    fn default() -> Self {
        Self {
            kind: TierPolicy::SizeBased,
            archive: true,
            cold_min_access: 3,
            cold_age_days: 30,
        }
    }
}

/// Per-level LSM configuration.
#[derive(Debug, Clone, Copy)]
pub struct LsmConfig {
    pub l0_max_size: u64,
    pub l1_max_size: u64,
    pub l2_max_size: u64,
    pub l3_max_size: u64,
    pub l0_tombstone_threshold: f32,
    pub l1_tombstone_threshold: f32,
    pub l2_tombstone_threshold: f32,
    pub l3_tombstone_threshold: f32,
    pub min_segment_size: u64,
    /// Initial size of a **freshly created** segment, in bytes (FIND-517).
    ///
    /// Every level (L0..L3) owns one segment file, so the address space a
    /// store reserves when it opens is `4 * initial_segment_bytes`. This used
    /// to be a hardcoded 64 MiB per level, which made an **empty** store
    /// reserve 256 MiB before a single record was written (the Node binding
    /// added fjall's own 64 MiB journal on top: 320 MiB per open store, the
    /// dominant memory cost of the Ego product).
    ///
    /// Segments grow on demand (`File::grow_to` doubles the file), so a small
    /// value only costs a few extra `set_len`+remap cycles. The compaction
    /// thresholds (`l0_max_size` and friends) are unaffected: they are
    /// compared against the **logical** write cursor, not the file size.
    pub initial_segment_bytes: u64,
    /// Tier promotion policy (hot/warm/cold/archive).
    pub tier: TierPolicyConfig,
}

impl Default for LsmConfig {
    fn default() -> Self {
        Self {
            l0_max_size: 64 * 1024 * 1024,        // 64 MB
            l1_max_size: 512 * 1024 * 1024,       // 512 MB
            l2_max_size: 4 * 1024 * 1024 * 1024,  // 4 GB
            l3_max_size: 32 * 1024 * 1024 * 1024, // 32 GB
            l0_tombstone_threshold: 0.20,
            l1_tombstone_threshold: 0.15,
            l2_tombstone_threshold: 0.10,
            l3_tombstone_threshold: 0.05,
            min_segment_size: 64 * 1024, // 64 KB
            // FIND-517: 1 MiB per level => 4 MiB per store (was 64 MiB/level).
            initial_segment_bytes: 1024 * 1024, // 1 MiB
            tier: TierPolicyConfig::default(),
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

    /// FIND-517: an **empty** store must not pre-reserve 64 MB per level.
    ///
    /// Before the fix, `open_or_create` hardcoded `64 * 1024 * 1024` as the
    /// initial size of every segment: opening a store with zero records
    /// reserved 4 x 64 MiB of address space (256 MiB) up front. The Node
    /// binding added fjall's own 64 MiB journal on top (320 MiB per open
    /// store) — the dominant memory cost of the Ego product.
    ///
    /// The segments grow on demand (`File::grow_to` doubles the file), so a
    /// small initial size costs a handful of extra `set_len`+remap cycles
    /// and nothing else.
    #[test]
    fn fresh_segments_do_not_reserve_64mb() {
        let dir = tempfile::tempdir().expect("tempdir");
        let config = crate::storage::engine::SegmentOptimizerConfig::default();

        let (registry, vfiles) = SegmentRegistry::open_or_create(dir.path(), &config, None)
            .expect("open_or_create succeeds");
        assert_eq!(vfiles.len(), 4, "one file per LSM level");
        assert_eq!(registry.len(), 4, "one registry entry per LSM level");

        for level in [
            SegmentLevel::L0,
            SegmentLevel::L1,
            SegmentLevel::L2,
            SegmentLevel::L3,
        ] {
            let path = dir.path().join(level.file_name());
            let len = std::fs::metadata(&path).expect("segment file exists").len();
            assert!(
                len <= 4 * 1024 * 1024,
                "FIND-517: {} reserves {len} bytes on an empty store; \
                 the pre-reserve must stay in the low-MB range",
                level.file_name()
            );
        }
    }

    /// FIND-517: the initial size of a fresh segment is **configurable**.
    ///
    /// The knob exists so a binding can expose it (`ConnectOptions`
    /// in the Node binding) and so the reserve can be covered by the
    /// caller's `memory_limit`.
    #[test]
    fn open_or_create_honours_initial_segment_bytes() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut config = crate::storage::engine::SegmentOptimizerConfig::default();
        config.lsm.initial_segment_bytes = 512 * 1024;

        let (_registry, _vfiles) = SegmentRegistry::open_or_create(dir.path(), &config, None)
            .expect("open_or_create succeeds");

        for level in [
            SegmentLevel::L0,
            SegmentLevel::L1,
            SegmentLevel::L2,
            SegmentLevel::L3,
        ] {
            let path = dir.path().join(level.file_name());
            let len = std::fs::metadata(&path).expect("segment file exists").len();
            assert_eq!(
                len,
                512 * 1024,
                "{} must honour lsm.initial_segment_bytes",
                level.file_name()
            );
        }
    }
}
