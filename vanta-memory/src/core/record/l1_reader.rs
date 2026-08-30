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
    read_namespace_records(db, &l1_namespace(session_key))
}

/// Read all persisted L1 records from an explicit `l1/<session>` namespace
/// (used by cross-session recall, MEM-40, where the namespace is already
/// resolved — e.g. enumerated via `list_namespaces`).
pub fn read_namespace_records(
    db: &vantadb::sdk::VantaEmbedded,
    namespace: &str,
) -> Result<Vec<MemoryRecord>, L1Error> {
    use vantadb::sdk::{VantaMemoryListOptions, VantaMemoryListPage};

    let mut records = Vec::new();
    let mut cursor: Option<usize> = None;

    loop {
        let options = VantaMemoryListOptions {
            limit: 1000,
            cursor,
            ..Default::default()
        };
        let page: VantaMemoryListPage = db.list(namespace, options)?;
        for record in page.records {
            if let Ok(mut mem) = serde_json::from_str::<MemoryRecord>(&record.payload) {
                mem.vector = usable_vector_filter(record.vector.as_deref());
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
            let mut mem: MemoryRecord = serde_json::from_str(&record.payload)?;
            mem.vector = usable_vector_filter(record.vector.as_deref());
            Ok(Some(mem))
        }
        None => Ok(None),
    }
}

/// Attach the node vector to a record, treating empty/all-zero vectors (how
/// the SDK reports "no vector") as `None` (MEM-46 learning; mirrors
/// `usable_vector` in the core SDK).
pub(crate) fn usable_vector_filter(vector: Option<&[f32]>) -> Option<Vec<f32>> {
    vector
        .filter(|v| !v.is_empty() && v.iter().any(|&x| x != 0.0))
        .map(<[f32]>::to_vec)
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

/// LLM-free candidate recall: rank existing records against the new memory's
/// content and return the top `k` (MEM-11).
///
/// D38 dual-pool (MEM-47): records WITH a usable vector are scored by cosine
/// similarity against the embedded content (semantic pool); records WITHOUT
/// one keep the keyword-overlap gate — a legacy record is never dropped just
/// because it has no vector. Both pools fuse via reciprocal-rank fusion, so
/// overlap counts and cosine similarities never compete directly.
///
/// # ponytail: full scan per new memory
/// O(records × dims) per call; fine at session-sized pools. Upgrade path:
/// HNSW query via the SDK if dedup ever needs cross-session reach.
pub fn recall_candidates(
    records: &[MemoryRecord],
    content: &str,
    top_k: usize,
    embed: Option<&crate::core::record::l1_writer::EmbedFn>,
) -> Vec<MemoryRecord> {
    let query_vector = embed.and_then(|hook| hook(content));

    // Pool 1 — legacy keyword overlap (doubles as the no-vector fallback).
    let mut keyword_pool: Vec<(usize, &MemoryRecord)> = records
        .iter()
        .map(|r| (overlap_score(&r.content, content), r))
        .filter(|(score, _)| *score > 0)
        .collect();
    keyword_pool.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| b.1.updated_at.cmp(&a.1.updated_at))
    });

    // Pool 2 — semantic similarity over records that carry a usable vector.
    let vector_pool: Vec<(f32, &MemoryRecord)> = match query_vector.as_deref() {
        Some(query) => records
            .iter()
            .filter_map(|r| {
                let record_vector = r.vector.as_deref()?;
                cosine_similarity(record_vector, query)
                    .filter(|sim| *sim >= MIN_COSINE_SIMILARITY)
                    .map(|sim| (sim, r))
            })
            .collect(),
        None => Vec::new(),
    };

    if vector_pool.is_empty() {
        // No semantic hits: byte-identical legacy ordering (regression-safe).
        return keyword_pool
            .into_iter()
            .take(top_k)
            .map(|(_, r)| r.clone())
            .collect();
    }
    let mut sorted_vector_pool = vector_pool;
    sorted_vector_pool.sort_by(|a, b| {
        b.0.total_cmp(&a.0)
            .then_with(|| b.1.updated_at.cmp(&a.1.updated_at))
    });

    let keyword_ids: Vec<String> = keyword_pool.iter().map(|(_, r)| r.id.clone()).collect();
    let vector_ids: Vec<String> = sorted_vector_pool
        .iter()
        .map(|(_, r)| r.id.clone())
        .collect();
    rrf_merge(&keyword_ids, &vector_ids, top_k)
        .into_iter()
        .filter_map(|id| records.iter().find(|r| r.id == id))
        .cloned()
        .collect()
}

/// Minimum cosine similarity for a stored vector to enter the semantic pool
/// (D38). Below the threshold the record simply stays with its keyword score.
pub(crate) const MIN_COSINE_SIMILARITY: f32 = 0.35;

/// Cosine similarity; `None` on dimension mismatch or zero norm (mirrors the
/// core SDK's zero-norm guard: an unusable vector never ranks).
pub(crate) fn cosine_similarity(a: &[f32], b: &[f32]) -> Option<f32> {
    if a.is_empty() || a.len() != b.len() {
        return None;
    }
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b = b.iter().map(|y| y * y).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return None;
    }
    Some(dot / (norm_a * norm_b))
}

/// Reciprocal-rank fusion (k=60, Cormack et al. 2009) of two ranked pools.
/// Inputs are ordered ids (rank 1 first); returns the top `top_k` fused ids.
/// Rank-based gains make heterogeneous scores (term counts vs cosine) merge
/// without normalization. Stable sort keeps keyword-pool order on ties.
pub(crate) fn rrf_merge(
    keyword_ranked: &[String],
    vector_ranked: &[String],
    top_k: usize,
) -> Vec<String> {
    const RRF_K: f32 = 60.0;
    let mut fused: Vec<(String, f32)> = Vec::new();
    let mut bump = |ranked: &[String]| {
        for (idx, id) in ranked.iter().enumerate() {
            let gain = 1.0 / (RRF_K + idx as f32 + 1.0);
            match fused.iter_mut().find(|(existing, _)| existing == id) {
                Some((_, score)) => *score += gain,
                None => fused.push((id.clone(), gain)),
            }
        }
    };
    bump(keyword_ranked);
    bump(vector_ranked);
    fused.sort_by(|a, b| b.1.total_cmp(&a.1));
    fused.into_iter().take(top_k).map(|(id, _)| id).collect()
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
            vector: None,
            heat: 0,
            superseded_by: None,
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
        let candidates = recall_candidates(&records, "user prefers dark mode", 2, None);
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].id, "m1");
        assert_eq!(candidates[1].id, "m2");

        let none = recall_candidates(&records, "rust cargo build", 5, None);
        assert!(none.is_empty());
    }
}
