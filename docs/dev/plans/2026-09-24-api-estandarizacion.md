# Plan de Ejecución: Estandarización 11 APIs VantaDB — Investigación + Síntesis + Validación

> **Inicio:** 2026-09-24
> **Estado:** ✅ COMPLETADO 18/18 (2026-09-24, inline sin subagentes)
> **Fuente:** Conversación 2026-09-24 (auditoría 11 superficies, sesiones ses_f2b6280b*) + 4 bloques web aportados por el usuario (casing, firmas, REST, estándares avanzados, cross-language) — ad-hoc informativo → normativo, no parte de `docs/dev/Backlog.md`
> **Autonomous:** false
> **Modo:** PLAN (read-only, sin cambios de código — solo investiga, sintetiza y valida)
> **Contexto libre:** Nadie usa el proyecto ni los paquetes — breaking changes ilimitados (`feat!:`) permitidos, dejando todo funcional para el siguiente release. MCP local (`vanta-cli server --mcp`, `opencode.jsonc`) debe refrescarse ante cualquier cambio.
> **SDP:** coordinated-web-search, api-design-principles, codebase-memory, systematic-debugging, writing-plans, progreso (fase PLAN)

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 18 |
| 🟡 DEFER | 0 |
| ❌ SKIP | 0 |
| 🔴 BLOQUEADO | 0 |

Status: ⬆️ uphill = 6 (orquestador/IPC, AST bindings, IQL_VERSION, OpenAPI-first owner, MCP local refresh, codegen single-schema) · ⬇️ downhill = 18 tasks con contrato mecánico definido

Fases: F0 inventario (01) → F1 11 investigaciones individuales funcionamiento+uso+código (02–12) → F2 conjunto+orquestador (13) → F3 web-cascade checklist bloques usuario (14) → F4 síntesis decisiones (15) → F5 re-validación fallo-por-fallo (16) → F6 docs/referencias/actualizaciones (17) → F7 plan implementación con comandos (18).

Las 11 superficies: (1) Rust core SDK, (2) Python, (3) TS, (4) Node nativo, (5) WASM, (6) HTTP+OpenAPI, (7) MCP, (8) IQL, (9) CLI, (10) vanta-proxy, (11) vanta-memory.

## Tasks

### Task 1: API-STD-01 — Inventario funcional 11 superficies + mapa conjunto vs individual

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🔴
- **Archivos clave:** `src/lib.rs:9-53`, `Cargo.toml:744-765`, `docs/api/` (19 files), `docs/api/VERSIONING.md:30-50`, `docs/api/BINDINGS_NAMESPACES.md:19-28`, `vantadb-python/`, `vantadb-ts/`, `vantadb-node/`, `vantadb-wasm/`, `vantadb-server/`, `vantadb-mcp/`, `vanta-memory/`, `vanta-proxy/`, `src/parser/`, `src/bin/vanta-cli.rs`
- **Verificación real:** ✅ CÓDIGO-REAL — `src/lib.rs:19` Embedded; workspace 7 members `Cargo.toml:744-752`; `docs/api/` 19 ficheros por glob; contrato 4 superficies `VERSIONING.md:35-38`; conteos 47/43/44 `BINDINGS_NAMESPACES.md:79,134,188`.
- **Gate Justificación:** Fija las 11 y el mapa conjunto (core single-owner) vs individual antes de investigar.
- **Gate Result:** ✅ DO
- **Contrato:** `ls docs/api/*.md = 19` Y `cargo metadata` con ≥4 bindings Y tabla 11 filas (superficie|entry-point|doc|funcionamiento 1 línea|conjunto-vs-individual)
- **Task file:** `docs/dev/tasks/API-STD-01.md`
- **Estado:** ✅ DONE
- **Branch:**
- **Commit:**
- **Cynefin:** 🟦 obvio
- **Top 3 riesgos:** 1. `.agents/` eliminado contamina paths 2. Feature≠superficie (GraphRAG/embeddings) 3. Docs a la deriva tomadas como verdad
- **Pre-mortem:** F1: contar `providers/` como API; F2: omitir Node; F3: inventario solo-docs sin entry-points
- **Stop conditions:** appetite >1d → DEFER; superficie inexistente → re-triaje
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🟢 | Superficie fantasma | glob+read por cada una | 1 iter sin green |
  | 🟢×🟡 | `.opencode/` repo separado | solo workspace VantaDB | diff toca `.opencode/` |
- **Uphill/Downhill:** ⬆️ 1 (lista 11) / ⬇️ tabla + mapa
- **DoD task:** contrato mecánico ✅ · task file sync · recitation
- **Iteraciones:** | — | — | — | — |
- **Notas:** Hallazgos nuevos → FIND-* vía `prompts/findings.md`.

### Task 2: API-STD-02 — INDIVIDUAL (1/11) Rust core SDK: funcionamiento, uso y código

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🔴
- **Archivos clave:** `src/lib.rs:9-196`, `src/sdk/api/memory.rs:29-44`, `src/sdk/types/record.rs:13-114`, `src/sdk/types/graph.rs:18-29`, `src/error.rs:184-287`, `src/binary_header.rs:20`, `src/engine/`, `docs/api/EMBEDDED_SDK.md`, `docs/dev/architecture/adr/041_anti_stutter.md:43-64`, `.opencode/rules/core-engine.md`, `.opencode/rules/api-contract.md`
- **Verificación real:** ✅ CÓDIGO-REAL — `Embedded` en `src/lib.rs:19`; stutter `VantaHeader` en `binary_header.rs:20` + re-export `lib.rs:167`; `Generic(ChainedError)` `error.rs:275-276`, `ResourceLimit(String)` `:184`, `InvalidInput(String)` `:283`; `QueryResult::Write.node_id: Option<u128>` sin serde `graph.rs:18-24` vs `StaleContext.node_id` con serde `:29`; aliases eliminados 0.6.0 + flat `get_memory/list_memory/delete_memory` removidos AST-012 (`BINDINGS_NAMESPACES.md:37-39,240-244`); `SnapshotRecord→MemoryRecord` pierde `superseded_by/at_ms` (`version_history.rs:144-145` None).
- **Gate Justificación:** Single-owner del estado — todo drift nace aquí y se hereda a 4 bindings + HTTP + MCP.
- **Gate Result:** ✅ DO
- **Contrato:** task file con (a) funcionamiento: qué hace el SDK (open/CRUD/search/graph/IQL/index/export), (b) uso: ejemplo mínimo `Embedded::open_with_config` + `put/get/search` compilable, (c) código: módulos + tipos + errores tipados vs catch-all, (d) veredicto por cada fallo Rust con `file:line` (✅ confirma | refuta con evidencia)
- **Task file:** `docs/dev/tasks/API-STD-02.md`
- **Estado:** ✅ DONE
- **Branch:**
- **Commit:**
- **Cynefin:** 🟨 complicado — decidir tipado errores y serde `u128` requiere experto
- **Top 3 riesgos:** 1. `u128` JSON pierde >2^53 en wire 2. `Generic` catch-all normaliza vaguedad 3. Breaking `feat!:` sin marcar (Regla 7)
- **Pre-mortem:** F1: documentar `MemoryInput` sin tocar `FilterOp` sin docs; F2: proponer `#[non_exhaustive]` donde rompe match; F3: medir zero-copy sin benchmark (Regla 9)
- **Stop conditions:** appetite >1d → solo matriz tipos+errores; rabbit hole HNSW → fuera de scope
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🔴 | Wire >2^53 | test decimal-string | repro falla |
  | 🟡×🟡 | Deuda P2 nueva sin pago (Regla 6) | pagar P2-5/P2-8 | PR con deuda |
- **Uphill/Downhill:** ⬆️ 1 (serde `u128` + tipado error) / ⬇️ ficha funcionamiento+uso+código
- **DoD task:** contrato ✅ · task file sync · recitation
- **Iteraciones:** | — | — | — | — |
- **Notas:** `codegraph_explore "Embedded put get search"` + blast radius; Regla 4 dual-API.

### Task 3: API-STD-03 — INDIVIDUAL (2/11) Python SDK: funcionamiento, uso y código

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🔴
- **Archivos clave:** `vantadb-python/src/lib.rs:356-372,538-562,648-712,811,1017-1255,1379-1498,1563-1681,1757-2123,2350`, `vantadb-python/src/types.rs:329`, `vantadb-python/src/vector.rs:59-74`, `docs/api/PYTHON_SDK.md`, `tests/api/python.rs`, `.opencode/rules/python-bindings.md`
- **Verificación real:** ✅ CÓDIGO-REAL — `get(id: u128)` node-level `:1563` + `delete(id, reason)` `:1577` (≠ TS namespace+key); `insert(id, content, vector…)` posicional `:648-649` + aliases `:538-540`; `put_batch` columnar `:701-712`; `search` híbrido `:1241-1255` + `parse_search_method` `:2350`; sin `search_multi` (ausencia en `:356-372,543-562`); sin `export_namespace_filtered/import_records/audit_deep` (`:1379` sin filtro, `:1498` shallow); solo-Python `put_batch_raw/search_batch/hardware_profile/recover_archived/query_structured/graph_page_rank` (`:811,1625,1681,1822,2123,1757,2061`); P2-5 dual-API `put_batch` (`AGENTS.md` Regla 6).
- **Gate Justificación:** Binding con más métodos únicos y semántica `get/delete` opuesta — fija su ficha antes de unificar.
- **Gate Result:** ✅ DO
- **Contrato:** ejemplo `import vantadb; db.put/search` real ejecutado (o documentado por qué no) Y tabla método-a-método (firma real + dominio memory/graph/wiki/system) Y cada fallo Python con `file:line` + repro
- **Task file:** `docs/dev/tasks/API-STD-03.md`
- **Estado:** ✅ DONE
- **Branch:**
- **Commit:**
- **Cynefin:** 🟨 complicado — PyO3 + kwargs vs posicional requiere decisión experta
- **Top 3 riesgos:** 1. `*` kwargs obliga nombrados y rompe posicional actual 2. `panic!` cruza a Python (debe mapear a excepción) 3. Tests `python.rs` fijan aliases legacy
- **Pre-mortem:** F1: unificar a objeto y romper columnar performante; F2: `u128` como int; F3: `bulk_import` dual sin dueño
- **Stop conditions:** appetite >1d → matriz+repro mínimo; FFI panic → auditar `expect/unwrap`
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🔴 | Rompe `tests/api/python.rs` | mapear tests antes | 2 iter sin green |
  | 🟢×🔴 | PyO3 panic | `map_vanta_error` + jerarquía MOD-20 | clippy deny |
- **Uphill/Downhill:** ⬆️ 1 (firma canónica) / ⬇️ ficha completa
- **DoD task:** contrato ✅ · task file sync · recitation
- **Iteraciones:** | — | — | — | — |
- **Notas:** Probar `maturin develop` o documentar entorno; `forward_to_db!` macro como fuente.

### Task 4: API-STD-04 — INDIVIDUAL (3/11) TypeScript SDK: funcionamiento, uso y código

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🔴
- **Archivos clave:** `vantadb-ts/src/vantadb.ts:423-1454` (put:423, putBatch:451, get:482, delete:502, search:601, distance:605-611, supersede:714, exportNamespace:822-838, importRecords:880-926, insertNode:1135-1140, traversals:1327-1454), `vantadb-ts/src/types.ts:48-293` (distance:143-147), `vantadb-ts/src/native.ts:94-104`, `vantadb-ts/src/errors.ts:99-149`, `vantadb-ts/README.md`, `docs/api/TS_SDK.md`
- **Verificación real:** ✅ CÓDIGO-REAL — `distance: h.score` sin invertir `:605-611` vs doc lower-is-better `types.ts:143-147`; traversals `number[]` `:1327-1331` vs insert `number|bigint` `:1135-1139` (mueren >2^53 en BFS); `importRecords` bucle `get+put` `:900-926` evitando wasm `:880-886` (pierde created_at/version/history, `skipped` siempre 0); sin `bulk_import` (grep cero); subclientes `memory/graph/system` + `degree` (`:309`); `native.ts` subset sync (`capabilities/close/delete/flush/get/list/put/search`).
- **Gate Justificación:** Único binding que renombra `score→distance` y trunca IDs — su ficha decide el estándar frontera.
- **Gate Result:** ✅ DO
- **Contrato:** `npm test` (o `tsc --noEmit`) estado documentado Y ejemplo `put/search/graphBfs` real Y cada fallo TS con `file:line` + valor antes/después (ej. score 0.9 → distance 0.9 bug visible)
- **Task file:** `docs/dev/tasks/API-STD-04.md`
- **Estado:** ✅ DONE
- **Branch:**
- **Commit:**
- **Cynefin:** 🟨 complicado — `string-u64` vs number + codegen fronteras
- **Top 3 riesgos:** 1. Cambiar `distance` a `score` rompe consumidores (cero externos, pero tests sí) 2. `bigint` en JSON 3. `native.ts` vs `vantadb.ts` divergen
- **Pre-mortem:** F1: invertir valor sin cambiar nombre (doble bug); F2: `importRecords` se conecta a wasm sin migrar conteos; F3: Zod schemas sin fuente single-schema
- **Stop conditions:** appetite >1d → matriz+repros; `node_modules` roto → documentar y seguir
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🔴 | >2^53 en BFS | string-u64 + test | repro falla |
  | 🟡×🟡 | Doble backend wasm/native | tabla por backend | divergencia |
- **Uphill/Downhill:** ⬆️ 1 (score/distance + u128) / ⬇️ ficha completa
- **DoD task:** contrato ✅ · task file sync · recitation
- **Iteraciones:** | — | — | — | — |
- **Notas:** Citar `FIND-10/79/125` donde apliquen; `serde_wasm_bindgen 0.6` serializa `u128` como Bung (ver `types.ts:292-293`).

### Task 5: API-STD-05 — INDIVIDUAL (4/11) Node nativo: funcionamiento, uso y código

- **Appetite:** max 1d
- **Esfuerzo:** 🟢 1h-1d
- **Prioridad:** 🟠
- **Archivos clave:** `vantadb-node/src/lib.rs:104-120,136-149,403-613`, `vantadb-node/index.d.ts:44-447` (MemoryInput:44-52, Record:67, SearchRequest:87-95, ListOptions:74-78, VantaDb:243-447, searchWithMethod:440, versions:389-447), `docs/api/NODE_SDK.md`, `vantadb-node/Cargo.toml` (NAPI-RS)
- **Verificación real:** ✅ CÓDIGO-REAL — superficie mínima `index.d.ts:243-447` sin export/import/bulk/audit/search_vector/hardware/recover (vs WASM `:1383-1564`); `put(MemoryInput)` objeto `:104-107` pero `method` separado `searchWithMethod` `:593`; `SearchRequest` sin `exclude_superseded` `:87-95`, `MemoryInput` sin `sparse_vector` `:44-52` (lectura sin escritura); `graphDegree` sobre `graph_degree_centrality` `:403-407`; Node-only `versions/vacuum/search_with_method` (`:389,401,440`).
- **Gate Justificación:** Decide si Node converge a TS/WASM o queda como binding mínimo NAPI (más rápido que WASM en servidor).
- **Gate Result:** ✅ DO
- **Contrato:** ejemplo Node real (o build `napi` documentado) Y tabla Node×(WASM/TS) método-a-método Y cada gap con `file:line`
- **Task file:** `docs/dev/tasks/API-STD-05.md`
- **Estado:** ✅ DONE
- **Branch:**
- **Commit:**
- **Cynefin:** 🟦 obvio — gap mecánico, falta rellenar o declarar mínimo
- **Top 3 riesgos:** 1. NAPI vs WASM duplican mantenimiento 2. `js_name` camelCase sin configurar 3. `.node` binario sin publish local
- **Pre-mortem:** F1: ampliar Node sin dueño y deriva de nuevo; F2: `searchWithMethod` se unifica mal; F3: error NAPI sin code
- **Stop conditions:** build NAPI >appetite → solo matriz `.d.ts`; premisa invalidada → DEFER a mínimo declarado
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🟡 | Doble stack NAPI+WASM | decidir 1 server-side | deriva |
  | 🟢×🟡 | `sparse_vector` solo lectura | añadir escritura o quitar lectura | inconsist |
- **Uphill/Downhill:** ⬆️ 1 (NAPI vs WASM server) / ⬇️ ficha completa
- **DoD task:** contrato ✅ · task file sync · recitation
- **Iteraciones:** | — | — | — | — |
- **Notas:** Validar `js_name` NAPI para camelCase automático (bloque usuario).

### Task 6: API-STD-06 — INDIVIDUAL (5/11) WASM: funcionamiento, uso y código

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟠
- **Archivos clave:** `vantadb-wasm/src/lib.rs:152-189,585-1035,1157,1202-1262,1383-1627,1769,1896`, `vantadb-wasm/Cargo.toml` (wasm-bindgen, `opt-level="s"`), `docs/api/WASM_API.md`, `docs/api/WASM_PERSISTENCE.md`, `docs/api/WASM_STANDALONE.md`, `benchmarks/wasm_bench.mjs`
- **Verificación real:** ✅ CÓDIGO-REAL — 47 fns (`BINDINGS_NAMESPACES.md:79-132`); OPFS `connect_persistent/connect_idb/save/load/auto-save` (`:585,618,944,973,1006-1035`); `score` emitido `:1262`; `put_batch` array-objetos `:1157`; `search_multi` `:1482`; `bulk_import(_bytes)` `:1555,1564`; `graph_degree` `:1896`; u128 como strings decimales `:1769`; P2-8 `collect_all_deduped()` O(n) (`:564-596`, verificado AGENTS.md Regla 6).
- **Gate Justificación:** Base de `vantadb-ts` en browser — su frontera `serde_wasm_bindgen` fija el wire JSON.
- **Gate Result:** ✅ DO
- **Contrato:** `wasm-pack build` o `pkg/` existente documentado + ejemplo browser/Node-wasm Y tabla 47 fns por dominio Y fallos WASM con `file:line` (P2-8, `to_js_err` FIND-10)
- **Task file:** `docs/dev/tasks/API-STD-06.md`
- **Estado:** ✅ DONE
- **Branch:**
- **Commit:**
- **Cynefin:** 🟨 complicado — OPFS/persistencia + `u128` wire requieren experto
- **Top 3 riesgos:** 1. `collect_all_deduped` O(n) en memoria limitada browser 2. `u128` como BigInt vs string 3. `opt-level="s"` vs debug
- **Pre-mortem:** F1: zero-copy donde WASM no lo soporta; F2: OPFS API cambia y `pkg/` stale; F3: bench `.mjs` sin baseline
- **Stop conditions:** toolchain wasm ausente → solo lectura código+`pkg/`; appetite >1d → matriz
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🟡 | P2-8 O(n) | pagar con P2-5 (Regla 6) | PR deuda |
  | 🟢×🟡 | `to_js_err` aplana Display | estructurado con code | `errors.ts:99` |
- **Uphill/Downhill:** ⬆️ 1 (wire `u128` + OPFS) / ⬇️ ficha completa
- **DoD task:** contrato ✅ · task file sync · recitation
- **Iteraciones:** | — | — | — | — |
- **Notas:** `errors.ts:99-149` como consumidor del error WASM.

### Task 7: API-STD-07 — INDIVIDUAL (6/11) HTTP server + OpenAPI: funcionamiento, uso y código

- **Appetite:** max 3d
- **Esfuerzo:** 🔴 2-3d
- **Prioridad:** 🔴
- **Archivos clave:** `src/server/router.rs:149-351`, `src/server/handlers.rs:252-1322`, `docs/api/openapi.yaml:36-1935`, `docs/api/HTTP_API.md`, `tests/api/openapi_yaml_parity.rs:90-144`, `tests/api/structured_api_v2.rs`, `.opencode/rules/server-mcp.md`
- **Verificación real:** ✅ CÓDIGO-REAL — verbos `POST /maintenance/purge|compact|flush|rebuild-index` (`:197-207,246-249` + YAML `:1043-1126`), `POST /export|/import`, `POST /conversation/add` + `GET /skill/listing` (`:218-219`); fuera `/api/v2`: `/health,/metrics,/dashboard` (`:149,232-233` vs `VERSIONING.md:37`); `201` impl (`:252,263,1212,1321`) vs YAML `200` (`:275,347,1203,1344`); `POST /threads/{id}` mensaje (`:212-217` + `:1234-1238` → debe ser `/messages`); paginación `limit+offset` (`:1129-1138`) vs `limit+cursor` (`:375-382,472-485`), `versions` solo `?version=` (`:281-286`) vs YAML `?limit` (`:434-438`), `threads` sin cursor (`:1162-1165`), `audit` solo limit (`:590-593`); `RecordInput` required 6 (`:1721-1738`) vs `Option` core (`record.rs:35-54`); ejemplo Lisp inválido (`:1727`) vs UPPERCASE (`parity.rs:90-144`); enum case-drift (`:1815-1818` vs MCP lowercase); `next_cursor string|null` (`:1933-1955`) vs `Option<usize>` (`:430,493`); tag `Skills` sin declarar (`:36-50` vs `:1406+`).
- **Gate Justificación:** Contrato externo + base del MCP local — decide OpenAPI-first y el esquema único paginación/error.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo test --test openapi_yaml_parity` resultado anotado Y tabla ruta-a-ruta (método+path router | status handler | status YAML | paginación | auth) con curl real por grupo (records/search/threads/maintenance/export) Y cada fallo con `file:line`
- **Task file:** `docs/dev/tasks/API-STD-07.md`
- **Estado:** ✅ DONE
- **Branch:**
- **Commit:**
- **Cynefin:** 🟨 complicado — fuente de verdad + auth + rate-limit
- **Top 3 riesgos:** 1. Nadie ownea el YAML 2. `/snapshot`-style sin auth se repite 3. Cursor vs offset a medias
- **Pre-mortem:** F1: gateway para monolito (over-eng); F2: cursor sin migrar offset; F3: `spaceId`-style camel en paths
- **Stop conditions:** server no levanta → solo lectura + parity test; appetite >3d → partir HTTP vs YAML
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🔴 | Auth rota MCP local | smoke antes/después | MCP no responde |
  | 🟡×🟡 | `tower_governor` vs proxy unused | un punto | divergen |
  | 🟢×🟡 | CORS `*` prod | auditar `tower-http` | grep Allow-Origin |
- **Uphill/Downhill:** ⬆️ 2 (owner YAML, paginación) / ⬇️ ficha + tabla rutas
- **DoD task:** contrato ✅ · task file sync · recitation
- **Iteraciones:** | — | — | — | — |
- **Notas:** Regla 3: desde ahora YAML y handler cambian en el mismo PR.

### Task 8: API-STD-08 — INDIVIDUAL (7/11) MCP: funcionamiento, uso y código

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟠
- **Archivos clave:** `vantadb-mcp/src/handlers/tools.rs:101-1939` (`memory_put:101`, `list_namespaces:218`, `memory_search:309`, `search_memory:373`, `inject_context:619-633`, `bulk_import_stream:990-991`, `query_iql:1641`), `vantadb-mcp/src/validation.rs:477-489`, `vantadb-mcp/src/error.rs:68-95`, `vantadb-mcp/src/handlers/{scenes,dreams,skills}.rs`, `docs/api/MCP.md:412`, `vanta-mcp-local.ps1`, `opencode.jsonc`
- **Verificación real:** ✅ CÓDIGO-REAL — alias doble `memory_search` `:309` vs `search_memory` `:373`; `memory_list_namespaces` `:218` vs `collection_list` `:703`, `collection_*` vs `memory_*` vs `skill_list` singular (`skills.rs:50`); `query_iql` string crudo `:1641` sin `invalid_params`; `thread_id: number` `:619-633` vs resto u128-string (`:490,1939`); `filter.labels: number[]` `:553` vs HTTP labels string; `bulk_import_stream` bypasa validación `:990-991`; errores string `error_content` `:477-479` (usado `scenes:20,157,176`, `dreams:18,152,162`) vs tipado `McpError::from_domain` (`error.rs:68-95`); `skill_extract` degrada `{success:false}` (`MCP.md:412`).
- **Gate Justificación:** Cara LLM del proyecto + MCP local de opencode — un nombre canónico + validación en borde + errores tipados.
- **Gate Result:** ✅ DO
- **Contrato:** `vanta-cli server --mcp` smoke (lista tools + 1 `memory_put/get/search`) documentado Y tabla tool×args (JSON Schema) con duplicados marcados Y cada fallo con `file:line`
- **Task file:** `docs/dev/tasks/API-STD-08.md`
- **Estado:** ✅ DONE
- **Branch:**
- **Commit:**
- **Cynefin:** 🟨 complicado — spec MCP (prompts/resources/tools) + schemas estrictos
- **Top 3 riesgos:** 1. Quitar alias rompe prompts del agente (cero externos, pero MCP local sí) 2. Validación estricta rechaza calls actuales 3. `thread_id` number vs string migra llamadas
- **Pre-mortem:** F1: Schemars sin fuente single-schema; F2: resources vs tools confundidos; F3: MCP local stale tras cambio
- **Stop conditions:** MCP no arranca → solo lectura; appetite >1d → solo tools memory/graph
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🔴 | MCP local stale | `vanta-mcp-local.ps1` + restart | tools viejas |
  | 🟡×🟡 | Bypass validación se normaliza | marcar by-design | `:990-991` |
- **Uphill/Downhill:** ⬆️ 1 (nombres canónicos) / ⬇️ ficha + smoke
- **DoD task:** contrato ✅ · task file sync · recitation
- **Iteraciones:** | — | — | — | — |
- **Notas:** Separar prompts/resources/tools per spec Anthropic; Pydantic/Zod/Schemars según superficie.

### Task 9: API-STD-09 — INDIVIDUAL (8/11) IQL: funcionamiento, uso y código

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟠
- **Archivos clave:** `src/parser/grammar.rs:47-56,92-121,349-439`, `src/parser/lexer.rs:41-144`, `src/parser/mod.rs:170-225,683-1225`, `src/sdk/api.rs:328-331`, `docs/api/IQL.md:16-76`, `.opencode/rules/query-dsl.md`
- **Verificación real:** ✅ CÓDIGO-REAL — docs 6 statements (`:16-26`) vs parser 7 con `SELECT…JOIN` (`:349,439` + tests `:1106-1225`); `FROM/MATCH` intercambiables (`IQL:46`, `:92`) + `SELECT` con otro `FROM` y default clonado (`:362-363` vs `:97-98`); colisión `INSERT` por orden `alt` (`:432-434` + test `:983-988`); bug `==` (`:47-56` + test `:170-176`); int→`Float(42.0)` (`lexer:138-144` + `:215-225`); keywords solo UPPERCASE (`lexer:41-73`, `non_keyword_ident` `:77-79`); solo `"` (`:85-122`), single-quote parse error (`api.rs:328-331`, `IQL:76` no lo prohíbe); sin `IQL_VERSION` (0 hits); `PROFILE rrf_k/candidate_k` (`:116-121`) sin gate; `PROFILE bogus` sin consumir (`:683-691`).
- **Gate Justificación:** Lenguaje query sin versión — decide 1 sintaxis + case + `IQL_VERSION` + AST JSON para bindings.
- **Gate Result:** ✅ DO
- **Contrato:** `rg IQL_VERSION src/` (=0) anotado Y tabla statement×(docs|parser|test) Y repro `==`, `42→Float`, `from` minúscula, `'quote'` Y cada fallo con `file:line`
- **Task file:** `docs/dev/tasks/API-STD-09.md`
- **Estado:** ✅ DONE
- **Branch:**
- **Commit:**
- **Cynefin:** 🟧 complejo — nom-combinators solo emergen probando; probe-sense-respond
- **Top 3 riesgos:** 1. Case-insensitive rompe alias minúsculas 2. Eliminar `MATCH` rompe tests 3. AST JSON sin dueño
- **Pre-mortem:** F1: versionar sin migrar `PROFILE`; F2: `SELECT` se documenta sin implementar JOIN real; F3: single-quote se permite a medias
- **Stop conditions:** rabbit hole nom → abortar con repros; appetite >1d → IQL a DEFER versionado
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🔴 | Int→float >2^53 | `parse_i64` antes que double | repro |
  | 🟡×🟡 | `==` deja `=` colgado | `!=/>=/<=/==` antes que `=` | test `:170-176` |
- **Uphill/Downhill:** ⬆️ 2 (case + versión) / ⬇️ ficha + repros
- **DoD task:** contrato ✅ · task file sync · recitation
- **Iteraciones:** | — | — | — | — |
- **Notas:** `systematic-debugging` para `==` y literales; exponer AST como JSON plano (bloque usuario §5).

### Task 10: API-STD-10 — INDIVIDUAL (9/11) CLI: funcionamiento, uso y código

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡
- **Archivos clave:** `src/cli.rs:43-321`, `src/cli_handlers/crud.rs:53-553`, `src/cli_handlers/search.rs:21-344`, `src/cli_handlers/server.rs:21-349` (spawn `vantadb-server --mcp` `:253-349`, tokio `:191-208`), `src/bin/vanta-cli.rs`
- **Verificación real:** ✅ CÓDIGO-REAL — `Import --in` (`:123-124`) vs `Export --out` (`:109-110`); `Query.query` posicional (`:133-134`); `limit` (`:218-219`) vs `top_k` (`:269-270,304-305,320-321`); `Count --json` número crudo (`:256-258` + `:551-553`); sin `--json` en put/get/list (`:43-79`); cajas `╭│╰` (`:178-235`, `:293-347`, `server.rs:21-65`); truncado `payload[..80]` (`:140-142,254,340-344`) y JSON recortado (`:70-80,192-201`); `count` sin DB → `0` exit 0 (`:522-524`, `:21-23`); lecturas abren RW (`:33-36` AUD-044); `cmd_put` bypass SDK (`:53-55,130-131`) vs `cmd_delete` vía `Embedded` (`:369`); `cmd_server` spawnea binario externo (`:253-349`).
- **Gate Justificación:** API para humanos y para agentes terminal — decide POSIX/GNU + Clap + `--json` siempre + salida completa JSON.
- **Gate Result:** ✅ DO
- **Contrato:** `--help` por comando capturado Y tabla flag×comando (nombre+posicional-vs-flag+salida json/humano) Y cada fallo con `file:line` + salida real pegada
- **Task file:** `docs/dev/tasks/API-STD-10.md`
- **Estado:** ✅ DONE
- **Branch:**
- **Commit:**
- **Cynefin:** 🟦 obvio — normalizar flags/salida es mecánico
- **Top 3 riesgos:** 1. `--json` global rompe scripts de cajas 2. `count` exit 0 oculta "sin DB" 3. Spawn PATH frágil en Windows
- **Pre-mortem:** F1: `--in/--out` se unifican sin migrar scripts; F2: truncado humano se lleva a JSON; F3: RW-lecturas con deadlock (Regla 8)
- **Stop conditions:** appetite >1d → solo flags+salida; deadlock audit → `vanta-chaos`
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🟡 | Scripts parsean cajas | `--json` opt-in primero, luego default | breaking `feat!:` |
  | 🟢×🔴 | RW en lectura | lock order audit | toca engine |
- **Uphill/Downhill:** ⬆️ 0 / ⬇️ ficha + tabla flags
- **DoD task:** contrato ✅ · task file sync · recitation
- **Iteraciones:** | — | — | — | — |
- **Notas:** Clap es estándar (bloque usuario §3); Typer/Click y Commander/Clack solo si hay CLI Python/TS (no hay).

### Task 11: API-STD-11 — INDIVIDUAL (10/11) vanta-proxy: funcionamiento, uso y código

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡
- **Archivos clave:** `vanta-proxy/src/server.rs:112-117,297-300,741-840`, `vanta-proxy/src/config.rs:12-272`, `vanta-proxy/config.toml`, `docs/api/PROXY.md:25-30`
- **Verificación real:** ✅ CÓDIGO-REAL — 10 registros, 8 endpoints (`:741-772`, `PROXY.md:27-30`); `GET /snapshot` singular (`:744`) vs `GET /api/v2/snapshots` plural, sin `authenticate` (`:816-840` vs `:297-300` que exige auth — expone turns/sessions/writeback/rate_limit/cost); `POST /session/advance` verbo (`:745,778-811`); `/{agent}/{spaceId}` camelCase (`:746-754`), `/v1/responses` sin forma prefijada; default self-loop `127.0.0.1:8096` = propio puerto (`config:272` vs `:12`, salvado por `points_at_self` `:245-251` + `:112-117`); `cache.ttl_secs=0` = nunca expira (`:117`, convención invertida); `rate_limit_per_minute parsed but unused` (`:192-193` vs governor real `router.rs:258-277`).
- **Gate Justificación:** Proxy transparente LLM — decide endpoints plurales, auth total, config sin self-loop, rate-limit en un punto.
- **Gate Result:** ✅ DO
- **Contrato:** ejemplo forward + 1 opt-in real Y tabla endpoint×(auth|config|convención) Y cada fallo con `file:line` + request real
- **Task file:** `docs/dev/tasks/API-STD-11.md`
- **Estado:** ✅ DONE
- **Branch:**
- **Commit:**
- **Cynefin:** 🟨 complicado — auth + forwarding byte-identical vs opt-ins
- **Top 3 riesgos:** 1. `/snapshot` abierto se minimiza 2. Self-loop default en prod 3. `ttl_secs=0` ambiguo eternamente
- **Pre-mortem:** F1: pluralizar sin migrar LLM callers; F2: rate-limit doble (proxy+server); F3: TCP localhost donde Unix socket basta (bloque usuario §4)
- **Stop conditions:** appetite >1d → solo auth+endpoints; proxy no levanta → lectura
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🔴×🔴 | `/snapshot` sin auth expone estado | auth ya (breaking `feat!:`) | exploit local |
  | 🟡×🟡 | Self-loop | default vacío + error claro | loop |
- **Uphill/Downhill:** ⬆️ 1 (auth) / ⬇️ ficha + tabla
- **DoD task:** contrato ✅ · task file sync · recitation
- **Iteraciones:** | — | — | — | — |
- **Notas:** D31 TOML+serde (`config.rs:1`); medir Unix socket vs TCP solo con benchmark (Regla 9).

### Task 12: API-STD-12 — INDIVIDUAL (11/11) vanta-memory: funcionamiento, uso y código

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡
- **Archivos clave:** `vanta-memory/src/lib.rs:15-18`, `vanta-memory/src/adapters/standalone/llm_runner.rs:104-112`, `vanta-memory/Cargo.toml:39-40`, `vanta-memory/src/core/...` (l0_recorder, l1_extractor/dedup, scene_index, persona_generator), `docs/api/VANTA_MEMORY.md:20-104`, `docs/api/BINDINGS_NAMESPACES.md:25-27,250-259`
- **Verificación real:** ✅ CÓDIGO-REAL — `conversation`/`skills` reservados, L0–L3 core-only (`:25-27`, diferido "requiere nuevo binding" `:250-259`); principio LLM opcional P4 (`VANTA_MEMORY.md:20-21`, `lib.rs:15-18`); sin `llm-driver` → `NotConfigured` (`llm_runner:104-112`, `Cargo:39-40`); deudas recall/dedup keyword-overlap hasta embeddings D37 (`:69,101-102`), `TokenEstimator chars/3` D21 (`:102`), `context engine ↔ worker` pendiente MEM-16 (`:104`); F1-F3 viven en `EMBEDDED_SDK.md` (`:14-16`).
- **Gate Justificación:** Decide si L0–L3 se exponen (nuevo binding, D42) o quedan core-only con API Rust estable.
- **Gate Result:** ✅ DO
- **Contrato:** ejemplo L0→L1→recall sin LLM (degradado) real Y tabla capa×módulo× bindings-hoy (ninguno) Y cada deuda con `file:line`
- **Task file:** `docs/dev/tasks/API-STD-12.md`
- **Estado:** ✅ DONE
- **Branch:**
- **Commit:**
- **Cynefin:** 🟧 complejo — pipeline LLM con degradación solo emerge probando
- **Top 3 riesgos:** 1. Exponer en bindings explota scope (D42/D43) 2. `chars/3` sin benchmark 3. Dedup keyword-overlap se fija sin embeddings
- **Pre-mortem:** F1: prometer `conversation` en bindings sin binding Rust; F2: wiki/skills se mezclan en `conversation`; F3: `NotConfigured` se trata como error fatal
- **Stop conditions:** scope binding nuevo → DEFER explícito D42; appetite >1d → solo ficha
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🟡 | Scope explosion bindings | DEFER D42 con dueño | propuesta exponer |
  | 🟢×🟡 | Estimador sin medir | benchmark (Regla 9) | números sin fuente |
- **Uphill/Downhill:** ⬆️ 1 (exponer o no) / ⬇️ ficha + degradado
- **DoD task:** contrato ✅ · task file sync · recitation
- **Iteraciones:** | — | — | — | — |
- **Notas:** No confundir con `vanta-pro` (open-core, repo separado).

### Task 13: API-STD-13 — CONJUNTO: arquitectura + orquestador + IPC (responder con código)

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🔴
- **Archivos clave:** `src/engine/` + `src/sdk/` (single-owner), `vantadb-python` (PyO3 cdylib `*.abi3.so`, `Cargo.toml` workspace-excluido providers por MSVC), `vantadb-node` (NAPI-RS `.node`), `vantadb-wasm/pkg` (`serde_wasm_bindgen 0.6`), `vantadb-ts/src/native.ts` vs `vantadb.ts` (doble backend wasm/native), `src/cli_handlers/server.rs:176-349` (spawn `vantadb-server --mcp`), `src/server/` + `vanta-proxy/` (HTTP TCP localhost), `vantadb-mcp/` (stdio), búsquedas `UnixStream|uds|WebSocket|shm` (= ausentes salvo `.gitignore:42` y `cdylib` MSVC nota)
- **Verificación real:** ✅ CÓDIGO-REAL (verificado 2026-09-24) — NO hay orquestador único: cada runtime carga el core in-process (Python `*.abi3.so` PyO3; Node `.node` NAPI; browser/Node-wasm `pkg/` wasm-bindgen; TS además `native.ts` alternativo). CLI llama SDK directo salvo modo `--mcp`, que hace `Command::new("vantadb-server").arg("--mcp").spawn()` con fallback PATH (`server.rs:262-324`). Entre procesos: HTTP TCP localhost (server/proxy) + MCP stdio; SIN Unix sockets / SHM / WebSockets (grep vacío). Estado vive en Rust (ficheros+WAL+VFile); bindings son vistas sin lifecycle propio.
- **Gate Justificación:** Responde tus 2 preguntas con verdad de código y fija single-ownership + fronteras IPC antes de estandarizar.
- **Gate Result:** ✅ DO
- **Contrato:** task file con diagrama (core→4 bindings in-process | CLI→SDK directo + spawn mcp | HTTP TCP | MCP stdio) Y respuestas citadas (`server.rs:262-324`, `native.ts` vs `vantadb.ts`, grep IPC vacío) Y decisión (single-owner Rust; Unix-socket solo si benchmark Regla 9 lo justifica)
- **Task file:** `docs/dev/tasks/API-STD-13.md`
- **Estado:** ✅ DONE
- **Branch:**
- **Commit:**
- **Cynefin:** 🟨 complicado — topología real vs propuesta
- **Top 3 riesgos:** 1. Inventar gateway para 1 proceso 2. Unix socket sin medir 3. Doble backend TS sin dueño
- **Pre-mortem:** F1: gRPC donde `.so` basta; F2: spawn PATH frágil se declara "IPC"; F3: ownership Python/Node manual
- **Stop conditions:** benchmark IPC >appetite → decisión sin números queda DEFER
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🟡 | Over-eng IPC | medir antes (Regla 9) | propuesta socket |
  | 🟢×🟡 | Spawn frágil Win | test PATH | MCP no arranca |
- **Uphill/Downhill:** ⬆️ 1 (IPC) / ⬇️ diagrama + decisiones
- **DoD task:** contrato ✅ · task file sync · recitation
- **Iteraciones:** | — | — | — | — |
- **Notas:** `codegraph_explore "cmd_server_mcp spawn vantadb-server"` + `detect_changes` inbound.

### Task 14: API-STD-14 — Web-checklist: tus 4 bloques ítem-por-ítem (confirma/matiza/descarta)

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟠
- **Archivos clave:** `.opencode/skills/coordinated-web-search/SKILL.md` (cascada keyless→Argus→MetaSearch→fallbacks), `.opencode/skills/api-design-principles/` (checklist, rest-best-practices, graphql-schema)
- **Verificación real:** 🟡 VERIFICAR — cada ítem con ≥2 capas y URL oficial; nunca inventar URLs (AGENTS.md Regla 14).
- **Gate Justificación:** Convierte tus bloques en decisiones trazables en vez de resumen.
- **Gate Result:** ✅ DO
- **Contrato:** tabla 30 filas (ítem | veredicto confirma/matiza/descarta | URLs ≥2 | aplica VantaDB sí/no/por qué). Ítems obligatorios: casing fronteras camelCase + kebab CLI/MCP/tools + `js_name` PyO3/NAPI; Options-object/Builder/kwargs `*`; codegen single-schema (JSON Schema/Protobuf); glosario get(fetch pesado)/create/delete/update-patch; `Result→excepción` sin panic con code+message+context; URLs plurales + kebab-vs-snake; JSON `application/json` + llaves; error envelope; OAuth2/OIDC JWT RS256 + Bearer vs `x-api-key`; verbos HTTP; versionado URL; códigos 200/201/400/401/403/404/429/500; `?page&limit&sort=-price` + cursor Stripe/GitHub; OpenAPI viva API-first; RFC 7807 `application/problem+json`; ISO 8601 UTC `Z`; CORS estricto + TLS 1.3; rate-limit 100/min + `X-RateLimit-*/Retry-After`; ETag/`304`/`Cache-Control`/`Idempotency-Key`; gateway (casi seguro NO aplica); correlation-id; logs JSON; zero-copy Arrow/Protobuf; PyO3 vs NAPI vs WASM; FFI `unsafe` encapsulado; MCP prompts/resources/tools + Pydantic/Zod/Schemars; CLI POSIX + Clap(+Typer/Commander si aplica) + `--json`; single-ownership; Unix-socket vs TCP; IQL AST JSON.
- **Task file:** `docs/dev/tasks/API-STD-14.md`
- **Estado:** ✅ DONE
- **Branch:**
- **Commit:**
- **Cynefin:** 🟨 complicado
- **Top 3 riesgos:** 1. Blog≠RFC 2. Gateway/GraphQL creep 3. Wedge MCP (restart, seguir keyless)
- **Pre-mortem:** F1: 1 motor como verdad; F2: casing por gusto; F3: paywall sin fallback Jina/Playwright
- **Stop conditions:** red caída → webfetch+Playwright; sin fuente → DEFER ítem, nunca inventar
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🟡 | Wedge | restart + keyless | timeout total |
  | 🟢×🟡 | No-oficial | exigir spec/RFC/GitHub | duda |
- **Uphill/Downhill:** ⬆️ 2 (casing, codegen) / ⬇️ tabla 30 filas
- **DoD task:** contrato ✅ · task file sync · recitation
- **Iteraciones:** | — | — | — | — |
- **Notas:** Delegar pesado a `vanta-research` (digest ≤500 + RESULTADO). Queries mínimas en plan anterior §API-STD-05.

### Task 15: API-STD-15 — Síntesis normativa + Gate P HITL (qué estandarizar/cambiar/mejorar)

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🔴
- **Archivos clave:** outputs 02–14, `docs/api/VERSIONING.md`, `CONSTRAINTS.md`, `.opencode/rules/api-contract.md|server-mcp.md|js-ecosystem.md|query-dsl.md|python-bindings.md`
- **Verificación real:** 🟡 VERIFICAR — cruzar fallos×estándares; 🔴/ambiguos vía `question` tool 1 ronda (Gate P `prompts/question-gates.md`) antes de fijar.
- **Gate Justificación:** Entrega normativa de tus puntos 1–2.
- **Gate Result:** ✅ DO
- **Contrato:** tabla EJE→DECISIÓN→ALCANCE (11 marcadas)→FUENTE (URL o `file:line`)→BREAKING(sí/`feat!:`) con cero decisiones sin fuente Y log `question` Gate P para cada 🔴 (nombres/semántica, firmas, errores, REST, OpenAPI, CLI/IQL + extras: casing, Options, codegen, verbos, auth, versionado, RFC7807, ISO8601, rate-limit, ETag/idempotencia, observabilidad, MCP schemas, IPC, AST)
- **Task file:** `docs/dev/tasks/API-STD-15.md`
- **Estado:** ✅ DONE
- **Branch:**
- **Commit:**
- **Cynefin:** 🟧 complejo — trade-offs compat-vs-limpieza
- **Top 3 riesgos:** 1. camelCase global rompe Python idiomático (frontera≠interior) 2. Sinónimos sin migración 3. OpenAPI-first sin owner
- **Pre-mortem:** F1: copiar bloques sin filtrar gateway/GraphQL; F2: Protobuf donde JSON basta (ponytail); F3: breaking sin `feat!:` (Regla 7)
- **Stop conditions:** sin consenso tras 1 ronda → DEFER con dueño; appetite >1d → partir por ejes
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🔴 | Breaking sin `feat!:` | marcar cada una | review |
  | 🟢×🟡 | ADR sin humano (Regla 5) | IA solo evidencia | tradeoff |
- **Uphill/Downhill:** ⬆️ 3 (casing, firmas, errores) / ⬇️ tabla normativa
- **DoD task:** contrato ✅ · task file sync · Gate P log · recitation
- **Iteraciones:** | — | — | — | — |
- **Notas:** Normativa en inglés (Doc Language Split), resumen ES ok; cada decisión con consecuencias+deuda (Reglas 5/6). **Gate P calendarizado aquí:** al abrir el task, 1 ronda `question` con los 🔴 (casing JSON, firma `put_batch`, `score/distance`, paginación, `IQL_VERSION`, exponer `vanta-memory` sí/no).

### Task 16: API-STD-16 — Re-validación fallo-por-fallo (~40 `file:line`, veredicto ✅/FIND-*)

- **Appetite:** max 3d
- **Esfuerzo:** 🔴 2-3d
- **Prioridad:** 🟠
- **Archivos clave:** todos 02–12 + `tests/api/` + `tests/sdk_serialization.rs` + `tests/query_result_*.rs` + `scripts/validate-docs-coverage.ps1` + `dev-tools/verify_changed.ps1` + `prompts/subagent-recovery.md` (SARL)
- **Verificación real:** ✅ CÓDIGO-REAL — contexto fresco (`vanta-review`/`vanta-audit`, nunca el investigador, P2-01). Checklist: bindings 15 (get/delete `lib.rs:1563,1577` vs wasm `:1202,1212` vs ts `:482,502`; score wasm `:1262` vs py `types:329` vs ts `:605-611`+`types:143-147`; put_batch py `:701-712` vs wasm `:1157` vs ts `:451`; TS number[] `:1327-1331` vs bigint `:1135-1139`; importRecords `:900-926`; Node `index.d.ts:243-447`; supersede doc `:215` vs wasm `:1445`+ts `:714`; +search_multi/bulk/graph_degree/wiki); red 14 (verbos `router:197-207,218-219`; sin-versión `:149,232-233`; 201 `:252,263,1212,1321` vs YAML; threads `:1234-1238`; paginación `:1129-1138` vs `:375-382` vs `:281-286`; RecordInput `:1721-1738`; Lisp `:1727`; enum `:1815-1818`; cursor `:1933-1955`; tag Skills `:36-50` vs `:1406+`); MCP 6 (`:309` vs `:373`, `:218` vs `:703`, `:1641`, `:619-633`, `:553`, `:990-991`, `validation:477-479` vs `error:68-95`); core 3 (`binary_header:20`, `error:275-276,184`, `graph:18-24`); IQL 7 (`IQL:16-26` vs `:349,439`; 3 lecturas; `INSERT` `:432-434`; `==` `:47-56`; int→float `:138-144`; UPPERCASE `:41-73`; sin VERSION); CLI 8 (`--in/out`, posicional, limit/top_k, count crudo, sin `--json`, cajas, truncado, exit 0, spawn, RW, bypass); proxy 6 (`/snapshot:744,816-840`, verbo `:745`, camel `:746-754`, self-loop `:272`, ttl `:117`, rate unused `:192-193`); memory 3 (`:250-259`, `llm_runner:104-112`, `:69,101-104`). Cada uno: ✅ persiste tal cual | ✅ ya no existe (con evidencia) | FIND-* nuevo.
- **Gate Justificación:** Tu punto 3 — validar lo conseguido y garantizar que no queda nada más.
- **Gate Result:** ✅ DO
- **Contrato:** `verify_changed.ps1` verde + `validate-docs-coverage.ps1` anotado + OCR sin Critical/High + tabla ~40 filas fallo→veredicto→evidencia (tu lista pegada queda así como contrato re-validable, no como cita)
- **Task file:** `docs/dev/tasks/API-STD-16.md`
- **Estado:** ✅ DONE
- **Branch:**
- **Commit:**
- **Cynefin:** 🟨 complicado — rigor, no invención
- **Top 3 riesgos:** 1. Reviewer no-fresco 2. Verde≠ausencia 3. Crash minimizado
- **Pre-mortem:** F1: solo lo listado, sin clases nuevas (auth/u128/paginación); F2: `count` exit 0 "ok"; F3: INCOMPLETE sin SARL
- **Stop conditions:** crash real → estabilizar (Regla 8); appetite >3d → waves bindings|red|core
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🔴 | Deadlock RW CLI | audit concurrencia | dashmap/Tokio |
  | 🟡×🟡 | `continue-on-error` (Regla 2) | Issue `flaky` | intermitente |
  | 🟢×🔴 | UB FFI | `// SAFETY:` + Miri | `unsafe` |
- **Uphill/Downhill:** ⬆️ 1 (clases nuevas) / ⬇️ tabla ~40 veredictos
- **DoD task:** contrato ✅ · task file sync · OCR · recitation
- **Iteraciones:** | — | — | — | — |
- **Notas:** Fuzz básico parser/CLI + `unwrap` fronteras + `unsafe` sin SAFETY (Regla 4).

### Task 17: API-STD-17 — Docs / referencias / actualizaciones (que no derive nunca más)

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡
- **Archivos clave:** `docs/api/` (19), `docs/api/VERSIONING.md`, `docs/api/BINDINGS_NAMESPACES.md`, `scripts/validate-docs-coverage.ps1`, `docs/CHANGELOG.md` (git-cliff, no manual), `cliff.toml`, `deny.toml`, `.github/` (dependabot), `docs/dev/operations/CI_POLICY.md`, `.opencode/rules/release-ci.md`
- **Verificación real:** 🟡 VERIFICAR — auditar deriva doc↔código (ej. `supersede` Python-only falso, `search` Python pure-ANN falso, tabla ausencias obsoleta `BINDINGS_NAMESPACES:70-73`; IQL 6-vs-7; YAML vs router) + cobertura `validate-docs-coverage.ps1` + política updates (dependabot, release-plz, Trusted Publishing OIDC sin secrets).
- **Gate Justificación:** Tu eje docs/referencias/actualizaciones — Regla 3 (mismo-PR) + nuevo contrato 11 superficies en VERSIONING.
- **Gate Result:** ✅ DO
- **Contrato:** `validate-docs-coverage.ps1` resultado + tabla deriva doc→código con dueño + propuesta owner por doc (`docs/api/*.md` ↔ código) + checklist updates (dependabot on/off, release-plz `feat!:`→minor/major, OIDC verificado `gh secret list` vacío)
- **Task file:** `docs/dev/tasks/API-STD-17.md`
- **Estado:** ✅ DONE
- **Branch:**
- **Commit:**
- **Cynefin:** 🟦 obvio — mecánico con owners
- **Top 3 riesgos:** 1. `CHANGELOG.md` manual (prohibido Regla 7) 2. Versión manual en Cargo.toml 3. Tags manuales
- **Pre-mortem:** F1: docs ES técnicas (van en inglés; ES solo Backlog/avance/research); F2: reportes a `reviews/`+`INDEX.md` mal ubicados; F3: planes temporales sin archivar
- **Stop conditions:** coverage rojo masivo → partir por `docs/api/`; appetite >1d → solo deriva crítica
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🟡 | Drift reaparece día 1 | mismo-PR Regla 3 + parity CI | PR sin docs |
  | 🟢×🟡 | Doc Language Split | EN técnico, ES planning | doc ES técnica |
- **Uphill/Downhill:** ⬆️ 1 (owners) / ⬇️ tabla deriva + checklist
- **DoD task:** contrato ✅ · task file sync · recitation
- **Iteraciones:** | — | — | — | — |
- **Notas:** vanta-memory sin coverage script (`VANTA_MEMORY.md:14`) → referencia manual.

### Task 18: API-STD-18 — Implementación: waves, comandos exactos, MCP local, release siguiente

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡
- **Archivos clave:** outputs 15/16/17, `docs/dev/operations/CI_POLICY.md`, `.opencode/rules/release-ci.md`, `.opencode/rules/open-core-licensing.md` (jamás `vantadb-pro`), `CONTRIBUTING.md`, `vanta-mcp-local.ps1`, `opencode.jsonc`, `CONSTRAINTS.md` + `dev-tools/floor-guard.ps1`, `dev-tools/verify.ps1`, `dev-tools/verify_changed.ps1`, `dev-tools/ocr-review.ps1`
- **Verificación real:** 🟡 VERIFICAR — sin código; cada wave con comando copiable + rollback. Orden (a confirmar en 15): W0 fundación (tipos base+error RFC7807+codegen+casing) → W1 bindings (`get/score/put_batch/u128`, `feat!:`) → W2 OpenAPI-first+REST+paginación → W3 MCP nombres/validación/errores → W4 proxy auth/endpoints/config → W5 `IQL_VERSION`+case+literales → W6 CLI `--json`+flags → W7 `vanta-memory` (solo si 15 dice exponer; si no, API Rust estable) → W8 VERSIONING 11 + docs + MCP local + gates.
- **Gate Justificación:** Tu "cómo se va a implementar todo" con breaking libre pero release funcional.
- **Gate Result:** ✅ DO
- **Contrato:** por wave: scope + `feat!:` commits + verify exacto (`dev-tools/verify_changed.ps1`; full `dev-tools/verify.ps1`; `cargo test --test openapi_yaml_parity`; `cargo test --test python_sdk_boundary`; MCP smoke `vanta-cli server --mcp`) + doc-sync mismo-PR (Regla 3) + rollback (revert commit + re-verify) + Deuda Regla 6 (pagar P2-5/P2-8) + benchmarks donde aplique (Regla 9 `canonical_p99`, entorno+fecha). MCP local: `pwsh vanta-mcp-local.ps1` + restart opencode + re-listar tools + smoke `memory_put/get/search`; si cambia `opencode.jsonc`, diff pegado. Release: `develop`→PR→`main`, release-plz bump+CHANGELOG+tags (nunca manual, Regla 7), `/audit quick` (`just verify`), luego `/ship` GO/NO-GO y `/rollback` si falla.
- **Task file:** `docs/dev/tasks/API-STD-18.md`
- **Estado:** ✅ DONE
- **Branch:**
- **Commit:**
- **Cynefin:** 🟦 obvio una vez decididas — secuenciar por dependencias
- **Top 3 riesgos:** 1. W1+W2 en paralelo con contrato compartido 2. MCP stale 3. Commit mal tipado publica breaking como patch
- **Pre-mortem:** F1: waves sin `feat!:`; F2: docs detrás; F3: gateway/Protobuf por entusiasmo (ponytail)
- **Stop conditions:** 🔴 sin dueño → BLOQUEADO con `question`; scope >1 release → partir
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🔴 | Deuda neta >0 | pagar P2 | PR deuda |
  | 🟡×🟡 | MCP stale | local script + restart | tools viejas |
  | 🟢×🔴 | Versión/tag manual | solo release-plz | edit version |
- **Uphill/Downhill:** ⬆️ 0 / ⬇️ waves + comandos + rollback
- **DoD task:** contrato ✅ · task file sync · recitation
- **Iteraciones:** | — | — | — | — |
- **Notas:** NO ejecutar aquí (PLAN read-only). Ejecución: `/pipeline task API-STD-0X` o `/pipeline run` (FAIL_MODE=parallel, MAX_CONCURRENT=3) tras aprobar. Cierre sesión: `skill progreso` + `ponytail-review` + `just verify` + OCR.
