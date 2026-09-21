# MOD-10: MCP tools — versions/supersede/vacuum/remove_edge

## Metadata
- **Plan file:** docs/plans/2026-08-25-batch-core-server-mcp.md (Task 3, Wave 2)
- **Fuente:** MOD-10 (plan file) — SDK methods sin tool MCP
- **Esfuerzo:** 🟡 2h
- **Prioridad:** 🟡
- **Tipo:** Rust (vantadb-mcp) + docs skill
- **Turns estimados:** 12
- **Creado:** 2026-08-25
- **Estado:** ✅ COMPLETED (implementación + verify; commit lo ejecuta el lead)
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 0 (5/5 steps ✅)

## Context Save Point (2026-08-25)

**Implementación completa y verificada. Pendiente: commit del lead (worker NO commitea — regla del plan).**

- Step 1-2: 4 tool definitions añadidas a `base_tools` (memory_versions + memory_supersede tras
  memory_list_namespaces; remove_edge tras graph_is_dag; vacuum tras compact_layout) + 4 arms de
  dispatch en `handle_tools_call`. Todos con validación de input en el boundary, shape MEM-32
  (error_content para errores de dominio), u128 como decimal strings en remove_edge, JSON manual
  para VacuumReport (no Serialize).
- Step 3: 5 tests nuevos en mcp_tests.rs (tools_list incluye 4 tools + round-trips versions/
  supersede/vacuum/remove_edge + validación de id inválido). Hallazgo: version snapshots dropean
  superseded_by/superseded_at_ms (`src/sdk/version_history.rs:144-145`) → supersede se verifica
  vía memory_get, no memory_versions (comportamiento core, no bug).
- Step 4: docs SKILL.md + api-reference.md actualizadas en ambos trees. Hash SAME ✅ (SKILL.md y
  api-reference.md idénticos, 0 mismatches tree-wide). Contadores normalizados al estado real del
  código: **72 tools (42 core + 30)**, que incluye las 2 tools de MCP-24 (search_with_method/
  search_multi) ya presentes en el worktree compartido.
- Step 5: verify full ✅ — `cargo test -p vantadb-mcp --test mcp_tests` **68/68** (incluye 5 tests
  MOD-10 + tests MCP-24) · `cargo check -p vantadb-mcp` ✅ · `cargo fmt --check` ✅ ·
  `cargo clippy -p vantadb-mcp --all-targets -- -D warnings` ✅.
- ⚠️ **Colaboración concurrente:** MCP-24 (tarea paralela Wave 2) edita los MISMOS archivos
  (tools.rs, mcp_tests.rs, SKILL.md) en el mismo worktree. El diff de tools.rs mezcla cambios de
  ambos. El lead debe mergear/commitear los 6 archivos combinados: tools.rs, mcp_tests.rs,
  SKILL.md ×2, api-reference.md ×2 + los task files MOD-10.md y MCP-24.md.
- ⚠️ WIP server: `campaign_update_task_state in-progress` bloqueado (MOD-13 en progreso por
  sub-agente paralelo). El lead debe resolver WIP al cerrar.

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `handle_tools_call` (tools.rs) es invocado por `server.rs` (dispatch JSON-RPC `tools/call`); `handle_tools_list` por `server.rs` (`tools/list`). Test `vantadb-mcp/tests/mcp_tests.rs` usa `handle_tools_call` directo. |
| Callees | `VantaEmbedded::versions/supersede/vacuum/remove_edge` (`src/sdk/api.rs`), helpers `validation.rs`, `vantadb::storage::engine::VacuumReport` |
| Implicaciones | Solo aditivo: 4 tools nuevas en `base_tools` + 4 arms de dispatch. No cambia ninguna tool existente ni la API del core. Sin semver. Solo toca `vantadb-mcp/` (handlers/tools.rs, tests) + docs skill. No toca `wal.rs`/`vector/`/`storage/` (Arch/Engine). Blast radius bajo y aislado. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):**
  - `vantadb-mcp/src/handlers/tools.rs` (1772 L, completo — base_tools L24-397, dispatch L429-1651, helpers al final)
  - `vantadb-mcp/src/validation.rs` (537 L — validate_identifier L11, parse_node_id L310, serialize_content L339, text_content L346, error_content L363, for_each_record L372)
  - `src/sdk/api.rs` (solo los 4 métodos: versions :450-474, supersede :836-899, vacuum :77-84, remove_edge :1247-1269)
  - `src/storage/engine/mod.rs` (VacuumReport :196-208 — NO Serialize)
  - `src/sdk/version_history.rs` (SnapshotRecord :130-148 — dropea supersession fields)
  - `skills/vantadb-mcp/SKILL.md` + `references/api-reference.md` (ambos trees) — sección de tools
  - `vantadb-mcp/tests/mcp_tests.rs` (patrón de tests: tools_list L278, maintenance_call L2621, recovery_call L3412, build_chain L3175)
- **Referencias hacia dentro (imports/deps):** tools.rs ya importa `crate::validation::*` y `serde_json::{json, Value}` — no requiere imports nuevos. `VacuumReport` es accesible vía `vantadb::storage::engine` (ya se usa `vantadb::storage::StorageEngine` en tools.rs).
- **Referencias entrantes:** ninguna otra herramienta/tool llama a los 4 métodos SDK vía MCP hoy (gap). El core SDK (`api.rs`) no cambia — solo se consume.
- **Veredicto impacto:** bajo, aditivo. Solo agrega tools a `vantadb-mcp` + tests + docs. No rompe contratos existentes.

## Contrato
"`cargo test -p vantadb-mcp --test mcp_tests` pasa; 4 tools nuevas round-trip; docs skill ×2 hash SAME"

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** shape de retorno de las tools existentes (`text_content(serialize_content(...))` para datos, `error_content(...)` para errores de dominio — MEM-32, nunca propagar JSON-RPC error para fallos de dominio). u128 ids SIEMPRE como decimal strings. `VacuumReport` serializado manualmente (no es Serialize). Docs SKILL.md y api-reference.md idénticos entre `skills/` y `.opencode/skills/` (hash SAME).
- **Comandos de verificación:** `cargo test -p vantadb-mcp --test mcp_tests` (el binario de test se llama `mcp_tests`; `cargo nextest` lo excluye del default-filter — usar el comando exacto); `cargo check -p vantadb-mcp`; `cargo fmt --check`; `cargo clippy -p vantadb-mcp --all-targets -- -D warnings`.
- **Deuda pendiente:** ninguna.

## Deuda técnica (Regla 6)

**Saldo neto:** sin deuda — 4 wrappers finos, sin unsafe, sin lógica duplicada (la lógica vive en el core SDK; tools.rs es thin wrapper). No introduce deuda.

## Definition of Done

| Nivel | Gate |
|-------|------|
| Task | `cargo test -p vantadb-mcp --test mcp_tests` pasa (4 tools round-trip) + check + fmt + clippy |
| Commit | Lo ejecuta el lead (worker NO commitea). Diff: tools.rs + mcp_tests.rs + SKILL.md ×2 + api-reference.md ×2 |
| Release | N/A (aditivo, sin release) |

## Fases explícitas — SECURITY | PERFORMANCE

- [x] **SECURITY** — `remove_edge` y `supersede` son mutadores con input de usuario (trust boundary):
  `remove_edge` valida ids vía `parse_node_id` (rechaza no-u128) y label vía `validate_identifier`
  (no vacío / no NUL / max len); `supersede` valida namespace/old_key/new_key vía `validate_identifier`.
  Los errores de dominio (key/edge inexistente) se devuelven como `error_content` self-correctable
  (MEM-32). Sin path traversal (no hay rutas), sin auth/sesiones, sin dependencias nuevas → no requiere
  `cargo audit`. Patrón idéntico a las tools de mutación existentes (`memory_put`, `memory_delete`,
  `collection_delete`).
- [x] **PERFORMANCE** — no toca hot paths del core (solo wrappers thin sobre API existente; `vacuum`
  es mantenimiento). No requiere benchmark (Regla 9 no aplica — no es optimización).

## Steps

### Step 1: tools.rs — añadir 4 definiciones a base_tools
- **Archivos:** `vantadb-mcp/src/handlers/tools.rs` (base_tools)
- **Acción:** añadir `memory_versions` + `memory_supersede` tras `memory_list_namespaces`;
  `remove_edge` tras `graph_is_dag`; `vacuum` tras `compact_layout`. Schemas JSON-RPC siguiendo el
  patrón de las tools existentes (required correctos).
- **Verify:** `cargo check -p vantadb-mcp` ✅
- **Estado:** ✅

### Step 2: tools.rs — añadir 4 arms de dispatch en handle_tools_call
- **Archivos:** `vantadb-mcp/src/handlers/tools.rs` (handle_tools_call)
- **Acción:** arms `memory_versions`/`memory_supersede` tras `memory_list_namespaces`; `vacuum` tras
  `compact_layout`; `remove_edge` tras `graph_is_dag`. Validación de input (validate_identifier /
  parse_node_id), delegación al SDK, shape de retorno definido en Context Save Point.
- **Verify:** `cargo check -p vantadb-mcp` ✅
- **Estado:** ✅

### Step 3: mcp_tests.rs — tests round-trip de las 4 tools
- **Archivos:** `vantadb-mcp/tests/mcp_tests.rs`
- **Acción:** 5 tests: tools_list incluye 4 tools; versions round-trip (2 puts → v1,v2, missing → []);
  supersede round-trip (verificado vía memory_get — snapshots dropean supersession); vacuum report
  shape; remove_edge round-trip (build_chain → remove → neighbors gone ambas direcciones + id inválido).
- **Verify:** `cargo test -p vantadb-mcp --test mcp_tests` ✅ 68/68
- **Estado:** ✅

### Step 4: docs skill — SKILL.md + api-reference.md en ambos trees (hash SAME)
- **Archivos:** `skills/vantadb-mcp/SKILL.md`, `.opencode/skills/vantadb-mcp/SKILL.md`,
  `skills/vantadb-mcp/references/api-reference.md`, `.opencode/skills/vantadb-mcp/references/api-reference.md`
- **Acción:** documentar las 4 tools nuevas (Memory CRUD: memory_versions + memory_supersede; Graph:
  remove_edge; Maintenance: vacuum) + tabla api-reference.md. Contadores normalizados al estado real
  del código combinado: 72 tools / 42 core (incluye MCP-24). Copia al tree `.opencode` y hash verificado.
- **Verify:** hash SAME ✅ + 0 mismatches tree-wide
- **Estado:** ✅

### Step 5: verify full
- **Archivos:** —
- **Acción:** correr verificación completa del contrato.
- **Verify:** `cargo test -p vantadb-mcp --test mcp_tests` 68/68 ✅ · `cargo check -p vantadb-mcp` ✅ ·
  `cargo fmt --check` ✅ · `cargo clippy -p vantadb-mcp --all-targets -- -D warnings` ✅
- **Estado:** ✅

## Dependencias
- Ninguna. Colaboración concurrente con MCP-24 (mismos archivos, worktree compartido) — el lead
  mergea el diff combinado.

## Review (GATE — agente distinto, P2-01)
- **Revisor:** vanta-review (verificar luego de implementar; gate de cierre).
- **Enfoque:** wrappers thin, shape de retorno correcto (vacuum manual JSON, versions records, u128
  strings en remove_edge), validación de input en mutadores, docs hash SAME.
- **Cómo se probó:** cargo test + check + fmt + clippy mecánico.
- **Veredicto:** pendiente.

## Notas
- `cargo nextest` excluye el binary `mcp_tests` del default-filter → el comando de test correcto es
  `cargo test -p vantadb-mcp --test mcp_tests` (contrato del plan).
- Hallazgo core (no bug): version snapshots (`SnapshotRecord`) dropean `superseded_by`/
  `superseded_at_ms` (`src/sdk/version_history.rs:144-145`) — `memory_versions` siempre muestra esos
  campos null. La supersession se lee del record vivo vía `memory_get`. Documentado en SKILL.md.
- MCP-24 (tarea paralela) ya había actualizado los contadores del header de SKILL.md a 68/38; el
  estado real del código combinado es 72/42 (38 + mis 4). Normalizado a 72/42.

