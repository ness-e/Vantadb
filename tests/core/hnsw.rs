//! HNSW Algorithm Core Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use vantadb::index::{CPIndex, VectorRepresentations};

#[test]
fn hnsw_core_logic_certification() {
    let mut harness = VantaHarness::new("CORE ENGINE (HNSW LOGIC)");

    harness.execute("Vector Math: Cosine Similarity Axioms", || {
        TerminalReporter::sub_step(
            "Verifying Identical (1.0), Orthogonal (0.0), and Opposite (-1.0) vectors...",
        );
        let a = VectorRepresentations::Full(vec![1.0, 0.0, 0.0]);
        let b = VectorRepresentations::Full(vec![1.0, 0.0, 0.0]);
        let sim = a.cosine_similarity(&b).unwrap();
        assert!((sim - 1.0).abs() < f32::EPSILON);

        let c = VectorRepresentations::Full(vec![0.0, 1.0, 0.0]);
        let sim_orthogonal = a.cosine_similarity(&c).unwrap();
        assert!(sim_orthogonal.abs() < f32::EPSILON);

        let d = VectorRepresentations::Full(vec![-1.0, 0.0, 0.0]);
        let sim_opposite = a.cosine_similarity(&d).unwrap();
        assert!((sim_opposite - (-1.0)).abs() < f32::EPSILON);

        TerminalReporter::success("Algebraic consistency confirmed.");
    });

    harness.execute("HNSW: Greedy Search Integrity", || {
        let mut index = CPIndex::new();
        TerminalReporter::sub_step("Populating sparse vector space...");
        index.add(1, 0, VectorRepresentations::Full(vec![1.0, 0.0, 0.0]), 0);
        index.add(2, 0, VectorRepresentations::Full(vec![0.8, 0.2, 0.0]), 0);
        index.add(3, 0, VectorRepresentations::Full(vec![0.0, 1.0, 0.0]), 0);
        index.add(4, 0, VectorRepresentations::Full(vec![0.0, 0.8, 0.2]), 0);

        let query = vec![0.0, 0.9, 0.1];
        let results = index.search_nearest(&query, None, None, 0, 2, None);

        assert_eq!(results.len(), 2);
        let top_match = results[0].0;
        assert!(top_match == 3 || top_match == 4);

        TerminalReporter::success("Greedy search converged on expected neighbors.");
    });
}
