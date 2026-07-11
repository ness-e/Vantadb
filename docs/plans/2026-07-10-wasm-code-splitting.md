# WASM Code Splitting Plan

**Date**: 2026-07-10
**Status**: Draft
**Owner**: Platform Team

---

## 1. Problem

The current `vantadb-wasm` crate (`vantadb-wasm/Cargo.toml`) compiles as a single `cdylib` that
pulls in the entire `vantadb` dependency tree including:

- **tantivy** (BM25 tokenizer, `~2MB` in WASM via `advanced-tokenizer` feature)
- **Graph traversal** (BFS/DFS/topo/DAG, always compiled — not feature-gated)
- **IQL executor** (`nom`-based query parser)

A user who only needs vector CRUD (`put`/`get`/`delete`/`list`) downloads the same ~1MB+ binary as
someone running full hybrid search. This is wasteful on bandwidth, parse time, and memory.

## 2. Goal

Split into **three separate WASM crates**, each loadable independently from JavaScript:

| Crate | Contains | Key feature flags on `vantadb` |
|-------|----------|--------------------------------|
| `vantadb-wasm-core` | HNSW + WAL + CRUD (put/get/delete/list/backup/export/import/flush/compact/purge/metrics/config) | `wasm` (default-features=false) |
| `vantadb-wasm-text` | Everything in core + BM25 (tantivy) + search + search_vector + explain + snippet + audit + repair | `wasm`, `advanced-tokenizer` |
| `vantadb-wasm-graph` | Everything in core + GraphRAG + IQL + BFS/DFS/topological/DAG + node/edge CRUD | `wasm` |

### 2.1 Method split (responsibility per crate)

| `#[wasm_bindgen]` method | core | text | graph |
|--------------------------|------|------|-------|
| `new` / `open` / `close` | ✓ | ✓ | ✓ |
| `connect_persistent` / `connect_idb` / `connect_worker` | ✓ | ✓ | ✓ |
| `save` / `save_idb` / `load` / `load_idb` / `delete_idb` | ✓ | ✓ | ✓ |
| `put` / `put_batch` / `get` / `delete` | ✓ | ✓ | ✓ |
| `list_namespaces` / `list` | ✓ | ✓ | ✓ |
| `flush` / `compact_wal` / `purge_expired` | ✓ | ✓ | ✓ |
| `rebuild_index` / `compact_layout` | ✓ | ✓ | ✓ |
| `export_namespace` / `export_all` / `import_records` / `import_file` | ✓ | ✓ | ✓ |
| `capabilities` / `operational_metrics` | ✓ | ✓ | ✓ |
| `worker_read` / `worker_write` / `worker_delete` | ✓ | ✓ | ✓ |
| `query` (IQL) | | | ✓ |
| `insert_node` / `get_node` / `delete_node` / `add_edge` | | | ✓ |
| `graph_bfs` / `graph_dfs` / `graph_topological_sort` / `graph_is_dag` | | | ✓ |
| `search` (vector/lexical/hybrid) | | ✓ | |
| `search_vector` | | ✓ | |
| `explain_memory_search` | | ✓ | |
| `generate_snippet` | | ✓ | |
| `audit_text_index` / `audit_text_index_deep` | | ✓ | |
| `repair_text_index` | | ✓ | |

> **Note**: `rebuild_index` (in core) already calls `rebuild_text_index_with_report()` internally on
> the Rust side — no method changes needed. If the text WASM isn't loaded, the text-index step
> is a no-op (no text data exists).

> **Note**: `search_vector` (`VantaEmbedded::search_vector` in `src/sdk/api.rs` lines 447–469) is a
> raw HNSW k-NN helper that bypasses BM25 entirely. It lives in `vantadb-wasm-text` but the
> underlying HNSW/search logic is always compiled — only the WASM binding is scoped to text.

## 3. Implementation approach

### 3.1 File structure (proposed)

```
vantadb-wasm/
├── Cargo.toml                         # ← existing, kept as umbrella (not used directly by TS)
├── src/lib.rs                         # ← existing, deprecated (or becomes empty re-export)
│
├── vantadb-wasm-core/
│   ├── Cargo.toml
│   └── src/lib.rs                     # VantaDBCore — CRUD + WAL + maintenance only
│
├── vantadb-wasm-text/
│   ├── Cargo.toml
│   └── src/lib.rs                     # VantaDBText — core + BM25 + search + snippet
│
├── vantadb-wasm-graph/
│   ├── Cargo.toml
│   └── src/lib.rs                     # VantaDBGraph — core + IQL + graph traversal
│
├── pkg/                               # wasm-pack output (per crate)
│   ├── core/
│   ├── text/
│   └── graph/
│
└── shared/                            # (optional) shared WASM helpers
    └── mod.rs                         # memory_record_to_js, from_js, to_js, to_js_err, WasmConfig, etc.
```

### 3.2 Crate dependency diagram

```
vantadb-wasm-core ──── vantadb (wasm)
vantadb-wasm-text  ──── vantadb (wasm + advanced-tokenizer)
vantadb-wasm-graph ──── vantadb (wasm)
```

Each crate defines its own `#\[wasm_bindgen\] struct VantaDB` (named `VantaDBCore`, `VantaDBText`,
`VantaDBGraph` or all just `VantaDB` with separate npm packages — TBD).

### 3.3 Duplication of shared Rust code

The helper free functions in the current `lib.rs` must be available in each crate:

- `to_js_err` (line 840)
- `memory_record_to_js` (line 844)
- `from_js` (line 894)
- `to_js` (line 898)
- `init_tracing` (line 833)
- `WasmConfig` / `build_config` (lines 38–66)
- `MemoryInput` / `SearchRequest` / `ListOptions` / `JsNodeRecord` / `JsOperationalMetrics` (lines 69–240)

**Option A** (recommended): Extract into a `vantadb-wasm-shared` crate or a shared module file
under `vantadb-wasm/shared/` that each crate `include!()`s.

**Option B**: Copy-paste (acceptable given the small surface; less indirection).

### 3.4 npm / TypeScript side

The JS `VantaDB` class (`vantadb-ts/src/vantadb.ts`) stays **unchanged** as the unified API.
It imports the correct WASM binding at construction time using a loader:

```typescript
// vantadb-ts/src/vantadb.ts — conceptual change
import { VantaDBCore } from "vantadb-wasm-core";
import { VantaDBText } from "vantadb-wasm-text";
import { VantaDBGraph } from "vantadb-wasm-graph";

export class VantaDB {
  private inner: VantaDBCore | VantaDBText | VantaDBGraph;

  static connect(options?: VantaConfig): VantaDB {
    const needs = detectCapabilities(options);
    const wasm = needs.text
      ? new VantaDBText(...)    // loads vantadb-wasm-text.wasm
      : needs.graph
        ? new VantaDBGraph(...) // loads vantadb-wasm-graph.wasm
        : new VantaDBCore(...);  // loads vantadb-wasm-core.wasm (smallest)
    return new VantaDB(wasm);
  }

  // Proxy methods — forward to inner (same shape as today)
  put(input: MemoryInput): MemoryRecord { return this.inner.put(input); }
  search(request: SearchRequest): SearchHit[] {
    if (!('search' in this.inner)) throw new VantaError("FEATURE_UNAVAILABLE", ...);
    return (this.inner as VantaDBText).search(request);
  }
  // ...
}
```

**Feature detection** is exposed via `capabilities()` (unchanged) — the TS wrapper can also do
static detection at `connect()` time based on the config/hint.

### 3.5 Cargo.toml per crate

**`vantadb-wasm-core/Cargo.toml`** (TBD paths):

```toml
[package]
name = "vantadb-wasm-core"
# ...
[lib]
crate-type = ["cdylib"]
[dependencies]
vantadb = { path = "../", default-features = false, features = ["wasm"] }
# ... wasm-bindgen, serde, serde-wasm-bindgen, serde_json, js-sys, etc.
```

**`vantadb-wasm-text/Cargo.toml`** (TBD paths):

```toml
[package]
name = "vantadb-wasm-text"
# ...
[dependencies]
vantadb = { path = "../", default-features = false, features = ["wasm", "advanced-tokenizer"] }
# ...
```

**`vantadb-wasm-graph/Cargo.toml`** (TBD paths):

```toml
[package]
name = "vantadb-wasm-graph"
# ...
[dependencies]
vantadb = { path = "../", default-features = false, features = ["wasm"] }
# ...
```

> **TBD**: Exact path for `vantadb` dependency — may need `path = "../../"` depending on final
> placement (nested vs flat within `vantadb-wasm/`).

## 4. Build flow

Each crate builds independently with `wasm-pack`:

```bash
# Core (smallest, ~400KB gzip)
wasm-pack build vantadb-wasm-core --target web --out-dir pkg/core

# Text (largest, ~1MB+ gzip, pulls tantivy)
wasm-pack build vantadb-wasm-text --target web --out-dir pkg/text

# Graph (~500KB gzip)
wasm-pack build vantadb-wasm-graph --target web --out-dir pkg/graph
```

Or via a workspace script:

```bash
# In workspace Cargo.toml, add members:
#   "vantadb-wasm/vantadb-wasm-core",
#   "vantadb-wasm/vantadb-wasm-text",
#   "vantadb-wasm/vantadb-wasm-graph"
```

**CI integration** (TBD): One `Makefile` / `justfile` target per crate, or a single `build:wasm`
that builds all three + copies into `vantadb-ts/` npm package.

**Release profile** (inherited from workspace `Cargo.toml` lines 561–564):
```toml
[profile.release.package.vantadb-wasm-core]
opt-level = "s"
strip = true
codegen-units = 1
# identical overrides for -text, -graph
```

## 5. Estimated effort: 1–2 days

| Step | Effort | Details |
|------|--------|---------|
| Scaffold 3 crate directories + Cargo.toml files | 1h | Copy `vantadb-wasm/Cargo.toml`, adjust features |
| Split `lib.rs` into 3 entry points + shared helpers | 2h | Extract `WasmConfig`, `from_js`, `to_js`, `to_js_err`, `memory_record_to_js`, etc. |
| Write `vantadb-wasm-core/src/lib.rs` | 1h | CRUD + WAL + export/import + maintenance only |
| Write `vantadb-wasm-text/src/lib.rs` | 1h | All core methods + search + snippet + audit + repair |
| Write `vantadb-wasm-graph/src/lib.rs` | 1h | All core methods + IQL + graph traversal + node/edge ops |
| Update `vantadb-ts` to use the loader + 3 packages | 2h | `vantadb.ts` wrapper, detect features at connect time |
| Update `Cargo.toml` workspace members | 15m | Add 3 new members |
| Build & verify binary sizes | 1h | `wasm-pack build` each, compare `.wasm` gzip |
| Update CI / Makefile / npm scripts | 30m | Add `build:wasm:core`, `build:wasm:text`, `build:wasm:graph` |
| **Total** | **~10h** | 1–2 calendar days |

## 6. Non-goals

1. **Do NOT change the public JS API.** `VantaDB` class (`vantadb-ts/src/vantadb.ts`) stays the
   same — all existing methods, signatures, and error types remain. Only the internal WASM loading
   changes.
2. **Do NOT refactor the Rust SDK.** `VantaEmbedded` in `src/sdk/` stays monolithic. Splitting is
   at the WASM binding layer only.
3. **Do NOT add new features.** No new methods, no new types, no new npm packages visible to
   consumers. The npm package name (`vantadb` / `vantadb-ts`) stays the same; the three crates are
   an implementation detail.
4. **Do NOT gate graph behind a Cargo feature** in the root `vantadb` crate. Graph code is always
   compiled; the WASM binding layer is where the split happens. For text, the `advanced-tokenizer`
   (tantivy) feature already exists and is the lever.

## 7. Open questions / TBD

- [ ] **Naming**: Should the three `#[wasm_bindgen]` structs share the name `VantaDB` (each crate
  exposes `VantaDB` and the TS loader picks the right one) or be distinct (`VantaDBCore`,
  `VantaDBText`, `VantaDBGraph`)?
- [ ] **npm packages**: One package per crate (`vantadb-wasm-core`, `vantadb-wasm-text`,
  `vantadb-wasm-graph`) or a single `vantadb-wasm` umbrella with sub-path imports?
- [ ] **Feature detection at connect time**: Should the TS wrapper auto-detect which WASM to load
  based on the config object (e.g., `enableTextSearch: true`), or always load the explicit variant
  the user requested?
- [ ] **Shared helpers**: Extract into `vantadb-wasm/shared/mod.rs` with `include!()` or duplicate
  per crate?
- [ ] **WASM feature flags**: Does the `wasm` feature in `vantadb` need new sub-features
  (`wasm-core`, `wasm-text`, `wasm-graph`) to help dead-code elimination, or is the current
  always-compile approach sufficient?

## 8. Appendix: Current size breakdown (estimated)

Based on `cargo bloat` and tantivy's known WASM footprint:

| Component | Estimated `.wasm` size | `.wasm.gz` |
|-----------|----------------------|------------|
| Core (HNSW + WAL + CRUD + serde) | ~800KB | ~350KB |
| Text (+ tantivy + tokenizer) | ~2MB | ~800KB |
| Graph (+ IQL parser + traversal) | ~900KB | ~400KB |
| **Monolithic (current)** | **~2.8MB** | **~1.1MB** |
| **Core-only (after split)** | **~800KB** | **~350KB** |

These are rough estimates — actual savings depend on LTO and wasm-opt effectiveness.
