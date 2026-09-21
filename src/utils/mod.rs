//! Utility modules extracted from the governance framework (archived 2024-06); these are production-ready.
//!
//! These are stateless, useful utilities for multi-writer and multi-agent scenarios
//! that don't require the full runtime governance framework.

pub mod confidence_metrics;
pub mod duplicate_prevention;
pub mod fs;

pub use confidence_metrics::compute_confidence_friction;
