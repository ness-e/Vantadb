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
fn read_key_line<R: std::io::BufRead>(
    reader: R,
    source_label: &str,
) -> Result<crate::crypto::Zeroizing<Vec<u8>>> {
    use crate::crypto::{Zeroize, Zeroizing};

    const MAX_KEY_LINE: usize = 256;
    let mut raw = Vec::new();
    for byte in reader.bytes() {
        match byte {
            Ok(b'\n') => break,
            Ok(b) => {
                if raw.len() >= MAX_KEY_LINE {
                    raw.zeroize();
                    return Err(Error::generic_error(format!(
                        "{source_label} line is too long (a hex master key is at most 100 bytes)"
                    )));
                }
                raw.push(b);
            }
            Err(e) => {
                raw.zeroize();
                return Err(Error::generic_error(format!(
                    "cannot read {source_label}: {e}"
                )));
            }
        }
    }
    let raw = Zeroizing::new(raw);
    let text = std::str::from_utf8(&raw[..])
        .map_err(|e| Error::generic_error(format!("{source_label} is not UTF-8: {e}")))?;
    Ok(Zeroizing::new(crate::crypto::decode_hex(text).map_err(
        |e| Error::generic_error(format!("{source_label} is not valid: {e}")),
    )?))
}

#[cfg(feature = "encryption")]
fn read_new_key_line<R: std::io::BufRead>(reader: R) -> Result<crate::crypto::Zeroizing<Vec<u8>>> {
    read_key_line(reader, "the new key from stdin")
}

/// Read master key material from `path` with full hardening (regular file, no
/// symlinks, `0600` mode on Unix) into a `Zeroizing` buffer.
#[cfg(feature = "encryption")]
fn read_key_file_zeroizing(
    path: &str,
    source_label: &str,
) -> Result<crate::crypto::Zeroizing<Vec<u8>>> {
    Ok(crate::crypto::Zeroizing::new(
        crate::crypto::read_key_file(std::path::Path::new(path))
            .map_err(|e| Error::generic_error(format!("{source_label} is not valid: {e}")))?,
    ))
}

/// Resolve master key material from the standard environment variables into a
/// `Zeroizing` buffer (audit R-4).
#[cfg(feature = "encryption")]
fn resolve_env_master_key_material(
    missing_key_message: &str,
) -> Result<crate::crypto::Zeroizing<Vec<u8>>> {
    use crate::crypto::{resolve_master_key_material_strict, CryptoError, Zeroizing};

    let env_key = std::env::var("VANTADB_ENCRYPTION_KEY")
        .ok()
        .map(|s| Zeroizing::new(s.into_bytes()));
    let env_key_str = env_key
        .as_ref()
        .map(|z| std::str::from_utf8(&z[..]).unwrap_or_default());
    let key_file = std::env::var("VANTADB_ENCRYPTION_KEY_FILE").ok();
    let key_cmd = std::env::var("VANTADB_ENCRYPTION_KEY_CMD").ok();
    resolve_master_key_material_strict(
        env_key_str,
        key_file.as_ref().map(std::path::Path::new),
        key_cmd.as_deref(),
    )
    .map_err(|e| match e {
        CryptoError::KeyNotSet => Error::generic_error(missing_key_message.to_string()),
        other => Error::generic_error(format!("encryption key error: {other}")),
    })
}

/// Locate the database's existing `encryption.registry` under `<db_path>/data`.
#[cfg(feature = "encryption")]
fn resolve_existing_registry_path(
    db_path: &str,
    missing_detail: &str,
) -> Result<(std::path::PathBuf, std::path::PathBuf)> {
    let data_dir = std::path::Path::new(db_path).join("data");
    let registry_path = crate::crypto::registry::KeyRegistry::path_in(&data_dir);
    if !registry_path.exists() {
        return Err(Error::generic_error(format!(
            "{db_path} has no encryption registry: {missing_detail}"
        )));
    }
    Ok((data_dir, registry_path))
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

    let base_path = std::path::Path::new(db_path);
    let (data_dir, registry_path) =
        resolve_existing_registry_path(db_path, "the database is not encrypted")?;

    // FIND-587: take `.vanta.lock` exclusively — rotating the registry while
    // a live engine holds it is a destructive race. The lock is held until
    // after the last fsync and released on drop. The handler never calls
    // `StorageEngine::open`, so there is no auto-deadlock.
    let _lock = acquire_rotation_lock(base_path)?;

    let current = resolve_env_master_key_material(
        "no current encryption key: set VANTADB_ENCRYPTION_KEY, \
         VANTADB_ENCRYPTION_KEY_FILE or VANTADB_ENCRYPTION_KEY_CMD",
    )?;

    // The new key, from exactly one source (the CLI ArgGroup guarantees it).
    let new_material = match (new_key_file, new_key_stdin) {
        (Some(path), false) => read_key_file_zeroizing(path, "the new key file")?,
        (None, true) => read_new_key_line(std::io::stdin().lock())?,
        // The ArgGroup rejects this at parse time; belt and braces for any
        // programmatic caller of this handler.
        _ => {
            return Err(Error::generic_error(
                "a new key is required: pass --new-key-file <path> or --new-key-stdin",
            ))
        }
    };

    // FIND-634: `open_existing`, not `open_or_create` — the `exists()` check
    // above races with a concurrent delete, and minting an empty registry
    // here would "rotate" a registry the database never had.
    let mut registry = KeyRegistry::open_existing(&data_dir, &current[..])
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

/// Verify a master key against the database's `encryption.registry` without
/// opening the database or taking `.vanta.lock` (FIND-646).
///
/// Key source precedence: `--key-file`, `--key-stdin`, or the standard
/// environment sources (`VANTADB_ENCRYPTION_KEY`, `VANTADB_ENCRYPTION_KEY_FILE`,
/// `VANTADB_ENCRYPTION_KEY_CMD`). All key material stays inside `Zeroizing`.
#[cfg(feature = "encryption")]
#[tracing::instrument(skip(key_file))]
pub fn cmd_encryption_verify_key(
    db_path: &str,
    key_file: Option<&str>,
    key_stdin: bool,
    json_output: bool,
) -> Result<()> {
    if key_file.is_some() && key_stdin {
        return Err(Error::generic_error(
            "pass at most one key source: --key-file or --key-stdin, not both",
        ));
    }
    verify_key_with_reader(
        db_path,
        key_file,
        key_stdin.then(|| std::io::stdin().lock()),
        json_output,
    )
}

#[cfg(feature = "encryption")]
fn verify_key_with_reader<R: std::io::BufRead>(
    db_path: &str,
    key_file: Option<&str>,
    stdin_reader: Option<R>,
    json_output: bool,
) -> Result<()> {
    use crate::cli_handlers::{print_json, print_success};

    let (data_dir, registry_path) = resolve_existing_registry_path(db_path, "cannot verify key")?;

    let material = match (key_file, stdin_reader) {
        (Some(path), None) => read_key_file_zeroizing(path, "the key file")?,
        (None, Some(reader)) => read_key_line(reader, "the key from stdin")?,
        (None, None) => resolve_env_master_key_material(
            "no encryption key: pass --key-file, --key-stdin, or set VANTADB_ENCRYPTION_KEY, \
             VANTADB_ENCRYPTION_KEY_FILE or VANTADB_ENCRYPTION_KEY_CMD",
        )?,
        (Some(_), Some(_)) => {
            return Err(Error::generic_error(
                "pass at most one key source: --key-file or --key-stdin, not both",
            ))
        }
    };

    let info = crate::crypto::verify_registry_key(&data_dir, &material[..])?;

    if json_output {
        return print_json(&serde_json::json!({
            "valid": true,
            "key_version": info.key_version,
            "artifacts": info.artifacts,
            "registry": registry_path.display().to_string(),
        }));
    }
    print_success(&format!(
        "Master key verified against {} (key_version {}, {} artefacts).",
        registry_path.display(),
        info.key_version,
        info.artifacts.len()
    ));
    Ok(())
}

/// Verify a master key against the database's `encryption.registry` without
/// opening the database or taking `.vanta.lock` (FIND-646).
///
/// This build has **no** `encryption` feature: there is nothing to verify.
#[cfg(not(feature = "encryption"))]
#[tracing::instrument(skip(key_file))]
pub fn cmd_encryption_verify_key(
    db_path: &str,
    key_file: Option<&str>,
    key_stdin: bool,
    json_output: bool,
) -> Result<()> {
    let _ = (db_path, key_file, key_stdin, json_output);
    Err(Error::generic_error(
        "this build has no `encryption` feature: the at-rest encryption cannot be \
         verified. Rebuild with `--features encryption`.",
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

    /// FIND-587: the new key goes through `read_key_file`, so on Unix a key
    /// file readable by group or other is refused before anything is
    /// rotated — the registry stays byte-for-byte identical.
    #[cfg(unix)]
    #[test]
    fn rotate_rejects_a_new_key_file_readable_by_others() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().to_str().expect("path");
        let old_key = key_hex(0x45);
        {
            let engine = StorageEngine::open_with_config(path, Some(keyed_config(path, &old_key)))
                .expect("open with the old key");
            engine.flush().expect("flush");
        }
        let registry = dir.path().join("data").join("encryption.registry");
        let before = std::fs::read(&registry).expect("registry");

        std::env::set_var("VANTADB_ENCRYPTION_KEY", &old_key);
        let key_file = dir.path().join("new.key");
        std::fs::write(&key_file, key_hex(0x46)).expect("new key file");
        std::fs::set_permissions(&key_file, std::fs::Permissions::from_mode(0o644))
            .expect("make the key file world-readable");
        let err = cmd_encryption_rotate(path, key_file.to_str(), false, true)
            .expect_err("a key file readable by others must be refused");
        std::env::remove_var("VANTADB_ENCRYPTION_KEY");

        assert!(format!("{err:?}").contains("0600"), "got: {err:?}");
        assert_eq!(
            std::fs::read(&registry).expect("registry"),
            before,
            "a refused rotation must not touch the registry"
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
        // `read_key_file` rejects a key file readable by group or other on
        // Unix; the default umask leaves it 0644.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&key_file, std::fs::Permissions::from_mode(0o600))
                .expect("restrict the key file to 0600");
        }

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

    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    struct EnvVarGuard {
        name: &'static str,
        prev: Option<std::ffi::OsString>,
    }

    impl EnvVarGuard {
        fn set(name: &'static str, val: &str) -> Self {
            let prev = std::env::var_os(name);
            std::env::set_var(name, val);
            Self { name, prev }
        }

        fn remove(name: &'static str) -> Self {
            let prev = std::env::var_os(name);
            std::env::remove_var(name);
            Self { name, prev }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match &self.prev {
                Some(v) => std::env::set_var(self.name, v),
                None => std::env::remove_var(self.name),
            }
        }
    }

    fn write_restricted_key_file(
        dir: &std::path::Path,
        name: &str,
        key: &str,
    ) -> std::path::PathBuf {
        let key_file = dir.join(name);
        std::fs::write(&key_file, key).expect("write key file");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&key_file, std::fs::Permissions::from_mode(0o600))
                .expect("restrict key file to 0600");
        }
        key_file
    }

    /// FIND-646: verifying the right key via `--key-file`, `--key-stdin`,
    /// `VANTADB_ENCRYPTION_KEY`, or `VANTADB_ENCRYPTION_KEY_FILE` succeeds and
    /// leaves `encryption.registry` bytes, mtime, and `.vanta.lock` untouched.
    #[test]
    fn verify_key_succeeds_with_correct_key_sources_without_modifying_registry() {
        let _env_guard = ENV_LOCK.lock().expect("env lock");
        let _clear_env = EnvVarGuard::remove("VANTADB_ENCRYPTION_KEY");
        let _clear_file = EnvVarGuard::remove("VANTADB_ENCRYPTION_KEY_FILE");
        let _clear_cmd = EnvVarGuard::remove("VANTADB_ENCRYPTION_KEY_CMD");

        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().to_str().expect("path");
        let right_key = key_hex(0x61);
        let other_key = key_hex(0x62);

        {
            let engine =
                StorageEngine::open_with_config(path, Some(keyed_config(path, &right_key)))
                    .expect("open encrypted db");
            engine.flush().expect("flush");
        }

        let registry_path = dir.path().join("data").join("encryption.registry");
        let lock_path = dir.path().join(".vanta.lock");
        let _ = std::fs::remove_file(&lock_path);
        let before_bytes = std::fs::read(&registry_path).expect("registry bytes");
        let before_mtime = std::fs::metadata(&registry_path)
            .expect("registry metadata")
            .modified()
            .expect("registry mtime");

        // 1) --key-file (even if VANTADB_ENCRYPTION_KEY holds a wrong key!)
        let key_file = write_restricted_key_file(dir.path(), "right.key", &right_key);
        {
            let _wrong_env = EnvVarGuard::set("VANTADB_ENCRYPTION_KEY", &other_key);
            cmd_encryption_verify_key(path, key_file.to_str(), false, true)
                .expect("--key-file must verify and take precedence over env");
        }

        // 2) --key-stdin (with trailing newline and ignored second line)
        let stdin_payload = format!("{right_key}\nignored-extra-line\n");
        verify_key_with_reader(
            path,
            None,
            Some(std::io::Cursor::new(stdin_payload.into_bytes())),
            true,
        )
        .expect("--key-stdin must verify");

        // 3) VANTADB_ENCRYPTION_KEY fallback
        {
            let _right_env = EnvVarGuard::set("VANTADB_ENCRYPTION_KEY", &right_key);
            cmd_encryption_verify_key(path, None, false, true)
                .expect("VANTADB_ENCRYPTION_KEY fallback must verify");
        }

        // 4) VANTADB_ENCRYPTION_KEY_FILE fallback
        {
            let _right_file_env = EnvVarGuard::set(
                "VANTADB_ENCRYPTION_KEY_FILE",
                key_file.to_str().expect("utf-8 path"),
            );
            cmd_encryption_verify_key(path, None, false, false)
                .expect("VANTADB_ENCRYPTION_KEY_FILE fallback must verify");
        }

        let after_bytes = std::fs::read(&registry_path).expect("registry bytes after");
        let after_mtime = std::fs::metadata(&registry_path)
            .expect("registry metadata after")
            .modified()
            .expect("registry mtime after");
        assert_eq!(after_bytes, before_bytes, "registry bytes must not change");
        assert_eq!(after_mtime, before_mtime, "registry mtime must not change");
        assert!(
            !lock_path.exists(),
            "verify-key must never create or take .vanta.lock"
        );
    }

    /// FIND-646: wrong key returns `Error::EncryptionKeyRejected` with the
    /// exact same Display text as `StorageEngine::open_with_config`, without
    /// modifying `encryption.registry`.
    #[test]
    fn verify_key_wrong_key_returns_encryption_key_rejected_matching_open() {
        let _env_guard = ENV_LOCK.lock().expect("env lock");
        let _clear_env = EnvVarGuard::remove("VANTADB_ENCRYPTION_KEY");
        let _clear_file = EnvVarGuard::remove("VANTADB_ENCRYPTION_KEY_FILE");
        let _clear_cmd = EnvVarGuard::remove("VANTADB_ENCRYPTION_KEY_CMD");

        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().to_str().expect("path");
        let right_key = key_hex(0x63);
        let wrong_key = key_hex(0x64);

        {
            let engine =
                StorageEngine::open_with_config(path, Some(keyed_config(path, &right_key)))
                    .expect("open encrypted db");
            engine.flush().expect("flush");
        }

        let registry_path = dir.path().join("data").join("encryption.registry");
        let before_bytes = std::fs::read(&registry_path).expect("registry bytes");
        let before_mtime = std::fs::metadata(&registry_path)
            .expect("registry metadata")
            .modified()
            .expect("registry mtime");

        let wrong_file = write_restricted_key_file(dir.path(), "wrong.key", &wrong_key);
        let file_err = cmd_encryption_verify_key(path, wrong_file.to_str(), false, true)
            .expect_err("wrong --key-file must be rejected");
        let stdin_err = verify_key_with_reader(
            path,
            None,
            Some(std::io::Cursor::new(format!("{wrong_key}\n").into_bytes())),
            true,
        )
        .expect_err("wrong --key-stdin must be rejected");

        assert!(
            matches!(file_err, Error::EncryptionKeyRejected { .. }),
            "expected EncryptionKeyRejected, got: {file_err}"
        );
        assert!(
            matches!(stdin_err, Error::EncryptionKeyRejected { .. }),
            "expected EncryptionKeyRejected, got: {stdin_err}"
        );

        let after_bytes = std::fs::read(&registry_path).expect("registry bytes after");
        let after_mtime = std::fs::metadata(&registry_path)
            .expect("registry metadata after")
            .modified()
            .expect("registry mtime after");
        assert_eq!(after_bytes, before_bytes, "registry bytes must stay intact");
        assert_eq!(after_mtime, before_mtime, "registry mtime must stay intact");

        let open_err =
            match StorageEngine::open_with_config(path, Some(keyed_config(path, &wrong_key))) {
                Err(e) => e,
                Ok(_) => panic!("wrong key must not open engine"),
            };
        let expected_db_err =
            crate::crypto::map_database_crypto_error(crate::crypto::CryptoError::DecryptionFailed);
        assert!(
            matches!(file_err, Error::EncryptionKeyRejected { ref artifact } if artifact == "database"),
            "expected database EncryptionKeyRejected, got: {file_err}"
        );
        assert!(
            matches!(open_err, Error::EncryptionKeyRejected { .. }),
            "expected open EncryptionKeyRejected, got: {open_err}"
        );
        assert_eq!(
            file_err.to_string(),
            expected_db_err.to_string(),
            "verify-key wrong-key error text must match init::crypto_error"
        );
        assert_eq!(stdin_err.to_string(), file_err.to_string());
        assert!(
            file_err
                .to_string()
                .contains("(wrong key or tampered data)"),
            "error must not distinguish wrong key from tampering: {file_err}"
        );
        assert!(
            open_err
                .to_string()
                .contains("(wrong key or tampered data)"),
            "open error must match same template: {open_err}"
        );
    }

    /// FIND-646: corrupted `encryption.registry` maps to `Error::EncryptionCorrupt`
    /// without rewriting or modifying the corrupted file on disk.
    #[test]
    fn verify_key_corrupt_registry_returns_encryption_corrupt_without_modifying_disk() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().to_str().expect("path");
        let key = key_hex(0x67);
        {
            let engine = StorageEngine::open_with_config(path, Some(keyed_config(path, &key)))
                .expect("open encrypted db");
            engine.flush().expect("flush");
        }
        let registry_path = dir.path().join("data").join("encryption.registry");
        let corrupt = b"CORRUPT_REGISTRY_HEADER";
        std::fs::write(&registry_path, corrupt).expect("corrupt registry");
        let before_mtime = std::fs::metadata(&registry_path)
            .expect("meta")
            .modified()
            .expect("mtime");

        let key_file = write_restricted_key_file(dir.path(), "valid.key", &key);
        let err = cmd_encryption_verify_key(path, key_file.to_str(), false, true)
            .expect_err("corrupt registry must fail");
        assert!(
            matches!(err, Error::EncryptionCorrupt { ref artifact, .. } if artifact == "database"),
            "expected EncryptionCorrupt, got: {err}"
        );
        assert_eq!(
            std::fs::read(&registry_path).expect("bytes"),
            corrupt,
            "corrupt registry bytes must stay untouched"
        );
        assert_eq!(
            std::fs::metadata(&registry_path)
                .expect("meta after")
                .modified()
                .expect("mtime after"),
            before_mtime
        );
    }

    /// FIND-646: database without `encryption.registry` fails with an explicit
    /// error stating that no encryption registry exists.
    #[test]
    fn verify_key_fails_explicitly_when_registry_missing() {
        let dir = tempfile::tempdir().expect("tempdir");
        let plain_path = dir.path().join("plaintext_db");
        let plain_str = plain_path.to_str().expect("path");
        {
            let cfg = Config {
                storage_path: plain_str.to_string(),
                backend_kind: disk_backend(),
                encryption_key: None,
                ..Config::default()
            };
            let engine =
                StorageEngine::open_with_config(plain_str, Some(cfg)).expect("open plaintext db");
            engine.flush().expect("flush");
        }
        let key_file = write_restricted_key_file(dir.path(), "some.key", &key_hex(0x65));

        let err = cmd_encryption_verify_key(plain_str, key_file.to_str(), false, true)
            .expect_err("plaintext database without registry must fail");
        assert!(
            err.to_string().contains("no encryption registry"),
            "unexpected error for plaintext db: {err}"
        );

        let missing_dir = dir.path().join("does_not_exist");
        let missing_err = cmd_encryption_verify_key(
            missing_dir.to_str().expect("path"),
            key_file.to_str(),
            false,
            true,
        )
        .expect_err("non-existent database without registry must fail");
        assert!(
            missing_err.to_string().contains("no encryption registry"),
            "unexpected error for missing db: {missing_err}"
        );
    }

    /// FIND-646: `verify-key` succeeds even while another OS process owns
    /// `.vanta.lock` exclusively (`StorageEngine` open on the database).
    #[test]
    fn verify_key_succeeds_while_another_process_holds_vanta_lock() {
        if let Ok(lock_db) = std::env::var("VANTADB_VERIFY_KEY_LOCK_HOLDER_DB") {
            let lock_key =
                std::env::var("VANTADB_VERIFY_KEY_LOCK_HOLDER_KEY").expect("child lock holder key");
            let _engine =
                StorageEngine::open_with_config(&lock_db, Some(keyed_config(&lock_db, &lock_key)))
                    .expect("child open holding .vanta.lock");
            use std::io::{Read as _, Write as _};
            println!("LOCK_READY");
            std::io::stdout().flush().expect("flush LOCK_READY");
            let mut sink = Vec::new();
            let _ = std::io::stdin().read_to_end(&mut sink);
            return;
        }

        struct ChildGuard(Option<std::process::Child>);
        impl Drop for ChildGuard {
            fn drop(&mut self) {
                if let Some(mut child) = self.0.take() {
                    drop(child.stdin.take());
                    let _ = child.kill();
                    let _ = child.wait();
                }
            }
        }

        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().to_str().expect("path");
        let key = key_hex(0x66);

        {
            let engine = StorageEngine::open_with_config(path, Some(keyed_config(path, &key)))
                .expect("create encrypted db");
            engine.flush().expect("flush");
        }

        let registry_path = dir.path().join("data").join("encryption.registry");

        let mod_rel = module_path!()
            .split_once("::")
            .map(|(_, rest)| rest)
            .expect("module path");
        let test_name =
            format!("{mod_rel}::verify_key_succeeds_while_another_process_holds_vanta_lock");

        let exe = std::env::current_exe().expect("current test exe");
        let mut child = std::process::Command::new(exe)
            .args(["--exact", &test_name, "--nocapture"])
            .env("VANTADB_VERIFY_KEY_LOCK_HOLDER_DB", path)
            .env("VANTADB_VERIFY_KEY_LOCK_HOLDER_KEY", &key)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("spawn lock-holder child");

        let stdout = child.stdout.take().expect("child stdout");
        let mut guard = ChildGuard(Some(child));

        use std::io::BufRead as _;
        let mut reader = std::io::BufReader::new(stdout);
        let mut ready = false;
        let mut line = String::new();
        while reader.read_line(&mut line).expect("read child stdout") > 0 {
            if line.trim() == "LOCK_READY" {
                ready = true;
                break;
            }
            line.clear();
        }
        assert!(ready, "child process did not emit LOCK_READY");

        // Confirm `.vanta.lock` is genuinely held exclusively by the child OS process.
        #[cfg(feature = "fs2")]
        {
            let busy = acquire_rotation_lock(dir.path())
                .expect_err("another OS process must hold .vanta.lock exclusively");
            assert!(
                matches!(busy, Error::DatabaseBusy(_)),
                "expected DatabaseBusy from child lock holder, got: {busy}"
            );
        }

        // Snapshot bytes & mtime AFTER child engine startup completed so we isolate
        // `verify-key` itself from the child's initial nonce reservation.
        let before_bytes = std::fs::read(&registry_path).expect("registry bytes");
        let before_mtime = std::fs::metadata(&registry_path)
            .expect("registry metadata")
            .modified()
            .expect("registry mtime");

        let key_file = write_restricted_key_file(dir.path(), "check.key", &key);
        cmd_encryption_verify_key(path, key_file.to_str(), false, true)
            .expect("verify-key must succeed while another process holds .vanta.lock");

        let after_bytes = std::fs::read(&registry_path).expect("registry bytes after");
        let after_mtime = std::fs::metadata(&registry_path)
            .expect("registry metadata after")
            .modified()
            .expect("registry mtime after");
        assert_eq!(after_bytes, before_bytes, "registry bytes must stay intact");
        assert_eq!(after_mtime, before_mtime, "registry mtime must stay intact");

        let mut child = guard.0.take().expect("child handle");
        drop(child.stdin.take());
        let status = child.wait().expect("wait child");
        assert!(status.success(), "lock-holder child exited with {status}");
    }
}
