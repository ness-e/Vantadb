---
title: WASM API Reference
kind: reference
status: active
description: "This page is the canonical entry point for VantaDB's WebAssembly surface — the"
aliases: [WASM_API]
tags: [vantadb, wasm, browser, api]
---

# WASM API Reference

> **Naming (ADR-0047 anti-stutter):** canonical names are `Client`
> (legacy `VantaDB` alias removed in 0.6.0, AST-010), `Config` (legacy
> `VantaConfig` removed), `SearchHit` / `MemorySearchHit` (legacy
> `VantaSearchHit` / `VantaMemorySearchHit` removed).
> `VANTADB_*` error codes (wire) are intentionally unchanged.

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
| `MemorySearchHit` (memory / hybrid search) | `search()`, `similar_to_key()`, `search_multi()` (all transports) | **`score`** — higher is more relevant | BM25 (text), cosine similarity ∈ [-1.0, 1.0], RRF-fused (vector + text). Pinned by `src/sdk/serialization/vector_types.rs::tests`. |
| `SearchHit` (raw ANN vector search) | `search_vector()` (core) → WASM `search_vector()` → TS wrapper `searchVector()` | **`distance`** — lower is more similar | Raw L2 / cosine distance. No sign flip, no similarity transform. |

### Per-transport field map

| Transport | API | Field on hit | Convention | Notes |
|---|---|---|---|---|
| Rust core (`vantadb`) | `MemorySearchHit` | `score` | higher is better | `[-1.0, 1.0]` cosine; `(-∞, 0.0]` Euclidean; BM25 ≥ 0; RRF-fused ≥ 0 |
| Rust core (`vantadb`) | `SearchHit` (raw ANN) | `distance` | lower is better | `[0.0, +∞)` |
| WASM binding (`vantadb-wasm`) | `SearchHit` (from `search` / `similar_to_key`) | `score` | higher is better | Mirrors `VantaMemorySearchHit` |
| WASM binding (`vantadb-wasm`) | `search_vector()` return | **`distance`** *(WSM-10)* | lower is better | Was mislabeled `score` before WSM-10 — fixed 2026-08-30 |
| TypeScript wrapper (`vantadb-ts`) | `SearchHit` | `score` | higher is better | **W1/API-02**: the pre-W1 `distance` rename (CODE-091) was removed — TS now matches every other transport |
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

## v2 records and query params (SCH-07, ADR-0046)

The WASM binding carries the v2 memory wire natively:

- **`MemoryRecord`** (get/put/list/search): `valid_at_ms`, `invalid_at_ms`,
  `confidence_class`, `confidence`, `last_validated_at_ms`, `derived_from`,
  `quarantined_at_ms`, `quarantine_reason`, `quarantined_by`,
  `quarantine_review_due_ms`. Optionals are omitted when absent; u64
  timestamps travel as decimal strings (policy string-u64).
- **`SearchRequestInput` / `ListOptionsInput`**: `as_of_ms`, `valid_window`
  (`{from_ms, to_ms}` half-open), `include_quarantined`, `min_confidence` —
  same wire names as the SDK (semantics in
  [`EMBEDDED_SDK.md`](EMBEDDED_SDK.md) → v2 search options).
- The selective-abstention signal (`abstained`, `abstention_reason`) is
  page-shaped; the WASM `search()` returns an array and does not carry it —
  declared parity note in
  [`BINDINGS_NAMESPACES.md`](BINDINGS_NAMESPACES.md#v2-wire-parity-sch-07).

### Cross-binding pointer

The full rationale and the pinned-CI tests live in
[`TS_SDK.md` → "Score, not distance (W1/API-02)"](TS_SDK.md#score-not-distance-w1api-02-supersedes-code-091).
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