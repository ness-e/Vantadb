//! ADR-0057 W6: the activation of the at-rest encryption.
//!
//! These tests pin the contract of the activation wave:
//!
//! * the engine resolves **one** context for the four artefacts (WAL,
//!   VantaFile, backend KV, HNSW image) and **one** registry for them;
//! * an encrypted database opened without a key fails with
//!   `EncryptionKeyRequired` — never with a corruption error;
//! * a plaintext database opened with a key stays plaintext (no implicit
//!   migration, no mixed mode) and says so;
//! * the rotation re-wraps the DEKs (no ciphertext is invalidated) and
//!   leaves a `encryption.registry.bak` for a manual rollback;
//! * the snapshot **copies** the registry instead of hard-linking it.

use super::*;
use crate::backend::BackendKind;
use crate::config::Config;
use crate::error::Error;
use crate::node::{FieldValue, NodeFlags, UnifiedNode, VectorRepresentations};

/// The hex master key of the tests.
fn key_hex(seed: u8) -> String {
    format!("{seed:02x}").repeat(32)
}

/// The on-disk backend of the compiled feature set.
#[cfg(feature = "fjall")]
fn disk_backend() -> BackendKind {
    BackendKind::Fjall
}
#[cfg(all(not(feature = "fjall"), feature = "rocksdb"))]
fn disk_backend() -> BackendKind {
    BackendKind::RocksDb
}

/// A config with the master key configured programmatically.
fn keyed_config(path: &str, key: &str) -> Config {
    Config {
        storage_path: path.to_string(),
        backend_kind: disk_backend(),
        encryption_key: Some(key.to_string()),
        ..Config::default()
    }
}

/// A config with **no** key.
fn plain_config(path: &str) -> Config {
    Config {
        storage_path: path.to_string(),
        backend_kind: disk_backend(),
        encryption_key: None,
        ..Config::default()
    }
}

/// A node with a vector and a canary payload.
fn node(id: u128, canary: &str) -> UnifiedNode {
    let mut node = UnifiedNode::new(id);
    node.vector = VectorRepresentations::Full(vec![0.25, 0.5, 0.75]);
    node.flags.set(NodeFlags::HAS_VECTOR);
    node.relational.insert(
        "w6_canary".to_string(),
        FieldValue::String(canary.to_string()),
    );
    node
}

/// The string value of a relational field of `node`.
fn field(node: &UnifiedNode, key: &str) -> Option<String> {
    match node.relational.get(key) {
        Some(FieldValue::String(value)) => Some(value.clone()),
        _ => None,
    }
}

/// Run `f` under a scoped WARN-level subscriber and return what it logged.
fn capture_warnings<F: FnOnce()>(f: F) -> String {
    use std::io::Write;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Default)]
    struct CaptureWriter(Arc<Mutex<Vec<u8>>>);

    impl Write for CaptureWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0
                .lock()
                .expect("capture buffer")
                .extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for CaptureWriter {
        type Writer = CaptureWriter;
        fn make_writer(&'a self) -> Self::Writer {
            self.clone()
        }
    }

    let sink = CaptureWriter::default();
    let subscriber = tracing_subscriber::fmt()
        .with_writer(sink.clone())
        .with_max_level(tracing::Level::WARN)
        .with_ansi(false)
        .finish();
    tracing::subscriber::with_default(subscriber, f);
    let bytes = sink.0.lock().expect("capture buffer").clone();
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Every file under `dir` (recursively).
fn files_under(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(files_under(&path));
        } else {
            out.push(path);
        }
    }
    out
}

/// ADR-0057 W6: the engine resolves **one** context — and therefore **one**
/// registry — for the four artefacts.
///
/// With one registry per artefact, every `persist` would rewrite the whole
/// file with that artefact's own copy and the entries of the others would be
/// lost (FIND-514): the file would end up holding a single entry.
#[test]
fn engine_resolves_one_context_for_the_four_artifacts() {
    use crate::crypto::registry::{Artifact, KeyRegistry};
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().to_str().expect("path");
    let key = key_hex(0x11);

    {
        let engine = StorageEngine::open_with_config(path, Some(keyed_config(path, &key)))
            .expect("open with a key");
        engine
            .insert(&node(1, "vantadb-w6-one-context"))
            .expect("insert");
        engine.flush().expect("flush");
    }

    let data_dir = dir.path().join("data");
    let material = crate::crypto::resolve_master_key_material_strict(Some(&key), None, None)
        .expect("the material");
    let registry = KeyRegistry::open_or_create(&data_dir, &material[..]).expect("registry");
    let artifacts = registry.artifacts();
    assert_eq!(
        artifacts.len(),
        4,
        "one registry for the four artefacts: {artifacts:?}"
    );
    for artefact in [
        Artifact::Wal,
        Artifact::VantaFile,
        Artifact::HnswImage,
        Artifact::BackendKv,
    ] {
        assert!(
            artifacts.contains(&artefact),
            "the registry must record {}: {artifacts:?}",
            artefact.as_str()
        );
    }
}

/// ADR-0057 W6 / D7: an encrypted database opened **without** a key fails
/// with `EncryptionKeyRequired` — never with a corruption error, and never
/// handing the ciphertext back as if it were plaintext (FIND-516).
#[test]
fn encrypted_db_without_key_fails_with_key_required() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().to_str().expect("path");
    let key = key_hex(0x22);

    {
        let engine = StorageEngine::open_with_config(path, Some(keyed_config(path, &key)))
            .expect("open with a key");
        engine
            .insert(&node(7, "vantadb-w6-requires-key"))
            .expect("insert");
        engine.flush().expect("flush");
    }

    // The same database, without the key.
    let err = StorageEngine::open_with_config(path, Some(plain_config(path)))
        .err()
        .expect("an encrypted database must not open without a key");
    assert!(
        matches!(err, Error::EncryptionKeyRequired { .. }),
        "the error must be EncryptionKeyRequired, got: {err}"
    );
}

/// ADR-0057 W6: an encrypted database opened **with** the key has its four
/// artefacts encrypted — and the payload never appears in the clear.
#[test]
fn encrypted_db_with_key_encrypts_the_four_artifacts() {
    use crate::crypto::registry::{Artifact, KeyRegistry};
    const CANARY: &str = "vantadb-w6-encrypts-the-four-artefacts-4b8e1c";
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().to_str().expect("path");
    let key = key_hex(0x33);

    {
        let engine = StorageEngine::open_with_config(path, Some(keyed_config(path, &key)))
            .expect("open with a key");
        engine.insert(&node(11, CANARY)).expect("insert");
        engine.flush().expect("flush");
    }

    // The registry records the four artefacts.
    let data_dir = dir.path().join("data");
    let material = crate::crypto::resolve_master_key_material_strict(Some(&key), None, None)
        .expect("material");
    let registry = KeyRegistry::open_or_create(&data_dir, &material[..]).expect("registry");
    for artefact in [
        Artifact::Wal,
        Artifact::VantaFile,
        Artifact::HnswImage,
        Artifact::BackendKv,
    ] {
        assert!(
            registry.artifacts().contains(&artefact),
            "the registry must record {}",
            artefact.as_str()
        );
    }

    // The payload is nowhere in the clear: not in the WAL, not in the
    // segments, not in the index image, not in the backend's files.
    for file in files_under(dir.path()) {
        let bytes = std::fs::read(&file).expect("read");
        assert!(
            !bytes.windows(CANARY.len()).any(|w| w == CANARY.as_bytes()),
            "the payload must not appear in cleartext in {}",
            file.display()
        );
    }
}

/// ADR-0057 W6 / D7: a **plaintext** database opened with a key stays
/// plaintext — no implicit migration, no mixed mode — and the mismatch is
/// said out loud.
#[test]
fn plaintext_db_with_key_opens_plaintext_and_warns() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().to_str().expect("path");

    // A plaintext database.
    {
        let engine = StorageEngine::open_with_config(path, Some(plain_config(path))).expect("open");
        engine
            .insert(&node(13, "vantadb-w6-stays-plaintext"))
            .expect("insert");
        engine.flush().expect("flush");
    }

    // The same database, now with a key configured.
    let key = key_hex(0x44);
    let logged = capture_warnings(|| {
        let engine = StorageEngine::open_with_config(path, Some(keyed_config(path, &key)))
            .expect("a plaintext database opens with a key");
        drop(engine);
    });
    assert!(
        logged.contains("PLAINTEXT"),
        "the mismatch must be said out loud; got: {logged}"
    );
    assert!(
        !dir.path().join("data").join("encryption.registry").exists(),
        "a plaintext database must not get a registry: the key is ignored for it"
    );
}

/// ADR-0057 W6: the registry is part of the snapshot mirror.
#[test]
fn registry_is_included_in_the_snapshot_mirror() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().to_str().expect("path");
    let key = key_hex(0x55);

    let engine = StorageEngine::open_with_config(path, Some(keyed_config(path, &key)))
        .expect("open with a key");
    engine
        .insert(&node(17, "vantadb-w6-snapshot-mirror"))
        .expect("insert");
    engine.flush().expect("flush");
    let snapshot = engine.create_snapshot("w6").expect("snapshot");
    drop(engine);

    let mirrored = snapshot.path.join("data").join("encryption.registry");
    assert!(
        mirrored.exists(),
        "the snapshot must carry the registry: {}",
        snapshot.path.display()
    );
}

/// ADR-0057 W6: the snapshot **copies** the registry — a hard link would let
/// the next write of the live database mutate the snapshot through the
/// shared inode.
#[test]
fn registry_is_copied_not_hardlinked_by_the_snapshot() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().to_str().expect("path");
    let key = key_hex(0x66);

    let engine = StorageEngine::open_with_config(path, Some(keyed_config(path, &key)))
        .expect("open with a key");
    engine
        .insert(&node(19, "vantadb-w6-snapshot-copy"))
        .expect("insert");
    engine.flush().expect("flush");
    let snapshot = engine.create_snapshot("w6").expect("snapshot");
    drop(engine);

    let live = dir.path().join("data").join("encryption.registry");
    let mirrored = snapshot.path.join("data").join("encryption.registry");
    let before = std::fs::read(&mirrored).expect("the mirrored registry");

    // Reopening the live database allocates a new nonce epoch and rewrites
    // the registry: the snapshot's copy must not change.
    {
        let engine = StorageEngine::open_with_config(path, Some(keyed_config(path, &key)))
            .expect("reopen with a key");
        engine.flush().expect("flush");
    }
    let after = std::fs::read(&mirrored).expect("the mirrored registry");
    assert_eq!(
        &before, &after,
        "the snapshot's registry must be a copy: the live write must not reach it"
    );
    assert_ne!(
        std::fs::read(&live).expect("the live registry"),
        after,
        "the live registry must have moved on (a new nonce epoch)"
    );
}

/// ADR-0057 W6 / `FIND-194`: the rotation re-wraps the DEKs — it never
/// re-derives them — so **no ciphertext already on disk is invalidated**.
#[test]
fn encryption_rotate_preserves_readability_of_pre_rotation_data() {
    use crate::crypto::registry::KeyRegistry;
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().to_str().expect("path");
    let old_key = key_hex(0x77);
    let new_key = key_hex(0x88);

    {
        let engine = StorageEngine::open_with_config(path, Some(keyed_config(path, &old_key)))
            .expect("open with the old key");
        engine
            .insert(&node(23, "vantadb-w6-rotation"))
            .expect("insert");
        engine.flush().expect("flush");
    }

    // The rotation, with the database closed.
    let data_dir = dir.path().join("data");
    let material = crate::crypto::resolve_master_key_material_strict(Some(&old_key), None, None)
        .expect("material");
    let new_material =
        crate::crypto::resolve_master_key_material_strict(Some(&new_key), None, None)
            .expect("material");
    let mut registry = KeyRegistry::open_or_create(&data_dir, &material[..]).expect("registry");
    let previous_version = registry.key_version();
    let new_version = registry.rotate(&new_material[..]).expect("rotation");
    assert_eq!(
        new_version,
        previous_version + 1,
        "the rotation bumps the key version"
    );
    drop(registry);

    // The data still reads with the **new** key…
    let engine = StorageEngine::open_with_config(path, Some(keyed_config(path, &new_key)))
        .expect("open with the new key");
    let read = engine.get(23).expect("get").expect("the node is there");
    assert_eq!(
        field(&read, "w6_canary"),
        Some("vantadb-w6-rotation".to_string()),
        "the data written before the rotation must still be readable"
    );
    drop(engine);

    // …and the old key no longer opens the database.
    let err = StorageEngine::open_with_config(path, Some(keyed_config(path, &old_key)))
        .err()
        .expect("the old key must not open the rotated database");
    assert!(
        matches!(err, Error::EncryptionKeyRejected { .. }),
        "the old key must be rejected, got: {err}"
    );
}

/// ADR-0057 W6: the rotation leaves the previous registry as
/// `encryption.registry.bak` — the manual rollback.
#[test]
fn encryption_rotate_leaves_a_bak_registry() {
    use crate::crypto::registry::{KeyRegistry, REGISTRY_BACKUP_FILE_NAME};
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().to_str().expect("path");
    let old_key = key_hex(0x99);
    let new_key = key_hex(0xaa);

    {
        let engine = StorageEngine::open_with_config(path, Some(keyed_config(path, &old_key)))
            .expect("open with the old key");
        engine.insert(&node(29, "vantadb-w6-bak")).expect("insert");
        engine.flush().expect("flush");
    }

    let data_dir = dir.path().join("data");
    let material = crate::crypto::resolve_master_key_material_strict(Some(&old_key), None, None)
        .expect("material");
    let new_material =
        crate::crypto::resolve_master_key_material_strict(Some(&new_key), None, None)
            .expect("material");
    let mut registry = KeyRegistry::open_or_create(&data_dir, &material[..]).expect("registry");
    registry.rotate(&new_material[..]).expect("rotation");
    drop(registry);

    let backup = data_dir.join(REGISTRY_BACKUP_FILE_NAME);
    assert!(backup.exists(), "the rotation must leave a backup registry");

    // The backup is the **old** registry: the old key still unwraps it.
    let old_material =
        crate::crypto::resolve_master_key_material_strict(Some(&old_key), None, None)
            .expect("material");
    let backup_registry = KeyRegistry::open_or_create(&data_dir, &old_material[..]);
    assert!(
        backup_registry.is_err(),
        "the current registry must not open with the old key"
    );
}

/// ADR-0057 W6: the registry keeps the four entries after two rotations —
/// the rotation re-wraps, it never drops an artefact.
#[test]
fn registry_keeps_the_four_entries_after_two_rotations() {
    use crate::crypto::registry::KeyRegistry;
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().to_str().expect("path");
    let first_key = key_hex(0xbb);
    let second_key = key_hex(0xcc);
    let third_key = key_hex(0xdd);

    {
        let engine = StorageEngine::open_with_config(path, Some(keyed_config(path, &first_key)))
            .expect("open with the first key");
        engine
            .insert(&node(31, "vantadb-w6-two-rotations"))
            .expect("insert");
        engine.flush().expect("flush");
    }

    let data_dir = dir.path().join("data");
    let material = crate::crypto::resolve_master_key_material_strict(Some(&first_key), None, None)
        .expect("material");
    let second_material =
        crate::crypto::resolve_master_key_material_strict(Some(&second_key), None, None)
            .expect("material");
    let third_material =
        crate::crypto::resolve_master_key_material_strict(Some(&third_key), None, None)
            .expect("material");

    let mut registry = KeyRegistry::open_or_create(&data_dir, &material[..]).expect("registry");
    assert_eq!(registry.artifacts().len(), 4, "four artefacts before");
    registry
        .rotate(&second_material[..])
        .expect("first rotation");
    assert_eq!(
        registry.artifacts().len(),
        4,
        "the rotation must not drop an artefact"
    );
    registry
        .rotate(&third_material[..])
        .expect("second rotation");
    assert_eq!(
        registry.artifacts().len(),
        4,
        "the second rotation must not drop an artefact"
    );
    drop(registry);

    // The database still opens with the last key.
    let engine = StorageEngine::open_with_config(path, Some(keyed_config(path, &third_key)))
        .expect("open with the third key");
    let read = engine.get(31).expect("get").expect("the node is there");
    assert_eq!(
        field(&read, "w6_canary"),
        Some("vantadb-w6-two-rotations".to_string())
    );
}

/// ADR-0057 W6: the snapshot of an encrypted database is restorable **with**
/// the key.
#[test]
fn snapshot_of_an_encrypted_db_is_restorable_with_the_key() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().to_str().expect("path");
    let key = key_hex(0xee);

    let engine = StorageEngine::open_with_config(path, Some(keyed_config(path, &key)))
        .expect("open with a key");
    engine
        .insert(&node(37, "vantadb-w6-snapshot-restore"))
        .expect("insert");
    engine.flush().expect("flush");
    let snapshot = engine.create_snapshot("w6").expect("snapshot");
    drop(engine);

    // The snapshot carries the encrypted registry…
    let mirrored = snapshot.path.join("data").join("encryption.registry");
    assert!(mirrored.exists(), "the snapshot must carry the registry");

    // …and it opens with the key.
    let snapshot_path = snapshot.path.to_str().expect("path");
    let engine =
        StorageEngine::open_with_config(snapshot_path, Some(keyed_config(snapshot_path, &key)))
            .expect("the snapshot opens with the key");
    drop(engine);
}

/// ADR-0057 W6: the snapshot of an encrypted database is **refused** without
/// the key — an encrypted snapshot must never open as if it were plaintext.
#[test]
fn snapshot_of_an_encrypted_db_is_refused_without_the_key() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().to_str().expect("path");
    let key = key_hex(0xf0);

    let engine = StorageEngine::open_with_config(path, Some(keyed_config(path, &key)))
        .expect("open with a key");
    engine
        .insert(&node(41, "vantadb-w6-snapshot-refused"))
        .expect("insert");
    engine.flush().expect("flush");
    let snapshot = engine.create_snapshot("w6").expect("snapshot");
    drop(engine);

    let snapshot_path = snapshot.path.to_str().expect("path");
    let err = StorageEngine::open_with_config(snapshot_path, Some(plain_config(snapshot_path)))
        .err()
        .expect("an encrypted snapshot must not open without a key");
    assert!(
        matches!(err, Error::EncryptionKeyRequired { .. }),
        "the error must be EncryptionKeyRequired, got: {err}"
    );
}

/// The documentation gate: `SECURITY.md` must publish the **real** scope of
/// the feature — the four artefacts, the opt-in and the declared limits.
#[test]
fn security_md_claim_matches_the_implemented_scope() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(root.join("SECURITY.md")).expect("SECURITY.md");
    for artefact in ["WAL", "HNSW", "VantaFile", "backend"] {
        assert!(
            text.contains(artefact),
            "SECURITY.md must name the {artefact} among the encrypted artefacts"
        );
    }
    assert!(
        text.contains("encryption") && text.contains("AES-256-GCM"),
        "SECURITY.md must name the cipher"
    );
    assert!(
        text.contains("opt-in") || text.contains("not in `default`"),
        "SECURITY.md must say that the feature is opt-in"
    );
}

/// The documentation gate: no document may still claim that the core
/// storage files are **not** encrypted at rest — the wiring landed
/// (ADR-0057 R-12: the inverse false claim is just as bad).
#[test]
fn no_document_still_says_not_encrypted_at_rest_yet() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for file in [
        "SECURITY.md",
        "src/crypto.rs",
        "docs/user/operations/CONFIGURATION.md",
    ] {
        let path = root.join(file);
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let lower = text.to_lowercase();
        assert!(
            !lower.contains("not encrypted at rest yet"),
            "{file} still claims the core storage files are not encrypted at rest"
        );
        assert!(
            !lower.contains("not yet wired"),
            "{file} still claims the feature is not wired to the storage paths"
        );
    }
}
