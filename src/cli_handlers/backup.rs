//! Backup command handlers — backup and restore.

use console::Term;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use web_time::{SystemTime, UNIX_EPOCH};

use crate::cli_handlers::diagnostics::Verbosity;
use crate::cli_handlers::{
    create_spinner, dir_size, human_readable_size, open_database, open_embedded, print_info,
    print_json, print_success, print_warning,
};
use crate::error::{ChainedError, Result};

// ── MANIFEST types ─────────────────────────────────────────────────────────

/// Backup type recorded in MANIFEST.json.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum BackupType {
    Base,
    Incremental,
}

/// Per-file entry in MANIFEST.json.
#[derive(Debug, Serialize, Deserialize)]
struct ManifestFile {
    name: String,
    size: u64,
    /// Hex-encoded CRC32C of the file contents.
    crc32c: String,
}

/// The encryption facts of a backup (FIND-507).
///
/// A backup of an encrypted database carries the **key registry** (it lives in
/// the data directory) but **never** the master key, which lives in the
/// environment of the process (ADR-0057 D2). Declaring it in the manifest is
/// what turns a later `EncryptionKeyRequired` into an explainable restore.
#[derive(Debug, Serialize, Deserialize)]
struct BackupEncryption {
    /// Path of the key registry, relative to the backup root.
    registry: String,
    /// Always `false`: the master key does not travel with the backup.
    master_key_included: bool,
    /// Where the operator reads how to provide the key (and what happens if
    /// the registry is lost).
    runbook: String,
}

/// File name of the key registry.
///
/// The literal of `crypto::registry::REGISTRY_FILE_NAME` (that module is
/// feature-gated; the backup path is not: it must recognise an encrypted
/// backup even in a build without the `encryption` feature).
const REGISTRY_FILE_NAME: &str = "encryption.registry";

/// The runbook that explains the master key and the registry loss.
const KEY_RUNBOOK: &str = "docs/user/operations/security/KEY_RUNBOOK.md";

/// Locate the key registry inside `dir` (recursively) and describe it.
///
/// `None` = the backup is plaintext (no registry anywhere in the tree).
///
/// P2-01 M5: the walk error is **propagated**, not swallowed into "plaintext"
/// — a fail-open here would skip both the manifest declaration and the restore
/// guard. It also reuses the manifest entries the caller already collected
/// (`files`), so the tree is walked once.
fn encryption_block(files: &[ManifestFile]) -> Option<BackupEncryption> {
    let registry = files
        .iter()
        .find(|f| {
            f.name
                .rsplit('/')
                .next()
                .is_some_and(|last| last == REGISTRY_FILE_NAME)
        })?
        .name
        .clone();
    Some(BackupEncryption {
        registry,
        master_key_included: false,
        runbook: KEY_RUNBOOK.to_string(),
    })
}

/// The manifest entries of `dir`, or the error if the tree cannot be walked.
///
/// A backup whose file list cannot be collected is not a backup: `cmd_backup`
/// aborts instead of writing a manifest that under-declares what it copied.
fn manifest_files(dir: &Path) -> Result<Vec<ManifestFile>> {
    collect_manifest_files(dir, dir).map_err(|e| {
        crate::error::Error::backup_error(format!(
            "Failed to list the copied files (the manifest would be incomplete): {e}"
        ))
    })
}

/// Refuse to restore an **encrypted** backup when no master key resolves.
///
/// Copying the files is not the problem: the database will simply not open
/// afterwards, and the operator would meet a bare `EncryptionKeyRequired` in
/// whatever command they run next. The error names the three key sources and
/// the runbook (FIND-507).
#[cfg(feature = "encryption")]
fn require_master_key_for_encrypted_backup(src: &Path) -> Result<()> {
    let Some(block) = encryption_block(&manifest_files(src)?) else {
        return Ok(());
    };
    let config = crate::config::Config::default();
    let env_key = config
        .encryption_key
        .clone()
        .or_else(|| std::env::var("VANTADB_ENCRYPTION_KEY").ok());
    match crate::crypto::resolve_master_key_material(
        env_key.as_deref(),
        config.encryption_key_file.as_deref(),
        config.encryption_key_cmd.as_deref(),
    ) {
        Ok(_) => Ok(()),
        Err(_) => Err(crate::error::Error::restore_error(format!(
            "the backup '{}' is ENCRYPTED (it carries '{}') but no master key resolved. \
             The master key never travels with a backup: provide it before restoring with \
             VANTADB_ENCRYPTION_KEY (hex), VANTADB_ENCRYPTION_KEY_FILE or \
             VANTADB_ENCRYPTION_KEY_CMD, or the restored database will not open. \
             Losing the registry or the key loses the database — see {KEY_RUNBOOK}.",
            src.display(),
            block.registry
        ))),
    }
}

/// No-op without the `encryption` feature: such a build cannot open an
/// encrypted database anyway, and the restore itself is a plain file copy.
#[cfg(not(feature = "encryption"))]
fn require_master_key_for_encrypted_backup(_src: &Path) -> Result<()> {
    Ok(())
}

/// MANIFEST.json written alongside every backup.
///
/// Provides enough metadata to validate backup integrity, chain incrementals
/// to their base, and target point-in-time restores via LSN in the future.
#[derive(Debug, Serialize, Deserialize)]
struct BackupManifest {
    /// "base" or "incremental"
    backup_type: BackupType,
    /// RFC 3339 timestamp at which the backup was created.
    created_at: String,
    /// VantaDB crate version.
    vantadb_version: String,
    /// Relative path to the base backup (null for base backups).
    base_ref: Option<String>,
    /// Encryption facts (null/absent for a plaintext database). Absent on
    /// backups written before FIND-507, hence `serde(default)`.
    #[serde(default)]
    encryption: Option<BackupEncryption>,
    /// Files included in this backup with integrity data.
    files: Vec<ManifestFile>,
}

/// Compute CRC32C of a file and return its hex string.
fn file_crc32c(path: &Path) -> std::io::Result<String> {
    use std::io::Read;
    let mut f = std::fs::File::open(path)?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    let checksum = crate::wal::compute_crc32c(&buf);
    Ok(format!("{:08x}", checksum))
}

/// Collect all regular files under `dir` recursively, returning relative
/// paths and their manifest entries.
fn collect_manifest_files(dir: &Path, base: &Path) -> std::io::Result<Vec<ManifestFile>> {
    let mut files = Vec::new();
    for entry in walkdir_flat(dir)? {
        let relative = entry
            .strip_prefix(base)
            .unwrap_or(&entry)
            .to_string_lossy()
            .replace('\\', "/");
        let meta = std::fs::metadata(&entry)?;
        let crc = file_crc32c(&entry).unwrap_or_else(|_| "error".to_string());
        files.push(ManifestFile {
            name: relative,
            size: meta.len(),
            crc32c: crc,
        });
    }
    files.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(files)
}

/// Flat recursive directory walker returning regular file paths.
fn walkdir_flat(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut result = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            result.extend(walkdir_flat(&path)?);
        } else {
            result.push(path);
        }
    }
    Ok(result)
}

/// Write `MANIFEST.json` to `dir`.
fn write_manifest(dir: &Path, manifest: &BackupManifest) -> Result<()> {
    let json = serde_json::to_string_pretty(manifest).map_err(|e| {
        crate::error::Error::backup_error(format!("Failed to serialize MANIFEST: {e}"))
    })?;
    let path = dir.join("MANIFEST.json");
    std::fs::write(&path, json).map_err(|e| {
        crate::error::Error::backup_error(format!("Failed to write MANIFEST.json: {e}"))
    })
}

fn copy_dir(src: &Path, dst: &Path, skip: Option<&Path>) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        if skip.is_some_and(|s| src_path == s) {
            continue;
        }
        let ft = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());
        if ft.is_dir() {
            copy_dir(&src_path, &dst_path, skip)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

#[tracing::instrument]
/// Create a filesystem-level backup of the database directory
pub fn cmd_backup(
    db_path: &str,
    out: Option<&str>,
    verbose: bool,
    json_output: bool,
) -> Result<()> {
    let src = std::path::Path::new(db_path);
    if !src.exists() {
        if json_output {
            return print_json(&serde_json::json!({
                "source": db_path,
                "backup_dir": serde_json::Value::Null,
            }));
        }
        print_warning(&format!(
            "Database directory does not exist at '{}'",
            db_path
        ));
        return Ok(());
    }

    let backup_dir = match out {
        Some(p) => PathBuf::from(p),
        None => {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
            let dir = format!("vantadb_backups/backup_{}", timestamp);
            PathBuf::from(dir)
        }
    };

    if backup_dir.join("vantadb.dat").exists() || backup_dir.join("vantadb.wal").exists() {
        return Err(crate::error::Error::Cli(ChainedError::msg(format!(
            "Backup destination '{}' already contains database files. Choose a different location or remove existing files.",
            backup_dir.display()
        ))));
    }

    // Open writable to flush, then drop before copying files
    {
        let spinner = create_spinner("Opening database...");
        let engine = open_database(db_path, false)?;
        spinner.set_message("Flushing database...");
        engine.flush()?;
    }

    copy_dir(src, &backup_dir, Some(&backup_dir)).map_err(|e| {
        crate::error::Error::backup_error(format!("Failed to copy database to backup: {e}"))
    })?;

    // Generate MANIFEST.json alongside the backup files.
    let spinner = create_spinner("Generating MANIFEST.json...");
    let now_ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Format as a simple RFC 3339-ish UTC string without pulling chrono.
    let created_at = format!(
        "{}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        1970 + now_ts / 31_536_000, // approximate — good enough for a label
        1,
        1,
        (now_ts % 86400) / 3600,
        (now_ts % 3600) / 60,
        now_ts % 60
    );
    let files = manifest_files(&backup_dir)?
        .into_iter()
        // Skip MANIFEST.json itself to avoid a circular reference.
        .filter(|f| f.name != "MANIFEST.json")
        .collect::<Vec<_>>();
    let manifest = BackupManifest {
        backup_type: BackupType::Base,
        created_at,
        vantadb_version: env!("CARGO_PKG_VERSION").to_string(),
        base_ref: None,
        // P2-01 M5: from the **already collected** entries — no second walk,
        // and no `.ok()` that would classify an unreadable tree as plaintext.
        encryption: encryption_block(&files),
        files,
    };
    // Non-fatal: log a warning but don't abort the backup on manifest failure.
    if let Err(e) = write_manifest(&backup_dir, &manifest) {
        tracing::warn!(error = %e, "Failed to write MANIFEST.json; backup data is intact");
    }
    spinner.finish_and_clear();

    if json_output {
        return print_json(&serde_json::json!({
            "source": db_path,
            "backup_dir": backup_dir.display().to_string(),
            "files": manifest.files.len(),
            "vantadb_version": manifest.vantadb_version,
            "encrypted": manifest.encryption.is_some(),
            "encryption_registry": manifest.encryption.as_ref().map(|e| e.registry.clone()),
        }));
    }

    let _ = Term::stdout().write_line("");
    print_success(&format!("Backup created at: {}", backup_dir.display()));

    if let Some(block) = &manifest.encryption {
        print_warning(&format!(
            "This is an ENCRYPTED database backup: it carries '{}' but NOT the master key. \
             Keep the key separately and provide it when restoring — losing it (or the \
             registry) loses the database. See {KEY_RUNBOOK}.",
            block.registry
        ));
    }

    if verbose {
        print_info(&format!(
            "Source: {}",
            src.canonicalize()
                .unwrap_or_else(|_| src.to_path_buf())
                .display()
        ));
        print_info(&format!(
            "Size: {}",
            human_readable_size(dir_size(src).unwrap_or(0) as u64)
        ));
    }

    Ok(())
}

/// Overwrite policy for the restore target (D2: replaces `force: bool`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OverwritePolicy {
    /// Fail when the target exists (default, safe).
    #[default]
    FailIfExists,
    /// Remove and recreate the target (`--force`).
    Overwrite,
}

/// Index rebuild after restore (D2: replaces `rebuild: bool`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IndexRebuild {
    #[default]
    No,
    Yes,
}

/// Execution mode for restore (D2: replaces `dry_run: bool`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RestoreMode {
    #[default]
    Apply,
    DryRun,
}

/// Options for [`cmd_restore`] (D2: replaces `force`/`rebuild`/`dry_run`/`verbose` bool flags).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RestoreOptions {
    pub overwrite: OverwritePolicy,
    pub rebuild: IndexRebuild,
    pub mode: RestoreMode,
    pub verbose: Verbosity,
}

#[tracing::instrument]
/// Validate a backup without restoring it (dry-run).
///
/// Read-only: checks the backup input (exists, is a directory, non-empty,
/// `MANIFEST.json` parses when present), reports total size, lists the files
/// that would be restored, and reports target conflicts — without touching
/// the target (no `create_dir_all`, no `remove_dir_all`, no copy, no open).
fn cmd_restore_dry_run(
    db_path: &str,
    input: &str,
    opts: RestoreOptions,
    json_output: bool,
) -> Result<()> {
    let src = std::path::Path::new(input);
    if !src.is_dir() {
        return Err(crate::error::Error::restore_error(format!(
            "Backup path is not a directory: '{input}'"
        )));
    }
    let mut files = walkdir_flat(src).map_err(|e| {
        crate::error::Error::restore_error(format!("Failed to list backup files: {e}"))
    })?;
    if files.is_empty() {
        return Err(crate::error::Error::restore_error(format!(
            "Backup directory is empty or invalid: '{input}'"
        )));
    }
    files.sort();
    let total = dir_size(src).unwrap_or(0) as u64;

    // Format check: MANIFEST.json must parse when present (light variant of
    // the runbook §3 check). Absence is a warning (legacy backup), not an error.
    let manifest_path = src.join("MANIFEST.json");
    let mut manifest_json = serde_json::Value::Null;
    if manifest_path.exists() {
        let raw = std::fs::read_to_string(&manifest_path).map_err(|e| {
            crate::error::Error::restore_error(format!("Failed to read backup MANIFEST.json: {e}"))
        })?;
        let manifest: BackupManifest = serde_json::from_str(&raw).map_err(|e| {
            crate::error::Error::restore_error(format!("Invalid backup MANIFEST.json: {e}"))
        })?;
        let kind = match manifest.backup_type {
            BackupType::Base => "base",
            BackupType::Incremental => "incremental",
        };
        manifest_json = serde_json::json!({
            "type": kind,
            "version": manifest.vantadb_version,
            "files": manifest.files.len(),
        });
        if !json_output {
            print_info(&format!(
                "Backup MANIFEST: type={kind} version={} files={}",
                manifest.vantadb_version,
                manifest.files.len()
            ));
        }
    } else if !json_output {
        print_warning("No MANIFEST.json found (legacy backup?) — proceeding with file listing");
    }

    let dst = std::path::Path::new(db_path);

    // FIND-507: an encrypted backup needs the master key at restore time; the
    // dry-run says so before the operator commits to a target.
    let encryption = manifest_files(src)
        .ok()
        .and_then(|files| encryption_block(&files));
    if let Some(block) = &encryption {
        if !json_output {
            print_warning(&format!(
                "This backup is ENCRYPTED (registry: '{}'): restoring needs the master key \
                 (VANTADB_ENCRYPTION_KEY / _FILE / _CMD). The key never travels with the \
                 backup. See {KEY_RUNBOOK}.",
                block.registry
            ));
        }
    }

    if json_output {
        return print_json(&serde_json::json!({
            "mode": "dry_run",
            "backup_dir": input,
            "target": db_path,
            "files": files.len(),
            "total_bytes": total,
            "would_overwrite": dst.exists(),
            "rebuild": matches!(opts.rebuild, IndexRebuild::Yes),
            "manifest": manifest_json,
            "encryption_registry": encryption.as_ref().map(|b| b.registry.clone()),
        }));
    }

    if dst.exists() {
        if matches!(opts.overwrite, OverwritePolicy::Overwrite) {
            print_warning(&format!(
                "Destination '{db_path}' exists — would remove and recreate (--force) (dry-run: no changes made)"
            ));
        } else {
            print_warning(&format!(
                "Destination '{db_path}' already exists — would require --force to overwrite (dry-run: no changes made)"
            ));
        }
    } else {
        print_info(&format!(
            "Destination '{db_path}' does not exist — would create it (dry-run: no changes made)"
        ));
    }

    for path in &files {
        let rel = path
            .strip_prefix(src)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        print_info(&format!(
            "Would restore: {rel} ({})",
            human_readable_size(size)
        ));
    }
    print_success(&format!(
        "Dry-run: would restore {} files ({}) from '{}' to '{db_path}'",
        files.len(),
        human_readable_size(total),
        src.display()
    ));
    if matches!(opts.rebuild, IndexRebuild::Yes) {
        print_info("Would rebuild indexes after restore (--rebuild)");
    }
    print_info("dry-run: re-run without `--dry-run` to apply");
    Ok(())
}

#[tracing::instrument]
/// Restore the database from a previously created backup directory
pub fn cmd_restore(
    db_path: &str,
    input: &str,
    opts: RestoreOptions,
    json_output: bool,
) -> Result<()> {
    let src = std::path::Path::new(input);
    if !src.exists() {
        return Err(crate::error::Error::restore_error(format!(
            "Backup directory does not exist at '{}'",
            input
        )));
    }

    if matches!(opts.mode, RestoreMode::DryRun) {
        return cmd_restore_dry_run(db_path, input, opts, json_output);
    }

    let dst = std::path::Path::new(db_path);

    if dst.exists() && !matches!(opts.overwrite, OverwritePolicy::Overwrite) {
        return Err(crate::error::Error::restore_error(
            "Destination database directory already exists. Use --force to overwrite.",
        ));
    }

    // FIND-507: an encrypted backup needs its master key **before** the files
    // are put in place — restoring blind leaves a database that will not open.
    require_master_key_for_encrypted_backup(src)?;

    let spinner = create_spinner("Restoring from backup...");

    if dst.exists() && matches!(opts.overwrite, OverwritePolicy::Overwrite) {
        std::fs::remove_dir_all(dst).map_err(|e| {
            crate::error::Error::restore_error(format!(
                "Failed to remove existing database directory: {e}"
            ))
        })?;
    }

    std::fs::create_dir_all(dst).map_err(|e| {
        crate::error::Error::restore_error(format!("Failed to create database directory: {e}"))
    })?;

    copy_dir(src, dst, None).map_err(|e| {
        crate::error::Error::restore_error(format!("Failed to restore from backup: {e}"))
    })?;

    spinner.set_message("Verifying restored database...");

    if matches!(opts.rebuild, IndexRebuild::Yes) {
        spinner.set_message("Rebuilding indexes...");
        let db = open_embedded(db_path, false)?;
        db.rebuild_index().map_err(|e| {
            crate::error::Error::restore_error(format!("Index rebuild after restore failed: {e}"))
        })?;
    }

    spinner.finish_and_clear();

    if json_output {
        return print_json(&serde_json::json!({
            "mode": "applied",
            "target": db_path,
            "restored_from": src
                .canonicalize()
                .unwrap_or_else(|_| src.to_path_buf())
                .display()
                .to_string(),
            "rebuild": matches!(opts.rebuild, IndexRebuild::Yes),
        }));
    }

    print_success(&format!(
        "Database restored from: {}",
        src.canonicalize()
            .unwrap_or_else(|_| src.to_path_buf())
            .display()
    ));

    if opts.verbose.is_verbose() {
        let src_size = dir_size(src).unwrap_or(0) as u64;
        let dst_size = dir_size(dst).unwrap_or(0) as u64;
        print_info(&format!("Backup size: {}", human_readable_size(src_size)));
        print_info(&format!("Restored size: {}", human_readable_size(dst_size)));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A backup tree that carries a key registry (the encrypted shape).
    fn write_registry_backup(dir: &Path) {
        std::fs::create_dir_all(dir.join("data")).unwrap();
        std::fs::write(
            dir.join("data").join(REGISTRY_FILE_NAME),
            b"VANTAREG-not-a-real-registry",
        )
        .unwrap();
    }

    /// FIND-507: the manifest declares the encryption facts (and says that the
    /// master key is **not** in the backup), and a plaintext backup declares
    /// nothing at all.
    #[test]
    fn manifest_declares_the_encryption_registry_when_present() {
        let dir = tempfile::tempdir().unwrap();

        let plain = dir.path().join("plain");
        std::fs::create_dir_all(&plain).unwrap();
        std::fs::write(plain.join("vanta.wal"), b"wal").unwrap();
        let plain_files = manifest_files(&plain).expect("list the backup tree");
        assert!(
            encryption_block(&plain_files).is_none(),
            "a plaintext backup must declare no encryption block"
        );

        let encrypted = dir.path().join("encrypted");
        write_registry_backup(&encrypted);
        let files = manifest_files(&encrypted).expect("list the backup tree");
        let block = encryption_block(&files).expect("the registry must be found");
        assert_eq!(block.registry, "data/encryption.registry");
        assert!(
            !block.master_key_included,
            "the master key never travels with the backup"
        );
        assert_eq!(block.runbook, KEY_RUNBOOK);
    }

    /// FIND-507: restoring an encrypted backup without a master key fails
    /// **before** touching the target, naming the key sources — not with a bare
    /// `EncryptionKeyRequired` in whatever command runs next.
    #[cfg(feature = "encryption")]
    #[test]
    fn restore_of_an_encrypted_backup_without_a_key_explains_the_requirement() {
        const VARS: [&str; 3] = [
            "VANTADB_ENCRYPTION_KEY",
            "VANTADB_ENCRYPTION_KEY_FILE",
            "VANTADB_ENCRYPTION_KEY_CMD",
        ];
        for var in VARS {
            std::env::remove_var(var);
        }

        let dir = tempfile::tempdir().unwrap();
        let backup = dir.path().join("backup");
        write_registry_backup(&backup);
        let target = dir.path().join("restored");

        let err = cmd_restore(
            target.to_str().unwrap(),
            backup.to_str().unwrap(),
            RestoreOptions::default(),
            true,
        )
        .expect_err("an encrypted backup without a key must not restore");
        let message = err.to_string();
        assert!(
            message.contains("master key"),
            "the error must name the missing master key, got: {message}"
        );
        assert!(
            message.contains("VANTADB_ENCRYPTION_KEY_FILE"),
            "the error must name the key sources, got: {message}"
        );
        assert!(
            !target.exists(),
            "the target must not be created without the key"
        );

        // With a key the restore proceeds: it is a file copy, the crypto
        // happens when the restored database is opened.
        std::env::set_var("VANTADB_ENCRYPTION_KEY", "ab".repeat(32));
        let restored = cmd_restore(
            target.to_str().unwrap(),
            backup.to_str().unwrap(),
            RestoreOptions::default(),
            true,
        );
        let outcome = restored;
        std::env::remove_var("VANTADB_ENCRYPTION_KEY");
        outcome.expect("a restore with a key must succeed");
        assert!(
            target.join("data").join(REGISTRY_FILE_NAME).exists(),
            "the registry must be restored"
        );
    }
}
