//! Entity metadata store (teams, users, agents, tasks, assets).
//!
//! [`EntityStore`] persists scoped entities as JSON records in the
//! `InternalMetadata` partition — the same partition pattern used by
//! [`crate::agentic::thread`] (data as serialized records, listed by key
//! prefix). Each entity is addressed by `namespace` + `collection` +
//! caller-supplied `entity_id`, so one generic store serves every entity
//! kind (D4, plan vanta-memory) without a fixed schema.
//!
//! Scope: `namespace` (deployment/tenant), `collection` (e.g. `user`,
//! `team`, `agent`, `task`, `asset`), `entity_id` (e.g. `usr-3mfxa3b9c1`).
//! Keys are `entity:{namespace}:{collection}::{entity_id}`; listing scans
//! the collection prefix. Values must not contain `{`, `}` or `:` (ids from
//! [`generate_id`] never do).

use crate::backend::{BackendPartition, BackendWriteOp};
use crate::error::{ChainedError, Result, VantaError};
use crate::node::FieldValue;
use crate::storage::StorageEngine;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use web_time::{SystemTime, UNIX_EPOCH};

// ── Types ──

/// A single stored entity (e.g. a user, team, agent, task or asset).
///
/// `fields` carries the entity's attributes as [`FieldValue`]s; the schema
/// of each collection is interpreted by consumers (permission checker, auth).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Entity {
    pub namespace: String,
    pub collection: String,
    pub entity_id: String,
    pub fields: HashMap<String, FieldValue>,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Paginated result of [`EntityStore::entity_list`].
#[derive(Debug, Clone, PartialEq)]
pub struct EntityPage {
    pub items: Vec<Entity>,
    pub total: usize,
}

// ── EntityStore ──

/// CRUD store for scoped entities backed by a [`StorageEngine`].
///
/// Each entity is a JSON record in the `InternalMetadata` partition under
/// key `entity:{namespace}:{collection}::{entity_id}`; listing scans the
/// collection prefix and paginates. Mirrors the `agentic::thread` partition
/// pattern (D4) without inventing new storage.
pub struct EntityStore<'a> {
    engine: &'a StorageEngine,
}

impl<'a> EntityStore<'a> {
    /// Wrap a storage engine reference.
    pub fn new(engine: &'a StorageEngine) -> Self {
        Self { engine }
    }

    /// Insert or replace an entity in `namespace`/`collection`.
    ///
    /// Upsert semantics: an existing `created_at` is preserved, `fields` are
    /// replaced wholesale and `updated_at` is refreshed.
    pub fn entity_set(
        &self,
        namespace: &str,
        collection: &str,
        entity_id: &str,
        fields: HashMap<String, FieldValue>,
    ) -> Result<Entity> {
        validate_key(namespace, collection, entity_id)?;
        let now = now_secs();
        let existing = self.entity_get(namespace, collection, entity_id)?;
        let entity = Entity {
            namespace: namespace.to_string(),
            collection: collection.to_string(),
            entity_id: entity_id.to_string(),
            fields,
            created_at: existing.map_or(now, |e| e.created_at),
            updated_at: now,
        };
        let bytes = serde_json::to_vec(&entity)
            .map_err(|e| VantaError::serialization(ChainedError::with_source("entity", e)))?;
        self.engine.put_to_partition(
            BackendPartition::InternalMetadata,
            &entity_key(namespace, collection, entity_id),
            &bytes,
        )?;
        Ok(entity)
    }

    /// Retrieve an entity by scope + id, or `None` when absent.
    pub fn entity_get(
        &self,
        namespace: &str,
        collection: &str,
        entity_id: &str,
    ) -> Result<Option<Entity>> {
        validate_key(namespace, collection, entity_id)?;
        match self.engine.get_from_partition(
            BackendPartition::InternalMetadata,
            &entity_key(namespace, collection, entity_id),
        )? {
            Some(bytes) => serde_json::from_slice(&bytes)
                .map(Some)
                .map_err(|e| VantaError::serialization(ChainedError::with_source("entity", e))),
            None => Ok(None),
        }
    }

    /// Delete an entity by scope + id. Returns `true` when it existed.
    pub fn entity_delete(
        &self,
        namespace: &str,
        collection: &str,
        entity_id: &str,
    ) -> Result<bool> {
        validate_key(namespace, collection, entity_id)?;
        let key = entity_key(namespace, collection, entity_id);
        let existed = self
            .engine
            .get_from_partition(BackendPartition::InternalMetadata, &key)?
            .is_some();
        self.engine
            .write_backend_batch(vec![BackendWriteOp::Delete {
                partition: BackendPartition::InternalMetadata,
                key,
            }])?;
        Ok(existed)
    }

    /// List entities in a `namespace`/`collection` with pagination.
    ///
    /// Items are ordered by `entity_id` for deterministic pages; `total` is
    /// the full collection size before `offset`/`limit` are applied.
    pub fn entity_list(
        &self,
        namespace: &str,
        collection: &str,
        limit: usize,
        offset: usize,
    ) -> Result<EntityPage> {
        validate_scope(namespace, collection)?;
        let rows = self.engine.scan_partition_prefix(
            BackendPartition::InternalMetadata,
            collection_prefix(namespace, collection).as_bytes(),
        )?;
        let mut entities: Vec<Entity> = Vec::with_capacity(rows.len());
        for (_, bytes) in rows {
            let entity: Entity = serde_json::from_slice(&bytes)
                .map_err(|e| VantaError::serialization(ChainedError::with_source("entity", e)))?;
            entities.push(entity);
        }
        entities.sort_by(|a, b| a.entity_id.cmp(&b.entity_id));
        let total = entities.len();
        let items: Vec<Entity> = entities.into_iter().skip(offset).take(limit).collect();
        Ok(EntityPage { items, total })
    }
}

// ── ID generation (port of TDAM `utils/id-generator.ts`) ──

const ID_CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
const ID_BASE: u64 = 36;
const ID_TS_LEN: u32 = 4;
const ID_RAND_LEN: usize = 6;

/// Generate a prefixed entity id like `usr-3mfxa3b9c1`.
///
/// Four base36 timestamp digits + six base36 random digits. The prefix
/// identifies the entity kind (`usr`, `team`, `agt`, `task`, `ast`, `uky`).
pub fn generate_id(prefix: &str) -> String {
    let ts = now_secs() % ID_BASE.pow(ID_TS_LEN);
    let ts_part = encode_base36(ts, ID_TS_LEN);
    let rand_part: String = (0..ID_RAND_LEN)
        .map(|_| ID_CHARS[rand::rng().random_range(0..ID_CHARS.len())] as char)
        .collect();
    format!("{prefix}-{ts_part}{rand_part}")
}

fn encode_base36(mut value: u64, length: u32) -> String {
    let mut out = String::new();
    for _ in 0..length {
        let idx = (value % ID_BASE) as usize;
        out.insert(0, ID_CHARS[idx] as char);
        value /= ID_BASE;
    }
    out
}

// ── helpers ──

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Key for a single entity record in the `InternalMetadata` partition.
fn entity_key(namespace: &str, collection: &str, entity_id: &str) -> Vec<u8> {
    format!(
        "entity:{{{}}}:{{{}}}::{{{}}}",
        namespace, collection, entity_id
    )
    .into_bytes()
}

/// Key prefix covering every entity record of a collection.
fn collection_prefix(namespace: &str, collection: &str) -> String {
    format!("entity:{{{}}}:{{{}}}::", namespace, collection)
}

fn validate_scope(namespace: &str, collection: &str) -> Result<()> {
    if namespace.is_empty() || collection.is_empty() {
        return Err(VantaError::InvalidInput(
            "namespace and collection must be non-empty".into(),
        ));
    }
    if namespace.contains(['{', '}', ':']) || collection.contains(['{', '}', ':']) {
        return Err(VantaError::InvalidInput(
            "namespace and collection must not contain '{', '}' or ':'".into(),
        ));
    }
    Ok(())
}

fn validate_key(namespace: &str, collection: &str, entity_id: &str) -> Result<()> {
    validate_scope(namespace, collection)?;
    if entity_id.is_empty() {
        return Err(VantaError::InvalidInput(
            "entity_id must be non-empty".into(),
        ));
    }
    if entity_id.contains(['{', '}', ':']) {
        return Err(VantaError::InvalidInput(
            "entity_id must not contain '{', '}' or ':'".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
