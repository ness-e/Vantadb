//! VER-10: write receipts — certified write E2E (receipt + verification).
//!
//! Contract (plan Task 59): the declared write path (`put`) can emit a
//! verifiable receipt with a reference to the VER-01 chained WAL (equivalent
//! evidence: chain reference + content binding + live re-scan), verification
//! re-checks schema → sha256 integrity → live record, and an edited receipt or
//! altered content is detected. Opt-in: plain `put` pays nothing.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use vantadb::config::Config;
use vantadb::sdk::{Embedded, MemoryInput, MemoryMetadata};
use vantadb::Value;

fn open_temp_db(dir: &tempfile::TempDir) -> Embedded {
    let config = Config {
        storage_path: dir.path().to_string_lossy().to_string(),
        read_only: false,
        ..Default::default()
    };
    Embedded::open_with_config(config).expect("open embedded")
}

fn write_input(namespace: &str, key: &str) -> MemoryInput {
    let mut metadata = MemoryMetadata::new();
    metadata.insert("stage".to_string(), Value::Int(3));
    metadata.insert("color".to_string(), Value::String("blue".to_string()));
    MemoryInput {
        namespace: namespace.to_string(),
        key: key.to_string(),
        payload: "certified write payload".to_string(),
        metadata,
        vector: Some(vec![0.1, 0.2, 0.3]),
        ..Default::default()
    }
}

#[test]
fn put_certified_emits_verifiable_receipt() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = open_temp_db(&dir);

    let (record, receipt) = db
        .put_certified(write_input("write", "alpha"))
        .expect("put_certified");

    // ── Receipt shape (mirrors the VER-02 certificate contract) ──
    assert_eq!(receipt.schema_version, 1);
    assert_eq!(receipt.namespace, "write");
    assert_eq!(receipt.key, "alpha");
    assert_eq!(receipt.node_id, record.node_id.to_string());
    assert_eq!(receipt.version, record.version);
    assert_eq!(receipt.status, "recorded");
    assert_eq!(receipt.surfaces.len(), 2, "store + wal exactly once");
    for surface in &receipt.surfaces {
        assert_eq!(
            surface.residues, 0,
            "surface '{}' must report 0 residues: {surface:?}",
            surface.surface
        );
        assert!(
            !surface.evidence.is_empty(),
            "surface '{}' must carry evidence",
            surface.surface
        );
    }
    assert!(receipt
        .surfaces
        .iter()
        .any(|s| s.surface == "store" && s.action == "written"));
    assert!(receipt
        .surfaces
        .iter()
        .any(|s| s.surface == "wal" && s.action == "frame-recorded"));
    // VER-01 chain is referenced (same builder as the purge certificate).
    assert!(receipt.chain.chained);
    assert_eq!(receipt.chain.format_version, 3);
    assert!(receipt.chain.verify_command.contains("verify"));
    // Declared limits are never silent.
    assert!(!receipt.out_of_scope.is_empty());
    // Content binding + self-integrity.
    assert_eq!(receipt.content.algorithm, "sha256");
    assert_eq!(receipt.content.sha256.len(), 64, "hex SHA-256");
    assert!(receipt
        .content
        .sha256
        .chars()
        .all(|c| c.is_ascii_hexdigit()));
    assert_eq!(receipt.integrity.algorithm, "sha256");
    assert_eq!(receipt.integrity.sha256.len(), 64, "hex SHA-256");

    // ── The receipt is verifiable (JSON is the SDK surface) ──
    let json = serde_json::to_string(&receipt).expect("serialize receipt");
    let verification = db
        .verify_write_receipt(&json)
        .expect("receipt must verify against the live record");
    assert_eq!(verification.schema_version, 1);
    assert_eq!(verification.namespace, "write");
    assert_eq!(verification.key, "alpha");
    assert_eq!(verification.version, record.version);
    assert_eq!(verification.status, "recorded");
    assert_eq!(verification.integrity, "ok");
    assert_eq!(verification.content, "match");
    assert!(
        verification
            .rechecked_surfaces
            .contains(&"store".to_string()),
        "store must be part of the re-scan"
    );

    db.close().expect("close");
}

#[test]
fn write_receipt_verifies_after_flush_and_reopen() {
    let dir = tempfile::tempdir().expect("tempdir");

    let json = {
        let db = open_temp_db(&dir);
        let (_record, receipt) = db
            .put_certified(write_input("write", "beta"))
            .expect("put_certified");
        db.flush().expect("flush");
        db.close().expect("close");
        serde_json::to_string(&receipt).expect("serialize receipt")
    };

    // A fresh handle over the same on-disk state must verify the receipt:
    // the record was persisted before the issuing process exited.
    let db = open_temp_db(&dir);
    let verification = db
        .verify_write_receipt(&json)
        .expect("receipt must verify after reopen");
    assert_eq!(verification.content, "match");
    assert_eq!(verification.version, 1);
    db.close().expect("close");
}

#[test]
fn verify_rejects_tampered_receipt() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = open_temp_db(&dir);
    let (_record, receipt) = db
        .put_certified(write_input("write", "gamma"))
        .expect("put_certified");

    // Tamper: flip the attested version (the integrity hash must catch it).
    let mut tampered = serde_json::to_value(&receipt).expect("to value");
    tampered["version"] = serde_json::json!(99);
    let err = db
        .verify_write_receipt(&tampered.to_string())
        .expect_err("tampered receipt must NOT verify");
    assert!(
        err.to_string().contains("integrity"),
        "error must name the integrity failure, got: {err}"
    );

    // Tamper: change the attested node id.
    let mut tampered_id = serde_json::to_value(&receipt).expect("to value");
    tampered_id["node_id"] = serde_json::json!("1");
    assert!(db.verify_write_receipt(&tampered_id.to_string()).is_err());

    db.close().expect("close");
}

#[test]
fn verify_detects_content_change_after_receipt() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = open_temp_db(&dir);
    let (_record, receipt) = db
        .put_certified(write_input("write", "delta"))
        .expect("put_certified");
    let json = serde_json::to_string(&receipt).expect("serialize");

    // The record is re-written with a different payload after emission →
    // the content binding (payload + version) no longer matches.
    let mut altered = write_input("write", "delta");
    altered.payload = "tampered payload".to_string();
    db.put(altered).expect("re-put with altered payload");

    let err = db
        .verify_write_receipt(&json)
        .expect_err("stale receipt must NOT verify once content changes");
    assert!(
        err.to_string().contains("content"),
        "error must name the content mismatch, got: {err}"
    );

    db.close().expect("close");
}

#[test]
fn verify_fails_when_record_deleted_after_receipt() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = open_temp_db(&dir);
    let (_record, receipt) = db
        .put_certified(write_input("write", "epsilon"))
        .expect("put_certified");
    let json = serde_json::to_string(&receipt).expect("serialize");

    db.delete("write", "epsilon").expect("delete");

    let err = db
        .verify_write_receipt(&json)
        .expect_err("receipt must NOT verify once the record is gone");
    assert!(
        err.to_string().contains("present"),
        "error must report the absent record, got: {err}"
    );

    db.close().expect("close");
}

/// H-1-style evasion (mirrors the VER-02 claimless test): a receipt with the
/// surface claims dropped and the integrity hash **recomputed** must still be
/// rejected — verification is not claim-driven.
#[test]
fn verify_rejects_claimless_receipt_even_with_recomputed_hash() {
    use sha2::{Digest, Sha256};

    let dir = tempfile::tempdir().expect("tempdir");
    let db = open_temp_db(&dir);
    let (_record, receipt) = db
        .put_certified(write_input("write", "zeta"))
        .expect("put_certified");

    // Craft the evasion: drop every surface claim and recompute the integrity
    // hash exactly like the producer would (canonical body, empty hash field).
    let mut claimless = receipt.clone();
    claimless.surfaces.clear();
    claimless.integrity.sha256.clear();
    let canonical = serde_json::to_vec(&claimless).expect("canonical bytes");
    let digest = Sha256::digest(&canonical);
    claimless.integrity.sha256 = digest.iter().map(|b| format!("{b:02x}")).collect();

    let err = db
        .verify_write_receipt(&serde_json::to_string(&claimless).expect("serialize"))
        .expect_err("claimless receipt must NOT verify, even with a valid hash");
    assert!(
        err.to_string().contains("surface"),
        "rejection must name the surface inventory, got: {err}"
    );

    // The pristine receipt still verifies (verification must not mutate).
    db.verify_write_receipt(&serde_json::to_string(&receipt).expect("serialize"))
        .expect("pristine receipt must still verify");

    db.close().expect("close");
}

#[test]
fn write_receipt_is_deterministic_across_equivalent_databases() {
    fn receipt_body() -> serde_json::Value {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = open_temp_db(&dir);
        let (_record, receipt) = db
            .put_certified(write_input("write", "eta"))
            .expect("put_certified");
        db.close().expect("close");
        let mut value = serde_json::to_value(&receipt).expect("to value");
        // `timestamp` varies by construction, and `integrity.sha256` is
        // derived from the timestamped body — every *claim* must otherwise
        // be byte-identical.
        let object = value.as_object_mut().expect("receipt object");
        object.remove("timestamp");
        object.remove("integrity");
        value
    }

    assert_eq!(
        receipt_body(),
        receipt_body(),
        "same state + same inputs → same receipt claims"
    );
}

#[test]
fn put_certified_rejects_non_finite_floats_before_writing() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = open_temp_db(&dir);

    let mut input = write_input("write", "nan");
    input.vector = Some(vec![0.5, f32::NAN]);
    let err = db
        .put_certified(input)
        .expect_err("non-finite floats must be rejected");
    assert!(
        err.to_string().contains("non-finite"),
        "error must name the non-finite float, got: {err}"
    );
    // The rejected certified write must not have been persisted.
    assert!(db.get("write", "nan").expect("get").is_none());

    // A non-finite metadata value is rejected the same way.
    let mut input = write_input("write", "nan-meta");
    input
        .metadata
        .insert("score".to_string(), Value::Float(f64::INFINITY));
    assert!(db.put_certified(input).is_err());
    assert!(db.get("write", "nan-meta").expect("get").is_none());

    db.close().expect("close");
}

#[test]
fn verify_fails_safe_when_live_record_has_non_finite_floats() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = open_temp_db(&dir);
    let (_record, receipt) = db
        .put_certified(write_input("write", "theta"))
        .expect("put_certified");
    let json = serde_json::to_string(&receipt).expect("serialize");

    // The record is re-written with a non-finite float: JSON would collapse
    // it to `null`, making the content comparison ambiguous — verification
    // must fail safe instead of comparing ambiguous hashes.
    let mut altered = write_input("write", "theta");
    altered.vector = Some(vec![0.5, f32::INFINITY]);
    db.put(altered).expect("re-put with non-finite float");

    let err = db
        .verify_write_receipt(&json)
        .expect_err("non-finite live content must NOT verify");
    assert!(
        err.to_string().contains("non-finite"),
        "error must name the non-finite float, got: {err}"
    );

    db.close().expect("close");
}

#[test]
fn write_receipt_binds_the_sparse_vector() {
    use std::collections::BTreeMap;

    let dir = tempfile::tempdir().expect("tempdir");
    let db = open_temp_db(&dir);

    let mut input = write_input("write", "lambda");
    input.sparse_vector = Some(vantadb::SparseVector(BTreeMap::from([(1, 0.5), (7, 0.25)])));
    let (_record, receipt) = db.put_certified(input).expect("put_certified");

    let json = serde_json::to_string(&receipt).expect("serialize");
    let verification = db
        .verify_write_receipt(&json)
        .expect("sparse vector must round-trip inside the binding");
    assert_eq!(verification.content, "match");

    // Altering the sparse coefficient is a content change.
    let mut altered = write_input("write", "lambda");
    altered.sparse_vector = Some(vantadb::SparseVector(BTreeMap::from([(1, 0.5), (7, 0.9)])));
    db.put(altered).expect("re-put with altered sparse vector");
    assert!(db.verify_write_receipt(&json).is_err());

    db.close().expect("close");
}

#[test]
fn verify_fails_after_the_record_ttl_expires() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = open_temp_db(&dir);

    let mut input = write_input("write", "kappa");
    input.ttl_ms = Some(1);
    let (_record, receipt) = db.put_certified(input).expect("put_certified");
    let json = serde_json::to_string(&receipt).expect("serialize");

    // Lazy TTL: a passed deadline reads as absent.
    std::thread::sleep(std::time::Duration::from_millis(50));

    let err = db
        .verify_write_receipt(&json)
        .expect_err("an expired record reads as absent");
    assert!(
        err.to_string().contains("not readable"),
        "error must report the expired record as not readable, got: {err}"
    );

    db.close().expect("close");
}
