//! Spill-to-disk of compacted content with recall (MEMG-06).
//!
//! The context engine replaces compacted messages with `[compacted N chars]`
//! stubs (see [`crate::context_engine::engine`]); this module persists the
//! full original payload under `spill/<session>` BEFORE the replacement
//! lands, so the compaction is reversible via [`SpillStorage::recall`] /
//! [`SpillStorage::recall_session`]. Mirrors the proven `offload/storage`
//! pattern: SDK records, sanitized keys, get-before-put dedup (D19).
//!
//! Recall is an EXPLICIT operation — never mixed into the L1 recall path
//! (`perform_auto_recall`), per the MEMG-06 pre-mortem.
//!
//! GC: [`SpillStorage::reclaim_as_of`] reuses the offload reclaimer's safety
//! rules ([`MIN_RETENTION_DAYS`] floor, undatable entries are never deleted).
//! The offload cursor gate does NOT apply here: spilled content was already
//! consumed by definition (it was compacted out of the live context), so
//! retention is the only gate.

use serde::{Deserialize, Serialize};

use crate::context_engine::engine::SpillSink;
use crate::context_engine::types::{ChatMessage, ChatRole};
use crate::core::conversation::now_ms;
use crate::core::prompts::l1_extraction::epoch_ms_to_rfc3339;
use crate::offload::reclaimer::{iso_to_epoch_secs, MIN_RETENTION_DAYS, SECS_PER_DAY};
use crate::utils::sanitize::{sanitize_component, sanitize_key};
use vantadb::sdk::{Embedded, MemoryInput, MemoryListOptions, MemoryListPage};

/// One spilled original message payload (record under `spill/<session>`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpilledMessage {
    /// Effective record key: the sanitized message id when present, else a
    /// synthetic content fingerprint (`anon-<fnv1a64>`).
    pub message_id: String,
    /// Session key this payload belongs to.
    pub session_key: String,
    /// Role of the original message.
    pub role: ChatRole,
    /// Full original content, byte-for-byte as compacted.
    pub content: String,
    /// ISO-8601 UTC spill time (datable by the reclaimer rules).
    pub spilled_at: String,
}

/// Errors surfaced by the spill surface. Mirrors `OffloadError`.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SpillError {
    #[error("vantadb: {0}")]
    Vanta(#[from] vantadb::error::Error),
    #[error("malformed spill payload: {0}")]
    Payload(#[from] serde_json::Error),
}

/// Outcome of one reclamation pass over a session.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SpillReclaimStats {
    /// Entries read from the store (corrupt payloads were already skipped
    /// during the read).
    pub scanned: usize,
    /// Entries actually deleted from the store.
    pub deleted: usize,
}

/// Spill storage over the VantaDB SDK.
pub struct SpillStorage {
    db: Embedded,
}

impl SpillStorage {
    /// Open spill storage over an already-open embedded database.
    pub fn new(db: Embedded) -> Self {
        Self { db }
    }

    /// Persist one compacted message's full payload. Returns `false` when an
    /// entry with the same effective key already exists — the caller must
    /// treat that as a no-op (idempotency, D19: first spill wins).
    ///
    /// `message` supplies identity (id/role); `original` is the full
    /// pre-compaction content (the message's own `content` is already the
    /// stub when called from a [`SpillSink`]).
    pub fn spill(
        &self,
        session_id: &str,
        message: &ChatMessage,
        original: &str,
    ) -> Result<bool, SpillError> {
        let key = effective_key(message, original);
        let ns = entries_namespace(session_id);
        if self.db.get(&ns, &key)?.is_some() {
            return Ok(false);
        }
        let entry = SpilledMessage {
            message_id: key.clone(),
            session_key: session_id.to_string(),
            role: message.role,
            content: original.to_string(),
            spilled_at: epoch_ms_to_rfc3339(now_ms()),
        };
        let payload = serde_json::to_string(&entry)?;
        self.db.put(MemoryInput {
            namespace: ns,
            key,
            payload,
            metadata: vantadb::sdk::MemoryMetadata::new(),
            vector: None,
            sparse_vector: None,
            ttl_ms: None,
            ..Default::default()
        })?;
        Ok(true)
    }

    /// Recall one spilled payload by message id (sanitized like `spill`).
    /// Missing or corrupt record → `None` (never fatal).
    pub fn recall(
        &self,
        session_id: &str,
        message_id: &str,
    ) -> Result<Option<SpilledMessage>, SpillError> {
        let record = self
            .db
            .get(&entries_namespace(session_id), &sanitize_key(message_id))?;
        let Some(record) = record else {
            return Ok(None);
        };
        match serde_json::from_str::<SpilledMessage>(&record.payload) {
            Ok(entry) => Ok(Some(entry)),
            Err(err) => {
                tracing::warn!(key = %record.key, "skipping corrupt spill entry: {err}");
                Ok(None)
            }
        }
    }

    /// Recall every spilled payload of a session (paginated list; order not
    /// guaranteed). Records whose payload fails to deserialize are skipped
    /// with a warning, never fatal.
    pub fn recall_session(&self, session_id: &str) -> Result<Vec<SpilledMessage>, SpillError> {
        let ns = entries_namespace(session_id);
        let mut entries = Vec::new();
        let mut cursor: Option<usize> = None;
        loop {
            let options = MemoryListOptions {
                limit: 1000,
                cursor,
                ..Default::default()
            };
            let page: MemoryListPage = self.db.list(&ns, options)?;
            for record in page.records {
                match serde_json::from_str::<SpilledMessage>(&record.payload) {
                    Ok(entry) => entries.push(entry),
                    Err(err) => {
                        tracing::warn!(key = %record.key, "skipping corrupt spill entry: {err}");
                    }
                }
            }
            match page.next_cursor {
                Some(next) => cursor = Some(next),
                None => break,
            }
        }
        Ok(entries)
    }

    /// Run a reclamation pass using the current wall clock.
    pub fn reclaim(
        &self,
        session_id: &str,
        retention_days: u64,
    ) -> Result<SpillReclaimStats, SpillError> {
        let now_secs = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(d) => d.as_secs() as i64,
            Err(err) => {
                tracing::warn!("spill reclaim skipped, system clock before epoch: {err}");
                return Ok(SpillReclaimStats::default());
            }
        };
        self.reclaim_as_of(session_id, retention_days, now_secs)
    }

    /// Testable core of [`Self::reclaim`]: delete entries older than
    /// `retention_days` as of `now_secs`. Same safety rules as the offload
    /// reclaimer: retention below [`MIN_RETENTION_DAYS`] disables the pass;
    /// entries with unparseable timestamps are never deleted.
    pub fn reclaim_as_of(
        &self,
        session_id: &str,
        retention_days: u64,
        now_secs: i64,
    ) -> Result<SpillReclaimStats, SpillError> {
        if retention_days < MIN_RETENTION_DAYS {
            tracing::debug!(
                retention_days,
                "spill reclaim skipped: retention below minimum ({MIN_RETENTION_DAYS})"
            );
            return Ok(SpillReclaimStats::default());
        }
        let entries = self.recall_session(session_id)?;
        let mut stats = SpillReclaimStats {
            scanned: entries.len(),
            deleted: 0,
        };
        let cutoff = now_secs - retention_days as i64 * SECS_PER_DAY;
        let ns = entries_namespace(session_id);
        for entry in &entries {
            let Some(ts) = iso_to_epoch_secs(&entry.spilled_at) else {
                tracing::warn!(
                    message_id = %entry.message_id,
                    "spill reclaim: skipping entry with unparseable timestamp"
                );
                continue;
            };
            if ts < cutoff {
                self.db.delete(&ns, &entry.message_id)?;
                stats.deleted += 1;
            }
        }
        Ok(stats)
    }
}

/// [`SpillSink`] persisting each replaced message into `spill/<session>`.
/// Persistence failures are logged and swallowed: the assembled context is
/// the primary artifact, the spill is the recovery copy.
pub struct DbSpillSink<'a> {
    storage: &'a SpillStorage,
    session_id: &'a str,
    spilled: usize,
}

impl<'a> DbSpillSink<'a> {
    /// Sink over a spill storage for one session.
    pub fn new(storage: &'a SpillStorage, session_id: &'a str) -> Self {
        Self {
            storage,
            session_id,
            spilled: 0,
        }
    }

    /// Messages newly persisted by this sink (deduped spills don't count).
    pub fn spilled_count(&self) -> usize {
        self.spilled
    }
}

impl SpillSink for DbSpillSink<'_> {
    fn spill(&mut self, message: &ChatMessage, original: &str) {
        match self.storage.spill(self.session_id, message, original) {
            Ok(true) => self.spilled += 1,
            Ok(false) => {}
            Err(err) => tracing::warn!(
                session = %self.session_id,
                %err,
                "spill persist failed; compaction continues"
            ),
        }
    }
}

/// `spill/<sanitized-session>` — spilled-payload records namespace.
fn entries_namespace(session_id: &str) -> String {
    format!("spill/{}", sanitize_component(session_id, 128, false))
}

/// Effective record key: sanitized message id when present and non-empty,
/// else `anon-<fnv1a64>` over the full content (stable across processes and
/// Rust versions — no std-hasher guarantees needed).
fn effective_key(message: &ChatMessage, original: &str) -> String {
    if let Some(id) = message.id.as_deref() {
        let sanitized = sanitize_key(id);
        if !sanitized.is_empty() {
            return sanitized;
        }
    }
    format!("anon-{:016x}", fnv1a64(original))
}

/// FNV-1a 64-bit (offset basis / prime constants, public domain).
fn fnv1a64(bytes: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use vantadb::config::Config;
    use vantadb::storage::BackendKind;

    fn open_db() -> Embedded {
        Embedded::open_with_config(Config {
            backend_kind: BackendKind::InMemory,
            read_only: false,
            ..Config::default()
        })
        .expect("open in-memory db")
    }

    fn msg(id: Option<&str>, role: ChatRole, content: &str) -> ChatMessage {
        let mut m = ChatMessage::new(role, content);
        m.id = id.map(str::to_string);
        m
    }

    /// Seed one record directly (bypasses `spill`) so timestamps are
    /// controllable for the GC tests.
    fn seed(db: &Embedded, session: &str, entry: &SpilledMessage) {
        db.put(MemoryInput {
            namespace: entries_namespace(session),
            key: entry.message_id.clone(),
            payload: serde_json::to_string(entry).expect("serialize"),
            metadata: vantadb::sdk::MemoryMetadata::new(),
            vector: None,
            sparse_vector: None,
            ttl_ms: None,
            ..Default::default()
        })
        .expect("seed");
    }

    fn entry(id: &str, ts: &str) -> SpilledMessage {
        SpilledMessage {
            message_id: id.into(),
            session_key: "s1".into(),
            role: ChatRole::User,
            content: format!("payload of {id}"),
            spilled_at: ts.into(),
        }
    }

    #[test]
    fn spill_then_recall_by_id_round_trips_full_content() {
        let storage = SpillStorage::new(open_db());
        let original = "full payload ".repeat(50);
        let m = msg(Some("msg-1"), ChatRole::User, "[compacted 650 chars]");
        assert!(storage.spill("s1", &m, &original).expect("spill"));
        let back = storage
            .recall("s1", "msg-1")
            .expect("recall")
            .expect("present");
        assert_eq!(
            back.content, original,
            "recall must return the full payload"
        );
        assert_eq!(back.role, ChatRole::User);
        assert_eq!(back.message_id, "msg-1");
        assert_eq!(back.session_key, "s1");
        assert!(!back.spilled_at.is_empty());
    }

    #[test]
    fn recall_by_id_is_none_for_unknown() {
        let storage = SpillStorage::new(open_db());
        assert!(storage.recall("s1", "nope").expect("recall").is_none());
    }

    #[test]
    fn recall_session_lists_all_and_sessions_are_isolated() {
        let storage = SpillStorage::new(open_db());
        storage
            .spill("s1", &msg(Some("a"), ChatRole::User, "stub"), "content a")
            .expect("a");
        storage
            .spill(
                "s1",
                &msg(Some("b"), ChatRole::Assistant, "stub"),
                "content b",
            )
            .expect("b");
        storage
            .spill("s2", &msg(Some("c"), ChatRole::User, "stub"), "content c")
            .expect("c");
        let s1 = storage.recall_session("s1").expect("s1");
        assert_eq!(s1.len(), 2);
        let s2 = storage.recall_session("s2").expect("s2");
        assert_eq!(s2.len(), 1);
        assert_eq!(s2[0].content, "content c");
    }

    #[test]
    fn duplicate_id_is_deduped_first_wins() {
        let storage = SpillStorage::new(open_db());
        let m = msg(Some("dup"), ChatRole::User, "stub");
        assert!(storage.spill("s1", &m, "first version").expect("first"));
        assert!(!storage.spill("s1", &m, "second version").expect("dup"));
        assert_eq!(storage.recall_session("s1").expect("all").len(), 1);
        assert_eq!(
            storage
                .recall("s1", "dup")
                .expect("recall")
                .unwrap()
                .content,
            "first version"
        );
    }

    #[test]
    fn anonymous_messages_get_content_stable_key() {
        let storage = SpillStorage::new(open_db());
        let anon = msg(None, ChatRole::User, "stub");
        assert!(storage
            .spill("s1", &anon, "same anonymous content")
            .expect("first"));
        // Same content, no id → same synthetic key → deduped.
        assert!(!storage
            .spill("s1", &anon, "same anonymous content")
            .expect("dup"));
        assert_eq!(storage.recall_session("s1").expect("all").len(), 1);
        // Different content → different key → stored.
        assert!(storage
            .spill("s1", &anon, "other anonymous content")
            .expect("other"));
        assert_eq!(storage.recall_session("s1").expect("all").len(), 2);
    }

    #[test]
    fn corrupt_payload_is_skipped_not_fatal() {
        let db = open_db();
        db.put(MemoryInput {
            namespace: entries_namespace("s1"),
            key: "bad".into(),
            payload: "{corrupt".into(),
            metadata: vantadb::sdk::MemoryMetadata::new(),
            vector: None,
            sparse_vector: None,
            ttl_ms: None,
            ..Default::default()
        })
        .expect("seed corrupt");
        let storage = SpillStorage::new(db);
        storage
            .spill("s1", &msg(Some("ok"), ChatRole::User, "stub"), "good")
            .expect("ok");
        let all = storage.recall_session("s1").expect("skip corrupt");
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].message_id, "ok");
        assert!(storage.recall("s1", "bad").expect("recall").is_none());
    }

    #[test]
    fn reclaim_removes_only_entries_past_retention() {
        let db = open_db();
        let storage = SpillStorage::new(db.clone());
        seed(&db, "s1", &entry("old", "2026-01-01T00:00:00Z"));
        seed(&db, "s1", &entry("recent", "2026-10-04T00:00:00Z"));
        seed(&db, "s1", &entry("undatable", "not-a-date"));

        // now = 2026-10-05; retention 3d → cutoff 2026-10-02.
        let now = iso_to_epoch_secs("2026-10-05T00:00:00Z").expect("parse");
        let stats = storage.reclaim_as_of("s1", 3, now).expect("reclaim");
        assert_eq!(stats.scanned, 3);
        assert_eq!(stats.deleted, 1);
        assert!(storage.recall("s1", "old").expect("old").is_none());
        assert!(storage.recall("s1", "recent").expect("recent").is_some());
        assert!(
            storage
                .recall("s1", "undatable")
                .expect("undatable")
                .is_some(),
            "undatable entries are never deleted"
        );
    }

    #[test]
    fn reclaim_below_min_retention_is_noop() {
        let db = open_db();
        let storage = SpillStorage::new(db.clone());
        seed(&db, "s1", &entry("old", "2026-01-01T00:00:00Z"));
        let now = iso_to_epoch_secs("2026-10-05T00:00:00Z").expect("parse");
        let stats = storage
            .reclaim_as_of("s1", MIN_RETENTION_DAYS - 1, now)
            .expect("reclaim");
        assert_eq!(stats.deleted, 0);
        assert!(storage.recall("s1", "old").expect("old").is_some());
    }

    #[test]
    fn db_spill_sink_persists_and_counts_deduped() {
        let db = open_db();
        let storage = SpillStorage::new(db);
        let mut sink = DbSpillSink::new(&storage, "s1");
        let original = "payload ".repeat(100);
        let m = msg(Some("m1"), ChatRole::Assistant, "[compacted 800 chars]");
        sink.spill(&m, &original);
        sink.spill(&m, &original); // dedup → not counted twice
        assert_eq!(sink.spilled_count(), 1);
        assert_eq!(
            storage.recall("s1", "m1").expect("recall").unwrap().content,
            original
        );
    }

    #[test]
    fn namespaces_and_keys_are_sanitized() {
        assert_eq!(entries_namespace("a/b c"), "spill/a_b_c");
        let storage = SpillStorage::new(open_db());
        let m = msg(Some("id/weird\n"), ChatRole::User, "stub");
        assert!(storage.spill("s/1", &m, "payload").expect("spill"));
        assert!(storage
            .recall("s/1", "id/weird\n")
            .expect("recall")
            .is_some());
    }
}
