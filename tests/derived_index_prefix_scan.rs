//! Prefix-scan certification for derived memory indexes.

use tempfile::tempdir;
use vantadb::{VantaEmbedded, VantaMemoryInput, VantaMemoryListOptions, VantaValue};

fn str_value(value: &str) -> VantaValue {
    VantaValue::String(value.to_string())
}

#[test]
fn namespace_and_filter_paths_use_prefix_scans() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");

    for i in 0..30usize {
        let namespace = if i % 2 == 0 { "agent/a" } else { "agent/b" };
        let mut input = VantaMemoryInput::new(namespace, format!("key-{i:02}"), "payload");
        input.metadata.insert(
            "kind".to_string(),
            str_value(if i % 3 == 0 { "task" } else { "note" }),
        );
        db.put(input).expect("put");
    }

    let before = db.operational_metrics();

    let page = db
        .list("agent/a", VantaMemoryListOptions::default())
        .expect("list");
    assert_eq!(page.records.len(), 15);
    assert!(page.records.iter().all(|r| r.namespace == "agent/a"));

    let mut filters = std::collections::BTreeMap::new();
    filters.insert("kind".to_string(), str_value("task"));
    let filtered = db
        .list(
            "agent/a",
            VantaMemoryListOptions {
                filters,
                limit: 100,
                cursor: None,
            },
        )
        .expect("filtered list");
    assert_eq!(filtered.records.len(), 5);

    let after = db.operational_metrics();
    assert!(after.derived_prefix_scans >= before.derived_prefix_scans + 2);
    assert_eq!(
        after.derived_full_scan_fallbacks,
        before.derived_full_scan_fallbacks
    );
}
