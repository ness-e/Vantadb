//! Backup command handlers — backup and restore.

use console::Term;
use std::path::{Path, PathBuf};
use web_time::{SystemTime, UNIX_EPOCH};

use crate::cli_handlers::{
    create_spinner, dir_size, human_readable_size, open_database, open_embedded, print_info,
    print_success, print_warning,
};
use crate::error::{ChainedError, Result};

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

    let spinner = create_spinner("Verifying backup...");
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
/// Restore the database from a previously created backup directory
pub fn cmd_restore(
    db_path: &str,
    input: &str,
    force: bool,
    rebuild: bool,
    verbose: bool,
) -> Result<()> {
    let src = std::path::Path::new(input);
    if !src.exists() {
        return Err(crate::error::VantaError::restore_error(format!(
            "Backup directory does not exist at '{}'",
            input
        )));
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
