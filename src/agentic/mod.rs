//! Agentic chat: message threads with TTL-based garbage collection.
//!
//! Provides [`MessageThread`], [`Message`], and [`ThreadStore`] for
//! creating, reading, updating, and deleting agentic conversation threads
//! with optional TTL expiry via the existing [`GcWorker`].
//!
//! Threads are stored as [`UnifiedNode`]s with messages serialized as JSON
//! on the node. Thread IDs are `u128` values generated via `rand::random`.
//! A threads index is maintained in the `InternalMetadata` partition for
//! efficient listing.

pub mod thread;
pub use thread::{Message, MessageThread, ThreadStore};
