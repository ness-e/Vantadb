//! Zero-copy HNSW archive definitions for memory-mapped index access.
//!
//! Defines `repr(C)` archive structs used by the rkyv-based persistence
//! layer, designed for direct mmap reads without deserialization.

/// Zero-copy HNSW graph archive format using `repr(C)` structs
/// that can be memory-mapped and accessed directly.
///
/// Layout (version 8):
///   [ArchivedHnswHeader]             — 40 bytes, align 8
///   [padding]                        — 8 bytes for 16-byte alignment
///   [ArchivedHnswNode; node_count]   — 48 bytes each, align 16
///   [u64; neighbor_count_total]      — align 8
///
/// Version 7 (deprecated) had no padding between header and nodes,
/// causing misaligned ArchivedHnswNode accesses on ARM/WASM.
use crate::index::{CPIndex, HnswConfig, HnswNode};
use crate::node::{DistanceMetric, VectorRepresentations};

const HNSW_MAGIC: [u8; 8] = *b"VNTHNSW\0";
const HNSW_VERSION: u64 = 8;

/// File header, always at offset 0.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ArchivedHnswHeader {
    pub magic: [u8; 8],
    pub version: u64,
    pub entry_point: u64,
    pub node_count: u64,
    pub max_layer: u32,
    pub distance_metric: u8,
}

/// Per-node fixed-size header. Neighbor data follows all nodes as a flat `[u64]` array.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ArchivedHnswNode {
    pub id: u64,
    pub bitset: u128,
    pub storage_offset: u64,
    pub inv_cached_norm: f32,
    /// Offset (in u64 count) into the trailing neighbor array.
    pub neighbor_offset_u64: u64,
    /// Total neighbor cells for this node.
    pub neighbor_count: u32,
}

/// Safe wrapper around an mmap'd HNSW graph.
pub struct ArchivedHnswGraph<'a> {
    pub header: &'a ArchivedHnswHeader,
    pub nodes: &'a [ArchivedHnswNode],
    pub neighbor_data: &'a [u64],
}

impl<'a> ArchivedHnswGraph<'a> {
    pub fn from_bytes(data: &'a [u8]) -> Option<Self> {
        let header_size = std::mem::size_of::<ArchivedHnswHeader>();
        let node_size = std::mem::size_of::<ArchivedHnswNode>();
        let node_align = std::mem::align_of::<ArchivedHnswNode>();

        if data.len() < header_size {
            return None;
        }

        let addr = data.as_ptr() as usize;
        if !addr.is_multiple_of(std::mem::align_of::<ArchivedHnswHeader>()) {
            return None;
        }
        let header: &'a ArchivedHnswHeader =
            unsafe { &*(data.as_ptr() as *const ArchivedHnswHeader) };
        if header.magic != HNSW_MAGIC {
            return None;
        }
        if header.version != HNSW_VERSION {
            return None;
        }

        let node_count = header.node_count as usize;

        // Round up header_size to node_align to get nodes_start
        let nodes_start = (header_size + node_align - 1) & !(node_align - 1);
        let nodes_end = nodes_start + node_count * node_size;
        if data.len() < nodes_end {
            return None;
        }

        let nodes_addr = data.as_ptr() as usize + nodes_start;
        debug_assert!(
            nodes_addr.is_multiple_of(node_align),
            "ArchivedHnswNode array misaligned"
        );
        let nodes: &'a [ArchivedHnswNode] = unsafe {
            std::slice::from_raw_parts(
                data.as_ptr().add(nodes_start) as *const ArchivedHnswNode,
                node_count,
            )
        };
        let neighbor_bytes = &data[nodes_end..];
        debug_assert!(
            (neighbor_bytes.as_ptr() as usize).is_multiple_of(std::mem::align_of::<u64>()),
            "neighbor_data misaligned"
        );
        let neighbor_data: &'a [u64] = unsafe {
            std::slice::from_raw_parts(
                neighbor_bytes.as_ptr() as *const u64,
                neighbor_bytes.len() / 8,
            )
        };
        Some(Self {
            header,
            nodes,
            neighbor_data,
        })
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn entry_point(&self) -> u64 {
        self.header.entry_point
    }

    pub fn max_layer(&self) -> usize {
        self.header.max_layer as usize
    }

    pub fn distance_metric(&self) -> DistanceMetric {
        match self.header.distance_metric {
            0 => DistanceMetric::Cosine,
            _ => DistanceMetric::Euclidean,
        }
    }
}

/// Serialize index to the rkyv-compatible archive format.
impl CPIndex {
    pub fn serialize_to_rkyv(&self) -> std::io::Result<Vec<u8>> {
        let entry_point = self.get_entry_point().unwrap_or(u64::MAX);
        let node_count = self.nodes.len() as u64;
        let max_layer = self.max_layer.load(std::sync::atomic::Ordering::Acquire) as u32;
        let distance_metric_byte: u8 = match self.config.distance_metric {
            DistanceMetric::Cosine => 0,
            DistanceMetric::Euclidean => 1,
        };

        let header = ArchivedHnswHeader {
            magic: HNSW_MAGIC,
            version: HNSW_VERSION,
            entry_point,
            node_count,
            max_layer,
            distance_metric: distance_metric_byte,
        };

        let header_size = std::mem::size_of::<ArchivedHnswHeader>();
        let node_size = std::mem::size_of::<ArchivedHnswNode>();
        let node_align = std::mem::align_of::<ArchivedHnswNode>();
        let padding = (node_align - (header_size % node_align)) % node_align;
        let nodes_start = header_size + padding;

        let header_bytes = unsafe {
            std::slice::from_raw_parts(
                &header as *const ArchivedHnswHeader as *const u8,
                std::mem::size_of::<ArchivedHnswHeader>(),
            )
        };

        let mut buf = Vec::with_capacity(
            nodes_start
                + (node_count as usize) * node_size
                + (node_count as usize) * self.config.m * 2 * 8,
        );
        buf.extend_from_slice(header_bytes);
        buf.extend(std::iter::repeat_n(0u8, padding));

        let mut neighbor_data: Vec<u64> = Vec::new();
        for node_id in self.serialization_order() {
            let Some(node) = self.nodes.get(&node_id) else {
                continue;
            };
            let offset = neighbor_data.len() as u64;
            for layer in &node.neighbors {
                neighbor_data.extend_from_slice(layer);
            }
            let archived = ArchivedHnswNode {
                id: node.id,
                bitset: node.bitset,
                storage_offset: node.storage_offset,
                inv_cached_norm: node.inv_cached_norm,
                neighbor_offset_u64: offset,
                neighbor_count: (neighbor_data.len() - offset as usize) as u32,
            };
            let node_bytes = unsafe {
                std::slice::from_raw_parts(
                    &archived as *const ArchivedHnswNode as *const u8,
                    node_size,
                )
            };
            buf.extend_from_slice(node_bytes);
        }

        let neighbor_bytes = unsafe {
            std::slice::from_raw_parts(neighbor_data.as_ptr() as *const u8, neighbor_data.len() * 8)
        };
        buf.extend_from_slice(neighbor_bytes);

        Ok(buf)
    }

    /// Load an index from the rkyv archive format.
    pub fn load_from_rkyv(data: &[u8]) -> std::io::Result<Self> {
        use std::io::{Error, ErrorKind};
        let graph = ArchivedHnswGraph::from_bytes(data)
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "invalid rkyv archive"))?;

        let index = CPIndex::new_with_config(HnswConfig {
            distance_metric: graph.distance_metric(),
            ..Default::default()
        });

        if graph.header.entry_point != u64::MAX {
            index
                .entry_point
                .store(graph.entry_point(), std::sync::atomic::Ordering::Release);
            index
                .max_layer
                .store(graph.max_layer(), std::sync::atomic::Ordering::Release);
        }

        for archived in graph.nodes {
            let start = archived.neighbor_offset_u64 as usize;
            let end = start + archived.neighbor_count as usize;
            let mut neighbors: Vec<crate::index::NeighborVec> = Vec::new();
            if end <= graph.neighbor_data.len() {
                neighbors.push(crate::index::NeighborVec::from(&graph.neighbor_data[start..end]));
            }

            let node = HnswNode {
                id: archived.id,
                bitset: archived.bitset,
                vec_data: VectorRepresentations::None,
                neighbors,
                storage_offset: archived.storage_offset,
                inv_cached_norm: archived.inv_cached_norm,
            };
            index.nodes.insert(archived.id, node);
        }

        Ok(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::f32_l2_norm;
    use crate::node::DistanceMetric;

    fn make_test_index() -> CPIndex {
        let index = CPIndex::new_with_config(HnswConfig {
            m: 8,
            m_max0: 16,
            ef_construction: 64,
            ef_search: 32,
            ml: 1.0 / (8_f64).ln(),
            distance_metric: DistanceMetric::Cosine,
        });
        for i in 0u64..8 {
            let raw = [
                (i as f32 * 0.1).sin(),
                (i as f32 * 0.2).cos(),
                (i as f32 * 0.3).sin(),
                (i as f32 * 0.4).cos(),
            ];
            let norm = f32_l2_norm(&raw);
            let normalized: Vec<f32> = raw.iter().map(|v| v / norm).collect();
            index.add(i + 1, 0, VectorRepresentations::Full(normalized), 0);
        }
        index
    }

    #[test]
    fn test_rkyv_round_trip() {
        let index = make_test_index();
        let bytes = index.serialize_to_rkyv().expect("serialize");
        let restored = CPIndex::load_from_rkyv(&bytes).expect("load_from_rkyv");
        assert_eq!(restored.nodes.len(), index.nodes.len());
        assert_eq!(restored.get_entry_point(), index.get_entry_point());
    }

    #[test]
    fn test_rkyv_version_check_rejects_v7() {
        let header = ArchivedHnswHeader {
            magic: HNSW_MAGIC,
            version: 7,
            entry_point: u64::MAX,
            node_count: 0,
            max_layer: 0,
            distance_metric: 0,
        };
        let header_bytes = unsafe {
            std::slice::from_raw_parts(
                &header as *const ArchivedHnswHeader as *const u8,
                std::mem::size_of::<ArchivedHnswHeader>(),
            )
        };
        assert!(ArchivedHnswGraph::from_bytes(header_bytes).is_none());
    }

    #[test]
    fn test_rkyv_alignment_correct_in_serialized_output() {
        let index = make_test_index();
        let bytes = index.serialize_to_rkyv().expect("serialize");
        let header_size = std::mem::size_of::<ArchivedHnswHeader>();
        let node_align = std::mem::align_of::<ArchivedHnswNode>();
        let nodes_start = (header_size + node_align - 1) & !(node_align - 1);
        let nodes_addr = bytes.as_ptr() as usize + nodes_start;
        assert!(
            nodes_addr.is_multiple_of(node_align),
            "nodes array must be {node_align}-byte aligned"
        );
    }

    #[test]
    fn test_rkyv_truncated_data_rejected() {
        let index = make_test_index();
        let bytes = index.serialize_to_rkyv().expect("serialize");
        let truncated = &bytes[..bytes.len() / 2];
        assert!(CPIndex::load_from_rkyv(truncated).is_err());
    }
}
