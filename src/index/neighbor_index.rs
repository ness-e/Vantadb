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
    /// Inbound-degree: for each node_id, how many neighbor lists currently
    /// contain it (counted across all layers).
    ///
    /// This drives the reachability invariant: `shrink_neighbors` never
    /// removes the last remaining incoming link of a node, so every node stays
    /// reachable from the entry point and no "island" nodes can appear.
    inbound: DashMap<u128, u32>,
}

impl HnswNeighborIndex {
    pub fn new() -> Self {
        Self {
            lists: DashMap::new(),
            id_to_meta: DashMap::new(),
            inbound: DashMap::new(),
        }
    }

    /// Number of neighbor lists (any layer) that currently contain `id`.
    /// Used by `shrink_neighbors` to preserve the node's last incoming link.
    #[inline]
    pub(crate) fn inbound_count(&self, id: u128) -> u32 {
        self.inbound.get(&id).map(|r| *r).unwrap_or(0)
    }

    fn increment_inbound(&self, id: u128) {
        *self.inbound.entry(id).or_insert(0) += 1;
    }

    fn decrement_inbound(&self, id: u128) {
        let mut now_zero = false;
        if let Some(mut r) = self.inbound.get_mut(&id) {
            *r = r.saturating_sub(1);
            now_zero = *r == 0;
        }
        // Drop the shard guard (scoped) before mutating the map: evict a
        // counter when it reaches zero so deleted reference sinks are fully
        // released instead of accumulating unbounded 0-count entries
        // (ERR-012).
        if now_zero {
            self.inbound.remove(&id);
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
                    self.increment_inbound(neighbor);
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
                    self.increment_inbound(neighbor);
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
    /// Maintains inbound counts: entries removed by the replace lose one
    /// inbound reference; entries added gain one.
    pub fn set_neighbors(&self, id: u128, layer: usize, neighbors: NeighborVec) {
        let entry = self.lists.entry((id, layer));
        match entry {
            dashmap::mapref::entry::Entry::Occupied(mut oc) => {
                let old = oc.get_mut();
                for &n in old.iter() {
                    if !neighbors.contains(&n) {
                        self.decrement_inbound(n);
                    }
                }
                for &n in neighbors.iter() {
                    if !old.contains(&n) {
                        self.increment_inbound(n);
                    }
                }
                *old = neighbors;
            }
            dashmap::mapref::entry::Entry::Vacant(v) => {
                for &n in neighbors.iter() {
                    self.increment_inbound(n);
                }
                v.insert(neighbors);
            }
        }
    }

    /// Remove a node and ALL trace of it from the index (ERR-012):
    ///
    /// 1. Drop the node's own outbound lists, decrementing each neighbor's
    ///    `inbound` counter (symmetrical with `add_neighbor` / `set_neighbors`).
    /// 2. Strip every remaining reference *to* the node from other nodes'
    ///    lists, so no live list points at a deleted id.
    /// 3. Evict the node's own counter entry once nothing references it.
    ///
    /// Without this, deletes leak: the node's lists stay in the index and its
    /// neighbors' `inbound` counters keep counting a dead edge forever.
    pub fn remove_node(&self, id: u128) {
        if let Some((_, num_layers)) = self.id_to_meta.remove(&id) {
            for layer in 0..num_layers {
                if let Some((_, neighbors)) = self.lists.remove(&(id, layer)) {
                    for &n in neighbors.iter() {
                        self.decrement_inbound(n);
                    }
                }
            }
        }
        // Scrub references to `id` from every other node's lists.
        // DashMap `iter()` holds the global lock: collect candidates first,
        // then mutate one shard at a time via `remove_neighbor`.
        let mut stale: Vec<(u128, usize)> = Vec::new();
        for entry in self.lists.iter() {
            if entry.value().contains(&id) {
                stale.push(*entry.key());
            }
        }
        for (owner, layer) in stale {
            self.remove_neighbor(owner, layer, id);
        }
        self.inbound.remove(&id);
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
            if let Some((_, cnt)) = self.inbound.remove(&old_id) {
                self.inbound.insert(new_id, cnt);
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
                if before != v.len() {
                    self.decrement_inbound(neighbor_id);
                    true
                } else {
                    false
                }
            }
            None => false,
        }
    }

    /// Retain only specific neighbors in a layer (used in repair_orphan_links).
    #[allow(dead_code)]
    pub fn retain_neighbors<F>(&self, id: u128, layer: usize, mut f: F)
    where
        F: FnMut(&mut u128) -> bool,
    {
        if let Some(mut v) = self.lists.get_mut(&(id, layer)) {
            let mut removed: Vec<u128> = Vec::new();
            v.retain(|n| {
                let keep = f(n);
                if !keep {
                    removed.push(*n);
                }
                keep
            });
            for n in removed {
                self.decrement_inbound(n);
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// INV-024: verify `inbound_count(id)` equals the real number of neighbor
    /// lists (across all layers) that contain `id`, and that the sum of all
    /// inbound counters equals the total number of edges.
    fn check_inbound_invariant(idx: &HnswNeighborIndex) {
        let mut actual: HashMap<u128, u32> = HashMap::new();
        let mut total_edges: u32 = 0;
        for entry in idx.lists.iter() {
            for &n in entry.value().iter() {
                *actual.entry(n).or_insert(0) += 1;
                total_edges += 1;
            }
        }

        for (id, expected) in &actual {
            assert_eq!(
                idx.inbound_count(*id),
                *expected,
                "inbound mismatch for node {id}"
            );
        }
        // Nodes that exist but have no incoming references must count 0.
        for entry in idx.id_to_meta.iter() {
            let id = *entry.key();
            assert_eq!(
                idx.inbound_count(id),
                actual.get(&id).copied().unwrap_or(0),
                "inbound mismatch for node {id}"
            );
        }

        let counter_sum: u32 = idx.inbound.iter().map(|r| *r).sum();
        assert_eq!(counter_sum, total_edges, "sum of inbound != total edges");
    }

    #[test]
    fn inbound_count_increments_on_insert() {
        let idx = HnswNeighborIndex::new();
        idx.allocate(1, 1);
        idx.allocate(2, 1);

        assert!(idx.add_neighbor(1, 0, 2));
        assert_eq!(idx.inbound_count(2), 1);

        // Duplicate insert must not double-count.
        assert!(!idx.add_neighbor(1, 0, 2));
        assert_eq!(idx.inbound_count(2), 1);
    }

    #[test]
    fn set_neighbors_replace_updates_both_sides() {
        let idx = HnswNeighborIndex::new();
        idx.allocate(1, 1);
        idx.allocate(2, 1);
        idx.allocate(3, 1);
        idx.allocate(4, 1);

        let mut old = NeighborVec::new();
        old.push(2);
        old.push(3);
        idx.set_neighbors(1, 0, old);
        assert_eq!(idx.inbound_count(2), 1);
        assert_eq!(idx.inbound_count(3), 1);

        // Replace [2,3] with [3,4]: 2 loses a reference, 4 gains one, 3 stays.
        let mut next = NeighborVec::new();
        next.push(3);
        next.push(4);
        idx.set_neighbors(1, 0, next);
        assert_eq!(idx.inbound_count(2), 0, "old target must be decremented");
        assert_eq!(idx.inbound_count(3), 1, "kept target unchanged");
        assert_eq!(idx.inbound_count(4), 1, "new target must be incremented");
    }

    #[test]
    fn remove_neighbor_decrements_target() {
        let idx = HnswNeighborIndex::new();
        idx.allocate(1, 1);
        idx.allocate(2, 1);

        let mut v = NeighborVec::new();
        v.push(2);
        idx.set_neighbors(1, 0, v);
        assert_eq!(idx.inbound_count(2), 1);

        assert!(idx.remove_neighbor(1, 0, 2));
        assert_eq!(idx.inbound_count(2), 0);

        // Removing an absent neighbor must not under-decrement.
        assert!(!idx.remove_neighbor(1, 0, 2));
        assert_eq!(idx.inbound_count(2), 0);
    }

    #[test]
    fn remove_node_cleans_all_targets_and_meta() {
        let idx = HnswNeighborIndex::new();
        idx.allocate(1, 2);
        idx.allocate(2, 1);
        idx.allocate(3, 1);

        let mut v = NeighborVec::new();
        v.push(2);
        idx.set_neighbors(1, 0, v);
        let mut v = NeighborVec::new();
        v.push(3);
        idx.set_neighbors(1, 1, v);
        assert_eq!(idx.inbound_count(2), 1);
        assert_eq!(idx.inbound_count(3), 1);

        idx.remove_node(1);
        assert_eq!(idx.inbound_count(2), 0);
        assert_eq!(idx.inbound_count(3), 0);
        assert!(
            idx.num_layers(1).is_none(),
            "removed node meta must be gone"
        );
    }

    #[test]
    fn remove_node_evicts_zero_inbound_node_after_delete() {
        // ERR-012: deleting a middle node in chain A→B→C must:
        // 1. Drop B's own lists and meta.
        // 2. Decrement B's former outbound neighbor inbound (C loses B's
        //    contribution).
        // 3. Scrub every inbound reference to B from other lists (A, C no
        //    longer point at a deleted id).
        // 4. Evict B's inbound counter entirely once nothing references it.
        let idx = HnswNeighborIndex::new();
        idx.allocate(1, 1); // A
        idx.allocate(2, 1); // B
        idx.allocate(3, 1); // C

        // A→B, B→C, C→B (bidirectional edges like real HNSW links).
        let mut a = NeighborVec::new();
        a.push(2);
        idx.set_neighbors(1, 0, a);
        let mut b = NeighborVec::new();
        b.push(3);
        idx.set_neighbors(2, 0, b);
        let mut c = NeighborVec::new();
        c.push(2);
        idx.set_neighbors(3, 0, c);

        assert_eq!(idx.inbound_count(2), 2); // referenced by A and C
        assert_eq!(idx.inbound_count(3), 1); // referenced by B
        check_inbound_invariant(&idx);

        idx.remove_node(2); // delete B

        // B's own meta and lists are gone.
        assert!(idx.num_layers(2).is_none(), "B's meta must be removed");
        assert!(
            idx.get_neighbors(2, 0).is_none(),
            "B's outbound lists must be removed"
        );
        // B's contribution to C is gone: C's inbound must drop to 0 and C's
        // list must no longer reference B.
        assert_eq!(idx.inbound_count(3), 0, "C's inbound must lose B's edge");
        assert!(
            !idx.get_neighbors(3, 0).unwrap_or_default().contains(&2),
            "C's list must not reference deleted B"
        );
        // A must also stop referencing B.
        assert!(
            !idx.get_neighbors(1, 0).unwrap_or_default().contains(&2),
            "A's list must not reference deleted B"
        );
        // Eviction: nothing references B anymore, so its counter entry is
        // removed entirely (zero-reference node released, no leak).
        assert!(
            idx.inbound.get(&2).is_none(),
            "deleted node's inbound entry must be evicted"
        );
        check_inbound_invariant(&idx);
    }

    #[test]
    fn property_inbound_consistent_after_mixed_ops() {
        // INV-024 property test: after a deterministic pseudo-random sequence of
        // insert / replace / remove, inbound counters must match the real edge
        // set. Uses a fixed-seed LCG (no external rand) so the sequence is
        // reproducible.
        const NODES: u128 = 6;
        const LAYERS: usize = 2;

        let idx = HnswNeighborIndex::new();
        for id in 0..NODES {
            idx.allocate(id, LAYERS);
        }

        let mut state: u64 = 0x1234_5678_9abc_def0;
        let mut next = move || {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (state >> 33) as u32
        };

        for step in 0..300u32 {
            let op = next() % 3;
            let src = (next() as u128) % NODES;
            let layer = (next() as usize) % LAYERS;
            match op {
                0 => {
                    let tgt = (next() as u128) % NODES;
                    idx.add_neighbor(src, layer, tgt);
                }
                1 => {
                    let mut v = NeighborVec::new();
                    for _ in 0..(next() % 4) {
                        let tgt = (next() as u128) % NODES;
                        if !v.contains(&tgt) {
                            v.push(tgt);
                        }
                    }
                    idx.set_neighbors(src, layer, v);
                }
                _ => {
                    let tgt = (next() as u128) % NODES;
                    idx.remove_neighbor(src, layer, tgt);
                }
            }
            if step % 25 == 24 {
                check_inbound_invariant(&idx);
            }
        }
        check_inbound_invariant(&idx);
    }
}
