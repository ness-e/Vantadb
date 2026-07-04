#![warn(missing_docs)]

//! Crate-level re-exports for the VantaDB server binary. This crate is the
//! executable entrypoint; most server logic lives in the `vantadb` crate.

/// Middleware layer for HTTP request processing.
pub mod middleware;
/// HTTP server entrypoint and configuration.
pub mod server;
