//! HTTP server split by concern (REVIEW-10).
//!
//! Prior to this module, the entire HTTP server — routing, RBAC, TLS, OTEL,
//! bootstrap and inline tests — lived in a single `src/cli_server.rs` file of
//! 5300+ lines. The split moves:
//!
//! - Shared types and DTOs into [`state`].
//! - Error builders into [`errors`].
//! - Middleware (auth, rate-limiting, circuit breaker, metrics) into [`middleware`].
//! - OTEL/tracing setup into [`telemetry`].
//! - Bootstrap (run, shutdown, auth validation, TLS) into [`bootstrap`].
//! - Handlers (health, CRUD, search, graph, maintenance, threads, conversation, skills, snapshots) into [`handlers`].
//! - Router topology (app, CORS, dashboard) into [`router`].
//! - Legacy facade [`routing`] for backward compatibility (re-exports).
//!
//! The public surface under `crate::cli_server` is unchanged: all symbols
//! are re-exported here so existing callers (e.g. `vantadb-server`,
//! `tests/request_id.rs`) keep working without edits.

pub mod bootstrap;
pub mod errors;
pub mod handlers;
pub mod middleware;
pub mod router;
pub mod routing;
pub mod state;
pub mod telemetry;

// ── Public API re-exports ────────────────────────────────────────────────────
// Preserve the historical `crate::cli_server::*` surface byte-for-byte so
// external callers and downstream binaries do not need to update their
// imports (REVIEW-10 pre-mortem: do not break the public API).

// Types (from `state`)
pub use state::{
    AuthIdentity, AuthRateLimiter, AuthState, ConversationTrigger, NodeDTO, QueryRequest,
    QueryResponse, ServerState, LONG_REQUEST_TIMEOUT, REQUEST_TIMEOUT,
};

// Functions — only those declared `pub` in the original file.
#[cfg(feature = "tls")]
pub use bootstrap::build_tls13_config;
pub use bootstrap::{run, validate_auth_config, wait_for_shutdown_signal};

pub use middleware::client_ip;
pub use middleware::{auth_middleware, circuit_breaker_middleware, request_metrics_middleware};

pub use router::{app, app_with_cors, mount_dashboard};

pub use telemetry::init_telemetry;
#[cfg(feature = "opentelemetry")]
pub use telemetry::shutdown_telemetry;

// Re-export routing facade for callers that import via `crate::server::routing::*`
pub use routing as routing_legacy;
