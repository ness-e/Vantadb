// ponytail: blanket allow — unwraps carry documented invariants at each call site.
#![allow(clippy::expect_used, clippy::unwrap_used)]

//! VER-01 — WAL hash-chain tamper-evidence, CLI surface (`vanta-cli verify`).
//!
//! End-to-end at the handler level (the bin only forwards the exit code):
//! 1. Clean WAL → `cmd_verify` exit 0.
//! 2. Deleted record (extirpation) → exit ≠0 (chain link broken).
//! 3. Altered record with recomputed CRC (what CRC32C alone cannot catch) →
//!    exit ≠0 (record hash mismatch).
//! 4. Legacy (pre-chain) WAL → exit 0, explicitly reported as legacy.
//!
//! Run: `cargo nextest run --profile audit -p vantadb --test wal_chain_verify --build-jobs 2`
//!
//! Offset of the first tamper is asserted in the unit tests inside `src/wal.rs`
//! (where the report type is crate-visible); here we pin the CLI contract.

use std::path::{Path, PathBuf};

use vantadb::config::SyncMode;
use vantadb::node::{FieldValue, UnifiedNode};
use vantadb::wal::{WalHeader, WalRecord, WalWriter};

/// prev_hash (32B) + record_hash (32B) appended to every v3 frame.
const CHAIN_EXTRA: usize = 64;

/// Build `<dir>/data/vanta.wal` with `count` records (v3, chained).
fn seed_wal_into(dir: &Path, count: u128) -> PathBuf {
    let data = dir.join("data");
    std::fs::create_dir_all(&data).expect("mkdir data");
    let wal = data.join("vanta.wal");
    {
        let mut w = WalWriter::open(&wal, SyncMode::Periodic).expect("open WAL");
        for i in 1..=count {
            w.append(&WalRecord::Insert(UnifiedNode::new(i)))
                .expect("append");
        }
        w.sync().expect("sync");
    }
    wal
}

/// Build a fresh `<db>/data/vanta.wal` with `count` records (v3, chained).
fn seed_wal(count: u128) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let wal = seed_wal_into(dir.path(), count);
    (dir, wal)
}

/// (start, end) byte range of every frame, honoring the on-disk format version.
fn frame_bounds(bytes: &[u8]) -> Vec<(usize, usize)> {
    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    let extra = if version >= 3 { CHAIN_EXTRA } else { 0 };
    let mut frames = Vec::new();
    let mut off = WalHeader::SIZE;
    while off + 8 + extra <= bytes.len() {
        let len = u32::from_le_bytes(bytes[off..off + 4].try_into().unwrap()) as usize;
        let end = off + 4 + len + 4 + extra;
        if end > bytes.len() {
            break;
        }
        frames.push((off, end));
        off = end;
    }
    frames
}

fn verify(db_dir: &Path) -> i32 {
    vantadb::cli_handlers::cmd_verify(db_dir.to_str().unwrap(), true).expect("cmd_verify runs")
}

#[test]
fn verify_clean_wal_exits_zero() {
    let (dir, _wal) = seed_wal(5);
    assert_eq!(verify(dir.path()), 0, "clean chained WAL must verify green");
}

#[test]
fn verify_deleted_record_exits_nonzero() {
    let (dir, wal) = seed_wal(5);
    let bytes = std::fs::read(&wal).expect("read WAL");
    let frames = frame_bounds(&bytes);
    assert_eq!(frames.len(), 5, "seeded 5 frames");
    // Extirpate the 2nd record: the 3rd record's prev_hash now points at a
    // record that is no longer on disk → chain link broken.
    let mut tampered = Vec::with_capacity(bytes.len());
    tampered.extend_from_slice(&bytes[..frames[1].0]);
    tampered.extend_from_slice(&bytes[frames[2].0..]);
    std::fs::write(&wal, &tampered).expect("write tampered WAL");

    assert_eq!(
        verify(dir.path()),
        1,
        "removing a record must fail verification (exit 1)"
    );
}

#[test]
fn verify_altered_record_with_recomputed_crc_exits_nonzero() {
    let (dir, wal) = seed_wal(5);
    let mut bytes = std::fs::read(&wal).expect("read WAL");
    let frames = frame_bounds(&bytes);
    assert_eq!(frames.len(), 5, "seeded 5 frames");

    // Alter the 3rd record's payload AND recompute its CRC32C so the frame is
    // internally consistent (CRC alone would accept it). The record_hash still
    // commits to the original bytes → tamper detected.
    let (start, _end) = frames[2];
    let len = u32::from_le_bytes(bytes[start..start + 4].try_into().unwrap()) as usize;
    let payload_start = start + 4;
    bytes[payload_start] ^= 0xFF;
    let crc = vantadb::wal::compute_crc32c(&bytes[payload_start..payload_start + len]);
    let crc_start = payload_start + len;
    bytes[crc_start..crc_start + 4].copy_from_slice(&crc.to_le_bytes());
    std::fs::write(&wal, &bytes).expect("write tampered WAL");

    assert_eq!(
        verify(dir.path()),
        1,
        "a rewritten record with a valid CRC must still fail verification (exit 1)"
    );
}

#[test]
fn verify_legacy_v2_wal_exits_zero_with_legacy_report() {
    // Craft a pre-chain (v2) WAL by hand: header with format_version = 2 plus
    // legacy frames `[len][payload][crc]`. The verify command must read it
    // (range-compat), report it as legacy, and exit 0 — never treat the
    // absence of a chain as manipulation.
    let dir = tempfile::tempdir().expect("tempdir");
    let data = dir.path().join("data");
    std::fs::create_dir_all(&data).expect("mkdir data");
    let wal = data.join("vanta.wal");

    let mut bytes = WalHeader::new(2).serialize().to_vec();
    for i in 1..=3u128 {
        let payload = postcard::to_allocvec(&WalRecord::Insert(UnifiedNode::new(i))).unwrap();
        bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&payload);
        bytes.extend_from_slice(&vantadb::wal::compute_crc32c(&payload).to_le_bytes());
    }
    std::fs::write(&wal, &bytes).expect("write v2 WAL");

    assert_eq!(
        verify(dir.path()),
        0,
        "a legacy (pre-chain) WAL verifies as legacy, not as tampered"
    );
}

// ── EXE-01 demo producer (CI: `wal-verify-demo.yml`) ────────────────────────

/// Demo directory: `WAL_VERIFY_DEMO_DIR` or `<workspace>/target/wal-verify-demo`.
fn demo_dir() -> PathBuf {
    std::env::var("WAL_VERIFY_DEMO_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| Path::new(env!("CARGO_MANIFEST_DIR")).join("target/wal-verify-demo"))
}

/// Seeds the fixtures `scripts/demo-verify-e2e.ps1` drives through the REAL
/// `vanta-cli verify` binary (the tests above pin the handler; this pins the
/// shipped CLI + exit codes in CI):
///
/// - `<demo>/clean` — pristine 5-record chain (must verify, exit 0).
/// - `<demo>/tampered-removed` — record 2 extirpated; the successor's
///   `prev_hash` dangles (chain link broken).
/// - `<demo>/tampered-rewritten` — decodable content tamper ('AAAA'→'BAAA')
///   with the CRC recomputed: the frame replays cleanly, only the chain sees
///   it (what CRC32C alone cannot catch).
///
/// Writes `<demo>/handoff.json` for the script.
#[test]
#[ignore = "demo producer: run via scripts/demo-verify-e2e.ps1 (--run-ignored ignored-only)"]
fn wal_verify_demo_producer() {
    let demo = demo_dir();
    if demo.exists() {
        std::fs::remove_dir_all(&demo).expect("clean demo dir");
    }
    std::fs::create_dir_all(&demo).expect("demo dir");

    // 1. clean: pristine chain.
    let clean = demo.join("clean");
    seed_wal_into(&clean, 5);

    // 2. tampered-removed: extirpate record 2 (successor's prev_hash dangles).
    let removed = demo.join("tampered-removed");
    let removed_wal = seed_wal_into(&removed, 5);
    let bytes = std::fs::read(&removed_wal).expect("read WAL");
    let frames = frame_bounds(&bytes);
    assert_eq!(frames.len(), 5, "seeded 5 frames");
    let mut extirpated = Vec::with_capacity(bytes.len());
    extirpated.extend_from_slice(&bytes[..frames[1].0]);
    extirpated.extend_from_slice(&bytes[frames[2].0..]);
    std::fs::write(&removed_wal, &extirpated).expect("write tampered WAL");

    // 3. tampered-rewritten: record 2 carries a string field so the alteration
    // stays postcard-decodable; CRC recomputed so the frame is consistent.
    let rewritten = demo.join("tampered-rewritten");
    let data = rewritten.join("data");
    std::fs::create_dir_all(&data).expect("mkdir data");
    let rewritten_wal = data.join("vanta.wal");
    {
        let mut w = WalWriter::open(&rewritten_wal, SyncMode::Periodic).expect("open WAL");
        w.append(&WalRecord::Insert(UnifiedNode::new(1)))
            .expect("append");
        let mut node = UnifiedNode::new(2);
        node.set_field("k", FieldValue::String("AAAA".to_string()));
        w.append(&WalRecord::Insert(node)).expect("append");
        w.append(&WalRecord::Insert(UnifiedNode::new(3)))
            .expect("append");
        w.sync().expect("sync");
    }
    let mut bytes = std::fs::read(&rewritten_wal).expect("read WAL");
    let frames = frame_bounds(&bytes);
    assert_eq!(frames.len(), 3, "seeded 3 frames");
    let (start, _) = frames[1];
    let len = u32::from_le_bytes(bytes[start..start + 4].try_into().unwrap()) as usize;
    let payload_start = start + 4;
    let marker = bytes[payload_start..payload_start + len]
        .windows(4)
        .position(|w| w == b"AAAA")
        .expect("string field present in record 2 payload");
    bytes[payload_start + marker] = b'B';
    let crc = vantadb::wal::compute_crc32c(&bytes[payload_start..payload_start + len]);
    let crc_start = payload_start + len;
    bytes[crc_start..crc_start + 4].copy_from_slice(&crc.to_le_bytes());
    std::fs::write(&rewritten_wal, &bytes).expect("write tampered WAL");

    // 4. handoff for the script.
    let handoff = serde_json::json!({
        "generated_by": "wal_verify_demo_producer",
        "clean": clean.display().to_string(),
        "tampered_removed": removed.display().to_string(),
        "tampered_rewritten": rewritten.display().to_string(),
    });
    std::fs::write(
        demo.join("handoff.json"),
        serde_json::to_string_pretty(&handoff).expect("handoff json"),
    )
    .expect("write handoff");
    println!(
        "WAL verify demo fixtures OK — handoff: {} (next: vanta-cli verify, real binary)",
        demo.join("handoff.json").display()
    );
}
