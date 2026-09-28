# WIRE-03: `query_sparse` + text-only en 3 bindings + filtros avanzados (`$and`/`$or`, range/datetime)

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 17, Fase F2 — wave WIRE-03 ‖ WIRE-04)
- **Fuente:** Backlog:968 (P56, 2026-09-24) + plan master Task 17
- **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🔴 · **Tipo:** feature-add (paridad bindings Py/TS/Node)
- **Branch:** develop · **Commit esperado (LEAD):** `feat(bindings): query_sparse + text-only + advanced filters (WIRE-03)`
- **Estado:** 🟡 6/6 steps ✅ + verify mecánico completo — pendiente SOLO commit + review fresco (LEAD; instrucción explícita del orquestador)
- **Creado:** 2026-09-27 · **last-synced:** 2026-09-27
- **Incógnitas (uphill):** 0 — `$or` dividido por stop condition (FIND-180); el resto entregado · **Pendientes (downhill):** 0 steps

## Contrato (verbatim del plan — ley)
> "`query_sparse` expuesto en Py/TS/Node con test de roundtrip Y text-only (`query_vector: []` = solo-BM25) por la puerta principal en los 3 Y filtros avanzados (`$and`/`$or`, range/datetime) equivalentes py↔js con tests Y stubs/d.ts sincronizados (`test_stub_drift` verde)"

**Matriz de verificación (por binding):**

| Cláusula | Py | TS | Node |
|---|---|---|---|
| `query_sparse` roundtrip | `put(sparse_vector=…)` → `search(query_sparse=…)` → hit (test) | `search({query_sparse})` vía `vantadb/native` → hit (test; WASM backend lanza error explícito) | `put({sparse_vector})` → `search({query_sparse})` → hit (test) |
| text-only `[]` + `text_query` | `[]` solo → resultado vacío (core skip, pre-existente); con `text_query` → BM25 (test) | `[]` solo → **error claro de intención** (guard nuevo, distingue `text_query`/`sparse`); con text → BM25 (test WASM) | `[]` solo → resultado vacío (core skip, pre-existente — `get_f32_vec` nunca rechazó); con text → BM25 (test) |
| filtros advanced (range/datetime, AND) | `count/delete_by_filter` con `$gte`+`$lte` + `datetime` (test) | `count/deleteByFilter/exportNamespace` aceptan DSL canónica `{"campo":{"$op":v}}` + `DateTime` (test) | `count/deleteByFilter` con `FilterItem[]` Gte/Lte + `{DateTime}` (test) |
| `$or` | **diferido** — no representable en `MemoryFilter = Vec<MemoryFilterItem>` (AND-only) → FIND-180 (stop condition del plan) | ídem | ídem |
| stubs/d.ts sync | `test_stub_drift.py` verde | `tsc` verde | `index.d.ts` regenerado + `tsc`/vitest verde |

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| **Callers** | Py: `AsyncClient.search/search_multi` (wrapper) + `.pyi` stubs ← `test_stub_drift.py`; Node: `VantaDb.search/searchMulti/explainSearch/searchWithMethod/put/putBatch` ← `vantadb-ts/src/native.ts`; TS: `Client.search/searchMulti/explainSearch` (wasm) + `count/deleteByFilter/exportNamespace` ← tests + usuarios del SDK |
| **Callees** | `MemorySearchRequest` (`src/sdk/serialization/vector_types.rs:10-55`) → `search_impl` (`src/sdk/search/mod.rs:100-292`, rutas text-only/vector/sparse/híbrida); `MemoryFilter`/`FilterOp`/`Value::DateTime` (`src/sdk/types/record.rs`, `src/sdk/types.rs:102-125`) |
| **Implicaciones** | Aditivo y rollback-friendly: kwargs/params nuevos al final y opcionales; TS relaja `query_vector` vacío SOLO con `text_query`/`query_sparse` presente (mensaje distinto si no → sin cambio de comportamiento para el input viejo); `$or` NO se simula (fuera del modelo core, FIND-180); WASM crate intacto |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/sdk/serialization/vector_types.rs` · `src/sdk/types/record.rs` · `vantadb-ts/src/guards.ts` · `vantadb-ts/src/metadata.ts` · `vantadb-python/tests/test_stub_drift.py` · `vantadb-ts/src/__tests__/d5a-validation.test.ts`.
- **Archivos leídos (rangos clave):** `vantadb-python/src/lib.rs` (:460-599 list/put_batch parse, :1000-1119 delete_by_filter/count, :1180-1369 search/search_multi, :2230-2429 explain/parse_search_request) · `vantadb-python/src/convert.rs` (:196-265 extract_vector, :690-750 py_dict_to_filter_ops) · `vantadb-python/vantadb_py/__init__.py` (:108-307) · `vantadb-node/src/lib.rs` (:200-214, :500-630, :640-1199) · `vantadb-node/index.d.ts` + `dts-header.d.ts` · `vantadb-ts/src/types.ts` (:11-155, :227-236) · `vantadb-ts/src/native.ts` (:1-80, :240-366) · `vantadb-ts/src/vantadb.ts` (:1-60, :530-869) · `vantadb-wasm/src/lib.rs` (:80-199 SearchRequest, :1260-1379 search/explain — solo lectura, fuera de scope) · `vantadb-mcp/src/validation.rs:309-349` y `src/cli_handlers/crud.rs:461-510` (forma canónica del DSL).
- **Referencias hacia dentro:** `py_dict_to_metadata` 4 usos en search paths (los que se extienden); `get_metadata`/`get_f32_vec`/`parse_filter_items` (node); `buildSearchRequestBase` compartido por los paths nativo y wasm de TS (`native.ts:331-340`; `vantadb.ts:580-596`) → cualquier relajación de validación aplica a AMBOS backends; `normalizeValue`/`assertTaggedValue` compartidos metadata/filtros; stubs Py sujetos a `test_stub_drift.py` (paridad de métodos + params + requeridos vía `inspect.signature`).
- **Referencias entrantes:** `vantadb-ts/src/__tests__/*` (11 archivos; `d5a-validation` pinea el mensaje de `query_vector` vacío), `vantadb-node/tests/api.test.ts` + `graph.test.ts` + `persistence.test.ts`, `vantadb-python/tests/*` (13 archivos), `docs/api/TS_SDK.md` + `docs/api/PYTHON_SDK.md` + `docs/api/NODE_SDK.md` (Regla 3).
- **Veredicto:** blast radius acotado a frontera FFI + validación TS. Núcleo NO se toca (solo 1 test de pin opcional en `src/sdk/serialization/mod.rs`). Sin cambios de wire on-disk (WAL/formatos intactos → review tier: **fast**, paths `vantadb-*/**` + tests; `src/sdk/**` del test-pin cae en adversarial ≥1 path → verificarlo al cierre).

## Spec (feature-add — Gate mecánico spec-first)

| # | Decisión | Opciones | Evidencia / Default |
|---|----------|----------|---------------------|
| 1 | **Forma canónica de filtros** | (a) nueva DSL `$and/$or` anidada / (b) **la existente cross-SDK** `{campo: valor}` + `{campo: {"$op": v}}` (flat AND) con `$eq/$neq/$gt/$gte/$lt/$lte` | (b): idéntica en `py_dict_to_filter_ops` (convert.rs:708), `parse_filter_ops` MCP (validation.rs:309), `parse_filter_json` CLI (crud.rs:461); mapea 1:1 a `MemoryFilterItem`/`FilterOp` core (record.rs:27-55); rango = `$gte`+`$lte`; datetime = `Value::DateTime` (types.rs:112) aceptado por Python (convert.rs:65) y por serde en Node |
| 2 | **`$or`** | (a) anidar en core `MemoryFilter` / (b) **diferir con FIND** | (b): `MemoryFilter = Vec<MemoryFilterItem>` AND-only (record.rs:55, doc líneas 11-25) — OR requiere cambiar el modelo core (stop condition del plan: "OR/range diferido con FIND"); range SÍ es representable (`$gt/$gte/$lt/$lte`) y se entrega. FIND-180 |
| 3 | **Text-only** | (a) aceptar `[]` siempre / (b) **aceptar `[]` solo con `text_query` no-vacío o `query_sparse` non-empty** | (b): pre-mortem F1 del plan ("TS acepta `[]` sin distinguir intención → exigir `text_query` presente"); error claro si `[]` solo; core ya rutea text-only (`src/sdk/search/mod.rs:111` `has_vector=false`, ruta "text-only" debug_ops.rs:315) |
| 4 | **`query_sparse` wire** | (a) lista de pares / (b) **mapa `{dim: peso}`** | (b): mismo shape que `MemoryInput.sparse_vector` ya documentado en TS/WASM (`types.ts:48-51`; wasm d.ts SparseVector) y `SparseVector(BTreeMap<u32,f32>)` core; `None`/`{}` = skip (core filtra vacío, mod.rs:112-115) |
| 5 | **Sparse en `put` Py/Node** | (a) solo search / (b) **put + search** | (b): roundtrip requiere insertar sparse (WASM/TS ya lo exponen en put; Py/Node no → test imposible). Simetría de superficie |
| 6 | **TS + WASM backend** | (a) pasar y dejar que WASM ignore (serde default) / (b) **DbError explícito** | (b): WASM `SearchRequest` no tiene `query_sparse` (wasm lib.rs:152-168) y el plan lo declara fuera de scope → el campo se descartaría en silencio (verificado en vivo: node actual ignora `query_sparse`); error explícito distingue intención. El path `vantadb/native` SÍ lo soporta |
| 7 | **TS filters** | (a) reescribir a `$op` / (b) **aditivo `FilterItem[] \| FilterInput`** | (b) Backlog:968 "filtros intercambiables … py↔js"; la DSL `$op` se normaliza a `FilterItem[]` en TS (sin tocar WASM); `FilterItem[]` PascalCase queda como forma nativa wire-compatible. DateTime: `Value` TS += `DateTime`/`ListDateTime` (+ `Date` JS → ISO) para paridad con Python `datetime` |
| 8 | **Stubs/d.ts** | — | `.pyi` (2) + `d.ts` node (header + generado) + `types.ts`; gates: `test_stub_drift.py`, `tsc`, `napi build` |
| 9 | **`$and`** | — | = AND implícito del objeto plano / lista `FilterItem[]` (ya soportado por los 3); sin wrapper anidado (no hay precedente ni necesidad) |

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** params nuevos AL FINAL y opcionales (compat posicional Py/Node); `[]` sin `text_query`/`query_sparse` sigue siendo error en TS (contrato D5a); WASM crate intacto; `json query_sparse: null` = `None`; sparse vacío = skip (paridad core); no tocar los PROHIBIDOS del prompt (WIP ajeno).
- **Comandos de verificación:** ver §Steps (cada step).
- **Deuda pendiente al abrir:** ninguna (FIND-180 nace al cierre).

## Herramientas
- `codegraph_codegraph_explore` (discovery ✅) + `codebase-memory-mcp` (búsquedas `query_sparse`/filtros ✅).
- Rust: `cargo check -p vantadb-python` · `cargo clippy -p vantadb-python --all-targets -- -D warnings` · Node standalone (workspace vacío): `cargo check --manifest-path vantadb-node/Cargo.toml` / `clippy` ídem · `cargo nextest run -p vantadb --lib sdk::serialization` (test-pin).
- Py: rebuild `maturin develop` (venv `vantadb-python/.venv`, abi3-py311) + `pytest` (venv) → `test_wire03.py` + `test_stub_drift.py`.
- Node: `npm run build` (napi) + `npx vitest run tests/wire03.test.ts` (o `npm test`).
- TS: `npm run build` (tsc) + `npm test` (vitest; WASM pkg ya compilado en `vantadb-wasm/pkg`).
- `campaign_verify_cmd` por step (⚠️ FIND-173: `pwsh scripts/...` SIN `-NoProfile -File`).
- Regla dura `-p` / `--manifest-path` (nunca cargo desde raíz sin selección).

## Restricciones de scope (PROHIBIDOS — WIP ajeno)
`setup-embeddings.ps1` · `src/cli_handlers/server.rs` · `src/lib.rs` · `src/embedding_health.rs` · `vantadb-mcp/src/handlers/tools.rs` · `tests/embedding*` · `perf-bench.yml` · `src/server/handlers.rs` · `src/gc.rs`. WASM (`vantadb-wasm/src/**`) fuera de scope — solo se coordina la shape por TS. No commit (LEAD), no self-review (LEAD).

## Stop conditions / Pre-mortem / Risk
- **Stop conditions:** si `$or` exigiera cambiar el modelo core → dividir (HECHO: `$or` diferido, resto entregado); rabbit hole: reescribir validaciones TS → NO (la DSL `$op` se agrega como función nueva, `normalizeValue`/`FilterItem[]` intactos en semántica).
- **Pre-mortem:** F1 TS `[]` sin intención → guard exige `text_query`/`query_sparse` (Spec #3); F2 divergencia py↔js → forma canónica única + tests espejo con los mismos valores; F3 stub drift → `test_stub_drift.py` + `tsc` + `napi build` en el step Node.
- **Risk Register:**

  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟠 | Validación TS rompe clientes | rechazar solo si vacío Y sin `text_query`/`sparse`; mensajes específicos | `d5a-validation` rojo |
  | 🟡×🟡 | Divergencia semántica filtros | misma DSL + tests espejo (mismos campos/valores) | review P2-01 |
  | 🟢×🟡 | Stub drift | drift test + tsc + napi build | CI binding |

## Steps

### Step 1: Python — `query_sparse` + `sparse_vector` en put + roundtrip/text-only tests + stubs
- **Archivos:** `vantadb-python/src/convert.rs` (helper `py_dict_to_sparse_vector`), `vantadb-python/src/lib.rs` (put/record_to_memory_input/search/search_multi/explain_memory_search/parse_search_request), `vantadb-python/vantadb_py/__init__.py` (SearchRequest + AsyncClient.search/search_multi), `vantadb-python/vantadb_py/vantadb_py.pyi` + `__init__.pyi`, `vantadb-python/tests/test_wire03.py` (nuevo).
- **RED:** `test_wire03.py` con (a) sparse roundtrip put→search, (b) sparse-only (sin dense/text), (c) text-only pin, (d) filtros `$gte/$lt`+datetime en count/delete — corre contra el pyd actual → falla por param inexistente (verificado en vivo).
- **GREEN:** helper + params al final de cada firma; `put` setea `input.sparse_vector`; search paths leen `query_sparse`.
- **Verify:** `maturin develop` + `pytest test_wire03.py test_stub_drift.py` + `cargo check -p vantadb-python` + clippy.
- **Estado:** ✅ DONE (2026-09-27) — 9 tests nuevos verdes + stub drift 7/7 + w1_surface 6/6 (22 passed, `campaign_verify_cmd` ✅); suite py completa 158 passed tras extender `test_w1_surface` con `query_sparse`; clippy `vantadb_py` limpio (`--no-deps`; warnings de `vantadb` dep en featureset py = pre-existentes, `cli` off); fmt aplicado.

### Step 2: Node — `query_sparse` + `sparse_vector` + d.ts + tests
- **Archivos:** `vantadb-node/src/lib.rs` (`parse_search_request`, `parse_memory_input`, helper `parse_sparse_vector` + unit tests Rust), `vantadb-node/dts-header.d.ts` + `index.d.ts` (regenerado por `napi build`), `vantadb-node/tests/wire03.test.ts` (nuevo).
- **RED:** test node con sparse roundtrip + sparse-only + text-only + (regresión) `query_sparse` ignorado hoy → 0 hits/no-op (verificado en vivo: campo descartado en silencio).
- **GREEN:** parser `{dim:peso}` → `SparseVector(BTreeMap<u32,f32>)` (keys string→u32, rechazo claro), wire en `MemorySearchRequest`/`MemoryInput`.
- **Verify:** `npm run build` + `npx vitest run tests/wire03.test.ts` + `cargo check --manifest-path vantadb-node/Cargo.toml` + clippy + `cargo test --manifest-path vantadb-node/Cargo.toml` (unit serde).
- **Estado:** ✅ DONE (2026-09-27) — Rust unit 7/7 (3 nuevos), wire03 vitest 5/5, suite node completa 41/41 (4 files), clippy/fmt limpios; `dts-header.d.ts` + `index.d.ts` regenerado por `napi build` (sparse_vector:52,70 / query_sparse:95); build local en perfil debug (el release prebuilt pesaba y hay presión de memoria — artefactos `.node` gitignored).

### Step 3: TS — `query_sparse` (native passthrough / wasm error), text-only guard, filtros DSL+DateTime, tests
- **Archivos:** `vantadb-ts/src/types.ts` (Value+DateTime/ListDateTime, SearchRequest.query_sparse, FilterOperator/FilterCondition/FilterInput/FilterSpec, CountInput/DeleteByFilterInput), `vantadb-ts/src/metadata.ts` (DateTime en `assertTaggedValue`/`normalizeValue`, `Date`→ISO, `normalizeFilterInput`), `vantadb-ts/src/guards.ts` (VALID_VALUE_TYPES + guard text-only/sparse), `vantadb-ts/src/vantadb.ts` (wasm: error explícito sparse; count/delete/export aceptan `FilterSpec`), `vantadb-ts/src/native.ts` (passthrough `query_sparse`), `vantadb-ts/src/__tests__/wire03.test.ts` (nuevo) + ajuste `d5a-validation.test.ts` si aplica (caso `[]` sin texto sigue rojo).
- **RED:** tests: text-only WASM; `[]` sin texto → DbError; `$op` DSL count ≡ `FilterItem[]` + datetime range; sparse wasm → error; native sparse roundtrip (skip si binding no disponible).
- **GREEN:** implementación mínima aditiva.
- **Verify:** `npm run build` (tsc) + `npm test`.
- **Estado:** ✅ DONE (2026-09-27) — `tsc` ✅; wire03 8/8 (incluye roundtrip sparse por `vantadb/native` + `$op` DSL ≡ `FilterItem[]` + Date/DateTime + texto-only + guard de intención + error explícito WASM); suite TS completa 322/322 (14 files); `eslint .` 0 problemas; `native.ts` put/putBatch ahora reenvían `sparse_vector` (drop silencioso previo).

### Step 4: Core — pin del contrato canónico (test-only)
- **Archivos:** `src/sdk/serialization/mod.rs` (test `test_advanced_filter_datetime_range` Gte+Lte con `Value::DateTime`).
- **Verify:** `cargo nextest run -p vantadb --lib sdk::serialization::` + fmt + clippy.
- **Estado:** ✅ DONE (2026-09-27) — `cargo nextest run -p vantadb --lib sdk::serialization --build-jobs 2` → 216/216 passed; `cargo fmt -p vantadb -- --check` ✅.

### Step 5: Docs (Regla 3) + FIND-180 (Backlog)
- **Archivos:** `docs/api/TS_SDK.md` / `docs/api/PYTHON_SDK.md` / `docs/api/NODE_SDK.md` (secciones search/filtros — verificar cuáles existen y su anclaje) + `docs/dev/Backlog.md` (FIND-180 `$or` diferido).
- **Verify:** `pwsh scripts/validate-docs-coverage.ps1` + markdownlint de los docs tocados.
- **Estado:** ✅ DONE (2026-09-27) — PYTHON_SDK.md (put/put_batch/search_multi/search/explain + nota text-only/sparse), TS_SDK.md (put/putBatch/SearchRequest + §Filters con DSL `$op`), NODE_SDK.md (sparse + text-only); `validate-docs-coverage` → 0 gaps ✅; markdownlint 5 archivos → 0 issues ✅; FIND-180 en Backlog ($or diferido).

### Step 6: Verify full + cierre (sin commit — LEAD)
- **Verify:** fmt + clippy (py/node/core) + pytest wire03+drift + node vitest + ts vitest + tsc; task file sync + RESULTADO §7.
- **Estado:** ✅ DONE (2026-09-27) — ver §Verificación final. Commit + review fresco = LEAD (instrucción explícita del orquestador).

## Verificación final (2026-09-27 — evidencia mecánica)

| Comando | Resultado |
|---|---|
| `pytest vantadb-python/tests/` (suite completa, venv abi3-py311) — re-run post-fixes | ✅ **159 passed**, 4 deselected |
| `pytest test_wire03 + test_stub_drift + test_w1_surface` | ✅ **23 passed** (incluye el nuevo caso N1 f32-overflow) |
| `cargo clippy -p vantadb_py --all-targets --no-deps -- -D warnings` | ✅ exit 0 (warnings de dep `vantadb` con featureset py = pre-existentes, `cli` off) |
| `cargo check/clippy/fmt --manifest-path vantadb-node/Cargo.toml` + `cargo test` — re-run post-fixes | ✅ 7/7 unit (incluye overflow f64→f32 en el parser) |
| `napi build` (cjs + esm) + `npx vitest run` (vantadb-node, 4 files) — re-run | ✅ 41/41 |
| `npm run build` (tsc, src) + `eslint .` + `npx vitest run` (vantadb-ts, 14 files) — re-run | ✅ **323/323** (9 wire03), 0 lint, build 0 errores |
| `npx tsc --noEmit` del test (tsconfig temporal, ver R1) | ✅ **0 errores** |
| `cargo nextest run -p vantadb --lib sdk::serialization --build-jobs 2` — re-run | ✅ 216/216 |
| `cargo clippy -p vantadb --lib --no-deps -- -D warnings` + `cargo fmt -p vantadb -- --check` | ✅ exit 0 |
| `pwsh scripts/validate-docs-coverage.ps1` | ✅ 0 gaps |
| `npx markdownlint-cli2` (5 archivos: 3 SDK docs + Backlog + task file) | ✅ 0 issues |
| **Probes del review** | Node `[]` solo → `{count:0, error:null}` ✅ · WASM `{}` → skip (test) ✅ · R3 negativo → test falla sin el fix ✅ |

**Artefactos regenerados (napi build):** `vantadb-node/index.d.ts` (+6: `sparse_vector`/`query_sparse`; header preservado vía `dts-header.d.ts`) y `index.cjs`/`index.js` (resync de versión del loader `0.5.0`→`0.7.0`, stale pre-existente — efecto de `napi build`, no del código de la tarea).

## Dependencias
- API-02 ✅ (shape de requests canónico). Wave F2: WIRE-03 ‖ WIRE-04 (archivos disjuntos — verificado). nextTask: WIRE-05.

## Review (GATE P2-01 — LEAD)
- **Ronda 1 (2026-09-27):** ❌ CHANGES REQUIRED (R1/R2/R3 + O1/O2/O3) — ver §Review fixes. Todo lo demás verde y reproducido.
- **Ronda 2:** pendiente — la corre el LEAD sobre el delta (incluye un re-review fresco).

## Review fixes (ronda 1 — aplicados 2026-09-27)

| ID | Fix aplicado | Evidencia |
|---|---|---|
| R1 | `UntrustedInput = FlatValue \| Value \| Date` en `types.ts` usado en `MetadataInput`, `FilterItem.value` y `FilterCondition`; `Date`→`DateTime` normalizado en `normalizeValue` (WASM) y `normalizeMetadataForNative` (native); frontera WASM de `vantadb.ts` (put/putBatch/list/importRecords) con erased casts (patrón FIND-125) porque el input tipado ahora es más ancho que la `.d.ts`; `NormalizedFilterItem` (value: `Value`) para count/delete/export | `npm run build` ✅ + `npx tsc --noEmit -p <tsconfig temporal: extends ./tsconfig.json, include src/__tests__/wire03.test.ts, exclude []>` → **0 errores** (antes 5× TS2322) + probe del ejemplo de docs (`filters: {when: {$gte: new Date(...)}}`) incluido en el test ✅ + roundtrip `Date` metadata por el backend nativo pineado ✅ |
| R2 | Matriz del task file corregida: Node `[]` solo → **resultado vacío** (`get_f32_vec` nunca rechazó — pre-existente; el core hace skip), no "sigue rechazando" | probe Node: `db.search({query_vector: []})` → `{"count":0,"error":null}` |
| R3 | Test sparse nativo rehecho **sparse-only** (`query_vector: []` + `query_sparse: {7:1.5}`) — ya no es vacuo | probe negativo: forwarding deshabilitado → `1 failed | 8 passed`; restaurado → `9 passed` |
| O1 | Guard WASM de `query_sparse` solo cuando `Object.keys(...).length > 0` (espejo de `hasSparse` en `buildSearchRequestBase`); `{}` = skip | test pineado "empty query_sparse {} is skip" ✅ |
| O2 | Guard `Number.isFinite` inline en `native.ts::_buildSearchRequest` (mensaje claro, ≤5 líneas) | cubre la puerta de `search`; `put`/`putBatch` delegan al mensaje por-dimensión del parser napi (`^query_sparse[7] must be a finite number`) |
| O3 | Asimetría de claves sparse documentada (Py exige `int`; JS acepta string keys) | `PYTHON_SDK.md` §put + `NODE_SDK.md` §Sparse vectors |

## Context Save Point
- **Cierre ✅ (2026-09-27):** steps 1-6 ✅ con verify mecánico por binding. Py: 22 focused + 158 full. Node: 7 unit + 41 vitest. TS: 322 vitest + tsc + eslint. Core: 216 nextest + fmt + clippy. Docs: coverage 0 gaps + markdownlint 0. Pendiente SOLO LEAD: review fresco P2-01 + commit (no ejecutados por instrucción explícita).
- **SDP:** base (`campaign-executor`, `progreso`, `source-driven-development`) + lifecycle BUILD (`incremental-implementation`, `test-driven-development`, `context-engineering`, `doubt-driven-development`) + `api-and-interface-design` (SDP v3) + `rust-write-tests` (área). Cargadas: source-driven-development · incremental-implementation · test-driven-development · api-and-interface-design · rust-write-tests.
- **Gates:** P: no · D: disparado-sin-question-tool (símbolos públicos nuevos exigidos por contrato/Backlog: kwargs `query_sparse`/`sparse_vector` + tipos TS `FilterInput`/`FilterSpec`/DateTime; aditivos, aprobados por plan — mismo precedente FIND-103) · V: no (0 fallas; 1 reset de budget por el corte de infra, documentado) · C: no.

## Recitation
```
=== RECITATION WIRE-03 ===
Objetivo activo: WIRE-03 — query_sparse + text-only + filtros avanzados en Py/TS/Node (delta review r1 aplicado)
Estado: steps 6/6 ✅ + fixes R1-R3/O1-O3 aplicados + re-verify completo; commit/review r2 = LEAD
Última acción: fixes de la ronda 1 (tipos Date en unions de entrada + frontera WASM con casts FIND-125 + NormalizedFilterItem; test sparse-only no vacuo con probe negativo; matriz Node [] corregida; guard {} WASM; N1 overflow f32 Py/Node; O2 guard finito; O3 asimetría documentada) + re-verify (py 159/23, node 41+7, ts 323 + tsc src + tsc test 0, core 216, docs 0)
Resultado: OK
Próxima acción: LEAD — re-review fresco P2-01 del delta + commit local `feat(bindings): query_sparse + text-only + advanced filters (WIRE-03)`; sin push
Contrato: query_sparse roundtrip en los 3 (Py test / Node test / TS native sparse-only test anti-vacuo) ✅ · text-only por la puerta principal en los 3 ✅ · filtros range/datetime py↔js equivalentes + tests espejo ✅ · stubs/d.ts sync (stub_drift 7/7 + tsc src + tsc test 0 + napi build) ✅ · $or diferido FIND-180 (stop condition del plan)
Invariantes: params al final/opcionales; `[]` solo comportamiento real documentado por binding (Py/Node: vacío pre-existente; TS: error de intención; WASM: fail-loud sparse); WASM crate intacto; PROHIBIDOS intactos; sin commit
Deuda: FIND-180 ($or); loader node 0.5.0→0.7.0 resync (disclosed)
last-synced: 2026-09-27
=== END RECITATION ===
```
