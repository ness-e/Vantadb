---
title: "ACID Transaction System for VantaDB"
status: draft
tags: [vantadb, research, database, transactions]
last_reviewed: 2026-07-03
aliases: []
---

# ACID Transaction System for VantaDB

## GOAL

Define what ACID means for VantaDB's specific architecture — an embedded, multi-threaded persistent memory and vector retrieval engine.

| Property | VantaDB-specific meaning |
|---|---|
| **Atomicity** | A batch of operations (insert node, update vector index, write metadata to KV) either commits entirely or is rolled back on crash. No partial node writes. |
| **Consistency** | Invariants post-transaction: (a) every node in HNSW has a corresponding entry in the KV backend; (b) edges never reference non-existent node IDs; (c) WAL checkpoint_seq ≤ total WAL records after recovery. |
| **Isolation** | Concurrent readers see a snapshot before the transaction starts (snapshot isolation). Concurrent writers are serialized — no interleaved partial updates across the three storage layers (KV, VantaFile, HNSW). |
| **Durability** | After `commit()` returns, data survives crash+replay. The WAL provides the durability contract; the KV backend (Fjall/RocksDB) provides the secondary durability layer. |

VantaDB does **not** need SQL-style multi-statement user transactions (BEGIN/COMMIT across ad-hoc queries). The primary use case is **single-batch atomicity**: a Python/TypeScript SDK call that inserts 5 nodes with edges and vectors must land as an atomic unit. The secondary use case is **multi-key compare-and-swap** for consistency in concurrent write scenarios.

---

## Constraints & Current Architecture

### Storage Layers (written in order during a mutation)

```
User mutation
  → WAL (vanta.wal, append-only, CRC32C)
  → VantaFile (vector_store.vanta, mmap'd, write_node_to_vstore)
  → KV Backend (Fjall/RocksDB, partition: "default" for metadata)
  → HNSW (vector_index.bin, CPIndex::add)
```

Each layer uses independent locking:

| Resource | Lock type |
|---|---|
| `HashMap<u64, UnifiedNode>` (volatile cache) | `RwLock` |
| `WalWriter` | `Arc<Mutex<Option<WalWriter>>>` |
| `VantaFile` (vector store) | `RwLock` |
| `CPIndex` (HNSW) | `ArcSwap` + insert `Mutex` |
| `dyn StorageBackend` (KV) | Backend-native (Fjall uses internal LSM locks) |

### Key Constraints

1. **No coordination between layers.** `StorageEngine::insert` writes to the WAL first, then VantaFile, then KV backend, then HNSW — but each step can fail independently, and there is no rollback mechanism for earlier steps.

2. **Fjall's batch is per-backend only.** `FjallBackend::write_batch` atomically commits across keyspaces, but does **not** cover VantaFile, HNSW, or WAL writes.

3. **VantaFile is append-oriented.** Once vector bytes are written, there is no "un-write" — only tombstone flags can be set in the header. A rolled-back transaction would leave a dead record.

4. **WAL is append-only with checkpoint replay.** The WAL already provides crash replay. It correctly recovers Insert/Update/Delete/Checkpoint records. But there is no **transaction begin/commit/abort** record type — every write is implicitly committed.

5. **HNSW mutations are not reversible.** `CPIndex::add` is a network-level graph mutation; removing a node requires a separate tombstone pass.

6. **SDK exposes `Arc<RwLock<StorageEngine>>`.** Python and TypeScript clients share the engine through a single mutex, which serializes all SDK-level operations. This already prevents most interleaving at the client level, but does nothing for partial failures within a single multi-step operation.

---

## Approach A: Fjall's Built-in Transaction Support

Fjall v3.1.x provides:

- **`Database::batch()` / `OwnedWriteBatch`**: Atomic commit across keyspaces within the same database. This is what `FjallBackend::write_batch` already uses.
- **`Keyspace::transaction()`**: Fjall 3.1 does **not** expose a user-facing `Transaction` type with begin/commit/rollback semantics like SQL databases. The `batch()` API is the only atomicity primitive.
- **`PersistMode::SyncAll`**: Strongest fsync guarantee. Combined with `batch()`, this gives you atomicity + durability for KV operations only.
- **`Database::persist()`**: Fsyncs the journal. The journal itself provides crash safety.

### Pros

- Zero additional code for KV-layer atomicity. Already in use via `FjallBackend::write_batch`.
- Fjall's journal-based architecture provides crash consistency out of the box — even without explicit `persist()`, data is recoverable after a crash.
- No external dependency or schema changes.

### Cons

- **KV-only.** The batch does not cover VantaFile, HNSW, or WAL writes. If `write_node_to_vstore` succeeds but the batch commit fails, you have an orphan vector in the VantaFile.
- No snapshot isolation. Fjall's iterators read from the current LSM state; there is no `BEGIN TRANSACTION` + `ROLLBACK` semantics.
- No hooks for multi-layer rollback. You would need to implement compensating writes (e.g., a tombstone for the VantaFile orphan) manually.
- The `Transaction` API mentioned in older Fjall docs was removed or never stabilized in v3.1. The current API is batch-only.

### Verdict

Fjall's batch is necessary but insufficient. It can be a building block for the KV component of a cross-layer transaction, but it cannot provide end-to-end ACID.

---

## Approach B: Custom Transaction Layer on Top of Current WAL

Extend the existing WAL to support transaction records (`Begin`, `Prepare`, `Commit`, `Abort`) and augment `StorageEngine` with a `Transaction` type that coordinates writes across all layers.

### Design

The existing `WalRecord` enum gains three new variants:

```rust
pub enum WalRecord {
    // Existing
    Insert(UnifiedNode),
    Update { id: u64, node: UnifiedNode },
    Delete { id: u64 },
    Checkpoint { node_count: u64, index_checksum: Option<u32>, timestamp: Option<u64> },
    // New
    Begin { txn_id: u64, timestamp: u64 },
    Prepare { txn_id: u64 },
    Commit { txn_id: u64 },
    Abort { txn_id: u64 },
}
```

The transaction lifecycle:

1. **Begin**: WAL appends `Begin { txn_id }`. All subsequent writes accumulate in a `Vec<WalRecord>` buffer (not written to VantaFile / HNSW / KV yet).
2. **Buffered writes**: The transaction holds an in-memory write set. Reads during the transaction check this buffer first (read-your-writes).
3. **Prepare** (optional, for two-phase): WAL appends `Prepare { txn_id }` and fsyncs. This marks the transaction as committable. If the process crashes after `Prepare`, recovery knows the transaction was ready.
4. **Commit**: Apply the buffered writes to VantaFile, KV, and HNSW **in order**. If any step fails, the whole transaction is rolled back by skipping it during WAL replay. Append `Commit { txn_id }` to WAL and fsync.
5. **Abort**: Discard the buffer. Append `Abort { txn_id }` to WAL.

### Recovery Semantics

During `WalReader::next_record` replay:

- If a `Begin` is seen without a matching `Commit` or `Abort`, the transaction is **rolled back** — its buffered writes are skipped.
- If a `Begin` + `Prepare` is seen without `Commit`, the transaction is **rolled forward**: the prepare marker confirms the data was durable, so the buffered writes are replayed.
- If a `Commit` is seen, the writes are already applied; replay proceeds normally.
- If an `Abort` is seen, the writes are skipped.

This gives you **atomicity** (all-or-nothing replay) and **durability** (fsync before commit).

### Concurrency Model

```rust
pub struct Transaction {
    engine: Arc<StorageEngine>,
    txn_id: u64,
    write_buffer: Vec<(WalRecord, PendingWrite)>,
    snapshot: SnapshotState, // frozen copy of relevant engine state
    state: AtomicU8,         // Active | Preparing | Committed | Aborted
}
```

- The transaction acquires a **write lease** from the engine (analogous to a `RwLock::write` but limited to one transaction at a time).
- Readers acquire a **snapshot** — a frozen copy of the volatile cache state — which provides snapshot isolation.
- Multiple concurrent transactions are **not** supported initially. Serialized transaction execution avoids deadlock, write skew, and lost update problems entirely. This is acceptable for an embedded engine where the SDK already serializes via `Arc<RwLock<StorageEngine>>`.

### Write-ahead vs Write-behind

The current `StorageEngine::insert` is write-ahead (WAL first, then stores). The transaction layer extends this: WAL gets the Begin/Commit markers, stores are updated only on commit. This means:

- VantaFile writes happen at commit time, not during buffering.
- HNSW `add()` happens at commit time — but HNSW is not WAL-backed (it's rebuilt from VantaFile on restart). This is fine because the WAL replay will rebuild HNSW anyway.
- KV backend writes happen at commit time, using `write_batch` for atomicity.

### Pros

- Full ACID across all storage layers (WAL, VantaFile, KV, HNSW).
- Reuses the proven WAL format, CRC32C integrity checks, and scan-forward recovery.
- No new crates or external dependencies.
- Clear semantics for crash recovery (rollback unprepared, roll forward prepared).
- Can extend to multi-node operations in the future.

### Cons

- Significant implementation surface: `Transaction` struct, WAL record versioning, replay logic changes, snapshot mechanics.
- VantaFile compaction must account for "phantom" records written by aborted transactions (addressed by tombstones or skip-lists during BFS compaction).
- Performance overhead: all writes must be buffered until commit, increasing memory pressure.
- Concurrency is reduced to serialized transaction execution (acceptable for embedded use).

---

## Approach C: SQLite-style Rollback Journal vs. WAL-mode Transactions

SQLite offers two transaction durability modes that are worth studying as inspiration:

### Rollback Journal Mode

Before modifying a page, SQLite copies the original page to a `-journal` file. On crash, the journal is replayed to undo partial writes. On success, the journal is deleted.

**For VantaDB:**

| Component | Analogy |
|---|---|
| `-journal` file | A new `vanta.rollback` file that stores pre-mutation copies of VantaFile pages |
| Recovery | On crash, check for `vanta.rollback`; if present, restore pages to their pre-transaction state |
| Cleanup | Delete `vanta.rollback` on successful commit |

**Problems:**
- VantaFile is mmap'd. Rolling back mmap'd pages requires `mprotect(PROT_WRITE)` + `memcpy` to overwrite the corrupted region — a risky operation if the process crashes mid-rollback.
- Does not integrate with the existing WAL — you would maintain **two** durability mechanisms, increasing complexity.
- No benefit for the KV backend (Fjall already provides crash consistency independently).

### WAL-mode Transactions (SQLite's own WAL)

SQLite's WAL mode appends changes to a WAL file. Readers read from the original database + WAL. A checkpoint merges WAL pages back into the main database.

**For VantaDB:**

| Component | Analogy |
|---|---|
| SQLite WAL | VantaDB's existing `vanta.wal` (already append-only) |
| Checkpoint | VantaDB's existing `compact_wal()` + checkpoint_seq |
| Readers | VantaDB reads from VantaFile + KV backend, not from the WAL |

**Observations:**
- VantaDB's current architecture is already WAL-mode-like: writes go to WAL first, then to the main stores. The WAL is the source of truth during crash recovery.
- SQLite's WAL adds a **shared memory** index for readers to locate pages in the WAL without blocking. VantaDB does not need this because reads go through the KV/HNSW, not through the WAL.
- SQLite's WAL provides **concurrent readers + one writer** (no readers block writers). VantaDB already has this via `RwLock<StorageEngine>` — multiple read guards can coexist.

### Verdict

Approach C is not a separate implementation path. It is a **validation** that VantaDB's existing WAL architecture (append-only, crash-recoverable) is already closer to SQLite's WAL mode than to the rollback journal. The gap is not the WAL format — it's the lack of transaction markers and cross-layer coordination. **Approach B builds on the same foundations as SQLite's WAL mode**, which is the right design.

---

## Recommended Approach

**Adopt Approach B (Custom Transaction Layer)** as the primary strategy, with the following phased delivery:

### Phase 1 — WAL Transaction Records

Add `Begin`, `Commit`, and `Abort` variants to `WalRecord`. Implement serialized `begin()` / `commit()` / `rollback()` on `StorageEngine` that write these markers to the WAL. No buffering yet — every write goes directly to the stores as it does today, but the WAL now tracks transaction boundaries. Recovery logic skips writes between unmatched `Begin`/`Abort` pairs.

**Cost:** ~2 weeks  
**Gains:** Crash atomicity for multi-write batches (the SDK can wrap N inserts in begin/commit).

### Phase 2 — Buffered Write Transactions

Introduce the `Transaction` struct with an in-memory write buffer. On `commit()`, flush to VantaFile, KV (via `write_batch`), and HNSW. On `rollback()`, discard the buffer. WAL recovery is updated to replay only committed transactions.

**Cost:** ~4 weeks  
**Gains:** True atomicity across all storage layers. No partial writes.

### Phase 3 — Snapshot Isolation (Optional)

Provide `Transaction::snapshot()` that freezes the volatile cache state at transaction start. Readers spawned from the snapshot see a consistent view even if other transactions commit concurrently.

**Cost:** ~2 weeks  
**Gains:** Repeatable reads for analytics-style queries.

### Why Not Fjall-only (A) or Journal-only (C)?

- **A** is insufficient — it does not cover the vector store or HNSW.
- **C** is redundant — VantaDB already has a better foundation in its existing WAL.
- **B** is the highest leverage: it extends a system that already works (the WAL) with minimal new machinery, covers all storage layers, and preserves the existing recovery correctness (CRC32C, scan-forward, checkpoint replay).

---

## Implementation Stub

Below is the proposed Rust API surface for the transaction system. No real implementation — only signatures and doc comments to establish the contract.

### WAL Transaction Records

```rust
// In src/wal.rs — new variants on WalRecord

/// Extends WalRecord with transaction markers.
impl WalRecord {
    /// Unique transaction ID for correlating Begin/Commit/Abort records.
    pub type TxnId = u64;

    /// Begin a transaction with the given ID and wall-clock timestamp.
    pub fn begin(txn_id: TxnId) -> Self;

    /// Mark a transaction as committed.
    pub fn commit(txn_id: TxnId) -> Self;

    /// Mark a transaction as aborted (rolled back).
    pub fn abort(txn_id: TxnId) -> Self;
}
```

### Transaction State Machine

```rust
/// Lifecycle of a transaction within VantaDB.
///
/// ```text
/// ┌──────┐  begin()  ┌────────┐  commit()  ┌──────────┐
/// │ Idle │ ────────→ │ Active │ ─────────→ │ Committed │
/// └──────┘           └────────┘            └──────────┘
///                       │  rollback()
///                       ▼
///                    ┌────────┐
///                    │ Aborted │
///                    └────────┘
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxnStatus {
    /// Transaction is in progress (buffering writes).
    Active,
    /// Transaction has been committed and all writes applied.
    Committed,
    /// Transaction has been rolled back; no writes applied.
    Aborted,
}
```

### Transaction Handle

```rust
/// An ACID transaction over VantaDB's storage layers.
///
/// The transaction buffers all mutations (insert, update, delete, relate)
/// in memory. On `commit()` the buffer is flushed in order:
///   1. WAL (Begin marker already written; Commit marker written)
///   2. VantaFile (vector data)
///   3. KV backend (metadata + edges, via `write_batch`)
///   4. HNSW index (vector index inserts)
///
/// If any step fails, the transaction is rolled back:
///   - WAL receives an Abort marker.
///   - VantaFile allocations are tombstoned via GC.
///   - HNSW is unaffected (the failed add is simply retried on next rebuild).
///
/// On crash recovery, any Active transaction without a Commit marker
/// is silently rolled back during WAL replay.
pub struct Transaction {
    engine: Arc<StorageEngine>,
    txn_id: u64,
    status: TxnStatus,
    /// Buffered write set: all mutations accumulated during this transaction.
    writes: Vec<(WalRecord, PendingStorageWrite)>,
    /// Enforces serialized transaction execution.
    _lease: EngineWriteLease,
}

impl Transaction {
    /// Begin a new transaction.
    ///
    /// Acquires a write lease from the engine, ensuring only one
    /// transaction is active at a time. Appends `Begin { txn_id }` to the WAL.
    pub fn begin(engine: Arc<StorageEngine>) -> Result<Self>;

    /// Insert a node within this transaction.
    ///
    /// The node is buffered in memory and becomes visible to reads
    /// within this transaction (read-your-writes). Not flushed to storage
    /// until `commit()`.
    pub fn insert(&mut self, node: UnifiedNode) -> Result<()>;

    /// Update a node within this transaction.
    pub fn update(&mut self, id: u64, node: UnifiedNode) -> Result<()>;

    /// Delete a node within this transaction.
    pub fn delete(&mut self, id: u64) -> Result<()>;

    /// Relate two nodes within this transaction.
    pub fn relate(&mut self, from: u64, to: u64, label: &str) -> Result<()>;

    /// Read a node within this transaction.
    ///
    /// Checks the transaction's write buffer first (read-your-writes),
    /// then falls back to the engine's persistent state.
    pub fn get(&self, id: u64) -> Result<Option<UnifiedNode>>;

    /// Commit the transaction.
    ///
    /// 1. Fsync the WAL (Commit marker).
    /// 2. Write buffered vector data to VantaFile.
    /// 3. Write buffered metadata to KV backend (via `write_batch`).
    /// 4. Update HNSW index.
    /// 5. Release the write lease.
    ///
    /// Returns `TxnStatus::Committed` on success.
    pub fn commit(mut self) -> Result<TxnStatus>;

    /// Rollback the transaction.
    ///
    /// Appends `Abort { txn_id }` to the WAL and discards the buffer.
    /// No storage layers are modified. The write lease is released.
    pub fn rollback(mut self) -> Result<TxnStatus>;
}
```

### Engine Integration

```rust
// On StorageEngine (src/storage/engine.rs)

impl StorageEngine {
    /// Begin a serialized transaction.
    ///
    /// Acquires the engine's write lease. While the lease is held,
    /// direct `insert()`/`update()`/`delete()` calls on the engine
    /// will block or return a `TxnInProgress` error.
    pub fn begin_txn(self: Arc<Self>) -> Result<Transaction>;

    /// Snapshot the current engine state for consistent reads.
    ///
    /// Returns a frozen view of the volatile cache and HNSW index
    /// at the time of the call. The snapshot lives until dropped.
    pub fn snapshot(&self) -> EngineSnapshot;
}
```

### Recovery Changes

```rust
/// Extended replay logic that respects transaction boundaries.
///
/// Recovery semantics:
/// - Records between an `UnmatchedBegin` and the next `Commit` are skipped.
/// - Records between a `Begin` and its matching `Commit` are replayed.
/// - Records outside any transaction scope (legacy WAL without Begin markers)
///   are replayed as before — backward compatible.
impl WalReader {
    /// Replay all records, filtering out aborted transactions.
    ///
    /// Returns the number of records that were actually applied.
    pub fn replay_transactional<F>(&mut self, handler: F) -> Result<u64>
    where
        F: FnMut(WalRecord) -> Result<()>;
}
```

### Error Handling

```rust
/// Error returned when a mutation is attempted during an active
/// transaction without going through the Transaction handle.
pub enum TxnError {
    TxnInProgress { current_txn_id: u64 },
    TxnNotFound { txn_id: u64 },
    WriteConflict { key: Vec<u8> },
    CommitFailed { txn_id: u64, step: &'static str },
}
```

### Compatibility

- WAL files written without transaction markers remain fully readable. `Begin`/`Commit`/`Abort` are optional variants that the existing `match` in `replay_all` handles as no-ops.
- Existing SDK clients that call `engine.insert()` directly still work — they operate on the engine's implicit auto-commit path (no transaction).
- The transaction is optional: users who do not need atomic batches continue with the direct API.

---

## Summary

| Criterion | Approach A (Fjall) | Approach B (Custom WAL) | Approach C (Journal) |
|---|---|---|---|
| Atomicity across all layers | ❌ KV only | ✅ Full | ❌ VantaFile only |
| Isolation | ❌ None | ✅ Snapshot (Phase 3) | ❌ None |
| Durability | ✅ (Fjall journal) | ✅ (WAL fsync) | ✅ (journal fsync) |
| Implementation cost | ~0 (already done) | ~6-8 weeks total | ~4-6 weeks |
| Risk of regression | Low | Medium (WAL replay changes) | High (mmap rollback) |
| Backward compatibility | ✅ | ✅ (new WAL variants) | ❌ (new file format) |

**Recommendation: Approach B, delivered in three phases.** The existing WAL provides 80% of the infrastructure needed. Adding transaction markers and a buffered write path gives VantaDB proper ACID semantics without new dependencies, risky mmap manipulation, or changes to the storage backend interface.
