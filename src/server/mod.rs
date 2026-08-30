//! HTTP server split by concern (REVIEW-10).
//!
//! Prior to this module, the entire HTTP server — routing, RBAC, TLS, OTEL,
//! bootstrap and inline tests — lived in a single `src/cli_server.rs` file of
//! 5300+ lines. The split moves:
//!
//! - Shared types and DTOs into [`state`].
//! - Routing, RBAC middleware, OTEL, TLS, bootstrap, handlers and tests
//!   into [`routing`].
//!
//! The public surface under `crate::cli_server` is unchanged: all symbols
//! are re-exported here so existing callers (e.g. `vantadb-server`,
//! `tests/request_id.rs`) keep working without edits.

pub mod routing;
pub mod state;

// ── Public API re-exports ────────────────────────────────────────────────────
// Preserve the historical `crate::cli_server::*` surface byte-for-byte so
// external callers and downstream binaries do not need to update their
// imports (REVIEW-10 pre-mortem: do not break the public API).
//
// The original `src/cli_server.rs` exposed only the following items; everything
// else was crate-private. This re-export block mirrors that surface.

// Types (from `state`)
pub use state::{
    AuthIdentity, AuthRateLimiter, AuthState, ConversationTrigger, NodeDTO, QueryRequest,
    QueryResponse, ServerState, LONG_REQUEST_TIMEOUT, REQUEST_TIMEOUT,
};

// Functions (from `routing`) — only those declared `pub` in the original file.
pub use routing::{
    app, app_with_cors, auth_middleware, circuit_breaker_middleware, client_ip, init_telemetry,
    request_metrics_middleware, run, wait_for_shutdown_signal,
};

// Feature-gated public items.
#[cfg(feature = "tls")]
pub use routing::build_tls13_config;

#[cfg(feature = "opentelemetry")]
pub use routing::shutdown_telemetry;
