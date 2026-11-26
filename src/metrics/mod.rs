//! Internal metrics collection and Prometheus integration.
//! Provides atomic counters, histograms, and snapshot types for
//! runtime introspection of engine behaviour and memory usage.

pub(crate) mod core;
// FIND-608 (ME-01): the empty `native` skeleton module was removed — its doc
// promised re-exports that never existed. Everything lives in `core`.

pub use core::*;
