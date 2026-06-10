//! Experimental and future-facing governance primitives.
//!
//! This crate is self-contained and provides all governance data structures
//! and operational logic that is not yet stable enough for the core:
//! admission control, conflict resolution, consistency buffers, maintenance workers,
//! and invalidation dispatch.

// Core governance modules (moved from vantadb core)
pub mod admission_filter;
pub mod conflict_resolver;
pub mod consistency;

pub use admission_filter::AdmissionFilter;
pub use conflict_resolver::{ConfidenceArbiter, ConflictResolver, ResolutionResult};
pub use consistency::{AuditableTombstone, ConsistencyBuffer, ConsistencyRecord};

// Experimental-only modules
pub mod invalidations;
pub mod maintenance_worker;

pub use invalidations::InvalidationDispatcher;
pub use maintenance_worker::MaintenanceWorker;
