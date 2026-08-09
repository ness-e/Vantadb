# ADMIN-08: Processes & Connections — panel de procesos y conexiones con kill/remove

## Metadata
- **Plan file:** docs/plans/2026-08-08-admin-console.md
- **Creado:** 2026-08-08
- **last-synced:** 2026-08-08
- **Estado:** ✅ COMPLETED

## Blast Radius
- `desktop/src/components/ProcessPanel.tsx` — NUEVO: lista conexiones (dot status, name, via, id, tag active) con botón "shutdown" por entrada; sección "Subprocesses" con estado vacío documentado.
- `desktop/src/App.tsx` — `<ProcessPanel connections activeId onShutdown={actions.disconnectId} onActivate={actions.activate} />` montado full-width entre DataExplorer y footer.
- `desktop/src/App.css` — reglas `.proc-sub`/`.proc-empty` (tokens ADMIN-03: muted uppercase h3 + nota muted).
- Core Rust: **NO tocado** — no hay registry de procesos (ver Notas).

## Contrato
"`npm run build` en `desktop/` ✅ (verificable); panel lista conexiones activas del ConnectionManager con acción shutdown por entrada (manager.remove vía `vanta_disconnect`); subprocesos como futura extensión documentada (no hay registry en el core)."

## Pasos
### Step 1: Leer fuentes — ✅
- `child_process.rs`: `McpSpawn` (spawn/pid/is_running/kill/request_shutdown) existe PERO **nunca se instancia** — es dead code (solo def + re-export en `connections/mod.rs`; grep: 0 usos en src). NO hay registry/listable de procesos.
- `manager.rs`: `add/remove/set_active/active_id/list_connections/active_info/health/ingest/ingest_batch/search/get/delete/list_records/shutdown_all`. **NO** `list_processes`/`kill_process`.
- `commands/mod.rs`: `connection.rs` + `data.rs` + `metrics.rs` — commands registrados en `lib.rs` (12). Existen `vanta_list_connections` → `manager.list_connections()` y `vanta_disconnect` → `manager.remove(id)`.
- `vanta.ts`: bridge ya expone `listConnections()`, `disconnect(id)`, `setActive(id)`.
- `App.tsx` + `App.css`: design system cream/ink/neon/paper, `.panel` 3px + sombra 6px, `.conn-list`, `.ghost`, `.tag`, `.dot`.

### Step 2: Zero-code planning — ✅
1. Camino mínimo (fallback del contrato): NO hay nada listable de procesos → panel de conexiones con shutdown (vía `vanta_disconnect` = manager.remove) + estado vacío documentado para subprocesos. CERO Rust — no inventar commands que no existan en el core.
2. `ProcessPanel.tsx` reusa `.panel`/`.conn-list`/`.ghost`/`.tag`/`.dot` + 2 reglas CSS nuevas.
3. Mount en App.tsx con props ya existentes del hook; build; commit.

### Step 3: Implementar — ✅
- **ProcessPanel.tsx** (69 líneas): props `connections/activeId/onShutdown/onActivate`; lista conexiones con botón "shutdown" (title "Shutdown (disconnect) this connection") — desconectar = matar/cerrar el backend (en adapters subprocess-backed fuerza kill del sidecar vía McpSpawn Drop). Sección Subprocesses: nota muted explicando que `McpSpawn` existe pero no hay registry wired — placeholder documentado.
- **App.tsx**: `import ProcessPanel` + mount (props del hook).
- **App.css**: `.proc-sub` (muted uppercase 0.75rem) + `.proc-empty`.

### Step 4: Verify — ✅ (parcial por pre-existente)
- `npx tsc --noEmit --strict ... ProcessPanel.tsx` (aislado) → EXIT 0, sin errores.
- `npx vite build` (workdir desktop) → ✅ 43 modules, built in 719ms.
- `npm run build` (tsc && vite build) → ❌ falla en `tsc` por error PRE-EXISTENTE en `ExportPanel.tsx:35` (`Stored | null` no asignable) — archivo untracked de otra línea (ADMIN-09), no tocado por mí.

### Step 5: Commit — ✅
- Commit `f5c69788` — `feat(ADMIN-08): processes and connections panel with kill/remove` — 1 archivo, 69 insertions: `desktop/src/components/ProcessPanel.tsx`.
- Pre-commit hook: sin Rust staged → cargo skip; actionlint ok.
- **Colisión con sesión paralela ADMIN-09:** mientras trabajaba, otra sesión committeó `e0e8ff3a` (snapshot export) que barrió MIS ediciones de App.tsx/App.css (import/mount ProcessPanel + `.proc-sub`) junto con su ExportPanel — mismo patrón documentado en ADMIN-07/ADMIN-06. Verificado: `git show e0e8ff3a:desktop/src/App.tsx` contiene `import ProcessPanel` + `<ProcessPanel`; `App.css` contiene `.proc-sub`. Mi commit final contiene SOLO el componente nuevo; el árbol queda COHERENTE (HEAD renderiza ProcessPanel y el archivo existe en HEAD).

## Dependencias
- ADMIN-03 (design system light) — estilos reutilizados.
- ADMIN-07 (DataExplorer) — patrón de componente autónomo + coexistencia con sesión paralela.
- DESKTOP-11 (child_process.rs / McpSpawn) — la pieza que habilita el future registry de procesos.

## Notas
- **Decisión de procesos (documentada):** el core NO expone registry de subprocesos. `McpSpawn` (DESKTOP-11) es el spawner pero no está instanciado en ninguna conexión ni registrado — no hay nada listable ni matable. El panel muestra el estado vacío con la nota "documented future extension". Cuando se wire un registry (p.ej. `McpSpawnRegistry` en el manager con `list()`/`kill(id)`), agregar commands `vanta_processes` + `vanta_process_kill` y renderizar la tabla real.
- **Acción kill/shutdown:** por entrada se usa `manager.remove(id)` (command existente `vanta_disconnect`) — desconecta el backend, libera recursos (path lock) y en adapters subprocess-backed el Drop del McpSpawn mata el sidecar. No se agregó `vanta_shutdown_all` porque `shutdown_all` solo corre en ExitRequested y no es requerido por el contrato.
- **Verificación de build bloqueada:** `npm run build` falla en `tsc` por `ExportPanel.tsx:35` (untracked, otra línea). Reportado, no tocado (regla: NO tocar archivos de otras líneas). `vite build` (bundler) ✅.
- Concurrencia: sesión concurrente activa sobre el mismo plan (admin-console). App.tsx/App.css ahora contienen cambios de ADMIN-08 + ADMIN-09; re-verificar montaje si se reworkean.

## Context Save Point
- **Fecha:** 2026-08-08
- **Branch:** develop
- **Commit:** f5c69788
- **CI pendiente:** no (vite build ✅ + tsc aislado ✅; `npm run build` completo bloqueado por error pre-existente en ExportPanel.tsx de ADMIN-09)
- **Decisiones:** camino mínimo = panel de conexiones con shutdown por entrada (vía `vanta_disconnect`/`manager.remove` existente); subprocesos = placeholder documentado (sin registry en core). Cero Rust (no inventar commands). Reuso total del design system existente.
- **Problemas conocidos:** (1) `ExportPanel.tsx:35` rompe `npm run build` — arreglar en la línea ADMIN-09; (2) sesión paralela puede volver a barrer App.tsx/App.css; (3) futuro: registry de McpSpawn para la tabla de procesos real.
- **Próxima tarea:** siguiente ADMIN en docs/plans/2026-08-08-admin-console.md (verificar con `campaign_get_next_task`).
