---
title: "Memory Telemetry — Schema and Validation"
type: operations
status: active
tags: [vantadb, operations, telemetry, memory]
last_reviewed: 2026-07-30
aliases: [memory_telemetry]
---

# Memory Telemetry — Schema and Validation

This document defines the VantaDB memory observability contract. It inventories
the metrics currently exposed, proposes a five-category schema that separates
core RAM / index RAM / page cache / mmap / ingest, and specifies how to label
the Prometheus series without breaking the public `OperationalMetrics` contract.

**Status:** design proposal (INV-002). No source code was changed; the
implementation of the proposed sampler and labels is a separate follow-up task.

---

## 0. Reconciliations with History

- **DISC-05 "Fix telemetría de memoria (~225 GB falsos en 34 GB)"** — marked ✅
  but pending formal re-verification. Root cause of the false ~225 GB report:
  heterogeneous numbers were summed as if they were one partition of RSS
  (logical estimates + physical residency + OS page cache). This proposal
  replaces that mental model: **each category is an orthogonal view with its
  own source and units**; the only aggregate number is OS-reported RSS. Never
  sum category values to claim a "memory footprint".
- The current `memory_telemetry_contract` test (`tests/memory_telemetry.rs`)
  already enforces that telemetry is process-scoped and that on-disk bytes are
  not reported as process memory. That test is preserved and remains the
  verification harness for this contract.

---

## 1. Current Metric Inventory

All gauges live in `src/metrics/core/registry.rs`, are feature-gated behind
`feature = "prometheus"` (`prometheus` crate v0.14), and are backed by the
atomic snapshots in `src/metrics/core/mod.rs`.

| Metric (gauge) | Source | Feature gate | What it physically measures | Proposed category |
| :--- | :--- | :--- | :--- | :--- |
| `vanta_process_rss_bytes` | `_get_rss_virt()` → `get_native_memory()` (PSAPI `QueryWorkingSetEx` on Windows) → fallback `sysinfo::Process::memory()` | prometheus | OS resident set of the whole process — contains heap, index, page cache, mmap and ingest all mixed together | **aggregate (RSS)** |
| `vanta_process_virtual_bytes` | same as above (`virtual_memory`) | prometheus | OS virtual memory of the process | **aggregate (VSZ)** |
| `vanta_hnsw_nodes_count` | `hnsw.nodes.len()` (maintenance.rs) | prometheus | HNSW node count | index (count) |
| `vanta_hnsw_logical_bytes` | `CPIndex::estimate_memory_bytes()` (`src/index/graph.rs:396`) | prometheus | Logical estimate: vector data (f32/binary/turbo/SQ8) + `HnswNode` size + 60 B/node + neighbor-index metadata | index (logical) |
| `vanta_mmap_resident_bytes` | `VantaFile::mmap_resident_bytes()` → `get_resident_bytes_impl` (`src/storage/vfile.rs:224`, `mincore`/`QueryWorkingSetEx`) | prometheus | Resident pages of mmap-backed files (vector stores + HNSW backend), summed in maintenance.rs | mmap (physical) |
| `vanta_volatile_cache_entries` | `volatile_cache.len()` | prometheus | Entries in the volatile hot-node LRU cache | page_cache |
| `vanta_volatile_cache_cap_bytes` | **hardcoded `0`** — maintenance.rs passes `0` for `cache_cap_bytes` | prometheus | Intended capacity of the volatile cache; currently always 0 (broken input) | page_cache |
| `vanta_jemalloc_allocated_bytes` | `tikv-jemalloc-ctl` (`stats::allocated`) | prometheus + jemalloc (linux/macos only) | Allocator-level allocated bytes — spans core heap + index + page cache + ingest buffers | allocator (cross-cutting) |
| `vanta_jemalloc_active_bytes` | `stats::active` | prometheus + jemalloc | Active pages in allocator | allocator (cross-cutting) |
| `vanta_jemalloc_metadata_bytes` | `stats::metadata` | prometheus + jemalloc | Allocator metadata overhead | allocator (cross-cutting) |
| `vanta_jemalloc_resident_bytes` | `stats::resident` | prometheus + jemalloc | Resident pages in allocator | allocator (cross-cutting) |
| `vanta_jemalloc_mapped_bytes` | `stats::mapped` | prometheus + jemalloc | Mapped address space | allocator (cross-cutting) |
| `vanta_jemalloc_retained_bytes` | `stats::retained` | prometheus + jemalloc | Retained (unmapped-but-reserved) pages | allocator (cross-cutting) |
| `vantadb_cache_warmer_*` (4 gauges) | `CacheWarmer::metrics()` (`src/cache_warmer.rs:172`) — `#[allow(dead_code)]`, sampler missing | prometheus | Co-access table size / events / prefetch hits | page_cache (aux) |

### Current Overlap Problem

`vanta_process_rss_bytes` is a single number that mixes every subsystem. There
is no way to answer "how much of RSS is the HNSW index?" or "how much is the
volatile cache?" from the exported series alone. Additionally:

1. `VOLATILE_CACHE_CAP_BYTES` is always exported as 0 (hardcoded argument at
   `src/storage/engine/maintenance.rs:115`).
2. `record_memory_breakdown` is only invoked from `flush()` — there is **no
   periodic sampler**, so all memory gauges go stale between flushes.
3. `MemoryGovernor::used_bytes` (`src/memory_governor.rs:16`) is updated from
   `check_memory_pressure()` (`src/storage/engine/stats.rs:150`) but is **not
   exported** as a gauge.
4. `CacheWarmer::metrics()` is dead code (`#[allow(dead_code)]`) — the
   co-access telemetry is never pushed.

---

## 2. Proposed Schema — Five Categories

Each category is an **orthogonal view** with its own measurement source. They
are not a partition of RSS; see the summation invariant in §5.

### 2.1 core

- **Definition:** process heap outside the index, page cache and ingest
  accounting: engine scaffolding, derived indexes (`NamespaceIndex`,
  `PayloadIndex`, `TextIndex`/BM25 postings), KV backend buffers (Fjall /
  RocksDB), WAL buffers, governor state.
- **Formula:** `core = rss − index − page_cache − mmap − ingest` (derived
  residual, only when the other four sources are present), or
  `core = jemalloc_allocated − index_logical − ingest` when the `jemalloc`
  feature is active and index/cache allocations are tracked.
- **Excludes:** anything already counted by another category (no double
  counting) and OS kernel page cache (not addressable by the process).
- **Source:** derived; no direct gauge today.

### 2.2 index

- **Definition:** logical footprint of the HNSW graph and its neighbor index —
  exactly what `CPIndex::estimate_memory_bytes()` computes.
- **Formula:** `index = Σ vec_data + Σ HnswNode + 60 B × total_nodes + neighbor_index_meta`
- **Includes:** full f32 vectors, binary, turbo, SQ8 (decompressed) variants,
  node structs, per-node neighbor metadata.
- **Excludes:** physical residency of mmap-backed vector files (that is `mmap`),
  volatile cache entries (`page_cache`).
- **Source:** `vanta_hnsw_logical_bytes` (existing), `vanta_hnsw_nodes_count`
  (existing), `vanta_volatile_cache_*` (existing but cap is broken).

### 2.3 page_cache

- **Definition:** the engine's volatile hot-node LRU cache plus the cache
  warmer co-access table. (This is **not** the OS kernel page cache — kernel
  page cache is not observable by the process and is explicitly out of scope.)
- **Formula:** `page_cache = Σ volatile_cache_entries × bytes_per_entry + cache_warmer_co_access`
- **Source:** `vanta_volatile_cache_entries` (existing),
  `vanta_volatile_cache_cap_bytes` (existing, needs fix), `vantadb_cache_warmer_*`
  (existing gauges, sampler missing).

### 2.4 mmap

- **Definition:** resident (in-RAM) pages of memory-mapped files — vector
  stores (`vstore_L*.vanta`) and the HNSW backend file.
- **Formula:** `mmap = Σ VantaFile::mmap_resident_bytes()` over
  `vector_store[]` + `hnsw.backend`
- **Source:** `vanta_mmap_resident_bytes` (existing, via
  `mincore`/`QueryWorkingSetEx`), already summed in
  `get_memory_stats()` (`src/storage/engine/stats.rs:64-76`).

### 2.5 ingest

- **Definition:** bytes accounted by `MemoryGovernor::used_bytes` — the
  ingestion accounting trail (allocated/freed tracking) plus in-flight
  ingestion buffers (`src/ingestion.rs`), backpressure queues and the
  `GraphAccumulator`.
- **Formula:** `ingest = MemoryGovernor::used_bytes()`
- **Source:** `MemoryGovernor` — atomic counter exists but is **not exported**
  as a gauge (gap).

---

## 3. Structure → Category Mapping

| Structure | Category | Gauge exists? | Value source |
| :--- | :--- | :--- | :--- |
| `CPIndex` (HNSW nodes, vec_data, neighbor_index) | index | ✅ `vanta_hnsw_logical_bytes`, `vanta_hnsw_nodes_count` | `estimate_memory_bytes()`, `nodes.len()` |
| Volatile LRU cache (`volatile_cache`) | page_cache | ✅ entries; ⚠️ cap broken (0) | `len()`, capacity field |
| `CacheWarmer` co-access table | page_cache | ✅ `vantadb_cache_warmer_*` (sampler missing) | `metrics()` (dead code) |
| `VantaFile` mmaps (vector_store L0-L3, vector_index.bin, hnsw backend) | mmap | ✅ `vanta_mmap_resident_bytes` | `get_resident_bytes_impl()` (mincore / QueryWorkingSetEx) |
| `MemoryGovernor` (`used_bytes`, watermarks) | ingest | ❌ no gauge | `used_bytes()` atomic |
| `GraphAccumulator` (DashMap) | ingest | ❌ | internal map |
| `StorageEngine` scaffolding, derived indexes, text/BM25 postings, KV backend buffers, WAL | core | ❌ (derived residual) | RSS minus other categories / jemalloc residual |
| OS-reported RSS / VSZ | aggregate | ✅ `vanta_process_rss_bytes` / `vanta_process_virtual_bytes` | `get_native_memory()` → sysinfo fallback |
| jemalloc stats (allocated/active/metadata/resident/mapped/retained) | allocator (cross-cutting) | ✅ `vanta_jemalloc_*` | `tikv-jemalloc-ctl` |

**Gauges that need a new sampler (phase 2):** `MemoryGovernor` → ingest gauge;
`CacheWarmer::metrics()` → wire into the periodic sampler; `VOLATILE_CACHE_CAP_BYTES`
→ stop passing hardcoded 0.

---

## 4. Label Proposal (validated against official docs)

**Decision: `IntGaugeVec` with a fixed `category` label — not the `metrics` /
`metrics-tracing` ecosystem.**

Validation (2026-07-30):

- The workspace depends on the `prometheus` crate directly (`Cargo.toml`:
  `prometheus = { version = "0.14", optional = true }`). The backlog's
  suggestion of `tracing::metrics` does not match the installed stack; adopting
  the `metrics`/`metrics-exporter-prometheus` ecosystem would add a second
  telemetry pipeline for no benefit.
- Official API confirmed from the crate source and docs
  (tikv/rust-prometheus, `src/gauge.rs`, docs.rs `IntGaugeVec`):
  - `IntGaugeVec` is `GenericGaugeVec<AtomicI64>` — "the integer version of
    GaugeVec. Provides better performance if metric values are all integers."
  - `IntGaugeVec::new(opts: Opts, label_names: &[&str]) -> Result<Self>` —
    "At least one label name must be provided."
  - `with_label_values(&["core"]).set(value)` — label accessor + set.
  - The workspace already uses this pattern for HTTP metrics:
    `HistogramVec::with_label_values(&[method, route])` in
    `src/metrics/core/registry.rs:795`.
- Prometheus cardinality guidance (official Prometheus docs, "Avoid High
  Cardinality") is satisfied: the `category` label has a fixed, bounded value
  set of 5.

### Proposed metric

```rust
// phase 2 sketch — registry.rs
pub static MEMORY_BY_CATEGORY: LazyLock<Option<IntGaugeVec>> = LazyLock::new(|| {
    let vec = match IntGaugeVec::new(
        prometheus::Opts::new(
            "vanta_memory_bytes",
            "VantaDB memory usage broken down by category (core|index|page_cache|mmap|ingest)",
        ),
        &["category"],
    ) {
        Ok(v) => v,
        Err(e) => { tracing::warn!("Failed to create MEMORY_BY_CATEGORY: {e}"); return None; }
    };
    match METRICS_REGISTRY.register(Box::new(vec.clone())) {
        Ok(_) => Some(vec),
        Err(e) => { tracing::warn!("Failed to register MEMORY_BY_CATEGORY: {e}"); None }
    }
});
```

```rust
// sampler sketch — set once per category
let g = MEMORY_BY_CATEGORY.as_ref()?;
g.with_label_values(&["core"]).set(core as i64);
g.with_label_values(&["index"]).set(index as i64);
g.with_label_values(&["page_cache"]).set(page_cache as i64);
g.with_label_values(&["mmap"]).set(mmap as i64);
g.with_label_values(&["ingest"]).set(ingest as i64);
```

Rationale for `IntGaugeVec` over five separate gauges: one family per concept
(`vanta_memory_bytes{category="..."}`) keeps the PromQL ergonomic
(`sum by (category) (vanta_memory_bytes)`) and the registry small; separate
metrics would duplicate the same help text and break the naming convention of
the existing `vanta_*_bytes` family. Cardinality stays at exactly 5 series.

### Periodic sampler placement

The sampler must run from a **periodic tick in the engine's maintenance
cycle**, not from `flush()`:

- Hook: the existing maintenance tick that already calls
  `check_memory_pressure()` (`src/storage/engine/stats.rs`) and the PERF-10
  governor sync — extend it to call `record_memory_breakdown(...)` +
  `CacheWarmer::metrics()` on an interval (1–5 s recommended; the cache warmer
  note in `src/cache_warmer.rs:169` explicitly requests "a periodic metrics
  sampler ... from the engine's tick loop").
- The `record_memory_breakdown` signature should grow the new sources
  (governor `used_bytes`, cache cap, cache warmer metrics) rather than the
  current hardcoded `0` for cache cap.

### Public contract impact

- **Preserved:** `OperationalMetrics` in `vantadb-ts/src/types.ts` (lines
  122–160) and the Rust snapshot (`src/metrics/core/snapshot.rs`,
  `MemoryBreakdownSnapshot`) keep their flat fields
  (`process_rss_bytes`, `process_virtual_bytes`, `mmap_resident_bytes`,
  `jemalloc_*`, `hnsw_logical_bytes`). The `IntGaugeVec` is **additive** on the
  `/metrics` endpoint only; no binding changes.
- **Migration (only if a future task decides to expose categories in the JSON
  SDK):** add `memory_by_category?: Record<Category, string | null>` as a new
  optional field on `OperationalMetrics`; keep all existing fields for one
  release, then deprecate only fields that are superseded. No field is removed
  in this proposal.
- WASM/Python conversions (`vantadb-wasm/src/lib.rs:194-199`,
  `src/sdk/serialization/conversions.rs`) are unaffected in phase 2.

---

## 5. Summation Invariant

Categories are **orthogonal views**, not a partition of RSS. The only
meaningful aggregate identity is the OS-reported RSS:

```
process_rss_bytes  (OS)
  ⊇ core (derived residual)
  ⊇ index (logical estimate — may under/over-state physical residency)
  ⊇ page_cache (engine LRU; NOT kernel page cache)
  ⊇ mmap (physical resident pages of mapped files)
  ⊇ ingest (governor accounting)
```

A residual reconciliation is expected and **allowed**:

```
core ≈ rss − index − page_cache − mmap − ingest
```

with tolerance for:

- allocator metadata (`jemalloc_metadata_bytes`) and allocator internal
  fragmentation,
- OS kernel page cache and shared-library pages (not addressable by the
  process),
- index being a *logical* estimate while mmap/ingest are *physical* or
  *accounting* figures.

**Rule:** never present `core + index + page_cache + mmap + ingest` as a
"total memory footprint". That is exactly the error DISC-05 fixed. The five
series exist so operators can compare each view to the others and to RSS —
not to sum them.

---

## 6. Phase 2 Implementation Plan (follow-up task)

When this design is approved, the implementation task (e.g. `OBS-*` or a new
`INV-*`) should:

1. Add `MEMORY_BY_CATEGORY: IntGaugeVec` to `src/metrics/core/registry.rs`
   (sketch in §4).
2. Extend the maintenance tick sampler (`src/storage/engine/stats.rs` /
   `maintenance.rs`) to run on a 1–5 s interval, calling
   `record_memory_breakdown` with real cache cap and new sources.
3. Export `MemoryGovernor::used_bytes()` as the `ingest` category and remove
   the hardcoded `cache_cap_bytes = 0`.
4. Wire `CacheWarmer::metrics()` into the sampler (removes
   `#[allow(dead_code)]`).
5. Derive `core` as the residual from RSS minus the other four categories
   (fall back to jemalloc residual when available).
6. Re-verify DISC-05: `cargo test --test memory_telemetry -- --nocapture`
   plus a manual PromQL check that the five series are individually
   monotone-stable under ingest and eviction.
7. No changes to `vantadb-ts/src/types.ts` unless the additive
   `memory_by_category` field is explicitly requested.

---

## 7. Memory Telemetry Verification

To run local stability measurements and profile memory consumption under
continuous inserts, execute the test harness:

```powershell
# Set report file path
$env:VANTA_CERT_REPORT="target/memory_telemetry.json"
cargo test --test memory_telemetry -- --nocapture
```

This validates that RSS memory does not leak and that MMap pages are released
correctly when the index is flushed to disk.

Additional verification for the phase 2 sampler:

```powershell
# Ensure the labeled series are exported
cargo run --features server --bin vantadb-server   # then: curl localhost:PORT/metrics | Select-String vanta_memory_bytes
```

---

## 8. Related Documents

- `docs/operations/PERFORMANCE_TUNING.md` — §2 Memory Limits (governor, hot/cold
  tiering, quantization).
- `docs/operations/CONFIGURATION.md` — §6 Memory Telemetry Caveat.
- `docs/architecture/ARCHITECTURE.md` — memory telemetry contract reference.
- `tests/memory_telemetry.rs` — contract harness.
