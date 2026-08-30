//! 🔁 Property-based tests for `WalRecord` roundtrip (GH-127).
//!
//! Covers:
//! 1. Pure in-memory byte roundtrip over all 8 `WalRecord` variants (≥1000 cases).
//! 2. Payload-size buckets: ~0 B, ~1 B, ~64 KB, ~1 MB — same byte roundtrip.
//! 3. File-backed roundtrip: `WalWriter::batch_append` → reopen → recover the full
//!    multiset of serialized bytes.
//! 4. Concurrent writes through `Arc<Mutex<WalWriter>>` (deterministic counts).
//!
//! `WalRecord` derives no `PartialEq`, so assertions compare postcard bytes
//! (`postcard::to_allocvec` is deterministic).
//!
//! Run: `cargo nextest run --profile audit -p vantadb --test proptest_wal_roundtrip --build-jobs 2`

use proptest::prelude::*;
use std::sync::{Arc, Mutex};
use vantadb::config::SyncMode;
use vantadb::node::{FieldValue, UnifiedNode};
use vantadb::wal::{WalReader, WalRecord, WalWriter};

// ── Strategy helpers ─────────────────────────────────────────────────────

fn arb_u128() -> impl Strategy<Value = u128> {
    (any::<u64>(), any::<u64>()).prop_map(|(hi, lo)| (hi as u128) << 64 | lo as u128)
}

/// A small `UnifiedNode` with an optional vector and a few relational fields.
fn arb_unified_node() -> impl Strategy<Value = UnifiedNode> {
    (
        arb_u128(),
        prop::collection::vec(-1.0f32..1.0, 0..16),
        prop::collection::btree_map("[a-zA-Z_][a-zA-Z0-9_]{0,10}", any::<String>(), 0..4),
    )
        .prop_map(|(id, vector, relational)| {
            let mut node = UnifiedNode::with_vector(id, vector);
            for (k, v) in relational {
                node.set_field(k, FieldValue::String(v));
            }
            node
        })
}

/// Arbitrary valid `WalRecord` covering all 7 variants.
fn arb_wal_record() -> impl Strategy<Value = WalRecord> {
    prop_oneof![
        arb_unified_node().prop_map(WalRecord::Insert),
        (arb_u128(), arb_unified_node()).prop_map(|(id, node)| WalRecord::Update { id, node }),
        arb_u128().prop_map(|id| WalRecord::Delete { id }),
        (
            any::<u64>(),
            prop::option::of(any::<u32>()),
            prop::option::of(any::<u64>())
        )
            .prop_map(
                |(node_count, index_checksum, timestamp)| WalRecord::Checkpoint {
                    node_count,
                    index_checksum,
                    timestamp,
                }
            ),
        (0u64..1000).prop_map(WalRecord::Begin),
        // WAL v2 (RES-01 / ACID Phase 4a): two-phase prepare marker.
        ((0u64..1000), any::<u32>())
            .prop_map(|(txn_id, op_count)| WalRecord::Prepare { txn_id, op_count })
            .boxed(),
        (0u64..1000).prop_map(WalRecord::Commit),
        (0u64..1000).prop_map(WalRecord::Abort),
    ]
}

/// Payload size class — maps to the ~0 B / ~1 B / 64 KB / ~1 MB buckets.
#[derive(Clone, Copy, Debug)]
enum PayloadBucket {
    /// Begin/Commit/Abort control records (variant tag + small varint).
    Tiny,
    /// Delete/Checkpoint — a handful of bytes.
    Small,
    /// Insert with a 64 KB `ext_metadata` blob.
    Medium,
    /// Insert with a ~1 MB `ext_metadata` blob (under the 10 MB reader guard).
    Large,
}

fn arb_tiny_record() -> impl Strategy<Value = WalRecord> {
    prop_oneof![
        (0u64..1000).prop_map(WalRecord::Begin),
        ((0u64..1000), any::<u32>())
            .prop_map(|(txn_id, op_count)| WalRecord::Prepare { txn_id, op_count })
            .boxed(),
        (0u64..1000).prop_map(WalRecord::Commit),
        (0u64..1000).prop_map(WalRecord::Abort),
    ]
}

fn arb_small_record() -> impl Strategy<Value = WalRecord> {
    prop_oneof![
        (0u64..1000).prop_map(|id| WalRecord::Delete { id: id.into() }),
        (
            0u64..1000,
            prop::option::of(0u32..1000),
            prop::option::of(0u64..1000)
        )
            .prop_map(
                |(node_count, index_checksum, timestamp)| WalRecord::Checkpoint {
                    node_count,
                    index_checksum,
                    timestamp,
                }
            ),
    ]
}

/// Insert carrying a payload of `size` bytes inside `ext_metadata`.
fn arb_blob_record(size: usize) -> impl Strategy<Value = WalRecord> {
    arb_u128().prop_map(move |id| {
        let mut node = UnifiedNode::new(id);
        node.ext_metadata.insert("blob".into(), vec![0u8; size]);
        WalRecord::Insert(node)
    })
}

fn arb_payload_record() -> impl Strategy<Value = (PayloadBucket, WalRecord)> {
    prop_oneof![
        2 => (Just(PayloadBucket::Tiny), arb_tiny_record()),
        2 => (Just(PayloadBucket::Small), arb_small_record()),
        1 => (Just(PayloadBucket::Medium), arb_blob_record(64 * 1024)),
        1 => (Just(PayloadBucket::Large), arb_blob_record(1024 * 1024)),
    ]
}

// ── 1. Pure in-memory byte roundtrip (all 7 variants, ≥1000 cases) ────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn test_pure_roundtrip_bytes(record in arb_wal_record()) {
        let bytes = postcard::to_allocvec(&record).unwrap();
        let decoded: WalRecord = postcard::from_bytes(&bytes).unwrap();
        let re_encoded = postcard::to_allocvec(&decoded).unwrap();
        prop_assert_eq!(bytes, re_encoded);
    }
}

// ── 2. Payload-size buckets ──────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn test_payload_size_roundtrip((bucket, record) in arb_payload_record()) {
        let bytes = postcard::to_allocvec(&record).unwrap();
        let decoded: WalRecord = postcard::from_bytes(&bytes).unwrap();
        let re_encoded = postcard::to_allocvec(&decoded).unwrap();
        prop_assert_eq!(bytes.as_slice(), re_encoded.as_slice());

        match bucket {
            PayloadBucket::Tiny => {
                prop_assert!(bytes.len() <= 16, "tiny payload too big: {}", bytes.len());
            }
            PayloadBucket::Small => {
                prop_assert!(bytes.len() < 64, "small payload too big: {}", bytes.len());
            }
            PayloadBucket::Medium => {
                prop_assert!(
                    (60_000..=70_000).contains(&bytes.len()),
                    "64 KB payload out of range: {}",
                    bytes.len()
                );
            }
            PayloadBucket::Large => {
                prop_assert!(
                    (900_000..=1_100_000).contains(&bytes.len()),
                    "~1 MB payload out of range: {}",
                    bytes.len()
                );
            }
        }
    }
}

// ── 3. File-backed roundtrip: batch_append → reopen → full multiset ──────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn test_file_roundtrip_batch(records in prop::collection::vec(arb_wal_record(), 1..30)) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("roundtrip.wal");

        // Serialize the original records once — they carry clock-derived fields
        // (e.g. `last_accessed`), so the expected blobs must come from these
        // exact objects, never from reconstructed nodes.
        let mut remaining: Vec<Vec<u8>> = records
            .iter()
            .map(|r| postcard::to_allocvec(r).unwrap())
            .collect();

        {
            let mut writer = WalWriter::open(&path, SyncMode::Periodic).unwrap();
            writer.batch_append(&records).unwrap();
        }

        let mut reader = WalReader::open(&path).unwrap();
        while let Some(record) = reader.next_record().unwrap() {
            let blob = postcard::to_allocvec(&record).unwrap();
            let pos = remaining.iter().position(|b| *b == blob);
            prop_assert!(
                pos.is_some(),
                "record read from WAL was not in the original batch"
            );
            remaining.remove(pos.unwrap());
        }

        prop_assert!(
            remaining.is_empty(),
            "{} records from the original batch were not recovered",
            remaining.len()
        );
    }
}

// ── 4. Concurrent writes (deterministic, not proptest) ───────────────────

#[test]
fn test_concurrent_wal_writes() {
    const THREADS: usize = 4;
    const PER_THREAD: usize = 25;

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("concurrent.wal");

    let writer = Arc::new(Mutex::new(
        WalWriter::open(&path, SyncMode::Periodic).unwrap(),
    ));

    let mut handles = Vec::new();
    for t in 0..THREADS {
        let writer = Arc::clone(&writer);
        handles.push(std::thread::spawn(move || {
            // Build the records first and return them so the expected multiset
            // is derived from the exact objects that were written.
            let records: Vec<WalRecord> = (0..PER_THREAD)
                .map(|i| WalRecord::Insert(UnifiedNode::new((t * PER_THREAD + i) as u128)))
                .collect();
            for record in &records {
                writer.lock().unwrap().append(record).unwrap();
            }
            records
        }));
    }

    let mut expected: Vec<Vec<u8>> = Vec::new();
    for handle in handles {
        for record in handle.join().unwrap() {
            expected.push(postcard::to_allocvec(&record).unwrap());
        }
    }
    drop(writer); // close the writer before reopening for reads

    let mut reader = WalReader::open(&path).unwrap();
    let mut found = Vec::new();
    while let Some(record) = reader.next_record().unwrap() {
        found.push(postcard::to_allocvec(&record).unwrap());
    }

    assert_eq!(
        found.len(),
        THREADS * PER_THREAD,
        "recovered record count must match written count"
    );

    let mut remaining = expected;
    for blob in found {
        let pos = remaining
            .iter()
            .position(|b| *b == blob)
            .expect("record from concurrent WAL was not written by any thread");
        remaining.remove(pos);
    }
    assert!(
        remaining.is_empty(),
        "{} written records were not recovered",
        remaining.len()
    );
}
