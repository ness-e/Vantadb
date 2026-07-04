#![warn(missing_docs)]

//! Haystack integration crate. Provides a [`VantaDBDocumentStore`] Python class
//! that implements the Haystack DocumentStore protocol backed by VantaDB.

#[cfg(feature = "python")]
mod python;

#[cfg(feature = "python")]
pub use python::*;
