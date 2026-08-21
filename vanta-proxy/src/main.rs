//! vanta-proxy binary: load TOML config, serve the wire proxy.

use std::path::PathBuf;

use vanta_proxy::config::ProxyConfig;
use vanta_proxy::error::ProxyError;
use vanta_proxy::server;

#[tokio::main]
async fn main() -> Result<(), ProxyError> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config_path = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("VANTA_PROXY_CONFIG").ok())
        .unwrap_or_else(|| "config.toml".to_string());
    let config = ProxyConfig::load(&PathBuf::from(&config_path))?;
    tracing::info!(
        config = %config_path,
        listen = %format!("{}:{}", config.server.host, config.server.port),
        upstream = %config.upstream.url,
        timeout_secs = config.upstream.forward_timeout_secs,
        "vanta-proxy starting"
    );

    let state = server::AppState::new(config.clone())?;
    let app = server::router(state);

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| ProxyError::Config(format!("bind {addr}: {e}")))?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| ProxyError::Forward(format!("serve: {e}")))?;
    tracing::info!("vanta-proxy stopped");
    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("shutdown signal received");
}
