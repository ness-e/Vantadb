//! Agentic chat: message threads with a lazily-evaluated TTL.
//!
//! Provides [`MessageThread`], [`Message`], and [`ThreadStore`] for
//! creating, reading, updating, and deleting agentic conversation threads.
//! Expiry is **not** background garbage collection: a thread's deadline lives on
//! the node and is evaluated by
//! [`ThreadStore::purge_expired_threads`](thread::ThreadStore::purge_expired_threads)
//! (FIND-800). See the [`thread`] module docs for the semantics.
//!
//! Threads are stored as [`UnifiedNode`](crate::node::UnifiedNode)s with messages serialized as JSON
//! on the node. Thread IDs are `u128` values generated via `rand::random`.
//! A threads index is maintained in the `InternalMetadata` partition for
//! efficient listing.

pub mod thread;
pub use thread::{
    Clock, CreateThread, ManualClock, Message, MessageThread, SystemClock, ThreadStore,
};
