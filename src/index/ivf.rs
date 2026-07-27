//! IVFFlat index — inverted file with flat (exact) distance computation.
//!
//! Builds a k-means clustering of indexed vectors, then searches only the
//! `nprobe` nearest clusters. Significantly faster than brute-force for
//! large datasets, at a small recall cost.
//!
//! ## ponytail
//! No external k-means dependency — simple manual Lloyd iteration with
//! Forgy initialization. No PQ/quantization — IVFFlat only.

use crate::index::distance::calculate_similarity;
use crate::node::{DistanceMetric, FilterBitset, VectorRepresentations};
use rand::seq::SliceRandom;
use rand::SeedableRng;

/// A single entry in an IVF inverted list.
#[derive(Clone, Debug)]
pub struct IvfEntry {
    pub id: u128,
    pub bitset: FilterBitset,
    pub vector: Vec<f32>,
}

/// Configuration for the IVF index.
#[derive(Clone, Debug, PartialEq)]
pub struct IvfConfig {
    /// Number of centroids / clusters. Default: `sqrt(n)` clamped to `[1, n]`.
    pub nlist: usize,
    /// Number of nearest clusters to probe during search. Default: 10.
    pub nprobe: usize,
    /// Distance metric used for all similarity computations.
    pub distance_metric: DistanceMetric,
}

impl Default for IvfConfig {
    fn default() -> Self {
        Self {
            nlist: 100,
            nprobe: 10,
            distance_metric: DistanceMetric::Cosine,
        }
    }
}

/// An inverted-file (IVF) index over a set of vectors.
///
/// The index stores vectors partitioned into `nlist` clusters. During search
/// only the `nprobe` clusters whose centroids are nearest to the query are
/// visited, making this much faster than flat scan for large collections.
#[derive(Clone, Debug)]
pub struct IvfIndex {
    /// Centroids of each cluster, length `nlist × dim`.
    pub centroids: Vec<Vec<f32>>,
    /// Per-centroid inverted lists of stored entries.
    pub inverted_lists: Vec<Vec<IvfEntry>>,
    pub config: IvfConfig,
}

/// Internal helper: extract a `Vec<f32>` from a node for k-means.
fn node_to_f32_slice(vector: &VectorRepresentations) -> Option<Vec<f32>> {
    match vector {
        VectorRepresentations::Full(v) => Some(v.clone()),
        VectorRepresentations::MmapFull(Some(mmap)) => {
            let len = mmap.len() / 4;
            if len == 0 || len > crate::index::graph::MAX_VEC_F32_LEN {
                return None;
            }
            // SAFETY: len bounded by MAX_VEC_F32_LEN; mmap kept alive by Arc.
            let slice = unsafe { std::slice::from_raw_parts(mmap.as_ptr() as *const f32, len) };
            Some(slice.to_vec())
        }
        _ => None,
    }
}

impl IvfIndex {
    /// Build an IVF index from the nodes in a DashMap.
    ///
    /// Runs k-means (Forgy init + Lloyd iteration, max 20 iterations,
    /// convergence at centroid movement < 1e-4) then populates the
    /// inverted lists.
    pub fn build(
        nodes: &dashmap::DashMap<u128, crate::index::graph::HnswNode>,
        config: &IvfConfig,
    ) -> Self {
        // Collect all valid vectors
        let mut entries: Vec<(u128, FilterBitset, Vec<f32>)> = Vec::new();
        for r in nodes.iter() {
            let node = r.value();
            if let Some(v) = node_to_f32_slice(&node.vec_data) {
                entries.push((node.id, node.bitset.clone(), v));
            }
        }

        let n = entries.len();
        if n == 0 {
            return Self {
                centroids: Vec::new(),
                inverted_lists: Vec::new(),
                config: config.clone(),
            };
        }

        let dim = entries[0].2.len();
        // Clamp nlist to [1, n]
        let nlist = config.nlist.clamp(1, n);

        let distance_metric = config.distance_metric;

        // ── Forgy initialization: pick nlist distinct random centroids ──
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let mut centroids: Vec<Vec<f32>> = Vec::with_capacity(nlist);
        {
            let mut indices: Vec<usize> = (0..n).collect();
            indices.shuffle(&mut rng);
            for &idx in indices.iter().take(nlist) {
                centroids.push(entries[idx].2.clone());
            }
        }

        // ── Lloyd iteration ──
        let max_iter = 20;
        let convergence_threshold = 1e-4_f32;
        let mut assignments = vec![0usize; n];

        for _iter in 0..max_iter {
            // Assign each entry to nearest centroid
            let mut changed = 0;
            for (i, (_, _, ref vec)) in entries.iter().enumerate() {
                let mut best = 0usize;
                let mut best_sim = f32::NEG_INFINITY;
                for (c, centroid) in centroids.iter().enumerate() {
                    let sim = calculate_similarity(
                        vec,
                        None,
                        None,
                        None,
                        &VectorRepresentations::Full(centroid.clone()),
                        distance_metric,
                    );
                    if sim > best_sim {
                        best_sim = sim;
                        best = c;
                    }
                }
                if assignments[i] != best {
                    changed += 1;
                    assignments[i] = best;
                }
            }

            // Recompute centroids as means of assigned vectors
            let mut new_centroids = vec![vec![0.0_f32; dim]; nlist];
            let mut counts = vec![0usize; nlist];
            for (i, (_, _, ref vec)) in entries.iter().enumerate() {
                let c = assignments[i];
                counts[c] += 1;
                for (d, &val) in vec.iter().enumerate() {
                    // SAFETY: `d` is bounded by `dim` which matches the centroids.
                    new_centroids[c][d] += val;
                }
            }
            for c in 0..nlist {
                if counts[c] > 0 {
                    let inv = 1.0 / counts[c] as f32;
                    new_centroids[c].iter_mut().for_each(|val| *val *= inv);
                } else {
                    // Empty cluster: re-initialize from a random entry
                    let idx = rand::Rng::random_range(&mut rng, 0..n);
                    new_centroids[c] = entries[idx].2.clone();
                }
            }

            // Check convergence (max centroid movement)
            let mut max_movement = 0.0_f32;
            for c in 0..nlist {
                // For Euclidean, similarity is -distance; we track change
                let diff = match distance_metric {
                    DistanceMetric::Euclidean => {
                        let dist = calculate_similarity(
                            &centroids[c],
                            None,
                            None,
                            None,
                            &VectorRepresentations::Full(new_centroids[c].clone()),
                            DistanceMetric::Euclidean,
                        );
                        (-dist).sqrt() // Euclidean distance
                    }
                    DistanceMetric::Cosine => {
                        // Cosine sim between old and new centroid: 1 - movement for
                        // angular difference. Movement towards similar = small change.
                        // Use squared L2 between centroids for convergence check.
                        let sq_sum: f32 = centroids[c]
                            .iter()
                            .zip(new_centroids[c].iter())
                            .map(|(a, b)| (a - b) * (a - b))
                            .sum();
                        sq_sum.sqrt()
                    }
                };
                max_movement = max_movement.max(diff);
            }
            centroids = new_centroids;

            if changed == 0 && max_movement < convergence_threshold {
                break;
            }
        }

        // ── Build inverted lists ──
        let mut inverted_lists: Vec<Vec<IvfEntry>> = (0..nlist).map(|_| Vec::new()).collect();
        for (i, (id, bitset, vec)) in entries.into_iter().enumerate() {
            let c = assignments[i];
            inverted_lists[c].push(IvfEntry {
                id,
                bitset,
                vector: vec,
            });
        }

        Self {
            centroids,
            inverted_lists,
            config: config.clone(),
        }
    }

    /// Search the IVF index for the `top_k` nearest neighbors.
    ///
    /// Finds the `nprobe` nearest centroids to `query`, then scores every
    /// entry in those centroid's inverted lists using `calculate_similarity`.
    /// Returns results sorted by descending similarity.
    pub fn search(
        &self,
        query: &[f32],
        top_k: usize,
        query_mask: &FilterBitset,
    ) -> Vec<(u128, f32)> {
        if self.centroids.is_empty() || top_k == 0 {
            return Vec::new();
        }

        let nprobe = self.config.nprobe.min(self.centroids.len());
        let metric = self.config.distance_metric;

        // Score all centroids and pick the nprobe nearest
        let mut centroid_scores: Vec<(usize, f32)> = self
            .centroids
            .iter()
            .enumerate()
            .map(|(i, centroid)| {
                let sim = calculate_similarity(
                    query,
                    None,
                    None,
                    None,
                    &VectorRepresentations::Full(centroid.clone()),
                    metric,
                );
                (i, sim)
            })
            .collect();

        // Sort by descending similarity
        centroid_scores
            .sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        centroid_scores.truncate(nprobe);

        // Scan all entries in the selected inverted lists
        let mut results: Vec<(u128, f32)> = Vec::new();

        for &(ci, _) in &centroid_scores {
            for entry in &self.inverted_lists[ci] {
                if !query_mask.is_all_set() && !entry.bitset.matches_mask(query_mask) {
                    continue;
                }
                let sim = calculate_similarity(
                    query,
                    None,
                    None,
                    None,
                    &VectorRepresentations::Full(entry.vector.clone()),
                    metric,
                );
                results.push((entry.id, sim));
            }
        }

        // Sort by descending similarity and truncate
        results.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(top_k);
        results
    }

    /// Serialize the IVF index to a binary buffer.
    ///
    /// Format (all values little-endian):
    /// - nlist (u64)
    /// - nprobe (u64)
    /// - distance_metric (u8: 0=Cosine, 1=Euclidean)
    /// - Number of centroids (u64)
    /// - For each centroid: dim (u64), then dim × f32 values
    /// - Number of inverted lists (u64, = nlist)
    /// - For each list: len (u64), then len × entries
    ///   - Each entry: id (u128), bitset byte count (u64), bitset bytes, dim (u64), dim × f32 values
    pub fn serialize_to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        // Config
        buf.extend_from_slice(&(self.config.nlist as u64).to_le_bytes());
        buf.extend_from_slice(&(self.config.nprobe as u64).to_le_bytes());
        let metric_byte: u8 = match self.config.distance_metric {
            DistanceMetric::Cosine => 0,
            DistanceMetric::Euclidean => 1,
        };
        buf.push(metric_byte);

        // Centroids
        buf.extend_from_slice(&(self.centroids.len() as u64).to_le_bytes());
        for centroid in &self.centroids {
            buf.extend_from_slice(&(centroid.len() as u64).to_le_bytes());
            for &val in centroid {
                buf.extend_from_slice(&val.to_le_bytes());
            }
        }

        // Inverted lists
        buf.extend_from_slice(&(self.inverted_lists.len() as u64).to_le_bytes());
        for list in &self.inverted_lists {
            buf.extend_from_slice(&(list.len() as u64).to_le_bytes());
            for entry in list {
                buf.extend_from_slice(&entry.id.to_le_bytes());
                let bs_bytes = entry.bitset.to_bytes();
                buf.extend_from_slice(&(bs_bytes.len() as u64).to_le_bytes());
                buf.extend_from_slice(&bs_bytes);
                buf.extend_from_slice(&(entry.vector.len() as u64).to_le_bytes());
                for &val in &entry.vector {
                    buf.extend_from_slice(&val.to_le_bytes());
                }
            }
        }

        buf
    }

    /// Deserialize an IVF index from a binary buffer.
    ///
    /// Returns `None` if the data is truncated or invalid.
    pub fn deserialize_from_bytes(data: &[u8]) -> Option<Self> {
        use std::io::{Cursor, Read};

        let mut cursor = Cursor::new(data);

        let read_u64 = |cursor: &mut Cursor<&[u8]>| -> Option<u64> {
            let mut buf = [0u8; 8];
            cursor.read_exact(&mut buf).ok()?;
            Some(u64::from_le_bytes(buf))
        };

        let read_u8 = |cursor: &mut Cursor<&[u8]>| -> Option<u8> {
            let mut buf = [0u8; 1];
            cursor.read_exact(&mut buf).ok()?;
            Some(buf[0])
        };

        let read_f32 = |cursor: &mut Cursor<&[u8]>| -> Option<f32> {
            let mut buf = [0u8; 4];
            cursor.read_exact(&mut buf).ok()?;
            Some(f32::from_le_bytes(buf))
        };

        // Config
        let nlist = read_u64(&mut cursor)? as usize;
        let nprobe = read_u64(&mut cursor)? as usize;
        let metric_byte = read_u8(&mut cursor)?;
        let distance_metric = match metric_byte {
            1 => DistanceMetric::Euclidean,
            _ => DistanceMetric::Cosine,
        };

        // Centroids
        let centroid_count = read_u64(&mut cursor)? as usize;
        let mut centroids = Vec::with_capacity(centroid_count);
        for _ in 0..centroid_count {
            let dim = read_u64(&mut cursor)? as usize;
            let mut centroid = Vec::with_capacity(dim);
            for _ in 0..dim {
                centroid.push(read_f32(&mut cursor)?);
            }
            centroids.push(centroid);
        }

        // Inverted lists
        let list_count = read_u64(&mut cursor)? as usize;
        let mut inverted_lists = Vec::with_capacity(list_count);
        for _ in 0..list_count {
            let entry_count = read_u64(&mut cursor)? as usize;
            let mut list = Vec::with_capacity(entry_count);
            for _ in 0..entry_count {
                let id = {
                    let mut buf = [0u8; 16];
                    cursor.read_exact(&mut buf).ok()?;
                    u128::from_le_bytes(buf)
                };
                let bs_len = read_u64(&mut cursor)? as usize;
                let mut bs_buf = vec![0u8; bs_len];
                cursor.read_exact(&mut bs_buf).ok()?;
                let (bitset, _consumed) = FilterBitset::from_bytes(&bs_buf).ok()?;
                let dim = read_u64(&mut cursor)? as usize;
                let mut vector = Vec::with_capacity(dim);
                for _ in 0..dim {
                    vector.push(read_f32(&mut cursor)?);
                }
                list.push(IvfEntry { id, bitset, vector });
            }
            inverted_lists.push(list);
        }

        Some(Self {
            centroids,
            inverted_lists,
            config: IvfConfig {
                nlist,
                nprobe,
                distance_metric,
            },
        })
    }
}

impl crate::index::VecIndex for IvfIndex {
    fn search(
        &self,
        query_vec: &[f32],
        query_mask: &crate::node::FilterBitset,
        top_k: usize,
        _vector_store: Option<&crate::storage::vfile::VantaFile>,
        _distance_metric: crate::node::DistanceMetric,
    ) -> Vec<(u128, f32)> {
        // IvfIndex does its own distance computation from stored vectors;
        // vector_store and distance_metric are unused here.
        self.search(query_vec, top_k, query_mask)
    }

    fn add(
        &self,
        _id: u128,
        _bitset: crate::node::FilterBitset,
        _vec_data: crate::node::VectorRepresentations,
        _storage_offset: u64,
    ) {
        // ponytail: IvfIndex is read-only after build; use IvfIndex::build().
        // Panicking here is intentional — it signals a programming error at
        // the integration level rather than silently dropping the add.
        panic!("IvfIndex is read-only after build; rebuild via IvfIndex::build()");
    }

    fn estimate_memory_bytes(&self) -> usize {
        let centroids_bytes: usize = self
            .centroids
            .iter()
            .map(|c| c.len() * std::mem::size_of::<f32>())
            .sum();
        let lists_bytes: usize = self
            .inverted_lists
            .iter()
            .map(|list| {
                list.len()
                    * (std::mem::size_of::<u128>()
                        + std::mem::size_of::<crate::node::FilterBitset>()
                        + std::mem::size_of::<Vec<f32>>())
            })
            .sum();
        let config_size = std::mem::size_of::<crate::index::ivf::IvfConfig>();
        centroids_bytes + lists_bytes + config_size
    }

    fn len(&self) -> usize {
        self.inverted_lists.iter().map(|list| list.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::graph::{HnswConfig, HnswNode, NeighborVec};
    use dashmap::DashMap;

    /// Helper: build a CPIndex with `n` distinct 2D vectors placed at
    /// regular angular intervals on the unit circle (so cosine similarity
    /// distinguishes them).
    fn make_nodes(n: usize) -> DashMap<u128, HnswNode> {
        let nodes = DashMap::new();
        for i in 0u128..(n as u128) {
            let angle = (i as f32) * std::f32::consts::TAU / (n as f32);
            let v = vec![angle.cos(), angle.sin()];
            nodes.insert(
                i,
                HnswNode {
                    id: i,
                    bitset: FilterBitset::new(),
                    vec_data: VectorRepresentations::Full(v),
                    neighbors: vec![NeighborVec::new()],
                    storage_offset: 0,
                    inv_cached_norm: 0.0,
                    norm_sq: 0.0,
                    flags: 0,
                },
            );
        }
        nodes
    }

    /// Build a toy CPIndex for integration tests (using the real index
    /// but with flat_threshold = None to disable flat search interfering).
    #[allow(dead_code)]
    fn make_toy_cpindex(n: usize) -> crate::index::CPIndex {
        use crate::index::graph::CPIndex;
        let config = HnswConfig {
            m: 8,
            m_max0: 16,
            ef_construction: 50,
            ef_search: 50,
            ml: 1.0 / (8_f64).ln(),
            distance_metric: DistanceMetric::Cosine,
            flat_threshold: None,
            index_type: crate::index::IndexType::Hnsw,
        };
        let index = CPIndex::new_with_config(config);
        for i in 0u128..(n as u128) {
            let angle = (i as f32) * std::f32::consts::TAU / (n as f32);
            let v = vec![angle.cos(), angle.sin()];
            index.add(i, FilterBitset::new(), VectorRepresentations::Full(v), 0);
        }
        index
    }

    // ── build tests ─────────────────────────────────────────────────

    #[test]
    fn test_ivf_build_empty() {
        let nodes = DashMap::new();
        let ivf = IvfIndex::build(&nodes, &IvfConfig::default());
        assert!(ivf.centroids.is_empty());
        assert!(ivf.inverted_lists.is_empty());
    }

    #[test]
    fn test_ivf_build_single_node() {
        let nodes = make_nodes(1);
        let ivf = IvfIndex::build(
            &nodes,
            &IvfConfig {
                nlist: 3,
                nprobe: 1,
                ..Default::default()
            },
        );
        assert_eq!(ivf.centroids.len(), 1, "nlist clamped to 1 for 1 node");
        assert_eq!(ivf.inverted_lists.len(), 1);
        // The single list should contain the one node
        assert_eq!(ivf.inverted_lists[0].len(), 1);
        assert_eq!(ivf.inverted_lists[0][0].id, 0);
    }

    #[test]
    fn test_ivf_build_clusters_assigned() {
        // With 8 points around a circle and nlist=4, each centroid should
        // have at least one entry.
        let nodes = make_nodes(8);
        let ivf = IvfIndex::build(
            &nodes,
            &IvfConfig {
                nlist: 4,
                nprobe: 2,
                distance_metric: DistanceMetric::Cosine,
            },
        );
        assert_eq!(ivf.centroids.len(), 4);
        assert_eq!(ivf.inverted_lists.len(), 4);
        let total: usize = ivf.inverted_lists.iter().map(|l| l.len()).sum();
        assert_eq!(total, 8);
    }

    // ── search tests ────────────────────────────────────────────────

    #[test]
    fn test_ivf_search_empty() {
        let ivf = IvfIndex {
            centroids: Vec::new(),
            inverted_lists: Vec::new(),
            config: IvfConfig::default(),
        };
        let results = ivf.search(&[1.0, 0.0], 5, &FilterBitset::all_set());
        assert!(results.is_empty());
    }

    #[test]
    fn test_ivf_search_small_vs_flat() {
        let n = 20;
        let nodes = make_nodes(n);
        let ivf = IvfIndex::build(
            &nodes,
            &IvfConfig {
                nlist: 4,
                nprobe: 4, // probe all = exact
                distance_metric: DistanceMetric::Cosine,
            },
        );

        let query = vec![1.0, 0.0];
        let ivf_results = ivf.search(&query, 5, &FilterBitset::all_set());
        assert_eq!(ivf_results.len(), 5, "should return top_k=5");

        // Flat brute-force for comparison
        let mut flat: Vec<(u128, f32)> = nodes
            .iter()
            .map(|r| {
                let node = r.value();
                let sim = calculate_similarity(
                    &query,
                    None,
                    None,
                    None,
                    &node.vec_data,
                    DistanceMetric::Cosine,
                );
                (node.id, sim)
            })
            .collect();
        flat.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        flat.truncate(5);

        // With nprobe=4 (all clusters probed), IVF should match flat exactly
        // on this small dataset
        assert_eq!(ivf_results.len(), flat.len());
        for (ivf_res, flat_res) in ivf_results.iter().zip(flat.iter()) {
            assert!(
                (ivf_res.1 - flat_res.1).abs() < 1e-5,
                "IVF score {} differs from flat {} for id={}",
                ivf_res.1,
                flat_res.1,
                ivf_res.0
            );
        }
    }

    #[test]
    fn test_ivf_search_known_top1() {
        // Point at (1,0) and (0,1). Query (0.99, 0.14) should have (1,0) as top-1.
        let nodes = DashMap::new();
        for (id, (x, y)) in [(0u128, (1.0, 0.0)), (1u128, (0.0, 1.0))] {
            nodes.insert(
                id,
                HnswNode {
                    id,
                    bitset: FilterBitset::new(),
                    vec_data: VectorRepresentations::Full(vec![x, y]),
                    neighbors: vec![NeighborVec::new()],
                    storage_offset: 0,
                    inv_cached_norm: 0.0,
                    norm_sq: 0.0,
                    flags: 0,
                },
            );
        }
        let ivf = IvfIndex::build(
            &nodes,
            &IvfConfig {
                nlist: 1,
                nprobe: 1,
                distance_metric: DistanceMetric::Cosine,
            },
        );

        let query = vec![0.99, 0.14]; // close to (1,0)
        let results = ivf.search(&query, 2, &FilterBitset::all_set());
        assert!(!results.is_empty());
        assert_eq!(results[0].0, 0, "top-1 should be id=0 (closest to (1,0))");
    }

    // ── serialization tests ─────────────────────────────────────────

    #[test]
    fn test_ivf_serialize_roundtrip() {
        let nodes = make_nodes(10);
        let ivf = IvfIndex::build(
            &nodes,
            &IvfConfig {
                nlist: 3,
                nprobe: 2,
                distance_metric: DistanceMetric::Cosine,
            },
        );

        let bytes = ivf.serialize_to_bytes();
        let deser = IvfIndex::deserialize_from_bytes(&bytes).expect("deserialize");
        assert_eq!(deser.centroids.len(), ivf.centroids.len());
        assert_eq!(deser.inverted_lists.len(), ivf.inverted_lists.len());
        assert_eq!(deser.config.nlist, ivf.config.nlist);
        assert_eq!(deser.config.nprobe, ivf.config.nprobe);
        assert_eq!(deser.config.distance_metric, ivf.config.distance_metric);

        // Same search results after round-trip
        let query = vec![1.0, 0.0];
        let before = ivf.search(&query, 5, &FilterBitset::all_set());
        let after = deser.search(&query, 5, &FilterBitset::all_set());
        assert_eq!(before, after);
    }

    #[test]
    fn test_ivf_serialize_empty() {
        let ivf = IvfIndex {
            centroids: Vec::new(),
            inverted_lists: Vec::new(),
            config: IvfConfig::default(),
        };
        let bytes = ivf.serialize_to_bytes();
        let deser = IvfIndex::deserialize_from_bytes(&bytes).expect("deserialize");
        assert!(deser.centroids.is_empty());
        assert!(deser.inverted_lists.is_empty());
    }

    #[test]
    fn test_ivf_serialize_euclidean() {
        let nodes = make_nodes(10);
        let ivf = IvfIndex::build(
            &nodes,
            &IvfConfig {
                nlist: 2,
                nprobe: 2,
                distance_metric: DistanceMetric::Euclidean,
            },
        );
        let bytes = ivf.serialize_to_bytes();
        let deser = IvfIndex::deserialize_from_bytes(&bytes).expect("deserialize");
        assert_eq!(deser.config.distance_metric, DistanceMetric::Euclidean);
    }

    // ── bitset filter tests ─────────────────────────────────────────

    #[test]
    fn test_ivf_search_with_bitset_filter() {
        let nodes = DashMap::new();
        for i in 0u128..4 {
            let mut bs = FilterBitset::new();
            if i == 0 || i == 2 {
                bs.set_bit(0);
            }
            nodes.insert(
                i,
                HnswNode {
                    id: i,
                    bitset: bs,
                    vec_data: VectorRepresentations::Full(vec![i as f32, 0.0]),
                    neighbors: vec![NeighborVec::new()],
                    storage_offset: 0,
                    inv_cached_norm: 0.0,
                    norm_sq: 0.0,
                    flags: 0,
                },
            );
        }
        let ivf = IvfIndex::build(
            &nodes,
            &IvfConfig {
                nlist: 2,
                nprobe: 2,
                distance_metric: DistanceMetric::Cosine,
            },
        );

        let mut mask = FilterBitset::new();
        mask.set_bit(0);
        let query = vec![1.0, 0.0];
        let results = ivf.search(&query, 10, &mask);
        // Only nodes 0 and 2 have bit 0 set
        for &(id, _) in &results {
            assert!(
                id == 0 || id == 2,
                "filtered results should only contain matching ids"
            );
        }
    }

    // ── nprobe clamping ─────────────────────────────────────────────

    #[test]
    fn test_ivf_nprobe_clamped() {
        let nodes = make_nodes(10);
        let ivf = IvfIndex::build(
            &nodes,
            &IvfConfig {
                nlist: 3,
                nprobe: 100, // larger than nlist
                distance_metric: DistanceMetric::Cosine,
            },
        );
        let query = vec![1.0, 0.0];
        // Should not panic — nprobe gets clamped to nlist (3)
        let results = ivf.search(&query, 5, &FilterBitset::all_set());
        assert!(!results.is_empty());
    }

    // ── integration with CPIndex ────────────────────────────────────

    #[test]
    fn test_ivf_via_cpindex_search() {
        use crate::index::graph::CPIndex;
        use crate::index::IndexType;

        let config = HnswConfig {
            m: 8,
            m_max0: 16,
            ef_construction: 50,
            ef_search: 50,
            ml: 1.0 / (8_f64).ln(),
            distance_metric: DistanceMetric::Cosine,
            flat_threshold: None,
            index_type: IndexType::Ivf,
        };
        let index = CPIndex::new_with_config(config);

        for i in 0u128..20 {
            let v = vec![(i as f32 * 0.1).sin(), (i as f32 * 0.1).cos()];
            index.add(i, FilterBitset::new(), VectorRepresentations::Full(v), 0);
        }

        let query = vec![0.0, 1.0];
        let results = index.search_nearest(&query, None, None, &FilterBitset::all_set(), 5, None);
        assert_eq!(results.len(), 5, "IVF should return 5 results");
        for &(_, score) in &results {
            assert!(score.is_finite(), "IVF score must be finite");
        }
    }

    #[test]
    fn test_ivf_hnsw_compare_results() {
        use crate::index::graph::CPIndex;
        use crate::index::IndexType;

        // Build HNSW index
        let hnsw_config = HnswConfig {
            m: 8,
            m_max0: 16,
            ef_construction: 200,
            ef_search: 200,
            ml: 1.0 / (8_f64).ln(),
            distance_metric: DistanceMetric::Cosine,
            flat_threshold: None,
            index_type: IndexType::Hnsw,
        };
        let hnsw_idx = CPIndex::new_with_config(hnsw_config);

        // Build IVF index with same data
        let ivf_config = HnswConfig {
            ef_search: 200,
            distance_metric: DistanceMetric::Cosine,
            flat_threshold: None,
            index_type: IndexType::Ivf,
            ..HnswConfig::default()
        };
        let ivf_idx = CPIndex::new_with_config(ivf_config);

        // Insert same 20 vectors into both
        for i in 0u128..20 {
            let angle = (i as f32) * std::f32::consts::TAU / 20.0;
            let v = vec![angle.cos(), angle.sin()];
            hnsw_idx.add(
                i,
                FilterBitset::new(),
                VectorRepresentations::Full(v.clone()),
                0,
            );
            ivf_idx.add(i, FilterBitset::new(), VectorRepresentations::Full(v), 0);
        }

        let query = vec![1.0, 0.0];
        let hnsw_results =
            hnsw_idx.search_nearest(&query, None, None, &FilterBitset::all_set(), 5, None);
        let ivf_results =
            ivf_idx.search_nearest(&query, None, None, &FilterBitset::all_set(), 5, None);

        // Both should find 5 results; top-1 should be the same (id=0 → (1,0))
        assert_eq!(hnsw_results.len(), 5);
        assert_eq!(ivf_results.len(), 5);
        assert_eq!(
            hnsw_results[0].0, ivf_results[0].0,
            "top-1 should match between HNSW and IVF: HNSW got id={}, IVF got id={}",
            hnsw_results[0].0, ivf_results[0].0
        );
    }

    #[test]
    fn test_ivf_euclidean_negative_scores() {
        use crate::index::graph::CPIndex;
        use crate::index::IndexType;

        let config = HnswConfig {
            distance_metric: DistanceMetric::Euclidean,
            flat_threshold: None,
            index_type: IndexType::Ivf,
            ..HnswConfig::default()
        };
        let index = CPIndex::new_with_config(config);

        for i in 0u128..10 {
            let v = vec![(i as f32) * 2.0, (i as f32) * 2.0];
            index.add(i, FilterBitset::new(), VectorRepresentations::Full(v), 0);
        }

        let query = vec![0.0, 0.0];
        let results = index.search_nearest(&query, None, None, &FilterBitset::all_set(), 3, None);
        for &(_, score) in &results {
            assert!(
                score <= 0.0,
                "Euclidean IVF score must be <= 0, got {}",
                score
            );
        }
    }

    #[test]
    fn test_ivf_topk_zero() {
        let nodes = make_nodes(10);
        let ivf = IvfIndex::build(
            &nodes,
            &IvfConfig {
                nlist: 2,
                nprobe: 1,
                distance_metric: DistanceMetric::Cosine,
            },
        );
        let results = ivf.search(&[1.0, 0.0], 0, &FilterBitset::all_set());
        assert!(results.is_empty());
    }

    #[test]
    fn test_ivf_many_nodes() {
        // 100 nodes, nlist = sqrt(100) = 10
        let nodes = make_nodes(100);
        let ivf = IvfIndex::build(
            &nodes,
            &IvfConfig {
                nlist: 10,
                nprobe: 10, // probe all
                distance_metric: DistanceMetric::Cosine,
            },
        );
        assert_eq!(ivf.centroids.len(), 10);

        let query = vec![1.0, 0.0];
        let results = ivf.search(&query, 10, &FilterBitset::all_set());
        assert_eq!(results.len(), 10);
        // Top-1 should be near (1,0) which is id=0
        assert_eq!(results[0].0, 0, "closest node to (1,0) should be id=0");
    }
}
