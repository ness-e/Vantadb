---
title: "ADR-022: WAL async/batch — consolidation of DRV-014/DRV-015 with current state"
type: adr
status: accepted
tags: [vantadb, architecture, adr, wal, durability, async, batch, fsync]
created: 2026-08-16
last_reviewed: 2026-08-16
related: [DRV-014-wal-batch-tradeoff.md, DRV-015-wal-async-roadmap.md]
---

# ADR-022: WAL async/batch — consolidation of DRV-014/DRV-015 with current state

## Status

Accepted (retroactive consolidation). Does **not** duplicate DRV-014/DRV-015;
it pins the current on-disk evidence so the WAL durability design is traceable
from the ADRs to the code (FND-21, P20d).

## Context

The WAL is the durability backbone: every mutation is appended before it
becomes visible. Two design decisions — sharded batch-append and the async I/O
roadmap — are already made in code and documented in two ADRs:

- **DRV-014** (`DRV-014-wal-batch-tradeoff.md`, accepted 2026-07-31): batch
  append per shard is an intentional tradeoff (1 lock + 1 `write_all` + 1
  `maybe_sync` per shard → 3-5× faster WAL writes) that deliberately reintroduced
  a `WalRecords` clone to group by shard.
- **DRV-015** (`DRV-015-wal-async-roadmap.md`, proposed 2026-08-12): two-phase
  async WAL roadmap (Phase 1 fsync group commit, platform-agnostic; Phase 2
  `io_uring`/`aio` behind a Linux-only `wal-io-uring` gate). **Roadmap only —
  no WAL code changed by that ADR.**

This ADR verifies the current state against the code so the two ADRs remain
grounded.

## Decision

1. **Sharded batch-append is the implemented write path.** `ShardedWal`
   (`src/wal_sharded.rs:9-14`) holds `Vec<Arc<parking_lot::Mutex<WalWriter>>>`
   plus an `AtomicUsize next_shard`; writes round-robin via
   `fetch_add` (`wal_sharded.rs:191`); `batch_append` groups records per shard
   and calls `WalWriter::batch_append` once per shard (`wal_sharded.rs:198-218`).
   `WalWriter::batch_append` (`src/wal.rs:297`) performs 1 lock + 1 `write_all`
   + 1 `maybe_sync`.
2. **fsync policy stays blocking, configurable.** `maybe_sync`
   (`src/wal.rs:342-356`) triggers on every batch under `SyncMode::Always` and,
   by default (`DEFAULT_PERIODIC_THRESHOLD = 1`, `src/wal.rs:340`), even under
   `Periodic`. `WalWriter::sync` (`src/wal.rs:358`) = `flush()` + `sync_data()` —
   a blocking `fdatasync` inside the shard lock (per DRV-015 §Context).
3. **Async WAL remains a roadmap, not implemented.** No `WalIo` abstraction, no
   group-commit coordinator, no `wal-io-uring` feature exist in the current
   source. DRV-015's status stays `proposed` until Phase 1 ships.

## Consequences

- **Positive:** DRV-014's 3-5× write throughput is in the current code; the
  sharded design (`wal_sharded.rs:191-218`) amortizes locking and syscalls;
  DRV-015 gives a concrete, low-risk upgrade path (Phase 1) without touching the
  on-disk format.
- **Negative:** the blocking `fdatasync` inside the shard lock remains the
  ceiling under `SyncMode::Always` (DRV-015 §Context — intra-shard
  serialization); the batch-append clone costs memory per batch (DRV-014).
- **Debt:** DRV-015 Phase 1 (group commit) and Phase 2 (`io_uring`) are
  unimplemented; watermark semantics (R1) and coordinator supervision (R2) are
  open risks until Phase 1 lands; `chaos_integrity`/`wal_resilience` remain the
  durability contract gate.

## Related

- DRV-014 (`DRV-014-wal-batch-tradeoff.md`) — batch-append tradeoff, baseline.
- DRV-015 (`DRV-015-wal-async-roadmap.md`) — async roadmap, phases & risks.
- `src/wal.rs` (`batch_append:297`, `maybe_sync:342-356`, `sync:358`,
  `DEFAULT_PERIODIC_THRESHOLD:340`), `src/wal_sharded.rs` (`ShardedWal:9-14`,
  round-robin `:191`, batch per shard `:198-218`).