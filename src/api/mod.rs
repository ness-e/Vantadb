//! Shared deterministic scoring and rank-fusion helpers (`scores`).
//!
//! HTTP server routes and handlers live under `crate::server` (behind the
//! `server` feature); this module exposes pure score-normalization utilities.

pub mod scores;
