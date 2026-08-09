//! Low-level storage operations: node serialization, backend I/O, partition resolution.

use crate::backend::BackendPartition;
use crate::error::{Result, VantaError};
use crate::node::{DiskNodeHeader, UnifiedNode};
use crate::storage::vfile::VantaFile;
use std::path::Path;
use zerocopy::IntoBytes;

use serde::de::DeserializeOwned;

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

/// Upper bound for a serialized node payload read from the KV store.
///
/// `postcard` does not validate length prefixes before allocating: a corrupt
/// or attacker-crafted prefix (e.g. `Vec<Edge>` claiming billions of entries
/// in a handful of bytes) drives `Vec::with_capacity` with the untrusted
/// value, which can abort or OOM the process before the payload is read.
/// Everything deserialized from persisted bytes goes through this cap
/// (AUDREP-45).
pub(crate) const MAX_PERSISTED_NODE_BYTES: usize = 128 * 1024 * 1024;

/// Deserialize a persisted node payload under a fixed size cap.
///
/// Rejects buffers larger than [`MAX_PERSISTED_NODE_BYTES`] before
/// `postcard::from_bytes` can act on an untrusted length prefix, converting
/// a corrupt/oversized payload into a clean [`VantaError`] instead of a
/// panic or OOM.
pub(crate) fn deserialize_node_payload<T: DeserializeOwned>(
    bytes: &[u8],
    label: &str,
) -> Result<T> {
    if bytes.len() > MAX_PERSISTED_NODE_BYTES {
        return Err(VantaError::serialization(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "{label} payload of {} bytes exceeds {} byte cap",
                bytes.len(),
                MAX_PERSISTED_NODE_BYTES
            ),
        )));
    }
    postcard::from_bytes(bytes).map_err(VantaError::serialization)
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
        // ponytail: saturating to avoid overflow if size > 2^63 (already past sane limits)
        let new_size = (vstore.size.saturating_mul(2)).max(total_needed.saturating_add(4096));
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
    // ponytail: saturating to avoid overflow in 64-byte alignment if total_needed near u64::MAX
    vstore.write_cursor = total_needed.saturating_add(63) & !63;
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

#[cfg(test)]
mod tests {
    use super::*;

    /// AUDREP-45: a corrupt/oversized persisted payload must fail cleanly,
    /// never panic. Covers truncated buffers and oversized length prefixes
    /// that postcard would otherwise feed to `Vec::with_capacity`.
    #[test]
    fn deserialize_node_payload_rejects_malformed_input() {
        // Truncated buffer: valid start, cut off mid-structure.
        let valid = postcard::to_allocvec(&NodeMetadata {
            relational: std::collections::BTreeMap::new(),
            edges: vec![crate::node::Edge::new(1, 0)],
            created_by_txn: 1,
            deleted_by_txn: None,
        })
        .unwrap();
        let truncated = &valid[..valid.len() - 3];
        assert!(
            deserialize_node_payload::<NodeMetadata>(truncated, "node metadata").is_err(),
            "truncated payload must error, not panic"
        );

        // Oversized length prefix: varint claims a huge Vec length in a tiny buffer.
        // Layout: relational map len (0) then edges vec len as a 10-byte varint.
        let mut crafted = vec![0u8]; // relational: empty map
        crafted.extend_from_slice(&[0xFF; 10]); // edges vec len = huge
        assert!(
            deserialize_node_payload::<NodeMetadata>(&crafted, "node metadata").is_err(),
            "oversized length prefix must error, not panic"
        );
    }

    /// AUDREP-45: exercises the `MAX_PERSISTED_NODE_BYTES` guard branch exactly
    /// (buffer beyond the cap is rejected before postcard can act on it) and
    /// proves a legitimate payload inside the cap still deserializes.
    #[test]
    fn deserialize_node_payload_cap_guard_rejects_and_ok_within_cap() {
        // Buffer larger than the cap → the guard fires (not postcard, not panic).
        let oversized = vec![0u8; MAX_PERSISTED_NODE_BYTES + 1024];
        let err = match deserialize_node_payload::<NodeMetadata>(&oversized, "node metadata") {
            Ok(_) => panic!("oversized payload must be rejected by the cap guard"),
            Err(e) => e,
        };
        assert!(
            err.to_string().contains("exceeds"),
            "guard error should mention the byte cap, got: {err}"
        );

        // Payload within the cap → deserializes normally.
        let meta = NodeMetadata {
            relational: std::collections::BTreeMap::new(),
            edges: vec![crate::node::Edge::new(7, 0)],
            created_by_txn: 3,
            deleted_by_txn: Some(9),
        };
        let bytes = postcard::to_allocvec(&meta).unwrap();
        let decoded: NodeMetadata =
            deserialize_node_payload(&bytes, "node metadata").expect("within cap must deserialize");
        assert_eq!(decoded.created_by_txn, 3);
        assert_eq!(decoded.deleted_by_txn, Some(9));
        assert_eq!(decoded.edges.len(), 1);
    }
}
