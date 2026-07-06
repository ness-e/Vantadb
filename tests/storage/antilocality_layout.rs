//! Certification test suite for physical HNSW node alignment (antilocality layout).
//! Verifies BFS ordering layout, search equivalence, monotonicity, and correctness.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use std::collections::{HashSet, VecDeque};
use tempfile::tempdir;
use vantadb::config::VantaConfig;
use vantadb::node::UnifiedNode;
use vantadb::storage::{BackendKind, StorageEngine};

/// Genera un vector pseudo-aleatorio determinista de la dimensión especificada usando un LCG.
fn generate_deterministic_vector(seed: u64, dim: usize) -> Vec<f32> {
    let mut state = seed;
    let mut vec = Vec::with_capacity(dim);
    for _ in 0..dim {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let val = (state >> 32) as f32 / (u32::MAX as f32);
        vec.push(val);
    }
    // Normalizar para Cosine similarity
    let sum_sq: f32 = vec.iter().map(|x| x * x).sum();
    let norm = sum_sq.sqrt();
    if norm > 0.0 {
        for val in &mut vec {
            *val /= norm;
        }
    }
    vec
}

#[test]
fn antilocality_layout_certification() {
    TerminalReporter::suite_banner("MMAP ANTILOCALITY LAYOUT COMPACTION CERTIFICATION", 3);
    let mut harness = VantaHarness::new("STORAGE LAYER (ANTILOCALITY LAYOUT)");

    harness.execute("BFS Compaction: Monotonicity and Offset Contiguity", || {
        let dir = tempdir().unwrap();
        let db_path = dir.path().to_str().unwrap();

        TerminalReporter::sub_step("Opening StorageEngine...");
        let config = VantaConfig {
            backend_kind: BackendKind::Fjall,
            ..Default::default()
        };
        let engine =
            StorageEngine::open_with_config(db_path, Some(config)).expect("Failed to open engine");

        TerminalReporter::sub_step("Inserting 200 nodes with random vectors...");
        for id in 1..=200u64 {
            let vec = generate_deterministic_vector(id, 16);
            let node = UnifiedNode::with_vector(id.into(), vec);
            engine.insert(&node).unwrap();
        }

        TerminalReporter::sub_step("Flushing WAL to persist inserts...");
        engine.flush().unwrap();

        TerminalReporter::sub_step("Running compact_layout_bfs...");
        let compacted_count = engine.compact_layout_bfs().expect("Compaction failed");
        assert_eq!(compacted_count, 200, "Should compact exactly 200 nodes");

        TerminalReporter::sub_step("Validating index structural consistency...");
        let hnsw = engine.hnsw.load();
        assert!(
            hnsw.validate_index().is_ok(),
            "HNSW graph validation failed post-compaction"
        );

        TerminalReporter::sub_step("Checking BFS offset monotonicity...");
        let entry_point_id = hnsw.get_entry_point().expect("Missing entry point");
        let mut bfs_order: Vec<u128> = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(entry_point_id);
        visited.insert(entry_point_id);

        while let Some(node_id) = queue.pop_front() {
            bfs_order.push(node_id);
            if let Some(node_ref) = hnsw.nodes.get(&node_id) {
                if let Some(layer0_neighbors) = node_ref.neighbors.first() {
                    for &neighbor_id in layer0_neighbors {
                        if visited.insert(neighbor_id) {
                            queue.push_back(neighbor_id);
                        }
                    }
                }
            }
        }

        // Agregar nodos aislados (si los hay)
        for entry in hnsw.nodes.iter() {
            let node_id: u128 = *entry.key();
            if visited.insert(node_id) {
                bfs_order.push(node_id);
            }
        }

        let mut last_offset = 0;
        for &node_id in &bfs_order {
            let node_ref = hnsw.nodes.get(&node_id).expect("Node should be in HNSW");
            let offset = node_ref.storage_offset;
            assert!(
                offset > last_offset,
                "Offsets are not strictly increasing. Node: {}, Offset: {}, Last: {}",
                node_id,
                offset,
                last_offset
            );
            last_offset = offset;
        }

        TerminalReporter::success(
            "BFS compaction successfully rearranged storage offsets monotonically.",
        );
    });

    harness.execute("Search Equivalence: Pre/Post Compaction Parity", || {
        let dir = tempdir().unwrap();
        let db_path = dir.path().to_str().unwrap();

        let config = VantaConfig {
            backend_kind: BackendKind::Fjall,
            ..Default::default()
        };
        let engine =
            StorageEngine::open_with_config(db_path, Some(config)).expect("Failed to open engine");

        for id in 1..=200u64 {
            let vec = generate_deterministic_vector(id * 7, 16);
            let node = UnifiedNode::with_vector(id.into(), vec);
            engine.insert(&node).unwrap();
        }
        engine.flush().unwrap();

        // Generar consultas deterministas para verificar equivalencia de resultados
        let mut queries = Vec::new();
        for q_id in 0..10u64 {
            queries.push(generate_deterministic_vector(q_id * 100, 16));
        }

        let mut pre_results = Vec::new();
        {
            let hnsw = engine.hnsw.load();
            let vs = engine.vector_store.read();
            for query in &queries {
                let hits = hnsw.search_nearest(
                    query,
                    None,
                    None,
                    &vantadb::node::ALL_BITSET,
                    5,
                    Some(&vs),
                );
                pre_results.push(hits);
            }
        }

        TerminalReporter::sub_step("Executing compact_layout_bfs...");
        let compacted_count = engine.compact_layout_bfs().expect("Compaction failed");
        assert_eq!(compacted_count, 200);

        let mut post_results = Vec::new();
        {
            let hnsw = engine.hnsw.load();
            let vs = engine.vector_store.read();
            for query in &queries {
                let hits = hnsw.search_nearest(
                    query,
                    None,
                    None,
                    &vantadb::node::ALL_BITSET,
                    5,
                    Some(&vs),
                );
                post_results.push(hits);
            }
        }

        assert_eq!(
            pre_results, post_results,
            "Search results must be identical pre and post physical layout compaction"
        );

        // Validar que todos los nodos siguen existiendo
        for id in 1..=200u128 {
            let node = engine
                .get(id)
                .unwrap()
                .expect("Node missing post-compaction");
            assert_eq!(node.id, id);
        }

        TerminalReporter::success("Pre/Post search results match perfectly, reachability intact.");
    });

    harness.execute("Edge Case: Compaction on Empty Database", || {
        let dir = tempdir().unwrap();
        let db_path = dir.path().to_str().unwrap();

        let config = VantaConfig {
            backend_kind: BackendKind::Fjall,
            ..Default::default()
        };
        let engine =
            StorageEngine::open_with_config(db_path, Some(config)).expect("Failed to open engine");

        let compacted_count = engine
            .compact_layout_bfs()
            .expect("Empty compaction failed");
        assert_eq!(
            compacted_count, 0,
            "Empty database compaction should do nothing and return 0"
        );

        let hnsw = engine.hnsw.load();
        assert!(
            hnsw.validate_index().is_ok(),
            "Empty index structure should be valid"
        );

        TerminalReporter::success("Empty database compaction handled gracefully.");
    });
}
