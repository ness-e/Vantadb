//! Filesystem durability helpers.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

/// Append `.tmp` to the complete file name.
pub(crate) fn tmp_path_for(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(".tmp");
    PathBuf::from(name)
}

/// Write and sync a temporary file, rename it over `path`, and sync its parent on POSIX.
///
/// The live path is never opened for writing. On an error after creating the
/// temporary file, it is removed best-effort; before rename, the live path
/// retains its previous complete contents. A parent sync failure after rename
/// returns an error even though the replacement has already occurred, so callers
/// must treat it as not durable and must not advance the checkpoint.
pub(crate) fn write_atomic(
    path: &Path,
    write: impl FnOnce(&mut BufWriter<File>) -> std::io::Result<()>,
) -> std::io::Result<()> {
    let tmp = tmp_path_for(path);
    let file = File::create(&tmp)?;
    let result = (|| {
        let mut writer = BufWriter::new(file);
        write(&mut writer)?;
        writer.flush()?;
        let file = writer.into_inner().map_err(|err| err.into_error())?;
        file.sync_all()?;
        drop(file);
        #[cfg(feature = "failpoints")]
        fail::fail_point!("atomic_write_before_rename", |_| {
            Err(std::io::Error::other(
                "Injected failure before atomic rename",
            ))
        });
        std::fs::rename(&tmp, path)?;
        sync_parent_dir(path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

/// fsync a file's parent directory so a preceding `rename` becomes durable.
///
/// POSIX filesystems (ext4/XFS) do not persist a directory `rename` until the
/// parent directory itself is fsync'd; without this, a crash can revert the
/// rename (AUDREP-35).
///
/// On Windows this is deliberately a no-op: opening a directory as a `File` and
/// calling `sync_all` is not allowed (`PermissionDenied`), and NTFS does not
/// rely on a directory fsync for rename durability the way POSIX does. Only the
/// directory-fsync step is skipped here — the `rename` itself is always
/// performed by the caller and its errors are never masked.
///
/// Returns `io::Result` so a genuine durability failure on POSIX surfaces;
/// Windows always returns `Ok`.
pub(crate) fn sync_parent_dir(path: &Path) -> std::io::Result<()> {
    #[cfg(not(windows))]
    {
        // A bare relative filename (`vanta.wal`) has `Some("")` as parent —
        // `File::open("")` would fail with ENOENT. Treat it as the current
        // directory (P2-01 review DUR-01).
        let dir = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let dir_file = std::fs::File::open(dir)?;
        dir_file.sync_all()
    }
    #[cfg(windows)]
    {
        let _ = path;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_atomic_replaces_file_and_removes_tmp() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vector_index.bin");
        std::fs::write(&path, b"old").unwrap();
        write_atomic(&path, |writer| writer.write_all(b"new")).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"new");
        assert!(!tmp_path_for(&path).exists());
    }

    #[test]
    fn test_write_atomic_closure_failure_preserves_file_and_removes_tmp() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vector_index.bin");
        std::fs::write(&path, b"old").unwrap();
        let result = write_atomic(&path, |writer| {
            writer.write_all(b"partial")?;
            Err(std::io::Error::other("write failed"))
        });
        assert!(result.is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"old");
        assert!(!tmp_path_for(&path).exists());
    }

    #[cfg(feature = "failpoints")]
    #[test]
    fn test_write_atomic_failure_preserves_file_and_removes_tmp() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vector_index.bin");
        std::fs::write(&path, b"old").unwrap();
        fail::cfg("atomic_write_before_rename", "return").unwrap();
        let result = write_atomic(&path, |writer| writer.write_all(b"new"));
        fail::remove("atomic_write_before_rename");
        assert!(result.is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"old");
        assert!(!tmp_path_for(&path).exists());
    }
}
