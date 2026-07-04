#![warn(missing_docs)]

//! Mem0 integration crate. Provides a [`VantaDBStore`] Python class that
//! implements the Mem0 vector-store protocol backed by VantaDB.

#[cfg(feature = "python")]
mod python;

#[cfg(feature = "python")]
pub use python::*;
