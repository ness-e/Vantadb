//! Verbatim forwarding engine: strip hop-by-hop headers, forward bytes
//! unmodified, stream the upstream response back without buffering.

use std::time::Duration;

use axum::body::Body;
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, Response};
use bytes::Bytes;
use reqwest::Client;

use crate::config::UpstreamConfig;
use crate::error::ProxyError;

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

/// Shared forwarder holding the pooled HTTP client.
#[derive(Clone)]
pub struct Forwarder {
    client: Client,
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
        Ok(Self { client })
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

        let resp = self
            .client
            .request(convert_method(method), url)
            .headers(upstream_headers)
            .body(body)
            .send()
            .await
            .map_err(classify_send)?;

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
