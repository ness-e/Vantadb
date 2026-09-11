#![cfg(debug_assertions)]
#![allow(clippy::expect_used, clippy::unwrap_used)]

//! Recovery certification for stale/corrupt derived memory index state.

use tempfile::tempdir;
use vantadb::{Embedded, MemoryInput, MemoryListOptions, Value};

fn str_value(value: &str) -> Value {
    Value::String(value.to_string())
}

#[test]
fn corrupt_state_and_missing_entries_rebuild_on_reopen() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().to_path_buf();

    {
        let db = Embedded::open(&path).expect("open");
        let mut first = MemoryInput::new("agent/main", "a", "alpha");
        first.metadata.insert("kind".to_string(), str_value("task"));
        db.put(first).expect("put first");
        db.put(MemoryInput::new("agent/main", "b", "beta"))
            .expect("put second");
        db.flush().expect("flush");

        db.debug_clear_derived_indexes_for_tests()
            .expect("clear derived indexes");
        db.debug_corrupt_derived_index_state_for_tests()
            .expect("corrupt derived state");
    }

    let reopened = Embedded::open(&path).expect("reopen");
    let page = reopened
        .list("agent/main", MemoryListOptions::default())
        .expect("list after repair");
    assert_eq!(page.records.len(), 2);

    let mut filters = std::collections::BTreeMap::new();
    filters.insert("kind".to_string(), str_value("task"));
    let filtered = reopened
        .list(
            "agent/main",
            MemoryListOptions {
                #[allow(deprecated)]
                filters,
                filter_ops: None,
                limit: 10,
                cursor: None,
                exclude_superseded: false,
            },
        )
        .expect("filtered after repair");
    assert_eq!(filtered.records.len(), 1);
    assert_eq!(filtered.records[0].key, "a");
}
