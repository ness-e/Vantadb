//! Agentic message thread storage with optional TTL.
//!
//! [`ThreadStore`] wraps a [`StorageEngine`] to persist conversation threads
//! as [`UnifiedNode`]s with JSON-serialized messages. Threads can opt into
//! TTL-based expiry via [`GcWorker`].

use crate::backend::BackendPartition;
use crate::error::{ChainedError, Error, Result};
use crate::gc::GcWorker;
use crate::node::{FieldValue, UnifiedNode};
use crate::storage::StorageEngine;
use parking_lot::Mutex;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
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

/// Immutable command object for [`ThreadStore::create`] (D3: F2 command-object).
///
/// Groups the data params so the `create` signature stays thin. `gc` stays a
/// separate param: it is a service handle (`&mut GcWorker`), not write data.
/// Borrows `title` (`&str` at every caller) and moves `metadata` (already
/// owned at every caller).
#[derive(Debug, Clone)]
pub struct CreateThread<'a> {
    pub title: &'a str,
    pub metadata: HashMap<String, String>,
    pub ttl_secs: Option<u64>,
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

// ── Clock (FIRST-Repeatable, C2T2) ──

/// Time source for [`ThreadStore`].
///
/// Production code uses [`SystemClock`] (wall time). Tests inject
/// [`ManualClock`] and travel forward instead of `sleep`ing through a TTL,
/// which is flaky under CI scheduling pressure.
pub trait Clock: Send + Sync + std::fmt::Debug {
    /// Current time in whole seconds since UNIX epoch.
    fn now_secs(&self) -> u64;
    /// Current time in milliseconds since UNIX epoch.
    fn now_ms(&self) -> u64;
}

/// [`Clock`] backed by wall time (`web_time`, WASM-safe).
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_secs(&self) -> u64 {
        now_secs()
    }
    fn now_ms(&self) -> u64 {
        now_ms()
    }
}

/// [`Clock`] with manually advanced virtual time, for deterministic TTL tests.
///
/// Both channels move together: `advance_secs` also advances millis so
/// `created_at` (secs) and message `timestamp` (ms) stay consistent.
#[derive(Debug, Default)]
pub struct ManualClock {
    secs: AtomicU64,
    ms: AtomicU64,
}

impl ManualClock {
    /// Create a clock pinned at `start_secs` / `start_ms`.
    pub fn new(start_secs: u64, start_ms: u64) -> Self {
        Self {
            secs: AtomicU64::new(start_secs),
            ms: AtomicU64::new(start_ms),
        }
    }

    /// Move virtual time forward by `delta` seconds (saturating).
    pub fn advance_secs(&self, delta: u64) {
        self.secs.fetch_add(delta, Ordering::SeqCst);
        self.ms
            .fetch_add(delta.saturating_mul(1_000), Ordering::SeqCst);
    }
}

impl Clock for ManualClock {
    fn now_secs(&self) -> u64 {
        self.secs.load(Ordering::SeqCst)
    }
    fn now_ms(&self) -> u64 {
        self.ms.load(Ordering::SeqCst)
    }
}

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
    clock: Arc<dyn Clock>,
    /// Serializes the message read-modify-write (FIND-599): the engine's
    /// `insert_lock` only covers the individual write, not the read + append,
    /// so two concurrent senders read the same message list and the second
    /// insert drops the first message. Shared across the stores of a database
    /// via `Arc` (same pattern as the SDK's `merge_lock`).
    message_lock: Arc<Mutex<()>>,
    /// Serializes the thread-index read-modify-write (FIND-600): the index is
    /// a single JSON blob, so two concurrent creators read the same id list
    /// and the second save drops the first thread id.
    index_lock: Arc<Mutex<()>>,
}

impl<'a> ThreadStore<'a> {
    /// Wrap a storage engine reference (wall-time clock).
    ///
    /// The store owns its locks, so a **single** store serializes its own
    /// read-modify-writes. Stores that share a database must share the locks
    /// too — the SDK does it with the crate-internal `with_shared_locks`.
    pub fn new(engine: &'a StorageEngine) -> Self {
        Self::with_shared_locks(engine, Arc::new(Mutex::new(())), Arc::new(Mutex::new(())))
    }

    /// Wrap a storage engine reference with an explicit [`Clock`].
    ///
    /// Tests pass `Arc::new(ManualClock::new(..))` to expire TTLs by
    /// advancing virtual time instead of sleeping.
    pub fn with_clock(engine: &'a StorageEngine, clock: Arc<dyn Clock>) -> Self {
        Self {
            engine,
            clock,
            message_lock: Arc::new(Mutex::new(())),
            index_lock: Arc::new(Mutex::new(())),
        }
    }

    /// Wrap a storage engine reference with locks shared with the other
    /// stores over the same database (wall-time clock).
    ///
    /// The SDK builds one store per call, so a store's own locks would not
    /// serialize anything: the handle shares its locks with every store it
    /// builds (FIND-599/FIND-600).
    pub(crate) fn with_shared_locks(
        engine: &'a StorageEngine,
        message_lock: Arc<Mutex<()>>,
        index_lock: Arc<Mutex<()>>,
    ) -> Self {
        Self {
            engine,
            clock: Arc::new(SystemClock),
            message_lock,
            index_lock,
        }
    }

    /// Create a new thread.
    ///
    /// `input.ttl_secs` — if set, the thread auto-expires after this many seconds.
    /// The thread's messages are deleted on sweep.
    ///
    /// `gc` — if provided, the TTL expiry is registered with the garbage
    /// collector so it can be cleaned up automatically.
    pub fn create(&self, input: CreateThread<'_>, gc: Option<&mut GcWorker<'a>>) -> Result<u128> {
        let CreateThread {
            title,
            metadata,
            ttl_secs,
        } = input;
        let thread_id = generate_id();
        let now = self.clock.now_secs();

        let empty_messages = serde_json::to_string(&Vec::<Message>::new())
            .map_err(|e| Error::serialization(ChainedError::with_source("messages", e)))?;
        let metadata_json = serde_json::to_string(&metadata)
            .map_err(|e| Error::serialization(ChainedError::with_source("metadata", e)))?;

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
        // FIND-599: serialize the whole read-modify-write. The engine's
        // `insert_lock` only covers the individual write, so without this
        // guard two senders read the same list and the second insert drops
        // the first message.
        let _guard = self.message_lock.lock();
        let now = self.clock.now_secs();
        let mut node = self
            .engine
            .get(thread_id)?
            .ok_or(Error::NodeNotFound(thread_id))?;

        let mut messages: Vec<Message> = self.load_messages(&node)?;
        messages.push(Message {
            role: role.to_string(),
            content: content.to_string(),
            timestamp: self.clock.now_ms(),
            metadata,
        });

        let messages_json = serde_json::to_string(&messages)
            .map_err(|e| Error::serialization(ChainedError::with_source("messages", e)))?;
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
    pub fn get(&self, thread_id: u128) -> Result<Option<MessageThread>> {
        match self.engine.get(thread_id)? {
            Some(node) => Ok(Some(self.node_to_thread(node)?)),
            None => Ok(None),
        }
    }

    /// List threads with pagination.
    pub fn list(&self, limit: usize, offset: usize) -> Result<Vec<MessageThread>> {
        let ids = self.load_thread_ids()?;
        let chunk: Vec<u128> = ids.into_iter().skip(offset).take(limit).collect();

        let mut threads = Vec::with_capacity(chunk.len());
        for id in chunk {
            if let Some(thread) = self.get(id)? {
                threads.push(thread);
            }
        }
        Ok(threads)
    }

    /// Delete a thread by its ID.
    pub fn delete(&self, thread_id: u128) -> Result<()> {
        self.engine.delete(thread_id, "delete")?;
        self.remove_thread_id(thread_id)
    }

    /// Remove all threads whose TTL has expired.
    ///
    /// Returns the number of threads purged.
    pub fn purge_expired_threads(&self) -> Result<usize> {
        // FIND-600: the rebuild rewrites the whole index, so the lock is held
        // from the load to the save (the helpers below do not lock again).
        let _guard = self.index_lock.lock();
        let now = self.clock.now_secs();
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
                        .is_some_and(|node| !self.is_expired(&node, now))
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
                .map_err(|e| Error::serialization(ChainedError::with_source("messages", e))),
            _ => Ok(Vec::new()),
        }
    }

    fn load_metadata(&self, node: &UnifiedNode) -> Result<HashMap<String, String>> {
        match node.get_field(FIELD_METADATA) {
            Some(FieldValue::String(json)) => serde_json::from_str(json)
                .map_err(|e| Error::serialization(ChainedError::with_source("metadata", e))),
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
            Some(bytes) => serde_json::from_slice(&bytes)
                .map_err(|e| Error::serialization(ChainedError::with_source("thread index", e))),
            None => Ok(Vec::new()),
        }
    }

    fn save_thread_ids(&self, ids: &[u128]) -> Result<()> {
        let bytes = serde_json::to_vec(ids)
            .map_err(|e| Error::serialization(ChainedError::with_source("thread index", e)))?;
        self.engine
            .put_to_partition(BackendPartition::InternalMetadata, THREAD_INDEX_KEY, &bytes)
    }

    fn add_thread_id(&self, id: u128) -> Result<()> {
        // FIND-600: the index is one JSON blob, so load -> push -> save has to
        // be atomic or the second save drops the first id.
        let _guard = self.index_lock.lock();
        let mut ids = self.load_thread_ids()?;
        if !ids.contains(&id) {
            ids.push(id);
            self.save_thread_ids(&ids)?;
        }
        Ok(())
    }

    fn remove_thread_id(&self, id: u128) -> Result<()> {
        let _guard = self.index_lock.lock();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::BackendKind;
    use crate::config::Config;
    use crate::sdk::Embedded;
    use std::collections::HashSet;
    use std::sync::Barrier;

    /// In-memory [`Embedded`] handle — the same handle the MCP server and the
    /// SDK hand out (`Clone + Send + Sync`, requests served in parallel).
    fn embedded() -> Embedded {
        Embedded::open_with_config(Config {
            storage_path: ":memory:".into(),
            backend_kind: BackendKind::InMemory,
            ..Default::default()
        })
        .expect("open embedded")
    }

    /// FIND-599: two concurrent `send_message` calls on the same thread must
    /// leave **both** messages. The message list is a read-modify-write
    /// (`get -> load_messages -> push -> insert`) and the engine's
    /// `insert_lock` only covers the individual write, so without a lock
    /// both writers read the same list and the second insert drops the first
    /// message — silently, with no error.
    #[test]
    fn concurrent_send_message_keeps_both_messages() {
        const MESSAGES: usize = 20;
        let db = embedded();
        let thread_id = db.create_thread("concurrent", None).expect("create thread");

        let barrier = Arc::new(Barrier::new(2));
        let handles: Vec<_> = (0..2)
            .map(|worker| {
                let db = db.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    for i in 0..MESSAGES {
                        // Align both writers right before the read-modify-write.
                        barrier.wait();
                        // Padding widens the read-modify-write window so the
                        // race is observable instead of a coin flip.
                        let content = format!("w{worker}-m{i}-{}", "x".repeat(512));
                        db.send_message(thread_id, "user", &content)
                            .expect("send message");
                    }
                })
            })
            .collect();
        for handle in handles {
            handle.join().expect("join writer");
        }

        let thread = db
            .get_thread(thread_id)
            .expect("get thread")
            .expect("thread exists");
        assert_eq!(
            thread.messages.len(),
            MESSAGES * 2,
            "both writers' messages must survive (got {} of {})",
            thread.messages.len(),
            MESSAGES * 2
        );
    }

    /// FIND-600: two concurrent `create_thread` calls must leave **both**
    /// threads in the index. The index is a single JSON blob in
    /// `InternalMetadata`, so its read-modify-write (`load -> push -> save`)
    /// has to be serialized: otherwise the second save overwrites the first
    /// id and the thread becomes invisible in `list_threads` (and immortal:
    /// the purge only walks the index).
    #[test]
    fn concurrent_create_thread_keeps_both_threads_in_the_index() {
        const THREADS: usize = 20;
        let db = embedded();
        // Pre-populate the index: the read-modify-write window grows with the
        // index size, so the race shows up reliably.
        for i in 0..50 {
            db.create_thread(&format!("pre-{i}"), None)
                .expect("create pre thread");
        }

        let barrier = Arc::new(Barrier::new(2));
        let handles: Vec<_> = (0..2)
            .map(|worker| {
                let db = db.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    let mut ids = Vec::new();
                    for i in 0..THREADS {
                        barrier.wait();
                        ids.push(
                            db.create_thread(&format!("w{worker}-{i}"), None)
                                .expect("create thread"),
                        );
                    }
                    ids
                })
            })
            .collect();
        let mut created = Vec::new();
        for handle in handles {
            created.extend(handle.join().expect("join creator"));
        }

        let listed: HashSet<u128> = db
            .list_threads(1000, 0)
            .expect("list threads")
            .iter()
            .map(|t| t.thread_id)
            .collect();
        for id in &created {
            assert!(
                listed.contains(id),
                "thread {id} vanished from the index (index holds {} of {} created ids)",
                listed.len(),
                created.len()
            );
        }
    }
}
