//! Entity metadata store (teams, users, agents, tasks, assets).
//!
//! [`EntityStore`](crate::entity::EntityStore) persists scoped entities as JSON records in the
//! `InternalMetadata` partition — the same partition pattern used by
//! [`crate::agentic::thread`] (data as serialized records, listed by key
//! prefix). Each entity is addressed by `namespace` + `collection` +
//! caller-supplied `id`, so one generic store serves every entity
//! kind (D4, plan vanta-memory) without a fixed schema.
//!
//! Scope: `namespace` (deployment/tenant), `collection` (e.g. `user`,
//! `team`, `agent`, `task`, `asset`), `id` (e.g. `usr-3mfxa3b9c1`).
//! Keys are `entity:{namespace}:{collection}::{id}`; listing scans
//! the collection prefix. Values must not contain `{`, `}` or `:` (ids from
//! [`generate_id`](crate::entity::generate_id) never do).
//!
//! Scene node anchors (MEM-12) live in the same `InternalMetadata` partition
//! under the `scene:` key family — see [`scene::SceneNodeStore`](crate::entity::SceneNodeStore).

use crate::backend::{BackendPartition, BackendWriteOp};
use crate::error::{ChainedError, Error, Result};
use crate::node::FieldValue;
use crate::storage::StorageEngine;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use web_time::{SystemTime, UNIX_EPOCH};

/// Allow-only permission checker over `entity_*` records (MEM-04).
pub mod checker;

/// Scene node anchors in the core graph (MEM-12, F4).
pub mod scene;

pub use scene::{SceneNode, SceneNodePage, SceneNodeStore, SceneNodeWrite};

// ── Types ──

/// A single stored entity (e.g. a user, team, agent, task or asset).
///
/// `fields` carries the entity's attributes as [`FieldValue`]s; the schema
/// of each collection is interpreted by consumers (permission checker, auth).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Entity {
    pub namespace: String,
    pub collection: String,
    #[serde(alias = "entity_id")]
    pub id: String,
    pub fields: HashMap<String, FieldValue>,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Paginated result of [`EntityStore::list`].
#[derive(Debug, Clone, PartialEq)]
pub struct EntityPage {
    pub items: Vec<Entity>,
    pub total: usize,
}

/// Immutable command object for [`EntityStore::set`] (D3: F2 command-object).
///
/// Groups the 4 write params so the `set` signature stays thin; `gc`-like
/// services are not part of the command (there are none here — pure data).
/// Borrows the key components (`validate_key` semantics unchanged) and moves
/// `fields` (already owned at every caller).
#[derive(Debug, Clone)]
pub struct EntityWrite<'a> {
    pub namespace: &'a str,
    pub collection: &'a str,
    pub id: &'a str,
    pub fields: HashMap<String, FieldValue>,
}

// ── EntityRepository (puerto DDD) ──

/// Puerto de repositorio de entidades (DDD) — DIP (C2M2).
///
/// Los consumidores (checker, auth, skills) dependen de este trait, no del
/// struct concreto [`EntityStore`]. `EntityStore` es el adaptador
/// `StorageEngine → EntityRepository` (el engine no conoce tipos de dominio).
pub trait EntityRepository {
    /// Insert or replace an entity (upsert semantics).
    fn set(&self, input: EntityWrite<'_>) -> Result<Entity>;

    /// Retrieve an entity by scope + id, or `None` when absent.
    fn get(&self, namespace: &str, collection: &str, id: &str) -> Result<Option<Entity>>;

    /// Delete an entity by scope + id. Returns `true` when it existed.
    fn delete(&self, namespace: &str, collection: &str, id: &str) -> Result<bool>;

    /// List entities with pagination (ordered by `id`; `total` pre-page).
    fn list(
        &self,
        namespace: &str,
        collection: &str,
        limit: usize,
        offset: usize,
    ) -> Result<EntityPage>;
}

// ── EntityStore ──

/// CRUD store for scoped entities backed by a [`StorageEngine`].
///
/// Each entity is a JSON record in the `InternalMetadata` partition under
/// key `entity:{namespace}:{collection}::{id}`; listing scans the
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
    pub fn set(&self, input: EntityWrite<'_>) -> Result<Entity> {
        let EntityWrite {
            namespace,
            collection,
            id,
            fields,
        } = input;
        validate_key(namespace, collection, id)?;
        let now = now_secs();
        let existing = self.get(namespace, collection, id)?;
        let entity = Entity {
            namespace: namespace.to_string(),
            collection: collection.to_string(),
            id: id.to_string(),
            fields,
            created_at: existing.map_or(now, |e| e.created_at),
            updated_at: now,
        };
        let bytes = serde_json::to_vec(&entity)
            .map_err(|e| Error::serialization(ChainedError::with_source("entity", e)))?;
        self.engine.put_to_partition(
            BackendPartition::InternalMetadata,
            &entity_key(namespace, collection, id),
            &bytes,
        )?;
        Ok(entity)
    }

    /// Retrieve an entity by scope + id, or `None` when absent.
    pub fn get(&self, namespace: &str, collection: &str, id: &str) -> Result<Option<Entity>> {
        validate_key(namespace, collection, id)?;
        match self.engine.get_from_partition(
            BackendPartition::InternalMetadata,
            &entity_key(namespace, collection, id),
        )? {
            Some(bytes) => serde_json::from_slice(&bytes)
                .map(Some)
                .map_err(|e| Error::serialization(ChainedError::with_source("entity", e))),
            None => Ok(None),
        }
    }

    /// Delete an entity by scope + id. Returns `true` when it existed.
    pub fn delete(&self, namespace: &str, collection: &str, id: &str) -> Result<bool> {
        validate_key(namespace, collection, id)?;
        let key = entity_key(namespace, collection, id);
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
    /// Items are ordered by `id` for deterministic pages; `total` is
    /// the full collection size before `offset`/`limit` are applied.
    pub fn list(
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
                .map_err(|e| Error::serialization(ChainedError::with_source("entity", e)))?;
            entities.push(entity);
        }
        entities.sort_by(|a, b| a.id.cmp(&b.id));
        let total = entities.len();
        let items: Vec<Entity> = entities.into_iter().skip(offset).take(limit).collect();
        Ok(EntityPage { items, total })
    }
}

impl EntityRepository for EntityStore<'_> {
    fn set(&self, input: EntityWrite<'_>) -> Result<Entity> {
        EntityStore::set(self, input)
    }

    fn get(&self, namespace: &str, collection: &str, id: &str) -> Result<Option<Entity>> {
        EntityStore::get(self, namespace, collection, id)
    }

    fn delete(&self, namespace: &str, collection: &str, id: &str) -> Result<bool> {
        EntityStore::delete(self, namespace, collection, id)
    }

    fn list(
        &self,
        namespace: &str,
        collection: &str,
        limit: usize,
        offset: usize,
    ) -> Result<EntityPage> {
        EntityStore::list(self, namespace, collection, limit, offset)
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
fn entity_key(namespace: &str, collection: &str, id: &str) -> Vec<u8> {
    format!("entity:{{{}}}:{{{}}}::{{{}}}", namespace, collection, id).into_bytes()
}

/// Key prefix covering every entity record of a collection.
fn collection_prefix(namespace: &str, collection: &str) -> String {
    format!("entity:{{{}}}:{{{}}}::", namespace, collection)
}

fn validate_scope(namespace: &str, collection: &str) -> Result<()> {
    if namespace.is_empty() || collection.is_empty() {
        return Err(Error::InvalidInput(
            "namespace and collection must be non-empty".into(),
        ));
    }
    if namespace.contains(['{', '}', ':']) || collection.contains(['{', '}', ':']) {
        return Err(Error::InvalidInput(
            "namespace and collection must not contain '{', '}' or ':'".into(),
        ));
    }
    Ok(())
}

fn validate_key(namespace: &str, collection: &str, id: &str) -> Result<()> {
    validate_scope(namespace, collection)?;
    if id.is_empty() {
        return Err(Error::InvalidInput("id must be non-empty".into()));
    }
    if id.contains(['{', '}', ':']) {
        return Err(Error::InvalidInput(
            "id must not contain '{', '}' or ':'".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod scene_tests;
