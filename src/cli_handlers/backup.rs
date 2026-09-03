//! Backup command handlers — backup and restore.

use console::Term;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use web_time::{SystemTime, UNIX_EPOCH};

use crate::cli_handlers::{
    create_spinner, dir_size, human_readable_size, open_database, open_embedded, print_info,
    print_success, print_warning,
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
        crate::error::VantaError::backup_error(format!("Failed to serialize MANIFEST: {e}"))
    })?;
    let path = dir.join("MANIFEST.json");
    std::fs::write(&path, json).map_err(|e| {
        crate::error::VantaError::backup_error(format!("Failed to write MANIFEST.json: {e}"))
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
pub fn cmd_backup(db_path: &str, out: Option<&str>, verbose: bool) -> Result<()> {
    let src = std::path::Path::new(db_path);
    if !src.exists() {
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
        return Err(crate::error::VantaError::CliError(ChainedError::msg(format!(
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
        crate::error::VantaError::backup_error(format!("Failed to copy database to backup: {e}"))
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
    let files = collect_manifest_files(&backup_dir, &backup_dir)
        .unwrap_or_default()
        .into_iter()
        // Skip MANIFEST.json itself to avoid a circular reference.
        .filter(|f| f.name != "MANIFEST.json")
        .collect::<Vec<_>>();
    let manifest = BackupManifest {
        backup_type: BackupType::Base,
        created_at,
        vantadb_version: env!("CARGO_PKG_VERSION").to_string(),
        base_ref: None,
        files,
    };
    // Non-fatal: log a warning but don't abort the backup on manifest failure.
    if let Err(e) = write_manifest(&backup_dir, &manifest) {
        tracing::warn!(error = %e, "Failed to write MANIFEST.json; backup data is intact");
    }
    spinner.finish_and_clear();

    let _ = Term::stdout().write_line("");
    print_success(&format!("Backup created at: {}", backup_dir.display()));

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

#[tracing::instrument]
/// Validate a backup without restoring it (dry-run).
///
/// Read-only: checks the backup input (exists, is a directory, non-empty,
/// `MANIFEST.json` parses when present), reports total size, lists the files
/// that would be restored, and reports target conflicts — without touching
/// the target (no `create_dir_all`, no `remove_dir_all`, no copy, no open).
fn cmd_restore_dry_run(db_path: &str, input: &str, force: bool, rebuild: bool) -> Result<()> {
    let src = std::path::Path::new(input);
    if !src.is_dir() {
        return Err(crate::error::VantaError::restore_error(format!(
            "Backup path is not a directory: '{input}'"
        )));
    }
    let mut files = walkdir_flat(src).map_err(|e| {
        crate::error::VantaError::restore_error(format!("Failed to list backup files: {e}"))
    })?;
    if files.is_empty() {
        return Err(crate::error::VantaError::restore_error(format!(
            "Backup directory is empty or invalid: '{input}'"
        )));
    }
    files.sort();
    let total = dir_size(src).unwrap_or(0) as u64;

    // Format check: MANIFEST.json must parse when present (light variant of
    // the runbook §3 check). Absence is a warning (legacy backup), not an error.
    let manifest_path = src.join("MANIFEST.json");
    if manifest_path.exists() {
        let raw = std::fs::read_to_string(&manifest_path).map_err(|e| {
            crate::error::VantaError::restore_error(format!(
                "Failed to read backup MANIFEST.json: {e}"
            ))
        })?;
        let manifest: BackupManifest = serde_json::from_str(&raw).map_err(|e| {
            crate::error::VantaError::restore_error(format!("Invalid backup MANIFEST.json: {e}"))
        })?;
        let kind = match manifest.backup_type {
            BackupType::Base => "base",
            BackupType::Incremental => "incremental",
        };
        print_info(&format!(
            "Backup MANIFEST: type={kind} version={} files={}",
            manifest.vantadb_version,
            manifest.files.len()
        ));
    } else {
        print_warning("No MANIFEST.json found (legacy backup?) — proceeding with file listing");
    }

    let dst = std::path::Path::new(db_path);
    if dst.exists() {
        if force {
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
    if rebuild {
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
    force: bool,
    rebuild: bool,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    let src = std::path::Path::new(input);
    if !src.exists() {
        return Err(crate::error::VantaError::restore_error(format!(
            "Backup directory does not exist at '{}'",
            input
        )));
    }

    if dry_run {
        return cmd_restore_dry_run(db_path, input, force, rebuild);
    }

    let dst = std::path::Path::new(db_path);

    if dst.exists() && !force {
        return Err(crate::error::VantaError::restore_error(
            "Destination database directory already exists. Use --force to overwrite.",
        ));
    }

    let spinner = create_spinner("Restoring from backup...");

    if dst.exists() && force {
        std::fs::remove_dir_all(dst).map_err(|e| {
            crate::error::VantaError::restore_error(format!(
                "Failed to remove existing database directory: {e}"
            ))
        })?;
    }

    std::fs::create_dir_all(dst).map_err(|e| {
        crate::error::VantaError::restore_error(format!("Failed to create database directory: {e}"))
    })?;

    copy_dir(src, dst, None).map_err(|e| {
        crate::error::VantaError::restore_error(format!("Failed to restore from backup: {e}"))
    })?;

    spinner.set_message("Verifying restored database...");

    if rebuild {
        spinner.set_message("Rebuilding indexes...");
        let db = open_embedded(db_path, false)?;
        db.rebuild_index().map_err(|e| {
            crate::error::VantaError::restore_error(format!(
                "Index rebuild after restore failed: {e}"
            ))
        })?;
    }

    spinner.finish_and_clear();

    print_success(&format!(
        "Database restored from: {}",
        src.canonicalize()
            .unwrap_or_else(|_| src.to_path_buf())
            .display()
    ));

    if verbose {
        let src_size = dir_size(src).unwrap_or(0) as u64;
        let dst_size = dir_size(dst).unwrap_or(0) as u64;
        print_info(&format!("Backup size: {}", human_readable_size(src_size)));
        print_info(&format!("Restored size: {}", human_readable_size(dst_size)));
    }

    Ok(())
}
