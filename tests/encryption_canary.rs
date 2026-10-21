// ponytail: blanket allow — this is a test; the invariants are documented per call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! The canary probe of the at-rest encryption (ADR-0057 W6).
//!
//! With the `encryption` feature compiled and a master key configured, a
//! **known payload** must not appear in the clear in **any** on-disk file:
//! the WAL, the VantaFile segments, the HNSW index image, the backend's
//! journal and SSTs, and the snapshots.
//!
//! This is the same criterion the `FIND-249` probe used, turned into an
//! executable test (ADR-0057 §8).

#![cfg(all(feature = "encryption", any(feature = "fjall", feature = "rocksdb")))]

use std::path::{Path, PathBuf};

use vantadb::{BackendKind, Config, Embedded, MemoryInput};

/// A payload unique to this test: it must never reach the disk in cleartext.
///
/// It has **several words** on purpose: the text index's **tokens** (and the
/// backend's keys) stay in cleartext by design — the index needs prefix
/// scans per namespace (ADR-0057 §5.3, limit 1; `FIND-513`). The probe
/// therefore looks for the **whole payload**, which only appears in the clear
/// if a *value* leaks.
const CANARY: &str = "vantadb w6 canary 6f1d3a9c7e2b4d8a";

/// The hex master key of the test.
fn key_hex(seed: u8) -> String {
    format!("{seed:02x}").repeat(32)
}

/// Every file under `dir` (recursively).
fn files_under(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(files_under(&path));
        } else {
            out.push(path);
        }
    }
    out
}

/// Assert that the canary is nowhere in the clear under `dir`.
fn assert_no_plaintext_canary(dir: &Path, label: &str) {
    let files = files_under(dir);
    assert!(
        !files.is_empty(),
        "{label}: the probe found no file to inspect under {}",
        dir.display()
    );
    for file in &files {
        let name = file.file_name().map(|n| n.to_string_lossy().to_string());
        // The lock file holds no payload.
        if name.as_deref() == Some(".vanta.lock") {
            continue;
        }
        let bytes = std::fs::read(file).expect("read");
        assert!(
            !bytes.windows(CANARY.len()).any(|w| w == CANARY.as_bytes()),
            "{label}: the payload must not appear in cleartext in {}",
            file.display()
        );
    }
}

/// The probe: an encrypted database must not leak its payload in the clear.
#[test]
fn encrypted_database_has_no_plaintext_canary_on_disk() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().to_str().expect("path");
    let config = Config {
        storage_path: path.to_string(),
        backend_kind: BackendKind::Fjall,
        encryption_key: Some(key_hex(0x5a)),
        ..Config::default()
    };
    let db = Embedded::open_with_config(config).expect("open an encrypted database");

    let mut input = MemoryInput::new("w6", "canary", CANARY);
    // The **metadata** is not used for the canary: the text index stores a
    // string field's value verbatim in its key (a declared limit of the text
    // index, `FIND-513`), while the payload is tokenized.
    input.metadata.insert(
        "marker".to_string(),
        vantadb::Value::String("marker".to_string()),
    );
    db.put(input).expect("put the canary");
    db.flush().expect("flush");
    db.create_snapshot("probe").expect("snapshot");
    drop(db);

    // 1. The live database: the WAL, the segments, the index image and the
    //    backend's files.
    assert_no_plaintext_canary(dir.path(), "live database");

    // 2. The snapshot: a mirror of the same artefacts.
    let snapshot = dir.path().join("data").join("snapshots").join("probe");
    assert!(
        snapshot.exists(),
        "the snapshot must exist: {}",
        snapshot.display()
    );
    assert_no_plaintext_canary(&snapshot, "snapshot");

    // 3. The data is really there (the probe must not pass on an empty
    //    database).
    let config = Config {
        storage_path: path.to_string(),
        backend_kind: BackendKind::Fjall,
        encryption_key: Some(key_hex(0x5a)),
        ..Config::default()
    };
    let db = Embedded::open_with_config(config).expect("reopen the encrypted database");
    let record = db
        .get("w6", "canary")
        .expect("get")
        .expect("the record is there");
    assert_eq!(record.payload, CANARY, "the payload must read back intact");
}
