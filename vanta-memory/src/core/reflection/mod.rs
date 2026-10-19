//! MEMG-21 — Reflection: periodic pass over episodic memory (lessons).
//!
//! Pull-based reflection over the L1 **episodic** records of a session: groups
//! episodes by scene and produces lessons that cite their source records,
//! written to a separate `reflection/<session>/<run_id>` namespace. The L1
//! store is **never** mutated — promotion stays explicit and out of scope
//! (FIND-290), mirroring the [`crate::core::dream`] invariant.
//!
//! Sources (policy, not calibration): Park et al., *Generative Agents*
//! (arXiv:2304.03442v2 §4.2) — reflections are generated periodically,
//! synthesize higher-level insights from recent observations and cite the
//! records that served as evidence. The in-crate precedent is
//! [`crate::core::dream`]: an optional LLM runner ([`Reflector`]) with an
//! LLM-free deterministic degradation (P4) — without a runner the pass emits
//! a digest lesson per scene (top-priority episodes, declared
//! [`DIGEST_TOP`]); nothing blocks and nothing is lost.
//!
//! ## Invariants
//!
//! 1. **Never mutates `l1/<session>`** — reads via `read_session_records`,
//!    writes only to `reflection/<session>/<run_id>`.
//! 2. **LLM-free by default** — deterministic lessons; a host may inject a
//!    [`Reflector`] for true synthesis.
//! 3. **Explicit precondition** — fewer than [`ReflectionConfig::min_episodic`]
//!    episodes → [`ReflectionError::NotEnoughMaterial`] (skip is observable,
//!    never silent).

use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::core::abstractions::{MemoryRecord, MemoryType};
use crate::core::conversation::sanitize_component;
use crate::core::dream::generate_run_id;
use crate::core::prompts::l1_extraction::epoch_ms_to_rfc3339;
use crate::core::record::l1_reader::read_session_records;
use crate::core::record::L1Error;

/// Top-`DIGEST_TOP` episodes (priority desc, then `updated_at` desc) cited in
/// an LLM-free digest lesson. Declared policy.
pub const DIGEST_TOP: usize = 3;

/// Configuration of one reflection pass.
///
/// Ponytail: plain struct with builder-style overrides, no env vars — hosts
/// construct one explicitly and hand it to [`reflect_session`]. `Debug` /
/// `Clone` are hand-rolled: [`Reflector`] is a trait object (no auto
/// `Debug`; `Clone` deliberately drops the runner, same choice as
/// `DreamConfig`).
pub struct ReflectionConfig {
    /// Minimum episodic records required to run a pass. Default: 3.
    pub min_episodic: usize,
    /// Optional LLM runner. `None` → the deterministic LLM-free digest.
    pub runner: Option<Box<dyn Reflector>>,
    /// Salt for run-id generation (tests use a fixed string).
    pub run_id_salt: String,
}

impl std::fmt::Debug for ReflectionConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReflectionConfig")
            .field("min_episodic", &self.min_episodic)
            .field("runner", &self.runner.as_ref().map(|r| r.label()))
            .field("run_id_salt", &self.run_id_salt)
            .finish()
    }
}

impl Clone for ReflectionConfig {
    /// Manual Clone: `Box<dyn Reflector>` is not `Clone` by design (hosts
    /// must reconstruct runners explicitly); the clone degrades to the
    /// LLM-free path.
    fn clone(&self) -> Self {
        Self {
            min_episodic: self.min_episodic,
            runner: None,
            run_id_salt: self.run_id_salt.clone(),
        }
    }
}

impl Default for ReflectionConfig {
    fn default() -> Self {
        Self {
            min_episodic: 3,
            runner: None,
            run_id_salt: String::new(),
        }
    }
}

impl ReflectionConfig {
    /// Builder-style override: minimum episodic records.
    pub fn with_min_episodic(mut self, min: usize) -> Self {
        self.min_episodic = min;
        self
    }

    /// Builder-style override: LLM runner (opt-in synthesis).
    pub fn with_reflector(mut self, runner: Box<dyn Reflector>) -> Self {
        self.runner = Some(runner);
        self
    }

    /// Builder-style override: run-id salt (test-only typical).
    pub fn with_run_id_salt(mut self, salt: impl Into<String>) -> Self {
        self.run_id_salt = salt.into();
        self
    }
}

/// Host-extensible reflection runner. The default (LLM-free) path lives in
/// [`reflect_episodic`]; hosts that want true synthesis inject a runner via
/// [`ReflectionConfig::with_reflector`].
pub trait Reflector: Send + Sync {
    /// Short human-readable identifier (`"mock"`, `"gpt-4.1"`, …), persisted
    /// on every [`ReflectionRun`] for auditability.
    fn label(&self) -> &str;

    /// Synthesize lessons from one batch of episodes. The caller writes the
    /// result to `reflection/<s>/<run_id>`; the originals are never touched.
    fn reflect(
        &self,
        episodes: Vec<MemoryRecord>,
        ctx: &ReflectionContext,
    ) -> Result<Vec<MemoryRecord>, String>;
}

/// Context handed to a [`Reflector`] when a host runs an LLM-driven pass.
#[derive(Debug, Clone)]
pub struct ReflectionContext {
    pub session_id: String,
    pub now_ms: u64,
    pub config: ReflectionConfig,
}

/// One reflection pass outcome (persisted under `reflection/<s>/<run_id>`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReflectionRun {
    /// Stable 16-char hex identifier (uuid-v7 style; deterministic in tests).
    pub run_id: String,
    /// Session this run reflected over.
    pub session_id: String,
    /// Epoch ms when the pass started.
    pub started_at_ms: u64,
    /// Epoch ms when the pass ended.
    pub ended_at_ms: u64,
    /// Number of episodic records scanned (input set size).
    pub episodes_scanned: usize,
    /// Ids of every episodic record scanned (sorted — provenance set).
    #[serde(default)]
    pub source_ids: Vec<String>,
    /// Runner label (`"none"` = LLM-free digest path).
    pub runner_label: String,
    /// The lessons produced (live in `reflection/<s>/<run_id>`, never replace
    /// L1 records).
    #[serde(default)]
    pub lessons: Vec<MemoryRecord>,
}

/// Errors surfaced by the reflection module.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ReflectionError {
    /// Underlying VantaDB storage error while writing the run.
    #[error("reflection store write failed: {0}")]
    Store(String),
    /// Store read failure (session scan or run load).
    #[error("reflection store read failed: {0}")]
    Read(String),
    /// The optional runner failed.
    #[error("reflection runner failed: {0}")]
    Runner(String),
    /// Empty session id.
    #[error("invalid session id: {0}")]
    InvalidSessionId(String),
    /// Fewer episodic records than the declared minimum (explicit skip).
    #[error("not enough episodic material: found {found}, required {required}")]
    NotEnoughMaterial { found: usize, required: usize },
}

impl From<L1Error> for ReflectionError {
    fn from(err: L1Error) -> Self {
        match err {
            L1Error::Vanta(v) => Self::Read(v.to_string()),
            L1Error::Serde(s) => Self::Read(s.to_string()),
        }
    }
}

// ── LLM-free digest (deterministic degradation, P4) ──

/// Deterministic LLM-free reflection: one `WorkMethod` lesson per scene,
/// built from the top-[`DIGEST_TOP`] episodes (priority desc, then
/// `updated_at` desc) and citing **every** source id of the scene in
/// `metadata.reflection.source_ids`.
///
/// Pure function (no I/O): the digest is the P4 degradation of Park's
/// synthesis — nothing blocks, nothing is lost; the [`Reflector`] runner
/// replaces it with true synthesis when configured.
pub fn reflect_episodic(
    records: &[MemoryRecord],
    session_id: &str,
    now_ms: u64,
) -> Vec<MemoryRecord> {
    let mut by_scene: BTreeMap<&str, Vec<&MemoryRecord>> = BTreeMap::new();
    for record in records
        .iter()
        .filter(|r| r.memory_type == MemoryType::Episodic)
    {
        by_scene
            .entry(record.scene_name.as_str())
            .or_default()
            .push(record);
    }

    let timestamp = epoch_ms_to_rfc3339(now_ms);
    let mut lessons = Vec::with_capacity(by_scene.len());
    for (scene, mut episodes) in by_scene {
        episodes.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| b.updated_at.cmp(&a.updated_at))
        });
        let mut source_ids: Vec<String> = episodes.iter().map(|r| r.id.clone()).collect();
        source_ids.sort();
        let digest = episodes
            .iter()
            .take(DIGEST_TOP)
            .map(|r| r.content.as_str())
            .collect::<Vec<_>>()
            .join(" | ");
        let content = format!(
            "Lesson from {} episode(s) in {scene}: {digest}",
            episodes.len()
        );
        let priority = episodes.iter().map(|r| r.priority).max().unwrap_or(0);
        // Scope stamps ride the highest-priority episode (the lesson is
        // session-local until an explicit promotion — FIND-290).
        let template = episodes[0];
        lessons.push(MemoryRecord {
            id: lesson_id(scene, &source_ids),
            content,
            memory_type: MemoryType::WorkMethod,
            priority,
            scene_name: scene.to_string(),
            source_message_ids: vec![],
            metadata: serde_json::json!({
                "reflection": {
                    "source": "episodic",
                    "source_ids": source_ids,
                    "episode_count": episodes.len(),
                }
            }),
            timestamps: vec![timestamp.clone()],
            created_at: timestamp.clone(),
            updated_at: timestamp.clone(),
            version: 1,
            session_key: session_id.to_string(),
            session_id: session_id.to_string(),
            task_id: None,
            team_id: template.team_id.clone(),
            user_id: None,
            agent_id: template.agent_id.clone(),
            vector: None,
            heat: 0,
            superseded_by: None,
        });
    }
    lessons
}

/// Deterministic lesson id from (scene, cited ids) — same input → same id.
fn lesson_id(scene: &str, source_ids: &[String]) -> String {
    let mut hasher = DefaultHasher::new();
    scene.hash(&mut hasher);
    source_ids.hash(&mut hasher);
    format!("reflect-{:016x}", hasher.finish())
}

// ── Store layer — reflection/<session>/<run_id> ──

/// Namespace for one reflection run: `reflection/<sanitized_session>/<run_id>`.
fn reflection_namespace(session_id: &str, run_id: &str) -> String {
    format!(
        "reflection/{}/{}",
        sanitize_component(session_id, 128, false),
        run_id
    )
}

/// Write a [`ReflectionRun`] to `reflection/<session>/<run_id>` (one JSON
/// record, atomic write). **The original L1 store is never touched.**
fn write_reflection_run(
    db: &vantadb::sdk::Embedded,
    run: &ReflectionRun,
) -> Result<(), ReflectionError> {
    let ns = reflection_namespace(&run.session_id, &run.run_id);
    let payload = serde_json::to_string(run)
        .map_err(|e| ReflectionError::Store(format!("serialize reflection run: {e}")))?;
    db.put(vantadb::sdk::MemoryInput {
        namespace: ns,
        key: "run.json".into(),
        payload,
        metadata: vantadb::sdk::MemoryMetadata::new(),
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
        ..Default::default()
    })
    .map_err(|e| ReflectionError::Store(format!("write reflection run: {e}")))?;
    Ok(())
}

/// Load the full [`ReflectionRun`] for inspection / replay.
pub fn load_reflection_run(
    db: &vantadb::sdk::Embedded,
    session_id: &str,
    run_id: &str,
) -> Result<Option<ReflectionRun>, ReflectionError> {
    let ns = reflection_namespace(session_id, run_id);
    match db
        .get(&ns, "run.json")
        .map_err(|e| ReflectionError::Read(e.to_string()))?
    {
        None => Ok(None),
        Some(record) => serde_json::from_str::<ReflectionRun>(&record.payload)
            .map(Some)
            .map_err(|e| ReflectionError::Read(format!("reflection run {run_id} corrupt: {e}"))),
    }
}

// ── The pass ──

/// Run one reflection pass over a session's episodic memory.
///
/// Scans `l1/<session>` read-only, filters the episodic records, enforces the
/// declared [`ReflectionConfig::min_episodic`] precondition, produces lessons
/// (runner when configured, deterministic digest otherwise) and persists the
/// [`ReflectionRun`] under `reflection/<session>/<run_id>`. Pull-based — the
/// owner calls it (same shape as `run_decay_pass` / `TimerScanner::run_once`).
pub fn reflect_session(
    db: &vantadb::sdk::Embedded,
    session_id: &str,
    now_ms: u64,
    config: &ReflectionConfig,
) -> Result<ReflectionRun, ReflectionError> {
    if session_id.is_empty() {
        return Err(ReflectionError::InvalidSessionId(
            "session_id must not be empty".into(),
        ));
    }
    let records = read_session_records(db, session_id)?;
    let episodes: Vec<MemoryRecord> = records
        .into_iter()
        .filter(|r| r.memory_type == MemoryType::Episodic)
        .collect();
    if episodes.len() < config.min_episodic {
        return Err(ReflectionError::NotEnoughMaterial {
            found: episodes.len(),
            required: config.min_episodic,
        });
    }

    let mut source_ids: Vec<String> = episodes.iter().map(|r| r.id.clone()).collect();
    source_ids.sort();
    let runner_label = config
        .runner
        .as_ref()
        .map(|r| r.label().to_string())
        .unwrap_or_else(|| "none".into());

    let lessons = match &config.runner {
        Some(runner) => {
            let ctx = ReflectionContext {
                session_id: session_id.to_string(),
                now_ms,
                config: config.clone(),
            };
            runner
                .reflect(episodes.clone(), &ctx)
                .map_err(ReflectionError::Runner)?
        }
        None => reflect_episodic(&episodes, session_id, now_ms),
    };

    let run = ReflectionRun {
        run_id: generate_run_id(&config.run_id_salt, session_id, now_ms),
        session_id: session_id.to_string(),
        started_at_ms: now_ms,
        ended_at_ms: now_ms,
        episodes_scanned: episodes.len(),
        source_ids,
        runner_label,
        lessons,
    };
    write_reflection_run(db, &run)?;
    Ok(run)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn episode(id: &str, scene: &str, content: &str, priority: i32, updated: &str) -> MemoryRecord {
        MemoryRecord {
            id: id.into(),
            content: content.into(),
            memory_type: MemoryType::Episodic,
            priority,
            scene_name: scene.into(),
            source_message_ids: vec![],
            metadata: serde_json::Value::Null,
            timestamps: vec![],
            created_at: updated.into(),
            updated_at: updated.into(),
            version: 1,
            session_key: "s".into(),
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

    fn persona(id: &str) -> MemoryRecord {
        let mut r = episode(
            id,
            "ui",
            "stable preference",
            80,
            "2026-01-20T10:00:00.000Z",
        );
        r.memory_type = MemoryType::Persona;
        r
    }

    #[test]
    fn reflect_episodic_groups_by_scene_and_ignores_other_types() {
        let records = vec![
            episode("e1", "ops", "one", 40, "2026-01-20T10:00:00.000Z"),
            episode("e2", "ops", "two", 90, "2026-01-20T10:00:00.000Z"),
            episode("e3", "ui", "three", 70, "2026-01-20T10:00:00.000Z"),
            persona("p1"),
        ];
        let lessons = reflect_episodic(&records, "sess-1", 1_769_817_600_000);
        assert_eq!(lessons.len(), 2, "one lesson per scene; persona ignored");
        let ops = lessons.iter().find(|l| l.scene_name == "ops").unwrap();
        assert_eq!(ops.memory_type, MemoryType::WorkMethod);
        assert_eq!(ops.priority, 90, "max priority of the scene");
        assert_eq!(ops.session_key, "sess-1");
        let ids = ops.metadata["reflection"]["source_ids"].as_array().unwrap();
        assert_eq!(ids.len(), 2);
        assert!(ids.iter().any(|v| v == "e1") && ids.iter().any(|v| v == "e2"));
    }

    #[test]
    fn reflect_episodic_digest_uses_top_priority_episodes() {
        let records = vec![
            episode("e1", "ops", "low", 10, "2026-01-20T10:00:00.000Z"),
            episode("e2", "ops", "high", 90, "2026-01-20T10:00:00.000Z"),
            episode("e3", "ops", "mid", 50, "2026-01-20T10:00:00.000Z"),
            episode("e4", "ops", "extra", 20, "2026-01-20T10:00:00.000Z"),
        ];
        let lessons = reflect_episodic(&records, "sess-1", 1_769_817_600_000);
        assert_eq!(lessons.len(), 1);
        let content = &lessons[0].content;
        assert!(content.starts_with("Lesson from 4 episode(s) in ops:"));
        // Top-3 by priority: high (90), mid (50), extra (20) — low (10) is out.
        assert!(content.contains("high") && content.contains("mid") && content.contains("extra"));
        assert!(
            !content.contains("low"),
            "only top-{DIGEST_TOP} in the digest"
        );
    }

    #[test]
    fn reflect_episodic_is_deterministic() {
        let records = vec![
            episode("e1", "ops", "one", 40, "2026-01-20T10:00:00.000Z"),
            episode("e2", "ops", "two", 90, "2026-01-20T10:00:00.000Z"),
        ];
        let first = reflect_episodic(&records, "sess-1", 1_769_817_600_000);
        let second = reflect_episodic(&records, "sess-1", 1_769_817_600_000);
        assert_eq!(first, second, "same input → identical lessons");
    }

    #[test]
    fn reflect_episodic_empty_is_empty() {
        assert!(reflect_episodic(&[], "sess-1", 1_769_817_600_000).is_empty());
        let only_persona = vec![persona("p1")];
        assert!(reflect_episodic(&only_persona, "sess-1", 1_769_817_600_000).is_empty());
    }

    #[test]
    fn config_defaults_are_declared() {
        let config = ReflectionConfig::default();
        assert_eq!(config.min_episodic, 3);
        assert!(config.runner.is_none());
        assert!(config.run_id_salt.is_empty());
        // Clone drops the runner by design (hosts reconstruct explicitly).
        let cloned = config.clone();
        assert!(cloned.runner.is_none());
    }
}
