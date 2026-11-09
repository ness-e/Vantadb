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

/// The vector canary: sixteen hand-picked high-entropy floats.
///
/// The probe L2-normalizes them **before** inserting, so any ingest- or
/// HNSW-side normalization is idempotent on the stored bits. The signature
/// searched on disk is the little-endian IEEE-754 image of the whole vector
/// (64 contiguous bytes ≥ the required 32), which only appears in the clear
/// if the raw floats leak.
const CANARY_VECTOR: [f32; 16] = [
    0.133_713_37,
    -0.424_242_4,
    0.271_828_2,
    -0.314_159_27,
    0.732_050_8,
    -0.577_215_7,
    0.918_273_7,
    -0.102_938_47,
    0.564_738_3,
    -0.817_263_54,
    0.293_847_56,
    -0.675_849_3,
    0.384_756_12,
    -0.908_172_6,
    0.458_493_01,
    -0.726_385,
];

/// The hex master key of the test.
fn key_hex(seed: u8) -> String {
    format!("{seed:02x}").repeat(32)
}

/// A unit-norm copy of `v` (the input is never zero — asserted).
fn l2_normalized(v: &[f32]) -> Vec<f32> {
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!(norm > f32::EPSILON, "the canary vector must be non-zero");
    v.iter().map(|x| x / norm).collect()
}

/// The on-disk signature of a stored vector: the little-endian IEEE-754
/// bytes of its floats, contiguous — 4·len bytes, 64 for the canary.
fn vector_signature(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|x| x.to_le_bytes()).collect()
}

/// True when `haystack` contains `needle` as a contiguous window.
fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty() && haystack.windows(needle.len()).any(|w| w == needle)
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

/// Assert that `needle` is nowhere in the clear under `dir`.
fn assert_no_plaintext_bytes(dir: &Path, needle: &[u8], label: &str) {
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
            !contains_bytes(&bytes, needle),
            "{label}: the signature must not appear in cleartext in {}",
            file.display()
        );
    }
}

/// Assert that the text canary is nowhere in the clear under `dir`.
fn assert_no_plaintext_canary(dir: &Path, label: &str) {
    assert_no_plaintext_bytes(dir, CANARY.as_bytes(), label);
}

/// Open (or reopen) the database at `path` with `encryption_key`.
fn open_db(path: &str, encryption_key: Option<String>) -> Embedded {
    let config = Config {
        storage_path: path.to_string(),
        backend_kind: BackendKind::Fjall,
        encryption_key,
        ..Config::default()
    };
    Embedded::open_with_config(config).expect("open database")
}

/// Put the vector canary and return the **stored** vector (what actually
/// reached the disk — the signature is derived from it, so the probe is
/// robust to any ingest-side bit change).
fn put_vector_canary(db: &Embedded) -> Vec<f32> {
    let mut input = MemoryInput::new("w6", "vcanary", "vector canary probe");
    input.vector = Some(l2_normalized(&CANARY_VECTOR));
    db.put(input).expect("put the vector canary");
    db.flush().expect("flush");
    db.get("w6", "vcanary")
        .expect("get")
        .expect("the record is there")
        .vector
        .expect("the vector is stored")
}

/// The `vstore_L*.vanta` segments and the `vector_index.bin` image under
/// `dir`, as the two vector-bearing artefact classes.
fn vector_artefacts(dir: &Path) -> (Vec<PathBuf>, Option<PathBuf>) {
    let files = files_under(dir);
    let mut segments = Vec::new();
    let mut index = None;
    for file in files {
        let name = file.file_name().map(|n| n.to_string_lossy().to_string());
        match name.as_deref() {
            Some(n) if n.starts_with("vstore_L") && n.ends_with(".vanta") => {
                segments.push(file);
            }
            Some("vector_index.bin") => index = Some(file),
            _ => {}
        }
    }
    (segments, index)
}

/// The probe: an encrypted database must not leak its payload in the clear.
#[test]
fn encrypted_database_has_no_plaintext_canary_on_disk() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().to_str().expect("path");
    let db = open_db(path, Some(key_hex(0x5a)));

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
    let db = open_db(path, Some(key_hex(0x5a)));
    let record = db
        .get("w6", "canary")
        .expect("get")
        .expect("the record is there");
    assert_eq!(record.payload, CANARY, "the payload must read back intact");
}

/// Positive control (FIND-592): **without** a key, the vector's IEEE-754
/// signature MUST appear in the clear in each class of vector-bearing
/// artefact — the `vstore_L*.vanta` segments and the `vector_index.bin`
/// HNSW image. If this fails, the negative control below is a false green.
#[test]
fn plaintext_vector_signature_is_visible_in_vstore_and_index() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().to_str().expect("path");
    let db = open_db(path, None);

    let stored = put_vector_canary(&db);
    let signature = vector_signature(&stored);
    assert!(
        signature.len() >= 32,
        "the signature must be >= 8 floats (32 bytes), got {}",
        signature.len()
    );
    drop(db);

    let (segments, index) = vector_artefacts(dir.path());

    // The segments exist and at least one holds the vector in the clear.
    assert!(
        !segments.is_empty(),
        "the probe found no vstore_L*.vanta segment under {}",
        dir.path().display()
    );
    let leaked: Vec<&PathBuf> = segments
        .iter()
        .filter(|f| contains_bytes(&std::fs::read(f).expect("read"), &signature))
        .collect();
    assert!(
        !leaked.is_empty(),
        "the vector signature must appear in cleartext in some \
         vstore_L*.vanta segment (checked: {:?})",
        segments
    );

    // The HNSW image exists (flush materializes it) and stores the vector.
    let index = index.expect(
        "vector_index.bin must exist after flush — without it the probe \
         would be vacuous",
    );
    let image = std::fs::read(&index).expect("read vector_index.bin");
    assert!(
        contains_bytes(&image, &signature),
        "the vector signature must appear in cleartext in {}",
        index.display()
    );
}

/// Negative control (FIND-592): with a key, the vector's signature appears
/// in **no** file of the live database or its snapshot, and the vector
/// reads back bit-identical after reopening with the key.
#[test]
fn encrypted_database_has_no_plaintext_vector_canary_on_disk() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().to_str().expect("path");
    let db = open_db(path, Some(key_hex(0x7e)));

    let stored = put_vector_canary(&db);
    let signature = vector_signature(&stored);
    db.create_snapshot("vprobe").expect("snapshot");
    drop(db);

    // 1. The live database — covers the WAL, the vstore_L*.vanta segments,
    //    vector_index.bin and the backend's files.
    assert_no_plaintext_bytes(dir.path(), &signature, "live database");

    // 2. The snapshot mirrors the same artefacts; it must not leak either.
    let snapshot = dir.path().join("data").join("snapshots").join("vprobe");
    assert!(
        snapshot.exists(),
        "the snapshot must exist: {}",
        snapshot.display()
    );
    assert_no_plaintext_bytes(&snapshot, &signature, "snapshot");

    // 3. Bit-exact read-back: the stored vector round-trips through the
    //    encrypted artefacts unchanged (no ingest normalization applies to
    //    an already unit-norm vector — verified by the positive control,
    //    which finds these exact bits on disk).
    let db = open_db(path, Some(key_hex(0x7e)));
    let read_back = db
        .get("w6", "vcanary")
        .expect("get")
        .expect("the record is there")
        .vector
        .expect("the vector is stored");
    assert_eq!(
        vector_signature(&read_back),
        signature,
        "the vector must read back bit-identical"
    );
}
