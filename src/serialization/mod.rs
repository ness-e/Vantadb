//! Serialization infrastructure for VantaDB nodes and indexes.
//!
//! Sub-modules implement zero-copy and portable archive formats used
//! during checkpointing, recovery, and mmap-based index access.

#[cfg(feature = "rkyv-serialization")]
pub mod rkyv_archives;
