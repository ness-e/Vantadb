//! D19 integration tests for the LLM-free scene index + META contract
//! (MEM-12, F4).
//!
//! Pattern AAA: arrange → act → assert. Uses an in-memory `VantaEmbedded`
//! (same setup as `l0_capture.rs`).

use vanta_memory::core::scene::scene_format::SceneBlock;
use vanta_memory::core::scene::scene_index::SceneError;
use vanta_memory::core::scene::{
    current_scene, get_scene, list_scenes, scene_namespace, upsert_scene,
};
use vantadb::config::VantaConfig;
use vantadb::sdk::VantaEmbedded;
use vantadb::storage::BackendKind;

fn open_db() -> VantaEmbedded {
    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        read_only: false,
        ..VantaConfig::default()
    };
    VantaEmbedded::open_with_config(config).expect("open in-memory db")
}

const SESSION: &str = "sess-1";
const SCENE: &str = "2024-08-01-22-10";
const SUMMARY: &str = "user researched VantaDB pricing";
const CONTENT: &str = "notes about the pricing page";

#[test]
fn scene_block_serde_roundtrip() {
    let block = SceneBlock::new(
        SCENE,
        vanta_memory::core::abstractions::SceneMeta {
            created: "2024-08-01T22:10:00.000Z".into(),
            updated: "2024-08-01T22:10:00.000Z".into(),
            summary: SUMMARY.into(),
            heat: 1,
        },
        CONTENT,
    );
    let json = serde_json::to_string(&block).expect("serialize");
    let back: SceneBlock = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, block);
    assert_eq!(back.index_entry().filename, SCENE);
}

#[test]
fn upsert_create_sets_heat_one_and_timestamps() {
    let db = open_db();
    let block = upsert_scene(&db, SESSION, SCENE, SUMMARY, CONTENT).expect("create scene");

    assert_eq!(block.scene_name, SCENE);
    assert_eq!(block.meta.summary, SUMMARY);
    assert_eq!(block.meta.heat, 1);
    assert_eq!(
        block.meta.created, block.meta.updated,
        "create: created == updated"
    );

    let got = get_scene(&db, SESSION, SCENE)
        .expect("get")
        .expect("exists");
    assert_eq!(got, block);
}

#[test]
fn upsert_update_preserves_created_bumps_heat() {
    let db = open_db();
    let first = upsert_scene(&db, SESSION, SCENE, SUMMARY, CONTENT).expect("create");

    // Same scene, new summary → UPDATE.
    let second =
        upsert_scene(&db, SESSION, SCENE, "updated summary", "updated content").expect("update");

    assert_eq!(second.meta.heat, 2, "UPDATE bumps heat to old+1");
    assert_eq!(second.meta.created, first.meta.created, "created preserved");
    assert!(
        second.meta.updated > first.meta.updated,
        "updated refreshed"
    );
    assert_eq!(second.meta.summary, "updated summary");
    assert_eq!(second.content, "updated content");
}

#[test]
fn get_missing_returns_none() {
    let db = open_db();
    assert!(get_scene(&db, SESSION, "nope")
        .expect("get missing")
        .is_none());
}

#[test]
fn list_scenes_sorted_by_heat_then_updated() {
    let db = open_db();

    upsert_scene(&db, SESSION, "2024-08-01-22-10", "s1", "c1").expect("create s1");
    upsert_scene(&db, SESSION, "2024-08-01-23-00", "s2", "c2").expect("create s2");
    // Update s1 again → heat 2, newer updated than s2 (heat 1).
    upsert_scene(&db, SESSION, "2024-08-01-22-10", "s1-updated", "c1-updated").expect("update s1");

    let entries = list_scenes(&db, SESSION).expect("list");
    assert_eq!(entries.len(), 2);
    assert_eq!(
        entries[0].filename, "2024-08-01-22-10",
        "highest heat first"
    );
    assert_eq!(entries[0].heat, 2);
    assert_eq!(entries[1].filename, "2024-08-01-23-00");
    assert_eq!(entries[1].heat, 1);
}

#[test]
fn sessions_are_isolated() {
    let db = open_db();
    upsert_scene(&db, SESSION, SCENE, SUMMARY, CONTENT).expect("create in sess-1");
    upsert_scene(&db, "sess-2", SCENE, "other", "other").expect("create in sess-2");

    assert_eq!(list_scenes(&db, SESSION).expect("list").len(), 1);
    assert_eq!(list_scenes(&db, "sess-2").expect("list").len(), 1);
    assert_eq!(scene_namespace(SESSION), "scene/sess-1");
    assert_eq!(scene_namespace("sess-2"), "scene/sess-2");
}

#[test]
fn current_scene_returns_most_recently_updated() {
    let db = open_db();
    assert!(current_scene(&db, SESSION).expect("none").is_none());

    upsert_scene(&db, SESSION, "2024-08-01-22-10", "s1", "c1").expect("create s1");
    upsert_scene(&db, SESSION, "2024-08-01-23-00", "s2", "c2").expect("create s2");
    // Update s1 → it becomes the current scene again.
    upsert_scene(&db, SESSION, "2024-08-01-22-10", "s1-updated", "c1-updated").expect("update s1");

    let current = current_scene(&db, SESSION)
        .expect("current")
        .expect("exists");
    assert_eq!(current.scene_name, "2024-08-01-22-10");
}

#[test]
fn scene_name_with_invalid_chars_is_sanitized_but_retrievable() {
    let db = open_db();
    let block = upsert_scene(&db, SESSION, "scene/../x", "s", "c").expect("create sanitized scene");

    // Key stored under the sanitized form; get with the raw name resolves it.
    let got = get_scene(&db, SESSION, "scene/../x")
        .expect("get")
        .expect("exists");
    assert_eq!(got, block);
}

#[test]
fn error_type_is_displayable() {
    let err = SceneError::Vanta(vantadb::error::VantaError::InvalidInput("x".into()));
    assert!(err.to_string().contains("vantadb:"));
}
