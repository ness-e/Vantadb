//! DiskANN-inspired in-memory Vamana graph index.
//!
//! Implements a simplified Vamana graph: greedy search with a bounded
//! priority queue, and insert with robust pruning.
//!
//! ## ponytail
//! No actual disk I/O — purely in-memory. The index stores all vectors
//! alongside the graph. A real DiskANN would spill centroids to disk.

use crate::index::distance::calculate_similarity;
use crate::node::{DistanceMetric, FilterBitset, VectorRepresentations};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::sync::Mutex;

/// Configuration for the DiskANN index.
#[derive(Clone, Debug)]
pub struct DiskAnnConfig {
    /// Search list size (L). Larger = more accurate but slower.
    pub search_list_size: usize,
    /// Construction list size (R). Larger = better recall, more edges.
    pub search_list_size_construction: usize,
    /// Pruning parameter (>1 = more aggressive, <1 = denser graph).
    pub alpha: f32,
    /// Distance metric.
    pub distance_metric: DistanceMetric,
}

impl Default for DiskAnnConfig {
    fn default() -> Self {
        Self {
            search_list_size: 50,
            search_list_size_construction: 50,
            alpha: 1.2,
            distance_metric: DistanceMetric::Cosine,
        }
    }
}

/// In-memory Vamana graph index.
pub struct DiskAnnIndex {
    graph: Mutex<HashMap<u128, Vec<u128>>>,
    vectors: Mutex<HashMap<u128, Vec<f32>>>,
    bitsets: Mutex<HashMap<u128, FilterBitset>>,
    pub medoid: Mutex<Option<u128>>,
    config: DiskAnnConfig,
}

#[allow(dead_code)]
impl DiskAnnIndex {
    pub fn new(config: DiskAnnConfig) -> Self {
        Self {
            graph: Mutex::new(HashMap::new()),
            vectors: Mutex::new(HashMap::new()),
            bitsets: Mutex::new(HashMap::new()),
            medoid: Mutex::new(None),
            config,
        }
    }

    /// Greedy search: returns `L` nearest neighbors from the graph.
    ///
    /// Uses a bounded max-heap for candidates (by similarity, descending)
    /// and a result set that holds the top results.
    fn greedy_search(
        &self,
        query: &[f32],
        start: u128,
        l_size: usize,
        visited: &mut HashSet<u128>,
    ) -> Vec<(u128, f32)> {
        let graph = self.graph.lock().unwrap();
        let vectors = self.vectors.lock().unwrap();

        // Max-heap: best similarity first
        let mut candidates: BinaryHeap<OrderedSim> = BinaryHeap::new();
        // Min-heap for results (worst at top, for bounded pruning)
        let mut results: Vec<(u128, f32)> = Vec::new();

        if let Some(sim) = vectors.get(&start).map(|v| {
            calculate_similarity(
                query,
                None,
                None,
                None,
                &VectorRepresentations::Full(v.clone()),
                self.config.distance_metric,
            )
        }) {
            candidates.push(OrderedSim(sim, start));
            visited.insert(start);
            results.push((start, sim));
        }

        while let Some(OrderedSim(d_cand, cand_id)) = candidates.pop() {
            // Prune: if the worst result is already better than candidate, stop
            if results.len() >= l_size {
                results.sort_unstable_by(|a, b| {
                    b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
                });
                results.truncate(l_size);
                if let Some(&(_, worst_sim)) = results.last() {
                    if d_cand < worst_sim {
                        // Check if all unseen neighbors could be worse
                        // (simplification: break early)
                        break;
                    }
                }
            }

            if let Some(neighbors) = graph.get(&cand_id) {
                for &nid in neighbors.iter() {
                    if visited.contains(&nid) {
                        continue;
                    }
                    visited.insert(nid);

                    if let Some(v) = vectors.get(&nid) {
                        let sim = calculate_similarity(
                            query,
                            None,
                            None,
                            None,
                            &VectorRepresentations::Full(v.clone()),
                            self.config.distance_metric,
                        );
                        candidates.push(OrderedSim(sim, nid));
                        results.push((nid, sim));
                    }
                }
            }
        }

        // Final sort and truncate
        results.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(l_size);
        results
    }

    /// Insert a vector into the Vamana graph.
    ///
    /// 1. Find nearest neighbors via greedy search (L=R)
    /// 2. Build neighbor set from search results (up to R)
    /// 3. Apply robust pruning: keep only diverse neighbors
    /// 4. Update reverse edges
    fn insert_vector(&self, id: u128, vec: Vec<f32>) {
        let config = &self.config;
        let mut graph = self.graph.lock().unwrap();
        let mut vectors = self.vectors.lock().unwrap();
        let medoid_opt = *self.medoid.lock().unwrap();

        // Store the vector first
        vectors.insert(id, vec.clone());

        if let Some(medoid) = medoid_opt {
            // Use medoid as start point for greedy search
            let mut visited = HashSet::new();
            let candidates = self.do_greedy_search_internal(
                &vec,
                medoid,
                config.search_list_size_construction,
                &graph,
                &vectors,
                &mut visited,
            );

            // Apply robust pruning: select up to R diverse neighbors
            let pruned = self.robust_prune(
                candidates,
                config.search_list_size_construction,
                config.alpha,
                &graph,
                &vectors,
            );

            // Update forward edges
            graph.insert(id, pruned.clone());

            // Update reverse edges
            for &neighbor_id in &pruned {
                let entry = graph.entry(neighbor_id).or_default();
                entry.push(id);
                // Trim reverse edge list to 2*R to bound growth
                if entry.len() > 2 * config.search_list_size_construction {
                    entry.sort_unstable_by(|&a, &b| {
                        let va = vectors.get(&a).unwrap();
                        let vb = vectors.get(&b).unwrap();
                        let sim_a = calculate_similarity(
                            &vec,
                            None,
                            None,
                            None,
                            &VectorRepresentations::Full(va.clone()),
                            config.distance_metric,
                        );
                        let sim_b = calculate_similarity(
                            &vec,
                            None,
                            None,
                            None,
                            &VectorRepresentations::Full(vb.clone()),
                            config.distance_metric,
                        );
                        sim_b
                            .partial_cmp(&sim_a)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                    entry.truncate(2 * config.search_list_size_construction);
                }
            }
        } else {
            // First node: becomes medoid
            graph.insert(id, Vec::new());
            *self.medoid.lock().unwrap() = Some(id);
        }
    }

    /// Perform greedy search on pre-locked graph/vectors (internal helper).
    fn do_greedy_search_internal(
        &self,
        query: &[f32],
        start: u128,
        l_size: usize,
        graph: &HashMap<u128, Vec<u128>>,
        vectors: &HashMap<u128, Vec<f32>>,
        visited: &mut HashSet<u128>,
    ) -> Vec<(u128, f32)> {
        let mut candidates: BinaryHeap<OrderedSim> = BinaryHeap::new();
        let mut results: Vec<(u128, f32)> = Vec::new();

        if let Some(sim) = vectors.get(&start).map(|v| {
            calculate_similarity(
                query,
                None,
                None,
                None,
                &VectorRepresentations::Full(v.clone()),
                self.config.distance_metric,
            )
        }) {
            candidates.push(OrderedSim(sim, start));
            visited.insert(start);
            results.push((start, sim));
        }

        while let Some(OrderedSim(_d_cand, cand_id)) = candidates.pop() {
            if let Some(neighbors) = graph.get(&cand_id) {
                for &nid in neighbors.iter() {
                    if visited.contains(&nid) {
                        continue;
                    }
                    visited.insert(nid);
                    if let Some(v) = vectors.get(&nid) {
                        let sim = calculate_similarity(
                            query,
                            None,
                            None,
                            None,
                            &VectorRepresentations::Full(v.clone()),
                            self.config.distance_metric,
                        );
                        candidates.push(OrderedSim(sim, nid));
                        results.push((nid, sim));
                    }
                }
            }
        }

        results.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(l_size);
        results
    }

    /// Robust pruning: given a list of candidates (sorted descending by sim),
    /// select up to R diverse neighbors such that no selected neighbor has
    /// similarity `alpha * sim(a, b)` greater than the query-to-neighbor sim.
    fn robust_prune(
        &self,
        mut candidates: Vec<(u128, f32)>,
        r: usize,
        alpha: f32,
        _graph: &HashMap<u128, Vec<u128>>,
        vectors: &HashMap<u128, Vec<f32>>,
    ) -> Vec<u128> {
        candidates
            .sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let mut selected: Vec<u128> = Vec::new();
        let mut remaining: Vec<(u128, f32)> = candidates;

        while let Some((p, _)) = remaining.first().copied() {
            selected.push(p);
            if selected.len() >= r {
                break;
            }
            let p_vec = match vectors.get(&p) {
                Some(v) => v,
                None => {
                    remaining.remove(0);
                    continue;
                }
            };
            remaining.remove(0);

            // Filter: keep q only if sim(p, q) * alpha < sim(query, q)
            remaining.retain(|&(q, sim_q)| {
                match vectors.get(&q) {
                    Some(q_vec) => {
                        let sim_pq = calculate_similarity(
                            p_vec,
                            None,
                            None,
                            None,
                            &VectorRepresentations::Full(q_vec.clone()),
                            self.config.distance_metric,
                        );
                        // Keep q if p is not too close to q
                        sim_pq * alpha < sim_q
                    }
                    None => false,
                }
            });
        }

        selected.truncate(r);
        selected
    }
}

/// Ordered similarity for max-heap (larger sim = higher priority).
#[derive(Clone, Debug, PartialEq)]
struct OrderedSim(f32, u128);

impl Eq for OrderedSim {}

impl PartialOrd for OrderedSim {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderedSim {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self
            .0
            .partial_cmp(&other.0)
            .unwrap_or(std::cmp::Ordering::Equal)
        {
            std::cmp::Ordering::Equal => other.1.cmp(&self.1),
            cmp => cmp,
        }
    }
}

impl crate::index::VecIndex for DiskAnnIndex {
    fn search(
        &self,
        query_vec: &[f32],
        query_mask: &FilterBitset,
        top_k: usize,
        _vector_store: Option<&crate::storage::vfile::VantaFile>,
        _distance_metric: DistanceMetric,
    ) -> Vec<(u128, f32)> {
        if top_k == 0 {
            return Vec::new();
        }

        let medoid = match *self.medoid.lock().unwrap() {
            Some(m) => m,
            None => return Vec::new(),
        };

        let mut visited = HashSet::new();
        let mut candidates = self.greedy_search(
            query_vec,
            medoid,
            self.config.search_list_size.max(top_k),
            &mut visited,
        );

        // Filter by bitset
        let bitsets = self.bitsets.lock().unwrap();
        if !query_mask.is_all_set() {
            candidates.retain(|(id, _)| {
                bitsets
                    .get(id)
                    .map(|bs| bs.matches_mask(query_mask))
                    .unwrap_or(false)
            });
        }
        drop(bitsets);

        candidates.truncate(top_k);
        candidates
    }

    fn add(
        &self,
        id: u128,
        bitset: FilterBitset,
        vec_data: VectorRepresentations,
        _storage_offset: u64,
    ) {
        let vec = match &vec_data {
            VectorRepresentations::Full(v) => v.clone(),
            _ => return, // skip non-full vectors (ponytail)
        };

        self.bitsets.lock().unwrap().insert(id, bitset);
        self.insert_vector(id, vec);
    }

    fn estimate_memory_bytes(&self) -> usize {
        let graph = self.graph.lock().unwrap();
        let vectors = self.vectors.lock().unwrap();

        let graph_bytes: usize = graph
            .values()
            .map(|neighbors| {
                neighbors.len() * std::mem::size_of::<u128>() + std::mem::size_of::<u128>()
            })
            .sum();

        let vec_bytes: usize = vectors.values().map(|v| v.len() * 4 + 16).sum();

        graph_bytes + vec_bytes + std::mem::size_of::<DiskAnnConfig>()
    }

    fn len(&self) -> usize {
        self.vectors.lock().unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::VecIndex;
    use crate::node::ALL_BITSET;

    #[test]
    fn test_diskann_empty() {
        let idx = DiskAnnIndex::new(DiskAnnConfig::default());
        let results = idx.search(&[1.0, 0.0], &ALL_BITSET, 5, None, DistanceMetric::Cosine);
        assert!(results.is_empty());
        assert_eq!(idx.len(), 0);
    }

    #[test]
    fn test_diskann_single_node() {
        let idx = DiskAnnIndex::new(DiskAnnConfig {
            search_list_size: 10,
            search_list_size_construction: 10,
            alpha: 1.2,
            distance_metric: DistanceMetric::Cosine,
        });
        idx.add(
            1,
            FilterBitset::new(),
            VectorRepresentations::Full(vec![1.0, 0.0, 0.0]),
            0,
        );

        assert_eq!(idx.len(), 1);
        let results = idx.search(
            &[1.0, 0.0, 0.0],
            &ALL_BITSET,
            5,
            None,
            DistanceMetric::Cosine,
        );
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, 1);
    }

    #[test]
    fn test_diskann_search_multiple() {
        let idx = DiskAnnIndex::new(DiskAnnConfig {
            search_list_size: 20,
            search_list_size_construction: 20,
            alpha: 1.2,
            distance_metric: DistanceMetric::Cosine,
        });
        // Insert 5 points on a circle
        for i in 0u128..5 {
            let angle = (i as f32) * std::f32::consts::TAU / 5.0;
            idx.add(
                i,
                FilterBitset::new(),
                VectorRepresentations::Full(vec![angle.cos(), angle.sin()]),
                0,
            );
        }

        let query = vec![1.0, 0.0];
        let results = idx.search(&query, &ALL_BITSET, 3, None, DistanceMetric::Cosine);
        assert_eq!(results.len(), 3, "should return top_k=3");
        // Top-1 should be id=0 (angle=0 -> (1,0))
        assert_eq!(results[0].0, 0, "closest to (1,0) should be id=0");
    }

    #[test]
    fn test_diskann_bitset_filter() {
        let idx = DiskAnnIndex::new(DiskAnnConfig::default());
        let mut bs_a = FilterBitset::new();
        bs_a.set_bit(0);
        let mut bs_b = FilterBitset::new();
        bs_b.set_bit(1);

        idx.add(1, bs_a, VectorRepresentations::Full(vec![1.0, 0.0]), 0);
        idx.add(2, bs_b, VectorRepresentations::Full(vec![0.0, 1.0]), 0);

        let mut mask = FilterBitset::new();
        mask.set_bit(0);
        let results = idx.search(&[1.0, 0.0], &mask, 5, None, DistanceMetric::Cosine);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, 1);
    }
}
