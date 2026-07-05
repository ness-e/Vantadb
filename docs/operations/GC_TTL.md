---
title: Garbage Collection & TTL Guide
type: operations
status: active
tags: [gc, ttl, operations]
last_reviewed: 2026-07-04
aliases: []
---

# Garbage Collection & TTL Guide

## How GC Works

VantaDB uses a TTL-based garbage collector implemented in [`src/gc.rs`](../../src/gc.rs). The `GcWorker` tracks node expiration timestamps via a `BTreeMap<u64, Vec<u64>>` that maps expiration timestamps (seconds since UNIX epoch) to lists of node IDs.

### Architecture

```
GcWorker
  ├── storage: &StorageEngine       — reference to the storage engine
  └── index_ttl: BTreeMap<u64, Vec<u64>>  — expiry → [node_id, ...]
```

### TTL Registration

When a record is created with `ttl_ms`, the engine computes `expires_at_ms = now_ms() + ttl_ms` and passes the expiry to `GcWorker::register_ttl(id, expiry_secs)`:

```rust
pub fn register_ttl(&mut self, id: u64, expiry_secs: u64) {
    self.index_ttl.entry(expiry_secs).or_default().push(id);
}
```

Multiple nodes can share the same expiry timestamp — they are grouped in a `Vec<u64>` under the same `BTreeMap` key.

### Sweep: TTL-Based Expiration

The `sweep()` method is the core GC loop:

```rust
pub fn sweep(&mut self) -> Result<usize>
```

1. Gets the current UNIX timestamp in seconds
2. Iterates the `BTreeMap` in sorted order (ascending by expiry)
3. For each expiry ≤ now, calls `storage.delete(id, "GC TTL Expired")` on every node ID
4. Removes empty expiry entries from the map
5. Returns the count of successfully deleted nodes

**Background vs Foreground:**
- **Foreground:** `purgeExpired()` / `purge_expired()` scans all live nodes and deletes those past their deadline. This is a full scan and can be expensive on large datasets.
- **Background:** In production, `GcWorker::sweep()` runs in a `tokio::spawn` loop on a periodic schedule. This incrementally processes only registered TTL entries without scanning all nodes.

### When TTL Is Set

TTL is accepted on `put()` / `putBatch()` via the `ttl_ms` parameter:

```ts
// TypeScript
db.put({
  namespace: "sessions",
  key: "session-123",
  payload: "...",
  ttl_ms: 3600_000,  // expires in 1 hour
});
```

The server-side computation is:

```rust
let expires_at_ms = input.ttl_ms.map(|ttl| timestamp.saturating_add(ttl));
```

`saturating_add` prevents overflow — if `timestamp + ttl` exceeds `u64::MAX`, the result saturates and the record effectively never expires.

## `purge_ttl_for_deleted()` — Why Manual Deletes Need TTL Cleanup (CODE-032)

When a record is deleted manually (via `delete()` / `deleteNode()`), the GC's TTL map (`index_ttl`) still holds a reference to the deleted node ID. If not cleaned up, these stale entries accumulate, causing:

- **Unbounded memory growth** — the `BTreeMap` grows with every TTL-registered record, even deleted ones
- **Wasted GC cycles** — `sweep()` attempts to delete already-deleted nodes on every iteration

The fix is `purge_ttl_for_deleted()`:

```rust
pub fn purge_ttl_for_deleted(&mut self, active_ids: &HashSet<u64>) {
    self.index_ttl.retain(|_, ids| {
        ids.retain(|id| active_ids.contains(id));
        !ids.is_empty()
    });
}
```

**When to call:** After any bulk delete operation (e.g., `delete_by_filter`, namespace deletion) or after a series of manual deletes. Provide the set of all currently active node IDs — entries for deleted nodes are removed in O(n log n).

## Error Recovery: GC Delete Failure Retry Semantics (CODE-031)

```rust
ids.retain(|&id| match self.storage.delete(id, "GC TTL Expired") {
    Ok(_) => {
        expired_count += 1;
        false  // remove from list — success
    }
    Err(e) => {
        tracing::error!("GC failed to delete node {id}: {e}");
        true   // keep in list — retry next sweep
    }
});
```

On transient storage failures (I/O errors, lock contention):

1. The failed node ID is **retained** in the TTL map
2. An error is logged at the `error` level with the node ID
3. On the next `sweep()` invocation, deletion is re-attempted

This provides **automatic retry** without explicit backoff logic. Permanent failures (e.g., node already deleted by another path) are handled gracefully because `delete()` is idempotent with respect to the TTL map.

## Configuration

TTL behavior is configured via the SDK, not the engine config:

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `ttl_ms` | `Option<u64>` | `None` (no expiry) | Time-to-live in milliseconds from record creation |

There is no global TTL default. Each record must opt in to TTL via the `ttl_ms` parameter.

Memory limits that interact with GC:

| Config Field | Effect |
|-------------|--------|
| `memory_limit` | When exceeded, triggers eviction (separate from TTL GC) |
| `rss_threshold` | Fraction of memory limit that triggers backpressure on writes |

## Best Practices for TTL Usage

1. **Use TTL for ephemeral data** — sessions, temporary caches, rate-limiting state, conversation windows
2. **Avoid TTL on long-lived reference data** — TTL tracking adds overhead to the `BTreeMap` and sweep cycles
3. **Batch TTL expiry** — set the same `ttl_ms` for batches of records that should expire together; this groups them under fewer `BTreeMap` keys
4. **Call `purge_ttl_for_deleted()` after bulk deletes** — prevents stale TTL entries from accumulating (CODE-032)
5. **Monitor GC logs** — check for `"GC failed to delete node"` messages which indicate persistent storage issues
6. **Do not rely on TTL alone for security-critical expiry** — TTL is eventually consistent; use explicit deletion for immediate revocation

## Interaction with Compact and Rebuild

### `compact_layout()`

Physical compaction reorders nodes in BFS order but does **not** remove TTL-expired records. Run `purgeExpired()` before `compactLayout()` to ensure expired nodes are physically removed before compaction.

### `rebuildIndex()`

Rebuilds the ANN (HNSW), derived, and text indexes from all live nodes. Expired nodes are skipped because `purgeExpired()` deletes them from the canonical storage layer. However, if `purgeExpired()` has not run recently, `rebuildIndex()` may still index nodes that would be expired.

### Recommended Order

```ts
// Full maintenance cycle
db.purgeExpired();     // 1. Remove expired records
db.compactLayout();    // 2. Compact physical layout
db.rebuildIndex();     // 3. Rebuild indexes from clean state
```
