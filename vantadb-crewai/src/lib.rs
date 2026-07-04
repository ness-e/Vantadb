#![warn(missing_docs)]

//! CrewAI integration crate. Provides a [`CrewAIMemory`] Python class backed by
//! VantaDB for long-term and RAG memory storage.

#[cfg(feature = "python")]
mod python;

#[cfg(feature = "python")]
pub use python::*;
