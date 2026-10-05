//! MEM-60 — Lifecycle: heat, decay, contradiction provenance.
//! MEMG-07 — Forgetting curve: declared per-type policy + read-side decay.
//!
//! Primitives on `MemoryRecord`:
//!
//! - [`bump_heat`]: increment heat on every successful read/access (signal of
//!   usefulness). Saturating; never overflows.
//! - [`decay_heat`]: halve heat (shift right) per maintenance pass. After
//!   enough passes heat hits 0 — that record is a candidate for pruning
//!   (the prune decision lives in the periodic maintenance job, not here).
//! - [`retention_factor`] / [`effective_heat`]: MEMG-07 forgetting curve —
//!   `2^(−age/half_life)` over [`DecayPolicy`], age measured from the last
//!   touch (`updated_at`). Read-side: the curve deprioritizes (effective
//!   heat), it never mutates or deletes; [`scan_decay`] reports how much
//!   decays and what crosses the explicit prune threshold.
//! - [`mark_contradiction`]: set `superseded_by` on the OLD record to the new
//!   key. The old record is **preserved** (provenance chain) — we never
//!   silently delete, only invalidate trackably.
//!
//! Ponytail: the primitive decay stays integer saturating arithmetic (no
//! async, no LLM); the float curve lives in the declared [`DecayPolicy`]
//! layer (MEMG-07) — policy, not calibration (N-09). Contradiction detection
//! here is caller-supplied (the writer sees the dedup signal and calls this
//! with both keys); heuristic detection belongs to the L1 dedup pipeline
//! (out of scope for MEM-60 L1).
//!
//! Audit log: every `mark_contradiction` emits a `tracing::info!` event with
//! the namespace + old_key + new_key. The full audit-log persistence layer is
//! a follow-up (matches the plan's "audit log" risk-register entry).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tracing::info;

use crate::core::abstractions::{MemoryRecord, MemoryType};

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

// ── MEMG-07: forgetting curve (declared policy, read-side) ──

/// Declared per-type half-lives of the L1 forgetting curve (MEMG-07).
///
/// **Policy, not calibration.** N-09 records that no canonical decay formula
/// for semantic memory has been validated; the defaults below are round,
/// tunable values declared for [`retention_factor`]. The full Ebbinghaus
/// vision (access frequency, importance, confirmations, salience — FUT-10)
/// lands in the composite-scoring work (MEMG-21); this type carries only the
/// type/age curve.
///
/// A type absent from the map never decays ([`Self::half_life_ms`] → `None`)
/// — `Instruction` by default (strict instructions are followed until
/// contradicted, never forgotten by a curve). Override with
/// [`Self::set_half_life`] / [`Self::clear_half_life`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecayPolicy {
    half_life_ms: HashMap<MemoryType, u64>,
}

impl Default for DecayPolicy {
    /// Declared defaults (days): persona 90 · episodic 7 · work_fact 30 ·
    /// work_task 14 · work_method 60 · work_artifact 30; instruction exempt.
    fn default() -> Self {
        let days = |d: u64| d * 24 * 60 * 60 * 1000;
        Self {
            half_life_ms: [
                (MemoryType::Persona, days(90)),
                (MemoryType::Episodic, days(7)),
                (MemoryType::WorkFact, days(30)),
                (MemoryType::WorkTask, days(14)),
                (MemoryType::WorkMethod, days(60)),
                (MemoryType::WorkArtifact, days(30)),
            ]
            .into_iter()
            .collect(),
        }
    }
}

impl DecayPolicy {
    /// Half-life for `memory_type`; `None` = the type never decays.
    pub fn half_life_ms(&self, memory_type: MemoryType) -> Option<u64> {
        self.half_life_ms.get(&memory_type).copied()
    }

    /// Set (or replace) the half-life of `memory_type`.
    pub fn set_half_life(&mut self, memory_type: MemoryType, half_life_ms: u64) {
        self.half_life_ms.insert(memory_type, half_life_ms);
    }

    /// Exempt `memory_type` from decay (remove its half-life).
    pub fn clear_half_life(&mut self, memory_type: MemoryType) {
        self.half_life_ms.remove(&memory_type);
    }
}

/// Retention factor `R ∈ [0, 1]` of a record as of `now_ms` (MEMG-07).
///
/// `R = 2^(−age/half_life)` — the half-life parametrization of the
/// exponential forgetting form `R = e^(−t/S)` (Wikipedia, *Forgetting
/// curve* §Equations). `age` = time since the record's last touch
/// (`updated_at`, which [`bump_heat`] refreshes on every access — use resets
/// retention, Ebbinghaus-style). Policy, not calibration: the same source
/// notes the simple exponential "was not found to provide a good fit" — tune
/// half-lives per deployment.
///
/// Exempt type, zero/negative age (clock skew) or unparseable timestamps
/// (`updated_at` → `created_at` fallback) → `1.0`: what cannot be aged is
/// never forgotten. `half_life_ms = 0` is clamped to 1 ms (no NaN/∞).
pub fn retention_factor(record: &MemoryRecord, policy: &DecayPolicy, now_ms: u64) -> f64 {
    let Some(half_life_ms) = policy.half_life_ms(record.memory_type) else {
        return 1.0;
    };
    let Some(last_touch_ms) =
        timestamp_ms(&record.updated_at).or_else(|| timestamp_ms(&record.created_at))
    else {
        return 1.0;
    };
    let age_ms = now_ms.saturating_sub(last_touch_ms);
    let half_life_ms = half_life_ms.max(1);
    0.5f64.powf(age_ms as f64 / half_life_ms as f64)
}

/// Decayed heat of a record as of `now_ms` (MEMG-07): `heat × retention`,
/// rounded. The curve **deprioritizes** — the stored `heat` is never mutated
/// by this function and nothing is ever deleted.
pub fn effective_heat(record: &MemoryRecord, policy: &DecayPolicy, now_ms: u64) -> u32 {
    let retention = retention_factor(record, policy, now_ms);
    (record.heat as f64 * retention).round() as u32
}

/// Outcome of one forgetting-curve scan over L1 records (MEMG-07).
///
/// "How much decays / what does not expire": disjoint categories
/// (`decayed` + `unchanged` + `exempt` = `scanned`) plus the explicit-gate
/// input (`below_threshold` = *effective* heat at or below
/// [`PRUNE_HEAT_THRESHOLD`]). The curve never discards — the report only
/// informs the explicit gate.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DecayReport {
    /// Records examined.
    pub scanned: usize,
    /// Records whose effective heat fell below their stored heat.
    pub decayed: usize,
    /// Records whose type decays and whose effective heat equals their
    /// stored heat (fresh, or rounding keeps them equal).
    pub unchanged: usize,
    /// Records whose type has no half-life (never decay).
    pub exempt: usize,
    /// Records whose effective heat is at/below [`PRUNE_HEAT_THRESHOLD`].
    pub below_threshold: usize,
    /// Σ stored heat before the curve.
    pub heat_total: u64,
    /// Σ effective heat after the curve.
    pub heat_effective: u64,
}

impl DecayReport {
    /// Heat the curve would forget (`heat_total − heat_effective`).
    pub fn heat_forgotten(&self) -> u64 {
        self.heat_total.saturating_sub(self.heat_effective)
    }
}

/// Apply the forgetting curve over `records` as of `now_ms` (pure, MEMG-07).
///
/// Read-only by construction: computes the report and mutates nothing — the
/// curve deprioritizes (via [`effective_heat`]), it never purges.
pub fn scan_decay(records: &[MemoryRecord], policy: &DecayPolicy, now_ms: u64) -> DecayReport {
    let mut report = DecayReport::default();
    for record in records {
        report.scanned += 1;
        let raw = u64::from(record.heat);
        let effective = u64::from(effective_heat(record, policy, now_ms));
        report.heat_total += raw;
        report.heat_effective += effective;
        if policy.half_life_ms(record.memory_type).is_none() {
            report.exempt += 1;
        } else if effective < raw {
            report.decayed += 1;
        } else {
            report.unchanged += 1;
        }
        if effective <= u64::from(PRUNE_HEAT_THRESHOLD) {
            report.below_threshold += 1;
        }
    }
    report
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

/// Parse an RFC3339 timestamp (`2026-08-20T10:00:00.000Z`) to epoch ms.
/// `None` when unparseable — callers fall back (never forget unparseable).
fn timestamp_ms(iso: &str) -> Option<u64> {
    chrono::DateTime::parse_from_rfc3339(iso)
        .ok()
        .and_then(|dt| u64::try_from(dt.timestamp_millis()).ok())
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

    // ── MEMG-07: forgetting curve (declared policy) ──

    /// Fixed instants as RFC3339 + epoch ms (no chrono in this test crate).
    /// `T0` = 2026-01-01T00:00:00.000Z, `NOW` = 2026-01-31T00:00:00.000Z.
    const T0: &str = "2026-01-01T00:00:00.000Z";
    const T0_MS: u64 = 1_767_225_600_000;
    const NOW_MS: u64 = 1_769_817_600_000;
    const NOW_ISO: &str = "2026-01-31T00:00:00.000Z";
    const SEVEN_DAYS_MS: u64 = 604_800_000;

    /// Fixture with explicit type/heat/updated_at (age control).
    fn aged(memory_type: MemoryType, heat: u32, updated_at: &str) -> MemoryRecord {
        let mut r = fixture();
        r.memory_type = memory_type;
        r.heat = heat;
        r.updated_at = updated_at.into();
        r.created_at = updated_at.into();
        r
    }

    #[test]
    fn retention_factor_zero_age_is_one() {
        let policy = DecayPolicy::default();
        let r = aged(MemoryType::Episodic, 5, NOW_ISO);
        assert!((retention_factor(&r, &policy, NOW_MS) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn retention_factor_at_one_half_life_is_half() {
        let policy = DecayPolicy::default();
        let r = aged(MemoryType::Episodic, 5, T0);
        let retention = retention_factor(&r, &policy, T0_MS + SEVEN_DAYS_MS);
        assert!(
            (retention - 0.5).abs() < 1e-12,
            "7d / 7d half-life → 0.5, got {retention}"
        );
    }

    #[test]
    fn retention_factor_at_two_half_lives_is_quarter() {
        let policy = DecayPolicy::default();
        let r = aged(MemoryType::Episodic, 5, T0);
        let retention = retention_factor(&r, &policy, T0_MS + 2 * SEVEN_DAYS_MS);
        assert!(
            (retention - 0.25).abs() < 1e-12,
            "14d / 7d half-life → 0.25, got {retention}"
        );
    }

    #[test]
    fn retention_factor_exempt_type_is_one() {
        let policy = DecayPolicy::default();
        let r = aged(MemoryType::Instruction, 5, "2020-01-01T00:00:00.000Z");
        assert!((retention_factor(&r, &policy, NOW_MS) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn retention_factor_clock_skew_is_one() {
        let policy = DecayPolicy::default();
        let r = aged(MemoryType::Episodic, 5, "2026-06-01T00:00:00.000Z");
        assert!((retention_factor(&r, &policy, T0_MS) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn retention_factor_custom_half_life() {
        let mut policy = DecayPolicy::default();
        policy.set_half_life(MemoryType::Persona, 86_400_000); // 1d
        let r = aged(MemoryType::Persona, 3, T0);
        let retention = retention_factor(&r, &policy, T0_MS + 86_400_000);
        assert!((retention - 0.5).abs() < 1e-12);
        policy.clear_half_life(MemoryType::Persona);
        assert!((retention_factor(&r, &policy, NOW_MS) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn retention_factor_unparseable_timestamps_is_one() {
        let policy = DecayPolicy::default();
        let mut r = aged(MemoryType::Episodic, 5, "not-a-timestamp");
        r.created_at = "also-garbage".into();
        assert!((retention_factor(&r, &policy, NOW_MS) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn zero_half_life_is_clamped_not_nan() {
        let mut policy = DecayPolicy::default();
        policy.set_half_life(MemoryType::Persona, 0);
        let r = aged(MemoryType::Persona, 2, T0);
        let retention = retention_factor(&r, &policy, T0_MS + 1);
        assert!(
            retention.is_finite() && (retention - 0.5).abs() < 1e-12,
            "hl=0 clamps to 1ms → 0.5 after 1ms, got {retention}"
        );
    }

    #[test]
    fn default_policy_declares_per_type_half_lives() {
        let policy = DecayPolicy::default();
        assert_eq!(
            policy.half_life_ms(MemoryType::Persona),
            Some(7_776_000_000)
        );
        assert_eq!(policy.half_life_ms(MemoryType::Episodic), Some(604_800_000));
        assert_eq!(policy.half_life_ms(MemoryType::Instruction), None);
        assert_eq!(
            policy.half_life_ms(MemoryType::WorkFact),
            Some(2_592_000_000)
        );
        assert_eq!(
            policy.half_life_ms(MemoryType::WorkTask),
            Some(1_209_600_000)
        );
        assert_eq!(
            policy.half_life_ms(MemoryType::WorkMethod),
            Some(5_184_000_000)
        );
        assert_eq!(
            policy.half_life_ms(MemoryType::WorkArtifact),
            Some(2_592_000_000)
        );
    }

    #[test]
    fn decay_policy_serde_round_trips() {
        let policy = DecayPolicy::default();
        let json = serde_json::to_string(&policy).expect("serialize policy");
        let back: DecayPolicy = serde_json::from_str(&json).expect("deserialize policy");
        assert_eq!(policy, back);
    }

    #[test]
    fn effective_heat_rounds_known_values() {
        let policy = DecayPolicy::default();
        let r = aged(MemoryType::Episodic, 8, T0);
        assert_eq!(effective_heat(&r, &policy, T0_MS + SEVEN_DAYS_MS), 4);
        let r = aged(MemoryType::Episodic, 3, T0);
        assert_eq!(effective_heat(&r, &policy, T0_MS + 2 * SEVEN_DAYS_MS), 1);
        let r = aged(MemoryType::Instruction, 2, "2020-01-01T00:00:00.000Z");
        assert_eq!(effective_heat(&r, &policy, NOW_MS), 2, "exempt keeps raw");
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
