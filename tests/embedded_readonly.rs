// EGO-09: `Embedded::open_readonly` — shared-lock reader semantics.
// ponytail: blanket allow — unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
use vantadb::error::Error;
use vantadb::{Embedded, MemoryInput};

fn write_one(dir: &std::path::Path) {
    let db = Embedded::open(dir).expect("writer opens");
    db.put(MemoryInput::new("ns", "k", "v"))
        .expect("seed write");
    db.close().expect("close writer");
}

#[test]
fn open_readonly_missing_path_is_not_found() {
    let dir = tempfile::tempdir().expect("tempdir");
    let missing = dir.path().join("never-initialized");
    let res = Embedded::open_readonly(&missing);
    assert!(
        matches!(res, Err(Error::NotFound { .. })),
        "read-only open must not create schema, got: {:?}",
        res.map(|_| ()),
    );
}

#[test]
fn open_readonly_sees_data_and_rejects_writes() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_one(dir.path());

    let ro = Embedded::open_readonly(dir.path()).expect("read-only opens");
    let got = ro
        .get("ns", "k")
        .expect("read works")
        .expect("seed visible");
    assert_eq!(got.payload, "v");
    let w = ro.put(MemoryInput::new("ns", "k2", "v2"));
    assert!(w.is_err(), "writes on a read-only handle must be rejected");
    ro.close().expect("close");
}

#[test]
fn open_readonly_while_writer_holds_lock_is_busy() {
    // Shared OS lock vs exclusive writer lock: a live writer blocks the
    // reader. Shared RW on one directory needs the daemon/socket pattern
    // (see `Embedded::open_readonly` docs).
    let dir = tempfile::tempdir().expect("tempdir");
    let _writer = Embedded::open(dir.path()).expect("writer holds exclusive lock");
    let res = Embedded::open_readonly(dir.path());
    assert!(
        matches!(res, Err(Error::DatabaseBusy(_))),
        "reader must fail while writer holds the lock, got: {:?}",
        res.map(|_| ()),
    );
}
