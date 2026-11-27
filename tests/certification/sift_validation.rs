// ponytail: blanket allow — unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::Path;

#[path = "../common/mod.rs"]
mod common;

use common::sift_loader::{read_fvecs, read_ivecs};

#[test]
fn validate_sift_dataset_integrity() {
    let base_path = Path::new("datasets/sift/sift_base.fvecs");
    let query_path = Path::new("datasets/sift/sift_query.fvecs");
    let groundtruth_path = Path::new("datasets/sift/sift_groundtruth.ivecs");

    // FIND-664 (TBH-01): a missing dataset FAILS, naming the file. This test
    // used to `return` green, so `certification.yml`'s sift-validation job
    // reported success while asserting nothing.
    if !common::require_datasets(
        &[base_path, query_path, groundtruth_path],
        "SIFT-1M",
        "python scripts/download_ground_truth.py --datasets sift-1m",
    ) {
        return;
    }

    println!("Loading SIFT1M base vectors...");
    let base = read_fvecs(base_path).expect("Failed to read base.fvecs");

    println!("Loading SIFT1M query vectors...");
    let query = read_fvecs(query_path).expect("Failed to read query.fvecs");

    println!("Loading SIFT1M ground truth...");
    let groundtruth = read_ivecs(groundtruth_path).expect("Failed to read groundtruth.ivecs");

    // Phase 2.1 Validation Logic from Roadmap
    assert_eq!(base.len(), 1_000_000, "Base must have 1M vectors");
    assert_eq!(base[0].len(), 128, "Base vectors must be 128D");

    assert_eq!(query.len(), 10_000, "Query must have 10K vectors");
    assert_eq!(query[0].len(), 128, "Query vectors must be 128D");

    assert_eq!(
        groundtruth.len(),
        10_000,
        "Groundtruth must have 10K entries"
    );
    assert_eq!(
        groundtruth[0].len(),
        100,
        "Groundtruth usually provides top 100 nearest neighbors"
    );

    println!("SIFT1M Dataset Integrity: PASSED ✅");
}
