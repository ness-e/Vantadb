//! Experimental and future-facing governance primitives.
//!
//! This crate depends on `vantadb` with the `governance` feature enabled,
//! which provides the core data structures (`AdmissionFilter`, `ConsistencyBuffer`,
//! `ConflictResolver`). This crate adds the operational logic that is not yet
//! stable enough for the core: maintenance workers, invalidation dispatch, etc.

// Re-export core governance types for convenience
pub use vantadb::governance::admission_filter;
pub use vantadb::governance::conflict_resolver;
pub use vantadb::governance::consistency;

pub use vantadb::governance::conflict_resolver::{ConfidenceArbiter, ConflictResolver, ResolutionResult};
pub use vantadb::governance::admission_filter::AdmissionFilter;
pub use vantadb::governance::consistency::{ConsistencyBuffer, ConsistencyRecord};

// Experimental-only modules
pub mod invalidations;
pub mod maintenance_worker;

pub use invalidations::InvalidationDispatcher;
pub use maintenance_worker::MaintenanceWorker;

pub use vantadb::governance::AuditableTombstone;
