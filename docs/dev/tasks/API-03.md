# Task API-03 — W2 HTTP/OpenAPI-first: REST + paginación (cursor único)

> **Plan:** `docs/dev/plans/2026-09-24-api-ejecucion.md` (§Task 3)
> **Campaign:** beca0c27-fd85-4489-8f93-8361888d662c
> **Estado:** ⏳ IN PROGRESS
> **Fecha:** 2026-09-25
> **Branch:** develop (commit lo hace el LEAD al cierre — política 2026-09-25; sin push)
> **Cynefin:** 🟨 complicado — owner YAML + rutas breaking + migración de consumidores
> **SDP:** feature-add · BUILD — campaign-executor, incremental-implementation, test-driven-development, context-engineering, source-driven-development, doubt-driven-development, api-and-interface-design (`campaign_discover_skills_v2` API-03, 8 skills)
> **Type:** auto-detect `docs` (ruido del label por `docs/api/` en claves) → re-clasificado **feature-add** (cambio de contrato HTTP + símbolos handler re-verificados) según re-validación §Paso 0.

## 1. Objetivo + contrato

Alinear el servidor HTTP a **OpenAPI-first**: plurales sin verbos, `/api/v2` total, status=YAML (201 para creates), paginación **cursor único** (cursor opaco string + `has_more` + `limit`; cero `offset`), `RecordInput` opcional, `next_cursor` 1 tipo (string|null), y los 5 drifts del YAML.

**Contrato (ley — `docs/dev/plans/2026-09-24-api-ejecucion.md` §Task 3):**

1. `cargo test --test openapi_yaml_parity` verde.
2. curl por grupo (records/search/threads/maintenance/export) con status = YAML.
3. `rg "maintenance/purge|conversation/add|skill/listing"` en router = 0.
4. `rg "offset"` en `handlers.rs` = 0 (cursor único).

## 2. Gate D / Plan de solución (Gate P aplicado)

- **Gate Justificación (plan):** contrato externo + base MCP; YAML pasa a owner efectivo.
- **Gate P (API-STD-15) ya decidió:** REST = plurales sin verbos · `/api/v2` total · `/threads/{id}/messages` · cursor opaco + `has_more` + `limit`; OpenAPI = `RecordInput` opcional · case único · `next_cursor` 1 tipo. **No hay preguntas abiertas** (el prompt de tarea prohíbe `question`; decisiones abiertas → BLOQUEO).
- **Pre-mortem (plan):** F1 gemelos v2 duplicados en vez de migrar → se **migran** rutas (sin gemelos legacy); F2 `RecordInput` opcional rompe validación MCP → MCP es in-process y su schema es propio (API-04), HTTP solo relaja `required` que el core ya acepta (`Option`); F3 gateway por entusiasmo → **NO** se agrega endpoint genérico de mantenimiento (4 sub-recursos separados).
- **Stop conditions (plan):** appetite >1sem → HTTP vs YAML en 2 PRs (no aplica: ~1d); server no levanta → solo lectura+parity (se intenta curl; si no levanta, se declara).

### Tabla de ruta-a-ruta (breaking `feat!:`)

| Antes | Después | Método | Status impl/YAML |
|---|---|---|---|
| `POST /api/v2/maintenance/purge` | `/api/v2/maintenance/expired-records` | `DELETE` | 200 |
| `POST /api/v2/maintenance/compact` | `/api/v2/maintenance/compactions` | `POST` | 200 |
| `POST /api/v2/maintenance/flush` | `/api/v2/maintenance/flushes` | `POST` | 200 |
| `POST /api/v2/maintenance/rebuild-index` (long-running) | `/api/v2/maintenance/index-rebuilds` | `POST` | 200 |
| `POST /api/v2/threads/{id}` (mensaje) | `/api/v2/threads/{id}/messages` | `POST` | 200 |
| `POST /conversation/add` | `/api/v2/conversations` | `POST` | 201 |
| `GET /skill/listing` | `GET /api/v2/skills` (junto a `POST`) | `GET` | 200 |
| `POST /api/v2/records` · `/records/batch` · `/threads` · `/snapshots/{name}` | (sin cambio de path) | `POST` | impl 201 → **YAML 200→201** |

**Fuera de alcance (FIND si Gate P lo exige):** `POST /api/v2/export` y `POST /api/v2/import` son igualmente verbos pero **no** están en la lista del plan (`/maintenance/*`, `/conversation/add`, `/skill/listing`, `/threads/{id}/messages`) ni en el grep del contrato → no se tocan. `/health`, `/metrics`, `/dashboard` quedan como excepciones documentadas (gemelos v2 existen para health/metrics; dashboard es UI loopback D12).

### Paginación (cursor único)

- Request: `limit` (clamp `MAX_K`) + `cursor` (**string opaco**; omitir = primera página).
- Response: colección + `next_cursor: string|null` + `has_more: bool`.
- Endpoints: `GET /api/v2/list`, `POST /api/v2/search`, `GET /api/v2/audit`, `GET /api/v2/threads`, `GET /api/v2/skills`.
- Cursor inválido → `400 {success:false,error:"invalid cursor: …"}`.
- **Deuda registrada:** `GET /records/{ns}/{key}/versions` mantiene `?version=` sin paginar (YAML documentaba `?limit` inexistente → se alinea el doc; paginación con cursor queda diferida, fuera del contrato).
- **Decisión de encoding (ponytail):** el cursor viaja como **string decimal** (mismo precedente que WASM `next_cursor_to_js`, `lib.rs:224`), documentado como opaco: el cliente lo devuelve tal cual, nunca lo parsea. Sin dependencia nueva (`base64` no es dep directa); si se necesita opacidad real más adelante, se cambia el encoding server-side sin romper el wire (ya es string).

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/server/router.rs` (408L) · `src/server/handlers.rs` (1541L) · `src/server/list_records.rs` (300L) · `src/server/routing.rs` (facade, verificado) · `docs/api/openapi.yaml` (2468L) · `tests/api/openapi_yaml_parity.rs` (329L) · `scripts/check_openapi_parity.mjs` (parity router↔YAML, gate-docs-21) · `vanta-memory/src/services/conversation_hook.rs` · `desktop/src/vanta-http-map.ts` (solo lectura, mapa HTTP del dashboard) · `docs/api/HTTP_API.md` (secciones ruta-por-ruta + Route Summary) · `.opencode/rules/server-mcp.md` · `docs/dev/tasks/API-STD-07.md` / `API-STD-15` / `API-STD-16`.
- **Referencias hacia dentro (imports/deps):** `router.rs` → `handlers::*` + `state`; `handlers.rs` → `sdk` (Memory*/Skill*/Embedded), `server::conversation`, `server::list_records`, `server::errors`; `routing.rs` re-exporta handlers (facade intacta: no se renombran funciones).
- **Referencias entrantes (grep por rutas/handlers):** `vantadb-server/tests/e2e.rs` (`/conversation/add` ×6, `/skill/listing` ×4) · `desktop/src/vanta-http-map.ts` (`/api/v2/export` — no cambia; `/api/v2/list`, `/api/v2/search`, `/api/v2/audit` consumen cursor — pass-through opaco, runtime intacto; tipos TS numéricos stale → nota) · `vanta-memory` (comentarios con `/conversation/add`) · `.github/workflows/gate-docs.yml` (parity script + versión YAML) · `tests/api/openapi_yaml_parity.rs` (paths `maintenance/rebuild-index`).
- **Veredicto impacto:** **alto** (breaking de API externa: rutas + tipos de paginación + status). Ningún consumidor in-tree rompe en runtime: MCP usa SDK in-process, dashboard HTTP pasa el cursor como opaco, e2e se actualiza mismo-PR. Docs mismo-PR (Regla 3). YAML es owner: rutas y schemas se cambian en lockstep (parity script gate-docs-21 exige paridad exacta router↔YAML).

## Contrato (mecánico)

```
cargo test --target-dir target/session-api01 --test openapi_yaml_parity   # verde
node scripts/check_openapi_parity.mjs                                     # router ↔ YAML exacto
rg "maintenance/purge|conversation/add|skill/listing" src/server/router.rs # 0
rg "offset" src/server/handlers.rs                                         # 0
curl por grupo (records/search/threads/maintenance/export) status=YAML
```

## Spec (decisiones)

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Nombres mantenimiento (plural sin verbo) | A `expired-records`/`compactions`/`flushes`/`index-rebuilds` · B `purges`/… (rechazada: `maintenance/purges` contiene el substring `maintenance/purge` → viola el grep del contrato) | ✅ decidido-por-evidencia (contrato grep + Backlog fila `API-03`: «plurales sin verbos (`/maintenance/*`…)») |
| 2 | `conversation/add` destino | A `POST /api/v2/conversations` (preserva semántica create-if-absent) · B forzar `POST /threads` + `/messages` (rompe el one-shot del bridge MEM-55) | ✅ A — decidido-por-evidencia (handler `conversation_add` + `ConversationTrigger`) |
| 3 | `skill/listing` destino | A `GET /api/v2/skills` (colección existente; F1: migrar, no duplicar) · B gemelo `/api/v2/skill-listings` | ✅ A |
| 4 | Message route | A `POST /threads/{id}/messages` (Gate P literal) | ✅ A (Gate P) |
| 5 | Status creates | A YAML→201 (REST-correct; impl ya 201) · B impl→200 (viola semántica HTTP) | ✅ A — status=YAML con YAML corregido |
| 6 | Cursor tipo | A string opaco decimal (precedente WASM; YAML ya `string|null`) · B `usize` (YAML tendría que documentar number; contradice “cursor opaco”) | ✅ A |
| 7 | `has_more` presencia | A en las 5 respuestas paginadas · B solo next_cursor | ✅ A (Gate P literal) |
| 8 | YAML drifts | 5: RecordInput required→3+metadata · ejemplo Lisp→IQL · enum `Dot`→`SparseDot` · `next_cursor` 1 tipo string · tag `Skills` declarado | ✅ decidido-por-evidencia (`node/vector_data.rs:11-21` = Cosine/Euclidean/SparseDot) |
| 9 | export/import verbos | A no tocar (fuera del alcance listado) · B renombrar a `/exports`,`/imports` | ✅ A + nota en §Deuda |
| 10 | Versions paginación | A alinear YAML a impl (`?version=`) · B implementar cursor (nuevo scope) | ✅ A (contrato no lo pide) + DEUDA |

## Invariantes de dominio (handoff — MUST)

- **Lockstep router↔YAML:** cada cambio de path/método debe tocar `router.rs` + `openapi.yaml` en el mismo commit, o `check_openapi_parity.mjs` (gate-docs-21) falla en CI. Verificación: `node scripts/check_openapi_parity.mjs`.
- **No tocar:** `src/wal.rs`, `src/vector/`, `src/storage/` (Arch/Engine) · WIP ajeno WIRE-10 (`examples/colab/`, `scripts/install.sh`, `skills/vantadb-mcp/assets/hooks/**`, `src/bin/vanta-cli.rs`, `src/cli.rs`, `src/cli_handlers/**`).
- **MCP intacto:** `vantadb-mcp` es in-process; no comparte las rutas HTTP (su contrato es propio, API-04). Los paths HTTP no son API del SDK.
- **Serialización SDK intacta:** `src/sdk/**` no se toca; el wire HTTP sigue usando el serde del SDK (solo cambia el tipo local del cursor en `handlers.rs`).
- **Facade `routing.rs`:** no se renombran funciones handler (solo strings de ruta) → re-exports intactos.
- **Deuda:** quedaría `versions` sin paginar + `export/import` verbales (nota), y dashboard TS con tipos de cursor numéricos stale (runtime OK).

## Steps (atómico ~100 líneas c/u)

| # | Step | Verify | Estado |
|---|------|--------|--------|
| 1 | RED: extender `openapi_yaml_parity` con invariantes del contrato (rutas nuevas, status 201, cursor string+has_more, 5 drifts, `MemoryInput` opcional serializado) | 5 tests nuevos fallan por las razones correctas (evidencia: run inicial 5 failed / 5 passed) | ✅ |
| 2 | Router + YAML lockstep: 7 rutas nuevas + statuses 201 + 5 drifts + `check_openapi_parity.mjs` | parity script ✅ (`Parity OK: openapi.yaml matches the registered router exactly`) + parity test 10/10 ✅ + `cargo check -p vantadb --features server` ✅ | ✅ |
| 3 | Cursor codec (`src/server/pagination.rs`, +3 unit tests) + list/search (handlers + YAML schemas) + e2e | e2e `test_e2e_list_cursor_pagination_roundtrip` ✅; `has_more` exacto (probe limit+1 en list single-ns) | ✅ |
| 4 | audit/threads/skills paginación (handlers + YAML) | e2e `test_e2e_threads_list_cursor_page` + `test_e2e_skills_listing_page_envelope` ✅ | ✅ |
| 5 | `HTTP_API.md` mismo-PR (secciones + Route Summary + cursor + 201) | `scripts/validate-docs-coverage.ps1` → 0 gaps ✅ | ✅ |
| 6 | Verify full scoped (`fmt`/`clippy -p vantadb --features server --all-targets`/`clippy -p vantadb-server --all-targets`/parity/e2e) + curl por grupo (15 rows, status=YAML) | todo ✅ (evidencia abajo) | ✅ |
| 7 | Review P2-01 (`vanta-review`, contexto fresco) + cierre (sin commit — lo hace el lead) | veredicto registrado en §Review | ⏳ |

### Evidencia de verificación (Step 6)

- `node scripts/check_openapi_parity.mjs` → `Parity OK` (router 37 paths / YAML 37 paths) · vía `campaign_verify_cmd` ✅
- `cargo test --target-dir target/session-api01 --test openapi_yaml_parity` → **10 passed / 0 failed** ✅ (`campaign_verify_cmd`)
- `cargo test --target-dir target/session-api01 -p vantadb-server --test e2e` → **16 passed / 0 failed** ✅ (`campaign_verify_cmd`)
- `cargo test --test structured_api_v2` → 1 passed / 1 ignored (Ollama externo) ✅
- `cargo clippy -p vantadb --features server --all-targets -- -D warnings` ✅ · `cargo clippy -p vantadb-server --all-targets -- -D warnings` ✅
- `rustfmt --edition 2021 --check <8 archivos tocados>` ✅ (no se corrió `cargo fmt --all` para no formatear WIP ajeno de API-02)
- `rg "maintenance/purge|conversation/add|skill/listing" src/server/router.rs` → 0 ✅
- `rg "offset" src/server/handlers.rs` → 0 ✅
- `scripts/validate-docs-coverage.ps1` → 0 gaps ✅
- **curl por grupo** (server real `vanta-cli server --http --port 18099`, script `target/session-api01/smoke-api03.ps1`): 15/15 filas status == YAML (records 201/200, search 200, threads 201/200/200, maintenance 200×4, export 200×2, conversations 201, skills 200) + asserts de paginación (page1 `has_more=true`+cursor string, page2 terminal, cursor inválido → 400) ✅
- OCR delegation (advisory): `dev-tools/ocr-review.ps1 -Format json` ejecutado → reglas por grupo generadas (sin API key ⇒ sin findings emitidos; input para el reviewer). Sin Critical/High reportados por el script.
- Desviación TDD documentada: los unit tests del codec (`pagination.rs`) y los e2e de paginación se escribieron junto al GREEN de handlers (el RED quedó probado por los 5 tests de parity YAML y por los asserts de rutas/paginación nuevos — pre-change fallaban con 404/`has_more` inexistente).

## Deuda / Regla 6 (net ≤ 0)

- **Pago:** se elimina paginación dual (offset+cursor) → un solo esquema; se eliminan 3 rutas verbales/legacy sin versión (deuda de contrato); `has_more` exacto (probe `limit+1`) en list/threads/search/audit/skills.
- **Nueva (declarada, no bloquea):** `versions` sin paginar (fuera de contrato); dashboard TS tipos de cursor numéricos (`desktop/src/vanta.ts:137,180`, `vanta-http-map.ts:311`) mientras el wire HTTP ya es string opaco — runtime pass-through OK (el transport hace `String(v)` y el cursor nunca se aritmetiza); tightening de tipos diferido. Quirk pre-existente multi-namespace: el cursor se aplica también por-namespace (`list_records.rs`) — no es regresión; search topado por `MAX_K` → terminal silencioso; e2e sin audit (requiere audit log configurado).
- **NOTICED BUT NOT TOUCHING:** doc-comments con `/conversation/add` en `vanta-memory` (`services/conversation_hook.rs:1`, `services/mod.rs:9`, `core/conversation/l0_recorder.rs:193`, `Cargo.toml:58`, `tests/l0_capture.rs:173`, `tests/conversation_hook.rs:4,76`) — el hook usa el trait `ConversationTrigger`, no la ruta; candidato a FIND-* de docs (crate de API-08). `docs/dev/architecture/mcp-35-http-fallback-spec.md:239-244` (spec histórica con rutas viejas de maintenance) — no es doc viva.

## DoD (3 niveles)

- [x] Contrato 4/4 mecánico (parity test + parity script + greps + curl por grupo).
- [x] Docs mismo-PR (openapi.yaml + HTTP_API.md) — Regla 3.
- [x] `rustfmt --check` + `cargo clippy -p vantadb --features server --all-targets -- -D warnings` + `-p vantadb-server --all-targets` verdes.
- [x] e2e `vantadb-server` verde (17/17: rutas migradas + paginación cursor + regression limit=0).
- [x] Review P2-01 registrado (§Review, APPROVE ronda 2).
- [x] Task file sync (+ plan file + recitation vía MCP).
- [x] **Commit: hecho — `94009297` (lead, local; sin push).**

## Review (P2-01)

- **Revisor:** `vanta-review` (contexto fresco, sesión distinta al implementador; WIP API-02 excluido del alcance).
- **Enfoque:** red-team primero — contrato mecánico re-ejecutado (parity script + parity test + greps) y live-fire del binario `vanta-cli` (puerto 18123, DB temporal limpiada al cierre): status 201, paginación cursor en list/threads/skills, cursor inválido, rutas legacy 404. F1 (gemelos legacy) / F2 (`RecordInput` ↔ `MemoryInput` / MCP) / F3 (gateway) cerrados con evidencia. Hallazgo adversarial: `limit=0` en `search`/`audit` rompe la progresión del cursor.
- **Cómo se probó:**
  - `node scripts/check_openapi_parity.mjs` → `Parity OK` (37 paths; 33 bajo `/api/v2/*`; excepciones `/health`, `/metrics`, `/dashboard`, `/dashboard/{path}`).
  - `cargo test --target-dir target/session-api01 --test openapi_yaml_parity` → **10 passed / 0 failed** (re-ejecutado por el reviewer).
  - `rg "maintenance/purge|conversation/add|skill/listing" src/server/router.rs` → 0 · `rg "offset" src/server/handlers.rs` → 0 · YAML/HTTP_API sin rutas vivas viejas ni `offset`/`, Dot` (solo nota histórica "was /maintenance/purge").
  - Live-fire: records/threads/conversations 201; `list?limit=1` page1 `has_more=true` + cursor string, page2 terminal `has_more=false`; cursor inválido → **400** en list/search/threads/skills (audit sin configurar → 404; con log configurado el decode 400 corre antes, `handlers.rs:663` vs `:674`); `/conversation/add`, `/skill/listing`, `POST /api/v2/maintenance/purge` → **404**; search `limit=0` → `{"records":[],"next_cursor":"0","has_more":true}` y con `cursor:"0"` respuesta idéntica (no progresa).
  - F2: `MemoryInput` (`src/sdk/types/record.rs:59-78`) required = namespace/key/payload/metadata; `vector`/`ttl_ms` (Option, serde implícito None) y `sparse_vector` (`serde(default)`) opcionales → `RecordInput.required` del YAML coincide; `vantadb-mcp` es in-process con schema propio (no comparte rutas HTTP) — intacto.
- **Veredicto:** 🔴 **REQUEST CHANGES** (1 High + 1 Medium + 1 Low, fixes pequeños)

| # | Sev | Ubicación | Hallazgo | Acción |
|---|-----|-----------|----------|--------|
| F-A | High | `src/server/handlers.rs:544` (y `:647`, `:676`) | `search` y `audit` no claman `limit`: con `limit=0` devuelven página vacía con `has_more=true` y `next_cursor` igual a la posición actual → el cliente que sigue el cursor no progresa (loop infinito). `list` (`:428`), `threads` (`:1252`) y `skills` (`:1439`) sí están protegidos (`.max(1)` / `clamp(1,200)`). Nota: YAML de search ya dice `minimum: 1`, así que 400 también es válido como fix. | Clamp `.max(1)` (una línea por endpoint) o 400 si `limit==0`; test de regresión: `has_more=true ⇒ la próxima página no repite el cursor`. |
| F-B | Medium | `docs/api/HTTP_API.md:758` | Route Summary dice `POST /api/v2/maintenance/expired-records`; la ruta real es `DELETE` (router, YAML `delete:`, y la propia sección `:435`). | Cambiar `POST` → `DELETE` en la tabla. |
| F-C | Low | `docs/api/openapi.yaml` audit `:584`, threads `:1174`, skills `:1408` | El 400 por cursor inválido (contrato nuevo) solo está documentado en list/search; audit/threads/skills pueden devolverlo sin response declarada (live-fire ≥400 en 3 de 5; audit por código). | Añadir `"400": $ref BadRequest` a los 3 GET paginados. |

- **Lo que está bien:** migración sin gemelos (legacy 404 verificado en vivo); parity script ↔ parity test en lockstep; 6 creates 201 consistentes impl↔YAML; cursor string + `has_more` en las 5 respuestas; `page_meta` garantiza `has_more == next_cursor.is_some()` por construcción (el fallo F-A es de progresión, no de esa invariante); scope respetado — `src/sdk/**`, `src/wal.rs`, `src/vector/**`, `src/storage/**` intactos y los archivos fuera del changeset declarado son WIP API-02.
- **Observaciones (no bloquean):** (1) fan-out multi-namespace aplica el cursor también por-namespace (quirk pre-existente documentado en `list_records.rs`; no es regresión, pero la 2ª página pierde registros — cuantificar si se usa multi-ns); (2) paginación de search topada por `MAX_K` (10k) → terminal silencioso más allá; (3) comentario de `ListRecordsCommand.limit` ("already clamped") stale con `limit+1`; (4) e2e de paginación no cubre audit/search (solo list/threads/skills); (5) DoD sin tildar y recitation API-03 del plan aún en "DISCOVERY" — correcto mientras el veredicto sea 🔴.
- **Dictamen ronda 1:** 🔴 CHANGES-REQUIRED — F-A High (`limit=0` sticky en search/audit, live-fire), F-B Medium (Route Summary `POST`→`DELETE`), F-C Low (400 no documentado en audit/threads/skills).
- **Fixes aplicados (ronda 2):**
  - F-A: `search` `page_size.max(1)` (`handlers.rs`), `audit` `clamp_limit(...).max(1)` (`handlers.rs`) + regression e2e `test_e2e_search_zero_limit_cursor_progresses` + assert live en el smoke (`limit=0` → página de 1, cursor progresa).
  - F-B: tabla `HTTP_API.md` → `DELETE /api/v2/maintenance/expired-records`.
  - F-C: `400` (BadRequest) agregado a audit/threads/skills GET en `openapi.yaml`.
  - Observación (3): doc de `ListRecordsCommand.limit` actualizado (`limit + 1` probe).
- **Re-verify post-fixes:** parity 10/10 ✅ · e2e **17/17** ✅ (`test_e2e_search_zero_limit_cursor_progresses`) · clippy `-p vantadb --features server --all-targets` + `-p vantadb-server --all-targets` ✅ · smoke live 15/15 + F-A assert ✅.
- **Observaciones ronda 1 (no bloquean, deuda registrada):** (1) fan-out multi-namespace aplica el cursor también por-namespace (quirk pre-existente `list_records.rs`); (2) paginación search topada por `MAX_K` → terminal silencioso; (4) e2e sin audit (requiere audit log configurado); (5) DoD/recitation al cerrar.
- **Dictamen ronda 2 (mismo reviewer, re-ejecutado con evidencia):** ✅ **APPROVE**
  - F-A re-verificado en código (`handlers.rs:544` `.max(1)` en search; `:676` `.max(1)` en audit) y **live-fire propio** (binario `vanta-cli` con `--features server`, puerto 18124, DB temporal eliminada al cierre): 3 records → `search limit=0` → `records=1 has_more=true next="1"`; página 2 con `cursor="1"` → `records=1 next="2"` ⇒ **el cursor progresa** (ronda 1: `next="0"` fijo). `list limit=0` → clamp, 1 registro. Cursor inválido → 400; `POST /api/v2/maintenance/purge` → 404.
  - F-B: `docs/api/HTTP_API.md:758` ahora `DELETE` ✅. F-C: `"400": BadRequest` presente en audit `:623`, threads GET `:1203`, skills GET `:1454` ✅. Observación (3): doc de `ListRecordsCommand.limit` actualizado (`limit + 1` probe) ✅.
  - Checks ronda 2 (re-ejecutados por el reviewer): `node scripts/check_openapi_parity.mjs` → Parity OK · parity test **10/10** · e2e completo **17/17** (incluye `test_e2e_search_zero_limit_cursor_progresses`) · `rg` router/handlers = 0 · scope intacto (`src/sdk/**`, `src/wal.rs`, `src/vector/**`, `src/storage/**`; `vantadb-node/tests/api.test.ts` = WIP API-02).
  - **Nit opcional (no bloquea):** el regression e2e retorna temprano si `has_more != true` → con <2 hits pasaría sin ejercer el assert de progresión; endurecer con `assert!(has_more)` (el harness tiene ≥2 hits: `test_e2e_text_search_fresh_db` verifica BM25 en fresh DB). → **Aplicado:** `test_e2e_search_zero_limit_cursor_progresses` ahora asserta `has_more == true` (e2e 17/17 re-corrido ✅).
  - Deuda ronda 1 (1)(2)(4) permanece como deuda declarada, no bloqueante para esta tarea.
- **Veredicto final:** ✅ **APPROVE** (ronda 2, mismo reviewer, fixes verificados con evidencia + live-fire propio).

## Context Save Point

**API-03 COMPLETO (código+docs+tests+review ✅; commit `94009297`).** Steps 1-7 ✅. Contrato 4/4 mecánico + smoke live 15/15 + parity script OK. Cambios SOLO en working tree (sin commit/push por política owner 2026-09-25):
`src/server/router.rs`, `src/server/handlers.rs`, `src/server/pagination.rs` (nuevo), `src/server/mod.rs`, `src/server/state.rs`, `src/server/conversation.rs`, `src/server/list_records.rs`, `docs/api/openapi.yaml`, `docs/api/HTTP_API.md`, `tests/api/openapi_yaml_parity.rs`, `vantadb-server/tests/e2e.rs`, `docs/dev/tasks/API-03.md` (12 paths).
Target de sesión: `target/session-api01` (el `target/debug` queda para el MCP). Siguiente: API-04 (depende de API-03) o cierre del lead (commit + `skill progreso`). WIP ajeno (no tocar): API-02 en `vantadb-python/**`, `vantadb-ts/**`, `vantadb-node/**`, `docs/api/{NODE_SDK,TS_SDK,WASM_API}.md`, `benchmarks/batch_vs_sequential_bench.py`.
