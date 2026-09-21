---
title: "ADR-025: Zero-copy Arrow in bindings — deferred (deliberate copies, measurable reopen signal)"
type: adr
status: accepted
tags: [vantadb, architecture, adr, serialization, arrow, python, node, wasm, zero-copy, deferral]
created: 2026-08-17
last_reviewed: 2026-08-17
related: [ADR-021-zero-copy-arrow-bindings.md]
---

# ADR-025: Zero-copy Arrow in bindings — deferred (deliberate copies, measurable reopen signal)

## Status

Accepted. Outcome of FND-04: **zero-copy Arrow in Python/Node bindings is
deferred.** The core already builds a `RecordBatch` (`src/columnar.rs:22`) but
it is disconnected (only tests consume it) and of partial schema (id +
flattened vectors — does not cover the `VantaMemoryRecord` shape). This ADR
records the decision to **keep the current per-binding serialization**, the
rationale (why each copy is deliberate), and the measurable reopen signal.
It is the deferral branch left open by ADR-021.

## Context

Verified current state (file:line evidence, from FND-04):

- **Core Arrow exists but is disconnected.** `nodes_to_record_batch()`
  (`src/columnar.rs:22`) produces a `RecordBatch` (id UInt64 +
  `vector_d0..N` Float32Array per dimension), but the only callers are tests
  (`tests/logic/columnar.rs:8,23`) and no public SDK API exposes it to
  bindings. There is no `arrow::ipc` writer usage anywhere in the repo
  (grep: 0 matches). The real SDK wire format is postcard + JSONL
  (`src/sdk/serialization/`), not Arrow IPC.
- **Python input is already zero-copy:** `extract_vector()` via `PyBuffer`
  (NumPy, memoryview, bytes, bytearray, array.array)
  (`vantadb-python/src/convert.rs:177-221`) and `FlatBufferView`
  (`vantadb-python/src/types.rs:18-40`).
- **Python output uses a deliberate safe copy:** the `.vector` getter returns
  an **owned `PyBytes`** (`vantadb-python/src/vector.rs:59-83`) — the
  SEC-01/AUDIT-01 anti-UAF fix. Cost today: `Vec<f32>` → `PyBytes` (owned,
  safety) → numpy.array = **2 copies**. Reverting to true zero-copy
  reintroduces the UAF that AUDIT-01 fixed.
- **WASM/TS output is already zero-copy (PERF-08):**
  `vantadb-wasm/src/lib.rs:1428-1447` emits `vector` as a
  `js_sys::Float32Array` via bulk `copy_from` (NaN/Inf sanitized to `0.0`);
  `vantadb-ts` inherits it. WASM **input** remains pending (`from_js`,
  `vantadb-wasm/src/lib.rs:1462-1474`) — a parsing issue, not vector
  serialization.
- **Node native is the worst path:** `list`/`search` serialize the full
  `VantaMemoryRecord` (including vector) through
  `serde_json::to_value` (`vantadb-node/src/lib.rs:136-162`). napi-rs
  supports native `TypedArray` zero-copy (same pattern as PERF-08, ~2h
  binding-only change), but the rest of the record would still go through
  JSON.
- **Interop option explored:** `arrow::ffi` (arrow 59.2.0, `ffi` feature) +
  `pyarrow.Array._import_from_c` is the official mechanism (Apache Arrow C
  Data Interface), but it requires client-side pyarrow (~90MB wheel, not in
  the stack) and the core `RecordBatch` covers only id + vectors, not the
  full `VantaMemoryRecord` shape.
- **Current volumes do not justify the cost:** typical retrieval top-k is
  10-100 records × 384-1536 dims = 15KB-614KB of vectors; copies run in
  µs-ms, dominated by HNSW traversal in ms. The mass case (>40MB of vectors
  per response) does not flow through `search`/`list` today.

## Decision

**Defer zero-copy Arrow in Python/Node bindings. Keep the current per-binding
serialization:** Python output with the deliberate owned `PyBytes` copy,
Node native with full `serde_json` serialization, WASM input as-is.
No code change in this ADR.

Rationale:

1. **RecordBatch disconnected and of partial schema.** The only core
   `RecordBatch` (`src/columnar.rs:22`) holds id + flattened vectors — it
   does not cover namespace/key/payload/metadata of `VantaMemoryRecord`.
   Full zero-copy Arrow requires a **new core API** that emits `RecordBatch`
   from `search`/`list` — core SDK work, out of bindings scope.
2. **Python input is already zero-copy** (`extract_vector` PyBuffer,
   `FlatBufferView`); the only remaining cost is output, and it is a
   **deliberate security copy** (SEC-01/AUDIT-01: owned `PyBytes` anti-UAF).
   Reverting reintroduces memory-safety risk.
3. **The hot JS path is already resolved by PERF-08** (Float32Array
   zero-copy in WASM, inherited by TS). The binding with the worst
   serialization (Node native) is not the most-used path — TS wraps WASM.
4. **pyarrow is not a desired dependency** (~90MB wheel, not in the stack).
   The correct mechanism (`arrow::ffi` C Data Interface) requires client-side
   pyarrow and unsafe FFI at the boundary (Regla 2: requires audit).
5. **Current volumes do not justify the cost** (see Context).

This ADR resolves the pending item left open by ADR-021 ("FND-04 (Python/Node
zero-copy plan or deferral ADR) remain open") on the **deferral** branch.

### Future plan signed (implement only if the reopen signal appears)

**Node output via TypedArray (~2h, binding-only):**
1. In `vantadb-node/src/lib.rs` `list`/`search`: build a `Float32Array` for
   `vector` with `napi::bindgen_prelude::Float32Array::new_with_length` +
   `copy_from_slice` (same pattern as `vantadb-wasm/src/lib.rs:1428-1447`).
2. Keep payload/metadata/keys as JSON — only the vector changes shape.
3. Update TS types in `vantadb-ts/src/types.ts` to `Float32Array | number[]`
   (precedent in PERF-08).
4. Verify: `cargo check -p vantadb-node`,
   `cargo clippy -p vantadb-node --all-targets -- -D warnings`,
   `cargo fmt --check`, plus a Node shape test.

**WASM input pending** (`from_js`, `vantadb-wasm/src/lib.rs:1462-1474`) is a
parsing problem, not vector serialization; keep as-is until there is a cost
signal.

## Consequences

- **Positive:** no behavior change; the AUDIT-01 security posture is
  maintained (no UAF regression); no new heavy dependency (pyarrow); the
  decision is recorded with a measurable reopen signal instead of being
  re-litigated.
- **Negative:** Python output still pays 2 copies of `Vec<f32>`; Node still
  JSON-serializes vectors (P2-7 debt: `src/sdk/serialization/` full-copy
  path); WASM input zero-copy remains pending.
- **Deferred (reopen signal):** reopen FND-04 (or a derived task) if
  **any** of the following measurable thresholds is met:
  1. **Large query benchmark:** `search_memory`/`list_memory` with
     `top_k=10_000`, 1536-dim vectors, in a namespace with **≥1M records**
     (≥40MB of vectors per response) shows serialization/boundary overhead
     (Rust `Vec` → numpy/JSON) **>30% of total query time**.
  2. **Product requirement:** mass analytical interop
     (pandas/polars/dataframe) becomes a product requirement → the pyarrow
     C Data Interface returns to the table (requires a new core API + a
     dependency decision on pyarrow).
  3. **Node profiling:** `serde_json::to_value` in `list`/`search` accounts
     for **>30% of query time** with payloads **>10MB** → implement option B
     (Node TypedArray, ~2h) directly.

## Related

- ADR-021 — zero-copy Arrow architecture record (this ADR is the deferral
  branch it left open for FND-04).
- FND-04 report: `docs/research/FND-04-arrow-zero-copy.md` (evidence
  source for this decision).
- PERF-08 — WASM `Float32Array` zero-copy precedent
  (`vantadb-wasm/src/lib.rs:1428-1447`).
- SEC-01/AUDIT-01 — owned `PyBytes` copy fix
  (`vantadb-python/src/vector.rs:59-83`).
- P2-7 — `src/sdk/serialization/` full-copy path debt (AGENTS.md Regla 6
  table).
