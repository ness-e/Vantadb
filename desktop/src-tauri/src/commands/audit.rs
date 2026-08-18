//! Audit-log IPC commands (VS-12).
//!
//! `vanta_audit_events` reads the active connection's audit JSONL (configured
//! on `NativeConnection::open` — default `<storage>/audit.jsonl`), filters in
//! Rust, and returns events newest-first (the log tail) with a cursor for
//! pagination into older events.

use std::path::Path;

use tauri::State;

use crate::connections::types::{AuditEvent, AuditPage};
use crate::error::VantaError;
use crate::AppState;

/// Default page size when the caller omits `limit`.
const DEFAULT_LIMIT: usize = 100;

/// Read audit events from the active connection's audit log (VS-12).
///
/// Filters (`namespace`/`op`/`outcome`) apply in Rust; results come back
/// newest-first (tail of the JSONL). `cursor` is the offset from a previous
/// page's `next_cursor` and continues into older events; `None` = newest page.
#[tauri::command]
pub async fn vanta_audit_events(
    state: State<'_, AppState>,
    namespace: Option<String>,
    op: Option<String>,
    outcome: Option<String>,
    limit: Option<usize>,
    cursor: Option<usize>,
) -> Result<AuditPage, VantaError> {
    let path = state
        .manager
        .audit_log_path()
        .await?
        .ok_or_else(|| VantaError::Unsupported("audit log no configurado".into()))?;
    read_audit_events(
        &path,
        namespace.as_deref(),
        op.as_deref(),
        outcome.as_deref(),
        limit.unwrap_or(DEFAULT_LIMIT),
        cursor,
    )
}

/// Read the audit JSONL at `path`, apply filters, and paginate newest-first.
///
/// `cursor` is a zero-based offset into the *filtered* newest-first list
/// (stable while the log is append-only and the filters don't change).
/// `next_cursor` is `Some(end)` when older events remain, `None` otherwise.
///
/// ponytail: reads the whole file (fine for desktop-sized audit logs); a
/// byte-offset tail read is the upgrade if the log grows large.
fn read_audit_events(
    path: &Path,
    namespace: Option<&str>,
    op: Option<&str>,
    outcome: Option<&str>,
    limit: usize,
    cursor: Option<usize>,
) -> Result<AuditPage, VantaError> {
    let content = std::fs::read_to_string(path)?;
    let mut matched: Vec<AuditEvent> = content
        .lines()
        .filter_map(|line| serde_json::from_str::<AuditEvent>(line).ok())
        .filter(|e| namespace.is_none_or(|n| e.namespace == n))
        .filter(|e| op.is_none_or(|o| e.op == o))
        .filter(|e| outcome.is_none_or(|o| e.outcome == o))
        .collect();
    // Tail: newest event (last line) first.
    matched.reverse();
    let start = cursor.unwrap_or(0).min(matched.len());
    let end = (start + limit).min(matched.len());
    let events = matched[start..end].to_vec();
    let next_cursor = (end < matched.len()).then_some(end);
    Ok(AuditPage {
        events,
        next_cursor,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Write a JSONL fixture into a temp file, return its path.
    fn write_fixture(events: &[&str]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "vantadb-desktop-audit-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join("audit.jsonl");
        std::fs::write(&path, events.join("\n") + "\n").expect("write fixture");
        path
    }

    fn ev(timestamp: &str, op: &str, namespace: &str, key: &str, outcome: &str) -> String {
        format!(
            r#"{{"timestamp":"{timestamp}","op":"{op}","namespace":"{namespace}","key":"{key}","outcome":"{outcome}","reason":null}}"#
        )
    }

    #[test]
    fn tail_reads_newest_first() {
        let path = write_fixture(&[
            &ev("2026-08-01T00:00:00Z", "put", "a", "k1", "ok"),
            &ev("2026-08-01T00:01:00Z", "put", "b", "k2", "ok"),
            &ev("2026-08-01T00:02:00Z", "delete", "a", "k1", "ok"),
        ]);
        let page = read_audit_events(&path, None, None, None, 100, None).expect("read");
        assert_eq!(page.events.len(), 3);
        assert_eq!(page.events[0].op, "delete", "newest event first (tail)");
        assert_eq!(page.events[0].timestamp, "2026-08-01T00:02:00Z");
        assert_eq!(page.events[2].op, "put", "oldest event last");
        assert_eq!(page.next_cursor, None, "everything fits in one page");
    }

    #[test]
    fn filters_by_namespace_op_and_outcome() {
        let path = write_fixture(&[
            &ev("2026-08-01T00:00:00Z", "put", "docs", "k1", "ok"),
            &ev("2026-08-01T00:01:00Z", "put", "mem", "k2", "ok"),
            &ev("2026-08-01T00:02:00Z", "delete", "docs", "k1", "err"),
        ]);
        let page = read_audit_events(&path, Some("docs"), None, None, 100, None).expect("read");
        assert_eq!(page.events.len(), 2, "both docs events match");
        assert!(page.events.iter().all(|e| e.namespace == "docs"));

        let page = read_audit_events(&path, None, Some("put"), None, 100, None).expect("read");
        assert_eq!(page.events.len(), 2, "both put events match");
        assert!(page.events.iter().all(|e| e.op == "put"));

        let page = read_audit_events(&path, Some("docs"), Some("delete"), Some("err"), 100, None)
            .expect("read");
        assert_eq!(page.events.len(), 1);
        assert_eq!(page.events[0].key, "k1");
        assert_eq!(page.events[0].outcome, "err");

        // Combined filter with no matches → empty page, no cursor.
        let page =
            read_audit_events(&path, Some("mem"), Some("delete"), None, 100, None).expect("read");
        assert!(page.events.is_empty());
        assert_eq!(page.next_cursor, None);
    }

    #[test]
    fn cursor_paginates_without_overlap() {
        let events: Vec<String> = (0..5)
            .map(|i| {
                ev(
                    &format!("2026-08-01T00:{i:02}:00Z"),
                    "put",
                    "docs",
                    &format!("k{i}"),
                    "ok",
                )
            })
            .collect();
        let refs: Vec<&str> = events.iter().map(String::as_str).collect();
        let path = write_fixture(&refs);

        // Page 1: newest 2 events (k4, k3).
        let p1 = read_audit_events(&path, None, None, None, 2, None).expect("page 1");
        assert_eq!(p1.events.len(), 2);
        assert_eq!(p1.events[0].key, "k4");
        let cursor = p1.next_cursor.expect("page 1 is full, must carry a cursor");
        assert_eq!(cursor, 2);

        // Page 2: next 2 older events (k2, k1), disjoint from page 1.
        let p2 = read_audit_events(&path, None, None, None, 2, Some(cursor)).expect("page 2");
        assert_eq!(p2.events.len(), 2);
        for e in &p2.events {
            assert!(
                !p1.events.iter().any(|x| x.key == e.key),
                "page 2 overlaps page 1: {}",
                e.key
            );
        }
        assert_eq!(p2.next_cursor, Some(4));

        // Page 3: last older event (k0); a short page means the end.
        let p3 = read_audit_events(
            &path,
            None,
            None,
            None,
            2,
            Some(p2.next_cursor.expect("page 2 is full")),
        )
        .expect("page 3");
        assert_eq!(p3.events.len(), 1);
        assert_eq!(p3.events[0].key, "k0");
        assert_eq!(p3.next_cursor, None, "a short page is the last page");
    }

    #[test]
    fn skips_malformed_lines_and_empty_file() {
        let path = write_fixture(&[
            "not json",
            &ev("2026-08-01T00:00:00Z", "put", "a", "k1", "ok"),
        ]);
        let page = read_audit_events(&path, None, None, None, 100, None).expect("read");
        assert_eq!(page.events.len(), 1, "malformed lines are skipped");

        let empty = write_fixture(&[]);
        let page = read_audit_events(&empty, None, None, None, 100, None).expect("read");
        assert!(page.events.is_empty());
        assert_eq!(page.next_cursor, None);
    }

    #[test]
    fn missing_file_errors() {
        let missing = std::env::temp_dir().join("vantadb-desktop-audit-missing.jsonl");
        assert!(read_audit_events(&missing, None, None, None, 100, None).is_err());
    }
}
