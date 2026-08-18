# VS-CORE-02: Contadores por namespace + stats TTL en el core SDK

## Metadata
- **Plan file:** `docs/plans/2026-08-18-vanta-studio-fase0.md` (Task 14)
- **Fuente:** plan file Task 14 (gap §8.3) — bloqueante de VS-04 (HOME) y sidebar
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🔴 (bloqueante de VS-04)
- **Tipo:** Rust (core SDK)
- **Turns estimados:** 7
- **Creado:** 2026-08-18T12:55
- **last-synced:** 2026-08-18T12:55
- **Estado:** ⏳ IN PROGRESS
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 0 steps (implementación completa; falta REVIEW gate + commit por lead)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `desktop/src-tauri` (bridge Tauri — FUTURO, VS-04; esta tarea NO lo toca), tests core, docs |
| Callees | `src/sdk/api.rs` (engine_handle, scan_nodes, memory_record_from_node, now_ms), `src/sdk/types.rs` (tipos públicos), `src/backend.rs` (BackendPartition — solo lectura) |
| Implicaciones | Aditivo 100% semver-safe: NINGÚN método existente se toca. Sin migración de datos, sin re-indexación, sin cambios de features, sin bindings |

## Impacto mapeado (Regla 0)

> GATE ANTES DE CUALQUIER EDICIÓN (MUST — AGENTS.md Regla 0).

- **Archivos leídos (completos):**
  - `src/sdk/api.rs` — leídos vía codegraph (verbatim): imports (1-16), `put` (206), `put_record_exact` (476-510), `list_namespaces` (512-536), `list` (538-656), `count` (1278-1303), helpers de tests (1484-1496, 1706-1719). El resto del archivo (1700+ líneas) se lee si el step lo requiere.
  - `src/sdk/types.rs` — leídos vía codegraph (verbatim): imports (4-6), `VantaMemoryFilter` (126-127), `VantaMemoryInput` (129-171), `VantaMemoryRecord` (173-201), `VantaMemoryListOptions`/`VantaMemoryListPage` (203-241).
  - `tests/memory_api.rs` — leído completo (1-130; resto relevante 431).
  - `docs/api/EMBEDDED_SDK.md` — leído (40-159): tabla Memory API + tipos.
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `api.rs` → `super::types::*` (incluye el tipo nuevo automáticamente), `super::serialization::{memory_record_from_node, now_ms, ...}`, `crate::error`, `crate::backend`. `types.rs` → `serde`, `std::collections::{BTreeMap, BTreeSet}`, `crate::node::SparseVector`.
- **Archivos que referencian a los editados (referencias entrantes):** `VantaEmbedded` tiene 49 callers (bindings Python/WASM/TS/Node, server, CLI, tests) — NINGUNO se rompe porque el cambio es aditivo. `VantaMemoryRecord` es serializado por bindings — no se modifica.
- **Veredicto impacto:** BAJO. Dos archivos core + 1 test de integración + 1 doc. Solo se AGREGAN símbolos nuevos (`VantaNamespaceStats`, `VantaNamespaceStatsMap`, `DEFAULT_EXPIRING_SOON_WINDOW_MS`, `namespace_stats`). Sin regresión posible en API existente.

## Contrato

"`cargo nextest run --profile audit --workspace --build-jobs 2` pasa Y el método `namespace_stats(&self, expiring_soon_window_ms: Option<u64>) -> Result<VantaNamespaceStatsMap>` devuelve `{namespace: {count, expiring_soon, expired}}` con conteos exactos verificados por tests, implementado con UNA sola pasada de scan (sin N llamadas paginadas de `count`/`list`)."

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) NO modificar métodos existentes (`count`, `list`, `list_namespaces`, `put`); cambio estrictamente aditivo. (2) NO tocar `desktop/` ni bindings (Python/WASM/TS) en esta tarea — la exponen en VS-04/VS-05. (3) Respetar `.opencode/rules/api-contract.md` (R-7: sin campos pub gateados por cfg — no aplica; aditivo semver-safe). (4) Sin `unwrap()`/`expect()` en código nuevo.
- **Comandos de verificación:** `cargo check -p vantadb` ✅; `cargo nextest run --profile audit --workspace --build-jobs 2` ✅.
- **Deuda pendiente:** ninguna (se cierra completa). Bindings expondrán `namespace_stats` en tareas posteriores (VS-04/VS-05).

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | VS-CORE-02: Contadores por namespace + stats TTL |
| `lastAction` | Implementación completa: Steps 1-5 + verify (fmt/clippy/nextest). 1929/1930 tests pasan; 1 fallo PRE-EXISTENTE en `src/storage/` (`test_consolidate_node_with_binary_vector`), confirmado también en worktree HEAD limpio — fuera de scope (Arch/Engine) |
| `result` | ✅ 5/5 steps implementados y verificados. Contrato de la feature cumplido (namespace_stats single-pass con conteos exactos); verify full del contrato NO 100% por fallo pre-existente ajeno |
| `nextAction` | REVIEW gate (vanta-review, contexto fresco, P2-01) + commit por lead + escalar fallo pre-existente a Arch/Engine |
| `contract` | Ver `## Contrato` arriba |
| `nextTask` | VS-CORE-01 (cursor en bridge desktop) o VS-04 (consume este) |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda — no se introduce deuda nueva (aditivo, una pasada O(n) equivalente al costo de `count()` pero en UNA llamada).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable se cumple (tests de namespace_stats + full nextest) + fmt/clippy clean |
| **Commit** | Commit atómico conventional `feat: VS-CORE-02 — ...`, solo archivos de esta tarea |
| **Release** | No aplica (sin release; verify.ps1 lo cubre el lead en push) |

## Herramientas necesarias
- Terminal: `cargo check -p vantadb`, `cargo nextest`, `cargo fmt`, `cargo clippy`
- codegraph_explore (blast radius — ya usado)
- MCP campaign: `campaign_verify_cmd`, `campaign_update_task_state`, `campaign_validate_command`

## Investigation Notes
- **Diseño (aditivo, single-pass):** el contrato prohíbe "N llamadas paginadas". `count()` (api.rs:1278) pagina `list` con PAGE_SIZE 1000 → llamarlo por namespace = N paginaciones. Solución: UNA pasada de `engine.scan_nodes()` + `memory_record_from_node` (mismo patrón que el fallback de `list_namespaces` api.rs:520-524). Clasificación TTL: `expired` si `expires_at_ms <= now`; `expiring_soon` si `now < expires_at_ms <= now + window` (if/else-if — un registro vencido NO cuenta como expiring_soon). Ventana default 24h (`DEFAULT_EXPIRING_SOON_WINDOW_MS`), parametrizable `Option<u64>` para VS-04 (p.ej. "próximos 7 días").
- **Naming:** `VantaNamespaceStats` (struct) + `VantaNamespaceStatsMap` (type alias `BTreeMap<String, VantaNamespaceStats>`), consistentes con la familia `VantaMemory*`. Method `namespace_stats` junto a `count`/`list_namespaces`.
- **Tests deterministas sin sleep:** casos "expired" vía `put_record_exact` (pub(crate), accesible desde `mod tests` de api.rs) con `expires_at_ms` explícito en el pasado. Casos expiring via `VantaMemoryInput.ttl_ms` relativo. Límites de ventana con `Some(window)` explícito.
- **SECURITY:** no aplica — sin input de usuario nuevo (Option<u64> interno validado por tipo), sin dependencias, sin FFI, sin persistencia nueva. Justificado.
- **PERFORMANCE:** no aplica hot-path — consulta de overview (una pasada), NO está en loop de search/ingestión; sin benchmark (Regla 9 aplica a optimizaciones, no features). Justificado.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 (5/5 steps ✅ — solo REVIEW gate + commit por lead) |
| % completado | 100% |

## Steps

### Step 1: Tipos públicos en types.rs
- **Archivos:** `src/sdk/types.rs` (+ re-export en `src/sdk/mod.rs` y `src/lib.rs`)
- **Acción:** agregar `VantaNamespaceStats` (struct derive Debug/Clone/Default/PartialEq/Eq/Serialize/Deserialize, campos `count`, `expiring_soon`, `expired` — u64), `VantaNamespaceStatsMap` (type alias `BTreeMap<String, VantaNamespaceStats>`), y `DEFAULT_EXPIRING_SOON_WINDOW_MS: u64 = 24 * 60 * 60 * 1000` con doc comments. Insertar tras `VantaMemoryListPage` (≈ línea 240), antes del `pub use`. Re-export en `sdk/mod.rs` y `lib.rs` (types es pub(crate) → sin re-export el tipo no es alcanzable).
- **Verify:** `cargo check -p vantadb`
- **Estado:** ✅ (2026-08-18 — cargo check -p vantadb exit 0)

### Step 2: Método namespace_stats en api.rs
- **Archivos:** `src/sdk/api.rs`
- **Acción:** implementar `pub fn namespace_stats(&self, expiring_soon_window_ms: Option<u64>) -> Result<VantaNamespaceStatsMap>` después de `count` (:1303). Single pass: `engine.scan_nodes()`, `memory_record_from_node`, `now = now_ms()`, `window = expiring_soon_window_ms.unwrap_or(DEFAULT_EXPIRING_SOON_WINDOW_MS)`, clasificación if/else-if TTL, `now.saturating_add(window)`. `#[tracing::instrument(skip(self), err)]` + doc con ejemplo.
- **Verify:** `cargo check -p vantadb` && `cargo clippy -p vantadb --all-targets -- -D warnings`
- **Estado:** ✅ (2026-08-18 — cargo check exit 0; clippy -D warnings exit 0 en verify final)
- **Nota:** helper `memory_record_from_node_include_expired`/`memory_record_from_node_inner` agregado en `src/sdk/serialization/mod.rs` (pub(crate), aditivo — `memory_record_from_node` original intacto) para que el scan observe registros expirados no purgados.

### Step 3: Unit tests deterministas en api.rs
- **Archivos:** `src/sdk/api.rs` (mod tests)
- **Acción:** tests: (1) db vacía → mapa vacío; (2) multi-namespace mixto (normal / expiring soon via ttl / expired via `put_record_exact` con expires_at pasado) → counts exactos; (3) boundaries de ventana (`expires_at == now+window` → expiring_soon; `== now+window+1` → no); (4) cross-check `namespace_stats().count == count(ns, None)`. Helper local `memory_record(ns, key, expires_at_ms)`.
- **Verify:** `cargo nextest run -p vantadb --lib -E 'test(namespace_stats)'`
- **Estado:** ✅ (2026-08-18 — 4 tests nuevos pasan: 13 run 13 passed con el filtro)
- **Nota:** 4 tests implementados (empty db, mixed aggregation, window boundaries, count cross-check). Cross-check tolera expired: `ns.count == db.count(ns, None) + ns.expired` (el expired es invisible a count() por lazy TTL pero visible en el scan físico).

### Step 4: Test de integración en tests/memory_api.rs
- **Archivos:** `tests/memory_api.rs`
- **Acción:** test end-to-end con `VantaEmbedded::open(tempdir)`: ns1 (1 normal + 1 ttl 1h expiring), ns2 (1 normal) → `namespace_stats(None)` con assert de counts + expiring_soon + expired + consistencia con `list_namespaces`.
- **Verify:** `cargo nextest run -p vantadb --test memory_api`
- **Estado:** ✅ (2026-08-18 — `namespace_stats_end_to_end` PASS)
- **Nota:** binary `memory_api` está excluido del default-filter de nextest → correr con `--ignore-default-filter`. Expired determinista con `ttl_ms = Some(1)` + `sleep(5ms)` (mismo patrón que `edge_cases.rs:delete_expired_ttl_record`).

### Step 5: Docs EMBEDDED_SDK.md
- **Archivos:** `docs/api/EMBEDDED_SDK.md`
- **Acción:** fila `namespace_stats(expiring_soon_window_ms)` en tabla Memory API + bloque `### VantaNamespaceStats` con campos. (R-1: apunta a símbolo real.)
- **Verify:** `pwsh scripts/validate-docs-coverage.ps1`
- **Estado:** ✅ (2026-08-18 — 58 items ok en EMBEDDED_SDK.md, "0 gaps")
- **Nota:** el script explota en la sección MCP (regex busca `fn handle_tools_list` en `vantadb-mcp/src/lib.rs`, pero vive en `handlers/tools.rs`) — bug PRE-EXISTENTE del script, ajeno a esta tarea; la sección SDK (la que valida este cambio) pasa.

## Dependencias
- Ninguna (tarea independiente; VS-04 y sidebar la consumen)

## Review (GATE — agente distinto, P2-01)
- **Revisor:** vanta-review (sub-agente, contexto fresco) — pendiente al cierre
- **Enfoque:** ¿single-scan correcto? ¿clasificación TTL / boundaries correctas? ¿API aditiva semver-safe?
- **Cómo se probó:** `campaign_verify_cmd` con comandos exactos de cada step + full nextest
- **Veredicto:** ⬜ pendiente

## Notas
- El plan file dice "reutilizando `count`/scan": se interpreta como reutilizar la infraestructura de escaneo de records (`memory_record_from_node`/`scan_nodes`), NO llamar a `count()` por namespace (eso sería N paginaciones, prohibido por el contrato).
- `src/metrics/core/snapshot.rs:41` (mención en Archivos clave) es contexto: `OperationalMetricsSnapshot` tiene 50 campos y NO contiene stats per-namespace → no es fuente de datos para esta tarea.
- No se documenta `count()` (existe en core pero no está en la tabla del doc) — fuera de scope; solo se agrega la fila de `namespace_stats`.

## Context Save Point
- **Fecha:** 2026-08-18 (cierre de implementación — post-ejecución del worker)
- **Branch:** develop (cambios de VS-CORE-02 SIN commitecar — el lead commitea)
- **CI pendiente:** verify full ejecutado por worker: fmt ✅, clippy -D warnings ✅, nextest audit 1929/1930 ✅/❌ (1 fallo pre-existente, ver abajo). REVIEW gate + commit pendientes (lead)
- **Decisiones:** single scan_nodes vs N llamadas count() — se eligió single scan porque el contrato prohíbe N paginaciones y el patrón ya existe en `list_namespaces` fallback. Ventana default 24h parametrizable. Scan observa expirados vía helper `memory_record_from_node_include_expired` (el path de lectura los oculta por lazy TTL) para que `expired` sea observable.
- **Problemas conocidos:** ⚠️ **FALLO PRE-EXISTENTE fuera de scope**: `storage::engine::tests::maintenance::test_consolidate_node_with_binary_vector` (panic en `maintenance.rs:272`, `unwrap()` en `get(42)` tras consolidate con Binary vector) — NO fue causado por esta tarea (reproducido en worktree limpio de HEAD 2573d8a5). Vive en `src/storage/` (ownership Arch/Engine); el worker no puede tocarlo. **Escalar a vanta-arch/vanta-engine.** 2do problema menor: `scripts/validate-docs-coverage.ps1` explota en sección MCP (regex `handle_tools_list` obsoleto) — pre-existente, ajeno.
- **Próxima tarea:** VS-CORE-01 / VS-04
