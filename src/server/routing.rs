//! Facade preserving `crate::server::routing::*` for backward compatibility.
//!
//! REVIEW-10: the 4609-line god-file has been split by concern into
//! `state`, `errors`, `middleware`, `telemetry`, `bootstrap`, `handlers`, and
//! `router`. This module re-exports the public surface so existing imports
//! (`crate::server::routing::app`, `crate::server::routing::auth_middleware`,
//! tests that `use super::*`, etc.) continue to work without edits.
//!
//! New code should import from the granular modules directly
//! (`crate::server::router::app`, `crate::server::handlers::health_check`, ...).

// ── Router ───────────────────────────────────────────────────────────────────
pub use crate::server::router::{app, app_with_cors, mount_dashboard};

// ── Middleware ───────────────────────────────────────────────────────────────
pub use crate::server::middleware::{
    auth_middleware, circuit_breaker_middleware, client_ip, request_metrics_middleware,
};

// ── Telemetry ────────────────────────────────────────────────────────────────
pub use crate::server::telemetry::init_telemetry;
#[cfg(feature = "opentelemetry")]
pub use crate::server::telemetry::shutdown_telemetry;

// ── Bootstrap ────────────────────────────────────────────────────────────────
#[cfg(feature = "tls")]
pub use crate::server::bootstrap::build_tls13_config;
pub use crate::server::bootstrap::{run, validate_auth_config, wait_for_shutdown_signal};

// ── Errors ───────────────────────────────────────────────────────────────────
pub use crate::server::errors::{
    not_found_response, panic_error_response, pool_error_response, query_error_response,
    thread_not_found_response, vanta_error_response, vanta_error_status,
};

// ── Handlers ─────────────────────────────────────────────────────────────────
pub use crate::server::handlers::{
    audit_events, conversation_add, execute_query, export_v2, graph_bfs, graph_centrality,
    graph_degree, graph_dfs, graph_pagerank, graph_v2_bfs, graph_v2_degree, graph_v2_dfs,
    health_check, health_v2, import_v2, iql_autocomplete, maintenance_compact, maintenance_flush,
    maintenance_purge, maintenance_rebuild_index, metrics_endpoint, metrics_v2, records_delete,
    records_delete_by_filter, records_get, records_list, records_put, records_put_batch,
    records_search, records_versions, skill_create, skill_delete, skill_listing, skill_patch,
    skill_update, snapshots_create, snapshots_list, threads_create, threads_delete, threads_get,
    threads_list, threads_send_message,
};

// ── State re-exports ─────────────────────────────────────────────────────────
pub use crate::server::state::{
    AuthIdentity, AuthState, ConversationTrigger, NodeDTO, QueryRequest, QueryResponse, RequestId,
    ServerState, AUTH_ENTITY_NS, LONG_REQUEST_TIMEOUT, REQUEST_TIMEOUT,
};

#[cfg(test)]
#[path = "cli_server_auth_tests.rs"]
mod auth_tests;
