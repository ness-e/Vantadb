//! SkillCoreSink (MEM-17) — persists extracted skill candidates.
//!
//! Port of TDAM `conversation-add/skill-core-sink.ts` with the contract its
//! doc comment demands: **"Must be idempotent (client retries may cause a
//! task to be extracted multiple times)."** TDAM's sink was a no-op asset
//! register because its extractor wrote through tool calls; this port's
//! pure-text extractor only *emits* candidates, so the sink IS the writer.
//!
//! Idempotency is double-layered:
//! 1. **Per-task cursor** (`{task_id}__applied` in the cursor namespace):
//!    re-applying an already-applied task is a no-op (MEM-09 L0 cursor
//!    pattern).
//! 2. **Content-hash upsert**: a candidate whose content hash matches the
//!    stored record is skipped — same semantics as MEM-06
//!    `SkillStore::create` (`idempotent = true` on equal content-hash).
//!
//! Integration note: vanta-memory only holds [`VantaEmbedded`] (the core
//! `SkillStore` needs `&StorageEngine`, not exposed by the SDK), so skills
//! land in the `skills_extract/{scope}` namespace with the same logical
//! fields (name/description/content/content_hash). Wiring against MEM-06's
//! store happens at the data plane (MEM-35/07).

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::core::conversation::l0_recorder::{sanitize_component, sanitize_key};
use vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemoryMetadata, VantaValue};

use super::archive::SkillArchiveError;
use crate::core::skill::skill_extractor::ExtractedSkillCandidate;

/// A persisted skill record (logical parity with MEM-06 `SkillRecord`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredSkill {
    pub name: String,
    pub description: String,
    pub content: String,
    pub content_hash: u64,
    pub updated_at_ms: u64,
}

/// Per-apply counters (what the run actually did).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SkillSinkCounts {
    /// New names written.
    pub created: usize,
    /// Existing names whose content changed.
    pub updated: usize,
    /// Candidates skipped because content-hash matched (or invalid).
    pub unchanged: usize,
}

/// Idempotent sink over the VantaDB SDK.
pub struct SkillCoreSink<'a> {
    db: &'a VantaEmbedded,
}

impl<'a> SkillCoreSink<'a> {
    pub fn new(db: &'a VantaEmbedded) -> Self {
        Self { db }
    }

    fn skills_ns(scope: &str) -> String {
        format!("skills_extract/{}", sanitize_component(scope, 64, false))
    }

    fn cursor_ns(scope: &str) -> String {
        format!(
            "skill_extract_cursor/{}",
            sanitize_component(scope, 64, false)
        )
    }

    /// Read a stored skill by name (test/introspection surface).
    pub fn read_skill(
        &self,
        scope: &str,
        name: &str,
    ) -> Result<Option<StoredSkill>, SkillArchiveError> {
        match self.db.get(&Self::skills_ns(scope), &sanitize_key(name))? {
            Some(record) => Ok(Some(serde_json::from_str(&record.payload)?)),
            None => Ok(None),
        }
    }

    fn write_skill(&self, scope: &str, stored: &StoredSkill) -> Result<(), SkillArchiveError> {
        let mut metadata = VantaMemoryMetadata::new();
        metadata.insert("kind".into(), VantaValue::String("skill".into()));
        metadata.insert("name".into(), VantaValue::String(stored.name.clone()));
        self.db.put(VantaMemoryInput {
            namespace: Self::skills_ns(scope),
            key: sanitize_key(&stored.name),
            payload: serde_json::to_string(stored)?,
            metadata,
            vector: None,
            sparse_vector: None,
            ttl_ms: None,
        })?;
        Ok(())
    }

    /// Apply candidates for one task. Re-processing the SAME task returns
    /// `Ok(None)` without touching the store (cursor hit).
    pub fn apply_candidates(
        &self,
        scope: &str,
        task_id: &str,
        candidates: &[ExtractedSkillCandidate],
        now_ms: u64,
    ) -> Result<Option<SkillSinkCounts>, SkillArchiveError> {
        let cursor_key = format!("{task_id}__applied");
        if self
            .db
            .get(&Self::cursor_ns(scope), &sanitize_key(&cursor_key))?
            .is_some()
        {
            return Ok(None); // already applied — idempotent no-op
        }

        let mut counts = SkillSinkCounts::default();
        for candidate in candidates {
            if candidate.name.trim().is_empty() || candidate.content.trim().is_empty() {
                counts.unchanged += 1;
                continue;
            }
            let hash = content_hash(&candidate.content);
            match self.read_skill(scope, &candidate.name)? {
                Some(existing) if existing.content_hash == hash => {
                    counts.unchanged += 1; // identical content → skip
                }
                Some(_) => {
                    self.write_skill(
                        scope,
                        &StoredSkill {
                            name: candidate.name.clone(),
                            description: candidate.description.clone(),
                            content: candidate.content.clone(),
                            content_hash: hash,
                            updated_at_ms: now_ms,
                        },
                    )?;
                    counts.updated += 1;
                }
                None => {
                    self.write_skill(
                        scope,
                        &StoredSkill {
                            name: candidate.name.clone(),
                            description: candidate.description.clone(),
                            content: candidate.content.clone(),
                            content_hash: hash,
                            updated_at_ms: now_ms,
                        },
                    )?;
                    counts.created += 1;
                }
            }
        }

        // Cursor LAST: a crash before this point leaves the task re-appliable
        // (the upsert layer still prevents duplicates).
        let mut metadata = VantaMemoryMetadata::new();
        metadata.insert("kind".into(), VantaValue::String("cursor".into()));
        self.db.put(VantaMemoryInput {
            namespace: Self::cursor_ns(scope),
            key: sanitize_key(&cursor_key),
            payload: format!(
                r#"{{"task_id":"{}","created":{},"updated":{},"unchanged":{}}}"#,
                task_id, counts.created, counts.updated, counts.unchanged
            ),
            metadata,
            vector: None,
            sparse_vector: None,
            ttl_ms: None,
        })?;
        Ok(Some(counts))
    }
}

/// Deterministic 64-bit content hash (std SipHash with fixed keys — stable
/// within and across runs of the same build; enough for dedup, not crypto).
fn content_hash(content: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db() -> VantaEmbedded {
        use vantadb::config::VantaConfig;
        use vantadb::storage::BackendKind;
        VantaEmbedded::open_with_config(VantaConfig {
            backend_kind: BackendKind::InMemory,
            ..VantaConfig::default()
        })
        .expect("open in-memory db")
    }

    fn cand(name: &str, content: &str) -> ExtractedSkillCandidate {
        ExtractedSkillCandidate {
            action: "create".into(),
            name: name.into(),
            description: "d".into(),
            content: content.into(),
        }
    }

    #[test]
    fn creates_then_dedups_by_content_hash() {
        let db = db();
        let sink = SkillCoreSink::new(&db);
        let first = sink
            .apply_candidates("agent-1", "t1", &[cand("s-a", "body")], 100)
            .expect("apply");
        assert_eq!(
            first,
            Some(SkillSinkCounts {
                created: 1,
                ..Default::default()
            })
        );

        // Same content under the same name → unchanged (content-hash upsert).
        let second = sink
            .apply_candidates("agent-1", "t2", &[cand("s-a", "body")], 200)
            .expect("apply");
        assert_eq!(
            second,
            Some(SkillSinkCounts {
                unchanged: 1,
                ..Default::default()
            })
        );

        // Changed content → update.
        let third = sink
            .apply_candidates("agent-1", "t3", &[cand("s-a", "body v2")], 300)
            .expect("apply");
        assert_eq!(
            third,
            Some(SkillSinkCounts {
                updated: 1,
                ..Default::default()
            })
        );
    }

    #[test]
    fn same_task_reapplies_as_noop() {
        let db = db();
        let sink = SkillCoreSink::new(&db);
        sink.apply_candidates("agent-1", "t1", &[cand("s-a", "body")], 100)
            .expect("first apply");
        // Client retry: same task → cursor hit, store untouched.
        let retry = sink
            .apply_candidates("agent-1", "t1", &[cand("s-a", "DIFFERENT")], 200)
            .expect("retry apply");
        assert_eq!(retry, None, "cursor makes the retry a no-op");
        let stored = sink
            .read_skill("agent-1", "s-a")
            .expect("read")
            .expect("exists");
        assert_eq!(stored.content, "body", "retry must not overwrite");
    }

    #[test]
    fn scopes_are_isolated() {
        let db = db();
        let sink = SkillCoreSink::new(&db);
        sink.apply_candidates("agent-1", "t1", &[cand("shared", "v1")], 100)
            .expect("apply a");
        let counts = sink
            .apply_candidates("agent-2", "t9", &[cand("shared", "v1")], 100)
            .expect("apply b");
        assert_eq!(
            counts,
            Some(SkillSinkCounts {
                created: 1,
                ..Default::default()
            })
        );
    }
}
