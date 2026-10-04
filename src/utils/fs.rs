//! Filesystem durability helpers.

use std::path::Path;

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
