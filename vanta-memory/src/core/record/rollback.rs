//! MEMG-17 (pieza a): rollback semántico de un registro L1 a una versión
//! retenida, con linaje append-only.
//!
//! Reuses the MEMG-13 version history (core `Embedded::versions`/`get_version`)
//! and the canonical L1 write path ([`put_record`]): a rollback is **itself a
//! new version**, so `read_record_versions` shows `v1..vN, vN+1 = restored`
//! and nothing in the history is rewritten or dropped.
//!
//! # What it reverts / what it does NOT (declared, never silent)
//!
//! - **Reverts:** the target version's payload fields (content, type,
//!   priority, scene, metadata, `superseded_by`, heat, ...) — the exact store
//!   delta is [`RollbackReport::changes`] (`live -> restored`).
//! - **Does NOT revert:** bookkeeping advances to the rollback write
//!   (`version`, `updated_at`, `timestamps`); newer versions are retained;
//!   cross-record state is not re-evaluated (another record's supersession of
//!   this one lives in *that* record's payload — restored verbatim here);
//!   deletes are not resurrected (`delete` purges the retained history — the
//!   snapshot restore is the data-only path, FIND-287).
//!
//! **Audit surface:** like [`crate::core::record::l1_reader::read_record_versions`]
//! this is an explicit admin/audit operation — it does not apply the recall
//! quarantine gate (SCH-05). The write goes through the canonical L1 write
//! path, never through recall.

use std::collections::BTreeSet;

use vantadb::error::Error;
use vantadb::sdk::Embedded;

use crate::core::prompts::l1_extraction::epoch_ms_to_rfc3339;
use crate::core::record::l1_reader::{
    diff_records, l1_namespace, read_record, read_record_version, read_record_versions,
    RecordFieldChange,
};
use crate::core::record::l1_writer::{put_record, L1Error};

/// Outcome of a semantic rollback (MEMG-17).
///
/// The report is the lineage record of the operation: which version was live,
/// which version was requested, which version the rollback appended, the
/// field-level store delta, and the declared out-of-scope limits (never
/// silent).
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackReport {
    /// Record id as passed by the caller.
    pub record_id: String,
    /// Session whose `l1/` namespace holds the record.
    pub session_key: String,
    /// Core storage version the record was at before the rollback (live tip).
    pub from_version: u64,
    /// Core storage version the caller asked to restore.
    pub to_version: u64,
    /// Core storage version written by the rollback (new live tip).
    pub new_version: u64,
    /// Field-level store delta (`diff_records(live, restored)`) — every
    /// top-level payload field whose value changed, ordered by field name.
    pub changes: Vec<RecordFieldChange>,
    /// Declared limits: what this operation does **not** revert.
    pub out_of_scope: Vec<String>,
}

/// Roll an L1 record back to a retained version.
///
/// Reads the live record and the requested `target_version`, writes the
/// target payload back through the canonical L1 write path (`version` and
/// `updated_at` advance to `now_ms`; the `timestamps` trail gains `now`),
/// and returns the [`RollbackReport`].
///
/// # Errors
///
/// - [`Error::NotFound`] when the record does not exist (never written, or
///   deleted — the delete purges the retained history) or when
///   `target_version` is not retained.
/// - [`Error::Backend`] when the record exists but its version history is
///   unavailable (best-effort history gap — the live tip version cannot be
///   established; the store is left untouched).
pub fn rollback_record(
    db: &Embedded,
    session_key: &str,
    record_id: &str,
    target_version: u64,
    now_ms: u64,
) -> Result<RollbackReport, L1Error> {
    let live = read_record(db, session_key, record_id)?.ok_or_else(|| Error::NotFound {
        kind: "l1 record".into(),
        id: format!("{session_key}/{record_id}"),
    })?;
    let target =
        read_record_version(db, session_key, record_id, target_version)?.ok_or_else(|| {
            Error::NotFound {
                kind: "l1 record version".into(),
                id: format!("{session_key}/{record_id} v{target_version}"),
            }
        })?;

    // Live tip core version from the retained history (the record exists, so
    // an empty history means the best-effort snapshot write was lost).
    let from_version = read_record_versions(db, session_key, record_id)?
        .last()
        .map(|v| v.version)
        .ok_or_else(|| {
            Error::backend_error(format!(
                "version history unavailable for {session_key}/{record_id} \
                 (rollback needs the live tip version)"
            ))
        })?;

    // Restored payload: target verbatim + bookkeeping advanced (declared).
    let mut restored = target.record;
    restored.version = live.version.saturating_add(1);
    restored.updated_at = epoch_ms_to_rfc3339(now_ms);
    restored.timestamps = restored_timestamps(&restored.timestamps, &restored.updated_at);

    // The vector travels on the node, not inside the payload (put_record
    // contract) — strip it from the write copy only, so `changes` still
    // reports the restored vector state.
    let mut for_write = restored.clone();
    let vector = for_write.vector.take();
    let written = put_record(db, &l1_namespace(session_key), &for_write, vector)?;

    let changes = diff_records(&live, &restored)?;
    Ok(RollbackReport {
        record_id: record_id.to_string(),
        session_key: session_key.to_string(),
        from_version,
        to_version: target_version,
        new_version: written.version,
        changes,
        out_of_scope: declared_scope(),
    })
}

/// The restored `timestamps` trail: target entries plus the rollback write
/// time, deduped and sorted (same shape the merge path builds).
fn restored_timestamps(base: &[String], now_iso: &str) -> Vec<String> {
    let mut set: BTreeSet<String> = base.iter().cloned().collect();
    set.insert(now_iso.to_string());
    set.into_iter().collect()
}

/// Declared limits of a semantic rollback — always present, never silent.
fn declared_scope() -> Vec<String> {
    vec![
        "append-only lineage: the rollback writes a NEW version; newer versions are retained (audit)".into(),
        "bookkeeping advances to the rollback write: `version` and `updated_at` are not restored to the target values — see `changes` for the exact store delta".into(),
        "no cross-record cascade: supersession/contradiction state of other records is not re-evaluated; the target's `superseded_by` field is restored verbatim".into(),
        "deleted records are not resurrected: `delete` purges the retained version history (snapshot restore is the data-only path — FIND-287)".into(),
        "audit surfaces (generation log, audit JSONL, WAL frames) are append-only and not rewritten".into(),
    ]
}
