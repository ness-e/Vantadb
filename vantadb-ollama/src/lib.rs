#![warn(missing_docs)]

//! Ollama integration crate. Provides a [`VantaDBOllama`] Python class that
//! wraps the Ollama local embedding API with VantaDB vector storage.

#[cfg(feature = "python")]
mod python;

#[cfg(feature = "python")]
pub use python::*;
