//! Storage of offloaded tool-call entries per session (port of the core of
//! TDAM `MC/offload/storage.ts`, MEM-20).
//!
//! TDAM appends JSONL lines with write-time dedup by `tool_call_id`. Here
//! each [`OffloadEntry`] is one store record under `offload/<session>`,
//! keyed by its sanitized `tool_call_id` — SDK upsert + get-before-put give
//! the same dedup guarantee without a file rewrite. The refs/MMD/registry
//! layers of TDAM are not ported: VantaDB records replace files, and text
//! sanitization is consolidated in `utils::sanitize` (MEM-19).

use crate::offload::state_manager::OffloadError;
use crate::offload::types::OffloadEntry;
use crate::utils::sanitize::{sanitize_component, sanitize_key};
use vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemoryListOptions, VantaMemoryListPage};

/// Storage of offloaded tool-call summaries over the VantaDB SDK.
pub struct OffloadStorage {
    db: VantaEmbedded,
}

impl OffloadStorage {
    /// Open an offload storage over an already-open embedded database.
    pub fn new(db: VantaEmbedded) -> Self {
        Self { db }
    }

    /// Whether an entry for `tool_call_id` already exists (dedup probe).
    pub fn has_entry(&self, session_id: &str, tool_call_id: &str) -> Result<bool, OffloadError> {
        let record = self
            .db
            .get(&entries_namespace(session_id), &sanitize_key(tool_call_id))?;
        Ok(record.is_some())
    }

    /// Persist one offloaded entry. Returns `false` when an entry with the
    /// same `tool_call_id` already exists — the caller must treat that as a
    /// no-op (idempotency, D19).
    pub fn append_entry(
        &self,
        session_id: &str,
        entry: &OffloadEntry,
    ) -> Result<bool, OffloadError> {
        let key = sanitize_key(&entry.tool_call_id);
        if self.db.get(&entries_namespace(session_id), &key)?.is_some() {
            return Ok(false);
        }
        let payload = serde_json::to_string(entry)?;
        self.db.put(VantaMemoryInput {
            namespace: entries_namespace(session_id),
            key,
            payload,
            metadata: vantadb::sdk::VantaMemoryMetadata::new(),
            vector: None,
            sparse_vector: None,
            ttl_ms: None,
        })?;
        Ok(true)
    }

    /// Read all stored entries for a session (paginated list; order not
    /// guaranteed — callers sort by `timestamp` if they need chronology).
    /// Records whose payload fails to deserialize are skipped with a
    /// warning, never fatal.
    pub fn read_entries(&self, session_id: &str) -> Result<Vec<OffloadEntry>, OffloadError> {
        let ns = entries_namespace(session_id);
        let mut entries = Vec::new();
        let mut cursor: Option<usize> = None;
        loop {
            let options = VantaMemoryListOptions {
                limit: 1000,
                cursor,
                ..Default::default()
            };
            let page: VantaMemoryListPage = self.db.list(&ns, options)?;
            for record in page.records {
                match serde_json::from_str::<OffloadEntry>(&record.payload) {
                    Ok(entry) => entries.push(entry),
                    Err(err) => {
                        tracing::warn!(key = %record.key, "skipping corrupt offload entry: {err}");
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
}

/// `offload/<sanitized-session>` — offloaded-entry records namespace.
pub(crate) fn entries_namespace(session_id: &str) -> String {
    format!("offload/{}", sanitize_component(session_id, 128, false))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_db() -> VantaEmbedded {
        let config = vantadb::config::VantaConfig {
            backend_kind: vantadb::storage::BackendKind::InMemory,
            read_only: false,
            ..vantadb::config::VantaConfig::default()
        };
        VantaEmbedded::open_with_config(config).expect("open in-memory db")
    }

    fn entry(id: &str) -> OffloadEntry {
        OffloadEntry {
            timestamp: "2026-08-20T10:00:00Z".into(),
            node_id: None,
            tool_call: "read_file(path=config.md)".into(),
            summary: "read config".into(),
            result_ref: "results/1.md".into(),
            tool_call_id: id.into(),
            session_key: None,
            score: None,
        }
    }

    #[test]
    fn append_then_read_roundtrip() {
        let storage = OffloadStorage::new(open_db());
        assert!(storage
            .append_entry("s1", &entry("call_a"))
            .expect("append"));
        let all = storage.read_entries("s1").expect("read");
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].tool_call_id, "call_a");
    }

    #[test]
    fn duplicate_tool_call_id_is_not_stored_twice() {
        let storage = OffloadStorage::new(open_db());
        assert!(storage.append_entry("s1", &entry("call_a")).expect("first"));
        assert!(!storage.append_entry("s1", &entry("call_a")).expect("dup"));
        assert_eq!(storage.read_entries("s1").expect("read").len(), 1);
        assert!(storage.has_entry("s1", "call_a").expect("has"));
        assert!(!storage.has_entry("s1", "call_b").expect("missing"));
    }

    #[test]
    fn sessions_are_isolated() {
        let storage = OffloadStorage::new(open_db());
        storage.append_entry("s1", &entry("call_a")).expect("s1");
        assert!(storage.read_entries("s2").expect("s2").is_empty());
    }

    #[test]
    fn corrupt_payload_is_skipped_not_fatal() {
        let db = open_db();
        db.put(VantaMemoryInput {
            namespace: entries_namespace("s1"),
            key: "bad".into(),
            payload: "{corrupt".into(),
            metadata: vantadb::sdk::VantaMemoryMetadata::new(),
            vector: None,
            sparse_vector: None,
            ttl_ms: None,
        })
        .expect("seed");
        let storage = OffloadStorage::new(db);
        storage
            .append_entry("s1", &entry("call_ok"))
            .expect("append");
        let all = storage.read_entries("s1").expect("skip corrupt");
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].tool_call_id, "call_ok");
    }
}
