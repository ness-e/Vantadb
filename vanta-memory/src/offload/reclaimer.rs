//! Reclaimer: GC of stale offloaded artifacts (port of TDAM
//! `MC/offload/reclaimer.ts`, MEM-42).
//!
//! TDAM walks the data directory by file mtime across 5 independent steps
//! (JSONL, refs, MMDs, logs, registry). Here the only artifact class is the
//! per-session offload records of [`crate::offload::storage::OffloadStorage`],
//! so the port collapses to one step over store records. Age comes from each
//! [`OffloadEntry::timestamp`] (ISO string inherited from the tool result) —
//! record-store mtimes do not exist.
//!
//! Safety rules (D19):
//! - Only *strictly pre-cursor* entries are reclaimed: an entry qualifies only
//!   when its timestamp is strictly older than the timestamp of the entry the
//!   L3 cursor ([`PluginState::last_offloaded_tool_call_id`]) points at. The
//!   cursor target itself therefore always survives — the cursor can never
//!   dangle into GC-ed data.
//! - `retention_days < 3` disables the reclaimer entirely (TDAM parity).
//! - Undatable entries (unparseable timestamp) are skipped, never guessed.
//! - Deletion is delete-by-key through the SDK: naturally idempotent, so a
//!   crash mid-run leaves a consistent state and a rerun finishes the job.
//! - LLM-free: pure deterministic logic, no runner involved.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::offload::state_manager::{OffloadError, OffloadStateManager};
use crate::offload::storage::{entries_namespace, OffloadStorage};
use crate::offload::types::OffloadEntry;
use crate::utils::sanitize::sanitize_key;
use vantadb::sdk::VantaEmbedded;

/// Minimum effective retention. Values below disable reclamation
/// (TDAM `reclaimer.ts:75-78`).
pub const MIN_RETENTION_DAYS: u64 = 3;

const SECS_PER_DAY: i64 = 86_400;

/// Outcome of one reclamation pass over a session.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReclaimStats {
    /// Entries examined (parsed successfully or skipped).
    pub scanned: usize,
    /// Entries actually deleted from the store.
    pub deleted: usize,
}

/// Garbage collector for offloaded tool-call entries.
pub struct OffloadReclaimer {
    db: VantaEmbedded,
}

impl OffloadReclaimer {
    /// Open a reclaimer over an already-open embedded database.
    pub fn new(db: VantaEmbedded) -> Self {
        Self { db }
    }

    /// Run a reclamation pass using the current wall clock.
    pub fn reclaim(
        &self,
        session_id: &str,
        retention_days: u64,
    ) -> Result<ReclaimStats, OffloadError> {
        let now_secs = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(d) => d.as_secs() as i64,
            Err(err) => {
                // Clock before the epoch: cannot date anything reliably —
                // conservatively reclaim nothing.
                tracing::warn!("offload reclaim skipped, system clock before epoch: {err}");
                return Ok(ReclaimStats::default());
            }
        };
        self.reclaim_as_of(session_id, retention_days, now_secs)
    }

    /// Run a reclamation pass as of an explicit instant (epoch seconds).
    /// Testable core of [`Self::reclaim`].
    pub fn reclaim_as_of(
        &self,
        session_id: &str,
        retention_days: u64,
        now_secs: i64,
    ) -> Result<ReclaimStats, OffloadError> {
        if retention_days < MIN_RETENTION_DAYS {
            tracing::debug!(
                retention_days,
                "offload reclaim skipped: retention below minimum ({MIN_RETENTION_DAYS})"
            );
            return Ok(ReclaimStats::default());
        }

        let state = OffloadStateManager::new(self.db.clone());
        let storage = OffloadStorage::new(self.db.clone());
        let entries = storage.read_entries(session_id)?;
        let mut stats = ReclaimStats {
            scanned: entries.len(),
            deleted: 0,
        };

        // The cursor marks how far compact context has summarized. Without a
        // cursor nothing is provably consumed — reclaim nothing.
        let Some(cursor_id) = state.last_offloaded_tool_call_id(session_id)? else {
            return Ok(stats);
        };
        // Horizon = timestamp of the cursor's own entry. Missing or undatable
        // cursor target → no provably-consumed prefix → reclaim nothing.
        let Some(cursor_ts) = entries
            .iter()
            .find(|e| sanitize_key(&e.tool_call_id) == cursor_id)
            .and_then(|e| iso_to_epoch_secs(&e.timestamp))
        else {
            tracing::warn!(
                session = %session_id,
                "offload reclaim skipped: cursor entry missing or undatable"
            );
            return Ok(stats);
        };

        let cutoff = now_secs - retention_days as i64 * SECS_PER_DAY;
        let ns = entries_namespace(session_id);
        for entry in &entries {
            let Some(ts) = iso_to_epoch_secs(&entry.timestamp) else {
                tracing::warn!(
                    tool_call_id = %entry.tool_call_id,
                    "offload reclaim: skipping entry with unparseable timestamp"
                );
                continue;
            };
            // Strictly pre-cursor (already consumed by L1) AND older than the
            // retention window. Strict `<` keeps the cursor target alive.
            if ts < cursor_ts && ts < cutoff && self.delete_entry(&ns, entry)? {
                stats.deleted += 1;
            }
        }
        Ok(stats)
    }

    /// Delete one entry by key. Returns whether it existed.
    fn delete_entry(&self, ns: &str, entry: &OffloadEntry) -> Result<bool, OffloadError> {
        self.db
            .delete(ns, &sanitize_key(&entry.tool_call_id))
            .map_err(OffloadError::from)
    }
}

/// Parse an ISO-8601 timestamp (`YYYY-MM-DDTHH:MM:SS`, optional `.frac`,
/// optional `Z` / `±HH:MM`) into UTC epoch seconds. `None` when malformed —
/// callers must treat undatable entries as un-GC-able.
fn iso_to_epoch_secs(ts: &str) -> Option<i64> {
    let b = ts.as_bytes();
    if b.len() < 19
        || b[4] != b'-'
        || b[7] != b'-'
        || b[10] != b'T'
        || b[13] != b':'
        || b[16] != b':'
    {
        return None;
    }
    let year: i64 = ts.get(0..4)?.parse().ok()?;
    let month: u32 = ts.get(5..7)?.parse().ok()?;
    let day: u32 = ts.get(8..10)?.parse().ok()?;
    let hour: i64 = ts.get(11..13)?.parse().ok()?;
    let min: i64 = ts.get(14..16)?.parse().ok()?;
    let sec: i64 = ts.get(17..19)?.parse().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }

    let mut idx = 19;
    if b.get(idx) == Some(&b'.') {
        idx += 1;
        while matches!(b.get(idx), Some(c) if c.is_ascii_digit()) {
            idx += 1;
        }
    }
    let offset: i64 = match b.get(idx) {
        // A timestamp without TZ marker is undatable (local vs UTC unknown):
        // reject — conservative, the entry is never GC-ed.
        None => return None,
        Some(b'Z') | Some(b'z') => 0,
        Some(sign @ (b'+' | b'-')) => {
            let oh: i64 = ts.get(idx + 1..idx + 3)?.parse().ok()?;
            if b.get(idx + 3) != Some(&b':') {
                return None;
            }
            let om: i64 = ts.get(idx + 4..idx + 6)?.parse().ok()?;
            let v = oh * 3600 + om * 60;
            if *sign == b'+' {
                v
            } else {
                -v
            }
        }
        _ => return None,
    };

    Some(days_from_civil(year, month, day) * SECS_PER_DAY + hour * 3600 + min * 60 + sec - offset)
}

/// Days since 1970-01-01 from a civil date (Howard Hinnant's algorithm).
fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = i64::from(if m > 2 { m - 3 } else { m + 9 });
    let doy = (153 * mp + 2) / 5 + i64::from(d) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;
    use vantadb::config::VantaConfig;
    use vantadb::storage::BackendKind;

    fn open_db() -> VantaEmbedded {
        let config = VantaConfig {
            backend_kind: BackendKind::InMemory,
            read_only: false,
            ..VantaConfig::default()
        };
        VantaEmbedded::open_with_config(config).expect("open in-memory db")
    }

    /// Seed a session with three entries: old (day 1), cursor (day 5),
    /// future (day 9); cursor points at the day-5 entry. `now` = day 30.
    fn seeded(db: &VantaEmbedded) -> (OffloadStateManager, OffloadStorage) {
        let state = OffloadStateManager::new(db.clone());
        let storage = OffloadStorage::new(db.clone());
        for (id, ts) in [
            ("call_old", "2026-08-01T10:00:00Z"),
            ("call_cur", "2026-08-05T10:00:00Z"),
            ("call_new", "2026-08-09T10:00:00Z"),
        ] {
            storage
                .append_entry(
                    "s1",
                    &OffloadEntry {
                        timestamp: ts.into(),
                        node_id: None,
                        tool_call: "read_file(path=x)".into(),
                        summary: "read x".into(),
                        result_ref: format!("results/{id}.md"),
                        tool_call_id: id.into(),
                        session_key: Some("s1".into()),
                        score: None,
                    },
                )
                .expect("seed entry");
        }
        state
            .set_last_offloaded_tool_call_id("s1", Some("call_cur"))
            .expect("set cursor");
        (state, storage)
    }

    /// Fixed "now" for tests: 2026-08-31T00:00:00Z in epoch seconds.
    fn now() -> i64 {
        iso_to_epoch_secs("2026-08-31T00:00:00Z").expect("fixed test instant")
    }

    #[test]
    fn retention_below_minimum_disables_reclaimer() {
        let db = open_db();
        seeded(&db);
        let r = OffloadReclaimer::new(db);
        for retention in [0, 1, 2] {
            let stats = r.reclaim_as_of("s1", retention, now()).expect("run");
            assert_eq!(stats.deleted, 0, "retention={retention} must be a no-op");
        }
    }

    #[test]
    fn stale_precursor_entries_are_deleted_recent_survive() {
        let db = open_db();
        let (_, storage) = seeded(&db);
        let r = OffloadReclaimer::new(db);
        // Retention 10 days, now = day 30 → cutoff day 20: only call_old (day 1,
        // strictly pre-cursor) qualifies.
        let stats = r.reclaim_as_of("s1", 10, now()).expect("run");
        assert_eq!(stats.scanned, 3);
        assert_eq!(stats.deleted, 1);
        let ids: Vec<_> = storage
            .read_entries("s1")
            .expect("read")
            .into_iter()
            .map(|e| e.tool_call_id)
            .collect();
        assert!(ids.contains(&"call_cur".to_string()));
        assert!(ids.contains(&"call_new".to_string()));
        assert!(!ids.contains(&"call_old".to_string()));
    }

    #[test]
    fn cursor_never_points_at_deleted_entry() {
        let db = open_db();
        let (state, _) = seeded(&db);
        let r = OffloadReclaimer::new(db);
        r.reclaim_as_of("s1", 10, now()).expect("run");
        let cursor = state
            .last_offloaded_tool_call_id("s1")
            .expect("cursor")
            .expect("non-empty");
        // The cursor target must still exist in the store after GC.
        let storage = OffloadStorage::new(r.db.clone());
        assert!(
            storage.has_entry("s1", &cursor).expect("probe"),
            "cursor target was GC-ed"
        );
    }

    #[test]
    fn post_cursor_entries_survive_even_when_stale() {
        let db = open_db();
        let state = OffloadStateManager::new(db.clone());
        let storage = OffloadStorage::new(db.clone());
        // Cursor at Aug 5; an entry exists AFTER the cursor (Aug 20) that is
        // already past the retention cutoff (Aug 21): it must survive — only
        // strictly pre-cursor entries are reclaimable.
        for (id, ts) in [
            ("call_cur", "2026-08-05T10:00:00Z"),
            ("call_after", "2026-08-20T10:00:00Z"),
        ] {
            storage
                .append_entry(
                    "s1",
                    &OffloadEntry {
                        timestamp: ts.into(),
                        node_id: None,
                        tool_call: "t".into(),
                        summary: "s".into(),
                        result_ref: format!("results/{id}.md"),
                        tool_call_id: id.into(),
                        session_key: None,
                        score: None,
                    },
                )
                .expect("seed");
        }
        state
            .set_last_offloaded_tool_call_id("s1", Some("call_cur"))
            .expect("cursor");
        let stats = OffloadReclaimer::new(db)
            .reclaim_as_of("s1", 10, now())
            .expect("run");
        assert_eq!(stats.deleted, 0);
        assert!(storage.has_entry("s1", "call_after").expect("probe"));
    }

    #[test]
    fn reclaim_is_idempotent_across_reruns() {
        let db = open_db();
        let (_, storage) = seeded(&db);
        let r = OffloadReclaimer::new(db);
        let first = r.reclaim_as_of("s1", 10, now()).expect("first");
        assert_eq!(first.deleted, 1);
        // Crash-recovery simulation: rerun finds nothing left to delete.
        let second = r.reclaim_as_of("s1", 10, now()).expect("second");
        assert_eq!(second.deleted, 0);
        assert_eq!(storage.read_entries("s1").expect("read").len(), 2);
    }

    #[test]
    fn no_cursor_means_no_gc() {
        let db = open_db();
        let (state, storage) = seeded(&db);
        // Clear the cursor: nothing is provably consumed.
        state
            .set_last_offloaded_tool_call_id("s1", None)
            .expect("clear cursor");
        let stats = OffloadReclaimer::new(db)
            .reclaim_as_of("s1", 10, now())
            .expect("run");
        assert_eq!(stats.deleted, 0);
        assert_eq!(storage.read_entries("s1").expect("read").len(), 3);
    }

    #[test]
    fn undatable_timestamp_is_skipped_not_deleted() {
        let db = open_db();
        let state = OffloadStateManager::new(db.clone());
        let storage = OffloadStorage::new(db.clone());
        for (id, ts) in [
            ("call_bad", "not-a-date"),
            ("call_cur", "2026-08-05T10:00:00Z"),
        ] {
            storage
                .append_entry(
                    "s1",
                    &OffloadEntry {
                        timestamp: ts.into(),
                        node_id: None,
                        tool_call: "t".into(),
                        summary: "s".into(),
                        result_ref: format!("results/{id}.md"),
                        tool_call_id: id.into(),
                        session_key: None,
                        score: None,
                    },
                )
                .expect("seed");
        }
        state
            .set_last_offloaded_tool_call_id("s1", Some("call_cur"))
            .expect("cursor");
        let stats = OffloadReclaimer::new(db)
            .reclaim_as_of("s1", 3, now())
            .expect("run");
        assert_eq!(stats.deleted, 0);
        assert!(storage.has_entry("s1", "call_bad").expect("probe"));
    }

    #[test]
    fn iso_parsing_known_instants_and_offsets() {
        assert_eq!(iso_to_epoch_secs("1970-01-01T00:00:00Z"), Some(0));
        // Leap day: 2000-01-01T00:00:00Z = 946_684_800 (+59 days → Feb 29).
        assert_eq!(iso_to_epoch_secs("2000-02-29T00:00:00Z"), Some(951_782_400));
        // Fractional seconds don't shift whole-second value.
        assert_eq!(
            iso_to_epoch_secs("2026-08-20T10:00:00.123456Z"),
            iso_to_epoch_secs("2026-08-20T10:00:00Z")
        );
        // Explicit offsets convert to UTC.
        assert_eq!(
            iso_to_epoch_secs("2026-08-20T12:00:00+02:00"),
            iso_to_epoch_secs("2026-08-20T10:00:00Z")
        );
        assert_eq!(
            iso_to_epoch_secs("2026-08-20T10:00:00"),
            None,
            "no TZ → reject"
        );
        assert_eq!(iso_to_epoch_secs("garbage"), None);
    }
}
