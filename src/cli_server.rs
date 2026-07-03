//! HTTP server startup and route wiring for VantaDB's CLI server.
//!
//! Builds an [`axum`] application, mounts middleware and API routes,
//! and binds to the configured address.

use std::collections::HashMap;
use std::sync::Arc;
#[cfg(feature = "opentelemetry")]
use std::sync::OnceLock;
#[cfg(feature = "tls")]
use std::time::Duration;
use std::time::Instant;

use axum::{
    extract::State,
    http::{header, StatusCode},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Json, Router,
};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;
use tokio::net::TcpListener;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tracing_subscriber::EnvFilter;
#[cfg(feature = "opentelemetry")]
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Registry};
#[cfg(feature = "opentelemetry")]
static OTEL_PROVIDER: OnceLock<opentelemetry_sdk::trace::SdkTracerProvider> = OnceLock::new();

use crate::config::{LogFormat, RbacConfig, VantaConfig};
use crate::console;
use crate::error::Result;
use crate::metrics;
use crate::node::{FieldValue, UnifiedNode};
use crate::rbac::{Permission, Rbac};
use crate::storage::StorageEngine;

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryRequest {
    pub query: String,
}

#[derive(Serialize, Deserialize)]
pub struct QueryResponse {
    pub success: bool,
    pub data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nodes: Option<Vec<NodeDTO>>,
}

#[derive(Serialize, Deserialize)]
pub struct NodeDTO {
    pub id: u64,
    pub semantic_cluster: u32,
    pub relational: std::collections::BTreeMap<String, FieldValue>,
    pub hits: u32,
    pub confidence_score: f32,
}

impl From<&UnifiedNode> for NodeDTO {
    fn from(n: &UnifiedNode) -> Self {
        Self {
            id: n.id,
            semantic_cluster: n.semantic_cluster,
            relational: n.relational.clone(),
            hits: n.hits,
            confidence_score: n.confidence_score,
        }
    }
}

pub struct ServerState {
    pub storage: Arc<StorageEngine>,
    pub semaphore: Arc<tokio::sync::Semaphore>,
    pub api_key: Option<Arc<str>>,
    pub rbac_config: RbacConfig,
}

pub fn app(state: Arc<ServerState>, rpm: u32) -> Router {
    let rbac = Arc::new(Rbac::new());
    rbac.add_role("admin", vec![Permission::Admin]);
    rbac.add_role("reader", vec![Permission::Read]);
    rbac.add_role("writer", vec![Permission::Read, Permission::Write]);
    let auth_state = AuthState::new(
        state.api_key.as_ref().map(|k| k.to_string()),
        state.rbac_config.clone(),
        rbac,
    );

    let public = Router::new().route("/health", get(health_check));

    let protected = Router::new()
        .route("/api/v2/query", post(execute_query))
        .route("/metrics", get(metrics_endpoint))
        .layer(middleware::from_fn(auth_middleware));

    let protected = if rpm > 0 {
        let period_ms = (60_000u64 / rpm as u64).max(1);
        let burst_size = (rpm / 10).max(1);

        let governor_config = GovernorConfigBuilder::default()
            .per_millisecond(period_ms)
            .burst_size(burst_size)
            .finish()
            .expect("GovernorConfig build failed");

        protected.layer(GovernorLayer::new(governor_config))
    } else {
        protected
    };

    Router::new()
        .merge(public)
        .merge(protected)
        .layer(middleware::from_fn(request_metrics_middleware))
        .layer(Extension(auth_state))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state)
}

pub struct AuthRateLimiter {
    failures: Mutex<HashMap<String, (u32, Instant)>>,
    max_attempts: u32,
    window_secs: u64,
}

impl AuthRateLimiter {
    pub fn new(max_attempts: u32, window_secs: u64) -> Self {
        Self {
            failures: Mutex::new(HashMap::new()),
            max_attempts,
            window_secs,
        }
    }

    pub fn is_rate_limited(&self, ip: &str) -> bool {
        let mut failures = self.failures.lock();
        let now = Instant::now();
        if let Some((count, first)) = failures.get(ip) {
            if now.duration_since(*first).as_secs() > self.window_secs {
                failures.remove(ip);
                return false;
            }
            *count >= self.max_attempts
        } else {
            false
        }
    }

    pub fn record_failure(&self, ip: &str) {
        let mut failures = self.failures.lock();
        let now = Instant::now();
        let entry = failures.entry(ip.to_string()).or_insert((0, now));
        if now.duration_since(entry.1).as_secs() > self.window_secs {
            *entry = (1, now);
        } else {
            entry.0 += 1;
        }
    }

    pub fn reset(&self, ip: &str) {
        self.failures.lock().remove(ip);
    }
}

#[derive(Clone)]
pub struct AuthState {
    pub api_key: Option<Arc<str>>,
    pub(crate) token_role_map: HashMap<String, String>,
    pub(crate) rbac: Arc<Rbac>,
    pub(crate) rate_limiter: Arc<AuthRateLimiter>,
}

impl AuthState {
    pub(crate) fn new(api_key: Option<String>, rbac_config: RbacConfig, rbac: Arc<Rbac>) -> Self {
        Self {
            api_key: api_key.map(|k| Arc::from(k.as_str())),
            token_role_map: rbac_config.token_role_map,
            rbac,
            rate_limiter: Arc::new(AuthRateLimiter::new(5, 60)),
        }
    }
}

pub async fn auth_middleware(
    Extension(auth): Extension<AuthState>,
    req: axum::extract::Request,
    next: middleware::Next,
) -> Response {
    // Health endpoint is always public
    if req.uri().path() == "/health" {
        return next.run(req).await;
    }

    // No API key configured — allow all (dev mode)
    let Some(expected_key) = &auth.api_key else {
        return next.run(req).await;
    };

    // Extract client IP for rate limiting
    let client_ip = req
        .extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|ci| ci.0.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Check rate limiting before processing auth
    if auth.rate_limiter.is_rate_limited(&client_ip) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "success": false,
                "error": "Too many authentication failures. Try again later.",
            })),
        )
            .into_response();
    }

    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    let authorized = match token {
        Some(token) => {
            let token_bytes = token.as_bytes();
            let expected_bytes = expected_key.as_bytes();
            token_bytes.ct_eq(expected_bytes).into()
        }
        None => false,
    };

    if authorized {
        // Check RBAC permissions
        if let Some(token_val) = token {
            if let Some(role) = auth.token_role_map.get(token_val) {
                let is_write = matches!(req.method().as_str(), "POST" | "PUT" | "PATCH" | "DELETE");
                let permission = if is_write {
                    Permission::Write
                } else {
                    Permission::Read
                };
                if !auth.rbac.has_permission(role, &permission) {
                    auth.rate_limiter.reset(&client_ip);
                    return (
                        StatusCode::FORBIDDEN,
                        Json(serde_json::json!({
                            "success": false,
                            "error": "Forbidden: insufficient permissions for this operation",
                        })),
                    )
                        .into_response();
                }
            }
        }
        auth.rate_limiter.reset(&client_ip);
        next.run(req).await
    } else {
        auth.rate_limiter.record_failure(&client_ip);
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "success": false,
                "error": "Unauthorized",
                "hint": "Provide a valid Bearer token in the Authorization header."
            })),
        )
            .into_response()
    }
}

#[tracing::instrument]
async fn health_check() -> Json<QueryResponse> {
    Json(QueryResponse {
        success: true,
        data: "OK".to_string(),
        node_id: None,
        nodes: None,
    })
}

#[tracing::instrument]
async fn metrics_endpoint() -> impl IntoResponse {
    let metrics_text = metrics::export_metrics_text();
    match Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain; version=0.0.4")
        .body(metrics_text)
    {
        Ok(resp) => resp.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to build metrics response: {e}"),
        )
            .into_response(),
    }
}

pub async fn request_metrics_middleware(
    req: axum::extract::Request,
    next: middleware::Next,
) -> Response {
    let start = std::time::Instant::now();
    let method = req.method().to_string();
    let route = req.uri().path().to_string();
    let res = next.run(req).await;
    let status = res.status();
    metrics::record_http_request(&method, &route, status.as_u16(), start);
    res
}

#[tracing::instrument(skip(state))]
async fn execute_query(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<QueryRequest>,
) -> Json<QueryResponse> {
    use crate::executor::{ExecutionResult, Executor};

    let _permit = match state.semaphore.clone().acquire_owned().await {
        Ok(p) => p,
        Err(_) => {
            return Json(QueryResponse {
                success: false,
                data: "Server concurrency semaphore closed".to_string(),
                node_id: None,
                nodes: None,
            });
        }
    };

    let storage = state.storage.clone();
    let query = payload.query.clone();

    let join_res = tokio::task::spawn_blocking(move || {
        let executor = Executor::new(&storage);
        executor.execute_hybrid(&query)
    })
    .await;

    let execution_result = match join_res {
        Ok(r) => r,
        Err(e) => {
            return Json(QueryResponse {
                success: false,
                data: format!("Internal server error: execution task panicked: {}", e),
                node_id: None,
                nodes: None,
            });
        }
    };

    match execution_result {
        Ok(ExecutionResult::Read(nodes)) => {
            let dtos: Vec<NodeDTO> = nodes.iter().map(NodeDTO::from).collect();
            Json(QueryResponse {
                success: true,
                data: format!("Read {} nodes.", nodes.len()),
                node_id: None,
                nodes: Some(dtos),
            })
        }
        Ok(ExecutionResult::Write {
            affected_nodes,
            message,
            node_id,
        }) => Json(QueryResponse {
            success: true,
            data: format!("Mutated {} nodes: {}", affected_nodes, message),
            node_id,
            nodes: None,
        }),
        Ok(ExecutionResult::StaleContext(summary_id)) => Json(QueryResponse {
            success: true,
            data: format!(
                "STALE_CONTEXT: Confidence Score critical. Rehydration available for summary {}",
                summary_id
            ),
            node_id: Some(summary_id),
            nodes: None,
        }),
        Err(e) => Json(QueryResponse {
            success: false,
            data: format!("Execution Error: {}", e),
            node_id: None,
            nodes: None,
        }),
    }
}

pub fn init_telemetry(is_mcp: bool, log_format: Option<LogFormat>) {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // Resolve format: explicit arg > VantaConfig default > legacy VANTADB_LOG_JSON
    let format = log_format.unwrap_or_else(|| {
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
    });

    let is_json = matches!(format, LogFormat::Json);
    let is_full = matches!(format, LogFormat::Full);

    #[cfg(feature = "opentelemetry")]
    _init_telemetry_otel(is_mcp, is_json, is_full, env_filter);

    #[cfg(not(feature = "opentelemetry"))]
    {
        if is_json {
            if is_mcp {
                tracing_subscriber::fmt()
                    .with_env_filter(env_filter)
                    .json()
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_file(true)
                    .with_line_number(true)
                    .with_ansi(false)
                    .with_writer(|| Box::new(std::io::stderr()) as Box<dyn std::io::Write + Send>)
                    .init();
            } else {
                tracing_subscriber::fmt()
                    .with_env_filter(env_filter)
                    .json()
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_file(true)
                    .with_line_number(true)
                    .with_ansi(false)
                    .init();
            }
        } else if is_full {
            if is_mcp {
                tracing_subscriber::fmt()
                    .with_env_filter(env_filter)
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_file(true)
                    .with_line_number(true)
                    .with_ansi(true)
                    .with_writer(|| Box::new(std::io::stderr()) as Box<dyn std::io::Write + Send>)
                    .init();
            } else {
                tracing_subscriber::fmt()
                    .with_env_filter(env_filter)
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_file(true)
                    .with_line_number(true)
                    .with_ansi(true)
                    .init();
            }
        } else if is_mcp {
            tracing_subscriber::fmt()
                .with_env_filter(env_filter)
                .with_writer(|| Box::new(std::io::stderr()) as Box<dyn std::io::Write + Send>)
                .init();
        } else {
            crate::console::init_logging(LogFormat::Compact);
        }
    }
}

#[cfg(feature = "opentelemetry")]
fn _init_telemetry_otel(is_mcp: bool, is_json: bool, is_full: bool, env_filter: EnvFilter) {
    use opentelemetry::trace::TracerProvider;
    use opentelemetry_otlp::WithExportConfig;

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

    if is_mcp {
        Registry::default()
            .with(env_filter)
            .with(telemetry)
            .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
            .init();
    } else if is_json {
        Registry::default()
            .with(env_filter)
            .with(telemetry)
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
        Registry::default()
            .with(env_filter)
            .with(telemetry)
            .with(
                tracing_subscriber::fmt::layer()
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_file(true)
                    .with_line_number(true),
            )
            .init();
    } else {
        Registry::default()
            .with(env_filter)
            .with(telemetry)
            .with(tracing_subscriber::fmt::layer())
            .init();
    }
}

#[cfg(feature = "opentelemetry")]
pub fn shutdown_telemetry() {
    if let Some(provider) = OTEL_PROVIDER.get() {
        if let Err(e) = provider.shutdown() {
            eprintln!("OTel provider shutdown error: {e}");
        }
    }
}

fn log_security_mode(config: &VantaConfig) {
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

pub async fn run(config: VantaConfig) -> Result<()> {
    init_telemetry(false, Some(config.log_format));

    console::print_banner();
    console::progress("Initializing storage engine...", None);

    let storage = match StorageEngine::open_with_config(&config.storage_path, Some(config.clone()))
    {
        Ok(s) => {
            console::ok("Storage engine opened", Some(&config.storage_path));
            Arc::new(s)
        }
        Err(e) => {
            console::error("Failed to open storage engine", Some(&e.to_string()));
            std::process::exit(1);
        }
    };

    log_security_mode(&config);

    let api_key: Option<Arc<str>> = config.api_key.as_deref().map(Arc::from);
    let semaphore = Arc::new(tokio::sync::Semaphore::new(config.max_blocking_threads));
    let rbac_config = config.rbac_config.clone();
    let state = Arc::new(ServerState {
        storage: storage.clone(),
        semaphore,
        api_key,
        rbac_config,
    });

    let rpm = config.rate_limit_rpm;
    let router = app(state, rpm);
    let addr = format!("{}:{}", config.host, config.port);

    serve_http_or_tls(router, addr, &config, storage.clone()).await;

    Ok(())
}

pub async fn wait_for_shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();
    #[cfg(unix)]
    let mut sigterm = match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
    {
        Ok(s) => s,
        Err(e) => {
            console::error("Failed to install SIGTERM handler", Some(&e.to_string()));
            return;
        }
    };

    #[cfg(unix)]
    tokio::select! {
        _ = ctrl_c => {},
        _ = sigterm.recv() => {},
    }
    #[cfg(not(unix))]
    let _ = ctrl_c.await;
}

#[cfg(feature = "tls")]
pub async fn build_tls13_config(
    cert_path: &str,
    key_path: &str,
) -> std::io::Result<rustls::ServerConfig> {
    use rustls::pki_types::{CertificateDer, PrivateKeyDer};
    use rustls_pemfile::Item;

    let cert_bytes = tokio::fs::read(cert_path).await?;
    let key_bytes = tokio::fs::read(key_path).await?;

    let certs: Vec<CertificateDer> = rustls_pemfile::certs(&mut cert_bytes.as_ref())
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let mut key_vec: Vec<Vec<u8>> = rustls_pemfile::read_all(&mut key_bytes.as_ref())
        .filter_map(|i| match i.ok()? {
            Item::Sec1Key(k) => Some(k.secret_sec1_der().to_vec()),
            Item::Pkcs1Key(k) => Some(k.secret_pkcs1_der().to_vec()),
            Item::Pkcs8Key(k) => Some(k.secret_pkcs8_der().to_vec()),
            _ => None,
        })
        .collect();

    if key_vec.len() != 1 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "expected exactly one private key in PEM file",
        ));
    }

    let key = PrivateKeyDer::try_from(
        key_vec
            .pop()
            .expect("key_vec has exactly one element after guard"),
    )
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let mut config =
        rustls::ServerConfig::builder_with_protocol_versions(&[&rustls::version::TLS13])
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    Ok(config)
}

#[cfg_attr(not(feature = "tls"), allow(unused_variables))]
async fn serve_http_or_tls(
    router: axum::Router,
    addr: String,
    config: &VantaConfig,
    storage: Arc<crate::storage::StorageEngine>,
) {
    #[cfg(feature = "tls")]
    if let (Some(cert), Some(key)) = (&config.tls_cert_path, &config.tls_key_path) {
        let tls_config = match build_tls13_config(cert, key).await {
            Ok(c) => axum_server::tls_rustls::RustlsConfig::from_config(Arc::new(c)),
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

        let handle = axum_server::Handle::new();
        let handle_clone = handle.clone();
        let storage_clone = storage.clone();
        tokio::spawn(async move {
            wait_for_shutdown_signal().await;
            console::warn("Shutting down TLS server gracefully...", None);
            if let Err(e) = storage_clone.flush() {
                console::error("Flush failed during shutdown", Some(&e.to_string()));
            } else {
                console::ok("Storage flushed", None);
            }
            #[cfg(feature = "opentelemetry")]
            shutdown_telemetry();
            handle_clone.graceful_shutdown(Some(Duration::from_secs(10)));
        });

        if let Err(e) = axum_server::bind_rustls(socket_addr, tls_config)
            .handle(handle)
            .serve(router.into_make_service_with_connect_info::<std::net::SocketAddr>())
            .await
        {
            console::error("TLS server terminated unexpectedly", Some(&e.to_string()));
            std::process::exit(1);
        }

        return;
    }

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

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let storage_clone = storage.clone();
    tokio::spawn(async move {
        wait_for_shutdown_signal().await;
        console::warn("Shutting down HTTP server gracefully...", None);
        let _ = shutdown_tx.send(());
    });

    if let Err(e) = axum::serve(
        listener,
        router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(async {
        let _ = shutdown_rx.await;
    })
    .await
    {
        console::error("Server terminated unexpectedly", Some(&e.to_string()));
        std::process::exit(1);
    }

    console::warn("Flushing storage after graceful drain...", None);
    if let Err(e) = storage_clone.flush() {
        console::error("Flush failed during shutdown", Some(&e.to_string()));
    } else {
        console::ok("Storage flushed successfully", None);
    }

    #[cfg(feature = "opentelemetry")]
    shutdown_telemetry();
}
