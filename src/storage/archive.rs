//! HNSW index rebuild, layout compaction, and graph traversal utilities.

use crate::error::{Result, VantaError};
use crate::index::CPIndex;
use crate::node::{DiskNodeHeader, FilterBitset};
use crate::storage::vfile::{MmapOptions, VantaFile};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::OpenOptions;
use std::path::PathBuf;
use web_time::Instant;
use zerocopy::IntoBytes;

use crate::storage::engine::{FLAG_TOMBSTONE, STORAGE_ALIGNMENT};
const BFS_QUEUE_CAPACITY: usize = 1024;

/// Rewrite the VantaFile with nodes in BFS order, returning the new offset map and file size.
pub(crate) fn compact_layout(
    vstore: &mut VantaFile,
    hnsw: &CPIndex,
    bfs_order: &[u128],
    header_size: u64,
) -> Result<(HashMap<u128, u64>, u64)> {
    // In-memory VantaFile has no disk backing to compact — return a trivial
    // offset map that preserves existing offsets (CODE-010).
    if vstore.file.is_none() {
        let offset_map: HashMap<u128, u64> = bfs_order
            .iter()
            .filter_map(|&id| hnsw.nodes.get(&id).map(|n| (id, n.storage_offset)))
            .collect();
        return Ok((offset_map, vstore.write_cursor));
    }
    if bfs_order.is_empty() {
        return Err(VantaError::ValidationError {
            field: "bfs_order".into(),
            reason: "BFS order is empty — refusing to compact (would destroy the database)".into(),
        });
    }
    let mut new_file_size: u64 = 64;
    for &node_id in bfs_order {
        if let Some(node_ref) = hnsw.nodes.get(&node_id) {
            let offset = node_ref.storage_offset;
            if let Some(old_header) = vstore.read_header(offset) {
                let vec_size = (old_header.vector_len as u64 * 4 + 63) & !63;
                new_file_size += header_size + vec_size;
            }
        }
    }
    new_file_size = (new_file_size + 4095) & !4095;

    let vstore_path = vstore.path.clone();
    let tmp_filename = format!(
        "{}.tmp",
        vstore_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("vector_store.vanta")
    );
    let tmp_path = vstore_path.with_file_name(tmp_filename);

    let tmp_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&tmp_path)
        .map_err(VantaError::IoError)?;
    tmp_file
        .set_len(new_file_size)
        .map_err(VantaError::IoError)?;

    // SAFETY: tmp_file is a valid, open file handle with set_len() called beforehand.
    // MmapMut::map_mut() requires the underlying file to be writable and have a valid size;
    // both hold here. The returned mmap is valid for the file's lifetime, which exceeds tmp_mmap.
    let mut tmp_mmap = unsafe {
        MmapOptions::new()
            .map_mut(&tmp_file)
            .map_err(VantaError::IoError)?
    };

    let mut new_offset_map: HashMap<u128, u64> = HashMap::with_capacity(bfs_order.len());
    let mut write_cursor: u64 = STORAGE_ALIGNMENT;

    for &node_id in bfs_order {
        if let Some(node_ref) = hnsw.nodes.get(&node_id) {
            let old_offset = node_ref.storage_offset;
            let old_header = match vstore.read_header(old_offset) {
                Some(h) => h,
                None => continue,
            };
            if (old_header.flags & FLAG_TOMBSTONE) != 0 {
                continue;
            }
            let vec_len = old_header.vector_len as u64;
            let vec_size_aligned = (vec_len * 4 + 63) & !63;
            let new_node_offset = write_cursor;
            let new_vec_offset = new_node_offset + header_size;
            let end = new_vec_offset + vec_size_aligned;
            if end > new_file_size {
                drop(tmp_mmap);
                tmp_file.set_len(end + 4096).map_err(VantaError::IoError)?;
                // SAFETY: tmp_file was extended via set_len() before this call, so the
                // file's size covers the new mapping. The previous mmap was dropped, so
                // there is no conflicting mapping on the same region.
                tmp_mmap = unsafe {
                    MmapOptions::new()
                        .map_mut(&tmp_file)
                        .map_err(VantaError::IoError)?
                };
            }
            let old_data = vstore.mmap_bytes();
            let src_start = old_offset as usize;
            let src_end = src_start + header_size as usize + vec_size_aligned as usize;
            let copy_len = (header_size + vec_size_aligned) as usize;
            tmp_mmap[write_cursor as usize..(write_cursor as usize + copy_len)]
                .copy_from_slice(&old_data[src_start..src_end.min(old_data.len())]);
            let mut new_header = old_header;
            new_header.vector_offset = new_vec_offset;
            tmp_mmap[write_cursor as usize..(write_cursor as usize + header_size as usize)]
                .copy_from_slice(new_header.as_bytes());
            new_offset_map.insert(node_id, new_node_offset);
            write_cursor += header_size + vec_size_aligned;
        }
    }

    std::fs::rename(&tmp_path, &vstore_path).map_err(VantaError::IoError)?;
    vstore.replace_backing_file(new_file_size)?;
    vstore.write_cursor = write_cursor;
    vstore.save_cursor()?;
    Ok((new_offset_map, new_file_size))
}

/// BFS traversal of the HNSW graph starting from the entry point, returning node IDs in visit order.
pub(crate) fn traverse_graph(hnsw: &CPIndex, entry_point_id: u128) -> Vec<u128> {
    let total_nodes = hnsw.nodes.len();
    let mut bfs_order: Vec<u128> = Vec::with_capacity(total_nodes);
    let mut visited: HashSet<u128> = HashSet::with_capacity(total_nodes);
    let mut queue: VecDeque<u128> = VecDeque::with_capacity(total_nodes.min(BFS_QUEUE_CAPACITY));
    queue.push_back(entry_point_id);
    visited.insert(entry_point_id);
    while let Some(node_id) = queue.pop_front() {
        bfs_order.push(node_id);
        if let Some(node_ref) = hnsw.nodes.get(&node_id) {
            if let Some(layer0) = node_ref.neighbors.first() {
                for &nid in layer0 {
                    if visited.insert(nid) {
                        queue.push_back(nid);
                    }
                }
            }
        }
    }
    for entry in hnsw.nodes.iter() {
        if visited.insert(*entry.key()) {
            bfs_order.push(*entry.key());
        }
    }
    bfs_order
}

/// Update each node's storage offset in the HNSW index after compaction.
pub(crate) fn reindex_nodes(hnsw: &CPIndex, new_offsets: &HashMap<u128, u64>) {
    for (&node_id, &new_offset) in new_offsets {
        if let Some(mut node_ref) = hnsw.nodes.get_mut(&node_id) {
            node_ref.storage_offset = new_offset;
        }
    }
}

/// Create a new CPIndex with the same backend configuration (mmap or in-memory) as the existing one.
pub(crate) fn fresh_index_like(existing: &CPIndex, index_path: PathBuf) -> CPIndex {
    let config = existing.config.clone();
    if existing.backend.is_mmap() {
        let mut idx = CPIndex::with_backend(crate::index::IndexBackend::new_mmap(index_path));
        idx.config = config;
        idx
    } else {
        CPIndex::new_with_config(config)
    }
}

/// Rebuild the entire HNSW index by scanning all nodes from the VantaFile.
pub(crate) fn rebuild_hnsw_from_vstore(
    hnsw: &mut CPIndex,
    vstore: &VantaFile,
    index_path: PathBuf,
) -> Result<crate::storage::IndexRebuildReport> {
    let started = Instant::now();
    let mut cursor = STORAGE_ALIGNMENT;
    let mut scanned_nodes = 0u64;
    let mut indexed_vectors = 0u64;
    let mut skipped_tombstones = 0u64;
    let header_size = std::mem::size_of::<DiskNodeHeader>() as u64;

    while cursor + header_size <= vstore.write_cursor {
        if let Some(header) = vstore.read_header(cursor) {
            if header.id != 0 {
                scanned_nodes += 1;
                if (header.flags & FLAG_TOMBSTONE) != 0 {
                    skipped_tombstones += 1;
                } else {
                    let vec_data = if header.vector_len > 0 {
                        let start = header.vector_offset as usize;
                        let end = start + (header.vector_len as usize * 4);
                        if end <= vstore.size as usize {
                            indexed_vectors += 1;
                            let slice = &vstore.mmap_bytes()[start..end];
                            debug_assert_eq!(
                                slice.as_ptr().align_offset(4),
                                0,
                                "f32 vector must be 4-byte aligned"
                            );
                            crate::node::VectorRepresentations::Full(
                                unsafe {
                                    std::slice::from_raw_parts(
                                        slice.as_ptr() as *const f32,
                                        header.vector_len as usize,
                                    )
                                }
                                .to_vec(),
                            )
                        } else {
                            crate::node::VectorRepresentations::None
                        }
                    } else {
                        crate::node::VectorRepresentations::None
                    };
                    hnsw.add(
                        header.id,
                        FilterBitset::from_u128(header.bitset),
                        vec_data,
                        cursor,
                    );
                }
            }
            cursor += header_size + ((header.vector_len as u64 * 4 + 63) & !63);
        } else {
            cursor += STORAGE_ALIGNMENT;
        }
    }
    Ok(crate::storage::IndexRebuildReport {
        scanned_nodes,
        indexed_vectors,
        skipped_tombstones,
        duration_ms: started.elapsed().as_millis() as u64,
        index_path,
        success: true,
    })
}
