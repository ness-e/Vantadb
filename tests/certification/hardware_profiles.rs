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

    // BLOCK 4: RSS Stability Under Load (ST2.2.2)
    harness.execute("RSS Stability Under Load", || {
        let storage = temp_storage();

        let (warmup_count, additional_count) = if cfg!(debug_assertions) {
            (2_000, 8_000)
        } else {
            (20_000, 80_000)
        };

        TerminalReporter::sub_step(&format!("Warming up database with {} vectors...", warmup_count));
        for i in 0..warmup_count {
            let mut node = vantadb::node::UnifiedNode::new(i as u64);
            node.vector = vantadb::node::VectorRepresentations::Full(vec![1.0; 128]);
            node.tier = vantadb::node::NodeTier::Cold;
            storage.insert(&node).unwrap();
        }
        storage.flush().unwrap();

        let stats_warm = storage.get_memory_stats();
        vantadb::metrics::record_memory_breakdown(
            stats_warm.node_count,
            stats_warm.logical_bytes,
            stats_warm.physical_rss,
            stats_warm.cache_entries as u64,
            0,
        );
        let rss_warm = vantadb::metrics::memory_breakdown_snapshot().process_rss_bytes;

        TerminalReporter::sub_step(&format!("Inserting {} additional vectors...", additional_count));
        for i in warmup_count..(warmup_count + additional_count) {
            let mut node = vantadb::node::UnifiedNode::new(i as u64);
            node.vector = vantadb::node::VectorRepresentations::Full(vec![1.0; 128]);
            node.tier = vantadb::node::NodeTier::Cold;
            storage.insert(&node).unwrap();
        }
        storage.flush().unwrap();

        let stats_final = storage.get_memory_stats();
        vantadb::metrics::record_memory_breakdown(
            stats_final.node_count,
            stats_final.logical_bytes,
            stats_final.physical_rss,
            stats_final.cache_entries as u64,
            0,
        );
        let rss_final = vantadb::metrics::memory_breakdown_snapshot().process_rss_bytes;

        println!("\n  {}", style("RSS STABILITY STATISTICS").bold().underlined());
        println!("  RSS Warmup:  {:.2} MB", rss_warm as f64 / (1024.0 * 1024.0));
        println!("  RSS Final:   {:.2} MB", rss_final as f64 / (1024.0 * 1024.0));

        if rss_warm > 0 {
            let drift = rss_final as f64 / rss_warm as f64;
            println!("  Memory Drift Ratio: {:.2}%", (drift - 1.0) * 100.0);
            assert!(drift <= 4.0, "Memory drift ratio too high: {:.2}", drift);
        }

        TerminalReporter::success("RSS stability and drift under control.");
    });
}
