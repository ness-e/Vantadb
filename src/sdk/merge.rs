//! Multi-writer merge policy (MEMG-05 v1): explicit last-write-wins over a
//! deterministic total order, with conflict detection.
//!
//! Scenario fixed in ADR-0055: several writers (devices/agents) edit the same
//! `(namespace, key)` record and their versions converge in one store through
//! the merge path ([`Embedded::merge_record`](crate::sdk::Embedded::merge_record)).
//! The declared policy is **explicit LWW**:
//!
//! 1. **Order key:** `(updated_at_ms, content bytes)`. The writer-side
//!    timestamp travels in the record (same as import); ties on
//!    `updated_at_ms` are broken by the lexicographic order of the canonical
//!    postcard encoding of `(payload, metadata)` — deterministic and identical
//!    on every replica, never arrival-order dependent.
//! 2. **Conflict detection:** equal `updated_at_ms` + different content is a
//!    detected conflict ([`MergeResult::conflict`]); different timestamps are a
//!    declared LWW update, not a conflict.
//!
//! Field scope: the conflict fingerprint ("content") is exactly
//! `payload + metadata`. Every other field — validity window (`valid_at_ms` /
//! `invalid_at_ms`), TTL, confidence/class, `derived_from`, `vector` /
//! `sparse_vector`, `last_validated_at_ms`, quarantine, `superseded_by`, and
//! the store-local `version` / `created_at_ms` — is NOT part of the
//! fingerprint. On a stored merge the incoming record is written as the whole
//! unit (its fields replace the stored ones), so a transport must carry
//! complete records: a missing derived field (e.g. `vector = None`) replaces
//! the stored one, same as the import path.
//!
//! Declared limits (see ADR-0055): wall-clock skew between writers can invert
//! the true causal order (upgrade path: vector clocks / CRDT-lite over
//! update operations); the equal-time tie-break is an arbitrary-but-convergent
//! byte order, not a semantic preference; local bookkeeping (`version`,
//! first-seen `created_at_ms`) is store-local and not convergent; `put` racing
//! `merge_record` on the same key is outside v1 coverage; the merge lock is
//! per-handle (same scope as `supersede_lock`), so one route per handle.

use crate::error::{Error, Result};
use crate::sdk::types::MemoryRecord;

/// How an incoming multi-writer record was resolved against the stored one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum MergeOutcome {
    /// No current record — the incoming record was stored.
    Inserted,
    /// The incoming record won the declared order — it replaced the stored one.
    Updated,
    /// The stored record is newer (or won the equal-time tie-break) — the
    /// incoming record was NOT stored.
    StaleRejected,
    /// The incoming content is identical to the stored record (under the
    /// `payload + metadata` fingerprint) with an older or equal clock — no write.
    AlreadyCurrent,
}

/// Non-silent decision returned by every merge
/// ([`Embedded::merge_record`](crate::sdk::Embedded::merge_record)): callers
/// always learn what won, what was rejected, or that nothing changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct MergeResult {
    /// What happened to the incoming record.
    pub outcome: MergeOutcome,
    /// `true` when both versions existed, differed and shared the same
    /// `updated_at_ms` — a detected conflict resolved by the content tie-break.
    pub conflict: bool,
    /// `updated_at_ms` of the version stored after the merge.
    pub winner_updated_at_ms: u64,
}

/// Internal policy decision (pure; see [`resolve_merge`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MergeDecision {
    /// Incoming must be stored (fresh insert, or it won the declared order).
    Store { conflict: bool },
    /// The stored record stays (incoming is stale or lost the tie-break).
    KeepExisting { conflict: bool },
    /// Content identical — nothing to write.
    Unchanged,
}

/// Canonical bytes of the record's *content* (payload + metadata). This is the
/// ONLY conflict fingerprint: every other field (validity window, TTL,
/// confidence, lineage, vectors, quarantine, `version`/`created_at_ms`) is
/// store-local or transport-carried data, not part of the claim comparison.
pub(crate) fn content_bytes(record: &MemoryRecord) -> Result<Vec<u8>> {
    postcard::to_allocvec(&(record.payload.as_str(), &record.metadata))
        .map_err(Error::serialization)
}

/// Pure merge policy: decides what to do with `incoming` given `existing`.
///
/// Total order: `(updated_at_ms, content bytes)` — lexicographic, with
/// `content_bytes(incoming) > content_bytes(existing)` winning an exact-time
/// tie. Equal content with an older or equal clock is a no-op; a newer clock
/// advances the order key. See module docs.
pub(crate) fn resolve_merge(
    existing: Option<&MemoryRecord>,
    incoming: &MemoryRecord,
) -> Result<MergeDecision> {
    let Some(existing) = existing else {
        return Ok(MergeDecision::Store { conflict: false });
    };
    let incoming_bytes = content_bytes(incoming)?;
    let existing_bytes = content_bytes(existing)?;
    if incoming_bytes == existing_bytes {
        // Identical content still advances the order key when the incoming
        // clock is newer: LWW convergence needs the stored `updated_at_ms` to
        // be the max of the write set, not the first-arrival value. Without
        // this, two arrival orders of the same write set leave different
        // clocks — and cascade into different winners for later writes.
        return Ok(if incoming.updated_at_ms > existing.updated_at_ms {
            MergeDecision::Store { conflict: false }
        } else {
            MergeDecision::Unchanged
        });
    }
    Ok(match incoming.updated_at_ms.cmp(&existing.updated_at_ms) {
        std::cmp::Ordering::Greater => MergeDecision::Store { conflict: false },
        std::cmp::Ordering::Less => MergeDecision::KeepExisting { conflict: false },
        std::cmp::Ordering::Equal => {
            // Same logical time + different content: a detected conflict. The
            // tie-break (greater canonical bytes wins) is arbitrary but
            // deterministic and identical on every replica — convergence
            // never depends on arrival order.
            if incoming_bytes > existing_bytes {
                MergeDecision::Store { conflict: true }
            } else {
                MergeDecision::KeepExisting { conflict: true }
            }
        }
    })
}

#[cfg(test)]
#[path = "merge_tests.rs"]
mod tests;
