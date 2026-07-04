#![warn(missing_docs)]

//! OpenAI integration crate. Provides a [`VantaDBOpenAI`] Python class that
//! wraps the OpenAI embedding API with VantaDB vector storage.

#[cfg(feature = "python")]
mod python;

#[cfg(feature = "python")]
pub use python::*;
