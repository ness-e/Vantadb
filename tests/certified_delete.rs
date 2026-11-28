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

/// EGO-16 / FIND-343: auditoría de borrado físico — tras delete + vacuum +
/// merge + WAL-compact + flush + close, ningún byte del payload puede seguir
/// legible en el directorio de datos.
///
/// Implementado en FIND-343 (2026-10-23): `FjallBackend` ahora implementa
/// `Compactable` mediante reescritura atómica de keyspaces. `compact_wal()`
/// delega en `Compactable::compact()` tras rotar el WAL, produciendo un
/// journal fresco (`0.jnl`) sin rastro de los datos eliminados.
/// El WAL rotado sigue siendo descartado por `rotate_all()`.
///
/// Corre en Fast Gate y en jobs de certificación/auditoría.
#[test]
fn physical_erasure_no_payload_bytes_after_full_maintenance() {
    use vantadb::PipelineMode;

    // P2-01 M4: with a key in the environment the database would be encrypted
    // and the canary would be absent **because of the ciphertext**, not
    // because of the erasure — a vacuous measurement. nextest runs one process
    // per test, so clearing the key here is safe and makes the premise honest.
    for var in [
        "VANTADB_ENCRYPTION_KEY",
        "VANTADB_ENCRYPTION_KEY_FILE",
        "VANTADB_ENCRYPTION_KEY_CMD",
    ] {
        std::env::remove_var(var);
    }

    // Marcador único e improbable: solo existe en el payload de este test.
    const MARKER: &str = "EGO16-PROBE-payload-debe-desaparecer-7f3a9c1e5b2d4860";
    const PAYLOAD: &str =
        "borrado fisico EGO16-PROBE-payload-debe-desaparecer-7f3a9c1e5b2d4860 fin";

    let dir = tempfile::tempdir().expect("tempdir");
    let db = open_temp_db(&dir);
    let mut input = certified_input("erase", "target");
    input.payload = PAYLOAD.to_string();
    db.put(input).expect("put");

    // The measurement is only meaningful on a **plaintext** database: no key
    // resolved means the engine never creates the key registry.
    fn has_registry(path: &std::path::Path) -> bool {
        let Ok(entries) = std::fs::read_dir(path) else {
            return false;
        };
        entries.flatten().any(|e| {
            let p = e.path();
            p.is_dir() && has_registry(&p)
                || p.file_name().is_some_and(|n| n == "encryption.registry")
        })
    }
    assert!(
        !has_registry(dir.path()),
        "the canary must be measured on a plaintext database: a key in the \
         environment would make 0 hits meaningless"
    );

    db.delete("erase", "target").expect("delete");

    // Mantenimiento completo: purga HNSW, reescritura de segmentos,
    // compactación WAL, descarga y cierre (los mmap/WAL deben estar
    // descargados antes del escaneo en disco).
    let vacuum = db.vacuum().expect("vacuum");
    assert!(vacuum.success, "vacuum must succeed");
    let pipe = db
        .pipeline(PipelineMode::MergeOnly)
        .expect("merge pipeline");
    assert!(pipe.success, "merge pipeline must succeed");
    db.compact_wal().expect("compact wal");
    db.flush().expect("flush");
    db.close().expect("close");

    // Escaneo crudo recursivo del directorio de datos.
    fn visit(path: &std::path::Path, needle: &[u8], hits: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(path) else {
            return;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                visit(&p, needle, hits);
            } else if let Ok(bytes) = std::fs::read(&p) {
                if bytes.windows(needle.len()).any(|w| w == needle) {
                    hits.push(p.to_string_lossy().to_string());
                }
            }
        }
    }
    let mut hits = Vec::new();
    visit(dir.path(), MARKER.as_bytes(), &mut hits);
    assert!(
        hits.is_empty(),
        "physical erasure failed: payload bytes still readable on disk in: {hits:?} \
         (API-level purge only — see EGO-16)"
    );
}

/// Raw recursive byte scan of the database directory (VER-02 / EGO-16 /
/// FIND-343 / FIND-826): the only measurement that answers "is the payload still
/// readable on disk".
fn scan_bytes(dir: &std::path::Path, needle: &[u8]) -> Vec<String> {
    fn visit(path: &std::path::Path, needle: &[u8], hits: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(path) else {
            return;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                visit(&p, needle, hits);
            } else if let Ok(bytes) = std::fs::read(&p) {
                if bytes.windows(needle.len()).any(|w| w == needle) {
                    hits.push(p.to_string_lossy().to_string());
                }
            }
        }
    }
    let mut hits = Vec::new();
    visit(dir, needle, &mut hits);
    hits
}

/// FIND-826: `forget(mode = Certified)` must erase the **bytes**, not just the
/// record. On HEAD the certified path is byte-identical to `hard`: the payload
/// survives in the Fjall journal (`0.jnl`) and `data/vanta.shard0.wal`, and the
/// vector in `vstore_L0.vanta`, until someone runs `compact_wal` +
/// `compact_layout` by hand — while the receipt already says `purged`.
///
/// The witness record must survive, so a "0 hits" cannot come from wiping the
/// database. Cheap enough for the Fast Gate: two records, one compaction.
#[test]
fn forget_certified_erases_payload_and_vector_bytes_with_witness_alive() {
    use vantadb::{ForgetMode, ForgetTarget};

    // P2-01 M4 (FIND-343): a key in the environment would encrypt the database
    // and 0 hits would mean "ciphertext", not "erased" — a vacuous measurement.
    for var in [
        "VANTADB_ENCRYPTION_KEY",
        "VANTADB_ENCRYPTION_KEY_FILE",
        "VANTADB_ENCRYPTION_KEY_CMD",
    ] {
        std::env::remove_var(var);
    }

    const MARKER: &str = "FIND826-PROBE-certified-erasure-9b1d7e4a2c8f0356";
    const WITNESS: &str = "FIND826-WITNESS-must-survive-every-forget-mode";

    let dir = tempfile::tempdir().expect("tempdir");
    let db = open_temp_db(&dir);

    let mut subject = certified_input("erase", "subject");
    subject.payload = format!("GDPR subject record marker {MARKER} fin");
    subject.vector = Some(vec![0.6, 0.8, 0.0]);
    db.put(subject).expect("put subject");

    let mut witness = certified_input("erase", "witness");
    witness.payload = WITNESS.to_string();
    // A different vector on purpose: a shared vector would make the vector-byte
    // needle match the witness and the instrument would report its own fixture.
    witness.vector = Some(vec![0.0, 0.28, 0.96]);
    db.put(witness).expect("put witness");

    // The needle is the vector **as stored** (normalization included), so the
    // test measures the bytes on disk instead of the bytes it wrote.
    let stored = db
        .get("erase", "subject")
        .expect("get")
        .expect("subject present");
    let vector_needle: Vec<u8> = stored
        .vector
        .as_ref()
        .expect("stored vector")
        .iter()
        .flat_map(|f| f.to_le_bytes())
        .collect();

    let report = db
        .forget(
            "erase",
            ForgetTarget::Key("subject".to_string()),
            ForgetMode::Certified,
        )
        .expect("certified forget");
    assert_eq!(report.forgotten_count, 1, "the subject must be forgotten");
    assert_eq!(
        report.certificates.len(),
        1,
        "one certificate per purged key"
    );

    db.flush().expect("flush");
    db.close().expect("close");

    fn scan(path: &std::path::Path, needle: &[u8]) -> Vec<String> {
        scan_bytes(path, needle)
    }

    let payload_hits = scan(dir.path(), MARKER.as_bytes());
    let vector_hits = scan(dir.path(), &vector_needle);
    let witness_hits = scan(dir.path(), WITNESS.as_bytes());

    assert!(
        payload_hits.is_empty(),
        "certified forget left the payload on disk in {payload_hits:?}: the \
         receipt claims `purged` while the bytes are recoverable until a manual \
         `compact_wal` + `compact_layout` (FIND-826, GDPR Art. 17)"
    );
    assert!(
        vector_hits.is_empty(),
        "certified forget left the vector bytes on disk in {vector_hits:?}: the \
         index still holds the embedding (FIND-826)"
    );
    assert!(
        !witness_hits.is_empty(),
        "the witness record must survive a certified forget — an empty database \
         is not an erasure"
    );
}

/// Same assertion for the single-key surface the CLI uses (`delete --attest`
/// wraps `delete_certified`): the receipt is what a regulator reads, so it must
/// not be emitted before the bytes are gone.
#[test]
fn delete_certified_erases_payload_bytes_too() {
    const MARKER: &str = "FIND826-PROBE-delete-certified-4d2f8a1c60e93b57";

    for var in [
        "VANTADB_ENCRYPTION_KEY",
        "VANTADB_ENCRYPTION_KEY_FILE",
        "VANTADB_ENCRYPTION_KEY_CMD",
    ] {
        std::env::remove_var(var);
    }

    let dir = tempfile::tempdir().expect("tempdir");
    let db = open_temp_db(&dir);
    let mut input = certified_input("erase", "single");
    input.payload = format!("single-key certified delete marker {MARKER} fin");
    db.put(input).expect("put");

    let cert = db.delete_certified("erase", "single").expect("delete");
    assert_eq!(cert.status, "purged");
    db.flush().expect("flush");
    db.close().expect("close");

    let hits = scan_bytes(dir.path(), MARKER.as_bytes());
    assert!(
        hits.is_empty(),
        "delete_certified left the payload on disk in {hits:?} while attesting \
         `purged` (FIND-826)"
    );
}

/// FIND-826 D1: the receipt IS the claim that the bytes are gone, so a
/// certified path whose physical rewrite cannot run must **fail closed** —
/// no certificate, an error that says the record is already erased logically.
///
/// The failure is forced deterministically: the backend compaction temp
/// directory is occupied by a file, so `Compactable::compact()` fails at its
/// stale-tmp cleanup (the same class of error that used to be swallowed as a
/// `warn!` under 12 threads).
#[test]
fn certified_delete_fails_closed_when_the_journal_rewrite_cannot_run() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = open_temp_db(&dir);
    db.put(certified_input("erase", "blocked")).expect("put");

    // `<parent>/.<db dir name>.fjall_compaction_tmp_<pid>` — per database since
    // D1; a regular file there makes `remove_dir_all` fail.
    let db_path = dir.path().to_path_buf();
    let blocker = db_path.parent().expect("parent").join(format!(
        ".{}.fjall_compaction_tmp_{}",
        db_path.file_name().expect("file name").to_string_lossy(),
        std::process::id()
    ));
    std::fs::write(&blocker, b"blocker").expect("write blocker");

    let err = db
        .delete_certified("erase", "blocked")
        .expect_err("a certified delete without the physical rewrite must not sign a receipt");
    let msg = err.to_string();
    assert!(
        msg.contains("already erased logically"),
        "the error must say the logical erase happened, got: {msg}"
    );
    assert!(
        msg.contains("physical rewrite"),
        "the error must name the physical rewrite as what failed, got: {msg}"
    );
    // …and it is true: the record is gone from the API even though the bytes
    // are not provably gone from the journal.
    assert!(db.get("erase", "blocked").expect("get").is_none());

    let _ = std::fs::remove_file(&blocker);
    db.close().expect("close");
}

/// The in-memory/WASM backend has no on-disk bytes to rewrite: a certified
/// forget there must still succeed (the WASM binding maps `"certified"` to this
/// same path) instead of failing on the compaction it does not need.
#[test]
fn forget_certified_on_in_memory_backend_still_succeeds() {
    use vantadb::{BackendKind, ForgetMode, ForgetTarget};

    let db = Embedded::open_with_config(Config {
        storage_path: ":memory:".into(),
        backend_kind: BackendKind::InMemory,
        ..Default::default()
    })
    .expect("open in-memory");

    db.put(certified_input("erase", "subject")).expect("put");
    let report = db
        .forget(
            "erase",
            ForgetTarget::Key("subject".to_string()),
            ForgetMode::Certified,
        )
        .expect("certified forget on an in-memory database");
    assert_eq!(report.forgotten_count, 1);
    assert_eq!(report.certificates.len(), 1);
    assert!(db.get("erase", "subject").expect("get").is_none());
    db.close().expect("close");
}

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
