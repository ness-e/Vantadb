//! VER-02: certified delete E2E — delete-path purge + purge certificate.
//!
//! Contract (plan Task 38): delete → 0 residues across store/indexes/shred +
//! a valid certificate (integrity hash + VER-01 chain reference), emitted by
//! the SDK (CLI/MCP are thin wrappers over the same core logic).
#![allow(clippy::expect_used, clippy::unwrap_used)]

use vantadb::config::Config;
use vantadb::sdk::{Embedded, MemoryInput, MemoryMetadata};
use vantadb::{Value, WalReader, WalRecord};

fn open_temp_db(dir: &tempfile::TempDir) -> Embedded {
    let config = Config {
        storage_path: dir.path().to_string_lossy().to_string(),
        read_only: false,
        ..Default::default()
    };
    Embedded::open_with_config(config).expect("open embedded")
}

fn certified_input(namespace: &str, key: &str) -> MemoryInput {
    let mut metadata = MemoryMetadata::new();
    metadata.insert("stage".to_string(), Value::Int(3));
    metadata.insert("color".to_string(), Value::String("blue".to_string()));
    MemoryInput {
        namespace: namespace.to_string(),
        key: key.to_string(),
        payload: "certified deletion target payload".to_string(),
        metadata,
        vector: Some(vec![0.1, 0.2, 0.3]),
        ..Default::default()
    }
}

/// Walk every on-disk WAL shard and look for the tombstone record of `node_id`.
fn wal_contains_delete(dir: &std::path::Path, node_id: u128) -> bool {
    let data = dir.join("data");
    if !data.exists() {
        return false;
    }
    for entry in std::fs::read_dir(&data).expect("read data dir") {
        let path = entry.expect("dir entry").path();
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        // Sharded WAL files are `vanta.shard<N>.wal` (the `.shards` meta file
        // is not a WAL).
        if !(name.starts_with("vanta.") && name.ends_with(".wal")) {
            continue;
        }
        let Ok(mut reader) = WalReader::open(&path) else {
            continue;
        };
        loop {
            match reader.next_record() {
                Ok(Some(WalRecord::Delete { id })) if id == node_id => return true,
                Ok(Some(_)) => {}
                Ok(None) => break,
                Err(_) => break,
            }
        }
    }
    false
}

#[test]
fn certified_delete_reports_zero_residues_with_verifiable_certificate() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = open_temp_db(&dir);
    let record = db
        .put(certified_input("cert", "alpha"))
        .expect("put record");

    let cert = db
        .delete_certified("cert", "alpha")
        .expect("delete_certified");

    // ── Certificate shape (C3) ──
    assert_eq!(cert.schema_version, 1);
    assert_eq!(cert.namespace, "cert");
    assert_eq!(cert.key, "alpha");
    assert_eq!(cert.node_id, record.node_id.to_string());
    assert_eq!(cert.status, "purged", "all surfaces clean → purged");
    assert!(
        cert.surfaces.len() >= 8,
        "certificate must inventory every surface, got {:?}",
        cert.surfaces
    );
    for surface in &cert.surfaces {
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
    // VER-01 chain is referenced (C2).
    assert!(cert.chain.chained);
    assert_eq!(cert.chain.format_version, 3);
    assert!(cert.chain.verify_command.contains("verify"));
    // Declared limits are never silent (pre-mortem F1).
    assert!(!cert.out_of_scope.is_empty());
    assert_eq!(cert.integrity.algorithm, "sha256");
    assert_eq!(cert.integrity.sha256.len(), 64, "hex SHA-256");

    // ── Zero residues reachable via the public API ──
    assert!(db.get("cert", "alpha").expect("get").is_none());
    assert_eq!(db.count("cert", None).expect("count"), 0);

    // ── WAL trace: tombstone record on disk (C2) ──
    // `put`/`delete` append to a buffered WAL; flush makes the frames durable
    // before the on-disk scan.
    db.flush().expect("flush WAL");
    assert!(
        wal_contains_delete(dir.path(), record.node_id),
        "WAL must carry the Delete tombstone for the record"
    );

    // ── Certificate is verifiable (integrity + re-scan) ──
    let json = serde_json::to_string(&cert).expect("serialize");
    let verification = db
        .verify_purge_certificate(&json)
        .expect("certificate must verify");
    assert_eq!(verification.status, "purged");
    assert!(
        verification
            .rechecked_surfaces
            .contains(&"store".to_string()),
        "store must be part of the re-scan"
    );

    db.close().expect("close");
}

#[test]
fn purge_certificate_is_deterministic_across_equivalent_databases() {
    fn certificate_body() -> serde_json::Value {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = open_temp_db(&dir);
        db.put(certified_input("cert", "beta")).expect("put");
        let cert = db.delete_certified("cert", "beta").expect("delete");
        db.close().expect("close");
        let mut value = serde_json::to_value(&cert).expect("to value");
        // `timestamp` varies by construction, and `integrity.sha256` is
        // derived from the timestamped body — every *claim* must otherwise
        // be byte-identical.
        let object = value.as_object_mut().expect("certificate object");
        object.remove("timestamp");
        object.remove("integrity");
        value
    }

    assert_eq!(
        certificate_body(),
        certificate_body(),
        "same state + same inputs → same certificate claims"
    );
}

#[test]
fn verify_rejects_tampered_certificate() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = open_temp_db(&dir);
    db.put(certified_input("cert", "gamma")).expect("put");
    let cert = db.delete_certified("cert", "gamma").expect("delete");

    // Tamper: flip a residues claim (the integrity hash must catch it).
    let mut tampered = serde_json::to_value(&cert).expect("to value");
    tampered["surfaces"][0]["residues"] = serde_json::json!(7);
    let err = db
        .verify_purge_certificate(&tampered.to_string())
        .expect_err("tampered certificate must NOT verify");
    assert!(
        err.to_string().contains("integrity"),
        "error must name the integrity failure, got: {err}"
    );

    // Tamper: change the attested node id.
    let mut tampered_id = serde_json::to_value(&cert).expect("to value");
    tampered_id["node_id"] = serde_json::json!("1");
    assert!(db
        .verify_purge_certificate(&tampered_id.to_string())
        .is_err());

    db.close().expect("close");
}

#[test]
fn verify_fails_when_residues_reappear_after_certificate() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = open_temp_db(&dir);
    db.put(certified_input("cert", "delta")).expect("put");
    let cert = db.delete_certified("cert", "delta").expect("delete");
    let json = serde_json::to_string(&cert).expect("serialize");

    // The record is re-created after the certificate was issued → the
    // certificate's "store absent" claim no longer holds.
    db.put(certified_input("cert", "delta")).expect("re-put");

    let err = db
        .verify_purge_certificate(&json)
        .expect_err("stale certificate must NOT verify once residues reappear");
    assert!(
        err.to_string().contains("residue"),
        "error must report reappeared residues, got: {err}"
    );

    db.close().expect("close");
}

#[test]
fn certified_delete_not_found_is_honest_not_silent() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = open_temp_db(&dir);

    let cert = db
        .delete_certified("cert", "ghost")
        .expect("delete_certified on missing key");

    assert_eq!(cert.status, "not_found");
    assert!(
        cert.surfaces.iter().any(|s| s.action == "not-assessed"),
        "record-dependent surfaces must be marked not-assessed, not clean: {:?}",
        cert.surfaces
    );
    // The certificate is still well-formed and verifiable.
    let verification = db
        .verify_purge_certificate(&serde_json::to_string(&cert).expect("serialize"))
        .expect("not_found certificate must verify");
    assert_eq!(verification.status, "not_found");

    db.close().expect("close");
}

/// H-1 (review VER-02): a **claimless** certificate — every surface claim
/// omitted — with a **recomputed** integrity hash must still fail
/// verification while residues are live. Verification is not claim-driven:
/// the canonical surface inventory is validated before any verdict.
#[test]
fn verify_rejects_claimless_certificate_even_with_recomputed_hash() {
    use sha2::{Digest, Sha256};

    let dir = tempfile::tempdir().expect("tempdir");
    let db = open_temp_db(&dir);
    db.put(certified_input("cert", "epsilon")).expect("put");
    let cert = db.delete_certified("cert", "epsilon").expect("delete");

    // Live residues: the record is re-created after the certified delete.
    db.put(certified_input("cert", "epsilon")).expect("re-put");
    assert!(db.get("cert", "epsilon").expect("get").is_some());

    // Craft the evasion: drop every surface claim and recompute the integrity
    // hash exactly like the producer would (canonical body, empty hash field).
    let mut claimless = cert.clone();
    claimless.surfaces.clear();
    claimless.integrity.sha256.clear();
    let canonical = serde_json::to_vec(&claimless).expect("canonical bytes");
    let digest = Sha256::digest(&canonical);
    claimless.integrity.sha256 = digest.iter().map(|b| format!("{b:02x}")).collect();

    let err = db
        .verify_purge_certificate(&serde_json::to_string(&claimless).expect("serialize"))
        .expect_err("claimless certificate must NOT verify, even with a valid hash");
    assert!(
        err.to_string().contains("surface"),
        "rejection must name the surface inventory, got: {err}"
    );
    // Verification must not mutate anything: the residues stay live.
    assert!(db.get("cert", "epsilon").expect("get").is_some());

    // And the pristine (claim-carrying) certificate now fails too: live
    // residues with a non-`not_found` status reject the verdict (residues-aware).
    let err = db
        .verify_purge_certificate(&serde_json::to_string(&cert).expect("serialize"))
        .expect_err("live residues must reject a purge certificate");
    assert!(
        err.to_string().contains("residue"),
        "residues-aware verdict must report residues, got: {err}"
    );

    db.close().expect("close");
}

// ── CLI surface (VER-02): `delete --attest` + `certificate verify` ──
#[cfg(feature = "cli")]
mod cli_surface {
    use super::*;
    use vantadb::cli_handlers;

    #[test]
    fn cli_delete_attest_purges_and_certificate_verify_accepts_it() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().to_string_lossy().to_string();

        let db = open_temp_db(&dir);
        db.put(certified_input("cli", "alpha")).expect("put");
        db.flush().expect("flush");
        db.close().expect("close");

        // `delete --attest` (handler; the bin dispatch is a thin wrapper).
        cli_handlers::cmd_delete_certified(&path, "cli", "alpha", None, false, true)
            .expect("cmd_delete_certified");

        // Record is physically gone after the attested delete.
        let db = open_temp_db(&dir);
        assert!(db.get("cli", "alpha").expect("get").is_none());
        db.close().expect("close");
    }

    #[test]
    fn cli_certificate_verify_returns_exit_code_0_valid_1_tampered() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().to_string_lossy().to_string();

        let db = open_temp_db(&dir);
        db.put(certified_input("cli", "beta")).expect("put");
        let cert = db.delete_certified("cli", "beta").expect("delete");
        db.close().expect("close");

        let cert_path = dir.path().join("certificate.json");
        let cert_file = cert_path.to_str().expect("utf8 path").to_string();
        std::fs::write(
            &cert_path,
            serde_json::to_string_pretty(&cert).expect("serialize"),
        )
        .expect("write certificate");

        let code = cli_handlers::cmd_certificate_verify(&path, &cert_file, true)
            .expect("verify valid certificate");
        assert_eq!(code, 0, "valid certificate → exit code 0");

        // The `delete --attest --json` envelope must round-trip too.
        let envelope = serde_json::json!({ "deleted": true, "certificate": &cert });
        std::fs::write(&cert_path, envelope.to_string()).expect("write envelope");
        let code =
            cli_handlers::cmd_certificate_verify(&path, &cert_file, true).expect("verify envelope");
        assert_eq!(code, 0, "certificate envelope → exit code 0");

        // Tampered certificate → exit code 1 (never a panic).
        let mut tampered = serde_json::to_value(&cert).expect("to value");
        tampered["surfaces"][0]["residues"] = serde_json::json!(3);
        std::fs::write(&cert_path, tampered.to_string()).expect("write tampered");
        let code = cli_handlers::cmd_certificate_verify(&path, &cert_file, true)
            .expect("verify tampered certificate");
        assert_eq!(code, 1, "tampered certificate → exit code 1");
    }

    #[test]
    fn cli_delete_attest_out_writes_roundtrippable_certificate() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().to_string_lossy().to_string();
        let cert_path = dir.path().join("certificate.json");
        let cert_file = cert_path.to_str().expect("utf8 path").to_string();

        let db = open_temp_db(&dir);
        db.put(certified_input("cli", "gamma")).expect("put");
        db.close().expect("close");

        // `delete --attest --out <file>`: the CLI writes the raw certificate
        // itself (UTF-8) — no shell redirection in the middle.
        cli_handlers::cmd_delete_certified(&path, "cli", "gamma", Some(&cert_file), false, true)
            .expect("cmd_delete_certified --out");
        assert!(cert_path.exists(), "--out must write the certificate file");

        let code = cli_handlers::cmd_certificate_verify(&path, &cert_file, true)
            .expect("verify --out file");
        assert_eq!(code, 0, "certificate written with --out must verify");
    }
}
