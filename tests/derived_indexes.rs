//! Derived namespace/payload index certification for persistent memory APIs.

use tempfile::tempdir;
use vantadb::{VantaEmbedded, VantaMemoryInput, VantaMemoryListOptions, VantaValue};

fn str_value(value: &str) -> VantaValue {
    VantaValue::String(value.to_string())
}

fn input(namespace: &str, key: &str, category: &str) -> VantaMemoryInput {
    let mut input = VantaMemoryInput::new(namespace, key, format!("{category} payload"));
    input
        .metadata
        .insert("category".to_string(), str_value(category));
    input
}

#[test]
fn derived_indexes_isolate_namespaces_and_filters() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");

    db.put(input("agent/a", "shared", "task")).expect("put a");
    db.put(input("agent/b", "shared", "task")).expect("put b");
    db.put(input("agent/a", "note", "note")).expect("put note");

    let page_a = db
        .list("agent/a", VantaMemoryListOptions::default())
        .expect("list a");
    assert_eq!(page_a.records.len(), 2);
    assert!(page_a
        .records
        .iter()
        .all(|record| record.namespace == "agent/a"));

    let mut filters = std::collections::BTreeMap::new();
    filters.insert("category".to_string(), str_value("task"));
    let filtered = db
        .list(
            "agent/a",
            VantaMemoryListOptions {
                filters,
                limit: 10,
                cursor: None,
            },
        )
        .expect("filtered list");
    assert_eq!(filtered.records.len(), 1);
    assert_eq!(filtered.records[0].key, "shared");
    assert_eq!(filtered.records[0].payload, "task payload");
}

#[test]
fn upsert_and_delete_keep_payload_indexes_current() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");

    db.put(input("agent/main", "item", "old")).expect("put old");

    let mut updated = VantaMemoryInput::new("agent/main", "item", "new payload");
    updated
        .metadata
        .insert("category".to_string(), str_value("new"));
    db.put(updated).expect("put updated");

    let mut old_filter = std::collections::BTreeMap::new();
    old_filter.insert("category".to_string(), str_value("old"));
    let old_page = db
        .list(
            "agent/main",
            VantaMemoryListOptions {
                filters: old_filter,
                limit: 10,
                cursor: None,
            },
        )
        .expect("old filter");
    assert!(old_page.records.is_empty());

    let mut new_filter = std::collections::BTreeMap::new();
    new_filter.insert("category".to_string(), str_value("new"));
    let new_page = db
        .list(
            "agent/main",
            VantaMemoryListOptions {
                filters: new_filter,
                limit: 10,
                cursor: None,
            },
        )
        .expect("new filter");
    assert_eq!(new_page.records.len(), 1);
    assert_eq!(new_page.records[0].payload, "new payload");

    assert!(db.delete("agent/main", "item").expect("delete"));
    let empty = db
        .list("agent/main", VantaMemoryListOptions::default())
        .expect("list after delete");
    assert!(empty.records.is_empty());
}

#[test]
fn rebuild_index_reconstructs_derived_indexes_from_canonical_records() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");

    db.put(input("agent/main", "a", "task")).expect("put a");
    db.put(input("agent/main", "b", "note")).expect("put b");
    db.flush().expect("flush");

    let report = db.rebuild_index().expect("rebuild");
    assert!(report.success);
    assert!(report.scanned_nodes >= 2);

    let namespaces = db.list_namespaces().expect("namespaces");
    assert_eq!(namespaces, vec!["agent/main".to_string()]);

    let page = db
        .list("agent/main", VantaMemoryListOptions::default())
        .expect("list");
    assert_eq!(page.records.len(), 2);
}
