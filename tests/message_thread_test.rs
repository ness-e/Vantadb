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

// ── TTL: el mecanismo es el purge explícito (FIND-800) ──
//
// FIND-800: el módulo prometía «auto-expiry» y ningún camino de producción ejecutaba el TTL
// (`Embedded` pasa `gc = None` y el `GcWorker` sólo se instancia en tests). La decisión fue no
// encender el GC (su índice vive en memoria, no sobrevive a un reinicio y su barrido deja ids
// colgantes en `_thread_ids`) y declarar el mecanismo que sí existe: `purge_expired_threads`.
// Estos dos tests son la forma ejecutable de ese contrato — no hay RED posible porque no se
// añade lógica, sólo se fija por escrito lo que el código ya hacía.

/// El TTL **no se evalúa en lectura**. Un hilo vencido sigue siendo legible hasta que corre el
/// purge explícito; el borde es estricto (`_expires_at < now`, `thread.rs:486-489`), así que en el
/// segundo exacto del deadline el hilo todavía vive.
#[test]
fn expired_thread_is_readable_until_the_explicit_purge_runs() {
    let (engine, _dir) = setup_engine();
    let clock = Arc::new(ManualClock::new(1_000_000, 1_000_000_000));
    let store = ThreadStore::with_clock(&engine, clock.clone());

    let ttl_secs = 60u64;
    let thread_id = store
        .create(
            CreateThread {
                title: "about to expire",
                metadata: HashMap::new(),
                ttl_secs: Some(ttl_secs),
            },
            None,
        )
        .expect("create with TTL");

    // En el segundo exacto del deadline todavía no está vencido (`<`, no `<=`).
    clock.advance_secs(ttl_secs);
    assert_eq!(
        store
            .purge_expired_threads()
            .expect("purge at the boundary"),
        0,
        "at exactly `_expires_at` the thread is not expired yet"
    );

    // Un segundo después está vencido, pero **nadie evalúa el TTL en lectura**: sigue visible
    // por id y en el listado. Esto es lo que el doc ahora declara.
    clock.advance_secs(1);
    assert!(
        store.get(thread_id).expect("get").is_some(),
        "an expired thread stays readable until the explicit purge runs"
    );
    assert_eq!(
        store.list(10, 0).expect("list").len(),
        1,
        "and it stays listed, too"
    );

    // El purge explícito es el mecanismo: borra el nodo y reconstruye el índice.
    assert_eq!(
        store.purge_expired_threads().expect("purge"),
        1,
        "the purge is what makes the TTL real"
    );
    assert!(store.get(thread_id).expect("get").is_none());
    assert!(store.list(10, 0).expect("list").is_empty());
}

/// El TTL es una ventana de **inactividad**: cada `send_message` re-arma el deadline con la
/// duración original (`thread.rs:372-382`). Un hilo con conversación activa nunca caduca; uno
/// sin actividad sí. Está declarado en el doc de `send_message` — este test lo fija.
#[test]
fn send_message_re_arms_the_ttl_so_an_active_thread_never_expires() {
    let (engine, _dir) = setup_engine();
    let clock = Arc::new(ManualClock::new(1_000_000, 1_000_000_000));
    let store = ThreadStore::with_clock(&engine, clock.clone());

    let ttl_secs = 60u64;
    let thread_id = store
        .create(
            CreateThread {
                title: "chatty",
                metadata: HashMap::new(),
                ttl_secs: Some(ttl_secs),
            },
            None,
        )
        .expect("create with TTL");

    // Actividad dentro de la ventana: re-arma el deadline.
    clock.advance_secs(50);
    store
        .send_message(thread_id, "user", "still here", HashMap::new(), None)
        .expect("send_message");

    // 100 s después de crearlo — 40 s después del deadline *original* — y sigue vivo, porque el
    // mensaje de los 50 s movió el deadline a los 110 s.
    clock.advance_secs(50);
    assert_eq!(
        store.purge_expired_threads().expect("purge while active"),
        0,
        "a message re-arms the TTL: this is an inactivity window, not an absolute deadline"
    );
    assert!(store.get(thread_id).expect("get").is_some());

    // Sin actividad, la ventana se cierra: 61 s después del último mensaje, caduca.
    clock.advance_secs(ttl_secs + 1);
    assert_eq!(
        store.purge_expired_threads().expect("purge after silence"),
        1,
        "with no further activity the inactivity window closes"
    );
    assert!(store.get(thread_id).expect("get").is_none());
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
