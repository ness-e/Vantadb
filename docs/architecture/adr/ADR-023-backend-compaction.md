---
title: "ADR-023: Backend compaction/config validated against the real access pattern (small frequent writes + random reads)"
type: adr
status: accepted
tags: [vantadb, architecture, adr, storage, backend, fjall, rocksdb, compaction]
created: 2026-08-16
last_reviewed: 2026-08-16
related: [ADR-020-storage-backend-default.md, ADR-014-pitr.md]
---

# ADR-023: Backend compaction/config validated against the real access pattern

## Status

Accepted. Outcome of FND-08 (P20a): audit of backend compaction/memtable/cache
configuration against VantaDB's real access pattern, with the decision to
**keep current configuration and defer tuning** until a benchmark can justify it.

## Context

VantaDB's real access pattern is **small frequent writes + random reads**:

- Writes: per-node `put`/`write_batch` of `NodeMetadata` (relational fields,
  edges), derived indexes (`text_index` postings, `payload_index`,
  `namespace_index`, `sparse_index`), tombstones — all small values (< 1 KiB
  typically), many per second.
- Reads: point `get(id)` by 16-byte u128 key (`src/storage/engine/get.rs:79`),
  `get_many` (multi_get), and `scan_prefix` over derived indexes.

The question (FND-08): is the backend (Fjall default, RocksDB opt-in) tuned for
this pattern, or are we inheriting defaults aimed at bulk-load / sequential
write ingestion?

### Current configuration (verified, file:line)

**Fjall (default backend)** — `src/backends/fjall_backend.rs:54-57`:

- `Database::builder(path).open()` — **all builder defaults**.
- `KeyspaceCreateOptions::default` for all 9 partitions (`:60-93`).
- Defaults (validated against fjall 3.1.8 source, `docs.rs/fjall`):
  - `cache_size` = **32 MiB fixed** (`db_config.rs:90`) — NOT scaled to RAM.
  - `worker_threads` = `min(#cores, 4)` (`db_config.rs:70`).
  - `max_journaling_size` = 512 MiB (`db_config.rs:77`).
  - `manual_journal_persist` = false → write batches flush to the OS
    automatically (`db_config.rs:80`); VantaDB WAL (`wal_sharded`) handles
    durability above the backend.
  - Per keyspace: `max_memtable_size` = **64 MiB** (`keyspace/options.rs:91`),
    `data_block_size` = **4 KiB** (`:95`), leveled compaction default (`:124`).

**RocksDB (opt-in fallback)** — `src/backends/rocksdb_backend.rs:32-142`:

- Already tuned for point reads: bloom filter 10 bits (`:48`),
  `cache_index_and_filter_blocks` + `pin_l0_filter_and_index_blocks_in_cache`
  (`:50-51`), LRU block cache ~75% of 60% RAM budget (`:58-60`), LZ4 (`:44`),
  write buffer clamp 8–128 MiB / max 2 (`:62-65`), `max_background_jobs(4)`
  (`:43`), mmap reads/writes when RAM < 16 GiB (`:80-89`).
- **Not set**: `level_compaction_dynamic_level_bytes`, `target_file_size_base`,
  `max_bytes_for_level_base`, `level0_file_num_compaction_trigger`,
  `num_levels`, `optimize_for_point_lookup` — i.e. no level-sizing/compaction
  trigger tuning.

### Classification (FND-08 analysis)

| Option | Assessment |
|--------|-----------|
| Fjall block size 4 KiB | **ok** — fjall docs: "for point read heavy workloads a sensible default is 4–8 KiB". Aligned with random reads. |
| Fjall memtable 64 MiB | **ok** — absorbs small frequent writes; 8–64 MiB is the documented sweet spot. |
| Fjall worker threads min(cores,4) | **ok** — matches write volume; compaction is background. |
| Fjall journal 512 MiB / persist-to-OS | **ok** — VantaDB WAL owns durability; journal flush policy is conservative. |
| Fjall cache_size fixed 32 MiB | **marginal gap** — fixed regardless of RAM/working set. Fine while the metadata working set fits in 32 MiB; under-provisioned for large DBs with heavy random reads of payload/derived indexes. |
| RocksDB bloom + pin L0 + LRU cache | **ok** — exactly the right tools for random point reads. |
| RocksDB missing level tuning | **marginal gap** — no `level_compaction_dynamic_level_bytes`, no level trigger sizing; on a write-heavy mixed workload this raises write amplification vs. the tuned default. RocksDB is opt-in, not default. |
| RocksDB mmap when RAM < 16 GiB | **ok** — deliberate resource-governance tradeoff, documented. |

**No gap-real found**: neither backend is configured with bulk-load-oriented
defaults that penalize the real pattern. Fjall's defaults are already
point-read/write friendly; RocksDB's cache/bloom/pinning choices dominate the
workload correctly. The two marginal gaps only matter at scale (working set >
32 MiB, or RocksDB as primary with sustained mixed load).

## Decision

**Keep the current backend configuration. Defer both marginal tuning changes.**
No code change in this ADR.

Rationale:

1. **Regla 9 (no optimization without measurement):** the existing benches
   (`benches/backend_compare.rs`, 5k records; `benches/canonical_p99.rs`,
   vector-search focused) use datasets far below the 32 MiB Fjall cache, so a
   before/after bench of `cache_size` or level tuning would measure noise, not
   the gap. Changing configuration without a bench that can demonstrate the
   difference is speculation, not engineering.
2. **Working-set uncertainty:** VantaDB's KV backend stores metadata + derived
   indexes (vectors live in `VantaFile`/mmap, not the backend), so the real
   working set is not yet measured. Scaling Fjall's cache to 20–25% of RAM
   (fjall's own recommendation) without knowing the working set risks wasting
   memory or hiding the gap.
3. **Scope discipline:** FND-08 complements FND-02; changing `src/storage/`
   beyond configuration is out of scope, and configuration changes require the
   bench evidence above.
4. **RocksDB is opt-in:** it is not the default path (`Cargo.toml:97`); its
   missing level tuning is a documented fallback nuance, not a default-path
   defect.

## Consequences

- **Positive:** no behavior change; the default (Fjall) path remains
  point-read/write tuned; audit evidence is recorded (this ADR + report
  `docs/Investigaciones/FND-08-backend-compaction.md`).
- **Negative:** the two marginal gaps remain: Fjall `cache_size` fixed at
  32 MiB, and RocksDB level-tuning options unset.
- **Deferred (reopen signal):** apply tuning when (a) a bench with dataset
  > 32 MiB working set shows read-latency regression vs. baseline
  (`backend_compare.rs` random_get), or (b) RocksDB becomes a supported
  primary with measured write amplification. At that point: set
  `cache_size` from `effective_memory` (fjall recommends 20–25% RAM) and
  enable `level_compaction_dynamic_level_bytes(true)` in RocksDB.
- **New rule:** `.opencode/rules/durability.md` now requires backend
  configuration to be justified against the documented access pattern
  (small frequent writes + random reads); any future config change must
  carry a before/after bench against `backend_compare.rs`.

## Related

- ADR-020 — backend default (Fjall) decision record.
- `benches/backend_compare.rs` — random-read benchmark (reference for
  reopen signal).
- FND-08 report: `docs/Investigaciones/FND-08-backend-compaction.md`.