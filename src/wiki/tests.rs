//! Wiki store tests (D19, MEM-28). Pattern AAA, in-memory engine — same
//! fixture as `src/entity/tests.rs`.

use super::{canonical_path, WikiState, WikiStore};
use crate::config::VantaConfig;
use crate::error::VantaError;
use crate::storage::{BackendKind, StorageEngine};

fn in_memory_engine() -> StorageEngine {
    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        read_only: false,
        ..VantaConfig::default()
    };
    StorageEngine::open_with_config(":memory:", Some(config)).expect("open in-memory engine")
}

const NS: &str = "default";
const SLUG: &str = "team-wiki";

// ── (a) create → pending ──

#[test]
fn create_starts_pending() {
    let engine = in_memory_engine();
    let store = WikiStore::new(&engine);

    let wiki = store.create(NS, SLUG).expect("create");

    assert_eq!(wiki.state, WikiState::Pending);
    assert!(wiki.run_id.is_none());
    assert!(wiki.sync_error.is_none());
    assert_eq!(wiki.version, 1);

    let got = store.get(NS, SLUG).expect("get").expect("exists");
    assert_eq!(got.state, WikiState::Pending);
}

#[test]
fn create_duplicate_conflicts() {
    let engine = in_memory_engine();
    let store = WikiStore::new(&engine);
    store.create(NS, SLUG).expect("create");

    let err = store.create(NS, SLUG).unwrap_err();
    assert!(matches!(err, VantaError::ExecutionConflict { .. }));
}

// ── (b) ingest while busy → 409-equivalent ──

#[test]
fn ingest_rejected_while_pending_and_processing() {
    let engine = in_memory_engine();
    let store = WikiStore::new(&engine);
    store.create(NS, SLUG).expect("create"); // pending

    let err = store.request_ingest(NS, SLUG).unwrap_err();
    assert!(
        matches!(err, VantaError::ExecutionConflict { .. }),
        "expected conflict while pending, got {err:?}"
    );

    let processing = store.begin_processing(NS, SLUG).expect("begin");
    let err = store.request_ingest(NS, SLUG).unwrap_err();
    assert!(
        matches!(err, VantaError::ExecutionConflict { .. }),
        "expected conflict while processing, got {err:?}"
    );
    // begin_processing from non-pending also conflicts
    let err = store.begin_processing(NS, SLUG).unwrap_err();
    assert!(matches!(err, VantaError::ExecutionConflict { .. }));
    assert_eq!(processing.state, WikiState::Processing);
}

// ── (c) full transition pending→processing→ready with run_id ──

#[test]
fn full_transition_to_ready_with_run_id() {
    let engine = in_memory_engine();
    let store = WikiStore::new(&engine);
    store.create(NS, SLUG).expect("create");

    let processing = store.begin_processing(NS, SLUG).expect("begin");
    assert_eq!(processing.state, WikiState::Processing);
    let run_id = processing.run_id.as_deref().expect("run_id assigned");
    assert!(!run_id.is_empty());

    let ready = store.complete(NS, SLUG, run_id).expect("complete");
    assert_eq!(ready.state, WikiState::Ready);
    assert!(ready.sync_error.is_none());

    let got = store.get(NS, SLUG).expect("get").expect("exists");
    assert_eq!(got.state, WikiState::Ready);
    assert_eq!(got.version, 3); // create + begin + complete
}

#[test]
fn stale_run_id_completion_rejected() {
    let engine = in_memory_engine();
    let store = WikiStore::new(&engine);
    store.create(NS, SLUG).expect("create");
    let first = store.begin_processing(NS, SLUG).expect("begin");
    let old_run = first.run_id.clone().expect("run_id");
    // simulate a rebuild cycle
    store.complete(NS, SLUG, &old_run).expect("complete");
    store.request_ingest(NS, SLUG).expect("re-request");
    let second = store.begin_processing(NS, SLUG).expect("begin again");
    let new_run = second.run_id.clone().expect("run_id");
    assert_ne!(old_run, new_run);

    let err = store.complete(NS, SLUG, &old_run).unwrap_err(); // late packet
    assert!(matches!(err, VantaError::ExecutionConflict { .. }));

    store
        .complete(NS, SLUG, &new_run)
        .expect("current run completes");
}

// ── (d) fail → failed with sync_error truncated 500 ──

#[test]
fn failure_truncates_sync_error_to_500() {
    let engine = in_memory_engine();
    let store = WikiStore::new(&engine);
    store.create(NS, SLUG).expect("create");
    let processing = store.begin_processing(NS, SLUG).expect("begin");
    let run_id = processing.run_id.expect("run_id");

    let huge = "x".repeat(2000);
    let failed = store.fail(NS, SLUG, &run_id, &huge).expect("fail");
    assert_eq!(failed.state, WikiState::Failed);
    let err_text = failed.sync_error.expect("sync_error stored");
    assert_eq!(err_text.chars().count(), 500);

    // re-ingest allowed after failure
    let wiki = store.request_ingest(NS, SLUG).expect("re-request");
    assert_eq!(wiki.state, WikiState::Pending);
    assert!(wiki.sync_error.is_none());
}

// ── (e) dedup path canónico type+title ──

#[test]
fn canonical_path_dedup_by_type_and_title() {
    let a1 = canonical_path("person", "Alice Smith");
    let a2 = canonical_path("person", "Alice Smith");
    assert_eq!(a1, a2, "same type+title must dedup to one path");
    assert_eq!(a1, "wiki/person/alice-smith.md");

    let b = canonical_path("concept", "Alice Smith");
    assert_ne!(a1, b, "different type must not collide");

    let c = canonical_path("person", "Alice  Smith--weird!");
    assert_eq!(
        c, "wiki/person/alice-smith-weird.md",
        "slugified + collapsed"
    );
}

#[test]
fn put_page_same_type_title_overwrites() {
    let engine = in_memory_engine();
    let store = WikiStore::new(&engine);
    store.create(NS, SLUG).expect("create");

    store
        .put_page(NS, SLUG, "person", "Alice Smith", "v1")
        .expect("put");
    store
        .put_page(NS, SLUG, "person", "Alice Smith", "v2")
        .expect("put");

    let pages = store.list_pages(NS, SLUG).expect("list");
    assert_eq!(pages.len(), 1, "dedup: one page for same type+title");
    assert_eq!(pages[0].content, "v2");
}

// ── (f) locked:true en páginas gestionadas + cascade delete ──

#[test]
fn managed_pages_locked_true_and_cascade_delete() {
    let engine = in_memory_engine();
    let store = WikiStore::new(&engine);
    store.create(NS, SLUG).expect("create");

    let p1 = store
        .put_page(NS, SLUG, "person", "Alice", "bio")
        .expect("put p1");
    assert!(p1.locked, "managed pages must be locked:true");

    store
        .put_page(NS, SLUG, "concept", "RRF Fusion", "how-to")
        .expect("put p2");
    assert_eq!(store.list_pages(NS, SLUG).expect("list").len(), 2);

    assert!(store.delete(NS, SLUG).expect("delete"), "wiki existed");
    assert!(store.get(NS, SLUG).expect("get").is_none(), "record gone");
    assert!(
        store.list_pages(NS, SLUG).expect("list").is_empty(),
        "cascade removed all pages"
    );

    // deleting again is a no-op false
    assert!(!store.delete(NS, SLUG).expect("second delete"));
}

#[test]
fn sanitization_rejects_bad_keys() {
    let engine = in_memory_engine();
    let store = WikiStore::new(&engine);

    assert!(store.create("", SLUG).is_err(), "empty namespace rejected");
    assert!(store.create(NS, "").is_err(), "empty slug rejected");
    assert!(store.create(NS, "bad:slug").is_err(), "':' rejected");
    let long = "a".repeat(513);
    assert!(store.create(NS, &long).is_err(), ">512 bytes rejected");

    store.create(NS, SLUG).expect("create");
    assert!(
        store.put_page(NS, SLUG, "type", "ti\0tle", "x").is_err(),
        "NUL in title rejected"
    );
}

#[test]
fn missing_wiki_operations_not_found() {
    let engine = in_memory_engine();
    let store = WikiStore::new(&engine);

    assert!(matches!(
        store.begin_processing(NS, "ghost"),
        Err(VantaError::NotFound { kind, .. }) if kind == "wiki"
    ));
    assert!(store.get(NS, "ghost").expect("get").is_none());
}
