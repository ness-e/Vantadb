# Wave 1-7: Bugfixes + Optimizations — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix 3 HNSW bugs + AuthRateLimiter vulnerability + add AVX-512/SQ8 SIMD + Python SDK optimizations

**Architecture:** Two parallel tracks: (A) Bugfixes — 4 isolated/weakly-coupled fixes; (B) Optimizations — 5 independent performance improvements

**Tech Stack:** Rust, wide crate (SIMD), PyO3, lru crate, rayon, HNSW

---

## 🔬 Findings: Cross-Task Dependencies

### Group A (Bug Fixes)

```
CODE-037 (AuthRateLimiter) ─── FULLY ISOLATED ─── only cli_server.rs
CODE-092 (Euclidean) ───────── ALREADY FIXED ─── no work needed
PERF-23 (ep_enter freeze) ───┐
PERF-28 (tombstone search) ───┼── MODERATELY COUPLED ── share search_layer/delete paths
PERF-27 (select_neighbors) ───┘   (can be fixed independently, but benefits from PERF-28)
```

### Group B (Optimizations)

```
PERF-21 (AVX-512 f32x16) ───┐
PERF-22 (SQ8 euclidean) ─────┼── COMPLEMENTARY ── different match arms in calculate_similarity()
                              │   Both in src/index/distance.rs, no file conflicts
PERF-15 (PyBuffer batch) ────┐
PERF-16 (#[pyclass] hits) ───┼── INDEPENDENT ── different methods, same file
PERF-30 (Config tuning) ─────┘   Fully isolated in src/config.rs
```

**Order within each group:** Any order is safe. Recommended: hardest/riskiest first (PERF-23 → PERF-28 → PERF-27 for Group A; PERF-21 → PERF-22 → PERF-16 → PERF-15 → PERF-30 for Group B).

---

## Group A — Bug Fixes

### Task A1: CODE-037 — AuthRateLimiter unbounded HashMap

**Files:**
- Modify: `src/cli_server.rs:146-211`
- Modify: `Cargo.toml` (add `lru` dep)

**Risk:** LOW — fully isolated to one file.

**Analysis:** `MAX_AUTH_FAILURE_ENTRIES = 10_000` is too high. A distributed attack with 10K+ unique IPs fills memory. `sweep_expired()` only runs when size > 10K, and only removes entries within 60s window — sustained attack keeps all entries.

**Fix strategy:** Replace `Mutex<HashMap<String, (u32, Instant)>>` with `Mutex<LruCache<String, (u32, Instant)>>` capacity 1000. LRU eviction drops oldest entries automatically — no need for `MAX_AUTH_FAILURE_ENTRIES` or manual `sweep_expired()`.

**Step 1: Add `lru` dependency**

```toml
# Cargo.toml — add under [dependencies]
lru = "0.12"
```

**Step 2: Replace AuthRateLimiter internals**

- `cli_server.rs:7`: change `use std::collections::HashMap;` → no longer needed (LruCache replaces it)
- `cli_server.rs:146`: Remove `MAX_AUTH_FAILURE_ENTRIES` constant
- `cli_server.rs:149-156`: Change field from `Mutex<HashMap<String, (u32, Instant)>>` to `Mutex<LruCache<String, (u32, Instant)>>`
- `cli_server.rs:160-165`: `new()` → `LruCache::new(1000)` instead of `HashMap::new()`
- `cli_server.rs:168-172`: Remove `sweep_expired()` entirely (LRU handles eviction)
- `cli_server.rs:175-190`: `is_rate_limited()` — use `cache.get(ip)` instead of `failures.get(ip)`, `cache.pop(ip)` instead of `failures.remove(ip)`. No len check.
- `cli_server.rs:193-205`: `record_failure()` — use `cache.put(ip, ...)` (LRU auto-evicts). No len check.
- `cli_server.rs:208-210`: `reset()` — use `cache.pop(ip)` instead of `failures.remove(ip)`

**Step 3: Add test for AuthRateLimiter**

- New test `test_auth_rate_limiter_exceeds_max` in `vantadb-server/tests/server.rs` (or `cli_server.rs` inline tests)
- Send 6 invalid auth requests from same IP → expect 429 on 6th
- Send 6 from different IPs → expect all succeed (LRU capacity is per-IP, not global)

---

### Task A2: PERF-23 — ep_enter freeze on delete

**Files:**
- Modify: `src/index/core.rs` (add `find_new_entry_point()` method)
- Modify: `src/storage/engine/ops.rs:542-553` (call EP replacement after delete)
- Modify: `src/storage/engine/init.rs:434-444` (handle EP in WAL replay delete)
- Modify: `tests/certification/hnsw_validation.rs` (add EP delete test)

**Risk:** MEDIUM — affects HNSW graph integrity. Shared code with PERF-28.

**Analysis:** When the entry point node is deleted, `hnsw.nodes.remove(&id)` removes it from the graph, but `entry_point` Mutex still holds the old ID. Next `search_nearest()` gets `None` from `self.nodes.get(&ep)` and returns empty results.

**Fix strategy:** After removing from `hnsw.nodes`, check if the removed ID was the entry point. If so, scan `hnsw.nodes` for any remaining node and promote it. Use the highest `max_layer` node (same criteria as `update_metadata()`).

**Step 1: Add `find_new_entry_point()` to CPIndex** (`src/index/core.rs`, around line 450)

```rust
/// Find a replacement entry point when the current one is deleted.
/// Scans all nodes and returns the one with the highest max_layer.
pub fn find_new_entry_point(&self) -> Option<u128> {
    self.nodes.iter()
        .max_by_key(|kv| kv.value().max_layer)
        .map(|kv| *kv.key())
}
```

**Step 2: Modify `StorageEngine::delete()`** (`src/storage/engine/ops.rs:553`)

After `hnsw.nodes.remove(&id)`, add:
```rust
// If we just removed the entry point, promote a replacement
if removed_id == *self.hnsw.read().entry_point.lock() {
    let mut ep = self.hnsw.read().entry_point.lock();
    if *ep == removed_id {
        *ep = self.hnsw.read().find_new_entry_point()
            .unwrap_or(u128::MAX); // ENTRY_POINT_NONE if empty
    }
}
```

**Step 3: Modify WAL replay delete** (`src/storage/engine/init.rs:434-444`)

After deleting from KV backend, add same EP replacement logic.
Note: WAL replay currently does NOT call `hnsw.nodes.remove(&id)` — it only writes tombstone. This causes a zombie node. For EP fix, we need to also call `hnsw.nodes.remove(&id)` during replay.

**Step 4: Add test**

In `tests/certification/hnsw_validation.rs`:
- Insert 5 nodes
- Delete the entry point (first inserted node, highest layer)
- Run search — must return results (the other 4 nodes)

---

### Task A3: PERF-28 — Tombstone mitigation in search

**Files:**
- Modify: `src/index/core.rs:562-570, 578-587, 694-711` (skip tombstoned nodes entirely, not just in results)
- Modify: `src/storage/engine/init.rs:434-444` (remove zombie nodes from hnsw.nodes during WAL replay)
- Modify: `src/index/core.rs:1002` (filter tombstones before select_neighbors)
- Modify: `tests/core/hnsw.rs` (add tombstone search test)

**Risk:** MEDIUM — affects search behavior. MODERATELY COUPLED with PERF-23.

**Analysis:** Two issues: (1) Tombstoned nodes are still added to `candidates` heap in `search_layer`, polluting the early-stopping heuristic. (2) WAL replay doesn't remove nodes from `hnsw.nodes`, creating zombies that remain in the graph forever.

**Fix strategy:**
1. In `search_layer()`: Skip adding tombstoned nodes to `candidates` (not just exclude from `results`). This prevents tombstone pollution of the candidate heap.
2. In `init.rs` WAL replay: After writing tombstone to vfile, also remove node from `hnsw.nodes` and check EP (ties to PERF-23).
3. In `select_neighbors()`: Pre-filter neighbor list to exclude tombstoned entries.

**Step 1: Fix search_layer candidate exclusion**

In `src/index/core.rs:562-587` — Entry point loop:
```rust
// Current: always adds to candidates, then checks eligibility
// Fix: skip entirely if tombstoned
if !eligible { continue; }
```

Same pattern at lines 694-711 — Neighbor loop:
```rust
// Current: adds to candidates regardless of tombstone
// Fix: skip if tombstoned
if !eligible { continue; }
```

**Step 2: Fix early stopping**

Lines 578-587: The `d_cand < worst.0` check remains the same, but since tombstoned nodes are no longer in candidates, the early stopping is no longer polluted.

**Step 3: Fix WAL replay zombies**

In `src/storage/engine/init.rs:434-444`, after writing tombstone:
```rust
hnsw.nodes.remove(&id);
// Also check entry point (ties to PERF-23)
if id == *ep {
    *ep = hnsw.find_new_entry_point().unwrap_or(u128::MAX);
}
```

**Step 4: Add test**

In `tests/core/hnsw.rs`: Delete a third of inserted nodes, verify search recall remains high.

---

### Task A4: PERF-27 — select_neighbors heuristic diversity

**Files:**
- Modify: `src/index/core.rs:729-835` (add tombstone filtering, reduce vector clones)
- Modify: `tests/certification/hnsw_recall.rs` (measure recall impact)
- Tests: `src/index/core.rs` inline tests

**Risk:** LOW — improves construction quality, no correctness dependency on other fixes.

**Analysis:** `select_neighbors` currently clones vector data repeatedly (`cand_slice.clone()`) and has no tombstone awareness. Making the diversity heuristic more efficient and zombie-aware improves graph quality.

**Fix strategy:**
1. Avoid `cand_slice` clone by using the existing `HnswNode.vec_data` reference
2. Add tombstone check before adding to selected neighbors

**Step 1: Optimize vector access**

Replace `cand.vec_data.as_f32_slice().map(|s| s.to_vec())` with direct slice references where possible.

**Step 2: Add tombstone filtering**

Before `select_neighbors`, filter the input `BinaryHeap<NodeSimMin>` to exclude tombstoned nodes.

---

### Task A5: CODE-092 — Euclidean distance (Verify + Mark)

**Files:** None needed — already fixed.

**Verification:** `grep` confirms:
- `distance.rs:138-145`: `euclidean_distance_squared_f32` already negates with `-` and applies `sqrt`
- `core.rs:1167-1172`: `search_nearest` already negates Euclidean scores

**Action:** Mark CODE-092 as ✅ in Backlog.md (already is).

---

## Group B — Optimizations

### Task B1: PERF-21 — AVX-512 f32x16 SIMD dispatch

**Files:**
- Modify: `src/index/distance.rs` — add 3 f32x16 kernel functions + runtime dispatch
- Modify (optional): `src/node.rs:290-344` — add Avx512 dispatch in `cosine_similarity()`

**Risk:** MEDIUM — new SIMD code paths need testing across CPU architectures.

**Analysis:** `wide` crate already provides `f32x16` (512-bit). Current code uses `f32x8` everywhere. `HardwareCapabilities::global().instructions` already detects `Avx512`. Simply add the f32x16 variants and route.

**3 kernel functions to add:**
1. `euclidean_distance_sq_f32x16(a, b) -> f32` — chunks_exact(16), f32x16 diff² accumulate
2. `f32_dot_product_f32x16(a, b) -> f32` — chunks_exact(16), f32x16 multiply-add
3. `f32_dot_and_norm_b_sq_f32x16(a, b) -> (f32, f32)` — combined dot + norm

**Runtime dispatch pattern:**

```rust
pub fn euclidean_distance_squared_f32(a: &[f32], b: &[f32]) -> f32 {
    let caps = hardware::HardwareCapabilities::global();
    match caps.instructions {
        InstructionSet::Avx512 => euclidean_distance_sq_f32x16(a, b),
        InstructionSet::Avx2 | InstructionSet::Neon => euclidean_distance_sq_f32x8(a, b),
        InstructionSet::Fallback => euclidean_distance_sq_scalar(a, b),
    }
}
```

---

### Task B2: PERF-22 — SQ8 euclidean vectorization

**Files:**
- Modify: `src/index/distance.rs:150-185` — SIMD-ize SQ8 euclidean loop
- Modify (optional): `src/vector/quantization.rs` — add SIMD sq8 euclidean kernel

**Risk:** MEDIUM — mixed-precision (i8→f32) SIMD is more complex than pure f32.

**Analysis:** The scalar loop in `sq8_similarity_fallback` does `(s as f32) * inv_scale` per element. This can be vectorized by loading i8 values, converting to f32 in-register, applying scale, and using f32 SIMD for diff².

**Fix strategy:**

```rust
DistanceMetric::Euclidean => {
    // Use wide::i8x32 or chunk-and-convert approach
    // For AVX2: 32 i8 values → broadcast to f32x8×4 → subtract from query f32x8 → FMA accumulate
    let mut sum_sq = f32x8::splat(0.0);
    let inv_scale_v = f32x8::splat(inv_scale);
    for (chunk_q, chunk_s) in raw_query.chunks_exact(8).zip(sq8_data.chunks_exact(8)) {
        // Load 8 i8 → cast to f32 → multiply by inv_scale
        let sq8_f32 = /* SIMD i8→f32 conversion */;
        let q = f32x8::from(chunk_q);
        let diff = q - (sq8_f32 * inv_scale_v);
        sum_sq = diff.mul_add(diff, sum_sq);
    }
    -(sum_sq.reduce_add() + remainder_handling)
}
```

---

### Task B3: PERF-16 — `#[pyclass]` for search hits (extend)

**Files:**
- Modify: `vantadb-python/src/lib.rs` — convert `put_batch_raw()` / `put_batch()` / `list_memory()` returns from dicts to `#[pyclass]` types
- Create: `vantadb-python/src/types.rs` — move VantaPySearchHit and add VantaPyMemoryRecord as `#[pyclass]`
- Modify: `vantadb-python/vantadb_py/__init__.py` — expose new pyclass types

**Risk:** LOW — `VantaPySearchHit` already exists and works. Extending pattern is mechanical.

**Analysis:** VantaPySearchHit already avoids 5 PyDict allocations per search result. The remaining dict-based returns (put_batch, put_batch_raw, list_memory) can be migrated to the same pattern.

**New #[pyclass] types:**
1. `VantaPyMemoryRecord` — wraps `VantaMemoryRecord` with getter properties
2. `VantaPyListResult` — wraps the list page with records + total_count

**Migration:**
- `memory_record_to_pydict_owned()` → replace with `VantaPyMemoryRecord::new(record)`
- `put_batch_raw()` return → `Vec<VantaPyMemoryRecord>` instead of `Vec<Py<PyAny>>`
- `list_memory()` return → `VantaPyListResult { records: Vec<VantaPyMemoryRecord>, total_count }`

---

### Task B4: PERF-15 — put_batch_raw zero-copy PyBuffer

**Files:**
- Modify: `vantadb-python/src/lib.rs:255-284, 977-1058` — avoid Vec copies in 2D buffer
- Modify: `src/sdk/api.rs` — optionally add slice-based batch insert
- Modify: `vantadb-python/tests/test_perf_15_16.py` — verify zero-copy behavior

**Risk:** LOW-MEDIUM — the PyBuffer already exists; just use `.as_slice()` instead of `.to_vec()`

**Analysis:** Current `extract_2d_buffer()` copies the entire numpy array into `Vec<f32>`, then per-row copies slices. Using `PyBuffer::as_slice(py)` gives a `&[Cell<f32>]` view with zero copy. The vector can be passed directly as `&[f32]` to the engine.

**Fix strategy:**
1. Change `extract_2d_buffer` to return a view struct `FlatBufferView { data: &[Cell<f32>], nrows, ndims }` instead of `Vec<f32>`
2. In `put_batch_raw`, use `view.slice(row)` references instead of `.to_vec()`
3. Pass `&[f32]` references to `VantaMemoryInput` instead of owned `Vec<f32>`

---

### Task B5: PERF-30 — Config tuning for batch ingestion

**Files:**
- Modify: `src/config.rs` — add batch_size, wal_buffer_size, flush_threshold fields
- Modify: `src/sdk/api.rs` — use config values for batch operations
- Modify: `src/storage/wal.rs` — use config values for WAL buffer sizing

**Risk:** LOW — fully isolated in config and plumbing.

**Analysis:** Current batch sizes and WAL buffer sizes are hardcoded. Making them configurable via `VantaConfig` allows tuning for different workloads (high-throughput ingestion vs low-latency reads).

**New config fields:**
```rust
pub batch_size: Option<usize>,      // Default: 1000
pub wal_buffer_size: Option<usize>,  // Default: 64KB
pub flush_threshold: Option<usize>,  // Default: 10_000 nodes
```

---

## Execution Plan

### Recommended Order (maximize parallel work)

**Phase 1 — Independent Parallel Tasks:**
| Agent | Task | Files | Est. |
|-------|------|-------|------|
| Agent 1 | CODE-037 (AuthRateLimiter) | cli_server.rs + Cargo.toml | 🟢 1h |
| Agent 2 | PERF-30 (Config tuning) | config.rs + api.rs + wal.rs | 🟢 4h |
| Agent 3 | PERF-23 (ep_enter freeze) | core.rs + ops.rs + init.rs | 🟡 1d |

**Phase 2 — After Phase 1:**
| Agent | Task | Depends on | Est. |
|-------|------|-----------|------|
| Agent 4 | PERF-28 (Tombstone search) + PERF-27 | Phase 1 (PERF-23 shared code) | 🟡 1d |
| Agent 5 | PERF-21 (AVX-512) + PERF-22 (SQ8) | None | 🟡 2-3d |

**Phase 3 — Python SDK:**
| Agent | Task | Depends on | Est. |
|-------|------|-----------|------|
| Agent 6 | PERF-16 (#[pyclass] hits) + PERF-15 (PyBuffer) | None | 🟡 2-3d |

**Total estimated effort:** 8-12 days engineering time across 3 phases.
**Max parallelism:** 3 simultaneous agents in Phase 1, 2 in Phase 2, 1 in Phase 3.
