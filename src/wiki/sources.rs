//! Local filesystem wiki sources (MEM-29 — D36: local paths only, no network).
//!
//! Recursively discovers `.md` files under a root directory, enforcing:
//! - **Path traversal guard**: every file is canonicalized and must remain
//!   inside the canonicalized root (`starts_with`) — symlink escapes and
//!   `..`-crafted paths are rejected/skipped.
//! - **SOURCE_CHAR_BUDGET**: at most [`SOURCE_CHAR_BUDGET`] characters of
//!   source content are returned per scan (TDAM ingest-v2/index.ts:78); the
//!   file that crosses the budget is truncated to fit.
//! - Non-`.md`, unreadable or non-UTF-8 files are skipped with a trace log
//!   (pre-mortem: binary files mixed into the tree).
//!
//! The traversal itself (`collect_text_files`, `pub(crate)`) is shared with
//! the document ingestors (MEMG-08, `super::ingestors`) so the traversal guard
//! lives exactly once.

use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

/// Total character budget for scanned sources (TDAM ingest-v2/index.ts:78).
pub const SOURCE_CHAR_BUDGET: usize = 28_000;

/// One discovered markdown source.
#[derive(Debug, Clone, PartialEq)]
pub struct SourceFile {
    /// Path relative to the scan root, forward-slash separated (stable across OS).
    pub rel_path: String,
    /// File content (possibly truncated to respect [`SOURCE_CHAR_BUDGET`]).
    pub content: String,
}

/// Recursively collect `.md` sources under `root`, oldest-budget-first in
/// lexicographic path order. Errors with [`Error::InvalidInput`] when the
/// root does not exist / is not a directory.
pub fn scan_local_sources(root: &Path) -> Result<Vec<SourceFile>> {
    let mut files = Vec::new();
    let mut budget = SOURCE_CHAR_BUDGET;
    collect_text_files(root, &|ext| ext == "md", &mut |rel_path, content| {
        let char_count = content.chars().count();
        if char_count >= budget {
            // Budget exhausted: include the truncation that fits, then stop.
            files.push(SourceFile {
                rel_path,
                content: content.chars().take(budget).collect(),
            });
            budget = 0;
            return false;
        }
        budget -= char_count;
        files.push(SourceFile { rel_path, content });
        true
    })?;
    Ok(files)
}

/// Guard: `path` (already canonicalized) must stay inside canonicalized
/// `root`. Trust-boundary check against `..`/symlink escape.
fn ensure_within_root(root: &Path, path: &Path) -> bool {
    path.starts_with(root)
}

/// Recursively collect UTF-8 text files under `root` in deterministic
/// (lexicographic path) order, applying the shared traversal guard
/// ([`ensure_within_root`]).
///
/// `accept` decides by file extension (no dot, as reported by `Path::extension`)
/// whether a file is eligible; `visit` receives `(rel_path, content)` for every
/// readable, non-empty accepted file and returns `false` to stop the walk
/// (callers use this for budget exhaustion). Only a missing/invalid root is an
/// error — unreadable files, non-UTF-8 content and escaping paths are skipped
/// with a trace log (pre-mortem: binary files mixed into the tree).
///
/// This is the single traversal implementation shared by the `.md` wiki
/// scanner ([`scan_local_sources`]) and the document ingestors
/// (`super::ingestors`) — the path-traversal guard must not be duplicated.
pub(crate) fn collect_text_files(
    root: &Path,
    accept: &dyn Fn(&str) -> bool,
    visit: &mut dyn FnMut(String, String) -> bool,
) -> Result<()> {
    let canon_root = root.canonicalize().map_err(|e| {
        Error::InvalidInput(format!(
            "wiki source root `{}` is not accessible: {e}",
            root.display()
        ))
    })?;
    if !canon_root.is_dir() {
        return Err(Error::InvalidInput(format!(
            "wiki source root `{}` is not a directory",
            root.display()
        )));
    }
    walk_files(&canon_root, &canon_root, accept, visit);
    Ok(())
}

/// Walk body of [`collect_text_files`]. Returns `false` when the walk must stop
/// (a `visit` callback signalled it); recursion propagates the stop.
fn walk_files(
    root: &Path,
    dir: &Path,
    accept: &dyn Fn(&str) -> bool,
    visit: &mut dyn FnMut(String, String) -> bool,
) -> bool {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        // Unreadable subtree: skip, do not fail the whole scan.
        Err(_) => return true,
    };
    let mut paths: Vec<PathBuf> = entries.filter_map(|e| e.ok()).map(|e| e.path()).collect();
    paths.sort(); // deterministic discovery order
    for path in paths {
        let Ok(meta) = std::fs::metadata(&path) else {
            continue;
        };
        if meta.is_dir() {
            if !walk_files(root, &path, accept, visit) {
                return false;
            }
            continue;
        }
        if !meta.is_file() {
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !accept(ext) {
            tracing::debug!(path = %path.display(), "wiki scan: skipped non-accepted entry");
            continue;
        }
        // Traversal guard: resolve symlinks/`..` and require containment.
        let Ok(canon) = path.canonicalize() else {
            continue;
        };
        if !ensure_within_root(root, &canon) {
            tracing::warn!(
                path = %path.display(),
                "wiki scan: source escapes root, skipped"
            );
            continue;
        }
        let content = match std::fs::read_to_string(&canon) {
            Ok(content) => content,
            Err(_) => {
                tracing::debug!(path = %path.display(), "wiki scan: unreadable/non-UTF-8 file skipped");
                continue;
            }
        };
        if content.is_empty() {
            continue;
        }
        if !visit(rel_path(root, &canon), content) {
            return false;
        }
    }
    true
}

/// Forward-slash relative path from `root` to `path` (both canonicalized).
fn rel_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_root() -> tempfile::TempDir {
        tempfile::tempdir().expect("tempdir")
    }

    fn write(root: &Path, rel: &str, content: &str) {
        let p = root.join(rel);
        fs::create_dir_all(p.parent().expect("parent")).expect("mkdir");
        fs::write(p, content).expect("write");
    }

    // ── (a) scanner descubre .md en path local recursivo ──

    #[test]
    fn scanner_discovers_md_files_recursively() {
        let root = temp_root();
        write(root.path(), "a.md", "root doc");
        write(root.path(), "sub/b.md", "nested doc");
        write(root.path(), "sub/deep/c.md", "deep doc");

        let files = scan_local_sources(root.path()).expect("scan");

        let rels: Vec<&str> = files.iter().map(|f| f.rel_path.as_str()).collect();
        assert_eq!(rels, vec!["a.md", "sub/b.md", "sub/deep/c.md"]);
        assert_eq!(files[0].content, "root doc");
    }

    #[test]
    fn scanner_filters_non_md_and_empty_files() {
        let root = temp_root();
        write(root.path(), "keep.md", "real");
        write(root.path(), "skip.txt", "not markdown");
        write(root.path(), "empty.md", "");

        let files = scan_local_sources(root.path()).expect("scan");

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].rel_path, "keep.md");
    }

    #[test]
    fn nonexistent_root_is_a_clear_error() {
        let err = scan_local_sources(Path::new("Z:/definitely/not/here")).unwrap_err();
        assert!(matches!(err, Error::InvalidInput(_)), "got {err:?}");
        let msg = err.to_string();
        assert!(msg.contains("not accessible"), "message: {msg}");
    }

    // ── (c) SOURCE_CHAR_BUDGET 28000 respeta ──

    #[test]
    fn total_content_respects_char_budget() {
        let root = temp_root();
        let big = "x".repeat(15_000);
        write(root.path(), "one.md", &big);
        write(root.path(), "two.md", &big);
        write(root.path(), "three.md", &big);

        let files = scan_local_sources(root.path()).expect("scan");

        let total: usize = files.iter().map(|f| f.content.chars().count()).sum();
        assert_eq!(total, SOURCE_CHAR_BUDGET, "budget consumed exactly");
        assert_eq!(files.len(), 2, "scan stops once the budget is exhausted");
        assert_eq!(files[0].content.chars().count(), 15_000, "first fits whole");
        assert_eq!(
            files[1].content.chars().count(),
            13_000,
            "second truncated to remaining budget"
        );
    }

    #[test]
    fn small_sources_stay_well_under_budget() {
        let root = temp_root();
        for i in 0..5 {
            write(root.path(), &format!("doc{i}.md"), &"y".repeat(100));
        }
        let files = scan_local_sources(root.path()).expect("scan");
        assert_eq!(files.len(), 5);
        let total: usize = files.iter().map(|f| f.content.chars().count()).sum();
        assert_eq!(total, 500);
    }

    // ── (e) path traversal guard ──

    #[test]
    fn traversal_guard_rejects_paths_outside_root() {
        let root = temp_root();
        let outside = root.path().join("..").join("outside-target");
        fs::create_dir_all(&outside).expect("mkdir outside");

        let canon_outside = outside.canonicalize().expect("canonicalize outside");
        let canon_root = root.path().canonicalize().expect("canonicalize root");

        assert!(!ensure_within_root(&canon_root, &canon_outside));
        assert!(ensure_within_root(
            &canon_root,
            &canon_root.join("sub").join("page.md")
        ));
    }

    #[test]
    fn symlink_escaping_root_is_skipped() {
        let root = temp_root();
        write(root.path(), "inside.md", "inside content");
        let external = temp_root();
        write(external.path(), "secret.md", "should not leak");

        let link = root.path().join("escape.md");
        // Symlink creation may be unavailable on Windows without privileges —
        // only assert when the link could actually be created.
        let target = external.path().join("secret.md");
        let linked;
        #[cfg(unix)]
        {
            linked = std::os::unix::fs::symlink(&target, &link).is_ok();
        }
        #[cfg(windows)]
        {
            linked = std::os::windows::fs::symlink_file(&target, &link).is_ok();
        }
        if linked {
            let files = scan_local_sources(root.path()).expect("scan");
            let rels: Vec<&str> = files.iter().map(|f| f.rel_path.as_str()).collect();
            assert!(
                !rels.iter().any(|r| r.contains("escape")),
                "escaped symlink must not be collected: {rels:?}"
            );
            assert_eq!(files.len(), 1, "only inside.md survives");
        }
    }
}
