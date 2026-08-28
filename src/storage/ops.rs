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
///
/// ADR-032: persists all `VectorRepresentations` variants (Full, Binary, Turbo, SQ8) using
/// a 4-bit kind field in `flags` (bits 10-13, `NodeFlags::VECTOR_KIND_*`) and a
/// kind-dependent `vector_len` / payload layout. Legacy files with kind=0 and len>0 are
/// read as Full for compat (see readers).
pub(crate) fn write_node_to_vstore(vstore: &mut VantaFile, node: &UnifiedNode) -> Result<u64> {
    let offset = vstore.write_cursor;
    let header_size = std::mem::size_of::<DiskNodeHeader>() as u64;
    // ADR-032: dispatch payload size / kind from the in-memory representation
    let (kind, vec_len_u32, payload_len): (u32, u32, u64) = match &node.vector {
        crate::node::VectorRepresentations::Full(v) => {
            let len = v.len();
            if len > u32::MAX as usize {
                return Err(VantaError::ResourceLimit(format!(
                    "node {} vector_len {} exceeds u32",
                    node.id, len
                )));
            }
            (
                crate::node::NodeFlags::VECTOR_KIND_FULL,
                len as u32,
                (len * 4) as u64,
            )
        }
        crate::node::VectorRepresentations::Binary(b) => {
            let len = b.len();
            if len > u32::MAX as usize {
                return Err(VantaError::ResourceLimit(format!(
                    "node {} Binary len {} exceeds u32",
                    node.id, len
                )));
            }
            (
                crate::node::NodeFlags::VECTOR_KIND_BINARY,
                len as u32,
                (len * 8) as u64,
            )
        }
        crate::node::VectorRepresentations::Turbo(t) => {
            let len = t.len();
            if len > u32::MAX as usize {
                return Err(VantaError::ResourceLimit(format!(
                    "node {} Turbo len {} exceeds u32",
                    node.id, len
                )));
            }
            (
                crate::node::NodeFlags::VECTOR_KIND_TURBO,
                len as u32,
                len as u64,
            )
        }
        crate::node::VectorRepresentations::SQ8(d, _) => {
            let len = d.len();
            if len > u32::MAX as usize {
                return Err(VantaError::ResourceLimit(format!(
                    "node {} SQ8 len {} exceeds u32",
                    node.id, len
                )));
            }
            // N i8 bytes + 4 bytes scale tail
            (
                crate::node::NodeFlags::VECTOR_KIND_SQ8,
                len as u32,
                len as u64 + 4,
            )
        }
        crate::node::VectorRepresentations::MmapFull(_) => {
            if let Some(slice) = node.vector.as_f32_slice() {
                let len = slice.len();
                if len > u32::MAX as usize {
                    return Err(VantaError::ResourceLimit(format!(
                        "node {} MmapFull len {} exceeds u32",
                        node.id, len
                    )));
                }
                (
                    crate::node::NodeFlags::VECTOR_KIND_FULL,
                    len as u32,
                    (len * 4) as u64,
                )
            } else {
                (crate::node::NodeFlags::VECTOR_KIND_NONE, 0, 0)
            }
        }
        crate::node::VectorRepresentations::None => {
            (crate::node::NodeFlags::VECTOR_KIND_NONE, 0, 0)
        }
    };
    let total_needed = offset + header_size + payload_len;
    if total_needed > vstore.size {
        // ponytail: saturating to avoid overflow if size > 2^63 (already past sane limits)
        let new_size = (vstore.size.saturating_mul(2)).max(total_needed.saturating_add(4096));
        vstore.grow_to(new_size)?;
    }
    let mut header = DiskNodeHeader::new(node.id);
    header.vector_offset = offset + header_size;
    header.vector_len = vec_len_u32;
    header.flags = crate::node::NodeFlags::with_vector_kind(node.flags.0, kind);
    header.bitset = node.bitset.to_u128();
    header.confidence_score = node.confidence_score;
    header.importance = node.importance;
    header.tier = match node.tier {
        crate::node::NodeTier::Hot => 1u8,
        crate::node::NodeTier::Cold => 0u8,
    };
    // ERR-029: `edge_count` is a u16 field in the fixed 64-byte DiskNodeHeader.
    // `as u16` silently truncates, so a node with >65,535 edges used to wrap
    // (e.g. 65,536 → 0) and corrupt the persisted header. The header layout is
    // on-disk format; widening the field would break every existing file, so
    // fail loudly instead of persisting a corrupt count.
    let edge_count = node.edges.len();
    if edge_count > u16::MAX as usize {
        return Err(VantaError::ResourceLimit(format!(
            "node {} has {edge_count} edges, exceeding the DiskNodeHeader u16 edge_count limit of {}",
            node.id, u16::MAX
        )));
    }
    header.edge_count = edge_count as u16;
    vstore.write_header(offset, &header)?;
    if payload_len > 0 {
        let dst_range =
            (offset + header_size) as usize..(offset + header_size + payload_len) as usize;
        let mmap = vstore.mmap_bytes_mut()?;
        match &node.vector {
            crate::node::VectorRepresentations::Full(v) => {
                let vec_bytes = v.as_bytes();
                mmap[dst_range].copy_from_slice(vec_bytes);
            }
            crate::node::VectorRepresentations::Binary(b) => {
                // SAFETY: &[u64] to &[u8] via raw parts is valid for LE copy; length checked via payload_len
                let src =
                    unsafe { std::slice::from_raw_parts(b.as_ptr() as *const u8, b.len() * 8) };
                mmap[dst_range].copy_from_slice(src);
            }
            crate::node::VectorRepresentations::Turbo(t) => {
                mmap[dst_range].copy_from_slice(t);
            }
            crate::node::VectorRepresentations::SQ8(d, scale) => {
                let n = d.len();
                // SAFETY: &[i8] to &[u8] is bitwise identical
                let src = unsafe { std::slice::from_raw_parts(d.as_ptr() as *const u8, n) };
                mmap[(offset + header_size) as usize..(offset + header_size + n as u64) as usize]
                    .copy_from_slice(src);
                mmap[(offset + header_size + n as u64) as usize
                    ..(offset + header_size + payload_len) as usize]
                    .copy_from_slice(&scale.to_le_bytes());
            }
            crate::node::VectorRepresentations::MmapFull(_) => {
                if let Some(slice) = node.vector.as_f32_slice() {
                    let vec_bytes = slice.as_bytes();
                    mmap[dst_range].copy_from_slice(vec_bytes);
                }
            }
            crate::node::VectorRepresentations::None => {}
        }
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
    use crate::storage::engine::STORAGE_ALIGNMENT;

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

    /// ERR-029: a node with more than 65,535 edges must be rejected at the
    /// persistence boundary (ResourceLimit), never silently wrapped by the
    /// `as u16` cast on the `DiskNodeHeader::edge_count` field. Uses 65,537
    /// edges: if the old truncating behavior ran, the persisted count would
    /// wrap to 1 — the test proves nothing is persisted on failure.
    #[test]
    fn write_node_to_vstore_rejects_over_u16_max_edges() {
        let mut vstore = VantaFile::create_in_memory(4096);
        let mut node = UnifiedNode::new(42);
        node.edges = vec![crate::node::Edge::new(1, 0); u16::MAX as usize + 2];

        let err = write_node_to_vstore(&mut vstore, &node).unwrap_err();
        assert!(
            err.to_string().contains("65535"),
            "error must mention the u16 edge_count limit, got: {err}"
        );

        // Failed write leaves the store untouched: cursor at the initial
        // position and the header area still zeroed (edge_count 0, not the
        // wrapped count).
        assert_eq!(
            vstore.write_cursor, STORAGE_ALIGNMENT,
            "failed write must not advance the cursor"
        );
        let header = vstore.read_header(STORAGE_ALIGNMENT);
        assert_eq!(
            header.map(|h| h.edge_count),
            Some(0),
            "no wrapped edge_count may be persisted"
        );
    }

    /// ERR-029: the valid boundary (exactly 65,535 edges) still persists
    /// without truncation.
    #[test]
    fn write_node_to_vstore_persists_u16_max_edges() {
        let mut vstore = VantaFile::create_in_memory(4096);
        let mut node = UnifiedNode::new(7);
        node.edges = vec![crate::node::Edge::new(1, 0); u16::MAX as usize];

        let offset = write_node_to_vstore(&mut vstore, &node).unwrap();
        let header = vstore.read_header(offset).expect("header must be readable");
        assert_eq!(header.edge_count, u16::MAX);
    }

    /// ADR-032: Binary vector must persist kind + payload and round-trip via header.
    #[test]
    fn write_node_to_vstore_persists_binary_vector() {
        let mut vstore = VantaFile::create_in_memory(4096);
        let mut node = UnifiedNode::new(1001);
        let data: Box<[u64]> =
            vec![0xDEADBEEFu64, 0xCAFE1234u64, 0x0123456789ABCDEFu64].into_boxed_slice();
        node.vector = crate::node::VectorRepresentations::Binary(data.clone());
        node.flags.set(crate::node::NodeFlags::HAS_VECTOR);

        let offset = write_node_to_vstore(&mut vstore, &node).unwrap();
        let header = vstore.read_header(offset).expect("header readable");
        assert_eq!(
            crate::node::NodeFlags::vector_kind(header.flags),
            crate::node::NodeFlags::VECTOR_KIND_BINARY
        );
        assert_eq!(header.vector_len as usize, data.len());
        // payload must be M*8 bytes after header
        let start = header.vector_offset as usize;
        let end = start + data.len() * 8;
        let raw = &vstore.mmap_bytes()[start..end];
        let (_, u64_slice, _) = unsafe { raw.align_to::<u64>() };
        assert_eq!(u64_slice, data.as_ref());
    }

    /// ADR-032: Turbo vector must persist kind + payload.
    #[test]
    fn write_node_to_vstore_persists_turbo_vector() {
        let mut vstore = VantaFile::create_in_memory(4096);
        let mut node = UnifiedNode::new(1002);
        let data: Box<[u8]> = vec![0xAB, 0xCD, 0xEF, 0x12, 0x34].into_boxed_slice();
        node.vector = crate::node::VectorRepresentations::Turbo(data.clone());
        node.flags.set(crate::node::NodeFlags::HAS_VECTOR);

        let offset = write_node_to_vstore(&mut vstore, &node).unwrap();
        let header = vstore.read_header(offset).expect("header readable");
        assert_eq!(
            crate::node::NodeFlags::vector_kind(header.flags),
            crate::node::NodeFlags::VECTOR_KIND_TURBO
        );
        assert_eq!(header.vector_len as usize, data.len());
        let start = header.vector_offset as usize;
        let end = start + data.len();
        assert_eq!(&vstore.mmap_bytes()[start..end], data.as_ref());
    }

    /// ADR-032: SQ8 vector must persist kind + i8 payload + scale tail.
    #[test]
    fn write_node_to_vstore_persists_sq8_vector() {
        let mut vstore = VantaFile::create_in_memory(4096);
        let mut node = UnifiedNode::new(1003);
        let data: Box<[i8]> = vec![10, -20, 30, -40, 127, -127].into_boxed_slice();
        let scale: f32 = 2.5;
        node.vector = crate::node::VectorRepresentations::SQ8(data.clone(), scale);
        node.flags.set(crate::node::NodeFlags::HAS_VECTOR);

        let offset = write_node_to_vstore(&mut vstore, &node).unwrap();
        let header = vstore.read_header(offset).expect("header readable");
        assert_eq!(
            crate::node::NodeFlags::vector_kind(header.flags),
            crate::node::NodeFlags::VECTOR_KIND_SQ8
        );
        assert_eq!(header.vector_len as usize, data.len());
        let n = data.len();
        let start = header.vector_offset as usize;
        let payload = &vstore.mmap_bytes()[start..start + n + 4];
        let raw_i8: Vec<i8> = payload[..n].iter().map(|&b| b as i8).collect();
        assert_eq!(raw_i8.as_slice(), data.as_ref());
        let scale_bytes: [u8; 4] = payload[n..n + 4].try_into().unwrap();
        assert!((f32::from_le_bytes(scale_bytes) - scale).abs() < f32::EPSILON);
    }

    /// ADR-032: Full vector still persists as before (kind=FULL, len*4).
    #[test]
    fn write_node_to_vstore_persists_full_vector() {
        let mut vstore = VantaFile::create_in_memory(4096);
        let mut node = UnifiedNode::new(1004);
        let data = vec![1.0f32, 2.5, -3.75, 0.0];
        node.vector = crate::node::VectorRepresentations::Full(data.clone());
        node.flags.set(crate::node::NodeFlags::HAS_VECTOR);

        let offset = write_node_to_vstore(&mut vstore, &node).unwrap();
        let header = vstore.read_header(offset).expect("header readable");
        assert_eq!(
            crate::node::NodeFlags::vector_kind(header.flags),
            crate::node::NodeFlags::VECTOR_KIND_FULL
        );
        assert_eq!(header.vector_len as usize, data.len());
        let start = header.vector_offset as usize;
        let end = start + data.len() * 4;
        let raw = &vstore.mmap_bytes()[start..end];
        let (_, f32_slice, _) = unsafe { raw.align_to::<f32>() };
        assert_eq!(f32_slice, data.as_slice());
    }

    /// ADR-032: None vector persists as kind=NONE with len 0 and no payload.
    #[test]
    fn write_node_to_vstore_persists_none_vector() {
        let mut vstore = VantaFile::create_in_memory(4096);
        let node = UnifiedNode::new(1005); // vector = None
        let offset = write_node_to_vstore(&mut vstore, &node).unwrap();
        let header = vstore.read_header(offset).expect("header readable");
        assert_eq!(
            crate::node::NodeFlags::vector_kind(header.flags),
            crate::node::NodeFlags::VECTOR_KIND_NONE
        );
        assert_eq!(header.vector_len, 0);
    }
}
