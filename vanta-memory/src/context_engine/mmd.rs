//! Persistent MMD (current-task memory) storage — MEM-24.
//!
//! The MMD is the agent's working memory for the current task: one active
//! record plus an append-only history, persisted as JSON records under
//! `mmd/<session>/active` and `mmd/<session>/history`. Format decision (D23,
//! closed by the Lead): the META contract [`SceneMeta`] is reused — no
//! Mermaid.
//!
//! Dedup: TDAM `mmd-injector.ts:372-374` fingerprints content as
//! `{len}:{first 64 chars}`; saving an active MMD with the same fingerprint
//! as the stored one is a no-op. Budget: content is capped at
//! [`MAX_MMD_CONTENT_CHARS`] (char-boundary safe) on save.

use crate::context_engine::token_estimator::truncate_content;
use crate::context_engine::types::ContextError;
use crate::core::abstractions::SceneMeta;
use crate::utils::sanitize::{sanitize_component, sanitize_key};
use vantadb::sdk::{VantaEmbedded, VantaMemoryInput};

/// Content ceiling in chars (TDAM ~1300-token guard ≈ 4000 chars at 3
/// chars/token). Enforced on save via char-boundary-safe truncation.
pub const MAX_MMD_CONTENT_CHARS: usize = 4000;

/// Prefix length of the dedup fingerprint (TDAM `mmd-injector.ts:372-374`).
const FINGERPRINT_PREFIX_CHARS: usize = 64;

/// Active-record key inside `mmd/<session>/active`.
const ACTIVE_KEY: &str = "__active";

/// One task's working memory: META contract + narrative content.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TaskMemory {
    /// META contract `{created, updated, summary, heat}` (D23).
    pub meta: SceneMeta,
    /// Task memory body (≤ [`MAX_MMD_CONTENT_CHARS`] after save).
    pub content: String,
}

/// Dedup fingerprint `{len}:{first 64 chars}` (TDAM parity).
pub fn fingerprint(content: &str) -> String {
    let prefix: String = content.chars().take(FINGERPRINT_PREFIX_CHARS).collect();
    format!("{}:{prefix}", content.chars().count())
}

/// Save (or replace) the active MMD for a session. Content over budget is
/// truncated on a char boundary first. If the stored active record has the
/// same fingerprint, this is a no-op (dedup).
pub fn save_active(
    db: &VantaEmbedded,
    session_id: &str,
    memory: &TaskMemory,
) -> Result<(), ContextError> {
    let memory = TaskMemory {
        meta: memory.meta.clone(),
        content: truncate_content(&memory.content, MAX_MMD_CONTENT_CHARS),
    };
    if let Some(existing) = load_active(db, session_id)? {
        if fingerprint(&existing.content) == fingerprint(&memory.content) {
            return Ok(()); // same content → no overwrite
        }
    }
    put_task_memory(db, &active_ns(session_id), ACTIVE_KEY, &memory)
}

/// Load the active MMD for a session. Missing or corrupt record → `None`
/// (never fatal, mirrors the offload state manager).
pub fn load_active(
    db: &VantaEmbedded,
    session_id: &str,
) -> Result<Option<TaskMemory>, ContextError> {
    match db.get(&active_ns(session_id), ACTIVE_KEY)? {
        None => Ok(None),
        Some(record) => match serde_json::from_str::<TaskMemory>(&record.payload) {
            Ok(memory) => Ok(Some(memory)),
            Err(err) => {
                tracing::warn!(session = %session_id, "corrupt active mmd, ignoring: {err}");
                Ok(None)
            }
        },
    }
}

/// Append one MMD to the session history. Stable key derived from content +
/// `meta.updated` (FNV-1a), so re-pushing identical data is idempotent.
pub fn push_history(
    db: &VantaEmbedded,
    session_id: &str,
    memory: &TaskMemory,
) -> Result<(), ContextError> {
    put_task_memory(db, &history_ns(session_id), &history_key(memory), memory)
}

/// Read up to `limit` history entries, oldest → newest. Records whose payload
/// fails to deserialize are skipped with a warning, never fatal.
pub fn list_history(
    db: &VantaEmbedded,
    session_id: &str,
    limit: usize,
) -> Result<Vec<TaskMemory>, ContextError> {
    use vantadb::sdk::{VantaMemoryListOptions, VantaMemoryListPage};
    let ns = history_ns(session_id);
    let mut entries = Vec::new();
    let mut cursor: Option<usize> = None;
    loop {
        let options = VantaMemoryListOptions {
            limit: 1000,
            cursor,
            ..Default::default()
        };
        let page: VantaMemoryListPage = db.list(&ns, options)?;
        for record in page.records {
            match serde_json::from_str::<TaskMemory>(&record.payload) {
                Ok(entry) => entries.push(entry),
                Err(err) => {
                    tracing::warn!(key = %record.key, "skipping corrupt mmd history entry: {err}")
                }
            }
        }
        match page.next_cursor {
            Some(next) => cursor = Some(next),
            None => break,
        }
    }
    // ISO 8601 timestamps sort lexically; stable tiebreak on key order is not
    // needed because keys embed the updated timestamp hash.
    entries.sort_by(|a, b| a.meta.updated.cmp(&b.meta.updated));
    let skip = entries.len().saturating_sub(limit);
    Ok(entries.into_iter().skip(skip).collect())
}

// ─── internals ──────────────────────────────────────────────────────────────

fn put_task_memory(
    db: &VantaEmbedded,
    ns: &str,
    key: &str,
    memory: &TaskMemory,
) -> Result<(), ContextError> {
    db.put(VantaMemoryInput {
        namespace: ns.to_string(),
        key: sanitize_key(key),
        payload: serde_json::to_string(memory)?,
        metadata: vantadb::sdk::VantaMemoryMetadata::new(),
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
    })?;
    Ok(())
}

/// FNV-1a over `content|updated` — stable across reopens, no new deps.
fn history_key(memory: &TaskMemory) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in format!("{}|{}", memory.content, memory.meta.updated).bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("h_{h:016x}")
}

fn active_ns(session_id: &str) -> String {
    format!("mmd/{}/active", sanitize_component(session_id, 128, false))
}

fn history_ns(session_id: &str) -> String {
    format!("mmd/{}/history", sanitize_component(session_id, 128, false))
}

#[cfg(test)]
mod tests {
    use super::*;
    use vantadb::config::VantaConfig;
    use vantadb::storage::BackendKind;

    fn open_db() -> VantaEmbedded {
        let config = VantaConfig {
            backend_kind: BackendKind::InMemory,
            read_only: false,
            ..Default::default()
        };
        VantaEmbedded::open_with_config(config).expect("open in-memory db")
    }

    fn memory(content: &str) -> TaskMemory {
        TaskMemory {
            meta: SceneMeta {
                created: "2026-08-21T10:00:00.000Z".into(),
                updated: "2026-08-21T10:05:00.000Z".into(),
                summary: "test task".into(),
                heat: 1,
            },
            content: content.into(),
        }
    }

    #[test]
    fn fingerprint_is_len_plus_prefix() {
        assert_eq!(fingerprint("hello world"), "11:hello world");
        let long: String = "x".repeat(100);
        assert_eq!(fingerprint(&long), format!("100:{}", "x".repeat(64)));
    }

    #[test]
    fn save_active_dedup_skips_identical_content() {
        let db = open_db();
        save_active(&db, "s1", &memory("task body")).expect("save");
        let stored = load_active(&db, "s1").expect("load").expect("some");
        // Same fingerprint → no-op (no error, still readable).
        save_active(&db, "s1", &memory("task body")).expect("dedup save");
        let again = load_active(&db, "s1").expect("load").expect("some");
        assert_eq!(stored, again);
    }

    #[test]
    fn save_active_truncates_over_budget_on_char_boundary() {
        let db = open_db();
        // Multi-byte chars: 'ñ' is 2 bytes — byte slicing would panic.
        let long = "ñ".repeat(MAX_MMD_CONTENT_CHARS + 50);
        save_active(&db, "s1", &memory(&long)).expect("save");
        let stored = load_active(&db, "s1").expect("load").expect("some");
        assert!(stored.content.chars().count() <= MAX_MMD_CONTENT_CHARS);
    }

    #[test]
    fn namespaces_are_session_scoped_and_sanitized() {
        assert_eq!(active_ns("a/b c"), "mmd/a_b_c/active");
        assert_eq!(history_ns("s1"), "mmd/s1/history");
    }

    #[test]
    fn history_push_list_orders_chronologically() {
        let db = open_db();
        let mut late = memory("late");
        late.meta.updated = "2026-08-21T12:00:00.000Z".into();
        let early = memory("early");
        push_history(&db, "s1", &early).expect("push");
        push_history(&db, "s1", &late).expect("push");
        let listed = list_history(&db, "s1", 10).expect("list");
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].content, "early");
        assert_eq!(listed[1].content, "late");
        // limit keeps the most recent entries.
        let last_one = list_history(&db, "s1", 1).expect("list");
        assert_eq!(last_one.len(), 1);
        assert_eq!(last_one[0].content, "late");
    }

    #[test]
    fn history_push_is_idempotent_for_identical_data() {
        let db = open_db();
        let m = memory("same");
        push_history(&db, "s1", &m).expect("push");
        push_history(&db, "s1", &m).expect("push again");
        assert_eq!(list_history(&db, "s1", 10).expect("list").len(), 1);
    }

    /// D19 (d): persistence survives reopen (new store handle over the same
    /// on-disk backend). Requires the `fjall` feature (persistent backend).
    #[test]
    #[cfg(feature = "fjall")]
    fn persistence_survives_reopen() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().to_string_lossy().to_string();
        let config = || VantaConfig {
            backend_kind: BackendKind::Fjall,
            storage_path: path.clone(),
            read_only: false,
            ..Default::default()
        };
        {
            let db = VantaEmbedded::open_with_config(config()).expect("open 1");
            save_active(&db, "s1", &memory("survives")).expect("save");
            push_history(&db, "s1", &memory("hist")).expect("push");
        } // drop → close
        let db2 = VantaEmbedded::open_with_config(config()).expect("reopen");
        let active = load_active(&db2, "s1").expect("load").expect("survives");
        assert_eq!(active.content, "survives");
        let hist = list_history(&db2, "s1", 10).expect("list");
        assert_eq!(hist.len(), 1);
        assert_eq!(hist[0].content, "hist");
    }
}
