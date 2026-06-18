# Durability Guarantees

> What happens when the power goes out, the process crashes, or the disk
> corrupts — and what VantaDB promises in each scenario.

---

## 1. Overview

VantaDB is an embedded database, not a client-server DBMS. Durability is
achieved through a **layered architecture** where each component handles a
specific failure mode:

| Layer | Role | Durability mechanism |
|---|---|---|
| **WAL** (`vanta.wal`) | Ordered journal of every mutation | CRC32C per record, auto-healing scan-forward, optional per-write `fsync` |
| **Backend KV** (Fjall) | Relational fields, metadata, internal indexes | Fjall's own journal (LSM crash consistency), `PersistMode::SyncAll` on flush |
| **Vector store** (`vector_store.vanta`) | Dense vector storage (mmap) | `msync` on flush, atomic RCU rename |
| **HNSW index** (`vector_index.bin`) | In-memory vector index | Atomic RCU persist to `.bin` on flush |

The write path is: **WAL first, then data, then indexes**. If the process
crashes mid-write, the WAL contains the mutation and replay restores it.

---

## 2. Write Path

Every mutation (insert, update, delete) follows this exact order:

```
1. WAL.append(record)          → vanta.wal
2. append_to_vstore(node)      → vector_store.vanta  (mmap)
3. backend.put(relational)     → Fjall keyspace
4. hnsw.add(node)              → HNSW in-memory index
```

All four steps must succeed for the mutation to be considered committed.
If step 1 fails (e.g. disk full), the mutation is rejected at the
application level — no partial state is written.

The WAL record carries a **CRC32C checksum** of its serialized payload.
In `SyncMode::Always`, an `fdatasync` is issued after every `append()`.
In `Periodic` mode (default), the OS page cache batches writes.

---

## 3. Flush & Checkpoint

`flush()` is the explicit durability barrier. It persists everything to
stable storage:

```
flush()
  ├── backend.flush()           ⟵ Fjall PersistMode::SyncAll (fdatasync)
  ├── vfile.flush()             ⟵ msync vector_store.vanta
  ├── read WAL record_count
  ├── save checkpoint_seq       ⟵ backend internal metadata
  ├── backend.flush()           ⟵ ensure checkpoint_seq survives
  └── save_vector_index()       ⟵ HNSW → vector_index.bin (RCU atomic rename)
```

At the end of a successful `flush()`, the system guarantees:

- All relational data is in the KV backend (Fjall SSTs)
- All vectors are in `vector_store.vanta` (flushed from mmap)
- The HNSW index is on disk as `vector_index.bin`
- `checkpoint_seq` records how many WAL entries are already flushed

### checkpoint_seq

Stored in the backend's internal metadata (`BackendPartition::InternalMetadata`,
key `b"checkpoint_seq"`). It records the total number of WAL records that
have been fully flushed. During crash recovery, WAL records whose sequence
number is ≤ `checkpoint_seq` are skipped — they're already in the backend.

If `checkpoint_seq` is missing or corrupt, it defaults to `0`, causing a
_full_ WAL replay (safe but slower).

---

## 4. Crash Recovery

When `VantaDB::open()` is called after an unclean shutdown:

```
open()
  ├── acquire file lock        ⟵ .vanta.lock (exclusive for writers)
  ├── open KV backend          ⟵ Fjall crash-recovery (automatic)
  ├── load HNSW from disk      ⟵ vector_index.bin
  │   └── if missing/corrupt → rebuild from VantaFile scan
  ├── read checkpoint_seq
  ├── if vanta.wal exists:
  │     open WalReader
  │     iterate WAL records
  │     skip if seq ≤ checkpoint_seq
  │     replay remaining:
  │       Insert  → write VantaFile + HNSW add
  │       Update  → write VantaFile + HNSW add
  │       Delete  → tombstone VantaFile header
  │     log replay count + duration
  └── open fresh WalWriter
```

### Idempotency

WAL replay is **idempotent**. Replaying the same record twice produces the
same state as replaying it once:

- **VantaFile** writes use the same node ID; overwriting is safe.
- **HNSW** `add()` is idempotent for the same node.
- **Fjall** deduplicates via LSM merge.

This is validated by `test_wal_replay_idempotence` in
`tests/durability_recovery.rs`.

---

## 5. Guarantees Table

### ✅ What VantaDB Guarantees

| Guarantee | Detail | Evidence |
|---|---|---|
| **No partial writes** | Every WAL record has CRC32C. Incomplete trailing records are truncated on next open. | ADR-002, `src/wal.rs` scan-forward |
| **Crash consistency** | After any number of crashes (including SIGKILL), the database opens without corruption and contains all committed data. | 200+ crash injection iterations (AUD-02 + AUD-03) |
| **Idempotent recovery** | Reopening the same database multiple times produces identical state. | `test_wal_replay_idempotence` |
| **Atomic checkpoint** | `checkpoint_seq` is persisted after data. If crash occurs mid-flush, WAL replay covers the gap. | `src/storage.rs:1721-1749` |
| **Graceful shutdown** | SIGTERM and Ctrl+C flush all pending data before exit. | `src/cli_server.rs:404-520` |
| **Multi-process isolation** | Exclusive `.vanta.lock` prevents concurrent write access. | `src/storage.rs:622-690` |
| **Middle corruption recovery** | If WAL is partially corrupt, Scan-Forward skips corrupt bytes and recovers subsequent records. | `test_wal_middle_corruption_auto_healing` |
| **CRC corruption detection** | Selective CRC corruption discards only the corrupt record; adjacent records survive. | `test_wal_selective_crc_corruption_recovery` |
| **WAL compaction safety** | `compact_wal()` flushes all data before rotating the WAL file. | `src/wal.rs:327-353` |
| **HNSW index recovery** | If `vector_index.bin` is missing or corrupt, HNSW is rebuilt from `vector_store.vanta`. | `src/storage.rs:745-768` |

### ❌ What VantaDB Does NOT Guarantee

| Non-guarantee | Reason | Mitigation |
|---|---|---|
| **Point-in-time recovery** | No snapshot or incremental backup API exists yet. | Manual filesystem-level snapshots (EBS, ZFS, btrfs). |
| **Replication** | VantaDB is embedded; no primary/replica WAL shipping. | Backup `vanta.wal` + data directory externally. |
| **Cross-region durability** | All data is local to the filesystem. | Use volume replication (e.g. EBS Multi-AZ). |
| **Zero data loss on `SyncMode::Periodic`** | OS page cache may lose the last few writes on power loss. | Use `SyncMode::Always` for maximum durability. |
| **Automatic WAL archival** | `compact_wal()` must be called explicitly. | Schedule periodic `compact_wal()` via cron / scheduler. |

---

## 6. Failure Scenarios

### 6.1 Process crash (SIGKILL, segfault, panic)

| State at crash | Outcome |
|---|---|
| During WAL append | Incomplete record truncated on next open via auto-healing. No committed data lost. |
| After WAL append, before backend write | WAL replay restores the mutation. |
| During `flush()` checkpoint_seq write | `checkpoint_seq` may be stale; WAL replay redundantly re-applies already-flushed records. Idempotent — no corruption. |
| During HNSW persist | `vector_index.bin` may be incomplete. On next open, HNSW is rebuilt from VantaFile. |

### 6.2 Power loss

- Behavior identical to SIGKILL, except OS page cache contents are lost.
- Writes acknowledged before the last `fsync()` survive.
- Writes in `SyncMode::Periodic` that were in the OS page cache but not yet
  `fsync`ed are lost. **This is the trade-off for throughput.**
- Use `SyncMode::Always` for power-loss resilience at the cost of 10-100x
  slower writes.

### 6.3 Disk full

- WAL `append()` returns `IoError`. The mutation is rejected.
- Existing data is intact. The database can still be opened read-only.
- Free disk space and retry, or call `compact_wal()` to rotate the WAL.

### 6.4 WAL file deleted externally

- If `vanta.wal` is deleted while the database is closed, the engine starts
  with an empty WAL. All data that was **flushed** (backend + HNSW) survives.
  Data that was only in the WAL is lost.
- If `vanta.wal` is deleted while the database is open, the engine continues
  writing to it (the file handle is still valid on most OSes). On close and
  reopen, the file is gone, and unflushed data is lost.

### 6.5 WAL file corruption

| Corruption type | Outcome |
|---|---|
| Trailing bytes (incomplete write) | Truncated to last valid record. |
| Middle bytes (garbage) | Scan-Forward skips garbage; subsequent records recovered. |
| CRC mismatch on single record | Only that record is discarded. |
| Magic/version mismatch | `IncompatibleFormat` error. Requires WAL deletion or restore from backup. |

### 6.6 External file truncation (mmap SIGBUS)

- Unix only. If `vector_store.vanta` is truncated by another process, a
  `SIGBUS` signal is raised.
- VantaDB installs a SIGBUS handler that sets an atomic flag. The application
  can detect this via `capabilities()` or operational metrics.
- The database should be closed and restored from backup.

### 6.7 Simultaneous process open

- The second process fails with `DatabaseBusy` error. Exclusive file lock
  (`try_lock_exclusive`) prevents concurrent write access from multiple
  processes.
- Multiple read-only processes can open simultaneously (shared lock).

---

## 7. Configuration Trade-offs

### SyncMode

| Mode | Behavior | Write throughput | Max data loss on power loss |
|---|---|---|---|
| `Always` | `fdatasync` per WAL append | ~50-200 writes/s | 0 — every write is on disk |
| `Periodic` (default) | Buffered writes, sync on `flush()` | ~10,000+ writes/s | Up to `flush_interval_ms` of writes |
| `Never` | No explicit sync | ~50,000+ writes/s | Entire WAL since last OS sync |

### flush_interval_ms

Default: `1000` (1 second). Controls how often the background flush timer
fires in `Periodic` mode. Lower values reduce data loss window, higher
values increase throughput.

### Compact WAL

`compact_wal()` calls `flush()` internally then rotates the WAL file.
Scheduling it periodically (e.g. every N writes or every hour) keeps the
WAL file small and reduces recovery time after a crash.

---

## 8. Backup Recommendations

### Minimal (no backup tooling)

```
cp -r /path/to/vantadb/ /backup/location/
```

Copy the entire data directory while the database is closed, or use
filesystem snapshots (ZFS, btrfs, LVM) for point-in-time consistency.

### With WAL archival

```
# 1. Compact WAL to archive current state
db.compact_wal()

# 2. Copy data directory (atomically if possible)
cp -r /path/to/vantadb/ /backup/location/
```

The archived `vanta.wal.<timestamp>` files contain the journal up to the
compaction point. Keep them if you need historical replay.

### Testing backups

Always verify a backup by opening it in a separate process:

```
VantaDB("/backup/location/vantadb", read_only=True)
```

If it opens and queries return expected data, the backup is valid.

### What to include in a backup

| File | Required? | Notes |
|---|---|---|
| `vanta.wal` | Yes | Current WAL (unflushed mutations) |
| `vector_store.vanta` | Yes | Vector data (mmap) |
| `vector_index.bin` | Optional | Rebuilt on open if missing |
| `internal.db/` | Yes | Fjall LSM tree (relational data) |
| `vanta.wal.*` (archived) | Optional | Previous WAL segments |
| `.vanta.lock` | No | Created on open |

---

## 9. Verified By

| Test suite | Location |
|---|---|
| Crash injection (SIGKILL) | `tests/storage/crash_injection.rs` |
| WAL corruption auto-healing | `tests/storage/wal_resilience.rs` |
| Fjall + WAL durability recovery | `tests/durability_recovery.rs` |
| Graceful shutdown | `vantadb-python/tests/` (server E2E tests) |
| Chaos testing (failpoints) | `tests/storage/chaos_testing.rs` |
