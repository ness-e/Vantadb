//! Agentic message thread storage with optional TTL.
//!
//! [`ThreadStore`] wraps a [`StorageEngine`] to persist conversation threads
//! as [`UnifiedNode`]s with JSON-serialized messages.
//!
//! # TTL: a window of inactivity, evaluated on demand (FIND-800)
//!
//! A thread created with a TTL carries its deadline **on the node**
//! (`_expires_at`, seconds) plus the window itself (`_ttl_secs`, used to
//! re-arm). No timer runs: the deadline is evaluated by
//! [`ThreadStore::purge_expired_threads`], reachable as
//! `Embedded::purge_expired_threads` and as the MCP tool
//! `thread_purge_expired`. Until that call, an expired thread stays readable.
//!
//! [`GcWorker`] is an optional, caller-owned accelerator that **no production
//! path supplies** — see its module docs for why it is not wired up.
//!
//! The window is inactivity-based: [`ThreadStore::send_message`] re-arms
//! `_expires_at` to `now + _ttl_secs`, so a thread with ongoing conversation
//! never expires and a silent one does.

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
/// serialized ID list in the `InternalMetadata` partition. Expiry is evaluated
/// only by [`Self::purge_expired_threads`]: the optional `gc` parameter of
/// `create`/`send_message` is a caller-owned accelerator, not the mechanism
/// (FIND-800).
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
    /// FIND-661: the store shares the **engine-owned** locks
    /// (`sdk_thread_message_lock`/`sdk_thread_index_lock`), so every store
    /// and every `Embedded` wrapper over this engine serializes the
    /// read-modify-writes against each other — a per-store lock let direct
    /// `ThreadStore::new` callers race SDK callers on the same data.
    pub fn new(engine: &'a StorageEngine) -> Self {
        Self::with_shared_locks(
            engine,
            engine.sdk_thread_message_lock.clone(),
            engine.sdk_thread_index_lock.clone(),
        )
    }

    /// Wrap a storage engine reference with an explicit [`Clock`].
    ///
    /// Tests pass `Arc::new(ManualClock::new(..))` to expire TTLs by
    /// advancing virtual time instead of sleeping. Shares the engine-owned
    /// thread locks (FIND-661), same as [`Self::new`].
    pub fn with_clock(engine: &'a StorageEngine, clock: Arc<dyn Clock>) -> Self {
        Self {
            engine,
            clock,
            message_lock: engine.sdk_thread_message_lock.clone(),
            index_lock: engine.sdk_thread_index_lock.clone(),
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
    /// `input.ttl_secs` — if set, the thread gets a TTL window of this many
    /// seconds, counted from now and **re-armed by every**
    /// [`Self::send_message`]: it expires after that much *inactivity*, not at
    /// a fixed absolute deadline. Nothing deletes it on a timer — the deadline
    /// is persisted as `_expires_at` and evaluated by
    /// [`Self::purge_expired_threads`], so an expired thread stays readable
    /// until that call (FIND-800).
    ///
    /// `ttl_secs = Some(0)` is not a "no TTL": it stamps `_expires_at = now`, so
    /// the thread expires at the next second **and cannot be rescued** — the
    /// re-arm in `send_message` requires `_ttl_secs > 0`. Use `None` for a
    /// thread that never expires.
    ///
    /// `gc` — an optional, caller-owned [`GcWorker`]: registering the expiry
    /// there lets that caller sweep the node earlier. It is not the mechanism,
    /// and `Embedded` never supplies one (see the module docs).
    pub fn create(&self, input: CreateThread<'_>, gc: Option<&mut GcWorker<'a>>) -> Result<u128> {
        // NOT-02 / FIND-600: serialize the two writes under one lock so
        // concurrent callers cannot interleave their index read-modify-writes.
        // (The pair is not atomic — see the ordering note before the writes.)
        let _guard = self.index_lock.lock();
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

        let mut ttl_expires_at = None;
        if let Some(ttl) = ttl_secs {
            let expires_at = now + ttl;
            node.set_field(FIELD_EXPIRES_AT, FieldValue::Int(expires_at as i64));
            node.set_field(FIELD_TTL_SECS, FieldValue::Int(ttl as i64));
            ttl_expires_at = Some(expires_at);
        }

        // FIND-798: write the index **before** the node.
        //
        // With the old order (node, then index) an index failure stranded a
        // node that no reader could reach: `list` and `purge_expired_threads`
        // both walk the index, not the backend, so the orphan was invisible
        // *and* immortal, and the error invited a retry that minted a fresh id
        // and inserted a second node.
        //
        // Inverted, a failure leaves at worst an index id with no node. That is
        // the cheap direction: `list` does not render it (it resolves before it
        // pages), the id costs one wasted slot in the index blob, and nothing
        // duplicates on retry. It is *not* self-recycling — `purge_expired_threads`
        // only rebuilds the index `if purged > 0`, and a dangling id never
        // increments `purged` — which is exactly why the compensation below
        // exists.
        self.add_thread_id_locked(thread_id)?;
        if let Err(e) = self.engine.insert(&node) {
            // Compensate so a failed create leaves no trace at all: drop the id
            // we just wrote. Best-effort by design — `save_thread_ids` takes the
            // same `insert_lock` the insert may have just timed out on, so under
            // sustained contention the cleanup can fail for the same reason. In
            // that case we keep the dangling id (cheap, see above) and still
            // return the *original* error, which is the one the caller needs.
            //
            // ponytail: the cleanup is logged, not retried, and has no
            // retry loop. Upgrade: fold index+node into one backend batch if a
            // failed create ever needs to be atomic down to the index.
            if let Err(cleanup) = self.remove_thread_id_locked(thread_id) {
                tracing::warn!(
                    thread_id,
                    error = %cleanup,
                    "create: insert failed and the index entry could not be rolled back; \
                     the thread id stays in the index with no node"
                );
            }
            return Err(e);
        }

        // Register the TTL only now that the thread exists. Registering it before
        // the writes left the GC holding an expiry — and a WAL `Delete` on the
        // next sweep — for an id that never got a node.
        if let (Some(gc), Some(expires_at)) = (gc, ttl_expires_at) {
            gc.register_ttl(thread_id, expires_at);
        }

        Ok(thread_id)
    }

    /// Append a message to an existing thread.
    ///
    /// **This re-arms the thread's TTL (FIND-800).** A message moves
    /// `_expires_at` to `now + _ttl_secs`, so the TTL is a window of
    /// *inactivity*: a thread with one message per hour and a one-hour TTL
    /// never expires, while the same thread goes quiet for an hour does. That
    /// is the intended semantics, not a leak — an absolute deadline would need
    /// its own field.
    ///
    /// `gc` — optional, caller-owned [`GcWorker`], kept in sync with the new
    /// deadline. `Embedded` never supplies one (see the module docs).
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
    ///
    /// Paging is applied to the threads that **exist**, not to the raw index:
    /// an index id whose node is gone (a failed `create` whose compensation also
    /// failed, or a TTL sweep that deleted the node without touching the index —
    /// `GcWorker::sweep` calls `StorageEngine::delete` directly) used to consume
    /// a page slot. Enough of them at the front of the index made a page come
    /// back empty while real threads sat further down, and the HTTP layer derives
    /// `has_more` from the returned count (`src/server/pagination.rs:38`), so it
    /// reported `has_more: false` with no cursor: the data became unreachable
    /// through the list (FIND-798).
    pub fn list(&self, limit: usize, offset: usize) -> Result<Vec<MessageThread>> {
        let ids = self.load_thread_ids()?;
        let mut threads = Vec::with_capacity(limit.min(ids.len()));
        let mut skipped = 0usize;
        for id in ids {
            if threads.len() >= limit {
                break;
            }
            let Some(thread) = self.get(id)? else {
                continue; // dangling id: not a position in the page
            };
            if skipped < offset {
                skipped += 1;
                continue;
            }
            threads.push(thread);
        }
        Ok(threads)
    }

    /// Delete a thread by its ID.
    pub fn delete(&self, thread_id: u128) -> Result<()> {
        let _guard = self.index_lock.lock();
        self.engine.delete(thread_id, "delete")?;
        self.remove_thread_id_locked(thread_id)
    }

    /// Remove all threads whose TTL has expired — **this is the TTL mechanism**
    /// (FIND-800).
    ///
    /// Walks the thread index, reads each node and deletes the ones whose
    /// `_expires_at` is already in the past (`< now`, strict, so the thread is
    /// still alive during the exact second of its deadline). Nothing calls this
    /// on a timer: it runs when the caller asks (`Embedded::purge_expired_threads`,
    /// MCP `thread_purge_expired`), so a deployment that never calls it never
    /// expires a thread.
    ///
    /// Cost is one `get` per indexed thread, plus — only when something was
    /// purged — a **second** pass over the survivors (another `get` per id) and
    /// a full rewrite of the index blob; the worst case is 2N reads plus one
    /// index write. Returns the number of threads purged.
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
        // NOT-02: propagate real storage errors instead of silencing them with ok().flatten().
        if purged > 0 {
            let all_ids = self.load_thread_ids()?;
            let mut remaining = Vec::with_capacity(all_ids.len());
            for id in all_ids {
                if let Some(node) = self.engine.get(id)? {
                    if !self.is_expired(&node, now) {
                        remaining.push(id);
                    }
                }
            }
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

    fn add_thread_id_locked(&self, id: u128) -> Result<()> {
        let mut ids = self.load_thread_ids()?;
        if !ids.contains(&id) {
            ids.push(id);
            self.save_thread_ids(&ids)?;
        }
        Ok(())
    }

    fn remove_thread_id_locked(&self, id: u128) -> Result<()> {
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

    /// Count nodes whose `_title` is `title`, from the **store** rather than the
    /// thread index.
    ///
    /// Deliberately not `list`: `list` walks the index, so it cannot see the
    /// orphan this module is about.
    ///
    /// Ceiling: `scan_nodes` is the HNSW-gated view (`storage/engine/ops.rs`
    /// skips a row with no index entry), not a raw partition scan. That is enough
    /// here — the orphan is a fully inserted node, so it is visible — but it is
    /// not a general "the backend is empty" oracle.
    fn nodes_titled(engine: &crate::storage::StorageEngine, title: &str) -> usize {
        engine
            .scan_nodes()
            .expect("scan nodes")
            .iter()
            .filter(|n| match n.get_field(FIELD_TITLE) {
                Some(FieldValue::String(t)) => t.as_str() == title,
                _ => false,
            })
            .count()
    }

    /// FIND-798: a `create` whose index write fails must leave **no** node
    /// behind, and the retry must leave exactly one.
    ///
    /// On `HEAD` the node is inserted first (`thread.rs:255`) and the index is
    /// updated second (`:256`), so an index failure strands a node that neither
    /// `list` nor `purge_expired_threads` can reach (both walk the index) while
    /// the error tells the user the create failed. The retry mints a fresh id
    /// (`generate_id`), so it adds a *second* node.
    ///
    /// The failure is injected through a real precondition rather than a test
    /// seam: the index is a JSON blob in `InternalMetadata` (`THREAD_INDEX_KEY`),
    /// so a corrupt blob makes `load_thread_ids` fail inside
    /// `add_thread_id_locked`. This exercises the `load` side; a `save_thread_ids`
    /// failure aborts at the same `?` in the same helper, before the node write,
    /// so it takes the same path but is not itself exercised here.
    #[test]
    fn create_failing_at_the_index_leaves_no_orphan_node() {
        let db = embedded();
        let engine = db.engine_handle().expect("engine handle");

        engine
            .put_to_partition(
                BackendPartition::InternalMetadata,
                THREAD_INDEX_KEY,
                b"{not json",
            )
            .expect("corrupt the thread index");

        assert!(
            db.create_thread("orphan", None).is_err(),
            "create must report the index failure"
        );

        let after_failure = nodes_titled(&engine, "orphan");
        assert_eq!(
            after_failure, 0,
            "a create that failed at the index must persist zero nodes (found {after_failure})"
        );

        // Repair the index and retry, exactly as a user would.
        engine
            .put_to_partition(BackendPartition::InternalMetadata, THREAD_INDEX_KEY, b"[]")
            .expect("repair the thread index");
        let retried = db.create_thread("orphan", None).expect("retry create");

        let after_retry = nodes_titled(&engine, "orphan");
        assert_eq!(
            after_retry, 1,
            "the retry must leave exactly one node (found {after_retry})"
        );
        let listed = db.list_threads(10, 0).expect("list threads");
        assert_eq!(listed.len(), 1, "exactly one thread must be listed");
        assert_eq!(listed[0].thread_id, retried);
    }

    /// FIND-798: the compensation branch — the `insert` fails *after* the index
    /// id was written, and the id must be rolled back.
    ///
    /// The failure is per-instance and deterministic: with two transactions open,
    /// `StorageEngine::insert` returns `InvalidInput` (`storage/engine/insert.rs`,
    /// the `sole_id() == None` branch) while `put_to_partition` — the index write —
    /// is transaction-unaware and still succeeds. That is precisely the ordering
    /// the compensation exists for, with no failpoint and no global state.
    #[test]
    fn create_failing_at_the_insert_rolls_back_the_index_entry() {
        let db = embedded();
        let engine = db.engine_handle().expect("engine handle");

        let txn_a = engine.begin_transaction().expect("begin txn a");
        let txn_b = engine.begin_transaction().expect("begin txn b");

        let error = db.create_thread("compensated", None);
        assert!(
            error.is_err(),
            "create must report the insert failure (two txns open)"
        );

        assert_eq!(
            nodes_titled(&engine, "compensated"),
            0,
            "a create that failed at the insert must persist zero nodes"
        );
        let ids: Vec<u128> = serde_json::from_slice(
            &engine
                .get_from_partition(BackendPartition::InternalMetadata, THREAD_INDEX_KEY)
                .expect("read index")
                .expect("index entry present"),
        )
        .expect("index is valid JSON");
        assert!(
            ids.is_empty(),
            "the compensation must drop the id written before the failed insert (found {ids:?})"
        );

        engine.abort_transaction(txn_a).expect("abort txn a");
        engine.abort_transaction(txn_b).expect("abort txn b");

        // Once the failure is gone the same create succeeds and leaves one of each.
        let id = db
            .create_thread("compensated", None)
            .expect("create thread");
        assert_eq!(nodes_titled(&engine, "compensated"), 1);
        let listed = db.list_threads(10, 0).expect("list threads");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].thread_id, id);
    }

    /// FIND-798: paging counts **threads**, not index slots.
    ///
    /// A dangling id (no node) used to consume a page slot, because `list` applied
    /// `skip`/`take` to the raw ids and filtered afterwards. Enough of them ahead
    /// of the real threads produced an empty page — and the HTTP layer derives
    /// `has_more` from the returned count, so it also stopped offering a cursor.
    #[test]
    fn list_pages_over_threads_not_over_dangling_index_ids() {
        let db = embedded();
        let engine = db.engine_handle().expect("engine handle");
        let real = db.create_thread("real", None).expect("create thread");

        // Three ids with no node, planted *ahead* of the real one.
        let ids: Vec<u128> = vec![u128::MAX, u128::MAX - 1, u128::MAX - 2, real];
        engine
            .put_to_partition(
                BackendPartition::InternalMetadata,
                THREAD_INDEX_KEY,
                &serde_json::to_vec(&ids).expect("serialize index"),
            )
            .expect("plant the index");

        let page = db.list_threads(1, 0).expect("list page");
        assert_eq!(
            page.len(),
            1,
            "a page of 1 must hold a real thread, not a dangling id"
        );
        assert_eq!(page[0].thread_id, real);

        assert_eq!(
            db.list_threads(10, 0).expect("list all").len(),
            1,
            "the dangling ids are not threads"
        );
        assert!(
            db.list_threads(10, 1).expect("list offset 1").is_empty(),
            "the dangling ids are not page positions either"
        );
    }

    /// FIND-798 guard: the happy path is untouched — one `create` yields one
    /// node in the backend and exactly one id in the index.
    #[test]
    fn create_leaves_one_node_and_one_index_entry() {
        let db = embedded();
        let engine = db.engine_handle().expect("engine handle");

        let id = db.create_thread("solitary", None).expect("create thread");

        assert_eq!(nodes_titled(&engine, "solitary"), 1, "one node");
        let listed = db.list_threads(10, 0).expect("list threads");
        assert_eq!(listed.len(), 1, "one index entry");
        assert_eq!(listed[0].thread_id, id);
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
