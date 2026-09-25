---
title: "ADR-038: WAL fsync-batching opt-in (group-commit)"
type: adr
status: proposed
tags: [vantadb, architecture, adr, wal, durability, fsync, batch, group-commit]
created: 2026-09-05
last_reviewed: 2026-09-05
related: [DRV-014-wal-batch-tradeoff.md, DRV-015-wal-async-roadmap.md, ADR-022-wal-batch-async.md]
---

# ADR-038: WAL fsync-batching opt-in (group-commit)

## Status

**Proposed** — spec only (`FUT-12-spec`, plan `2026-09-04-durability-release-readiness.md` Task 9).
Becomes `accepted` when the implementation lands with the §Acceptance evidence
(≥10× bench + bounded loss-window test + `wal_resilience`/`chaos_integrity` green).
Supersedes nothing; extends `DRV-015` Phase 1 (group commit) from roadmap to buildable spec
and is grounded in `ADR-022` (current-state pin).

## Context

The WAL is the durability backbone: every mutation is appended before it becomes visible.

Current write path (verified against code, 2026-09-05):

- `WalWriter::append` / `batch_append` (`src/wal.rs:305-370`): frame
  `[len u32 LE][postcard payload][crc32c u32 LE]`, 1 `write_all` per call.
- `ShardedWal::batch_append` (`src/wal_sharded.rs:215-235`): groups records by
  round-robin shard, one `batch_append` per shard — 1 lock + 1 `write_all` +
  at most 1 `maybe_sync` per shard (3-5× WAL-write win, `DRV-014`).
- `maybe_sync` (`src/wal.rs:377-394`): `Always` → `sync()` every write;
  `Periodic` → `sync()` after threshold (default 1, i.e. every write unless
  `flush_threshold` configured); `Never` → no auto-sync, explicit `sync()` only
  (`FIND-63`).
- `sync()` (`src/wal.rs:397-402`) = `flush()` + `sync_data()` — a **blocking
  `fdatasync` inside the shard lock**.
- Measurement rig exists: `benches/wal_throughput.rs` (3 SyncMode × 4 batch
  sizes, 10k records/iter, cold fsync per iter).

The remaining bottleneck is the **fsync, not the write** (`DRV-015` §Context,
`ADR-022` §Decision): each batch pays a full blocking `fdatasync` on the writer
thread. `DRV-015` Phase 1 (group commit, platform-agnostic) is the agreed
direction but was roadmap-only — no coordinator, no config, no bench, no test.
This ADR turns Phase 1 into a buildable, opt-in spec.

User decisions (already taken, Gate P + plan Task 9 — not re-asked):

1. Batching is **100% opt-in**; the default path stays byte-identical.
2. Design is **group-commit with a time/size window reusing `batch_append`**.
3. Acceptance is **≥10× batch throughput + a declared, testable loss window**.

## Decision

Ship fsync group-commit as an **opt-in WAL mode** behind additive config.
Default (`enabled = false`) compiles and behaves exactly as today.

### Design (group-commit, time/size window, reuses `batch_append`)

- Writers no longer call `sync_data()` inline when the mode is enabled. They
  serialize into the existing framing via the existing `batch_append` path and
  enqueue a commit request (shard id + framed bytes) on a bounded
  `crossbeam` channel; the writer thread returns once bytes are queued
  (queued ≠ durable — see §Loss window).
- A single **coordinator** (dedicated OS thread, `std::thread`; no new syscalls,
  no `io_uring`, all platforms) drains the queue per cycle. A cycle closes when
  **either** `max_batch_records` is reached **or** `max_wait_ms` expires —
  whichever comes first. Per cycle it calls `WalWriter::batch_append` **once per
  non-empty shard** (reusing the DRV-014 path: 1 lock + 1 `write_all` per shard)
  followed by **one `sync()` per touched shard**, then advances a `SeqCst`
  durable watermark and acknowledges exactly the requests covered by that flush.
- Acknowledgement rule (watermark correctness, `DRV-015` R1): a write is
  reported durable **only after** the coordinator's `sync()` covering it
  completes. Readers/checkpoints wait on the watermark, never on "returned".
- Failure rule (`DRV-015` R2): coordinator flush failure surfaces as
  `VantaError::WalError` (loud) and un-acked requests stay un-acked; the
  coordinator is supervised by the engine (restart or fail-loud, never
  silent loss). Coordinator death with in-flight queued bytes = those bytes
  are **not** acknowledged (they fall inside the declared loss window on crash,
  and are retried-or-errored on live failure — implementation picks one and
  documents it; behavior outside the window is unchanged).
- Concurrency: `parking_lot::Mutex` per shard (existing) + lock-free enqueue
  (`crossbeam`) preferred on the hot path; `Send + Sync` on all shared types;
  no `panic` on hot paths — degrade to loud error.

### Contract First (typed interface, defined before implementation)

```rust
/// Opt-in group-commit config. All fields additive; default = today's behavior.
pub struct WalBatchConfig {
    /// Master switch. `false` (default) = current path, byte-identical.
    pub enabled: bool,               // default: false
    /// Cycle closes when this many records are queued (per coordinator cycle).
    pub max_batch_records: usize,    // default: 1000
    /// Cycle closes when this long elapses since the cycle opened (ms).
    pub max_wait_ms: u64,            // default: 5
    /// Backpressure: enqueue beyond this errors instead of growing unbounded.
    pub max_queued_records: usize,   // default: 100_000
}
```

- Evolution: additive `Option<T>`/defaulted fields only — One-Version Rule, no
  `StorageEngineV2`-style fork; public enums touched stay `#[non_exhaustive]`.
- Validation at boundaries only: env parsing (`VANTADB_WAL_BATCH_*`) and builder
  args validate ranges (`max_batch_records > 0`, `max_wait_ms > 0`); the
  engine↔storage↔wal core trusts the typed struct (no re-validation inside).
- Error semantics: single `VantaError` (`WalError` + stable `code`), never
  `panic`/`Option`-silence on the write path.
- Naming follows the project table: `WalBatchConfig::{enabled, max_batch_records,
  max_wait_ms, max_queued_records}`, `is_durable(seq) -> bool` on the watermark.

### Hyrum surface (what is / isn't promised)

- Promised when `enabled = false`: today's observable behavior, unchanged
  (framing, sync cadence per `SyncMode`, error codes, recovery output).
- Promised when `enabled = true`: throughput win (§Acceptance) + loss window
  **exactly** the declared bound (§Loss window) + `returned ≠ durable`
  (durable = watermark-acked) + on-disk format and recovery output unchanged.
- Explicitly **not** promised: iteration order across shards (documented
  unordered, as today), exact cycle timing (only the upper bound `max_wait_ms`
  is a contract), any Phase 2 `io_uring` behavior (out of scope).

### Loss window (declared and testable)

With `enabled = true`, crash loss is bounded by: **writes queued within the
last open cycle** (`≤ max_batch_records`, `≤ max_wait_ms`) **+ bytes written but
not yet `sync()`-ed**. Default config (`1000` records / `5` ms) → worst case is
5 ms of ingest plus the in-flight batch. The bound is a function of config, so
operators trade latency/throughput against loss by tuning the two knobs; the
default path (`enabled = false`) keeps today's bound (≤1 record under
`Periodic`-default/`Always`).

## Acceptance

All three must hold before this ADR flips to `accepted`:

1. **≥10× batch throughput** — new `wal_group_commit` criterion bench (extends
   `benches/wal_throughput.rs`, same 10k-records/iter cold-fsync method):
   opt-in group-commit vs single-record `append` under the same `SyncMode`
   shows **≥10× ops/sec**, and writer-thread latency samples exclude `fdatasync`
   latency. Numbers + env (CPU/RAM/OS) recorded; baseline is the DRV-014 path,
   not a strawman (Regla 9/11).
2. **Declared loss window, tested** — new `group_commit_loss_window_bounded`
   test: kill -9 mid-window (or `failpoints` equivalent) → recovery replays all
   watermark-acked records and loses **at most** the declared bound; plus
   `wal_resilience` and `chaos_integrity` green with `enabled = true`.
3. **Default intact** — full `wal` suite green with `enabled = false` and **zero**
   changed expectations in existing tests (byte-identical default path).

## Explicit limits (what this does NOT do)

- **Default intact, zero behavior change without the flag.** No existing caller,
  test expectation, sync cadence, or error string changes when `enabled = false`.
- **No on-disk format change** (`WalHeader`, framing, `postcard` wire format),
  no recovery/quarantine change (`recover_valid_records`,
  `quarantine_corrupt_tail`), no sharding change.
- **No Phase 2** (`io_uring`/`libaio`, `wal-io-uring` gate) — stays future work
  under `DRV-015`.
- **`SyncMode::Always` + batching is a new durability mode**, available only
  under the flag and documented as `returned = queued, durable = watermark-acked`.
  It does not weaken `Always` on the default path.
- **No unbounded queues**: `max_queued_records` backpressure is mandatory; list
  paths stay paginated (no unbounded `Vec` returns added).

## Consequences

- **Positive:** writer threads stop paying inline `fdatasync` on every platform
  with zero new syscalls and zero new `unsafe`; `batch_append` reuse keeps the
  DRV-014 win and compounds it (fewer syncs per record); loss/throughput tradeoff
  becomes two explicit knobs instead of folklore.
- **Negative:** new durability mode to document (`queued` vs `durable`);
  coordinator thread + channel add a (small) steady-state cost even at low load;
  engine commit protocol must respect the watermark (integration work, bounded).
- **Debt if shipped without §Acceptance:** an unmeasured coordinator is
  speculation (Regla 9) — the bench + loss-window test are merge gates, not
  follow-ups.

## Risks

- **R1 — Watermark correctness** (inherits `DRV-015` R1): ack-before-fsync =
  data-loss lie. Mitigation: `SeqCst` watermark, ack-only-after-sync,
  `wal_resilience` regression. Test: §Acceptance.2.
- **R2 — Coordinator failure** (inherits `DRV-015` R2): death mid-flush must not
  ack in-flight bytes. Mitigation: supervised coordinator, loud `WalError`,
  un-acked stays un-acked. Test: kill/failpoint test in §Acceptance.2.
- **R3 — Benchmark honesty** (inherits `DRV-015` R5): win must be vs the real
  DRV-014 baseline with cold fsync. Mitigation: extend `wal_throughput.rs`
  method unchanged, publish env + command. Gate: §Acceptance.1.

## Release impact

**Does not block release.** Spec-only deliverable; the future implementation is
additive (new config struct + coordinator module + bench + tests) with the
blocking `sync_data` path as the compiled default. No migration, no format bump,
no feature-gate matrix change.

## Related

- `DRV-014-wal-batch-tradeoff.md` — sharded batch-append baseline (3-5×).
- `DRV-015-wal-async-roadmap.md` — Phase 1 group-commit roadmap this spec builds.
- `ADR-022-wal-batch-async.md` — current-state pin (group commit unimplemented).
- `ADR-003-sync-async-decoupling.md` — sync/async boundary convention.
- `src/wal.rs:305-402` (`append`, `batch_append`, `maybe_sync`, `sync`),
  `src/wal_sharded.rs:215-235` (per-shard batch), `src/config.rs:85-103`
  (`SyncMode`), `benches/wal_throughput.rs` (measurement rig).

## Future tracking

- Implementation task (registers here when opened): new `WalBatchConfig` +
  coordinator + `wal_group_commit` bench + `group_commit_loss_window_bounded`
  test + `wal_resilience`/`chaos_integrity` run with `enabled = true`.
- On landing with §Acceptance evidence: flip this ADR `proposed → accepted`,
  record measured win + env in this file, and close the loop on `DRV-015` Phase 1.
