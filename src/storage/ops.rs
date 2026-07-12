//! Low-level storage operations: node serialization, backend I/O, partition resolution.

use crate::backend::{BackendPartition, StorageBackend};
use crate::error::{Result, VantaError};
use crate::node::{DiskNodeHeader, UnifiedNode};
use crate::storage::vfile::VantaFile;
use std::sync::Arc;
use zerocopy::IntoBytes;

/// Serialized metadata stored per node in the KV backend.
#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct NodeMetadata {
    /// Relational field values attached to the node.
    pub relational: crate::node::RelFields,
    /// Graph edges originating from the node.
    pub edges: Vec<crate::node::Edge>,
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

/// Insert a node's metadata into the KV backend under the given column family.
pub(crate) fn insert_node_to_backend(
    backend: &Arc<dyn StorageBackend>,
    node: &UnifiedNode,
    cf_name: &str,
) -> Result<()> {
    let partition = partition_from_cf_name(cf_name)?;
    let key = node.id.to_le_bytes();
    let metadata = NodeMetadata {
        relational: node.relational.clone(),
        edges: node.edges.clone(),
    };
    let val = postcard::to_allocvec(&metadata)
        .map_err(|e| VantaError::SerializationError(Box::new(e)))?;
    backend.put(partition, &key, &val)?;
    Ok(())
}

/// Reject paths containing `..` components or absolute paths (when relative expected)
/// to prevent directory traversal.
pub(crate) fn prevent_path_traversal(path: &str) -> Result<()> {
    use std::path::Component;
    let p = std::path::Path::new(path);
    let mut components = p.components().peekable();

    // Reject absolute paths — all storage paths should be relative
    if let Some(Component::RootDir) = components.peek() {
        return Err(VantaError::ValidationError {
            field: "path".into(),
            reason: format!("Path '{path}' is absolute — only relative paths are allowed"),
        });
    }

    // Reject Windows prefix paths like \\?\ or C:\
    #[cfg(windows)]
    if let Some(Component::Prefix(_)) = components.peek() {
        return Err(VantaError::ValidationError {
            field: "path".into(),
            reason: format!("Path '{path}' has a Windows prefix — rejected for security"),
        });
    }

    for component in components {
        if component == Component::ParentDir {
            return Err(VantaError::ValidationError {
                field: "path".into(),
                reason: format!("Path '{path}' contains '..' traversal — rejected for security"),
            });
        }
    }
    Ok(())
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
        "internal_metadata" => Ok(BackendPartition::InternalMetadata),
        other => Err(VantaError::InvalidInput(format!(
            "Unknown column family: '{}'",
            other
        ))),
    }
}
