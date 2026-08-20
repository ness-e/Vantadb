//! Scene node anchors in the core graph (MEM-12, F4).
//!
//! A scene is the L2 memory unit that groups an episode of conversation; the
//! scene NODE is the LLM-free anchor the L2 strategy reads/writes when it
//! updates the graph (CREATE/UPDATE/MERGE — MEM-14). It carries the META
//! contract `{created, updated, summary, heat}` (same contract as
//! `vanta-memory::core::abstractions::SceneMeta`) plus its identity:
//! `namespace` (deployment/tenant) + `session_id` (the L0/L1 session) +
//! `scene_name` (e.g. `2024-08-01-22-10` from the L1 scene segment).
//!
//! Storage reuses the exact [`super::EntityStore`] partition pattern (D4): a
//! JSON record in the `InternalMetadata` partition under key
//! `scene:{namespace}:{session_id}::{scene_name}` — distinguishable from
//! `entity:*` records in the same partition scan, listed by key prefix.
//!
//! The store is intentionally dumb CRUD: `set` replaces wholesale and never
//! computes timestamps or heat. META semantics (preserve `created`, bump
//! `updated`, heat = old+1) live in the L2 strategy — see
//! `vanta-memory/src/core/scene/scene_index.rs::upsert_scene` for the SDK-side
//! counterpart and MEM-14 for the graph write path.

use serde::{Deserialize, Serialize};

use crate::backend::{BackendPartition, BackendWriteOp};
use crate::error::{ChainedError, Result, VantaError};
use crate::storage::StorageEngine;

use super::{validate_key, validate_scope};

/// A single stored scene node anchor (META contract + identity).
///
/// `created`/`updated` are RFC 3339 UTC strings (ISO 8601, e.g.
/// `2024-08-01T22:10:00.000Z`); `heat` follows the `SceneMeta` convention
/// (CREATE = 1, UPDATE = old + 1).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneNode {
    pub namespace: String,
    pub session_id: String,
    pub scene_name: String,
    pub created: String,
    pub updated: String,
    pub summary: String,
    pub heat: u32,
}

/// Paginated result of [`SceneNodeStore::scene_node_list`].
#[derive(Debug, Clone, PartialEq)]
pub struct SceneNodePage {
    pub items: Vec<SceneNode>,
    pub total: usize,
}

// ── SceneNodeStore ──

/// CRUD store for scene node anchors backed by a [`StorageEngine`].
///
/// Each node is a JSON record in the `InternalMetadata` partition under key
/// `scene:{namespace}:{session_id}::{scene_name}`; listing scans the
/// session prefix and paginates. Same partition pattern as
/// [`super::EntityStore`] (D4) — no new storage mechanism.
pub struct SceneNodeStore<'a> {
    engine: &'a StorageEngine,
}

impl<'a> SceneNodeStore<'a> {
    /// Wrap a storage engine reference.
    pub fn new(engine: &'a StorageEngine) -> Self {
        Self { engine }
    }

    /// Insert or replace a scene node anchor in `namespace`/`session_id`.
    ///
    /// Wholesale replace: the caller (L2 strategy) computes `created`,
    /// `updated`, `summary` and `heat` — this store never mutates them
    /// (preserving `created` on update is the strategy's job, MEM-14).
    pub fn scene_node_set(
        &self,
        namespace: &str,
        session_id: &str,
        scene_name: &str,
        created: &str,
        updated: &str,
        summary: &str,
        heat: u32,
    ) -> Result<SceneNode> {
        validate_key(namespace, session_id, scene_name)?;
        let node = SceneNode {
            namespace: namespace.to_string(),
            session_id: session_id.to_string(),
            scene_name: scene_name.to_string(),
            created: created.to_string(),
            updated: updated.to_string(),
            summary: summary.to_string(),
            heat,
        };
        let bytes = serde_json::to_vec(&node)
            .map_err(|e| VantaError::serialization(ChainedError::with_source("scene", e)))?;
        self.engine.put_to_partition(
            BackendPartition::InternalMetadata,
            &scene_key(namespace, session_id, scene_name),
            &bytes,
        )?;
        Ok(node)
    }

    /// Retrieve a scene node by scope + scene name, or `None` when absent.
    pub fn scene_node_get(
        &self,
        namespace: &str,
        session_id: &str,
        scene_name: &str,
    ) -> Result<Option<SceneNode>> {
        validate_key(namespace, session_id, scene_name)?;
        match self.engine.get_from_partition(
            BackendPartition::InternalMetadata,
            &scene_key(namespace, session_id, scene_name),
        )? {
            Some(bytes) => serde_json::from_slice(&bytes)
                .map(Some)
                .map_err(|e| VantaError::serialization(ChainedError::with_source("scene", e))),
            None => Ok(None),
        }
    }

    /// Delete a scene node by scope + scene name. Returns `true` when it existed.
    pub fn scene_node_delete(
        &self,
        namespace: &str,
        session_id: &str,
        scene_name: &str,
    ) -> Result<bool> {
        validate_key(namespace, session_id, scene_name)?;
        let key = scene_key(namespace, session_id, scene_name);
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

    /// List scene node anchors in a `namespace`/`session_id` with pagination.
    ///
    /// Items are ordered by `scene_name` for deterministic pages; `total` is
    /// the full session size before `offset`/`limit` are applied.
    pub fn scene_node_list(
        &self,
        namespace: &str,
        session_id: &str,
        limit: usize,
        offset: usize,
    ) -> Result<SceneNodePage> {
        validate_scope(namespace, session_id)?;
        let rows = self.engine.scan_partition_prefix(
            BackendPartition::InternalMetadata,
            scene_prefix(namespace, session_id).as_bytes(),
        )?;
        let mut nodes: Vec<SceneNode> = Vec::with_capacity(rows.len());
        for (_, bytes) in rows {
            let node: SceneNode = serde_json::from_slice(&bytes)
                .map_err(|e| VantaError::serialization(ChainedError::with_source("scene", e)))?;
            nodes.push(node);
        }
        nodes.sort_by(|a, b| a.scene_name.cmp(&b.scene_name));
        let total = nodes.len();
        let items: Vec<SceneNode> = nodes.into_iter().skip(offset).take(limit).collect();
        Ok(SceneNodePage { items, total })
    }
}

// ── helpers ──

/// Key for a single scene node record in the `InternalMetadata` partition.
fn scene_key(namespace: &str, session_id: &str, scene_name: &str) -> Vec<u8> {
    format!(
        "scene:{{{}}}:{{{}}}::{{{}}}",
        namespace, session_id, scene_name
    )
    .into_bytes()
}

/// Key prefix covering every scene node record of a session.
fn scene_prefix(namespace: &str, session_id: &str) -> String {
    format!("scene:{{{}}}:{{{}}}::", namespace, session_id)
}
