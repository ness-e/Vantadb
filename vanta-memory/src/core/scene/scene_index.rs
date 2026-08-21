//! Scene index by session — LLM-free (MEM-12, F4).
//!
//! The L2 anchor: read/write the current scene of a session WITHOUT an LLM
//! call (Principio 4). Persistence goes through the VantaDB SDK (Principio
//! 2): blocks live under the `scene/<session>` namespace, key = sanitized
//! scene name, payload = the serialized [`SceneBlock`].
//!
//! There is no denormalized `scene_index.json`: the records ARE the source
//! of truth and [`list_scenes`] derives the index entries by listing (avoids
//! the TDAM write-sync bug, `scene-index.ts:63-84`).
//!
//! Semantics (documented in `core::abstractions::SceneMeta`): CREATE sets
//! `heat = 1`, UPDATE bumps `heat = old + 1` and refreshes `updated` while
//! preserving `created`. The MERGE branch (`heat = sum + 1`) belongs to the
//! L2 strategy (MEM-14).
//!
//! Source: `docs/research/tdam/02-scene-persona.md` §53 (TDAM
//! `scene-index.ts`).

use thiserror::Error;

use crate::core::abstractions::{SceneIndexEntry, SceneMeta};
use crate::core::conversation::{now_ms, sanitize_component, sanitize_key};
use crate::core::prompts::l1_extraction::epoch_ms_to_rfc3339;
use crate::core::scene::scene_format::{SceneBlock, SOFT_DELETE_MARKER};

/// Errors from the LLM-free scene index.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SceneError {
    /// Underlying VantaDB storage error.
    #[error("vantadb: {0}")]
    Vanta(#[from] vantadb::error::VantaError),
    /// Scene block failed to (de)serialize.
    #[error("scene record: {0}")]
    Serde(#[from] serde_json::Error),
}

/// `scene/<sanitized-session>` — persisted scene block records namespace.
pub fn scene_namespace(session_key: &str) -> String {
    format!("scene/{}", sanitize_component(session_key, 128, false))
}

/// Create or update the scene block of a session (the L2 anchor).
///
/// CREATE: `created = updated = now`, `heat = 1`. UPDATE: `created`
/// preserved, `updated = now`, `heat = old + 1`. Returns the written block.
pub fn upsert_scene(
    db: &vantadb::sdk::VantaEmbedded,
    session_key: &str,
    scene_name: &str,
    summary: &str,
    content: &str,
) -> Result<SceneBlock, SceneError> {
    let now = epoch_ms_to_rfc3339(now_ms());
    let existing = get_scene(db, session_key, scene_name)?;
    let meta = match existing {
        Some(block) => SceneMeta {
            created: block.meta.created,
            updated: now.clone(),
            summary: summary.to_string(),
            heat: block.meta.heat + 1,
        },
        None => SceneMeta {
            created: now.clone(),
            updated: now,
            summary: summary.to_string(),
            heat: 1,
        },
    };
    let block = SceneBlock::new(scene_name, meta, content);
    write_scene_block(db, session_key, &block)?;
    Ok(block)
}

/// Mark a scene block as soft-deleted (MEM-14): sets `deleted: true` and
/// replaces the content with [`SOFT_DELETE_MARKER`][crate::core::scene::scene_format::SOFT_DELETE_MARKER],
/// preserving the META contract (`created`/`updated`/`summary`/`heat` — parity
/// with the TDAM cleanup that never touches META).
///
/// Idempotent: a missing scene returns `Ok(None)`; an already-deleted scene
/// returns the existing block unchanged. Returns the written block on success.
pub fn soft_delete_scene(
    db: &vantadb::sdk::VantaEmbedded,
    session_key: &str,
    scene_name: &str,
) -> Result<Option<SceneBlock>, SceneError> {
    let Some(mut block) = get_scene(db, session_key, scene_name)? else {
        return Ok(None);
    };
    if block.deleted {
        return Ok(Some(block));
    }
    block.deleted = true;
    block.content = SOFT_DELETE_MARKER.to_string();
    write_scene_block(db, session_key, &block)?;
    Ok(Some(block))
}

/// Read a scene block by name, if present.
pub fn get_scene(
    db: &vantadb::sdk::VantaEmbedded,
    session_key: &str,
    scene_name: &str,
) -> Result<Option<SceneBlock>, SceneError> {
    let ns = scene_namespace(session_key);
    let key = sanitize_key(scene_name);
    match db.get(&ns, &key)? {
        Some(record) => Ok(Some(serde_json::from_str(&record.payload)?)),
        None => Ok(None),
    }
}

/// List the scene index entries of a session, ordered for navigation
/// (heat descending, then `updated` descending). Soft-deleted scenes are
/// excluded (MEM-14); recover them via [`get_scene`].
pub fn list_scenes(
    db: &vantadb::sdk::VantaEmbedded,
    session_key: &str,
) -> Result<Vec<SceneIndexEntry>, SceneError> {
    let blocks = read_blocks(db, session_key)?;
    let mut entries: Vec<SceneIndexEntry> = blocks
        .iter()
        .filter(|b| !b.is_deleted())
        .map(SceneBlock::index_entry)
        .collect();
    entries.sort_by(|a, b| b.heat.cmp(&a.heat).then_with(|| b.updated.cmp(&a.updated)));
    Ok(entries)
}

/// The current (most recently updated) scene block of a session, if any.
/// Soft-deleted scenes are excluded (MEM-14).
///
/// Compares `updated` lexicographically: every timestamp comes from
/// [`epoch_ms_to_rfc3339`] with a fixed-width format, so string order equals
/// chronological order.
pub fn current_scene(
    db: &vantadb::sdk::VantaEmbedded,
    session_key: &str,
) -> Result<Option<SceneBlock>, SceneError> {
    let blocks = read_blocks(db, session_key)?;
    Ok(blocks
        .into_iter()
        .filter(|b| !b.is_deleted())
        .max_by(|a, b| a.meta.updated.cmp(&b.meta.updated)))
}

// ── helpers ──

/// Write a scene block exactly as given (payload = serialized block).
///
/// Public because the L2 strategy (MEM-14) needs exact-write for the MERGE
/// branch — `upsert_scene` cannot express `heat = sum + 1`. All other callers
/// go through [`upsert_scene`] (CREATE/UPDATE heat semantics) or
/// [`soft_delete_scene`].
pub fn write_scene_block(
    db: &vantadb::sdk::VantaEmbedded,
    session_key: &str,
    block: &SceneBlock,
) -> Result<(), SceneError> {
    use vantadb::sdk::{VantaMemoryInput, VantaMemoryMetadata};

    let ns = scene_namespace(session_key);
    let key = sanitize_key(&block.scene_name);
    let payload = serde_json::to_string(block)?;
    db.put(VantaMemoryInput {
        namespace: ns,
        key,
        payload,
        metadata: VantaMemoryMetadata::new(),
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
    })?;
    Ok(())
}

/// Read every scene block of a session (paged via list, tolerant of
/// individual corrupt records — same pattern as `l1_reader::read_session_records`).
///
/// `pub(crate)`: the gateway query handler (MEM-21) needs full blocks
/// (content) which the index entries do not carry.
pub(crate) fn read_blocks(
    db: &vantadb::sdk::VantaEmbedded,
    session_key: &str,
) -> Result<Vec<SceneBlock>, SceneError> {
    use vantadb::sdk::{VantaMemoryListOptions, VantaMemoryListPage};

    let ns = scene_namespace(session_key);
    let mut blocks = Vec::new();
    let mut cursor: Option<usize> = None;

    loop {
        let options = VantaMemoryListOptions {
            limit: 1000,
            cursor,
            ..Default::default()
        };
        let page: VantaMemoryListPage = db.list(&ns, options)?;
        for record in page.records {
            if let Ok(block) = serde_json::from_str::<SceneBlock>(&record.payload) {
                blocks.push(block);
            } else {
                tracing::debug!(key = %record.key, "scene record failed to deserialize; skipped");
            }
        }
        match page.next_cursor {
            Some(next) => cursor = Some(next),
            None => break,
        }
    }

    Ok(blocks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::abstractions::SceneSegment;
    use vantadb::config::VantaConfig;
    use vantadb::storage::BackendKind;

    fn open_db() -> vantadb::sdk::VantaEmbedded {
        let config = VantaConfig {
            backend_kind: BackendKind::InMemory,
            read_only: false,
            ..VantaConfig::default()
        };
        vantadb::sdk::VantaEmbedded::open_with_config(config).expect("open in-memory db")
    }

    #[test]
    fn namespace_is_scene_prefixed_and_sanitized() {
        assert_eq!(scene_namespace("sess-1"), "scene/sess-1");
        // sanitize_component replaces non-safe chars with '_'.
        assert_eq!(scene_namespace("sess/../1"), "scene/sess_.._1");
    }

    #[test]
    fn error_wraps_vanta_error() {
        let err = SceneError::Vanta(vantadb::error::VantaError::InvalidInput("x".into()));
        assert!(err.to_string().contains("vantadb:"));
    }

    #[test]
    fn scene_name_flows_from_l1_segment() {
        // The L1 wire (SceneSegment.scene_name) feeds SceneBlock directly.
        let segment = SceneSegment {
            scene_name: "2024-08-01-22-10".into(),
            message_ids: vec![],
            memories: vec![],
        };
        let block = SceneBlock::new(
            segment.scene_name,
            SceneMeta {
                created: "2024-08-01T22:10:00.000Z".into(),
                updated: "2024-08-01T22:10:00.000Z".into(),
                summary: "s".into(),
                heat: 1,
            },
            "",
        );
        assert_eq!(block.scene_name, "2024-08-01-22-10");
    }

    #[test]
    fn soft_delete_marks_and_preserves_meta() {
        let db = open_db();
        let block = upsert_scene(&db, "sess-1", "scene-a", "summary", "content").expect("create");
        let created = block.meta.created.clone();
        let heat = block.meta.heat;

        let deleted = soft_delete_scene(&db, "sess-1", "scene-a")
            .expect("soft delete")
            .expect("exists");
        assert!(deleted.is_deleted());
        assert_eq!(deleted.content, SOFT_DELETE_MARKER);
        assert_eq!(deleted.meta.created, created, "created preserved");
        assert_eq!(deleted.meta.heat, heat, "heat preserved");
    }

    #[test]
    fn soft_delete_missing_returns_none() {
        let db = open_db();
        let result = soft_delete_scene(&db, "sess-1", "ghost").expect("no error");
        assert!(result.is_none());
    }

    #[test]
    fn soft_delete_is_idempotent() {
        let db = open_db();
        upsert_scene(&db, "sess-1", "scene-a", "s", "c").expect("create");
        soft_delete_scene(&db, "sess-1", "scene-a").expect("first delete");
        let again = soft_delete_scene(&db, "sess-1", "scene-a")
            .expect("second delete")
            .expect("exists");
        assert!(again.is_deleted());
    }

    #[test]
    fn list_and_current_exclude_deleted_but_get_recovers() {
        let db = open_db();
        upsert_scene(&db, "sess-1", "scene-live", "s", "c").expect("create live");
        upsert_scene(&db, "sess-1", "scene-dead", "s", "c").expect("create dead");
        soft_delete_scene(&db, "sess-1", "scene-dead").expect("delete");

        let names: Vec<String> = list_scenes(&db, "sess-1")
            .expect("list")
            .iter()
            .map(|e| e.filename.clone())
            .collect();
        assert_eq!(
            names,
            vec!["scene-live"],
            "deleted excluded from list: {names:?}"
        );

        let current = current_scene(&db, "sess-1")
            .expect("current")
            .expect("live exists");
        assert_eq!(
            current.scene_name, "scene-live",
            "deleted excluded from current"
        );

        let recovered = get_scene(&db, "sess-1", "scene-dead")
            .expect("get")
            .expect("recover");
        assert!(recovered.is_deleted(), "get_scene returns deleted blocks");
    }

    #[test]
    fn upsert_resurrects_deleted_scene() {
        let db = open_db();
        upsert_scene(&db, "sess-1", "scene-a", "old", "old content").expect("create");
        soft_delete_scene(&db, "sess-1", "scene-a").expect("delete");

        let revived = upsert_scene(&db, "sess-1", "scene-a", "new", "new content").expect("upsert");
        assert!(!revived.is_deleted(), "upsert writes a live block");
        assert_eq!(revived.content, "new content");
        assert_eq!(revived.meta.heat, 2, "heat bumps from old+1 (old was 1)");
    }
}
