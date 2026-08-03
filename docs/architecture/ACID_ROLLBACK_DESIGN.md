---
title: ACID Rollback — Multi-Layer Design
type: architecture
status: active
tags: [vantadb, architecture, acid, transactions, wal, mvcc, rollback]
last_reviewed: 2026-08-03
aliases: [INV-010, ACID_ROLLBACK_DESIGN]
---

# ACID Rollback — Multi-Layer Design (INV-010)

Date: 2026-08-03
Status: design (no code changes — `cargo check -p vantadb` green)

## 1. Executive Summary

VantaDB has ACID Phase 1 (WAL `Begin`/`Commit`/`Abort` + crash-recovery
skip-mask), Phase 2 (buffered write transactions), and Phase 3 (MVCC snapshot
isolation via `NodeMetadata.created_by_txn` / `deleted_by_txn`). What is
missing is **coordinated multi-layer rollback**: the WAL is the only layer
with true rollback semantics (abort = skip on replay). VantaFile, HNSW, and
the derived indexes (edge / scalar / cardinality) have no compensation path
for a transaction that fails or aborts *after* the commit point has been made
durable, and recovery today loses MVCC stamps (`replay_write_node` writes
`created_by_txn: 0`).

This document maps the current state, designs a two-phase commit/rollback
protocol with the WAL as the single commit arbiter, evaluates the three
approaches from the (recovered) `ACID_TRANSACTIONS.md` research, and proposes
a phased plan (4a–4d) with verifiable acceptance criteria.

**Recommended decision:** extend the existing Approach B (custom transaction
layer over the existing WAL) with an explicit `Prepare` record and reorder the
commit point from *before* store application to *after* it. The WAL batch
`Begin + ops` becomes the durable "prepare" point; `Commit` is appended only
after all stores are applied; `Abort` is appended on any apply failure. This
makes runtime apply failures compensable (recovery skips them) while keeping
crash atomicity (recovery rolls forward a prepared transaction). No new
dependencies, no mmap page-journaling, one backward-compatible WAL format bump
(v1 → v2).

## 2. Research Gap: `ACID_TRANSACTIONS.md`

### 2.1 The gap

The bitácora (`docs/progreso/bitacora.md:96-108, 615-618`) references
`docs/research/ACID_TRANSACTIONS.md` as a "14-page analysis with 3 evaluated
approaches". **The file does not exist in the working tree** — it was deleted
by commit `8b1c52cd` ("docs: prune stale audit reports, plans, research,
reviews, and personal logs").

### 2.2 Recovery

The document **was preserved in git history** and has been recovered verbatim:

```
git show 8b1c52cd^:docs/research/ACID_TRANSACTIONS.md
```

(created/touched by `0b3b8353` "feat: fase 4 completada", deleted by
`8b1c52cd`). Recommendation: restore it to `docs/research/ACID_TRANSACTIONS.md`
in a follow-up docs change so the analysis it depends on (P3/P4/R3 in the
bitácora) is not orphaned again.

### 2.3 What the lost document contained (recovered summary)

The original doc defined ACID for VantaDB's embedded vector engine, analyzed
the write path (WAL → VantaFile → KV → HNSW), documented the lack of
cross-layer coordination, and evaluated three approaches:

| Approach | Idea | Verdict (original) |
|---|---|---|
| **A — Fjall-only** | Use Fjall's `batch()` / journal as the transaction primitive | **Rejected** — KV-only; does not cover VantaFile or HNSW; no snapshot isolation; no cross-layer rollback hooks |
| **B — Custom transaction layer over existing WAL** | Add `Begin`/`Prepare`/`Commit`/`Abort` to `WalRecord`; buffer writes; WAL is the commit arbiter | **Recommended** — full ACID across all layers, reuses proven WAL/CRC32C/recovery, no new deps, phased delivery |
| **C — SQLite-style rollback journal / WAL-mode** | `-journal` undo file + mmap page restore, or emulate SQLite WAL-mode | **Rejected** — mmap rollback is risky (crash mid-`mprotect`), duplicates the existing WAL, and VantaDB is already WAL-mode-like |

The original plan: Phase 1 WAL transaction records → Phase 2 buffered
transactions → Phase 3 snapshot isolation. **All three phases were
implemented** (Phase 1: P3 in bitácora; Phase 2: P3/P4; Phase 3: VFY-011 +
`docs/research/MVCC_SNAPSHOT_ISOLATION.md`). What the original doc deferred —
and what this design completes — is the *rollback side*: the original doc
assumed "if any step fails, the whole transaction is rolled back by skipping
it during WAL replay", but never designed the runtime compensation path, the
commit-point ordering, or the MVCC×recovery interaction. That is INV-010.

## 3. Current State by Layer

### 3.1 Table

| Layer | File(s) | Rollback today | MVCC | Notes |
|---|---|---|---|---|
| **WAL** | `src/wal.rs`, `src/wal_sharded.rs`, `src/storage/engine/init.rs` | ✅ Abort = skip-mask on replay | n/a | `WalRecord::{Begin,Commit,Abort}` (v1). `batch_append` writes a whole batch in one `write_all` + at most one `maybe_sync` per shard → single commit point. Recovery: two-pass skip-mask skips `Begin..Abort` and unclosed `Begin`. |
| **VantaFile** | `src/storage/vfile.rs`, `src/storage/ops.rs` | ❌ Deferred (P4) | ❌ | Append-only mmap; `write_cursor` persisted in file bytes `[16..24]` via `save_cursor()`; no un-write. Only compensation today: `FLAG_TOMBSTONE` (0x8) set when the KV write fails (P4 fix) and on delete. |
| **KV backend** | `src/backends/*`, `src/storage/ops.rs` | ⚠️ Partial | ✅ Partial | `NodeMetadata { created_by_txn, deleted_by_txn }`. Deletes are stamped, not removed; GC via `gc_mvcc_versions(safe_cutoff)`. **Single version per key** — an update overwrites the previous version (no version chain). |
| **HNSW (CPIndex)** | `src/index/graph.rs`, `src/storage/engine/mod.rs` | ❌ | ❌ | In-memory DashMap + neighbor lists; `hnsw.nodes.remove(&id)` is physical (loses neighbor lists); entry-point promotion on removal. Not WAL-backed — rebuilt from VantaFile if empty at startup. Mutated at commit under `insert_lock` (plus Rayon micro-batch `pending_hnsw_batch`, 64 ops). |
| **Derived: edge / scalar / cardinality / cache** | `src/edge_index.rs`, `src/scalar_index.rs`, `src/storage/engine/ops.rs` | ❌ | ❌ | **Eagerly mutated at `insert()`/`delete()` call time — before the txn-buffer check** (see 3.2). Not compensated on abort; not replayed by recovery; recreated empty per process (init.rs:140-141). |

### 3.2 Key findings from the code

**F1 — Commit point is durable *before* stores are applied.**
`commit_transaction` (ops.rs:300-365) appends the full batch
`[Begin, ops..., Commit]` via `batch_append` *first*, then applies to stores.
This is correct for crash consistency (recovery replays committed txns) but
creates the **"liar problem"**: if a store apply fails at runtime (not a
crash), the caller receives `Err` even though the transaction is already
durable and *will* be applied on next restart. There is no runtime
compensation path — the comment at ops.rs:1437-1441
("Full saga/2PC is deferred to ACID Phase 0") acknowledges this for `delete`,
and insert only has the P4 per-op KV-failure tombstone.

**F2 — Recovery drops MVCC stamps.**
`replay_write_node` (mod.rs:381-406) writes `created_by_txn: 0`.
Post-restart, every replayed node is visible to *any* snapshot
(`0 <= S.txn_id`), so snapshot isolation is silently weakened for data that
went through WAL replay.

**F3 — Derived indexes are mutated eagerly, outside the txn.**
`insert()` (ops.rs:619-630) updates `edge_index` + `scalar_index` (and
cardinality stats at 544-617) **before** the active-txn buffering check
(ops.rs:647-667); `delete()` does the same (ops.rs:1472-1479) before its
buffering check (ops.rs:1482-1500). `commit_transaction` never touches them;
`abort_transaction` never reverts them. Consequence: an aborted txn leaves
stale derived entries (ghost scalar/edge entries for never-committed nodes;
missing entries for still-committed nodes). They are recreated empty per
process and repopulated on the next non-txn mutation, so a restart partially
hides the problem — but within a long-running process they are inconsistent.

**F4 — VantaFile has no batch commit range.**
The file cursor advances per node (`write_node_to_vstore`); there is no
per-transaction cursor watermark or segment-level versioning, so "roll back
the writes of txn T" is not expressible. Tombstoning is the only tool, and it
requires knowing the offsets (recoverable from HNSW `storage_offset` for
applied nodes).

**F5 — HNSW physical remove is not reversible.**
Deleting a node removes it from the DashMap immediately (ops.rs:1552) with no
retained copy of its vector/neighbor lists. Compensating an aborted *delete*
would need the pre-image, which is only available if captured before removal.

**F6 — Abort is trivially correct for *buffered* writes.**
Because buffered ops never reach stores pre-commit, `abort_transaction`
(drop buffer + WAL `Abort`) is clean. All rollback complexity is in the
**commit window** (F1/F4/F5) and the **derived-index window** (F3).

### 3.3 What already works (do not re-architect)

- WAL record framing, CRC32C, `recover_valid_records` truncation, two-pass
  skip-mask recovery, checkpoint_seq.
- Buffered write transactions + `active_txns`/`txn_buffers` + first-writer-wins
  conflict detection (ops.rs:229-294).
- MVCC visibility rule in `Snapshot` (Phase 3) and eager-on-commit GC.
- Per-op tombstone compensation for KV failure (P4).
- Backend abstraction (`StorageBackend`, `write_batch` for Fjall/RocksDB).

## 4. Two-Phase Rollback Protocol

### 4.1 Principle

**The WAL is the single commit arbiter.** Every transaction goes through one
durable "prepare" point and one durable "commit" point; every other layer
(VantaFile, KV, HNSW, derived) is a *materialization* of WAL state and must be
either (a) idempotently re-materializable from the WAL on recovery, or
(b) immediately compensatable at runtime with the transaction's own buffer as
the undo log.

New WAL record (format v2):

```rust
WalRecord::Prepare(u64) // txn_id — durable "committable" marker
```

### 4.2 Sequence

```
Thread                        WAL (durable)                Stores (VantaFile / KV / HNSW / derived)
──────                        ────────────                 ────────────────────────────────────────
begin_transaction()   ───────► (buffer only; Begin is lazy)   [no store I/O]
insert_in_txn() / delete_in_txn() → txn_buffers[txn_id]      [no store I/O]
  ...
commit_transaction():
  1. batch_append [Begin(t), ops...] ── fsync ──► PREPARE POINT (durable, committable)
  2. apply ops in order:
       VantaFile (append, record offsets in buffer)
       KV backend (write_batch; keep pre-image of prev NodeMetadata)
       HNSW (under insert_lock; keep undo data from buffer)
       derived indexes (edge/scalar/cardinality)
  3. ── if any step fails ──► append [Abort(t)] + run runtime compensation (4.4)
  4. batch_append [Commit(t)] ── fsync ──► COMMIT POINT (durable, done)

abort_transaction():
  drop buffer; append [Abort(t)]                      [no store I/O — nothing was applied]
```

Step 1 is today's `batch_append` minus the trailing `Commit`. Step 4 is a new
single-record append after the stores are applied. `Begin` may also be written
at `begin_transaction()` time for observability; it is not required for
correctness because step 1 is the first durable artifact.

### 4.3 Recovery semantics (init.rs skip-mask v2)

| WAL tail observed | Current behavior (v1) | New behavior (v2) | Result |
|---|---|---|---|
| `Begin` … no `Commit`/`Abort` | skip (`skip_mask[start..]`) | **skip** (no `Prepare` → never committable) | aborted ✅ |
| `Begin` … `Abort` | skip | skip | aborted ✅ |
| `Begin` … `Prepare` … no `Commit` | n/a (no Prepare) | **roll forward** (idempotent re-apply with the txn's `txn_id` as `created_by_txn`) | committed ✅ |
| `Begin` … `Prepare` … `Commit` | replay | replay | committed ✅ |
| bare ops / legacy v1 WAL | replay | replay unchanged | backward compatible ✅ |

The one semantic change: a transaction with a durable `Prepare` is
**rolled forward** on recovery, whereas today an unclosed `Begin` is skipped.
This is safe because `Prepare` is only written by v2 code, and v1 WALs never
contain it — so old files keep today's behavior exactly.

**Crash windows (in order of risk):**

| Window | Where the crash lands | Durable WAL | Recovery action | Consistency |
|---|---|---|---|---|
| W1 | before step 1 completes | nothing / partial shard | skip (no Prepare) | ✅ txn not durable |
| W2 | mid-apply (VantaFile or KV or HNSW partially materialized) | `Begin+ops+Prepare` | roll forward (idempotent) | ✅ full apply |
| W3 | between step 3 and step 4 | `Begin+ops+Prepare` | roll forward | ✅ |
| W4 | after `Commit` | full | replay as committed | ✅ |
| W5 | after `Abort` | `Abort` | skip | ✅ |

W2 is the critical one: store writes between `Prepare` and `Commit` are
allowed to be partial because replay re-applies them idempotently. Each apply
step must therefore be **idempotent** (see 4.4), which the current per-op
paths mostly are (vstore append + KV put + HNSW add for the same node id).

### 4.4 Per-layer compensations (runtime abort, no crash)

When apply fails (step 3), append `Abort` **first** (durable verdict), then
invert whatever was already applied, using the txn buffer as the undo log:

| Layer | Forward op | Compensation on abort | Cost / notes |
|---|---|---|---|
| **VantaFile** | append header+vector at cursor, advance cursor | tombstone the header at the recorded offset (`FLAG_TOMBSTONE`); never rewind the cursor (other txns may have appended) | space reclaimed later by `compact_layout_bfs`; no cursor race |
| **KV backend** | `put` NodeMetadata (write_batch) | if pre-image existed → restore pre-image; else `delete(key)` | requires capturing pre-image at first touch of the key (F5 — add to buffer) |
| **HNSW** | `add` / `nodes.remove` under `insert_lock` | invert: `remove(id)` for inserts; re-`add(id, vector, offset)` for deletes (data kept in buffer) | deferred-apply it last so fewer inversions; rebuild-from-VantaFile is the ultimate fallback |
| **Derived (edge/scalar/cardinality)** | mutate per op | remove/restore entries for the txn's node ids | see Phase 4d — ideally deferred until after `Commit` so no compensation is ever needed |

**Ordering rule:** apply in increasing compensation cost (VantaFile first —
cheap tombstone; derived last — either deferred past Commit or never-touched
until Commit). HNSW before derived so its undo data is available from the
buffer.

### 4.5 Interaction with MVCC Phase 3

1. **Preserve stamps through recovery (fixes F2):** `replay_write_node` gains
   a `txn_id` parameter. Recovery tracks the current txn id from `Begin` /
   `Prepare` and stamps `created_by_txn` accordingly; bare (non-txn) records
   keep the pseudo-txn `next_txn_id` semantics used by non-txn `apply_insert`.
   Without this, post-restart snapshots see aborted-window writes and Phase 3
   guarantees are void after every restart.
2. **Abort vs delete stamps:** `stamp_deleted_in_backend` only runs at commit.
   With the new ordering, an aborted delete never stamps — consistent. If a
   prepared txn partially applied a delete and then aborts, compensation
   restores `deleted_by_txn: None` from the pre-image.
3. **Single-version limitation (documented, not fixed in Phase 4a):** KV keeps
   one version per key; an update overwrites the previous `NodeMetadata`, so a
   snapshot taken before that update cannot see the old version. True
   multi-version chains are out of scope; Phase 4b preserves the *pre-image*
   only long enough for abort compensation, not for historical reads.
4. **GC safety:** `gc_mvcc_versions` must never reclaim a version whose
   `deleted_by_txn` could still be read by an active or *prepared* txn's
   snapshot. `safe_cutoff` must be the minimum of active/prepared txn ids, not
   just `next_txn_id` (today's default at ops.rs:80-82).

### 4.6 Non-transactional fast path

Non-txn `insert()` / `delete()` (the hot path, ops.rs:542 / 1443) route
through the same WAL-first ordering with a single-op implicit transaction:
`[op]` prepare → apply → `[Commit]`. On apply error, append `Abort` and run
the single-op compensation. This upgrades the P4 per-op tombstone to a
complete protocol at negligible cost (one extra record append only on the
commit step); `vanta-tuner` should validate the delta against
`SyncMode::Never/Periodic`.

## 5. Approach Evaluation (recovered research vs. current implementation)

The original research recommended **Approach B**; Phase 1–3 implemented it.
INV-010 re-evaluates the three approaches against the *current* codebase, with
the rollback requirement in scope:

| Criterion | A — Fjall-only | B — Custom WAL layer (+ INV-010) | C — SQLite journal |
|---|---|---|---|
| Covers VantaFile/HNSW | ❌ | ✅ | ⚠️ VantaFile only |
| Rollback across layers | ❌ (KV-only) | ✅ (Prepare/Commit + compensation) | ⚠️ mmap page-restore |
| Reuses existing WAL/CRC/recovery | — | ✅ (v1 → v2, backward-compatible) | ❌ second mechanism |
| Crash atomicity | ✅ KV-only | ✅ | ⚠️ crash mid-`mprotect` |
| MVCC integration | ❌ none | ✅ stamps preserved through recovery | ❌ none |
| Implementation cost | ~0 (already used) | ~2–3 phases (4a–4d) | high (mmap + new file format) |
| Regression risk | low | medium (recovery semantics) | high |

**Decision: keep Approach B, complete its deferred rollback half.** The
current implementation already validates Approach B's core bet (WAL as commit
arbiter + buffered writes + snapshot isolation all shipped and tested). The
remaining gaps are exactly the ones Approach B's original doc deferred with a
one-line assumption ("rolled back by skipping it during WAL replay") that only
holds for *crash* rollback, not *runtime* rollback. INV-010 adds the `Prepare`
record, the commit-point reorder, and the compensation table — no new
dependency, no new storage format beyond one additive WAL variant, and the
recovered research's verdict stands.

## 6. Implementation Plan (Phases 4a–4d)

Each phase is independently shippable; `cargo check -p vantadb`, the existing
577-test suite (1 pre-existing failure), and `cargo fmt --check` must stay
green after each.

### Phase 4a — WAL `Prepare` + commit-point reorder

**Goal:** runtime apply failures become compensable; prepared txns roll
forward on recovery; MVCC stamps survive recovery (fix F1, F2).

**Files:**
- `src/wal.rs` — add `WalRecord::Prepare(u64)`; `WAL_FORMAT_VERSION: u16 = 2`
  (header compat already accepts `≤ current`); keep v1 parse path.
- `src/storage/engine/ops.rs` — `commit_transaction`: `batch_append [Begin,
  ops...]` → apply → `batch_append [Commit]`; on apply error `batch_append
  [Abort]` + call `compensate_applied(txn_id)` (stub for 4b–4c). Route non-txn
  insert/delete through the same ordering.
- `src/storage/engine/init.rs` — skip-mask v2: unclosed `Begin` w/o `Prepare`
  → skip; `Begin+Prepare` w/o `Commit` → roll forward with txn id;
  `replay_write_node(..., txn_id)` stamps `created_by_txn`.
- `src/wal_sharded.rs` — verify `batch_append` grouping with the new record
  (no code change expected).

**Acceptance criteria:**
1. Unit/proptest: `Prepare` round-trips through `WalReader` with valid CRC.
2. Crash test (chaos): kill process between prepare and commit → on restart
   the txn is applied with `created_by_txn = t` (visible to snapshots ≥ t,
   invisible to snapshots < t).
3. Crash test: kill mid-apply → restart replays idempotently, no duplicate
   vstore entries, HNSW and KV consistent.
4. Error-injection (failpoints): force KV put failure after Prepare → `Abort`
   is written; on restart the txn is absent; caller receives an error that is
   now truthful.
5. Old v1 WAL (no `Prepare` records): replay identical to today (existing
   recovery tests pass unchanged).
6. `cargo check -p vantadb` green; existing suite green.

**Risk:** medium — recovery semantics change; mitigated by the
`Prepare`-presence gate (v1 files never trigger roll-forward) and chaos
coverage. **Durability:** unchanged (`sync_mode` still governs fsync).

### Phase 4b — KV MVCC compensation + pre-image capture

**Goal:** aborts restore the previous committed state (fix F5-for-KV, 4.4
compensation), GC is snapshot/prepare-safe.

**Files:**
- `src/storage/ops.rs` — `NodeMetadata` unchanged on disk; add buffer-side
  `PreImage { key, prev: Option<NodeMetadata> }` captured at first touch in
  `insert_in_txn` / `delete_in_txn` (or at apply time).
- `src/storage/engine/ops.rs` — `compensate_applied()`: for each applied op,
  restore `prev` or `backend.delete(key)`; for applied deletes restore
  `deleted_by_txn: None` from pre-image.
- `src/storage/engine/ops.rs` — `gc_mvcc_versions` cutoff = min of active +
  prepared txn ids.

**Acceptance criteria:**
1. Test: begin txn, update node A, abort → node A has its pre-txn value;
   snapshot created before the txn reads the old value.
2. Test: begin txn, delete node B, abort → node B alive, `deleted_by_txn`
   None, HNSW/vstore untouched.
3. Test: GC with an active/prepared txn does not reclaim versions its
   snapshot could reach.
4. Chaos: inject KV failure mid-apply → compensation restores pre-images; WAL
   replay on restart yields the same state.

**Risk:** low-medium (in-memory buffer growth per txn). **Durability:**
unchanged.

### Phase 4c — VantaFile batch watermark + HNSW commit protocol

**Goal:** express "writes of txn T" on the vector store and make HNSW
mutation per-txn atomic (fix F4, F5).

**Files:**
- `src/storage/vfile.rs` — add per-txn write watermark (remember `write_cursor`
  at prepare; abort tombstones `[watermark..commit_cursor)` range) OR segment
  append-per-txn; keep `save_cursor` semantics for crash (watermark is
  in-memory; WAL owns durability).
- `src/index/graph.rs` / `src/storage/engine/mod.rs` — HNSW apply/undo under
  the existing `insert_lock`: keep `(id, vector, storage_offset)` for deletes
  in the txn buffer until Commit; compensate by re-add/remove.
- `src/storage/engine/maintenance.rs` — `compact_layout_bfs` / vacuum reclaim
  tombstoned ranges from aborted txns.

**Acceptance criteria:**
1. Test: abort a prepared txn after vstore append → all its slots
   tombstoned, cursor not rewound, subsequent txns append after it.
2. Test: abort a prepared txn after HNSW add → node removed from HNSW and
   from KV; no orphan vector readable by search.
3. Test: abort a prepared txn after HNSW remove (delete op) → node re-added
   with original vector/offset; search finds it.
4. Compaction test: aborted txn space is reclaimed and `fresh_hnsw` /
   `rebuild_vector_index` produce identical results with/without aborts.
5. Concurrency test: parallel txns on different nodes with one abort do not
   corrupt the vstore cursor or neighbor lists.

**Risk:** high (vstore cursor + HNSW graph invariants); requires chaos
coverage of W2 (crash mid-apply) with VantaFile/HNSW partially materialized.

### Phase 4d — Derived-index transactional consistency

**Goal:** eliminate stale edge/scalar/cardinality entries on abort (fix F3).

**Files:**
- `src/storage/engine/ops.rs` — move `edge_index` / `scalar_index` /
  cardinality-stats mutation from the eager pre-txn block into the apply step
  (after `Prepare`), or keep eager but add abort compensation.
- `src/scalar_index.rs`, `src/edge_index.rs` — add remove-by-node (exists for
  scalar) and batch undo helpers.

**Acceptance criteria:**
1. Test: txn inserts node with edges + fields, abort → `filter_field`,
   edge lookup, and cardinality stats show no trace of the node.
2. Test: txn deletes node, abort → `filter_field` / edge lookup still return
   the node.
3. Test: non-txn inserts/deletes unchanged (fast path equivalent).
4. Count-consistency check (per MUTATION_RECOVERY_PROTOCOL) passes after any
   abort sequence.

**Risk:** low-medium. **Durability:** unchanged (derived state is
per-process; rebuilt on restart per existing protocol).

### Rollout order & cross-cutting risks

- **4a is the keystone** (unblocks truthful errors + MVCC-safe recovery);
  land it with chaos coverage before 4b–4d.
- **WAL format bump coordination:** 4a alone bumps v1→v2. `vanta-lead` must
  set CI to test v1-file → v2-reader and v2-writer → v1-file rejection
  (or documented downgrade path). Old `WAL_FORMAT_VERSION` files stay readable
  because `validate_compat` accepts `≤ current`.
- **`vanta-audit`:** review the compensation paths (unsafe mmap writes on
  tombstone, pre-image postcard round-trips).
- **`vanta-chaos`:** W1–W5 crash-window matrix plus failpoint-injected
  failures at each apply step.
- **`vanta-tuner`:** measure non-txn fast-path delta (one extra record
  append) under `SyncMode::Never` vs `Periodic`.

## 7. ADR + Impact Analysis

### Architecture Decision

Adopt two-phase commit/rollback with the WAL as single commit arbiter:
`Begin+ops` batch = durable **prepare**; stores applied in compensation-cost
order with buffer-as-undo-log; `Commit` appended after apply, `Abort` on
failure. New `WalRecord::Prepare(u64)`; WAL format v2 (backward-compatible);
recovery rolls forward prepared-but-uncommitted txns, skips everything else.
Alternatives considered: Fjall-only (rejected, KV-only), SQLite journal
(rejected, mmap risk + duplicate WAL), "commit-first, no runtime
compensation" (current state — rejected due to the liar problem, F1).
Reference: recovered `docs/research/ACID_TRANSACTIONS.md` (restore pending).

### Impact Analysis

- **Modules affected:** `src/wal.rs`, `src/storage/engine/ops.rs`,
  `src/storage/engine/init.rs`, `src/storage/engine/mod.rs`,
  `src/storage/ops.rs`, `src/storage/vfile.rs`, `src/index/graph.rs`,
  `src/scalar_index.rs`, `src/edge_index.rs`, `src/storage/engine/maintenance.rs`.
- **Concurrency model:** unchanged primitives — `active_txns` /
  `txn_buffers` mutexes, `insert_lock` (FairMutex) for HNSW, per-store
  `RwLock`; no new locks; compensation runs under the same locks the forward
  ops used.
- **Durability guarantee:** unchanged (`sync_mode` governs fsync); the
  prepare point strengthens the guarantee — a returned `Err` now means "not
  durable", never "durable-but-claimed-failed".
- **Backward compatibility:** additive — new `WalRecord` variant, WAL format
  v2 header still `validate_compat`-readable from v1 files; SDK public API
  unchanged (`begin/commit/abort_transaction` signatures untouched);
  non-txn hot path behavior identical on success.
- **SDK:** no breaking change; no major version bump required (additive).

## Appendix — Key code references

| Symbol | Location | Relevance |
|---|---|---|
| `WalRecord::{Begin,Commit,Abort}` | `src/wal.rs:33-65` | v1 txn markers |
| `batch_append` | `src/wal.rs:286`, `src/wal_sharded.rs:86` | atomic commit point |
| `recover_valid_records` / skip-mask | `src/wal.rs:229`, `src/storage/engine/init.rs:478-503` | recovery v1 |
| `replay_write_node` (`created_by_txn: 0`) | `src/storage/engine/mod.rs:381-406` | F2 |
| `commit_transaction` | `src/storage/engine/ops.rs:300-365` | F1 |
| `abort_transaction` | `src/storage/engine/ops.rs:370-381` | clean abort |
| `apply_insert_with_txn` | `src/storage/engine/ops.rs:403-446` | MVCC stamp on insert |
| `stamp_deleted_in_backend` | `src/storage/engine/ops.rs:384-400` | MVCC stamp on delete |
| `delete` ACID note ("Phase 0") | `src/storage/engine/ops.rs:1437-1441` | acknowledged gap |
| eager edge/scalar updates | `src/storage/engine/ops.rs:619-630, 1472-1479` | F3 |
| `NodeMetadata` | `src/storage/ops.rs:10-21` | MVCC partial |
| `gc_mvcc_versions` | `src/storage/engine/ops.rs:80-109` | GC cutoff |
| `write_node_to_vstore` | `src/storage/ops.rs:24-60` | VantaFile append |
| VantaFile cursor persist | `src/storage/vfile.rs:412, 534-536` | F4 |
| HNSW remove / entry-point | `src/storage/engine/ops.rs:1523-1564`, `src/index/graph.rs` | F5 |
| ACID_TRANSACTIONS.md (recovered) | `git show 8b1c52cd^:docs/research/ACID_TRANSACTIONS.md` | research source |
