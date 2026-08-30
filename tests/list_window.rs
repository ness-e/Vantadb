//! Cursor-based pagination contract for `VantaEmbedded::list` (FIND-24).
//!
//! Regression coverage of the perf fix that turned `list(limit=100)` over a
//! 10k-record namespace from O(10k) into O(100): the prefix-scan iterator now
//! early-exits at `limit`, with `cursor` consumed during the scan instead of
//! materializing the full candidate set and slicing in memory.
//!
//! Contract:
//!  - All keys appear exactly once across a full paginated walk.
//!  - Pagination over a namespace holding >limit records works without OOM
//!    and without skipping or duplicating records.
//!  - Cursor pages do not regress in count when filters narrow the set.

use vantadb::config::VantaConfig;
use vantadb::{
    VantaEmbedded, VantaFilterOp, VantaMemoryFilterItem, VantaMemoryInput, VantaMemoryListOptions,
    VantaValue,
};

fn open_in_memory_db() -> VantaEmbedded {
    let dir = tempfile::tempdir().expect("tempdir");
    let config = VantaConfig {
        storage_path: dir.path().to_string_lossy().into_owned(),
        ..Default::default()
    };
    VantaEmbedded::open_with_config(config).expect("open in-memory db")
}

#[test]
fn list_window_cursor_walks_all_records_without_duplicates() {
    let db = open_in_memory_db();
    let ns = "window-ns";

    // Insert 250 records (>4 pages of limit=100 so pagination must engage).
    let total: u32 = 250;
    for i in 0..total {
        db.put(VantaMemoryInput::new(
            ns,
            format!("k-{i:04}"),
            format!("payload {i}"),
        ))
        .expect("put");
    }

    let mut collected: Vec<String> = Vec::new();
    let mut cursor: Option<usize> = None;
    let page_size: usize = 100;
    let mut pages = 0usize;

    loop {
        let page = db
            .list(
                ns,
                VantaMemoryListOptions {
                    limit: page_size,
                    cursor,
                    ..Default::default()
                },
            )
            .expect("list");

        for r in &page.records {
            assert!(
                !collected.contains(&r.key),
                "duplicate key across pages: {} (page {})",
                r.key,
                pages
            );
            collected.push(r.key.clone());
        }
        pages += 1;
        assert!(
            pages <= 8,
            "pagination did not terminate within 8 pages (collected {})",
            collected.len()
        );

        match page.next_cursor {
            Some(c) => cursor = Some(c),
            None => break,
        }
    }

    assert_eq!(
        collected.len(),
        total as usize,
        "all records must be visited across paginated windows"
    );
}

#[test]
fn list_window_cursor_with_filter_is_consistent() {
    let db = open_in_memory_db();
    let ns = "window-ns-filter";

    for i in 0..150u32 {
        let tag = if i % 2 == 0 { "even" } else { "odd" };
        let mut input = VantaMemoryInput::new(ns, format!("k-{i:04}"), format!("payload {i}"));
        input
            .metadata
            .insert("tag".to_string(), VantaValue::String(tag.to_string()));
        db.put(input).expect("put");
    }

    // Cursor + filter: walking the filtered namespace must yield only the
    // even-tagged records and not duplicate or drop any.
    let mut seen_even: Vec<String> = Vec::new();
    let mut cursor: Option<usize> = None;
    loop {
        let page = db
            .list(
                ns,
                VantaMemoryListOptions {
                    limit: 30,
                    cursor,
                    filter_ops: Some(vec![VantaMemoryFilterItem {
                        field: "tag".to_string(),
                        op: VantaFilterOp::Eq,
                        value: VantaValue::String("even".to_string()),
                    }]),
                    ..Default::default()
                },
            )
            .expect("list");
        for r in &page.records {
            let is_even = r
                .metadata
                .get("tag")
                .map(|v| matches!(v, VantaValue::String(s) if s == "even"))
                .unwrap_or(false);
            assert!(is_even, "filter must exclude odd records (got {})", r.key);
            seen_even.push(r.key.clone());
        }
        match page.next_cursor {
            Some(c) => cursor = Some(c),
            None => break,
        }
    }

    assert_eq!(
        seen_even.len(),
        75,
        "expected 75 even-tagged records (half of 150)"
    );
}
