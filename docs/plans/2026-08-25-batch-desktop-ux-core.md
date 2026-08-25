# Plan de Ejecución: Batch Desktop UX/DAUD + Core menor (2026-08-25)

> **Inicio:** 2026-08-25
> **Estado:** ⏳ EN PROGRESO
> **Fuente:** docs/Backlog.md (selección del lead + confirmación del usuario 2026-08-25)
> **Modo:** FAIL_MODE=parallel, MAX_CONCURRENT=3

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 8 |
| 🟡 DEFER | 4+ |
| ❌ SKIP | 0 |
| 🔴 BLOQUEADO | 1 (CORE-01) |

Status: ⬆️ uphill = 1 (DAUD-02 decisión owner DEFER) · ⬇️ downhill = 8

> **Agrupación:** las tareas del mismo directorio desktop se agrupan en un solo sub-agente (lección del batch anterior: NO paralelizar sub-agentes que editan los mismos archivos). 8 tareas agrupadas cubren ~20 filas del backlog.

## Tasks

### Task 1: UX-a11y — UX-02 + UX-03 + UX-04 + UX-06 + UX-07 + UX-08 (grupo a11y desktop)

- **Appetite:** max 1d
- **Esfuerzo:** 🟡
- **Prioridad:** 🟡
- **Archivos clave:** `desktop/src/components/DataExplorer.tsx`, `ResultsList.tsx`, `ingest/ImportPaste.tsx`, `ingest/ImportDrop.tsx`, `ingest/useModalFocus.ts`, `home/HomeOverview.tsx`, `layout/WorkspaceShell.tsx`, `graph/GraphLens.tsx`, `space/SpaceLens.tsx`, `inspector/Inspector.tsx`, `memory/MemoryLens.tsx`, `MetricsGrid.tsx`
- **Verificación real:** ✅ CÓDIGO-REAL — Paso 0 confirmó: UX-02 (ResultsList `<li onClick>` sin tabIndex/aria-selected), UX-03 (useModalFocus.ts existe completo, falta conectar en ImportPaste/ImportDrop), UX-06 (text-neon en texto pequeño ≈3:1 falla AA), UX-07 (MemoryLens usa role=tab pero nav sin tablist/flechas), UX-04/UX-08 por verificar en DISCOVERY.
- **Gate Justificación:** a11y desktop (WCAG): teclado, focus trap, labels, contraste, tabs ARIA, canvas fallback. Alta densidad de fixes de bajo riesgo.
- **Gate Result:** ✅ DO
- **Contrato:** `cd desktop && npm run build` exit 0 + `npx vitest run` pasa; keyboard: Enter/Espacio abre Inspector desde grid; focus trap conectado en ambos modales; contraste texto-neon ≥4.5:1 (o token accent-text); tabs ARIA con tablist + flechas
- **Task file:** `skills/campaign-executor/tasks/UX-A11Y.md`
- **Estado:** ⬜ PENDING

  **Pre-mortem:**
  - Fallo 1: tocar DataExplorer (grid virtualizado TanStack) sin romper virtualización
  - Fallo 2: conectar useModalFocus sin duplicar el useEffect de Escape existente
  - Fallo 3: cambiar text-neon a token nuevo rompe el look linocut (neón es identidad)
  - **Stop conditions:** si el cambio de contraste requiere re-diseñar el token neón (identidad), usar `--color-accent-text` (~#C24000) solo para texto, no tocar el neón de fondo.
  - **Cynefin:** 🟨 complicado — a11y en grid virtualizado. **Top 3 riesgos:** (1) romper virtualización; (2) tokens visuales; (3) focus trap duplicado.

### Task 2: UX-polish — UX-09 + UX-10 + UX-11 + UX-12 + UX-13 + UX-14 + UX-15 + UX-17 (grupo pulido desktop)

- **Appetite:** max 1d
- **Esfuerzo:** 🟡
- **Prioridad:** 🟢
- **Archivos clave:** `desktop/src/components/space/SpaceLens.tsx`, `consolidate/ConsolidateLens.tsx`, `DataExplorer.tsx`, `home/HomeOverview.tsx`, `layout/WorkspaceShell.tsx`, `activity/ActivityPanel.tsx`, `memory/MemoryLens.tsx`, `MetricsGrid.tsx`, `KpiCards.tsx`, `SplashScreen.tsx`, `indices/IndicesLens.tsx`, `lens/retrieval/RetrievalLens.tsx`, `ingest/IngestForm.tsx`
- **Verificación real:** 🟡 VERIFICAR — backlog detalla cada uno (confirmar en DISCOVERY): UX-09 (3 lenguajes de confirm destructiva), UX-10 (densidad grid desborde 1440px), UX-11 (empty states sin salida), UX-12 (RESUMEN sin énfasis), UX-13 (banner audit filtra internos), UX-14 (PersonaPanel traga errores), UX-15 (misc menor: badge err, fmtBytes, splash teclado, notice ✕), UX-17 (grid no refresca tras ingest).
- **Gate Justificación:** pulido UX desktop de bajo riesgo; effort 🟡 agrupado.
- **Gate Result:** ✅ DO
- **Contrato:** `cd desktop && npm run build` exit 0 + `npx vitest run` pasa; empty states con acción; fmtBytes compartido; PersonaPanel propaga error real
- **Task file:** `skills/campaign-executor/tasks/UX-POLISH.md`
- **Estado:** ⬜ PENDING

  **Pre-mortem:**
  - Fallo 1: UX-17 (onRefresh) toca el flujo ingest — riesgo de regresión en import
  - Fallo 2: unificar confirm destructiva cambia UX de SpaceLens (window.confirm nativo)
  - **Stop conditions:** —. **Cynefin:** 🟦 obvio. **Top 3 riesgos:** (1) flujo ingest; (2) confirm destructiva; (3) densidad grid.

### Task 3: DAUD-limpi — DAUD-03 + DAUD-04 + DAUD-05 + DAUD-06 + DAUD-07 + DAUD-08 (grupo limpieza desktop post-fix)

- **Appetite:** max 1d
- **Esfuerzo:** 🟢
- **Prioridad:** 🟢
- **Archivos clave:** `desktop/src/App.css`, `desktop/src/index.css`, `desktop/src/components/layout/WorkspaceShell.tsx`, `desktop/src/components/mark/mark-studio.tsx`, `desktop/src/components/activity/Timeline.tsx`, `desktop/DESIGN_DECISIONS.md`
- **Verificación real:** 🟡 VERIFICAR — backlog (P37): DAUD-03 (hover-translate global afecta TitleBar), DAUD-04 (reglas :root body/.dark body redundantes), DAUD-05 (utilidades sin uso: halftone/grid-tech/speed-lines/animate-rise), DAUD-06 (glifos ✎ y 5 en mark-studio/Timeline), DAUD-07 (doc convención iconos), DAUD-08 (drop stash@{0}).
- **Gate Justificación:** limpieza CSS/iconos post-fix; effort 🟢. DAUD-08 es git stash drop (lead).
- **Gate Result:** ✅ DO
- **Contrato:** `cd desktop && npm run build` exit 0 + `npx vitest run` pasa + grep utilidades muertas = 0 usos confirmado; stash@{0} dropeado (verify diff vs worktree = 0)
- **Task file:** `skills/campaign-executor/tasks/DAUD-LIMPI.md`
- **Estado:** ⬜ PENDING

  **Pre-mortem:**
  - Fallo 1: borrar utilidades que se usan en runtime vía strings (no detectables por grep TSX)
  - Fallo 2: drop stash@{0} si contiene algo real
  - **Stop conditions:** DAUD-08: verificar diff stash vs worktree = 0 ANTES de dropear; si difiere, no dropear y reportar. **Cynefin:** 🟦 obvio. **Top 3 riesgos:** (1) utilidades runtime; (2) stash real; (3) convención iconos.

### Task 4: E2E+visual — UX-19 + DAUD-01 (smoke E2E guard + verificación visual runtime)

- **Appetite:** max 1d
- **Esfuerzo:** 🟡
- **Prioridad:** 🟡
- **Archivos clave:** `desktop/e2e/` (nuevo), `desktop/src/App.css`, `desktop/src/index.css`
- **Verificación real:** 🟡 VERIFICAR — UX-19: convertir el smoke E2E (ingest→teclado→borrar→papelera→restore→paleta) en test Playwright permanente `desktop/e2e/`; DAUD-01: verificación visual runtime del FIX-D1 (padding 24px→0, tokens flipping) con capturas light/dark.
- **Gate Justificación:** guard de regresión del flujo crítico desktop (hoy depende de QA manual); effort 🟡.
- **Gate Result:** ✅ DO
- **Contrato:** test Playwright en `desktop/e2e/` corre (npx playwright test) y pasa; capturas light/dark confirmando sin marco crema en dark
- **Task file:** `skills/campaign-executor/tasks/E2E-VISUAL.md`
- **Estado:** ⬜ PENDING

  **Pre-mortem:**
  - Fallo 1: Playwright requiere server desktop en dev (vite) — el e2e debe arrancarlo
  - Fallo 2: el flujo completo (ingest con labels + restore) depende de datos sembrados
  - **Stop conditions:** si el e2e requiere Tauri (no solo vite) — limitar a web build (`embedded`) como el smoke original. **Cynefin:** 🟨 complicado — test runner. **Top 3 riesgos:** (1) server dev; (2) datos; (3) Tauri vs vite.

### Task 5: MOD-15 — nits agrupados server

- **Appetite:** max 3h
- **Esfuerzo:** 🟢
- **Prioridad:** 🟢
- **Archivos clave:** `vantadb-server/src/middleware.rs`, `vantadb-server/Cargo.toml`, `vantadb-server/src/main.rs`
- **Verificación real:** 🟡 VERIFICAR — backlog: middleware.rs re-export redundante, feature sysinfo vacía, main.rs abre engine raw sin comentario, falta constructor ServerState para tests.
- **Gate Justificación:** higiene server; effort 🟢.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo check -p vantadb-server` + test server verde; nits resueltos o documentados
- **Task file:** `skills/campaign-executor/tasks/MOD-15.md`
- **Estado:** ⬜ PENDING

  **Pre-mortem:** —. **Stop conditions:** —. **Cynefin:** 🟦 obvio. **Top 3 riesgos:** (1) re-export rompe imports.

### Task 6: FIND-17 — identidad de marca inconsistente

- **Appetite:** max 3h
- **Esfuerzo:** 🟡
- **Prioridad:** 🟢
- **Archivos clave:** `pyproject.toml`, `package.json` (vantadb-ts/node), README badges, URLs de repo
- **Verificación real:** 🟡 VERIFICAR — backlog: repo GitHub `ness-e/Vantadb` vs crate/npm `vantadb` vs PyPI `vantadb-py` vs dominio sin DNS. Auditar consistencia + decidir convención.
- **Gate Justificación:** DX/marca pre-launch; effort 🟡. Es decisión de convención — documentar, no forzar renames (renames romperían semver).
- **Gate Result:** ✅ DO
- **Contrato:** auditoría de nombres documentada; convención decidida y documentada (ADR o nota); links/URLs actualizados donde no rompen
- **Task file:** `skills/campaign-executor/tasks/FIND-17.md`
- **Estado:** ⬜ PENDING

  **Pre-mortem:**
  - Fallo 1: renombrar crate/npm rompe semver/usuarios — NO renombrar, solo documentar
  - **Stop conditions:** renames NO se hacen en este batch (solo auditoría + decisión documentada). **Cynefin:** 🟦 obvio. **Top 3 riesgos:** (1) rename roto; (2) decisión sin owner.

### Task 7: TIR-08 — criterios de investigación en research-agent

- **Appetite:** max 1h
- **Esfuerzo:** 🟢
- **Prioridad:** 🟢
- **Archivos clave:** `.opencode/task-system/agents/research-agent.md` (o el archivo del agente research)
- **Verificación real:** ✅ CÓDIGO-REAL — backlog P18 TIR-08: decisión registrada "IMPLEMENTAR parcial (criterios 1 y 2 en research-agent.md ~6 líneas)".
- **Gate Justificación:** implementar decisión ya tomada (criterios de investigación: stop si saturación <20%, broadening/narrowing); effort 🟢.
- **Gate Result:** ✅ DO
- **Contrato:** research-agent.md contiene los criterios 1-2; verificación con rg
- **Task file:** `skills/campaign-executor/tasks/TIR-08.md`
- **Estado:** ⬜ PENDING

  **Pre-mortem:** —. **Stop conditions:** —. **Cynefin:** 🟦 obvio. **Top 3 riesgos:** (1) archivo no existe con ese nombre.

### Task 8: FIND-11 — rutas alternativas sin pulir (desktop README + docs)

- **Appetite:** max 3h
- **Esfuerzo:** 🟢
- **Prioridad:** 🟢
- **Archivos clave:** `desktop/README.md` (nuevo), `vantadb-ts/README.md`, `docs/`
- **Verificación real:** 🟡 VERIFICAR — backlog: `desktop/` sin README ni instalador público, sin hooks/ejemplos React, bundle .wasm 1.3MB sin doc de lazy-load, confusión `vantadb-node` vs `vantadb` en npm.
- **Gate Justificación:** docs DX; effort 🟢. Documentar (no código).
- **Gate Result:** ✅ DO
- **Contrato:** `desktop/README.md` creado (instalación + desarrollo); nota lazy-load wasm; docs coverage 0 gaps
- **Task file:** `skills/campaign-executor/tasks/FIND-11.md`
- **Estado:** ⬜ PENDING

  **Pre-mortem:** —. **Stop conditions:** —. **Cynefin:** 🟦 obvio. **Top 3 riesgos:** (1) docs drift.

## DEFER

| ID | Motivo |
|----|--------|
| DAUD-02 | Decisión de diseño FILTROS activo — requiere owner (showFilters vs filterActive) |
| MOD-05 | Deprecar InMemoryEngine (~850 líneas) — refactor grande, no es AHORA |
| REVIEW-10 / REVIEW-12 | God-file splits (cli_server.rs ~4100L, api.rs ~2500L) — refactors grandes, batch dedicado |
| MCP-34 (resto) | snapshot_restore = feature core nueva (Arch/Engine) |
| FIND-20 / FIND-21 | Tauri window state / menú contextual — investigación Tauri + batch desktop dedicado |
| PRO-01..06, FUT-02..11, RES-01..15, DEC-01/02 | Roadmap / Pro / decisiones — fuera de ejecución |

## SKIP

| ID | Motivo |
|----|--------|
| — | — |

## BLOQUEADO

| ID | Motivo |
|----|--------|
| CORE-01 | Persistencia Binary on-disk — requiere ADR de formato (Regla 5) |
| AUD-042 | tantivy ≥0.27 no publicada (upstream) |

## Waves

- **Wave 0**: UX-a11y (1) · MOD-15 (5) · TIR-08 (7)
- **Wave 1**: UX-polish (2) · FIND-17 (6) · FIND-11 (8)
- **Wave 2**: DAUD-limpi (3) · E2E+visual (4)

> MAX_CONCURRENT = 3. Sub-agentes NO commitean; el lead verifica mecánico y commitea por tarea. Desktop agrupado: 1 sub-agente por área (a11y, polish, limpi, e2e) — nunca 2 en paralelo sobre el mismo directorio.

## Notas

- plan-adjust [2026-08-25]: creado desde docs/Backlog.md (limpio, 77 activas). Usuario confirmó 8 tareas agrupadas desktop+core.
- Paso 0: UX-02/03/06/07 verificados reales contra código; resto 🟡 VERIFICAR en DISCOVERY.
- CodeGraph auto-sync deshabilitado (lock de otro proceso) — sub-agentes deben leer archivos directos.
- ⬆️ uphill = 1 (DAUD-02 DEFER) · ⬇️ downhill = 8