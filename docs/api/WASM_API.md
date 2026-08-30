---
title: WASM API Reference
type: api
status: active
tags: [vantadb, wasm, browser, api]
last_reviewed: 2026-08-30
aliases: [WASM_API]
---

# WASM API Reference

This page is the canonical entry point for VantaDB's WebAssembly surface — the
`vantadb-wasm` crate compiled to `wasm32-unknown-unknown` and consumed from
JavaScript / TypeScript. The WASM API has three documentation layers; this
page is the index that ties them together and documents cross-cutting
semantics (notably the **score vs distance** convention, WSM-10).

## Doc layers

| Layer | Lives in | Purpose |
|---|---|---|
| **Binding types (canonical WASM surface)** | [`vantadb-wasm/src/vantadb_wasm.dts`](../../vantadb-wasm/src/vantadb_wasm.d.ts) | Hand-written TypeScript declarations of every wasm-bindgen export. Source of truth for the JS-visible shape. |
| **Wrapper API (TS ergonomics layer)** | [`TS_SDK.md`](TS_SDK.md) | The `vantadb-ts` package — typed methods, async wrappers, sub-client accessors. |
| **Runtime (browser console build)** | [`WASM_STANDALONE.md`](WASM_STANDALONE.md) + [`WASM_PERSISTENCE.md`](WASM_PERSISTENCE.md) | How to build / run the standalone browser console; OPFS / IndexedDB / Worker backends. |

If you are calling the WASM binding directly from raw JavaScript (no
`vantadb-ts` wrapper), use the binding-types layer. If you are writing a
TypeScript app, the wrapper API is the recommended path.

## Score vs distance semantics (WSM-10)

VantaDB exposes two distinct result types across its WASM / TS / Node
transports, and the field name carries semantic weight. **Read this section
before writing code that compares or sorts hits.**

### The two result types

| Result type | Source APIs | Field convention | Math |
|---|---|---|---|
| `VantaMemorySearchHit` (memory / hybrid search) | `search()`, `similar_to_key()`, `search_multi()` (all transports) | **`score`** — higher is more relevant | BM25 (text), cosine similarity ∈ [-1.0, 1.0], RRF-fused (vector + text). Pinned by `src/sdk/serialization/vector_types.rs::tests`. |
| `VantaSearchHit` (raw ANN vector search) | `search_vector()` (core) → WASM `search_vector()` → TS wrapper `searchVector()` | **`distance`** — lower is more similar | Raw L2 / cosine distance. No sign flip, no similarity transform. |

### Per-transport field map

| Transport | API | Field on hit | Convention | Notes |
|---|---|---|---|---|
| Rust core (`vantadb`) | `VantaMemorySearchHit` | `score` | higher is better | `[-1.0, 1.0]` cosine; `(-∞, 0.0]` Euclidean; BM25 ≥ 0; RRF-fused ≥ 0 |
| Rust core (`vantadb`) | `VantaSearchHit` (raw ANN) | `distance` | lower is better | `[0.0, +∞)` |
| WASM binding (`vantadb-wasm`) | `SearchHit` (from `search` / `similar_to_key`) | `score` | higher is better | Mirrors `VantaMemorySearchHit` |
| WASM binding (`vantadb-wasm`) | `search_vector()` return | **`distance`** *(WSM-10)* | lower is better | Was mislabeled `score` before WSM-10 — fixed 2026-08-30 |
| TypeScript wrapper (`vantadb-ts`) | `SearchHit` | `distance` | **lower is better** *(inverted from WASM)* | CODE-091: pinned in CI; consumers must invert comparison when porting |
| TypeScript wrapper (`vantadb-ts`) | `searchVector()` return | `distance` | lower is better | Mirrors WASM `search_vector()` |
| Node binding (`vantadb-node`) | `MemorySearchHit` | `score` | higher is better | Same convention as Rust core |
| Python binding (`vantadb-python`) | `hit.score` | higher is better | Same convention as Rust core |
| HTTP API (`POST /api/v2/search`) | `score` | higher is better | Same convention as Rust core |

### Which to use when

- Use **`search()`** (returns `score`) when you want ranking by relevance
  across text + vector channels, and you want the field to mean "higher is
  better" the way every other rank-style metric in your stack does.
- Use **`search_vector()`** (returns `distance`) when you need raw nearest-
  neighbor geometry — for example, thresholding by a distance cutoff, or
  computing your own similarity transform downstream.

### Cross-binding pointer

The full rationale and the pinned-CI tests live in
[`TS_SDK.md` → "Distance vs Score (CODE-091)"](TS_SDK.md#distance-vs-score-code-091).
The Node-side rationale is documented inline in
[`vantadb-node/src/lib.rs` `search()` docstring](../../vantadb-node/src/lib.rs).
The cross-binding parity note for the bindings namespace map is in
[`BINDINGS_NAMESPACES.md`](BINDINGS_NAMESPACES.md#score-vs-distance-convention-code-091--wsm-10).

## When to read what

- **Building a TS app on top of `vantadb-ts`?** Start at
  [`TS_SDK.md`](TS_SDK.md). The score-vs-distance section is at the top of
  the `search()` reference.
- **Calling the WASM binding directly from JS?** Read this file, then the
  binding types file [`vantadb-wasm/src/vantadb_wasm.d.ts`](../../vantadb-wasm/src/vantadb_wasm.d.ts).
- **Building the standalone browser console?** See
  [`WASM_STANDALONE.md`](WASM_STANDALONE.md).
- **Persistence backends (OPFS / IDB / Worker)?** See
  [`WASM_PERSISTENCE.md`](WASM_PERSISTENCE.md).
- **Cross-binding parity, gaps, or scoring semantics across all transports?**
  See [`BINDINGS_NAMESPACES.md`](BINDINGS_NAMESPACES.md).