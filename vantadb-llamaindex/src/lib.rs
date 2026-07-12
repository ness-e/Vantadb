#![warn(missing_docs)]

//! LlamaIndex vector store integration crate.
//!
//! Provides a [`VantaDBVectorStore`] Python class that implements the
//! LlamaIndex ``VectorStore`` interface backed by VantaDB for persistent
//! vector storage and similarity search.
//!
//! Usage::
//!
//!     from vantadb_llamaindex import VantaDBVectorStore
//!
//!     store = VantaDBVectorStore("/tmp/vantadb-llamaindex")
//!     store.add(["hello world"], [[0.1, 0.2, ...]])
//!     results = store.query([0.1, 0.2, ...], similarity_top_k=5)

#[cfg(feature = "python")]
mod python;

#[cfg(feature = "python")]
pub use python::*;
