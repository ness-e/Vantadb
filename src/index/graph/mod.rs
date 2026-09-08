//! HNSW graph index: types, mmap prefetch, core CPIndex and tests.
//! Split from graph.rs (FIND-48) — all public paths preserved via re-exports.

mod core;
mod prefetch;
#[cfg(test)]
mod tests;
mod types;

pub use core::*;
pub use prefetch::*;
pub use types::*;
