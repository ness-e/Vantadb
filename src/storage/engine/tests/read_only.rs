//! FIND-632 / ADR-0001: the **logical** `read_only` barrier of the Fjall
//! backend.
//!
//! Fjall 3.1.12 has no physical read-only open (fjall#313): every `open`
//! takes an exclusive `.lock` and persists the journal. VantaDB therefore
//! honours `read_only` on Fjall as a logical barrier — no engine data,
//! registry or key material is written — while a small, pinned set of Fjall
//! metadata files may still change on disk. That set is the allowlist in
//! [`super::read_only_files`]; these tests pin both sides of the contract:
//!
//! * `fjall_read_only_open_never_mutates_files` — the diff of a before/after
//!   filesystem snapshot of a read_only open must stay inside the allowlist;
//! * the open-time failure modes (`.lock` contention, non-writable
//!   filesystem) surface as typed, actionable errors;
//! * every mutating `StorageBackend` call on a `read_only` backend returns a
//!   typed error — never silently writes, never panics.

use super::*;
use crate::backend::{BackendKind, BackendPartition, BackendWriteOp, StorageBackend};
use crate::backends::fjall_backend::FjallBackend;
use crate::config::Config;
use crate::error::Error;
use crate::node::{FieldValue, UnifiedNode, VectorRepresentations};

/// A config selecting the Fjall backend rooted at `path`.
fn fjall_config(path: &str, read_only: bool) -> Config {
    Config {
        storage_path: path.to_string(),
        backend_kind: BackendKind::Fjall,
        read_only,
        ..Config::default()
    }
}

/// A node with a vector and a canary payload.
fn canary_node(id: u128, canary: &str) -> UnifiedNode {
    let mut node = UnifiedNode::new(id);
    node.vector = VectorRepresentations::Full(vec![0.5, 0.25, 0.125]);
    node.relational.insert(
        "ro_canary".to_string(),
        FieldValue::String(canary.to_string()),
    );
    node
}

/// FIND-632 / ADR-0001: a `read_only` open of a populated Fjall database
/// must not mutate any file outside the Fjall open-metadata allowlist
/// (journal persist, lock, keyspace version markers — see
/// `read_only_files::fjall_open_allowlist` for the cited causes).
///
/// Every mutated/created/removed path is classified: inside the allowlist
/// → logged Fjall metadata; outside → hard failure with the full diff.
#[test]
fn fjall_read_only_open_never_mutates_files() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().to_str().expect("path");

    {
        let engine = StorageEngine::open_with_config(path, Some(fjall_config(path, false)))
            .expect("open writer engine");
        engine
            .insert(&canary_node(71, "vantadb-find632-ro-fjall"))
            .expect("insert");
        engine.flush().expect("flush");
    }

    let before = super::read_only_files::snapshot_tree(dir.path());

    {
        let engine = StorageEngine::open_with_config(path, Some(fjall_config(path, true)))
            .expect("a read_only open of a Fjall database still opens (logical barrier)");
        let read = engine.get(71).expect("get").expect("the node is there");
        assert_eq!(
            read.relational.get("ro_canary"),
            Some(&FieldValue::String("vantadb-find632-ro-fjall".to_string())),
            "a read_only open still reads the data"
        );
    }

    let after = super::read_only_files::snapshot_tree(dir.path());
    let (created, mutated, removed) = super::read_only_files::diff_snapshots(&before, &after);

    let is_allowed = |p: &std::path::PathBuf| super::read_only_files::fjall_open_allowlist(p);
    let bad_created: Vec<_> = created.iter().filter(|p| !is_allowed(p)).collect();
    let bad_mutated: Vec<_> = mutated.iter().filter(|p| !is_allowed(p)).collect();
    let bad_removed: Vec<_> = removed.iter().filter(|p| !is_allowed(p)).collect();

    assert!(
        bad_created.is_empty() && bad_mutated.is_empty() && bad_removed.is_empty(),
        "a read_only open must not touch files outside the Fjall metadata \
         allowlist (ADR-0001). Unexpected created: {bad_created:?}; mutated: \
         {bad_mutated:?}; removed: {bad_removed:?}. \
         Allowed-but-changed Fjall metadata: created {created:?}; mutated \
         {mutated:?}; removed {removed:?}"
    );
}

/// Residual (ADR-0001): a `read_only` open of a database **without** a
/// persisted HNSW image must still open — `recover_state` rebuilds the index
/// from the vstore in memory — and must not write `data/vector_index.bin`
/// (or any file outside the Fjall open-metadata allowlist). `init_indexes`
/// selects a RAM-resident fresh index under `read_only` so nothing file-backed
/// can be persisted even by a future ungated caller.
#[test]
fn fjall_read_only_open_without_hnsw_image_does_not_write() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().to_str().expect("path");
    let image = dir.path().join("data").join("vector_index.bin");

    {
        let engine = StorageEngine::open_with_config(path, Some(fjall_config(path, false)))
            .expect("open writer engine");
        engine
            .insert(&canary_node(72, "vantadb-ro-no-hnsw-image"))
            .expect("insert");
        engine.flush().expect("flush persists the index image");
    }
    assert!(
        image.exists(),
        "the RW flush must have persisted data/vector_index.bin"
    );
    std::fs::remove_file(&image).expect("delete the HNSW image");

    let before = super::read_only_files::snapshot_tree(dir.path());

    {
        let engine = StorageEngine::open_with_config(path, Some(fjall_config(path, true)))
            .expect("a read_only open without an HNSW image must still open");
        assert!(engine.is_read_only());
        // The vstore rebuild repopulated the index in memory: the node is
        // indexed and readable again.
        assert_eq!(
            engine.index_coordinator().node_count(),
            1,
            "recover_state must rebuild the index from the vstore"
        );
        let read = engine.get(72).expect("get").expect("the node is there");
        assert_eq!(
            read.relational.get("ro_canary"),
            Some(&FieldValue::String("vantadb-ro-no-hnsw-image".to_string())),
            "a read_only open still reads the data"
        );
    }

    let after = super::read_only_files::snapshot_tree(dir.path());
    let (created, mutated, removed) = super::read_only_files::diff_snapshots(&before, &after);

    let is_allowed = |p: &std::path::PathBuf| super::read_only_files::fjall_open_allowlist(p);
    let bad_created: Vec<_> = created.iter().filter(|p| !is_allowed(p)).collect();
    let bad_mutated: Vec<_> = mutated.iter().filter(|p| !is_allowed(p)).collect();
    let bad_removed: Vec<_> = removed.iter().filter(|p| !is_allowed(p)).collect();

    assert!(
        bad_created.is_empty() && bad_mutated.is_empty() && bad_removed.is_empty(),
        "a read_only open without an HNSW image must not touch files outside \
         the Fjall metadata allowlist (ADR-0001). Unexpected created: \
         {bad_created:?}; mutated: {bad_mutated:?}; removed: {bad_removed:?}. \
         Allowed-but-changed Fjall metadata: created {created:?}; mutated \
         {mutated:?}; removed {removed:?}"
    );
    assert!(
        !image.exists(),
        "the read_only open must not recreate data/vector_index.bin"
    );
}

/// FIND-632: a second open of a Fjall directory whose `.lock` is already
/// held — even a `read_only` one — fails with `Error::DatabaseBusy`, because
/// Fjall takes an exclusive file lock on every open
/// (fjall-3.1.12 `src/locked_file.rs:55-77`).
///
/// Runs on Windows too: the two handles live in the same process.
#[test]
fn fjall_read_only_reports_database_busy_when_locked() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().to_str().expect("path");

    let writer = FjallBackend::open(path, &fjall_config(path, false))
        .expect("first handle takes the Fjall lock");

    let err = FjallBackend::open(path, &fjall_config(path, true))
        .err()
        .expect("a second open — read_only included — must not succeed while the lock is held");
    assert!(
        matches!(err, Error::DatabaseBusy(_)),
        "a held Fjall lock surfaces as DatabaseBusy, got: {err}"
    );
    assert!(
        err.to_string().contains(path),
        "the DatabaseBusy message must name the contended path: {err}"
    );

    drop(writer);
}

/// FIND-632: on a non-writable filesystem a `read_only` Fjall open cannot
/// even take the lock or persist the journal — it must fail with a typed
/// `Error::Io` that says so and points at the RocksDB backend for physical
/// read-only.
///
/// Unix-only (chmod 0555); skipped when running as root, which ignores
/// permission bits.
#[cfg(unix)]
#[test]
fn fjall_read_only_reports_typed_error_on_unwritable_dir() {
    use std::os::unix::fs::PermissionsExt;

    if unsafe { libc::geteuid() } == 0 {
        eprintln!("skipping: running as root, permission bits are not enforced");
        return;
    }

    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().to_str().expect("path");

    {
        let backend = FjallBackend::open(path, &fjall_config(path, false))
            .expect("writer creates the database");
        backend
            .put(BackendPartition::Default, b"k", b"v")
            .expect("seed a write");
        backend.flush().expect("flush");
    }

    // Chmod-ing only the root is not enough: Fjall reopens the existing
    // `lock` file and journal for writing, which a 0555 directory still
    // allows. Make the whole tree read-only to simulate a read-only mount,
    // and restore it on drop so the TempDir can be removed.
    fn chmod_tree(root: &std::path::Path, dir_mode: u32, file_mode: u32) -> std::io::Result<()> {
        for entry in std::fs::read_dir(root)? {
            let p = entry?.path();
            let mode = if p.is_dir() {
                chmod_tree(&p, dir_mode, file_mode)?;
                dir_mode
            } else {
                file_mode
            };
            std::fs::set_permissions(&p, std::fs::Permissions::from_mode(mode))?;
        }
        Ok(())
    }
    struct RestoreWritable<'a>(&'a std::path::Path);
    impl Drop for RestoreWritable<'_> {
        fn drop(&mut self) {
            let _ = std::fs::set_permissions(self.0, std::fs::Permissions::from_mode(0o755));
            let _ = chmod_tree(self.0, 0o755, 0o644);
        }
    }

    let _restore = RestoreWritable(dir.path());
    chmod_tree(dir.path(), 0o555, 0o444).expect("chmod tree read-only");
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o555))
        .expect("chmod 0555");

    let err = FjallBackend::open(path, &fjall_config(path, true))
        .err()
        .expect("a read_only open on a non-writable dir must fail");
    let Error::Io(ref io) = err else {
        panic!("non-writable dir must surface a typed Io error, got: {err}");
    };
    assert!(
        matches!(
            io.kind(),
            std::io::ErrorKind::PermissionDenied | std::io::ErrorKind::ReadOnlyFilesystem
        ),
        "the Io kind must stay PermissionDenied/ReadOnlyFilesystem, got: {}",
        io.kind()
    );
    assert!(
        err.to_string().contains("rocksdb"),
        "the message must point at RocksDB for physical read-only: {err}"
    );

    // Restore writability so the tempdir can be cleaned up.
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o755))
        .expect("chmod 0755");
}

/// FIND-632 / ADR-0001: every mutating `StorageBackend` call on a backend
/// opened `read_only` returns a typed `Validation { field: "read_only" }`
/// error — the same shape the engine-level `guard_write_allowed` produces —
/// never silently writes and never panics.
#[test]
fn fjall_read_only_rejects_backend_writes_typed() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().to_str().expect("path");

    {
        let backend = FjallBackend::open(path, &fjall_config(path, false))
            .expect("writer creates the database");
        backend
            .put(BackendPartition::Default, b"seed", b"v")
            .expect("seed");
        backend.flush().expect("flush");
    }

    let backend = FjallBackend::open(path, &fjall_config(path, true))
        .expect("read_only backend still opens (logical barrier)");

    let assert_read_only_err = |res: crate::error::Result<()>, op: &str| {
        let err = res
            .err()
            .unwrap_or_else(|| panic!("`{op}` must fail on a read_only backend"));
        assert!(
            matches!(err, Error::Validation { ref field, .. } if field == "read_only"),
            "`{op}` must be Validation{{field:\"read_only\"}}, got: {err}"
        );
    };

    assert_read_only_err(backend.put(BackendPartition::Default, b"k", b"v"), "put");
    assert_read_only_err(backend.delete(BackendPartition::Default, b"seed"), "delete");
    assert_read_only_err(
        backend.write_batch(vec![BackendWriteOp::Put {
            partition: BackendPartition::Default,
            key: b"k".to_vec(),
            value: b"v".to_vec(),
        }]),
        "write_batch",
    );
    assert_read_only_err(backend.flush(), "flush");

    let compact_err = backend
        .as_compactable()
        .expect("fjall is compactable")
        .compact()
        .expect_err("`compact` must fail on a read_only backend");
    assert!(
        matches!(compact_err, Error::Validation { ref field, .. } if field == "read_only"),
        "`compact` must be Validation{{field:\"read_only\"}}, got: {compact_err}"
    );

    // Reads still work — the barrier is logical, the data is served.
    assert_eq!(
        backend
            .get(BackendPartition::Default, b"seed")
            .expect("get")
            .as_deref(),
        Some(b"v".as_slice()),
        "a read_only backend still reads"
    );
}
