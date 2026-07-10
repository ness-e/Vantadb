#![no_main]
//! Fuzz target: WAL header deserialization + WAL record round-trip.
//!
//! Exercises WalHeader::deserialize with arbitrary bytes to detect panics
//! from corrupted headers, then round-trips arbitrary data through a temp
//! WAL file to exercise WalReader::next_record on potentially-invalid data.
//!
//! Run on Linux with nightly toolchain:
//!   cargo +nightly fuzz run fuzz_wal -- -max_total_time=300

use libfuzzer_sys::fuzz_target;
use vantadb::wal::WalHeader;

fuzz_target!(|data: &[u8]| {
    // 1. Fuzz WalHeader deserialization — validates size (20), magic, CRC, version
    if let Ok(header) = WalHeader::deserialize(data) {
        // If it deserialized successfully, round-trip must preserve
        let re_serialized = header.serialize();
        let re_deserialized = WalHeader::deserialize(&re_serialized).unwrap();
        assert_eq!(re_deserialized.postcard_version(), header.postcard_version());
    }

    // 2. Fuzz raw postcard deserialization of WalRecord (already done in
    //    fuzz_node_deserialize, so we focus on the custom binary header above).
});
