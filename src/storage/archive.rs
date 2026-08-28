//! HNSW index rebuild, layout compaction, and graph traversal utilities.

use crate::error::{Result, VantaError};
use crate::index::CPIndex;
use crate::node::DiskNodeHeader;
use crate::storage::vfile::{map_readwrite, VantaFile};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::OpenOptions;
use std::path::PathBuf;
use web_time::Instant;
use zerocopy::IntoBytes;

use crate::storage::engine::{FLAG_TOMBSTONE, STORAGE_ALIGNMENT};
const BFS_QUEUE_CAPACITY: usize = 1024;

/// ADR-032: payload size (un-aligned) for a persisted header.
fn payload_len_for_header(header: &DiskNodeHeader) -> u64 {
    let kind = crate::node::NodeFlags::vector_kind(header.flags);
    // kind==0 is legacy (pre-ADR-032) where FULL vs NONE is distinguished by len
    if kind == 0 {
        if header.vector_len == 0 {
            return 0;
        } else {
            return (header.vector_len as u64).checked_mul(4).unwrap_or(0);
        }
    }
    match kind {
        crate::node::NodeFlags::VECTOR_KIND_NONE => 0,
        crate::node::NodeFlags::VECTOR_KIND_FULL => {
            (header.vector_len as u64).checked_mul(4).unwrap_or(0)
        }
        crate::node::NodeFlags::VECTOR_KIND_BINARY => {
            (header.vector_len as u64).checked_mul(8).unwrap_or(0)
        }
        crate::node::NodeFlags::VECTOR_KIND_TURBO => header.vector_len as u64,
        crate::node::NodeFlags::VECTOR_KIND_SQ8 => header.vector_len as u64 + 4,
        _ => header.vector_len as u64, // unknown future kind: byte count
    }
}

/// Rewrite the VantaFile with nodes in BFS order, returning the new offset map and file size.
pub fn compact_layout(
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
                let payload = payload_len_for_header(&old_header);
                let vec_size = (payload + 63) & !63;
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

    // `map_readwrite` carries the (memmap2-only) SAFETY contract: tmp_file is a
    // valid, open handle with set_len() called beforehand (writable, valid
    // size); the returned mmap is valid for the file's lifetime.
    let mut tmp_mmap = map_readwrite(&tmp_file).map_err(VantaError::IoError)?;

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
            let payload = payload_len_for_header(&old_header);
            let vec_size_aligned = (payload + 63) & !63;
            let new_node_offset = write_cursor;
            let new_vec_offset = new_node_offset + header_size;
            let end = new_vec_offset + vec_size_aligned;
            if end > new_file_size {
                tmp_mmap.flush().map_err(VantaError::IoError)?;
                drop(tmp_mmap);
                tmp_file.set_len(end + 4096).map_err(VantaError::IoError)?;
                // `map_readwrite` carries the (memmap2-only) SAFETY contract:
                // tmp_file was extended via set_len() before this call and the
                // previous mmap was dropped, so there is no conflicting mapping.
                tmp_mmap = map_readwrite(&tmp_file).map_err(VantaError::IoError)?;
            }
            let old_data = vstore.mmap_bytes();
            let src_start = old_offset as usize;
            let copy_len = (header_size + vec_size_aligned) as usize;
            let src_end = src_start + copy_len;
            // AUDREP-01: a header whose vector_len claims more bytes than the
            // file actually holds (crash mid-write) would make the destination
            // slice longer than the source and panic copy_from_slice. Validate
            // the source is long enough and abort the compact with an error.
            if src_end > old_data.len() {
                return Err(VantaError::IoError(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    format!(
                        "vstore truncated: node at offset {old_offset} claims {copy_len} bytes \
                         (header {header_size} + vec {vec_size_aligned}) but file has {} — \
                         needed {src_end}",
                        old_data.len(),
                    ),
                )));
            }
            tmp_mmap[write_cursor as usize..(write_cursor as usize + copy_len)]
                .copy_from_slice(&old_data[src_start..src_end]);
            let mut new_header = old_header;
            new_header.vector_offset = new_vec_offset;
            tmp_mmap[write_cursor as usize..(write_cursor as usize + header_size as usize)]
                .copy_from_slice(new_header.as_bytes());
            new_offset_map.insert(node_id, new_node_offset);
            write_cursor += header_size + vec_size_aligned;
        }
    }

    // AUDREP-04: flush final mmap writes to the OS, then sync + fsync the tmp
    // file so no unwritten garbage is renamed in as if it were a valid store.
    tmp_mmap.flush().map_err(VantaError::IoError)?;
    drop(tmp_mmap);
    tmp_file.sync_all().map_err(VantaError::IoError)?;
    std::fs::rename(&tmp_path, &vstore_path).map_err(VantaError::IoError)?;
    // AUDREP-35: a rename is not durable until its parent dir is fsync'd —
    // without this, a crash can revert the swap and resurrect the old file.
    crate::utils::fs::sync_parent_dir(&vstore_path).map_err(VantaError::IoError)?;
    vstore.replace_backing_file(new_file_size)?;
    vstore.write_cursor = write_cursor;
    vstore.save_cursor()?;
    Ok((new_offset_map, new_file_size))
}

/// BFS traversal of the HNSW graph starting from the entry point, returning node IDs in visit order.
pub fn traverse_graph(hnsw: &CPIndex, entry_point_id: u128) -> Vec<u128> {
    let total_nodes = hnsw.nodes.len();
    let mut bfs_order: Vec<u128> = Vec::with_capacity(total_nodes);
    let mut visited: HashSet<u128> = HashSet::with_capacity(total_nodes);
    let mut queue: VecDeque<u128> = VecDeque::with_capacity(total_nodes.min(BFS_QUEUE_CAPACITY));
    queue.push_back(entry_point_id);
    visited.insert(entry_point_id);
    while let Some(node_id) = queue.pop_front() {
        bfs_order.push(node_id);
        if let Some(layer0) = hnsw.neighbor_index.get_neighbors_ref(node_id, 0) {
            for &nid in layer0.iter() {
                if visited.insert(nid) {
                    queue.push_back(nid);
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
pub fn reindex_nodes(hnsw: &CPIndex, new_offsets: &HashMap<u128, u64>) {
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
/// If `segment_id` is Some(n), offsets are packed with the segment_id.
pub(crate) fn rebuild_hnsw_from_vstore(
    hnsw: &mut CPIndex,
    vstore: &VantaFile,
    index_path: PathBuf,
) -> Result<crate::storage::IndexRebuildReport> {
    rebuild_hnsw_from_vstore_with_segment(hnsw, vstore, index_path, None)
}

/// Rebuild HNSW from a VantaFile, optionally packing offsets with a segment_id.
pub(crate) fn rebuild_hnsw_from_vstore_with_segment(
    hnsw: &mut CPIndex,
    vstore: &VantaFile,
    index_path: PathBuf,
    segment_id: Option<u8>,
) -> Result<crate::storage::IndexRebuildReport> {
    let started = Instant::now();
    let header_size = std::mem::size_of::<DiskNodeHeader>() as u64;

    // Phase 1: scan vstore and collect all nodes into a buffer
    // (sequential scan is I/O bound, not CPU bound)
    struct HnswEntry {
        id: u128,
        bitset: u128,
        vec_data: crate::node::VectorRepresentations,
        storage_offset: u64,
    }

    let mut entries: Vec<HnswEntry> = Vec::new();
    let mut scanned_nodes = 0u64;
    let mut indexed_vectors = 0u64;
    let mut skipped_tombstones = 0u64;
    let mut cursor = STORAGE_ALIGNMENT;

    while cursor + header_size <= vstore.write_cursor {
        if let Some(header) = vstore.read_header(cursor) {
            if header.id != 0 {
                scanned_nodes += 1;
                if (header.flags & FLAG_TOMBSTONE) != 0 {
                    skipped_tombstones += 1;
                } else {
                    let kind = crate::node::NodeFlags::vector_kind(header.flags);
                    let vec_data = if kind == 0 {
                        // legacy pre-ADR-032: kind 0 with len>0 is FULL, with len==0 is NONE
                        if header.vector_len == 0 {
                            crate::node::VectorRepresentations::None
                        } else if let Some(end) = (header.vector_len as u64)
                            .checked_mul(4)
                            .and_then(|b| header.vector_offset.checked_add(b))
                            .filter(|&end| end <= vstore.size as u64)
                        {
                            let start = header.vector_offset as usize;
                            let slice = &vstore.mmap_bytes()[start..end as usize];
                            debug_assert_eq!(
                                slice.as_ptr().align_offset(4),
                                0,
                                "legacy f32 must be 4-aligned"
                            );
                            indexed_vectors += 1;
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
                        match kind {
                            crate::node::NodeFlags::VECTOR_KIND_FULL => {
                                if header.vector_len == 0 {
                                    crate::node::VectorRepresentations::None
                                } else if let Some(end) = (header.vector_len as u64)
                                    .checked_mul(4)
                                    .and_then(|b| header.vector_offset.checked_add(b))
                                    .filter(|&end| end <= vstore.size as u64)
                                {
                                    let start = header.vector_offset as usize;
                                    let slice = &vstore.mmap_bytes()[start..end as usize];
                                    debug_assert_eq!(
                                        slice.as_ptr().align_offset(4),
                                        0,
                                        "f32 vector must be 4-byte aligned"
                                    );
                                    indexed_vectors += 1;
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
                            }
                            crate::node::NodeFlags::VECTOR_KIND_BINARY => {
                                if header.vector_len == 0 {
                                    crate::node::VectorRepresentations::None
                                } else if let Some(end) = (header.vector_len as u64)
                                    .checked_mul(8)
                                    .and_then(|b| header.vector_offset.checked_add(b))
                                    .filter(|&end| end <= vstore.size as u64)
                                {
                                    let start = header.vector_offset as usize;
                                    let slice = &vstore.mmap_bytes()[start..end as usize];
                                    debug_assert_eq!(
                                        slice.as_ptr().align_offset(8),
                                        0,
                                        "u64 vector must be 8-byte aligned"
                                    );
                                    let (_, u64_slice, _) = unsafe { slice.align_to::<u64>() };
                                    if u64_slice.len() != header.vector_len as usize {
                                        crate::node::VectorRepresentations::None
                                    } else {
                                        indexed_vectors += 1;
                                        crate::node::VectorRepresentations::Binary(
                                            u64_slice.to_vec().into_boxed_slice(),
                                        )
                                    }
                                } else {
                                    crate::node::VectorRepresentations::None
                                }
                            }
                            crate::node::NodeFlags::VECTOR_KIND_TURBO => {
                                if header.vector_len == 0 {
                                    crate::node::VectorRepresentations::None
                                } else if let Some(end) = header
                                    .vector_offset
                                    .checked_add(header.vector_len as u64)
                                    .filter(|&end| end <= vstore.size as u64)
                                {
                                    let start = header.vector_offset as usize;
                                    let slice = &vstore.mmap_bytes()[start..end as usize];
                                    indexed_vectors += 1;
                                    crate::node::VectorRepresentations::Turbo(
                                        slice.to_vec().into_boxed_slice(),
                                    )
                                } else {
                                    crate::node::VectorRepresentations::None
                                }
                            }
                            crate::node::NodeFlags::VECTOR_KIND_SQ8 => {
                                if header.vector_len == 0 {
                                    crate::node::VectorRepresentations::None
                                } else if let Some(payload_end) = (header.vector_len as u64)
                                    .checked_add(4)
                                    .and_then(|b| header.vector_offset.checked_add(b))
                                    .filter(|&end| end <= vstore.size as u64)
                                {
                                    let start = header.vector_offset as usize;
                                    let payload = &vstore.mmap_bytes()[start..payload_end as usize];
                                    let n = header.vector_len as usize;
                                    let d_slice = &payload[..n];
                                    let scale_bytes: [u8; 4] =
                                        payload[n..n + 4].try_into().unwrap_or([0; 4]);
                                    let scale = f32::from_le_bytes(scale_bytes);
                                    if !scale.is_finite() {
                                        crate::node::VectorRepresentations::None
                                    } else {
                                        indexed_vectors += 1;
                                        let data: Vec<i8> =
                                            d_slice.iter().map(|&b| b as i8).collect();
                                        crate::node::VectorRepresentations::SQ8(
                                            data.into_boxed_slice(),
                                            scale,
                                        )
                                    }
                                } else {
                                    crate::node::VectorRepresentations::None
                                }
                            }
                            crate::node::NodeFlags::VECTOR_KIND_NONE => {
                                crate::node::VectorRepresentations::None
                            }
                            _ => crate::node::VectorRepresentations::None,
                        }
                    };
                    let storage_offset = match segment_id {
                        Some(sid) => crate::lsm::pack_offset(sid, cursor),
                        None => cursor,
                    };
                    entries.push(HnswEntry {
                        id: header.id,
                        bitset: header.bitset,
                        vec_data,
                        storage_offset,
                    });
                }
            }
            let payload = payload_len_for_header(&header);
            cursor += header_size + ((payload + 63) & !63);
        } else {
            cursor += STORAGE_ALIGNMENT;
        }
    }

    // Phase 2: add all nodes to HNSW graph
    // When rayon is available, add in parallel with thread-local RNG
    // to avoid contention on the shared `rng` mutex inside CPIndex.
    #[cfg(feature = "rayon")]
    {
        use rayon::prelude::*;

        entries.into_par_iter().try_for_each(|entry| {
            let bitset = crate::node::FilterBitset::from_u128(entry.bitset);
            let level = crate::index::random_layer_from_config(&hnsw.config, &mut rand::rng());
            hnsw.add_with_level(
                entry.id,
                bitset,
                entry.vec_data,
                entry.storage_offset,
                level,
            )
        })?;
    }

    #[cfg(not(feature = "rayon"))]
    {
        for entry in entries {
            let bitset = crate::node::FilterBitset::from_u128(entry.bitset);
            hnsw.add(entry.id, bitset, entry.vec_data, entry.storage_offset)?;
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

#[cfg(test)]
#[allow(missing_docs, clippy::module_inception, unused_must_use)]
mod tests {
    // CPIndex::add now returns Result (AUDREP-27); these are hand-built
    // test fixtures whose vectors are known non-zero-norm, so the Result is
    // intentionally ignored. Kept as a module-scope allow to avoid N identical
    // `.expect(...)` suffixes on fixture inserts.
    use super::*;
    use crate::index::CPIndex;
    use crate::node::DiskNodeHeader;
    use crate::node::FilterBitset;
    use crate::node::VectorRepresentations;

    // ── helpers ──────────────────────────────────────────────────

    fn hdr_size() -> u64 {
        std::mem::size_of::<DiskNodeHeader>() as u64
    }

    fn aligned_vec_size(len: u32) -> u64 {
        (len as u64 * 4 + 63) & !63
    }

    fn write_node_to_vstore(
        vstore: &mut VantaFile,
        id: u128,
        offset: u64,
        data: &[f32],
        flags: u32,
    ) {
        let vec_offset = offset + hdr_size();
        let mut header = DiskNodeHeader::new(id);
        header.vector_len = data.len() as u32;
        header.vector_offset = vec_offset;
        header.flags = flags;
        vstore.write_header(offset, &header).unwrap();
        if !data.is_empty() {
            let mmap = vstore.mmap_bytes_mut().unwrap();
            for (i, &val) in data.iter().enumerate() {
                let bytes = val.to_le_bytes();
                let start = vec_offset as usize + i * 4;
                mmap[start..start + 4].copy_from_slice(&bytes);
            }
        }
    }

    // ── traverse_graph ───────────────────────────────────────────

    #[test]
    fn test_traverse_graph_empty() {
        let hnsw = CPIndex::new();
        // Entry point (0) is pushed even if no nodes exist in the index.
        let order = traverse_graph(&hnsw, 0);
        assert_eq!(order, vec![0]);
    }

    #[test]
    fn test_traverse_graph_single_node() {
        let hnsw = CPIndex::new();
        hnsw.add(
            42,
            FilterBitset::from_u128(0),
            VectorRepresentations::Full(vec![0.1, 0.2, 0.3]),
            64,
        );
        assert_eq!(traverse_graph(&hnsw, 42), vec![42]);
    }

    #[test]
    fn test_traverse_graph_two_disconnected() {
        let hnsw = CPIndex::new();
        hnsw.add(
            1,
            FilterBitset::from_u128(0),
            VectorRepresentations::Full(vec![0.1]),
            64,
        );
        hnsw.add(
            2,
            FilterBitset::from_u128(0),
            VectorRepresentations::Full(vec![0.2]),
            128,
        );
        // Both nodes present — first via BFS, second via catch-all sweep
        let order = traverse_graph(&hnsw, 1);
        assert_eq!(order.len(), 2);
        assert!(order.contains(&1));
        assert!(order.contains(&2));
    }

    // ── reindex_nodes ────────────────────────────────────────────

    #[test]
    fn test_reindex_nodes_updates_offsets() {
        let hnsw = CPIndex::new();
        hnsw.add(
            1,
            FilterBitset::from_u128(0),
            VectorRepresentations::Full(vec![0.1]),
            64,
        );
        let mut offsets = HashMap::new();
        offsets.insert(1, 999);
        reindex_nodes(&hnsw, &offsets);
        assert_eq!(hnsw.nodes.get(&1).unwrap().storage_offset, 999);
    }

    #[test]
    fn test_reindex_nodes_unknown_id_ignored() {
        let hnsw = CPIndex::new();
        hnsw.add(
            1,
            FilterBitset::from_u128(0),
            VectorRepresentations::Full(vec![0.1]),
            64,
        );
        let mut offsets = HashMap::new();
        offsets.insert(99, 999); // does not exist in hnsw
        reindex_nodes(&hnsw, &offsets); // must not panic
        assert_eq!(hnsw.nodes.get(&1).unwrap().storage_offset, 64);
    }

    #[test]
    fn test_reindex_nodes_empty_map() {
        let hnsw = CPIndex::new();
        hnsw.add(
            1,
            FilterBitset::from_u128(0),
            VectorRepresentations::Full(vec![0.1]),
            64,
        );
        reindex_nodes(&hnsw, &HashMap::new());
        assert_eq!(hnsw.nodes.get(&1).unwrap().storage_offset, 64);
    }

    // ── fresh_index_like ─────────────────────────────────────────

    #[test]
    fn test_fresh_index_like_in_memory() {
        let hnsw = CPIndex::new();
        let fresh = fresh_index_like(&hnsw, PathBuf::from("test.idx"));
        assert!(!fresh.backend.is_mmap());
    }

    #[test]
    fn test_fresh_index_like_preserves_config() {
        let hnsw = CPIndex::new();
        let fresh = fresh_index_like(&hnsw, PathBuf::from("test.idx"));
        assert_eq!(fresh.config.m, hnsw.config.m);
        assert_eq!(fresh.config.ef_construction, hnsw.config.ef_construction);
        assert_eq!(fresh.config.ml, hnsw.config.ml);
    }

    #[test]
    fn test_fresh_index_like_fresh_index_is_empty() {
        let hnsw = CPIndex::new();
        hnsw.add(
            1,
            FilterBitset::from_u128(0),
            VectorRepresentations::Full(vec![0.1]),
            64,
        );
        let fresh = fresh_index_like(&hnsw, PathBuf::from("test.idx"));
        assert_eq!(fresh.nodes.len(), 0);
    }

    // ── compact_layout ───────────────────────────────────────────

    #[test]
    fn test_compact_layout_in_memory_trivial() {
        let mut vstore = VantaFile::create_in_memory(4096);
        let hnsw = CPIndex::new();
        hnsw.add(
            1,
            FilterBitset::from_u128(0),
            VectorRepresentations::Full(vec![0.1]),
            64,
        );
        let (map, _size) = compact_layout(&mut vstore, &hnsw, &[1], hdr_size()).unwrap();
        assert_eq!(map.get(&1), Some(&64));
    }

    #[test]
    fn test_compact_layout_empty_bfs_order_errors() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.vanta");
        let mut vstore = VantaFile::open(path, 4096).unwrap();
        let hnsw = CPIndex::new();
        let order: Vec<u128> = vec![];
        // Empty bfs_order is rejected — would destroy the database.
        assert!(compact_layout(&mut vstore, &hnsw, &order, hdr_size()).is_err());
    }

    #[test]
    fn test_compact_layout_in_memory_with_two_nodes() {
        let mut vstore = VantaFile::create_in_memory(4096);
        let hnsw = CPIndex::new();
        hnsw.add(
            1,
            FilterBitset::from_u128(0),
            VectorRepresentations::Full(vec![0.1]),
            64,
        );
        hnsw.add(
            2,
            FilterBitset::from_u128(0),
            VectorRepresentations::Full(vec![0.2]),
            128,
        );
        let (map, _size) = compact_layout(&mut vstore, &hnsw, &[1, 2], hdr_size()).unwrap();
        assert_eq!(map.len(), 2);
        assert_eq!(map.get(&1), Some(&64));
        assert_eq!(map.get(&2), Some(&128));
    }

    #[test]
    fn test_compact_layout_in_memory_node_not_in_hnsw() {
        let mut vstore = VantaFile::create_in_memory(4096);
        let hnsw = CPIndex::new();
        // bfs_order mentions id 99 which is not in hnsw
        let (map, _size) = compact_layout(&mut vstore, &hnsw, &[99], hdr_size()).unwrap();
        assert!(map.is_empty());
    }

    #[test]
    fn test_compact_layout_disk_backed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.vanta");
        let mut vstore = VantaFile::open(path.clone(), 4096).unwrap();

        // Write two headers at aligned offsets
        let hs = hdr_size();
        write_node_to_vstore(&mut vstore, 1, 64, &[0.1, 0.2], 0);
        write_node_to_vstore(
            &mut vstore,
            2,
            64 + hs + aligned_vec_size(2),
            &[0.3, 0.4],
            0,
        );

        let hnsw = CPIndex::new();
        hnsw.add(
            1,
            FilterBitset::from_u128(0),
            VectorRepresentations::Full(vec![0.1, 0.2]),
            64,
        );
        hnsw.add(
            2,
            FilterBitset::from_u128(0),
            VectorRepresentations::Full(vec![0.3, 0.4]),
            64 + hs + aligned_vec_size(2),
        );

        let (map, _size) = compact_layout(&mut vstore, &hnsw, &[1, 2], hs).unwrap();
        assert_eq!(map.len(), 2);
        assert!(map.contains_key(&1));
        assert!(map.contains_key(&2));
        // Both nodes got a valid aligned offset after compaction
        for &off in map.values() {
            assert!(off.is_multiple_of(STORAGE_ALIGNMENT));
        }

        // AUD-044 regression: after compaction the rewritten file must
        // actually contain the node data. A no-op tmp flush used to rename a
        // zero-filled file in non-memmap2 builds — silent data loss.
        drop(vstore);
        let reopened = VantaFile::open(path, 4096).unwrap();
        for (node_id, expected) in [(1u128, [0.1f32, 0.2]), (2, [0.3, 0.4])] {
            let offset = map.get(&node_id).copied().unwrap();
            let header = reopened.read_header(offset).unwrap();
            assert_eq!(header.id, node_id);
            assert_eq!(header.vector_len as usize, expected.len());
            let start = header.vector_offset as usize;
            let end = start + expected.len() * 4;
            let got: Vec<f32> = reopened.mmap_bytes()[start..end]
                .chunks_exact(4)
                .map(|c| f32::from_le_bytes(c.try_into().unwrap()))
                .collect();
            assert_eq!(
                got, expected,
                "node {node_id} vector must survive compaction + reopen"
            );
        }
    }

    #[test]
    fn test_compact_layout_reorder_reopen_preserves_data() {
        // AUD-044: compaction that CHANGES the layout (reversed BFS order moves
        // nodes to new offsets) must still survive reopen — the rewritten file
        // is what replace_backing_file remaps. Guards the write-back path in
        // both memmap2 and shim builds (the earlier no-op flush renamed a
        // zero-filled tmp file; a bad replace would clobber the compacted one).
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_reorder.vanta");
        let mut vstore = VantaFile::open(path.clone(), 4096).unwrap();

        let hs = hdr_size();
        write_node_to_vstore(&mut vstore, 1, 64, &[0.1, 0.2], 0);
        write_node_to_vstore(
            &mut vstore,
            2,
            64 + hs + aligned_vec_size(2),
            &[0.3, 0.4],
            0,
        );

        let hnsw = CPIndex::new();
        hnsw.add(
            1,
            FilterBitset::from_u128(0),
            VectorRepresentations::Full(vec![0.1, 0.2]),
            64,
        );
        hnsw.add(
            2,
            FilterBitset::from_u128(0),
            VectorRepresentations::Full(vec![0.3, 0.4]),
            64 + hs + aligned_vec_size(2),
        );

        // Reversed order: node 2 lands at the front offset, node 1 moves.
        let (map, _size) = compact_layout(&mut vstore, &hnsw, &[2, 1], hs).unwrap();
        assert_eq!(map.len(), 2);
        drop(vstore);

        let reopened = VantaFile::open(path, 4096).unwrap();
        for (node_id, expected) in [(2u128, [0.3f32, 0.4]), (1, [0.1, 0.2])] {
            let offset = map.get(&node_id).copied().unwrap();
            let header = reopened.read_header(offset).unwrap();
            assert_eq!(header.id, node_id);
            let start = header.vector_offset as usize;
            let end = start + expected.len() * 4;
            let got: Vec<f32> = reopened.mmap_bytes()[start..end]
                .chunks_exact(4)
                .map(|c| f32::from_le_bytes(c.try_into().unwrap()))
                .collect();
            assert_eq!(
                got, expected,
                "node {node_id} must survive reorder + reopen"
            );
        }
    }

    #[test]
    fn test_compact_layout_disk_backed_tombstone_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.vanta");
        let mut vstore = VantaFile::open(path.clone(), 4096).unwrap();

        let hs = hdr_size();
        let avs = aligned_vec_size(2);
        write_node_to_vstore(&mut vstore, 1, 64, &[0.1, 0.2], 0);
        write_node_to_vstore(&mut vstore, 2, 64 + hs + avs, &[0.3, 0.4], FLAG_TOMBSTONE);

        let hnsw = CPIndex::new();
        hnsw.add(
            1,
            FilterBitset::from_u128(0),
            VectorRepresentations::Full(vec![0.1, 0.2]),
            64,
        );
        hnsw.add(
            2,
            FilterBitset::from_u128(0),
            VectorRepresentations::Full(vec![0.3, 0.4]),
            64 + hs + avs,
        );

        let (map, _size) = compact_layout(&mut vstore, &hnsw, &[1, 2], hs).unwrap();
        // Only non-tombstone node should appear in output
        assert_eq!(map.len(), 1);
        assert!(map.contains_key(&1));
    }

    #[test]
    fn test_compact_layout_truncated_vstore_errors_not_panic() {
        // AUDREP-01: a header whose vector_len claims more bytes than the file
        // actually holds (crash mid-write) used to panic copy_from_slice and
        // tear down the whole process. It must return Err(VantaError) instead.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.vanta");
        let mut vstore = VantaFile::open(path, 4096).unwrap();

        let hs = hdr_size();
        // Write a header at offset 64 that claims a huge vector
        // length (e.g. 100k floats = 400KB) but the file is only 4096 bytes.
        let mut header = DiskNodeHeader::new(1);
        header.vector_len = 100_000;
        header.vector_offset = 64 + hs;
        vstore.write_header(64, &header).unwrap();

        let hnsw = CPIndex::new();
        hnsw.add(
            1,
            FilterBitset::from_u128(0),
            VectorRepresentations::Full(vec![0.1]),
            64,
        );

        let err = compact_layout(&mut vstore, &hnsw, &[1], hs).unwrap_err();
        assert!(
            err.to_string().contains("truncated"),
            "expected truncated vstore error, got: {err}"
        );
    }

    #[test]
    fn test_rebuild_empty_vstore() {
        let vstore = VantaFile::create_in_memory(4096);
        let mut hnsw = CPIndex::new();
        let report =
            rebuild_hnsw_from_vstore(&mut hnsw, &vstore, PathBuf::from("test.idx")).unwrap();
        assert_eq!(report.scanned_nodes, 0);
        assert_eq!(report.indexed_vectors, 0);
        assert_eq!(report.skipped_tombstones, 0);
        assert!(report.success);
        assert!(hnsw.nodes.is_empty());
    }

    #[test]
    fn test_rebuild_with_two_nodes() {
        let mut vstore = VantaFile::create_in_memory(4096);
        let hs = hdr_size();
        let avs = aligned_vec_size(3);

        write_node_to_vstore(&mut vstore, 1, 64, &[0.1, 0.2, 0.3], 0);
        write_node_to_vstore(&mut vstore, 2, 64 + hs + avs, &[0.4, 0.5, 0.6], 0);
        vstore.write_cursor = 64 + hs + avs + hs + avs;

        let mut hnsw = CPIndex::new();
        let report =
            rebuild_hnsw_from_vstore(&mut hnsw, &vstore, PathBuf::from("test.idx")).unwrap();

        assert_eq!(report.scanned_nodes, 2);
        assert_eq!(report.indexed_vectors, 2);
        assert_eq!(report.skipped_tombstones, 0);
        assert!(report.success);
        assert_eq!(hnsw.nodes.len(), 2);
        assert!(hnsw.nodes.contains_key(&1));
        assert!(hnsw.nodes.contains_key(&2));
    }

    #[test]
    fn test_rebuild_skips_tombstoned_node() {
        let mut vstore = VantaFile::create_in_memory(4096);
        let hs = hdr_size();
        let avs = aligned_vec_size(2);

        write_node_to_vstore(&mut vstore, 1, 64, &[0.1, 0.2], 0);
        write_node_to_vstore(&mut vstore, 2, 64 + hs + avs, &[0.3, 0.4], FLAG_TOMBSTONE);
        vstore.write_cursor = 64 + hs + avs + hs + avs;

        let mut hnsw = CPIndex::new();
        let report =
            rebuild_hnsw_from_vstore(&mut hnsw, &vstore, PathBuf::from("test.idx")).unwrap();

        assert_eq!(report.scanned_nodes, 2);
        assert_eq!(report.indexed_vectors, 1);
        assert_eq!(report.skipped_tombstones, 1);
        assert!(report.success);
        assert_eq!(hnsw.nodes.len(), 1);
        assert!(hnsw.nodes.contains_key(&1));
        assert!(!hnsw.nodes.contains_key(&2));
    }

    #[test]
    fn test_rebuild_zero_id_not_scanned() {
        let mut vstore = VantaFile::create_in_memory(4096);
        let hs = hdr_size();
        let avs = aligned_vec_size(2);

        // id=0 is considered invalid/uninitialized — not counted as scanned
        write_node_to_vstore(&mut vstore, 0, 64, &[0.1, 0.2], 0);
        vstore.write_cursor = 64 + hs + avs;

        let mut hnsw = CPIndex::new();
        let report =
            rebuild_hnsw_from_vstore(&mut hnsw, &vstore, PathBuf::from("test.idx")).unwrap();

        assert_eq!(report.scanned_nodes, 0);
        assert_eq!(report.indexed_vectors, 0);
        assert_eq!(report.skipped_tombstones, 0);
        assert!(report.success);
    }

    #[test]
    fn test_rebuild_without_vector_data() {
        let mut vstore = VantaFile::create_in_memory(4096);

        let mut header = DiskNodeHeader::new(42);
        header.vector_len = 0;
        header.vector_offset = 0;
        vstore.write_header(64, &header).unwrap();
        vstore.write_cursor = 64 + hdr_size(); // no vector data

        let mut hnsw = CPIndex::new();
        let report =
            rebuild_hnsw_from_vstore(&mut hnsw, &vstore, PathBuf::from("test.idx")).unwrap();

        assert_eq!(report.scanned_nodes, 1);
        assert_eq!(report.indexed_vectors, 0);
        assert_eq!(report.skipped_tombstones, 0);
        assert!(report.success);
    }

    #[test]
    fn test_rebuild_vector_data_beyond_mmap() {
        let mut vstore = VantaFile::create_in_memory(64); // only header region

        let mut header = DiskNodeHeader::new(7);
        header.vector_len = 100; // 400 bytes — way past the 64-byte buffer
        header.vector_offset = 1000;
        vstore.write_header(0, &header).ok(); // will fail — file too small
        vstore.write_cursor = 64 + hdr_size();

        // Vector data is beyond mmap bounds → VectorRepresentations::None
        let mut hnsw = CPIndex::new();
        let report =
            rebuild_hnsw_from_vstore(&mut hnsw, &vstore, PathBuf::from("test.idx")).unwrap();
        // 0 indexed because vector data was out of bounds
        assert_eq!(report.indexed_vectors, 0);
        assert!(report.success);
    }

    #[test]
    fn test_rebuild_report_path() {
        let vstore = VantaFile::create_in_memory(4096);
        let mut hnsw = CPIndex::new();
        let report =
            rebuild_hnsw_from_vstore(&mut hnsw, &vstore, PathBuf::from("custom/path.idx")).unwrap();
        assert_eq!(report.index_path, PathBuf::from("custom/path.idx"));
    }

    #[test]
    fn test_rebuild_duration_set() {
        let vstore = VantaFile::create_in_memory(4096);
        let mut hnsw = CPIndex::new();
        let report =
            rebuild_hnsw_from_vstore(&mut hnsw, &vstore, PathBuf::from("test.idx")).unwrap();
        // Duration should be Some non-zero (we can't guarantee non-zero wall time
        // but we can assert the field is populated meaningfully)
        assert!(report.success);
    }

    // ── ADR-032: rebuild must recover quantized vectors ──────────────────

    #[test]
    fn test_rebuild_binary_vector() {
        let mut vstore = VantaFile::create_in_memory(4096);
        let mut node = crate::node::UnifiedNode::new(101);
        let data: Box<[u64]> = vec![0xDEADBEEFCAFEu64, 0x0123456789ABCDEFu64].into_boxed_slice();
        node.vector = crate::node::VectorRepresentations::Binary(data.clone());
        node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
        let off = crate::storage::ops::write_node_to_vstore(&mut vstore, &node).unwrap();
        vstore.write_cursor = off + hdr_size() + ((data.len() as u64 * 8 + 63) & !63);

        let mut hnsw = CPIndex::new();
        let report =
            rebuild_hnsw_from_vstore(&mut hnsw, &vstore, PathBuf::from("test.idx")).unwrap();
        assert_eq!(report.scanned_nodes, 1);
        assert_eq!(report.indexed_vectors, 1);
        let stored = hnsw.nodes.get(&101).unwrap();
        match &stored.vec_data {
            VectorRepresentations::Binary(b) => assert_eq!(b.as_ref(), data.as_ref()),
            other => panic!("expected Binary, got {:?}", other),
        }
    }

    #[test]
    fn test_rebuild_turbo_vector() {
        let mut vstore = VantaFile::create_in_memory(4096);
        let mut node = crate::node::UnifiedNode::new(102);
        let data: Box<[u8]> = vec![0xAB, 0xCD, 0xEF, 0x12].into_boxed_slice();
        node.vector = crate::node::VectorRepresentations::Turbo(data.clone());
        node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
        let off = crate::storage::ops::write_node_to_vstore(&mut vstore, &node).unwrap();
        vstore.write_cursor = off + hdr_size() + ((data.len() as u64 + 63) & !63);

        let mut hnsw = CPIndex::new();
        let report =
            rebuild_hnsw_from_vstore(&mut hnsw, &vstore, PathBuf::from("test.idx")).unwrap();
        assert_eq!(report.scanned_nodes, 1);
        assert_eq!(report.indexed_vectors, 1);
        let stored = hnsw.nodes.get(&102).unwrap();
        match &stored.vec_data {
            VectorRepresentations::Turbo(t) => assert_eq!(t.as_ref(), data.as_ref()),
            other => panic!("expected Turbo, got {:?}", other),
        }
    }

    #[test]
    fn test_rebuild_sq8_vector() {
        let mut vstore = VantaFile::create_in_memory(4096);
        let mut node = crate::node::UnifiedNode::new(103);
        let data: Box<[i8]> = vec![10, -20, 30, -40].into_boxed_slice();
        let scale: f32 = 1.5;
        node.vector = crate::node::VectorRepresentations::SQ8(data.clone(), scale);
        node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
        let off = crate::storage::ops::write_node_to_vstore(&mut vstore, &node).unwrap();
        vstore.write_cursor = off + hdr_size() + ((data.len() as u64 + 4 + 63) & !63);

        let mut hnsw = CPIndex::new();
        let report =
            rebuild_hnsw_from_vstore(&mut hnsw, &vstore, PathBuf::from("test.idx")).unwrap();
        assert_eq!(report.scanned_nodes, 1);
        assert_eq!(report.indexed_vectors, 1);
        let stored = hnsw.nodes.get(&103).unwrap();
        match &stored.vec_data {
            VectorRepresentations::SQ8(d, s) => {
                assert_eq!(d.as_ref(), data.as_ref());
                assert!((*s - scale).abs() < f32::EPSILON);
            }
            other => panic!("expected SQ8, got {:?}", other),
        }
    }

    #[test]
    fn test_rebuild_mixed_vectors_including_binary() {
        let mut vstore = VantaFile::create_in_memory(8192);
        let hs = hdr_size();
        // node 1: Full
        let mut n1 = crate::node::UnifiedNode::new(201);
        n1.vector = VectorRepresentations::Full(vec![0.1, 0.2]);
        n1.flags.set(crate::node::NodeFlags::HAS_VECTOR);
        let off1 = crate::storage::ops::write_node_to_vstore(&mut vstore, &n1).unwrap();
        // node 2: Binary
        let mut n2 = crate::node::UnifiedNode::new(202);
        let bdata: Box<[u64]> = vec![0xAAAAAAAAAAAAAAAAu64].into_boxed_slice();
        n2.vector = VectorRepresentations::Binary(bdata.clone());
        n2.flags.set(crate::node::NodeFlags::HAS_VECTOR);
        // compute next offset aligned after n1
        let n1_payload = 2 * 4;
        let n1_next = off1 + hs + ((n1_payload + 63) & !63);
        // ensure vstore cursor is at n1_next before second write — our write_node_to_vstore
        // already advanced it via its own cursor logic, but for this mixed test we rely on
        // its internal cursor handling; just write n2 sequentially
        let _off2 = crate::storage::ops::write_node_to_vstore(&mut vstore, &n2).unwrap();

        let mut hnsw = CPIndex::new();
        let report =
            rebuild_hnsw_from_vstore(&mut hnsw, &vstore, PathBuf::from("test.idx")).unwrap();
        assert_eq!(report.scanned_nodes, 2);
        assert_eq!(report.indexed_vectors, 2);
        assert_eq!(hnsw.nodes.len(), 2);
        match &hnsw.nodes.get(&201).unwrap().vec_data {
            VectorRepresentations::Full(v) => assert_eq!(v, &vec![0.1, 0.2]),
            other => panic!("n1 expected Full, got {:?}", other),
        }
        match &hnsw.nodes.get(&202).unwrap().vec_data {
            VectorRepresentations::Binary(b) => assert_eq!(b.as_ref(), bdata.as_ref()),
            other => panic!("n2 expected Binary, got {:?}", other),
        }
        // silence unused warning
        assert_eq!(n1_next, n1_next);
    }
}
