# UX-A11Y — UX-02 + UX-03 + UX-04 + UX-06 + UX-07 + UX-08 (grupo a11y desktop)

> **Campaign:** 6a6c322a-6a6a-4d17-9b34-3166181cbc4a · **Plan:** docs/plans/2026-08-25-batch-desktop-ux-core.md
> **Estado:** ⏳ IN PROGRESS · **Effort:** 🟡 · **Prioridad:** 🟡
> **Contrato:** `cd desktop && npm run build` exit 0 + `npx vitest run` pasa; keyboard: Enter/Espacio abre Inspector desde grid; focus trap conectado en ambos modales; contraste texto-neon ≥4.5:1 (o token accent-text); tabs ARIA con tablist + flechas
> **NO COMMIT** — el lead verifica mecánico y commitea por tarea.

## Impacto mapeado (Regla 0) — verificada en DISCOVERY (archivos leídos completos)

| Archivo | Refs hacia dentro (imports) | Refs entrantes | Veredicto |
|---|---|---|---|
| `desktop/src/index.css` | tailwind, fonts, tokens, utilities | importado por App/main (vía `@import "tailwindcss"` + main.tsx) | Añadir token `--color-accent-text` (aditivo, no rompe) |
| `components/DataExplorer.tsx` | vanta, undo, favorites, batchSelection, copy, export | WorkspaceShell (`<DataExplorer>` en surface memorias) | Fila `<tr>` keyboard (tabIndex/onKeyDown/aria-selected) sin tocar virtualizer |
| `components/ResultsList.tsx` | vanta (SearchResult) | WorkspaceShell (resultados de búsqueda global) | `<li>` keyboard + score accent-text |
| `components/ingest/ImportPaste.tsx` | vanta, parseImport | WorkspaceShell (lazy, botón IMPORT CSV/JSON) | Conectar useModalFocus + borrar useEffect Escape |
| `components/ingest/ImportDrop.tsx` | vanta, parseImport | WorkspaceShell (lazy, botón IMPORT ARCHIVO) | Conectar useModalFocus + borrar useEffect Escape |
| `components/ingest/useModalFocus.ts` | react | (debe) ImportPaste/ImportDrop | Ya completo — solo conexión |
| `components/IngestForm.tsx` | vanta | WorkspaceShell (surface memorias) | UX-04: labels visibles, error inline, confirm inline |
| `components/home/HomeOverview.tsx` | vanta, lucide | WorkspaceShell (surface resumen) | text-neon texto pequeño → accent-text |
| `components/layout/WorkspaceShell.tsx` | ~25 imports (lenses, stores, bridge) | App (root) | text-neon texto pequeño → accent-text |
| `components/graph/GraphLens.tsx` | r3f, GraphScene, IqlConsole, useGraphData | WorkspaceShell (lazy, surface iql) | Canvas role=img + aria-label + lista sr-only nodos |
| `components/space/SpaceLens.tsx` | regl-scatterplot, useProjection, SelectionBar | WorkspaceShell (lazy, surface espacio) | Canvas role=img + aria-label + lista sr-only puntos |
| `components/inspector/Inspector.tsx` | GeneralTab/MetadataTab/VectorTab/PayloadTab/HistorialTab, shared | WorkspaceShell (lazy, master-detail) | Tabs ARIA tablist + flechas; text-neon → accent-text |
| `components/memory/MemoryLens.tsx` | vanta, LensShell | WorkspaceShell (surface memoria) | Tabs ARIA tablist + flechas; text-neon → accent-text |
| `components/MetricsGrid.tsx` | vanta, useMetricsPoll | WorkspaceShell (surface resumen) | badge salud text-neon → accent-text |
| `desktop/src/hooks/useTablist.ts` (NUEVO, si aplica) | react | Inspector + MemoryLens | Decisión: INLINE por componente (ponytail: 10 líneas, no vale un archivo) |

Veredicto global: cambios aditivos de accesibilidad sobre 14 archivos UI (sin API pública, sin hot path, sin bindings). No se rompe ningún import entrante.

## Spec (decisiones por evidencia)

- **UX-06 token:** ✅ decidido-por-evidencia (WCAG 1.4.3 AA ≥4.5:1; ratio medido: #FF5500 sobre #FBF9F5 = 3.07:1 ❌; #C24000 sobre crema = 4.99:1 ✅, sobre paper #F2EDE2 = 4.82:1 ✅; dark: #FF5500 sobre #0a0a0a = 6.09:1 ✅ → `--accent-text: #C24000` light / `#FF5500` dark; identidad neón preservada como bg/borde/fill). Token nuevo en `@theme inline` siguiendo patrón `var()` existente (--color-cream).
- **UX-06 alcance:** ✅ decidido-por-evidencia (auditoría P34 + criterio: texto pequeño significativo sobre fondo claro en los 12 archivos clave + IngestForm; glifos decorativos aria-hidden y neón como bg/borde se conservan — WCAG 1.4.11 no-text 3:1 se cumple con #FF5500 3.07:1).
- **UX-02 DataExplorer:** ✅ decidido-por-evidencia (contrato del task: tabIndex=0 + onKeyDown Enter/Espacio + aria-selected; guard `e.target === e.currentTarget` para que Enter en botones hijos no abra Inspector; NO se toca la estructura del virtualizer — solo atributos del `<tr>`). Estado local `openKey` refleja master-detail (aria-selected válido en role=row).
- **UX-02 ResultsList:** ✅ decidido-por-evidencia (patrón listbox: `<ol role="listbox">` + `<li role="option">` hace válido `aria-selected`; tabIndex=0 + Enter/Espacio abren Inspector).
- **UX-03:** ✅ decidido-por-evidencia (hook useModalFocus.ts ya implementa trap+Escape+restore — ref: useModalFocus.ts:15-50; conexión con ref en el overlay; borrar useEffect de Escape duplicado porque el hook lo cubre con captura, ImportPaste.tsx:66-74).
- **UX-04:** ✅ decidido-por-evidencia (WCAG 3.3.2: labels visibles wrapping `<label>`; error inline `role="alert"` anclado al form; confirm inline de 2 pasos consistente con DeleteButton de DataExplorer.tsx:221-256, eliminando window.confirm).
- **UX-07:** ✅ decidido-por-evidencia (WAI-ARIA APG tabs: role=tablist + aria-orientation + role=tab + roving tabindex (activo 0, resto -1) + ArrowLeft/Right (Home/End opcional APG) + id/aria-controls + role=tabpanel. MemoryLens.test.tsx ya consulta `getByRole("tab")` — se conserva role=tab).
- **UX-08:** ✅ decidido-por-evidencia (WCAG 1.1.1: role="img" + aria-label descriptivo en el canvas + lista alternativa sr-only de nodos/puntos; cap 200 para scatter — evitar spam SR; GraphNode.label es el display name — vanta.ts:436-444).

## Steps

1. ✅ **index.css** — token `--color-accent-text` (light #C24000 / dark #FF5500) + comentario FIND-24
2. ✅ **ImportPaste + ImportDrop** — conectar useModalFocus, borrar useEffect Escape
3. ✅ **IngestForm** — labels visibles, error inline, confirm inline (UX-04)
4. ✅ **DataExplorer** — fila keyboard (tabIndex/onKeyDown/aria-selected) + "Cargando más…" accent-text
5. ✅ **ResultsList** — listbox keyboard + score accent-text
6. ✅ **Inspector** — tabs tablist+flechas+tabpanel; text-neon → accent-text
7. ✅ **MemoryLens** — tabs tablist+flechas+tabpanel; text-neon → accent-text
8. ✅ **HomeOverview + WorkspaceShell + MetricsGrid** — text-neon → accent-text
9. ✅ **GraphLens + SpaceLens** — canvas role=img + lista sr-only (UX-08)
10. ✅ **Verify** — `cd desktop && npm run build` ✅ + `npx vitest run` ✅ (68/68)

## Context Save Point (2026-08-25, cierre)

- **Verify:** `cd desktop && npm run build` → exit 0 (tsc + vite build, 16.5s; warning chunk size pre-existente de GraphLens/three no relacionado). `npx vitest run` → 11 files / 68 tests ✅. MemoryLens.test.tsx (getByRole tab) sigue verde.
- **Contrato cumplido:** Enter/Espacio abre Inspector (DataExplorer rows + ResultsList listbox); useModalFocus conectado en ImportPaste/ImportDrop (Escape duplicado eliminado); token `--color-accent-text` definido (light #C24000 4.99:1 ✅ / dark #FF5500 6.09:1 ✅) aplicado a texto pequeño; tabs con role="tablist" + aria-orientation + roving tabindex + flechas ←/→ en Inspector y MemoryLens.
- **NO COMMIT** (worker prohibido) — el lead commitea SOLO `desktop/src/components/*` (13 archivos) + `desktop/src/index.css` + `.opencode/skills/campaign-executor/tasks/UX-A11Y.md`. ⚠️ El worktree contiene cambios de sub-agentes paralelos (MOD-15 vantadb-server, AGT-04 commands, FIND-17 docs) — NO incluirlos en este commit.
- **Server WIP guard:** `campaign_update_task_state("completed")` bloqueado por one-task-at-a-time (AGT-04/MOD-15 in-progress) — el orquestador/lead debe marcarla completed tras verify.
- **progreso:** NO ejecutado (lo corre el lead al commitear; no tocar Backlog/docs-avance sin estado server completo).
- **Deuda/colaterales:** window.confirm nativo persiste en ImportPaste/ImportDrop (fuera de scope UX-04=IngestForm) — candidato FIND. text-neon restante en archivos tocados es decorativo (glyphs aria-hidden, ✓ con aria-label, hover) — intencional.