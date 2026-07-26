//! Agentic message thread storage with optional TTL.
//!
//! [`ThreadStore`] wraps a [`StorageEngine`] to persist conversation threads
//! as [`UnifiedNode`]s with JSON-serialized messages. Threads can opt into
//! TTL-based expiry via [`GcWorker`].

use crate::backend::BackendPartition;
use crate::error::{ChainedError, Result, VantaError};
use crate::gc::GcWorker;
use crate::node::{FieldValue, UnifiedNode};
use crate::storage::StorageEngine;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use web_time::{SystemTime, UNIX_EPOCH};

// ── Types ──

/// A single message in an agentic conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub timestamp: u64,
    pub metadata: HashMap<String, String>,
}

/// A conversation thread containing an ordered list of messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageThread {
    pub thread_id: u128,
    pub title: String,
    pub messages: Vec<Message>,
    pub created_at: u64,
    pub updated_at: u64,
    pub metadata: HashMap<String, String>,
}

// ── Field keys stored on each thread node ──

const FIELD_TITLE: &str = "_title";
const FIELD_MESSAGES: &str = "_messages";
const FIELD_CREATED_AT: &str = "_created_at";
const FIELD_UPDATED_AT: &str = "_updated_at";
const FIELD_METADATA: &str = "_metadata";
const FIELD_EXPIRES_AT: &str = "_expires_at";
const FIELD_TTL_SECS: &str = "_ttl_secs";

/// InternalMetadata key for the sorted list of thread IDs.
const THREAD_INDEX_KEY: &[u8] = b"_thread_ids";

// ── helpers ──

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn generate_id() -> u128 {
    rand::rng().random()
}

// ── ThreadStore ──

/// CRUD store for agentic message threads backed by a [`StorageEngine`].
///
/// Threads are stored as nodes in the default partition and indexed via a
/// serialized ID list in the `InternalMetadata` partition. Callers that want
/// TTL-based expiry pass a [`GcWorker`] reference into the relevant methods.
pub struct ThreadStore<'a> {
    engine: &'a StorageEngine,
}

impl<'a> ThreadStore<'a> {
    /// Wrap a storage engine reference.
    pub fn new(engine: &'a StorageEngine) -> Self {
        Self { engine }
    }

    /// Create a new thread.
    ///
    /// `ttl_secs` — if set, the thread auto-expires after this many seconds.
    /// The thread's messages are deleted on sweep.
    ///
    /// `gc` — if provided, the TTL expiry is registered with the garbage
    /// collector so it can be cleaned up automatically.
    pub fn create_thread(
        &self,
        title: &str,
        metadata: HashMap<String, String>,
        ttl_secs: Option<u64>,
        gc: Option<&mut GcWorker<'a>>,
    ) -> Result<u128> {
        let thread_id = generate_id();
        let now = now_secs();

        let empty_messages = serde_json::to_string(&Vec::<Message>::new())
            .map_err(|e| VantaError::serialization(ChainedError::with_source("messages", e)))?;
        let metadata_json = serde_json::to_string(&metadata)
            .map_err(|e| VantaError::serialization(ChainedError::with_source("metadata", e)))?;

        let mut node = UnifiedNode::new(thread_id);
        node.set_field(FIELD_TITLE, FieldValue::String(title.to_string()));
        node.set_field(FIELD_MESSAGES, FieldValue::String(empty_messages));
        node.set_field(FIELD_CREATED_AT, FieldValue::Int(now as i64));
        node.set_field(FIELD_UPDATED_AT, FieldValue::Int(now as i64));
        node.set_field(FIELD_METADATA, FieldValue::String(metadata_json));

        if let Some(ttl) = ttl_secs {
            let expires_at = now + ttl;
            node.set_field(FIELD_EXPIRES_AT, FieldValue::Int(expires_at as i64));
            node.set_field(FIELD_TTL_SECS, FieldValue::Int(ttl as i64));
            if let Some(gc) = gc {
                gc.register_ttl(thread_id, expires_at);
            }
        }

        self.engine.insert(&node)?;
        self.add_thread_id(thread_id)?;

        Ok(thread_id)
    }

    /// Append a message to an existing thread.
    ///
    /// The thread's TTL (if any) is refreshed by re-registering with the
    /// original duration. Pass `gc` to keep the GC index in sync.
    pub fn send_message(
        &self,
        thread_id: u128,
        role: &str,
        content: &str,
        metadata: HashMap<String, String>,
        gc: Option<&mut GcWorker<'a>>,
    ) -> Result<()> {
        let now = now_secs();
        let mut node = self
            .engine
            .get(thread_id)?
            .ok_or(VantaError::NodeNotFound(thread_id))?;

        let mut messages: Vec<Message> = self.load_messages(&node)?;
        messages.push(Message {
            role: role.to_string(),
            content: content.to_string(),
            timestamp: now_ms(),
            metadata,
        });

        let messages_json = serde_json::to_string(&messages)
            .map_err(|e| VantaError::serialization(ChainedError::with_source("messages", e)))?;
        node.set_field(FIELD_MESSAGES, FieldValue::String(messages_json));
        node.set_field(FIELD_UPDATED_AT, FieldValue::Int(now as i64));

        // Refresh TTL if the thread was created with one
        if let Some(FieldValue::Int(ttl_secs)) = node.get_field(FIELD_TTL_SECS) {
            if *ttl_secs > 0 {
                let new_expires_at = now + *ttl_secs as u64;
                node.set_field(FIELD_EXPIRES_AT, FieldValue::Int(new_expires_at as i64));
                if let Some(gc) = gc {
                    gc.register_ttl(thread_id, new_expires_at);
                }
            }
        }

        self.engine.insert(&node)?;
        Ok(())
    }

    /// Retrieve a thread by its ID.
    pub fn get_thread(&self, thread_id: u128) -> Result<Option<MessageThread>> {
        match self.engine.get(thread_id)? {
            Some(node) => Ok(Some(self.node_to_thread(node)?)),
            None => Ok(None),
        }
    }

    /// List threads with pagination.
    pub fn list_threads(&self, limit: usize, offset: usize) -> Result<Vec<MessageThread>> {
        let ids = self.load_thread_ids()?;
        let chunk: Vec<u128> = ids.into_iter().skip(offset).take(limit).collect();

        let mut threads = Vec::with_capacity(chunk.len());
        for id in chunk {
            if let Some(thread) = self.get_thread(id)? {
                threads.push(thread);
            }
        }
        Ok(threads)
    }

    /// Delete a thread by its ID.
    pub fn delete_thread(&self, thread_id: u128) -> Result<()> {
        self.engine.delete(thread_id, "delete_thread")?;
        self.remove_thread_id(thread_id)
    }

    /// Remove all threads whose TTL has expired.
    ///
    /// Returns the number of threads purged.
    pub fn purge_expired_threads(&self) -> Result<usize> {
        let now = now_secs();
        let ids = self.load_thread_ids()?;
        let mut purged = 0;

        for id in &ids {
            if let Some(node) = self.engine.get(*id)? {
                if self.is_expired(&node, now) {
                    self.engine.delete(*id, "ttl_expired")?;
                    purged += 1;
                }
            }
        }

        // Rebuild the index — keep only non-expired threads that still exist
        if purged > 0 {
            let remaining: Vec<u128> = self
                .load_thread_ids()?
                .into_iter()
                .filter(|id| {
                    self.engine
                        .get(*id)
                        .ok()
                        .flatten()
                        .is_some_and(|node| !self.is_expired(&node, now_secs()))
                })
                .collect();
            self.save_thread_ids(&remaining)?;
        }

        Ok(purged)
    }

    // ── internal helpers ──

    fn is_expired(&self, node: &UnifiedNode, now: u64) -> bool {
        node.get_field(FIELD_EXPIRES_AT).is_some_and(|f| match f {
            FieldValue::Int(exp) => *exp > 0 && (*exp as u64) < now,
            _ => false,
        })
    }

    fn load_messages(&self, node: &UnifiedNode) -> Result<Vec<Message>> {
        match node.get_field(FIELD_MESSAGES) {
            Some(FieldValue::String(json)) => serde_json::from_str(json)
                .map_err(|e| VantaError::serialization(ChainedError::with_source("messages", e))),
            _ => Ok(Vec::new()),
        }
    }

    fn load_metadata(&self, node: &UnifiedNode) -> Result<HashMap<String, String>> {
        match node.get_field(FIELD_METADATA) {
            Some(FieldValue::String(json)) => serde_json::from_str(json)
                .map_err(|e| VantaError::serialization(ChainedError::with_source("metadata", e))),
            _ => Ok(HashMap::new()),
        }
    }

    fn node_to_thread(&self, node: UnifiedNode) -> Result<MessageThread> {
        let title = match node.get_field(FIELD_TITLE) {
            Some(FieldValue::String(s)) => s.clone(),
            _ => String::new(),
        };
        let messages = self.load_messages(&node)?;
        let metadata = self.load_metadata(&node)?;
        let created_at = match node.get_field(FIELD_CREATED_AT) {
            Some(FieldValue::Int(v)) => *v as u64,
            _ => 0,
        };
        let updated_at = match node.get_field(FIELD_UPDATED_AT) {
            Some(FieldValue::Int(v)) => *v as u64,
            _ => 0,
        };

        Ok(MessageThread {
            thread_id: node.id,
            title,
            messages,
            created_at,
            updated_at,
            metadata,
        })
    }

    fn load_thread_ids(&self) -> Result<Vec<u128>> {
        match self
            .engine
            .get_from_partition(BackendPartition::InternalMetadata, THREAD_INDEX_KEY)?
        {
            Some(bytes) => serde_json::from_slice(&bytes).map_err(|e| {
                VantaError::serialization(ChainedError::with_source("thread index", e))
            }),
            None => Ok(Vec::new()),
        }
    }

    fn save_thread_ids(&self, ids: &[u128]) -> Result<()> {
        let bytes = serde_json::to_vec(ids)
            .map_err(|e| VantaError::serialization(ChainedError::with_source("thread index", e)))?;
        self.engine
            .put_to_partition(BackendPartition::InternalMetadata, THREAD_INDEX_KEY, &bytes)
    }

    fn add_thread_id(&self, id: u128) -> Result<()> {
        let mut ids = self.load_thread_ids()?;
        if !ids.contains(&id) {
            ids.push(id);
            self.save_thread_ids(&ids)?;
        }
        Ok(())
    }

    fn remove_thread_id(&self, id: u128) -> Result<()> {
        let mut ids = self.load_thread_ids()?;
        ids.retain(|&i| i != id);
        self.save_thread_ids(&ids)
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
