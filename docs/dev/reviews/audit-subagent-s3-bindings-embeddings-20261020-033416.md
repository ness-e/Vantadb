---
title: "S3 — Deep Audit: Bindings Parity + Embeddings"
kind: review
status: active
description: "Informe crudo del subagente S3: paridad de superficie entre bindings (PyO3, napi-rs, wasm-bindgen, ffi-core, ts) y el pipeline de embeddings/providers. Base de los hallazgos `S3-*` del ledger (bindings y embeddings)."
tags: [audit, reviews, bindings, embeddings, subagent]
---

# Informe del subagente — DEEP AUDIT: BINDINGS PARITY + EMBEDDINGS

**Repo:** `C:\Users\Eros\VantaDB Proyect\VantaDB`
**Mode:** READ-ONLY (no file in the repo was created, edited or deleted; no git write commands were run)
**Date:** 2026-10-20
**Scope:** `vantadb-python/` (PyO3) · `vantadb-node/` (napi-rs) · `vantadb-wasm/` (wasm-bindgen) · `vantadb-ffi-core/` · `vantadb-ts/` (TS wrapper over WASM + Node) vs the canonical Rust surface in `src/sdk/`; plus the embeddings stack (`src/llm.rs`, `src/embedding_health.rs`, `src/physical_plan/vector.rs`, `vanta-mcp` `embed_texts`, `embeddings/`, `setup-embeddings.ps1`) against `docs/api/`.

---

## PART A — BINDINGS PARITY

### A1 | P1 | node / ts (cross-backend divergence)
- **Location:** `vantadb-node/src/lib.rs:1533`, `vantadb-node/src/lib.rs:1506`, `vantadb-ts/src/native.ts:525-540`, `vantadb-ts/src/types.ts:228`, `vantadb-ts/src/vantadb.ts:714`
- **Claim/Break:** The TS SDK declares `exclude_superseded?: boolean` on `SearchRequest` (`vantadb-ts/src/types.ts:228`) and honours it **only** on the WASM backend (`vantadb-ts/src/vantadb.ts:714` → `exclude_superseded: request.exclude_superseded ?? false`). The Node/native path drops it: `vantadb-ts/src/native.ts:525-540` (`_buildSearchRequest`) never copies the field, and the Node parser hardcodes it away:
  - `vantadb-node/src/lib.rs:1533` → `exclude_superseded: false,` (inside `parse_search_request`)
  - `vantadb-node/src/lib.rs:1506` → `exclude_superseded: false,` (inside `parse_list_options`)
  The `index.d.ts` `SearchRequest` (`vantadb-node/index.d.ts:140-164`) and `MemoryListOptions` (`vantadb-node/index.d.ts:107-125`) declare no `exclude_superseded` at all, and the option parsers silently ignore unknown keys (no `deny_unknown_fields` equivalent).
- **Why it matters:** The same TS call returns different results depending on the backend. A user who supersedes a record and then filters with `exclude_superseded: true` gets the superseded record back on the native backend and does not on WASM. Python wires it correctly (`vantadb-python/src/lib.rs:1345` → `exclude_superseded` in the `MemorySearchRequest`), so the divergence is also cross-binding. `docs/api/BINDINGS_NAMESPACES.md:187` lists `exclude_superseded` on `search()` for WASM/TS/Python but never mentions that Node lacks it, and the "SDK Surface Differences" table (lines 179-197) has no Node column at all.
- **Trigger:**
  ```ts
  // native backend (vantadb/native)
  await db.put({namespace:"ns", key:"a", payload:"v1"});
  await db.put({namespace:"ns", key:"b", payload:"v2"});
  await db.supersede("ns","a","b");
  const hits = await db.search({namespace:"ns", query_vector:[0.1,0.2], exclude_superseded:true});
  // → "a" (the superseded record) is still in `hits` on the Node backend.
  // Same call on the WASM backend → "a" is absent.
  ```
  Node-only: `db.list("ns", { exclude_superseded: true })` → the flag is ignored, superseded records are returned.
- **Fix:** In `vantadb-node/src/lib.rs`, read `exclude_superseded` in `parse_search_request` and `parse_list_options` (use `get_opt_bool(obj, "exclude_superseded")?`); add the field to `index.d.ts` `SearchRequest` + `MemoryListOptions`; in `vantadb-ts/src/native.ts`, copy `exclude_superseded` in `_buildSearchRequest` and in the `list` wire object.
- **Verify:** A Node test asserting a superseded record is filtered when the flag is set; a TS parity test running the same scenario against both backends (`vantadb-ts/src/__tests__/parity.test.ts` pattern).

### A2 | P2 | python (`__all__` / star-import)
- **Location:** `vantadb-python/vantadb/__init__.py:18-28` (runtime `__all__`) vs `vantadb-python/vantadb_py/__init__.py:52-74` (source of the star import)
- **Claim/Break:** `vantadb/__init__.py` does `from vantadb_py import *` (line 15) and then declares its own `__all__` (lines 18-28) listing only 9 names: `Client, AsyncClient, ListResult, Record, SearchHit, Vector, SearchRequest, __version__, connect`. The 11 error classes (`Error`, `NotFoundError`, `ValidationError`, `CorruptError`, `StorageError`, `ConflictError`, `UnsupportedError`, `ResourceLimitError`, `BusyError`, `NoVectorError`, `TimeoutError`) and `error_to_dict` are **missing**, even though `vantadb_py.__all__` (lines 52-74) exports all of them and `docs/api/ERROR_HANDLING.md:288-350` + `docs/api/PYTHON_SDK.md:1390,1417` document them as the canonical Python surface.
  Runtime-verified:
  ```
  star-import names: ['AsyncClient', 'Client', 'ListResult', 'Record', 'SearchHit', 'SearchRequest', 'Vector', 'connect']
  Error in star? False
  error_to_dict in star? False
  ```
- **Why it matters:** `from vantadb import *` silently drops typed error handling: `except NotFoundError:` raises `NameError` at the point of use. The documented canonical import (`import vantadb`) keeps working for `vantadb.Error` / `vantadb.error_to_dict` because the star import binds them as module attributes — but the `__all__` contract that `from vantadb import *` relies on is wrong, and the stub (`vantadb-python/vantadb/__init__.pyi:18-29`) mirrors the same incomplete list, so a type checker cannot catch it.
  The existing anti-drift test only compares `vantadb_py/__init__.py` to `vantadb_py/__init__.pyi` (`vantadb-python/tests/test_stub_drift.py:266-277`); the `vantadb/` alias package is **not** covered.
- **Trigger:**
  ```python
  from vantadb import *          # noqa
  try:
      db.put("ns", "k", "v")
  except NotFoundError:          # NameError: name 'NotFoundError' is not defined
      ...
  ```
- **Fix:** Mirror `vantadb_py.__all__` in `vantadb-python/vantadb/__init__.py:18-28` and in `vantadb-python/vantadb/__init__.pyi:18-29`; extend `test_stub_drift.py` with an alias-package parity test (`vantadb/__init__.py.__all__ == vantadb_py/__init__.py.__all__` and every name in `__all__` resolvable at runtime).
- **Verify:** `python -c "import vantadb; assert set(vantadb.__all__) == set(vantadb.vantadb_py.__all__)"` plus a new pytest case.

### A3 | P2 | python (phantom module-level API documented in the spec)
- **Location:** `docs/api/PYTHON_SDK.md:678` · `vantadb-python/vantadb/__init__.pyi:15,28` · `vantadb-python/vantadb_py/__init__.py:27-46` (import list) and `:52-74` (`__all__`) · native fn exists at `vantadb-python/src/lib.rs:2854-2857`
- **Claim/Break:** `docs/api/PYTHON_SDK.md:678` documents
  ```
  vantadb.memory_node_id(namespace: str, key: str) -> int
  ```
  The native module exports `memory_node_id` (`vantadb-python/src/lib.rs:2854`, `#[pyfunction]`), but neither `vantadb_py/__init__.py` nor `vantadb/__init__.py` imports it, and it is in neither `__all__`. Runtime-verified:
  ```
  vantadb has memory_node_id attr: False
  vantadb_py has memory_node_id attr: False
  native module has memory_node_id: True
  Client.memory_node_id: 56149042414448487831425790591554898341
  ```
  Yet `vantadb-python/vantadb/__init__.pyi:15` imports it and `:28` lists it in `__all__` → **the stub declares an attribute that raises `AttributeError` at runtime**. The stub-drift test does not cover the alias package, so this passes CI.
- **Why it matters:** Type checkers accept `vantadb.memory_node_id("ns","k")`; at runtime it raises `AttributeError: module 'vantadb' has no attribute 'memory_node_id'`. This is the exact "stub says yes, runtime says no" class of drift the anti-drift suite exists to catch — but the suite stops at `vantadb_py`.
- **Trigger:**
  ```python
  import vantadb
  vantadb.memory_node_id("ns", "k")
  # AttributeError: module 'vantadb' has no attribute 'memory_node_id'
  ```
  (Documented workaround that does work: `vantadb.Client.memory_node_id("ns","k")` — the `@staticmethod` at `vantadb-python/src/lib.rs:748-750`.)
- **Fix:** Add `memory_node_id` to the `from .vantadb_py import (...)` list in `vantadb-python/vantadb_py/__init__.py` and to both `__all__`s (`vantadb_py/__init__.py:52-74`, `vantadb/__init__.py:18-28`); extend `test_stub_drift.py` to the alias package.
- **Verify:** `python -c "import vantadb; assert vantadb.memory_node_id('a','b') == vantadb.Client.memory_node_id('a','b')"`.

### A4 | P2 | python (exception-class mapping vs the canonical table)
- **Location:** `vantadb-python/src/convert.rs:876-904` vs `docs/api/ERROR_HANDLING.md:311-323`
- **Claim/Break:** The doc's exception tree is:
  ```
  ├── ValidationError   # DimensionMismatch, DuplicateNode, Validation, InvalidInput, …
  ├── CorruptError      # IncompatibleFormat, WALVersionMismatch, Schema, Serialization, …
  ├── StorageError      # Io, Wal, Backend
  ├── ConflictError     # ExecutionConflict, NodeIdCollision, CycleDetected
  ```
  The implementation (`vantadb-python/src/convert.rs`):
  - `:883` `CoreError::Serialization(_)` and `:885` `CoreError::Schema(_)` → **ValidationError** (doc says CorruptError)
  - `:886` `CoreError::NodeIdCollision(_)` → **ValidationError** (doc says ConflictError)
  - `:891` `CoreError::Wal(_)` → **CorruptError** (doc says StorageError)
  The `.code` attribute still comes from `Error::code()` (`vantadb-python/src/convert.rs:928` → `src/error.rs:386-391`), which classifies `Serialization`/`Schema` as `VANTADB_CORRUPT` — so the raised class and the raised code **disagree**.
- **Why it matters:** `except vantadb.CorruptError:` does not catch a corrupt-schema/serialization failure (it is a `ValidationError` whose `.code` says `VANTADB_CORRUPT`); `except vantadb.StorageError:` does not catch a WAL failure; `except vantadb.ConflictError:` does not catch an id collision. The doc explicitly states the Python subclasses "mirror the Rust enum 1:1" (`docs/api/ERROR_HANDLING.md:306-309`).
- **Trigger:**
  ```python
  import vantadb
  try:
      db.put("ns","k","v", metadata={"x": object()})   # any Serialization/Schema-class core error
  except vantadb.CorruptError:      # NOT raised
      ...
  except vantadb.ValidationError:  # raised — but exc.code == "VANTADB_CORRUPT"
      print(exc.code)               # 'VANTADB_CORRUPT'  ← class and code disagree
  ```
- **Fix:** Align `map_vanta_error` in `vantadb-python/src/convert.rs:876-904` with the documented tree (move `Serialization`/`Schema` → `CorruptError`, `Wal` → `StorageError`, `NodeIdCollision` → `ConflictError`), or amend `docs/api/ERROR_HANDLING.md:311-323` to describe the code as authoritative. Add one test per group asserting class **and** `.code`.
- **Verify:** `vantadb-python/tests/test_typed_errors.py` — extend with the three groups.

### A5 | P2 | ts (fabricated result on an unsupported native method)
- **Location:** `vantadb-ts/src/native.ts:940-947`
- **Claim/Break:**
  ```ts
  const src = String(sourceId);
  const tgt = String(targetId);
  if (src === tgt) return [src];
  const visited = await this.inner.graphBfs([src], maxDepth, direction);
  if (visited.includes(tgt)) {
    return [src, tgt];       // ← a 2-element path that is not a path
  }
  return [];
  ```
  `graphShortestPath` is not implemented by `vantadb-node` (no `graph_shortest_path` in `vantadb-node/src/lib.rs`), so the fallback always runs.
- **Why it matters:** If `src → x → tgt` (2 hops), the returned "path" is `["src","tgt"]` — indistinguishable from a direct edge `src → tgt`. A caller that trusts the array will draw/act on a non-existent edge. This is a silently wrong answer, not an error.
- **Trigger:**
  ```ts
  // graph: 1 -> 2 -> 3
  const p = await db.graphShortestPath("1", "3");   // native backend
  // p === ["1","3"]  (looks like a direct edge; the real path is 1->2->3)
  ```
- **Fix:** Either implement real parent-tracking BFS in the adapter, or throw `new DbError(ERROR_CODES.UNSUPPORTED_ERROR, "graphShortestPath is not available on the native backend")` instead of fabricating a path. (`ERROR_CODES` has no `UNSUPPORTED_ERROR`; add one or reuse `VALIDATION_ERROR` with an explicit message.)
- **Verify:** A TS test asserting the 2-hop case either returns the true path or throws.

### A6 | P2 | ts (silent empty / fabricated payload on unsupported native methods)
- **Location:** `vantadb-ts/src/native.ts:817-825` (`findNodesByLabel`), `vantadb-ts/src/native.ts:1144-1162` (`metricsPrometheus`)
- **Claim/Break:**
  - `findNodesByLabel`: `if (typeof (... ).findNodesByLabel === "function") {...} return [];` — the Node binding has no such method, so the method **always** returns `[]`.
  - `metricsPrometheus`: `... return "# HELP vantadb_up Whether VantaDB native engine is running\n# TYPE vantadb_up gauge\nvantadb_up 1\n";` — the Node binding has no `metrics_prometheus`, so the adapter **fabricates** a Prometheus payload that says the engine is up, with no real metrics.
- **Why it matters:** Both are declared members of the public `NativeClientBinding` interface (`vantadb-ts/src/native.ts:120, 178`) and are typed as working APIs, but on the only backend that can provide them they are hardcoded placeholders. `[]` is indistinguishable from "no nodes match"; a scraper consuming the fabricated `# HELP vantadb_up` payload sees a healthy engine and no metrics.
- **Trigger:**
  ```ts
  await db.findNodesByLabel("depends_on");   // → [] always, on the native backend
  await db.metricsPrometheus();               // → hardcoded "vantadb_up 1" string
  ```
- **Fix:** Throw `DbError` with an explicit "not available on the native backend" code in both fallbacks, or implement the two methods in `vantadb-node/src/lib.rs` (`find_nodes_by_label`, `metrics_prometheus`).
- **Verify:** TS native-backend tests asserting the throw (or the real values once implemented).

### A7 | P2 | wasm (sparse query silently dropped; the WASM doc never mentions sparse)
- **Location:** `vantadb-wasm/src/lib.rs:1297`, `:1403`, `:1550`; `SearchRequest` struct `vantadb-wasm/src/lib.rs:180-215`; `docs/api/WASM_API.md` (no `sparse` occurrence at all)
- **Claim/Break:** The WASM `SearchRequest` struct has no `query_sparse` field and no `#[serde(deny_unknown_fields)]`; the three search entry points hardcode `query_sparse: None`:
  - `:1297` `query_sparse: None,` (in `search`)
  - `:1403` `query_sparse: None,` (in `explain_memory_search`)
  - `:1550` `query_sparse: None,` (in `search_multi`)
  The TS wrapper knows about this and fails loudly (`vantadb-ts/src/vantadb.ts:700-709` throws `VALIDATION_ERROR` for a non-empty `query_sparse` on WASM), but a **direct** `vantadb-wasm` consumer gets silent data loss.
- **Why it matters:** `vantadb-wasm` is a published npm package (`docs/api/SURFACE_MANIFEST.md:27`, PUBLIC). A direct caller passing `query_sparse` gets dense/text-only results with no error — wrong results, no signal. `docs/api/WASM_API.md` documents neither `query_sparse` (unsupported) nor `sparse_vector` (which **is** supported on `put`, `vantadb-wasm/src/lib.rs:147-152`), so the WASM surface is undocumented on both sides.
- **Trigger:**
  ```js
  // direct vantadb-wasm consumer
  const hits = db.search({ namespace:"ns", query_vector: [], query_sparse: {7: 1.5} });
  // → runs as a text/dense-only search; the sparse weights are dropped, no error, no warning.
  ```
- **Fix:** Add `query_sparse` to the WASM `SearchRequest` (reusing `deserialize_sparse_vector`, `vantadb-wasm/src/lib.rs:161-177`) and wire it into the three call sites; or add `#[serde(deny_unknown_fields)]` so unknown keys raise. Document `sparse_vector`/`query_sparse` in `docs/api/WASM_API.md`.
- **Verify:** `vantadb-wasm/tests/wasm_tests.rs` — a case asserting either sparse support or a typed rejection.

### A8 | P2 | node (capability gaps vs the other three bindings, undocumented)
- **Location:** `vantadb-node/src/lib.rs` (napi surface: no `generate_snippet`, no `audit_text_index`, no `operational_metrics`, no `graph_page_rank`, no `graph_degree_centrality`, no raw-ANN `search_vector`); `docs/api/BINDINGS_NAMESPACES.md:179-197`
- **Claim/Break:** Python exposes `generate_snippet` (`vantadb-python/src/lib.rs:2552`), `audit_text_index` (`:1802`) and `operational_metrics` (`:1859`); WASM exposes `generate_snippet` (`vantadb-wasm/src/lib.rs:1994`), `audit_text_index` (`:1668`), `audit_text_index_deep` (`:1678`) and `operational_metrics` (`:1728`); TS exposes `generateSnippet`, `auditTextIndex`, `auditTextIndexDeep`, `operationalMetrics` (`vantadb-ts/src/vantadb.ts:1749, 1217, 1233, 1311`). The Node binding exposes **none** of them. The "SDK Surface Differences" table (`docs/api/BINDINGS_NAMESPACES.md:179-197`) has columns `WASM | TS | Python` — **no Node column** — so the Node gaps are invisible in the canonical cross-binding surface document.
- **Why it matters:** The doc that `docs/api/SURFACE_MANIFEST.md:54` names as the canonical "Binding method surface (per-SDK)" silently omits one of the four SDKs, so a reader diffing Node against the other bindings finds undocumented missing methods.
- **Trigger:** `nodeClient.generateSnippet(...)` / `auditTextIndex(...)` / `operationalMetrics()` have no Node equivalent; migrating a WASM/TS/Python program to Node loses these operations with no compile-time signal (the Node `index.d.ts` simply lacks them).
- **Fix:** Add a `Node` column to the "SDK Surface Differences" table in `docs/api/BINDINGS_NAMESPACES.md`, or implement the missing methods in `vantadb-node/src/lib.rs`.
- **Verify:** Doc review; `vantadb-node/tests/parity.test.ts` extension.

### A9 | P3 | docs (canonical cross-binding surface tables are stale)
- **Location:** `docs/api/BINDINGS_NAMESPACES.md:261` ("45 public methods"), `:313` ("Totals: memory 17 · graph 11 · wiki 1 · system 16 = 45"), `:317` ("49 pyclass methods"), `:372` ("memory 19 · graph 11 · wiki 1 · system 18 = 49 pyclass methods ✔")
- **Claim/Break:** The TS class actually implements 46 public methods — the tables omit `scan` (`vantadb-ts/src/vantadb.ts:672`, `MemoryClient.scan` at `vantadb-ts/src/vantadb.ts:59`); the doc header still says "45 public methods (verificado 2026-09-15)". The Python `Client` actually exposes **50** methods (54 public attributes minus the 4 properties `memory`/`graph`/`system`/`wiki`, verified at runtime) and the table omits `query_structured` (`vantadb-python/src/lib.rs:2058`).
- **Why it matters:** The document is generated-by-hand and is used as the parity contract; the drift is exactly the kind a diff-based CI check would catch but there is none for these tables.
- **Trigger:** A reader comparing `docs/api/BINDINGS_NAMESPACES.md:265-311` with `vantadb-ts/src/vantadb.ts` finds `scan` missing and the totals off by one; the same for `query_structured` in the Python table.
- **Fix:** Add `scan` to the TS table and bump the totals to 46; add `query_structured` to the Python table and bump to 50; note the verification date.
- **Verify:** A script that counts the public methods of each binding and compares to the table (same approach as `vantadb-python/tests/test_stub_drift.py`).

### A10 | P3 | embeddings/bindings (no embeddings surface in Python or TS)
- **Location:** `vantadb-python/src/lib.rs` (no embedding method), `vantadb-ts/src/` (no embedding method), `vantadb-node/src/lib.rs` (no embedding method)
- **Claim/Break:** Neither the Python, TS nor Node binding exposes any embedding capability, although the core has `EmbeddingProvider` (`src/llm.rs:31`) and the MCP server exposes `embed_texts` (`vantadb-mcp/src/handlers/tools.rs:2984`). The `docs/api/EMBEDDINGS.md:166` row says "**Python/TS SDK** | BYO-vector remains; `embed-local` is Rust-side" — i.e. it is documented as a deliberate limitation.
- **Why it matters:** This is already tracked (`docs/dev/Backlog.md:484` FIND-325 "Python SDK sin superficie de embeddings", `:485` FIND-326 "TS SDK sin superficie de embeddings"). Reported here as **KNOWN / already-tracked**, not a new defect — but it means the "bindings parity" contract for embeddings cannot be claimed as complete.
- **Trigger:** `db.embed_texts(["hola mundo"])` does not exist in Python/TS/Node; a user of the Python SDK cannot produce a vector locally.
- **Fix:** Tracked by FIND-325/FIND-326 — no new action from this audit.
- **Verify:** n/a (already in the backlog).

---

## PART B — EMBEDDINGS

### B1 | P1 | embeddings (IQL vector-search silently swallows embedding failures)
- **Location:** `src/physical_plan/vector.rs:57`, `:67-68`, `:150-151`, `:161-162`
- **Claim/Break:**
  ```rust
  if let Ok(vec) = provider.embed_query(&self.query_vec_text) {
      vector = Some(vec);
  }
  ```
  The `Err` arm is dropped. The same pattern appears in `PhysicalVectorRefine::open()` (`:150`, `:161`). With the default strict policy (`strict_embeddings: true`, `src/config.rs:395`) a missing ONNX runtime or model weights produces `Err(Error::Embedding(...))` (`src/llm.rs:561-568`), which is discarded, `vector` stays `None`, and the operator produces **zero rows** with no error.
- **Why it matters:** "No matches" and "embeddings unavailable" are indistinguishable to the caller. A misconfigured deployment silently returns empty result sets for every vector query. This is precisely the "never silent" contract the rest of the stack tries to honour (`src/embedding_health.rs:1-8`, DEF-08).
- **Trigger:**
  ```
  # strict mode, ORT unavailable (or model weights missing)
  VANTADB_EMBEDDING_PROVIDER=local vanta-cli --db ./db query \
    "FROM memory WHERE text ~ 'rust expert', min = 0.5 FETCH text, score"
  # → 0 rows, exit 0, no error, no warning.
  ```
- **Fix:** Propagate the provider error (map to `Error::Embedding` and return it from `open()`), or at minimum log at `warn!` and surface a typed error so the caller can distinguish "unavailable" from "no match".
- **Verify:** A core test with a deliberately broken provider asserting `Err`, plus a CLI-level test asserting a non-zero exit / explicit message.

### B2 | P1 | embeddings (`VANTADB_ALLOW_DUMMY_EMBEDDINGS` is a no-op on the main provider path)
- **Location:** `src/llm.rs:192-197` (`LocalOnnxProvider::new` → `LlmCfg { local_model_path, ..Default::default() }`) vs `src/llm.rs:200-217` (`from_llm_cfg`) and `src/config.rs:1114-1124` (the env parsing) and `src/config.rs:395` (`LlmCfg::default().strict_embeddings = true`)
- **Claim/Break:** `LlmCfg::default()` hardcodes `strict_embeddings: true` (`src/config.rs:395`). `LocalOnnxProvider::new` builds its `LlmCfg` from `Default::default()` (`src/llm.rs:193-196`), so it **never reads the environment** and `allow_dummy` is always `false` (`src/llm.rs:208`). Only `LocalOnnxProvider::from_llm_cfg(&Config::default().llm_cfg())` honours `VANTADB_STRICT_EMBEDDINGS` / `VANTADB_ALLOW_DUMMY_EMBEDDINGS` (`src/config.rs:1114-1124`), and that constructor is called from exactly **one** place: `src/physical_plan/vector.rs:66`.
  Every other caller uses `new()`:
  - `src/llm.rs:74`, `:81`, `:109` — `get_embedding_provider()` (the factory used by `src/executor.rs:383,542` and `src/physical_plan/vector.rs:56,149`)
  - `vantadb-mcp/src/handlers/tools.rs:4080` — `cached_local_provider_for` (the MCP `embed_texts` model path)
- **Why it matters:** The escape hatch documented in `docs/api/EMBEDDINGS.md:45` ("dummy fallback requires `VANTADB_ALLOW_DUMMY_EMBEDDINGS=true`") and in the error text itself (`src/llm.rs:564-566`) has **no effect** on the factory path or the MCP path. Setting it does not enable the synthetic fallback; the provider still fails closed.
- **Trigger:**
  ```bash
  export VANTADB_ALLOW_DUMMY_EMBEDDINGS=true
  export VANTADB_EMBEDDING_PROVIDER=local
  # no ORT dylib / no model weights
  vanta-cli --db ./db put --namespace ns --key k --payload v --text "hola mundo"
  # → the node is stored WITHOUT a vector (src/executor.rs:389-391 swallows the Err with a warn!)
  #   even though the operator explicitly asked for dummy embeddings.
  ```
- **Fix:** `LocalOnnxProvider::new` should derive its `LlmCfg` from `Config::default().llm_cfg()` (env-aware) rather than `LlmCfg::default()`, or take an explicit `strict` flag; then `get_embedding_provider()` and the MCP path inherit the documented behaviour.
- **Verify:** A test asserting `VANTADB_ALLOW_DUMMY_EMBEDDINGS=true` yields deterministic (non-error) vectors through `get_embedding_provider()` and through the MCP `embed_texts` model path.

### B3 | P2 | embeddings (recovery hint points at a variable with no effect)
- **Location:** `src/error.rs:429-431`, `src/llm.rs:561-568`
- **Claim/Break:**
  ```rust
  Error::Embedding(_) => Some(
      "Configure ORT_DYLIB_PATH / local model weights or set VANTADB_ALLOW_DUMMY_EMBEDDINGS=true",
  ),
  ```
  and the provider error text: *"…or configure VANTADB_ALLOW_DUMMY_EMBEDDINGS=true to permit synthetic fallback"*. The provider that raises this error was built through `LocalOnnxProvider::new` (B2) and therefore ignores that variable. The advice is inert for the paths that actually emit it.
- **Why it matters:** The hint is the operator's only remediation pointer; following it changes nothing and the next call fails identically.
- **Trigger:** Any `Error::Embedding` raised from `get_embedding_provider()`/`embed_texts` (model path) → `exc.hint` says to set `VANTADB_ALLOW_DUMMY_EMBEDDINGS=true` → setting it → same error.
- **Fix:** Fix B2 (make the hint true) or reword the hint to name the variables that are actually read (`ORT_DYLIB_PATH`, `VANTADB_LOCAL_MODEL`).
- **Verify:** Test asserting the hint matches the effective configuration.

### B4 | P1 | embeddings (default provider is `ollama`, not `local`, even with `embed-local` compiled)
- **Location:** `src/config.rs:394` and `src/config.rs:1108-1111` (default `embedding_provider = "ollama"`), `src/llm.rs:65-86` (the factory's `"ollama"` arm), `src/cli_handlers/config.rs:497` (generated TOML template `embedding_provider = "ollama"`), vs `docs/api/EMBEDDINGS.md:58`
- **Claim/Break:** `docs/api/EMBEDDINGS.md:58` states: *"With `embed-local` active, default `VANTADB_EMBEDDING_PROVIDER` is `local` (default model `embeddings/models/multilingual-e5-small/onnx` [384d] …)"*. The code default is `"ollama"` in both the `Config`/`LlmCfg` default (`src/config.rs:394`) and the env parser (`src/config.rs:1110`). `get_embedding_provider()` (`src/llm.rs:67-69`) therefore routes to `OllamaProvider`, which POSTs to `http://localhost:11434/api/embed`.
- **Why it matters:** With `--features embed-local` and no env var set, the local ONNX model is **never loaded**; the engine tries to reach a local Ollama server that usually is not running, gets a connection error, and (see `src/executor.rs:389-391`) stores the node **without a vector** after a `warn!`. The user believes they enabled offline local embeddings and gets neither offline behaviour nor local vectors. The `setup-embeddings.ps1` wizard compensates by setting `VANTADB_EMBEDDING_PROVIDER=local` in the session, but any other entry point (library, MCP launched by an agent, `vanta-cli` run directly) does not.
- **Trigger:**
  ```bash
  cargo build --features embed-local --bin vanta-cli
  unset VANTADB_EMBEDDING_PROVIDER
  vanta-cli --db ./db put --namespace ns --key k --payload v --text "hola mundo"
  # → tries http://localhost:11434/api/embed → connection refused → node stored with NO vector.
  # The local model in embeddings/models/multilingual-e5-small/onnx is never touched.
  ```
- **Fix:** Make the default `"local"` when the `embed-local` feature is compiled (feature-dependent default in `Config`/`LlmCfg`), keeping `"ollama"` for `remote-inference`-only builds; or correct `docs/api/EMBEDDINGS.md:58` and add a startup warning when `embed-local` is compiled but the provider is remote.
- **Verify:** A test asserting the factory's default provider per feature set; an end-to-end check that a node stored through the CLI carries a vector with `embed-local` and no env var.

### B5 | P2 | embeddings (MCP `embed_texts` hard-errors on the `model` path while the default path falls back)
- **Location:** `vantadb-mcp/src/handlers/tools.rs:4122-4126` (dispatch), `:4098-4111` (`embed_texts_with_manifest_model`), `:3072-3074` (handler maps the error), vs `:4174-4186` (the fallback branch)
- **Claim/Break:** With `model=<manifest id>` the handler returns early:
  ```rust
  if let Some(id) = model {
      return embed_texts_with_manifest_model(texts, id);   // Err → propagated
  }
  ```
  and `embed_texts_with_manifest_model` (`:4107-4110`) propagates the provider error. The handler then converts it to `McpError::internal_error(e)` (`:3073-3074`) — a hard JSON-RPC error. The **no-`model`** path (`:4174-4186`) catches the same failure and serves a deterministic fallback with `fallback: true` + `warning`.
  Because `LocalOnnxProvider::new` is strict (B2), "ORT unusable / model missing" is exactly the failure that hard-errors on the `model` path.
- **Why it matters:** `docs/api/MCP.md:306` states the response *"always carries `fallback: false` (real vectors) or `fallback: true` + `warning` (deterministic hash, no semantic signal — never silent, Q5)"*, and `skills/vantadb-mcp/SKILL.md:187` says *"never a hard error, never silent"*. The `model=` path violates that contract; the same degradation is soft in one path and hard in the other.
- **Trigger:**
  ```json
  {"name":"embed_texts","arguments":{"texts":["hola mundo"],"model":"multilingual-e5-small"}}
  ```
  with ORT unavailable → JSON-RPC error `-32603` instead of `{"embeddings":[[...]],"fallback":true,"warning":"..."}`.
  The same call **without** `model` returns the fallback envelope.
- **Fix:** In `embed_texts_with_manifest_model`, downgrade a provider error to the same `(fallback=true, warning=…)` shape used by the default path (or route both paths through one function that owns the Q5 policy).
- **Verify:** `vantadb-mcp/tests/test_embed_texts.rs` — a case with `model` set and no model files on disk, asserting `fallback == true` and a `warning` (not an error).

### B6 | P2 | embeddings (MCP `embed_texts` reports a hardcoded model name)
- **Location:** `vantadb-mcp/src/handlers/tools.rs:3075-3081`
- **Claim/Break:**
  ```rust
  let dim = embeddings.first().map(|v| v.len()).unwrap_or(0);
  let mut result = json!({
      "embeddings": embeddings,
      "model": model_opt.unwrap_or_else(|| "multilingual-e5-small".to_string()),
      "dim": dim,
      ...
  });
  ```
  When the caller does not pass `model`, the response always says `"multilingual-e5-small"` — regardless of the active provider (`ollama` → `all-minilm`; `openai` → `text-embedding-3-small`; local → whatever `VANTADB_LOCAL_MODEL` points at).
- **Why it matters:** The `model` field is the value an agent uses to decide which embedding space the vectors live in. With `VANTADB_EMBEDDING_PROVIDER=openai` the response reads `{"model":"multilingual-e5-small","dim":1536}` — the name and the dimension contradict each other, and a caller that caches vectors per `model` will mix two embedding spaces in one bucket.
- **Trigger:**
  ```bash
  VANTADB_EMBEDDING_PROVIDER=openai VANTADB_OPENAI_API_KEY=sk-...
  # embed_texts {texts:["hola"]}  →  {"model":"multilingual-e5-small","dim":1536,...}
  ```
- **Fix:** Return the effective model (from the provider's `LlmCfg`: `llm_model` / `openai_model` / the manifest id of `VANTADB_LOCAL_MODEL`), not a hardcoded default.
- **Verify:** MCP test asserting `model` reflects the configured provider/model.

### B7 | P2 | embeddings (health probe / MCP file check can be satisfied by a *different* model's files)
- **Location:** `src/embedding_health.rs:187-198`, `vantadb-mcp/src/handlers/tools.rs:3907-3918`
- **Claim/Break:** Both "are the model files present?" checks include hardcoded fallbacks to the default model:
  ```rust
  let onnx_candidates = [
      base.join("model.onnx"),
      base.join("onnx/model.onnx"),
      base.join("model_int8.onnx"),
      PathBuf::from("embeddings/models/multilingual-e5-small/onnx/model.onnx"),   // ← different model!
  ];
  let tok_candidates = [
      base.join("tokenizer.json"),
      base.join("../tokenizer.json"),
      base.join("../../tokenizer.json"),
      PathBuf::from("embeddings/models/multilingual-e5-small/tokenizer.json"),   // ← different model!
  ];
  ```
- **Why it matters:** Point `VANTADB_LOCAL_MODEL` at `embeddings/models/embeddinggemma-2/onnx` (exactly what `setup-embeddings.ps1` does — it sets `VANTADB_LOCAL_MODEL` to the selected model's dir) and the probe finds the **e5** files instead, reporting `model_present: true` / `fallback: false`. The DEF-08 notice (the "never silent" contract, `src/embedding_health.rs:1-8`) then says the setup is healthy while the configured model is absent, and the MCP `embed_texts` path (`:4070-4078`) takes the same decision and skips its "model files not found" error.
- **Trigger:**
  ```bash
  export VANTADB_LOCAL_MODEL=$PWD/embeddings/models/embeddinggemma-2/onnx
  rm -rf embeddings/models/embeddinggemma-2/onnx     # configured model absent
  # multilingual-e5-small is on disk:
  vanta-cli --db ./db status --json
  # → embedding.model_present = true, embedding.fallback = false  (WRONG)
  ```
- **Fix:** Drop the hardcoded `multilingual-e5-small` candidates from both lists (or only use them when `dir` **is** the default path), and evaluate the candidates relative to the requested `dir` only.
- **Verify:** Unit test: `model_files_present(<empty dir>)` must be `false` even when the default model exists on disk; same for `local_model_files_present`.

### B8 | P2 | embeddings (the benchmark never runs inference)
- **Location:** `benchmarks/embed_bench.py:233-263`
- **Claim/Break:**
  ```python
  enc = tok.encode(text)
  # dummy run: solo medir path ort si disponible; ...
  dt = (time.perf_counter() - t0) * 1000
  # no ejecutamos session.run completo (requiere attention_mask etc) — medimos dummy normalizado + costo load
  vec = dummy_embed(text, dim, group)
  return vec, dt
  ...
  dt = max(dt, dim * 0.002)  # 384->0.77ms, 1024->2ms, 4096->8ms
  ```
  `session.run` is never called. The returned vector is a hash-based dummy (`dummy_embed`, `:82-99`) and the returned latency is either the **session construction** time or a synthetic `dim * 0.002` floor. The bench then reports "p50 embed (ms)", "recall@10" and "multi cos" computed from those dummies.
- **Why it matters:** `docs/api/EMBEDDINGS.md:176` lists
  ```
  python benchmarks/embed_bench.py --models multilingual-e5-small,bge-m3 --dataset tiny-en-es-1k
  ```
  under "Verification (contract §7)", i.e. as the evidence that the embedding path works. The numbers it produces are synthetic and cannot validate anything about real inference, real recall, or real latency.
- **Trigger:**
  ```bash
  python benchmarks/embed_bench.py --models multilingual-e5-small --dataset tiny-en-es-1k
  # → "p50 embed (ms)" ≈ 0.77 (= 384*0.002) and "multi cos" from hash dummies,
  #   regardless of whether the model files are present or ORT works.
  ```
- **Fix:** Either implement the real `session.run` path (mean-pool + L2, as `embeddings/sanity_embed.py:87-118` does) or rename/flag the columns as synthetic (`embed_ms (SIMULATED)`) and remove the command from the "Verification" list in `docs/api/EMBEDDINGS.md:176`.
- **Verify:** Compare the bench's `multi cos` against `embeddings/sanity_report.json` for the same model — they must agree if the bench is real.

### B9 | P2 | embeddings (IQL vector arm is capped at 5 candidates regardless of LIMIT)
- **Location:** `src/physical_plan/vector.rs:80`, `src/planner.rs:427-432`
- **Claim/Break:**
  ```rust
  let neighbors = {
      let index = self.storage.hnsw.load();
      let vs = self.storage.vector_store[0].read();
      index.search_nearest(&vec, None, None, &crate::node::ALL_BITSET, 5, Some(&*vs))
  };
  ```
  The 5th argument of `search_nearest` is `top_k` (`src/index/search/nearest.rs:19-27`). The query's `LIMIT` is applied by a *later* operator (`src/planner.rs:427-432`), so it can only truncate what the vector arm already produced.
- **Why it matters:** Any IQL query with a vector-similarity condition returns **at most 5 rows**, no matter how large the LIMIT is and no matter how many records match. This is the only place the "VECTOR_SEARCH"-style path is implemented, so it caps every SQL/IQL vector query.
- **Trigger:**
  ```
  FROM memory WHERE text ~ 'rust expert', min = 0.1 FETCH text, score
  ```
  against a namespace with 50 similar records → 5 rows returned, silently.
- **Fix:** Thread the plan's `LIMIT`/`top_k` into `PhysicalVectorSearch::new` (the planner already has it at `src/planner.rs:234-235`) and use it as the `search_nearest` `top_k` (with a sane floor).
- **Verify:** A core test with ≥10 matching nodes asserting the row count follows the LIMIT.

### B10 | P2 | docs (`VECTOR_SEARCH('…')` does not exist anywhere in the codebase)
- **Location:** `docs/api/EMBEDDINGS.md:163` · `docs/user/QUICKSTART.md:264` · `docs/user/tutorials/05-embedding-integrations.md:112` vs `src/parser/grammar.rs:69-80` and `docs/api/IQL.md:275`
- **Claim/Break:** The docs advertise the IQL form `VECTOR_SEARCH('hola mundo')` and cite `src/physical_plan/vector.rs:51`. A repo-wide search for `VECTOR_SEARCH` returns **only markdown** — no Rust, Python, TS or script defines or parses it. The real IQL vector predicate is `field ~ "text", min = score`:
  - `docs/api/IQL.md:275` — `<field> ~ "<text_query>", min = <score>`
  - `src/parser/grammar.rs:69-80` — `p.bio ~ "rust expert", min = 0.88` → `Condition::VectorSim(field, query, min_score)`
  The cited line `src/physical_plan/vector.rs:51` is a closing brace; the provider selection lives at lines 54-74.
- **Why it matters:** A user following the QUICKSTART writes a query that the IQL parser does not understand and gets a parse error (or, worse, a different query than intended). The three documents (API reference, quickstart, tutorial) all repeat the phantom.
- **Trigger:**
  ```
  vanta-cli --db ./db query "VECTOR_SEARCH('hola mundo')"
  # → IQL parse error (no VECTOR_SEARCH function exists).
  ```
- **Fix:** Replace `VECTOR_SEARCH('x')` with the real syntax (`text ~ "x", min = 0.5`) in `docs/api/EMBEDDINGS.md:163`, `docs/user/QUICKSTART.md:264`, `docs/user/tutorials/05-embedding-integrations.md:112`, and correct the line reference to `src/physical_plan/vector.rs:54-74`.
- **Verify:** `grep -r "VECTOR_SEARCH" docs/ src/` returns only historical plan/task files.

### B11 | P2 | docs (Ollama endpoint mismatch)
- **Location:** `src/llm.rs:767` vs `docs/api/EMBEDDINGS.md:46`
- **Claim/Break:** The doc's provider table says `OllamaProvider` → `POST /api/embeddings`. The code posts to `/api/embed`:
  ```rust
  let url = format!("{}/api/embed", self.base_url);
  ```
  (`/api/embeddings` is the **OpenAI-compatible** shape; Ollama's native endpoint is `/api/embed`.)
- **Why it matters:** A user configuring a proxy or an OpenAI-compatible gateway in front of Ollama will wire the wrong path and get a 404 that surfaces as a generic "Inference Bridge returned error status" (`src/llm.rs:780-783`) with no hint of the path.
- **Trigger:** Point `VANTADB_LLM_URL` at a gateway that only implements `/api/embeddings` (or vice-versa) → the embedding call fails with an opaque status error.
- **Fix:** Correct `docs/api/EMBEDDINGS.md:46` to `POST /api/embed`.
- **Verify:** Doc grep; the code path is unambiguous.

### B12 | P3 | docs (model-matrix counts are stale — the manifest has 10 models)
- **Location:** `embeddings/manifest.json` (10 entries: …, `qwen3-embedding-8b`, `embeddinggemma-2`) vs `docs/api/EMBEDDINGS.md:89` ("~22 GB if all 9 × ONNX+HF"), `:94` ("manifest.json # 9 models"), `:133` ("Model matrix — 9 models"), `:145-149` (the 9-row table + "Balance: 3 EN + 3 ES + 3 Combined"), `:105` ("--all --skip-exception # CI-friendly (8 models, ~6 GB)"); `embeddings/README.md:22` ("## Modelos (9)"), `:36` ("3 EN + 3 ES + 3 Combined"), `:87` ("Con flag: 9 modelos"); `embeddings/download.py:69` ("descarga los 9 modelos (por defecto)")
- **Claim/Break:** The manifest lists **10** models; `embeddinggemma-2` (768d, `onnx-community/embeddinggemma-2-ONNX`, rev `daa72c5`) is in the manifest and on disk (`embeddings/models/embeddinggemma-2/`) but absent from every table. The group balance is now **3 EN + 3 ES + 4 combined**. `--all --skip-exception` yields **9** models, not 8. The MCP hardcoded fallback table was already updated to 10 ids (`vantadb-mcp/src/handlers/tools.rs:3975-3990`) — the docs were not.
- **Why it matters:** The model matrix is the operator-facing selection guide; a model that exists and is downloadable is invisible, and the size/download estimates are wrong.
- **Trigger:** `python embeddings/download.py --all --skip-exception` → 9 models downloaded; the doc says 8. `grep -c '"id"' embeddings/manifest.json` → 10, doc says 9.
- **Fix:** Regenerate the tables in `docs/api/EMBEDDINGS.md` and `embeddings/README.md` from `embeddings/manifest.json` (single source of truth) and fix the `--help` string in `embeddings/download.py:69`.
- **Verify:** A doc check that re-derives the table from the manifest (the repo already has `node scripts/docs/...` generators for other docs).

### B13 | P3 | docs (`verify.py` does not do what the API reference claims)
- **Location:** `docs/api/EMBEDDINGS.md:97` vs `embeddings/verify.py:1-20` and `:141-203`
- **Claim/Break:** The doc says: "`verify.py` — ort+tokenizers dim+cosine+ONNXvsHF>0.98, writes verify.log". The script's own docstring says: *"Nivel: smoke (no verificación numérica) … los thresholds cosine se imprimen como hints, NO se asertan. La verificación numérica real vive en `sanity_embed.py`"*. Its `verify_model` (`:141-203`) opens the ONNX session and the tokenizer, then prints `"OK (onnx=… dim=…)"`; the "multi cosine threshold" line (`:199-202`) is a **print**, never compared to a value. There is **no** ONNX-vs-HF comparison anywhere in `embeddings/` (`sanity_embed.py` has 3 models and no HF baseline either).
- **Why it matters:** The doc presents `verify.py` as the numeric gate for the embedding stack; it is a structural smoke test. A corrupt-but-loadable model passes it.
- **Trigger:** `python embeddings/verify.py --all` → "PASS" for every downloaded model, regardless of embedding quality; `grep -rn "0.98" embeddings/` → no hits.
- **Fix:** Correct `docs/api/EMBEDDINGS.md:97` to describe the smoke level and point at `embeddings/sanity_embed.py` for numeric checks (and at the "ONNXvsHF" comparison as **not implemented**).
- **Verify:** Doc review.

### B14 | P3 | embeddings (`sanity_embed.py` resolves paths relative to CWD)
- **Location:** `embeddings/sanity_embed.py:24-25` vs `embeddings/verify.py:29-30` and `embeddings/download.py:22-24`
- **Claim/Break:**
  ```python
  MODELS = pathlib.Path("embeddings/models")
  REPORT = pathlib.Path("embeddings/sanity_report.json")
  ```
  Both are **relative to the current working directory**, whereas `verify.py` and `download.py` anchor on `pathlib.Path(__file__).parent`.
- **Why it matters:** Running `python embeddings/sanity_embed.py` from anywhere other than the repo root looks for the models in the wrong place (`FileNotFoundError` → "FAIL load") and writes/overwrites `embeddings/sanity_report.json` relative to the CWD instead of the repo. `embeddings/sanity_report.json` is a checked-in artifact, so a run from a subdirectory silently creates a second copy.
- **Trigger:**
  ```bash
  cd /tmp && python "$REPO/embeddings/sanity_embed.py"
  # → "FAIL load: … missing onnx or tokenizer" (models not found), and writes /tmp/embeddings/sanity_report.json
  ```
- **Fix:** `MODELS = pathlib.Path(__file__).parent / "models"` and `REPORT = pathlib.Path(__file__).parent / "sanity_report.json"`.
- **Verify:** Run from a non-repo CWD and confirm it still resolves the repo models.

### B15 | P3 | setup script (the "falls back to dummy" claim contradicts the strict default)
- **Location:** `setup-embeddings.ps1:434` vs `src/llm.rs:72-84` and `src/llm.rs:192-197` and `src/config.rs:395`
- **Claim/Break:** The script prints:
  ```
  [setup] embeddings OFF: sin modelo local ni provider (el engine cae a embeddings dummy si falta el modelo; DEF-08).
  ```
  With the default `strict_embeddings = true` the engine does **not** fall back to dummy vectors: `LocalOnnxProvider::new` never returns `Err` (it always returns `Ok`, `src/llm.rs:192-197`), so the `unwrap_or_else(|_| LocalOnnxProvider::new_dummy(384))` fallback at `src/llm.rs:74-76` / `:80-83` / `:108-110` is **unreachable**; the provider instead returns `Err(Error::Embedding(...))` (`src/llm.rs:561-568`) and the caller stores the node **without a vector** (`src/executor.rs:389-391`) or the IQL path returns zero rows (B1).
- **Why it matters:** The operator is told the engine degrades to dummy vectors; in reality it degrades to *no vectors*, which changes recall semantics (empty results vs. noisy results) and is not what DEF-08 describes.
- **Trigger:** `pwsh -NoProfile -File setup-embeddings.ps1 -NonInteractive -Modules none` → the message above is printed; a subsequent `vanta-cli put --text "..."` stores the record with no vector.
- **Fix:** Reword the message to match the strict default ("el engine omite el vector (strict_embeddings=true); dummy requiere VANTADB_ALLOW_DUMMY_EMBEDDINGS=true"), or make the fallback real.
- **Verify:** Run the script with `-Modules none` and compare the message with the observed put behaviour.

### B16 | P3 | docs (`LocalOnnxProvider` internals drifted)
- **Location:** `docs/api/EMBEDDINGS.md:66` and `:70` vs `src/llm.rs:116-125` and `src/llm.rs:473-494`
- **Claim/Break:**
  - The doc quotes the struct as `pub struct LocalOnnxProvider { session: Option<Mutex<Session>>, tokenizer: Option<Tokenizer>, dim: usize, model_dir: String }`. The real struct (`src/llm.rs:116-125`) also has `family: EmbedFamily` and `allow_dummy: bool` — and `allow_dummy` is exactly the field that decides the strict/dummy behaviour, i.e. the doc omits the field that matters most.
  - The doc says `try_load_session` "probes `model.onnx`, `onnx/model.onnx`, `model_int8.onnx`, or any `*.onnx` **recursively**". The code (`src/llm.rs:473-494`) scans **one level** (`read_dir`) and additionally probes `<entry>/model.onnx`; there is no recursion.
  - The doc says "`try_load_session` calls `ort::init().commit()` once"; the code now uses `ort::init_from(&path)` with an explicit dylib path (`src/llm.rs:381-423`, `resolve_ort_dylib_path` at `:313-369`).
- **Why it matters:** The doc is the reference for the local-embedding internals; the omitted `allow_dummy` field is precisely the mechanism behind B2/B3, and the "recursive" claim overstates the on-disk search.
- **Trigger:** A reader auditing the strict/dummy behaviour from the doc concludes the struct has no such field.
- **Fix:** Update `docs/api/EMBEDDINGS.md:66,70` to the current struct fields and the current session-load order.
- **Verify:** Doc review.

### B17 | P3 | embeddings (`embed_batch` is a sequential loop, not a batched API call)
- **Location:** `src/llm.rs:45-52` (trait default) and `src/llm.rs:698-705` (the local override) vs `docs/api/EMBEDDINGS.md:165`
- **Claim/Break:**
  ```rust
  fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
      // ponytail: sequential batch, true batched inference if throughput matters
      let mut out = Vec::with_capacity(texts.len());
      for t in texts { out.push(self.embed(t)?); }
      Ok(out)
  }
  ```
  For `OllamaProvider` / `OpenAIProvider` the default trait impl is used, i.e. **one HTTP request per text**. `docs/api/EMBEDDINGS.md:165` says the MCP tool "reuses `embed_batch`", which reads as a single batched provider call; the MCP response for 128 texts issues 128 sequential HTTP round-trips (each with its own connect/timeout budget, `src/llm.rs:824-828`).
- **Why it matters:** Throughput and failure semantics differ from the implied contract: one failure aborts the whole batch, and the per-request timeout (30 s) is multiplied by the batch size in the worst case.
- **Trigger:** `embed_texts` with 128 texts against a remote provider → 128 sequential POSTs; the 25k-token budget check (`vantadb-mcp/src/handlers/tools.rs:3043-3060`) does not bound wall-clock time.
- **Fix:** Implement a real batched path for the remote providers (OpenAI's `/v1/embeddings` accepts an array `input`; Ollama's `/api/embed` accepts an array too), or document the sequential semantics in `docs/api/EMBEDDINGS.md:165`.
- **Verify:** A test asserting one HTTP request per batch for the remote providers.

---

## Coverage

### Part A — bindings parity
| Surface | Enumerated | Method-level diff done | Notes |
|---|---|---|---|
| `src/sdk/` (canonical Rust) | ✅ | ✅ | `pub fn` inventory across `api/{admin,graph,memory,namespaces,search,sharing}.rs`, `builder.rs`, `graph.rs`, `gds.rs`, `search/*`, `serialization/*` |
| `vantadb-python/` (PyO3) | ✅ | ✅ | 50 public `Client` methods (runtime `dir()`), 4 sub-client getters, module fns `connect`/`memory_node_id`; stubs `vantadb_py.pyi`, `vantadb_py/__init__.pyi`, `vantadb/__init__.pyi` compared to impl |
| `vantadb-node/` (napi) | ✅ | ✅ | 46 `#[napi]` methods + `AbortHandle`; `index.d.ts` compared method-by-method to the impl |
| `vantadb-wasm/` (wasm-bindgen) | ✅ | ✅ | 53 `pub fn` (matches the doc's "53"); `SearchRequest`/`ListOptions` serde shapes compared to the core request |
| `vantadb-ts/` (wrapper) | ✅ | ✅ | 46 class methods + 4 sub-client interfaces vs `native.ts` (`NativeClientBinding`, 60+ members) and the WASM client |
| `vantadb-ffi-core/` | ✅ | ✅ | `OpGate`/`OpGuard`/`clamp_top_k`; contract holds, no defect found |
| `docs/api/` | ✅ | ✅ | 30 files listed; `BINDINGS_NAMESPACES.md`, `ERROR_HANDLING.md`, `PYTHON_SDK.md`, `NODE_SDK.md`, `TS_SDK.md`, `WASM_API.md`, `MCP.md`, `SEARCH_PARITY.md`, `SURFACE_MANIFEST.md`, `EMBEDDINGS.md` cross-checked |

**Not verified (marked, not claimed):**
- `vantadb-node/index.js` / `index.cjs` are napi-generated loaders — not hand-audited.
- `vantadb-wasm/pkg/*.js` and `vantadb-wasm/e2e/pkg-nomodules/*` are generated artifacts — not hand-audited.
- The `vantadb-ts` **WASM** sub-client (`memory`/`graph`/`system`/`wiki` getters) was verified by *shape* (delegation only) but not exercised at runtime (no built wasm artifact was loaded).
- Runtime verification of the Python surface used the prebuilt `vantadb_py.pyd` in the repo venv; it was **not** rebuilt from source in this session (read-only constraint).
- MCP tool behaviour was verified by source reading; the `vantadb-mcp` server was not started.

### Part B — embeddings
| Check | Result |
|---|---|
| Batching | ❌ `embed_batch` is a sequential loop (`src/llm.rs:45-52, 698-705`) → finding **B17** |
| Max batch size / overflow | ✅ MCP enforces `texts.len() <= 128` (`vantadb-mcp/src/handlers/tools.rs:2992-2996`) and a 25k-token budget (`:3037-3060`); the Rust providers have no batch cap (no batching) |
| Cache key correctness | ✅ No embedding-result cache exists in the Rust providers (nothing to collide). The MCP model-session cache is keyed by manifest id (`vantadb-mcp/src/handlers/tools.rs:4065, 4093`) with a 2-entry cap (`:4045, 4087-4091`) — id-keyed, no dim/model collision. The Python `LRU_CACHE` (`vantadb-python/src/convert.rs:48-51`) is a metadata-coercion cache, not an embedding cache |
| L2 normalization | ✅ applied in the ONNX mean-pool path (`src/llm.rs:677-683`) and in the deterministic fallback (`src/llm.rs:534-540`); the zero-norm case is guarded (`if norm > 1e-9`) → a zero vector stays zero rather than producing NaN. The **remote** providers (Ollama/OpenAI) do **not** normalize — the doc claims "mean-pool + L2" only for `LocalOnnxProvider` (`docs/api/EMBEDDINGS.md:45`), so no doc violation, but the transports differ in norm |
| Zero vector | ✅ guarded (`src/llm.rs:510-516, 534-540, 679-683`); no NaN path |
| Dimension validation | ✅ vectors are validated at the binding edge (`vantadb-python/src/convert.rs:204-266`, `MAX_VEC_DIM`) and the core rejects mismatches (`Error::DimensionMismatch` → `VANTADB_VALIDATION_ERROR`). The one-dim-per-namespace rule is enforced at the storage layer, and the doc's "one model per namespace" rule is documented (`docs/api/EMBEDDINGS.md:61,153-155`). ⚠️ But the "files present" check that gates the dim decision can be satisfied by the wrong model → **B7** |
| Error paths (HTTP failure / timeout / empty response / 429) | ⚠️ HTTP failure and empty response are handled (`src/llm.rs:772-796`, `:877-906`); timeouts are fixed at connect 10 s / total 30 s (`src/llm.rs:739-743, 824-828`). **429 is not handled**: a rate-limit response is collapsed into a generic `Error::generic_error("OpenAI returned error status 429: …")` (`src/llm.rs:887-894`) with no retry/backoff and no `Retry-After` parsing. No `Error` variant distinguishes it → `.retriable` is `false` (`src/error.rs:415-424`), so a client retry policy will not retry a rate limit |
| Fallback behaviour | ⚠️ Documented as "never silent" and mostly implemented, but inconsistent: the MCP `model=` path hard-errors (**B5**), the IQL path returns empty (**B1**), and the documented `VANTADB_ALLOW_DUMMY_EMBEDDINGS` escape hatch is inert (**B2**, **B3**) |
| Concurrency / shared mutable state | ✅ `LocalOnnxProvider` holds `Option<parking_lot::Mutex<ort::session::Session>>` (`src/llm.rs:117`) — the session is mutex-guarded and `EmbeddingProvider: Send + Sync` (`src/llm.rs:31`). The MCP model cache is a `Mutex<HashMap<..>>` behind a `OnceLock` (`vantadb-mcp/src/handlers/tools.rs:4048-4055`) with a bounded size. No unsynchronised shared state found. ⚠️ the cache lock is held across `.lock()` only (clone-out), so no I/O under the lock — OK |
| Cost / token accounting | ⚠️ The MCP tool uses a **heuristic** token estimate `(len + 3) / 4` (`vantadb-mcp/src/handlers/tools.rs:3044`) purely for the budget cutoff; **no token usage or cost is reported** in the response envelope (`:3076-3085`) and the OpenAI provider does not read `usage` from the API response (`src/llm.rs:858-864` deserialises only `data`). No doc claims cost reporting, so this is a gap, not a contradiction |
| `embeddings/` python dir | checked: `download.py`, `verify.py`, `sanity_embed.py`, `manifest.json`, `manifest.lock`, `README.md` → **B12, B13, B14** |
| `setup-embeddings.ps1` | checked end-to-end → **B15**; the ORT ≥1.27 enforcement and the `ORT_DYLIB_PATH` normalization are correct (`Install-OrtStore`/`Ensure-OrtNative`) |

### Findings index
| ID | Sev | Area | One-liner |
|---|---|---|---|
| A1 | P1 | node/ts | `exclude_superseded` silently dropped on Node `search`/`list` → TS native backend diverges from WASM/Python |
| A2 | P2 | python | `vantadb.__all__` omits all 11 error classes + `error_to_dict` → `from vantadb import *` drops typed errors |
| A3 | P2 | python | `vantadb.memory_node_id` documented + stubbed, absent at runtime (AttributeError) |
| A4 | P2 | python | Python exception classes disagree with `ERROR_HANDLING.md` §5 for Serialization/Schema, NodeIdCollision, Wal |
| A5 | P2 | ts | `graphShortestPath` fabricates a 2-element path when the native method is missing |
| A6 | P2 | ts | `findNodesByLabel` returns `[]` and `metricsPrometheus` returns a fabricated payload on the native backend |
| A7 | P2 | wasm | WASM `search`/`search_multi`/`explain` silently drop `query_sparse`; WASM_API.md never mentions sparse |
| A8 | P2 | node/docs | Node lacks `generate_snippet`/`audit_text_index`/`operational_metrics`/`graph_page_rank`; the canonical parity table has no Node column |
| A9 | P3 | docs | `BINDINGS_NAMESPACES.md` TS table misses `scan` (45→46), Python table misses `query_structured` (49→50) |
| A10 | P3 | bindings | No embeddings surface in Python/TS/Node — already tracked as FIND-325/FIND-326 |
| B1 | P1 | embeddings | IQL vector operator swallows embedding errors → silent empty result set |
| B2 | P1 | embeddings | `VANTADB_ALLOW_DUMMY_EMBEDDINGS` is a no-op via `LocalOnnxProvider::new` (factory + MCP) |
| B3 | P2 | embeddings | `Error::Embedding` recovery hint names a variable with no effect |
| B4 | P1 | embeddings | Default provider is `ollama`, not `local`, with `embed-local` compiled (doc says otherwise) |
| B5 | P2 | embeddings | MCP `embed_texts` with `model=` hard-errors instead of the documented fallback |
| B6 | P2 | embeddings | MCP `embed_texts` always reports `"model":"multilingual-e5-small"` |
| B7 | P2 | embeddings | Health probe / MCP file check can be satisfied by a different model's files → false "healthy" |
| B8 | P2 | embeddings | `benchmarks/embed_bench.py` never runs inference (dummy vectors + simulated latency) |
| B9 | P2 | embeddings | IQL vector arm capped at 5 candidates regardless of LIMIT |
| B10 | P2 | docs | `VECTOR_SEARCH('…')` documented in 3 files but does not exist in the codebase |
| B11 | P2 | docs | Ollama endpoint documented as `/api/embeddings`; code posts to `/api/embed` |
| B12 | P3 | docs | Model matrix says 9 models; the manifest has 10 (`embeddinggemma-2` missing everywhere) |
| B13 | P3 | docs | `verify.py` documented as a numeric gate ("ONNXvsHF>0.98"); it is a structural smoke test |
| B14 | P3 | embeddings | `sanity_embed.py` resolves paths relative to CWD (unlike verify.py/download.py) |
| B15 | P3 | setup | `setup-embeddings.ps1` claims the engine falls back to dummy; the strict default fails closed |
| B16 | P3 | docs | `LocalOnnxProvider` internals doc drifted (missing `allow_dummy`/`family`, "recursive" onnx scan, `ort::init()`) |
| B17 | P3 | embeddings | `embed_batch` is sequential (N HTTP requests for remote providers) |
| — | P3 | embeddings | HTTP 429 is not classified as retriable (no `Retry-After`, no backoff) — noted in Coverage, not counted as a numbered finding |

**Totals: 10 Part-A findings (1×P1, 7×P2, 2×P3) + 17 Part-B findings (3×P1, 8×P2, 6×P3) = 27 numbered findings.**
