// ponytail: blanket allow — unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]

//! SCH-06 / ADR-0046 §D3 — time-travel integration suite (`AS OF`, valid-time).
//!
//! Reference dates: no sleeps and no `now()` in assertions. The suite anchors
//! every boundary on timestamps the records themselves carry — synthetic
//! `valid_at_ms` values written through the public `MemoryInput` boundary and
//! the `superseded_at_ms`/`invalid_at_ms` the engine stamps on `supersede()`.
//! That keeps the assertions exact at machine-clock resolution (the ADR
//! predicate `valid_at <= T < invalid_at` is inclusive start / exclusive end).
//!
//! Axes (ADR §D3-4): `AS OF` narrows the **valid** axis; the **transaction**
//! axis of 0.8.0 is per-key (`versions()`/`get_version`) — the suite asserts
//! both, and that they disagree by design (a retroactive validity claim is
//! visible `AS OF` its valid time while its transaction history starts now).

use tempfile::tempdir;
use vantadb::{
    Embedded, MemoryInput, MemoryListOptions, MemorySearchRequest, QueryResult, ValidWindow,
};

const NS: &str = "tt";

fn open(dir: &std::path::Path) -> Embedded {
    Embedded::open(dir).expect("open db")
}

fn search_keys(db: &Embedded, as_of_ms: Option<u64>) -> Vec<String> {
    let mut keys: Vec<String> = db
        .search(MemorySearchRequest {
            namespace: NS.to_string(),
            text_query: Some("temporal".to_string()),
            top_k: 10,
            as_of_ms,
            ..Default::default()
        })
        .expect("search")
        .into_iter()
        .map(|hit| hit.record.key)
        .collect();
    keys.sort();
    keys
}

fn search_window_keys(db: &Embedded, from_ms: u64, to_ms: u64) -> Vec<String> {
    let mut keys: Vec<String> = db
        .search(MemorySearchRequest {
            namespace: NS.to_string(),
            text_query: Some("temporal".to_string()),
            top_k: 10,
            valid_window: Some(ValidWindow { from_ms, to_ms }),
            ..Default::default()
        })
        .expect("search")
        .into_iter()
        .map(|hit| hit.record.key)
        .collect();
    keys.sort();
    keys
}

fn list_keys(db: &Embedded, options: MemoryListOptions) -> Vec<String> {
    let mut keys: Vec<String> = db
        .list(NS, options)
        .expect("list")
        .records
        .into_iter()
        .map(|record| record.key)
        .collect();
    keys.sort();
    keys
}

/// `old` [1000, end) superseded by `new`; returns (`t_end`, `t_new`).
fn seeded_pair(db: &Embedded) -> (u64, u64) {
    db.put(MemoryInput {
        valid_at_ms: Some(1000),
        ..MemoryInput::new(NS, "old", "temporal old")
    })
    .expect("put old");
    db.put(MemoryInput::new(NS, "new", "temporal new"))
        .expect("put new");
    db.supersede(NS, "old", "new").expect("supersede");
    let old = db.get(NS, "old").expect("get old").expect("old present");
    let new = db.get(NS, "new").expect("get new").expect("new present");
    let end = old.invalid_at_ms.expect("supersede sets invalid_at");
    assert_eq!(
        old.superseded_at_ms,
        Some(end),
        "D3-3: invalid_at stays aligned with superseded_at"
    );
    (end, new.created_at_ms)
}

// ─── C2: predicado D3 con fechas de referencia (search + list) ────────────

#[test]
fn as_of_point_predicate_is_inclusive_start_and_exclusive_end() {
    let dir = tempdir().expect("tempdir");
    let db = open(dir.path());
    let (end, _t_new) = seeded_pair(&db);

    // Start inclusive: T before 1000 sees nothing; T == 1000 sees `old`.
    assert!(search_keys(&db, Some(999)).is_empty());
    assert_eq!(search_keys(&db, Some(1000)), vec!["old"]);
    assert_eq!(
        list_keys(
            &db,
            MemoryListOptions {
                as_of_ms: Some(1000),
                ..Default::default()
            }
        ),
        vec!["old"]
    );

    // End exclusive: `old` is still valid one ms before its window closes and
    // NOT at the boundary itself (`new` may already be valid there too).
    let before_end = search_keys(&db, Some(end - 1));
    assert!(
        before_end.contains(&"old".to_string()),
        "old must be valid at end-1: {before_end:?}"
    );
    assert!(
        !search_keys(&db, Some(end)).contains(&"old".to_string()),
        "invalid_at == T must exclude (end exclusive)"
    );

    // The successor owns the instant where `old` closes (its window is open).
    assert_eq!(search_keys(&db, Some(end)), vec!["new"]);
    assert_eq!(
        list_keys(
            &db,
            MemoryListOptions {
                as_of_ms: Some(end),
                ..Default::default()
            }
        ),
        vec!["new"]
    );

    // Default (no AS OF) is unchanged: both records remain visible.
    assert_eq!(search_keys(&db, None), vec!["new", "old"]);
}

#[test]
fn valid_window_uses_half_open_overlap_on_search_and_list() {
    let dir = tempdir().expect("tempdir");
    let db = open(dir.path());
    let (end, _t_new) = seeded_pair(&db);

    // Query window ending exactly at `old`'s start does not touch it.
    assert!(search_window_keys(&db, 0, 1000).is_empty());
    // One ms later it overlaps.
    assert_eq!(search_window_keys(&db, 0, 1001), vec!["old"]);
    // Window ending exactly at `old`'s end does not touch `old`...
    assert_eq!(search_window_keys(&db, end, end + 1), vec!["new"]);
    // 0..end crosses both windows.
    assert_eq!(search_window_keys(&db, 0, end + 1), vec!["new", "old"]);
    // List mirrors the search semantics.
    assert_eq!(
        list_keys(
            &db,
            MemoryListOptions {
                valid_window: Some(ValidWindow {
                    from_ms: 0,
                    to_ms: 1001
                }),
                ..Default::default()
            }
        ),
        vec!["old"]
    );

    // Empty/inverted windows are rejected at the boundary (never clamped).
    let err = db
        .search(MemorySearchRequest {
            namespace: NS.to_string(),
            text_query: Some("temporal".to_string()),
            top_k: 10,
            valid_window: Some(ValidWindow {
                from_ms: 10,
                to_ms: 10,
            }),
            ..Default::default()
        })
        .expect_err("empty window must be rejected");
    assert!(err.to_string().contains("valid_window"));

    let err = db
        .list(
            NS,
            MemoryListOptions {
                valid_window: Some(ValidWindow {
                    from_ms: 20,
                    to_ms: 10,
                }),
                ..Default::default()
            },
        )
        .expect_err("inverted window must be rejected");
    assert!(err.to_string().contains("valid_window"));
}

// ─── C2: persiste tras reopen + IQL `AS OF` ≡ SDK + eje transaction ───────

#[test]
fn time_travel_survives_reopen_and_matches_iql_as_of() {
    let dir = tempdir().expect("tempdir");
    {
        let db = open(dir.path());
        // Retroactive validity claim: written now, valid since 500.
        db.put(MemoryInput {
            valid_at_ms: Some(500),
            ..MemoryInput::new(NS, "retro", "temporal retro")
        })
        .expect("put retro");
        db.put(MemoryInput::new(NS, "fresh", "temporal fresh"))
            .expect("put fresh");
        db.flush().expect("flush");
        db.close().expect("close");
    }

    let db = open(dir.path());

    // Valid axis after reopen.
    assert!(search_keys(&db, Some(499)).is_empty());
    assert_eq!(search_keys(&db, Some(500)), vec!["retro"]);

    // IQL `AS OF` agrees with the SDK path (same predicate, same axis).
    let iql_keys = |db: &Embedded, iql: &str| -> Vec<String> {
        let mut keys: Vec<String> = match db.query(iql).expect("query") {
            QueryResult::Read(nodes) => nodes
                .into_iter()
                .filter_map(|node| match node.fields.get("__vanta_key") {
                    Some(vantadb::Value::String(key)) => Some(key.clone()),
                    _ => None,
                })
                .collect(),
            other => panic!("expected Read, got {other:?}"),
        };
        keys.sort();
        keys
    };
    assert_eq!(iql_keys(&db, "SELECT * FROM tt AS OF 500"), vec!["retro"]);
    assert!(iql_keys(&db, "SELECT * FROM tt AS OF 499").is_empty());

    // Transaction axis (ADR §D3-4): per-key history records WHEN the system
    // learned the state — which is `now`, not the claimed valid time.
    let versions = db.versions(NS, "retro").expect("versions");
    assert!(!versions.is_empty(), "a put persists its v1 snapshot");
    let first = &versions[0];
    assert_eq!(first.valid_at_ms, 500, "valid axis travels in the snapshot");
    assert!(
        first.created_at_ms > 500,
        "transaction axis is the real write time ({}), independent of valid_at (500)",
        first.created_at_ms
    );
    let fetched = db
        .get_version(NS, "retro", first.version)
        .expect("get_version")
        .expect("version present");
    assert_eq!(fetched.valid_at_ms, 500);
    assert_eq!(fetched.payload, "temporal retro");
}

// ─── C2 (extensión D3-6): exclude_superseded cubre ventanas cerradas ──────

#[test]
fn exclude_superseded_drops_ended_validity_windows_after_reopen() {
    let dir = tempdir().expect("tempdir");
    {
        let db = open(dir.path());
        let _ = seeded_pair(&db);
        db.flush().expect("flush");
        db.close().expect("close");
    }

    let db = open(dir.path());
    let hidden = |db: &Embedded| -> Vec<String> {
        let mut keys: Vec<String> = db
            .search(MemorySearchRequest {
                namespace: NS.to_string(),
                text_query: Some("temporal".to_string()),
                top_k: 10,
                exclude_superseded: true,
                ..Default::default()
            })
            .expect("search")
            .into_iter()
            .map(|hit| hit.record.key)
            .collect();
        keys.sort();
        keys
    };
    assert_eq!(hidden(&db), vec!["new"], "ended window dropped");
    assert_eq!(
        list_keys(
            &db,
            MemoryListOptions {
                exclude_superseded: true,
                ..Default::default()
            }
        ),
        vec!["new"]
    );
    // Default keeps every record (zero breaking change).
    assert_eq!(search_keys(&db, None), vec!["new", "old"]);
}
