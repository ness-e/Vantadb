use crate::index::graph::NeighborVec;
use dashmap::DashMap;

/// Lock-free neighbor list index using DashMap.
///
/// Separates neighbor storage from `HnswNode` so that neighbor lists can be
/// read/written independently without holding a DashMap shard lock on the
/// node itself. Each (node_id, layer) pair is stored as a separate DashMap
/// entry, enabling concurrent reads/writes across different nodes/layers.
///
/// # Layout
/// `lists` is a `DashMap<(u128, usize), NeighborVec>` mapping each
/// (node_id, layer) pair to its neighbor list.
/// `id_to_meta` maps `node_id` → `num_layers`.
pub(crate) struct HnswNeighborIndex {
    /// DashMap of (node_id, layer) → neighbor list. No per-layer locks needed
    /// — DashMap shards provide concurrent access.
    lists: DashMap<(u128, usize), NeighborVec>,
    /// Maps `node_id` → `num_layers`.
    pub(crate) id_to_meta: DashMap<u128, usize>,
}

impl HnswNeighborIndex {
    pub fn new() -> Self {
        Self {
            lists: DashMap::new(),
            id_to_meta: DashMap::new(),
        }
    }

    /// Allocate space for a node with `num_layers` layers.
    /// Each layer starts empty. Panics if `id` already exists.
    pub fn allocate(&self, id: u128, num_layers: usize) {
        for layer in 0..num_layers {
            self.lists.insert((id, layer), NeighborVec::new());
        }
        self.id_to_meta.insert(id, num_layers);
    }

    /// Read a layer's neighbor list (cloned, for search/validation).
    pub fn get_neighbors(&self, id: u128, layer: usize) -> Option<NeighborVec> {
        self.lists.get(&(id, layer)).map(|v| v.clone())
    }

    /// Get the number of layers for a node, without cloning the neighbor list.
    pub fn num_layers(&self, id: u128) -> Option<usize> {
        self.id_to_meta.get(&id).map(|r| *r)
    }

    /// Get the length of a layer's neighbor list without cloning.
    #[allow(dead_code)]
    pub fn len_neighbors(&self, id: u128, layer: usize) -> Option<usize> {
        self.lists.get(&(id, layer)).map(|v| v.len())
    }

    /// Push a neighbor into a layer's list. Returns `true` if added (not duplicate).
    /// Returns `false` if the entry doesn't exist or neighbor already present.
    #[allow(dead_code)]
    pub fn add_neighbor(&self, id: u128, layer: usize, neighbor: u128) -> bool {
        match self.lists.get_mut(&(id, layer)) {
            Some(mut v) => {
                if v.contains(&neighbor) {
                    false
                } else {
                    v.push(neighbor);
                    true
                }
            }
            None => false,
        }
    }

    /// Combined try-add + check-if-full in ONE DashMap entry access.
    ///
    /// Attempts to add `neighbor` to the layer list for `id`. If the neighbor
    /// was added (not a duplicate) AND the list length now exceeds `max_len`,
    /// returns `(true, Some(cloned_list))`. Otherwise returns `(added, None)`.
    ///
    /// Replaces the 3-access pattern (add_neighbor + len_neighbors + get_neighbors)
    /// used by `connect_layer_neighbors` during HNSW rebuild — reduces DashMap
    /// shard lock acquisitions from 3 to 1 per neighbor connection.
    pub fn try_add_and_get_if_full(
        &self,
        id: u128,
        layer: usize,
        neighbor: u128,
        max_len: usize,
    ) -> (bool, Option<NeighborVec>) {
        let entry = self.lists.entry((id, layer));
        match entry {
            dashmap::mapref::entry::Entry::Occupied(mut occupied) => {
                let list = occupied.get_mut();
                if list.contains(&neighbor) {
                    (false, None)
                } else {
                    list.push(neighbor);
                    if list.len() > max_len {
                        (true, Some(list.clone()))
                    } else {
                        (true, None)
                    }
                }
            }
            dashmap::mapref::entry::Entry::Vacant(_) => (false, None),
        }
    }

    /// Replace a layer's neighbor list entirely (used after shrink/prune).
    pub fn set_neighbors(&self, id: u128, layer: usize, neighbors: NeighborVec) {
        self.lists.insert((id, layer), neighbors);
    }

    /// Remove a node's metadata and all layer entries.
    /// Used during reindex when a node ID is removed.
    #[allow(dead_code)]
    pub fn remove_node(&self, id: u128) {
        if let Some((_, num_layers)) = self.id_to_meta.remove(&id) {
            for layer in 0..num_layers {
                self.lists.remove(&(id, layer));
            }
        }
    }

    /// Replace a node's ID while keeping its neighbor data.
    /// Used during reindex.
    #[allow(dead_code)]
    pub fn replace_id(&self, old_id: u128, new_id: u128) {
        if let Some((_, num_layers)) = self.id_to_meta.remove(&old_id) {
            self.id_to_meta.insert(new_id, num_layers);
            for layer in 0..num_layers {
                if let Some((_, neighbors)) = self.lists.remove(&(old_id, layer)) {
                    self.lists.insert((new_id, layer), neighbors);
                }
            }
        }
    }

    /// Remove a specific neighbor from a layer's list (used in repair_orphan_links).
    #[allow(dead_code)]
    pub fn remove_neighbor(&self, id: u128, layer: usize, neighbor_id: u128) -> bool {
        match self.lists.get_mut(&(id, layer)) {
            Some(mut v) => {
                let before = v.len();
                v.retain(|n| *n != neighbor_id);
                before != v.len()
            }
            None => false,
        }
    }

    /// Retain only specific neighbors in a layer (used in repair_orphan_links).
    #[allow(dead_code)]
    pub fn retain_neighbors<F>(&self, id: u128, layer: usize, f: F)
    where
        F: FnMut(&mut u128) -> bool,
    {
        if let Some(mut v) = self.lists.get_mut(&(id, layer)) {
            v.retain(f);
        }
    }

    /// Collect all neighbor data (for serialization).
    pub fn collect_all(&self) -> Vec<(u128, Vec<NeighborVec>)> {
        let mut result = Vec::new();
        for entry in self.id_to_meta.iter() {
            let id = *entry.key();
            let num_layers = *entry.value();
            let mut layers = Vec::with_capacity(num_layers);
            for layer in 0..num_layers {
                if let Some(v) = self.lists.get(&(id, layer)) {
                    layers.push(v.clone());
                } else {
                    layers.push(NeighborVec::new());
                }
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
            let num_layers = *entry.value();
            let layers = (0..num_layers)
                .map(|layer| {
                    self.lists
                        .get(&(id, layer))
                        .map(|v| v.clone())
                        .unwrap_or_default()
                })
                .collect::<Vec<NeighborVec>>();
            f(id, &layers);
        }
    }

    /// Returns `true` if the neighbor index contains no entries.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.lists.is_empty()
    }

    /// Returns the number of (node_id, layer) entries in the index.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.lists.len()
    }
}
