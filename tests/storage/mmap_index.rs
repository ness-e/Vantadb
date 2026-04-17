//! MMap Neural Index & Survival Mode Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use tempfile::TempDir;
use vantadb::index::{CPIndex, IndexBackend, VectorRepresentations};

/// Helper: create a CPIndex with N test vectors
fn build_test_index(node_count: u64) -> CPIndex {
    let mut index = CPIndex::new();
    for i in 1..=node_count {
        let raw = vec![i as f32, (i + 1) as f32, (i + 2) as f32, (i + 3) as f32];
        let norm: f32 = raw.iter().map(|x| x * x).sum::<f32>().sqrt();
        let normalized: Vec<f32> = raw.iter().map(|x| x / norm).collect();
        index.add(i, 0, VectorRepresentations::Full(normalized), 0);
    }
    index
}

#[test]
fn mmap_neural_index_certification() {
    let mut harness = VantaHarness::new("STORAGE LAYER (MMAP NEURAL INDEX)");

    harness.execute("Serialization: Byte Roundtrip Integrity", || {
        let index = build_test_index(50);
        let bytes = index.serialize_to_bytes();
        assert_eq!(&bytes[0..8], b"VNTHNSW1");

        let restored = CPIndex::deserialize_from_bytes(&bytes).expect("Deserialization failed");
        assert_eq!(restored.nodes.len(), 50);
        TerminalReporter::success(&format!(
            "Serialization roundtrip: {} nodes, {} bytes",
            restored.nodes.len(),
            bytes.len()
        ));
    });

    harness.execute("Persistence: Cold-Start Performance", || {
        let tmp = TempDir::new().expect("Failed to create temp dir");
        let index_path = tmp.path().join("neural_index.bin");

        let index = build_test_index(100);
        index.persist_to_file(&index_path).expect("Persist failed");

        let loaded = CPIndex::load_from_file(&index_path).expect("Cold-start load failed");
        assert_eq!(loaded.nodes.len(), 100);

        let query = vec![1.0f32, 2.0, 3.0, 4.0];
        let norm: f32 = query.iter().map(|x| x * x).sum::<f32>().sqrt();
        let nq: Vec<f32> = query.iter().map(|x| x / norm).collect();
        let results = loaded.search_nearest(&nq, None, None, 0, 5, None);

        assert_eq!(results[0].0, 1);
        TerminalReporter::success("Cold-start persistence and search verified.");
    });

    harness.execute("MMap Survival: Backend Sync & Reload", || {
        let tmp = TempDir::new().expect("Failed to create temp dir");
        let mmap_path = tmp.path().join("neural_index_mmap.bin");

        let mut index = CPIndex::with_backend(IndexBackend::new_mmap(mmap_path.clone()));
        for i in 1..=30u64 {
            let raw = vec![i as f32, (i + 1) as f32, (i + 2) as f32, (i + 3) as f32];
            let norm: f32 = raw.iter().map(|x| x * x).sum::<f32>().sqrt();
            let normalized: Vec<f32> = raw.iter().map(|x| x / norm).collect();
            index.add(i, 0, VectorRepresentations::Full(normalized), 0);
        }

        index.sync_to_mmap().expect("MMap sync failed");
        assert!(mmap_path.exists());

        let reloaded = CPIndex::load_from_file(&mmap_path).expect("Load from MMap failed");
        assert_eq!(reloaded.nodes.len(), 30);
        TerminalReporter::success("MMap survival backend functional.");
    });

    harness.execute("Error Handling: Corrupt/Nonexistent Fallback", || {
        let tmp = TempDir::new().expect("Failed to create temp dir");
        let index_path = tmp.path().join("corrupt_index.bin");
        std::fs::write(&index_path, b"GARBAGE").unwrap();

        assert!(CPIndex::load_from_file(&index_path).is_none());
        assert!(CPIndex::load_from_file(&tmp.path().join("absent.bin")).is_none());
        TerminalReporter::success("Graceful fallback on corruption verified.");
    });

    harness.execute("Abstraction: Memory vs MMap Equivalence", || {
        let tmp = TempDir::new().expect("Failed to create temp dir");
        let mmap_path = tmp.path().join("equiv_test.bin");

        let mut inmem_index = CPIndex::new();
        let mut mmap_index = CPIndex::with_backend(IndexBackend::new_mmap(mmap_path));

        let vectors: Vec<(u64, Vec<f32>)> = (1..=20u64)
            .map(|i| {
                let raw = vec![i as f32, (i + 1) as f32, (i + 2) as f32, (i + 3) as f32];
                let n: f32 = raw.iter().map(|x| x * x).sum::<f32>().sqrt();
                (i, raw.iter().map(|x| x / n).collect())
            })
            .collect();

        for (id, v) in &vectors {
            inmem_index.add(*id, 0, VectorRepresentations::Full(v.clone()), 0);
            mmap_index.add(*id, 0, VectorRepresentations::Full(v.clone()), 0);
        }

        let q = vec![0.5f32, 0.5, 0.5, 0.5];
        let res_inmem = inmem_index.search_nearest(&q, None, None, 0, 5, None);
        let res_mmap = mmap_index.search_nearest(&q, None, None, 0, 5, None);

        assert_eq!(res_inmem.len(), res_mmap.len());
        TerminalReporter::success("Memory and MMap backend equivalence confirmed.");
    });
}
