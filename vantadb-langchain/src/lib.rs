#![warn(missing_docs)]

//! LangChain vector store integration crate.
//!
//! Provides a [`VantaDBVectorStore`] Python class that implements the LangChain
//! ``VectorStore`` interface backed by VantaDB for persistent vector storage
//! and similarity search.
//!
//! Usage::
//!
//!     from vantadb_langchain import VantaDBVectorStore
//!     from langchain_core.documents import Document
//!
//!     store = VantaDBVectorStore("/tmp/vantadb-langchain")
//!     store.add_texts(["Paris is the capital of France"], [[0.1, 0.2, ...]])
//!     results = store.similarity_search_by_vector([0.1, 0.2, ...], k=5)

#[cfg(feature = "python")]
mod python;

#[cfg(feature = "python")]
pub use python::*;
