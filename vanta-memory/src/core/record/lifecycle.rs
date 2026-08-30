//! MEM-60 — Lifecycle: heat, decay, contradiction provenance.
//!
//! Three primitives on `MemoryRecord`:
//!
//! - [`bump_heat`]: increment heat on every successful read/access (signal of
//!   usefulness). Saturating; never overflows.
//! - [`decay_heat`]: halve heat (shift right) per maintenance pass. After
//!   enough passes heat hits 0 — that record is a candidate for pruning
//!   (the prune decision lives in the periodic maintenance job, not here).
//! - [`mark_contradiction`]: set `superseded_by` on the OLD record to the new
//!   key. The old record is **preserved** (provenance chain) — we never
//!   silently delete, only invalidate trackably.
//!
//! Ponytail: simple saturating arithmetic, no float decay curve, no async, no
//! LLM. Contradiction detection here is caller-supplied (the writer sees the
//! dedup signal and calls this with both keys); heuristic detection belongs
//! to the L1 dedup pipeline (out of scope for MEM-60 L1).
//!
//! Audit log: every `mark_contradiction` emits a `tracing::info!` event with
//! the namespace + old_key + new_key. The full audit-log persistence layer is
//! a follow-up (matches the plan's "audit log" risk-register entry).

use serde::{Deserialize, Serialize};
use tracing::info;

use crate::core::abstractions::MemoryRecord;

/// Heat value carried by every L1 record (MEM-60).
///
/// `u32` matches the `SceneMeta.heat` wire for consistency. Default
/// `default_heat() = 0` so records written before MEM-60 parse as cold.
pub const DEFAULT_HEAT: u32 = 0;

/// Heat value at or below which a record is prune-eligible.
///
/// Caller (maintenance job) decides what to do — `lifecycle` only signals
/// eligibility. Chosen as 1 because after one decay pass, a record with
/// `heat = 1` becomes `heat = 0`; below that point the record has not been
/// accessed since creation or its last decay round.
pub const PRUNE_HEAT_THRESHOLD: u32 = 1;

/// Outcome of [`mark_contradiction`] — returned to the caller so the
/// maintenance log can record what happened (in addition to the `tracing`
/// event the function itself emits).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContradictionProvenance {
    /// Namespace the old record lived under.
    pub namespace: String,
    /// Key of the record that was invalidated.
    pub old_key: String,
    /// Key of the new record that supersedes it.
    pub new_key: String,
    /// Wall-clock millis when the contradiction was recorded.
    pub recorded_at_ms: u64,
}

/// Bump heat by `n` on access (saturating).
///
/// `n` defaults to 1 — one successful read = one heat bump. Callers can
/// pass higher values for batch reads, but the saturating add keeps the
/// field bounded.
pub fn bump_heat(record: &mut MemoryRecord, n: u32) {
    record.heat = record.heat.saturating_add(n.max(1));
    record.updated_at = now_iso8601();
}

/// Decay heat by a single pass — equivalent to `heat / 2` (shift right).
///
/// Returns the new heat value. When the returned value is `0` (or below
/// `PRUNE_HEAT_THRESHOLD`), the caller may consider the record
/// prune-eligible.
pub fn decay_heat(record: &mut MemoryRecord) -> u32 {
    record.heat >>= 1;
    record.updated_at = now_iso8601();
    record.heat
}

/// Mark the OLD record as contradicted by the NEW record.
///
/// The old record is **not** deleted. `superseded_by` is set to `new_key`,
/// preserving the audit chain (any reader can trace which new record
/// invalidated this one). The caller is responsible for actually persisting
/// the updated old record.
///
/// Returns the [`ContradictionProvenance`] for the caller's audit log.
pub fn mark_contradiction(
    old: &mut MemoryRecord,
    new_key: impl Into<String>,
    now_ms: u64,
) -> ContradictionProvenance {
    let new_key = new_key.into();
    old.superseded_by = Some(new_key.clone());
    old.updated_at = now_iso8601();
    let provenance = ContradictionProvenance {
        namespace: namespace_of(old),
        old_key: old.id.clone(),
        new_key,
        recorded_at_ms: now_ms,
    };
    // Audit log: tracing event (persistent audit log is follow-up).
    info!(
        namespace = %provenance.namespace,
        old_key = %provenance.old_key,
        new_key = %provenance.new_key,
        "contradiction: old record invalidated by new"
    );
    provenance
}

/// `true` when heat has decayed to or below the prune threshold.
pub fn is_prune_eligible(record: &MemoryRecord) -> bool {
    record.heat <= PRUNE_HEAT_THRESHOLD
}

// ── helpers ──

/// ISO-8601 wall-clock now (UTC, millisecond precision). Format is fixed-
/// width so lexicographic order equals chronological order (parity with
/// `SceneMeta.created`/`updated`).
///
/// Ponytail: uses only `std::time::SystemTime` — avoids adding a `chrono`
/// dep to `vanta-memory/Cargo.toml` (which is intentionally lean).
fn now_iso8601() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    millis_to_iso8601(now_ms)
}

/// Format a unix-ms timestamp as `YYYY-MM-DDTHH:MM:SS.sssZ`. Pure math, no
/// dependency on a date-time crate. Supports the range 1970-01-01 →
/// 9999-12-31 (the wire range used across the crate).
fn millis_to_iso8601(ms: u64) -> String {
    // Days from 1970-01-01. 86400000 = 24*60*60*1000.
    let total_seconds = ms / 1000;
    let millis = ms % 1000;
    let days = total_seconds / 86400;
    let secs_of_day = total_seconds % 86400;

    let hour = (secs_of_day / 3600) as u8;
    let minute = ((secs_of_day % 3600) / 60) as u8;
    let second = (secs_of_day % 60) as u8;

    // Gregorian date from days-since-epoch (Howard Hinnant's algorithm).
    let z = days as i64 + 719468;
    let era = if z >= 0 {
        z / 146097
    } else {
        (z - 146096) / 146097
    };
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u8;
    let m = if mp < 10 {
        (mp + 3) as u8
    } else {
        (mp - 9) as u8
    };
    let year = if m <= 2 { y + 1 } else { y };

    format!(
        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{millis:03}Z",
        year = year,
        month = m,
        day = d,
        hour = hour,
        minute = minute,
        second = second,
        millis = millis,
    )
}

/// Namespace derivation: prefer the record's `session_key` (L1 wire carries
/// the session scope). Falls back to a placeholder if missing.
fn namespace_of(record: &MemoryRecord) -> String {
    if !record.session_key.is_empty() {
        format!("l1/{}", record.session_key)
    } else {
        "l1/unknown".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::abstractions::{MemoryType, SceneSegment};

    fn fixture() -> MemoryRecord {
        MemoryRecord {
            id: "m1".into(),
            content: "user prefers dark mode".into(),
            memory_type: MemoryType::Persona,
            priority: 80,
            scene_name: "ui-setup".into(),
            source_message_ids: vec![],
            metadata: serde_json::Value::Null,
            timestamps: vec![],
            created_at: "2026-08-20T10:00:00.000Z".into(),
            updated_at: "2026-08-20T10:00:00.000Z".into(),
            version: 1,
            session_key: "sess-1".into(),
            session_id: "".into(),
            task_id: None,
            team_id: None,
            user_id: None,
            agent_id: None,
            vector: None,
            heat: 5,
            superseded_by: None,
        }
    }

    #[test]
    fn bump_heat_increments_and_saturates() {
        let mut r = fixture();
        bump_heat(&mut r, 1);
        assert_eq!(r.heat, 6);
        bump_heat(&mut r, 3);
        assert_eq!(r.heat, 9);
        bump_heat(&mut r, u32::MAX);
        assert_eq!(r.heat, u32::MAX, "saturating");
    }

    #[test]
    fn decay_heat_halves_via_shift() {
        let mut r = fixture();
        r.heat = 100;
        let after = decay_heat(&mut r);
        assert_eq!(after, 50);
        assert_eq!(r.heat, 50);
        let after = decay_heat(&mut r);
        assert_eq!(after, 25);
        let after = decay_heat(&mut r);
        assert_eq!(after, 12, "100 -> 50 -> 25 -> 12 (floor)");
    }

    #[test]
    fn decay_heat_reaches_zero_for_low_values() {
        let mut r = fixture();
        r.heat = 1;
        let after = decay_heat(&mut r);
        assert_eq!(after, 0, "1 -> 0");
        assert!(is_prune_eligible(&r), "eligible at 0");
    }

    #[test]
    fn mark_contradiction_preserves_old_record_and_sets_pointer() {
        let mut old = fixture();
        let new_key = "m2".to_string();
        let provenance = mark_contradiction(&mut old, new_key.clone(), 1_700_000_000_000);

        assert_eq!(old.superseded_by.as_deref(), Some(new_key.as_str()));
        assert_eq!(old.id, "m1", "old id preserved");
        assert_eq!(
            old.content, "user prefers dark mode",
            "old content preserved"
        );
        assert_eq!(provenance.old_key, "m1");
        assert_eq!(provenance.new_key, "m2");
        assert_eq!(provenance.recorded_at_ms, 1_700_000_000_000);
        assert_eq!(provenance.namespace, "l1/sess-1");
    }

    #[test]
    fn default_heat_is_zero() {
        assert_eq!(DEFAULT_HEAT, 0);
    }

    #[test]
    fn prune_threshold_is_one() {
        assert_eq!(PRUNE_HEAT_THRESHOLD, 1);
        let mut r = fixture();
        r.heat = 2;
        assert!(!is_prune_eligible(&r));
        r.heat = 1;
        assert!(is_prune_eligible(&r));
        r.heat = 0;
        assert!(is_prune_eligible(&r));
    }

    // Reference-only: avoid an unused-import warning on SceneSegment.
    #[allow(dead_code)]
    fn _scene_segment_ref() -> SceneSegment {
        SceneSegment {
            scene_name: "ui".into(),
            message_ids: vec![],
            memories: vec![],
        }
    }
}
