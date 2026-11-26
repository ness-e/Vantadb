---
title: "ADR-0061: Implicit WAL transactions for multi-chunk put_batch — all-or-nothing batches"
kind: adr
status: accepted
description: "Every put_batch is framed in an implicit WAL transaction (Begin/Prepare/Commit) reusing the 2PC markers shipped before v0.9.0, with a dedicated txn id, one insert_lock for the whole batch, and recovery-side discard+undo — zero on-disk format change"
tags: [vantadb, architecture, adr, storage, wal, transactions, atomicity]
created: "2026-11-24"
owner: vanta-arch
---

# ADR-0061: Implicit WAL transactions for multi-chunk `put_batch` — all-or-nothing batches

## Status

Accepted. Implements FIND-640 / plan task T54. Amends
[ADR-0060](./ADR-0060-on-disk-compatibility-v090-v0100.md) §Rollback (see its
2026-11-24 amendment). Design analysis: `docs/dev/tasks/FIND-640.md`.

## Context and problem statement

Before this change, `batch_insert_with_opts` committed each 1000-record chunk as
an independent durable unit: per chunk it appended `WalRecord::Insert` records
(with no transaction framing), wrote the KV batch, and updated HNSW. A crash or
kill between chunk *k* and *k+1* left a persisted **prefix** of the batch —
violating the documented `put_batch` all-or-nothing contract (FIND-221 staged
only validation-before-write, not commit atomicity).

The key discovery of the design phase: the two-phase-commit protocol was already
implemented end-to-end (RES-01 / TXN-01):

- `WalRecord::{Begin, Prepare, Commit, Abort}` exist — **byte-identical** — in
  `v0.9.0`'s `wal.rs` (`git show v0.9.0:src/wal.rs`).
- `commit_transaction` (`txn.rs`) already writes `[Begin + ops + Prepare]` as one
  `batch_append`, applies, then writes `Commit` as the durability commit point.
- Recovery (`init.rs`) already (a) tracks `open_txn` and discards transaction
  ranges lacking a `Commit` via `skip_mask`, and (b) runs a physical **undo
  pass** that purges KV/HNSW/vstore entries stamped with an uncommitted
  `created_by_txn`.

FIND-640 therefore required no new markers, no recovery changes, and no format
change — only framing `batch_insert_with_opts` inside an implicit transaction.

## Decision

`batch_insert_with_opts` with WAL enabled (`!opts.skip_wal && self.wal.is_some()`)
executes as an implicit transaction under **one** `insert_lock` guard held for
the entire batch:

1. **WAL phase** (all durable truth first): `Begin(batch_txn)` → per-chunk
   `batch_append` of `WalRecord::Insert` records → `Prepare{txn_id, op_count}`.
   Only one chunk's `Vec<WalRecord>` is alive at a time — the RAM bound is
   unchanged.
2. **Apply phase**: per-chunk staging as before (vstore → KV → HNSW), with
   `created_by_txn` stamped to `batch_txn` via `build_kv_put_op` /
   `stage_one_node` / `serialize_batch_nodes` so the recovery undo pass can
   discriminate batch entries.
3. **`Commit(batch_txn)`** — the commit point. Durable `Commit` ⇒ replay
   re-applies the `Insert`s (idempotent upsert) and the whole batch is visible
   after reopen.
4. **Error in-process** → the existing rollback (HNSW remove, vstore tombstone,
   KV restore/delete, cache evict) runs, then `Abort(batch_txn)` is appended
   best-effort. **Crash without `Commit`** → recovery discards the frame and the
   undo pass purges the applied prefix → **0 records**.

Supporting decisions:

- **`TxnManager::alloc_implicit_id()`** allocates a dedicated txn id via
  `fetch_add` **without** registering in `active` — registering would make
  `insert()` buffer foreign writes into the implicit transaction.
- **One `insert_lock` for the whole batch is a correctness requirement**, not an
  optimisation: recovery's `open_txn` assumes a transaction's records occupy
  contiguous `global_seq` slots. Holding the lock (every WAL mutation already
  serializes on it since FIND-651/ERR-010) prevents a foreign writer's records
  from interleaving inside the frame, where they would be misattributed to
  `batch_txn` and discarded on crash recovery.
- **`Prepare` is kept** for uniformity with the RES-01 invariant ("every
  multi-op transaction emits Prepare"); `op_count` doubles as a future integrity
  cross-check. `Abort` on the error path is symmetric with `commit_transaction`.
- **`opts.skip_wal = true`** keeps the previous per-chunk behaviour — all-or-
  nothing cannot be guaranteed without a WAL, and `skip_wal` is an explicit
  opt-in for regenerable bulk loads (documented in `put_batch`).
- **No code fork for single-chunk batches**: the frame costs 3 small records
  (~40-120 bytes) — simpler than branching on chunk count.

## Consequences

### Positive

- `put_batch` is genuinely all-or-nothing across chunk boundaries and crashes,
  verified by 10 failpoint/crash-consistency tests
  (`src/storage/engine/tests/batch_atomicity.rs`): fail-second-chunk, crash
  before `Commit`, panic mid-apply, crash before apply, sharded WAL
  (`wal_shards=4`), WAL rotation mid-batch, checkpoint boundary, upsert
  residual, and concurrent-insert exclusion.
- Zero on-disk format change: `WAL_FORMAT_VERSION=3`, `WAL_POSTCARD_VERSION=1`,
  chain hash unchanged — ADR-0060 inventory unaffected; `v0.9.0` reads WALs
  containing these frames (its `open_txn` discard handles them).
- Flushes cannot split a frame: `flush_locked` takes `insert_lock` (ERR-010),
  which the batch now holds end-to-end, so `checkpoint_seq` sees a batch
  entirely or not at all. Auto-rotation mid-batch is covered because recovery
  orders rotated segments chronologically (CRASH-01).
- `SyncMode` interacts fail-safe: with `Never`, losing the `Commit` loses the
  batch — never resurrects an uncommitted one.

### Negative / residuals

- **R1 — lock held longer.** `insert_lock` is held for the whole batch rather
  than per chunk, so concurrent single writes wait longer. `put_batch` is a
  bulk API already de-facto serialized; the `insert_lock_timeout_ms` yields a
  typed retriable error. If contention is measured, follow-up is recovery
  support for multiple concurrent open transactions — not needed now.
- **R3 — upsert residual.** A crash before `Commit` on a batch that *updated*
  an existing key loses the previous version: the undo pass deletes the new
  entry but the WAL never carried the old value. Identical to an aborted
  explicit transaction today (TXN-01). The real fix — WAL-logging the prior
  value — is backlog work; the residual is documented in `put_batch` docs and
  covered by `put_batch_multi_chunk_atomic_upsert_residual`.
- **R4 — compound downgrade.** Crash mid-batch on `v0.10.0` *then* opening the
  directory with `v0.9.0`: `v0.9.0` discards the WAL frame but has no undo
  pass, so the applied prefix remains visible in KV. Compound event, recorded
  in the ADR-0060 amendment; `v0.9.0` is not retroactively patchable.

## Verification

- `cargo nextest run -p vantadb --features failpoints -E
  "test(put_batch_multi_chunk_atomic)"` — 10/10 green (2026-11-24).
- Regression: lib suite 2625/2626, the single failure an unrelated timing-bound
  test passing in isolation; both feature matrices (`default`, `failpoints`)
  compile clean.
- Perf: dedicated `benches/batch_txn_overhead.rs` (disk engine, WAL enabled,
  `SyncMode::Never`) gates the frame cost; expected delta is 3 marker records
  per batch — see plan T54 for the ≤10% regression gate.
