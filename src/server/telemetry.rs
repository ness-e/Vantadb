//! Telemetry initialization (tracing + OpenTelemetry) and shutdown.
//!
//! REVIEW-10: extracted from `routing.rs` — all tracing/OTEL setup lives here.

use crate::config::LogFormat;
use tracing_subscriber::EnvFilter;

#[cfg(feature = "opentelemetry")]
use std::sync::OnceLock;

#[cfg(feature = "opentelemetry")]
static OTEL_PROVIDER: OnceLock<opentelemetry_sdk::trace::SdkTracerProvider> = OnceLock::new();

/// Initialise the tracing subscriber with optional OpenTelemetry and MCP support.
pub fn init_telemetry(is_mcp: bool, log_format: Option<LogFormat>) {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let format = resolve_log_format(log_format);
    let is_json = matches!(format, LogFormat::Json);
    let is_full = matches!(format, LogFormat::Full);

    #[cfg(feature = "opentelemetry")]
    _init_telemetry_otel(is_mcp, is_json, is_full, env_filter);

    #[cfg(not(feature = "opentelemetry"))]
    init_telemetry_fmt(is_mcp, is_json, is_full, env_filter);
}

fn resolve_log_format(log_format: Option<LogFormat>) -> LogFormat {
    log_format.unwrap_or_else(|| {
        let legacy = std::env::var("VANTADB_LOG_JSON")
            .map(|v| v == "1" || v == "true")
            .unwrap_or(false);
        if legacy {
            LogFormat::Json
        } else {
            std::env::var("VANTADB_LOG_FORMAT")
                .ok()
                .map(|v| LogFormat::from_env_value(&v))
                .unwrap_or_default()
        }
    })
}

#[cfg(not(feature = "opentelemetry"))]
fn init_telemetry_fmt(is_mcp: bool, is_json: bool, is_full: bool, env_filter: EnvFilter) {
    let stderr = || Box::new(std::io::stderr()) as Box<dyn std::io::Write + Send>;

    if is_json {
        let sub = tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .json()
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true)
            .with_ansi(false);
        if is_mcp {
            sub.with_writer(stderr).init();
        } else {
            sub.init();
        }
    } else if is_full {
        let sub = tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true)
            .with_ansi(true);
        if is_mcp {
            sub.with_writer(stderr).init();
        } else {
            sub.init();
        }
    } else if is_mcp {
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_writer(stderr)
            .init();
    } else {
        crate::console::init_logging(LogFormat::Compact);
    }
}

#[cfg(feature = "opentelemetry")]
fn _init_telemetry_otel(is_mcp: bool, is_json: bool, is_full: bool, env_filter: EnvFilter) {
    use opentelemetry::trace::TracerProvider;
    use opentelemetry_otlp::WithExportConfig;
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Registry};

    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    let exporter = match opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint.clone())
        .build()
    {
        Ok(exporter) => exporter,
        Err(e) => {
            eprintln!(
                "⚠️ Failed to create OTLP exporter (endpoint: {}), continuing without tracing: {e}",
                endpoint
            );
            return;
        }
    };

    let service_name =
        std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "vantadb-server".to_string());

    let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(
            opentelemetry_sdk::Resource::builder_empty()
                .with_service_name(service_name.clone())
                .build(),
        )
        .build();

    let _ = OTEL_PROVIDER.set(provider.clone());
    let tracer = provider.tracer(service_name.clone());
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let subscriber = Registry::default().with(env_filter).with(telemetry);

    if is_mcp {
        subscriber
            .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
            .init();
    } else if is_json {
        subscriber
            .with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_file(true)
                    .with_line_number(true),
            )
            .init();
    } else if is_full {
        subscriber
            .with(
                tracing_subscriber::fmt::layer()
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_file(true)
                    .with_line_number(true),
            )
            .init();
    } else {
        subscriber.with(tracing_subscriber::fmt::layer()).init();
    }
}

/// Shut down the OpenTelemetry tracer provider, flushing any pending spans.
#[cfg(feature = "opentelemetry")]
pub fn shutdown_telemetry() {
    if let Some(provider) = OTEL_PROVIDER.get() {
        if let Err(e) = provider.shutdown() {
            eprintln!("OTel provider shutdown error: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_log_format_defaults_to_compact() {
        // No env vars set → default Compact
        std::env::remove_var("VANTADB_LOG_JSON");
        std::env::remove_var("VANTADB_LOG_FORMAT");
        assert_eq!(resolve_log_format(None), LogFormat::Compact);
    }

    #[test]
    fn resolve_log_format_legacy_json_env() {
        std::env::set_var("VANTADB_LOG_JSON", "1");
        assert_eq!(resolve_log_format(None), LogFormat::Json);
        std::env::remove_var("VANTADB_LOG_JSON");
    }

    #[test]
    fn resolve_log_format_explicit_env() {
        std::env::set_var("VANTADB_LOG_FORMAT", "full");
        assert_eq!(resolve_log_format(None), LogFormat::Full);
        std::env::remove_var("VANTADB_LOG_FORMAT");
    }
}
