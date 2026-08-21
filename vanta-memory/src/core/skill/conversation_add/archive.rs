//! Archive store + trigger (MEM-17) — port of TDAM
//! `conversation-add/{buffer-storage,trigger-service,prepare-archive}.ts`,
//! consolidated: persistence is VantaDB records (Principio 2), never COS
//! JSONL; the Redis tasks-mutex collapses to single-record read-modify-write.
//!
//! **Ordering invariant** (TDAM incident 2026-07-20): the archive record is
//! written BEFORE the task entry. A worker only ever sees a task whose
//! archive already exists — a slow/failed archive write can never strand a
//! ghost task pointing at nothing.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::core::conversation::l0_recorder::{now_ms, sanitize_component, sanitize_key};
use vantadb::error::VantaError;
use vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemoryMetadata, VantaValue};

use super::compressor::{CompressOptions, SkillMessage};
use super::oversize::{apply_oversize_strategy, OversizeOptions};

/// Errors surfaced by the skill archive/task registry.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SkillArchiveError {
    #[error("vantadb: {0}")]
    Vanta(#[from] VantaError),
    #[error("malformed skill archive payload: {0}")]
    Serde(#[from] serde_json::Error),
}

/// One archived skill-extraction task (persisted in the tasks namespace).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillTaskEntry {
    pub task_id: String,
    pub session_id: String,
    /// Archive record key (resolvable in the archive namespace).
    pub archive_key: String,
    pub archived_at_ms: u64,
    /// `pending` | `done` | `dropped`.
    pub status: String,
}

/// Result of a successful [`trigger_archive`] call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerResult {
    pub task_id: String,
    pub archive_key: String,
    pub archived_at_ms: u64,
}

/// Outcome of [`prepare_archive_payload`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedPayload {
    pub messages: Vec<SkillMessage>,
    pub used_compress: bool,
    pub used_oversize: bool,
}

fn session_component(session_id: &str) -> String {
    sanitize_component(session_id, 64, false)
}

/// Namespace holding archive payloads for one session.
pub fn archive_namespace(session_id: &str) -> String {
    format!("skill_archive/{}", session_component(session_id))
}

/// Namespace holding task entries for one session.
pub fn tasks_namespace(session_id: &str) -> String {
    format!("skill_tasks/{}", session_component(session_id))
}

/// Assemble the archive payload: compress tool payloads, then apply the
/// oversize fallback when the combined buffer still exceeds the chunk budget
/// (TDAM `prepareArchivePayload`; oversize only fires on the compress path).
pub fn prepare_archive_payload(
    existing: &[SkillMessage],
    incoming: &[SkillMessage],
    force_compress: bool,
    compress: &CompressOptions,
    oversize: &OversizeOptions,
) -> PreparedPayload {
    let compressed = super::compressor::compress_messages(incoming, compress);
    // "used" = something actually changed, not just the flag being set.
    let used_compress = force_compress && compressed.iter().zip(incoming).any(|(c, o)| c != o);

    let mut combined = existing.to_vec();
    combined.extend(compressed);

    if !force_compress {
        return PreparedPayload {
            messages: combined,
            used_compress,
            used_oversize: false,
        };
    }
    let result = apply_oversize_strategy(&combined, oversize);
    PreparedPayload {
        used_oversize: result.truncated,
        messages: result.messages,
        used_compress,
    }
}

/// Persistent archive + task registry over the VantaDB SDK.
pub struct ArchiveStore<'a> {
    db: &'a VantaEmbedded,
}

impl<'a> ArchiveStore<'a> {
    pub fn new(db: &'a VantaEmbedded) -> Self {
        Self { db }
    }

    fn put_json(
        &self,
        ns: &str,
        key: &str,
        value: &impl Serialize,
    ) -> Result<(), SkillArchiveError> {
        let mut metadata = VantaMemoryMetadata::new();
        metadata.insert("kind".into(), VantaValue::String("skill_archive".into()));
        self.db.put(VantaMemoryInput {
            namespace: ns.to_string(),
            key: sanitize_key(key),
            payload: serde_json::to_string(value)?,
            metadata,
            vector: None,
            sparse_vector: None,
            ttl_ms: None,
        })?;
        Ok(())
    }

    fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        ns: &str,
        key: &str,
    ) -> Result<Option<T>, SkillArchiveError> {
        match self.db.get(ns, &sanitize_key(key))? {
            Some(record) => Ok(Some(serde_json::from_str(&record.payload)?)),
            None => Ok(None),
        }
    }

    /// Write an archive payload. Idempotent per `(session, archived_at_ms)`:
    /// re-writing the same key overwrites with equivalent content.
    pub fn write_archive(
        &self,
        session_id: &str,
        archived_at_ms: u64,
        messages: &[SkillMessage],
    ) -> Result<String, SkillArchiveError> {
        let key = format!("{archived_at_ms}");
        self.put_json(&archive_namespace(session_id), &key, &messages)?;
        Ok(key)
    }

    /// Read back an archive payload (`None` = ghost reference).
    pub fn read_archive(
        &self,
        session_id: &str,
        archive_key: &str,
    ) -> Result<Option<Vec<SkillMessage>>, SkillArchiveError> {
        self.get_json(&archive_namespace(session_id), archive_key)
    }

    /// Register a task entry (called AFTER the archive exists).
    pub fn register_task(&self, entry: &SkillTaskEntry) -> Result<(), SkillArchiveError> {
        self.put_json(&tasks_namespace(&entry.session_id), &entry.task_id, entry)
    }

    /// Read a task entry by id.
    pub fn read_task(
        &self,
        session_id: &str,
        task_id: &str,
    ) -> Result<Option<SkillTaskEntry>, SkillArchiveError> {
        self.get_json(&tasks_namespace(session_id), task_id)
    }

    /// Persist a status transition for a task entry.
    pub fn set_task_status(
        &self,
        entry: &SkillTaskEntry,
        status: &str,
    ) -> Result<SkillTaskEntry, SkillArchiveError> {
        let mut updated = entry.clone();
        updated.status = status.to_string();
        self.register_task(&updated)?;
        Ok(updated)
    }
}

/// Trigger one archive (TDAM `SkillTriggerService.archive`, §7.4 order):
/// ① derive ids → ② write archive FIRST → ③ register the task entry.
///
/// `task_id` is deterministic per trigger instant
/// (`skill-extract-task-{archived_at_ms}`): no uuid dependency, and a client
/// retry at the same millisecond lands on the same idempotent records.
pub fn trigger_archive(
    db: &VantaEmbedded,
    session_id: &str,
    buffer_at_trigger: &[SkillMessage],
    archived_at_ms: u64,
) -> Result<TriggerResult, SkillArchiveError> {
    let store = ArchiveStore::new(db);
    let archive_key = store.write_archive(session_id, archived_at_ms, buffer_at_trigger)?;
    let task_id = format!("skill-extract-task-{archived_at_ms}");
    store.register_task(&SkillTaskEntry {
        task_id: task_id.clone(),
        session_id: session_id.to_string(),
        archive_key,
        archived_at_ms,
        status: "pending".into(),
    })?;
    Ok(TriggerResult {
        task_id,
        archive_key: format!("{archived_at_ms}"),
        archived_at_ms,
    })
}

/// Convenience wrapper using the wall clock (tests inject their own ms via
/// [`trigger_archive`] directly).
pub fn trigger_archive_now(
    db: &VantaEmbedded,
    session_id: &str,
    buffer_at_trigger: &[SkillMessage],
) -> Result<TriggerResult, SkillArchiveError> {
    trigger_archive(db, session_id, buffer_at_trigger, now_ms())
}
