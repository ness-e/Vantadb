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
//! **MEM-64 history layer**: every successful create/update also writes an
//! append-only [`SkillVersion`] snapshot to
//! `skills_extract/{scope}/_versions/{name}/{version_seq}`, with
//! `prev_version_seq` pointing at the prior snapshot. The latest pointer
//! stays at `skills_extract/{scope}/{name}` (status quo). Read API:
//! [`SkillCoreSink::list_skill_versions`].
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
use vantadb::sdk::{
    VantaEmbedded, VantaMemoryInput, VantaMemoryListOptions, VantaMemoryMetadata, VantaValue,
};

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

/// Append-only history snapshot for a skill (MEM-64).
///
/// Each successful create/update of a `StoredSkill` writes one of these to
/// `skills_extract/{scope}/_versions/{name}/{version_seq}` with the prior
/// version's `version_seq` recorded in `prev_version_seq`. The history is
/// additive — it never modifies or deletes prior snapshots, so a client can
/// reconstruct every state a skill has ever had.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillVersion {
    pub scope: String,
    pub name: String,
    /// Monotonically increasing within `(scope, name)`. Starts at 1 on the
    /// very first version; subsequent snapshots increment from the latest.
    pub version_seq: u64,
    /// `version_seq` of the snapshot that preceded this one. `None` for the
    /// first version of a name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_version_seq: Option<u64>,
    pub content: String,
    pub content_hash: u64,
    pub updated_at_ms: u64,
    /// `true` for the snapshot that established the name (no prior versions).
    pub created: bool,
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

    /// Append-only history namespace for a `(scope, name)` (MEM-64).
    fn versions_ns(scope: &str, name: &str) -> String {
        format!(
            "skills_extract/{}/_versions/{}",
            sanitize_component(scope, 64, false),
            sanitize_key(name),
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
                    // MEM-64: chain the new version off the latest snapshot.
                    let prev = self.last_version_seq(scope, &candidate.name)?;
                    let next_seq = prev.unwrap_or(0) + 1;
                    let version = SkillVersion {
                        scope: scope.to_string(),
                        name: candidate.name.clone(),
                        version_seq: next_seq,
                        prev_version_seq: prev,
                        content: candidate.content.clone(),
                        content_hash: hash,
                        updated_at_ms: now_ms,
                        created: false,
                    };
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
                    self.record_version(scope, &version)?;
                    counts.updated += 1;
                }
                None => {
                    // MEM-64: first version for this name.
                    let version = SkillVersion {
                        scope: scope.to_string(),
                        name: candidate.name.clone(),
                        version_seq: 1,
                        prev_version_seq: None,
                        content: candidate.content.clone(),
                        content_hash: hash,
                        updated_at_ms: now_ms,
                        created: true,
                    };
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
                    self.record_version(scope, &version)?;
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

    /// Find the latest `version_seq` written for `(scope, name)`.
    /// ponytail: O(n) scan over the `_versions/{name}` namespace, replace
    /// with a versioned-secondary index if the count per name ever explodes
    /// (typical: <100).
    fn last_version_seq(&self, scope: &str, name: &str) -> Result<Option<u64>, SkillArchiveError> {
        let ns = Self::versions_ns(scope, name);
        let opts = VantaMemoryListOptions {
            limit: 10_000,
            ..VantaMemoryListOptions::default()
        };
        let page = self.db.list(&ns, opts)?;
        let mut latest: Option<u64> = None;
        for entry in page.records {
            let v: SkillVersion = serde_json::from_str(&entry.payload)?;
            if latest.is_none_or(|prev| v.version_seq > prev) {
                latest = Some(v.version_seq);
            }
        }
        Ok(latest)
    }

    /// Persist a [`SkillVersion`] snapshot. Cursor is written LAST, so a
    /// failure here surfaces to the caller and a retry will re-emit the
    /// snapshot.
    fn record_version(&self, scope: &str, version: &SkillVersion) -> Result<(), SkillArchiveError> {
        let mut metadata = VantaMemoryMetadata::new();
        metadata.insert("kind".into(), VantaValue::String("skill_version".into()));
        metadata.insert("name".into(), VantaValue::String(version.name.clone()));
        metadata.insert(
            "version_seq".into(),
            VantaValue::Int(version.version_seq as i64),
        );
        self.db.put(VantaMemoryInput {
            namespace: Self::versions_ns(scope, &version.name),
            key: sanitize_key(&format!("{:020}", version.version_seq)),
            payload: serde_json::to_string(version)?,
            metadata,
            vector: None,
            sparse_vector: None,
            ttl_ms: None,
        })?;
        Ok(())
    }

    /// Read every recorded version of a skill, oldest first.
    pub fn list_skill_versions(
        &self,
        scope: &str,
        name: &str,
    ) -> Result<Vec<SkillVersion>, SkillArchiveError> {
        let ns = Self::versions_ns(scope, name);
        let opts = VantaMemoryListOptions {
            limit: 10_000,
            ..VantaMemoryListOptions::default()
        };
        let page = self.db.list(&ns, opts)?;
        let mut versions: Vec<SkillVersion> = page
            .records
            .into_iter()
            .map(|e| serde_json::from_str(&e.payload))
            .collect::<Result<_, _>>()?;
        versions.sort_by_key(|v| v.version_seq);
        Ok(versions)
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

    /// MEM-64: every successful create/update must emit a version snapshot,
    /// and `list_skill_versions` must return them ordered.
    #[test]
    fn records_history_on_create_and_update() {
        let db = db();
        let sink = SkillCoreSink::new(&db);

        sink.apply_candidates("agent-1", "t1", &[cand("alpha", "v1")], 100)
            .expect("first create");
        sink.apply_candidates("agent-1", "t2", &[cand("alpha", "v2")], 200)
            .expect("update");
        sink.apply_candidates("agent-1", "t3", &[cand("alpha", "v3")], 300)
            .expect("update again");

        let versions = sink
            .list_skill_versions("agent-1", "alpha")
            .expect("list versions");
        assert_eq!(versions.len(), 3, "one snapshot per create/update");
        assert_eq!(versions[0].version_seq, 1);
        assert!(versions[0].created);
        assert_eq!(versions[0].prev_version_seq, None);
        assert_eq!(versions[1].version_seq, 2);
        assert_eq!(versions[1].prev_version_seq, Some(1));
        assert!(!versions[1].created);
        assert_eq!(versions[2].version_seq, 3);
        assert_eq!(versions[2].prev_version_seq, Some(2));

        // Identical content → no new version (content-hash unchanged branch).
        sink.apply_candidates("agent-1", "t4", &[cand("alpha", "v3")], 400)
            .expect("unchanged");
        let versions = sink
            .list_skill_versions("agent-1", "alpha")
            .expect("list versions after unchanged");
        assert_eq!(versions.len(), 3, "unchanged does NOT emit a snapshot");

        // Latest pointer still resolves to v3.
        let latest = sink.read_skill("agent-1", "alpha").expect("read");
        assert_eq!(latest.unwrap().content, "v3");
    }
}
