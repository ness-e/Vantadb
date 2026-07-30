use crate::index::graph::NeighborVec;
use dashmap::DashMap;

/// Flat, lock-friendly neighbor list index.
///
/// Separates neighbor storage from `HnswNode` so that neighbor lists can be
/// read/written independently without holding a DashMap shard lock on the
/// node itself. Each (node_id, layer) pair gets its own `RwLock`, enabling
/// concurrent reads across different nodes/layers.
///
/// # Layout
/// `lists` is a flat `Vec<RwLock<NeighborVec>>` indexed by `(start + layer)`.
/// `id_to_meta` maps `node_id` → `(start_index_in_lists, num_layers)`.
/// When a node is allocated, `num_layers` contiguous entries are pushed.
pub(crate) struct HnswNeighborIndex {
    /// Flat Vec of per-layer neighbor lists. Each entry is a `RwLock` protecting
    /// one (node_id, layer) pair's neighbor list.
    lists: parking_lot::Mutex<Vec<parking_lot::RwLock<NeighborVec>>>,
    /// Maps `node_id` → `(start_index_in_lists, num_layers)`.
    /// Start index is the position in `lists` where this node's layer-0 list begins.
    pub(crate) id_to_meta: DashMap<u128, (usize, usize)>,
}

impl HnswNeighborIndex {
    pub fn new() -> Self {
        Self {
            lists: parking_lot::Mutex::new(Vec::new()),
            id_to_meta: DashMap::new(),
        }
    }

    /// Allocate contiguous space for a node with `num_layers` layers.
    /// Each layer starts empty. Panics if `id` already exists.
    pub fn allocate(&self, id: u128, num_layers: usize) {
        let start = {
            let mut lists = self.lists.lock();
            let s = lists.len();
            lists.extend((0..num_layers).map(|_| parking_lot::RwLock::new(NeighborVec::new())));
            s
        };
        self.id_to_meta.insert(id, (start, num_layers));
    }

    /// Read a layer's neighbor list (cloned, for search/validation).
    pub fn get_neighbors(&self, id: u128, layer: usize) -> Option<NeighborVec> {
        let (start, num_layers) = *self.id_to_meta.get(&id)?;
        if layer >= num_layers {
            return None;
        }
        let lists = self.lists.lock();
        let guard = lists[start + layer].read();
        Some(guard.clone())
    }

    /// Get the number of layers for a node.
    pub fn num_layers(&self, id: u128) -> Option<usize> {
        self.id_to_meta.get(&id).map(|m| m.1)
    }

    /// Push a neighbor into a layer's list. Returns `true` if added (not duplicate).
    /// Takes a write lock. Returns `false` if already present.
    pub fn add_neighbor(&self, id: u128, layer: usize, neighbor: u128) -> bool {
        let (start, num_layers) = match self.id_to_meta.get(&id) {
            Some(m) => (m.0, m.1),
            None => return false,
        };
        if layer >= num_layers {
            return false;
        }
        let lists = self.lists.lock();
        let mut guard = lists[start + layer].write();
        if guard.contains(&neighbor) {
            return false;
        }
        guard.push(neighbor);
        true
    }

    /// Replace a layer's neighbor list entirely (used after shrink/prune).
    pub fn set_neighbors(&self, id: u128, layer: usize, neighbors: NeighborVec) {
        let (start, num_layers) = match self.id_to_meta.get(&id) {
            Some(m) => (m.0, m.1),
            None => return,
        };
        if layer >= num_layers {
            return;
        }
        let lists = self.lists.lock();
        let mut guard = lists[start + layer].write();
        *guard = neighbors;
    }

    /// Remove a node's metadata entry (does NOT deallocate from `lists` vec).
    /// Used during reindex when a node ID is removed.
    #[allow(dead_code)]
    pub fn remove_node(&self, id: u128) {
        self.id_to_meta.remove(&id);
    }

    /// Replace a node's ID while keeping its neighbor data.
    /// Used during reindex.
    #[allow(dead_code)]
    pub fn replace_id(&self, old_id: u128, new_id: u128) {
        if let Some(meta) = self.id_to_meta.remove(&old_id) {
            self.id_to_meta.insert(new_id, meta.1);
        }
    }

    /// Remove a specific neighbor from a layer's list (used in repair_orphan_links).
    #[allow(dead_code)]
    pub fn remove_neighbor(&self, id: u128, layer: usize, neighbor_id: u128) -> bool {
        let (start, num_layers) = match self.id_to_meta.get(&id) {
            Some(m) => (m.0, m.1),
            None => return false,
        };
        if layer >= num_layers {
            return false;
        }
        let lists = self.lists.lock();
        let mut guard = lists[start + layer].write();
        let before = guard.len();
        guard.retain(|n| *n != neighbor_id);
        before != guard.len()
    }

    /// Retain only specific neighbors in a layer (used in repair_orphan_links).
    #[allow(dead_code)]
    pub fn retain_neighbors<F>(&self, id: u128, layer: usize, f: F)
    where
        F: FnMut(&mut u128) -> bool,
    {
        let (start, num_layers) = match self.id_to_meta.get(&id) {
            Some(m) => (m.0, m.1),
            None => return,
        };
        if layer >= num_layers {
            return;
        }
        let lists = self.lists.lock();
        let mut guard = lists[start + layer].write();
        guard.retain(f);
    }

    /// Collect all neighbor data (for serialization).
    pub fn collect_all(&self) -> Vec<(u128, Vec<NeighborVec>)> {
        let mut result = Vec::new();
        for entry in self.id_to_meta.iter() {
            let id = *entry.key();
            let (start, num_layers) = *entry.value();
            let lists = self.lists.lock();
            let mut layers = Vec::with_capacity(num_layers);
            for i in 0..num_layers {
                layers.push(lists[start + i].read().clone());
            }
            result.push((id, layers));
        }
        result
    }

    /// Iterate all neighbor data (for stats/validation).
    pub fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(u128, &[NeighborVec]),
    {
        for entry in self.id_to_meta.iter() {
            let id = *entry.key();
            let (start, num_layers) = *entry.value();
            let layers = {
                let lists = self.lists.lock();
                (0..num_layers)
                    .map(|i| lists[start + i].read().clone())
                    .collect::<Vec<NeighborVec>>()
                // lists guard dropped here, before the callback
            };
            f(id, &layers);
        }
    }
}
