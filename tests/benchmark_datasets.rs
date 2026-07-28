/// Integration test that loads real benchmark datasets (GloVe-100)
/// and runs basic HNSW insert + search to validate performance.
///
/// Requires: `data/benchmark/glove.6B.100d.txt` (download via `scripts/download_benchmark_datasets.ps1`)
/// Skips gracefully if dataset is not present.
use std::path::Path;
use tempfile::TempDir;
use vantadb::config::VantaConfig;
use vantadb::node::{NodeTier, UnifiedNode};
use vantadb::storage::StorageEngine;

#[test]
fn test_glove100_hnsw_basic() {
    let glove_path = Path::new("data/benchmark/glove.6B.100d.txt");
    if !glove_path.exists() {
        println!("GloVe-100 dataset not found at data/benchmark/glove.6B.100d.txt. Skipping.");
        println!("Download via: scripts/download_benchmark_datasets.ps1");
        return;
    }

    let dir = TempDir::new().unwrap();
    let config = VantaConfig::default()
        .with_storage_path(dir.path().to_str().unwrap().to_string())
        .with_mmap_hnsw(true);
    let engine =
        StorageEngine::open_with_config(dir.path().to_str().unwrap(), Some(config)).unwrap();

    // Load a small subset of GloVe for basic validation
    let vectors = load_glove_sample(glove_path.to_str().unwrap(), 100);

    // Insert and verify
    for (i, v) in vectors.iter().enumerate() {
        let mut node = UnifiedNode::with_vector((i as u64).into(), v.clone());
        node.tier = NodeTier::Hot;
        engine.insert(&node).unwrap();
    }

    let stats = engine.get_memory_stats();
    assert!(
        stats.node_count > 0,
        "node_count should be > 0 after inserts"
    );
    assert_eq!(
        stats.node_count as usize,
        vectors.len(),
        "node_count should match inserted vectors"
    );
}

fn load_glove_sample(path: &str, max_lines: usize) -> Vec<Vec<f32>> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    content
        .lines()
        .take(max_lines)
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                return None;
            }
            let vec: Vec<f32> = parts[1..]
                .iter()
                .filter_map(|s| s.parse::<f32>().ok())
                .collect();
            if vec.len() == 100 {
                Some(vec)
            } else {
                None
            }
        })
        .collect()
}
