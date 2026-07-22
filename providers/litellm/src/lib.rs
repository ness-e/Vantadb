#![warn(missing_docs)]

//! LiteLLM integration crate. Provides a [`VantaDBLiteLLM`] Python class that
//! wraps the LiteLLM embedding API with VantaDB vector storage.

#[cfg(feature = "python")]
mod python;

#[cfg(feature = "python")]
pub use python::*;
