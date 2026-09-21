# UX-POLISH: Grupo pulido UX desktop (UX-09, 10, 11, 12, 13, 14, 15, 17)

## Metadata
- **Plan file:** docs/plans/2026-08-25-batch-desktop-ux-core.md (Task 2)
- **Fuente:** docs/Backlog.md (auditoría P34, grupo pulido desktop)
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟢
- **Tipo:** Frontend (React 19 + Tailwind v4 + Tauri, desktop/)
- **Turns estimados:** 18
- **Creado:** 2026-08-25T12:30
- **last-synced:** 2026-08-25T12:30
- **Estado:** ✅ COMPLETED (sync 2026-08-25 — 13/13 steps ✅; stale cleanup por FIND-23)
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 12 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | WorkspaceShell (compone IngestForm, DataExplorer, MetricsGrid, KpiCards, IndicesLens, RetrievalLens, MemoryLens, SpaceLens, ActivityPanel) · App (SplashScreen) |
| Callees | vanta.ts (vantaErrorMessage) · store/undo · indices/indices-core (fmtBytes) · export/export-jsonl |
| Implicaciones | Ningún contrato SDK/core cambia (todo es UI interna desktop). Props React internos agregados son opcionales (default noop/undefined → callers existentes no se rompen). Display de KpiCards cambia KiB→KB (unificación fmtBytes pedida por el ticket). Confirmación destructiva de SpaceLens pasa de window.confirm nativo a inline 2 pasos (patrón DeleteButton). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** DataExplorer.tsx (931L), SpaceLens.tsx (319L), SelectionBar.tsx (68L), ConsolidateLens.tsx (621L), ConfirmDiscard.tsx (137L), TrashLens.tsx (138L), HomeOverview.tsx (403L), WorkspaceShell.tsx (1100L), ActivityPanel.tsx (343L), MemoryLens.tsx (495L), MetricsGrid.tsx (171L), KpiCards.tsx (94L), IngestForm.tsx (145L), ResultsList.tsx (69L), indices/indices-core.ts (116L), IndicesLens.tsx (214L), lens/retrieval/RetrievalLens.tsx (536L), layout/SplashScreen.tsx (61L), layout/LensShell.tsx (37L), graph/GraphLens.tsx (149L, editado solo línea 43), index.css (sectores 240-339, grep tokens)
- **Archivos referenciados hacia dentro (imports/deps):** vanta.ts (vantaErrorMessage, ingest, get), store/undo (undoStore), indices/indices-core (fmtBytes), components/export/export-jsonl, store/favorites
- **Archivos que referencian a los editados (referencias entrantes):** WorkspaceShell.tsx importa IngestForm/DataExplorer/MetricsGrid/KpiCards/IndicesLens/RetrievalLens/MemoryLens/SpaceLens( lazy)/ActivityPanel/ResultsList · App.tsx usa SplashScreen (verif. grep) · SelectionBar solo lo usa SpaceLens · ConfirmDiscard solo lo usa ConsolidateLens
- **Veredicto impacto:** **bajo** — cambios UI-internos con props opcionales; sin API pública SDK; virtualización TanStack no se toca (solo clases de celda); pruebas existentes (indices-core.test.ts importa fmtBytes desde indices-core → se preserva con re-export)

## Contrato
"`cd desktop && npm run build` exit 0 + `npx vitest run` pasa; empty states con acción (DataExplorer/ResultsList); fmtBytes compartido (1 implementación, re-export desde indices-core); PersonaPanel propaga error real (onError + vantaErrorMessage)"

## Spec (SDD — no feature-add: props React internos opcionales, sin símbolos públicos de SDK)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Dónde vive el `fmtBytes` compartido | A: nuevo `src/lib/format.ts` (home limpio, 1 archivo nuevo + re-export en indices-core para test) / B: reusar `indices/indices-core.ts` directo (0 archivos, pero MetricsGrid/KpiCards dependerían de un módulo de lente) | A | ✅ decidido-por-evidencia (indices-core.test.ts:6 importa de indices-core → re-export preserva test sin modificar) |
| 2 | Semántica del fmtBytes compartido (decimal vs binario) | A: decimal (indices-core/MetricsGrid, ya testado) / B: binario (KpiCards MiB) | A | ✅ decidido-por-evidencia (2 de 3 sitios usan decimal + test existente; KpiCards cambia KiB→KB = la unificación pedida por el ticket) |
| 3 | Unificación confirm destructiva en ConsolidateLens | A: ConfirmDiscard se renderiza inline (sacar fixed overlay → caja en flujo, conserva 2 pasos + radio papelera/permanente) / B: reescribir a button-swap estilo DeleteButton (pierde elección papelera/permanente) / C: no tocar (queda 3er lenguaje) | A | ✅ decidido-por-evidencia (conserva funcionalidad trash/purge + tipo-to-confirm; elimina el 3er lenguaje modal; diff chico) |
| 4 | Empty state "Ir a Ingestar" en DataExplorer | A: prop opcional `onGoToIngest` + scrollIntoView al form (id ingest-form) / B: sin navegación (solo mensaje) | A | ✅ decidido-por-evidencia (contrato exige empty states con acción) |
| 5 | UX-10 toggle de visibilidad de columnas | A: solo densidades (payload vw-cap + TTL w-20) / B: agregar toggle completo | A | ✅ decidido-por-evidencia (REGLA CRÍTICA del usuario dice "ajustar densidades sin romper el grid"; toggle = feature aparte → FIND en Notas) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** virtualización TanStack del grid (DataExplorer) intacta — solo clases de celda/estilos; `key={gridKey}` remount del grid no rompe el import (patrón ya usado por batch delete e import modals); ConfirmDiscard conserva elección papelera/permanente; `role="tablist"` y focus management de MemoryLens intactos; contrato VS-08/VS-03 (master-detail, undoStore) intacto
- **Comandos de verificación:** `cd desktop && npm run build` (tsc + vite) exit 0; `cd desktop && npx vitest run` (68/68 esperado)
- **Deuda pendiente:** toggle de visibilidad de columnas del grid (UX-10, fuera de scope → candidato FIND-*); window.confirm restante en ImportPaste/ImportDrop (línea 89/107, ya anotado como candidato FIND por UX-A11Y)

## Recitation (canónico — se sincroniza a campaign_update_task_state al cierre)

| Campo recitation (MCP) | ← fuente |
|------------------------|----------|
| `activeGoal` | UX-POLISH: pulido UX desktop (8 items auditoría P34) con build+vitest verdes, sin commit |
| `lastAction` | Step 13 (verify): `cd desktop && npm run build` exit 0 + `npx vitest run` pasa |
| `result` | OK (al cierre) |
| `nextAction` | Lead: verifica mecánico + commitea solo archivos desktop/* + task file |
| `contract` | Ver ## Contrato + evidencia abajo |
| `nextTask` | Siguiente del plan: DAUD-LIMPI (Task 3) |

contract (cierre):
```
verificacion: "cd desktop && npm run build" (exit 0) + "npx vitest run" (68/68) ✅
evidencia:
  - claim: Empty states con acción (DataExplorer "Limpiar búsqueda"/"Ir a Ingestar", ResultsList "Limpiar búsqueda")
    evidencia: desktop/src/components/DataExplorer.tsx + ResultsList.tsx + WorkspaceShell.tsx (prop onGoToIngest/onClearSearch)
    confianza: alta
  - claim: fmtBytes compartido único (lib/format.ts), re-export desde indices-core; test existente verde sin modificar
    evidencia: desktop/src/lib/format.ts (nuevo), indices-core.ts, MetricsGrid.tsx, KpiCards.tsx; indices-core.test.ts sin cambios
    confianza: alta
  - claim: PersonaPanel propaga error real (onError + vantaErrorMessage, ya no catch(() => {}))
    evidencia: desktop/src/components/memory/MemoryLens.tsx (PersonaPanel)
    confianza: alta
  - claim: Sin window.confirm nuevo; SpaceLens usa inline 2 pasos (SelectionBar armed); ConfirmDiscard inline
    evidencia: grep window.confirm en desktop/src = solo ImportPaste/ImportDrop (pre-existentes, FIND)
    confianza: alta
artefactos: [.opencode/skills/campaign-executor/tasks/UX-POLISH.md]
invariantes: virtualización TanStack intacta; gridKey remount patrón existente; trash/purge choice conservado; tablist/focus MemoryLens intacto
deuda: toggle visibilidad columnas grid (UX-10) → FIND; window.confirm ImportPaste/ImportDrop → FIND (pre-existente)
queda_pendiente: LEAD verifica mecánico y commitea solo archivos desktop/* + task file (NO COMMIT del worker)
```

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Deuda registrada (0) — ningún `unsafe`/hot-path; KpiCards pierde su formatter binario local (deuda eliminada) y gana el compartido decimal (deuda 0). Unificación = -2 implementaciones duplicadas. Sin deuda nueva.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato: build exit 0 + vitest pasa + 3 condiciones específicas (empty states, fmtBytes, PersonaPanel) + no window.confirm nuevo |
| **Commit** | El lead commitea (worker NO commitea — regla del plan); conventional commit; archivos solo desktop/* + task file |
| **Release** | No aplica (batch interno, sin release) — justificado en Notas |

## Herramientas necesarias
- bash (npm run build, npx vitest run)
- Read/Grep directos (CodeGraph auto-sync deshabilitado — lock de otro proceso)

**Skills cargadas (SDP):** frontend-ui-engineering (pulido UI desktop: empty states, a11y, densidades) · code-simplification (unificación fmtBytes 3→1 sin cambiar semántica) · base: campaign-executor, progreso, ponytail (full). SDP: sin candidatos adicionales (cambios = thin props + clases; sin lógica nueva que amerite TDD).

## Investigation Notes
- fmtBytes existe en 3 sitios: indices-core.ts:17 (decimal, testado), MetricsGrid.tsx:7 (decimal idéntico), KpiCards.tsx:16 (binario MiB/KiB). El ticket dice "2 → 1" (no cuenta el 3ro por ser idéntico al canónico). Plan: lib/format.ts (decimal) + re-export en indices-core → test intacto.
- `h-[calc(100dvh-112px)]` duplicado en SpaceLens.tsx:214 y GraphLens.tsx:43 (ambos con min-h-[480px]) → utility CSS `lens-height` en index.css.
- window.confirm en desktop: SpaceLens.tsx:194 (se elimina), ImportPaste.tsx:89 + ImportDrop.tsx:107 (pre-existentes, fuera de scope — FIND).
- `--color-destructive` existe en index.css:95 → text-destructive/bg-destructive/10 disponibles en Tailwind v4.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 12 steps |
| % completado | 0% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — NO aplica: cambios UI-internos desktop, sin trust boundary nuevo (el prop onError propagación no amplía superficie de input; no hay auth/datos/dependencias nuevas). Justificado.
- [ ] **PERFORMANCE** — NO aplica: sin hot paths (el grid ya está virtualizado; solo cambian clases de celda). Justificado.

## Steps

### Step 1: Task file + DISCOVERY
- **Archivos:** `.opencode/skills/campaign-executor/tasks/UX-POLISH.md`
- **Acción:** crear task file con Impacto mapeado (Regla 0), Spec, invariantes, steps
- **Verify:** Read del task file completo
- **Estado:** ✅ COMPLETED

### Step 2: UX-17 — IngestForm onRefresh (grid refresca tras ingest manual)
- **Archivos:** `desktop/src/components/IngestForm.tsx`, `desktop/src/components/layout/WorkspaceShell.tsx`
- **Acción:** prop opcional `onRefresh?: () => void` en IngestForm; llamar `onRefresh?.()` tras ingest exitoso; WorkspaceShell lo pasa como `() => setGridKey((k) => k + 1)`
- **Verify:** `cd desktop && npx tsc --noEmit`
- **Estado:** ✅ COMPLETED

### Step 3: UX-11 — Empty states con acción
- **Archivos:** `desktop/src/components/DataExplorer.tsx`, `desktop/src/components/ResultsList.tsx`, `desktop/src/components/layout/WorkspaceShell.tsx`
- **Acción:** DataExplorer: prop `onGoToIngest?`; estado vacío list → botón "Ir a Ingestar"; vacío search → botón "Limpiar búsqueda" (reset query + fetchFirst list). ResultsList: prop `onClearSearch?`; "No matches." → "Sin coincidencias." + botón limpiar. WorkspaceShell: wire props (scroll al form id ingest-form; setResults(null)).
- **Verify:** `cd desktop && npx tsc --noEmit`
- **Estado:** ✅ COMPLETED

### Step 4: UX-14 — PersonaPanel propaga error real
- **Archivos:** `desktop/src/components/memory/MemoryLens.tsx`
- **Acción:** PersonaPanel acepta `onError`; `.catch(() => {})` → `.catch((err) => onError(vantaErrorMessage(err)))`; render con `onError={actions.onError}`
- **Verify:** `cd desktop && npx tsc --noEmit`
- **Estado:** ✅ COMPLETED

### Step 5: UX-09 — Confirm destructiva unificada (inline 2 pasos)
- **Archivos:** `desktop/src/components/space/SelectionBar.tsx`, `desktop/src/components/space/SpaceLens.tsx`, `desktop/src/components/consolidate/ConfirmDiscard.tsx`
- **Acción:** SelectionBar: estado `armed` interno (botón eliminar arma → muestra "¿BORRAR N?" + ✕, patrón DeleteButton/TrashLens). SpaceLens: quitar window.confirm de handleDelete. ConfirmDiscard: sacar `fixed inset-0 z-50` overlay → caja inline en flujo (conserva 2 pasos + radio papelera/permanente).
- **Verify:** `cd desktop && npx tsc --noEmit` + grep `window.confirm` (sin apariciones nuevas)
- **Estado:** ✅ COMPLETED

### Step 6: UX-10 — Densidad grid MEMORIAS
- **Archivos:** `desktop/src/components/DataExplorer.tsx`
- **Acción:** Payload `max-w-[420px]` → `max-w-[min(24vw,420px)]`; TTL `w-24` → `w-20`. No tocar virtualización (solo clases de celda).
- **Verify:** `cd desktop && npx tsc --noEmit`
- **Estado:** ✅ COMPLETED

### Step 7: UX-12 — RESUMEN sin salto visual (título unificado)
- **Archivos:** `desktop/src/components/home/HomeOverview.tsx`
- **Acción:** extraer header (eyebrow + h1 + refresh) y renderizarlo igual en loading y loaded; loading muestra placeholder bajo el header en vez de card distinta.
- **Verify:** `cd desktop && npx tsc --noEmit`
- **Estado:** ✅ COMPLETED

### Step 8: UX-13 — Banner audit redactado + details técnico
- **Archivos:** `desktop/src/components/activity/ActivityPanel.tsx`
- **Acción:** texto usuario ("La conexión activa no tiene audit log habilitado…") + `<details>` técnico (Unsupported/NativeConnection::open/VantaConfig).
- **Verify:** `cd desktop && npx tsc --noEmit`
- **Estado:** ✅ COMPLETED

### Step 9: UX-15a — MetricsGrid badge err → text-destructive
- **Archivos:** `desktop/src/components/MetricsGrid.tsx`
- **Acción:** ternario de badge: warn → neon/accent (actual), err → `border-destructive bg-destructive/10 text-destructive`.
- **Verify:** `cd desktop && npx tsc --noEmit`
- **Estado:** ✅ COMPLETED

### Step 10: UX-15b — fmtBytes compartido
- **Archivos:** `desktop/src/lib/format.ts` (nuevo), `desktop/src/components/indices/indices-core.ts`, `desktop/src/components/MetricsGrid.tsx`, `desktop/src/components/KpiCards.tsx`
- **Acción:** crear lib/format.ts con fmtBytes decimal (copia de indices-core); indices-core re-exporta (`export { fmtBytes } from "../../lib/format"` — test intacto); MetricsGrid y KpiCards importan de lib/format y eliminan su formatter local.
- **Verify:** `cd desktop && npx vitest run src/indices-core.test.ts`
- **Estado:** ✅ COMPLETED

### Step 11: UX-15c-h — misc restante
- **Archivos:** `desktop/src/components/layout/SplashScreen.tsx`, `desktop/src/components/layout/WorkspaceShell.tsx`, `desktop/src/components/ResultsList.tsx`, `desktop/src/components/indices/IndicesLens.tsx`, `desktop/src/components/activity/ActivityPanel.tsx`, `desktop/src/components/lens/retrieval/RetrievalLens.tsx`, `desktop/src/components/space/SpaceLens.tsx`, `desktop/src/components/graph/GraphLens.tsx`, `desktop/src/index.css`
- **Acción:** splash saltable por teclado (tabIndex/role/onKeyDown + hint); notice bar con ✕ enfocable; microcopy ES/EN (ResultsList, WorkspaceShell "Stored N record(s)", IndicesLens "metrics unavailable"); buttons press en ActivityPanel granularity; skeleton loading en IndicesLens; RetrievalLens header sin jerga (meta/subtitle usuario); utility `.lens-height` en index.css + reemplazar `h-[calc(100dvh-112px)] min-h-[480px]` en SpaceLens/GraphLens.
- **Verify:** `cd desktop && npx tsc --noEmit`
- **Estado:** ✅ COMPLETED

### Step 12: Verify contrato completo
- **Archivos:** —
- **Acción:** `cd desktop && npm run build` (exit 0) + `cd desktop && npx vitest run` (pasa)
- **Verify:** ambos comandos exit 0
- **Estado:** ✅ COMPLETED

### Step 13: Task file + RESULTADO
- **Archivos:** `.opencode/skills/campaign-executor/tasks/UX-POLISH.md`
- **Acción:** actualizar steps ✅ + Context Save Point; devolver bloque RESULTADO (sin commit — regla del plan)
- **Verify:** Read del task file
- **Estado:** ✅ COMPLETED

## Dependencias
- Task 1 (UX-A11Y) — ya ✅ COMPLETED, mismo directorio desktop (no edita los mismos archivos excepto DataExplorer/ResultsList/MemoryLens: verificar no pisar sus cambios)

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta el lead/vanta-review ANTES de commitear (worker NO commitea). Este task file queda listo para el gate.

- **Revisor:** vanta-review (lead lo invoca) — verifica diff contra este task file
- **Enfoque:** unificación confirm destructiva (¿se preservó trash/purge?) · fmtBytes (¿1 sola impl?) · UX-17 (¿remount no rompe import?)
- **Cómo se probó:** `cd desktop && npm run build` + `npx vitest run` reales (no auto-reporte)
- **Veredicto:** pendiente (lead)

## Notas
- UX-10 toggle de visibilidad de columnas NO se implementa (REGLA CRÍTICA: "ajustar densidades sin romper el grid") → candidato FIND-*.
- window.confirm en ImportPaste/ImportDrop queda pre-existente (anotado como FIND por UX-A11Y).
- KpiCards cambia "MiB/KiB" → "MB/KB" (unificación pedida; documentado en Spec #2).
- Release: no aplica (batch interno, sin release) — justificado.
- CodeGraph auto-sync deshabilitado (lock otro proceso) → lectura directa con Read/Grep (hecho).

## Context Save Point (cierre 2026-08-25T15:00)

- **Steps 1-13 ✅ COMPLETED.** Todos los cambios implementados y verificados.
- **Verify contrato (Step 12):** `cd desktop && npm run build` → **exit 0** (tsc + vite, 2863 modules, 12.6s; warning chunk GraphLens 967KB pre-existente). `cd desktop && npx vitest run` → **68/68 passed** (11 files, 23.5s). `indices-core.test.ts` verde sin modificar (re-export de fmtBytes).
- **Archivos tocados (SOLO desktop/src + task file):** DataExplorer.tsx, IngestForm.tsx, KpiCards.tsx, MetricsGrid.tsx, ResultsList.tsx, activity/ActivityPanel.tsx, consolidate/ConfirmDiscard.tsx, graph/GraphLens.tsx, home/HomeOverview.tsx, indices/IndicesLens.tsx, indices/indices-core.ts, layout/SplashScreen.tsx, layout/WorkspaceShell.tsx, lens/retrieval/RetrievalLens.tsx, memory/MemoryLens.tsx, space/SelectionBar.tsx, space/SpaceLens.tsx, index.css, **lib/format.ts (nuevo)**.
- **Grep window.confirm:** solo ImportPaste.tsx:89 + ImportDrop.tsx:107 (pre-existentes, FIND). SpaceLens ya no usa confirm nativo.
- **WIP guard:** campaign_update_task_state bloqueado (FIND-11 in-progress por otro agente del batch — one-task-at-a-time). El lead marca UX-POLISH completed tras verificar/commitear.
- **git status:** el worktree tiene cambios de otros agentes (AGT-04, TIR-08, FIND-11, assets, docs/plans) — NO incluirlos en el commit de UX-POLISH. Commit del lead = solo `desktop/src/**` de este diff + task file.
- **Deuda registrada:** 0 (unificación = -2 formatters duplicados). FINDs: toggle visibilidad columnas grid; window.confirm ImportPaste/Drop.
- **Handoff lead:** verificar `cd desktop && npm run build` + `npx vitest run` mecánicamente, commitear solo archivos desktop/* de UX-POLISH, marcar completed en plan file.