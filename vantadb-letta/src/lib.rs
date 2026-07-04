#![warn(missing_docs)]

//! Letta (MemGPT) integration crate. Provides a [`LettaStore`] Python class
//! for storing and retrieving agent memories in VantaDB.

#[cfg(feature = "python")]
mod python;

#[cfg(feature = "python")]
pub use python::*;
