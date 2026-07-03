//! Vector operations: quantization and transform pipelines.
//!
//! Sub-modules provide SQ8 quantization, FWHT-based variance
//! redistribution, and other vector preprocessing for retrieval.

pub mod governor;
pub mod quantization;
pub mod transform;
