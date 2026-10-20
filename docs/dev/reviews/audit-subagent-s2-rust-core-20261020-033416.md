---
title: "S2 — Deep Audit: Rust Core (`src/`)"
kind: review
status: active
description: "Informe crudo del subagente S2: auditoría profunda del core Rust (engine, storage/WAL, backends, índices vector/text/HNSW, graph, memory, sdk, config, error, cli). Base de los hallazgos de la familia `VDB-*` del ledger."
tags: [audit, reviews, rust, core, subagent]
---

# Informe del subagente — Deep Audit: Rust Core (`src/`)

**Repo:** `C:\Users\Eros\VantaDB Proyect\VantaDB`
**Scope:** `src/` — engine, storage/wal, backends, indexes (vector/text/hnsw), graph, memory/dreaming, sdk, config, error, cli, query parser/planner/executor, edge_index, vfile_mmap
**Mode:** READ-ONLY deep audit (no files in the repo created/edited/deleted; no git write commands)
**Method:** `codegraph_codegraph_explore` (1 call) + `codebase-memory-mcp` (list_projects) for navigation, then exhaustive `grep`/`read` of the actual source. Every finding below was confirmed by reading the real source lines and by tracing a concrete call path from a user-reachable entry point.
**Tooling note:** `cargo` is **not on PATH** in this environment (`C:\Users\Eros\.cargo\bin\cargo.exe` exists but PATH does not include it), so per the audit contract `cargo check` / `cargo clippy` were **not** run. All claims below are source-level, not compiler-verified.

---

## Findings

### VDB-01 | P1 | vector index / distance — vector dimension mismatch is never validated, and under `Euclidean` it produces the *maximum possible* score, so dimension-mismatched records outrank genuine matches

- **Location:**
  - `src/index/distance/kernels.rs:107` — `if a.len() != b.len() || a.is_empty() { return 0.0; }` (f32x8 euclidean kernel)
  - `src/index/distance/kernels.rs:143` — same guard in the f32x16 kernel
  - `src/index/distance/mapper.rs:175` — `DistanceMetric::Euclidean => -euclidean_distance_squared_f32(raw_query, f),`
  - `src/index/flat.rs:44` — `_ => calculate_similarity(query_vec, None, None, None, &node.vec_data, metric),`
  - `src/index/search/layer.rs:50` — `DistanceMetric::Euclidean => -euclidean_distance_squared_f32(query_vec, vec),`
  - `src/error.rs:146-151` — `Error::DimensionMismatch { expected, got }` **exists but is never constructed anywhere in `src/`** (only referenced by the match arm at `src/error.rs:400` and `src/server/errors.rs:44`, plus `error.rs` tests)
  - `src/sdk/serialization/mod.rs:611` — `node.vector = VectorRepresentations::Full(vec.clone());` (put path — no length check)
  - `src/sdk/api/memory.rs:2489` — `node.vector = VectorRepresentations::Full(v.clone());` (bulk-import path — no length check)
- **Claim/Break:**
  `euclidean_distance_squared_f32(a, b)` returns **`0.0`** when `a.len() != b.len()` (the SIMD kernels short-circuit on a length mismatch and return the identity distance instead of an error/`None`). `calculate_similarity` negates it → the *score* becomes `-0.0` → `0.0`, which is the **best score Euclidean can produce** (identical vectors also score `0.0`). Because `src/error.rs` defines `DimensionMismatch` but no layer ever validates or raises it, and the put path copies the caller vector verbatim, a corpus can legitimately contain records with different vector lengths.
  Consequences:
  - **Flat path** (`src/index/flat.rs`, used by default for any DB with ≤ `flat_threshold` = 10 000 nodes — `src/index/search/neighbors.rs:57-62`): a mismatched candidate scores `0.0` and **outranks every correctly-dimensioned candidate** (whose scores are negative).
  - **HNSW vstore path** (`src/index/search/layer.rs:50`): same `0.0` → same wrong ranking.
  - **HNSW inline path** (`src/index/graph/core.rs:215-231` → `euclidean_distance_sq_with_norms`, `src/index/distance/metrics.rs:70-84`): `f32_dot_product` returns `0.0` on a length mismatch, so the score becomes `-(q_norm² + node_norm²)` — a *large negative* value. **The three backends therefore disagree for the identical input.**
- **Why it matters:** search results are silently wrong (wrong top-1) with no error, no warning and no log line. Under the default `Cosine` metric the same mismatch yields `0.0` (a neutral, plausible score), so the bug hides until a caller switches `distance_metric` to `Euclidean`. The presence of an unused `Error::DimensionMismatch` shows the invariant was *intended* and never implemented.
- **Trigger:**
  1. `Embedded::put(ns, "a", payload, vector=[0.0, 1.0, 0.0])` → 3-dim record.
  2. `Embedded::put(ns, "b", payload, vector=[1.0, 0.0, 0.0, 0.0, 0.0])` → 5-dim record (no dimension check on put).
  3. `Embedded::search(MemorySearchRequest{ namespace: ns, query_vector: vec![1.0,0.0,0.0], distance_metric: DistanceMetric::Euclidean, top_k: 1, .. })`.
  4. Observed ranking: record **"b"** (dimension-mismatched) is returned first with score `0.0`, while record **"a"** (true distance √2) scores `-2.0`. The correct answer is "a".
- **Fix:**
  1. Make the kernels return an *undefined* value instead of `0.0` on a length mismatch — e.g. `f32::NAN` — and have `calculate_similarity`/`metric_score` propagate `NAN`; `src/index/graph/types.rs:164-176` already pins `NaN` below every finite score and `src/index/search/nearest.rs:192` (`result.retain(|ns| !ns.0.is_nan())`) already drops NaN candidates. This removes the "mismatch == perfect match" inversion with a 3-line change.
  2. Validate the dimension at the trust boundary: reject a put whose vector length differs from the namespace/index dimension with `Error::DimensionMismatch { expected, got }` (the error already exists), and validate the query vector against the index dimension in `src/sdk/search/mod.rs` before dispatching to the index.
  3. Add a regression test: mixed-dimension corpus + `Euclidean` + `top_k=1` must return the dimension-matching record.
- **Verify:**
  ```rust
  // new test: src/sdk/search/tests.rs
  // put "a" (3-dim) and "b" (5-dim) in ns; search with a 3-dim query,
  // distance_metric = Euclidean, top_k = 1 → hits[0].record.key == "a"
  ```
  Plus: `cargo test --lib sdk::search::tests::euclidean_dimension_mismatch_ranks_matching_dim_first`.

---

### VDB-02 | P2 | vector index / SDK search — the Euclidean score scale differs per backend (`-d²` vs `-d`), so `range.min_score` / `range.max_score` filter different sets depending on DB size and filter selectivity

- **Location:**
  - `src/index/flat.rs:44-51` — flat path returns the **raw** negative squared distance (`-d²`)
  - `src/index/search/nearest.rs:196-204` — HNSW path transforms it: `DistanceMetric::Euclidean => -(-score).max(0.0).sqrt(),` (→ `-d`)
  - `src/sdk/search/vector.rs:54-58` — pre-filter brute-force path transforms to `-d`
  - `src/sdk/search/vector.rs:204-208` — post-filter brute-force fallback transforms to `-d`
  - `src/sdk/search/page.rs:264-269` — `apply_range` filters on the raw `hit.score`: `hit.score >= min && hit.score <= max`
- **Claim/Break:** the *same* query returns two different score scales depending on which code path produced the hit:
  - no filters → `FilterStrategy::PostFilter` (`src/cost_estimator.rs:176-195`) → index path → `flat_search` → score = `-d²`
  - highly selective filters → `FilterStrategy::PreFilter` → brute force → score = `-d`
  - DB with > 10 000 nodes → HNSW path → score = `-d`
  The `RangeFilter` window is applied *after* fusion on `hit.score`, so the window means a different physical distance depending on the path.
- **Why it matters:** `range.min_score = -4.0` keeps hits with `d ≤ 2` on the flat path but `d ≤ 4` on the HNSW path; a filter that suddenly starts (or stops) matching when the DB crosses 10 000 nodes is indistinguishable from a data problem. It also makes the score returned to the caller (`MemorySearchHit.score`) mean two different things.
- **Trigger:**
  1. Same DB, same records, same query, `distance_metric = Euclidean`.
  2. Request A: no `filters` → `hits[0].score == -4.0` for a record at distance 2.
  3. Request B: add a highly selective `filters` entry → `hits[0].score == -2.0` for the *same* record at distance 2.
  4. `range.min_score = -3.0` → the flat path keeps it (`-4.0 <= -3.0`), the prefilter path drops it (`-2.0 > -3.0`).
- **Fix:** normalise the score in one place. Apply the `-(-score).max(0.0).sqrt()` Euclidean transform to the flat path as well (i.e. inside `flat_search` / `calculate_similarity`'s caller), or normalise at the SDK assembly boundary in `src/sdk/search/vector.rs` so every path emits `-d`.
- **Verify:** unit test asserting `flat_search` and the HNSW path return the same score for the same (query, node) pair, and that `apply_range` behaves identically for both.

---

### VDB-03 | P2 | node bitset — the filter-bitset mechanism cannot represent the masks it is given: positions are truncated to `u32` on write and only the low 128 bits are persisted, so masks built from 128-bit node ids never match and distinct node ids collide

- **Location:**
  - `src/node/bitset.rs:75-77` — `pub fn set_bit(&mut self, pos: usize) { self.inner.add(pos as u32); }`
  - `src/node/bitset.rs:101-112` — `to_u128()` keeps **only bits 0..128**
  - `src/storage/ops.rs:163` — `header.bitset = node.bitset.to_u128();` (the persisted header field is a `u128`)
  - `src/sdk/search/lexical.rs:208` — `bitset.set_bit(node_id as usize);`
  - `src/sdk/search/lexical.rs:221` — `bitset.set_bit(record.node_id as usize);`
  - `src/sdk/serialization/mod.rs:85-91` — `memory_node_id` = `XxHash3_128(ns + "\0" + key)` → **uniformly distributed over 128 bits**
- **Claim/Break:** `node_id as usize` truncates a `u128` hash to its low 64 bits and `set_bit` truncates further to the low **32** bits. Two distinct records whose `XxHash3_128` ids share the same low 32 bits therefore map to the **same bit**. Separately, `to_u128()` (used when the node header is written) persists only the first 128 bits, while the query mask uses bit indices up to `2^32-1` — so even a correctly-populated node bitset could never satisfy the mask. In production `node.bitset` is never populated at all (the only non-test `set_bit` calls are the two in `lexical.rs`), so `matches_mask` is always false for a non-empty mask.
- **Why it matters:** the `InFilter` strategy builds a mask that the index then uses to reject candidates — it rejects **100 %** of them. The code papers over this with a brute-force fallback (`src/sdk/search/vector.rs:176-211`, comment: *"node bitsets are never populated on insert, so HNSW rejects all candidates"*), so results are currently correct but every filtered vector search silently degrades to a full scan. The u32 collision is a latent correctness bug the moment node bitsets are ever populated.
- **Trigger:** with ≥ ~100 000 records in one namespace the birthday bound makes a low-32-bit collision near-certain (n²/2³² ≈ 1.16 collisions at n = 100 000): two records whose ids differ only above bit 32 share a mask bit, so a filter on one of them matches the other.
- **Fix:** index the bitset by a **dense, per-namespace ordinal** (not by the hash id) — e.g. keep a `u128 → ordinal` map and set `bit(ordinal)`; make `set_bit` return an error (or take `u32`) instead of silently truncating, and widen the persisted header field (or persist the full roaring bitmap) instead of `to_u128()`.
- **Verify:** test with two records engineered to share the low 32 bits of their `memory_node_id` → the filter mask must contain exactly one of them; plus a test that a filtered vector search returns the same hit set through the `InFilter` path and the brute-force fallback.

---

### VDB-04 | P2 | WAL durability — the writer can emit records larger than the reader's hard 10 MB ceiling; every recovery path classifies such a record as corruption and silently drops it from replay

- **Location:**
  - `src/wal.rs:407-413` — `let payload = postcard::to_allocvec(record)?; let len = payload.len() as u32;` → **no size cap** on write
  - `src/wal.rs:458-459` — same in `batch_append`
  - `src/wal.rs:638` — `if len == 0 || len > 10_000_000 || ... { return false; }` (`check_record_at` → "not a valid record")
  - `src/wal.rs:1067-1070` — `if len > 0 && len <= 10_000_000 && ...` (`WalReader::next_record`)
  - `src/wal.rs:939` — `if len == 0 || len > 10_000_000 { WalVerifyStatus::Corrupt { .. } }` (`verify_wal_file`)
  - `src/wal.rs:356-367` — on open, `file.set_len(valid_end)` truncates the tail after `recover_valid_records`
  - `src/wal.rs:656-670` — `scan_forward_valid` scans **byte by byte** to the end of the file
  - `src/node/vector_data.rs:7` / `src/config.rs:41,57` — `MAX_VEC_F32_LEN` / `MAX_F32_VEC_LEN` / `MAX_VEC_DIM` are **declared but never enforced** in the Rust core (only asserted in `config.rs` unit tests); `MAX_VEC_F32_LEN` is used *only* for the `MmapFull` branch (`src/node/vector_data.rs:163`), never for `Full`
- **Claim/Break:** the writer accepts any payload size; all three readers cap a record at 10 000 000 bytes and treat anything larger as **corruption**. Consequences on the next `open()`:
  1. `recover_valid_records` → `check_record_at` returns false at the oversized record → `try_scan_forward` → `scan_forward_valid` walks the remaining file **one byte at a time** (each step = one `seek` + one 4-byte read). For a 12 MB tail that is ~12 M syscalls → a multi-second startup stall on *every* open.
  2. The oversized record itself is **never replayed**: the acknowledged, WAL-durable insert is silently dropped from recovery. If the crash happened between the WAL append and the store write, the write is lost — the exact scenario the WAL exists to cover.
  3. If no valid record follows, `file.set_len(valid_end)` **destroys** the oversized record and everything after it (a copy is kept in `<path>.corrupt`, but the data is gone from the live WAL).
- **Why it matters:** silent loss of an acknowledged durable write + a startup DoS, from a single input that no layer validates. The writer/reader length contract is asymmetric: the writer has no ceiling, the reader has a hard one.
- **Trigger:** `Embedded::put(ns, "big", payload, vector = vec![1.0f32; 3_000_000])` (3 M f32 = 12 MB > 10 MB). The `WalRecord::Insert(UnifiedNode)` frame is ~12 MB → written successfully → `engine.flush()` → reopen the DB → the record is classified corrupt → skipped during replay (and, with no following record, truncated away).
- **Fix:**
  1. Enforce one shared ceiling on the write side: reject a record whose serialised size exceeds the reader's cap with a clear `Error` (`Error::VectorLenOverflow` / a new `Error::wal_error`), instead of writing a frame the reader will reject.
  2. Make the reader's ceiling explicit and shared (a `pub(crate) const MAX_WAL_RECORD_BYTES`), and make the reader distinguish *"oversized but well-formed"* from *"corrupt"* — an oversized record should be a hard error, not a trigger for byte-by-byte scan-forward.
  3. Enforce `MAX_VEC_DIM` / `MAX_F32_VEC_LEN` at the SDK put boundary (they are already declared and re-exported from `src/lib.rs:177` but never used).
- **Verify:** new test in `src/storage/engine/tests/init.rs`: insert a node whose WAL frame exceeds 10 MB → the write must fail with a typed error (not succeed), and a reopen must replay every previously acknowledged record.

---

### VDB-05 | P2 | SDK search — `top_k` is not clamped in the Rust SDK, and it flows straight into a `HashSet::with_capacity(top_k * 3)` allocation

- **Location:**
  - `src/config.rs:51` — `pub const MAX_K: usize = 10_000;` (the cap exists)
  - `src/server/handlers.rs:370-374` — `clamp_limit` — the **only** place `MAX_K` is applied (HTTP surface)
  - `src/sdk/search/mod.rs:230-232` — the SDK only checks `request.top_k == 0`
  - `src/sdk/search/vector.rs:104-108` — `let budget = … .max(top_k);` → unbounded
  - `src/index/search/nearest.rs:115-121` — `ef_search = static_ef.max(top_k)`
  - `src/index/search/nearest.rs:145-149` — `HashSet::with_capacity_and_hasher(ef_search.max(top_k).saturating_mul(3), RandomState::new())`
  - `src/cli_handlers/search.rs:85` — `top_k: limit` (CLI, unclamped)
- **Claim/Break:** `MAX_K` is documented as *"Maximum value accepted for `top_k` / `k` across all search entry points (ERR-022)"*, but the enforcement lives only in the HTTP handler. The Rust SDK (and the CLI, which feeds it) pass `top_k` through unmodified into `HashSet::with_capacity_and_hasher(top_k * 3, …)`.
- **Why it matters:** `HashMap::with_capacity` for an absurd capacity hits `handle_alloc_error` → **process abort** (SIGABRT), not a catchable `Err`. A single oversized `top_k` kills the server process.
- **Trigger:**
  1. DB with > 10 000 nodes (so the HNSW path — and therefore the `visited` allocation — is taken; the flat path returns before it).
  2. `Embedded::search(MemorySearchRequest{ namespace, query_vector, top_k: 1_000_000_000, .. })` (or `vanta search --limit 1000000000`).
  3. `budget = 1e9` → `ef_search = 1e9` → `HashSet::with_capacity_and_hasher(3_000_000_000)` → ~72 GB allocation → abort.
- **Fix:** clamp `top_k` inside the SDK (`src/sdk/search/mod.rs`, right next to the `top_k == 0` check) with the same `MAX_K` + warning pattern as `clamp_limit`, so every binding (Rust, CLI, MCP, Python, WASM) inherits the same guard.
- **Verify:** `Embedded::search` with `top_k = usize::MAX` must return an empty/limited result set (never abort); assert the effective `top_k` is ≤ `MAX_K`.

---

### VDB-06 | P3 | storage / transactions — `commit_transaction` on an unknown or already-committed transaction id appends a spurious `Commit` record to the WAL and returns `Ok(())`

- **Location:** `src/storage/engine/txn.rs:281-287`
  ```rust
  let Some(buffer) = self.txn.take(txn_id) else {
      // Phase 1 fallback: if no buffering, just append Commit marker
      if let Some(ref sharded) = self.wal {
          sharded.append(&crate::wal::WalRecord::Commit(txn_id))?;
      }
      return Ok(());
  };
  ```
- **Claim/Break:** `TxnManager::take` (`src/storage/engine/txn.rs:140-145`) returns `None` both when the txn was never begun **and** when it was already committed. In both cases a bare `Commit(txn_id)` is written to the WAL. The recovery slice-mask (`src/storage/engine/init.rs:532-540`) treats a `Commit` with no matching open `Begin` as a no-op, so the immediate effect is benign — but the WAL now contains a commit marker for a transaction that never existed, and the call reports success for an invalid operation.
- **Why it matters:** low direct impact today, but it is a contract violation on a durability-critical path: "commit an unknown txn" succeeds silently instead of returning an error, and it pollutes the WAL with a marker that a future 2PC recovery heuristic could misread.
- **Trigger:** `engine.begin_transaction()` → `t` → `engine.commit_transaction(t)` → `engine.commit_transaction(t)` (second call). The second call appends a second `Commit(t)` to the WAL and returns `Ok(())`.
- **Fix:** distinguish "never begun" from "already finished" — have `TxnManager::take` report which — and return `Error::InvalidInput("transaction {id} is not active")` for the never-begun case; only write the `Commit` marker for a genuinely empty-but-active transaction.
- **Verify:** `commit_transaction` on an unknown id must return `Err` and must not grow the WAL record count.

---

### VDB-07 | P3 | storage — the WAL copy of a node and the stored copy disagree on `last_accessed` (the two write paths diverge in *opposite* directions)

- **Location:**
  - `src/storage/engine/insert.rs:209-215` — single-record path: the **WAL** copy gets `wal_node.last_accessed = now_ms`, but `apply_insert(node)` stores the **original** node (old `last_accessed`)
  - `src/storage/engine/insert.rs:447-451` (`stage_one_node`) — batch path: the **stored** copy gets `active_node.last_accessed = now_ms`
  - `src/storage/engine/insert.rs:339-353` (`append_batch_wal`) — batch path: the **WAL** copy keeps the original `last_accessed`
- **Claim/Break:** after a crash + replay, the reconstructed node's `last_accessed` differs from the in-memory node's, and the discrepancy is in the *opposite direction* depending on which write path was used.
- **Why it matters:** `last_accessed` drives LRU eviction (`src/storage/engine/maintenance.rs`). The divergence is small and self-healing, but it means the eviction order after a restart is not the order the running process had, and the two insert paths behave differently.
- **Trigger:** `engine.insert(node)` with a stale `last_accessed` → the WAL record carries `now_ms`, the vstore copy carries the stale value → after `engine.flush()` + reopen (WAL replay), the node's `last_accessed` is `now_ms`, whereas without a crash it is the stale value.
- **Fix:** normalise once — set `node.last_accessed = now_ms` on the *same* value that is written to both the WAL and the store (mutate the caller's copy before both writes, or pass `now_ms` into `apply_insert`).
- **Verify:** after `insert` + `flush` + reopen, `get(id).last_accessed` must equal the value observed before the reopen.

---

### VDB-08 | P3 | storage / delete — the pre-delete probe swallows the storage error, so a read failure is indistinguishable from "node not found"

- **Location:** `src/storage/engine/delete.rs:38` — `let pre_delete_node = self.get(id).ok().flatten();`
- **Claim/Break:** `.ok()` discards the `Err` from the storage read. A transient read failure (I/O error, corrupt metadata) is treated as "the node does not exist".
- **Why it matters:** the value is used only for the cardinality/secondary-index bookkeeping (`apply_delete_stats_for_node`, `src/storage/engine/delete.rs:75-112`). When the read fails, the counters are **not** decremented and the secondary index entries are **not** removed → the cardinality statistics and the scalar/edge indexes drift permanently from the real data, which then feeds the selectivity estimates used to pick the search strategy (`src/cost_estimator.rs:176-195`).
- **Trigger:** delete a node while the backend read of its metadata fails (e.g. a corrupted KV entry) → `get` returns `Err` → `.ok().flatten()` → `None` → `apply_delete_stats_for_node` is skipped → stale cardinality counts and stale scalar-index entries remain.
- **Fix:** distinguish "not found" from "read failed" — use `self.get(id)?` and treat only `Ok(None)` as "not found", or log the error explicitly before degrading.
- **Verify:** inject a backend read error during `delete` and assert the cardinality counters are either decremented or the error is surfaced — never silently skipped.

---

## Coverage

### Reviewed in depth (read line-by-line, with call-path tracing)

| Area | Files / paths | Notes |
|---|---|---|
| WAL (write + recovery) | `src/wal.rs` (2078 lines, read in full), `src/wal_sharded.rs` (read: shard layout, round-robin, `verify_shard_counts`, `recover`, salvage) | Verified: CRC32C framing, chained (v3) framing, chain-head recovery across reopen, round-robin global-seq derivation (`global = shard + N*local`), per-shard skip arithmetic, shard-count coherence guard, quarantine-on-truncate, `verify_wal_file` chain validation. **No ordering bug found in the WAL append/sync/rotate sequence** — `WalWriter::append` → `maybe_sync` → `try_auto_rotate` ordering is correct, and `rotate`/`try_auto_rotate` both `sync_parent_dir` the new directory entry. Findings: VDB-04 |
| 2PC / transactions | `src/storage/engine/txn.rs` (full), `src/storage/engine/insert.rs` (full), `src/storage/engine/delete.rs` (full) | Verified the `Begin → ops → Prepare → apply → Commit` ordering, the `insert_lock` (ERR-010) critical section, the recovery slice-mask in `src/storage/engine/init.rs:511-557`, `TxnManager` id allocation / conflict checks / probe. Findings: VDB-06, VDB-07, VDB-08 |
| Recovery / open | `src/storage/engine/init.rs:397-626` (`recover_state`), `src/storage/wal.rs` | Verified the global-seq sort, the per-shard skip arithmetic, the ERR-011 shard-count guard, the tombstone+`remove_node` delete replay. **No data-loss bug found in the replay ordering.** |
| Vector index / distance | `src/index/distance/{kernels,metrics,mapper}.rs`, `src/index/flat.rs`, `src/index/search/{layer,nearest,neighbors,mod}.rs`, `src/index/graph/{core,types}.rs` | Verified: SIMD kernels are length-safe (no OOB read); NaN is pinned below finite scores and dropped from results; the zero-norm cosine guard (AUDREP-27/55) is enforced on both insert and query. Findings: VDB-01, VDB-02 |
| Index serialization | `src/index/serialize/bytes.rs:228-519` | Verified `take_bytes` bounds checks, the `node_count` plausibility cap, the `layer_count` cap, `FilterBitset::from_bytes` caps. The `pos += padding` overshoot (line 399-400) is caught by the next `take_bytes` → `Err`, not a panic. **No OOB found.** |
| Node storage | `src/storage/ops.rs`, `src/storage/engine/get.rs:40-200`, `src/node/bitset.rs`, `src/node/vector_data.rs` | Verified the `u16` edge-count overflow guard (ERR-029), the `u32` vector-len overflow guard, the mmap `align_to` reinterpretation, the SQ8/Binary/Turbo decoders' `checked_mul`/`checked_add` bounds. Finding: VDB-03 |
| SDK search | `src/sdk/search/{mod,vector,lexical,page,fusion,multi,mmr}.rs`, `src/sdk/serialization/mod.rs`, `src/sdk/api/memory.rs` | Verified the request validation (`min_confidence`, NaN/∞ rejection, `top_k == 0`), the pagination/cursor assembly, the RRF budget clamps. Findings: VDB-02, VDB-03, VDB-05 |
| Server auth | `src/server/middleware.rs:39-200`, `src/server/state.rs:530-611`, `src/server/pagination.rs` | Verified: constant-time API-key comparison, JWT fallback, deny-by-default L2/L3 identity resolution, `X-Forwarded-For` honoured only behind a trusted proxy, `limit` clamped to `MAX_K`, opaque cursor decoding. **No auth bypass found.** |
| Config / error | `src/config.rs` (spot-checked the env-var parsing, `SyncMode`, `MAX_*` constants), `src/error.rs` | Verified that `Error::DimensionMismatch` is defined but never constructed (root cause of VDB-01), and that `MAX_K`/`MAX_VEC_DIM`/`MAX_F32_VEC_LEN` are declared but unenforced in the core. |

### Reviewed by targeted grep + spot-read (patterns swept across all of `src/`)

`unwrap()` / `expect()` / `panic!` / `unreachable!` / `todo!` / `unimplemented!` outside `#[cfg(test)]` modules (result: only `binary_header.rs:76`, `crypto.rs:270/283/293/346/353`, `server/router.rs:308`, `server/state.rs:209`, `storage/vfile_mmap.rs:535`, `sdk/types.rs:244-288`, `index/serialize/bytes.rs:21/410` — all verified as genuinely infallible or guarded); swallowed errors (`let _ =`, `if let Err(_)`, `.ok()`); `as u32|u16|u8|usize|i32` truncation casts; unchecked `- 1` subtraction; `u64 -> usize` / `f64 -> usize` casts; NaN/∞ handling; `#[cfg(feature = ...)]` divergence; `todo!`/`unimplemented!` (none found).

### NOT reviewed in depth (explicitly out of my coverage — do not assume clean)

- `src/server/handlers.rs` (58 KB) and `src/server/bootstrap.rs` — only the auth/limit/range helpers were read; the full handler surface (import/export/restore endpoints) was not audited.
- `src/migration.rs` (42 KB), `src/schema.rs` — migration correctness not audited.
- `src/storage/archive.rs` (46 KB) — dump/restore/backup paths not audited beyond a grep sweep.
- `src/sdk/serialization/impl_*.rs` (≈ 145 KB) — the export/rebuild/text-index/sparse-index serialisers were not line-audited.
- `src/metrics/`, `src/llm.rs`, `src/skills.rs`, `src/wiki/`, `src/entity/`, `src/shred/`, `src/graphrag/`, `src/tui/`, `src/cli_handlers/*` (beyond `search.rs`), `src/index/{ivf,scann,diskann,neighbor_index}.rs`, `src/index/port_impl.rs`.
- `src/storage/vfile.rs` / `src/storage/vfile_mmap.rs` — only the `AlignedBytes` allocation/dealloc and the `grow_zeroed` path were read; the mmap remap / SIGBUS-handler paths were not audited.
- Concurrency: I traced the `insert_lock` → `cache.volatile` → `cardinality_stats` ordering and the `TxnManager` `active` → `buffers` ordering, and found **no deadlock** (the `insert_lock` is never held while `cardinality_stats.write()` is taken — FIND-345's fix is in place, and `evict_cold_nodes_with_reason_locked` uses the `AssumeHeld` variant). I did **not** run a lock-order checker, and `spawn_blocking` / `tokio` lock-across-`.await` usage was not exhaustively swept.
- No dynamic verification was possible: `cargo` is not on `PATH`, so nothing was compiled, linted, fuzzed or run. Every finding above is source-level only.
