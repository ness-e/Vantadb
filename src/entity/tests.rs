//! Dedicated CRUD tests for the entity store (D19).
//!
//! Pattern AAA: arrange → act → assert. Uses an in-memory `StorageEngine`
//! (same setup as `src/storage/engine/tests/mod.rs`).

use super::{generate_id, EntityStore, EntityWrite};
use crate::config::Config;
use crate::node::FieldValue;
use crate::storage::{BackendKind, StorageEngine};
use std::collections::HashMap;

fn in_memory_engine() -> StorageEngine {
    let config = Config {
        backend_kind: BackendKind::InMemory,
        read_only: false,
        ..Config::default()
    };
    StorageEngine::open_with_config(":memory:", Some(config)).expect("open in-memory engine")
}

fn user_fields(name: &str, active: bool) -> HashMap<String, FieldValue> {
    let mut fields = HashMap::new();
    fields.insert("name".to_string(), FieldValue::String(name.to_string()));
    fields.insert("active".to_string(), FieldValue::Bool(active));
    fields
}

#[test]
fn set_get_roundtrip() {
    let engine = in_memory_engine();
    let store = EntityStore::new(&engine);

    let stored = store
        .set(EntityWrite {
            namespace: "default",
            collection: "user",
            id: "usr-1",
            fields: user_fields("alice", true),
        })
        .expect("set user");

    assert_eq!(stored.namespace, "default");
    assert_eq!(stored.collection, "user");
    assert_eq!(stored.id, "usr-1");
    assert_eq!(
        stored.fields.get("name"),
        Some(&FieldValue::String("alice".into()))
    );
    assert_eq!(stored.fields.get("active"), Some(&FieldValue::Bool(true)));
    assert!(stored.created_at > 0);
    assert_eq!(stored.created_at, stored.updated_at);

    let got = store
        .get("default", "user", "usr-1")
        .expect("get user")
        .expect("user exists");
    assert_eq!(got, stored);
}

#[test]
fn get_missing_returns_none() {
    let engine = in_memory_engine();
    let store = EntityStore::new(&engine);

    let got = store.get("default", "user", "usr-missing").expect("get");
    assert!(got.is_none());
}

#[test]
fn set_upsert_preserves_created_at_and_refreshes_updated_at() {
    let engine = in_memory_engine();
    let store = EntityStore::new(&engine);

    let first = store
        .set(EntityWrite {
            namespace: "default",
            collection: "user",
            id: "usr-1",
            fields: user_fields("alice", true),
        })
        .expect("first set");

    let second = store
        .set(EntityWrite {
            namespace: "default",
            collection: "user",
            id: "usr-1",
            fields: user_fields("alice-updated", false),
        })
        .expect("second set");

    assert_eq!(
        second.created_at, first.created_at,
        "created_at preserved on upsert"
    );
    assert!(
        second.updated_at >= first.updated_at,
        "updated_at refreshed on upsert"
    );
    assert_eq!(
        second.fields.get("name"),
        Some(&FieldValue::String("alice-updated".into())),
        "fields replaced wholesale"
    );
}

#[test]
fn delete_existing_returns_true_then_get_none() {
    let engine = in_memory_engine();
    let store = EntityStore::new(&engine);

    store
        .set(EntityWrite {
            namespace: "default",
            collection: "user",
            id: "usr-1",
            fields: user_fields("alice", true),
        })
        .expect("set");

    let deleted = store.delete("default", "user", "usr-1").expect("delete");
    assert!(deleted, "delete of existing entity returns true");

    let got = store.get("default", "user", "usr-1").expect("get");
    assert!(got.is_none(), "entity gone after delete");
}

#[test]
fn delete_missing_returns_false() {
    let engine = in_memory_engine();
    let store = EntityStore::new(&engine);

    let deleted = store
        .delete("default", "user", "usr-ghost")
        .expect("delete");
    assert!(!deleted, "delete of missing entity returns false");
}

#[test]
fn list_paginates_sorted_by_id() {
    let engine = in_memory_engine();
    let store = EntityStore::new(&engine);

    // Insert out of order; listing must be deterministic by id.
    for id in ["usr-z", "usr-a", "usr-m", "usr-b", "usr-y"] {
        store
            .set(EntityWrite {
                namespace: "default",
                collection: "user",
                id,
                fields: user_fields(id, true),
            })
            .expect("set");
    }

    let page = store.list("default", "user", 2, 1).expect("list");

    assert_eq!(page.total, 5, "total counts the whole collection");
    assert_eq!(
        page.items.iter().map(|e| e.id.as_str()).collect::<Vec<_>>(),
        vec!["usr-b", "usr-m"],
        "sorted by id, offset 1 limit 2"
    );
}

#[test]
fn list_isolates_namespaces() {
    let engine = in_memory_engine();
    let store = EntityStore::new(&engine);

    store
        .set(EntityWrite {
            namespace: "ns-a",
            collection: "user",
            id: "usr-1",
            fields: user_fields("alice", true),
        })
        .expect("set ns-a");
    store
        .set(EntityWrite {
            namespace: "ns-b",
            collection: "user",
            id: "usr-1",
            fields: user_fields("bob", true),
        })
        .expect("set ns-b");

    let page = store.list("ns-a", "user", 10, 0).expect("list ns-a");
    assert_eq!(page.total, 1, "namespaces are isolated");
    assert_eq!(page.items[0].namespace, "ns-a");
    assert_eq!(
        page.items[0].fields.get("name"),
        Some(&FieldValue::String("alice".into()))
    );
}

#[test]
fn list_isolates_collections() {
    let engine = in_memory_engine();
    let store = EntityStore::new(&engine);

    store
        .set(EntityWrite {
            namespace: "default",
            collection: "user",
            id: "usr-1",
            fields: user_fields("alice", true),
        })
        .expect("set user");
    store
        .set(EntityWrite {
            namespace: "default",
            collection: "team",
            id: "team-1",
            fields: user_fields("acme", true),
        })
        .expect("set team");

    let page = store.list("default", "user", 10, 0).expect("list users");
    assert_eq!(page.total, 1, "collections are isolated");
    assert_eq!(page.items[0].collection, "user");
}

#[test]
fn invalid_inputs_rejected() {
    let engine = in_memory_engine();
    let store = EntityStore::new(&engine);

    assert!(
        store
            .set(EntityWrite {
                namespace: "",
                collection: "user",
                id: "usr-1",
                fields: HashMap::new(),
            })
            .is_err(),
        "empty namespace"
    );
    assert!(
        store
            .set(EntityWrite {
                namespace: "default",
                collection: "",
                id: "usr-1",
                fields: HashMap::new(),
            })
            .is_err(),
        "empty collection"
    );
    assert!(
        store
            .set(EntityWrite {
                namespace: "default",
                collection: "user",
                id: "",
                fields: HashMap::new(),
            })
            .is_err(),
        "empty id"
    );
    assert!(
        store
            .set(EntityWrite {
                namespace: "def{ault",
                collection: "user",
                id: "usr-1",
                fields: HashMap::new(),
            })
            .is_err(),
        "namespace braces"
    );
    assert!(
        store
            .set(EntityWrite {
                namespace: "default",
                collection: "user",
                id: "usr:1",
                fields: HashMap::new(),
            })
            .is_err(),
        "id colon"
    );

    assert!(
        store.get("default", "user", "").is_err(),
        "get with empty id"
    );
    assert!(
        store.delete("default", "user", "").is_err(),
        "delete with empty id"
    );
    assert!(
        store.list("default", "", 10, 0).is_err(),
        "list with empty collection"
    );
}

#[test]
fn generate_id_format_and_uniqueness() {
    let a = generate_id("usr");
    let b = generate_id("usr");
    let c = generate_id("team");

    assert!(a.starts_with("usr-"), "prefix applied");
    assert!(b.starts_with("usr-"), "prefix applied");
    assert!(c.starts_with("team-"), "prefix applied");
    // "{prefix}-" (4) + 4 ts digits + 6 random digits = 14 chars.
    assert_eq!(a.len(), 14);
    assert_ne!(a, b, "ids are unique");
    assert!(a
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-'));
}

#[test]
fn legacy_json_deserializes_to_new_field() {
    // STU-003 S1 (RED): formato viejo en disco usa `entity_id`; el struct
    // nuevo expone `id` con `#[serde(alias = "entity_id")]`.
    let entity: super::Entity = serde_json::from_str(
        r#"{"namespace":"default","collection":"user","entity_id":"usr-1","fields":{},"created_at":1,"updated_at":1}"#,
    )
    .expect("legacy entity JSON deserializes");
    assert_eq!(entity.id, "usr-1");

    let bytes = serde_json::to_vec(&entity).expect("serialize");
    let value: serde_json::Value = serde_json::from_slice(&bytes).expect("value");
    assert_eq!(value["id"], "usr-1");
    assert!(
        value.get("entity_id").is_none(),
        "new writes use `id`, not `entity_id`"
    );
}
