//! L1 memory reader + LLM-free candidate recall (MEM-11).
//!
//! Phase 1 of the two-phase dedup: read the persisted L1 records of a session
//! and recall top-k candidate pools per new memory WITHOUT an LLM call
//! (Principio 4 — recall is optional; when it yields nothing, the pipeline
//! stores everything).
//!
//! Persistence goes through the VantaDB SDK (Principio 2): records live under
//! the `l1/<session>` namespace, key = sanitized record id, payload = the
//! serialized [`MemoryRecord`].

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::core::abstractions::MemoryRecord;
use crate::core::conversation::{sanitize_component, sanitize_key};
use crate::core::record::L1Error;

/// `l1/<sanitized-session>` — persisted L1 memory records namespace.
pub fn l1_namespace(session_key: &str) -> String {
    format!("l1/{}", sanitize_component(session_key, 128, false))
}

/// Read all persisted L1 records of a session (idempotent, paged via list).
pub fn read_session_records(
    db: &vantadb::sdk::VantaEmbedded,
    session_key: &str,
) -> Result<Vec<MemoryRecord>, L1Error> {
    use vantadb::sdk::{VantaMemoryListOptions, VantaMemoryListPage};

    let ns = l1_namespace(session_key);
    let mut records = Vec::new();
    let mut cursor: Option<usize> = None;

    loop {
        let options = VantaMemoryListOptions {
            limit: 1000,
            cursor,
            ..Default::default()
        };
        let page: VantaMemoryListPage = db.list(&ns, options)?;
        for record in page.records {
            if let Ok(mem) = serde_json::from_str::<MemoryRecord>(&record.payload) {
                records.push(mem);
            } else {
                tracing::debug!(key = %record.key, "l1 record failed to deserialize; skipped");
            }
        }
        match page.next_cursor {
            Some(next) => cursor = Some(next),
            None => break,
        }
    }

    Ok(records)
}

/// Read a single L1 record by id, if present.
pub fn read_record(
    db: &vantadb::sdk::VantaEmbedded,
    session_key: &str,
    record_id: &str,
) -> Result<Option<MemoryRecord>, L1Error> {
    let ns = l1_namespace(session_key);
    let key = sanitize_key(record_id);
    match db.get(&ns, &key)? {
        Some(record) => {
            let mem = serde_json::from_str(&record.payload)?;
            Ok(Some(mem))
        }
        None => Ok(None),
    }
}

/// Tokenize content into a lowercased set of significant terms (words of ≥3
/// chars). Stopword-free on purpose: the heuristic is a cheap recall gate, not
/// a ranking engine — see [`recall_candidates`] for the documented ceiling.
pub(crate) fn significant_terms(content: &str) -> HashSet<String> {
    content
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .map(str::to_lowercase)
        .filter(|t| t.chars().count() >= 3)
        .collect()
}

/// Overlap score between two contents (count of shared significant terms).
pub(crate) fn overlap_score(a: &str, b: &str) -> usize {
    let a_terms = significant_terms(a);
    let b_terms = significant_terms(b);
    a_terms.intersection(&b_terms).count()
}

/// LLM-free candidate recall: rank existing records by keyword overlap with
/// the new memory and return the top `k`.
///
/// # ponytail: naive overlap recall, no vector search
/// `vanta-memory` has no LLM-free embeddings, so this is a cheap token-overlap
/// gate. Same degradation as TDAM when vector/FTS are unavailable
/// (l1-dedup.ts:89-97 skips dedup → store-all). Upgrade path: wire the
/// VantaDB vector index once the core search API is exposed to this crate.
pub fn recall_candidates(
    records: &[MemoryRecord],
    content: &str,
    top_k: usize,
) -> Vec<MemoryRecord> {
    let mut scored: Vec<(usize, &MemoryRecord)> = records
        .iter()
        .map(|r| (overlap_score(&r.content, content), r))
        .filter(|(score, _)| *score > 0)
        .collect();
    // Higher overlap first; ties break by newest updated_at (stable-ish).
    scored.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| b.1.updated_at.cmp(&a.1.updated_at))
    });
    scored
        .into_iter()
        .take(top_k)
        .map(|(_, r)| r.clone())
        .collect()
}

/// Marker types so the reader surface is self-describing in docs/tests.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct L1ReaderStats {
    pub session_key: String,
    pub record_count: usize,
}

#[cfg(test)]
mod tests {
    use super::{l1_namespace, overlap_score, recall_candidates, significant_terms};
    use crate::core::abstractions::{MemoryRecord, MemoryType};

    fn record(id: &str, content: &str, updated: &str) -> MemoryRecord {
        MemoryRecord {
            id: id.into(),
            content: content.into(),
            memory_type: MemoryType::Persona,
            priority: 80,
            scene_name: "s".into(),
            source_message_ids: vec![],
            metadata: serde_json::Value::Null,
            timestamps: vec![updated.into()],
            created_at: updated.into(),
            updated_at: updated.into(),
            version: 1,
            session_key: "sk".into(),
            session_id: "".into(),
            task_id: None,
            team_id: None,
            user_id: None,
            agent_id: None,
        }
    }

    #[test]
    fn namespace_sanitizes_session() {
        assert_eq!(l1_namespace("session a/b"), "l1/session_a_b");
        assert_eq!(l1_namespace("plain"), "l1/plain");
    }

    #[test]
    fn significant_terms_ignore_short_words_and_case() {
        let terms = significant_terms("Deploy the APP and run tests");
        assert!(terms.contains("deploy"));
        assert!(terms.contains("tests"));
        assert!(terms.contains("the")); // 3 chars -> kept (>=3)
        assert!(terms.contains("and")); // 3 chars -> kept (>=3)
        assert!(terms.contains("app")); // 3 chars -> kept (>=3)
        assert!(!terms.contains("to")); // 2 chars -> dropped (<3)
    }

    #[test]
    fn overlap_scores_shared_terms() {
        // Shared significant terms: {user, dark, mode} = 3.
        assert_eq!(
            overlap_score("user prefers dark mode", "user likes dark mode"),
            3
        );
        assert_eq!(
            overlap_score("completely different", "no shared words here"),
            0
        );
    }

    #[test]
    fn recall_ranks_by_overlap_and_respects_top_k() {
        let records = vec![
            record(
                "m1",
                "user prefers dark mode and vim",
                "2026-08-20T10:00:00.000Z",
            ),
            record("m2", "user prefers light theme", "2026-08-20T10:00:00.000Z"),
            record("m3", "team uses postgres", "2026-08-20T10:00:00.000Z"),
        ];
        let candidates = recall_candidates(&records, "user prefers dark mode", 2);
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].id, "m1");
        assert_eq!(candidates[1].id, "m2");

        let none = recall_candidates(&records, "rust cargo build", 5);
        assert!(none.is_empty());
    }
}
