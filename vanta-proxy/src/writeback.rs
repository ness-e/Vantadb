//! L0 write-back: fire-and-forget tracking + retry + graceful flush.
//!
//! TDAM parity (`pending-writes.ts`): streaming responses cannot await the
//! L0 write, so writes run detached with bounded retry (3 attempts,
//! 500ms→1s→2s backoff — `withL0Retry`). A write that exhausts its retries
//! lands in a pending queue persisted to disk; a graceful shutdown
//! (SIGTERM/SIGINT) drains it before exit.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::json;

/// A boxed retryable L0 write job. Re-runnable so a graceful-shutdown flush
/// can replay failures.
pub type L0Job = Arc<dyn Fn() -> L0Future + Send + Sync>;
pub type L0Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>>;

pub(crate) const DEFAULT_ATTEMPTS: u32 = 3;
pub(crate) const DEFAULT_BASE_MS: u64 = 500;

struct PendingEntry {
    label: String,
    job: L0Job,
}

/// Shared write-back coordinator (cheap to clone).
#[derive(Clone)]
pub struct WriteBack {
    inner: Arc<Inner>,
}

struct Inner {
    pending: Mutex<Vec<PendingEntry>>,
    /// Best-effort persistence of failed-write labels across restarts.
    persist_path: Option<PathBuf>,
}

impl WriteBack {
    pub fn new(persist_path: Option<PathBuf>) -> Self {
        Self {
            inner: Arc::new(Inner {
                pending: Mutex::new(Vec::new()),
                persist_path,
            }),
        }
    }

    /// Fire-and-forget tracked write (TDAM `trackWrite` + `withL0Retry`):
    /// spawns a task running `job` with 3 attempts and exponential backoff
    /// (500ms→1s→2s). On final failure the job is queued for the shutdown
    /// flush and its label persisted to disk.
    pub fn track(&self, label: impl Into<String>, job: L0Job) {
        let this = self.clone();
        let label = label.into();
        tokio::spawn(async move {
            if with_l0_retry(|| job(), DEFAULT_ATTEMPTS, DEFAULT_BASE_MS)
                .await
                .is_err()
            {
                tracing::error!(write = %label, "L0 write failed after retries — queued for flush");
                this.enqueue(PendingEntry { label, job });
            }
        });
    }

    fn enqueue(&self, entry: PendingEntry) {
        self.inner
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(entry);
        self.persist();
    }

    /// Number of pending (failed) writes awaiting flush.
    pub fn pending_count(&self) -> usize {
        self.inner
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }

    /// Labels of the pending writes, oldest first (`/snapshot` audit view).
    pub fn pending_labels(&self) -> Vec<String> {
        self.inner
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .map(|e| e.label.clone())
            .collect()
    }

    /// Persist pending labels as JSON lines (best-effort; never blocks or
    /// fails the wire). The closures themselves are not serializable — the
    /// file is an audit trail of what was lost on a hard crash.
    fn persist(&self) {
        let Some(path) = &self.inner.persist_path else {
            return;
        };
        let (body, count) = {
            let pending = self
                .inner
                .pending
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let body: Vec<String> = pending.iter().map(|e| e.label.clone()).collect();
            (
                serde_json::to_string(&json!({ "pending": body })).unwrap_or_default(),
                body.len(),
            )
        };
        // ponytail: full-file rewrite per failure — fine at proxy scale;
        // switch to append-only log if failure rates ever grow.
        match std::fs::write(path, body) {
            Ok(()) => tracing::debug!(path = %path.display(), count, "pending queue persisted"),
            Err(e) => {
                tracing::warn!(path = %path.display(), error = %e, "could not persist pending queue")
            }
        }
    }

    /// Drain pending writes once each before `deadline` elapses (graceful
    /// shutdown / SIGTERM-SIGINT handler). Successful jobs leave the queue;
    /// still-failing ones stay persisted for the next start.
    pub async fn flush(&self, deadline: Duration) -> usize {
        let snapshot: Vec<PendingEntry> = {
            let mut pending = self
                .inner
                .pending
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            std::mem::take(&mut *pending)
        };
        if snapshot.is_empty() {
            return 0;
        }
        let drained = tokio::time::timeout(deadline, async {
            let mut ok_labels = Vec::new();
            for entry in &snapshot {
                if (entry.job)().await.is_ok() {
                    ok_labels.push(entry.label.clone());
                    tracing::info!(write = %entry.label, "flushed pending L0 write");
                }
            }
            ok_labels
        })
        .await;

        let ok: Vec<String> = drained.unwrap_or_default();
        let remaining: Vec<PendingEntry> = snapshot
            .into_iter()
            .filter(|e| !ok.contains(&e.label))
            .collect();
        let n = remaining.len();
        {
            let mut pending = self
                .inner
                .pending
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            *pending = remaining;
        }
        if n > 0 {
            self.persist();
        } else if let Some(path) = &self.inner.persist_path {
            let _ = std::fs::remove_file(path);
        }
        tracing::info!(
            drained = ok.len(),
            remaining = n,
            "write-back flush complete"
        );
        n
    }
}

/// Bounded exponential-backoff retry (TDAM `withL0Retry`): `attempts`
/// attempts, sleeping `base_ms << i` between them (500→1000→2000 by
/// default). Every failure is retried — callers pass only retryable work.
pub async fn with_l0_retry<F, Fut>(mut f: F, attempts: u32, base_ms: u64) -> Result<(), String>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<(), String>>,
{
    let mut last_err = String::new();
    for attempt in 0..attempts {
        match f().await {
            Ok(()) => return Ok(()),
            Err(e) => {
                last_err = e;
                tracing::warn!(attempt = attempt + 1, attempts, "L0 write attempt failed");
                if attempt + 1 < attempts {
                    tokio::time::sleep(Duration::from_millis(base_ms << attempt)).await;
                }
            }
        }
    }
    Err(last_err)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    fn job_from_counter(counter: Arc<AtomicU32>, fail_times: u32) -> L0Job {
        Arc::new(move || {
            let c = Arc::clone(&counter);
            Box::pin(async move {
                if c.fetch_add(1, Ordering::SeqCst) < fail_times {
                    Err("upstream 503".to_string())
                } else {
                    Ok(())
                }
            }) as L0Future
        })
    }

    #[tokio::test]
    async fn retry_backoff_recovers_on_third_attempt_with_expected_wait() {
        let counter = Arc::new(AtomicU32::new(0));
        let job = job_from_counter(Arc::clone(&counter), 2);
        let started = std::time::Instant::now();
        // base 10ms → waits 10ms + 20ms between the three attempts.
        let result = with_l0_retry(|| job(), 3, 10).await;
        assert!(result.is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 3);
        // Two backoffs of 10ms+20ms must have elapsed (≥30ms total).
        assert!(started.elapsed() >= Duration::from_millis(30));
    }

    #[tokio::test]
    async fn retry_exhaustion_returns_last_error() {
        let counter = Arc::new(AtomicU32::new(0));
        let job = job_from_counter(Arc::clone(&counter), u32::MAX);
        let result = with_l0_retry(|| job(), 3, 1).await;
        assert_eq!(result.unwrap_err(), "upstream 503");
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn track_enqueues_after_retry_exhaustion_and_persists() {
        let dir = std::env::temp_dir().join(format!("vanta-wb-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("tmpdir");
        let path = dir.join("pending.json");
        let wb = WriteBack::new(Some(path.clone()));
        let counter = Arc::new(AtomicU32::new(0));

        let c2 = Arc::clone(&counter);
        let failing: L0Job = Arc::new(move || {
            let c = Arc::clone(&c2);
            Box::pin(async move {
                c.fetch_add(1, Ordering::SeqCst);
                Err::<(), String>("always fails".into())
            }) as L0Future
        });
        wb.track("turn-42", failing);

        // Wait for the spawned task to exhaust retries (base is 500ms → ~1.5s).
        tokio::time::sleep(Duration::from_millis(2500)).await;
        assert_eq!(wb.pending_count(), 1, "failed write queued");
        assert!(counter.load(Ordering::SeqCst) >= 3, "three attempts made");

        let persisted = std::fs::read_to_string(&path).expect("persisted file");
        assert!(
            persisted.contains("turn-42"),
            "label persisted: {persisted}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn flush_drains_pending_queue_within_deadline() {
        // No persist path → in-memory only.
        let wb = WriteBack::new(None);
        let calls = Arc::new(AtomicU32::new(0));
        let c2 = Arc::clone(&calls);
        let now_ok: L0Job = Arc::new(move || {
            let c = Arc::clone(&c2);
            Box::pin(async move {
                c.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }) as L0Future
        });
        wb.enqueue(PendingEntry {
            label: "queued-turn".into(),
            job: now_ok,
        });
        assert_eq!(wb.pending_count(), 1);

        let remaining = wb.flush(Duration::from_secs(5)).await;
        assert_eq!(remaining, 0, "healthy job drained");
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(wb.pending_count(), 0);
    }

    #[tokio::test]
    async fn flush_keeps_still_failing_jobs_for_next_start() {
        let wb = WriteBack::new(None);
        let dead: L0Job = Arc::new(|| Box::pin(async { Err("down".into()) }) as L0Future);
        wb.enqueue(PendingEntry {
            label: "doomed".into(),
            job: dead,
        });
        let remaining = wb.flush(Duration::from_secs(5)).await;
        assert_eq!(remaining, 1);
        assert_eq!(wb.pending_count(), 1, "still-failing write stays queued");
    }

    #[test]
    fn default_backoff_schedule_matches_tdam_500_1000_2000() {
        // Contract check without waiting: base<<i for i=0..2.
        assert_eq!(
            [0u64, 1, 2].map(|i| DEFAULT_BASE_MS << i),
            [500, 1000, 2000]
        );
        assert_eq!(DEFAULT_ATTEMPTS, 3);
    }
}
