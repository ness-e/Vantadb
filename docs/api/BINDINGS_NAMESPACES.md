---
title: "Bindings Namespace Map"
type: api
status: active
tags: [vantadb, api, bindings, namespaces]
last_reviewed: 2026-09-15
aliases: []
related: []
---

# Bindings Namespace Map

> **Status:** canonical contract for SDKB campaign (`docs/dev/plans/2026-08-22-vantadb-bindings-sdk.md`).
> **Decisions:** D42 (sub-clients TS/Python only — zero WASM/Rust changes), D43 (v1 groups already-exposed methods only; vanta-memory pipeline is core-only and deferred), D45 (additive → minor bump).
> **Rule:** every public method of every SDK maps to **exactly one** domain. `system` is the catch-all for orphan operations (capabilities, import/export, metrics, lifecycle).
>
> **Sub-client docs:** TypeScript examples in [`vantadb-ts/README.md` → Domain Sub-clients](../../vantadb-ts/README.md#domain-sub-clients) · Python examples in [`PYTHON_SDK.md` → Domain Sub-clients](./PYTHON_SDK.md#domain-sub-clients).

## Domain Taxonomy

| Domain | Covers |
|---|---|
| `memory` | Records in a namespace: put/get/list/search/supersede/TTL purge, text snippets over payloads |
| `graph` | Node/edge CRUD and traversals (BFS, DFS, topological sort, DAG check, PageRank, degree) |
| `conversation` | Reserved — L0–L3 context-engine pipeline lives in crate `vanta-memory`, **not exposed via bindings today** (D43) |
| `skills` | Reserved — no binding surface exists today (D43) |
| `wiki` | Summary/archive lifecycle over nodes (recover archived nodes). Full wiki features are core-only (D43) |
| `system` | Catch-all: constructors/lifecycle, capabilities/hardware profile, metrics, IQL query engine, index maintenance, compaction, import/export |

## Casing Contract (Gate P — API-01 foundation)

> **Status:** normative target declared in W0. The binding **migration** lands
> in W1 (`API-02`) and later waves; nothing here is retroactive yet. Source:
> `docs/dev/tasks/API-STD-15.md` §2 ("Casing"), decision approved in Gate P.

| Surface | Convention | Example |
|---|---|---|
| Rust identifiers (native interior) | `snake_case` fields/methods, `PascalCase` types | `node_id`, `MemoryRecord` |
| JSON payloads across bindings (WASM/Node/TS/MCP) — **target** | `camelCase` | `nodeId`, `createdAtMs`, `affectedNodes` |
| Python attributes/methods (native interior) | `snake_case` | `rec.node_id`, `db.put_batch(...)` |
| TypeScript/JS public API | `camelCase` | `putBatch`, `oldKey`, `topK` |
| CLI flags and subcommands | `kebab-case` | `--read-only`, `list-namespaces` |
| New MCP tools | tool name `kebab-case`, params `camelCase` | `memory-put`, `topK` |

Rules for the migration window:

- **One casing per payload.** Do not mix conventions in a single wire shape:
  a new field added to a still-snake_case payload stays `snake_case` until the
  whole payload migrates. Each DTO is internally consistent.
- **`node_id`/`id` as decimal strings** (u128 > 2^53) is casing-neutral and
  already shipped (API-01 Step 2) — `nodeId` renames are W1 work.
- **Native interiors never rename for wire polish.** Python `snake_case`
  attributes and Rust struct fields stay; only the JSON boundary changes.

## Score vs Distance Convention (CODE-091 / WSM-10)

> **Full per-transport map and rationale:**
> [`WASM_API.md` → "Score vs distance semantics (WSM-10)"](WASM_API.md#score-vs-distance-semantics-wsm-10).
> TS-side: [`TS_SDK.md` → "Distance vs Score (CODE-091)"](TS_SDK.md#distance-vs-score-code-091).
> Node-side: [`NODE_SDK.md` → Search § "Score is relevance, not a distance (WSM-10)"](NODE_SDK.md#search).
>
> **Naming (ADR-041 anti-stutter):** canonical names are `MemorySearchHit.score`
> (memory/hybrid search) and `SearchHit.distance` (raw ANN). Legacy aliases
> `VantaMemorySearchHit` / `VantaSearchHit` were removed in 0.6.0 (AST-010).

| Transport | Memory/hybrid search field | Raw ANN field | Convention |
|---|---|---|---|
| Rust core | `MemorySearchHit.score` | `SearchHit.distance` | `score` higher-is-better; `distance` lower-is-better |
| WASM binding (`vantadb-wasm`) | `SearchHit.score` | `search_vector()` → **`distance`** *(WSM-10, was `score` before)* | matches core |
| TypeScript wrapper (`vantadb-ts`) | `SearchHit.distance` *(inverted)* | `searchVector()` → `distance` | **`distance`** lower-is-better in both (CODE-091) |
| Node binding (`vantadb-node`) | `score` | (no raw ANN binding) | `score` higher-is-better |
| Python binding (`vantadb-python`) | `hit.score` | `(node_id, distance)` tuple | `score` higher-is-better; `distance` lower-is-better |
| HTTP API | `score` | n/a | `score` higher-is-better |

**When writing cross-binding code:** always read the field by name, never
assume the value semantics from the name alone. The TS wrapper is the only
binding that renames the field — every other transport exposes `score` for
relevance and `distance` for raw ANN distance.

## SDK Surface Differences (verified 2026-09-15 via grep en `vantadb-ts/src/vantadb.ts`)

| Capability | WASM | TS | Python |
|---|---|---|---|
| `supersede(namespace, old_key, new_key)` | ✅ | ✅ | ✅ |
| `count(namespace, filter?)` | ✅ | ✅ | ✅ |
| `similar_to_key(namespace, key, top_k)` | ✅ | ✅ | ✅ |
| `remove_edge(source_id, target_id, label)` | ✅ | ✅ | ✅ |
| `search_multi(namespaces, request)` | ✅ | ✅ | ❌ |
| `sparse_vector` on `put()` / `put_batch()` | ✅ | ✅ | ✅ |
| `exclude_superseded` on `search()` | ✅ | ✅ | ✅ |
| `exclude_superseded` on `list()` | ✅ (WSM-06) | ✅ | ❌ |
| `filter_ops` on `search()` | ❌ (core limitation: flat `filters` only) | ❌ | ❌ |
| `search_profile` on `search()` | ❌ (advanced, internal `None`) | ❌ | ❌ |
| Node CRUD by explicit id (`insert_node`/`get_node`/`delete_node`) | ✅ | ✅ | ⚠️ via `insert`/`get`/`delete` (`id: u128`) |
| `graph_page_rank` / `graph_degree_centrality` | ❌ (has `graph_degree`) | ❌ (has `graphDegree`) | ✅ both |
| `delete_by_filter` / `search_vector` / `audit_text_index_deep` / `export_namespace_filtered` / `import_records` | ✅ | ✅ | ⚠️ `delete_by_filter` ✅; `search_vector` / `audit_text_index_deep` / `export_namespace_filtered` / `import_records` ❌ |
| `bulk_import` / `bulk_import_bytes` | ✅ | ❌ | ✅ |
| `put_batch_raw` / `search_batch` / `search_batch_requests` / `hardware_profile` | ❌ | ❌ | ✅ |
| `recover_archived_nodes(summary_id)` | ❌ | ❌ | ✅ (wiki) |
| Hybrid search request shape | `search(request)` | `search(SearchRequest)` | `search(vector)` pure ANN + separate `explain_memory_search` |

⚠️ **Naming hazard:** `get`/`delete` are memory-record ops (namespace+key) in WASM/TS but **node-level ops (`id: u128`, graph domain)** in Python. Sub-client design must not blindly mirror names across SDKs.

## WASM (`vantadb-wasm/src/lib.rs`) — 47 pub fns

| Method | Domain | Notes |
|---|---|---|
| `new` | system | constructor |
| `open` | system | constructor |
| `close` | system | lifecycle |
| `put` | memory | supports `sparse_vector` (WSM-06) |
| `put_batch` | memory | supports `sparse_vector` (WSM-06) |
| `get` | memory | namespace+key |
| `delete` | memory | namespace+key |
| `delete_by_filter` | memory | full operator `filter_ops` |
| `list` | memory | supports `exclude_superseded` (WSM-06) |
| `list_namespaces` | memory | |
| `search` | memory | hybrid request; supports `exclude_superseded` |
| `search_vector` | memory | pure ANN |
| `search_multi` | memory | cross-namespace hybrid search |
| `similar_to_key` | memory | vector search from existing key |
| `count` | memory | optional operator filter |
| `supersede` | memory | mark record as superseded |
| `explain_memory_search` | memory | explain plan |
| `generate_snippet` | memory | text highlight over payload |
| `purge_expired` | memory | TTL housekeeping |
| `insert_node` | graph | |
| `get_node` | graph | |
| `delete_node` | graph | |
| `add_edge` | graph | |
| `remove_edge` | graph | |
| `graph_bfs` | graph | |
| `graph_dfs` | graph | |
| `graph_topological_sort` | graph | |
| `graph_is_dag` | graph | |
| `graph_filtered_traversal` | graph | |
| `graph_degree` | graph | |
| `capabilities` | system | |
| `operational_metrics` | system | |
| `query` | system | IQL engine |
| `flush` | system | durability flush |
| `compact_wal` | system | WAL maintenance |
| `compact_layout` | system | storage layout |
| `rebuild_index` | system | index maintenance |
| `reindex_hnsw_from_text` | system | index maintenance |
| `repair_text_index` | system | index maintenance |
| `audit_text_index` | system | index diagnostics |
| `audit_text_index_deep` | system | index diagnostics |
| `export_all` | system | portability |
| `export_namespace` | system | portability |
| `export_namespace_filtered` | system | portability |
| `import_file` | system | portability |
| `import_records` | system | portability |
| `bulk_import` | system | portability |
| `bulk_import_bytes` | system | portability |

**Totals:** memory 15 · graph 11 · system 21 = 47 ✔

## TypeScript (`vantadb-ts/src/vantadb.ts`) — 43 public methods (verificado 2026-09-15: +`count`/`supersede`/`similarToKey`/`searchMulti`/`removeEdge`)

(`native.ts` implements the sync subset: `capabilities`, `close`, `delete`, `flush`, `get`, `list`, `listNamespaces`, `put`, `putBatch`, `search`.)

| Method | Domain | Exposed today | Notes |
|---|---|---|---|
| `put` | memory | ✅ | delegates to wasm `put` |
| `putBatch` | memory | ✅ | |
| `get` | memory | ✅ | namespace+key |
| `delete` | memory | ✅ | namespace+key |
| `deleteByFilter` | memory | ✅ | |
| `list` | memory | ✅ | |
| `listNamespaces` | memory | ✅ | |
| `search` | memory | ✅ | hybrid request |
| `searchVector` | memory | ✅ | pure ANN |
| `explainSearch` | memory | ✅ | wraps wasm `explain_memory_search` |
| `generateSnippet` | memory | ✅ | |
| `purgeExpired` | memory | ✅ | TTL housekeeping |
| `count` | memory | ✅ | optional operator filter |
| `supersede` | memory | ✅ | mark record as superseded |
| `similarToKey` | memory | ✅ | vector search from existing key |
| `searchMulti` | memory | ✅ | cross-namespace hybrid search |
| `insertNode` | graph | ✅ | |
| `getNode` | graph | ✅ | |
| `deleteNode` | graph | ✅ | |
| `addEdge` | graph | ✅ | |
| `removeEdge` | graph | ✅ | |
| `graphBfs` | graph | ✅ | |
| `graphDfs` | graph | ✅ | |
| `graphTopologicalSort` | graph | ✅ | |
| `graphIsDag` | graph | ✅ | |
| `graphFilteredTraversal` | graph | ✅ | |
| `graphDegree` | graph | ✅ | |
| `close` | system | ✅ | lifecycle |
| `capabilities` | system | ✅ | |
| `operationalMetrics` | system | ✅ | |
| `query` | system | ✅ | IQL |
| `flush` | system | ✅ | |
| `compactWal` | system | ✅ | |
| `compactLayout` | system | ✅ | |
| `rebuildIndex` | system | ✅ | |
| `reindexHnswFromText` | system | ✅ | |
| `repairTextIndex` | system | ✅ | |
| `auditTextIndex` | system | ✅ | |
| `auditTextIndexDeep` | system | ✅ | |
| `exportAll` | system | ✅ | |
| `exportNamespace` | system | ✅ | |
| `importRecords` | system | ✅ | |
| `importFile` | system | ✅ | |

**Totals:** memory 16 · graph 11 · system 16 = 43 ✔

**Not exposed in TS (wasm-only or Python-only), deferred per D43/D42:** `graph_page_rank`/`graph_degree_centrality` (Python-only), `bulk_import`/`bulk_import_bytes` (wasm/Python-only), `hardware_profile` (Python-only), `recover_archived_nodes` (Python-only). Do NOT add wrappers in SDKB-02 — v1 is grouping only.

## Python (`vantadb-python/src/lib.rs`) — 44 pyclass methods (+ module-level `connect()`)

| Method | Domain | Exposed today | Notes |
|---|---|---|---|
| `new` | system | ✅ | constructor |
| `connect` *(module fn)* | system | ✅ | alias of `new` |
| `insert` | graph | ✅ | node insert by explicit id |
| `get` | graph | ✅ | **node get by `id: u128`** (≠ TS semantics) |
| `delete` | graph | ✅ | **node delete by id** (≠ TS semantics) |
| `add_edge` | graph | ✅ | |
| `graph_bfs` | graph | ✅ | |
| `graph_dfs` | graph | ✅ | |
| `graph_topological_sort` | graph | ✅ | |
| `graph_is_dag` | graph | ✅ | |
| `graph_page_rank` | graph | ✅ | Python-only |
| `graph_degree_centrality` | graph | ✅ | Python-only (= wasm `graph_degree`) |
| `put` | memory | ✅ | also on `db.memory` |
| `put_batch` | memory | ✅ | also on `db.memory` |
| `put_batch_raw` | memory | ✅ | Python-only, also on `db.memory` |
| `delete_by_filter` | memory | ✅ | operator filter_ops (flat → `$eq`, or `{"$op": value}`), also on `db.memory` |
| `count` | memory | ✅ | optional operator filter, also on `db.memory` |
| `similar_to_key` | memory | ✅ | vector search from an existing key, also on `db.memory` |
| `search` | memory | ✅ | hybrid (ex-`search_memory`, AST-008), also on `db.memory` |
| `search_vector` | memory | ✅ | pure ANN (ex-`search`, AST-008), also on `db.memory` |
| `search_batch` | memory | ✅ | Python-only |
| `search_batch_requests` | memory | ✅ | Python-only |
| `explain_memory_search` | memory | ✅ | |
| `supersede` | memory | ✅ | **Python-only** |
| `generate_snippet` | memory | ✅ | |
| `purge_expired` | memory | ✅ | TTL |
| `list_namespaces` | memory | ✅ | |
| `capabilities` | system | ✅ | |
| `hardware_profile` | system | ✅ | Python-only |
| `operational_metrics` | system | ✅ | |
| `query` | system | ✅ | IQL |
| `flush` | system | ✅ | |
| `compact_wal` | system | ✅ | |
| `compact_layout` | system | ✅ | |
| `rebuild_index` | system | ✅ | |
| `reindex_hnsw_from_text` | system | ✅ | |
| `repair_text_index` | system | ✅ | |
| `audit_text_index` | system | ✅ | deep variant absent in Python |
| `export_namespace` | system | ✅ | |
| `export_all` | system | ✅ | |
| `import_file` | system | ✅ | |
| `bulk_import` | system | ✅ | |
| `bulk_import_bytes` | system | ✅ | |
| `recover_archived_nodes` | wiki | ✅ | summary-node shadow archive recovery |
| `close` | system | ✅ | lifecycle |

**Totals:** memory 15 · graph 10 · wiki 1 · system 18 = 44 pyclass methods ✔ (+ module-level `connect()` → system, 45 total surface)

> **AST-012 (anti-stutter, TS `MemoryClient` parity):** flat `get_memory` /
> `list_memory` / `delete_memory` were REMOVED (direct rename, no aliases).
> The memory path is `db.memory.get` / `.list` / `.delete` (real methods,
> single implementation); flat `get` / `delete` stay node-level (`id: u128`).
> (`search_memory`→`search`, `search`→`search_vector` already renamed by AST-008.)
>
> **Naming hazard reminder:** Python `insert` is classified as graph (node-level). The name collides with memory-record insertion semantics in other ecosystems — sub-client tests must use the real signatures above.

**Not exposed in Python (wasm/TS-only), deferred per D42:** `audit_text_index_deep`, `export_namespace_filtered`, `import_records`.

## Core-Only Capabilities (D43 — deferred, NOT part of this campaign)

| Capability | Lives in | Would land under |
|---|---|---|
| Memory pipeline L0–L3 (persona, compression, recall assembly) | crate `vanta-memory` | `conversation` |
| Context engine / narrative assembly | crate `vanta-memory` | `conversation` |
| Wiki pages / TDAM chunker / wiki seeds | core (`vantadb::wiki`) | `wiki` |
| Skill stores | future | `skills` |

Exposing any of these requires new Rust bindings (out of scope per D42). Tracked as post-campaign work.

## Sub-Client Design v1

### TypeScript (SDKB-02)

Lazy getters on `VantaDB` returning frozen delegate objects. Arrow functions capture `this` (no bind drift):

```ts
class VantaDB {
  // ...flat methods unchanged...

  private _memory?: Readonly<MemoryClient>;
  get memory(): Readonly<MemoryClient> {
    return (this._memory ??= Object.freeze({
      put: (input: MemoryInput) => this.put(input),
      search: (req: SearchRequest) => this.search(req),
      // ...one arrow per mapped method...
    }));
  }
  get graph(): Readonly<GraphClient> { /* same pattern */ }
  get wiki(): Readonly<WikiClient> { /* empty in TS v1 */ }
  get system(): Readonly<SystemClient> { /* same pattern */ }
}
```

- **Delegation only** — zero new logic (D43); stop condition from plan applies.
- `conversation`/`skills` getters are omitted in v1 (no methods exist; D43).
- Types reuse existing `types.ts`; no duplicates.
- `db.memory.x(...) === db.x(...)` result/firma identity is the test contract
  (holds in TS where the flat is memory-first).

### Python (SDKB-03 shipped, AST-012 rename applied)

PyO3 `#[pyclass]` delegate structs (`MemoryClient`, `GraphClient`,
`SystemClient`, `WikiClient`) hold `db: Py<Client>`; shared-name methods
forward verbatim via the `forward_to_db!` macro (single source of truth).
Exposed as read-only attributes via `#[getter]` on `Client` (fresh instance
per access — no shared state).

AST-012 (anti-stutter): `db.memory` exposes short names `get`/`list`/
`delete`/`search` (TS `MemoryClient` parity) as REAL methods — the
implementation moved from the removed flat `get_memory`/`list_memory`/
`delete_memory` (direct rename, no aliases). Flat `get`/`delete` stay
node-level (`id: u128`): the Python test contract is
`db.memory.get/list/delete/search` ≡ removed-flat behavior with identical
signature and result, and `db.graph.*` ≡ flat node ops. `AsyncVantaDB`
mirrors this with `db.memory` (`get`/`list`/`delete` via `to_thread`).

### Cross-SDK rule

Sub-clients group by **domain**, never by mirrored method name. Where semantics diverge (`get`/`delete`, `search`), each SDK's sub-client exposes its own real surface — this document is the arbiter.
