//! Dedicated CRUD tests for the scene node store (D19, MEM-12).
//!
//! Pattern AAA: arrange → act → assert. Uses an in-memory `StorageEngine`
//! (same setup as `super::tests`).

use super::SceneNodeStore;
use crate::config::Config;
use crate::storage::{BackendKind, StorageEngine};

fn in_memory_engine() -> StorageEngine {
    let config = Config {
        backend_kind: BackendKind::InMemory,
        read_only: false,
        ..Config::default()
    };
    StorageEngine::open_with_config(":memory:", Some(config)).expect("open in-memory engine")
}

const NS: &str = "default";
const SESSION: &str = "sess-1";
const SCENE: &str = "2024-08-01-22-10";
const CREATED: &str = "2024-08-01T22:10:00.000Z";
const UPDATED: &str = "2024-08-01T22:12:00.000Z";
const SUMMARY: &str = "user researched VantaDB pricing";

#[test]
fn set_get_roundtrip() {
    let engine = in_memory_engine();
    let store = SceneNodeStore::new(&engine);

    let stored = store
        .set(NS, SESSION, SCENE, CREATED, UPDATED, SUMMARY, 1)
        .expect("set scene");

    assert_eq!(stored.namespace, NS);
    assert_eq!(stored.session_id, SESSION);
    assert_eq!(stored.name, SCENE);
    assert_eq!(stored.created, CREATED);
    assert_eq!(stored.updated, UPDATED);
    assert_eq!(stored.summary, SUMMARY);
    assert_eq!(stored.heat, 1);

    let got = store
        .get(NS, SESSION, SCENE)
        .expect("get scene")
        .expect("scene exists");
    assert_eq!(got, stored);
}

#[test]
fn get_missing_returns_none() {
    let engine = in_memory_engine();
    let store = SceneNodeStore::new(&engine);

    let got = store.get(NS, SESSION, "nope").expect("get missing");
    assert!(got.is_none());
}

#[test]
fn set_replaces_wholesale() {
    let engine = in_memory_engine();
    let store = SceneNodeStore::new(&engine);

    store
        .set(NS, SESSION, SCENE, CREATED, UPDATED, SUMMARY, 1)
        .expect("set first");
    // L2-style update: caller preserves created, bumps updated/heat.
    let replaced = store
        .set(
            NS,
            SESSION,
            SCENE,
            CREATED,
            "2024-08-01T22:20:00.000Z",
            "new summary",
            2,
        )
        .expect("set update");

    assert_eq!(
        replaced.created, CREATED,
        "caller-supplied created preserved"
    );
    assert_eq!(replaced.updated, "2024-08-01T22:20:00.000Z");
    assert_eq!(replaced.summary, "new summary");
    assert_eq!(replaced.heat, 2);
    let got = store
        .get(NS, SESSION, SCENE)
        .expect("get scene")
        .expect("scene exists");
    assert_eq!(got, replaced);
}

#[test]
fn validation_rejects_bad_name() {
    let engine = in_memory_engine();
    let store = SceneNodeStore::new(&engine);

    let err = store
        .set(NS, SESSION, "bad:name", CREATED, UPDATED, SUMMARY, 1)
        .expect_err("colon in name rejected");
    assert!(err.to_string().contains("must not contain"), "err: {err}");

    let err = store
        .get(NS, "", SCENE)
        .expect_err("empty session rejected");
    assert!(err.to_string().contains("non-empty"), "err: {err}");
}

#[test]
fn list_isolates_sessions_and_paginates() {
    let engine = in_memory_engine();
    let store = SceneNodeStore::new(&engine);

    for (i, scene) in ["a-scene", "b-scene", "c-scene"].iter().enumerate() {
        store
            .set(NS, SESSION, scene, CREATED, UPDATED, SUMMARY, i as u32 + 1)
            .expect("set scene");
    }
    store
        .set(NS, "other-session", "z-scene", CREATED, UPDATED, SUMMARY, 1)
        .expect("set other session");

    let page = store.list(NS, SESSION, 2, 0).expect("list page 1");
    assert_eq!(page.total, 3, "total counts only this session");
    let names: Vec<&str> = page.items.iter().map(|n| n.name.as_str()).collect();
    assert_eq!(names, ["a-scene", "b-scene"], "sorted by name");

    let page2 = store.list(NS, SESSION, 2, 2).expect("list page 2");
    let names2: Vec<&str> = page2.items.iter().map(|n| n.name.as_str()).collect();
    assert_eq!(names2, ["c-scene"]);
}

#[test]
fn delete_returns_existed() {
    let engine = in_memory_engine();
    let store = SceneNodeStore::new(&engine);

    store
        .set(NS, SESSION, SCENE, CREATED, UPDATED, SUMMARY, 1)
        .expect("set scene");

    assert!(store.delete(NS, SESSION, SCENE).expect("delete existing"));
    assert!(!store.delete(NS, SESSION, SCENE).expect("delete missing"));
    assert!(store
        .get(NS, SESSION, SCENE)
        .expect("get after delete")
        .is_none());
}

#[test]
fn legacy_json_deserializes_to_new_field() {
    // STU-003 S1 (RED): formato viejo en disco usa `scene_name`; el struct
    // nuevo expone `name` con `#[serde(alias = "scene_name")]`.
    let node: super::SceneNode = serde_json::from_str(
        r#"{"namespace":"default","session_id":"s","scene_name":"n","created":"c","updated":"u","summary":"s","heat":1}"#,
    )
    .expect("legacy scene JSON deserializes");
    assert_eq!(node.name, "n");

    let bytes = serde_json::to_vec(&node).expect("serialize");
    let value: serde_json::Value = serde_json::from_slice(&bytes).expect("value");
    assert_eq!(value["name"], "n");
    assert!(
        value.get("scene_name").is_none(),
        "new writes use `name`, not `scene_name`"
    );
}
