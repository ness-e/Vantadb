//! Utility modules extracted from experimental governance.
//!
//! These are stateless, useful utilities for multi-writer and multi-agent scenarios
//! that don't require the full runtime governance framework.

pub mod duplicate_prevention;
pub mod confidence_metrics;

pub use duplicate_prevention::DuplicatePreventionFilter;
pub use confidence_metrics::{compute_confidence_friction, OriginCollisionTracker};