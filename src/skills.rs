//! Versioned skill store (agent skills / memory skills).
//!
//! Port of TDAM `skill-store.ts` + `skill-versioning.ts` onto the entity
//! pattern (D4, plan vanta-memory) — no new storage, no FTS/vec index.
//!
//! Layout (all in the `InternalMetadata` partition, `skills` namespace):
//!
//! * Collection `skill` — one row per immutable version:
//!   `entity:{skills}:{skill}::{skill_id}~v{version}`
//! * Collection `skill_head` — the unique partial index
//!   `(owner_agent, name) WHERE is_head`:
//!   `entity:{skills}:{skill_head}::{owner_agent}#{name}`
//!
//! Concurrency (Regla 8): every write is an atomic [`StorageEngine::write_backend_batch`]
//! (version row + index row together). The optimistic lock `expected_version`
//! serializes concurrent writers: each writer re-reads the head inside the
//! same task and the batch commit is atomic, so a stale `expected_version`
//! fails with [`VantaError::ExecutionConflict`] before any write lands.
//!
//! `content_hash` is FNV-1a 64-bit (hex) over the content only, mirroring TDAM
//! `computeContentHash` — deliberately non-cryptographic: it exists for
//! idempotency detection, not integrity.

use crate::backend::{BackendPartition, BackendWriteOp};
use crate::entity::{generate_id, Entity, EntityStore};
use crate::error::{ChainedError, Result, VantaError};
use crate::node::FieldValue;
use crate::sdk::types::{
    SkillCreateInput, SkillListOptions, SkillListPage, SkillPatchInput, SkillRecord,
    SkillUpdateInput, SkillWriteResult,
};
use crate::storage::StorageEngine;
use std::collections::HashMap;
use web_time::{SystemTime, UNIX_EPOCH};

/// Namespace holding all skill entities (contract: "namespace `skills`").
pub const SKILL_NS: &str = "skills";
/// Collection holding one entity per immutable skill version.
pub const SKILL_COLLECTION: &str = "skill";
/// Collection holding the unique partial index `(owner, name) WHERE is_head`.
pub const SKILL_HEAD_COLLECTION: &str = "skill_head";
/// TTL cleanup keeps this many most-recent non-head versions per skill.
pub const KEEP_RECENT: usize = 3;

/// Versioned skill store backed by a [`StorageEngine`].
pub struct SkillStore<'a> {
    engine: &'a StorageEngine,
    entities: EntityStore<'a>,
}

impl<'a> SkillStore<'a> {
    /// Wrap a storage engine reference.
    pub fn new(engine: &'a StorageEngine) -> Self {
        Self {
            engine,
            entities: EntityStore::new(engine),
        }
    }

    // ── Reads ──

    /// Resolve the current head of `skill_id`, or `None` when the skill
    /// does not exist.
    pub fn get_head(&self, skill_id: &str) -> Result<Option<SkillRecord>> {
        validate_skill_id(skill_id)?;
        for row in self.scan_versions(skill_id)? {
            if bool_field(&row, "is_head")? {
                return Ok(Some(entity_to_record(row)?));
            }
        }
        Ok(None)
    }

    /// Resolve a specific immutable version, or `None` when absent.
    pub fn get_version(&self, skill_id: &str, version: u64) -> Result<Option<SkillRecord>> {
        validate_skill_id(skill_id)?;
        self.entities
            .entity_get(
                SKILL_NS,
                SKILL_COLLECTION,
                &version_entity_id(skill_id, version),
            )?
            .map(entity_to_record)
            .transpose()
    }

    /// List the versions of a skill, newest first, with pagination.
    pub fn list_versions(
        &self,
        skill_id: &str,
        limit: usize,
        offset: usize,
    ) -> Result<SkillListPage> {
        validate_skill_id(skill_id)?;
        let mut rows = self.scan_versions(skill_id)?;
        rows.sort_by_key(|e| e.fields.get("version").and_then(FieldValue::as_int));
        rows.reverse();
        let total = rows.len();
        let items: Vec<SkillRecord> = rows
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(entity_to_record)
            .collect::<Result<Vec<_>>>()?;
        Ok(SkillListPage { items, total })
    }

    /// List skill heads (current versions) with optional owner / name-prefix
    /// filters, ordered by name.
    pub fn list(&self, opts: SkillListOptions) -> Result<SkillListPage> {
        let page = self
            .entities
            .entity_list(SKILL_NS, SKILL_HEAD_COLLECTION, usize::MAX, 0)?;
        let mut records = Vec::with_capacity(page.items.len());
        for index in page.items {
            let skill_id = str_field(&index, "skill_id")?;
            let version = int_field(&index, "version")? as u64;
            let Some(head) = self.get_version(&skill_id, version)? else {
                continue; // stale index row (should not happen under batch writes)
            };
            if let Some(owner) = &opts.owner_agent {
                if head.owner_agent != *owner {
                    continue;
                }
            }
            if let Some(prefix) = &opts.name_prefix {
                if !head.name.starts_with(prefix.as_str()) {
                    continue;
                }
            }
            records.push(head);
        }
        records.sort_by(|a, b| a.name.cmp(&b.name));
        let total = records.len();
        let items = records
            .into_iter()
            .skip(opts.offset)
            .take(opts.limit)
            .collect();
        Ok(SkillListPage { items, total })
    }

    // ── Writes (atomic batches + optimistic lock) ──

    /// Create a new skill as version 1.
    ///
    /// Idempotent: when a head with the same `(owner_agent, name)` and the
    /// same `content_hash` already exists, the existing head is returned with
    /// `idempotent = true`. A different content under the same `(owner, name)`
    /// is rejected with `ExecutionConflict` (unique partial index).
    pub fn create(&self, input: SkillCreateInput) -> Result<SkillWriteResult> {
        validate_owner(&input.owner_agent)?;
        validate_name(&input.name)?;
        let content_hash = fnv1a_64(&input.content);
        if let Some(head) = self.head_by_owner_name(&input.owner_agent, &input.name)? {
            if head.content_hash == content_hash {
                return Ok(SkillWriteResult {
                    record: head,
                    idempotent: true,
                });
            }
            return Err(VantaError::ExecutionConflict {
                resource: format!("skill:{}/{}", input.owner_agent, input.name),
                detail: format!(
                    "name '{}' already exists for owner '{}'",
                    input.name, input.owner_agent
                ),
            });
        }
        let now = now_secs();
        let skill_id = generate_id("skl");
        let record = SkillRecord {
            skill_id,
            version: 1,
            is_head: true,
            owner_agent: input.owner_agent.clone(),
            name: input.name.clone(),
            description: input.description,
            content: input.content,
            content_hash,
            metadata: input.metadata,
            created_at: now,
            updated_at: now,
            expires_at: input.ttl_secs.map(|ttl| now + ttl),
        };
        self.write_batch(vec![
            BackendWriteOp::Put {
                partition: BackendPartition::InternalMetadata,
                key: version_key(&record.skill_id, record.version),
                value: serialize_entity(&record_to_entity(&record)?)?,
            },
            BackendWriteOp::Put {
                partition: BackendPartition::InternalMetadata,
                key: head_index_key(&input.owner_agent, &input.name),
                value: serialize_entity(&head_index_entity(&record))?,
            },
        ])?;
        Ok(SkillWriteResult {
            record,
            idempotent: false,
        })
    }

    /// Replace a skill's head with a new version (append-only).
    ///
    /// `expected_version` must match the current head version (optimistic
    /// lock). Returns the unchanged head with `idempotent = true` when
    /// content, description and metadata are all unchanged.
    pub fn update(
        &self,
        skill_id: &str,
        expected_version: u64,
        input: SkillUpdateInput,
    ) -> Result<SkillWriteResult> {
        let head = self.require_head(skill_id)?;
        self.check_version(skill_id, &head, expected_version)?;
        let new_content_hash = fnv1a_64(&input.content);
        if head.content_hash == new_content_hash
            && head.description == input.description
            && input.metadata.as_ref().is_none_or(|m| &head.metadata == m)
        {
            return Ok(SkillWriteResult {
                record: head,
                idempotent: true,
            });
        }
        let now = now_secs();
        let metadata = input.metadata.unwrap_or_else(|| head.metadata.clone());
        let new_record = SkillRecord {
            version: head.version + 1,
            is_head: true,
            description: input.description,
            content: input.content,
            content_hash: new_content_hash,
            metadata,
            created_at: now,
            updated_at: now,
            expires_at: head
                .expires_at
                .map(|e| now + e.saturating_sub(head.created_at)),
            ..head.clone()
        };
        self.commit_new_head(&head, &new_record)
    }

    /// Patch a skill's head: only the provided fields change.
    pub fn patch(
        &self,
        skill_id: &str,
        expected_version: u64,
        input: SkillPatchInput,
    ) -> Result<SkillWriteResult> {
        let head = self.require_head(skill_id)?;
        self.check_version(skill_id, &head, expected_version)?;
        let new_content = input.content.unwrap_or_else(|| head.content.clone());
        let new_description = input
            .description
            .unwrap_or_else(|| head.description.clone());
        let new_metadata = input.metadata.unwrap_or_else(|| head.metadata.clone());
        let new_content_hash = fnv1a_64(&new_content);
        if head.content_hash == new_content_hash
            && head.description == new_description
            && head.metadata == new_metadata
        {
            return Ok(SkillWriteResult {
                record: head,
                idempotent: true,
            });
        }
        let now = now_secs();
        let new_record = SkillRecord {
            version: head.version + 1,
            is_head: true,
            description: new_description,
            content: new_content,
            content_hash: new_content_hash,
            metadata: new_metadata,
            created_at: now,
            updated_at: now,
            expires_at: head
                .expires_at
                .map(|e| now + e.saturating_sub(head.created_at)),
            ..head.clone()
        };
        self.commit_new_head(&head, &new_record)
    }

    /// Delete every version of a skill plus its head index row.
    ///
    /// `expected_version` must match the current head (optimistic lock).
    /// Returns `true` when the skill existed.
    pub fn delete(&self, skill_id: &str, expected_version: u64) -> Result<bool> {
        let Some(head) = self.get_head(skill_id)? else {
            return Ok(false);
        };
        self.check_version(skill_id, &head, expected_version)?;
        let mut ops = Vec::new();
        for row in self.scan_versions(skill_id)? {
            ops.push(BackendWriteOp::Delete {
                partition: BackendPartition::InternalMetadata,
                key: entity_key(
                    SKILL_NS,
                    SKILL_COLLECTION,
                    &version_entity_id(skill_id, version_from_entity_id(&row.entity_id)?),
                ),
            });
        }
        ops.push(BackendWriteOp::Delete {
            partition: BackendPartition::InternalMetadata,
            key: head_index_key(&head.owner_agent, &head.name),
        });
        self.write_batch(ops)?;
        Ok(true)
    }

    /// Delete expired non-head versions of a skill, keeping the
    /// [`KEEP_RECENT`] most-recent non-head versions (port of TDAM
    /// `cleanupExpiredVersionsForSkill`). Returns the number of versions
    /// deleted. The head is never deleted by TTL.
    pub fn cleanup_expired_versions(&self, skill_id: &str, now: u64) -> Result<usize> {
        validate_skill_id(skill_id)?;
        let mut rows = self.scan_versions(skill_id)?;
        rows.sort_by_key(|e| e.fields.get("version").and_then(FieldValue::as_int));
        rows.reverse();
        let mut kept_non_head = 0usize;
        let mut ops = Vec::new();
        let mut deleted = 0usize;
        for row in rows {
            let is_head = bool_field(&row, "is_head")?;
            if is_head {
                continue;
            }
            let expires_at = match row.fields.get("expires_at").and_then(FieldValue::as_int) {
                Some(exp) => exp as u64,
                None => continue, // never expires
            };
            if expires_at >= now {
                continue;
            }
            if kept_non_head < KEEP_RECENT {
                kept_non_head += 1;
                continue;
            }
            ops.push(BackendWriteOp::Delete {
                partition: BackendPartition::InternalMetadata,
                key: entity_key(SKILL_NS, SKILL_COLLECTION, &row.entity_id),
            });
            deleted += 1;
        }
        if !ops.is_empty() {
            self.write_batch(ops)?;
        }
        Ok(deleted)
    }

    // ── Internal helpers ──

    fn require_head(&self, skill_id: &str) -> Result<SkillRecord> {
        self.get_head(skill_id)?
            .ok_or_else(|| VantaError::NotFound {
                kind: "skill".into(),
                id: skill_id.into(),
            })
    }

    fn check_version(&self, skill_id: &str, head: &SkillRecord, expected: u64) -> Result<()> {
        if head.version != expected {
            return Err(VantaError::ExecutionConflict {
                resource: format!("skill:{skill_id}"),
                detail: format!("expected version {expected}, head is {}", head.version),
            });
        }
        Ok(())
    }

    /// Resolve the head index row for `(owner_agent, name)`, then its record.
    fn head_by_owner_name(&self, owner: &str, name: &str) -> Result<Option<SkillRecord>> {
        let Some(index) = self.entities.entity_get(
            SKILL_NS,
            SKILL_HEAD_COLLECTION,
            &head_entity_id(owner, name),
        )?
        else {
            return Ok(None);
        };
        let skill_id = str_field(&index, "skill_id")?;
        let version = int_field(&index, "version")? as u64;
        self.get_version(&skill_id, version)
    }

    /// Persist `old_head` (demoted to non-head) + `new_head` + updated index
    /// in one atomic batch.
    fn commit_new_head(
        &self,
        old_head: &SkillRecord,
        new_head: &SkillRecord,
    ) -> Result<SkillWriteResult> {
        let mut old = record_to_entity(old_head)?;
        old.fields.insert("is_head".into(), FieldValue::Bool(false));
        self.write_batch(vec![
            BackendWriteOp::Put {
                partition: BackendPartition::InternalMetadata,
                key: entity_key(SKILL_NS, SKILL_COLLECTION, &old.entity_id),
                value: serialize_entity(&old)?,
            },
            BackendWriteOp::Put {
                partition: BackendPartition::InternalMetadata,
                key: version_key(&new_head.skill_id, new_head.version),
                value: serialize_entity(&record_to_entity(new_head)?)?,
            },
            BackendWriteOp::Put {
                partition: BackendPartition::InternalMetadata,
                key: head_index_key(&new_head.owner_agent, &new_head.name),
                value: serialize_entity(&head_index_entity(new_head))?,
            },
        ])?;
        Ok(SkillWriteResult {
            record: new_head.clone(),
            idempotent: false,
        })
    }

    fn write_batch(&self, ops: Vec<BackendWriteOp>) -> Result<()> {
        self.engine.write_backend_batch(ops)
    }

    /// Scan all version rows of a skill (any version, any order).
    fn scan_versions(&self, skill_id: &str) -> Result<Vec<Entity>> {
        // Keys are `entity:{skills}:{skill}::{skill_id}~v{N}` (entity_id wrapped
        // in braces). The prefix opens the entity_id brace but leaves `~v`
        // inside so it matches every version of this skill only.
        let prefix = format!("entity:{{skills}}:{{skill}}::{{{skill_id}~v");
        let rows = self
            .engine
            .scan_partition_prefix(BackendPartition::InternalMetadata, prefix.as_bytes())?;
        rows.into_iter()
            .map(|(_, bytes)| {
                serde_json::from_slice(&bytes).map_err(|e| {
                    VantaError::serialization(ChainedError::with_source("skill version", e))
                })
            })
            .collect()
    }
}

// ── Record ↔ Entity mapping ──

fn record_to_entity(record: &SkillRecord) -> Result<Entity> {
    let mut fields = HashMap::new();
    fields.insert("version".into(), FieldValue::Int(record.version as i64));
    fields.insert("is_head".into(), FieldValue::Bool(record.is_head));
    fields.insert(
        "owner_agent".into(),
        FieldValue::String(record.owner_agent.clone()),
    );
    fields.insert("name".into(), FieldValue::String(record.name.clone()));
    fields.insert(
        "description".into(),
        FieldValue::String(record.description.clone()),
    );
    fields.insert("content".into(), FieldValue::String(record.content.clone()));
    fields.insert(
        "content_hash".into(),
        FieldValue::String(record.content_hash.clone()),
    );
    let metadata_json = serde_json::to_string(&record.metadata)
        .map_err(|e| VantaError::serialization(ChainedError::with_source("skill metadata", e)))?;
    fields.insert("metadata".into(), FieldValue::String(metadata_json));
    fields.insert(
        "created_at".into(),
        FieldValue::Int(record.created_at as i64),
    );
    fields.insert(
        "updated_at".into(),
        FieldValue::Int(record.updated_at as i64),
    );
    fields.insert(
        "expires_at".into(),
        record
            .expires_at
            .map(|e| FieldValue::Int(e as i64))
            .unwrap_or(FieldValue::Null),
    );
    Ok(Entity {
        namespace: SKILL_NS.into(),
        collection: SKILL_COLLECTION.into(),
        entity_id: version_entity_id(&record.skill_id, record.version),
        fields,
        created_at: record.created_at,
        updated_at: record.updated_at,
    })
}

fn entity_to_record(entity: Entity) -> Result<SkillRecord> {
    Ok(SkillRecord {
        skill_id: entity
            .entity_id
            .split("~v")
            .next()
            .unwrap_or_default()
            .to_string(),
        version: int_field(&entity, "version")? as u64,
        is_head: bool_field(&entity, "is_head")?,
        owner_agent: str_field(&entity, "owner_agent")?,
        name: str_field(&entity, "name")?,
        description: str_field(&entity, "description")?,
        content: str_field(&entity, "content")?,
        content_hash: str_field(&entity, "content_hash")?,
        metadata: {
            let raw = str_field(&entity, "metadata")?;
            serde_json::from_str(&raw).map_err(|e| {
                VantaError::serialization(ChainedError::with_source("skill metadata", e))
            })?
        },
        created_at: int_field(&entity, "created_at")? as u64,
        updated_at: int_field(&entity, "updated_at")? as u64,
        expires_at: entity
            .fields
            .get("expires_at")
            .and_then(FieldValue::as_int)
            .map(|exp| exp as u64),
    })
}

/// Head index entity: one row per `(owner, name)` pointing at the current head.
fn head_index_entity(record: &SkillRecord) -> Entity {
    let mut fields = HashMap::new();
    fields.insert(
        "skill_id".into(),
        FieldValue::String(record.skill_id.clone()),
    );
    fields.insert("version".into(), FieldValue::Int(record.version as i64));
    fields.insert(
        "owner_agent".into(),
        FieldValue::String(record.owner_agent.clone()),
    );
    fields.insert("name".into(), FieldValue::String(record.name.clone()));
    fields.insert(
        "updated_at".into(),
        FieldValue::Int(record.updated_at as i64),
    );
    Entity {
        namespace: SKILL_NS.into(),
        collection: SKILL_HEAD_COLLECTION.into(),
        entity_id: head_entity_id(&record.owner_agent, &record.name),
        fields,
        created_at: record.created_at,
        updated_at: record.updated_at,
    }
}

// ── Field accessors ──

fn str_field(entity: &Entity, name: &str) -> Result<String> {
    entity
        .fields
        .get(name)
        .and_then(FieldValue::as_str)
        .map(str::to_string)
        .ok_or_else(|| VantaError::ValidationError {
            field: name.into(),
            reason: "missing or wrong type".into(),
        })
}

fn int_field(entity: &Entity, name: &str) -> Result<i64> {
    entity
        .fields
        .get(name)
        .and_then(FieldValue::as_int)
        .ok_or_else(|| VantaError::ValidationError {
            field: name.into(),
            reason: "missing or wrong type".into(),
        })
}

fn bool_field(entity: &Entity, name: &str) -> Result<bool> {
    entity
        .fields
        .get(name)
        .and_then(FieldValue::as_bool)
        .ok_or_else(|| VantaError::ValidationError {
            field: name.into(),
            reason: "missing or wrong type".into(),
        })
}

// ── Keys, hashing, validation ──

/// FNV-1a 64-bit over the content bytes, hex-encoded.
///
/// Stable across runs and platforms (unlike `std::hash::DefaultHasher`),
/// dependency-free, WASM-safe. Non-cryptographic by design — used only for
/// idempotency detection, mirroring TDAM's MD5 role.
fn fnv1a_64(input: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

fn version_entity_id(skill_id: &str, version: u64) -> String {
    format!("{skill_id}~v{version}")
}

fn version_from_entity_id(entity_id: &str) -> Result<u64> {
    let version = entity_id
        .split("~v")
        .nth(1)
        .ok_or_else(|| VantaError::ValidationError {
            field: "entity_id".into(),
            reason: "malformed version entity id".into(),
        })?;
    version.parse().map_err(|_| VantaError::ValidationError {
        field: "entity_id".into(),
        reason: "malformed version entity id".into(),
    })
}

fn head_entity_id(owner: &str, name: &str) -> String {
    format!("{owner}#{name}")
}

fn version_key(skill_id: &str, version: u64) -> Vec<u8> {
    entity_key(
        SKILL_NS,
        SKILL_COLLECTION,
        &version_entity_id(skill_id, version),
    )
}

fn head_index_key(owner: &str, name: &str) -> Vec<u8> {
    entity_key(
        SKILL_NS,
        SKILL_HEAD_COLLECTION,
        &head_entity_id(owner, name),
    )
}

fn entity_key(namespace: &str, collection: &str, entity_id: &str) -> Vec<u8> {
    format!(
        "entity:{{{}}}:{{{}}}::{{{}}}",
        namespace, collection, entity_id
    )
    .into_bytes()
}

fn serialize_entity(entity: &Entity) -> Result<Vec<u8>> {
    serde_json::to_vec(entity)
        .map_err(|e| VantaError::serialization(ChainedError::with_source("skill entity", e)))
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn validate_skill_id(skill_id: &str) -> Result<()> {
    if skill_id.is_empty() || skill_id.contains(['#', '~', '{', '}', ':']) {
        return Err(VantaError::InvalidInput(
            "skill_id must be non-empty and must not contain '#', '~', '{', '}' or ':'".into(),
        ));
    }
    Ok(())
}

fn validate_owner(owner: &str) -> Result<()> {
    if owner.is_empty() || owner.contains(['#', '{', '}', ':']) {
        return Err(VantaError::ValidationError {
            field: "owner_agent".into(),
            reason: "must be non-empty and must not contain '#', '{', '}' or ':'".into(),
        });
    }
    Ok(())
}

fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() || name.contains(['#', '{', '}', ':']) {
        return Err(VantaError::ValidationError {
            field: "name".into(),
            reason: "must be non-empty and must not contain '#', '{', '}' or ':'".into(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
