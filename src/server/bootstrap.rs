//! Server bootstrap: config validation, TLS, run loop, graceful shutdown.
//!
//! REVIEW-10: extracted from `routing.rs` — everything that starts and runs
//! the HTTP/TLS server.

use crate::circuit_breaker::CircuitBreaker;
use crate::config::VantaConfig;
use crate::connection_pool::ConnectionPool;
use crate::error::ChainedError;
use crate::error::Result;
use crate::server::router::{app_with_cors, mount_dashboard};
use crate::server::state::ServerState;
use crate::server::telemetry::init_telemetry;
use crate::storage::StorageEngine;
use crate::VantaError;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tracing;

/// Whether `host` binds only the loopback interface (`127.0.0.0/8`,
/// `::1`, or the literal name `localhost`). Unresolvable hostnames are
/// treated as non-loopback (fail closed).
fn is_loopback_host(host: &str) -> bool {
    let h = host.trim();
    let h = h.strip_prefix('[').unwrap_or(h);
    let h = h.strip_suffix(']').unwrap_or(h);
    h.eq_ignore_ascii_case("localhost")
        || h.parse::<std::net::IpAddr>()
            .map(|ip| ip.is_loopback())
            .unwrap_or(false)
}

/// Validate that the auth configuration is consistent.
///
/// Refuse-to-start policy (FIND-07): the server does NOT start when it binds a
/// non-loopback host without an API key — an unauthenticated instance exposed
/// to the network is an accident waiting to happen. Override explicitly with
/// `--allow-insecure` (dev only), which logs a prominent WARNING instead.
/// Also returns an error if `require_auth` is set but no key is configured.
/// SRV-04: `alt_api_key` requires `api_key` to be set (rotation needs a primary).
pub fn validate_auth_config(config: &VantaConfig) -> Result<()> {
    if config.alt_api_key.is_some() && config.api_key.is_none() {
        return Err(VantaError::InvalidInput(
            "alt_api_key requires api_key to be set (rotation needs a primary key)".into(),
        ));
    }
    if config.require_auth && config.api_key.is_none() {
        crate::console::error(
            "Forced authentication enabled but no API key configured",
            Some(
                "Set the VANTADB_API_KEY environment variable to provide an authentication \
                 token. Alternatively, unset VANTADB_REQUIRE_AUTH / remove --require-auth \
                 to allow unauthenticated (dev) mode.",
            ),
        );
        return Err(VantaError::InvalidInput(
            "require_auth is set but no api_key is configured".into(),
        ));
    }
    if config.api_key.is_none() && !is_loopback_host(&config.host) {
        if config.allow_insecure {
            crate::console::warn(
                "INSECURE MODE: HTTP server exposed on non-loopback host WITHOUT authentication",
                Some(&format!(
                    "host '{}' accepts unauthenticated requests from any reachable client. \
                     Set VANTADB_API_KEY (or remove --allow-insecure) to secure this server.",
                    config.host
                )),
            );
        } else {
            crate::console::error(
                "Refusing to start: non-loopback host without an API key",
                Some(&format!(
                    "Binding '{}' without VANTADB_API_KEY exposes an unauthenticated \
                     server to the network. Fix either way: (1) set VANTADB_API_KEY to \
                     enable Bearer auth, or (2) bind a loopback host (127.0.0.1/localhost/::1), \
                     or (3) pass --allow-insecure to override this check in dev.",
                    config.host
                )),
            );
            return Err(VantaError::InvalidInput(format!(
                "non-loopback host '{}' without api_key; set VANTADB_API_KEY, bind a \
                 loopback host, or pass --allow-insecure",
                config.host
            )));
        }
    }
    Ok(())
}

fn log_security_mode(config: &VantaConfig) {
    let auth_status = match (&config.api_key, config.require_auth) {
        (Some(_), true) => "Bearer token auth ✓ (forced)",
        (Some(_), false) => "Bearer token auth ✓",
        (None, true) => "ERROR: require_auth but no key configured",
        (None, false) => "No auth (dev mode)",
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

    crate::console::ok(
        "Security",
        Some(&format!(
            "{} | {} | {}",
            auth_status, rate_status, tls_status
        )),
    );
}

/// Flush storage and log the result using spawn_blocking to avoid blocking Tokio.
async fn flush_on_shutdown_async(storage: Arc<StorageEngine>) {
    crate::console::warn("Flushing storage before exit...", None);
    let flush_res = tokio::task::spawn_blocking(move || storage.flush()).await;

    match flush_res {
        Ok(Err(e)) => crate::console::error("Flush failed during shutdown", Some(&e.to_string())),
        Ok(Ok(())) => crate::console::ok("Storage flushed", None),
        Err(e) => {
            crate::console::error("Flush task panicked during shutdown", Some(&e.to_string()))
        }
    }
    #[cfg(feature = "opentelemetry")]
    crate::server::telemetry::shutdown_telemetry();
}

/// Returns `true` if the server completed a graceful shutdown (flush was called).
#[cfg_attr(not(feature = "tls"), allow(unused_variables))]
async fn serve_http_or_tls(
    router: axum::Router,
    addr: String,
    config: &VantaConfig,
    storage: Arc<StorageEngine>,
) -> bool {
    #[cfg(feature = "tls")]
    if let (Some(cert), Some(key)) = (&config.tls_cert_path, &config.tls_key_path) {
        let tls_config = match build_tls13_config(cert, key).await {
            Ok(c) => axum_server::tls_rustls::RustlsConfig::from_config(Arc::new(c)),
            Err(e) => {
                crate::console::error("Failed to load TLS certificate/key", Some(&e.to_string()));
                flush_on_shutdown_async(storage.clone()).await;
                return false;
            }
        };

        let socket_addr: std::net::SocketAddr = match addr.parse() {
            Ok(a) => a,
            Err(e) => {
                crate::console::error("Invalid bind address", Some(&e.to_string()));
                flush_on_shutdown_async(storage.clone()).await;
                return false;
            }
        };

        crate::console::print_ready(&format!("https://{}", addr));

        let handle = axum_server::Handle::new();
        let handle_clone = handle.clone();
        let storage_clone = storage.clone();
        tokio::spawn(async move {
            wait_for_shutdown_signal().await;
            crate::console::warn("Shutting down TLS server gracefully...", None);
            flush_on_shutdown_async(storage_clone).await;
            handle_clone.graceful_shutdown(Some(Duration::from_secs(10)));
        });

        if let Err(e) = axum_server::bind_rustls(socket_addr, tls_config)
            .handle(handle)
            .serve(router.into_make_service_with_connect_info::<std::net::SocketAddr>())
            .await
        {
            crate::console::error("TLS server terminated unexpectedly", Some(&e.to_string()));
            flush_on_shutdown_async(storage.clone()).await;
            return false;
        }

        flush_on_shutdown_async(storage.clone()).await;
        return true;
    }

    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => {
            crate::console::ok("TCP listener bound", Some(&addr));
            l
        }
        Err(e) => {
            crate::console::error("Failed to bind port", Some(&e.to_string()));
            flush_on_shutdown_async(storage.clone()).await;
            return false;
        }
    };

    crate::console::print_ready(&addr);

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    tokio::spawn(async move {
        wait_for_shutdown_signal().await;
        crate::console::warn("Shutting down HTTP server gracefully...", None);
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
        crate::console::error("Server terminated unexpectedly", Some(&e.to_string()));
    }

    flush_on_shutdown_async(storage.clone()).await;
    true
}

/// Build a rustls TLS 1.3 server config from PEM certificate and key files.
#[cfg(feature = "tls")]
pub async fn build_tls13_config(
    cert_path: &str,
    key_path: &str,
) -> std::io::Result<rustls::ServerConfig> {
    use rustls::pki_types::pem::PemObject;
    use rustls::pki_types::{CertificateDer, PrivateKeyDer};

    let cert_bytes = tokio::fs::read(cert_path).await?;
    let key_bytes = tokio::fs::read(key_path).await?;

    let certs: Vec<CertificateDer> = CertificateDer::pem_slice_iter(&cert_bytes)
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let mut keys: Vec<PrivateKeyDer> = PrivateKeyDer::pem_slice_iter(&key_bytes)
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    if keys.len() != 1 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "expected exactly one private key in PEM file",
        ));
    }

    let key = keys.pop().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "expected exactly one private key",
        )
    })?;

    // Include TLSv1.2 alongside TLSv1.3 for compatibility with legacy HTTP
    // clients (e.g. older curl, Java 8, Python <3.7) that do not support
    // TLSv1.3 exclusively.
    let mut config = rustls::ServerConfig::builder_with_protocol_versions(&[
        &rustls::version::TLS12,
        &rustls::version::TLS13,
    ])
    .with_no_client_auth()
    .with_single_cert(certs, key)
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    Ok(config)
}

/// Start the HTTP (or TLS) server, binding to the address in the config.
pub async fn run(config: VantaConfig) -> Result<()> {
    init_telemetry(false, Some(config.log_format));

    crate::console::print_banner();

    validate_auth_config(&config)?;

    crate::console::progress("Initializing storage engine...", None);

    let storage = match StorageEngine::open_with_config(&config.storage_path, Some(config.clone()))
    {
        Ok(s) => {
            crate::console::ok("Storage engine opened", Some(&config.storage_path));
            Arc::new(s)
        }
        Err(e) => {
            crate::console::error("Failed to open storage engine", Some(&e.to_string()));
            return Err(e);
        }
    };

    log_security_mode(&config);

    let api_key: Option<Arc<str>> = config.api_key.as_deref().map(Arc::from);
    let alt_api_key: Option<Arc<str>> = config.alt_api_key.as_deref().map(Arc::from);
    let circuit_breaker = Arc::new(CircuitBreaker::new(
        config.circuit_breaker_failure_threshold,
        Duration::from_secs(config.circuit_breaker_open_timeout_secs),
    ));
    let pool = Arc::new(ConnectionPool::new(
        config.max_connections,
        Duration::from_millis(config.pool_acquire_timeout_ms),
    ));
    let rbac_config = config.rbac_config.clone();
    let state = Arc::new(ServerState {
        storage: storage.clone(),
        db: crate::sdk::VantaEmbedded::from_engine(storage.clone()),
        circuit_breaker,
        pool,
        api_key,
        alt_api_key,
        rbac_config,
        trusted_proxies: config.trusted_proxies.clone(),
        conversation_trigger: None,
    });

    // MOD-12 (MCP-01 twin): a raw StorageEngine skips the
    // `VantaEmbedded::open_with_config` index reconciliation, so lexical/hybrid
    // searches fail on fresh DBs with "text_index not found". Ensure index
    // state at startup: idempotent — no-op when counts match, writes fresh
    // empty state for new DBs. Read-only engines cannot rebuild, so they are
    // skipped (same guard as `open_with_config`).
    if !config.read_only {
        if let Err(e) = state.db.ensure_indexes_current() {
            crate::console::error(
                "Failed to ensure index state at startup; text search may be unavailable",
                Some(&e.to_string()),
            );
        }
    }

    let rpm = config.rate_limit_rpm;
    let router = app_with_cors(state, rpm, &config.allowed_origins);
    let router = mount_dashboard(router, config.dashboard_dir.as_deref());
    let addr = format!("{}:{}", config.host, config.port);

    if !serve_http_or_tls(router, addr, &config, storage.clone()).await {
        return Err(VantaError::CliError(ChainedError::msg(
            "Server exited with errors",
        )));
    }

    Ok(())
}

/// Wait for SIGINT (or SIGTERM on Unix) to trigger graceful shutdown.
pub async fn wait_for_shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();
    #[cfg(unix)]
    let mut sigterm = match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
    {
        Ok(s) => s,
        Err(e) => {
            crate::console::error("Failed to install SIGTERM handler", Some(&e.to_string()));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::VantaConfig;
    use crate::VantaError;

    #[test]
    fn validate_auth_allows_key_without_require() {
        let cfg = VantaConfig {
            api_key: Some("sk-test".into()),
            require_auth: false,
            ..Default::default()
        };
        assert!(validate_auth_config(&cfg).is_ok());
    }

    #[test]
    fn validate_auth_allows_no_key_without_require() {
        let cfg = VantaConfig {
            api_key: None,
            require_auth: false,
            host: "127.0.0.1".into(),
            ..Default::default()
        };
        assert!(validate_auth_config(&cfg).is_ok());
    }

    #[test]
    fn validate_auth_allows_key_with_require() {
        let cfg = VantaConfig {
            api_key: Some("sk-test".into()),
            require_auth: true,
            ..Default::default()
        };
        assert!(validate_auth_config(&cfg).is_ok());
    }

    #[test]
    fn validate_auth_rejects_no_key_with_require() {
        let cfg = VantaConfig {
            api_key: None,
            require_auth: true,
            ..Default::default()
        };
        let err = validate_auth_config(&cfg).unwrap_err();
        match err {
            VantaError::InvalidInput(msg) => {
                assert!(msg.contains("require_auth"), "msg: {msg}");
            }
            other => panic!("expected InvalidInput, got {other:?}"),
        }
    }

    /// FIND-07 (a): non-loopback host + no key → refuse with actionable message.
    #[test]
    fn refuse_to_start_non_loopback_without_key() {
        for host in ["0.0.0.0", "192.168.1.10", "example.com", "::"] {
            let cfg = VantaConfig {
                api_key: None,
                require_auth: false,
                allow_insecure: false,
                host: host.into(),
                ..Default::default()
            };
            let err = validate_auth_config(&cfg).unwrap_err();
            match err {
                VantaError::InvalidInput(msg) => {
                    assert!(
                        msg.contains("VANTADB_API_KEY") && msg.contains("allow-insecure"),
                        "host {host}: msg lacks remediation: {msg}"
                    );
                }
                other => panic!("expected InvalidInput for {host}, got {other:?}"),
            }
        }
    }

    /// FIND-07 (b): same non-loopback host + `--allow-insecure` → starts
    /// (with a prominent WARNING logged to console).
    #[test]
    fn allow_insecure_bypasses_non_loopback_refusal() {
        let cfg = VantaConfig {
            api_key: None,
            require_auth: false,
            allow_insecure: true,
            host: "0.0.0.0".into(),
            ..Default::default()
        };
        assert!(validate_auth_config(&cfg).is_ok());
    }

    /// FIND-07 (c): loopback hosts without a key start normally.
    #[test]
    fn loopback_hosts_start_normally() {
        for host in ["127.0.0.1", "localhost", "::1", "[::1]"] {
            let cfg = VantaConfig {
                api_key: None,
                require_auth: false,
                allow_insecure: false,
                host: host.into(),
                ..Default::default()
            };
            assert!(
                validate_auth_config(&cfg).is_ok(),
                "loopback host {host} must start without a key"
            );
        }
    }

    /// FIND-07: an API key makes any host acceptable regardless of the override.
    #[test]
    fn api_key_accepts_any_host() {
        let cfg = VantaConfig {
            api_key: Some("sk-test".into()),
            require_auth: false,
            allow_insecure: false,
            host: "0.0.0.0".into(),
            ..Default::default()
        };
        assert!(validate_auth_config(&cfg).is_ok());
    }

    #[test]
    fn is_loopback_host_classification() {
        assert!(is_loopback_host("127.0.0.1"));
        assert!(is_loopback_host("127.9.9.9"));
        assert!(is_loopback_host("localhost"));
        assert!(is_loopback_host("LOCALHOST"));
        assert!(is_loopback_host("::1"));
        assert!(is_loopback_host("[::1]"));
        assert!(!is_loopback_host("0.0.0.0"));
        assert!(!is_loopback_host("::"));
        assert!(!is_loopback_host("192.168.1.10"));
        assert!(!is_loopback_host("db.internal")); // unresolvable → fail closed
        assert!(!is_loopback_host(""));
    }
}
