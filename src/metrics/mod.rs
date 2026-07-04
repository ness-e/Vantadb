//! Internal metrics collection and Prometheus integration.
//! Provides atomic counters, histograms, and snapshot types for
//! runtime introspection of engine behaviour and memory usage.

pub(crate) mod core;
pub(crate) mod native;
pub(crate) mod snapshot;

pub use core::*;
