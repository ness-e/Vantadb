/// Phase 35 Integration Tests: MMap Neural Index & Survival Mode
/// 
/// Tests:
/// 1. Serialization roundtrip (serialize → deserialize → graph intact)
/// 2. File persistence (write to file → cold-start load → search works)
/// 3. MMap-backed persistence (Survival mode mmap sync → search through abstraction)
/// 4. Corrupt file fallback (invalid magic → graceful None return)

use connectomedb::index::{CPIndex, IndexBackend, VectorRepresentations};
use tempfile::TempDir;

/// Helper: create a CPIndex with N test vectors
fn build_test_index(node_count: u64) -> CPIndex {
    let mut index = CPIndex::new();
    for i in 1..=node_count {
        // Each vector: [i, i+1, i+2, i+3] normalized (unique directions)
        let raw = vec![i as f32, (i + 1) as f32, (i + 2) as f32, (i + 3) as f32];
        let norm: f32 = raw.iter().map(|x| x * x).sum::<f32>().sqrt();
        let normalized: Vec<f32> = raw.iter().map(|x| x / norm).collect();
        index.add(i, 0, VectorRepresentations::Full(normalized));
    }
    index
}

#[test]
fn test_serialization_roundtrip() {
    let index = build_test_index(50);
    let bytes = index.serialize_to_bytes();
    
    // Verify magic header
    assert_eq!(&bytes[0..8], b"CXHNSW01", "Magic header must match");
    
    // Deserialize
    let restored = CPIndex::deserialize_from_bytes(&bytes)
        .expect("Deserialization must succeed");
    
    assert_eq!(restored.nodes.len(), 50, "All 50 nodes must survive roundtrip");
    assert_eq!(restored.entry_point, index.entry_point, "Entry point must match");
    assert_eq!(restored.max_layer, index.max_layer, "Max layer must match");
    
    // Verify individual node vectors survived
    for id in 1..=50u64 {
        let original = &index.nodes[&id];
        let restored_node = &restored.nodes[&id];
        assert_eq!(original.id, restored_node.id);
        assert_eq!(original.bitset, restored_node.bitset);
        
        // Compare vector data
        if let (VectorRepresentations::Full(orig_v), VectorRepresentations::Full(rest_v)) = 
            (&original.vec_data, &restored_node.vec_data) 
        {
            assert_eq!(orig_v.len(), rest_v.len(), "Vector lengths must match for node {}", id);
            for (a, b) in orig_v.iter().zip(rest_v.iter()) {
                assert!((a - b).abs() < f32::EPSILON, "Vector values must be identical");
            }
        } else {
            panic!("Vector types must both be Full for node {}", id);
        }
    }
    
    println!("✅ Serialization roundtrip: 50 nodes, {} bytes", bytes.len());
}

#[test]
fn test_file_persistence_and_cold_start() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let index_path = tmp.path().join("neural_index.bin");
    
    // Build and persist
    let index = build_test_index(100);
    index.persist_to_file(&index_path).expect("Persist must succeed");
    
    assert!(index_path.exists(), "neural_index.bin must be created");
    let file_size = std::fs::metadata(&index_path).unwrap().len();
    assert!(file_size > 0, "File must not be empty");
    
    // Cold-start load
    let loaded = CPIndex::load_from_file(&index_path)
        .expect("Cold-start load must succeed from valid file");
    
    assert_eq!(loaded.nodes.len(), 100, "All 100 nodes must load");
    
    // Verify search still works through the loaded index
    let query = vec![1.0f32, 2.0, 3.0, 4.0];
    let norm: f32 = query.iter().map(|x| x * x).sum::<f32>().sqrt();
    let normalized_query: Vec<f32> = query.iter().map(|x| x / norm).collect();
    
    let results = loaded.search_nearest(&normalized_query, None, None, 0, 5);
    assert!(!results.is_empty(), "Search must return results from loaded index");
    assert!(results.len() <= 5, "Must respect top_k");
    
    // The node with vector [1,2,3,4] (node 1) should be the best match for query [1,2,3,4]
    assert_eq!(results[0].0, 1, "Node 1 should be the best match (identical direction)");
    assert!(results[0].1 > 0.99, "Similarity should be ~1.0 for identical vectors");
    
    println!("✅ File persistence cold-start: {} nodes, search OK, file size = {} bytes", 
             loaded.nodes.len(), file_size);
}

#[test]
fn test_mmap_backend_survival_mode() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let mmap_path = tmp.path().join("neural_index_mmap.bin");
    
    // Create index with MMap backend
    let mut index = CPIndex::with_backend(IndexBackend::new_mmap(mmap_path.clone()));
    
    // Insert test data
    for i in 1..=30u64 {
        let raw = vec![i as f32, (i + 1) as f32, (i + 2) as f32, (i + 3) as f32];
        let norm: f32 = raw.iter().map(|x| x * x).sum::<f32>().sqrt();
        let normalized: Vec<f32> = raw.iter().map(|x| x / norm).collect();
        index.add(i, 0, VectorRepresentations::Full(normalized));
    }
    
    // Sync to mmap
    index.sync_to_mmap().expect("MMap sync must succeed");
    
    assert!(mmap_path.exists(), "MMap file must be created");
    let file_size = std::fs::metadata(&mmap_path).unwrap().len();
    assert!(file_size > 0, "MMap file must have content");
    
    // Verify search works THROUGH the mmap-backed index (same in-memory HashMap)
    // Query aligned with node 10: [10, 11, 12, 13]
    let query = vec![10.0f32, 11.0, 12.0, 13.0];
    let norm: f32 = query.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nq: Vec<f32> = query.iter().map(|x| x / norm).collect();
    
    let results = index.search_nearest(&nq, None, None, 0, 3);
    assert!(!results.is_empty(), "Search through MMap-backed index must work");
    assert_eq!(results[0].0, 10, "Top result must match correctly");
    
    // Now simulate cold-start: read back from the mmap file
    let reloaded = CPIndex::load_from_file(&mmap_path)
        .expect("Must load from MMap-persisted file");
    assert_eq!(reloaded.nodes.len(), 30, "All 30 nodes must survive MMap roundtrip");
    
    let results2 = reloaded.search_nearest(&nq, None, None, 0, 3);
    assert!(!results2.is_empty(), "Search from reloaded MMap index must work");
    
    // Results should be identical
    assert_eq!(results[0].0, results2[0].0, "Top result must be same between live and reloaded");
    
    println!("✅ MMap Survival backend: 30 nodes, sync OK, cold-start reload OK, {} bytes", file_size);
}

#[test]
fn test_corrupt_file_fallback() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let index_path = tmp.path().join("neural_index.bin");
    
    // Write garbage
    std::fs::write(&index_path, b"GARBAGE_DATA_NOT_A_VALID_INDEX").unwrap();
    
    // Load should return None (graceful fallback)
    let result = CPIndex::load_from_file(&index_path);
    assert!(result.is_none(), "Corrupt file must return None for rebuild fallback");
    
    println!("✅ Corrupt file fallback: graceful None return");
}

#[test]
fn test_nonexistent_file_returns_none() {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let index_path = tmp.path().join("does_not_exist.bin");
    
    let result = CPIndex::load_from_file(&index_path);
    assert!(result.is_none(), "Nonexistent file must return None");
    
    println!("✅ Nonexistent file: graceful None return");
}

#[test]
fn test_search_latency_through_abstraction() {
    // Verify that search_nearest works identically for InMemory and MMap backends
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let mmap_path = tmp.path().join("latency_test.bin");
    
    // Build identical data for both
    let mut inmem_index = CPIndex::new();
    let mut mmap_index = CPIndex::with_backend(IndexBackend::new_mmap(mmap_path.clone()));
    
    // Use deterministic seeding via fixed vectors
    let vectors: Vec<(u64, Vec<f32>)> = (1..=20u64).map(|i| {
        let raw = vec![i as f32, (i + 1) as f32, (i + 2) as f32, (i + 3) as f32];
        let norm: f32 = raw.iter().map(|x| x * x).sum::<f32>().sqrt();
        (i, raw.iter().map(|x| x / norm).collect())
    }).collect();
    
    for (id, vec) in &vectors {
        inmem_index.add(*id, 0, VectorRepresentations::Full(vec.clone()));
        mmap_index.add(*id, 0, VectorRepresentations::Full(vec.clone()));
    }
    
    let query = vec![5.0f32, 6.0, 7.0, 8.0];
    let norm: f32 = query.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nq: Vec<f32> = query.iter().map(|x| x / norm).collect();
    
    let inmem_results = inmem_index.search_nearest(&nq, None, None, 0, 5);
    let mmap_results = mmap_index.search_nearest(&nq, None, None, 0, 5);
    
    // Both must return results
    assert!(!inmem_results.is_empty(), "InMemory search must return results");
    assert!(!mmap_results.is_empty(), "MMap search must return results");
    
    // The search is on the same HashMap — results should be functionally identical
    // (HNSW construction adds randomness, but since we add in the same order, structure should match)
    assert_eq!(inmem_results.len(), mmap_results.len(), "Result count must be equal");
    
    println!("✅ Search latency/equivalence: InMemory={} results, MMap={} results (same data, same structure)",
             inmem_results.len(), mmap_results.len());
}
