---
title: "RES-01 - ACID Phase 4a: WAL v2 with WalRecord::Prepare"
date: 2026-08-25
status: DRAFT - research output, pending human ADR (Regla 5)
task: RES-01 / docs/plans/2026-08-25-batch-core-fixes-research.md
---

# RES-01: WAL v2 with `WalRecord::Prepare` ΓÇö Analysis and Implementation Plan

## 1. Question

Should VantaDB move from the current single-phase commit point (`[Begin + ops + Commit]`
written as one atomic WAL batch) to a two-phase commit with an explicit
`WalRecord::Prepare`? What does it gain (multi-layer rollback, truthful errors, MVCC
base) and what does it cost (extra fsync per commit, recovery complexity, format v2)?

## 2. Current state (verified file:line)

- **`WalRecord` enum** ΓÇö `src/wal.rs:40-73`: `Insert(UnifiedNode)`, `Update{id,node}`,
  `Delete{id}`, `Checkpoint{node_count,index_checksum,timestamp}`, `Begin(u64)`,
  `Commit(u64)`, `Abort(u64)`. No Prepare variant.
- **Format version** ΓÇö `WAL_FORMAT_VERSION = 1` (`src/wal.rs:12`);
  `WAL_POSTCARD_VERSION = 1` (`src/wal.rs:17`). Header is a 20-byte `VantaHeader`
  (magic `VWAL`) + CRC32C (`src/wal.rs:79-126`). Record framing:
  `[len:u32 LE][postcard payload][crc32c:u32 LE]` (`src/wal.rs:182-184`).
- **Version compatibility** ΓÇö range-based: header accepts any `format_version <= current`
  (`src/wal.rs:140-152`). Downgrade (new format read by old binary) is NOT supported.
- **Commit point today** ΓÇö `StorageEngine::commit_transaction`
  (`src/storage/engine/txn.rs:119-191`):
  1. Unregister txn from `active_txns`, drain `txn_buffers` (`txn.rs:121-136`).
  2. Build one batch `[Begin(txn), ops..., Commit(txn)]` (`txn.rs:146-155`).
  3. `ShardedWal::batch_append(wal_records)` ΓÇö one `write_all` + at most one
     `sync_data` per shard (`src/wal_sharded.rs:215-235`; `WalWriter::batch_append`
     `src/wal.rs:297-335`; sync policy `src/wal.rs:338-363`,
     `DEFAULT_PERIODIC_THRESHOLD = 1` so every append syncs by default,
     `src/wal.rs:340-355`).
  4. Apply buffered ops to stores with MVCC stamps (`txn.rs:162-188`;
     `apply_insert_with_txn` `txn.rs:233-285`, non-transactional apply with a
     best-effort tombstone fix-up on KV failure `txn.rs:255-269`).

  **The commit point is the durability of step 3**: if the process crashes during the
  append, the tail of the batch (including `Commit`) is lost, and recovery discards the
  incomplete prefix.
- **Crash-atomic replay (MOD-02, resolved)** ΓÇö `src/storage/engine/init.rs:505-551`:
  records are re-ordered by global round-robin seq (`init.rs:464-504`), then a
  skip-mask discards any `[Begin..]` prefix whose matching `Commit` never became
  durable; `Abort` closes its own extent; a trailing open batch at EOF is discarded
  fail-safe. Lesson ref: `.opencode/task-system/memory/lessons.md:150`.
- **Prior decisions** ΓÇö ADR DRV-014
  (`docs/architecture/adr/DRV-014-wal-batch-tradeoff.md`): the clone of records for
  shard-grouped `batch_append` was deliberately re-introduced in `cae92db3` for a
  measured 3-5x WAL write speedup; do not "fix" it back. Related roadmap:
  `docs/architecture/adr/DRV-015-wal-async-roadmap.md` (async/group-commit direction).

## 3. Known gap that motivates Prepare: errors are not truthful

If step 4 (apply) fails after step 3 already made `Commit` durable:

- `commit_transaction` returns `Err` to the caller (`txn.rs:163-188` propagates).
- But the txn buffer was already dropped (`txn.rs:133-136`) and the WAL holds
  `Commit` ΓÇö on restart, replay re-applies ALL ops (`init.rs:559-599`).
- Net effect: the caller was told the commit failed, yet the data resurrects.

This violates truthful-error semantics and is fixable independently of WAL v2
(routed to backlog as FIND row, see section 8).

## 4. Proposed design

### 4.1 Format (v2)

```rust
pub const WAL_FORMAT_VERSION: u16 = 2;

/// Phase-1 marker: ops for this txn are durable but not yet committed.
Prepare {
    /// Transaction being prepared.
    txn_id: u64,
    /// Number of ops between Begin and Prepare (integrity cross-check).
    op_count: u32,
}
```

Framing, header layout, CRC scheme, sharding and rotation are unchanged. Only the
enum grows. Migration:

- v2 binary reading v1 WAL: trivially fine (no `Prepare` present); range-based compat
  already accepts it (`wal.rs:140-152`).
- v1 binary reading v2 WAL: unknown postcard tag fails deserialization; scan-forward
  skips affected records. Documented rule: downgrade requires dump/restore (same hint
  as today, `wal.rs:150`).

### 4.2 Commit flow (two-phase)

```
commit_transaction(txn):
  1. drain buffer (unchanged)
  2. batch_append([Begin, ops..., Prepare{txn}]) + sync     // phase 1, durable
  3. apply ops to stores                                     // non-transactional
     on error: append Abort(txn) + sync, return Err          // TRUTHFUL error;
                                                               // replay will discard
  4. batch_append([Commit(txn)]) + sync                      // phase 2 = COMMIT POINT
  return Ok
```

### 4.3 Recovery

Prepared txns may interleave with other txns' records between phase 1 and phase 2
(two separate `batch_append` calls land at different global-seq positions). The MOD-02
slice-based skip-mask assumes contiguous extents and cannot express this. Recovery v2:

- Two-pass replay (recommended, bounded memory):
  - Pass 1: scan all shards, build `HashMap<u64, TxnOutcome>` from
    `Begin/Prepare/Commit/Abort` markers only (no payload buffering).
  - Pass 2: replay ops of txns whose outcome is `Committed`; discard
    `Open`/`Prepared`/`Aborted`.
- Keep the MOD-02 path for v1 records ΓÇö mixed WALs (v1 tail + v2 head) must work
  during migration: markers are disjoint, both passes coexist keyed by outcome map.

### 4.4 MVCC base

Persist `max_committed_txn` in `InternalMetadata` at checkpoint (alongside
`checkpoint_seq`, `init.rs:421-424`). Snapshot visibility
(`get_with_snapshot`, `txn.rs:292-398`, filter `txn.rs:313-322`) gains a committed
watermark so data stamped by an uncommitted/aborted txn_id is never visible even if
physically present ΓÇö the prerequisite for stable snapshots under concurrent apply and
for future multi-version retention.

## 5. What Prepare gains

1. **Truthful errors**: an apply failure becomes a real rollback (Abort after Prepare);
   the caller's `Err` matches what recovery will do.
2. **Multi-layer rollback**: abort becomes possible after durability (phase 1),
   enabling higher layers (engine, bindings) to roll back a durable-but-uncommitted txn.
3. **MVCC base**: committed watermark decouples visibility from physical apply order.
4. **Group-commit enabler**: phases can be pipelined later (aligns with DRV-015 async
   roadmap) to amortize fsyncs across concurrent txns.

## 6. Costs and risks

| Cost | Magnitude | Mitigation |
|---|---|---|
| Extra fsync per commit (2 instead of 1; today every append already syncs, `wal.rs:340-355`) | up to ~2x commit latency on fsync-bound paths | measure first (Regla 9, canonical_p99); group commit later |
| Recovery rewrite (outcome map replaces slice mask) | medium; must handle mixed v1/v2 | two-pass design; keep MOD-02 path |
| Prepared-op interleaving breaks contiguous-slot assumption | inherent | pass-1 outcome map (section 4.3) |
| Format v2: downgrade unsupported | low (dump/restore documented) | docs + version gate |
| Memory during replay pass 1 | O(markers) only | two-pass, no payload buffering |
| Scope creep toward full 2PC/coordinator | real risk | this design has NO lock phase, NO participant set ΓÇö it is a redo-log prepare, nothing more |

## 7. Interaction with MOD-02

**Complement, not replace.** MOD-02 solved crash-atomicity for the *single-batch*
commit model and its skip-mask logic remains correct for v1 records. WAL v2 supersedes
the slice-based mechanism with an outcome map because Prepare/Commit phases are no
longer contiguous ΓÇö but the invariant MOD-02 established (an uncommitted txn's ops are
never applied) is preserved and extended (now also covers post-prepare apply failures).

## 8. Findings routed to backlog

| Row | Type | Content |
|---|---|---|
| FIND (new) | bug | Truthful-error gap: apply failure after durable Commit resurrects ops on restart (`txn.rs:133-190` + replay `init.rs:559-599`). Independently fixable: on apply failure write `Abort(txn)` and make replay honor Abort-after-Commit, or defer buffer drop until apply succeeds. |
| ADR (Regla 5) | decision | WAL v2 Prepare tradeoff: accept +1 fsync/commit in exchange for truthful errors + rollback + MVCC base. Owner must articulate context/decision/consequences; this doc provides evidence only. |

## 9. Implementation plan (atomic steps)

- **S1** ΓÇö Add `Prepare { txn_id, op_count }` variant; bump `WAL_FORMAT_VERSION` to 2;
  roundtrip + mixed-version unit tests. Verify: `cargo nextest run -p vantadb wal`.
- **S2** ΓÇö Two-phase `commit_transaction` behind `config.wal_prepare: bool`
  (default `false`): flow per section 4.2; failpoint test mid-apply asserting
  Abort + Err + no resurrection on replay.
- **S3** ΓÇö Recovery v2 outcome-map (two-pass), coexisting with MOD-02 for v1 records;
  chaos tests via existing `failpoints` feature (`wal_append_fail`, `wal.rs:272-277`).
- **S4** ΓÇö Persist `max_committed_txn` at checkpoint; wire watermark into
  snapshot visibility; update `docs/api` durability docs (Doc-Driven Development).
- **S5** ΓÇö Bench before/after with flag on/off against `benches/canonical_p99.rs`
  (Regla 9); record in `docs/operations/BENCHMARKS.md`. Go/no-go data for owner.
- **S6** ΓÇö After owner ADR: flip default, close backlog rows, changelog via release-plz.

## 10. Recommendation

**Conditional GO.** The keystone value is real (truthful errors + multi-layer rollback +
MVCC base), and the architecture cost is contained (no new locks, no coordinator,
framing unchanged). But the +1 fsync per commit is a measurable latency tradeoff on the
hottest write path, and per Regla 5 that tradeoff belongs to the owner: implement S1-S2
behind a flag, produce S5 numbers, then let the human ADR decide default-on.
