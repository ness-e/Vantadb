/// Zero-copy HNSW graph archive format using `repr(C)` structs
/// that can be memory-mapped and accessed directly.
///
/// Layout:
///   [ArchivedHnswHeader]
///   [ArchivedHnswNode; node_count]
///   [u64; neighbor_count_total]
use crate::index::{CPIndex, HnswConfig, HnswNode};
use crate::node::{DistanceMetric, VectorRepresentations};

const HNSW_MAGIC: [u8; 8] = *b"VNTHNSW\0";

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
        if data.len() < header_size {
            return None;
        }
        let header: &'a ArchivedHnswHeader =
            unsafe { &*(data.as_ptr() as *const ArchivedHnswHeader) };
        if &header.magic != &HNSW_MAGIC {
            return None;
        }
        let node_count = header.node_count as usize;
        let nodes_start = header_size;
        let nodes_end = nodes_start + node_count * node_size;
        if data.len() < nodes_end {
            return None;
        }
        let nodes: &'a [ArchivedHnswNode] = unsafe {
            std::slice::from_raw_parts(
                data.as_ptr().add(nodes_start) as *const ArchivedHnswNode,
                node_count,
            )
        };
        let neighbor_bytes = &data[nodes_end..];
        let neighbor_data: &'a [u64] = unsafe {
            std::slice::from_raw_parts(
                neighbor_bytes.as_ptr() as *const u64,
                neighbor_bytes.len() / 8,
            )
        };
        Some(Self { header, nodes, neighbor_data })
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
            version: 7,
            entry_point,
            node_count,
            max_layer,
            distance_metric: distance_metric_byte,
        };

        let header_bytes = unsafe {
            std::slice::from_raw_parts(
                &header as *const ArchivedHnswHeader as *const u8,
                std::mem::size_of::<ArchivedHnswHeader>(),
            )
        };

        let mut buf = Vec::with_capacity(
            std::mem::size_of::<ArchivedHnswHeader>()
                + (node_count as usize) * std::mem::size_of::<ArchivedHnswNode>()
                + (node_count as usize) * self.config.m * 2 * 8,
        );
        buf.extend_from_slice(header_bytes);

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
                    std::mem::size_of::<ArchivedHnswNode>(),
                )
            };
            buf.extend_from_slice(node_bytes);
        }

        let neighbor_bytes = unsafe {
            std::slice::from_raw_parts(
                neighbor_data.as_ptr() as *const u8,
                neighbor_data.len() * 8,
            )
        };
        buf.extend_from_slice(neighbor_bytes);

        Ok(buf)
    }

    /// Load an index from the rkyv archive format.
    pub fn load_from_rkyv(data: &[u8]) -> std::io::Result<Self> {
        use std::io::{Error, ErrorKind};
        let graph = ArchivedHnswGraph::from_bytes(data).ok_or_else(|| {
            Error::new(ErrorKind::InvalidData, "invalid rkyv archive")
        })?;

        let index = CPIndex::new_with_config(HnswConfig {
            distance_metric: graph.distance_metric(),
            ..Default::default()
        });

        if graph.header.entry_point != u64::MAX {
            index.entry_point.store(graph.entry_point(), std::sync::atomic::Ordering::Release);
            index.max_layer.store(graph.max_layer(), std::sync::atomic::Ordering::Release);
        }

        for archived in graph.nodes {
            let start = archived.neighbor_offset_u64 as usize;
            let end = start + archived.neighbor_count as usize;
            let mut neighbors: Vec<Vec<u64>> = Vec::new();
            if end <= graph.neighbor_data.len() {
                neighbors.push(graph.neighbor_data[start..end].to_vec());
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
