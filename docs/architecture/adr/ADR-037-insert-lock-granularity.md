---
title: "ADR-037: insert_lock Granularity — Keep the Global Write Lock, Measure First"
type: adr
status: accepted
tags: [vantadb, architecture, adr, concurrency, storage, wal, hnsw]
created: 2026-09-04
last_reviewed: 2026-09-04
---

# ADR-037: `insert_lock` Granularity — Keep the Global Write Lock, Measure First

## Context

With the async ingestion channel already optimal (RES-03 closed as MEDIDO-NO-APLICA;
FIND-57 set the pipeline default to `worker_count = 1` at 111.5 ops/s), the
ingestion ceiling is the engine's serial write path: one global
`insert_lock: FairMutex<()>` (`src/storage/engine/mod.rs:318`) plus one WAL
fsync per write (`src/wal.rs:374`, `DEFAULT_PERIODIC_THRESHOLD = 1`).
Throughput degrades monotonically when adding workers (−43% with w=4;
`docs/operations/BENCHMARKS.md` §13).

The WAL half of the ceiling is tracked separately (FUT-12 — durability-policy
decision pending). This ADR covers ONLY the `insert_lock` half (FIND-59):
is the global lock gratuitous serialization that finer granularity
(sharding, RCU, batching) can lift for >2× gain over the §13 baseline, or is
it load-bearing correctness?

Evidence: full read of `mod.rs` / `insert.rs` / `delete.rs` / `txn.rs` /
`ingestion.rs`, partial read of `ops.rs` / `maintenance.rs` / `get.rs` /
`init.rs` / `wal.rs` / `wal_sharded.rs` / `config.rs`, plus codegraph over
`CPIndex::add`. Zero production code changed (analysis task).

## Invariants Protected by the Global Lock

1. **WAL-vs-index ordering (ERR-010).** `insert()` / `delete()` /
   `batch_insert()` append the WAL record and queue the HNSW mutation under
   one guard; `flush()` holds the same guard across
   `[drain → serialize → count → checkpoint_seq write]`
   (`maintenance.rs:39-48`). So a checkpoint always covers exactly the
   quiescent set of WAL records ≤ `checkpoint_seq`: no invisible records, no
   replay duplicates. Break the single guard and checkpoints can count a WAL
   record whose HNSW mutation is still queued.
2. **HNSW topology atomicity.** `CPIndex::add()` (`src/index/graph.rs:595`)
   is a multi-step read-modify-write over shared global state: entry-point
   read (`AtomicU128`, L332), `search_layer` descent, **bidirectional**
   `connect_layer_neighbors`, `max_layer` update. The maps underneath are
   sharded-concurrent (`DashMap nodes`, L330), but two concurrent `add()`s
   interleave entry-point/max_layer reads and miss each other's reverse
   edges (ghost/orphan links, entry-point resurrection — the PERF-23 class
   of bugs). The lock serializes graph *topology* writes, not map access.
3. **Pending-batch drain atomicity.** `pending_hnsw_batch` accumulates ops
   from `insert()`/`delete()` and drains atomically under one acquisition
   (`ops.rs:135-156`); the opportunistic `try_push_pending_hnsw` never blocks
   (`ops.rs:162-204`). Sharded locks would need a two-level drain protocol.
4. **delete ↔ consolidate race (FND-02-M3).** `delete()` removes from HNSW +
   backend under the lock; `consolidate_node_inner` holds it for the whole
   critical section plus a liveness version-check
   (`maintenance.rs:267-293`). Without one common lock, deletes resurrect or
   leave zombies.
5. **Checkpoint write atomicity.** `checkpoint_seq` is recorded after the
   snapshot serializes, under the same guard (`maintenance.rs:67-93`);
   recovery replays by skipping exactly `checkpoint_seq` records round-robin
   across shards (`init.rs:460-467`).

Readers pay nothing for all of this: `get()`/`search` never take
`insert_lock` (RCU via `ArcSwap<CPIndex>` + `DashMap` + `try_write` degrade,
`get.rs:102-129`). Recovery is single-threaded at open and needs no lock.

## Decision Matrix — Alternatives × Risk × Estimated Gain

Baseline: §13 post-FIND-57, p1/w1 **111.5 ops/s** (~9 ms/op). Per-op split
(fsync vs HNSW-add vs lock overhead) is **not measured** — stated explicitly;
FIND-61 (spike) resolves it. Estimates below are bounded reasoning, not claims.

| # | Alternative | What changes | Risks (R1/R2, Regla 8, durability, WASM) | Est. gain vs §13 | Verdict |
|---|-------------|--------------|------------------------------------------|------------------|---------|
| (a) | Shard the lock (N locks, hash-by-id) | N `FairMutex`es; per-node vstore/backend/cache ops parallelize | **High.** No `namespace` exists in the core (`UnifiedNode` = id/bitset/vector/relational/edges — namespace lives in the SDK/memory layer), so the shard key would be id-hash: new observable partitioning with zero natural boundary. HNSW topology (entry_point/max_layer/bidirectional edges), `checkpoint_seq`, and WAL ordering stay **global** → two-level locking (shard + global) under Regla 8's fixed order `cardinality_stats → insert_lock → {...}`; multiplies FND-02-class deadlock surface and breaks the delete↔consolidate single-lock protocol. WASM-safe (parking_lot) but pointless there (single thread). | **~0.** The serial section that dominates (HNSW `add` + fsync-per-op to one device) stays serial; Amdahl caps the win at the parallelizable fraction (vstore append + `backend.put`), minus two-level protocol overhead. Cannot clear the >2× gate even in theory without touching fsync. | **Reject** |
| (b) | RCU / lock-free write path | Per-node locking or lock-free graph writes; global lock removed | **Very high.** Bidirectional edge writes are multi-node write sets → lock-ordering-by-id protocol or atomic entry-point CAS + versioning; re-opens every FND-02/ERR-010/ERR-014 race the current protocol closed (invisible records, ghost entries, zombie/resurrect). Violates Regla 8's single global order. WASM: atomics-heavy redesign must still degrade to single-thread — test matrix doubles. | **~0** for the same reason as (a): fsync-per-op unchanged, and it is the dominant term. All risk, no reachable gain. | **Reject** |
| (c) | Batch inserts under one lock take | Already EXISTS at engine layer: `batch_insert_with_opts` does WAL `batch_append` + KV `write_batch` + bulk HNSW under ONE guard (`insert.rs:750-805`). Missing only at pipeline layer: `AsyncIngestionPipeline::process` calls `engine.insert` per task (`ingestion.rs:107-116`) — one fsync per op. | **Medium, and out of scope.** Amortizing N ops per fsync widens the crash-loss window from 1 op to N ops → durability-policy change, which is FUT-12's explicitly reserved decision, not this task's. As a *pure lock-half* change (same fsync count, fewer acquisitions) the gain is lock-overhead only — negligible (<5% of a ~9 ms op). Ordering already not preserved (§13 pre-mortem 1), so batching adds no new ordering regression; latency-per-ack shape changes (needs its own bench contract). | **Plausibly >2×, but the gain comes from fewer fsyncs = FUT-12's half**, not from lock granularity. | **Defer to FIND-61 spike + FUT-12** (do not implement standalone) |
| (d) | Do nothing (document why the lock is necessary) | Docs only: this ADR + §13 note | **None.** No surface change, no Hyrum expansion, no Regla 8 touch, WASM-agnostic. | 0 (keeps 111.5 ops/s baseline intact; the ceiling is lifted by FUT-12, not here) | **RECOMMENDED** |

## Decision

**(d) — Keep the global `insert_lock`. No granularity change.**

Rationale: the lock is load-bearing for five correctness invariants
(ERR-010 checkpointing, HNSW topology atomicity, batch-drain atomicity,
delete↔consolidate protocol, checkpoint write atomicity) that were each paid
for with real incidents (FND-02, ERR-010, ERR-014, PERF-23). Every finer
granularity keeps the dominant cost (fsync-per-op, FUT-12's half) serial
while multiplying the exact deadlock/ordering surface Regla 8 exists to
suppress. No alternative clears the task's >2× gate on the lock half alone;
(c)'s real lever is fsync amortization, which belongs to FUT-12's durability
decision. Revisit only with measured evidence (FIND-61).

Collateral observation (unverified, out of scope, NOT a new row):
`commit_transaction()` (`txn.rs:119-213`) appends its WAL batch and applies
stores WITHOUT `insert_lock` (only opportunistic `try_push`), unlike every
other mutating path — the ERR-010 coverage of the txn-commit path deserves
verification. Folded into FIND-61's scope rather than asserted as a finding.

## API Contract Assessment

- **Contrato definido:** none new — (d) changes no trait, struct, enum, endpoint, or error variant. The write-path contract (`insert`/`batch_insert`/`delete` semantics + `VantaError::Timeout` on contended `acquire_insert_lock`) is unchanged.
- **Hyrum surface:** unchanged by (d). Noted cost of (a): a shard count / hash function / per-shard ordering guarantee would become observable behavior consumers depend on (shard-count config, cross-shard ordering) — surface expansion with no reachable gain is exactly what Hyrum's Law warns against.
- **Evolución:** no additive or breaking change under (d). If FIND-61 ever justifies pipeline batching, it must land as an **additive opt-in** (new pipeline mode/options, default `insert()` untouched) per the One-Version Rule — never a silent default change (cf. FIND-57 precedent).
- **Validación:** boundaries unchanged (FFI/HTTP/CLI validate; engine internals trust typed contracts). No new validation sites.
- **Naming & coherencia:** no new public names introduced.

## Impact Analysis

- **Modules affected:** none (docs only: this ADR, Backlog, BENCHMARKS §13, avance). Follow-up spike FIND-61 touches bench/config only, 0 prod code.
- **Concurrency model:** unchanged — `FairMutex` global + RCU reads + per-shard WAL mutexes + Regla 8 order intact.
- **Durability guarantee:** unchanged — `SyncMode::Periodic` + threshold 1 (fsync per write) default; policy decisions stay in FUT-12.
- **Backward compatibility:** transparent — no bump, no deprecation cycle.
- **Module boundaries:** no trait touched; `cargo modules --acyclic` N/A (0 code); no feature gates touched (analysis notes `async-ingestion` never enters the wasm build).

## Implementation Plan

No implementation. Follow-up is the measurement spike FIND-61 (timeboxed
≤1d, bench/config only, 0 prod code):

1. Decompose per-op cost with existing harness: `SyncMode::Always` (baseline)
   vs `SyncMode::Never` (lock+HNSW only) vs `skip_wal` batch — isolates the
   fsync term from the lock+HNSW term on the §13 matrix.
2. Prototype pipeline-level micro-batching (N={8,16,32}) behind a bench-only
   flag; report ops/s + p50-ack-latency + crash-window widening.
3. Verify ERR-010 coverage of `commit_transaction` (interleaving review; flush
   during commit — checkpoint-vs-queued-mutation).
4. Gate: open a build slice ONLY if (2) shows ≥2× on §13 AND FUT-12 has
   decided the batch loss-window policy; otherwise close with numbers.

## Verification Checklist

- [x] Contrato tipado definido antes de implementar — N/A, 0 código; decisión documentada antes de cualquier cambio futuro
- [x] `VantaError` mapping consistente — sin cambios; `Timeout` en `acquire_insert_lock` intacto
- [x] `#[non_exhaustive]` — sin enums nuevos/extendidos
- [x] Sin ciclos — 0 código, boundaries intactos
- [x] Feature matrix — 0 código; nota: `async-ingestion` fuera del build wasm (verificado en RES-03)
- [x] ADR escrito — este documento (cambio significativo analizado, decisión (d))

## Consequences

- Pros: five hard-won invariants stay intact; zero regression risk; no new observable surface; FUT-12 keeps single ownership of the durability/throughput tradeoff; §13 baseline stays the honest ceiling reference.
- Cons: ingestion stays at ~111.5 ops/s until FUT-12 (accepted — throughput work without a durability decision would be the wrong optimization).
- Debt: per-op fsync-vs-lock split still unmeasured → FIND-61 spike (timeboxed, tracked, low priority).
