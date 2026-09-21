//! Verbatim forwarding engine: strip hop-by-hop headers, forward bytes
//! unmodified, stream the upstream response back without buffering.

use std::time::Duration;

use axum::body::Body;
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, Response};
use bytes::Bytes;
use reqwest::Client;

use crate::config::UpstreamConfig;
use crate::error::ProxyError;

/// Base backoff between PRX-02 failover attempts (decisión 4).
pub const FAILOVER_BASE_BACKOFF_MS: u64 = 100;
/// Cap for the exponential backoff (decisión 4).
pub const FAILOVER_MAX_BACKOFF_MS: u64 = 2000;
/// Consecutive failures after which an upstream is skipped (decisión 5 —
/// mirrors `DEGRADED_ENTER_FAILURES` in rate_limit.rs).
pub const FAILOVER_SKIP_AFTER_FAILURES: u32 = 3;

/// True when an upstream HTTP status should trigger failover to the next
/// upstream: 429 (quota) or 5xx (broken). Same partition as
/// `UpstreamHealth::observe` — other 4xx is a client error, retrying
/// elsewhere won't help. Pure and total.
pub fn is_retryable_status(status: u16) -> bool {
    status == 429 || (500..600).contains(&status)
}

/// Exponential backoff for failover attempt `attempt` (0-based):
/// `min(base_ms * 2^attempt, max_ms)`, saturating. Pure — exact values
/// unit-tested. No jitter by design (decisión 4: determinism).
/// `ponytail:` no jitter — add `+/−25%` if real traffic shows thundering herd.
pub fn backoff_delay(attempt: u32, base_ms: u64, max_ms: u64) -> Duration {
    let scaled = base_ms.saturating_mul(1u64 << attempt.min(31));
    Duration::from_millis(scaled.min(max_ms))
}

/// Per-upstream consecutive-failure counters (PRX-02 decisión 5). Hysteresis
/// without timers: skip at ≥3 consecutive failures, reset on one success,
/// fail-open (try all) when everything is skipped so a full outage still
/// surfaces the real upstream status instead of a synthetic error.
#[derive(Debug, Default)]
pub struct UpstreamHealthSet {
    failures: Vec<u32>,
}

impl UpstreamHealthSet {
    /// Counter set sized for `n` upstreams; auto-grows on out-of-range index.
    pub fn new(n: usize) -> Self {
        Self {
            failures: vec![0; n],
        }
    }

    /// True when upstream `i` should be skipped this round.
    pub fn should_skip(&self, i: usize) -> bool {
        self.failures
            .get(i)
            .is_some_and(|&f| f >= FAILOVER_SKIP_AFTER_FAILURES)
    }

    /// True when every upstream is skipped — caller must fail-open.
    pub fn all_skipped(&self, n: usize) -> bool {
        n > 0 && (0..n).all(|i| self.should_skip(i))
    }

    /// One success clears the streak (fast recovery on probe).
    pub fn note_success(&mut self, i: usize) {
        if let Some(slot) = self.failures.get_mut(i) {
            *slot = 0;
        }
    }

    /// One failure (retryable status or transport error) extends the streak.
    pub fn note_failure(&mut self, i: usize) {
        if self.failures.len() <= i {
            self.failures.resize(i + 1, 0);
        }
        self.failures[i] = self.failures[i].saturating_add(1);
    }
}

/// Headers that must not be forwarded (RFC 9110 §7.6.1) plus framing headers
/// the HTTP client manages itself.
const HOP_BY_HOP: &[&str] = &[
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailer",
    "trailers",
    "transfer-encoding",
    "upgrade",
    // Framing / routing headers owned by the forwarder:
    "host",
    "content-length",
    // Proxy-consumed credential (D34): the internal user key must never leak
    // to the upstream.
    "x-vanta-user-key",
];

fn is_hop_by_hop(name: &str) -> bool {
    HOP_BY_HOP.iter().any(|h| name.eq_ignore_ascii_case(h))
}

fn filter_headers(src: &HeaderMap, out: &mut reqwest::header::HeaderMap) {
    for (name, value) in src {
        if !is_hop_by_hop(name.as_str()) {
            if let (Ok(n), Ok(v)) = (
                reqwest::header::HeaderName::from_bytes(name.as_str().as_bytes()),
                reqwest::header::HeaderValue::from_bytes(value.as_bytes()),
            ) {
                out.insert(n, v);
            }
        }
    }
}

fn response_headers_into_axum(src: &reqwest::header::HeaderMap) -> axum::http::HeaderMap {
    let mut out = axum::http::HeaderMap::with_capacity(src.len());
    for (name, value) in src {
        if is_hop_by_hop(name.as_str()) {
            continue;
        }
        if let (Ok(n), Ok(v)) = (
            HeaderName::from_bytes(name.as_str().as_bytes()),
            HeaderValue::from_bytes(value.as_bytes()),
        ) {
            out.insert(n, v);
        }
    }
    out
}

/// Shared forwarder holding the pooled HTTP client plus PRX-02 failover
/// health (per-upstream consecutive-failure counters, shared across clones).
#[derive(Clone)]
pub struct Forwarder {
    client: Client,
    failover_health: std::sync::Arc<std::sync::Mutex<UpstreamHealthSet>>,
}

impl Forwarder {
    /// Build a forwarder with a total request timeout of `cfg.forward_timeout_secs`.
    ///
    /// # Errors
    /// Returns [`ProxyError::Config`] if the client cannot be built.
    pub fn new(cfg: &UpstreamConfig) -> Result<Self, ProxyError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(cfg.forward_timeout_secs))
            // ponytail: client-level timeout caps total stream duration at 600s —
            // streams longer than the timeout are cut; per-phase timeouts if real traffic hits it.
            .build()
            .map_err(|e| ProxyError::Config(format!("http client: {e}")))?;
        Ok(Self {
            client,
            failover_health: std::sync::Arc::new(std::sync::Mutex::new(UpstreamHealthSet::new(1))),
        })
    }

    /// Send one request to one upstream and return the raw response so the
    /// caller can inspect the status before streaming (failover needs the
    /// status without consuming the body).
    async fn send_once(
        &self,
        cfg: &UpstreamConfig,
        method: Method,
        wire_path: &str,
        headers: &HeaderMap,
        body: Bytes,
    ) -> Result<reqwest::Response, ProxyError> {
        let base = cfg.url.trim_end_matches('/');
        let url = format!("{base}{wire_path}");

        let mut upstream_headers = reqwest::header::HeaderMap::new();
        filter_headers(headers, &mut upstream_headers);
        if !cfg.api_key.is_empty() {
            let _ = upstream_headers.insert(
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", cfg.api_key))
                    .map_err(|e| ProxyError::Forward(format!("api key header: {e}")))?,
            );
        }

        self.client
            .request(convert_method(method), url)
            .headers(upstream_headers)
            .body(body)
            .send()
            .await
            .map_err(classify_send)
    }

    /// Forward `body` verbatim to `{base_url}{wire_path}` and return the
    /// upstream response as a streaming axum response (no buffering — SSE-safe).
    ///
    /// # Errors
    /// - [`ProxyError::UpstreamTimeout`] → mapped to 504
    /// - [`ProxyError::UpstreamUnreachable`] → mapped to 502
    pub async fn forward(
        &self,
        cfg: &UpstreamConfig,
        method: Method,
        wire_path: &str,
        headers: &HeaderMap,
        body: Bytes,
    ) -> Result<Response<Body>, ProxyError> {
        let resp = self
            .send_once(cfg, method, wire_path, headers, body)
            .await?;
        into_axum(resp)
    }

    /// PRX-02 failover: try `upstreams` in config order, moving to the next
    /// on 429/5xx or transport error with exponential backoff between
    /// attempts. Unhealthy upstreams (≥3 consecutive failures) are skipped
    /// unless all are skipped (fail-open). The LAST attempt's outcome is
    /// returned verbatim — including a 429/5xx status — so clients see the
    /// real upstream signal, never a synthetic error. Streaming starts only
    /// on the winning response (no buffering — SSE-safe).
    ///
    /// # Errors
    /// - [`ProxyError::Config`] when `upstreams` is empty (cannot happen via
    ///   `upstreams_resolved`, which always yields ≥1)
    /// - the last attempt's transport error when every upstream is down
    pub async fn forward_with_failover(
        &self,
        upstreams: &[UpstreamConfig],
        method: Method,
        wire_path: &str,
        headers: &HeaderMap,
        body: Bytes,
    ) -> Result<Response<Body>, ProxyError> {
        if upstreams.is_empty() {
            return Err(ProxyError::Config(
                "forward_with_failover: no upstreams configured".to_string(),
            ));
        }
        let fail_open = self
            .failover_health
            .lock()
            .map(|h| h.all_skipped(upstreams.len()))
            .unwrap_or(true);
        let mut failed_attempts: u32 = 0;
        let mut last_err: Option<ProxyError> = None;

        for (i, cfg) in upstreams.iter().enumerate() {
            if !fail_open && self.failover_health.lock().is_ok_and(|h| h.should_skip(i)) {
                continue;
            }
            match self
                .send_once(cfg, method.clone(), wire_path, headers, body.clone())
                .await
            {
                Ok(resp) => {
                    let retryable = is_retryable_status(resp.status().as_u16());
                    // Retryable AND a later candidate will be tried → drop this
                    // body (status already observed) and move on. Otherwise
                    // this response — success or last-viable retryable — goes
                    // to the client verbatim with its real status.
                    if retryable
                        && Self::has_viable_later(upstreams, i, fail_open, &self.failover_health)
                    {
                        self.note_failover_failure(i);
                        drop(resp);
                        failed_attempts = failed_attempts.saturating_add(1);
                        Self::sleep_backoff(failed_attempts).await;
                        continue;
                    }
                    if retryable {
                        self.note_failover_failure(i);
                    } else {
                        self.note_failover_success(i);
                    }
                    return into_axum(resp);
                }
                Err(e) => {
                    self.note_failover_failure(i);
                    last_err = Some(e);
                    failed_attempts = failed_attempts.saturating_add(1);
                    Self::sleep_backoff(failed_attempts).await;
                }
            }
        }
        Err(last_err.unwrap_or(ProxyError::Forward(
            "all upstreams skipped by health, fail-open found none".to_string(),
        )))
    }

    /// True when at least one later candidate will be tried after index `i`.
    fn has_viable_later(
        upstreams: &[UpstreamConfig],
        i: usize,
        fail_open: bool,
        health: &std::sync::Arc<std::sync::Mutex<UpstreamHealthSet>>,
    ) -> bool {
        if i + 1 >= upstreams.len() {
            return false;
        }
        if fail_open {
            return true;
        }
        let Ok(h) = health.lock() else {
            return true;
        };
        ((i + 1)..upstreams.len()).any(|j| !h.should_skip(j))
    }

    /// Backoff sleep before the next attempt (`failed_attempts` ≥ 1).
    async fn sleep_backoff(failed_attempts: u32) {
        let attempt = failed_attempts.saturating_sub(1);
        tokio::time::sleep(backoff_delay(
            attempt,
            FAILOVER_BASE_BACKOFF_MS,
            FAILOVER_MAX_BACKOFF_MS,
        ))
        .await;
    }

    fn note_failover_success(&self, i: usize) {
        if let Ok(mut h) = self.failover_health.lock() {
            h.note_success(i);
        }
    }

    fn note_failover_failure(&self, i: usize) {
        if let Ok(mut h) = self.failover_health.lock() {
            h.note_failure(i);
        }
    }
}

/// Convert a raw upstream response into a streaming axum response.
///
/// # Errors
/// Returns [`ProxyError::Forward`] when the response cannot be built.
fn into_axum(resp: reqwest::Response) -> Result<Response<Body>, ProxyError> {
    let status = resp.status();
    let resp_headers = response_headers_into_axum(resp.headers());
    // Streaming passthrough: chunks flow through as the upstream produces them.
    let stream = resp.bytes_stream();
    let mut builder = Response::builder().status(axum_status(status));
    if let Some(hm) = builder.headers_mut() {
        hm.extend(resp_headers);
    }
    builder
        .body(Body::from_stream(stream))
        .map_err(|e| ProxyError::Forward(format!("build response: {e}")))
}

fn convert_method(m: Method) -> reqwest::Method {
    reqwest::Method::from_bytes(m.as_str().as_bytes()).unwrap_or(reqwest::Method::POST)
}

fn axum_status(s: reqwest::StatusCode) -> axum::http::StatusCode {
    axum::http::StatusCode::from_u16(s.as_u16()).unwrap_or(axum::http::StatusCode::BAD_GATEWAY)
}

fn classify_send(e: reqwest::Error) -> ProxyError {
    if e.is_timeout() {
        ProxyError::UpstreamTimeout
    } else if e.is_connect() || e.is_request() {
        ProxyError::UpstreamUnreachable(e.to_string())
    } else {
        ProxyError::Forward(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retryable_is_429_and_5xx_only() {
        // PRX-02 contrato: failover dispara en quota agotada y upstreams rotos.
        for status in [429, 500, 502, 503, 599] {
            assert!(is_retryable_status(status), "{status} must be retryable");
        }
        for status in [200, 201, 400, 401, 403, 404, 422] {
            assert!(
                !is_retryable_status(status),
                "{status} must NOT be retryable"
            );
        }
    }

    #[test]
    fn backoff_doubles_until_cap() {
        // PRX-02 contrato: backoff exponencial con valores exactos.
        let delays: Vec<u64> = (0..8)
            .map(|a| backoff_delay(a, 100, 2000).as_millis() as u64)
            .collect();
        assert_eq!(delays, vec![100, 200, 400, 800, 1600, 2000, 2000, 2000]);
    }

    #[test]
    fn backoff_saturates_without_overflow() {
        assert_eq!(backoff_delay(100, 100, 2000), Duration::from_millis(2000));
        assert_eq!(
            backoff_delay(u32::MAX, u64::MAX, u64::MAX),
            Duration::from_millis(u64::MAX)
        );
    }

    #[test]
    fn health_skips_after_three_failures_and_recovers_on_success() {
        // PRX-02 decisión 5: histéresis sin timers.
        let mut h = UpstreamHealthSet::new(2);
        assert!(!h.should_skip(0));
        h.note_failure(0);
        h.note_failure(0);
        assert!(!h.should_skip(0), "2 failures must not skip yet");
        h.note_failure(0);
        assert!(h.should_skip(0), "3 failures must skip");
        assert!(!h.should_skip(1), "sibling unaffected");
        h.note_success(0);
        assert!(!h.should_skip(0), "one success resets");
    }

    #[test]
    fn health_fail_open_when_all_skipped() {
        let mut h = UpstreamHealthSet::new(2);
        for i in 0..2 {
            for _ in 0..3 {
                h.note_failure(i);
            }
        }
        assert!(h.all_skipped(2));
        assert!(!UpstreamHealthSet::new(2).all_skipped(2));
        assert!(!h.all_skipped(0), "empty set never fail-opens");
    }
}
