//! Hardware Profiles Certification — Vanta Certification Edition
//!
//! Validates hardware detection and emergency threshold logic.
//! Sequential execution via a unified entry point to avoid console overlapping.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use console::style;
use std::sync::Arc;
use vantadb::storage::StorageEngine;

fn temp_storage() -> Arc<StorageEngine> {
    let dir = tempfile::tempdir().expect("Failed to create temp dir");
    Arc::new(StorageEngine::open(dir.path().to_str().unwrap()).expect("Failed to open storage"))
}

#[tokio::test]
async fn hardware_certification_full() {
    let mut harness = VantaHarness::new("HARDWARE CERTIFICATION");

    // BLOCK 1: Emergency Logic
    harness.execute("Thermal & OOM Thresholds", || {
        let storage = temp_storage();
        TerminalReporter::sub_step("Verifying maintenance triggers...");
        // Verify that it starts false
        assert!(!storage
            .emergency_maintenance_trigger
            .load(std::sync::atomic::Ordering::Acquire));
        TerminalReporter::success("Maintenance flags are initially clean.");
    });

    // BLOCK 2: Detection Profile
    harness.execute("System Capability Audit", || {
        let caps = vantadb::hardware::HardwareCapabilities::global();
        TerminalReporter::sub_step("Reading system topology...");
        assert!(
            caps.total_memory > 0,
            "System memory must be greater than 0"
        );
        assert!(
            caps.logical_cores > 0,
            "Logical cores must be greater than 0"
        );
        assert!(
            caps.resource_score >= 1,
            "Resource score must be calculated"
        );

        println!("\n  {}", style("DETECTED PROFILE").bold().underlined());
        println!(
            "  {} Core Count:   {}",
            style("🧵").cyan(),
            caps.logical_cores
        );
        println!(
            "  {} Total Memory: {:.2} GB",
            style("🧠").magenta(),
            caps.total_memory as f64 / (1024.0 * 1024.0 * 1024.0)
        );
        println!(
            "  {} SIMD Support: {:?}",
            style("⚡").yellow(),
            caps.instructions
        );
        println!("  {} Profile Tier: {:?}", style("🏆").green(), caps.profile);

        TerminalReporter::success("System hardware profile correctly identified.");
    });

    // BLOCK 3: Fallback Math
    harness.execute("ALGORITHMIC FALLBACK", || {
        let a = vantadb::node::VectorRepresentations::Full(vec![1.0, 0.0, 0.0]);
        let b = vantadb::node::VectorRepresentations::Full(vec![1.0, 0.0, 0.0]);
        let c = vantadb::node::VectorRepresentations::Full(vec![0.0, 1.0, 0.0]);

        TerminalReporter::sub_step("Calculating Cosine Similarity (Fallbacks)...");
        // Test similarity computation regardless of the active instruction set branch.
        let sim_ab = a.cosine_similarity(&b).unwrap();
        assert!((sim_ab - 1.0).abs() < 1e-6);

        let sim_ac = a.cosine_similarity(&c).unwrap();
        assert!(sim_ac.abs() < 1e-6); // orthogonal = 0

        TerminalReporter::success("Mathematical consistency verified.");
    });
}
