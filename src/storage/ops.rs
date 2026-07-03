use crate::backend::{BackendPartition, StorageBackend};
use crate::error::{Result, VantaError};
use crate::index::CPIndex;
use crate::node::{DiskNodeHeader, UnifiedNode};
use crate::storage::vfile::VantaFile;
use std::sync::Arc;
use zerocopy::IntoBytes;

const FLAG_TOMBSTONE: u32 = 0x8;

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct NodeMetadata {
    pub relational: crate::node::RelFields,
    pub edges: Vec<crate::node::Edge>,
}

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
    header.bitset = node.bitset;
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

#[allow(dead_code)]
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
        .map_err(|e| VantaError::SerializationError(e.to_string()))?;
    backend.put(partition, &key, &val)?;
    Ok(())
}

#[allow(dead_code)]
pub(crate) fn get_node_from_backend(
    backend: &dyn StorageBackend,
    id: u64,
    hnsw: &CPIndex,
    vstore: &VantaFile,
) -> Result<Option<UnifiedNode>> {
    let key = id.to_le_bytes();
    let metadata_res = match backend.get(BackendPartition::Default, &key)? {
        Some(r) => r,
        None => return Ok(None),
    };
    let metadata: NodeMetadata = postcard::from_bytes(&metadata_res)
        .map_err(|e| VantaError::SerializationError(e.to_string()))?;
    let index_node = match hnsw.nodes.get(&id) {
        Some(n) => n,
        None => return Ok(None),
    };
    let header = match vstore.read_header(index_node.storage_offset) {
        Some(h) => h,
        None => return Ok(None),
    };
    if (header.flags & FLAG_TOMBSTONE) != 0 {
        return Ok(None);
    }
    let vec_start = header.vector_offset as usize;
    let vec_end = vec_start + (header.vector_len as usize * 4);
    if vec_end > vstore.size as usize {
        return Ok(None);
    }
    let vec_bytes = &vstore.mmap_bytes()[vec_start..vec_end];
    let f32_vec: &[f32] = unsafe {
        std::slice::from_raw_parts(vec_bytes.as_ptr() as *const f32, header.vector_len as usize)
    };
    let mut node = UnifiedNode::new(id);
    node.bitset = header.bitset;
    node.vector = crate::node::VectorRepresentations::Full(f32_vec.to_vec());
    node.relational = metadata.relational;
    node.edges = metadata.edges;
    node.confidence_score = header.confidence_score;
    node.importance = header.importance;
    node.tier = if header.tier == 1 {
        crate::node::NodeTier::Hot
    } else {
        crate::node::NodeTier::Cold
    };
    node.flags = crate::node::NodeFlags(header.flags);
    Ok(Some(node))
}

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
