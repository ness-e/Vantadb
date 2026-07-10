//! Serialization infrastructure for VantaDB nodes and indexes.
//!
//! Sub-modules implement zero-copy and portable archive formats used
//! during checkpointing, recovery, and mmap-based index access.

// Disabled — rkyv zero-copy archive format is kept as reference for future
// format iterations. The active codec is postcard (see CPIndex serialize/deserialize).
// To re-enable, replace `#[cfg(any())]` with `pub mod` and ensure rkyv
// dependencies are added to Cargo.toml.
#[cfg(any())]
pub mod rkyv_archives;
