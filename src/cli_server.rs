//! Backward-compatibility shim for the historical `crate::cli_server` module.
//!
//! REVIEW-10 split the 5327-line god-file into [`crate::server`]:
//! - [`crate::server::state`]    — shared types (`ServerState`, DTOs,
//!   `AuthState` / `AuthIdentity`, `RequestId`, `ConversationTrigger`).
//! - [`crate::server::routing`] — routing, RBAC middleware, telemetry, TLS,
//!   bootstrap and inline tests.
//!
//! Everything that used to live in `crate::cli_server` is re-exported here so
//! existing callers (`vantadb-server`, `tests/request_id.rs`,
//! `tests/rbac_namespace.rs`, `tests/server_auth_rotation.rs`,
//! `vanta-memory/tests/conversation_hook.rs`) keep working without edits.

pub use crate::server::*;
