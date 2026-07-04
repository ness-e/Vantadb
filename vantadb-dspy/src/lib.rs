#![warn(missing_docs)]

//! DSPy integration crate. Provides a [`VantaDBRM`] Python class that implements
//! the DSPy retriever protocol backed by VantaDB.

#[cfg(feature = "python")]
mod python;

#[cfg(feature = "python")]
pub use python::*;
