//! FIND-632 / ADR-0001: filesystem snapshot helpers shared by the
//! `read_only` open tests.
//!
//! A `read_only` open of the Fjall backend is a **logical** barrier (Fjall
//! 3.1.12 has no physical read-only mode — fjall#313): the engine writes no
//! data, registry or key material, but Fjall itself still mutates a small,
//! fixed set of its own metadata files during open. [`fjall_open_allowlist`]
//! is the single source of truth for that set; every entry cites the fjall
//! source line that causes the mutation. Anything outside the allowlist —
//! SST/segment files, blobs, `encryption.registry`, VantaFile/WAL/index
//! files — is a real violation and fails the test.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Content fingerprint of one file: length + SHA-256 of its bytes.
///
/// Two axes are kept because a file can be truncated-and-rewritten to the
/// same length; the hash catches content changes that `len` hides.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct FileFingerprint {
    pub len: u64,
    pub sha256: [u8; 32],
}

/// Recursive snapshot of every file under `root`, keyed by **relative** path
/// (root itself is never stored as a key — only its descendants).
pub(super) fn snapshot_tree(root: &Path) -> BTreeMap<PathBuf, FileFingerprint> {
    fn walk(dir: &Path, root: &Path, out: &mut BTreeMap<PathBuf, FileFingerprint>) {
        for entry in std::fs::read_dir(dir).expect("read dir for snapshot") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.is_dir() {
                walk(&path, root, out);
            } else {
                let bytes = std::fs::read(&path).expect("read file for snapshot");
                let digest: [u8; 32] = {
                    use sha2::Digest;
                    let mut h = sha2::Sha256::new();
                    h.update(&bytes);
                    h.finalize().into()
                };
                out.insert(
                    path.strip_prefix(root)
                        .expect("relative path")
                        .to_path_buf(),
                    FileFingerprint {
                        len: bytes.len() as u64,
                        sha256: digest,
                    },
                );
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out
}

/// Diff two snapshots. Returns `(created, mutated, removed)` relative paths.
pub(super) fn diff_snapshots(
    before: &BTreeMap<PathBuf, FileFingerprint>,
    after: &BTreeMap<PathBuf, FileFingerprint>,
) -> (Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>) {
    let created: Vec<PathBuf> = after
        .keys()
        .filter(|p| !before.contains_key(*p))
        .cloned()
        .collect();
    let removed: Vec<PathBuf> = before
        .keys()
        .filter(|p| !after.contains_key(*p))
        .cloned()
        .collect();
    let mutated: Vec<PathBuf> = before
        .iter()
        .filter(|(p, fp)| after.get(*p).is_some_and(|a| a != *fp))
        .map(|(p, _)| p.clone())
        .collect();
    (created, mutated, removed)
}

/// The Fjall open-time metadata allowlist — **measured** on fjall 3.1.12 /
/// lsm-tree 3.1.10 (FIND-632) by snapshotting (path, len, sha256) a closed,
/// populated database before and after a `read_only` open:
///
/// | Observed diff | Pattern | Cause |
/// |---|---|---|
/// | mutated | `<n>.jnl` at db root | `fjall-3.1.12/src/db.rs:597` persists the replayed journal with `PersistMode::SyncAll` on every open; `src/journal/writer.rs:124-185` creates it via `create_new` + `set_len(64 MiB)` when missing |
/// | removed | `keyspaces/0/v{N}` | `lsm-tree` recovery of the internal meta-keyspace `keyspaces/0` (fjall-3.1.12 `src/db.rs:601-606`) rewrites/drops stale version markers on open (the `v0..v14` set the FIND-508 RED already observed) |
///
/// `rel` is the path relative to the database root (the `path` passed to
/// `StorageEngine::open_with_config`). Matching is exact: **not allowed** —
/// anything under `data/` (VantaFile/WAL/registry/index engine artefacts),
/// SST/segment/blob/table data files under `keyspaces/**`, and any file this
/// function does not name. Set `FIND632_DENY_ALLOWLIST=1` to force every
/// path out of the allowlist — the RED probe used to capture the diff above.
pub(super) fn fjall_open_allowlist(rel: &Path) -> bool {
    // FIND-632 RED probe: deny everything to re-observe the raw set of files
    // Fjall mutates on a read_only open (e.g. after a fjall version bump).
    if crate::utils::env_present(std::env::var_os("FIND632_DENY_ALLOWLIST")) {
        return false;
    }
    let mut components = rel.components().map(|c| c.as_os_str());
    let first = components.next().unwrap_or_default();

    // Fjall journal files live at the database root: `<n>.jnl`
    // (single-component relative path with the `.jnl` extension).
    if components.next().is_none() && rel.extension().is_some_and(|ext| ext == "jnl") {
        return true;
    }

    // Internal meta-keyspace version markers: `keyspaces/0/v<N>`.
    if first == "keyspaces" {
        let rest: Vec<&std::ffi::OsStr> = rel.components().skip(1).map(|c| c.as_os_str()).collect();
        if rest.len() == 2 && rest[0] == "0" {
            let name = rest[1].to_string_lossy();
            return name.len() > 1
                && name.starts_with('v')
                && name[1..].chars().all(|c| c.is_ascii_digit());
        }
    }

    false
}

/// Site 3 of FIND-738: `FIND632_DENY_ALLOWLIST` is a RED probe documented as
/// "set to `1` to force every path out". Defined-but-empty must mean "not set",
/// because the empty form is what a shell leaves behind (`FIND632_DENY_ALLOWLIST=`
/// plus a forgotten value) and it would deny the whole allowlist — every
/// `read_only` test failing for a reason nobody asked for.
#[test]
fn deny_allowlist_flag_ignores_empty_values() {
    use std::ffi::OsString;
    assert!(!crate::utils::env_present(None), "absent = probe off");
    assert!(
        !crate::utils::env_present(Some(OsString::new())),
        "defined-but-empty = probe off"
    );
    assert!(
        crate::utils::env_present(Some(OsString::from("1"))),
        "any value = probe on (documented form is `=1`)"
    );
}
