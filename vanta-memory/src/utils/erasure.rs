//! MEMG-17 (piezas c + b): erasure criptográfica por destrucción de DEK y
//! recibos verificables (contrato VER-02).
//!
//! # Model
//!
//! A **scope** (caller-chosen label, e.g. an agent or tenant) registers a
//! random 32-byte DEK (OS CSPRNG) that is stored **wrapped** with the core
//! master [`Cipher`] (AES-256-GCM; [`Cipher::from_env`] resolves
//! `VANTADB_ENCRYPTION_KEY` in production) as a record under the reserved
//! namespace [`ERASURE_DEK_NAMESPACE`]. Payloads are sealed under the DEK
//! ([`seal`]) and can only be opened with it ([`open`]).
//!
//! [`erase_scope`] destroys the wrapped DEK through the core delete path
//! (backend tombstone + version-history purge) and emits an
//! [`ErasureReceipt`]: everything sealed under that DEK becomes
//! cryptographically unrecoverable. This is composed from the existing crypto
//! primitives — no cipher is implemented here.
//!
//! # Honest by construction (mirrors `src/attestation.rs`, VER-02)
//!
//! The receipt inventories the re-checkable surfaces (`dek_record`,
//! `version_history`, `wal`), always carries its declared limits
//! (`out_of_scope`) and an sha256 self-hash (`integrity`); verification is
//! **not claim-driven** — schema first, live re-scan second. The receipt
//! contains no key material. Signing (ML-DSA-65) is not sanctioned by any
//! spec and is declared out of scope.
//!
//! Requires the crate feature `erasure` (pulls the core `encryption`
//! feature).

use rand::RngCore;
use sha2::{Digest, Sha256};
use thiserror::Error;

use vantadb::attestation::{chain_evidence, CertificateIntegrity, SurfaceReport};
use vantadb::crypto::Cipher;
use vantadb::sdk::{Embedded, MemoryInput, MemoryMetadata};

use crate::core::conversation::sanitize_key;

/// Reserved namespace holding the wrapped per-scope DEKs.
pub const ERASURE_DEK_NAMESPACE: &str = "erasure/dek";

/// Erasure receipt schema version (v1, MEMG-17).
pub const ERASURE_RECEIPT_SCHEMA_VERSION: u32 = 1;

/// The canonical re-checkable surfaces a valid receipt must cover exactly once.
const ERASURE_SURFACES: [&str; 3] = ["dek_record", "version_history", "wal"];

/// Errors surfaced by the erasure surface.
#[derive(Debug, Error)]
pub enum ErasureError {
    /// A DEK is already registered for this scope (refusing to rekey —
    /// erase first).
    #[error("erasure scope '{0}' already has a registered DEK (erase it first)")]
    ScopeExists(String),
    /// No DEK registered for this scope (never created, or already erased).
    #[error("erasure scope '{0}' has no registered DEK (never created or already erased)")]
    DekNotFound(String),
    /// The stored DEK record is malformed (not hex, not 32 bytes).
    #[error("stored DEK for scope '{0}' is malformed")]
    DekMalformed(String),
    /// The master key cannot unwrap the DEK (wrong master key?).
    #[error("master key cannot unwrap the DEK for scope '{0}' (wrong master key?)")]
    DekUnwrap(String),
    /// A sealed payload failed to decrypt (tampered, truncated, wrong scope).
    #[error(
        "sealed payload for scope '{0}' failed to decrypt (tampered, truncated, or wrong scope)"
    )]
    DecryptFailed(String),
    /// The erasure receipt is structurally invalid.
    #[error("erasure receipt is invalid: {0}")]
    ReceiptInvalid(String),
    /// Store error.
    #[error(transparent)]
    Store(#[from] vantadb::error::Error),
    /// Serialization error.
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

/// A verifiable erasure receipt: which scope's DEK was destroyed, on which
/// surfaces, with what proof (contract mirror of the VER-02 purge
/// certificate).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ErasureReceipt {
    /// Schema version ([`ERASURE_RECEIPT_SCHEMA_VERSION`]).
    pub schema_version: u32,
    /// ISO 8601 UTC timestamp.
    pub timestamp: String,
    /// Erasure scope (as supplied by the caller).
    pub scope: String,
    /// Why the erasure happened (audit reason).
    pub reason: String,
    /// `erased` (DEK destroyed, clean) | `residues` (something remains) |
    /// `not_found` (no DEK registered; surfaces still scanned).
    pub status: String,
    /// Fixed-order per-surface inventory.
    pub surfaces: Vec<SurfaceReport>,
    /// VER-01 chain reference (same builder the purge certificates use).
    pub chain: vantadb::attestation::ChainEvidence,
    /// Declared limits — always present, never silent.
    pub out_of_scope: Vec<String>,
    /// Receipt self-integrity (sha256 over the canonical body).
    pub integrity: CertificateIntegrity,
}

/// Register a fresh random DEK for `scope`.
///
/// The scope maps to the DEK record key via [`sanitize_key`] (same rules the
/// record ids use); scopes whose sanitized forms collide are the same scope,
/// so a second `create_scope` for them is rejected.
///
/// Concurrency: the exists-check + put is not atomic — serialize concurrent
/// `create_scope` calls for the same scope in the host (a lost race would
/// replace a DEK and orphan data sealed under the first one).
///
/// # Errors
///
/// [`ErasureError::ScopeExists`] when a DEK is already registered for the
/// scope (erase it first — rekeying would orphan previously sealed data).
pub fn create_scope(db: &Embedded, master: &Cipher, scope: &str) -> Result<(), ErasureError> {
    let key = sanitize_key(scope);
    if db.get(ERASURE_DEK_NAMESPACE, &key)?.is_some() {
        return Err(ErasureError::ScopeExists(scope.to_string()));
    }
    let mut dek = [0u8; 32];
    rand::rng().fill_bytes(&mut dek);
    let wrapped = master.encrypt(&dek);
    db.put(MemoryInput {
        namespace: ERASURE_DEK_NAMESPACE.to_string(),
        key,
        payload: hex_lower(&wrapped),
        metadata: MemoryMetadata::new(),
        ..Default::default()
    })?;
    Ok(())
}

/// Seal `plaintext` under the scope's DEK.
///
/// Returns the ciphertext blob (caller-held — this module does not track or
/// delete sealed copies). [`open`] with the same scope and master recovers
/// the plaintext; after [`erase_scope`] it is unrecoverable.
pub fn seal(
    db: &Embedded,
    master: &Cipher,
    scope: &str,
    plaintext: &[u8],
) -> Result<Vec<u8>, ErasureError> {
    let dek = load_dek(db, master, scope)?;
    Ok(Cipher::new(&dek).encrypt(plaintext))
}

/// Open a blob sealed by [`seal`] under the same scope.
///
/// # Errors
///
/// [`ErasureError::DekNotFound`] when the scope was never created or was
/// erased; [`ErasureError::DecryptFailed`] when the AEAD tag rejects the
/// payload (tamper/truncation/wrong scope).
pub fn open(
    db: &Embedded,
    master: &Cipher,
    scope: &str,
    blob: &[u8],
) -> Result<Vec<u8>, ErasureError> {
    let dek = load_dek(db, master, scope)?;
    Cipher::new(&dek)
        .decrypt(blob)
        .map_err(|_| ErasureError::DecryptFailed(scope.to_string()))
}

/// Destroy the scope's DEK (cryptographic erasure) and emit the receipt.
///
/// Deletes the wrapped DEK record through the core delete path — backend
/// tombstone + retained-version purge (VS-CORE-07) — then inventories the
/// re-checkable surfaces. Emits `not_found` receipts too (nothing to erase is
/// still attested, never silent).
///
/// # Durability (cross-process verification)
///
/// The receipt reflects the state of this live engine handle. A verifier that
/// opens the database **read-only** does not replay the WAL, so callers that
/// intend to verify the receipt after this process exits must persist the
/// erase first — call [`Embedded::flush`] or [`Embedded::close`] after this
/// call (same caveat class as `delete_certified`).
pub fn erase_scope(
    db: &Embedded,
    scope: &str,
    reason: &str,
) -> Result<ErasureReceipt, ErasureError> {
    let key = sanitize_key(scope);
    let existed = db.delete(ERASURE_DEK_NAMESPACE, &key)?;
    let dek_residues = u64::from(db.get(ERASURE_DEK_NAMESPACE, &key)?.is_some());
    let version_residues = db.versions(ERASURE_DEK_NAMESPACE, &key)?.len() as u64;

    let surfaces = if existed {
        erased_surfaces(dek_residues, version_residues)
    } else {
        not_found_surfaces(dek_residues, version_residues)
    };
    let residues_total: u64 = surfaces.iter().map(|s| s.residues).sum();
    let status = match (existed, residues_total) {
        (false, _) => "not_found",
        (true, 0) => "erased",
        (true, _) => "residues",
    };

    let receipt = ErasureReceipt {
        schema_version: ERASURE_RECEIPT_SCHEMA_VERSION,
        timestamp: now_iso(),
        scope: scope.to_string(),
        reason: reason.to_string(),
        status: status.to_string(),
        surfaces,
        chain: chain_evidence(),
        out_of_scope: declared_limits(),
        integrity: CertificateIntegrity {
            algorithm: "sha256".into(),
            sha256: String::new(),
        },
    };
    finalize(receipt)
}

/// Result of verifying a stored receipt against the live database.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ErasureVerification {
    /// Schema version of the verified receipt.
    pub schema_version: u32,
    /// Receipt scope (echoed).
    pub scope: String,
    /// Receipt status (echoed).
    pub status: String,
    /// `ok` when the integrity hash matched.
    pub integrity: String,
    /// Surfaces re-scanned against the live state.
    pub rechecked_surfaces: Vec<String>,
    /// Residues currently reachable across the re-scanned surfaces.
    pub residues_now: u64,
}

/// Verify a stored receipt (JSON) against the live database.
///
/// Schema + integrity hash first, then a live re-scan of the re-checkable
/// surfaces (`dek_record`, `version_history`). Verification is **not
/// claim-driven**: before any verdict the receipt must be structurally valid
/// ([`validate_schema`]) — known `status`, non-empty declared limits, all
/// canonical surfaces present exactly once. A surface the receipt claimed
/// clean (`residues == 0`) that now holds an entry fails the verification
/// (residues reappeared — e.g. the scope was re-created), and live residues
/// fail it unless the receipt is a `not_found` report.
///
/// The receipt is not bound to a database instance — verification re-checks
/// the scope against whichever database is opened.
pub fn verify_erasure_receipt(
    db: &Embedded,
    receipt_json: &str,
) -> Result<ErasureVerification, ErasureError> {
    let receipt: ErasureReceipt = serde_json::from_str(receipt_json)
        .map_err(|e| ErasureError::ReceiptInvalid(format!("malformed receipt JSON: {e}")))?;

    if receipt.schema_version != ERASURE_RECEIPT_SCHEMA_VERSION {
        return Err(ErasureError::ReceiptInvalid(format!(
            "unsupported receipt schema version {} (expected {ERASURE_RECEIPT_SCHEMA_VERSION})",
            receipt.schema_version
        )));
    }
    let expected = hex_lower(&Sha256::digest(canonical_bytes(&receipt)?));
    if receipt.integrity.algorithm != "sha256" || receipt.integrity.sha256 != expected {
        return Err(ErasureError::ReceiptInvalid(
            "integrity hash mismatch — the receipt was edited or corrupted".into(),
        ));
    }
    validate_schema(&receipt)?;

    let key = sanitize_key(&receipt.scope);
    let mut rechecked: Vec<String> = Vec::new();
    let mut residues_now = 0u64;
    let mut broken: Vec<String> = Vec::new();

    let dek_present = db.get(ERASURE_DEK_NAMESPACE, &key)?.is_some();
    rechecked.push("dek_record".to_string());
    if dek_present {
        residues_now += 1;
        if claims_zero(&receipt, "dek_record") {
            broken.push("dek_record".to_string());
        }
    }

    let versions = db.versions(ERASURE_DEK_NAMESPACE, &key)?.len() as u64;
    rechecked.push("version_history".to_string());
    residues_now += versions;
    if versions > 0 && claims_zero(&receipt, "version_history") {
        broken.push("version_history".to_string());
    }

    if !broken.is_empty() {
        return Err(ErasureError::ReceiptInvalid(format!(
            "{residues_now} residue(s) reappeared after the receipt was issued (surfaces: {})",
            broken.join(", ")
        )));
    }
    if residues_now > 0 && receipt.status != "not_found" {
        return Err(ErasureError::ReceiptInvalid(format!(
            "{residues_now} residue(s) found on the re-checkable surfaces; receipt status '{}' does not attest a clean erasure",
            receipt.status
        )));
    }

    Ok(ErasureVerification {
        schema_version: receipt.schema_version,
        scope: receipt.scope,
        status: receipt.status,
        integrity: "ok".into(),
        rechecked_surfaces: rechecked,
        residues_now,
    })
}

/// Whether the receipt claims `surface` has zero residues.
fn claims_zero(receipt: &ErasureReceipt, surface: &str) -> bool {
    receipt
        .surfaces
        .iter()
        .find(|s| s.surface == surface)
        .is_some_and(|s| s.residues == 0)
}

/// Load and unwrap the scope's DEK.
fn load_dek(db: &Embedded, master: &Cipher, scope: &str) -> Result<[u8; 32], ErasureError> {
    let key = sanitize_key(scope);
    let record = db
        .get(ERASURE_DEK_NAMESPACE, &key)?
        .ok_or_else(|| ErasureError::DekNotFound(scope.to_string()))?;
    let wrapped =
        decode_hex(&record.payload).ok_or_else(|| ErasureError::DekMalformed(scope.to_string()))?;
    let dek = master
        .decrypt(&wrapped)
        .map_err(|_| ErasureError::DekUnwrap(scope.to_string()))?;
    dek.as_slice()
        .try_into()
        .map_err(|_| ErasureError::DekMalformed(scope.to_string()))
}

fn surface(surface: &str, action: &str, residues: u64, evidence: &str) -> SurfaceReport {
    SurfaceReport {
        surface: surface.into(),
        action: action.into(),
        residues,
        evidence: evidence.into(),
    }
}

fn erased_surfaces(dek_residues: u64, version_residues: u64) -> Vec<SurfaceReport> {
    vec![
        surface(
            "dek_record",
            "deleted",
            dek_residues,
            "point-read of the DEK record under `erasure/dek` is absent after the delete",
        ),
        surface(
            "version_history",
            "deleted",
            version_residues,
            "live version-history prefix scan for the DEK key is empty (delete purges retained versions)",
        ),
        surface(
            "wal",
            "tombstone-recorded",
            0,
            "Delete frame recorded by the core delete path (append-only history keeps the trace by design); `vanta-cli verify` is the chain authority",
        ),
    ]
}

fn not_found_surfaces(dek_residues: u64, version_residues: u64) -> Vec<SurfaceReport> {
    vec![
        surface(
            "dek_record",
            "scanned",
            dek_residues,
            "point-read of the DEK record under `erasure/dek`",
        ),
        surface(
            "version_history",
            "scanned",
            version_residues,
            "live version-history prefix scan for the DEK key",
        ),
        surface(
            "wal",
            "not-recorded",
            0,
            "no delete occurred for this scope in this call",
        ),
    ]
}

/// Declared limits of a cryptographic erasure — always present, never silent.
fn declared_limits() -> Vec<String> {
    vec![
        "cryptographic erasure is scoped to the wrapped DEK record in the live store: payloads sealed under it become unrecoverable without the DEK — this is logical key destruction, not secure erase of the sealed bytes (same media caveat as VER-02)".into(),
        "the master key is not destroyed: erasure holds while the wrapped DEK is unrecoverable from every store you control".into(),
        "copies of the wrapped DEK outside the live store (snapshots, backups, archived WAL segments, exports) are not destroyed — destroy or rotate the master key (FIND-194) or delete those stores to close them".into(),
        "sealed blobs are caller-held: this module does not track or delete ciphertext copies".into(),
        "the DEK record's derived indexes (text/sparse postings) are cleaned by the core delete path but not re-scanned by this receipt — a silent cleanup regression there would not be detected by verification (declared, same class as VER-02's record-derived surfaces)".into(),
        "no digital signature: receipt integrity is an sha256 self-hash plus a live re-scan (the VER-02 contract); ML-DSA-65 signing is not sanctioned by any spec and an actor who recomputes the hash is not detected".into(),
        "audit logs: delete events (scope) survive in the audit JSONL until retention drops them".into(),
    ]
}

/// Canonical bytes for the integrity hash: the receipt body with
/// `integrity.sha256` emptied (stable across re-reads).
fn canonical_bytes(receipt: &ErasureReceipt) -> Result<Vec<u8>, ErasureError> {
    let mut canonical = receipt.clone();
    canonical.integrity.sha256.clear();
    serde_json::to_vec(&canonical).map_err(ErasureError::Serde)
}

/// The producer must never emit a structurally invalid receipt.
fn finalize(mut receipt: ErasureReceipt) -> Result<ErasureReceipt, ErasureError> {
    validate_schema(&receipt)?;
    let digest = Sha256::digest(canonical_bytes(&receipt)?);
    receipt.integrity.sha256 = hex_lower(&digest);
    Ok(receipt)
}

/// Structural validation (claim-driven rejection): a well-formed receipt must
/// carry a known `status`, a non-empty `out_of_scope` and **every canonical
/// surface exactly once** — no omissions, duplicates or unknowns. A receipt
/// that omits the surface holding live residues is rejected before any
/// verification verdict.
fn validate_schema(receipt: &ErasureReceipt) -> Result<(), ErasureError> {
    const VALID_STATUS: [&str; 3] = ["erased", "residues", "not_found"];
    if !VALID_STATUS.contains(&receipt.status.as_str()) {
        return Err(ErasureError::ReceiptInvalid(format!(
            "unknown status '{}' (expected one of {VALID_STATUS:?})",
            receipt.status
        )));
    }
    if receipt.out_of_scope.is_empty() {
        return Err(ErasureError::ReceiptInvalid(
            "declared limits must not be empty".into(),
        ));
    }
    for canonical in ERASURE_SURFACES {
        let occurrences = receipt
            .surfaces
            .iter()
            .filter(|s| s.surface == canonical)
            .count();
        if occurrences != 1 {
            return Err(ErasureError::ReceiptInvalid(format!(
                "surface '{canonical}' must appear exactly once (found {occurrences})"
            )));
        }
    }
    if let Some(unknown) = receipt
        .surfaces
        .iter()
        .find(|s| !ERASURE_SURFACES.contains(&s.surface.as_str()))
    {
        return Err(ErasureError::ReceiptInvalid(format!(
            "unknown surface '{}'",
            unknown.surface
        )));
    }
    if receipt.surfaces.len() != ERASURE_SURFACES.len() {
        return Err(ErasureError::ReceiptInvalid(format!(
            "expected {} canonical surfaces, found {}",
            ERASURE_SURFACES.len(),
            receipt.surfaces.len()
        )));
    }
    Ok(())
}

/// ISO 8601 UTC timestamp with millisecond precision.
fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

fn hex_lower(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn decode_hex(s: &str) -> Option<Vec<u8>> {
    // Byte-wise decode: slicing a &str at even indices could split a
    // multi-byte char (panic) on a corrupted store payload — never do that.
    let bytes = s.as_bytes();
    if !bytes.len().is_multiple_of(2) {
        return None;
    }
    let mut out = Vec::with_capacity(bytes.len() / 2);
    for chunk in bytes.chunks_exact(2) {
        let hi = (chunk[0] as char).to_digit(16)?;
        let lo = (chunk[1] as char).to_digit(16)?;
        out.push(((hi as u8) << 4) | lo as u8);
    }
    Some(out)
}
