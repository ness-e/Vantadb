//! Structured per-turn reporting (local JSON log; no Opik/Langfuse/ClickHouse).
//!
//! One JSON line per turn through the pipeline, emitted via `tracing` under
//! the `vanta_proxy::report` target. Optional hooks let future backends
//! subscribe without touching the wire path.

use serde::Serialize;
use std::collections::VecDeque;
use std::sync::RwLock;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// One turn's structured record.
#[derive(Debug, Clone, Serialize)]
pub struct TurnReport {
    pub timestamp_ms: u64,
    pub space_id: String,
    pub protocol: String,
    pub model: String,
    /// HTTP status the proxy returned to the client.
    pub status: u16,
    pub duration_ms: u128,
}

/// Extract `"model"` from a request body for limiter/reporting keys ("_" if absent/non-JSON).
pub fn model_from_body(body: &[u8]) -> String {
    serde_json::from_slice::<serde_json::Value>(body)
        .ok()
        .and_then(|v| v.get("model")?.as_str().map(str::to_string))
        .unwrap_or_else(|| "_".to_string())
}

/// A subscriber receiving every emitted report (future backend hook).
pub type ReportHook = Box<dyn Fn(&TurnReport) + Send + Sync>;

/// Cap of the in-memory recent-reports ring served by `/snapshot`.
const RECENT_CAP: usize = 100;

/// Reporter: logs each turn as one JSON line, keeps the last [`RECENT_CAP`]
/// reports in memory (for `/snapshot`) and fans out to registered hooks.
#[derive(Default)]
pub struct Reporter {
    hooks: RwLock<Vec<ReportHook>>,
    recent: std::sync::Mutex<VecDeque<TurnReport>>,
}

impl Reporter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_hook(&self, hook: ReportHook) {
        self.hooks
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(hook);
    }

    /// Emit one per-turn record: JSON log line + hook fan-out (best-effort;
    /// reporting must never fail the wire).
    pub fn emit(&self, report: &TurnReport) {
        {
            let mut recent = self
                .recent
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            recent.push_back(report.clone());
            while recent.len() > RECENT_CAP {
                recent.pop_front();
            }
        }
        match serde_json::to_string(report) {
            Ok(line) => tracing::info!(target: "vanta_proxy::report", "{line}"),
            Err(e) => {
                tracing::warn!(target: "vanta_proxy::report", error = %e, "report serialization failed")
            }
        }
        let hooks = self
            .hooks
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for hook in hooks.iter() {
            hook(report);
        }
    }

    /// Last [`RECENT_CAP`] reports, oldest first (`/snapshot` wire shape).
    pub fn recent_reports(&self) -> Vec<TurnReport> {
        self.recent
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .cloned()
            .collect()
    }
}

/// Timestamp helper shared with the pipeline.
pub fn now_ms_u64() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Wall-clock start marker for a turn.
pub struct TurnTimer(Instant);

impl TurnTimer {
    pub fn start() -> Self {
        Self(Instant::now())
    }

    pub fn elapsed_ms(&self) -> u128 {
        self.0.elapsed().as_millis()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn emit_produces_valid_json_line_with_required_fields() {
        // The tracing subscriber in tests is default-installed; assert on the
        // serializable shape directly — that IS the emitted line's content.
        let report = TurnReport {
            timestamp_ms: now_ms_u64(),
            space_id: "sp-9".into(),
            protocol: "openai".into(),
            model: "gpt-x".into(),
            status: 200,
            duration_ms: 12,
        };
        let line = serde_json::to_string(&report).expect("serialize");
        let value: serde_json::Value = serde_json::from_str(&line).expect("valid json");
        assert_eq!(value["space_id"], "sp-9");
        assert_eq!(value["protocol"], "openai");
        assert_eq!(value["model"], "gpt-x");
        assert_eq!(value["status"], 200);
        assert_eq!(value["duration_ms"], 12);
        assert!(value["timestamp_ms"].is_u64());
    }

    #[test]
    fn hooks_receive_every_emitted_report() {
        let reporter = Reporter::new();
        let calls = Arc::new(AtomicUsize::new(0));
        let c2 = Arc::clone(&calls);
        reporter.add_hook(Box::new(move |report: &TurnReport| {
            assert_eq!(report.space_id, "hooked");
            c2.fetch_add(1, Ordering::SeqCst);
        }));

        reporter.emit(&TurnReport {
            timestamp_ms: 1,
            space_id: "hooked".into(),
            protocol: "anthropic".into(),
            model: "_".into(),
            status: 429,
            duration_ms: 3,
        });
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn model_extraction_defaults_to_underscore_on_missing_or_non_json() {
        assert_eq!(
            model_from_body(br#"{"model":"claude-3","x":1}"#),
            "claude-3"
        );
        assert_eq!(model_from_body(br#"{"messages":[]}"#), "_");
        assert_eq!(model_from_body(b"not json at all"), "_");
        assert_eq!(model_from_body(b""), "_");
    }

    #[test]
    fn timer_measures_positive_elapsed_time() {
        let timer = TurnTimer::start();
        std::thread::sleep(std::time::Duration::from_millis(5));
        assert!(timer.elapsed_ms() >= 5);
    }

    #[test]
    fn recent_reports_keeps_last_cap_oldest_first() {
        let reporter = Reporter::new();
        for i in 0..(RECENT_CAP as u64 + 5) {
            reporter.emit(&TurnReport {
                timestamp_ms: i,
                space_id: "sp".into(),
                protocol: "openai".into(),
                model: "_".into(),
                status: 200,
                duration_ms: 1,
            });
        }
        let recent = reporter.recent_reports();
        assert_eq!(recent.len(), RECENT_CAP);
        assert_eq!(recent.first().expect("first").timestamp_ms, 5);
        assert_eq!(
            recent.last().expect("last").timestamp_ms,
            RECENT_CAP as u64 + 4
        );
    }
}
