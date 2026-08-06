//! Low-level storage operations: node serialization, backend I/O, partition resolution.

use crate::backend::BackendPartition;
use crate::error::{Result, VantaError};
use crate::node::{DiskNodeHeader, UnifiedNode};
use crate::storage::vfile::VantaFile;
use std::path::Path;
use zerocopy::IntoBytes;

/// Serialized metadata stored per node in the KV backend.
#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct NodeMetadata {
    /// Relational field values attached to the node.
    pub relational: crate::node::RelFields,
    /// Graph edges originating from the node.
    pub edges: Vec<crate::node::Edge>,
    /// Transaction ID that created/updated this version (0 = pre-MVCC).
    pub created_by_txn: u64,
    /// Transaction ID that deleted this version (None = alive).
    pub deleted_by_txn: Option<u64>,
}

/// Write a node's header and vector data into the VantaFile at the current cursor position.
pub(crate) fn write_node_to_vstore(vstore: &mut VantaFile, node: &UnifiedNode) -> Result<u64> {
    let offset = vstore.write_cursor;
    let header_size = std::mem::size_of::<DiskNodeHeader>() as u64;
    let vec_len = if let crate::node::VectorRepresentations::Full(ref v) = node.vector {
        v.len()
    } else {
        0
    };
    let vec_size = (vec_len * 4) as u64;
    let total_needed = offset + header_size + vec_size;
    if total_needed > vstore.size {
        let new_size = (vstore.size * 2).max(total_needed + 4096);
        vstore.grow_to(new_size)?;
    }
    let mut header = DiskNodeHeader::new(node.id);
    header.vector_offset = offset + header_size;
    header.vector_len = vec_len as u32;
    header.flags = node.flags.0;
    header.bitset = node.bitset.to_u128();
    header.confidence_score = node.confidence_score;
    header.importance = node.importance;
    header.tier = match node.tier {
        crate::node::NodeTier::Hot => 1u8,
        crate::node::NodeTier::Cold => 0u8,
    };
    header.edge_count = node.edges.len() as u16;
    vstore.write_header(offset, &header)?;
    if let crate::node::VectorRepresentations::Full(ref vec) = node.vector {
        let vec_bytes = vec.as_bytes();
        vstore.mmap_bytes_mut()?
            [(offset + header_size) as usize..(offset + header_size + vec_size) as usize]
            .copy_from_slice(vec_bytes);
    }
    vstore.write_cursor = (total_needed + 63) & !63;
    vstore.save_cursor()?;
    Ok(offset)
}

/// Reject paths containing `..` components to prevent directory traversal.
/// Absolute paths are allowed — the `..` check is the real security boundary.
pub(crate) fn prevent_path_traversal(path: &str) -> Result<()> {
    use std::path::Component;
    for component in std::path::Path::new(path).components() {
        if component == Component::ParentDir {
            return Err(VantaError::ValidationError {
                field: "path".into(),
                reason: format!("Path '{path}' contains '..' traversal — rejected for security"),
            });
        }
    }
    Ok(())
}

/// Resolve a user-supplied path against a trusted base directory and verify
/// the final resolved path stays within the base.  This prevents:
///
/// * `..` traversal (already rejected by `prevent_path_traversal`)
/// * Absolute paths that escape the base directory
/// * Symlink-based escapes inside the base directory
///
/// For paths that do not yet exist (e.g. export to a new file), the parent
/// directory is canonicalized and the filename appended — the containment
/// check still applies to the parent.
///
/// # Errors
///
/// Returns `ValidationError` if the resolved path falls outside the base
/// directory, or `IoError` on filesystem canonicalization failure.
pub(crate) fn resolve_against_base(base: &Path, user_path: &Path) -> Result<std::path::PathBuf> {
    // ── 1. reject `..` components ──────────────────────────────────────
    prevent_path_traversal(&user_path.to_string_lossy())?;

    // ── 2. combine with base (relative → join; absolute → use as-is) ──
    let combined = if user_path.is_relative() {
        base.join(user_path)
    } else {
        user_path.to_path_buf()
    };

    // ── 3. canonicalize ────────────────────────────────────────────────
    let canonical = if combined.exists() {
        combined.canonicalize().map_err(VantaError::IoError)?
    } else {
        let parent = combined.parent().unwrap_or(Path::new("."));
        let file_name = combined
            .file_name()
            .ok_or_else(|| VantaError::ValidationError {
                field: "path".into(),
                reason: format!(
                    "Path '{}' has no filename component — cannot resolve against base",
                    user_path.display(),
                ),
            })?;
        let canonical_parent = parent.canonicalize().map_err(VantaError::IoError)?;
        canonical_parent.join(file_name)
    };

    // ── 4. verify containment ─────────────────────────────────────────
    let canonical_base = base.canonicalize().map_err(VantaError::IoError)?;
    if canonical.starts_with(&canonical_base) {
        Ok(canonical)
    } else {
        Err(VantaError::ValidationError {
            field: "path".into(),
            reason: format!(
                "Path '{}' resolves to '{}' which is outside the allowed directory '{}'",
                user_path.display(),
                canonical.display(),
                canonical_base.display(),
            ),
        })
    }
}

/// Map a column family name string to its `BackendPartition` variant.
pub(crate) fn partition_from_cf_name(cf_name: &str) -> Result<BackendPartition> {
    match cf_name {
        "default" => Ok(BackendPartition::Default),
        "tombstone_storage" => Ok(BackendPartition::TombstoneStorage),
        "compressed_archive" => Ok(BackendPartition::CompressedArchive),
        "tombstones" => Ok(BackendPartition::Tombstones),
        "namespace_index" => Ok(BackendPartition::NamespaceIndex),
        "payload_index" => Ok(BackendPartition::PayloadIndex),
        "text_index" => Ok(BackendPartition::TextIndex),
        "sparse_index" => Ok(BackendPartition::SparseIndex),
        "internal_metadata" => Ok(BackendPartition::InternalMetadata),
        other => Err(VantaError::InvalidInput(format!(
            "Unknown column family: '{}'",
            other
        ))),
    }
}
