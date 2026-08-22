//! Scheduled wiki re-ingest (MEM-45 — D40: port of TDAM
//! `MemoryKnowledge/src/store/auto-sync-scheduler.ts`).
//!
//! Pull-based (zero threads): the owner polls [`AutoSyncScheduler::tick`] from
//! its own loop; the deadline lives in a [`ManagedTimer`] driven by the
//! injected [`Clock`], so tests use [`FakeClock`] and never sleep.
//!
//! Change detection is a plain per-file FNV-1a hash of every scanned source —
//! no FS watcher (stop condition: watchers require new dependencies).
//! // ponytail: full-content rescan per tick; swap for a watcher only when
//! // source roots grow past ~10k files.
//!
//! Semantics:
//! - **Disabled by default** ([`AutoSyncConfig::default`]).
//! - Interval clamped to ≥ [`MIN_INTERVAL_MS`] (re-ingest storm guard).
//! - Busy guard pre-run (MEM-28): while the wiki is `pending|processing` the
//!   tick is skipped and the hashes are NOT updated, so the change is detected
//!   again on the next pass.
//! - A fresh `run_id` per build is minted by the core store
//!   (`begin_processing`, MEM-31); late packets from older runs are discarded.
//! - First due pass has no baseline → reconciling re-ingest (documented).

use std::collections::HashMap;
use std::path::Path;

use crate::core::abstractions::LlmRunner;
use crate::ingest::callback::ProgressTracker;
use crate::ingest::{worker, IngestConfig, IngestError};
use crate::utils::managed_timer::{Clock, ManagedTimer};

/// Lower bound for the interval between auto-sync passes (re-ingest storm
/// guard). Anything smaller is clamped up to this.
pub const MIN_INTERVAL_MS: u64 = 60_000;

/// Default interval between passes: 5 minutes.
pub const DEFAULT_INTERVAL_MS: u64 = 5 * MIN_INTERVAL_MS;

/// Auto-sync scheduler configuration. **Off by default** — opt-in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutoSyncConfig {
    /// Master switch. `false` by default; ticks are full no-ops while off.
    pub enabled: bool,
    /// Milliseconds between passes. Clamped ≥ [`MIN_INTERVAL_MS`].
    pub interval_ms: u64,
}

impl Default for AutoSyncConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_ms: DEFAULT_INTERVAL_MS,
        }
    }
}

impl AutoSyncConfig {
    /// Build a config with the interval clamped to ≥ [`MIN_INTERVAL_MS`].
    pub fn new(enabled: bool, interval_ms: u64) -> Self {
        Self {
            enabled,
            interval_ms: interval_ms.max(MIN_INTERVAL_MS),
        }
    }
}

/// What a single [`AutoSyncScheduler::tick`] did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TickOutcome {
    /// Not due yet, disabled by default-config, or due but nothing changed —
    /// no re-ingest happened.
    Idle,
    /// Changes were pending but the wiki is busy (`pending|processing`,
    /// MEM-28) — skipped without touching the stored hashes.
    Busy,
    /// A re-ingest build ran with this fresh `run_id`.
    Reingested { run_id: String },
}

/// Pull-based re-ingest scheduler over [`ManagedTimer`]/[`Clock`] (MEM-16).
pub struct AutoSyncScheduler<'a, C: Clock> {
    timer: ManagedTimer<'a, C>,
    config: AutoSyncConfig,
    /// FNV-1a per file from the last completed pass (`None` until then).
    last_hashes: Option<HashMap<String, u64>>,
}

impl<'a, C: Clock> AutoSyncScheduler<'a, C> {
    /// Create the scheduler. When enabled, the first pass is due one interval
    /// from now and acts as reconciliation (no baseline yet).
    pub fn new(config: AutoSyncConfig, clock: &'a C) -> Self {
        let mut timer = ManagedTimer::new("auto-sync", clock);
        if config.enabled {
            timer.schedule(config.interval_ms, Box::new(|| {}));
        }
        Self {
            timer,
            config,
            last_hashes: None,
        }
    }

    pub fn config(&self) -> &AutoSyncConfig {
        &self.config
    }

    /// One pull step. At most one re-ingest build fires per due tick.
    #[allow(clippy::too_many_arguments)]
    pub fn tick<R: LlmRunner>(
        &mut self,
        store: &vantadb::wiki::WikiStore<'_>,
        namespace: &str,
        slug: &str,
        root: &Path,
        runner: Option<&R>,
        ingest_config: &IngestConfig,
        progress: Option<&ProgressTracker>,
    ) -> Result<TickOutcome, IngestError> {
        if !self.config.enabled || !self.timer.poll() {
            return Ok(TickOutcome::Idle);
        }
        self.reschedule();

        let wiki = store
            .get(namespace, slug)?
            .ok_or_else(|| IngestError::NotFound {
                namespace: namespace.to_string(),
                slug: slug.to_string(),
            })?;
        if wiki.state.is_busy() {
            // MEM-28 busy guard. Hashes stay stale on purpose: after the
            // current build finishes, the next pass still sees the change.
            return Ok(TickOutcome::Busy);
        }
        let hashes = hash_sources(root)?;
        if self.last_hashes.as_ref() == Some(&hashes) {
            return Ok(TickOutcome::Idle);
        }
        let report = worker::run_with_progress(
            store,
            namespace,
            slug,
            root,
            runner,
            ingest_config,
            progress,
        )?;
        self.last_hashes = Some(hashes);
        Ok(TickOutcome::Reingested {
            run_id: report.run_id,
        })
    }

    fn reschedule(&mut self) {
        self.timer
            .schedule(self.config.interval_ms, Box::new(|| {}));
    }
}

/// Per-file FNV-1a hash of every scanned source (`rel_path → hash`).
///
/// Uses [`vantadb::wiki::scan_local_sources`], so traversal guards and the
/// 28k char budget apply identically to what the worker would ingest.
fn hash_sources(root: &Path) -> Result<HashMap<String, u64>, IngestError> {
    Ok(vantadb::wiki::scan_local_sources(root)?
        .into_iter()
        .map(|f| (f.rel_path, fnv1a(f.content.as_bytes())))
        .collect())
}

/// FNV-1a 64-bit (stdlib-only change fingerprint).
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::abstractions::LlmError;
    use crate::ingest::callback::{IngestPhase, IngestProgress};
    use crate::utils::managed_timer::FakeClock;

    const NS: &str = "default";
    const SLUG: &str = "team-wiki";

    struct NeverRunner;
    impl LlmRunner for NeverRunner {
        fn run(
            &self,
            _params: &crate::core::abstractions::LlmRunParams,
        ) -> Result<String, LlmError> {
            Err(LlmError::NotConfigured)
        }
    }

    fn in_memory_engine() -> vantadb::storage::StorageEngine {
        let config = vantadb::config::VantaConfig {
            backend_kind: vantadb::storage::BackendKind::InMemory,
            read_only: false,
            ..vantadb::config::VantaConfig::default()
        };
        vantadb::storage::StorageEngine::open_with_config(":memory:", Some(config))
            .expect("open in-memory engine")
    }

    fn source_dir(files: &[(&str, &str)]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        for (name, content) in files {
            std::fs::write(dir.path().join(name), content).expect("write source");
        }
        dir
    }

    /// Drive the wiki to `ready` (created wikis start `pending`, i.e. busy).
    fn ready_wiki(store: &vantadb::wiki::WikiStore<'_>) -> String {
        store.create(NS, SLUG).expect("create");
        let wiki = store.begin_processing(NS, SLUG).expect("begin");
        let run_id = wiki.run_id.clone().expect("run_id");
        store.complete(NS, SLUG, &run_id).expect("complete");
        run_id
    }

    // ── (a) el intervalo dispara re-ingest al detectar cambio ──

    #[test]
    fn due_interval_triggers_re_ingest_on_change() {
        let src = source_dir(&[("notes.md", "# notes\nhello world")]);
        let engine = in_memory_engine();
        let store = vantadb::wiki::WikiStore::new(&engine);
        ready_wiki(&store);
        let clock = FakeClock::new(1_000);
        let mut sched = AutoSyncScheduler::new(AutoSyncConfig::new(true, MIN_INTERVAL_MS), &clock);

        // Not due yet.
        assert_eq!(
            sched
                .tick(
                    &store,
                    NS,
                    SLUG,
                    src.path(),
                    None::<&NeverRunner>,
                    &IngestConfig::default(),
                    None
                )
                .expect("tick"),
            TickOutcome::Idle
        );

        clock.advance(MIN_INTERVAL_MS);
        // First due pass has no baseline → reconciling re-ingest.
        let first = sched
            .tick(
                &store,
                NS,
                SLUG,
                src.path(),
                None::<&NeverRunner>,
                &IngestConfig::default(),
                None,
            )
            .expect("tick");
        assert!(
            matches!(&first, TickOutcome::Reingested { run_id } if !run_id.is_empty()),
            "expected Reingested, got {first:?}"
        );

        clock.advance(MIN_INTERVAL_MS);
        // Due again but nothing changed since the baseline → idle.
        assert_eq!(
            sched
                .tick(
                    &store,
                    NS,
                    SLUG,
                    src.path(),
                    None::<&NeverRunner>,
                    &IngestConfig::default(),
                    None
                )
                .expect("tick"),
            TickOutcome::Idle
        );

        // A file change is detected on the next interval.
        std::fs::write(src.path().join("notes.md"), "# notes\nCHANGED").expect("rewrite");
        clock.advance(MIN_INTERVAL_MS);
        let again = sched
            .tick(
                &store,
                NS,
                SLUG,
                src.path(),
                None::<&NeverRunner>,
                &IngestConfig::default(),
                None,
            )
            .expect("tick");
        assert!(matches!(again, TickOutcome::Reingested { .. }));
    }

    // ── (b) busy guard respeta pending/processing ──

    #[test]
    fn busy_guard_skips_re_ingest_and_keeps_hashes_stale() {
        let src = source_dir(&[("notes.md", "# notes\nhello world")]);
        let engine = in_memory_engine();
        let store = vantadb::wiki::WikiStore::new(&engine);
        store.create(NS, SLUG).expect("create"); // `pending` == busy (MEM-28)
        let clock = FakeClock::new(0);
        let mut sched = AutoSyncScheduler::new(AutoSyncConfig::new(true, MIN_INTERVAL_MS), &clock);
        clock.advance(MIN_INTERVAL_MS);

        let outcome = sched
            .tick(
                &store,
                NS,
                SLUG,
                src.path(),
                None::<&NeverRunner>,
                &IngestConfig::default(),
                None,
            )
            .expect("tick");

        assert_eq!(outcome, TickOutcome::Busy, "busy wiki must not re-ingest");
        let wiki = store.get(NS, SLUG).expect("get").expect("wiki");
        assert_eq!(wiki.state, vantadb::wiki::WikiState::Pending);
        // Hashes untouched: once the build finishes, the next pass still sees
        // the change (stale-baseline reconciliation).
        assert!(sched.last_hashes.is_none());
    }

    // ── (c) disabled by default ──

    #[test]
    fn disabled_by_default_is_a_full_no_op() {
        assert!(!AutoSyncConfig::default().enabled);

        let src = source_dir(&[("notes.md", "# notes\nhello world")]);
        let engine = in_memory_engine();
        let store = vantadb::wiki::WikiStore::new(&engine);
        ready_wiki(&store);
        let clock = FakeClock::new(0);
        let mut sched = AutoSyncScheduler::new(AutoSyncConfig::default(), &clock);
        clock.advance(10 * MIN_INTERVAL_MS); // way past any deadline

        assert_eq!(
            sched
                .tick(
                    &store,
                    NS,
                    SLUG,
                    src.path(),
                    None::<&NeverRunner>,
                    &IngestConfig::default(),
                    None
                )
                .expect("tick"),
            TickOutcome::Idle,
            "disabled scheduler must never re-ingest"
        );
        let wiki = store.get(NS, SLUG).expect("get").expect("wiki");
        assert_eq!(wiki.state, vantadb::wiki::WikiState::Ready);
    }

    // ── (d) run_id fresco por build; paquetes tardíos descartados ──

    #[test]
    fn each_build_gets_a_fresh_run_id_and_stale_packets_are_discarded() {
        let src = source_dir(&[("notes.md", "# notes\nhello world")]);
        let engine = in_memory_engine();
        let store = vantadb::wiki::WikiStore::new(&engine);
        ready_wiki(&store);
        let tracker = ProgressTracker::new();
        let clock = FakeClock::new(0);
        let mut sched = AutoSyncScheduler::new(AutoSyncConfig::new(true, MIN_INTERVAL_MS), &clock);

        clock.advance(MIN_INTERVAL_MS);
        let first = sched
            .tick(
                &store,
                NS,
                SLUG,
                src.path(),
                None::<&NeverRunner>,
                &IngestConfig::default(),
                Some(&tracker),
            )
            .expect("tick");
        let TickOutcome::Reingested { run_id: run1 } = first else {
            panic!("expected Reingested");
        };

        std::fs::write(src.path().join("notes.md"), "# notes\nv2").expect("rewrite");
        clock.advance(MIN_INTERVAL_MS);
        let second = sched
            .tick(
                &store,
                NS,
                SLUG,
                src.path(),
                None::<&NeverRunner>,
                &IngestConfig::default(),
                Some(&tracker),
            )
            .expect("tick");
        let TickOutcome::Reingested { run_id: run2 } = second else {
            panic!("expected Reingested");
        };

        assert_ne!(run1, run2, "each build mints a fresh run_id (MEM-31)");
        // Late packet from the superseded build is discarded by the tracker;
        // only the active run reports progress.
        let stale = IngestProgress::new(&run1, IngestPhase::Extracting, 10, 0, 0, 0);
        assert!(!tracker.update_progress(stale), "old-run packet rejected");
        assert_eq!(tracker.wiki_status(&run1), None);
        let fresh = IngestProgress::new(&run2, IngestPhase::Indexing, 5, 5, 0, 0);
        assert!(tracker.update_progress(fresh));
        assert!(tracker.wiki_status(&run2).is_some());
    }

    #[test]
    fn interval_is_clamped_to_the_minimum() {
        assert_eq!(
            AutoSyncConfig::new(true, 1_000).interval_ms,
            MIN_INTERVAL_MS
        );
        assert_eq!(
            AutoSyncConfig::new(false, 0).interval_ms,
            MIN_INTERVAL_MS,
            "zero/negative intervals clamp up (re-ingest storm guard)"
        );
    }
}
