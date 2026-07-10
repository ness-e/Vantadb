#![no_main]
//! Fuzz target: HNSW index deserialization.
//!
//! Exercises CPIndex::deserialize_from_bytes with arbitrary byte sequences
//! to detect panics, infinite loops, or memory safety violations in the
//! active (non-rkyv) archive format.
//!
//! The deserializer performs internal bounds checks and version validation
//! before constructing the CPIndex from raw bytes, making it suitable as
//! a first line of defence against storage corruption bugs.
//!
//! Run on Linux with nightly toolchain:
//!   cargo +nightly fuzz run fuzz_archive -- -max_total_time=300

use libfuzzer_sys::fuzz_target;
use vantadb::index::CPIndex;

fuzz_target!(|data: &[u8]| {
    let _ = CPIndex::deserialize_from_bytes(data, false);
});
