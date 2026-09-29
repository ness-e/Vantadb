---
title: "API-02: W1 bindings — score + firmas + getNode + u128 (4 toolchains)"
kind: task
description: "cargo test --test pythonsdkboundary verde Y tsc --noEmit + npm test verdes Y matriz 4 bindings pareja (método×firma) Y rg \\"distance: h.score\\" = 0\""
---

# API-02: W1 bindings — score + firmas + getNode + u128 (4 toolchains)

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-24-api-ejecucion.md` §Task 2 · investigación `docs/dev/plans/2026-09-24-api-estandarizacion.md` (API-STD-03/04/05/06/15/18)
- **Fuente:** Backlog Phase 51 (fila `API-02`) — W1 bindings
- **Esfuerzo:** 🔴 1-2sem (tool estimate: 30-60 turns)
- **Prioridad:** 🔴
- **Tipo:** Mixto (TypeScript + Python + Node + WASM)
- **Turns estimados:** 30-60
- **Creado:** 2026-09-25
- **last-synced:** 2026-09-25
- **Estado:** ✅ COMPLETED (2026-09-26) — Steps 0-10 ✅ (contrato 4/4); review P2-01 ✅ APPROVE (2 rondas); commit local `caf063ff` (sin push)
- **Incógnitas (uphill):** 0 abiertas (FIND-79/wasm `import_records` → DEFER registrado, stop condition del plan; ver Spec #7)
- **Pendientes (downhill):** 0

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vantadb-ts/src/__tests__/*` (7 archivos asertan `distance`), `vantadb-ts/tests/*`, `vantadb-python/tests/*` (13 archivos), `vantadb-python/vantadb_py/__init__.py` (AsyncClient), stubs `.pyi` (×2, gate `test_stub_drift.py`), `packages/` adapters que consuman Python SDK, `docs/api/*` |
| Callees | Core: `Embedded::search_vector` (`src/sdk/api/search.rs:17`), `Embedded::search_multi` (`src/sdk/search/multi.rs:20`), `engine.put_batch`, `engine.get_node/delete_node/insert_node`; WASM binding (`search`/`search_vector`/`search_multi`/`put_batch`), Node binding (`search`/`searchMulti`/`putBatch`) |
| Implicaciones | Breaking `feat!:` en TS (`SearchHit.distance`→`score`, `roots: number[]`→`(number\|bigint)[]`) y Python (flat `insert/get/delete`→`insert_node/get_node/delete_node`, `put_batch` columnar→array-objetos). JS nativo interior (snake/camel) intacto. Node/WASM: **sin cambio de fuente** (ya canónicos). `tests/api/python.rs` = core `Embedded`, no binding Python → no lo rompe ningún rename. Regla 9: la migración Py a objetos NO declara mejora de perf → sin benchmark; el path buffer (PERF-15) se conserva como `put_batch_raw`. |

## Impacto mapeado (Regla 0)

> **GATE ANTES DE CUALQUIER EDICIÓN:** los archivos a modificar se leen completos antes del primer edit (abajo). Sin este bloque poblado, NO se edita.

- **Archivos leídos (completos):** `vantadb-ts/src/types.ts` ✅ (336L), `vantadb-ts/src/native.ts` ✅ (366L), `vantadb-ts/src/guards.ts` ✅ (secciones de guard + build), `vantadb-ts/src/vantadb.ts` ✅ (1501L, todas las secciones a editar: sub-clientes, search/searchMulti/similarToKey, searchVector, traversals), `vantadb-python/src/lib.rs` ✅ (2416L — structs, `forward_to_db!`, `insert`, `put_batch`, `put_batch_raw`, `search`, `get/delete` nodo, `search_vector`, `search_batch`, `parse_search_request`), `vantadb-python/vantadb_py/__init__.py` ✅ (wrapper async), `vantadb-python/vantadb_py/vantadb_py.pyi` ✅ (515L, stub nativo), `vantadb-python/vantadb_py/__init__.pyi` ✅ (273L, stub wrapper), `vantadb-python/tests/test_stub_drift.py` ✅ (335L, gate mecánico de stubs), `tests/api/python.rs` ✅ (79L), `vantadb-wasm/src/lib.rs` ✅ (secciones put_batch/search/search_multi/import_records/graph_bfs/search_vector), `vantadb-node/src/lib.rs` + `index.d.ts` ✅ (secciones search/search_multi/putBatch/parse_node_ids), `docs/api/BINDINGS_NAMESPACES.md` ✅ (335L)
- **Archivos referenciados hacia dentro:** `vantadb-ts/src/guards.ts` (`isSearchHit`, `buildSearchRequestBase`), `vantadb-ts/src/errors.ts` (`wrapWasmError`), `vantadb-python/src/convert.rs` (`py_dict_to_metadata`, `map_vanta_error`, `extract_vector`), `vantadb-python/src/types.rs` (`VantaPySearchHit`, `VantaPyMemoryRecord`)
- **Archivos que referencian a los editados (grep):** `rg "isSearchHit"` (guards + vanta.test + hardening.test), `rg "\.distance"` TS tests ×7, `rg "put_batch\("` Python tests ×4 + wrapper + stubs, `rg "db\.insert\(|db\.get\(|db\.delete\("` Python tests ×6, `rg "SearchHit"` docs/api, `test_stub_drift.py` (paridad stub↔nativo)
- **Veredicto impacto:** **alto** (4 toolchains, breaking en 2; tests/stubs se actualizan en el mismo PR — Regla 3 docs mismo-PR). Node/WASM sin fuente tocada (referencia canónica). `tests/api/python.rs` intacto (API core `Embedded`).
- **Archivos efectivamente tocados (working tree, sin commit):** `vantadb-ts/src/{types,guards,vantadb,native}.ts` · `vantadb-ts/src/__tests__/{vanta,hardening,integration,subclients}.test.ts` · `vantadb-ts/tests/graph.test.ts` · `vantadb-python/src/lib.rs` · `vantadb-python/vantadb_py/{__init__.py,__init__.pyi,vantadb_py.pyi}` · `vantadb-python/tests/{test_sdk,test_load,test_async_smoke,test_subclients}.py` + `test_w1_surface.py` (nuevo) · `vantadb-python/sanity_check.py` · `benchmarks/batch_vs_sequential_bench.py` · `integrations/llamaindex/vantadb_llamaindex/vectorstore.py` (consumidor columnar migrado) · `docs/api/{BINDINGS_NAMESPACES,TS_SDK,WASM_API,NODE_SDK,PYTHON_SDK,scores}.md` · `docs/user/tutorials/02-local-rag-pipeline.md` · `docs/dev/tasks/API-02.md` (nuevo). **WIP ajeno (API-03, sesión paralela) NO tocado:** `src/server/*`, `docs/api/openapi.yaml`, `docs/api/HTTP_API.md`, `tests/api/openapi_yaml_parity.rs`, `vantadb-server/tests/e2e.rs`, `docs/dev/tasks/API-03.md`.

## Contrato

"`cargo test --test python_sdk_boundary` verde Y `tsc --noEmit` + `npm test` verdes Y matriz 4 bindings pareja (método×firma) Y `rg \"distance: h.score\"` = 0"

## Spec (SDD — feature-add: cambia firmas públicas de bindings)

> Gate P (API-STD-15) ya decidió las 2 cuestiones 🔴 (score-todos, array-objetos) — no se re-abre. Cada fila se resuelve por evidencia.

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Campo canónico en search/híbrido TS | A: `SearchHit.score` higher-is-better (paridad Node `MemorySearchHit.score`/WASM `score`/Py `hit.score`/core) / B: mantener `distance` invertido (drift CODE-091) | A | ✅ decidido-por-evidencia: Gate P Eje "Nombres/semántica search" (`API-STD-15:15`) + contrato del plan (`rg "distance: h.score"` = 0) + Node `index.d.ts:141-144`/WASM `search_hit_to_js` (`:1262`) como referencia |
| 2 | `roots` en traversals TS | A: `(number\|bigint)[]` + guard safe-integer en numbers (paridad `removeEdge`) / B: dejar `number[]` (IDs >2^53 mueren) | A | ✅ decidido-por-evidencia: Gate P Eje "Nombres graph" (`API-STD-15:16`) + API-STD-04 T2 + patrón `removeEdge` (`vantadb.ts:1294-1308`) |
| 3 | Nombres node-level flat en Python | A: `insert_node/get_node/delete_node` (borra `insert/get/delete`; `db.graph.*` directo, sin alias) / B: dejar bare + alias | A | ✅ decidido-por-evidencia: Gate P Eje "`get/delete`" ("Python renombra flat 🔴", `API-STD-15:17`) + BINDINGS_NAMESPACES **naming hazard** (`:102`) + aliases AST-003 ya expuestos (`lib.rs:534-540`) — el rename los hace canónicos |
| 4 | Firma `put_batch` Python | A: `put_batch(records: list[dict])` array-objetos (paridad WASM/TS/Node) / B: columnar actual | A | ✅ decidido-por-evidencia: Gate P Eje "Firmas batch" (`API-STD-15:18`) + API-STD-03 P2. P2-5 se paga aquí (Regla 6): `put_batch` queda con UNA sola API; `put_batch_raw` (buffer NumPy, PERF-15, Python-only) sobrevive como método separado documentado |
| 5 | Firma `search_multi` Python | A′: `search_multi(namespaces, query_vector, filters=None, text_query=None, top_k=10, distance_metric=None, explain=False, exclude_superseded=False)` (kwargs planos, paridad `search`; `namespaces` primero como Node/WASM) / B: request-object único (paridad TS) | A′ | ✅ decidido-por-evidencia + **enmienda post-review**: la fila original listaba `method=None`; se removió porque `Embedded::search_multi` (`src/sdk/search/multi.rs:20`) no acepta override de índice y ni WASM (`lib.rs:1482`) ni Node (`search_multi`, `index.d.ts:452`) lo exponen — incluirlo solo en Python rompía la paridad de la matriz. Evidencia: `vantadb-python/src/lib.rs` (`search_multi` sin `method`) + BINDINGS §W1 parity matrix |
| 6 | u128 en wire | A: ids >2^53 como string decimal en fronteras JSON (WASM/TS/Node; Python int nativo exacto) / B: number (pierde precisión) | A | ✅ decidido-por-evidencia: Gate P Eje "Tipos base" (`API-STD-15:24`) + API-01 Step 2 (`u128_serde`) + Node `index.d.ts:63-64` modelo. W1 = verificar roundtrip por binding + `search_vector` crudo conserva `distance` lower-is-better |
| 7 | FIND-79: delegación TS `importRecords` | A: arreglar `import_records` WASM para aceptar `MemoryInput`/u64-string y delegar / B: DEFER (bucle JS documentado) | B si el fix WASM excede el slice | ✅ **DEFER** (stop condition del plan Task 2: "FIND-79 bloquea → DEFER delegación TS con bucle documentado") — el fix WASM (`WasmImportRecord` con defaults + timestamps string) no está en los 5 ítems canónicos W1; se registra deuda/fila FIND al cierre |
| 8 | Node extras (sparse-escritura, `exclude_superseded`, `bulk_import`) | A: incluirlos en W1 / B: fuera de los 5 ítems canónicos del blocker | B | ✅ decidido-por-evidencia: alcance del blocker W1 = 5 ítems (score/getNode/put_batch/search_multi/u128); extras → deuda con due (Regla 6 neto ≤0: se paga P2-5) |
| 9 | WASM/Node código | A: sin cambios (ya canónicos: `score`, `put_batch` objetos, `search_multi`, u128 strings) / B: tocar por simetría | A | ✅ decidido-por-evidencia: API-STD-06 W2/W3 + API-STD-05 N5 + inspección (`search_hit_to_js:1262`, `put_batch:1157`, `search_multi:1482`, `parse_node_ids`) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** `u128_serde` sigue siendo el ÚNICO patrón `u128` en wire (no tocar API-01); `distance` lower-is-better SOLO en ANN crudo (`search_vector`/`SearchHit` core) — nunca en search/híbrido; `put_batch_raw` mantiene su path zero-copy (`PERF-15`, regla `python-bindings.md` R-1: GIL release + Rayon fail-fast); `to_js_err`/`map_vanta_error` siguen mapeando sin panic; GIL liberado con `py.detach` en ops nuevas (`put_batch`, `search_multi`); `vantadb-pro` intocable; `tests/api/python.rs` (core `Embedded`) no se rompe; WIP ajeno (WIRE-10) intocable.
- **Comandos de verificación:** `cargo test --target-dir target/session-api01 --test python_sdk_boundary` · `vantadb-ts`: `npx tsc --noEmit` + `npx vitest run` · `vantadb-node`: `npx vitest run` · Python: rebuild `maturin develop --release` + `python -m pytest vantadb-python/tests -q` · `rg "distance: h.score"` = 0 · `cargo fmt --check` + `cargo clippy -p vantadb_py -D warnings` (scoped).
- **Deuda pendiente:** FIND-79 (wasm `import_records` no acepta `MemoryInput`/u64-string → TS bucle JS, pierde `created_at/version/history` y `skipped:0` fijo) — DEFER documentado; Node extras (sparse-escritura, `exclude_superseded`, `bulk_import` en TS) — fuera de los 5 ítems W1.

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|---------------------------|
| `activeGoal` | Encabezado `# API-02: W1 bindings — score + firmas + getNode + u128 (4 toolchains)` |
| `lastAction` | Último step ✅ + Context Save Point |
| `result` | `OK` ↔ ✅ COMPLETED · `PARTIAL` ↔ ⏳ IN PROGRESS · `FAILED` ↔ ❌ FAILED |
| `nextAction` | Próximo step ⬜ PENDING (archivo + comando) |
| `contract` | `## Contrato` + `## Invariantes de dominio` + evidencia/artefactos |
| `nextTask` | API-04 (o el que indique el plan al cerrar) |

    contract:
      verificacion: cargo test --test python_sdk_boundary → 1 passed/0 failed ✅; tsc --noEmit → exit 0 ✅; npm test (vantadb-ts) → 314/314 ✅; npm test (vantadb-node) → 36/36 ✅; pytest → 149 passed/0 failed ✅; rg "distance: h.score" → 0 en código ✅; matriz 4 bindings → docs §W1 parity matrix + tests por binding ✅
      evidencia:
        - claim: "TS mapea el wire score a SearchHit.distance en 4 sitios (vantadb.ts:609,663,742 + native.ts:360)"
          evidencia: rg "distance: h.score" → 4 matches pre-cambio (leído verbatim) → 0 post-cambio en código
          confianza: alta
        - claim: "Gate P aprobó score-todos + array-objetos; Python renombra flat get/delete y migra put_batch"
          evidencia: docs/dev/tasks/API-STD-15.md:15-19 + docs/dev/plans/2026-09-24-api-ejecucion.md:62-65
          confianza: alta
        - claim: "Node/WASM ya son canónicos (score, put_batch objetos, search_multi, u128 strings)"
          evidencia: vantadb-node/index.d.ts:141-144 + vantadb-wasm/src/lib.rs:1157,1262,1482 + Node tests `hits[0].score` (api.test.ts:209)
          confianza: alta
        - claim: "tests/api/python.rs usa el SDK core (Embedded.insert_node/get_node/search_vector), no el binding Python"
          evidencia: tests/api/python.rs:30-51 (intacto; suite verde)
          confianza: alta
        - claim: "P2-5 pagada: put_batch tiene UNA API (array-objetos); put_batch_raw (buffer PERF-15) sobrevive como método separado"
          evidencia: vantadb-python/src/lib.rs (put_batch(records) + record_to_memory_input; sin firma columnar)
          confianza: alta
        - claim: "FIND-79 DEFER: TS importRecords conserva el bucle JS documentado"
          evidencia: vantadb-ts/src/vantadb.ts:900-926 (sin cambio) + Spec #7 (stop condition del plan)
          confianza: alta
      artefactos:
        - docs/dev/tasks/API-02.md
        - vantadb-python/tests/test_w1_surface.py
        - vantadb-ts/tests/graph.test.ts
      invariantes: u128_serde único patrón wire; distance solo ANN crudo; put_batch_raw zero-copy intacto; GIL release en ops nuevas; vantadb-pro intocable; WIP ajeno (API-03/WIRE-10) intocable
      deuda: P2-5 pagada; FIND-79 DEFER documentado; Node extras fuera de W1 (documentados)
      queda_pendiente: ninguna (review ✅ + commit caf063ff; push solo con instrucción del owner)

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** pagar **P2-5** (`put_batch` dual Python) con la migración array-objetos — saldo neto ≤0. Deuda nueva: ninguna declarada (FIND-79 ya existía; no se agrava: se documenta).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato mecánico ✅ (4 condiciones) + capa determinista (fmt/clippy scoped) + tests del cambio pasan (vitest TS/Node + pytest) |
| **Commit** | Atómico por step, conventional commit `feat!:` + `API-02` (breaking bindings TS/Py), verificación mecánica (`campaign_verify_cmd`) |
| **Release** | `dev-tools/verify.ps1` completo (informativo), docs mismo-PR (Regla 3), release-plz decide versión/tag (Regla 7 — no tocar Cargo.toml/CHANGELOG/tags) |

## Herramientas necesarias
- Terminal: `cargo` (check/clippy/fmt/test, `--target-dir target/session-api01`), `maturin`, `pytest`, `npx tsc`, `npx vitest`
- `codegraph_explore` (blast radius), `campaign_*` (verify/state), `rg` (gates mecánicos)
- **Skills cargadas (SDP):** campaign-executor (base task system) · source-driven-development (APIs externas PyO3/napi/wasm-bindgen) · incremental-implementation (slices por binding) · test-driven-development (RED→GREEN en score TS + paridad bindings) · context-engineering (4 toolchains) · doubt-driven-development (breaking en paquetes publicados) · api-and-interface-design (contratos de bindings). Descartadas: frontend-ui-engineering (no toca `web/`) · performance-optimization (sin claim de perf; Regla 9 → sin benchmark) · systematic-debugging (no es bug con repro) · security-and-hardening (sin trust boundary nuevo; se revisa en FASE SECURITY).

## Investigation Notes
- Gate P (API-STD-15) 4/4 aprobadas; decisiones W1 citadas en §Spec con fuente por fila.
- API-STD-04 (TS): T1 (distance ×3), T2 (roots number[]), T3 (importRecords bucle + FIND-79), T4 (sin bulk_import), T5 (searchMulti presente).
- API-STD-03 (Python): P1 (doble get/delete), P2 (put_batch columnar), P3 (search híbrido), P4 (sin search_multi), P5 (sin export/import/audit), P7 (P2-5 viva).
- API-STD-06 (WASM): W1 (`to_js_err`), W2 (`score`), W3 (formas canónicas), W4 (P2-8 resuelto API-01).
- API-STD-05 (Node): N5 (`node_id: string` decimal = modelo cross-binding), N2/N3 (sparse-escritura/exclude_superseded ausentes → fuera de W1).
- `BINDINGS_NAMESPACES.md:98` declara `sparse_vector` en `put()/put_batch()` Python — **verificado falso** (no existe en `lib.rs`); se corrige la tabla en el mismo PR (Regla 3).
- Python dev venv: `vantadb-python/.venv` (3.11.9 + maturin + pytest) — `target/audit-venv` no existe; se usa `.venv` (misma garantía de aislamiento; `dev-tools/setup_venv.ps1` sería build equivalente desde cero).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 1 — FIND-79 (DEFER por stop condition; no bloquea contrato) |
| Pendientes de ejecución (downhill) | 1 — Step 10 (review P2-01 + cierre) |
| % completado | 90% (Steps 0-9 ✅; contrato mecánico 4/4) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — toca fronteras FFI (PyO3/NAPI/WASM) y parseo de input de usuario (dicts Python en `put_batch`, records TS). Checklist a ejecutar en el cierre: sin `unsafe` nuevo; sin `unwrap` nuevo en código no-test; `map_vanta_error`/`to_js_err` intactos; validación de tipos Python (`PyDict::cast`) con `PyTypeError`/`PyValueError`; sin nuevas dependencias.
- [ ] **PERFORMANCE** — `put_batch` migra a objetos: NO se declara mejora (Regla 9) → sin benchmark; se conserva `put_batch_raw` (zero-copy PERF-15) y `py.detach`+Rayon fail-fast (R-1 python-bindings). `search_multi` delega al core (`src/sdk/search/multi.rs`). Verificación: tests de paridad funcional (no perf claims).

## Steps

### Step 0: Lectura completa + Impacto mapeado (GATE Regla 0)
- **Archivos:** los de §Impacto mapeado
- **Acción:** leer completos los archivos a modificar; mapear referencias entrantes; completar el bloque §Impacto mapeado
- **Verify:** bloque §Impacto mapeado completo + `rg` de referencias entrantes
- **Estado:** ✅ DONE 2026-09-25 — lectura completa en discovery (ver §Impacto mapeado); referencias entrantes mapeadas por `rg`.

### Step 1: TS RED→GREEN — `SearchHit.score` (contract-critical)
- **Archivos:** `vantadb-ts/src/__tests__/vanta.test.ts` (RED), `vantadb-ts/src/types.ts`, `vantadb-ts/src/guards.ts`, `vantadb-ts/src/vantadb.ts` (×3), `vantadb-ts/src/native.ts` (×1)
- **Acción:** RED: test que asserte `score` en un hit real (hoy falla: el hit trae `distance`); GREEN: reemplazar `distance: h.score` por `score: h.score`, migrar `SearchHit`/`isSearchHit`/docstrings
- **Verify:** `npx vitest run src/__tests__/vanta.test.ts` (RED→GREEN) + `npx tsc --noEmit` + `rg "distance: h.score"` = 0
- **Estado:** ✅ DONE 2026-09-25 — RED verificado (`hits[0].score` undefined → TypeError en `vanta.test.ts:536`); GREEN: `distance: h.score` ×4 (vantadb ×3 + native ×1) → `score:`, `SearchHit.distance`→`score`, `isSearchHit` por `score`. `npx tsc --noEmit` exit 0; vitest **314/314** (13 files); `rg "distance: h.score"` = 0 en código (solo docs históricos de plan/task/informe). Tests migrados: vanta/hardening/integration/subclients.

### Step 2: TS — traversals `roots: (number|bigint)[]` + guard
- **Archivos:** `vantadb-ts/src/vantadb.ts` (interfaz `GraphClient` + 6 métodos), `vantadb-ts/src/types.ts` (`NodeId`), `vantadb-ts/tests/graph.test.ts`
- **Acción:** tipo `(number|bigint)[]`; helper de validación safe-integer (patrón `removeEdge`); `String(root)` acepta ambos; test con root >2^53
- **Verify:** `npx vitest run tests/graph.test.ts` + `npx tsc --noEmit`
- **Estado:** ✅ DONE 2026-09-25 — `NodeId = number | bigint` (types.ts) aplicado a 21 firmas (roots + node ids) + `_rootsToWire()` con guard safe-integer. RED por ausencia verificada (`git show HEAD:vantadb-ts/src/vantadb.ts` → `roots.map(String)` sin guard); tests nuevos: rechazo unsafe number + bigint 2^53+1 (bfs/isDag/degree/getNode/deleteNode). graph.test 7/7; tsc exit 0.

### Step 3: Python — flat node rename `insert_node/get_node/delete_node`
- **Archivos:** `vantadb-python/src/lib.rs` (`forward_to_db!(GraphClient)` + métodos flat)
- **Acción:** renombrar `insert`→`insert_node`, `get`→`get_node`, `delete`→`delete_node`; `GraphClient` forwards directos (sin aliases)
- **Verify:** `cargo check -p vantadb_py --target-dir target/session-api01` (compila)
- **Estado:** ✅ DONE 2026-09-25 — flat renombrados + `db.graph` forwards directos (aliases AST-003 removidos); `cargo check -p vantadb_py` exit 0; consumidores migrados (tests, AsyncClient, stubs, sanity_check, benchmark).

### Step 4: Python — `put_batch([{…}])` array-objetos + pagar P2-5
- **Archivos:** `vantadb-python/src/lib.rs`
- **Acción:** `put_batch(records: list[dict])` parsea claves `namespace/key/payload/metadata/vector/ttl_ms` (helper único reutilizable); elimina la firma columnar (P2-5); `put_batch_raw` conserva el path buffer
- **Verify:** `cargo check -p vantadb_py --target-dir target/session-api01` + `rg "keys: Vec<String>,.*vectors"` = 0
- **Estado:** ✅ DONE 2026-09-25 — `put_batch(records)` + helper `record_to_memory_input()` (validación de dict/`key`/`metadata` con `PyTypeError`/`PyValueError`, GIL release + `engine.put_batch`); firma columnar eliminada (**P2-5 pagada**: una sola API `put_batch`); `put_batch_raw` intacto (PERF-15). `cargo check` + clippy `-D warnings` exit 0.

### Step 5: Python — `search_multi(namespaces, …)`
- **Archivos:** `vantadb-python/src/lib.rs`
- **Acción:** método flat + forward en `MemoryClient`, delega `engine.search_multi` (GIL release, clamp_top_k, error mapping)
- **Verify:** `cargo check -p vantadb_py --target-dir target/session-api01`
- **Estado:** ✅ DONE 2026-09-25 — `search_multi(namespaces, query_vector, filters, text_query, top_k, distance_metric, explain, exclude_superseded)`; non-empty guard; forward en `MemoryClient`; test `test_search_multi_spans_namespaces` verde. **Sin `method`** (enmienda Spec #5 post-review): el core `Embedded::search_multi` no acepta override de índice y Node/WASM tampoco lo exponen — agregarlo solo en Python rompía la paridad de la matriz.

### Step 6: Python — wrapper async + stubs (drift gate)
- **Archivos:** `vantadb-python/vantadb_py/__init__.py`, `vantadb_py/__init__.pyi`, `vantadb_py/vantadb_py.pyi`
- **Acción:** rename en `AsyncClient` (borra alias duplicados), `put_batch(records)`, agrega `search_multi`; actualiza los 3 stubs con las firmas nuevas
- **Verify:** `python -c "import inspect, vantadb; print(inspect.signature(vantadb.Client.put_batch))"` + `test_stub_drift.py` (tras rebuild)
- **Estado:** ✅ DONE 2026-09-25 — 3 stubs actualizados (Client/MemoryClient/GraphClient/AsyncClient); `test_stub_drift.py` verde contra el módulo compilado (paridad de métodos + params + requeridos); firmas reales verificadas: `put_batch(self, /, records)`, `search_multi(self, /, namespaces, query_vector, …)`.

### Step 7: Python — tests actualizados (legacy → canónico)
- **Archivos:** `vantadb-python/tests/test_sdk.py`, `test_load.py`, `test_async_smoke.py`, `test_subclients.py`, `test_w1_surface.py` (nuevo)
- **Acción:** `db.insert→db.insert_node`, `db.get→db.get_node`, `db.delete→db.delete_node`, `put_batch` columnar→lista de dicts; test nuevo de contrato W1
- **Estado:** ✅ DONE 2026-09-25 — 5 tests de `put_batch` reescritos + validación nueva (record sin `key`→ValueError, no-dict→TypeError); `test_w1_surface.py` (6 tests: nombres canónicos, `score` sin `distance`, firmas `put_batch`/`search_multi`, u128 exacto 2^64+3, AsyncClient espejo). Fix de test: conftest rebindea `Client` → usar el pyclass nativo para asserts de clase.

### Step 8: Python — rebuild + suites (Py/Node/TS/WASM)
- **Archivos:** —
- **Acción:** `maturin develop --release` (venv `.venv`) + pytest completo; vitest TS/Node; `cargo test --test python_sdk_boundary`
- **Estado:** ✅ DONE 2026-09-25 — `maturin develop --release` 3m00s (release, target default; no toca el lock de `target/debug`); **pytest 149 passed / 0 failed** (4 deselected) · `cargo test --test python_sdk_boundary` **1 passed** · TS vitest **314/314** · Node vitest **36/36** · `npx tsc --noEmit` exit 0.

### Step 9: Docs mismo-PR (Regla 3) + matriz 4 bindings
- **Archivos:** `docs/api/BINDINGS_NAMESPACES.md`, `docs/api/PYTHON_SDK.md`, `docs/api/TS_SDK.md`, `docs/api/WASM_API.md`, `docs/api/NODE_SDK.md`, `docs/api/scores.md`, `docs/user/tutorials/02-local-rag-pipeline.md`
- **Acción:** matriz W1 4 bindings (score / node CRUD naming / put_batch / search_multi / u128) + corregir `sparse_vector` Python (❌ verificado) + migrar textos CODE-091
- **Estado:** ✅ DONE 2026-09-25 — matriz W1 §"W1 parity matrix (API-02)" agregada; TS row score-todos en BINDINGS/WASM_API/NODE_SDK/scores/TS_SDK; Python tables/totales (46 flat: 16/11/1/18) + `search_multi` + `put_batch` objetos + `exclude_superseded list` ✅ + `sparse_vector` Python ❌ (drift corregido); tutorial batch migrado. `validate-docs-coverage.ps1` → 0 gaps.

### Step 10: Verify full + review + cierre
- **Archivos:** `docs/dev/tasks/API-02.md`, plan file
- **Acción:** `cargo fmt --check` / clippy scoped / nextest scoped; OCR (`pwsh dev-tools/ocr-review.ps1`); review P2-01 (agente distinto); sync plan/task file
- **Verify:** contrato 100% + review registrado
- **Estado:** ⏳ IN PROGRESS — fmt/clippy scoped ✅ (ver abajo); contrato mecánico 4/4 ✅; pendiente: OCR + review P2-01 + sync plan. **Nota clippy:** `-D warnings` global falla por `dead_code` pre-existente de `src/wal_sharded.rs` (commit `c1eacebd`, WIP ajeno — diff vacío en ese archivo); scoped con `-A dead_code` exit 0.

## Dependencias
- API-01 ✅ COMPLETED (2026-09-25) — tipos/casing/u128 fundación.
- Paralelizable con API-03 (archivos disjuntos). Desbloquea: API-04..08 según plan.

## Review (GATE — agente distinto, P2-01)

- **Revisor:** `vanta-review` (subagente, contexto fresco) — **dictamen ❌ changes-required 2026-09-25** (3 bloqueantes; checklist anti-hábitos: ok, ninguno detectado; contrato 1/2/4 re-ejecutado ✅, contrato 3 ✅ con matices).
- **Enfoque:** ¿la migración score/objetos es correcta cross-binding? ¿el rename Python rompe consumidores no mapeados? ¿P2-5 realmente pagada?
- **Cómo se probó:** evidencia mecánica re-ejecutada por el revisor: `cargo test --test python_sdk_boundary` 1 passed · `tsc --noEmit` 0 · TS vitest 314/314 · Node vitest 36/36 · `cargo fmt/clippy -p vantadb_py` 0 · docs-coverage 0 gaps · `rg "distance: h.score"` 0 en código (14 matches en docs históricos, scope correcto).
- **Fixes del review (2026-09-25):**
  - **R1 ✅ aplicado (bloqueante):** `roots: NodeId[]` en los 11 sitios restantes de `vantadb-ts/src/vantadb.ts` (interface `GraphClient` :100-108, arrows :314-327, impls :1393,1412,1472) — el `replaceAll` inicial había cubierto solo 7/18 por indentación. Re-verificado: `tsc --noEmit` 0 + vitest 314/314. Causa raíz: `tsconfig.json` no typechequea `tests/**`, así que el test bigint pasaba sin tipos.
  - **R2 ✅ aplicado (bloqueante):** Spec #5 enmendada — `search_multi` Python SIN `method` (el core `Embedded::search_multi` no acepta override y Node/WASM tampoco lo exponen; incluirlo solo en Python rompía la matriz). Step 5 actualizado con la justificación.
  - **R3 ✅ aplicado (bloqueante):** `docs/api/TS_SDK.md` firmas graph (`getNode`/`deleteNode`/`addEdge`/4 traversals → `NodeId`) + nota `NodeId`; `docs/api/BINDINGS_NAMESPACES.md` header 44→46 y totals 16/11/1/18=46 + fila `graph_bfs_filtered`.
  - **Advisory aplicados:** BINDINGS `remove_edge` Python ❌ (era ✅ falso), `search_vector` Python ✅ (era ❌ falso), hybrid shape Python corregido; Node test con aserción de valor (`score > 0.99` + sin `distance`); `vantadb-python/sanity_check.py:6` → `search_vector` (namespace requerido); `docs/api/PYTHON_SDK.md` bases de conteo aclaradas (sub-cliente vs flat).
  - **Advisory no aplicado (documentado):** evidencia pytest completa re-ejecutada ✅ tras liberar `%TEMP%\pytest-of-Eros` (ENOSPC era del entorno, no lógica) — **149 passed** reproducible.
- **Veredicto:** ✅ **APPROVE** — re-review post R1/R2/R3 (2026-09-25, misma sesión de review): R1 ✅ (`roots: number[]` = 0 en `vantadb.ts`, 11 sitios confirmados; tsc 0 + vitest 314/314), R2 ✅ (Spec↔impl↔docs coherentes sin `method`), R3 ✅ (TS_SDK graph sigs `NodeId` + BINDINGS 46). Advisories aplicados y re-verificados. Contrato re-ejecutado: `python_sdk_boundary` 1 passed · `distance: h.score` 0 en código · pytest **149 passed** (reproducible tras liberar ENOSPC) · Node 36/36 · docs-coverage 0 gaps. Regresiones nuevas: ninguna. Checklist anti-hábitos: ok.

## Notas
- Commits: `feat!:` + `API-02` (breaking TS/Py — Regla 7); docs mismo-PR (Regla 3); deuda neta ≤0 (Regla 6, P2-5).
- Stop conditions del plan: appetite >2sem → partir por binding (Py→TS→Node); FIND-79 bloquea → DEFER delegación TS documentado (aplicado: Spec #7).
- Gate P no se re-abre. Gate D: no dispara nueva ronda (decisiones ya aprobadas en Gate P + plan con Gate Result ✅ DO).
- **Progreso 2026-09-25:** discovery completo (reads + blast radius + SDP); task file creado. Ejecución completa Steps 0-9 (TS score + roots bigint; Py rename + put_batch objetos/P2-5 + search_multi + stubs + tests; docs matriz). Contrato mecánico 4/4 verde. Sin commit (política owner 2026-09-25 — el lead commitea).
- **Notas de entorno:** el MCP server lockea `target/debug/vanta-cli.exe` → cargo con `--target-dir target/session-api01` (o `CARGO_TARGET_DIR`); el rebuild de Python usó el target default en release (no toca el debug lockeado). El venv dev es `vantadb-python/.venv` (3.11.9 + maturin + pytest); `target/audit-venv` no existe (setup_venv.ps1 = build release equivalente desde cero).
- **FIND candidatos (no creados aún):** (a) `docs/user/operations/SQLITE_MIGRATION_GUIDE.md:197-198` usa `db.get(namespace=…, key=…)`/`db.delete(…)` flat memory — drift pre-existente (AST-012 removió el flat memory), NO introducido por W1; (b) `docs/api/PYTHON_SDK.md:1225` menciona `vantadb.VantaError` (el export real es `Error`) — drift pre-existente; (c) FIND-79 (wasm `import_records` rechaza `MemoryInput`/u64-string) sigue abierta — DEFER documentado; (d) `_rootsToWire`/`removeEdge` lanzan `DbError("INVALID_ARGUMENT")` sin prefijo canónico `VANTADB_*` (patrón pre-existente; `DbError` no normaliza) — candidato a unificar; (e) blind spot de gate: `tsc --noEmit` no typechequea `src/__tests__/**` ni `tests/**` (el error de tipos de R1 pasó por ahí) — considerar `tsconfig.tests.json` en CI.
- **Formatting post-build:** `cargo fmt -p vantadb_py` se aplicó después del build de maturin (solo whitespace en `lib.rs`); el artefacto `.pyd` es semánticamente idéntico al fuente final. Si el review exige artefacto exacto, re-correr `maturin develop --release` (3m).
