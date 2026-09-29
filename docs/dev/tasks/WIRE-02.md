---
title: "WIRE-02: MCP — enforce de perfil en `tools/call` + default `agent` + fusión de superficie + fix doc"
kind: task
description: "tools/call de tool fuera de perfil → error real Tool not found: <name> (not in profile <profile>) (test) Y default agent con smoke tools/list ≤45 Y superficie fusionada con conteo real documentado en MCP.md (tabla + error, consistente..."
---

# WIRE-02: MCP — enforce de perfil en `tools/call` + default `agent` + fusión de superficie + fix doc

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 16, F2 — Wave WIRE-02 ‖ WIRE-03 ‖ WIRE-04)
- **Fuente:** Backlog fila `WIRE-02` (P56) + informe `VantaDB-Informe-Analisis-Completo.md:197-203,368,428` (A3, M4)
- **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🟠 · **Tipo:** Feature (`feat(mcp):`)
- **Turns estimados:** 20-30
- **Creado:** 2026-09-27 · **last-synced:** 2026-09-27
- **Estado:** ⏳ IN PROGRESS — implementación + verify de área ✅ (2026-09-27; ronda de fixes del review P2-01 aplicada; suites **239/239** = 136 default + 103 `mcp_tests` vía `--ignore-default-filter`); re-review + commit → LEAD
- **SDP (Paso 0b, SDP v3 BUILD):** pinned `documentation-and-adrs` + `api-and-interface-design` + `test-driven-development` + `systematic-debugging`; base `source-driven-development` + `security-and-hardening`; (campaign-executor/progreso auto)
- **Dep:** API-04 ✅ (aliases canonicalizados, `MCP.md:536`) · **nextTask:** WIRE-03
- **Incógnitas (uphill):** 0 · **Pendientes (downhill):** 5 steps

## Contrato (verbatim del plan Task 16)
"`tools/call` de tool fuera de perfil → error real `Tool not found: <name> (not in profile <profile>)` (test) Y default `agent` con smoke `tools/list` ≤45 Y superficie fusionada con conteo real documentado en `MCP.md` (tabla + error, consistente con `handle_tools_list` y sus meta-tests) Y aliases API-04 siguen dispatchables"

## Blast Radius (verificado con codegraph + greps, 2026-09-27)

| Dirección | Módulos |
|-----------|---------|
| Callers | `handle_tools_call` ← `server.rs:141,301,927` (`McpConfig::from_storage`), `dispatch_request`/stdio; 30+ callers en tests del crate + `vantadb-server/tests/mcp_integration.rs` |
| Callees | `profile_allowed_tools` (1 caller hoy: `handle_tools_list:1010`), `McpError::method_not_found` (-32601), `code::handle_code_tool`, alias arms (`search_memory:1662`, `collection_list:2128`) |
| Implicaciones | `McpConfig` 75 callers (codegraph): test files usan `McpConfig::default()` → con default `agent` los tests de familias extendidas (code/skill/wiki/dream/scene/context/thread) deben pinar `Full` explícito. Script `validate-docs-coverage.ps1` §6 (tool↔MCP.md) solo valida nombres base del bloque `handle_tools_list` (no conteos) — fusionar exige mantener cada nombre listado documentado |
| WIP ajeno PROHIBIDO | `setup-embeddings.ps1`, `src/cli_handlers/server.rs`, `src/lib.rs`, `src/embedding_health.rs`, `tests/embedding*` (slice activa); `src/config.rs`, `src/gc.rs`, `src/sdk/**`, `src/server/**` (WIRE-04); bindings Py/TS/Node (WIRE-03); `perf-bench.yml` |

## Impacto mapeado (Regla 0)
- **Archivos leídos (regiones completas de edición + entorno):** `vantadb-mcp/src/handlers/tools.rs` (header 1-76, listing 80-1020, `profile_allowed_tools` 1023-1201, `handle_tools_call` head 1204-1259 + dispatch tail 2940-3019), `vantadb-mcp/src/config.rs` (1-142 completo), `vantadb-mcp/src/error.rs` (1-146 completo), `vantadb-mcp/src/code.rs` (header 1-150: mapa tool↔primitiva), `vantadb-mcp/tests/mcp_tests.rs` (helpers 1-30, meta-tests 4490-4780, conteos), `docs/api/MCP.md` (170-300, 300-490, 490-560), informe §3.6 (:191-203, :368, :428), `docs/dev/tasks/WIRE-10.md` (formato)
- **Referencias hacia dentro:** `McpProfile` export (`lib.rs:33`); `LEGACY_SEARCH_ALIAS`/`LEGACY_COLLECTION_ALIAS` (mcp_tests.rs:14-15); meta-tests de conteos (mcp_tests.rs:4667, :4557); `validate-docs-coverage.ps1:168-183`
- **Referencias entrantes:** `docs/api/MCP.md` tabla perfiles (:229-233) + error (:265) + families (:198) + parity (:536); `config.rs:8-11` (doc perfiles)
- **Veredicto impacto:** medio-alto — 1 gate nuevo en hot path de dispatch (O(1) HashSet), 1 variante de enum público (`McpProfile::Agent`), fusión de listing no-breaking (precedente API-04), colateral en ~10 archivos de test del crate (pin de perfil)

## Spec (decisiones — lógica nueva con símbolo público nuevo: `McpProfile::Agent` + gate)

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Mecanismo de enforce | A) Gate en `handle_tools_call` sobre allowlist del perfil (O(1), simétrico con listing) / B) filtrar en `server.rs` (no cubre librería/tests) | ✅ A (simetría listing↔dispatch; server.rs fuera de scope) |
| 2 | Aliases + proyecciones absorbidas | A) resolver alias→canónico y permitir si el canónico está listado (API-04 precedent) / B) bloquear todo lo no listado (rompe contrato "aliases siguen dispatchables") | ✅ A |
| 3 | Default `agent` | A) `#[default]` en variante nueva + `McpConfig::default()` la usa (honesto; exige pin `Full` en tests extendidos) / B) solo `from_storage` (dos defaults, incoherente) | ✅ A |
| 4 | Fusión | A) consolidación de LISTING (precedente API-04): absorber proyecciones redundantes como dispatch-only, handlers intactos, sin breaking de dispatch / B) fusionar handlers/schemas (breaking, أfuera de appetite) | ✅ A — informe §3.6: `code_*`×8 aportan ~2 primitivas reales, triple solape introspección |
| 5 | Alcance de la fusión | Absorber 6 `code_*` (callers/callees/impact/node/files/status) → full 85→79 listadas. Delta a ~65 requiere tools nuevos fusionados (breaking) → Stop condition: diferir y anotar en Backlog | ✅ A + Backlog |
| 6 | Política `full` post-change | `full` queda opt-in (`VANTADB_MCP_PROFILE=full`) documentado + notas de migración (pre-mortem F1) | ✅ |

## Invariantes de dominio (handoff — MUST)
- **No romper dispatch**: toda tool listada en `handle_tools_list` debe seguir despachando en su perfil; aliases API-04 (`search_memory`, `collection_list`) siguen `tools/call`-ables (contrato).
- **Errores**: nombres desconocidos conservan el fall-through pelado `Tool not found: {name}`; solo los conocidos-fuera-de-perfil agregan `(not in profile <profile>)` (documentado en `ERROR_HANDLING`/MCP.md).
- **Conteos**: MCP.md ↔ `handle_tools_list` ↔ meta-tests deben coincidir exactamente (full 79 / dev 36 / memory 20 / agent 37).
- **Comandos de verificación:** `cargo nextest run --profile audit -p vantadb-mcp` + `cargo clippy -p vantadb-mcp --all-targets -- -D warnings` + `cargo fmt --check` + `pwsh scripts/validate-docs-coverage.ps1`.
- **Deuda pendiente:** ninguna al abrir.

## Recitation
```
=== RECITATION ===
Objetivo activo: WIRE-02 — enforce perfil MCP + default agent + fusión superficie + doc
Estado: in-progress (implementación + verify de área ✅; ACCEPT/commit → LEAD)
Última acción: Steps 1-5 ✅ + ronda de fixes CRITICAL/REQUIRED/NIT del review P2-01 aplicada (13 sitios de `mcp_tests` pinnados a `default_config()`, meta-test API-04 a 79, docs `config.rs` Dev/Memory); verify: `nextest -p vantadb-mcp` **136/136** (default-filter) + `--ignore-default-filter -E 'binary(mcp_tests)'` **103/103** = **239/239** + 4 tests de contrato + fmt/clippy + docs-coverage 0 gaps + server `mcp_integration` 1/1; FIND-181 (delta ~65 diferido)
Resultado: OK
Próxima acción: LEAD — review fresco (P2-01) + commit local `feat(mcp): profile enforcement + agent default + surface fusion (WIRE-02)`; sin push
Contrato: ver ## Contrato — verificado (enforce + default + fusión + aliases) ✅
Invariantes: dispatch de listadas intacto; aliases/absorbidas dispatchables vía canónico; conteos 79/36/20/37 consistentes doc↔código↔meta-tests
Deuda: FIND-181 (fusión ~65 con esquemas nuevos → Gate D)
Próxima tarea si completa: WIRE-03
last-synced: 2026-09-27
=== END RECITATION ===
```

## Definition of Done (3 niveles)
- **Task:** contrato ✅ + meta-tests verdes + `validate-docs-coverage` exit 0
- **Commit:** ⏳ LEAD (local, `feat(mcp): profile enforcement + agent default + surface fusion (WIRE-02)`)
- **Release:** notas de migración: default `agent` (breaking de superficie) + `VANTADB_MCP_PROFILE=full` para compatibilidad total

## Steps

### Step 1: Enforce del perfil en `tools/call` (RED→GREEN)
- **Archivos:** `vantadb-mcp/src/handlers/tools.rs` (gate + `absorbed_canonical` + `profile_allows_call`), `vantadb-mcp/tests/mcp_tests.rs` (test enforce)
- **Acción:** gate tras parsear `name`: conocido-fuera-de-perfil → `method_not_found` exacto `Tool not found: <name> (not in profile <profile>)`; alias→canónico permitido si canónico listado
- **Verify:** test RED (message mismatch) → GREEN; `cargo nextest -p vantadb-mcp --test mcp_tests enforce`
- **Estado:** ✅ DONE (RED pre-change: sin gate — `profile_allowed_tools` solo lo consumía el listing (codegraph) y `code_search` despachaba bajo `memory`; GREEN: gate en `handle_tools_call` + `McpProfile::as_str` + `absorbed_canonical`/`profile_allows_call`/`tool_is_known`; `test_mcp_profile_enforcement_dispatch` verde: error exacto memory/agent, fall-through pelado para desconocido, alias dispatchable, absorbed dispatchable bajo full)

### Step 2: Perfil `agent` + default (RED→GREEN) + colateral tests
- **Archivos:** `vantadb-mcp/src/config.rs` (`Agent` + `#[default]` + `as_str` + docs), `vantadb-mcp/src/handlers/tools.rs` (allowlist agent 37), tests del crate (pin `Full` en familias extendidas)
- **Acción:** `agent` = memory(20) + threads(6) + scenes(5) + context(1) + wiki-read(5) = 37; default `agent`; `full` opt-in
- **Verify:** smoke `tools/list` = 37 ≤45 por `McpConfig::default()`; suite crate verde
- **Estado:** ✅ DONE (RED pre-change: `McpConfig::default()` era `Full` (meta-test :4667 = 85) y `McpProfile::Agent` no existía; GREEN: variante `Agent` `#[default]` + allowlist 37; smoke `test_mcp_agent_default_surface` verde (37 ≤45, includes/excludes); colateral: 11 test files pinan `Full` — helper `full_config()` en code/skills/wiki×4/dream/scene×2/context/thread + `default_config()`→Full en mcp_tests + `vantadb-server/tests/mcp_integration.rs`; suite: `nextest -p vantadb-mcp` **136/136** (default-filter, excluye mcp_tests) + `--ignore-default-filter -E 'binary(mcp_tests)'` **103/103** — ⚠️ `mcp_tests` está excluido del default-filter (`.config/nextest.toml:62`) ✅)

### Step 3: Fusión de superficie (consolidación de listing, no-breaking)
- **Archivos:** `vantadb-mcp/src/handlers/tools.rs` (allowlist Full −6 code_*), `vantadb-mcp/tests/mcp_tests.rs` (conteos + absorbed)
- **Acción:** absorber como dispatch-only: `code_callers`/`code_callees`/`code_impact`/`code_node` → `code_explore`; `code_status` → `capabilities`; `code_files` → `code_search` (stub "not supported"). Full 85→79 listadas. Delta a ~65 (stop condition) → Backlog
- **Verify:** `test_mcp_tool_profiles` full=79 + absorbed nunca listadas + dispatchables bajo full
- **Estado:** ✅ DONE (GREEN: full=79 / dev=36 / memory=20 / agent=37; absorbed nunca listadas en ningún perfil + dispatchables bajo full (test); `test_all_eight_code_tools_listed` → `test_code_tools_listed_vs_absorbed`; annotations coverage 79)

### Step 4: Doc consistente (`MCP.md` + comentarios stale)
- **Archivos:** `docs/api/MCP.md` (families :198, tabla perfiles :229, error :265, aliases/absorbidas :214, extended :396, parity :536), `vantadb-mcp/src/config.rs` + `tools.rs` (comentarios "87"/"85")
- **Acción:** conteos reales, tabla agent default, error real, tabla de absorbidas, notas de migración; `validate-docs-coverage` se co-rrige por nombre
- **Verify:** `pwsh scripts/validate-docs-coverage.ps1` exit 0
- **Estado:** ✅ DONE (`validate-docs-coverage.ps1` exit 0 — "vantadb-mcp (tools) - 47 items ok en MCP.md"; MCP.md: families 79 listadas, tabla perfiles con `agent` default + `full` opt-in + migración, §Legacy & absorbed (8 filas), extended 38→32 listadas, parity last-sync 2026-09-27)

### Step 5: Verify full + cierre
- **Archivos:** task file, plan file, Backlog (delta fusión deferred)
- **Acción:** fmt + clippy + nextest crate + docs-coverage; sync plan/task/Backlog; recitation
- **Verify:** `campaign_verify_cmd` con contrato completo
- **Estado:** ✅ DONE (fmt ✅ / clippy `-p vantadb-mcp --all-targets -- -D warnings` ✅ / `nextest --profile audit -p vantadb-mcp` **136/136** + `... --ignore-default-filter -E 'binary(mcp_tests)'` **103/103** (= **239/239**) ✅ / 4 tests de contrato ✅ / docs-coverage 0 gaps ✅ / server `mcp_integration` 1 passed — `CARGO_TARGET_DIR=target/session-wire02` por lock FIND-177; re-review + commit → LEAD)

## Dependencias
- API-04 ✅ (aliases canónicos). Wave F2: WIRE-03 ‖ WIRE-04 (archivos disjuntos). Siguiente: WIRE-03.

## Save Point (reanudación)
- Implementación terminada 2026-09-27; verify de área ✅ (nextest 136/136, fmt, clippy, docs-coverage). Pendiente único: review fresco + commit (LEAD). Artefactos: task file + plan Task 16 (Estado/Notas) + `FIND-181` (Backlog, delta fusión).

## Review (GATE P2-01) — ronda 1 aplicada; re-review LEAD
- Revisor: LEAD (vanta-review) — contexto distinto al implementador (sesión worker WIRE-02, `ses_f1abdb5f5ffel78p3rmiPEA2tr`).
- **Ronda 1 (review fresco): ❌ CHANGES REQUIRED — 1 Critical + 1 Required + 1 Nit, aplicados 2026-09-27:**
  - **Critical:** `mcp_tests` está excluido del default-filter (`.config/nextest.toml:62`) → la suite "136/136" no lo ejecutaba; 6 tests rojos por `McpConfig::default()` crudo (ahora `agent`). Fix: 13 sitios pinnados a `default_config()` (Full) — :122/:147/:343/:1936/:2993/:3695/:4051/:4333/:4533/:5378/:5424/:5520/:5760 — + meta-test API-04 `test_api04_tools_list_canonical_names_no_duplicates` conteo 85→79 (comentario :5372-5375). Repro/verificación: `--ignore-default-filter -E 'binary(mcp_tests)'` → **103/103**.
  - **Required:** evidencia del task file con ambos comandos (136 default + 103 ignore-filter = 239/239) — corregido en Metadata/Recitation/Steps 2/5.
  - **Nit:** `config.rs` docs de variantes stale (`Dev (~35)`→36, `Memory (~18)`→20) — aplicado.
- Evidencia para el re-review: `test_mcp_profile_enforcement_dispatch` + `test_mcp_agent_default_surface` + `test_mcp_tool_profiles` + `test_mcp_tool_annotations_coverage` (4/4 ✅) · 239/239 · clippy/fmt · `validate-docs-coverage` 0 gaps · server `mcp_integration` 1/1.

## Notas
- Fuente de la fusión: informe §3.6 (:199, :203, :428) + `code.rs:8-20` (mapa tool↔primitiva: callers/callees/explore comparten BFS; files = stub not-supported; status = `operational_metrics()` ≈ `capabilities`).
- Stop condition del plan aplicada parcialmente: la fusión no-breaking cierra en 79 (no ~65); el delta restante exige tools fusionadas con esquemas nuevos (nuevo símbolo público → Gate D) → diferido a Backlog.
- Regla dura `-p`: todos los verifies con `-p vantadb-mcp`; `CARGO_BUILD_JOBS=2` (presión de memoria).
