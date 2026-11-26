//! Dynamic value completion for `--namespace` / `--key` (clap `CompleteEnv`).
//!
//! Best-effort by contract: any failure (no DB, locked backend, slow disk)
//! yields zero candidates — never an error, never stdout noise, never a panic.
//! The completer runs inside the short-lived `COMPLETE=<shell>` child process,
//! so a slow backend only delays one TAB press, never the shell.
//!
//! Design notes:
//! - The DB path resolves exactly like the CLI itself: `--db` / `-d` from the
//!   current argv, else `VANTADB_STORAGE_PATH`, else `./db` (see `Cli::db`).
//! - The database opens read-only (`open_embedded(path, true)`); a missing or
//!   locked backend returns no candidates instead of creating or blocking.
//! - `--key` additionally needs `--namespace` from the same argv; without it
//!   there is nothing to complete (empty list, not an error).

use std::ffi::OsStr;

use clap_complete::CompletionCandidate;

use crate::cli_handlers::db::open_embedded;

/// Upper bound so a huge namespace never floods the shell.
const MAX_CANDIDATES: usize = 50;

fn prefix_of(current: &OsStr) -> String {
    current.to_string_lossy().into_owned()
}

fn keep_prefix(all: Vec<String>, prefix: &str) -> Vec<CompletionCandidate> {
    all.into_iter()
        .filter(|v| v.starts_with(prefix))
        .take(MAX_CANDIDATES)
        .map(CompletionCandidate::new)
        .collect()
}

/// Value of `--flag`, `-s` or `--flag=value` in the current (incomplete) argv.
/// `flag_short` is `None` when the flag has no short form.
fn argv_value(flag_long: &str, flag_short: Option<char>) -> Option<String> {
    let short = flag_short.map(|c| format!("-{c}"));
    let mut it = std::env::args_os().skip(1);
    while let Some(a) = it.next() {
        let s = a.to_string_lossy();
        if s == flag_long || short.as_deref() == Some(&s) {
            return it.next().map(|v| v.to_string_lossy().into_owned());
        }
        if let Some(v) = s.strip_prefix(&format!("{flag_long}=")) {
            return Some(v.to_owned());
        }
    }
    None
}

fn db_path() -> Option<std::path::PathBuf> {
    if let Some(p) = argv_value("--db", Some('d')) {
        return Some(std::path::PathBuf::from(p));
    }
    match std::env::var("VANTADB_STORAGE_PATH") {
        Ok(p) if !p.trim().is_empty() => Some(std::path::PathBuf::from(p)),
        _ => Some(std::path::PathBuf::from("./db")),
    }
}

fn guarded(
    f: impl FnOnce() -> Vec<CompletionCandidate> + std::panic::UnwindSafe,
) -> Vec<CompletionCandidate> {
    std::panic::catch_unwind(f).unwrap_or_default()
}

/// Complete `--namespace` from the namespaces present in the resolved DB.
pub fn complete_namespaces(current: &OsStr) -> Vec<CompletionCandidate> {
    let prefix = prefix_of(current);
    guarded(move || {
        let mut out = Vec::new();
        if let Some(path) = db_path() {
            if path.is_dir() {
                if let Ok(db) = open_embedded(&path.to_string_lossy(), true) {
                    if let Ok(all) = db.list_namespaces() {
                        out = keep_prefix(all, &prefix);
                    }
                }
            }
        }
        out
    })
}

/// Complete `--key` from the keys of `--namespace` in the resolved DB.
pub fn complete_keys(current: &OsStr) -> Vec<CompletionCandidate> {
    let prefix = prefix_of(current);
    guarded(move || {
        let mut out = Vec::new();
        if let Some(ns) = argv_value("--namespace", None) {
            if !ns.trim().is_empty() {
                if let Some(path) = db_path() {
                    if path.is_dir() {
                        if let Ok(db) = open_embedded(&path.to_string_lossy(), true) {
                            let opts = crate::sdk::MemoryListOptions {
                                #[allow(deprecated)]
                                filters: crate::sdk::MemoryMetadata::new(),
                                filter_ops: None,
                                limit: MAX_CANDIDATES,
                                cursor: None,
                                exclude_superseded: false,
                                as_of_ms: None,
                                valid_window: None,
                                include_quarantined: false,
                                min_confidence: None,
                            };
                            if let Ok(page) = db.list(&ns, opts) {
                                let all: Vec<String> =
                                    page.records.into_iter().map(|r| r.key).collect();
                                out = keep_prefix(all, &prefix);
                            }
                        }
                    }
                }
            }
        }
        out
    })
}
