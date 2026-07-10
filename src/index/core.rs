#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use crate::index::distance::{cosine_sim_f32, cosine_sim_with_query_norm, f32_l2_norm};
    use crate::index::*;
    use crate::node::DistanceMetric;
    use rand::Rng;

    #[test]
    fn cosine_with_precomputed_query_norm_matches_full_path() {
        let a = vec![0.12, 0.88, 0.54, 0.31];
        let b = vec![0.11, 0.89, 0.55, 0.30];
        let norm_a = f32_l2_norm(&a);
        let expected = cosine_sim_f32(&a, &b);
        let optimized = cosine_sim_with_query_norm(&a, norm_a, &b);
        assert!(
            (expected - optimized).abs() < 1e-6,
            "expected {expected}, got {optimized}"
        );
    }

    #[test]
    fn serialization_order_preserves_search_results() {
        let index = CPIndex::new_with_config(HnswConfig {
            m: 8,
            m_max0: 16,
            ef_construction: 64,
            ef_search: 32,
            ml: 1.0 / (8_f64).ln(),
            distance_metric: DistanceMetric::Cosine,
            ..HnswConfig::default()
        });

        for i in 0..64u128 {
            let raw = [
                (i as f32 * 0.01).sin(),
                (i as f32 * 0.02).cos(),
                (i as f32 * 0.03).sin(),
                (i as f32 * 0.04).cos(),
            ];
            let norm = f32_l2_norm(&raw);
            let normalized: Vec<f32> = raw.iter().map(|v| v / norm).collect();
            index.add(
                i + 1,
                FilterBitset::new(),
                VectorRepresentations::Full(normalized),
                0,
            );
        }

        let query = vec![0.1, 0.9, 0.2, 0.4];
        let before = index.search_nearest(&query, None, None, &crate::node::ALL_BITSET, 5, None);

        let bytes = index.serialize_to_bytes();
        let restored = CPIndex::deserialize_from_bytes(&bytes, true).expect("deserialize");
        let after = restored.search_nearest(&query, None, None, &crate::node::ALL_BITSET, 5, None);

        assert_eq!(before, after);
        assert_eq!(restored.nodes.len(), index.nodes.len());
    }

    #[test]
    fn concurrent_search_during_insert() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;
        use std::sync::Mutex;
        use std::thread;
        use std::time::Duration;

        let index = Arc::new(CPIndex::new_with_config(HnswConfig {
            m: 16,
            m_max0: 32,
            ef_construction: 64,
            ef_search: 32,
            ml: 1.0 / (16_f64).ln(),
            distance_metric: DistanceMetric::Cosine,
            ..HnswConfig::default()
        }));

        let stop = Arc::new(AtomicBool::new(false));
        let insert_mutex = Arc::new(Mutex::new(()));
        let mut handles = Vec::new();

        for t in 0..2 {
            let index = index.clone();
            let stop = stop.clone();
            let insert_mutex = insert_mutex.clone();
            handles.push(thread::spawn(move || {
                let mut rng = rand::rng();
                let start_id = t * 1000;
                for i in 0..1000 {
                    if stop.load(Ordering::Relaxed) {
                        break;
                    }
                    let id = (start_id + i) as u128;
                    let raw_vec: Vec<f32> = (0..32).map(|_| rng.random::<f32>()).collect();
                    let norm = f32_l2_norm(&raw_vec);
                    let vec: Vec<f32> = if norm > 0.0 {
                        raw_vec.iter().map(|v| v / norm).collect()
                    } else {
                        raw_vec
                    };

                    let _guard = insert_mutex.lock().unwrap();
                    index.add(
                        id,
                        FilterBitset::all_set(),
                        VectorRepresentations::Full(vec),
                        0,
                    );
                }
            }));
        }

        for _ in 0..4 {
            let index = index.clone();
            let stop = stop.clone();
            handles.push(thread::spawn(move || {
                let mut rng = rand::rng();
                while !stop.load(Ordering::Relaxed) {
                    let query: Vec<f32> = (0..32).map(|_| rng.random::<f32>()).collect();
                    let norm = f32_l2_norm(&query);
                    let q_vec = if norm > 0.0 {
                        query.iter().map(|v| v / norm).collect()
                    } else {
                        query
                    };
                    let _res =
                        index.search_nearest(&q_vec, None, None, &crate::node::ALL_BITSET, 5, None);
                    thread::sleep(Duration::from_micros(10));
                }
            }));
        }

        thread::sleep(Duration::from_millis(1000));
        stop.store(true, Ordering::Relaxed);

        for handle in handles {
            let _ = handle.join();
        }

        assert!(index.validate_index().is_ok());
    }

    #[test]
    fn concurrent_insert_preserves_hnsw_invariants() {
        use crate::node::UnifiedNode;
        use crate::storage::engine::StorageEngine;
        use std::sync::Arc;
        use std::thread;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let db_path = dir.path().to_str().unwrap();
        let storage = Arc::new(StorageEngine::open(db_path).unwrap());

        let mut handles = Vec::new();
        for t in 0..4 {
            let storage = storage.clone();
            handles.push(thread::spawn(move || {
                let mut rng = rand::rng();
                let start_id = t * 500 + 1;
                for i in 0..500 {
                    let id = (start_id + i) as u128;
                    let raw_vec: Vec<f32> = (0..32).map(|_| rng.random::<f32>()).collect();
                    let norm = f32_l2_norm(&raw_vec);
                    let vec: Vec<f32> = if norm > 0.0 {
                        raw_vec.iter().map(|v| v / norm).collect()
                    } else {
                        raw_vec
                    };

                    let mut node = UnifiedNode::new(id);
                    node.vector = VectorRepresentations::Full(vec);
                    storage.insert(&node).unwrap();
                }
            }));
        }

        for handle in handles {
            let _ = handle.join();
        }

        let hnsw = storage.hnsw.load();
        assert!(hnsw.validate_index().is_ok());

        let ep = hnsw.get_entry_point().expect("Should have entry point");
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(ep);
        visited.insert(ep);

        while let Some(node_id) = queue.pop_front() {
            if let Some(node) = hnsw.nodes.get(&node_id) {
                for layer in &node.neighbors {
                    for &neighbor in layer {
                        if visited.insert(neighbor) {
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
        }

        assert_eq!(
            visited.len(),
            hnsw.nodes.len(),
            "Not all nodes are reachable from the entry point!"
        );
    }

    fn build_small_test_index() -> CPIndex {
        let index = CPIndex::new_with_config(HnswConfig {
            m: 8,
            m_max0: 16,
            ef_construction: 64,
            ef_search: 32,
            ml: 1.0 / (8_f64).ln(),
            distance_metric: DistanceMetric::Cosine,
            ..HnswConfig::default()
        });
        for i in 0..16u128 {
            let raw = [
                (i as f32 * 0.01).sin(),
                (i as f32 * 0.02).cos(),
                (i as f32 * 0.03).sin(),
                (i as f32 * 0.04).cos(),
            ];
            let norm = f32_l2_norm(&raw);
            let normalized: Vec<f32> = raw.iter().map(|v| v / norm).collect();
            index.add(
                i + 1,
                FilterBitset::from_u128(0),
                VectorRepresentations::Full(normalized),
                0,
            );
        }
        index
    }

    #[test]
    fn deserialize_truncated_never_panics() {
        let index = build_small_test_index();
        let bytes = index.serialize_to_bytes();
        for len in 0..bytes.len() {
            let result = CPIndex::deserialize_from_bytes(&bytes[..len], true);
            assert!(
                result.is_err(),
                "Expected Err for truncated input at {len}/{} bytes, got Ok",
                bytes.len()
            );
        }
        let full = CPIndex::deserialize_from_bytes(&bytes, true);
        assert!(
            full.is_ok(),
            "Full bytes must deserialize: {:?}",
            full.err()
        );
    }

    #[test]
    fn deserialize_garbage_after_valid_header() {
        let mut garbage = vec![0u8; 512];
        let header =
            crate::binary_header::VantaHeader::new(*b"VNDX", graph::VECTOR_INDEX_VERSION, 0);
        let hdr = header.serialize();
        garbage[..hdr.len()].copy_from_slice(&hdr);
        let result = CPIndex::deserialize_from_bytes(&garbage, true);
        assert!(result.is_err() || result.unwrap().nodes.is_empty());
    }

    #[test]
    fn sync_to_mmap_preserves_data() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vector_index.bin");

        let index = build_small_test_index();
        index.persist_to_file(&path).expect("persist_to_file");
        drop(index);

        let mut loaded = CPIndex::load_from_file(&path, true).expect("load mmap");
        let query = vec![0.1, 0.9, 0.2, 0.4];
        let before = loaded.search_nearest(&query, None, None, &crate::node::ALL_BITSET, 5, None);

        let raw = [99f32.sin(), 99f32.cos(), 99f32.sin(), 99f32.cos()];
        let norm = f32_l2_norm(&raw);
        let normalized: Vec<f32> = raw.iter().map(|v| v / norm).collect();
        loaded.add(
            999,
            FilterBitset::new(),
            VectorRepresentations::Full(normalized),
            0,
        );

        loaded.sync_to_mmap().expect("sync_to_mmap");

        let reloaded = CPIndex::load_from_file(&path, true).expect("reload mmap");
        assert_eq!(reloaded.nodes.len(), loaded.nodes.len());
        let after = reloaded.search_nearest(&query, None, None, &crate::node::ALL_BITSET, 5, None);
        assert_eq!(before, after);
    }

    fn node_count_offset() -> usize {
        let header_size = crate::binary_header::VantaHeader::SIZE;
        let max_layer = 8;
        let config = 5 * 8;
        let metric_byte = 1;
        let ep_exists = 1;
        let ep_id = 16;
        header_size + max_layer + config + metric_byte + ep_exists + ep_id
    }

    #[test]
    fn persist_and_load_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vector_index.bin");

        let index = build_small_test_index();
        index.persist_to_file(&path).expect("persist_to_file");

        let loaded = CPIndex::load_from_file(&path, false).expect("load_from_file");
        assert_eq!(loaded.nodes.len(), index.nodes.len());

        let query = vec![0.1, 0.9, 0.2, 0.4];
        let before = index.search_nearest(&query, None, None, &crate::node::ALL_BITSET, 5, None);
        let after = loaded.search_nearest(&query, None, None, &crate::node::ALL_BITSET, 5, None);
        assert_eq!(before, after);
    }

    #[test]
    fn validate_index_detects_self_loop_and_broken_reference() {
        let index = build_small_test_index();

        assert!(index.validate_index().is_ok(), "clean index must pass");

        let first_id = *index.nodes.iter().next().unwrap().key();
        let second_id = *index.nodes.iter().skip(1).next().unwrap().key();

        if let Some(mut node) = index.nodes.get_mut(&first_id) {
            if !node.neighbors.is_empty() && !node.neighbors[0].is_empty() {
                node.neighbors[0].push(second_id + 9999);
            }
        }

        let result = index.validate_index();
        assert!(result.is_err(), "Must detect broken reference");
        let msgs = result.unwrap_err();
        assert!(
            msgs.iter().any(|m| m.contains("non-existent")),
            "Must mention non-existent neighbor: {msgs:?}"
        );
    }

    #[test]
    fn validate_index_empty_index() {
        let index = CPIndex::new_with_config(HnswConfig {
            m: 8,
            m_max0: 16,
            ef_construction: 64,
            ef_search: 32,
            ml: 1.0 / (8_f64).ln(),
            distance_metric: DistanceMetric::Cosine,
            ..HnswConfig::default()
        });
        assert!(index.validate_index().is_ok(), "empty index must pass");
        let st = index.stats();
        assert_eq!(st.node_count, 0);
        assert_eq!(st.max_layer, 0);
        assert_eq!(st.orphan_count, 0);
    }

    #[test]
    fn stats_after_insertions() {
        let index = build_small_test_index();
        let st = index.stats();
        assert_eq!(st.node_count, 16);
        assert!(st.max_layer > 0);
        assert_eq!(st.violation_count, 0);
        assert!(st.avg_connections_l0 > 0.0);
    }

    #[test]
    fn deserialize_absurd_node_count() {
        let index = build_small_test_index();
        let mut bytes = index.serialize_to_bytes();

        let offset = node_count_offset();
        assert!(
            offset + 8 <= bytes.len(),
            "node_count offset {} exceeds buffer len {}",
            offset,
            bytes.len()
        );
        bytes[offset..offset + 8].copy_from_slice(&u64::MAX.to_le_bytes());
        let result = CPIndex::deserialize_from_bytes(&bytes, true);
        assert!(result.is_err(), "Absurd node_count must return Err");
    }

    #[test]
    fn flat_search_matches_hnsw_on_small_dataset() {
        let index = CPIndex::new_with_config(HnswConfig {
            flat_threshold: Some(100),
            ..HnswConfig::default()
        });

        for i in 0..20u128 {
            let raw = [
                (i as f32 * 0.1).sin(),
                (i as f32 * 0.2).cos(),
                (i as f32 * 0.3).sin(),
                (i as f32 * 0.4).cos(),
            ];
            let norm = f32_l2_norm(&raw);
            let normalized: Vec<f32> = raw.iter().map(|v| v / norm).collect();
            index.add(
                i + 1,
                FilterBitset::new(),
                VectorRepresentations::Full(normalized),
                0,
            );
        }

        let query = vec![0.1, 0.9, 0.2, 0.4];
        let results = index.search_nearest(&query, None, None, &crate::node::ALL_BITSET, 5, None);

        assert_eq!(results.len(), 5, "flat search should return top_k results");
        for (_id, score) in &results {
            assert!(!score.is_nan(), "flat search scores must not be NaN");
        }
        for w in results.windows(2) {
            assert!(
                w[0].1 >= w[1].1 - f32::EPSILON,
                "flat search scores must be descending: {} < {}",
                w[0].1,
                w[1].1
            );
        }
    }

    #[test]
    fn flat_search_used_when_under_threshold() {
        let index = CPIndex::new_with_config(HnswConfig {
            flat_threshold: Some(50),
            ..HnswConfig::default()
        });

        for i in 0..10u128 {
            index.add(
                i + 1,
                FilterBitset::new(),
                VectorRepresentations::Full(vec![i as f32; 4]),
                0,
            );
        }

        let query = vec![0.0; 4];
        let results = index.search_nearest(&query, None, None, &crate::node::ALL_BITSET, 3, None);
        assert_eq!(results.len(), 3);
    }
}
