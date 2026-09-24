# MVCC Snapshot Isolation Design

## Context

ACID Phase 3 builds on Phase 1 (WAL records: Begin/Commit/Abort) and Phase 2
(buffered write transactions with single-txn mutex). The gap: no concurrent
transaction support, no snapshot isolation, no MVCC versioning.

**Problem:** Readers block on writers? No — readers read committed state and
txn buffers provide read-your-writes. But there's no isolation: a long read
sees partial writes from in-flight transactions, violating the Isolation
property of ACID.

**Solution:** Lightweight MVCC with per-record version stamps (`created_by_txn`,
`deleted_by_txn`) on `NodeMetadata`, plus a `Snapshot` struct that filters
reads by visibility rules.

## Design

### 1. Versioned NodeMetadata

```rust
pub(crate) struct NodeMetadata {
    pub relational: RelFields,
    pub edges: Vec<Edge>,
    pub created_by_txn: u64,        // txn that created this version
    pub deleted_by_txn: Option<u64>, // txn that deleted this version
}
```

- `created_by_txn` = txn_id that inserted/updated this node
- `deleted_by_txn` = txn_id that deleted this node (None = alive)
- Both are set when the txn commits and applies its buffer to stores

### 2. Snapshot Struct

```rust
pub struct Snapshot {
    pub txn_id: u64,       // snapshot taken at this txn_id
}
```

- Created by `begin_snapshot()` — captures current `next_txn_id`
- Read-only — no buffer, no WAL writes
- Passed to `get_with_snapshot()` for isolated reads

### 3. Visibility Rule

A version is **visible** to snapshot `S` if:

```
created_by_txn <= S.txn_id
AND (deleted_by_txn IS NULL OR deleted_by_txn > S.txn_id)
```

### 4. Concurrent Transaction Tracking

Replace `active_txn_id: Mutex<Option<u64>>` with:

```rust
active_txns: Mutex<HashSet<u64>>,
```

Multiple transactions can be active simultaneously:
- `begin_transaction()` → register in `active_txns`, return txn_id
- `commit_transaction()` → apply buffer, unregister
- `abort_transaction()` → drop buffer, unregister

### 5. Write-Write Conflict Detection

**First-writer-wins:** before buffering an insert/delete, check if any *other*
active transaction has already buffered an operation on the same node ID.

```rust
fn check_write_conflict(&self, id: u128, my_txn_id: u64) -> Result<()> {
    let buffers = self.txn_buffers.lock();
    for (&txn_id, ops) in buffers.iter() {
        if txn_id == my_txn_id { continue; }
        for op in ops {
            match op {
                BufferedWrite::Insert(node) if node.id == id => return conflict,
                BufferedWrite::Delete(del_id) if *del_id == id => return conflict,
                _ => {}
            }
        }
    }
    Ok(())
}
```

### 6. Read Path

```
get(id) → check txn buffer (read-your-writes) → check committed store
get_with_snapshot(id, snapshot) → txn buffer → commit store filtered by visibility
```

- Plain `get()` (no snapshot): all committed data (existing behavior, unchanged)
- `get_with_snapshot()`: only data committed before snapshot's txn_id

### 7. Garbage Collection

Simple periodic GC: when a txn commits, scan for metadata entries where
`deleted_by_txn` is set and no active snapshot could reach them.

For Phase 3, GC is eager-on-commit: when committing a delete, immediately
remove any version that is no longer visible to any possible snapshot.

### 8. Snapshot Isolation Tests

| Test | What it proves |
|------|---------------|
| `test_snapshot_sees_consistent_state` | Long-lived snapshot doesn't see uncommitted writes from concurrent txn |
| `test_write_write_conflict` | Two concurrent txns can't modify the same node |
| `test_concurrent_txn_isolation` | Concurrent txns don't interfere |
| `test_snapshot_does_not_block_writer` | Read snapshot doesn't prevent writes (no locks) |

## Files Changed

| File | Change |
|------|--------|
| `src/storage/ops.rs` | `NodeMetadata` gains `created_by_txn`, `deleted_by_txn` |
| `src/storage/engine/mod.rs` | `StorageEngine` gains `snapshot`, `active_txns` replaces `active_txn_id` |
| `src/storage/engine/ops.rs` | New `begin_snapshot()`, `get_with_snapshot()`, write-write conflict check, concurrent txn support |
| `src/storage/engine/tests/ops.rs` | Tests for snapshot isolation, write-write conflict, concurrent txns |

## Not Changed (deferred/P1 compatible)

- **WAL format:** No new record types needed — existing Begin/Commit/Abort + Insert/Delete suffice
- **VantaFile/vector store:** No versioning added to disk format — deferred to Phase 4
- **HNSW:** No MVCC — HNSW entries are added/removed at commit time atomically
- **GC:** Eager-on-commit for Phase 3; background thread deferred
- **Backend persistence:** `NodeMetadata` serialization handles new fields transparently
