//! Unit tests for the storage engine, organized by source module.
//!
//! Split from the original 4076-line `tests.rs` into per-feature files.

use super::*; // StorageEngine, MemoryStats, EvictionReason, EvictionReport, constants, etc.
use crate::backend::BackendKind;
use crate::config::VantaConfig;
use crate::node::UnifiedNode;

// ─── Sub-modules (one per source module) ──────────────────────

mod engine;
mod incremental;
mod init;
mod maintenance;
mod ops;
mod scalar_index;
mod stats;
mod types;

// ─── Shared helpers used by all sub-modules ───────────────────

pub(super) fn in_memory_engine() -> StorageEngine {
    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        read_only: false,
        ..VantaConfig::default()
    };
    StorageEngine::open_with_config(":memory:", Some(config))
        .expect("Failed to open in-memory engine")
}

pub(super) fn in_memory_read_only() -> StorageEngine {
    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        read_only: true,
        ..VantaConfig::default()
    };
    StorageEngine::open_with_config(":memory:", Some(config))
        .expect("Failed to open read-only in-memory engine")
}

/// In-memory engine pre-wired with all four LSM tiers (L0..L3).
///
/// The plain `in_memory_engine()` helper creates a single L0 segment, which is
/// enough for most tests but not for tier promotion (L1/L2/L3 targets). This
/// mirrors the persistent layout produced by `SegmentRegistry::open_or_create`.
pub(super) fn in_memory_tiered_engine() -> StorageEngine {
    let mut engine = in_memory_engine();
    for _ in 1..4 {
        let vs = crate::storage::vfile::VantaFile::create_in_memory(64 * MIB);
        engine.vector_store.push(parking_lot::RwLock::new(vs));
    }
    engine
}

pub(super) fn sample_node(id: u128) -> UnifiedNode {
    let mut node = UnifiedNode::new(id);
    node.vector = crate::node::VectorRepresentations::Full(vec![0.1, 0.2, 0.3]);
    node
}
