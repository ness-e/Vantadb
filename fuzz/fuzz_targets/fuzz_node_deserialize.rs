#![no_main]
//! Fuzz target: VantaDB node and WAL deserialization.
//!
//! Tests all critical `bincode::deserialize` paths against arbitrary byte
//! sequences. A panic or unsafe memory access in any of these paths would
//! be a critical security / stability vulnerability since these paths are
//! hit whenever VantaDB reads data from persistent storage.
//!
//! Run on Linux with nightly toolchain:
//!   cargo +nightly fuzz run fuzz_node_deserialize -- -max_total_time=300

use libfuzzer_sys::fuzz_target;
use vantadb::{UnifiedNode, WalRecord};

fuzz_target!(|data: &[u8]| {
    // 1. Primary node deserialization — hit on every storage read
    let _: Result<UnifiedNode, _> = bincode::deserialize(data);

    // 2. WAL record deserialization — hit on every recovery and replication event
    let _: Result<WalRecord, _> = bincode::deserialize(data);
});
