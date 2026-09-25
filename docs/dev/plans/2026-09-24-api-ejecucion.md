# Plan de Ejecución: Estandarización 11 APIs — Ejecución W0–W8

> **Campaign ID:** beca0c27-fd85-4489-8f93-8361888d662c
> **Inicio:** 2026-09-24
> **Estado:** ⬜ PENDING (0/9)
> **Fuente:** `docs/dev/Backlog.md` Phase 51 (filas `API-01..API-09`)
> **Autonomous:** false
> **Modo:** PLAN (este archivo no cambia código; la ejecución es `/pipeline run` o `/pipeline task API-0X`)
> **Investigación base (Paso 0 ya hecho):** `docs/dev/plans/2026-09-24-api-estandarizacion.md` + task files `docs/dev/tasks/API-STD-01..18.md` (11 fichas individuales, web-checklist 31 ítems, síntesis 18 ejes con Gate P 4/4, re-validación 40 fallos + 2 FIND-NEW). Cada tarea abajo cita su evidencia.
> **Reglas globales:** breaking `feat!:` (nadie usa los paquetes); docs mismo-PR (Regla 3); deuda neta ≤0 (Regla 6, pagar P2-5/P2-8); benchmark antes de optimizar (Regla 9); `vantadb-pro` intocable; release-plz decide versiones/tags (Regla 7); MCP local refresh ante cambio de tools.
> **SDP:** codebase-memory, systematic-debugging, test-driven-development, progreso, ponytail (full)

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 9 |
| 🟡 DEFER | 0 |
| ❌ SKIP | 0 |
| 🔴 BLOQUEADO | 0 |

Status: ⬆️ uphill = 3 (codegen single-schema, YAML owner efectivo, `IQL_VERSION` gate) · ⬇️ downhill = 9 tasks con contrato mecánico

Orden: API-01 → (API-02, API-03 en paralelo tras 01) → API-04/05/06/07/08 (tras 02+03 según deps) → API-09 (última). MAX_CONCURRENT=3.

## Tasks

### Task 1: API-01 — W0 fundación tipos+error+casing+u128

- **Appetite:** max 1sem
- **Esfuerzo:** 🔴 3-5d
- **Prioridad:** 🔴
- **Archivos clave:** `src/sdk/types/record.rs:35-114`, `src/sdk/types/graph.rs:12-32`, `src/error.rs:184-287`, `src/binary_header.rs:20`, `vantadb-python/src/types.rs`, `vantadb-ts/src/types.ts`, `vantadb-node/index.d.ts`, `vantadb-wasm/src/lib.rs`, `src/sdk/version_history.rs:144-145`
- **Verificación real:** ✅ CÓDIGO-REAL (API-STD-02/05/06): `Write.node_id` sin `u128_serde` (`graph.rs:18-24`) vs 2 sitios con él; `FilterOp` sin docs (`record.rs:13-20`); `Generic/ResourceLimit(String)` (`error.rs:184-287`); `VantaHeader` (`binary_header.rs:20`); Node `node_id: string` modelo (`index.d.ts:63-64`); napi solo-numbers (`native.ts:315`, FIND-NEW-01).
- **Gate Justificación:** Sin tipos/error/casing comunes, W1–W8 construyen sobre drift. Primera por dependencias.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo test --test sdk_serialization` verde Y test wire `u128` >2^53 redondo en 4 bindings Y `rg Generic\( error` con tipado o doc-diseño Y `dev-tools/verify_changed.ps1` verde
- **Task file:** `docs/dev/tasks/API-01.md`
- **Estado:** ⏳ IN PROGRESS (Steps 1-2 ✅ 2026-09-25: u128 wire RED→GREEN)
- **Branch:** develop
- **Commit:** f86584f6 (+70e553f7 task file)
- **Cynefin:** 🟨 complicado — serde cross-binding + codegen requieren experto
- **Top 3 riesgos:** 1. codegen single-schema sin dueño 2. `u128→string` rompe tests que esperan number 3. Tipar errores rompe `map_vanta_error`
- **Pre-mortem:** F1: JSON-string para `u128` sin migrar napi; F2: RFC 9457 a medias (solo type/title); F3: casing global rompe Python snake interior
- **Stop conditions:** appetite >1sem → partir tipos vs errores; codegen sin consenso → manual 1 vez + DEFER codegen
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🔴 | Rompe `sdk_serialization` | fijar snapshots en el PR | test rojo |
  | 🟡×🟡 | Deuda nueva sin pago | pagar P2-8 aquí | PR con deuda |
- **Uphill/Downhill:** ⬆️ 1 (codegen) / ⬇️ tipos+error+casing+u128
- **DoD task:** contrato ✅ · task file sync · recitation
- **Iteraciones:** | — | — | — | — |
- **Notas:** Commits `feat!:` + ID. Docs tipos mismo-PR.

### Task 2: API-02 — W1 bindings score+firmas+getNode+u128

- **Appetite:** max 2sem
- **Esfuerzo:** 🔴 1-2sem
- **Prioridad:** 🔴
- **Archivos clave:** `vantadb-python/src/lib.rs:402-2376`, `vantadb-ts/src/vantadb.ts:423-1454`, `vantadb-ts/src/native.ts`, `vantadb-node/src/lib.rs`, `vantadb-node/index.d.ts`, `vantadb-wasm/src/lib.rs:1157-1896`, `tests/api/python.rs`
- **Verificación real:** ✅ CÓDIGO-REAL (API-STD-03/04/05/06): `distance: h.score` ×3 (TS `:609,663,742`); traversals `number[]` (TS `:90-107`); `importRecords` bucle (TS `:900-926`); Py flat nodo (`:1563,1577`) + columnar (`:702`) + sin `search_multi`; Node mínimo + sin sparse-escritura; Gate P: score-todos, array-objetos.
- **Gate Justificación:** Grupo más roto (15 fallos); Gate P ya decidió las 2 cuestiones 🔴.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo test --test python_sdk_boundary` verde Y `tsc --noEmit` + `npm test` verdes Y matriz 4 bindings pareja (método×firma) Y `rg "distance: h.score"` = 0
- **Task file:** `docs/dev/tasks/API-02.md`
- **Estado:** ⬜ PENDING
- **Branch:**
- **Commit:**
- **Cynefin:** 🟨 complicado — 4 toolchains (PyO3/NAPI/wasm-bindgen/tsc)
- **Top 3 riesgos:** 1. `tests/api/python.rs` fijan aliases 2. FIND-79 (wasm import) sin resolver bloquea TS 3. napi `.node` por plataforma
- **Pre-mortem:** F1: migrar Py a objetos sin benchmark (Regla 9); F2: `bigint` en JSON; F3: `native.ts` vs `vantadb.ts` divergen más
- **Stop conditions:** appetite >2sem → partir por binding (Py→TS→Node); FIND-79 bloquea → DEFER delegación TS con bucle documentado
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🔴 | Tests fijan legacy | actualizar tests en el PR | rojo |
  | 🟡×🟡 | P2-5 sin pagar | pagar aquí (Regla 6) | PR deuda |
- **Uphill/Downhill:** ⬆️ 1 (FIND-79) / ⬇️ resto bindings
- **DoD task:** contrato ✅ · task file sync · recitation
- **Iteraciones:** | — | — | — | — |
- **Notas:** Dep: API-01. Paralelizable con API-03 tras 01.

### Task 3: API-03 — W2 HTTP/OpenAPI-first REST+paginación

- **Appetite:** max 1sem
- **Esfuerzo:** 🔴 1sem
- **Prioridad:** 🔴
- **Archivos clave:** `src/server/router.rs:149-351`, `src/server/handlers.rs:252-1521`, `docs/api/openapi.yaml`, `docs/api/HTTP_API.md`, `tests/api/openapi_yaml_parity.rs`, `tests/api/structured_api_v2.rs`
- **Verificación real:** ✅ CÓDIGO-REAL (API-STD-07): verbos URL (`router:197-207,218-219,244-247`); gemelos v2 parciales (`:155,233`); `CREATED` ×6 vs YAML `200`; `POST /threads/{id}`; offset+cursor; 5 YAML drifts.
- **Gate Justificación:** Contrato externo + base MCP; YAML debe pasar a owner.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo test --test openapi_yaml_parity` verde Y curl por grupo (records/search/threads/maintenance/export) con status=YAML Y `rg "maintenance/purge|conversation/add|skill/listing"` en router = 0 Y `rg "offset" handlers.rs` = 0 (cursor único)
- **Task file:** `docs/dev/tasks/API-03.md`
- **Estado:** ⬜ PENDING
- **Branch:**
- **Commit:**
- **Cynefin:** 🟨 complicado — owner YAML + auth + migraciones
- **Top 3 riesgos:** 1. YAML e impl derivan a la vez 2. Clientes del dashboard usan rutas viejas 3. Cursor sin migrar `threads/audit`
- **Pre-mortem:** F1: gemelos v2 se duplican en vez de migrar; F2: `RecordInput` opcional rompe validación MCP; F3: gateway por entusiasmo
- **Stop conditions:** appetite >1sem → HTTP vs YAML en 2 PRs; server no levanta → solo lectura+parity
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🔴 | Rompe MCP local | smoke antes/después | tools caídas |
  | 🟢×🟡 | CORS `*` prod | auditar tower-http | grep Allow-Origin |
- **Uphill/Downhill:** ⬆️ 1 (owner YAML) / ⬇️ rutas+status+paginación
- **DoD task:** contrato ✅ · task file sync · recitation
- **Iteraciones:** | — | — | — | — |
- **Notas:** Dep: API-01. Paralelizable con API-02.

### Task 4: API-04 — W3 MCP nombres+schemas+errores+refresh local

- **Appetite:** max 1sem
- **Esfuerzo:** 🟡 3-5d
- **Prioridad:** 🟠
- **Archivos clave:** `vantadb-mcp/src/handlers/tools.rs:101-1973`, `vantadb-mcp/src/validation.rs:477-489`, `vantadb-mcp/src/error.rs:68-95`, `docs/api/MCP.md`, `vanta-mcp-local.ps1`, `opencode.jsonc:76-88`
- **Verificación real:** ✅ CÓDIGO-REAL (API-STD-08): alias doble (`:309/:373→:1691,1695`); doble listado (`:218/:703`); `thread_id:number` (`:631`, mensaje mitigado AUD-050 `:1970-1973`); bypass by-design (`:991`); `query_iql` crudo (`:1641`); string vs tipados.
- **Gate Justificación:** Cara LLM + MCP local de este entorno: sin refresh local el agente trabaja con tools viejas.
- **Gate Result:** ✅ DO
- **Contrato:** smoke `vanta-cli server --mcp` (put/get/search) verde Y `tools/list` sin duplicados Y `rg '"name": "search_memory"|"name": "collection_list"'` = 0 (o solo canónicos) Y `thread_id` string en schema
- **Task file:** `docs/dev/tasks/API-04.md`
- **Estado:** ⬜ PENDING
- **Branch:**
- **Commit:**
- **Cynefin:** 🟨 complicado — spec MCP + Schemars + compat prompts
- **Top 3 riesgos:** 1. Quitar alias rompe prompts guardados 2. Schema estricto rechaza calls actuales 3. MCP local stale
- **Pre-mortem:** F1: Schemars sin fuente single-schema; F2: `bulk_import_stream` se "normaliza" perdiendo throughput; F3: olvidar restart opencode
- **Stop conditions:** appetite >1sem → solo tools memory/graph; MCP no arranca → lectura
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🔴 | MCP local stale | launcher + restart + re-listar | tools viejas |
  | 🟡×🟡 | Bypass mal entendido | marcar by-design en doc | confusión |
- **Uphill/Downhill:** ⬆️ 0 / ⬇️ nombres+schemas+errores+refresh
- **DoD task:** contrato ✅ · task file sync · recitation
- **Iteraciones:** | — | — | — | — |
- **Notas:** Dep: API-03.

### Task 5: API-05 — W4 proxy auth/endpoints/config (SEGURIDAD)

- **Appetite:** max 1sem
- **Esfuerzo:** 🟡 2-3d
- **Prioridad:** 🔴
- **Archivos clave:** `vanta-proxy/src/server.rs:744-840`, `vanta-proxy/src/config.rs:117-307`, `vanta-proxy/config.toml`, `docs/api/PROXY.md`
- **Verificación real:** ✅ CÓDIGO-REAL (API-STD-11): `snapshot()` sin headers/auth (`:816-840` leído entero); verbo (`:745`); camel (`:747-754`); self-loop (`config:272` + test `:288-289`); ttl 0 (`:129`); rate-unused.
- **Gate Justificación:** Único hallazgo de seguridad (exposición sessions/cost sin auth). Prioridad máxima aunque esfuerzo medio.
- **Gate Result:** ✅ DO
- **Contrato:** `curl /snapshot` sin credencial → 401 Y con credencial → 200 Y `cargo test -p vanta-proxy` verde Y `rg spaceId` en server.rs = 0
- **Task file:** `docs/dev/tasks/API-05.md`
- **Estado:** ⬜ PENDING
- **Branch:**
- **Commit:**
- **Cynefin:** 🟦 obvio — auth + renames mecánicos
- **Top 3 riesgos:** 1. Se minimiza como "solo loopback" 2. Self-loop en prod 3. `ttl=0` eterno
- **Pre-mortem:** F1: auth rompe desktop que consume `/snapshot`; F2: rate doble proxy+server; F3: TCP→socket sin medir
- **Stop conditions:** desktop depende `/snapshot` abierto → auth con excepción loopback documentada, no revert
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🔴×🔴 | Exposición activa | fix YA en este PR | exploit local |
  | 🟡×🟡 | Callers LLM con paths viejos | migrar + doc | 404s |
- **Uphill/Downhill:** ⬆️ 0 / ⬇️ auth+endpoints+config
- **DoD task:** contrato ✅ · task file sync · recitation
- **Iteraciones:** | — | — | — | — |
- **Notas:** Dep: API-03. `feat!:` seguridad.

### Task 6: API-06 — W5 IQL versión+sintaxis+literales+AST

- **Appetite:** max 1sem
- **Esfuerzo:** 🟡 3-5d
- **Prioridad:** 🟠
- **Archivos clave:** `src/parser/grammar.rs:47-439`, `src/parser/lexer.rs:41-144`, `src/parser/mod.rs:170-1225`, `src/sdk/api.rs:328-331`, `docs/api/IQL.md`
- **Verificación real:** ✅ CÓDIGO-REAL (API-STD-09): SELECT existe (`:349`); orden-alt (`:434,439`); PROFILE (`:117`); 0 `IQL_VERSION`; bugs lexer/parser citados.
- **Gate Justificación:** Lenguaje sin versión bloquea evolucionar queries en W2/W3/W6.
- **Gate Result:** ✅ DO
- **Contrato:** `rg IQL_VERSION src/` ≥1 (definido + gateado) Y tests parser verdes Y repros `==` / `42→Int` / `from` minúscula / `'quote'` con comportamiento decidido Y ejemplo YAML válido
- **Task file:** `docs/dev/tasks/API-06.md`
- **Estado:** ⬜ PENDING
- **Branch:**
- **Commit:**
- **Cynefin:** 🟧 complejo — nom-combinators, probe-sense-respond
- **Top 3 riesgos:** 1. Case-insensitive rompe alias 2. Quitar `MATCH` rompe tests `:1106-1225` 3. AST sin dueño
- **Pre-mortem:** F1: versionar sin migrar `PROFILE`; F2: `SELECT` documentado sin JOIN real; F3: single-quote a medias
- **Stop conditions:** rabbit hole nom → repros + DEFER resto; appetite >1sem → versionado mínimo + fixes críticos
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🔴 | Int→float >2^53 | `parse_i64` primero | repro |
  | 🟡×🟡 | `==` colgado | orden `==` antes que `=` | test |
- **Uphill/Downhill:** ⬆️ 2 (case, versión) / ⬇️ resto
- **DoD task:** contrato ✅ · task file sync · recitation
- **Iteraciones:** | — | — | — | — |
- **Notas:** Dep: API-01. `systematic-debugging` para `==`/literales.

### Task 7: API-07 — W6 CLI POSIX+--json+flags

- **Appetite:** max 1sem
- **Esfuerzo:** 🟡 2-3d
- **Prioridad:** 🟡
- **Archivos clave:** `src/cli.rs:43-321`, `src/cli_handlers/crud.rs:53-553`, `src/cli_handlers/search.rs:21-344`, `src/cli_handlers/server.rs:21-349`
- **Verificación real:** ✅ CÓDIGO-REAL (API-STD-10): `--in` (`:123`) vs `--out`; posicional (`:134`); `limit` vs `top_k`; `--json` parcial (`:91,192,222,257,273`); cajas+truncado; exit 0; spawn; RW; bypass.
- **Gate Justificación:** API humana/agentes-terminal; depende de IQL estable (`query`).
- **Gate Result:** ✅ DO
- **Contrato:** `--help` × comando capturado Y `--json` en TODOS con salida completa (diff humano vs json) Y `count` sin DB → exit≠0 Y `rg "println!(\"{count}\")"` revisado Y lecturas sin `ensure_indexes_current`-RW o audit concurrencia (Regla 8)
- **Task file:** `docs/dev/tasks/API-07.md`
- **Estado:** ⬜ PENDING
- **Branch:**
- **Commit:**
- **Cynefin:** 🟦 obvio — normalización mecánica
- **Top 3 riesgos:** 1. Scripts parsean cajas 2. Spawn PATH Windows 3. Deadlock RW-lectura
- **Pre-mortem:** F1: `--json` default rompe scripts (opt-in primero); F2: truncado en JSON; F3: `cmd_put` sigue bypass
- **Stop conditions:** deadlock audit → `vanta-chaos`; appetite >1sem → flags+salida, concerns a DEFER
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🟡 | Scripts rotos | `--json` opt-in → default + `feat!:` | quejas |
  | 🟢×🔴 | RW lectura | lock-order audit | toca engine |
- **Uphill/Downhill:** ⬆️ 0 / ⬇️ flags+salida+concerns
- **DoD task:** contrato ✅ · task file sync · recitation
- **Iteraciones:** | — | — | — | — |
- **Notas:** Dep: API-06.

### Task 8: API-08 — W7 vanta-memory API Rust estable (NO exponer)

- **Appetite:** max 1sem
- **Esfuerzo:** 🟡 2-3d
- **Prioridad:** 🟡
- **Archivos clave:** `vanta-memory/src/lib.rs:15-18`, `vanta-memory/src/adapters/standalone/llm_runner.rs:66-218`, `vanta-memory/src/core/`, `docs/api/VANTA_MEMORY.md`
- **Verificación real:** ✅ CÓDIGO-REAL (API-STD-12): degradación por diseño (`:106-111`, test `:209-218`); core-only D42/D43; deudas D37/D21/MEM-16.
- **Gate Justificación:** Gate P: NO exponer (scope) + estabilizar Rust. Sin binding nuevo.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo test -p vanta-memory` verde Y degradado sin `llm-driver` verificado (test `:209-218` pasa) Y D37/D21/MEM-16 con benchmark o DEFER fundado Y 0 símbolos nuevos en bindings
- **Task file:** `docs/dev/tasks/API-08.md`
- **Estado:** ⬜ PENDING
- **Branch:**
- **Commit:**
- **Cynefin:** 🟧 complejo — pipeline LLM emergente
- **Top 3 riesgos:** 1. Exponer "de paso" (scope) 2. `chars/3` sin medir 3. Dedup sin embeddings
- **Pre-mortem:** F1: binding nuevo por entusiasmo; F2: `NotConfigured` como fatal; F3: wiki/skills mezclados
- **Stop conditions:** propuesta exponer → BLOQUEADO (Gate P decidió core-only); appetite >1sem → solo estabilizar
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🟡 | Scope explosion | DEFER D42 con dueño | propuesta |
  | 🟢×🟡 | Sin benchmark | Regla 9 o DEFER | números sin fuente |
- **Uphill/Downhill:** ⬆️ 1 (deudas) / ⬇️ estabilización
- **DoD task:** contrato ✅ · task file sync · recitation
- **Iteraciones:** | — | — | — | — |
- **Notas:** Dep: API-01.

### Task 9: API-09 — W8 cierre VERSIONING+docs+MCP+gates

- **Appetite:** max 3d
- **Esfuerzo:** 🟢 2d
- **Prioridad:** 🟡
- **Archivos clave:** `docs/api/VERSIONING.md`, `docs/api/` (19 files), `scripts/validate-docs-coverage.ps1`, `CONSTRAINTS.md`, `dev-tools/verify.ps1`, `dev-tools/ocr-review.ps1`, `opencode.jsonc`, `vanta-mcp-local.ps1`
- **Verificación real:** ✅ CÓDIGO-REAL (API-STD-17): coverage existe (no cubre memory); 27 workflows, 0 tokens publish (solo `GITHUB_TOKEN`+opcionales); 7 derivas con dueño; OIDC vigente.
- **Gate Justificación:** Cierra la campaña: contrato 11 superficies + docs + gates. Última por dependencias.
- **Gate Result:** ✅ DO
- **Contrato:** `VERSIONING.md` lista 11 superficies Y `validate-docs-coverage.ps1` verde Y `dev-tools/verify.ps1` verde Y MCP re-smoke verde Y `ocr-review.ps1` sin Critical/High Y plan 18/18 + este 9/9 para `/ship`
- **Task file:** `docs/dev/tasks/API-09.md`
- **Estado:** ⬜ PENDING
- **Branch:**
- **Commit:**
- **Cynefin:** 🟦 obvio — checklist de cierre
- **Top 3 riesgos:** 1. Docs detrás del código 2. MCP stale 3. Versión/tag manual
- **Pre-mortem:** F1: CHANGELOG manual; F2: ES en docs técnicas; F3: planes sin archivar
- **Stop conditions:** coverage rojo masivo → por docs; scope >1 release → partir
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🟡 | Drift día 1 | mismo-PR Regla 3 + parity CI | PR sin docs |
  | 🟢×🔴 | Tag manual | solo release-plz | edit version |
- **Uphill/Downhill:** ⬆️ 0 / ⬇️ cierre total
- **DoD task:** contrato ✅ · task file sync · recitation
- **Iteraciones:** | — | — | — | — |
- **Notas:** Dep: API-01..08. Al cerrar: `skill progreso` + archivar planes + `/audit quick` → `/ship`.

```
Plan creado en docs/dev/plans/2026-09-24-api-ejecucion.md
Próximo paso recomendado:
  /pipeline run docs/dev/plans/2026-09-24-api-ejecucion.md  → ejecutar (MAX_CONCURRENT=1, inline sin subagentes)
  /pipeline task API-01                                      → primera tarea (W0 fundación)
```

=== RECITATION API-01 ===
Campaign ID: beca0c27-fd85-4489-8f93-8361888d662c
Objetivo activo: API-01: W0 fundación — tipos base + error envelope + casing + u128 wire
Estado: in-progress
Última acción: Steps 3,4,6,7,8 verificados (docs FilterOp; envelope code+message+context + Generic by-design; casing norm + punteros en 4 tipos; P2-8 pre-pagada AUD-043; wire u128>2^53 verde core 17/17 + Py 2/2 + Node 28/28 + TS 1/1 + WASM 30/30). verify_changed 4/4. Step 5: evidencia + recomendación B (rename Header + alias) en ADR-041 §Evidencia; firma owner pendiente.
Resultado: PARTIAL
Próxima acción: Owner firma ADR-041 (A/B/C) en docs/dev/architecture/adr/041_anti_stutter.md (§Firmado por); luego vanta-lead commitea los archivos API-01 (feat! API-01). No re-ejecutar Steps 3-8.
Contrato: verificacion: cargo test --test sdk_serialization -> 17 passed/0 failed | dev-tools/verify_changed.ps1 -> ALL 4 PASS | wasm-pack test --node -> 30 passed/0 failed | pytest tests/test_wire_u128.py -> 2 passed | vitest tests/wire-u128.test.ts -> 1 passed | vitest api.test.ts -> 28 passed | cargo doc -> exit 0 | tsc --noEmit -> exit 0 | cargo check -p vantadb_py -> exit 0
evidencia:
  - claim: u128 >2^53 cruza los 4 bindings sin pérdida (string decimal o bigint exacto; nunca f64). evidencia: tests por binding + pkg wasm fresco verificado manual (typeof string, value 9007199254740993). confianza: alta
  - claim: Generic( en src/error.rs con doc-diseño by-design (ResourceLimit 12/Schema 14/InvalidInput 64 callers). evidencia: src/error.rs + docs/api/ERROR_HANDLING.md §'The Generic catch-all'. confianza: alta
  - claim: P2-8 ya pagada (HashSet<u128>, orden de primera aparición); task file citaba líneas stale 564-596, real 752-784. evidencia: commit 9dcbff5a + test verde en wasm-pack test. confianza: alta
  - claim: VantaHeader sin huella on-disk; exclusión ADR con razón contradicha. evidencia: src/binary_header.rs:19,49-57; 64 refs/8 archivos. confianza: alta
  - claim: casing Gate P declarado sin renames (migración = W1/API-02). evidencia: BINDINGS_NAMESPACES.md §Casing Contract + tsc/cargo check verdes. confianza: alta
artefactos: src/sdk/types/record.rs, src/error.rs, docs/api/ERROR_HANDLING.md, docs/api/BINDINGS_NAMESPACES.md, docs/dev/architecture/adr/041_anti_stutter.md, vantadb-ts/src/types.ts, vantadb-wasm/src/vantadb_wasm.d.ts, vantadb-node/index.d.ts, vantadb-python/src/types.rs, vantadb-wasm/src/lib.rs, vantadb-python/tests/test_wire_u128.py, vantadb-ts/tests/wire-u128.test.ts, vantadb-node/tests/api.test.ts, docs/dev/tasks/API-01.md
invariantes: u128_serde ÚNICO patrón wire u128; validate_compat intacto; vantadb-pro intocable; sin panic en bindings; code()/Display sin cambios
deuda: P2-8 pagada (AUD-043); P2-5 diferida a API-02; FIND candidatos: types.ts node_id string vs pkg bigint; wasm d.ts IqlResult{kind} drift; anti_stutter_map.json:94 razón incorrecta
queda_pendiente: firma owner ADR-041 (BLOQUEO Step 5); commit (solo vanta-lead); WIP ajeno WIRE-09 en working tree NO commitear con API-01
Próxima tarea si completa: API-02
=== END RECITATION ===
