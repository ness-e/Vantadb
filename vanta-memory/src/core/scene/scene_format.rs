//! Scene block format (MEM-12, F4).
//!
//! A scene block is the persisted unit of an L2 scene: identity
//! (`scene_name`) + the META contract [`SceneMeta`] (`{created, updated,
//! summary, heat}`) + the scene `content`. It is the SDK-side companion of
//! the core scene node anchor (`vantadb::entity::scene::SceneNodeStore`).
//!
//! The TDAM original stores scenes as Markdown files with
//! `-----META-START-----` delimiters (`scene-format.ts:18-48`); here the
//! block is a JSON record (payload) under the `scene/<session>` namespace —
//! the META contract travels inside the JSON, so the delimiter markers are
//! not ported (they would be dead code in a record store).
//!
//! Source: `docs/research/tdam/02-scene-persona.md` §52-53.

use serde::{Deserialize, Serialize};

use crate::core::abstractions::{SceneIndexEntry, SceneMeta};

/// Content marker of a soft-deleted scene (TDAM parity: the LLM "deletes" by
/// writing this marker; the record store keeps the block with `deleted: true`
/// instead of unlinking — see [`SceneBlock::deleted`]).
pub const SOFT_DELETE_MARKER: &str = "[DELETED]";

/// A persisted scene block: identity + META contract + content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SceneBlock {
    /// Scene name (`scene_name` from the L1 scene segment, e.g.
    /// `2024-08-01-22-10`).
    pub scene_name: String,
    /// META contract `{created, updated, summary, heat}`.
    pub meta: SceneMeta,
    /// Scene content (narrative/notes of the episode).
    pub content: String,
    /// Soft-delete flag (MEM-14). `#[serde(default)]` keeps records written
    /// before MEM-14 parse as live (backward compatible).
    #[serde(default)]
    pub deleted: bool,
}

impl SceneBlock {
    /// Build a new scene block (live: `deleted = false`).
    pub fn new(scene_name: impl Into<String>, meta: SceneMeta, content: impl Into<String>) -> Self {
        Self {
            scene_name: scene_name.into(),
            meta,
            content: content.into(),
            deleted: false,
        }
    }

    /// Whether the block was soft-deleted (hidden from navigation, still
    /// recoverable via [`crate::core::scene::scene_index::get_scene`]).
    pub fn is_deleted(&self) -> bool {
        self.deleted
    }

    /// Derive the scene index entry used for listing/navigation.
    ///
    /// `filename` is the scene name (the block key); this matches the TDAM
    /// `SceneIndexEntry` shape (`scene-index.ts:9-15`) so consumers of the
    /// index see the same contract.
    pub fn index_entry(&self) -> SceneIndexEntry {
        SceneIndexEntry {
            filename: self.scene_name.clone(),
            summary: self.meta.summary.clone(),
            heat: self.meta.heat,
            created: self.meta.created.clone(),
            updated: self.meta.updated.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta() -> SceneMeta {
        SceneMeta {
            created: "2024-08-01T22:10:00.000Z".into(),
            updated: "2024-08-01T22:12:00.000Z".into(),
            summary: "user researched VantaDB pricing".into(),
            heat: 2,
        }
    }

    #[test]
    fn serde_roundtrip_snake_case() {
        let block = SceneBlock::new("2024-08-01-22-10", meta(), "content here");

        let json = serde_json::to_string(&block).expect("serialize");
        assert!(json.contains("\"scene_name\""), "snake_case field: {json}");
        assert!(json.contains("\"meta\""), "meta field: {json}");
        assert!(json.contains("\"heat\":2"), "heat value: {json}");

        let back: SceneBlock = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, block);
    }

    #[test]
    fn index_entry_maps_meta() {
        let block = SceneBlock::new("2024-08-01-22-10", meta(), "content");
        let entry = block.index_entry();

        assert_eq!(entry.filename, "2024-08-01-22-10");
        assert_eq!(entry.summary, "user researched VantaDB pricing");
        assert_eq!(entry.heat, 2);
        assert_eq!(entry.created, "2024-08-01T22:10:00.000Z");
        assert_eq!(entry.updated, "2024-08-01T22:12:00.000Z");
    }

    #[test]
    fn scene_name_matches_l1_segment_contract() {
        // SceneSegment.scene_name (L1 wire) feeds SceneBlock.scene_name directly.
        let block = SceneBlock::new("2024-08-01-22-10", meta(), "");
        assert_eq!(block.scene_name, "2024-08-01-22-10");
    }

    #[test]
    fn new_block_is_live() {
        let block = SceneBlock::new("s", meta(), "c");
        assert!(!block.is_deleted());
        assert!(!block.deleted);
    }

    #[test]
    fn serde_roundtrip_preserves_deleted_flag() {
        let mut block = SceneBlock::new("s", meta(), "c");
        block.deleted = true;
        let json = serde_json::to_string(&block).expect("serialize");
        assert!(json.contains("\"deleted\":true"), "flag serialized: {json}");
        let back: SceneBlock = serde_json::from_str(&json).expect("deserialize");
        assert!(back.is_deleted());
    }

    #[test]
    fn old_record_without_deleted_field_parses_as_live() {
        // Backward compatibility: records written before MEM-14 have no
        // `deleted` field — they must parse as live.
        let json = r#"{"scene_name":"s","meta":{"created":"c","updated":"u","summary":"s","heat":1},"content":"c"}"#;
        let block: SceneBlock = serde_json::from_str(json).expect("deserialize old record");
        assert!(!block.is_deleted());
    }

    #[test]
    fn soft_delete_marker_matches_tdam() {
        assert_eq!(SOFT_DELETE_MARKER, "[DELETED]");
    }
}
