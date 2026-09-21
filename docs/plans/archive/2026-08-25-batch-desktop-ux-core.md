# Plan de Ejecución: Batch Desktop UX/DAUD + Core menor (2026-08-25)

> **Campaign ID:** 6a6c322a-6a6a-4d17-9b34-3166181cbc4a
> **Inicio:** 2026-08-25
> **Estado:** ✅ COMPLETADO
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
- **Estado:** ✅ COMPLETED

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
- **Estado:** ✅ COMMITTED `fix(desktop)` - pulido UX 8 items, build + 68/68

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
- **Estado:** ✅ COMMITTED `fix(desktop)` - DAUD 5/6 (DAUD-08 stash NO dropeado, reportado)

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
- **Estado:** ✅ COMMITTED `test(desktop)` - Playwright 3/3, DAUD-01 píxeles ok

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
- **Estado:** ✅ COMMITTED `refactor(server)` - nits server 4/4, tests 42/42, review APPROVE
efactor(server)\ - nits server 4/4, tests 42/42, review APPROVE

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
- **Estado:** ✅ COMMITTED `docs` - ADR-030 brand naming, 0 renames

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
- **Estado:** ✅ COMMITTED `docs` - verificado, ya en research-agent.md (1c7660dc), 0 diff

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
- **Estado:** ✅ COMMITTED `docs` - desktop README + lazy-load wasm, coverage 0 gaps

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

=== RECITATION MOD-15 ===
Campaign ID: 6a6c322a-6a6a-4d17-9b34-3166181cbc4a
Objetivo activo: MOD-15 — resolver 4 nits de higiene del server: middleware.rs re-export redundante, feature sysinfo vacía, main.rs engine raw sin comentario, ServerState para tests
Estado: completed
Última acción: 4 nits resueltos: (1) middleware.rs eliminado + lib.rs + server-mcp.md scope; (2) feature sysinfo=[] + dev-dep sysinfo removidos + #[allow(unexpected_cfgs)] en 2 includes del harness; (3) comentario embedded-first en main.rs (ref MCP-01); (4) helper build_server_state documentado + 3 sitios idénticos refactorizados + imports muertos limpiados. Review vanta-review ✅ approve (mejora opcional aplicada). Sin commit (regla: lead commitea).
Resultado: OK
Próxima acción: Lead: git add (solo archivos MOD-15 listados en RESULTADO) && commit 'refactor(server): MOD-15 — nits server (middleware.rs, sysinfo feature, main.rs comment, ServerState test helper)' + skill progreso
Contrato: verificacion: cargo check -p vantadb-server ✅ · cargo test -p vantadb-server ✅ 42/42 (3 main + 5 bench + 2 cli_args + 12 e2e + 1 mcp + 19 server) · cargo fmt --check ✅ · cargo clippy -p vantadb-server --all-targets -- -D warnings ✅ 0 warnings · evidencia: grep workspace vantadb_server::middleware = 0 consumidores (eliminación segura); diff refactor 3 sitios verificado por vanta-review diff por diff · artefactos: .opencode/skills/campaign-executor/tasks/MOD-15.md · invariantes: sin cambio de comportamiento server (auth/rate-limit/timeouts/rutas intactos), re-exports vantadb_server::server intactos · deuda: ninguna · queda_pendiente: LEAD verifica workspace (just verify) y commitea solo los archivos de MOD-15; ejecutar skill progreso; AGT-04 cerrado como FAILED (WIP huérfano, plan inexistente — reversible)
Próxima tarea si completa: Siguiente del plan: FIND-17 / TIR-08 (Wave 0 restante)
=== END RECITATION ===

=== RECITATION FIND-17 ===
Campaign ID: 6a6c322a-6a6a-4d17-9b34-3166181cbc4a
Objetivo activo: FIND-17 — auditar consistencia de nombres en artefactos públicos y documentar convención única pre-launch (sin renames).
Estado: completed
Última acción: DISCOVERY (lectura directa de 11 artefactos, verificación live de 6 registries, 65+ citas del repo en docs) + ADR-030 (PROPOSED, tabla de auditoría + convención propuesta) + nota de convención en README/README_ES. Docs coverage 0 gaps.
Resultado: OK
Próxima acción: Lead: verifica mecánico, acepta ADR-030 (PROPOSED → ACCEPTED, Regla 5) y commitea `docs:` FIND-17. Owner resuelve: dominio canónico, PyPI ownership DevpNess→ness-e, publicar vantadb-node.
Contrato: verificacion: scripts/validate-docs-coverage.ps1 ✅ 0 gaps; git diff = solo docs (README, README_ES, ADR-030, task file); cero renames. evidencia: (1) crates.io `vantadb` existe 0.5.0 — crates.io/api/v1/crates/vantadb, alta; (2) PyPI `vantadb-py` existe 0.5.0 owner DevpNess — pypi.org/pypi/vantadb-py/json, alta; (3) npm `vantadb` existe 0.5.0, `vantadb-node` 404 nunca publicado — registry.npmjs.org, alta; (4) GitHub `ness-e/Vantadb` live, About=vantadb.vercel.app — github.com/ness-e/Vantadb, alta; (5) `vantadb.dev` DNS muerto — GET Transport error, alta; (6) badges README no rotos (workflows existen en .github/workflows/) — Get-ChildItem, alta. artefactos: .opencode/skills/campaign-executor/tasks/FIND-17.md, docs/architecture/adr/ADR-030-brand-identity-naming-convention.md, README.md, README_ES.md. invariantes: cero renames; docs en inglés; ADR PROPOSED (decisión owner, Regla 5). deuda: decisión owner — dominio canónico, PyPI ownership, publicar vantadb-node, refrescar metadata PyPI. queda_pendiente: lead verifica + acepta ADR + commitea (NO COMMIT del worker).
Próxima tarea si completa: FIND-11 (Task 8, mismo plan)
=== END RECITATION ===

=== RECITATION 1 ===
Campaign ID: 6a6c322a-6a6a-4d17-9b34-3166181cbc4a
Objetivo activo: UX-A11Y: fixes a11y desktop (UX-02/03/04/06/07/08) con build+vitest verdes, sin commit
Estado: completed
Última acción: Implementados steps 1-9 (13 archivos + index.css + task file); verify step 10: npm run build exit 0 + vitest 68/68. Task file actualizado con Context Save Point de cierre.
Resultado: OK
Próxima acción: Lead: verificar mecánico, commitear solo desktop/* + UX-A11Y.md, marcar completed (WIP guard bloquea worker)
Contrato: {"verificacion":"cd desktop && npm run build (exit 0) && npx vitest run (68/68)","evidencia":[{"claim":"Build y tests verdes tras los 13 archivos editados","evidencia":"desktop: npm run build exit 0; vitest 11 files/68 tests","confianza":"alta"},{"claim":"Contrato a11y cumplido: teclado grid, focus trap, accent-text, tabs tablist","evidencia":"DataExplorer.tsx handleRowKey; useModalFocus conectado en ImportPaste/Drop; index.css --color-accent-text; Inspector/MemoryLens role=tablist","confianza":"alta"}],"artefactos":[".opencode/skills/campaign-executor/tasks/UX-A11Y.md"],"invariantes":"Virtualizer TanStack intacto; role=tab conservado (tests); neón como bg/borde no tocado","deuda":"window.confirm en ImportPaste/ImportDrop (candidato FIND); commit pendiente del lead","queda_pendiente":"Commit del lead SOLO archivos desktop (worktree tiene cambios de MOD-15/AGT-04/FIND-17 — no incluirlos); marcar completed (server WIP guard: AGT-04/MOD-15 in-progress)"}
Próxima tarea si completa: ninguna (task única)
=== END RECITATION ===

=== RECITATION TIR-08 ===
Campaign ID: 6a6c322a-6a6a-4d17-9b34-3166181cbc4a
Objetivo activo: Implementar criterios de investigación TIR-08 (stop por saturación <20% + broadening/narrowing) en research-agent.md
Estado: completed
Última acción: Verificación mecánica del contrato ejecutada: rg -n 'saturación|Re-enfoque|narrowing|broadening|TIR-08' → 4 matches (líneas 30-33). CERO ediciones — el contenido ya estaba implementado y commiteado en 1c7660dc (2026-08-22, '+5 líneas': header, STOP saturación, Re-enfoque, jitter WONTFIT). Task file TIR-08.md creado con Impacto mapeado (Regla 0) y steps ✅.
Resultado: OK
Próxima acción: Lead: verifica rg del contrato, confirma diff vacío (cambio ya en 1c7660dc — NO hay commit pendiente), marca TIR-08 ✅ en plan. Siguiente tarea del plan: FIND-11 (Task 8)
Contrato: verificacion: rg -n 'saturación|Re-enfoque|narrowing|broadening|TIR-08' .opencode/task-system/prompts/research-agent.md → 4 matches (30-33) ✅ | evidencia: claim='research-agent.md contiene criterios 1-2 TIR-08 (contrato del plan)' evidencia='.opencode/task-system/prompts/research-agent.md:30-33; git show 1c7660dc (+5 líneas diff)' confianza='alta'; claim='sin diff pendiente para commitear' evidencia='git diff HEAD -- research-agent.md vacío' confianza='alta' | artefactos: .opencode/skills/campaign-executor/tasks/TIR-08.md | invariantes: no tocar otros prompts del task-system (d8bf0e2a otra sesión); formato de research-agent.md intacto; criterios no duplicados | deuda: ninguna | queda_pendiente: LEAD verifica rg + confirma cierre sin commit (implementación ya en 1c7660dc); si el lead quiere un commit de trazabilidad TIR-08 (solo task file), es decisión suya
Próxima tarea si completa: FIND-11 (Task 8, mismo plan)
=== END RECITATION ===

=== RECITATION FIND-11 ===
Campaign ID: 6a6c322a-6a6a-4d17-9b34-3166181cbc4a
Objetivo activo: FIND-11 — desktop README + lazy-load wasm + npm naming (docs, no código)
Estado: completed
Última acción: Steps 1-4 completos: DISCOVERY (estado real verificado) + desktop/README.md creado + 2 secciones en vantadb-ts/README.md (lazy-load wasm + npm naming) + validate-docs-coverage.ps1 0 gaps exit 0. Task file y plan file actualizados con recitation.
Resultado: OK
Próxima acción: Lead: verify mecánico y commit SOLO desktop/README.md + vantadb-ts/README.md + FIND-11.md
Contrato: verificacion: pwsh scripts/validate-docs-coverage.ps1 ✅ 0 gaps exit 0; evidencia: desktop/README.md creado (Test-Path True, grep tauri dev/install/public installer, alta); nota lazy-load wasm en vantadb-ts/README.md:80-108 (~1.3MB, vite-plugin-wasm, SSR/hooks, alta); aclaracion vantadb vs vantadb-node en vantadb-ts/README.md:110-122 (vantadb publicado 0.5.0, vantadb-node 404 nunca publicado, ADR-030, alta); bundle wasm 1364555 bytes = 1.30MB (Get-ChildItem, alta); anchors existentes intactos (Domain Sub-clients:280, Runtimes:313, alta); artefactos: desktop/README.md, vantadb-ts/README.md, .opencode/skills/campaign-executor/tasks/FIND-11.md; invariantes: cero código tocado, docs EN, no duplicar docs/desktop/README.md (enlazado); deuda: ninguna; queda_pendiente: LEAD commitea por tarea (worker NO commitea); WIP guard bloqueó claim in-progress mientras TIR-08 corria — trabajo registrado en task file igual
Próxima tarea si completa: DAUD-limpi / E2E+visual (Wave 2, mismo plan)
=== END RECITATION ===

=== RECITATION DAUD-LIMPI ===
Campaign ID: 6a6c322a-6a6a-4d17-9b34-3166181cbc4a
Objetivo activo: DAUD-LIMPI: limpieza desktop post-fix (DAUD-03..08), build+vitest verdes, sin commit
Estado: completed
Última acción: Steps 2-7 completos: DAUD-03 (exclusión TitleBar), DAUD-04 (body consolidado), DAUD-05 (5 utilities + 4 overrides dark + animate-rise borradas), DAUD-06 (✎→Pencil ×2: WorkspaceShell + ProxyDashboard hallazgo extra), DAUD-07 (§5 DESIGN_DECISIONS). Verify 2×: build exit 0 + vitest 68/68. DAUD-08: NO dropeado (diff 242 archivos, contenido único).
Resultado: OK
Próxima acción: Lead: verifica mecánico, commitea los 5 archivos + task file, decide DAUD-08 (inspeccionar git stash show -p stash@{0})
Contrato: verificacion: cd desktop && npm run build exit 0 (2×) + npx vitest run 11 files/68 tests (2×) + grep utilidades 0 defs/usos + grep ✎ 0 usos funcionales; evidencia: claim='utilidades muertas 0 usos TSX' evidencia='rg: solo index.css comentarios + prototype/index.html no-bundled' confianza='alta'; claim='stash@{0} NO dropeado' evidencia='git diff stash@{0} = 242 archivos; diff vs HEAD src/storage/wal/mcp/node/python = 1456+/302-' confianza='alta'; artefactos: .opencode/skills/campaign-executor/tasks/DAUD-LIMPI.md; invariantes: TitleBar hover propio intacto; keyframes vanta-rise vivo; glifos identidad intactos; logic.ts React-free intacto; deuda: DAUD-08 decisión lead; queda_pendiente: lead commitea por tarea; WIP guard (FIND-11/UX-POLISH stale) no bloqueó implementación
Próxima tarea si completa: E2E-VISUAL (Task 4)
=== END RECITATION ===

=== RECITATION UX-POLISH ===
Campaign ID: 6a6c322a-6a6a-4d17-9b34-3166181cbc4a
Objetivo activo: UX-POLISH: grupo pulido UX desktop (UX-09/10/11/12/13/14/15/17) con build+vitest verdes, sin commit
Estado: completed
Última acción: Implementados steps 1-13 (19 archivos desktop/src + lib/format.ts nuevo). Verify: npm run build exit 0 + vitest 68/68. Task file con Context Save Point.
Resultado: OK
Próxima acción: Lead: verifica mecánico y commitea solo desktop/src/** de UX-POLISH + task file; worktree tiene cambios de otros agentes (AGT-04/TIR-08/FIND-11) — no incluirlos
Contrato: {"verificacion":"cd desktop && npm run build (exit 0) + npx vitest run (68/68)","evidencia":[{"claim":"Empty states con acción (DataExplorer Ir a Ingestar/Limpiar búsqueda, ResultsList Limpiar búsqueda)","evidencia":"DataExplorer.tsx + ResultsList.tsx + WorkspaceShell.tsx","confianza":"alta"},{"claim":"fmtBytes compartido único (lib/format.ts) con re-export en indices-core; test verde sin modificar","evidencia":"desktop/src/lib/format.ts (nuevo), indices-core.ts, MetricsGrid.tsx, KpiCards.tsx; vitest 68/68","confianza":"alta"},{"claim":"PersonaPanel propaga error real (onError + vantaErrorMessage)","evidencia":"MemoryLens.tsx","confianza":"alta"},{"claim":"Sin window.confirm nuevo; SpaceLens usa inline 2 pasos; ConfirmDiscard inline","evidencia":"grep window.confirm: solo ImportPaste/Drop pre-existentes","confianza":"alta"}],"artefactos":[".opencode/skills/campaign-executor/tasks/UX-POLISH.md","desktop/src/lib/format.ts"],"invariantes":"virtualización TanStack intacta; gridKey remount patrón existente; trash/purge choice conservado; tablist MemoryLens intacto","deuda":"toggle visibilidad columnas grid → FIND; window.confirm ImportPaste/Drop pre-existente → FIND","queda_pendiente":"LEAD verifica mecánico y commitea por tarea (worker NO commitea); WIP guard bloqueó claim (FIND-11 in-progress); skill progreso"}
Próxima tarea si completa: DAUD-LIMPI (Task 3 del plan)
=== END RECITATION ===

=== RECITATION E2E-VISUAL ===
Campaign ID: 6a6c322a-6a6a-4d17-9b34-3166181cbc4a
Objetivo activo: E2E-VISUAL: smoke E2E guard Playwright (UX-19) + verificación visual runtime FIX-D1 (DAUD-01) en desktop/e2e/
Estado: completed
Última acción: Implementados steps 1-9: desktop/e2e/ (serve.mjs, helpers.ts, flujo-critico.spec.ts, daud01-temas.spec.ts) + desktop/playwright.config.ts + @playwright/test + vitest exclude e2e + gitignore + FIND-23 en Backlog. Verify: npx playwright test 3/3 passed (30.9s), vitest 68/68, npm run build exit 0, píxeles dark=rgb(10,10,10). Hallazgos: FIND-23 (namespace mapping HTTP), config Playwright en CWD, preflight Tailwind pisa inputs desnudos, Settings embebido oculta perfiles.
Resultado: OK
Próxima acción: Lead: verifica mecánico, commitea SOLO archivos E2E-VISUAL (desktop/package.json, package-lock.json, vitest.config.ts, .gitignore, desktop/e2e/*, desktop/playwright.config.ts, E2E-VISUAL.md, docs/Backlog.md fila FIND-23), limpia WIP guard stale (FIND-11/UX-POLISH), ejecuta skill progreso
Contrato: verificacion: cd desktop && npx playwright test → 3 passed exit 0 (contrato) · npx vitest run → 11 files/68 tests ✅ · npm run build → exit 0 ✅ · píxeles screenshots: dark TL/TR/BL rgb(10,10,10), light rgb(251,249,245) ✅ | evidencia: claim='guard E2E corre y pasa' evidencia='npx playwright test exit 0, 3 passed (30.9s), log e2e-run7.log' confianza='alta'; claim='sin marco crema en dark' evidencia='e2e/screenshots/daud01-dark.png píxeles rgb(10,10,10) + body computed rgb(10,10,10)' confianza='alta'; claim='hallazgo FIND-23' evidencia='docs/Backlog.md fila FIND-23; vanta-http-map.ts:93 namespace:item.namespace ?? ""' confianza='alta' | artefactos: desktop/e2e/*, desktop/playwright.config.ts, .opencode/skills/campaign-executor/tasks/E2E-VISUAL.md, e2e/screenshots/*.png (gitignored) | invariantes: cero cambios desktop/src/**; vitest 68/68; build OK; NO commit (lead commitea) | deuda: FIND-23 pendiente de fix (workaround en test); bins vanta-cli requieren --features server (infra local resuelta) | queda_pendiente: lead commitea por tarea, WIP guard stale, skill progreso
Próxima tarea si completa: ninguna (Task 4 del plan — siguiente: DAUD-limpi ya commiteado; plan completo)
=== END RECITATION ===
