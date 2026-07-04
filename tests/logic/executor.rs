//! Query Executor & Result Projection Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use vantadb::index::{CPIndex, VectorRepresentations};
use vantadb::node::FilterBitset;

#[test]
fn engine_executor_certification() {
    let mut harness = VantaHarness::new("LOGIC LAYER (QUERY EXECUTOR)");

    harness.execute("Math: Cosine Similarity Projection", || {
        let vec_a = VectorRepresentations::Full(vec![1.0, 0.0, 0.0]);
        let vec_b = VectorRepresentations::Full(vec![1.0, 0.0, 0.0]);
        let vec_c = VectorRepresentations::Full(vec![0.0, 1.0, 0.0]);

        assert!((vec_a.cosine_similarity(&vec_b).unwrap() - 1.0).abs() < f32::EPSILON);
        assert!((vec_a.cosine_similarity(&vec_c).unwrap() - 0.0).abs() < f32::EPSILON);
        TerminalReporter::success("Scalar similarity math validated.");
    });

    harness.execute("Search: Bitset + Nearest Neighbor Projection", || {
        let idx = CPIndex::new();
        TerminalReporter::sub_step("Setting up tiered bitmask dataset...");
        // Match mask + High sim
        idx.add(
            1,
            FilterBitset::from(0b11u128),
            VectorRepresentations::Full(vec![1.0, 0.0]),
            0,
        );
        // Match mask + Low sim
        idx.add(
            2,
            FilterBitset::from(0b11u128),
            VectorRepresentations::Full(vec![0.0, 1.0]),
            0,
        );
        // Fails mask
        idx.add(
            3,
            FilterBitset::new(),
            VectorRepresentations::Full(vec![1.0, 0.0]),
            0,
        );

        let filter_mask = FilterBitset::from(0b10u128);
        let res = idx.search_nearest(&[1.0, 0.0], None, None, &filter_mask, 2, None);

        assert_eq!(res.len(), 2, "Failed to ignore bitset-filtered nodes");
        assert_eq!(res[0].0, 1, "Incorrect result ranking");
        assert_eq!(res[1].0, 2);

        TerminalReporter::success("Bitset filter and NN ranking integrated correctly.");
    });
}
