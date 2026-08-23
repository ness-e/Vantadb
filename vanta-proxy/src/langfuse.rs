//! Optional Langfuse/OTel span export over hand-built OTLP-JSON (MEM-56).
//!
//! Decision (ponytail): the OTLP/HTTP endpoint accepts JSON payloads, so one
//! turn becomes a fixed-shape `resourceSpans` JSON built with `serde_json`
//! and POSTed with plain `reqwest` — no opentelemetry SDK (~30 transitives)
//! to export six fields per turn.
//!
//! P4 guarantee: registering the hook spawns ONE worker thread owning a
//! blocking client; the hook itself only does an unbounded channel send, so
//! reporting never blocks or fails the wire path.

use crate::config::ReportConfig;
use crate::report::{ReportHook, TurnReport};
use std::sync::mpsc;

/// POST timeout: a hung collector must not pile up exports forever.
const EXPORT_TIMEOUT_SECS: u64 = 5;
/// OTel-valid trace/span ids derived from nanosecond timestamps.
const NANOS_PER_SEC: u128 = 1_000_000_000;

/// Build the hook wiring per-turn reports to the configured OTLP endpoint.
///
/// Returns `None` when no endpoint is configured (disabled by default) —
/// zero overhead in that case.
pub fn langfuse_hook(config: &ReportConfig) -> Option<ReportHook> {
    if !config.enabled() {
        return None;
    }
    let endpoint = config.langfuse_endpoint.clone();
    let auth = config.langfuse_auth_header.clone();
    let (tx, rx) = mpsc::channel::<TurnReport>();
    if std::thread::Builder::new()
        .name("langfuse-export".into())
        .spawn(move || {
            for report in rx {
                if let Err(e) = post_span(&endpoint, &auth, &report) {
                    tracing::warn!(
                        target: "vanta_proxy::report",
                        error = %e,
                        "langfuse span export failed (turn continues)"
                    );
                }
            }
        })
        .is_err()
    {
        tracing::warn!(target: "vanta_proxy::report", "could not spawn langfuse exporter");
        return None;
    }
    Some(Box::new(move |report: &TurnReport| {
        // Unbounded send: infallible for our purposes, never blocks.
        let _ = tx.send(report.clone());
    }))
}

fn post_span(endpoint: &str, auth: &str, report: &TurnReport) -> Result<(), String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(EXPORT_TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("client build: {e}"))?;
    let mut request = client
        .post(endpoint)
        .header("content-type", "application/json")
        .json(&otlp_payload(report));
    if !auth.is_empty() {
        request = request.header("authorization", auth);
    }
    let response = request
        .send()
        .map_err(|e| format!("post {endpoint}: {e}"))?;
    if !response.status().is_success() {
        return Err(format!("export rejected: HTTP {}", response.status()));
    }
    Ok(())
}

/// One turn → one OTLP/HTTP JSON export payload (`TracesService` shape).
fn otlp_payload(report: &TurnReport) -> serde_json::Value {
    serde_json::json!({
        "resourceSpans": [{
            "resource": {
                "attributes": [
                    {"key": "service.name",
                     "value": {"stringValue": "vanta-proxy"}},
                ],
            },
            "scopeSpans": [{
                "scope": {"name": "vanta_proxy"},
                "spans": [{
                    "traceId": trace_id(report),
                    "spanId": span_id(report),
                    "name": "llm.turn",
                    "kind": "SPAN_KIND_CLIENT",
                    "startTimeUnixNano": start_nanos(report),

                    "endTimeUnixNano": end_nanos(report),
                    "attributes": [
                        {"key": "vanta.space_id",
                         "value": {"stringValue": report.space_id}},
                        {"key": "llm.request.protocol",
                         "value": {"stringValue": report.protocol}},
                        {"key": "llm.response.model",
                         "value": {"stringValue": report.model}},
                        {"key": "http.response.status_code",
                         "value": {"intValue": i64::from(report.status)}},
                        {"key": "vanta.duration_ms",
                         "value": {"doubleValue": report.duration_ms as f64}},
                    ],
                }],
            }],
        }],
    })
}

fn start_nanos(report: &TurnReport) -> String {
    (u128::from(report.timestamp_ms) * NANOS_PER_SEC).to_string()
}

fn end_nanos(report: &TurnReport) -> String {
    (u128::from(report.timestamp_ms + report.duration_ms as u64) * NANOS_PER_SEC).to_string()
}

/// 32-hex-char OTel trace id from the turn timestamp (ns precision makes
/// per-turn collision negligible; observability ids need no cryptography).
fn trace_id(report: &TurnReport) -> String {
    format!("{:032x}", u128::from(report.timestamp_ms) * NANOS_PER_SEC)
}

/// 16-hex-char OTel span id.
fn span_id(report: &TurnReport) -> String {
    format!(
        "{:016x}",
        u128::from(report.timestamp_ms) & 0xffff_ffff_ffff_ffff
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::Reporter;
    use std::io::{Read, Write};
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    fn sample_report() -> TurnReport {
        TurnReport {
            timestamp_ms: 1_700_000_000_000,
            space_id: "sp-9".into(),
            protocol: "openai".into(),
            model: "gpt-x".into(),
            status: 200,
            duration_ms: 12,
        }
    }

    /// D19: payload shape is valid OTLP/JSON with one span carrying the turn.
    #[test]
    fn otlp_payload_has_one_span_with_turn_attributes() {
        let value = otlp_payload(&sample_report());
        let span = &value["resourceSpans"][0]["scopeSpans"][0]["spans"][0];
        assert_eq!(span["name"], "llm.turn");
        assert_eq!(span["traceId"].as_str().map(str::len), Some(32));
        assert_eq!(span["spanId"].as_str().map(str::len), Some(16));
        let attrs = span["attributes"].as_array().expect("attributes");
        let get = |k: &str| {
            attrs
                .iter()
                .find(|a| a["key"] == k)
                .and_then(|a| a["value"]["stringValue"].as_str())
        };
        assert_eq!(get("vanta.space_id"), Some("sp-9"));
        assert_eq!(get("llm.request.protocol"), Some("openai"));
        assert_eq!(get("llm.response.model"), Some("gpt-x"));
        let status = attrs
            .iter()
            .find(|a| a["key"] == "http.response.status_code")
            .and_then(|a| a["value"]["intValue"].as_i64());
        assert_eq!(status, Some(200));
    }

    /// Minimal HTTP/1.1 collector capturing exactly one POST body.
    fn spawn_collector() -> (String, Arc<Mutex<String>>) {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");
        let captured = Arc::new(Mutex::new(String::new()));
        let cap = Arc::clone(&captured);
        std::thread::spawn(move || {
            let Ok((mut stream, _)) = listener.accept() else {
                return;
            };
            let mut data = Vec::new();
            let mut buf = [0u8; 8192];
            while let Ok(n) = stream.read(&mut buf) {
                if n == 0 {
                    break;
                }
                data.extend_from_slice(&buf[..n]);
                if let Some(pos) = data.windows(4).position(|w| w == b"\r\n\r\n") {
                    let head = String::from_utf8_lossy(&data[..pos]).to_ascii_lowercase();
                    let len: usize = head
                        .lines()
                        .find_map(|l| l.strip_prefix("content-length:")?.trim().parse().ok())
                        .unwrap_or(0);
                    if data.len() >= pos + 4 + len {
                        *cap.lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner) =
                            String::from_utf8_lossy(&data[pos + 4..]).to_string();
                        break;
                    }
                }
            }
            let _ = stream.write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 2\r\n\r\n{}");
        });
        (format!("http://{addr}"), captured)
    }

    /// D19: turno → span emitido hacia un collector OTLP mockeado.
    #[test]
    fn turn_is_exported_as_otlp_span_to_mock_collector() {
        let (endpoint, captured) = spawn_collector();
        let config = ReportConfig {
            langfuse_endpoint: endpoint,
            langfuse_auth_header: "Basic dGVzdA==".into(),
        };
        let hook = langfuse_hook(&config).expect("hook when enabled");
        let reporter = Reporter::new();
        reporter.add_hook(hook);
        reporter.emit(&sample_report());

        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline && captured.lock().unwrap().is_empty() {
            std::thread::sleep(Duration::from_millis(10));
        }
        let body = captured.lock().unwrap().clone();
        let value: serde_json::Value = serde_json::from_str(&body).expect("valid json");
        let span = &value["resourceSpans"][0]["scopeSpans"][0]["spans"][0];
        assert_eq!(span["name"], "llm.turn");
    }

    /// D19: sin endpoint configurado no hay exportación ni overhead.
    #[test]
    fn disabled_when_no_endpoint_configured() {
        assert!(!ReportConfig::default().enabled());
        assert!(!crate::config::ProxyConfig::default().report.enabled());
        assert!(langfuse_hook(&ReportConfig::default()).is_none());
        assert!(langfuse_hook(&ReportConfig {
            langfuse_endpoint: String::new(),
            langfuse_auth_header: "x".into(),
        })
        .is_none());
    }

    /// D19/P4: endpoint caído → error capturado, el proxy sigue (no bloquea).
    #[test]
    fn network_failure_returns_err_without_blocking() {
        // Puerto libre garantizado: lo asignamos y soltamos el listener.
        let dead = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let endpoint = format!("http://{}", dead.local_addr().expect("addr"));
        drop(dead);

        let report = sample_report();
        let start = Instant::now();
        let result = post_span(&endpoint, "", &report);
        assert!(result.is_err(), "unreachable endpoint must error");
        assert!(
            start.elapsed() < Duration::from_secs(EXPORT_TIMEOUT_SECS),
            "failure must surface fast, not block"
        );

        // End-to-end: hook registrado contra endpoint caído nunca rompe emit.
        let hook = langfuse_hook(&ReportConfig {
            langfuse_endpoint: endpoint,
            langfuse_auth_header: String::new(),
        })
        .expect("hook");
        let reporter = Reporter::new();
        reporter.add_hook(hook);
        for i in 0..3 {
            reporter.emit(&TurnReport {
                space_id: format!("sp-{i}"),
                ..sample_report()
            });
        }
        // Llegar acá sin colgarse ni paniquear ES el assert del P4.
    }
}
