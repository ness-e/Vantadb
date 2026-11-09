//! Encryption command handlers — rotate the master key of a database
//! (ADR-0057 W6).
//!
//! The rotation **re-wraps** the per-artefact DEKs under a new KEK: the DEKs
//! themselves are never re-derived, so **no ciphertext already on disk is
//! invalidated** (`FIND-194`). The operation is O(number of artefacts) = 4,
//! not O(data).
//!
//! The previous registry is left as `encryption.registry.bak`: copying it
//! back is a manual rollback, possible while the **old** master key still
//! exists.
//!
//! The database must be **closed**: the rotation rewrites the registry that
//! the live engine holds open.

use crate::error::{Error, Result};

/// Acquire `.vanta.lock` under `base_path` **exclusively**, with the exact
/// semantics of the engine (`src/storage/engine/init.rs`): same lock path,
/// same `fs2::FileExt::try_lock_exclusive` (a no-op without the `fs2`
/// feature), the same `Error::DatabaseBusy` when the lock is held, and the
/// same lock-metadata write so a later engine open reports who holds it.
///
/// The rotation must fail **immediately** when a live process owns the lock —
/// nothing on disk is touched. The returned `File` owns the lock and releases
/// it on drop: the caller keeps it alive until after the last fsync.
#[cfg(feature = "encryption")]
fn acquire_rotation_lock(base_path: &std::path::Path) -> Result<std::fs::File> {
    use std::fs::OpenOptions;

    let lock_path = base_path.join(".vanta.lock");
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        // The lock file must not be truncated: the metadata of the holder
        // that may still own it lives inside.
        .truncate(false)
        .open(&lock_path)
        .map_err(Error::Io)?;

    #[cfg(feature = "fs2")]
    {
        fs2::FileExt::try_lock_exclusive(&file).map_err(|_| {
            let holder_detail = crate::storage::lock::read_lock_metadata(&lock_path)
                .map(|meta| {
                    format!(
                        " (held by '{}', PID {} on host '{}')",
                        meta.process_name, meta.pid, meta.hostname
                    )
                })
                .unwrap_or_default();
            Error::DatabaseBusy(format!(
                "Database at '{}' is locked by another process{holder_detail}. \
                 Cannot acquire exclusive writer lock for the key rotation.",
                base_path.display()
            ))
        })?;
        let _ = crate::storage::lock::write_lock_metadata(
            &file,
            &crate::storage::lock::LockMetadata::current(),
        );
    }

    Ok(file)
}

/// Read one key line from `reader` (stdin in production, a `Cursor` in
/// tests): at most `MAX_KEY_LINE` bytes — a hex master key is ≤100 bytes —
/// with the trailing newline stripped by `decode_hex`'s internal `trim`.
/// The raw buffer is zeroized on every path, including a mid-read error
/// (audit R-3): an unbounded `read_until` on an endless stdin is an OOM
/// vector and a partial key must never linger unwiped.
#[cfg(feature = "encryption")]
fn read_new_key_line<R: std::io::BufRead>(reader: R) -> Result<crate::crypto::Zeroizing<Vec<u8>>> {
    use crate::crypto::{Zeroize, Zeroizing};

    const MAX_KEY_LINE: usize = 256;
    let mut raw = Vec::new();
    for byte in reader.bytes() {
        match byte {
            Ok(b'\n') => break,
            Ok(b) => {
                if raw.len() >= MAX_KEY_LINE {
                    raw.zeroize();
                    return Err(Error::generic_error(
                        "the new key line is too long (a hex master key is at most 100 bytes)",
                    ));
                }
                raw.push(b);
            }
            Err(e) => {
                raw.zeroize();
                return Err(Error::generic_error(format!(
                    "cannot read the new key from stdin: {e}"
                )));
            }
        }
    }
    let raw = Zeroizing::new(raw);
    let text = std::str::from_utf8(&raw[..])
        .map_err(|e| Error::generic_error(format!("the new key from stdin is not UTF-8: {e}")))?;
    Ok(Zeroizing::new(crate::crypto::decode_hex(text).map_err(
        |e| Error::generic_error(format!("the new key from stdin is not valid: {e}")),
    )?))
}

/// Rotate the master key of the database at `db_path`.
///
/// The **current** key comes from the usual sources (environment → key file
/// → key command); the **new** key comes from `--new-key-file` or
/// `--new-key-stdin` (FIND-587 — there is no `--new-key <hex>`: a key on the
/// command line is visible in the process list and the shell history).
#[cfg(feature = "encryption")]
// `new_key_file` leads to the new master key: it must never reach a span
// field (the default subscriber prints the span with its fields, and the
// logs are centralised).
#[tracing::instrument(skip(new_key_file))]
pub fn cmd_encryption_rotate(
    db_path: &str,
    new_key_file: Option<&str>,
    new_key_stdin: bool,
    json_output: bool,
) -> Result<()> {
    use crate::cli_handlers::{print_info, print_json, print_success};
    use crate::crypto::registry::{KeyRegistry, REGISTRY_BACKUP_FILE_NAME};
    use crate::crypto::{resolve_master_key_material_strict, CryptoError};

    let base_path = std::path::Path::new(db_path);
    let data_dir = base_path.join("data");
    let registry_path = KeyRegistry::path_in(&data_dir);
    if !registry_path.exists() {
        return Err(Error::generic_error(format!(
            "{} has no encryption registry: the database is not encrypted",
            db_path
        )));
    }

    // FIND-587: take `.vanta.lock` exclusively — rotating the registry while
    // a live engine holds it is a destructive race. The lock is held until
    // after the last fsync and released on drop. The handler never calls
    // `StorageEngine::open`, so there is no auto-deadlock.
    let _lock = acquire_rotation_lock(base_path)?;

    // The current key: the same three sources the engine uses. The env var
    // IS the hex key: its bytes are wrapped in `Zeroizing` so the `String`
    // copy does not linger after the scope ends (audit R-4).
    let env_key = std::env::var("VANTADB_ENCRYPTION_KEY")
        .ok()
        .map(|s| crate::crypto::Zeroizing::new(s.into_bytes()));
    let env_key_str = env_key
        .as_ref()
        .map(|z| std::str::from_utf8(&z[..]).unwrap_or_default());
    let key_file = std::env::var("VANTADB_ENCRYPTION_KEY_FILE").ok();
    let key_cmd = std::env::var("VANTADB_ENCRYPTION_KEY_CMD").ok();
    let current = resolve_master_key_material_strict(
        env_key_str,
        key_file.as_ref().map(std::path::Path::new),
        key_cmd.as_deref(),
    )
    .map_err(|e| match e {
        CryptoError::KeyNotSet => Error::generic_error(
            "no current encryption key: set VANTADB_ENCRYPTION_KEY, \
             VANTADB_ENCRYPTION_KEY_FILE or VANTADB_ENCRYPTION_KEY_CMD",
        ),
        other => Error::generic_error(format!("encryption key error: {other}")),
    })?;

    // The new key, from exactly one source (the CLI ArgGroup guarantees it).
    // Every byte buffer holding key material lives in a `Zeroizing` holder so
    // it is wiped on drop (ADR-0057 C-5). The hex text is read as raw bytes
    // because the local `Zeroize` trait has no `String` impl.
    let new_material = match (new_key_file, new_key_stdin) {
        (Some(path), false) => {
            // `read_key_file` applies the same hardening as the *current*
            // key source (ADR-0057 D2): regular file, never a symlink,
            // `0600` mode on Unix (audit R-2).
            crate::crypto::Zeroizing::new(
                crate::crypto::read_key_file(std::path::Path::new(path)).map_err(|e| {
                    Error::generic_error(format!("the new key file is not valid: {e}"))
                })?,
            )
        }
        (None, true) => read_new_key_line(std::io::stdin().lock())?,
        // The ArgGroup rejects this at parse time; belt and braces for any
        // programmatic caller of this handler.
        _ => {
            return Err(Error::generic_error(
                "a new key is required: pass --new-key-file <path> or --new-key-stdin",
            ))
        }
    };

    let mut registry = KeyRegistry::open_or_create(&data_dir, &current[..])
        .map_err(|e| Error::generic_error(format!("cannot open the encryption registry: {e}")))?;
    let artifacts = registry.artifacts();
    let previous_version = registry.key_version();
    let new_version = registry
        .rotate(&new_material[..])
        .map_err(|e| Error::generic_error(format!("the rotation failed: {e}")))?;

    let backup = data_dir.join(REGISTRY_BACKUP_FILE_NAME);

    if json_output {
        return print_json(&serde_json::json!({
            "rotated": true,
            "key_version": new_version,
            "previous_key_version": previous_version,
            "artifacts": artifacts.iter().map(|a| a.as_str()).collect::<Vec<_>>(),
            "registry": registry_path.display().to_string(),
            "backup": backup.display().to_string(),
        }));
    }
    print_success(&format!(
        "Master key rotated: key_version {previous_version} → {new_version} \
         ({} artefacts re-wrapped; no ciphertext was re-encrypted).",
        artifacts.len()
    ));
    print_info(&format!("Registry: {}", registry_path.display()));
    print_info(&format!(
        "Previous registry kept for a manual rollback: {}",
        backup.display()
    ));
    Ok(())
}

/// Rotate the master key of the database at `db_path`.
///
/// This build has **no** `encryption` feature: there is nothing to rotate.
#[cfg(not(feature = "encryption"))]
// `new_key_file` leads to the new master key: it must never reach a span
// field (the default subscriber prints the span with its fields, and the
// logs are centralised).
#[tracing::instrument(skip(new_key_file))]
pub fn cmd_encryption_rotate(
    db_path: &str,
    new_key_file: Option<&str>,
    new_key_stdin: bool,
    json_output: bool,
) -> Result<()> {
    let _ = (db_path, new_key_file, new_key_stdin, json_output);
    Err(Error::generic_error(
        "this build has no `encryption` feature: the at-rest encryption cannot be \
         rotated. Rebuild with `--features encryption`.",
    ))
}

#[cfg(all(test, feature = "encryption"))]
mod tests {
    use super::*;
    use crate::backend::BackendKind;
    use crate::config::Config;
    use crate::storage::engine::StorageEngine;

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

    /// FIND-587: `--new-key-stdin` reads exactly one line and hex-decodes it.
    #[test]
    fn read_new_key_line_decodes_one_line() {
        use std::io::Cursor;
        let key = key_hex(0x50);
        let mut input = key.clone();
        input.push('\n');
        input.push_str("ignored second line");
        let material =
            read_new_key_line(Cursor::new(input.into_bytes())).expect("one hex line must decode");
        assert_eq!(material.len(), 32);
        assert_eq!(material[0], 0x50);
    }

    /// FIND-587: a non-hex line from stdin is rejected, never panics.
    #[test]
    fn read_new_key_line_rejects_garbage() {
        use std::io::Cursor;
        let err = read_new_key_line(Cursor::new(b"not-hex\n".to_vec()))
            .expect_err("garbage must be rejected");
        assert!(err.to_string().contains("stdin"), "unexpected: {err}");
    }

    /// FIND-587 (a): with `.vanta.lock` held exclusively (a live engine),
    /// `rotate` must fail with `DatabaseBusy` and leave `encryption.registry`
    /// byte-for-byte identical — no `.bak`, no partial rotation.
    #[test]
    fn rotate_fails_when_vanta_lock_is_held() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().to_str().expect("path");
        let old_key = key_hex(0x41);

        // A live engine holds `.vanta.lock` exclusively.
        let _engine = StorageEngine::open_with_config(path, Some(keyed_config(path, &old_key)))
            .expect("open with the old key");

        let registry_path = dir.path().join("data").join("encryption.registry");
        let before = std::fs::read(&registry_path).expect("registry bytes");

        // The new key would come from a file — it must never even be read.
        let key_file = dir.path().join("new.key");
        std::fs::write(&key_file, key_hex(0x42)).expect("new key file");

        let err = cmd_encryption_rotate(path, key_file.to_str(), false, true)
            .expect_err("the rotation must fail while the database is locked");
        assert!(
            matches!(err, Error::DatabaseBusy(_)),
            "expected DatabaseBusy, got: {err}"
        );

        let after = std::fs::read(&registry_path).expect("registry bytes");
        assert_eq!(
            before, after,
            "the registry must be byte-for-byte identical when the lock is held"
        );
        assert!(
            !dir.path()
                .join("data")
                .join("encryption.registry.bak")
                .exists(),
            "no backup may be written when the rotation never started"
        );
    }

    /// FIND-587 (c): a rotation fed by `--new-key-file` re-wraps the DEKs —
    /// the database opens with the new key, is rejected with the old one, and
    /// the previous registry survives as `encryption.registry.bak`.
    #[test]
    fn rotate_with_new_key_file_rotates_and_leaves_a_bak() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().to_str().expect("path");
        let old_key = key_hex(0x43);
        let new_key = key_hex(0x44);

        {
            let engine = StorageEngine::open_with_config(path, Some(keyed_config(path, &old_key)))
                .expect("open with the old key");
            engine.flush().expect("flush");
        }

        // The current key resolves from the environment, as in production.
        std::env::set_var("VANTADB_ENCRYPTION_KEY", &old_key);
        let key_file = dir.path().join("new.key");
        std::fs::write(&key_file, &new_key).expect("new key file");

        cmd_encryption_rotate(path, key_file.to_str(), false, true)
            .expect("the rotation must succeed");
        std::env::remove_var("VANTADB_ENCRYPTION_KEY");

        assert!(
            dir.path()
                .join("data")
                .join("encryption.registry.bak")
                .exists(),
            "the rotation must leave a backup registry"
        );

        // The database opens with the new key…
        StorageEngine::open_with_config(path, Some(keyed_config(path, &new_key)))
            .expect("open with the new key");

        // …and the old key no longer opens it.
        // `expect_err` needs `T: Debug` and `StorageEngine` does not
        // implement it; a match keeps the intent without the bound.
        let err = match StorageEngine::open_with_config(path, Some(keyed_config(path, &old_key))) {
            Err(e) => e,
            Ok(_) => panic!("the old key must not open the rotated database"),
        };
        assert!(
            matches!(err, Error::EncryptionKeyRejected { .. }),
            "the old key must be rejected, got: {err}"
        );
    }
}
