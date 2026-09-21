//! CPIndex serialization: byte-format codec (`bytes`) and file/mmap
//! persistence (`file`). Byte format is `VNDX` and MUST stay stable
//! (P2-7 deferred — this module only moves code, never changes format).

mod bytes;
mod file;

#[cfg(test)]
#[allow(missing_docs)]
mod test_util {
    use crate::index::graph::{CPIndex, HnswConfig, HnswNode};
    use crate::index::IndexBackend;
    use crate::node::{FilterBitset, VectorRepresentations};
    use portable_atomic::AtomicU128;
    use rand::SeedableRng;
    use std::sync::atomic::{AtomicU64, AtomicUsize};

    /// Helper: build a small CPIndex with a single Full vector node.
    pub(crate) fn single_full_node_index() -> CPIndex {
        let nodes = dashmap::DashMap::new();
        let neighbor_index = crate::index::neighbor_index::HnswNeighborIndex::new();
        let id = 42u128;
        nodes.insert(
            id,
            HnswNode {
                id,
                bitset: FilterBitset::new(),
                vec_data: VectorRepresentations::Full(vec![0.1, 0.2, 0.3, 0.4]),
                storage_offset: 0,
                inv_cached_norm: 1.0,
                norm_sq: 1.0,
                flags: 0,
                neighbor_lists: Vec::new(),
            },
        );
        // Also insert the neighbor so validation passes
        nodes.insert(
            99u128,
            HnswNode {
                id: 99,
                bitset: FilterBitset::new(),
                vec_data: VectorRepresentations::Full(vec![0.5, 0.6, 0.7, 0.8]),
                storage_offset: 0,
                inv_cached_norm: 1.0,
                norm_sq: 1.0,
                flags: 0,
                neighbor_lists: Vec::new(),
            },
        );
        // Populate neighbor_index
        neighbor_index.allocate(42, 1);
        neighbor_index.set_neighbors(42, 0, smallvec::smallvec![99u128]);
        neighbor_index.allocate(99, 1);
        neighbor_index.set_neighbors(99, 0, smallvec::smallvec![42u128]);
        CPIndex {
            nodes,
            neighbor_index,
            max_layer: AtomicUsize::new(0),
            entry_point: AtomicU128::new(42),
            backend: IndexBackend::InMemory,
            config: HnswConfig::default(),
            total_nodes: AtomicU64::new(2),
            rng: parking_lot::Mutex::new(rand::rngs::StdRng::seed_from_u64(42)),
            ivf_index: parking_lot::Mutex::new(None),
            ivf_built_at_node_count: AtomicUsize::new(0),
            scann_index: parking_lot::Mutex::new(None),
            scann_built_at_node_count: AtomicUsize::new(0),
        }
    }
}
