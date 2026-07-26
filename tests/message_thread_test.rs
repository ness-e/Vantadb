//! Integration tests for agentic message threads.
//!
//! Covers: create, send, list, delete, and TTL-based expiry.

use std::collections::HashMap;
use tempfile::tempdir;
use vantadb::agentic::ThreadStore;
use vantadb::config::VantaConfig;
use vantadb::gc::GcWorker;
use vantadb::sdk::VantaEmbedded;
use vantadb::storage::{BackendKind, StorageEngine};

fn setup_engine() -> (StorageEngine, tempfile::TempDir) {
    let dir = tempdir().expect("tempdir");
    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        ..Default::default()
    };
    let engine = StorageEngine::open_with_config(dir.path().to_str().unwrap(), Some(config))
        .expect("Failed to open StorageEngine");
    (engine, dir)
}

fn setup_embedded() -> (VantaEmbedded, tempfile::TempDir) {
    let dir = tempdir().expect("tempdir");
    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        ..Default::default()
    };
    let db = VantaEmbedded::open_with_config(config.clone()).expect("open");
    (db, dir)
}

// ── test_create_and_send ──

#[test]
fn test_create_and_send() {
    let (engine, _dir) = setup_engine();
    let store = ThreadStore::new(&engine);

    let thread_id = store
        .create_thread("test thread", HashMap::new(), None, None)
        .expect("create_thread");

    store
        .send_message(thread_id, "user", "Hello!", HashMap::new(), None)
        .expect("send_message");
    store
        .send_message(thread_id, "assistant", "Hi there!", HashMap::new(), None)
        .expect("send_message");

    let thread = store
        .get_thread(thread_id)
        .expect("get_thread")
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

// ── test_list_threads ──

#[test]
fn test_list_threads() {
    let (engine, _dir) = setup_engine();
    let store = ThreadStore::new(&engine);

    let _id1 = store
        .create_thread("Thread A", HashMap::new(), None, None)
        .expect("create_thread A");
    let _id2 = store
        .create_thread("Thread B", HashMap::new(), None, None)
        .expect("create_thread B");
    let _id3 = store
        .create_thread("Thread C", HashMap::new(), None, None)
        .expect("create_thread C");

    // All threads
    let all = store.list_threads(10, 0).expect("list_threads");
    assert_eq!(all.len(), 3);

    // Pagination: limit=2
    let page = store.list_threads(2, 0).expect("list_threads page");
    assert_eq!(page.len(), 2);

    // Pagination: offset=2
    let rest = store.list_threads(10, 2).expect("list_threads rest");
    assert_eq!(rest.len(), 1);

    // Offset beyond total
    let empty = store.list_threads(10, 10).expect("list_threads empty");
    assert!(empty.is_empty());
}

// ── test_delete_thread ──

#[test]
fn test_delete_thread() {
    let (engine, _dir) = setup_engine();
    let store = ThreadStore::new(&engine);

    let thread_id = store
        .create_thread("to-delete", HashMap::new(), None, None)
        .expect("create_thread");

    // Exists before delete
    assert!(store.get_thread(thread_id).unwrap().is_some());

    // Delete
    store.delete_thread(thread_id).expect("delete_thread");

    // Gone after delete
    assert!(store.get_thread(thread_id).unwrap().is_none());

    // Not in list
    let all = store.list_threads(10, 0).unwrap();
    assert!(all.iter().all(|t| t.thread_id != thread_id));
}

// ── test_thread_ttl_expiry ──

#[test]
fn test_thread_ttl_expiry() {
    let (engine, _dir) = setup_engine();
    let store = ThreadStore::new(&engine);
    let mut gc = GcWorker::new(&engine);

    let ttl_secs = 1u64;
    let thread_id = store
        .create_thread("ephemeral", HashMap::new(), Some(ttl_secs), Some(&mut gc))
        .expect("create_thread with TTL");

    // Thread exists right away
    assert!(store.get_thread(thread_id).unwrap().is_some());

    // Wait for TTL to expire
    std::thread::sleep(std::time::Duration::from_secs(ttl_secs + 1));

    // Run GcWorker sweep — this deletes the expired node from storage
    let swept = gc.sweep().expect("gc sweep");
    assert_eq!(swept, 1, "GcWorker should have swept 1 expired thread");

    // Thread should be gone
    assert!(store.get_thread(thread_id).unwrap().is_none());
}

// ── test_create_thread_via_embedded ──

#[test]
fn test_create_thread_via_embedded() {
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
