---
title: "DRV-015: WAL async roadmap — io_uring/aio + fsync group commit"
type: adr
status: proposed
date: 2026-08-12
tags: [vantadb, wal, performance, durability, adr, async, io_uring]
created: 2026-08-12
last_reviewed: 2026-08-12
owner: vanta-arch
related: [DRV-014-wal-batch-tradeoff.md]
---

# DRV-015: WAL async roadmap — io_uring/aio + fsync group commit

## Context

The WAL is the durability backbone of VantaDB: every mutation is appended
before it becomes visible. Today the WAL write path is already sharded and
batch-append is in place (see **DRV-014**, "WAL batch-append tradeoff",
accepted 2026-07-31): `ShardedWal` (`src/wal_sharded.rs`) spreads writes
across N shards (`Vec<Arc<parking_lot::Mutex<WalWriter>>>`, round-robin via
an `AtomicUsize next_shard`), and `WalWriter::batch_append` (`src/wal.rs:297`)
does **1 lock + 1 `write_all` + 1 `maybe_sync` per shard** — the 3-5× speedup
documented in DRV-014.

The remaining bottleneck is **the fsync, not the write**. In the current code
(`src/wal.rs`):

- `WalWriter::sync` (`src/wal.rs:352`) = `self.writer.flush()?` +
  `self.writer.get_ref().sync_data()?` — a **blocking `fdatasync`** inside the
  shard's `Mutex`.
- `maybe_sync` (`src/wal.rs:336`) triggers that fsync on **every batch** when
  `SyncMode::Always`, and by default (`DEFAULT_PERIODIC_THRESHOLD = 1`,
  `src/wal.rs:334`) even under `Periodic` it fsyncs after a single record.
- Each of the N shards therefore serializes its own blocking fsync under its
  own lock. With `SyncMode::Always` (the safe default for durability) a write
  thread cannot proceed past the fsync until the kernel returns; under
  contention the shards do not help because the cost is the syscall latency
  itself, not the lock.

So the perceived "fsync is serialized" problem has two distinct layers:

1. **Intra-shard serialization:** the issuing thread blocks on `sync_data()`
   inside the hot path.
2. **Inter-shard duplication:** N shards each pay a full `fdatasync` for the
   same logical commit point, even though a single `fdatasync` on each of the
   N files is necessary for correctness (they are separate files). What *is*
   wasteful is doing N independent syscalls when the kernel can coalesce the
   flush of all N files in one scheduling decision, and doing it on the
   writer thread instead of offloading it.

This ADR is a **roadmap**, not an implementation. It records the decision to
pursue async WAL I/O in two phases (plataform-agnostic group commit first,
`io_uring`/`aio` second) and the tradeoffs/risks, so the debate is not
re-litigated later. No WAL code is changed by this ADR.

## Decision

Adopt a **two-phase async WAL I/O roadmap**, gated behind a uniform
`WalIo` abstraction so the blocking implementation (`flush` + `sync_data`)
remains the default and the async backends are opt-in.

### Phase 1 — fsync group commit (plataform-agnostic, no new syscalls)

Introduce a **WAL sync coordinator** (background task / dedicated thread)
that coalesces the `sync` requests from all shards into batched,
non-blocking flushes:

- Writers no longer call `sync_data()` inline. They enqueue a "sync requested"
  signal for their shard and return; the coordinator drains the queue,
  performs the `fdatasync` per file (necessary because each shard is a
  separate file), and only then advances the durable watermark.
- This converts the **intra-shard blocking fsync** into an **offloaded,
  amortized** flush: the writer thread is freed immediately, and multiple
  writers committing around the same time share one flush cycle.
- Implementable with `std::sync` + a `crossbeam`/`tokio` channel + either a
  dedicated OS thread (`std::thread` + `sync_data`) or an async task that
  calls `sync_data` via `tokio::task::spawn_blocking`. **No `io_uring`,
  no platform-specific code** — works on Linux, Windows, macOS.

**Milestone (verifiable):** a benchmark shows writer-thread latency no longer
contains `fdatasync` latency (measured via `criterion` in `benches/` + a new
`wal_group_commit` bench), while `chaos_integrity` / `wal_resilience` still
prove zero data loss on crash at the chosen `SyncMode`.

### Phase 2 — `io_uring` / `aio` async submission backend (Linux-only)

Behind a feature gate `wal-io-uring` (off by default), add a backend that
submits the `write` + `fdatasync` via Linux `io_uring` (kernel 5.1+), or
`libaio` as a fallback, behind the same `WalIo` trait:

- `io_uring` lets us submit the buffered write and the `fdatasync` as
  linked/queued SQEs without blocking a userspace thread, and supports
  **`IORING_OP_FSYNC`** and **`IORING_OP_FADVISE`** — ideal for WAL.
- `libaio` (`io_submit` / `io_getevents`) is the older Linux AIO
  (direct-I/O-oriented, `aio_fsync` is supported) and serves as a fallback
  for kernels without `io_uring` or for parity with stored configurations.
- **Platform gate:** `io_uring` and `libaio` are **Linux-only**. On Windows /
  macOS the `wal-io-uring` feature **must not compile** (cfg-gated); those
  platforms keep Phase-1 group commit + `spawn_blocking` as their durable
  path. (macOS has no `io_uring`; Windows would need overlapped I/O / `FILE_FLAG_OVERLAPPED`
  which is out of scope for Phase 2.)

**Milestone (verifiable):** with `wal-io-uring` on Linux, a `wal_async`
bench shows reduced p99 append latency vs Phase 1 at equal durability, and
the same `chaos_integrity` crash-recovery contract holds. CI for this feature
runs only on the Linux runner (`--features wal-io-uring`), never on the
Windows/macOS gates.

### What this intentionally does NOT do

- It does **not** change the WAL on-disk format (`WalHeader` / record framing
  at `src/wal.rs`), the `postcard` wire format, or the recovery/quarantine
  logic (`recover_valid_records`, `quarantine_corrupt_tail`).
- It does **not** remove sharding or batch-append (DRV-014 stays in force).
- It does **not** relax durability: `SyncMode::Always` still means "fsynced
  before the call returns is durable"; group commit only relaxes *which
  thread* pays the syscall and *how many* syscalls are coalesced, bounded by
  a configurable flush interval (default ≤ a few ms) so crash loss stays
  within DRV-014's "lose at most one record" envelope.

## Consequences

- **Pros:**
  - Phase 1 removes the inline blocking `fdatasync` from the writer hot path
    on **every** platform, with zero new syscalls and zero new unsafe.
  - Phase 2 gives Linux a true async submit path (`io_uring`) and amortizes
    the flush further, lowering p99 under high write concurrency.
  - Uniform `WalIo` trait keeps the current blocking path as the safe default
    and makes the async backends swappable + testable behind feature gates.
  - No change to the durability contract or on-disk format → low blast radius
    for Phase 1; Phase 2 is feature-gated and CI-isolated to Linux.
- **Cons:**
  - Group commit introduces a durable-watermark boundary: readers / checkpoints
    must wait on the coordinator's watermark, not assume "returned = fsynced",
    when `SyncMode::Periodic` amortizes. Requires care in the engine commit
    protocol (see Risks).
  - `io_uring` adds a Linux-only dependency surface and a new crate
    (`tokio-uring` or `rio`); raises the maintenance/audit cost (a new
    `unsafe`/FFI boundary → must pass `vanta-audit` + Miri before merge).
  - Async error propagation (a coordinator flush failure) must surface as a
    `VantaError::wal_error` and trigger graceful degradation, not silent loss.
- **Tradeoff summary:** the blocking fsync is the simplest correct thing
  today. Phase 1 buys most of the latency win (offloading) with minimal risk;
  Phase 2 buys the rest (true async submit) but only on Linux and at higher
  complexity cost. We do Phase 1 first precisely because it is plataform-agnostic
  and low-risk, and treat Phase 2 as a Linux optimization, not a correctness
  change.

## Risks

- **R1 — Watermark correctness:** group commit must not report a write as
  durable before the coordinator's fsync completes. Mitigation: a
  `SeqCst` durable-watermark counter; the engine's commit await waits on it;
  covered by `wal_resilience` + `chaos_integrity` regression tests.
- **R2 — Coordinator crash vs data loss:** if the coordinator thread dies
  mid-flush, in-flight un-fsynced records must not be acknowledged.
  Mitigation: acknowledgement only after watermark advance; coordinator
  supervised by the engine, restarted or failing loud.
- **R3 — `io_uring` ABIs / kernel drift:** older kernels (CI images) may lack
  ops. Mitigation: `wal-io-uring` cfg-gated, runtime capability probe,
  automatic fallback to Phase-1 path on `ENOSYS`/unsupported.
- **R4 — New unsafe/FFI audit surface (Phase 2):** `io_uring` syscalls via
  raw fds. Mitigation: confined to the `WalIo` backend module, `// SAFETY:`
  documented, Miri + `vanta-audit` gate before merge (per AGENTS.md Regla 4).
- **R5 — Benchmark honesty:** p99 under concurrency must be measured against
  the real DRV-014 baseline, not a strawman. Mitigation: reuse `benches/`
  criterion harness (see PERF-02 baseline rig) and publish numbers in
  `docs/benchmarks/`.

## Platform dependencies

| Backend | Platform | Status | Fallback |
|---------|----------|--------|----------|
| Blocking `sync_data` (current) | All | Default, unchanged | — |
| fsync group commit (Phase 1) | All (Linux/Win/macOS) | Plataform-agnostic | — (is itself the fallback) |
| `io_uring` (Phase 2) | **Linux 5.1+** | Opt-in `wal-io-uring` | `libaio` if `io_uring` ops unavailable |
| `libaio` (Phase 2 fallback) | **Linux** | Opt-in `wal-io-uring` | Phase-1 group commit |
| `io_uring` / `libaio` | Windows / macOS | **Does not compile** (cfg-gated out) | Phase-1 group commit + `spawn_blocking` |

## Phases & verifiable milestones

| Phase | Scope | Gate / verifiable milestone | Blocks release? |
|-------|-------|------------------------------|-----------------|
| 1 | fsync group commit coordinator (all platforms) | `wal_group_commit` criterion bench shows writer latency excludes `fdatasync`; `chaos_integrity` + `wal_resilience` pass at current durability | **No** |
| 2 | `io_uring`/`aio` backend behind `wal-io-uring` (Linux only) | `wal_async` bench: lower p99 vs Phase 1 at equal durability; same crash-recovery contract; CI only on Linux runner | **No** |

## Release impact

**This roadmap does NOT block the current release.** Every deliverable is
additive (new coordinator module / new feature-gated backend) and the
existing blocking `sync_data` path stays the compiled default. Release ships
DRV-014's batch-append unchanged; Phase 1/2 land later as perf work behind
their own tasks once PERF-02 baseline rig is in place to measure them
honestly.

## Related

- `DRV-014-wal-batch-tradeoff.md` — sharded batch-append (3-5×), the baseline
  this roadmap extends.
- `src/wal.rs` — `WalWriter::sync` (`:352`), `maybe_sync` (`:336`),
  `DEFAULT_PERIODIC_THRESHOLD` (`:334`), `batch_append` (`:297`).
- `src/wal_sharded.rs` — `ShardedWal`, `Vec<Arc<Mutex<WalWriter>>>`,
  round-robin `next_shard`.
- ADR `003_sync_async_decoupling.md` — existing sync/async architecture
  boundary (reuse its `WalIo`-style abstraction convention).
- PERF-02 — baseline rig (prerequisite for honest Phase 1/2 measurement).
- `chaos_integrity`, `wal_resilience` heavy-cert tests (durability contract).

## Future tracking

- Land PERF-02 baseline rig before implementing Phase 1 (measurement gate).
- Phase 1 PR must include the `wal_group_commit` bench + `chaos_integrity`
  extension asserting watermark durability.
- Phase 2 PR: cfg-gate `wal-io-uring`, runtime probe + fallback, Miri +
  `vanta-audit` pass on the new backend module.
- Revisit this ADR when Phase 1 ships to flip status `proposed → accepted`
  and record the measured win.
