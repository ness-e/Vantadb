//! Certified purge (VER-02): per-surface residue inventory + JSON certificate.
//!
//! # Scope (honest by construction)
//!
//! `src/shred/` is **JSON Shredding** (typed columnar metadata storage), not
//! secure-delete. The certificate therefore declares, per surface, what was
//! swept and what remains — and separates *logical* purge (no entry is
//! reachable) from *physical* purge (bytes may survive in storage segments,
//! page cache or media). Out-of-scope surfaces (backups, archived WAL
//! segments, exports, parametric unlearning) are always listed — never silent.
//!
//! # Integrity vs authenticity
//!
//! `integrity.sha256` detects edits/corruption of the certificate JSON
//! (deterministic hash over the canonical body). It does **not** authenticate
//! it: there is no engine key, so an actor who recomputes the hash is not
//! detected. Cryptographic signing is a `vanta-audit` decision (VER-01 §Diseño
//! deferred the external anchor the same way). The WAL side references the
//! VER-01 hash-chain (`vanta-cli verify` is the chain authority).

use crate::backend::BackendPartition;
use crate::error::{Error, Result};
use crate::sdk::serialization::{impl_sparse_index, namespace_index_key};
use crate::sdk::types::MemoryRecord;
use crate::sdk::version_history::version_prefix;
use crate::storage::engine::StorageEngine;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Purge certificate schema version (v1, VER-02).
pub const PURGE_CERTIFICATE_SCHEMA_VERSION: u32 = 1;

/// The canonical purge surfaces a valid certificate must cover exactly once
/// (H-1, review VER-02): schema validation rejects claimless or partial
/// certificates before any verification verdict is issued.
const PURGE_SURFACES: [&str; 9] = [
    "store",
    "shred",
    "vector_index",
    "vector_store",
    "derived_index",
    "text_index",
    "sparse_index",
    "version_history",
    "wal",
];

/// One line of the per-surface inventory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurfaceReport {
    /// Stable surface name (`store`, `shred`, `vector_index`, `vector_store`,
    /// `derived_index`, `text_index`, `sparse_index`, `version_history`, `wal`).
    pub surface: String,
    /// What the delete path did on this surface: `deleted`, `tombstoned`,
    /// `tombstone-recorded`, `none-needed`, `scanned`, `not-assessed`, ...
    pub action: String,
    /// Entries of the deleted record still reachable on this surface (0 = clean).
    pub residues: u64,
    /// How the verdict was reached (absence scan, batch op, receipt, limit).
    pub evidence: String,
}

/// WAL (VER-01) linkage: the certificate references the chained WAL that
/// carries the delete tombstone. Full chain validation is `vanta-cli verify`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChainEvidence {
    /// Chain scheme implemented by the WAL framing.
    pub scheme: String,
    /// Current WAL on-disk format version (`WAL_FORMAT_VERSION`).
    pub format_version: u32,
    /// Whether that format carries `prev_hash ‖ record_hash` frames.
    pub chained: bool,
    /// `referenced` (not re-verified per delete — see module docs).
    pub status: String,
    /// Command that re-verifies the chain read-only.
    pub verify_command: String,
}

/// Self-integrity of the certificate JSON (edit/corruption detection).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertificateIntegrity {
    /// Hash algorithm (`sha256`).
    pub algorithm: String,
    /// Hex digest over the canonical certificate body with this field empty.
    pub sha256: String,
}

/// A purge certificate: what was deleted, on which surfaces, with what proof.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PurgeCertificate {
    /// Schema version ([`PURGE_CERTIFICATE_SCHEMA_VERSION`]).
    pub schema_version: u32,
    /// ISO 8601 UTC timestamp (single inherently variable field).
    pub timestamp: String,
    /// Record namespace.
    pub namespace: String,
    /// Record key.
    pub key: String,
    /// Deterministic node id, decimal string (u128 is not JSON-safe).
    pub node_id: String,
    /// Why the delete happened (audit reason).
    pub reason: String,
    /// `purged` (clean) | `residues` (store gone, something remains) |
    /// `not_found` (nothing to delete; keyless surfaces still scanned).
    pub status: String,
    /// Fixed-order per-surface inventory.
    pub surfaces: Vec<SurfaceReport>,
    /// VER-01 chain reference.
    pub chain: ChainEvidence,
    /// Declared limits — always present, never silent.
    pub out_of_scope: Vec<String>,
    /// Certificate self-integrity.
    pub integrity: CertificateIntegrity,
}

/// Result of verifying a stored certificate against the live database.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PurgeCertificateVerification {
    /// Schema version of the verified certificate.
    pub schema_version: u32,
    /// Certificate namespace.
    pub namespace: String,
    /// Certificate key.
    pub key: String,
    /// Certificate status (echoed).
    pub status: String,
    /// `ok` when the integrity hash matched.
    pub integrity: String,
    /// Surfaces re-scanned against the live state.
    pub rechecked_surfaces: Vec<String>,
    /// Residues currently reachable across the re-scanned surfaces.
    pub residues_now: u64,
}

/// Declared limits of a certified purge (never silent).
fn declared_limits() -> Vec<String> {
    vec![
        "physical media: bytes may survive in storage segments, page cache, filesystem journals or SSD remap — this is a logical/physical-entry purge, not secure erase".into(),
        "backups and snapshots stored outside the live database directory are not touched".into(),
        "archived WAL segments (shipping archive) are not rewritten: the append-only log keeps history by design".into(),
        "exports are copies outside the store perimeter (WIRE-09 sandbox governs export/import)".into(),
        "audit logs: delete events (namespace/key) survive in the audit JSONL — including rotated archives (DEFAULT_AUDIT_MAX_FILES) — until retention drops them (I-2, review VER-02)".into(),
        "no parametric unlearning: embedding-model weights are never modified".into(),
    ]
}

/// Reference the VER-01 hash-chain without re-verifying it per delete.
///
/// Public since MEMG-17: `vanta-memory` erasure receipts embed the same
/// evidence (single source for scheme/format/verifier).
pub fn chain_evidence() -> ChainEvidence {
    let format_version = crate::wal::WAL_FORMAT_VERSION;
    ChainEvidence {
        scheme: "sha256-prev-hash".into(),
        format_version: u32::from(format_version),
        chained: crate::wal::is_chained(format_version),
        status: "referenced".into(),
        verify_command: "vanta-cli verify".into(),
    }
}

fn surface(surface: &str, action: &str, residues: u64, evidence: &str) -> SurfaceReport {
    SurfaceReport {
        surface: surface.into(),
        action: action.into(),
        residues,
        evidence: evidence.into(),
    }
}

/// Count how many `Delete` ops from `ops` still have their key present.
fn delete_op_residues(
    engine: &StorageEngine,
    ops: &[crate::backend::BackendWriteOp],
) -> Result<u64> {
    let mut residues = 0u64;
    for op in ops {
        if let crate::backend::BackendWriteOp::Delete { partition, key } = op {
            if engine.get_from_partition(*partition, key)?.is_some() {
                residues += 1;
            }
        }
    }
    Ok(residues)
}

/// Inventory for a record that WAS deleted by this path (full evidence).
fn swept_surfaces(engine: &StorageEngine, record: &MemoryRecord) -> Result<Vec<SurfaceReport>> {
    let node_id = record.node_id;
    let store_residues = u64::from(
        engine
            .get_from_partition(BackendPartition::Default, &node_id.to_le_bytes())?
            .is_some(),
    );
    let shred_residues =
        u64::from(crate::shred::ShreddedRowStore::get(node_id, &*engine.backend)?.is_some());
    let vector_index_residues = u64::from(engine.hnsw.load().storage_offset_of(node_id).is_some());

    let derived_ops = crate::sdk::Embedded::derived_delete_ops(record)?;
    let derived_residues = delete_op_residues(engine, &derived_ops)?;

    // Key-only ops: postings + doc-stats entries for the deleted record.
    // (`text_index_ops_for_replace` is NOT reused here — it re-derives and
    // re-decrements term/namespace stats, which the delete already applied;
    // running it again would go negative. The certificate only needs the
    // exact posting/stats keys to prove they are gone.)
    let mut text_ops =
        crate::text_index::posting_delete_ops(&record.namespace, &record.key, &record.payload);
    text_ops.push(crate::text_index::doc_stats_delete_op(
        &record.namespace,
        &record.key,
    ));
    let text_residues = delete_op_residues(engine, &text_ops)?;

    let sparse_ops = impl_sparse_index::sparse_index_ops_for_replace(Some(record), None)?;
    let sparse_residues = delete_op_residues(engine, &sparse_ops)?;

    let version_residues = engine
        .scan_partition_prefix(
            BackendPartition::Versions,
            &version_prefix(&record.namespace, &record.key),
        )?
        .len() as u64;

    Ok(vec![
        surface(
            "store",
            "deleted",
            store_residues,
            "point-read of backend partition Default is absent after the delete",
        ),
        surface(
            "shred",
            "deleted",
            shred_residues,
            "InternalMetadata key shred::<node_id> absent (JSON Shredding column store)",
        ),
        surface(
            "vector_index",
            "deleted",
            vector_index_residues,
            "HNSW graph has no entry for the node",
        ),
        surface(
            "vector_store",
            if record.vector.is_some() {
                "tombstoned"
            } else {
                "none-needed"
            },
            0,
            "vector header FLAG_TOMBSTONE set by the delete path; physical bytes remain until segment compaction (declared limit)",
        ),
        surface(
            "derived_index",
            "deleted",
            derived_residues,
            "NamespaceIndex + PayloadIndex keys rebuilt from the deleted record are absent",
        ),
        surface(
            "text_index",
            "deleted",
            text_residues,
            "posting keys for the record terms are absent (term/namespace stats recomputed by the replace)",
        ),
        surface(
            "sparse_index",
            if record.sparse_vector.is_some() {
                "deleted"
            } else {
                "none-needed"
            },
            sparse_residues,
            "sparse posting keys rebuilt from the deleted record are absent",
        ),
        surface(
            "version_history",
            "deleted",
            version_residues,
            "Versions-partition prefix scan for the key is empty",
        ),
        surface(
            "wal",
            "tombstone-recorded",
            0,
            "WalRecord::Delete appended before store I/O (append-only history keeps the trace by design)",
        ),
    ])
}

/// Inventory for a key that was NOT found (scans only what is reconstructible).
fn unscanned_surfaces(
    engine: &StorageEngine,
    namespace: &str,
    key: &str,
    node_id: u128,
) -> Result<Vec<SurfaceReport>> {
    let store_residues = u64::from(
        engine
            .get_from_partition(BackendPartition::Default, &node_id.to_le_bytes())?
            .is_some(),
    );
    let shred_residues =
        u64::from(crate::shred::ShreddedRowStore::get(node_id, &*engine.backend)?.is_some());
    let vector_index_residues = u64::from(engine.hnsw.load().storage_offset_of(node_id).is_some());
    let version_residues = engine
        .scan_partition_prefix(BackendPartition::Versions, &version_prefix(namespace, key))?
        .len() as u64;

    let not_assessed = "record not found — the key set of this surface cannot be rebuilt; not assessed (never reported as clean)";
    Ok(vec![
        surface(
            "store",
            "scanned",
            store_residues,
            "point-read of backend partition Default",
        ),
        surface(
            "shred",
            "scanned",
            shred_residues,
            "InternalMetadata key shred::<node_id> point-read",
        ),
        surface(
            "vector_index",
            "scanned",
            vector_index_residues,
            "HNSW graph lookup",
        ),
        surface("vector_store", "not-assessed", 0, not_assessed),
        surface("derived_index", "not-assessed", 0, not_assessed),
        surface("text_index", "not-assessed", 0, not_assessed),
        surface("sparse_index", "not-assessed", 0, not_assessed),
        surface(
            "version_history",
            "scanned",
            version_residues,
            "Versions-partition prefix scan for the key",
        ),
        surface(
            "wal",
            "not-recorded",
            0,
            "no delete occurred for this key in this call",
        ),
    ])
}

/// Canonical bytes used for the integrity hash: the certificate body with the
/// `integrity.sha256` field emptied (so the digest is stable across re-reads).
fn canonical_bytes(cert: &PurgeCertificate) -> Result<Vec<u8>> {
    let mut canonical = cert.clone();
    canonical.integrity.sha256.clear();
    serde_json::to_vec(&canonical).map_err(Error::serialization)
}

fn hex_lower(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Structural validation of a certificate (H-1): a well-formed certificate
/// must carry a known `status`, a non-empty `out_of_scope` and **every
/// canonical surface exactly once** — no omissions, duplicates or unknowns.
///
/// This is what makes verification non-claim-driven: a certificate that omits
/// the surface holding live residues (even with a recomputed integrity hash)
/// is rejected before any verdict is issued.
fn validate_schema(cert: &PurgeCertificate) -> Result<()> {
    const VALID_STATUS: [&str; 3] = ["purged", "residues", "not_found"];
    if !VALID_STATUS.contains(&cert.status.as_str()) {
        return Err(Error::Validation {
            field: "certificate.status".into(),
            reason: format!(
                "unknown status '{}' (expected one of {:?})",
                cert.status, VALID_STATUS
            ),
        });
    }
    if cert.out_of_scope.is_empty() {
        return Err(Error::Validation {
            field: "certificate.out_of_scope".into(),
            reason: "declared limits must not be empty".into(),
        });
    }
    for canonical in PURGE_SURFACES {
        let occurrences = cert
            .surfaces
            .iter()
            .filter(|s| s.surface == canonical)
            .count();
        if occurrences != 1 {
            return Err(Error::Validation {
                field: "certificate.surfaces".into(),
                reason: format!(
                    "surface '{canonical}' must appear exactly once (found {occurrences})"
                ),
            });
        }
    }
    if let Some(unknown) = cert
        .surfaces
        .iter()
        .find(|s| !PURGE_SURFACES.contains(&s.surface.as_str()))
    {
        return Err(Error::Validation {
            field: "certificate.surfaces".into(),
            reason: format!("unknown surface '{}'", unknown.surface),
        });
    }
    if cert.surfaces.len() != PURGE_SURFACES.len() {
        return Err(Error::Validation {
            field: "certificate.surfaces".into(),
            reason: format!(
                "expected {} canonical surfaces, found {}",
                PURGE_SURFACES.len(),
                cert.surfaces.len()
            ),
        });
    }
    Ok(())
}

fn finalize(mut cert: PurgeCertificate) -> Result<PurgeCertificate> {
    // The producer must never emit a structurally invalid certificate.
    validate_schema(&cert)?;
    let digest = Sha256::digest(canonical_bytes(&cert)?);
    cert.integrity.sha256 = hex_lower(&digest);
    Ok(cert)
}

/// Build a purge certificate for `(namespace, key)`.
///
/// `record` is `Some` when the record was found and the delete path swept it;
/// `None` when nothing was found (surfaces are still scanned/inventoried, and
/// record-dependent ones are marked `not-assessed` — never silently clean).
pub(crate) fn build_certificate(
    engine: &StorageEngine,
    record: Option<&MemoryRecord>,
    namespace: &str,
    key: &str,
    node_id: u128,
    reason: &str,
) -> Result<PurgeCertificate> {
    let surfaces = match record {
        Some(record) => swept_surfaces(engine, record)?,
        None => unscanned_surfaces(engine, namespace, key, node_id)?,
    };
    let residues_total: u64 = surfaces.iter().map(|s| s.residues).sum();
    let status = match (record.is_some(), residues_total) {
        (false, _) => "not_found",
        (true, 0) => "purged",
        (true, _) => "residues",
    };
    let cert = PurgeCertificate {
        schema_version: PURGE_CERTIFICATE_SCHEMA_VERSION,
        timestamp: crate::audit::now_iso(),
        namespace: namespace.to_string(),
        key: key.to_string(),
        node_id: node_id.to_string(),
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
    finalize(cert)
}

/// Whether the certificate claims `surface` has zero residues.
fn cert_claims_zero(cert: &PurgeCertificate, surface: &str) -> bool {
    cert.surfaces
        .iter()
        .find(|s| s.surface == surface)
        .is_some_and(|s| s.residues == 0)
}

/// Record a re-scan result for `surface`; flag it when the certificate
/// claimed it clean but it currently holds an entry (residues reappeared).
fn recheck_surface(
    cert: &PurgeCertificate,
    surface: &str,
    present: bool,
    rechecked: &mut Vec<String>,
    residues_now: &mut u64,
    broken: &mut Vec<String>,
) {
    rechecked.push(surface.to_string());
    if present {
        *residues_now += 1;
        if cert_claims_zero(cert, surface) {
            broken.push(surface.to_string());
        }
    }
}

/// Verify a stored certificate: schema + integrity hash first, then re-scan
/// the surfaces that are re-checkable without the deleted record (`store`,
/// `shred`, `vector_index`, `version_history`, exact `NamespaceIndex` key).
///
/// Verification is **not claim-driven** (H-1): before any verdict, the
/// certificate must be structurally valid ([`validate_schema`]) — status in
/// the known set, non-empty declared limits and all canonical surfaces present
/// exactly once. A claimless/partial certificate is rejected even when its
/// integrity hash was recomputed.
///
/// A surface that the certificate claimed clean (`residues == 0`) but that now
/// holds an entry fails the verification (residues reappeared), and any
/// live residues on the re-checked surfaces fail the verification unless the
/// certificate is a `not_found` report. Record-derived surfaces
/// (payload/text/sparse) are covered by the integrity hash only — their key
/// set cannot be rebuilt after deletion (documented limit).
pub(crate) fn verify_certificate(
    engine: &StorageEngine,
    cert: &PurgeCertificate,
) -> Result<PurgeCertificateVerification> {
    if cert.schema_version != PURGE_CERTIFICATE_SCHEMA_VERSION {
        return Err(Error::Validation {
            field: "certificate.schema_version".into(),
            reason: format!(
                "unsupported certificate schema version {} (expected {})",
                cert.schema_version, PURGE_CERTIFICATE_SCHEMA_VERSION
            ),
        });
    }
    let expected = hex_lower(&Sha256::digest(canonical_bytes(cert)?));
    if cert.integrity.algorithm != "sha256" || cert.integrity.sha256 != expected {
        return Err(Error::Validation {
            field: "certificate.integrity".into(),
            reason: "integrity hash mismatch — the certificate was edited or corrupted".into(),
        });
    }
    validate_schema(cert)?;

    let node_id: u128 = cert.node_id.parse().map_err(|_| Error::Validation {
        field: "certificate.node_id".into(),
        reason: "node_id is not a valid u128".into(),
    })?;

    let mut rechecked: Vec<String> = Vec::new();
    let mut residues_now = 0u64;
    let mut broken: Vec<String> = Vec::new();

    let store_present = engine
        .get_from_partition(BackendPartition::Default, &node_id.to_le_bytes())?
        .is_some();
    recheck_surface(
        cert,
        "store",
        store_present,
        &mut rechecked,
        &mut residues_now,
        &mut broken,
    );

    let shred_present = crate::shred::ShreddedRowStore::get(node_id, &*engine.backend)?.is_some();
    recheck_surface(
        cert,
        "shred",
        shred_present,
        &mut rechecked,
        &mut residues_now,
        &mut broken,
    );

    let vector_present = engine.hnsw.load().storage_offset_of(node_id).is_some();
    recheck_surface(
        cert,
        "vector_index",
        vector_present,
        &mut rechecked,
        &mut residues_now,
        &mut broken,
    );

    let versions = engine
        .scan_partition_prefix(
            BackendPartition::Versions,
            &version_prefix(&cert.namespace, &cert.key),
        )?
        .len() as u64;
    rechecked.push("version_history".to_string());
    residues_now += versions;
    if versions > 0 && cert_claims_zero(cert, "version_history") {
        broken.push("version_history".to_string());
    }

    let namespace_present = engine
        .get_from_partition(
            BackendPartition::NamespaceIndex,
            &namespace_index_key(&cert.namespace, &cert.key),
        )?
        .is_some();
    recheck_surface(
        cert,
        "derived_index",
        namespace_present,
        &mut rechecked,
        &mut residues_now,
        &mut broken,
    );

    if !broken.is_empty() {
        return Err(Error::Validation {
            field: "certificate.residues".into(),
            reason: format!(
                "{residues_now} residue(s) reappeared after the certificate was issued (surfaces: {})",
                broken.join(", ")
            ),
        });
    }
    // H-1: residues-aware verdict. With the canonical surfaces validated above,
    // any live residues on the re-checkable surfaces reject the certificate
    // unless it is a `not_found` report (which attests no purge at all).
    if residues_now > 0 && cert.status != "not_found" {
        return Err(Error::Validation {
            field: "certificate.residues".into(),
            reason: format!(
                "{residues_now} residue(s) found on the re-checkable surfaces; certificate status '{}' does not attest a clean purge",
                cert.status
            ),
        });
    }

    Ok(PurgeCertificateVerification {
        schema_version: cert.schema_version,
        namespace: cert.namespace.clone(),
        key: cert.key.clone(),
        status: cert.status.clone(),
        integrity: "ok".into(),
        rechecked_surfaces: rechecked,
        residues_now,
    })
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;

    fn minimal_certificate() -> PurgeCertificate {
        PurgeCertificate {
            schema_version: PURGE_CERTIFICATE_SCHEMA_VERSION,
            timestamp: "2026-09-29T00:00:00Z".into(),
            namespace: "ns".into(),
            key: "k".into(),
            node_id: "42".into(),
            reason: "test".into(),
            status: "purged".into(),
            surfaces: PURGE_SURFACES
                .iter()
                .map(|s| surface(s, "deleted", 0, "absent"))
                .collect(),
            chain: chain_evidence(),
            out_of_scope: declared_limits(),
            integrity: CertificateIntegrity {
                algorithm: "sha256".into(),
                sha256: String::new(),
            },
        }
    }

    #[test]
    fn finalize_computes_stable_hex_digest() {
        let cert = finalize(minimal_certificate()).unwrap();
        assert_eq!(cert.integrity.sha256.len(), 64);
        assert!(cert.integrity.sha256.chars().all(|c| c.is_ascii_hexdigit()));

        // Re-finalizing (hash already present) produces the same digest: the
        // canonical body empties the field before hashing.
        let again = finalize(cert.clone()).unwrap();
        assert_eq!(again.integrity.sha256, cert.integrity.sha256);
    }

    #[test]
    fn canonical_bytes_ignore_the_integrity_hash_value() {
        let cert = finalize(minimal_certificate()).unwrap();
        let mut other = cert.clone();
        other.integrity.sha256 = "deadbeef".into();
        assert_eq!(
            canonical_bytes(&cert).unwrap(),
            canonical_bytes(&other).unwrap(),
            "the digest must not depend on the stored hash field"
        );
    }

    #[test]
    fn chain_evidence_references_the_wal_chain() {
        let chain = chain_evidence();
        assert_eq!(chain.scheme, "sha256-prev-hash");
        assert!(chain.chained, "VER-01 chain format must be referenced");
        assert_eq!(
            chain.format_version,
            u32::from(crate::wal::WAL_FORMAT_VERSION)
        );
        assert!(chain.verify_command.contains("verify"));
    }

    #[test]
    fn declared_limits_are_never_empty() {
        let limits = declared_limits();
        assert!(limits.iter().any(|l| l.contains("parametric unlearning")));
        assert!(limits.iter().any(|l| l.contains("backups")));
        assert!(
            limits.iter().any(|l| l.contains("audit logs")),
            "I-2: audit-log retention must be a declared limit"
        );
    }

    // ── H-1: schema validation (no claim-driven verification) ──

    #[test]
    fn validate_schema_accepts_a_well_formed_certificate() {
        validate_schema(&minimal_certificate()).expect("canonical certificate is valid");
    }

    #[test]
    fn validate_schema_rejects_missing_surface() {
        let mut cert = minimal_certificate();
        cert.surfaces.retain(|s| s.surface != "shred");
        let err = validate_schema(&cert).expect_err("missing surface must be rejected");
        assert!(err.to_string().contains("shred"), "got: {err}");
    }

    #[test]
    fn validate_schema_rejects_duplicated_surface() {
        let mut cert = minimal_certificate();
        cert.surfaces.push(surface("store", "deleted", 0, "dup"));
        let err = validate_schema(&cert).expect_err("duplicate surface must be rejected");
        assert!(err.to_string().contains("exactly once"), "got: {err}");
    }

    #[test]
    fn validate_schema_rejects_unknown_surface_and_status() {
        let mut cert = minimal_certificate();
        cert.surfaces[0] = surface("mystery", "deleted", 0, "?");
        assert!(validate_schema(&cert).is_err(), "unknown surface");

        let mut cert = minimal_certificate();
        cert.surfaces = PURGE_SURFACES
            .iter()
            .map(|s| surface(s, "deleted", 0, "absent"))
            .collect();
        cert.status = "probably_fine".into();
        let err = validate_schema(&cert).expect_err("unknown status must be rejected");
        assert!(err.to_string().contains("status"), "got: {err}");
    }

    #[test]
    fn validate_schema_rejects_empty_declared_limits() {
        let mut cert = minimal_certificate();
        cert.out_of_scope.clear();
        let err = validate_schema(&cert).expect_err("empty out_of_scope must be rejected");
        assert!(err.to_string().contains("out_of_scope"), "got: {err}");
    }

    #[test]
    fn finalize_refuses_to_emit_a_structurally_invalid_certificate() {
        let mut cert = minimal_certificate();
        cert.surfaces.clear();
        assert!(
            finalize(cert).is_err(),
            "the producer must never emit a claimless certificate"
        );
    }
}
