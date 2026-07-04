//! VantaDB Enterprise — proprietary features requiring a commercial license.
//!
//! This crate extends `vantadb` with enterprise-grade features:
//! - **Encryption at rest** (AES-256-GCM, ChaCha20-Poly1305)
//! - **Audit logging** (JSONL format, timestamped operations)
//! - **RBAC** (scoped API tokens, role-based access control)
//! - **Async replication** (WAL shipping to replicas)
//!
//! # License
//! This crate is proprietary. Use requires a valid VantaDB Enterprise license.

#![warn(missing_docs)]
#![doc(html_root_url = "https://docs.rs/vantadb-enterprise")]

pub mod audit;
pub mod encryption;
pub mod rbac;
pub mod replication;

/// Enterprise license verification.
pub mod license;

/// Re-export enterprise configuration.
pub use config::EnterpriseConfig;

mod config;
