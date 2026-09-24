---
title: "ADR-021: Zero-copy Arrow in bindings — core exposes Arrow, bindings vary"
type: adr
status: accepted
tags: [vantadb, architecture, adr, serialization, arrow, wasm, python, node, zero-copy]
created: 2026-08-16
last_reviewed: 2026-08-16
related: []
---

# ADR-021: Zero-copy Arrow in bindings — core exposes Arrow, bindings vary

## Status

Accepted (retroactive). Documents the de-facto serialization architecture and
the per-binding zero-copy state (PERF-08 done; Python/Node pending — FND-04).

## Context

VantaDB needs to ship vectors/records across bindings (WASM, Python, Node)
efficiently. The project's approach evolved in code without a single ADR
pinning it down: the core exposes an Apache Arrow columnar conversion, and each
binding decides independently how much zero-copy it achieves. The result was
inconsistent — WASM got a zero-copy hot path (PERF-08) while Python/Node still
serialize via full copies (P2-7 debt, FND-04 pending).

## Decision

**The core exposes Arrow as the analytical interchange format; bindings adopt
zero-copy incrementally, per-binding, with full serialization as the safe
fallback.** Concretely:

1. **Core Arrow module (default feature):** `arrow` is a default Cargo feature
   (`Cargo.toml:97`), gated module `src/columnar.rs` (`src/lib.rs:73-74`).
   `nodes_to_record_batch` (`src/columnar.rs:22`) converts `UnifiedNode`
   collections into a `RecordBatch` with one flat `Float32Array` column per
   vector dimension (`src/columnar.rs:37-48`), documented as "enabling
   zero-copy analytical scans and Python/Polars interop" (`src/columnar.rs:1-4`).
2. **WASM output zero-copy — DONE (PERF-08):** `vantadb-wasm/src/lib.rs:1428-1447`
   emits `vector` as a `js_sys::Float32Array` via bulk `copy_from` (with
   NaN/Inf sanitized to `0.0`) instead of letting `serde_wasm_bindgen` build a
   JS `number[]` element-by-element (`lib.rs:1469-1474` comment: "same shape;
   avoids N allocs/Reflect per vector on the search/list/put hot path").
3. **WASM input zero-copy — PENDING:** `from_js` still uses
   `serde_wasm_bindgen::from_value` (`vantadb-wasm/src/lib.rs:1462-1464`);
   input zero-copy "requires touching MemoryInput/VantaNodeInput parsing
   (outside PERF-08 scope)" (`lib.rs:1473-1474`).
4. **Python/Node zero-copy — PENDING (FND-04):** `vantadb-python/` has no Arrow
   usage in its Rust bindings (`rg "arrow" vantadb-python/src/` → none; only
   `vantadb_py/migrate/lancedb.py:70` consumes *external* `to_arrow()` from
   LanceDB). Node bindings likewise show no Float32Array/Arrow path
   (`rg "arrow|Float32Array" vantadb-node/` → none). These still copy through
   the full serialization path (`src/sdk/serialization/`, P2-7 debt: "full
   serialization without zero-copy path").

## Consequences

- **Positive:** the hot WASM search/list/put path avoids per-f32 allocations and
  `Reflect` calls (`vantadb-wasm/src/lib.rs:1428-1447`); the Arrow core module
  is a stable interchange point for future Polars/Pandas/Numpy interop.
- **Negative:** cross-binding inconsistency — WASM output is zero-copy but WASM
  input and Python/Node are not; `arrow` v59 is a heavy default dependency for
  the core crate.
- **Debt:** P2-7 (`src/sdk/serialization/` full-copy path) and FND-04
  (Python/Node zero-copy plan or deferral ADR) remain open; the WASM input
  zero-copy is tracked in the `lib.rs:1473-1474` ponytail comment.
- **Guardrail:** the per-binding decision must stay explicit — a binding either
  documents its zero-copy status here or in its own ADR when it diverges.

## Related

- P2-7 (AGENTS.md Regla 6 debt table), PERF-08 (WASM Float32Array zero-copy),
  FND-04 (Python/Node zero-copy — pending backlog task).