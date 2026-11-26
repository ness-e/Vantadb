---
title: "ADR-0001: Unified Configuration Architecture and Read-Only Barrier"
kind: adr
status: accepted
description: "Status: Accepted"
tags: [vantadb, architecture, adr]
---

## Status

Status: Accepted

## Context

In the MVP versions of VantaDB, configuration options were fragmented
across multiple mutable interfaces (e.g., `VantaOpenOptions`), allowing
structural engine parameters to be modified at runtime. This inconsistent
mutability caused internal consistency problems and corruption risks.

Additionally, write operations (such as inserts, updates, and deletes)
would proceed deep into the storage engine pipeline before validating the
operational state of the database (e.g., whether the engine was opened in
`read_only` mode), incurring unnecessary CPU costs, memory allocation, and
lock contention before failing.

## Decision

1. **Structural Consolidation:** Unify all initialization options into a
   single, immutable, strongly-typed structure called `VantaConfig`. All SDK
   and server entry points consume this consolidated configuration at engine
   open time (`open_with_config`).

2. **Early Entry Barrier (Fail-Fast):** Implement the
   `guard_write_allowed(&self.config)` guard function in the public storage
   API. This routine immediately evaluates the write-protection flag:

   ```rust
   pub fn guard_write_allowed(config: &VantaConfig) -> Result<()> {
       if config.read_only {
           return Err(Error::Validation { field: "read_only", .. });
       }
       Ok(())
   }
   ```

3. **Inject into Mutators:** Invoke this barrier as the first line of
   execution in all methods that alter the logical or physical state of the
   engine: `insert`, `put`, `delete`, `add_edge`, and `flush`.

## Consequences

### Benefits

* **Predictable Isolation:** The engine aborts unauthorized operations in
  $O(1)$ time without acquiring read/write locks or opening transactions in
  underlying engines (RocksDB, Fjall), preventing resource degradation.
* **Thread Safety:** Since `VantaConfig` is completely immutable after
  initialization, any possibility of data races or inconsistencies from hot
  reconfiguration in highly concurrent environments is eliminated.
* **API Simplification:** Redundant APIs and deprecated methods are removed,
  improving maintainability for clients integrating VantaDB as an embedded
  engine.

### Technical Debt / Costs

* **Mandatory Re-initialization:** If a node needs to change operational
  state (from read-only replica to read-write primary), the engine instance
  must be explicitly closed and reopened with a new `VantaConfig` structure.
  This is considered desirable behavior in industrial distributed system
  architectures.

## Amendment 2026-11-09 — Per-backend `read_only` semantics (FIND-632)

The read-only barrier is not implemented uniformly across storage backends.
This section records the semantics that each backend actually guarantees so
the contract of `VantaConfig::read_only` is honest rather than aspirational.

| Backend | Barrier | `read_only` semantics |
| ------- | ------- | ------------------- |
| **RocksDB** | Physical | OS rejects all writes |
| **Fjall** | Logical | Writes rejected by backend |
| **InMemory** | N/A | Guard barrier only |

* **RocksDB** opens with `open_cf_descriptors_read_only`: the OS/filesystem
  rejects any write and no file is created or mutated.
* **Fjall** (the default backend): no engine data, `encryption.registry`,
  key material, VantaFile/WAL/index artefact or backend write is performed;
  every mutating `StorageBackend` call returns
  `Error::Validation { field: "read_only" }`.
* **InMemory** holds no durable state; the engine-level
  `guard_write_allowed` barrier applies.

### Fjall: why the barrier is logical

`fjall` 3.1.12 / `lsm-tree` 3.1.10 expose **no read-only open mode**
(upstream request: [fjall-rs/fjall#313](https://github.com/fjall-rs/fjall/issues/313),
open). Every `Database::open`:

* takes an **exclusive** file lock on `.lock` (`src/locked_file.rs`,
  opened with `read(true).write(true)`);
* replays and then **persists the journal** with `PersistMode::SyncAll`
  (`src/db.rs` — the journal is created via `create_new` when missing);
* may rewrite per-keyspace version/manifest markers under `keyspaces/`.

Consequences, now encoded in `FjallBackend` (FIND-632):

1. A `read_only` open **still requires write permission** on the database
   directory and an **exclusive lock** — no concurrent reader alongside a
   writer, and no operation on a read-only filesystem. Both failure modes
   surface as typed errors (`Error::DatabaseBusy` for lock contention, an
   actionable `Error::Io` for `PermissionDenied`/`ReadOnlyFilesystem` that
   points at the RocksDB backend for physical read-only).
2. The set of files Fjall may mutate during a `read_only` open is pinned by
   an exact allowlist (journal `*.jnl`, `.lock`, keyspace version/manifest
   markers), shared by the tests `fjall_read_only_open_never_mutates_files`
   and `read_only_open_never_writes_the_registry`. Data files (SST/segments,
   blobs, `encryption.registry`, VantaFile/WAL/index) are never in the
   allowlist.

Follow-up: physical read-only on Fjall is tracked as **FIND-648**
(`v0.11.0`), contingent on upstream `fjall#313`.

Note: the MCP secondary reader (MCP-35) does not depend on this — ADR-0036
rejected read-only replicas for that design.
