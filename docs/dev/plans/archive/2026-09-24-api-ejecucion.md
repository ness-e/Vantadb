---
title: "Plan de Ejecución: Estandarización 11 APIs — Ejecución W0–W8"
kind: plan
status: archived
description: "Status: ⬆️ uphill = 3 (codegen single-schema, YAML owner efectivo, IQLVERSION gate) · ⬇️ downhill = 9 tasks con contrato mecánico"
---

# Plan de Ejecución: Estandarización 11 APIs — Ejecución W0–W8

> **Campaign ID:** beca0c27-fd85-4489-8f93-8361888d662c
> **Inicio:** 2026-09-24
> **Estado:** ✅ COMPLETADO 9/9 (2026-09-26) — API-09 cerrada (contrato 6/6, commit 032cbd0f); campaña cerrada (avance + planes archivados); siguiente: `/audit quick` → `/ship`
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
- **Estado:** ✅ COMPLETED (2026-09-25) — review P2-01 ✅ tras R1/R2/R3; contrato 4/4; commit (local, sin push)
- **Branch:** develop
- **Commit:** f86584f6 + 04bfad3d + 23ef7f63 + (fix R1/R2 + docs cierre — local)
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
- **Estado:** ✅ COMPLETED (2026-09-25) — contrato 4/4 ✅; review P2-01 ✅ APPROVE (2 rondas: ❌ → R1/R2/R3 → ✅); commit local `caf063ff` (sin push)
- **Branch:** develop
- **Commit:** caf063ff
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
- **Notas:** Dep: API-01. Paralelizable con API-03 tras 01. **Cierre 2026-09-25:** contrato 4/4 re-ejecutado (python_sdk_boundary 1 passed · tsc 0 · TS vitest 314/314 · Node 36/36 · pytest 149 passed · `rg "distance: h.score"` 0 en código · matriz W1 en BINDINGS_NAMESPACES) + review P2-01 ✅. Deuda: P2-5 pagada; FIND-79 DEFER; Node extras fuera de W1. Working tree sin commit (lead commitea).

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
- **Estado:** ✅ COMPLETED (2026-09-25) — contrato 4/4 (parity 10/10 + script OK + greps 0 + smoke curl 15/15) + review P2-01 ✅ APPROVE ronda 2; commit local `94009297` (sin push)
- **Branch:** develop
- **Commit:** 94009297
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
- **Estado:** ✅ COMPLETED (2026-09-26) — review P2-01 ✅ APPROVE (ronda 2; ronda 1 F1–F5 corregidos); contrato 4/4 (smoke 11/11 + tools/list 85 sin duplicados + rg = 0 + thread_id string); commit local `bbcd9360` (sin push)
- **Branch:** develop
- **Commit:** bbcd9360
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
- **Notas:** Dep: API-03. **Cierre técnico 2026-09-26:** nombres canónicos (`memory_search`/`memory_list_namespaces`; legacy `search_memory`/`collection_list` unlisted pero dispatchables), schemas estrictos base (`additionalProperties:false`, Schemars codegen diferido con justificación), `invalid_params` temprano + errores tipados (`not_found`/`resource_limit`/`validation` envelopes), prompt `recall_search` (+redirect) y registros prompts/resources/tools separados; `thread_id` u128-string (legacy u64). Docs mismo-PR (MCP.md + skill ×2 espejos SAME + integraciones + opencode.jsonc 85). Review P2-01 ✅ (ronda 2). Detalle: `docs/dev/tasks/API-04.md`. **Pendiente owner:** rebuild `target/debug` + restart OpenCode (refresh local MCP).

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
- **Estado:** ✅ COMPLETED (2026-09-26) — contrato 4/4 (401/200/401 smoke live + suite proxy 291/0 ×3 + `spaceId`=0); review P2-01 ❌ R1 (comentario stale) → fix aplicado → ✅; commit local `f0c3f95f` (sin push)
- **Branch:** develop
- **Commit:** f0c3f95f
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
- **Notas:** Dep: API-03. `feat!:` seguridad. **Cierre técnico 2026-09-25:** auth D34 en `/snapshot` (sin bypass loopback; decisión Spec #2), rutas canónicas (`/sessions/advance`, `space_id`, `/{agent}/{space_id}/v1/responses`), upstream default vacío + fail-fast, `ttl_secs=0` = cache off, rate-limit docs truth; PROXY.md + EXPERIMENTAL_FEATURES sync; FIND-155 desktop; smoke 401/200 con binario real. Detalle: `docs/dev/tasks/API-05.md`.

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
- **Estado:** ✅ COMPLETED (2026-09-25) — contrato 4/4 (`--lib parser` 127 + `--test parser` 4 + parity 11 + `rg IQL_VERSION` 14 hits); review P2-01 ronda 1 ❌ → fixes R1/R2/R3 → ronda 2 ✅ APPROVE; Steps 0-8 ✅
- **Branch:** develop
- **Commit:** a6f6a70b (local, sin push)
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
- **Notas:** Dep: API-01. `systematic-debugging` para `==`/literales. **Cierre 2026-09-25:** `IQL_VERSION=1` + gate `iql_supports()` (PROFILE), `==` longest-match, `42`→`Int` exacto i64 (era Float), case UPPERCASE-only y single-quote rechazado documentados/pinneados, `SELECT` default alias `target` (defaults idénticos), AST JSON serde + shape documentado; FIND-156 (AST en bindings) + FIND-157 (coerción Int/Float pre-existente). Working tree sin commit (lead commitea); push solo con instrucción del owner.

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
- **Estado:** ✅ COMPLETED (2026-09-26) — contrato 5/5 + smoke 56/56 + fmt/clippy/tests verdes + lib 2124/0; review P2-01 ronda 1 ❌ → fixes → ronda 2 ✅ APPROVE; lead verify 88/88 (`-p vantadb`) + gate `#[cfg(not(feature = "server"))]` del test server (builds unificados sin hang)
- **Branch:** develop
- **Commit:** f6c395ef (local, sin push)
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
- **Estado:** ✅ COMPLETED (2026-09-26) — contrato 4/4 + 28 targets 0 failed + degradación verificada + 0 símbolos en bindings; review P2-01 ronda 2 ✅ APPROVE
- **Branch:** develop
- **Commit:** ade86a1c (local, sin push)
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
- **Estado:** ✅ COMPLETED (2026-09-26) — contrato 6/6: VERSIONING 11 superficies · `validate-docs-coverage.ps1` 0 gaps · `dev-tools/verify.ps1` ALL 11 PASS (incluye fix tooling: `cargo llvm-cov nextest run`→`nextest`, coverage real 81.63% ≥60) · MCP re-smoke 11/11 + probe 2/2 (score/distance) · OCR sin Critical/High (diff docs-only, 0 reviewable) · plan 18/18 + 9/9. Review: doubt-driven degradado + OCR; revisión independiente formal → LEAD (`/audit quick` → `/ship`).
- **Branch:** develop
- **Commit:** 032cbd0f (local, sin push)
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
Estado: completed
Última acción: Cierre completo: review P2-01 ✅ (2 rondas: ❌ → R1/R2 fixes + R3 waiver → ✅); commits locales 3713773b (fix R1/R2) + 925c66b6 (docs cierre); contrato 4/4.
Resultado: OK
Próxima acción: API-02 y API-03 en paralelo (plan api-ejecucion.md); push pendiente de instrucción del owner
Contrato: verificacion: cargo test --test sdk_serialization -> 18 passed/0 failed | wire u128 >2^53 en 4 bindings (Py 2/2, TS 1/1, Node 28/28, WASM 30/30) | verify_changed 4/4 | review P2-01 APPROVE
Próxima tarea si completa: API-02
=== END RECITATION ===

=== RECITATION API-03 ===
Campaign ID: beca0c27-fd85-4489-8f93-8361888d662c
Objetivo activo: API-03: W2 HTTP/OpenAPI-first REST + paginación (cursor único)
Estado: completed
Última acción: Completo: 7/7 steps. Rutas migradas (/maintenance/* plurales, /api/v2/conversations, GET /api/v2/skills, /threads/{id}/messages), status=YAML (201 en 6 creates), cursor opaco string + has_more + limit en list/search/audit/threads/skills, 5 drifts YAML, HTTP_API.md sync, parity test 10/10 + script OK + e2e 17/17 + smoke curl 15/15; review P2-01 ronda 1 CHANGES (limit=0 sticky en search/audit, tabla docs, 400 sin documentar) → fixes → ronda 2 APPROVE. Sin commit (política owner; lo hace el lead).
Resultado: OK
Próxima acción: Lead: commit local selectivo de los 12 paths API-03 (excluir WIP API-02) + skill progreso. Luego API-04 (depende de API-03).
Contrato: verificacion: node scripts/check_openapi_parity.mjs -> Parity OK (37 paths) | cargo test --target-dir target/session-api01 --test openapi_yaml_parity -> 10 passed/0 failed | cargo test --target-dir target/session-api01 -p vantadb-server --test e2e -> 17 passed/0 failed | rg 'maintenance/purge|conversation/add|skill/listing' src/server/router.rs -> 0 | rg offset src/server/handlers.rs -> 0 | curl por grupo (records/search/threads/maintenance/export + conversations/skills) 15/15 status=YAML (script target/session-api01/smoke-api03.ps1) | clippy -p vantadb --features server --all-targets -D warnings + -p vantadb-server --all-targets OK | evidencia: claim 'cursor opaco + has_more + limit' evidencia 'handlers.rs + pagination.rs + e2e test_e2e_list_cursor_pagination_roundtrip + smoke live' confianza alta; claim 'review P2-01' evidencia 'docs/dev/tasks/API-03.md §Review ronda 2 APPROVE' confianza alta; artefactos: docs/dev/tasks/API-03.md, src/server/pagination.rs (nuevo); invariantes: router.rs <-> openapi.yaml lockstep (gate-docs-21), no tocar src/sdk/wal/vector/storage, MCP in-process intacto, no commit/push por política owner; deuda: versions sin paginar (fuera de contrato), dashboard TS cursor types numericos (runtime pass-through OK), comentarios /conversation/add en vanta-memory (candidato FIND-*), quirk multi-ns cursor por-namespace pre-existente; queda_pendiente: commit local selectivo del lead + skill progreso + push solo con instrucción del owner
Próxima tarea si completa: API-04
=== END RECITATION ===

=== RECITATION API-02 ===
Campaign ID: beca0c27-fd85-4489-8f93-8361888d662c
Objetivo activo: API-02: W1 bindings — score + firmas + getNode + u128 (4 toolchains)
Estado: in-progress
Última acción: Cierre técnico completo: TS score (4 sitios) + roots NodeId/guard; Python rename insert_node/get_node/delete_node + put_batch([{...}]) (P2-5 pagada) + search_multi + stubs + tests; docs matriz W1. Review P2-01: ❌→R1/R2/R3 aplicados→✅ APPROVE. Working tree sin commit.
Resultado: PARTIAL
Próxima acción: Lead: commit local de API-02 (git add <archivos listados en task file §Impacto mapeado>; feat!: API-02) — NO push. Luego continuar API-04 (o cierre de API-03 paralela).
Contrato: verificacion: cargo test --target-dir target/session-api01 --test python_sdk_boundary -> 1 passed/0 failed | npx tsc --noEmit -> exit 0 | npx vitest run (vantadb-ts) -> 314/314 | npx vitest run (vantadb-node) -> 36/36 | pytest vantadb-python/tests -> 149 passed/0 failed | rg "distance: h.score" -> 0 en codigo (solo docs historicos) | matriz W1 -> docs/api/BINDINGS_NAMESPACES.md §W1 parity matrix + tests por binding | evidencia: claim=R1 review (roots number[] residual en 11 sitios) corregido y re-verificado (tsc 0, vitest 314/314) -> file: vantadb-ts/src/vantadb.ts:91-1472 + review session | claim=Spec #5 enmendada sin method (core search_multi no acepta override; Node/WASM tampoco) -> file: docs/dev/tasks/API-02.md:48 | claim=Python 46 metodos flat reales (16 memory/11 graph/18 system/1 wiki) -> tool: dir(vantadb_py.vantadb_py.Client) | claim=P2-5 pagada (una sola API put_batch) -> file: vantadb-python/src/lib.rs (put_batch(records) + record_to_memory_input) | claim=FIND-79 DEFER aceptado por stop condition -> file: docs/dev/tasks/API-02.md Spec #7 | artefactos: docs/dev/tasks/API-02.md, vantadb-python/tests/test_w1_surface.py, vantadb-ts/tests/graph.test.ts | invariantes: u128_serde unico patron wire; distance solo ANN crudo; put_batch_raw zero-copy intacto; GIL release en ops nuevas; vantadb-pro intocable; WIP ajeno (API-03/WIRE-10) intocable | deuda: P2-5 pagada; FIND-79 DEFER documentado; Node extras fuera de W1; FIND candidatos (d) INVALID_ARGUMENT sin prefijo, (e) tsc no typechequea tests | queda_pendiente: commit local del lead + sync final; push solo con instruccion explicita del owner
Próxima tarea si completa: API-04
=== END RECITATION ===

=== RECITATION API-05 ===
Campaign ID: beca0c27-fd85-4489-8f93-8361888d662c
Objetivo activo: API-05: W4 proxy — auth /snapshot + endpoints + config (SEGURIDAD)
Estado: in-progress
Última acción: 8/8 steps: RED 401/200 (falso→probado) + auth D34 en /snapshot; rutas canónicas (/sessions/advance, {space_id}, /{agent}/{space_id}/v1/responses); upstream default vacío + fail-fast; ttl_secs=0 = cache off (TTL_DISABLED eliminado); rate-limit docs truth; PROXY.md + EXPERIMENTAL_FEATURES + FIND-155 (desktop, ya barrido por commit WIRE-11 concurrente); smoke binario real 401/200.
Resultado: PARTIAL
Próxima acción: vanta-review P2-01 (cambio de auth de red; evidencia en docs/dev/tasks/API-05.md §Review/§Notas) → veredicto → commit local del LEAD (lista de archivos en task file §Notas). Luego API-06/07/08.
Contrato: verificacion: cargo test --target-dir target/session-api01 -p vanta-proxy -> exit 0, 291 passed/0 failed/18 binarios (x3 corridas) | cargo fmt -p vanta-proxy --check + cargo fmt --all -- --check -> exit 0 | cargo clippy --target-dir target/session-api01 -p vanta-proxy --all-targets --no-deps -- -D warnings -> exit 0 | smoke binario+curl -> no-key=401, with-key=200 (sk-smoke), bad-key=401 | rg spaceId vanta-proxy/src/server.rs -> 0 | rg -c '\.route\(' server.rs -> 11 | scripts/validate-docs-coverage.ps1 -> 0 gaps | evidencia: claim=auth obligatoria en /snapshot sin bypass loopback -> file: vanta-proxy/src/server.rs:871 (snapshot+authenticate) + vanta-proxy/tests/api05_snapshot_auth.rs::snapshot_requires_user_key + smoke logs target/session-api01/tmp/api05-smoke.*.log (confianza alta) | claim=rutas canónicas -> file: server.rs:788-823 + handlers/responses.rs::responses_prefixed (confianza alta) | claim=config fail-fast upstream vacío + self-loop explícito -> file: config.rs:253,358-370 + tests config (confianza alta) | claim=ttl=0 desactiva cache -> file: cache.rs:110-122 + tests zero_ttl_disables_cache_even_when_enabled (confianza alta) | claim=desktop rompe (F1) -> file: desktop/src/components/proxy/ProxyDashboard.tsx:59-63 -> FIND-155 Backlog:253 (confianza alta) | claim=rate-limit SI cableado (1 punto) -> file: server.rs:161,367; fix solo docs (confianza alta) | artefactos: docs/dev/tasks/API-05.md, vanta-proxy/tests/api05_snapshot_auth.rs, target/session-api01/tmp/api05-smoke.toml|authdb|logs | invariantes: auth D34 en TODAS las rutas sin bypass loopback; wire /v1 byte-identical sin opt-in (pipeline intacto); no tocar mcp/server/bindings/parser/pro; no commit/push por worker | deuda: ninguna nueva (PR paga la exposición X1); nota: output-side cost tracking queda acoplado al path buffered (cache off => input-only) documentado en PROXY.md; FIND-155 para desktop | queda_pendiente: review P2-01 + commit local del lead + push solo con instruccion del owner
Próxima tarea si completa: API-06
=== END RECITATION ===

=== RECITATION API-06 ===
Campaign ID: beca0c27-fd85-4489-8f93-8361888d662c
Objetivo activo: API-06: W5 IQL — IQL_VERSION + sintaxis + literales (==/Int/quotes) + AST JSON
Estado: completed
Última acción: Steps 0-8 ✅. RED→GREEN para ==/42→Int/big-int; gate IQL_VERSION (PROFILE); SELECT default alias target; Serialize en AST + shape test; docs IQL.md + openapi example (fully-consumed test); FIND-156/157 en Backlog. Review P2-01 ronda 1 ❌ → R1/R2/R3 aplicados → ronda 2 ✅ APPROVE. Contrato 4/4 vía campaign_verify_cmd. Sin commit (lead).
Resultado: OK
Próxima acción: Lead: commit local selectivo (src/parser/**, src/query.rs, src/lib.rs, tests/logic/parser.rs, tests/logic/snapshots/parser__dml_insert_ast.snap, tests/api/openapi_yaml_parity.rs, docs/api/IQL.md, docs/api/openapi.yaml, docs/dev/tasks/API-06.md, docs/dev/Backlog.md, docs/dev/plans/2026-09-24-api-ejecucion.md) + skill progreso; push solo con instrucción del owner. Luego API-07 (W6 CLI).
Contrato: verificacion: campaign_verify_cmd 4/4 → rg IQL_VERSION exit 0 · cargo test --target-dir target/session-api01 -p vantadb --lib parser 127/0 · --test parser 4/0 · --test openapi_yaml_parity 11/0 | fmt/clippy -D warnings/docs-coverage verdes | lib subsets query 78/executor 24/physical 51/planner 13 + --test executor 1/1 + --test mutations 1/1 | evidencia: claim == longest-match -> grammar.rs:47-60 + RED 5 failed; claim 42→Int exacto i64 -> lexer.rs:128-158 + test_parse_literal_int_above_2_53_is_exact; claim gate -> mod.rs:17-27 + grammar.rs:120 + lib.rs:180; claim example YAML fully-consumed -> openapi_yaml_parity nueva test; claim review -> docs/dev/tasks/API-06.md §Review ronda 2 ✅ APPROVE; artefactos: docs/dev/tasks/API-06.md, docs/api/IQL.md; invariantes: UPPERCASE-only pinneado, PROFILE v1 parsea, openapi parity verde, no bindings/MCP/server tocados, snapshot actualizado a propósito; deuda: FIND-156 (AST JSON en bindings) + FIND-157 (coerción Int/Float pre-existente); queda_pendiente: commit local + skill progreso del lead; push solo con instrucción del owner
Próxima tarea si completa: API-07
=== END RECITATION ===

=== RECITATION API-04 ===
Campaign ID: beca0c27-fd85-4489-8f93-8361888d662c
Objetivo activo: API-04: W3 MCP — nombres canónicos + schemas estrictos + errores tipados + refresh local
Estado: completed
Última acción: Completo 8/8 steps. Canonicalización (memory_search/memory_list_namespaces listados; legacy search_memory/collection_list unlisted pero dispatchables; prompt recall_search + redirect), schemas estrictos base (additionalProperties:false, 47 tools; Schemars codegen diferido con justificación), invalid_params temprano + errores tipados (factories not_found/resource_limit + error_content_mcp), thread_id u128-string (legacy u64), docs mismo-PR (+espejos hash-SAME), smoke live MCP 11/11, review P2-01 ronda 1 🔴 F1-F5 → fixes → ronda 2 ✅ APPROVE. Sin commit (política owner; lo hace el lead).
Resultado: OK
Próxima acción: Lead: commit local selectivo del changeset API-04 (16 tracked + docs/dev/tasks/API-04.md; excluir WIP API-05/06) + skill progreso. Owner: rebuild target/debug (vanta-cli+vantadb-server) + restart OpenCode + re-listar tools (refresh local MCP).
Contrato: verificacion: cargo fmt -p vantadb-mcp -- --check -> 0 | cargo clippy --target-dir target/session-api01 -p vantadb-mcp --all-targets --jobs 2 -- -D warnings -> 0 | cargo test --target-dir target/session-api01 -p vantadb-mcp --jobs 2 -> exit 0 (20/20 binaries; mcp_tests 101/101) | scripts/validate-docs-coverage.ps1 -> 0 gaps (47 tools MCP.md + 10 pares espejos hash-SAME) | python target/session-api01/smoke-api04.py -> SMOKE API-04 OK 11 checks | rg '"name": "search_memory"|"name": "collection_list"' (repo excl target/docs/dev) -> 0 matches | evidencia: claim='tools/list 85 unicos sin duplicados y legacy no listado' evidencia='smoke checks 2-3 + test test_api04_tools_list_canonical_names_no_duplicates' confianza alta; claim='strict schemas base 47 con additionalProperties:false y dicts abiertos' evidencia='smoke check 3 + test_api04_base_tool_schemas_strict' confianza alta; claim='thread_id string en schema, u128 string aceptado, legacy u64; float/negativo rechazados con data.code' evidencia='smoke check 3 + probes reviewer ronda 2 (2^53+1 y u64::MAX aceptados; float/negativo -32602 typed)' confianza alta; claim='param-level -32602 con data.code; dominio isError tipado MEM-32' evidencia='test_api04_query_iql/memory_recall/inject_context + test_api04_domain_errors_are_typed_envelopes + reviewer raw wire' confianza alta; claim='prompt recall_search sin colisión + redirect legacy byte-identico' evidencia='test_api04_prompts_are_separated_from_tools + reviewer' confianza alta; claim='bulk_import_stream by-design intacto' evidencia='tools.rs solo cambio envelope de errores; MCP.md marcado' confianza alta; claim='review P2-01' evidencia='docs/dev/tasks/API-04.md §Review ronda 2 APPROVE' confianza alta; artefactos: docs/dev/tasks/API-04.md, target/session-api01/smoke-api04.py (+log), docs/api/MCP.md; invariantes: 1 nombre por tool en tools/list (legacy solo dispatch), prompts/resources/tools registros separados, errores JSON-RPC Err=param-level / isError tipado=dominio, no tocar target/debug (MCP owner), no tocar WIP API-05/06 ni src/server|vanta-proxy|src/parser|bindings, no commit/push; deuda: Schemars codegen single-source diferido (FIND candidato), strictness+errores tipados familias extendidas (38 tools) pendientes (FIND candidato), envelopes internal_error sin data.code en fallos String (FIND candidato); queda_pendiente: commit local del lead + skill progreso + push solo con instruccion del owner; refresh local MCP (rebuild target/debug + restart OpenCode) es del owner
Próxima tarea si completa: API-07
=== END RECITATION ===

=== RECITATION 8 ===
Campaign ID: beca0c27-fd85-4489-8f93-8361888d662c
Objetivo activo: API-08: W7 vanta-memory — API Rust estable (NO exponer, Gate P core-only)
Estado: completed
Última acción: Cierre corregido: FIND-160 registrada en Backlog (cobertura doc vanta-memory post-MEM-38; NOTICED BUT NOT TOUCHING del discovery). Changeset final: docs/api/VANTA_MEMORY.md (+107/-20) + vanta-memory/src/lib.rs (doc-only, +11/-1) + docs/dev/tasks/API-08.md (nuevo) + docs/dev/Backlog.md (fila FIND-160).
Resultado: OK
Próxima acción: Lead: commit local selectivo (4 paths del changeset API-08; excluir WIP masivo API-07) + skill progreso + bookkeeping plan; push solo con instrucción del owner. Luego API-09 (W8 cierre).
Contrato: verificacion: cargo test --target-dir target/session-api01 -p vanta-memory → 28 targets/0 failed/EXIT=0 | focused degrade llm_free_mode_reports_not_configured → 1 passed (llm_runner.rs:237-250) | precise-tokens golden → 1 passed | rg "vanta[_-]memory" bindings → 0 matches | validate-docs-coverage → 0 gaps | fmt/clippy scoped → 0 | campaign_verify_cmd 4/4 | evidencia: claims D37/D21/MEM-16+MEM-48 re-verificadas claim-por-claim por reviewer (ses_f23e51bc6ffeeKMMEmlG7z2aZx ronda 2 APPROVE); artefactos: docs/dev/tasks/API-08.md, docs/api/VANTA_MEMORY.md, docs/dev/Backlog.md (FIND-160), target/session-api01/api08-*.log|diff|json; invariantes: core-only 0 símbolos bindings; degradación P4 intacta; no tocar cli/mcp/proxy/server/parser ni ingest/skill; no commit/push worker; deuda: residual D37 (keyword sin provider) + residual scoring (offload-entry scores) con triggers documentados; rustdoc warnings pre-existentes fuera de scope; queda_pendiente: commit local del lead + skill progreso + bookkeeping
Próxima tarea si completa: API-09
=== END RECITATION ===

=== RECITATION API-07 ===
Campaign ID: beca0c27-fd85-4489-8f93-8361888d662c
Objetivo activo: API-07: W6 CLI — POSIX + --json global + flags simétricos
Estado: completed
Última acción: 12/12 steps ✅. Superficie CLI normalizada: --json global (clap global=true) en todos los comandos con salida completa y pureza (missing-DB → null/[]/objetos); --in/--out simétricos (+alias --input); --limit canónico (+alias --top-k); QUERY posicional (+alias --query); truncado humano TTY-gated char-safe; count sin DB → Err/exit≠0 y count --json objeto; put vía SDK Embedded::put+flush; lecturas search/count en read-only (shared lock, sin ensure_indexes_current); logs a stderr (console.rs); docs+completions regeneradas; smoke 56/56, cli_tests 88/88, lib 2124/0, fmt/clippy 0. Review P2-01 ronda 1 ❌ (5 hallazgos) → fixes → ronda 2 ✅ APPROVE. Sin commit (política owner; lo hace el lead).
Resultado: OK
Próxima acción: Lead: commit local selectivo de API-07 (24 archivos src/tests/docs/completions + docs/dev/tasks/API-07.md + plan; EXCLUIR changeset API-08: vanta-memory/**, docs/api/VANTA_MEMORY.md, docs/dev/tasks/API-08.md) + skill progreso; push solo con instrucción del owner. Luego API-08/API-09.
Contrato: verificacion: cargo fmt --all --check -> exit 0 | cargo clippy --target-dir target/session-api01 -p vantadb --features cli,fjall,memmap2,fs2,roaring --lib --bins --test cli_tests -- -D warnings -> exit 0 | cargo test --test cli_tests -> 88 passed/0 failed | cargo test --lib -> 2124 passed/0 failed/2 ignored | cargo build --bin vanta-cli -> exit 0 | pwsh -NoProfile -File target/session-api01/smoke-api07.ps1 -> 56/56 checks, exit 0 | evidencia: claim='--help × comando capturado (contrato#1)' evidencia='target/session-api01/help/help_*.txt 39 capturas exit 0 + smoke check' confianza alta; claim='--json en TODOS con salida completa + humano truncado solo TTY (contrato#2)' evidencia='smoke 23+ checks parse/pureza + payload 304 chars íntegro json y piped + tests api07_cli_binary + unit truncate_for_term' confianza alta; claim='count sin DB exit≠0 (contrato#3)' evidencia='binario exit 1 + test_count_missing_db_errors + smoke' confianza alta; claim='println!("{count}") revisado (contrato#4)' evidencia='rg src=0 + count --json objeto {namespace,count,filter}' confianza alta; claim='lecturas sin ensure_indexes_current-RW (contrato#5, Regla 8)' evidencia='search.rs 4 fns open_embedded(true) shared lock + rg RW opens=0 + init.rs:199-217 try_lock_shared + builder.rs:113-114 ensure solo !read_only + regresión AUD-044 verde porque put vía SDK mantiene índices' confianza alta; claim='put vía SDK no bypass' evidencia='crud.rs cmd_put Embedded::put(MemoryInput)+flush; reserved __vanta_ y JSON inválido siguen rechazados (tests)' confianza alta; claim='logs a stderr; -v --json puro' evidencia='console.rs with_writer(stderr) 3 formatos + smoke put -v --json pure' confianza alta; claim='review P2-01' evidencia='docs/dev/tasks/API-07.md §Review ronda 2 APPROVE (sesión vanta-review ses_f23c884f5ffeuBuIxhFvAYwvJ0)' confianza alta; artefactos: docs/dev/tasks/API-07.md, target/session-api01/smoke-api07.ps1, target/session-api01/help/ (39), target/session-api01/ocr-api07.json, completions/ (4 shells), docs/user/operations/CONFIGURATION.md (+4 docs); invariantes: mcp-call/server --mcp/desktop sidecar intactos; put rechaza __vanta_ + flush ERR-050b; search fresco tras put sin rebuild (AUD-044); salidas humanas stdout/errores stderr; src/parser, src/server, bindings, vanta-memory, vantadb-mcp, vanta-proxy intocados; no commit/push por worker; deuda: D1 staleness de índices en DB escrita por CLI viejo hasta rebuild-index (documentado en CONFIGURATION.md upgrade note); D2 spawn vantadb-server PATH/cwd pre-existente; nit completions bash lista --query oculto; queda_pendiente: commit local selectivo del lead (excluir API-08) + skill progreso + push solo con instrucción del owner; nota: disco C ~2GB libres — deps de target/session-api01/debug (~19GB) candidatos a limpieza (costo: rebuild) si el próximo build falla por StorageFull
Próxima tarea si completa: API-08 → API-09
=== END RECITATION ===

=== RECITATION API-09 ===
Campaign ID: beca0c27-fd85-4489-8f93-8361888d662c
Objetivo activo: API-09: W8 cierre — VERSIONING 11 superficies + docs/api sincronizadas + gates de cierre
Estado: completed
Última acción: Cierre completo 8/8 steps: VERSIONING 11 superficies + sync docs (scores.md MCP score/distance; ERROR_HANDLING/HTTP_API RFC9457+code) + FIND-161; validate-docs-coverage 0 gaps; verify.ps1 r2 ALL 11 PASS (fix tooling llvm-cov: `nextest run`→`nextest`, coverage real 81.63% ≥60); MCP re-smoke 11/11 + probe 2/2; OCR 0 Critical/High; plan 9/9. Changeset docs-only + dev-tools/verify.ps1; sin commit (LEAD).
Resultado: OK
Próxima acción: LEAD: commit local selectivo (7 tracked + docs/dev/tasks/API-09.md) + skill progreso + archivar planes + /audit quick → /ship. Owner: rebuild target/debug + restart OpenCode (refresh MCP local; el re-smoke corrió contra target/session-api01).
Contrato: verificacion: `pwsh scripts/validate-docs-coverage.ps1` -> exit 0, 0 gaps | `$env:CARGO_TARGET_DIR='target\session-api01'; pwsh dev-tools/verify.ps1` -> ALL 11 PASS, exit 0 (r2; 1a corrida roja por bug de tooling corregido) | `python target/session-api01/smoke-api04.py` -> SMOKE API-04 OK 11 checks | `python target/session-api01/probe-api09.py` -> PROBE OK 2 checks | `pwsh dev-tools/ocr-review.ps1 -Format json` -> spec ok, 0 Critical/High (diff docs-only: 0 reviewable) | plan 18/18 + 9/9 (headers + Task 9 COMPLETED)
evidencia:
- claim: 'VERSIONING.md lista las 11 superficies (fuente API-STD-01 + Gate P API-STD-15)' evidencia: 'docs/api/VERSIONING.md (SURFACE ROWS=11; 20 links Test-Path 0 bad; sin rows=11 verificado)' confianza alta
- claim: 'scores.md MCP: memory_search=score (hybrid) / search_semantic=distance (raw ANN)' evidencia: 'probe live: memory_search score=1.0 sin distance, search_semantic distance=0.0; src: vantadb-mcp/src/handlers/tools.rs:3140-3143,1858-1913 + src/sdk/serialization/vector_types.rs:69-76' confianza alta
- claim: 'RFC 9457 en HTTP no implementado; envelope real {success,code,error|data}; FIND-161 registrada' evidencia: 'rg problem+json src/server = 0; src/server/errors.rs:132-163; docs/dev/Backlog.md FIND-161' confianza alta
- claim: 'fix verify.ps1 coverage sin debilitar el gate (umbral 60 intacto)' evidencia: 'target/session-api01/api09-coverage.log TOTAL 81.63% lineas exit 0; api09-verify-r2.log ALL 11 PASS; forma canonica CI ci-rust.yml:363 (sin token run)' confianza alta
- claim: 'MCP re-smoke verde contra binarios del session dir' evidencia: 'smoke-api04.py 11/11 + probe 2/2 (binarios target/session-api01/debug)' confianza alta
artefactos: docs/api/VERSIONING.md, docs/api/scores.md, docs/api/ERROR_HANDLING.md, docs/api/HTTP_API.md, docs/dev/Backlog.md (FIND-161), docs/dev/plans/2026-09-24-api-ejecucion.md (9/9), dev-tools/verify.ps1 (fix), docs/dev/tasks/API-09.md, target/session-api01/{api09-verify.log, api09-verify-r2.log, api09-coverage.log, api09-ocr.json, probe-api09.py}
invariantes: no se toco codigo de producto ni CHANGELOG/versiones (release-plz); no se archivaron planes (LEAD); gates con CARGO_TARGET_DIR=target\session-api01; umbral coverage 60 intacto; docs tecnicas EN
deuda: FIND-161 nueva (RFC 9457 HTTP pendiente, dueno sugerido server); deuda previa intacta (FIND-155..160, FIND-79 DEFER); sin deuda tecnica nueva (docs-only + 1 fix de tooling)
queda_pendiente: LEAD — commit local selectivo (7 tracked + task file), skill progreso, archivar planes, /audit quick -> /ship; OWNER — rebuild target/debug + restart OpenCode (refresh MCP local)
Próxima tarea si completa: ninguna — ultima de la campana; cierre = LEAD (progreso + archivado + /ship)
=== END RECITATION ===
