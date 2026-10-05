#![warn(missing_docs)]

//! Crate-level re-exports for the VantaDB server binary. This crate is the
//! executable entrypoint; most server logic lives in the `vantadb` crate.

/// HTTP server entrypoint and configuration.
pub mod server;

/// WIRE-16 (ADR-0054 T3): host wiring for the `vanta-memory` scheduler —
/// conversation bridge + scheduler loop attached through the server's
/// deferred `on_storage_ready` hook.
pub mod scheduler;
