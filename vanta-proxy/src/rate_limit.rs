//! In-process rate limiter (D24: no Redis, D35: default 60 req/min).
//!
//! Sliding window of 60s keyed by `spaceId×model` (TDAM parity:
//! redis-store.ts:324-326 bucket dimension). Thread-safe via a single Mutex
//! over the window map; fail-open when the mechanism is degraded (TDAM
//! guard.ts:40-51): degraded → allow + warn log, never block the wire.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::Body;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use serde_json::json;

/// Sliding window size (TDAM parity).
pub const WINDOW_MS: u64 = 60_000;

/// Outcome of one rate-limit check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateDecision {
    Allowed { remaining: u32 },
    Limited { retry_after_secs: u64 },
}

struct Bucket {
    hits: VecDeque<u64>,
}

/// In-process sliding-window limiter. Local-first by design (D37 accepted:
/// state is lost across instances — single-instance deployment).
pub struct RateLimiter {
    limit: u32,
    window_ms: u64,
    /// Conscience flag (D24/TDAM guard.ts): true → allow everything + warn.
    degraded: AtomicBool,
    buckets: Mutex<HashMap<String, Bucket>>,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn bucket_key(space_id: &str, model: &str) -> String {
    // Unit separator: cannot appear in header-derived ids.
    format!("{space_id}\u{1f}{model}")
}

impl RateLimiter {
    pub fn new(limit: u32) -> Self {
        Self {
            limit,
            window_ms: WINDOW_MS,
            degraded: AtomicBool::new(false),
            buckets: Mutex::new(HashMap::new()),
        }
    }

    pub fn set_degraded(&self, degraded: bool) {
        self.degraded.store(degraded, Ordering::Relaxed);
    }

    pub fn is_degraded(&self) -> bool {
        self.degraded.load(Ordering::Relaxed)
    }

    /// Consume one slot for (`space_id`, `model`) if under the limit.
    ///
    /// Fail-open contract (guard.ts:40-51): a degraded mechanism allows the
    /// request through with a warning instead of failing the wire.
    pub fn check(&self, space_id: &str, model: &str) -> RateDecision {
        if self.is_degraded() {
            tracing::warn!(
                space_id = %space_id,
                model = %model,
                "rate-limiter degraded — allowing request (fail-open)"
            );
            return RateDecision::Allowed {
                remaining: self.limit,
            };
        }

        let now = now_ms();
        // Poison recovery: a panicking holder must not take down limiting
        // forever; recover the guard and continue (fail-conscious, not
        // fail-closed).
        let mut buckets = match self.buckets.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let bucket = buckets
            .entry(bucket_key(space_id, model))
            .or_insert_with(|| Bucket {
                hits: VecDeque::new(),
            });

        while let Some(&ts) = bucket.hits.front() {
            if now.saturating_sub(ts) < self.window_ms {
                break;
            }
            bucket.hits.pop_front();
        }

        if bucket.hits.len() as u32 >= self.limit {
            // Oldest hit leaves the window at ts + window → retry then.
            let oldest = *bucket.hits.front().unwrap_or(&now);
            let retry_after_secs = ((oldest + self.window_ms).saturating_sub(now) / 1000).max(1);
            return RateDecision::Limited { retry_after_secs };
        }

        bucket.hits.push_back(now);
        RateDecision::Allowed {
            remaining: self.limit - bucket.hits.len() as u32,
        }
    }
}

fn header_value(value: &str) -> axum::http::HeaderValue {
    value
        .parse()
        .unwrap_or_else(|_| axum::http::HeaderValue::from_static("0"))
}

/// Build the 429 response with `Retry-After` + `x-ratelimit-*` headers
/// (TDAM parity: guard.ts:111-121).
pub fn limited_response(
    space_id: &str,
    model: &str,
    limit: u32,
    decision: RateDecision,
) -> axum::response::Response<Body> {
    let retry_after_secs = match decision {
        RateDecision::Limited { retry_after_secs } => retry_after_secs,
        RateDecision::Allowed { .. } => 0,
    };
    let body = json!({
        "error": {
            "type": "rate_limit_error",
            "message": format!(
                "rate limit exceeded for space `{space_id}` × model `{model}` ({limit} req/min); retry in {retry_after_secs}s"
            )
        }
    });
    let mut headers = HeaderMap::new();
    headers.insert("retry-after", header_value(&retry_after_secs.to_string()));
    headers.insert("x-ratelimit-limit", header_value(&limit.to_string()));
    headers.insert("x-ratelimit-remaining", header_value("0"));
    headers.insert(
        "x-ratelimit-reset",
        header_value(&retry_after_secs.to_string()),
    );
    (StatusCode::TOO_MANY_REQUESTS, headers, axum::Json(body)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::time::Duration;

    #[test]
    fn sliding_window_blocks_excess_and_sets_retry_after() {
        let rl = RateLimiter::new(3);
        assert!(matches!(
            rl.check("sp1", "gpt-x"),
            RateDecision::Allowed { remaining: 2 }
        ));
        assert!(matches!(
            rl.check("sp1", "gpt-x"),
            RateDecision::Allowed { remaining: 1 }
        ));
        assert!(matches!(
            rl.check("sp1", "gpt-x"),
            RateDecision::Allowed { remaining: 0 }
        ));
        assert!(matches!(
            rl.check("sp1", "gpt-x"),
            RateDecision::Limited {
                retry_after_secs: 1..
            }
        ));
    }

    #[test]
    fn limited_response_carries_429_retry_after_and_ratelimit_headers() {
        let rl = RateLimiter::new(1);
        let _ = rl.check("s", "m");
        let decision = rl.check("s", "m");
        let resp = limited_response("s", "m", 1, decision);
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        let h = resp.headers();
        let retry_after = h["retry-after"]
            .to_str()
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);
        assert!(
            (1..=60).contains(&retry_after),
            "retry-after must be within the window, got {retry_after}"
        );
        assert_eq!(h["x-ratelimit-limit"], "1");
        assert_eq!(h["x-ratelimit-remaining"], "0");
        assert_eq!(h["x-ratelimit-reset"], h["retry-after"]);
    }

    #[test]
    fn concurrent_threads_cannot_overshoot_the_window() {
        let rl = Arc::new(RateLimiter::new(5));
        const THREADS: usize = 8;
        const CHECKS: usize = 50;
        let mut handles = Vec::with_capacity(THREADS);
        for _ in 0..THREADS {
            let rl = Arc::clone(&rl);
            handles.push(std::thread::spawn(move || {
                let mut allowed = 0usize;
                for _ in 0..CHECKS {
                    if matches!(rl.check("shared", "model-a"), RateDecision::Allowed { .. }) {
                        allowed += 1;
                    }
                }
                allowed
            }));
        }
        let total: usize = handles.into_iter().map(|h| h.join().expect("thread")).sum();
        assert_eq!(total, 5, "sliding window admitted exactly the limit");
    }

    #[test]
    fn fail_open_when_degraded_allows_over_limit_with_flag_set() {
        let rl = RateLimiter::new(2);
        rl.set_degraded(true);
        assert!(rl.is_degraded());
        for _ in 0..10 {
            assert!(matches!(rl.check("d", "m"), RateDecision::Allowed { .. }));
        }
        // Recovering the conscience restores enforcement.
        rl.set_degraded(false);
        assert!(matches!(rl.check("d", "m"), RateDecision::Allowed { .. }));
        assert!(matches!(rl.check("d", "m"), RateDecision::Allowed { .. }));
        assert!(matches!(
            rl.check("d", "m"),
            RateDecision::Limited {
                retry_after_secs: 1..
            }
        ));
    }

    #[test]
    fn different_space_or_model_gets_independent_bucket() {
        let rl = RateLimiter::new(1);
        assert!(matches!(rl.check("a", "m1"), RateDecision::Allowed { .. }));
        assert!(matches!(rl.check("a", "m1"), RateDecision::Limited { .. }));
        assert!(matches!(rl.check("a", "m2"), RateDecision::Allowed { .. }));
        assert!(matches!(rl.check("b", "m1"), RateDecision::Allowed { .. }));
    }

    #[test]
    fn window_expiry_frees_slots_without_sleeping_the_test_clock() {
        // Direct clock manipulation via a short window keeps this test fast
        // and deterministic: 20ms window, wait 30ms.
        let mut rl = RateLimiter::new(1);
        rl.window_ms = 20;
        assert!(matches!(rl.check("t", "m"), RateDecision::Allowed { .. }));
        assert!(matches!(rl.check("t", "m"), RateDecision::Limited { .. }));
        std::thread::sleep(Duration::from_millis(30));
        assert!(matches!(rl.check("t", "m"), RateDecision::Allowed { .. }));
    }
}
