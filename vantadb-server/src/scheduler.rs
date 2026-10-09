//! WIRE-16: wires the `vanta-memory` scheduler into the HTTP server
//! (ADR-0054 T3).
//!
//! `vantadb-server` is the host of the memory scheduler: the conversation
//! bridge (`HttpCaptureBridge`) is injected as the server's
//! [`ConversationTrigger`](vantadb::cli_server::ConversationTrigger) and the
//! loop helper (`vanta_memory::services::scheduler`) runs as a
//! [`BackgroundService`](vantadb::cli_server::BackgroundService), joined by
//! the server after its HTTP loop returns.
//!
//! # Single writer
//!
//! The scheduler runs in the process that owns the DB (this server, in HTTP
//! mode). The host cannot open the DB itself — a second
//! `StorageEngine::open_with_config` on the same path fails with
//! `DatabaseBusy` — so the wiring is attached through
//! [`ServerHooks::on_storage_ready`], which hands the host a clone of the
//! server's `Embedded` handle once storage is open.
//!
//! # Configuration (env, R-5)
//!
//! - `VANTADB_SCHEDULER_INTERVAL_MS`: scheduler pass interval in ms.
//!   Default `60000` (mirrors `VANTADB_TTL_SWEEP_INTERVAL_MS`); `0` disables
//!   the loop (the conversation bridge stays wired — captures are never
//!   lost).
//! - Runner: the FIND-112 surface (`VANTADB_INGEST_*`, inherited
//!   `VANTADB_LLM_*`/`VANTADB_OPENAI_*`, and `VANTADB_INGEST_CONFIG` or
//!   `<storage>/data/vanta-ingest.toml`). Secrets are env-only — the TOML
//!   never carries a key. No real engine configured → the pass skips
//!   observably (P4: nothing lost, nothing burns).

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use vanta_memory::core::abstractions::LlmRunner;
use vanta_memory::ingest::runner_config::{build_ingest_runner, ConcreteRunner, IngestRunnerCfg};
use vanta_memory::services::conversation_hook::HttpCaptureBridge;
use vanta_memory::services::scheduler::spawn_memory_scheduler;
use vanta_memory::utils::{LocalStateBackend, SystemClock};
use vantadb::cli_server::ServerHooks;
use vantadb::sdk::Embedded;

/// Default scheduler interval (ms): mirrors `VANTADB_TTL_SWEEP_INTERVAL_MS`
/// (`src/config.rs`). `0` disables the loop.
pub const DEFAULT_INTERVAL_MS: u64 = 60_000;

/// Parse the `VANTADB_SCHEDULER_INTERVAL_MS` value (R-5: warn + safe default,
/// never panic). `0` is a valid value (disables the loop).
pub fn parse_interval_ms(raw: Option<&str>) -> u64 {
    match raw {
        None => DEFAULT_INTERVAL_MS,
        Some(s) => match s.trim().parse::<u64>() {
            Ok(v) => v,
            Err(_) => {
                tracing::warn!(
                    value = %s,
                    default = DEFAULT_INTERVAL_MS,
                    "invalid VANTADB_SCHEDULER_INTERVAL_MS, using default"
                );
                DEFAULT_INTERVAL_MS
            }
        },
    }
}

/// Scheduler interval from the process environment.
pub fn scheduler_interval_ms_from_env() -> u64 {
    parse_interval_ms(
        std::env::var("VANTADB_SCHEDULER_INTERVAL_MS")
            .ok()
            .as_deref(),
    )
}

/// Pure path resolution (testable, no env): an explicit non-empty override
/// wins; else `<storage_path>/data/vanta-ingest.toml`.
fn resolve_ingest_toml_path(override_path: Option<&str>, storage_path: &str) -> PathBuf {
    match override_path.map(str::trim).filter(|p| !p.is_empty()) {
        Some(p) => PathBuf::from(p),
        None => Path::new(storage_path)
            .join("data")
            .join("vanta-ingest.toml"),
    }
}

/// Resolve the ingest runner TOML path the way `vantadb-mcp` does:
/// `VANTADB_INGEST_CONFIG` wins, else `<storage_path>/data/vanta-ingest.toml`
/// (the engine's `data_dir`; see `StorageEngine::init_storage`). An absent
/// file is not an error — defaults mean "no runner".
pub fn ingest_toml_path(storage_path: &str) -> PathBuf {
    resolve_ingest_toml_path(
        std::env::var("VANTADB_INGEST_CONFIG").ok().as_deref(),
        storage_path,
    )
}

/// Load the runner config from env + TOML (FIND-112 surface).
pub fn ingest_runner_cfg(storage_path: &str) -> IngestRunnerCfg {
    IngestRunnerCfg::from_env_toml(&ingest_toml_path(storage_path))
}

/// Production runner factory (built once, called once per pass — FIND-112):
/// maps the concrete runner from the config. A degraded config (no real
/// engine: local provider, or OpenAI without a key) maps to `None`, so the
/// pass skips observably instead of burning tasks through retry/dead-letter
/// (P4; WIRE-15 contract).
pub fn ingest_runner_factory(
    cfg: IngestRunnerCfg,
) -> impl Fn() -> Option<ConcreteRunner> + Send + Sync + 'static {
    move || match build_ingest_runner(&cfg) {
        Some(ConcreteRunner::None) => None,
        other => other,
    }
}

/// Wire the conversation bridge + scheduler into `hooks` using the server's
/// `Embedded` handle (called from [`ServerHooks::on_storage_ready`]).
///
/// The bridge is always wired (L0 capture + L1 enqueue; LLM-free, best-effort
/// — data is never lost). The loop is spawned only when `interval_ms > 0`;
/// it is joined by the server through the WIRE-14 `BackgroundService` seam.
/// Returns the shared queue so hosts/tests can observe its depth.
pub fn wire_memory<F, R>(
    hooks: &mut ServerHooks,
    db: Embedded,
    interval_ms: u64,
    runner_factory: F,
) -> Arc<LocalStateBackend<SystemClock>>
where
    F: Fn() -> Option<R> + Send + Sync + 'static,
    R: LlmRunner + Send + 'static,
{
    let queue = Arc::new(LocalStateBackend::new(SystemClock));
    hooks.conversation_trigger = Some(Arc::new(HttpCaptureBridge::new(db.clone(), queue.clone())));
    if interval_ms > 0 {
        let scheduler = spawn_memory_scheduler(
            queue.clone(),
            db,
            Duration::from_millis(interval_ms),
            runner_factory,
        );
        hooks.background_services.push(Box::new(scheduler));
    }
    queue
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_interval_ms_defaults_and_validates() {
        assert_eq!(parse_interval_ms(None), DEFAULT_INTERVAL_MS);
        assert_eq!(parse_interval_ms(Some("0")), 0, "0 disables the loop");
        assert_eq!(parse_interval_ms(Some("1500")), 1500);
        assert_eq!(parse_interval_ms(Some(" abc ")), DEFAULT_INTERVAL_MS);
        assert_eq!(parse_interval_ms(Some("-5")), DEFAULT_INTERVAL_MS);
    }

    #[test]
    fn ingest_runner_factory_degrades_to_none_without_engine() {
        let factory = ingest_runner_factory(IngestRunnerCfg::defaults());
        assert!(
            factory().is_none(),
            "local default has no chat engine → skip (P4), never burn tasks"
        );
    }

    #[test]
    fn ingest_runner_factory_builds_real_runner_for_configured_provider() {
        let cfg = IngestRunnerCfg::from_toml_str("[ingest]\nprovider = \"ollama\"\n");
        let factory = ingest_runner_factory(cfg);
        assert!(
            factory().is_some(),
            "a configured provider must build a real runner"
        );
    }

    #[test]
    fn resolve_ingest_toml_path_prefers_override_and_falls_back_to_storage() {
        let fallback = resolve_ingest_toml_path(None, "/tmp/vanta-x");
        assert!(fallback.ends_with(Path::new("data").join("vanta-ingest.toml")));
        let overridden = resolve_ingest_toml_path(Some("  /custom/ingest.toml  "), "/tmp/vanta-x");
        assert_eq!(overridden, PathBuf::from("/custom/ingest.toml"));
        let blank = resolve_ingest_toml_path(Some("   "), "/tmp/vanta-x");
        assert!(
            blank.ends_with(Path::new("data").join("vanta-ingest.toml")),
            "blank override must fall back to the storage path"
        );
    }
}
