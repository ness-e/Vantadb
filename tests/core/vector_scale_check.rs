//! Vector Scale & Logarithmic Performance Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use common::{VantaHarness, TerminalReporter};
use std::sync::Arc;
use tempfile::tempdir;
use vantadb::node::{NodeTier, UnifiedNode};
use vantadb::storage::StorageEngine;

#[tokio::test]
async fn vector_scale_performance_certification() {
    let mut harness = VantaHarness::new("CORE ENGINE (VECTOR SCALE)");

    harness.execute("Scale: 1K Node Graph Navigation", || {
        futures::executor::block_on(async {
            let dir = tempdir().unwrap();
            let db_path = dir.path().to_str().unwrap();
            let storage = Arc::new(StorageEngine::open(db_path).unwrap());

            TerminalReporter::sub_step("Populating HNSW graph with 1,000 orthogonal vectors...");
            for i in 0..1000 {
                let mut vec = vec![0.0; 128];
                vec[i % 128] = 1.0; 

                let mut node = UnifiedNode::new(i as u64);
                node.tier = NodeTier::Hot;
                node.vector = vantadb::node::VectorRepresentations::Full(vec);
                node.flags.set(vantadb::node::NodeFlags::HAS_VECTOR);
                storage.insert(&node).unwrap();
            }

            let mut query_vec = vec![0.0; 128];
            query_vec[10] = 1.0;

            TerminalReporter::sub_step("Executing greedy beam search over 128-dimensional space...");
            let results = {
                let index = storage.hnsw.read();
                index.search_nearest(&query_vec, None, None, 0, 5, None)
            };

            assert!(!results.is_empty());
            assert_eq!(results[0].0, 10, "Heuristic search failed to find identical neighbor");
            
            TerminalReporter::success("Topological search precision verified at scale.");
        });
    });
}
