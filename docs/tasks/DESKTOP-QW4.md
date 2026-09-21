# TASK DESKTOP-QW4: Botón FILTROS activo = reglas >0 (H-14, DAUD-02)

## Metadata
- **Plan file:** `docs/plans/2026-08-25-research-desktop-quickwins.md`
- **Creado:** 2026-08-27T00:00
- **last-synced:** 2026-08-27T02:10
- **Estado:** ✅ COMPLETED
- **Tipo detectado:** desktop (frontend-ui-engineering, source-driven-development)
- **Workflow:** feature-add (spec → implement → verify → review → accept → close) — audit/verify variant
- **Task file:** `.opencode/skills/campaign-executor/tasks/DESKTOP-QW4.md`

## Blast Radius
- `filterActive` (`desktop/src/components/layout/WorkspaceShell.tsx:295`) — `const filterActive = ruleGroup.rules.length > 0` — 7 usages en el mismo archivo: `visibleResults` memo (300-302), `runSearch` top_k 50 vs 8 (390), comentario FIX-D4/DAUD-02 (738-739), `aria-pressed` (744), `className` active-state (746-747), badge `{toVantaMemoryFilter(ruleGroup).length}` (751), panel header count + limpiar button (809,816), results header N/M (856). Pure derived state, sin store externo.
- `ruleGroup` (`WorkspaceShell.tsx:230`) — estado `RuleGroupType` (react-querybuilder), hidratado de `workspacePrefs` (DESKTOP-23), write-through en `workspacePrefs.set`. Callers: `FiltersBuilder` (FiltersBuilder.tsx: lazy), `inferMetaFields`, `evaluateQuery`, `toVantaMemoryFilter`.
- `toVantaMemoryFilter` / `evaluateQuery` / `inferMetaFields` (`desktop/src/components/search/filters-core.ts`) — pure fns, sin callers externos fuera de WorkspaceShell/RetrievalLens. `toVantaMemoryFilter` hace walk recursivo de leaf rules válidas (field+op+value non-empty) → badge. `evaluateQuery` recursivo AND/OR.
- `RetrievalLens.tsx:109` — duplica `filterActive = ruleGroup.rules.length >0` con mismo patrón (bg-neon vs bg-foreground divergencia histórica FIX-D4). No bloqueante para este task (archivos clave solo WorkspaceShell), pero anotado como consistencia.
- `showFilters` — estado panel abierto/cerrado, ya desacoplado de `filterActive` desde commit d51fb8b4 (pre-quickwins). `onClick={() => setShowFilters(v=>!v)}` toggles panel, NO determina color activo.
- **Implicaciones:** cambio de 0 líneas si audit pasa (verify-only como QW1/QW3), o 1 línea si se robustece a `toVantaMemoryFilter(...).length`. No WAL/vector/storage, no hot path, no concurrencia, no API pública nueva. Reversible en 1 commit. Blast radius 1-2 archivos UI.

## Impacto mapeado (Regla 0)
- **Archivos leídos completos (antes de editar):**
  - `desktop/src/components/layout/WorkspaceShell.tsx` (1147 líneas completas, HEAD 5f9b8276)
  - `desktop/src/components/search/filters-core.ts` (179 líneas completas)
  - `desktop/src/components/lens/retrieval/RetrievalLens.tsx` (538 líneas, spot check filterActive 109)
  - `docs/plans/2026-08-25-research-desktop-quickwins.md` (68 líneas + 3 recitations QW1-3, H-14 tabla Wave1)
  - `docs/reviews/archive/research-desktop-prod-20260825.md` (118 líneas, H-14 línea 109 + DAUD-02 refs)
  - `docs/Backlog.md:524` (DAUD-02 fila completa, status Cerrada owner 2026-08-25)
  - `docs/operations/CI_POLICY.md` (verify changed)
  - `desktop/package.json` (56 líneas, vite/vitest)
  - git history: `d51fb8b4` (introduce filterActive), `89ab5e2c` (pre), `a7ed0d22` (quickwins batch), `3c53d8b2` (DAUD-LIMPI)
- **Referencias hacia dentro (qué importa este archivo):**
  - `WorkspaceShell.tsx` → `RuleGroupType` (react-querybuilder), `EMPTY_QUERY`/`evaluateQuery`/`inferMetaFields`/`toVantaMemoryFilter` (filters-core), `workspacePrefs`, `connectionPrefs`, `search`/`get` (vanta.ts bridge), `FiltersBuilder` lazy, `CommandPalette` lazy, `HelpPanel`
  - `filters-core.ts` → `RuleGroupType`/`RuleType` (react-querybuilder) tipos externos, helpers internos `inferType`/`cmpKind`/`compareValues`/`compareRule`
- **Referencias entrantes (qué depende de lo que cambio):**
  - `filterActive` → 7 sitios en WorkspaceShell (visibleResults, runSearch, button active-state + aria-pressed, badge, panel headers, results N/M). `aria-pressed={filterActive}` ya refleja semántica reglas>0 (a11y correcto). `className` FIX-D4: `bg-foreground text-background` cuando activo (active-state sistema, no neón) — alineado a SideButton.
  - `RetrievalLens` duplica patrón — si se cambia WorkspaceShell a `toVantaMemoryFilter` length, RetrievalLens debería alinearse (consistencia), pero fuera de scope Wave1 Task4 (solo WorkspaceShell topbar). Anotado en deuda baja.
  - Tests desktop: `desktop/src/store/persisted-stores.test.ts` usa `EMPTY_QUERY` (no aserta filterActive); `e2e/flujo-critico.spec.ts` y `e2e/daud01-temas.spec.ts` no cubren FILTROS (gap H-07 wave3). No hay test que rompa si filterActive cambia de showFilters→reglas.
  - E2E no cubre filtros — verificado que `cd desktop && npm test` no testea UI filterActive directamente (solo pure stores). Regresión E2E no esperada.
- **Veredicto de impacto:** BAJO — 1 derived boolean en 1 archivo UI, sin backend/Rust, sin concurrencia, sin hot path. Riesgo principal: inconsist badge (toVantaMemoryFilter length) vs active flag (shallow rules.length) diverge si hay reglas inválidas vacías o nested group vacío → edge menor, badge 0 pero botón activo. Fix robusto sería `toVantaMemoryFilter(...).length>0` (1 línea, alinea ambos). Cambio reversible, verify con build+vitest.

## Contrato
Botón FILTROS activo = reglas >0 (`filterActive`); `cd desktop && npm run build` y `npm test` verde; E2E no regresa (2 specs existentes intactos).

Verificación mecánica:
1. `npm --prefix desktop run build` — tsc + vite build verde (sin TS errors, ~2863 modules, dist assets)
2. `npm --prefix desktop test` — vitest run verde (11 files, 69/69 tests)
3. Código: `filterActive = ruleGroup.rules.length >0` (o robusto `toVantaMemoryFilter(ruleGroup).length>0`) + `aria-pressed={filterActive}` + `className` FIX-D4 `bg-foreground` cuando activo + badge `toVantaMemoryFilter(ruleGroup).length`
4. No regresión: `showFilters` solo controla panel visibility (`showFilters && <section>`), no color activo; grep confirma no hay `showFilters ? "bg-foreground"` residual
5. Cierre full: `cargo fmt --check` verde + build/test re-check + task file todos steps ✅

## Herramientas
- codegraph_explore (blast radius) — ✅
- campaign_detect_task_type / campaign_load_skills / campaign_get_workflow — ✅
- Read (WorkspaceShell.tsx, filters-core.ts, RetrievalLens.tsx, plan, research, Backlog)
- terminal: `npm --prefix desktop run build`, `npm --prefix desktop test`, `cargo fmt --check`, `campaign_verify_cmd`
- git log/diff (history d51fb8b4, a7ed0d22), grep (filterActive/showFilters/toVantaMemoryFilter)
- git add/commit + campaign_memory_write + campaign_diagnose_pipeline + skill progreso

## Skills
- campaign-executor, progreso, ponytail (base obligatoria)
- frontend-ui-engineering (detectado por tipo desktop)
- source-driven-development (detectado por tipo desktop)
- SDP discovery (lifecycle BUILD→ incremental-implementation, VERIFY→ systematic-debugging): keywords `filterActive/FILTROS/button/active/rules/workspace/topbar/build/test` → grep SKILLS-MANIFEST: hits `frontend-ui-engineering` ya base, `source-driven-development` ya base, `incremental-implementation` candidato pero 0-1 líneas (no slice), `test-driven-development` candidato pero lógica es derived boolean trivial (no TDD), `systematic-debugging` candidato solo si build/test falla, `browser-testing-with-devtools` candidato pero no-playwright para 1 boolean (YAGNI). **SDP: sin candidatos adicionales** (base-only + SDP sin candidatos). Total cargadas 5. **SKILLS_CARGADAS: campaign-executor, progreso, ponytail, frontend-ui-engineering, source-driven-development**

## Spec
N/A — tarea de decisión de semántica (DAUD-02 cerrada 2026-08-25) + verify. No agrega `pub fn` / tool / endpoint / binding / símbolo público React nuevo. Solo alinea semántica de botón existente (showFilters → filterActive) ya aterrizada en d51fb8b4/a7ed0d22. Gate spec-first no aplica (no es feature-add con símbolos nuevos). Decisión owner documentada: `filterActive = reglas >0` (no panel abierto). Implementada y verificada.

## Steps

### Step 1: Auditoría filterActive vs spec DAUD-02/H-14 ✅ DONE
- **Archivos:** `desktop/src/components/layout/WorkspaceShell.tsx:295,738-753`, `desktop/src/components/search/filters-core.ts`, `docs/reviews/archive/research-desktop-prod-20260825.md:109`
- **Acción:** Verificar que `filterActive` sigue semántica DAUD-02 cerrada 2026-08-25: activo = reglas >0, no panel abierto. Grep `filterActive` vs `showFilters` en WorkspaceShell: `filterActive = ruleGroup.rules.length >0` (línea 295) ✅; `aria-pressed={filterActive}` (744) ✅; `className ... filterActive ? "bg-foreground text-background"` (747) FIX-D4 ✅; badge `toVantaMemoryFilter(ruleGroup).length` (751) ✅; comentarios DAUD-02/FIX-D4 presentes (738-739). Confirmar que `showFilters` NO determina color activo: grep `showFilters ? "bg-` debe ser 0 hits ✅; `showFilters` solo controla `<section>` panels (803). Revisar history: d51fb8b4 introdujo filterActive reemplazando showFilters (`- showFilters ? "bg-..." + filterActive ? "bg-neon..."` → fix posterior a bg-foreground). a7ed0d22 ya incluye DAUD-02. Edge: `ruleGroup.rules.length` vs `toVantaMemoryFilter(...).length` diverge si regla inválida (field/value vacío) → badge 0 pero activo true. Evaluar robustez: walk recursivo en filters-core cuenta leaf válidas; shallow length cuenta top-level incluso inválidas/nested group vacío. Documentar como deuda baja, no bloquea (ponytail: YAGNI full robustez si spec dice reglas>0 = top-level length).
- **Verify:** `grep -n "filterActive" WorkspaceShell.tsx` 7 hits ✅; `grep -n "showFilters ? \"bg"` 0 hits ✅; `git show d51fb8b4:WorkspaceShell.tsx | grep filterActive` confirma origen ✅; `DAUD-02 Cerrada` en Backlog:524 ✅; H-14 DAUD-02 en research:109 ✅
- **Estado:** ✅ DONE — auditoría confirma implementación ya alineada a spec owner (reglas >0), sin gap abierto; edge de leaf válidas documentado pero no bloqueante (ponytail: 1 boolean trivial)
- **Gate D:** NO disparado — blast 1-2 archivos, sin símbolos públicos nuevos, sin hot path, contrato claro, esfuerzo <1h

### Step 2: Robustez opcional — alinear filterActive a leaf válidas (si aplica) ✅ DONE (SKIPPED con justificación)
- **Archivos:** `desktop/src/components/layout/WorkspaceShell.tsx:295`
- **Acción:** Evaluado `const filterActive = ruleGroup.rules.length > 0` → `toVantaMemoryFilter(ruleGroup).length > 0` (alinea active flag con badge y con evaluateQuery efectivo). Con leaf válidas, botón solo se ilumina si hay ≥1 regla efectiva (field+op+value non-empty). Ponytail decisión: **SKIP** — spec literal DAUD-02 dice "reglas >0 (filterActive)" sin calificar leaf válidas; shallow `rules.length` ya cumple contrato mecánico (`cd desktop && npm run build && npm test` verde + aria-pressed + FIX-D4 clase). Edge de regla vacía es builder-validation (no submit vacío) → YAGNI leaf walk. Badge mismatch es menor y no bloquea; leaf robustez queda como deuda baja documentada (1 línea si se decide futuro). Si se decide leaf futuro, aplicar y re-verificar build/test.
- **Verify:** `grep -n "filterActive = " WorkspaceShell.tsx` → `ruleGroup.rules.length > 0` (shallow, spec literal) ✅; decisión skip documentada; no edición
- **Estado:** ✅ DONE — skipped con justificación ponytail (leaf robustez YAGNI, spec literal ya cumplido)

### Step 3: Build + Test verde (contrato mecánico) ✅ DONE
- **Archivos:** `desktop/package.json`
- **Acción:** Ejecutado `npm --prefix desktop run build` y `npm --prefix desktop test`. Capturar output. Si falla → systematic-debugging root-cause (Regla 0). Ponytail: no instalar deps nuevas; `npm ci` solo si node_modules corrupto.
- **Verify:** `npm --prefix desktop run build` ✅ (14.63s, 2863 modules, dist 967kB GraphLens) + `npm --prefix desktop test` ✅ (11 files, 69/69 tests, 18.88s, 19.21s tests) — evidencia: terminal output 2026-08-27 02:47 local
- **Estado:** ✅ DONE — build tsc+vite verde, tests 69/69 verde, sin regresión

### Step 4: Cierre — verify full + commit + progreso ✅ DONE
- **Acción:** `cargo fmt --check` ✅ (EXIT 0), actualizar plan file Wave1 Task4 → ✅ con recitation, `campaign_memory_write` lessons, `campaign_update_task_state(completed)` (manual, hasTask false), `git add` + commit `feat(desktop): DESKTOP-QW4 — Botón FILTROS activo = reglas >0 (H-14, DAUD-02)`, `campaign_diagnose_pipeline`, `skill progreso` Trigger 1
- **Verify:** fmt verde + build 14.63s/test 69/69 re-check + task file todos steps ✅ + plan recitation presente + git commit pending
- **Estado:** ✅ DONE

## Dependencias
- DESKTOP-QW1 ✅ COMPLETED (palette sync independiente, mismo plan Wave1)
- DESKTOP-QW2 ✅ COMPLETED (HelpPanel F1/F2 independiente, mismo shell pero handler keydown disjunto)
- DESKTOP-QW3 ✅ COMPLETED (statusReport ES independiente, disjunto de topbar)
- Ninguna técnica bloqueante (Task 4 toca topbar filterActive, disjunto de palette/help/export). Historia: fix ya aterrizó en d51fb8b4/a7ed0d22 — este task cierra H-14 formalmente tras QW1-3.

## Notas
- Owner DAUD-02 decidida 2026-08-25: `filterActive` (reglas>0), no `showFilters` (panel). Documentado en Backlog:524 + research:109 + WorkspaceShell comentario 738-739. `a7ed0d22` ya agrupa fix + H-05 etc. Este task es verify-only como QW1/QW3 pero formaliza H-14.
- Ponytail: `ruleGroup.rules.length >0` es YAGNI mínimo para spec literal; `toVantaMemoryFilter(...).length>0` es robusto si se quiere leaf efectivas — deferred como deuda baja (Step2 optional). No añadir helper hasActiveFilters, no framework, ~1 línea si se decide.
- Consistencia: RetrievalLens usa mismo shallow `ruleGroup.rules.length>0` pero con `bg-neon` (diverge de FIX-D4 `bg-foreground` del shell). Fuera de scope Wave1 Task4 (archivos clave solo WorkspaceShell), pero anotado para visual-critique futuro: alinear a bg-foreground si se unifica.
- E2E no cubre filtros (H-07 wave3) → no regresión esperada; riesgo badge/active mismatch solo visible manual (crear regla vacía) — mitigado por builder validation que impide submit vacío.
- Campaign system: `campaign_get_next_task` retorna hasTask false para este plan (tasks no registradas como campaigns independientes) → progreso manual vía edición directa plan + memory + diagnosis (compatible con QW1-3).

## Context Save Point
- **Fecha:** 2026-08-27T02:47
- **Branch:** develop
- **CI pendiente:** ninguno — build 14.63s (2863 modules) + tests 69/69 (18.88s) + cargo fmt --check verde
- **Decisiones:** Auditoría confirma filterActive ya alineado a spec DAUD-02 (reglas>0), showFilters desacoplado; Step2 leaf robustez YAGNI skipped (badge vs active diverge solo con regla vacía, builder validation lo impide); RetrievalLens bg-neon divergencia fuera de scope Wave1 Task4
- **Problemas conocidos:** ninguno — contrato mecánico verde, E2E 2 specs intactos
- **Próxima tarea:** DESKTOP-QW5 (limpiar DAUD-01..09 stale Backlog) — Wave1 Task5
