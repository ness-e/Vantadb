//! Server bootstrap: config validation, TLS, run loop, graceful shutdown.
//!
//! REVIEW-10: extracted from `routing.rs` — everything that starts and runs
//! the HTTP/TLS server.

use crate::circuit_breaker::CircuitBreaker;
use crate::config::Config;
use crate::connection_pool::ConnectionPool;
use crate::error::ChainedError;
use crate::error::Result;
use crate::server::router::{app_with_cors, mount_dashboard};
use crate::server::state::{BackgroundService, ServerHooks, ServerState};
use crate::server::telemetry::init_telemetry;
use crate::storage::StorageEngine;
use crate::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;

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
pub fn validate_auth_config(config: &Config) -> Result<()> {
    if config.alt_api_key.is_some() && config.api_key.is_none() {
        return Err(Error::InvalidInput(
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
        return Err(Error::InvalidInput(
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
            return Err(Error::InvalidInput(format!(
                "non-loopback host '{}' without api_key; set VANTADB_API_KEY, bind a \
                 loopback host, or pass --allow-insecure",
                config.host
            )));
        }
    }
    Ok(())
}

fn log_security_mode(config: &Config) {
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
    config: &Config,
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
///
/// Pre-WIRE-14 entry point, unchanged: equivalent to calling
/// [`run_with_hooks`] with [`ServerHooks::default()`].
pub async fn run(config: Config) -> Result<()> {
    run_with_hooks(config, ServerHooks::default()).await
}

/// Start the HTTP (or TLS) server with host-provided hooks (WIRE-14).
///
/// Additive seam over [`run`]: hosts inject a [`crate::server::state::ConversationTrigger`]
/// (post-save hook for `POST /api/v2/conversations`) and background services
/// they have already started. The server owns those services for its lifetime
/// and joins them after the HTTP loop returns — the same lifecycle as the
/// built-in TTL sweeper — so no service task outlives `run`.
///
/// WIRE-16: hosts that need the server's database handle (single writer) set
/// [`ServerHooks::on_storage_ready`] and complete their trigger/services
/// there, once storage is open.
pub async fn run_with_hooks(config: Config, mut hooks: ServerHooks) -> Result<()> {
    init_telemetry(
        crate::server::telemetry::TelemetrySink::Stdout,
        Some(config.log_format),
    );

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

    let db = crate::sdk::Embedded::from_engine(storage.clone());

    // MOD-12 (MCP-01 twin): a raw StorageEngine skips the
    // `Embedded::open_with_config` index reconciliation, so lexical/hybrid
    // searches fail on fresh DBs with "text_index not found". Ensure index
    // state at startup: idempotent — no-op when counts match, writes fresh
    // empty state for new DBs. Read-only engines cannot rebuild, so they are
    // skipped (same guard as `open_with_config`). Runs before the deferred
    // host wiring (WIRE-16) so a host-started service never races the
    // reconcile.
    if !config.read_only {
        if let Err(e) = db.ensure_indexes_current() {
            crate::console::error(
                "Failed to ensure index state at startup; text search may be unavailable",
                Some(&e.to_string()),
            );
        }
    }

    // WIRE-16: deferred host wiring — the host receives a clone of the
    // server's `Embedded` handle (single writer: the server owns the one
    // open) and completes its trigger/services before `ServerState` and the
    // router are built.
    if let Some(on_storage_ready) = hooks.on_storage_ready.take() {
        on_storage_ready(&mut hooks, db.clone());
    }

    let ServerHooks {
        conversation_trigger,
        mut background_services,
        ..
    } = hooks;

    let api_key: Option<Arc<str>> = config.api_key.as_deref().map(Arc::from);
    let alt_api_key: Option<Arc<str>> = config.alt_api_key.as_deref().map(Arc::from);
    let jwt_secret: Option<Arc<str>> = config.jwt_secret.as_deref().map(Arc::from);
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
        db,
        circuit_breaker,
        pool,
        api_key,
        alt_api_key,
        jwt_secret,
        rbac_config,
        trusted_proxies: config.trusted_proxies.clone(),
        conversation_trigger,
    });

    // WIRE-04: background TTL sweeper — keeps the TTL promise without manual
    // `purge_expired` calls (physically purges expired records + indexes).
    // Disabled on read-only engines (purge is a write) and when the configured
    // interval is 0. Held for the server's lifetime; stopped after the run
    // loop returns so no sweep races the shutdown flush. WIRE-14: joined
    // through the same host seam as injected services, after them.
    if !config.read_only && config.ttl_sweep_interval_ms > 0 {
        crate::console::ok(
            "TTL sweeper",
            Some(&format!(
                "purging expired records every {}ms",
                config.ttl_sweep_interval_ms
            )),
        );
        let sweeper = crate::gc::spawn_memory_ttl_sweeper(
            state.db.clone(),
            Duration::from_millis(config.ttl_sweep_interval_ms),
        );
        background_services.push(Box::new(sweeper));
    }

    let rpm = config.rate_limit_rpm;
    let router = app_with_cors(state, rpm, &config.allowed_origins);
    let router = mount_dashboard(router, config.dashboard_dir.as_deref());
    let addr = format!("{}:{}", config.host, config.port);

    if !serve_http_or_tls(router, addr, &config, storage.clone()).await {
        return Err(Error::Cli(ChainedError::msg("Server exited with errors")));
    }

    // WIRE-14: graceful stop of every owned background service — host services
    // first (registration order), then the built-in TTL sweeper. Mirrors the
    // pre-WIRE-14 join: no service races the shutdown flush.
    shutdown_background_services(background_services).await;

    Ok(())
}

/// Stop and join every background service, in registration order.
async fn shutdown_background_services(services: Vec<Box<dyn BackgroundService>>) {
    for service in services {
        service.shutdown().await;
    }
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
    use crate::config::Config;
    use crate::server::state::{BackgroundService, ServerHooks};
    use crate::Error;

    #[test]
    fn validate_auth_allows_key_without_require() {
        let cfg = Config {
            api_key: Some("sk-test".into()),
            require_auth: false,
            ..Default::default()
        };
        assert!(validate_auth_config(&cfg).is_ok());
    }

    #[test]
    fn validate_auth_allows_no_key_without_require() {
        let cfg = Config {
            api_key: None,
            require_auth: false,
            host: "127.0.0.1".into(),
            ..Default::default()
        };
        assert!(validate_auth_config(&cfg).is_ok());
    }

    #[test]
    fn validate_auth_allows_key_with_require() {
        let cfg = Config {
            api_key: Some("sk-test".into()),
            require_auth: true,
            ..Default::default()
        };
        assert!(validate_auth_config(&cfg).is_ok());
    }

    #[test]
    fn validate_auth_rejects_no_key_with_require() {
        let cfg = Config {
            api_key: None,
            require_auth: true,
            ..Default::default()
        };
        let err = validate_auth_config(&cfg).unwrap_err();
        match err {
            Error::InvalidInput(msg) => {
                assert!(msg.contains("require_auth"), "msg: {msg}");
            }
            other => panic!("expected InvalidInput, got {other:?}"),
        }
    }

    /// FIND-07 (a): non-loopback host + no key → refuse with actionable message.
    #[test]
    fn refuse_to_start_non_loopback_without_key() {
        for host in ["0.0.0.0", "192.168.1.10", "example.com", "::"] {
            let cfg = Config {
                api_key: None,
                require_auth: false,
                allow_insecure: false,
                host: host.into(),
                ..Default::default()
            };
            let err = validate_auth_config(&cfg).unwrap_err();
            match err {
                Error::InvalidInput(msg) => {
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
        let cfg = Config {
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
            let cfg = Config {
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
        let cfg = Config {
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

    // ── WIRE-14: host seam — background-service lifecycle ────────────────────

    /// Fake host service mirroring the TTL sweeper lifecycle (watch + join):
    /// a spawned loop that runs until its watch channel flips, with a
    /// `stopped` flag set right before the task completes — so a completed
    /// join implies the flag is set.
    struct FakeService {
        shutdown_tx: tokio::sync::watch::Sender<bool>,
        handle: Option<tokio::task::JoinHandle<()>>,
        stopped: Arc<std::sync::atomic::AtomicBool>,
    }

    impl FakeService {
        /// Spawn the fake loop; the returned receiver fires once it is running.
        fn spawn() -> (Self, tokio::sync::oneshot::Receiver<()>) {
            use std::sync::atomic::Ordering;
            let (shutdown_tx, mut shutdown_rx) = tokio::sync::watch::channel(false);
            let (started_tx, started_rx) = tokio::sync::oneshot::channel();
            let stopped = Arc::new(std::sync::atomic::AtomicBool::new(false));
            let stopped_task = stopped.clone();
            let handle = tokio::spawn(async move {
                let _ = started_tx.send(());
                let _ = shutdown_rx.changed().await;
                stopped_task.store(true, Ordering::SeqCst);
            });
            (
                Self {
                    shutdown_tx,
                    handle: Some(handle),
                    stopped,
                },
                started_rx,
            )
        }
    }

    impl BackgroundService for FakeService {
        fn shutdown(
            mut self: Box<Self>,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> {
            Box::pin(async move {
                let _ = self.shutdown_tx.send(true);
                if let Some(handle) = self.handle.take() {
                    let _ = handle.await;
                }
            })
        }
    }

    /// WIRE-14 contract: a host-spawned service handed to the seam is stopped
    /// and joined by the server's graceful-shutdown path.
    #[tokio::test]
    async fn background_services_shutdown_joins_spawned_service() {
        use std::sync::atomic::Ordering;
        // Arrange: the host starts the service before handing it over.
        let (service, started) = FakeService::spawn();
        let stopped = service.stopped.clone();
        assert!(started.await.is_ok(), "fake service loop must start");

        // Act: the server's shutdown path.
        shutdown_background_services(vec![Box::new(service)]).await;

        // Assert: the loop saw the stop signal and the join completed (the
        // flag is stored before the task returns; shutdown awaits its handle).
        assert!(
            stopped.load(Ordering::SeqCst),
            "service loop must be stopped and joined"
        );
    }

    /// Every registered service is joined, not just the first.
    #[tokio::test]
    async fn background_services_shutdown_joins_every_registered_service() {
        use std::sync::atomic::Ordering;
        let (first, first_started) = FakeService::spawn();
        let (second, second_started) = FakeService::spawn();
        let first_stopped = first.stopped.clone();
        let second_stopped = second.stopped.clone();
        assert!(first_started.await.is_ok(), "first loop must start");
        assert!(second_started.await.is_ok(), "second loop must start");

        shutdown_background_services(vec![Box::new(first), Box::new(second)]).await;

        assert!(first_stopped.load(Ordering::SeqCst), "first must be joined");
        assert!(
            second_stopped.load(Ordering::SeqCst),
            "second must be joined"
        );
    }

    /// Defaults keep the pre-WIRE-14 behavior exactly (plan contract):
    /// no trigger, no services.
    #[test]
    fn default_server_hooks_keep_pre_wire14_behavior() {
        let hooks = ServerHooks::default();
        assert!(hooks.conversation_trigger.is_none());
        assert!(hooks.background_services.is_empty());
        // WIRE-16: the deferred-wiring hook defaults inert too.
        assert!(hooks.on_storage_ready.is_none());
    }

    /// WIRE-16: the deferred host-wiring hook is settable with the documented
    /// signature — hosts complete trigger/services once the server shares its
    /// `Embedded` handle (single-writer; a second open is `DatabaseBusy`).
    #[test]
    fn on_storage_ready_defaults_inert_and_accepts_host_wiring() {
        let mut hooks = ServerHooks::default();
        assert!(
            hooks.on_storage_ready.is_none(),
            "default must keep the pre-WIRE-16 behavior"
        );
        hooks.on_storage_ready = Some(Box::new(
            |_hooks: &mut ServerHooks, _db: crate::sdk::Embedded| {},
        ));
        assert!(hooks.on_storage_ready.is_some());
    }

    /// The built-in TTL sweeper participates in the same lifecycle seam.
    #[test]
    fn memory_ttl_sweeper_implements_background_service() {
        fn assert_impl<T: BackgroundService>() {}
        assert_impl::<crate::gc::MemoryTtlSweeper>();
    }

    /// Runtime proof of the sweeper's trait impl: a real sweeper boxed as the
    /// seam sees it is joined through the generic path (no hang, no recursion
    /// into the trait method — the impl delegates to the inherent shutdown).
    #[tokio::test]
    async fn ttl_sweeper_joins_through_background_service_seam() {
        let dir = match tempfile::tempdir() {
            Ok(d) => d,
            Err(e) => panic!("tempdir: {e}"),
        };
        let config = Config {
            backend_kind: crate::storage::BackendKind::InMemory,
            ..Default::default()
        };
        let path = match dir.path().to_str() {
            Some(p) => p,
            None => panic!("non-utf8 tempdir path"),
        };
        let storage = match StorageEngine::open_with_config(path, Some(config)) {
            Ok(s) => Arc::new(s),
            Err(e) => panic!("open storage: {e}"),
        };
        let db = crate::sdk::Embedded::from_engine(storage);
        let sweeper = crate::gc::spawn_memory_ttl_sweeper(db, Duration::from_millis(20));

        shutdown_background_services(vec![Box::new(sweeper)]).await;
    }

    /// The historical facades (`cli_server` and `server::routing`) re-export
    /// the WIRE-14 seam for external hosts (WIRE-15/16). Compile-time pin.
    #[test]
    fn facades_reexport_wire14_seam() {
        let _hooks = crate::cli_server::ServerHooks::default();
        let _run = crate::cli_server::run_with_hooks;
        let _legacy_run = crate::server::routing::run_with_hooks;
        fn assert_trait<T: crate::cli_server::BackgroundService>() {}
        assert_trait::<crate::gc::MemoryTtlSweeper>();
    }
}
