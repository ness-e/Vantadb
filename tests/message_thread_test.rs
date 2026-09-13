// ponytail: blanket allow — unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Integration tests for agentic message threads.
//!
//! Covers: create, send, list, delete, and TTL-based expiry.

use std::collections::HashMap;
use std::sync::Arc;
use tempfile::tempdir;
use vantadb::agentic::{CreateThread, ManualClock, ThreadStore};
use vantadb::config::Config;
use vantadb::sdk::Embedded;
use vantadb::storage::{BackendKind, StorageEngine};

fn setup_engine() -> (StorageEngine, tempfile::TempDir) {
    let dir = tempdir().expect("tempdir");
    let config = Config {
        backend_kind: BackendKind::InMemory,
        ..Default::default()
    };
    let engine = StorageEngine::open_with_config(dir.path().to_str().unwrap(), Some(config))
        .expect("Failed to open StorageEngine");
    (engine, dir)
}

fn setup_embedded() -> (Embedded, tempfile::TempDir) {
    let dir = tempdir().expect("tempdir");
    let config = Config {
        backend_kind: BackendKind::InMemory,
        ..Default::default()
    };
    let db = Embedded::open_with_config(config.clone()).expect("open");
    (db, dir)
}

// ── test_create_and_send ──

#[test]
fn test_create_and_send() {
    let (engine, _dir) = setup_engine();
    let store = ThreadStore::new(&engine);

    let thread_id = store
        .create(
            CreateThread {
                title: "test thread",
                metadata: HashMap::new(),
                ttl_secs: None,
            },
            None,
        )
        .expect("create");

    store
        .send_message(thread_id, "user", "Hello!", HashMap::new(), None)
        .expect("send_message");
    store
        .send_message(thread_id, "assistant", "Hi there!", HashMap::new(), None)
        .expect("send_message");

    let thread = store
        .get(thread_id)
        .expect("get")
        .expect("thread should exist");

    assert_eq!(thread.title, "test thread");
    assert_eq!(thread.messages.len(), 2);
    assert_eq!(thread.messages[0].role, "user");
    assert_eq!(thread.messages[0].content, "Hello!");
    assert_eq!(thread.messages[1].role, "assistant");
    assert_eq!(thread.messages[1].content, "Hi there!");
    assert!(thread.created_at > 0);
    assert!(thread.updated_at >= thread.created_at);
}

// ── test_list ──

#[test]
fn test_list() {
    let (engine, _dir) = setup_engine();
    let store = ThreadStore::new(&engine);

    let _id1 = store
        .create(
            CreateThread {
                title: "Thread A",
                metadata: HashMap::new(),
                ttl_secs: None,
            },
            None,
        )
        .expect("create A");
    let _id2 = store
        .create(
            CreateThread {
                title: "Thread B",
                metadata: HashMap::new(),
                ttl_secs: None,
            },
            None,
        )
        .expect("create B");
    let _id3 = store
        .create(
            CreateThread {
                title: "Thread C",
                metadata: HashMap::new(),
                ttl_secs: None,
            },
            None,
        )
        .expect("create C");

    // All threads
    let all = store.list(10, 0).expect("list");
    assert_eq!(all.len(), 3);

    // Pagination: limit=2
    let page = store.list(2, 0).expect("list page");
    assert_eq!(page.len(), 2);

    // Pagination: offset=2
    let rest = store.list(10, 2).expect("list rest");
    assert_eq!(rest.len(), 1);

    // Offset beyond total
    let empty = store.list(10, 10).expect("list empty");
    assert!(empty.is_empty());
}

// ── test_delete ──

#[test]
fn test_delete() {
    let (engine, _dir) = setup_engine();
    let store = ThreadStore::new(&engine);

    let thread_id = store
        .create(
            CreateThread {
                title: "to-delete",
                metadata: HashMap::new(),
                ttl_secs: None,
            },
            None,
        )
        .expect("create");

    // Exists before delete
    assert!(store.get(thread_id).unwrap().is_some());

    // Delete
    store.delete(thread_id).expect("delete");

    // Gone after delete
    assert!(store.get(thread_id).unwrap().is_none());

    // Not in list
    let all = store.list(10, 0).unwrap();
    assert!(all.iter().all(|t| t.thread_id != thread_id));
}

// ── test_thread_ttl_expiry ──

#[test]
fn test_thread_ttl_expiry() {
    // FIRST-Repeatable (C2T2): virtual time via ManualClock — no sleep, no
    // scheduling flake. TTL semantics preserved: a real ttl_secs is set and
    // expiry is proven in both states (not-yet-expired, then expired).
    // GcWorker::sweep is intentionally not used: it reads wall time.
    let (engine, _dir) = setup_engine();
    let clock = Arc::new(ManualClock::new(1_000_000, 1_000_000_000));
    let store = ThreadStore::with_clock(&engine, clock.clone());

    let ttl_secs = 60u64;
    let thread_id = store
        .create(
            CreateThread {
                title: "ephemeral",
                metadata: HashMap::new(),
                ttl_secs: Some(ttl_secs),
            },
            None,
        )
        .expect("create with TTL");

    // Thread exists right away
    assert!(store.get(thread_id).unwrap().is_some());

    // Not yet expired → purge removes nothing
    let swept = store.purge_expired_threads().expect("purge before expiry");
    assert_eq!(swept, 0, "nothing must expire before its TTL elapses");
    assert!(store.get(thread_id).unwrap().is_some());

    // Travel past the TTL → purge removes exactly the expired thread
    clock.advance_secs(ttl_secs + 1);
    let swept = store.purge_expired_threads().expect("purge after expiry");
    assert_eq!(swept, 1, "purge should have removed 1 expired thread");

    // Thread should be gone
    assert!(store.get(thread_id).unwrap().is_none());
}

// ── test_create_via_embedded ──

#[test]
fn test_create_via_embedded() {
    let (db, _dir) = setup_embedded();

    let thread_id = db.create_thread("embedded test", None).expect("create");
    assert!(thread_id > 0);

    let thread = db
        .get_thread(thread_id)
        .expect("get")
        .expect("should exist");
    assert_eq!(thread.title, "embedded test");
    assert_eq!(thread.messages.len(), 0);
}

// ── test_send_and_list_via_embedded ──

#[test]
fn test_send_and_list_via_embedded() {
    let (db, _dir) = setup_embedded();

    let id = db.create_thread("chat", None).expect("create");
    db.send_message(id, "user", "msg1").expect("send");
    db.send_message(id, "user", "msg2").expect("send");

    let thread = db.get_thread(id).expect("get").expect("exists");
    assert_eq!(thread.messages.len(), 2);

    let list = db.list_threads(10, 0).expect("list");
    assert_eq!(list.len(), 1);

    db.delete_thread(id).expect("delete");
    assert!(db.get_thread(id).unwrap().is_none());
}
