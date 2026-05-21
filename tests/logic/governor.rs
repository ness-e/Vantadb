//! Resource Governor & OOM Protection Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use std::sync::atomic::Ordering;
use vantadb::governor::{ResourceGovernor, ALLOCATED_BYTES};

#[test]
fn engine_governor_certification() {
    let mut harness = VantaHarness::new("LOGIC LAYER (RESOURCE GOVERNOR)");

    harness.execute("OOM: Strategic Allocation & Safeguards", || {
        let governor = ResourceGovernor::new(1024 * 1024, 1000); // 1MB limit

        TerminalReporter::sub_step("Requesting 512KB (valid allocation)...");
        assert!(governor.request_allocation(512 * 1024).is_ok());
        assert_eq!(ALLOCATED_BYTES.load(Ordering::SeqCst), 512 * 1024);

        TerminalReporter::sub_step("Requesting 600KB (total 1.1MB, exceeding 1MB limit)...");
        let result = governor.request_allocation(600 * 1024);
        assert!(result.is_err(), "Governor failed to block OOM condition");

        TerminalReporter::sub_step("Releasing memory and verifying neutrality...");
        governor.free_allocation(512 * 1024);
        assert_eq!(ALLOCATED_BYTES.load(Ordering::SeqCst), 0);

        TerminalReporter::success("OOM protection and state-tracking verified.");
    });
}
