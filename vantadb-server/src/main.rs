#[cfg(feature = "custom-allocator")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use clap::Parser;
use std::sync::Arc;
use tokio::net::TcpListener;
use vantadb::console;
use vantadb::storage::StorageEngine;
use vantadb_server::server::{app, ServerState};

#[derive(Parser, Debug)]
#[command(name = "vantadb-server")]
#[command(about = "VantaDB local server wrapper and MCP interface")]
struct ServerCli {
    /// Run as a Model Context Protocol (MCP) server over standard I/O
    #[arg(long)]
    mcp: bool,
}

#[tokio::main]
async fn main() {
    let cli = ServerCli::parse();
    let is_mcp = cli.mcp;

    // ── Initialize structured logging & telemetry ───────────────────────────
    init_telemetry(is_mcp);

    if !is_mcp {
        console::print_banner();
        console::progress("Initializing storage engine...", None);
    }

    // ── Load Configuration ──────────────────────────────────────────────────
    let config = vantadb::config::VantaConfig::from_env();

    // ── Open storage engine ─────────────────────────────────────────────────
    let storage = match StorageEngine::open_with_config(&config.storage_path, Some(config.clone()))
    {
        Ok(s) => {
            if !is_mcp {
                console::ok("Storage engine opened", Some(&config.storage_path));
            }
            Arc::new(s)
        }
        Err(e) => {
            console::error("Failed to open storage engine", Some(&e.to_string()));
            std::process::exit(1);
        }
    };

    // ── Bootstrap Invalidation Dispatcher ──────────────────────────────────
    #[cfg(feature = "governance")]
    let invalidation_tx = {
        let mut dispatcher = experimental_governance::InvalidationDispatcher::new(256);
        let tx = dispatcher.sender();
        if let Some(rx) = dispatcher.take_receiver() {
            std::thread::spawn(move || {
                experimental_governance::invalidations::invalidation_listener(rx);
            });
        }
        tx
    };

    // ── Background maintenance worker ───────────────────────────────────────
    #[cfg(feature = "governance")]
    {
        let maintenance_storage_ctx = storage.clone();
        let _maintenance_handle = experimental_governance::MaintenanceWorker::start(
            maintenance_storage_ctx,
            invalidation_tx.clone(),
        );
    }

    #[cfg(feature = "governance")]
    if !is_mcp {
        console::ok(
            "Background workers started",
            Some("maintenance_worker · invalidations"),
        );
    }

    // ── Serve MCP or HTTP ───────────────────────────────────────────────────
    if is_mcp {
        vantadb_mcp::run_stdio_server(storage).await;
    } else {
        // ── Log active security mode ─────────────────────────────────────
        log_security_mode(&config);

        let api_key: Option<Arc<str>> = config.api_key.as_deref().map(Arc::from);
        let semaphore = Arc::new(tokio::sync::Semaphore::new(config.max_blocking_threads));
        let state = Arc::new(ServerState {
            storage: storage.clone(),
            semaphore,
            api_key,
        });

        let rpm = config.rate_limit_rpm;
        let router = app(state, rpm);
        let addr = format!("{}:{}", config.host, config.port);

        serve_http_or_tls(router, addr, &config).await;
    }
}

/// Logs the active security configuration to the console at startup.
fn log_security_mode(config: &vantadb::config::VantaConfig) {
    let auth_status = if config.api_key.is_some() {
        "Bearer token auth ✓"
    } else {
        "No auth (dev mode)"
    };

    let rate_status = if config.rate_limit_rpm == 0 {
        "Rate limit disabled".to_string()
    } else {
        format!("Rate limit {} req/min", config.rate_limit_rpm)
    };

    let tls_status = {
        #[cfg(feature = "tls")]
        {
            if config.tls_cert_path.is_some() && config.tls_key_path.is_some() {
                "TLS ✓ (rustls)"
            } else {
                "TLS feature active but no cert/key configured — falling back to plain HTTP"
            }
        }
        #[cfg(not(feature = "tls"))]
        "Plain HTTP"
    };

    console::ok(
        "Security",
        Some(&format!(
            "{} | {} | {}",
            auth_status, rate_status, tls_status
        )),
    );
}

/// Binds and serves the router over plain TCP or TLS depending on the feature
/// flag and runtime configuration.
#[cfg_attr(not(feature = "tls"), allow(unused_variables))]
async fn serve_http_or_tls(
    router: axum::Router,
    addr: String,
    config: &vantadb::config::VantaConfig,
) {
    // ── TLS path (requires --features tls AND cert/key paths) ───────────────
    #[cfg(feature = "tls")]
    if let (Some(cert), Some(key)) = (&config.tls_cert_path, &config.tls_key_path) {
        use axum_server::tls_rustls::RustlsConfig;

        let tls_config = match RustlsConfig::from_pem_file(cert, key).await {
            Ok(c) => c,
            Err(e) => {
                console::error("Failed to load TLS certificate/key", Some(&e.to_string()));
                std::process::exit(1);
            }
        };

        let socket_addr: std::net::SocketAddr = match addr.parse() {
            Ok(a) => a,
            Err(e) => {
                console::error("Invalid bind address", Some(&e.to_string()));
                std::process::exit(1);
            }
        };

        console::print_ready(&format!("https://{}", addr));

        if let Err(e) = axum_server::bind_rustls(socket_addr, tls_config)
            .serve(router.into_make_service_with_connect_info::<std::net::SocketAddr>())
            .await
        {
            console::error("TLS server terminated unexpectedly", Some(&e.to_string()));
            std::process::exit(1);
        }

        return;
    }

    // ── Plain HTTP path (default) ────────────────────────────────────────────
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => {
            console::ok("TCP listener bound", Some(&addr));
            l
        }
        Err(e) => {
            console::error("Failed to bind port", Some(&e.to_string()));
            std::process::exit(1);
        }
    };

    console::print_ready(&addr);

    if let Err(e) = axum::serve(
        listener,
        router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    {
        console::error("Server terminated unexpectedly", Some(&e.to_string()));
        std::process::exit(1);
    }
}

/// Initializes structured JSON logging and OpenTelemetry OTLP tracing based on configuration.
fn init_telemetry(is_mcp: bool) {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let use_json = std::env::var("VANTADB_LOG_JSON")
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false);

    #[cfg(feature = "opentelemetry")]
    {
        use opentelemetry_otlp::WithExportConfig;

        let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:4317".to_string());

        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint)
            .build()
            .expect("Failed to create OTLP exporter");

        let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
            .with_batch_exporter(exporter)
            .with_resource(
                opentelemetry_sdk::Resource::builder_empty()
                    .with_service_name("vantadb-server")
                    .build(),
            )
            .build();

        use opentelemetry::trace::TracerProvider;
        let tracer = provider.tracer("vantadb-server");
        let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

        if use_json {
            let fmt_layer = tracing_subscriber::fmt::layer().json();
            if is_mcp {
                Registry::default()
                    .with(env_filter)
                    .with(telemetry)
                    .with(fmt_layer.with_writer(std::io::stderr))
                    .init();
            } else {
                Registry::default()
                    .with(env_filter)
                    .with(telemetry)
                    .with(fmt_layer)
                    .init();
            }
        } else {
            let fmt_layer = tracing_subscriber::fmt::layer();
            if is_mcp {
                Registry::default()
                    .with(env_filter)
                    .with(telemetry)
                    .with(fmt_layer.with_writer(std::io::stderr))
                    .init();
            } else {
                Registry::default()
                    .with(env_filter)
                    .with(telemetry)
                    .with(fmt_layer)
                    .init();
            }
        }
    }

    #[cfg(not(feature = "opentelemetry"))]
    {
        if use_json {
            let fmt_layer = tracing_subscriber::fmt::layer().json();
            if is_mcp {
                Registry::default()
                    .with(env_filter)
                    .with(fmt_layer.with_writer(std::io::stderr))
                    .init();
            } else {
                Registry::default().with(env_filter).with(fmt_layer).init();
            }
        } else {
            if is_mcp {
                let fmt_layer = tracing_subscriber::fmt::layer();
                Registry::default()
                    .with(env_filter)
                    .with(fmt_layer.with_writer(std::io::stderr))
                    .init();
            } else {
                vantadb::console::init_logging();
            }
        }
    }
}
