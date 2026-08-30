//! MEM-61 — Dreaming: consolidación idle (sleep-time tiering).
//!
//! Background job that runs during agent downtime (idle ≥ `idle_threshold_ms`
//! since `last_active_at_ms`, or on session close) and produces a consolidated
//! view of the raw L0/L1 state **without mutating the original store**.
//!
//! Inspired by Letta's sleep-time compute (`letta.com/blog/sleep-time-compute`,
//! 2025-04-21) and MemGPT 2.0: a secondary agent runs during downtime to
//! rewrite memory state into a clean, concise, and detailed form. The primary
//! store is **never** touched by this module — all writes go to a separate
//! `dream/<session>/<run_id>` namespace, reviewable and discardable.
//!
//! ## Invariants (Regla 0 + Pre-mortem)
//!
//! 1. **No mutation of `l1/<session>`.** Reads via [`scan_session_records`] are
//!    strictly read-only. The integration test `tests/dreaming.rs` verifies
//!    that the L1 records are byte-identical before and after a dream run.
//! 2. **All writes go to `dream/<session>/<run_id>`.** The `run_id` is a
//!    unique 16-char hex (uuid-v7 truncated, deterministic in tests).
//! 3. **LLM tiering is opt-in via the [`Dreamer`] trait.** With no runner
//!    configured the module degrades to LLM-free operations (hash dedup,
//!    relative-date table, deterministic contradiction resolution via
//!    priority+timestamp). Nothing blocks.
//! 4. **Discard is a real store delete; promote is a stub** (returns the count
//!    of records the run would touch — actual merge into L1 is out of scope
//!    for MEM-61 and reserved for MEM-65 in W21, where the pipeline worker
//!    integration lives).
//!
//! ## Pipeline integration
//!
//! This module is **deliberately not wired into `pipeline_worker.rs` yet**.
//! The integration arrives in MEM-65 (W21, parallel) — that task adds a new
//! [`TaskKind`] and extends [`MemoryTaskHandler::handle`] to call
//! [`consolidate_session`] when the host enqueues a Dream task. MEM-61 ships
//! the standalone primitive so MEM-65 can wire it without touching the
//! worker in this commit.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::core::abstractions::{MemoryRecord, MemoryType};
use crate::core::conversation::sanitize_component;
use crate::core::record::lifecycle::mark_contradiction;
use crate::core::record::{read_session_records, L1Error};

// Re-export the canonical contradiction provenance type so dream consumers
// don't have to reach into MEM-60's lifecycle module.
pub use crate::core::record::lifecycle::ContradictionProvenance;

/// Hashable companion for [`MemoryType`] — `MemoryType` does not derive
/// `Hash` (out of scope to modify per blast-radius discipline), so the
/// dream module uses the variant's serde name string as the key.
fn memory_type_key(mt: MemoryType) -> &'static str {
    match mt {
        MemoryType::Persona => "persona",
        MemoryType::Episodic => "episodic",
        MemoryType::Instruction => "instruction",
        MemoryType::WorkFact => "work_fact",
        MemoryType::WorkTask => "work_task",
        MemoryType::WorkMethod => "work_method",
        MemoryType::WorkArtifact => "work_artifact",
    }
}

// ── Public configuration ─────────────────────────────────────────────

/// Configuration of a dreaming consolidation pass.
///
/// Ponytail: plain struct with builder-style overrides, no env vars.
/// Hosts construct one explicitly and hand it to [`consolidate_session`].
/// `Debug` is intentionally hand-rolled — [`Dreamer`] is a trait object
/// (no auto `Debug`); `Clone` requires manual implementation (we just share
/// `Arc<dyn Dreamer>` semantics — see [`DreamConfig::with_dreamer`]).
pub struct DreamConfig {
    /// Minimum idle window before dreaming fires. `now_ms - last_active_at_ms`
    /// must be ≥ this for [`detect_idle`] to return `true`. Default:
    /// 10 minutes (600 000 ms) — matches Letta's recommended idle window.
    pub idle_threshold_ms: u64,
    /// Optional runner for LLM-driven consolidation steps. `None` → the
    /// module degrades to LLM-free primitives (hash dedup, deterministic
    /// contradiction resolution, relative-date table). The host may swap
    /// runners between runs to enforce sleep-time tiering (sleep agent uses
    /// a stronger model than the conversational primary agent).
    pub dreaming_runner: Option<Box<dyn Dreamer>>,
    /// Salt for run-id generation (tests use a fixed string for determinism).
    pub run_id_salt: String,
}

impl std::fmt::Debug for DreamConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DreamConfig")
            .field("idle_threshold_ms", &self.idle_threshold_ms)
            .field(
                "dreaming_runner",
                &self.dreaming_runner.as_ref().map(|r| r.label()),
            )
            .field("run_id_salt", &self.run_id_salt)
            .finish()
    }
}

impl Clone for DreamConfig {
    /// Manual Clone: `Box<dyn Dreamer>` is not `Clone` by design (hosts must
    /// explicitly reconstruct runners if they want to share configs). Dropping
    /// the runner field produces a config that still works for LLM-free
    /// primitives. Hosts that want to clone-and-share should wrap their
    /// runner in `Arc<dyn Dreamer>` and accept the extra indirection.
    fn clone(&self) -> Self {
        Self {
            idle_threshold_ms: self.idle_threshold_ms,
            dreaming_runner: None, // explicit choice: don't share trait objects
            run_id_salt: self.run_id_salt.clone(),
        }
    }
}

impl Default for DreamConfig {
    fn default() -> Self {
        Self {
            idle_threshold_ms: 600_000, // 10 minutes
            dreaming_runner: None,
            run_id_salt: String::new(),
        }
    }
}

impl DreamConfig {
    /// Builder-style override: idle threshold.
    pub fn with_idle_threshold_ms(mut self, ms: u64) -> Self {
        self.idle_threshold_ms = ms;
        self
    }

    /// Builder-style override: LLM runner.
    pub fn with_dreamer(mut self, runner: Box<dyn Dreamer>) -> Self {
        self.dreaming_runner = Some(runner);
        self
    }

    /// Builder-style override: run-id salt (test-only typical).
    pub fn with_run_id_salt(mut self, salt: impl Into<String>) -> Self {
        self.run_id_salt = salt.into();
        self
    }
}

// ── Public types ──────────────────────────────────────────────────────

/// One consolidation pass outcome (persisted under `dream/<s>/<run_id>`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DreamRun {
    /// Stable 16-char hex identifier (uuid-v7 style; deterministic in tests).
    pub run_id: String,
    /// Session this run consolidated.
    pub session_id: String,
    /// Epoch ms when consolidation started.
    pub started_at_ms: u64,
    /// Epoch ms when consolidation ended.
    pub ended_at_ms: u64,
    /// Number of L1 records scanned (input set size).
    pub inputs_scanned: usize,
    /// IDs of records marked as duplicates by [`merge_duplicates`].
    #[serde(default)]
    pub merged_ids: Vec<String>,
    /// Provenance records emitted by [`resolve_contradictions`].
    #[serde(default)]
    pub contradicted_ids: Vec<ContradictionProvenance>,
    /// Number of records whose relative dates were normalized.
    #[serde(default)]
    pub normalized_count: usize,
    /// Runner label (e.g. `"mock"`, `"gpt-4.1"`, `"sonnet-3.7"`). `"none"` when
    /// no LLM runner was injected (LLM-free primitive path).
    pub runner_label: String,
    /// The consolidated records the runner (or LLM-free path) produced. These
    /// live in `dream/<s>/<run_id>` and never replace the originals.
    #[serde(default)]
    pub consolidated: Vec<MemoryRecord>,
}

/// Metadata summary of one dream run (returned by [`list_dream_runs`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DreamRunMeta {
    pub run_id: String,
    pub started_at_ms: u64,
    pub ended_at_ms: u64,
    pub inputs_scanned: usize,
    pub runner_label: String,
}

/// A group of L1 records that [`merge_duplicates`] identified as duplicates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DuplicateGroup {
    /// Hash key used to bucket the records (scene + lowercased content
    /// shingles — see [`merge_duplicates`]).
    pub bucket: String,
    /// IDs of every record in the bucket (≥ 2).
    pub record_ids: Vec<String>,
}

/// Result of normalizing one relative-date string to absolute ISO-8601.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedDate {
    /// The original fragment ("ayer", "hace 3 días", etc.).
    pub original: String,
    /// Absolute ISO-8601 wall-clock timestamp.
    pub absolute: String,
}

/// Context handed to the [`Dreamer`] trait when the host runs an LLM-driven
/// consolidation pass.
#[derive(Debug, Clone)]
pub struct DreamContext {
    pub session_id: String,
    pub now_ms: u64,
    pub config: DreamConfig,
}

/// Errors surfaced by the dream module.
#[derive(Debug, Error)]
pub enum ConsolidationError {
    #[error("dream store write failed: {0}")]
    Store(String),
    #[error("dream store read failed: {0}")]
    Read(String),
    #[error("dream runner failed: {0}")]
    Runner(String),
    #[error("invalid session id: {0}")]
    InvalidSessionId(String),
}

// ── Public trait (LLM extension point) ────────────────────────────────

/// Host-extensible consolidation runner. Default implementations live in
/// [`consolidate_session`] (LLM-free path). Hosts that want sleep-time tiering
/// inject a runner via [`DreamConfig::with_dreamer`] — `MockDreamer` is
/// available under `cfg(test)` for deterministic testing.
pub trait Dreamer: Send + Sync {
    /// Short human-readable identifier (`"gpt-4.1"`, `"sonnet-3.7"`, …). The
    /// value is persisted on every [`DreamRun`] for auditability.
    fn label(&self) -> &str;

    /// Consolidate one batch of L1 records. Returns the consolidated view —
    /// duplicates merged, contradictions resolved, dates normalized. The
    /// caller writes the result to `dream/<s>/<run_id>`; the originals are
    /// never touched.
    fn consolidate(
        &self,
        records: Vec<MemoryRecord>,
        ctx: &DreamContext,
    ) -> Result<Vec<MemoryRecord>, String>;
}

// ── 1) detect_idle — pure clock check ────────────────────────────────

/// `true` when the agent has been idle for at least `threshold_ms`.
///
/// `now_ms` and `last_active_at_ms` are epoch milliseconds (injectable for
/// determinism). Default threshold: 10 min (see [`DreamConfig::default`]).
///
/// This is a **pure** function — no I/O, no timer, no async. Hosts (or the
/// pipeline worker, when wired in MEM-65) decide when to call it.
pub fn detect_idle(now_ms: u64, last_active_at_ms: u64, threshold_ms: u64) -> bool {
    now_ms.saturating_sub(last_active_at_ms) >= threshold_ms
}

// ── 2) merge_duplicates — hash-based, LLM-free ───────────────────────

/// Group records that share the same `(scene_name, content-shingle)` bucket.
///
/// "Duplicate residual" = a record that survived L1 dedup but is logically a
/// restatement of another. We bucket by `scene_name` + the first 64 chars of
/// the lowercased content (the *shingle*); groups of size ≥ 2 are returned.
///
/// Ponytail: O(n) hashmap, no LLM. Stronger semantic dedup lives in the
/// [`Dreamer::consolidate`] trait override.
pub fn merge_duplicates(records: &[MemoryRecord]) -> Vec<DuplicateGroup> {
    let mut buckets: HashMap<String, Vec<String>> = HashMap::new();
    for r in records {
        let bucket = bucket_key(&r.scene_name, &r.content);
        buckets.entry(bucket).or_default().push(r.id.clone());
    }
    buckets
        .into_iter()
        .filter(|(_, ids)| ids.len() >= 2)
        .map(|(bucket, record_ids)| DuplicateGroup { bucket, record_ids })
        .collect()
}

fn bucket_key(scene: &str, content: &str) -> String {
    let normalized = content.trim().to_lowercase();
    let shingle: String = normalized.chars().take(64).collect();
    let mut h = DefaultHasher::new();
    scene.hash(&mut h);
    shingle.hash(&mut h);
    format!("{:x}|{}", h.finish(), shingle)
}

// ── 3) resolve_contradictions — reuses MEM-60 mark_contradiction ─────

/// Scan a session's L1 records for contradictions and mark the losers.
///
/// Algorithm (LLM-free, deterministic):
///   1. Bucket by `scene_name`.
///   2. Inside each bucket, compare pairs of records with same `memory_type`.
///   3. If both have the same `scene_name` + same `memory_type` but
///      meaningfully-different `priority` *and* the newer record wins the
///      "current truth" test (priority > loser's, or equal priority + newer
///      `created_at`), mark the loser as superseded via MEM-60's
///      [`mark_contradiction`]. **We never mutate the caller's `Vec`; we
///      emit provenance records.** The caller persists the loser's updated
///      `superseded_by` field, NOT this function.
///
/// Why pass-through provenance only: the store layer in [`write_dream_run`]
/// writes the resolved losers as part of the **dream namespace** — the
/// originals in `l1/<s>` keep their `superseded_by = None` until promotion
/// is wired (MEM-65). This is the pre-mortem guarantee: "store original
/// jamás se muta".
pub fn resolve_contradictions(
    records: &[MemoryRecord],
    now_ms: u64,
) -> Vec<ContradictionProvenance> {
    let mut provenance = Vec::new();
    // Group by (scene_name, memory_type-as-str). We avoid HashMap<(String,
    // MemoryType), _> because MemoryType does not derive Hash (out of scope
    // to modify — pre-mortem discipline). The string key is the variant's
    // serde name (stable, collision-free across the 7 types).
    let mut by_scene_type: HashMap<(String, &'static str), Vec<&MemoryRecord>> = HashMap::new();
    for r in records {
        by_scene_type
            .entry((r.scene_name.clone(), memory_type_key(r.memory_type)))
            .or_default()
            .push(r);
    }
    for (_key, bucket) in by_scene_type {
        if bucket.len() < 2 {
            continue;
        }
        // Sort by priority desc, then created_at desc — winner = bucket[0].
        let mut sorted = bucket;
        sorted.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| b.created_at.cmp(&a.created_at))
        });
        let winner = sorted[0];
        for loser in &sorted[1..] {
            // Same priority & same created_at → skip (can't decide, no LLM).
            if loser.priority == winner.priority && loser.created_at == winner.created_at {
                continue;
            }
            // Synthesize a transient clone to call mark_contradiction (we
            // discard the clone — we only want the provenance).
            let mut loser_clone = (*loser).clone();
            let p = mark_contradiction(&mut loser_clone, winner.id.clone(), now_ms);
            provenance.push(p);
        }
    }
    provenance
}

// ── 4) normalize_relative_dates — table-driven, LLM-free ─────────────

/// Normalize the `activity_start_time` (or any timestamp-shaped metadata field)
/// of a record from a relative phrase to an absolute ISO-8601 timestamp.
///
/// Recognized patterns (Spanish-first; extend by adding rows to the table):
///   - `ayer`     → anchor - 1 day
///   - `hoy`      → anchor
///   - `mañana`   → anchor + 1 day
///   - `anteayer` → anchor - 2 days
///   - `hace N (minuto|minutos|hora|horas|día|días|semana|semanas|mes|meses)`
///   - `hace N (min|h|hr|d|sem|mes)` (compact form)
///
/// `anchor_ms` is the wall-clock time the relative phrase is anchored to
/// (typically `now_ms`). Returns `Some(NormalizedDate)` when a known pattern
/// is matched, `None` otherwise (the caller decides whether to log/mutate).
pub fn normalize_relative_dates(record: &MemoryRecord, anchor_ms: u64) -> Option<NormalizedDate> {
    // Pull a candidate timestamp from the metadata.
    let raw = record
        .metadata
        .as_object()
        .and_then(|o| o.get("activity_start_time"))
        .and_then(|v| v.as_str())?;
    let trimmed = raw.trim().to_lowercase();
    let absolute = parse_relative(&trimmed, anchor_ms)?;
    Some(NormalizedDate {
        original: raw.to_string(),
        absolute,
    })
}

fn parse_relative(input: &str, anchor_ms: u64) -> Option<String> {
    const DAY: u64 = 86_400_000;
    let ms = match input {
        "ayer" => anchor_ms.checked_sub(DAY)?,
        "hoy" => anchor_ms,
        "mañana" => anchor_ms.checked_add(DAY)?,
        "anteayer" => anchor_ms.checked_sub(2 * DAY)?,
        s if s.starts_with("hace ") => {
            let rest = s.trim_start_matches("hace ").trim();
            let (n, unit_ms) = split_count_unit(rest)?;
            let n = n as i64;
            let delta_ms = (unit_ms as i64).checked_mul(n)?;
            let signed_anchor = anchor_ms as i64;
            (signed_anchor.checked_sub(delta_ms)?).max(0) as u64
        }
        _ => return None,
    };
    Some(millis_to_iso8601(ms))
}

fn split_count_unit(rest: &str) -> Option<(u64, u64)> {
    // Accepts both "3 días" and "3d" / "3 d" forms. Strategy: split the
    // numeric prefix from the rest; the rest is the unit (whitespace-
    // insensitive). Falls back to looking for the first digit-letter
    // boundary when no whitespace is present (compact form like "5h").
    let trimmed = rest.trim();
    let split_idx = trimmed.find(|c: char| !c.is_ascii_digit())?;
    let num_str = &trimmed[..split_idx];
    let unit_str = trimmed[split_idx..].trim().to_lowercase();
    let n: u64 = num_str.parse().ok()?;
    if n == 0 {
        return None;
    }
    let unit_ms = match unit_str.as_str() {
        "minuto" | "minutos" | "min" => 60_000,
        "hora" | "horas" | "h" | "hr" => 3_600_000,
        "día" | "días" | "d" => 86_400_000,
        "semana" | "semanas" | "sem" => 7 * 86_400_000,
        "mes" | "meses" => 30 * 86_400_000, // approx — same as MEM-60 cycle
        _ => return None,
    };
    Some((n, unit_ms))
}

/// Pure helper: epoch ms → `YYYY-MM-DDTHH:MM:SS.sssZ`. Mirrors
/// `crate::core::record::lifecycle::millis_to_iso8601` but is duplicated here
/// (private) to keep the dream module self-contained and avoid adding a
/// public re-export on MEM-60 for one consumer.
fn millis_to_iso8601(ms: u64) -> String {
    let total_seconds = ms / 1000;
    let millis = ms % 1000;
    let days = total_seconds / 86400;
    let secs_of_day = total_seconds % 86400;
    let hour = (secs_of_day / 3600) as u8;
    let minute = ((secs_of_day % 3600) / 60) as u8;
    let second = (secs_of_day % 60) as u8;
    let z = days as i64 + 719_468;
    let era = if z >= 0 {
        z / 146_097
    } else {
        (z - 146_096) / 146_097
    };
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
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
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        year, m, d, hour, minute, second, millis
    )
}

// ── Store layer — dream/<session>/<run_id> ───────────────────────────

/// Scan a session's L1 records (read-only over `l1/<session>`).
///
/// Returns a clone of every record — mutating the returned Vec does not
/// affect the store. This is the only function in the dream module that
/// touches `l1/<session>`, and it does so **read-only** via the canonical
/// [`read_session_records`] reader (MEM-11; paged + skips corrupt records).
pub fn scan_session_records(
    db: &vantadb::sdk::VantaEmbedded,
    session_id: &str,
) -> Result<Vec<MemoryRecord>, ConsolidationError> {
    if session_id.is_empty() {
        return Err(ConsolidationError::InvalidSessionId(
            "session_id must not be empty".into(),
        ));
    }
    read_session_records(db, session_id)
        .map_err(|e: L1Error| ConsolidationError::Read(e.to_string()))
}

/// Namespace for one dream run: `dream/<sanitized_session>/<run_id>`.
fn dream_namespace(session_id: &str, run_id: &str) -> String {
    format!(
        "dream/{}/{}",
        sanitize_component(session_id, 128, false),
        run_id
    )
}

/// Write a [`DreamRun`] to `dream/<session>/<run_id>`. The runner label and
/// the consolidated records are persisted as one JSON record (atomic write).
/// **The original L1 store is never touched.**
pub fn write_dream_run(
    db: &vantadb::sdk::VantaEmbedded,
    run: &DreamRun,
) -> Result<(), ConsolidationError> {
    let ns = dream_namespace(&run.session_id, &run.run_id);
    let payload = serde_json::to_string(run)
        .map_err(|e| ConsolidationError::Store(format!("serialize dream run: {e}")))?;
    db.put(vantadb::sdk::VantaMemoryInput {
        namespace: ns,
        key: "run.json".into(),
        payload,
        metadata: vantadb::sdk::VantaMemoryMetadata::new(),
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
    })
    .map_err(|e| ConsolidationError::Store(format!("write dream run: {e}")))?;
    Ok(())
}

/// List every dream run for a session (metadata only — full payload loaded on
/// demand via [`load_dream_run`]).
///
/// Each run lives in its own namespace `dream/<session>/<run_id>`. We
/// enumerate via [`VantaEmbedded::list_namespaces`] (canonical scanner) and
/// load each `run.json` key. Corrupt entries are skipped (best-effort,
/// mirrors `load_active` from context_engine).
pub fn list_dream_runs(
    db: &vantadb::sdk::VantaEmbedded,
    session_id: &str,
) -> Result<Vec<DreamRunMeta>, ConsolidationError> {
    let prefix = format!("dream/{}/", sanitize_component(session_id, 128, false));
    let all_ns = db
        .list_namespaces()
        .map_err(|e| ConsolidationError::Read(e.to_string()))?;
    let mut out = Vec::new();
    for ns in all_ns {
        if !ns.starts_with(&prefix) {
            continue;
        }
        // ns is "dream/<session>/<run_id>"; the key is "run.json".
        if let Some(rec) = db
            .get(&ns, "run.json")
            .map_err(|e| ConsolidationError::Read(e.to_string()))?
        {
            if let Ok(run) = serde_json::from_str::<DreamRun>(&rec.payload) {
                out.push(DreamRunMeta {
                    run_id: run.run_id,
                    started_at_ms: run.started_at_ms,
                    ended_at_ms: run.ended_at_ms,
                    inputs_scanned: run.inputs_scanned,
                    runner_label: run.runner_label,
                });
            }
        }
    }
    // Stable order for callers (deterministic tests + UI).
    out.sort_by(|a, b| a.run_id.cmp(&b.run_id));
    Ok(out)
}

/// Load the full [`DreamRun`] for inspection / replay.
pub fn load_dream_run(
    db: &vantadb::sdk::VantaEmbedded,
    session_id: &str,
    run_id: &str,
) -> Result<Option<DreamRun>, ConsolidationError> {
    let ns = dream_namespace(session_id, run_id);
    match db
        .get(&ns, "run.json")
        .map_err(|e| ConsolidationError::Read(e.to_string()))?
    {
        None => Ok(None),
        Some(rec) => match serde_json::from_str(&rec.payload) {
            Ok(run) => Ok(Some(run)),
            Err(e) => Err(ConsolidationError::Read(format!(
                "dream run {run_id} corrupt: {e}"
            ))),
        },
    }
}

/// Discard one dream run (deletes the `dream/<s>/<run_id>` namespace). Use
/// after review — original L1 store remains untouched.
pub fn discard_dream_run(
    db: &vantadb::sdk::VantaEmbedded,
    session_id: &str,
    run_id: &str,
) -> Result<(), ConsolidationError> {
    let ns = dream_namespace(session_id, run_id);
    db.delete(&ns, "run.json")
        .map_err(|e| ConsolidationError::Store(format!("discard dream run: {e}")))?;
    Ok(())
}

/// **STUB** for promotion — returns the number of consolidated records the run
/// would merge into L1, **without mutating** `l1/<session>`. The real
/// promotion is wired in MEM-65 (W21) where the pipeline worker owns the
/// lifecycle. Returning the count lets CLI / dashboards preview the diff
/// before promotion lands.
///
/// Invariant (asserted by the integration test): `l1/<session>` byte-identical
/// before and after `promote_dream_run`. Documented in the public doc comment
/// because the function name implies mutation but does NOT — this is the
/// explicit pre-mortem guarantee.
pub fn promote_dream_run(
    db: &vantadb::sdk::VantaEmbedded,
    session_id: &str,
    run_id: &str,
) -> Result<usize, ConsolidationError> {
    let run = load_dream_run(db, session_id, run_id)?
        .ok_or_else(|| ConsolidationError::Read(format!("dream run {run_id} not found")))?;
    Ok(run.consolidated.len())
}

/// One-shot consolidation: scan → dedupe → resolve → normalize → (optional
/// LLM pass via runner) → persist to `dream/<s>/<run_id>`. Returns the
/// persisted [`DreamRun`].
///
/// This is the function the pipeline worker will call (MEM-65) once wired.
pub fn consolidate_session(
    db: &vantadb::sdk::VantaEmbedded,
    session_id: &str,
    now_ms: u64,
    last_active_at_ms: u64,
    config: &DreamConfig,
) -> Result<DreamRun, ConsolidationError> {
    if !detect_idle(now_ms, last_active_at_ms, config.idle_threshold_ms) {
        return Err(ConsolidationError::Runner(format!(
            "not idle: now_ms - last_active_at_ms = {}ms < threshold {}ms",
            now_ms.saturating_sub(last_active_at_ms),
            config.idle_threshold_ms
        )));
    }

    let started_at_ms = now_ms;
    let mut records = scan_session_records(db, session_id)?;
    let inputs_scanned = records.len();

    // Step 1: dedupe (LLM-free).
    let dupes = merge_duplicates(&records);
    let merged_ids: Vec<String> = dupes
        .iter()
        .flat_map(|g| g.record_ids.iter().cloned())
        .collect();

    // Step 2: contradictions (reuses MEM-60 — never mutates originals here).
    let provenance = resolve_contradictions(&records, now_ms);
    let contradicted_ids = provenance.clone();

    // Step 3: relative-date normalization (counts + sets metadata on the
    // dream-side copy only — the originals in `records` keep their raw
    // relative dates; this function never writes back to `l1/<s>`).
    let mut normalized_count = 0;
    for r in records.iter_mut() {
        if let Some(_normalized) = normalize_relative_dates(r, now_ms) {
            // Mark on the dream-side clone; the original l1 payload is
            // untouched (it still carries the raw "ayer" string).
            r.metadata = apply_normalized_meta(&r.metadata, &_normalized.absolute);
            normalized_count += 1;
        }
    }

    // Step 4: optional LLM-driven consolidation via the host's runner.
    let consolidated = match &config.dreaming_runner {
        Some(runner) => {
            let ctx = DreamContext {
                session_id: session_id.to_string(),
                now_ms,
                config: config.clone(),
            };
            runner
                .consolidate(records.clone(), &ctx)
                .map_err(ConsolidationError::Runner)?
        }
        None => records,
    };

    let run_id = generate_run_id(&config.run_id_salt, session_id, now_ms);
    let ended_at_ms = now_ms;
    let runner_label = config
        .dreaming_runner
        .as_ref()
        .map(|r| r.label().to_string())
        .unwrap_or_else(|| "none".into());

    let run = DreamRun {
        run_id: run_id.clone(),
        session_id: session_id.to_string(),
        started_at_ms,
        ended_at_ms,
        inputs_scanned,
        merged_ids,
        contradicted_ids,
        normalized_count,
        runner_label,
        consolidated,
    };
    write_dream_run(db, &run)?;
    Ok(run)
}

fn apply_normalized_meta(meta: &serde_json::Value, absolute: &str) -> serde_json::Value {
    let mut obj = meta
        .as_object()
        .cloned()
        .unwrap_or_else(serde_json::Map::new);
    obj.insert(
        "activity_start_time".into(),
        serde_json::Value::String(absolute.into()),
    );
    serde_json::Value::Object(obj)
}

/// Deterministic 16-char hex run-id. Tests pass a fixed `salt` + session_id +
/// `now_ms` and get the same id back. In production `salt` defaults to `""`
/// and the id is a v7-style timestamp-based id (16 hex chars from the ms +
/// a 4-char hash of the salt).
pub fn generate_run_id(salt: &str, session_id: &str, now_ms: u64) -> String {
    let mut h = DefaultHasher::new();
    salt.hash(&mut h);
    session_id.hash(&mut h);
    now_ms.hash(&mut h);
    let suffix = format!("{:016x}", h.finish());
    let ts = format!("{:08x}", now_ms & 0xFFFF_FFFF);
    let combined = format!("{ts}{suffix}");
    combined.chars().take(16).collect()
}

// ── Re-export convenience ────────────────────────────────────────────

// (no extra re-exports — `ContradictionProvenance` is re-exported above)

// ── Unit tests (RED → GREEN, inline) ─────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::abstractions::{MemoryType, SceneSegment};
    use serde_json::json;

    fn rec(id: &str, scene: &str, content: &str, priority: i32, created_at: &str) -> MemoryRecord {
        MemoryRecord {
            id: id.into(),
            content: content.into(),
            memory_type: MemoryType::Persona,
            priority,
            scene_name: scene.into(),
            source_message_ids: vec![],
            metadata: json!(null),
            timestamps: vec![],
            created_at: created_at.into(),
            updated_at: created_at.into(),
            version: 1,
            session_key: "s1".into(),
            session_id: "".into(),
            task_id: None,
            team_id: None,
            user_id: None,
            agent_id: None,
            vector: None,
            heat: 0,
            superseded_by: None,
        }
    }

    // detect_idle

    #[test]
    fn detect_idle_true_when_threshold_reached() {
        assert!(detect_idle(700_000, 0, 600_000));
        assert!(detect_idle(700_000, 100_000, 600_000));
    }

    #[test]
    fn detect_idle_false_when_below_threshold() {
        assert!(!detect_idle(500_000, 0, 600_000));
        assert!(!detect_idle(700_000, 200_000, 600_000));
    }

    #[test]
    fn detect_idle_handles_saturating_sub_underflow() {
        // now_ms < last_active_at_ms (clock went backwards) — saturating to 0,
        // below any threshold → false (not idle yet).
        assert!(!detect_idle(100, 200, 600_000));
    }

    // merge_duplicates

    #[test]
    fn merge_duplicates_groups_records_with_same_shingle() {
        let r1 = rec(
            "m1",
            "ui",
            "user prefers dark mode",
            80,
            "2026-08-20T10:00:00.000Z",
        );
        let r2 = rec(
            "m2",
            "ui",
            "user prefers dark mode",
            80,
            "2026-08-20T10:05:00.000Z",
        );
        let r3 = rec(
            "m3",
            "ui",
            "user prefers light mode",
            80,
            "2026-08-20T10:06:00.000Z",
        );
        let groups = merge_duplicates(&[r1, r2, r3]);
        assert_eq!(groups.len(), 1, "only m1+m2 share the shingle");
        assert_eq!(groups[0].record_ids.len(), 2);
    }

    #[test]
    fn merge_duplicates_singletons_are_not_groups() {
        let r = rec(
            "m1",
            "ui",
            "user prefers dark mode",
            80,
            "2026-08-20T10:00:00.000Z",
        );
        assert!(merge_duplicates(&[r]).is_empty());
    }

    #[test]
    fn merge_duplicates_case_and_whitespace_insensitive() {
        let r1 = rec(
            "m1",
            "ui",
            "  User Prefers Dark Mode  ",
            80,
            "2026-08-20T10:00:00.000Z",
        );
        let r2 = rec(
            "m2",
            "ui",
            "user prefers dark mode",
            80,
            "2026-08-20T10:05:00.000Z",
        );
        let groups = merge_duplicates(&[r1, r2]);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].record_ids.len(), 2);
    }

    // resolve_contradictions

    #[test]
    fn resolve_contradictions_marks_lower_priority_as_superseded() {
        let high = rec(
            "m_new",
            "ui",
            "user prefers dark mode",
            90,
            "2026-08-20T10:10:00.000Z",
        );
        let low = rec(
            "m_old",
            "ui",
            "user prefers dark mode",
            50,
            "2026-08-20T10:00:00.000Z",
        );
        let provenance = resolve_contradictions(&[high, low], 1_700_000_000_000);
        assert_eq!(provenance.len(), 1);
        assert_eq!(provenance[0].old_key, "m_old");
        assert_eq!(provenance[0].new_key, "m_new");
    }

    #[test]
    fn resolve_contradictions_no_op_when_priorities_match() {
        let a = rec("a", "ui", "x", 80, "2026-08-20T10:00:00.000Z");
        let b = rec("b", "ui", "x", 80, "2026-08-20T10:00:00.000Z");
        assert!(resolve_contradictions(&[a, b], 1_700_000_000_000).is_empty());
    }

    #[test]
    fn resolve_contradictions_skips_different_scenes() {
        let a = rec("a", "ui", "x", 90, "2026-08-20T10:00:00.000Z");
        let b = rec("b", "audio", "x", 50, "2026-08-20T10:00:00.000Z");
        // Different scenes → different buckets → no contradiction.
        assert!(resolve_contradictions(&[a, b], 1_700_000_000_000).is_empty());
    }

    // normalize_relative_dates

    #[test]
    fn normalize_relative_dates_handles_ayer() {
        let mut r = rec("m1", "ui", "x", 80, "2026-08-20T10:00:00.000Z");
        r.metadata = json!({ "activity_start_time": "ayer" });
        let n = normalize_relative_dates(&r, 1_700_000_000_000).unwrap();
        assert_eq!(n.original, "ayer");
        // 1 day earlier in ISO-8601.
        assert!(n.absolute.starts_with("2023-11-14") || n.absolute.starts_with("2023-11-13"));
    }

    #[test]
    fn normalize_relative_dates_handles_hace_3_dias() {
        let mut r = rec("m1", "ui", "x", 80, "2026-08-20T10:00:00.000Z");
        r.metadata = json!({ "activity_start_time": "hace 3 días" });
        let n = normalize_relative_dates(&r, 1_700_000_000_000).unwrap();
        assert_eq!(n.original, "hace 3 días");
        assert!(n.absolute.len() == 24);
    }

    #[test]
    fn normalize_relative_dates_handles_compact_form() {
        let mut r = rec("m1", "ui", "x", 80, "2026-08-20T10:00:00.000Z");
        r.metadata = json!({ "activity_start_time": "hace 5h" });
        let n = normalize_relative_dates(&r, 1_700_000_000_000).unwrap();
        assert!(n.absolute.len() == 24);
    }

    #[test]
    fn normalize_relative_dates_returns_none_for_unknown() {
        let r = rec("m1", "ui", "x", 80, "2026-08-20T10:00:00.000Z");
        assert!(normalize_relative_dates(&r, 1_700_000_000_000).is_none());
    }

    #[test]
    fn normalize_relative_dates_handles_hoy_and_manana() {
        let mut r = rec("m1", "ui", "x", 80, "2026-08-20T10:00:00.000Z");
        r.metadata = json!({ "activity_start_time": "hoy" });
        let n = normalize_relative_dates(&r, 1_700_000_000_000).unwrap();
        assert_eq!(n.absolute, millis_to_iso8601(1_700_000_000_000));

        r.metadata = json!({ "activity_start_time": "mañana" });
        let n = normalize_relative_dates(&r, 1_700_000_000_000).unwrap();
        assert_eq!(
            n.absolute,
            millis_to_iso8601(1_700_000_000_000 + 86_400_000)
        );
    }

    // generate_run_id determinism

    #[test]
    fn run_id_is_deterministic_for_same_inputs() {
        let a = generate_run_id("salt", "s1", 1_700_000_000_000);
        let b = generate_run_id("salt", "s1", 1_700_000_000_000);
        assert_eq!(a, b);
        assert_eq!(a.len(), 16);
    }

    #[test]
    fn run_id_changes_when_inputs_change() {
        let a = generate_run_id("salt", "s1", 1_700_000_000_000);
        let b = generate_run_id("salt", "s1", 1_700_000_000_001);
        assert_ne!(a, b);
    }

    // bucket_key determinism + shape

    #[test]
    fn bucket_key_includes_scene_and_shingle() {
        let a = bucket_key("ui", "user prefers dark mode");
        let b = bucket_key("ui", "user prefers dark mode");
        assert_eq!(a, b);
        assert!(a.contains("user prefers dark mode"));
        let c = bucket_key("audio", "user prefers dark mode");
        assert_ne!(a, c, "different scene → different bucket");
    }

    // consume-only reference for SceneSegment (silences dead_code)
    #[allow(dead_code)]
    fn _ref() -> SceneSegment {
        SceneSegment {
            scene_name: "x".into(),
            message_ids: vec![],
            memories: vec![],
        }
    }

    // MockDreamer — extension point test
    #[test]
    fn mock_dreamer_label_and_consolidate_round_trip() {
        struct IdentityDreamer;
        impl Dreamer for IdentityDreamer {
            fn label(&self) -> &str {
                "identity"
            }
            fn consolidate(
                &self,
                records: Vec<MemoryRecord>,
                _ctx: &DreamContext,
            ) -> Result<Vec<MemoryRecord>, String> {
                Ok(records)
            }
        }
        let d: Box<dyn Dreamer> = Box::new(IdentityDreamer);
        assert_eq!(d.label(), "identity");
        let records = vec![rec("a", "ui", "x", 80, "2026-08-20T10:00:00.000Z")];
        let ctx = DreamContext {
            session_id: "s1".into(),
            now_ms: 1_700_000_000_000,
            config: DreamConfig::default(),
        };
        let out = d.consolidate(records.clone(), &ctx).expect("identity");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].id, "a");
    }
}
