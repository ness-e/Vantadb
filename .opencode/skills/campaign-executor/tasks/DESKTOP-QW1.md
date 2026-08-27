# TASK DESKTOP-QW1: CommandPalette — sincronizar union completa Surface (H-02)

## Metadata
- **Plan file:** `docs/plans/2026-08-25-research-desktop-quickwins.md`
- **Creado:** 2026-08-26T00:00
- **last-synced:** 2026-08-26T01:20
- **Estado:** ✅ COMPLETED
- **Tipo detectado:** desktop (frontend-ui-engineering, source-driven-development)
- **Workflow:** refactor (audit → migrate → cleanup → verify → review → accept → close)
- **Task file:** `.opencode/skills/campaign-executor/tasks/DESKTOP-QW1.md`

## Blast Radius
- `PaletteSurface` (`desktop/src/components/palette/CommandPalette.tsx:27`) — type alias duplicado de `Surface`; 1 consumer interno (`CommandPaletteProps.onNavigate`), 1 caller externo (`WorkspaceShell.tsx:1083` via `onNavigate`). No tests directos.
- `Surface` (`desktop/src/components/layout/WorkspaceShell.tsx:83`) — union canónica (12 valores); consumidores: `SideButton` active checks, `surface` state, `CommandPalette` onNavigate prop, `MemoryLens`/`ProxyDashboard`/`Settings` surfaces.
- `CommandPalette` component — lazy en WorkspaceShell (69), montado en paletteOpen state; 8 grupos Command (Navegación, Lentes, Favoritos, Namespaces, Acciones, Historial).
- `WorkspaceShell` — shell único; sidebar + topbar + central surfaces + inspector + palette + help.
- **Implicaciones:** cambio aditivo/verify-only. Si falta algún miembro (memoria/proxy/ajustes) → agregar PaletteItem + keyword + union. No toca WAL/vector/storage, no rompe API pública. Blast radius 2 archivos + indirectos (tests desktop).

## Impacto mapeado (Regla 0)
- **Archivos leídos completos (antes de editar):**
  - `desktop/src/components/palette/CommandPalette.tsx` (529 líneas completas)
  - `desktop/src/components/layout/WorkspaceShell.tsx` (1143 líneas completas)
  - `desktop/package.json` (56 líneas)
  - `docs/plans/2026-08-25-research-desktop-quickwins.md` (37 líneas)
  - `docs/reviews/archive/research-desktop-prod-20260825.md` (hallazgo H-02)
- **Referencias hacia dentro (qué importa este archivo):**
  - `CommandPalette.tsx` → `cmdk` (Command, useCommandState), `lucide-react` (icons), `../../vanta` (list, vantaErrorMessage), `../../store/favorites` (Favorite)
  - `WorkspaceShell.tsx` → `CommandPalette` (lazy), `MemoryLens`, `ProxyDashboard`, `Settings`, `RetrievalLens`, `IndicesLens`, `ConsolidateLens`, `GraphLens`, `SpaceLens`, `ActivityPanel`, `Inspector`, `FiltersBuilder`
- **Referencias entrantes (qué depende de lo que cambio):**
  - `WorkspaceShell` consume `PaletteSurface` via `onNavigate` callback tipado; `cmdk` filtra por `value`/`keywords`
  - Sidebar `SideButton` es fuente de verdad visual; palette debe espejarla
  - Tests desktop: `vitest run` (69 specs aprox), `e2e/daud01-temas.spec.ts`, `e2e/flujo-critico.spec.ts` — ninguno cubre palette directamente, pero build rompe si union diverge
- **Veredicto de impacto:** BAJO — 2 archivos UI, sin backend/Rust, sin hot path, sin concurrencia. Si union diverge → TS error en `onNavigate` (type mismatch) detectado por `tsc`. Fix es agregar 1-3 PaletteItems + union member. Reversible en 1 commit.

## Contrato
CommandPalette sincroniza union completa Surface: verificar que `PaletteSurface` incluye `memoria`/`proxy`/`ajustes` y que `CommandPalette.tsx` las expone todas con `PaletteItem` + keywords. Si falta alguna, agregar. `cd desktop && npm run build` y `npm test` verde; no regresión en palette (grupos/lentes/favoritos/historial intactos).

Verificación mecánica:
1. `campaign_verify_cmd("cd desktop && npm run build")` — tsc + vite build verde
2. `campaign_verify_cmd("cd desktop && npm test")` — vitest run verde
3. Diff check: `PaletteSurface` set === `Surface` set (memoria/proxy/ajustes presentes)
4. Cierre full: `cargo fmt --check` (no-op desktop) + `cargo clippy` (si aplica workspace)

## Herramientas
- codegraph_explore (blast radius) — ✅
- campaign_detect_task_type / campaign_load_skills — ✅
- terminal: `cd desktop && npm run build`, `cd desktop && npm test`, `campaign_verify_cmd`
- Read/Edit (solo si falta superficie)

## Skills
- campaign-executor, progreso, ponytail (base obligatoria)
- frontend-ui-engineering (detectado por tipo desktop)
- source-driven-development (detectado por tipo desktop)
- SDP discovery: keywords `palette/surface/command/desktop/tauri/workspace` → SKILLS-MANIFEST grep solo hit `ui-ux-pro-max` (palette) y `color-expert` (palette generation) — ambos irrelevantes para CommandPalette sync. **SDP: sin candidatos adicionales.**

## Spec
N/A — tarea de verificación/sincronía, no feature-add con símbolos públicos nuevos. No agrega `pub fn` / endpoint / tool. Solo alinea type union existente. Gate spec-first no aplica (no es feature-add con lógica nueva).

## Steps

### Step 1: Auditoría de sincronía Surface ↔ PaletteSurface ✅ DONE
- **Archivos:** `desktop/src/components/palette/CommandPalette.tsx:27-39`, `desktop/src/components/layout/WorkspaceShell.tsx:83`
- **Acción:** Comparar unions. WorkspaceShell `Surface` = 12 valores: resumen, memorias, papelera, actividad, retrieval, indices, consolidar, iql, espacio, memoria, proxy, ajustes. CommandPalette `PaletteSurface` = mismos 12 valores (líneas 27-39). Verificar cada uno tiene `PaletteItem` en el grupo Lentes/Navegación: resumen/mem­orias/papelera (Navegación 201-218), actividad/retrieval/indices/iql/consolidar/espacio/memoria/proxy/ajustes (Lentes 221-287). `memoria` (265-271), `proxy` condicional `proxyConfigured` (272-280), `ajustes` (281-287) presentes. Keywords ES/EN presentes.
- **Verify:** `node -e "diff check"` manual + `campaign_verify_cmd("cd desktop && npx tsc --noEmit")` implícito en build
- **Estado:** ✅ DONE — unions idénticas, palette expone las 3 superficies requeridas (memoria/proxy/ajustes). No se requirió edición.
- **Gate D:** NO disparado — blast 2 archivos, sin símbolos públicos nuevos, sin hot path, contrato claro.

### Step 2: Build + Test verde (contrato mecánico) ✅ DONE
- **Archivos:** `desktop/package.json`, `desktop/tsconfig.json`, `desktop/vite.config.ts`
- **Acción:** Ejecutar `cd desktop && npm run build` (tsc + vite) y `cd desktop && npm test` (vitest). Capturar output. Si falla → diagnosticar (systematic-debugging). Ponytail: no instalar deps nuevas si ya existen; `npm ci` solo si node_modules corrupto.
- **Verify:** `campaign_verify_cmd("npm --prefix desktop run build")` ✅ (8.04s, 2863 modules, dist 967kB GraphLens chunk) + `campaign_verify_cmd("npm --prefix desktop test")` ✅ (11 files, 69 tests, 21.77s)
- **Estado:** ✅ DONE — build tsc+vitez verde, tests 69/69 verde. Evidencia: campaign_verify_cmd passed ambos.

### Step 3: Cierre — verify full + commit + progreso ✅ DONE
- **Acción:** Actualizar plan file (Wave 1 Task 1 → ✅), `campaign_memory_write` lesson, `campaign_update_task_state(completed)`, `git add` + commit `feat(desktop): DESKTOP-QW1 — CommandPalette union sync (H-02)`, `campaign_diagnose_pipeline`, `skill progreso` (Trigger 1).
- **Verify:** `campaign_verify_cmd("cargo fmt --check")` ✅ (2.9s) + `campaign_verify_cmd("npm --prefix desktop run build")` ✅ re-check (8.04s) + `npm --prefix desktop test` ✅ (69 tests)
- **Estado:** ✅ DONE

## Dependencias
- Ninguna (Wave 1 Task 1 es independiente; Tasks 2-5 tocan archivos disjuntos topbar/help/statusReport)

## Notas
- H-02 ya estaba resuelta en código (commit previo trajo memoria/proxy/ajustes a PaletteSurface). Esta tarea es verificación mecánica + cierre del gate H-02.
- Duplicación Surface/PaletteSurface es intencional (evita ciclo lazy) — documentado en línea 24-26. Mantener sincronía manual; si se añade nueva surface, actualizar ambos.
- `proxy` es condicional en palette y sidebar (proxyConfigured) — contrato lo respeta.

## Context Save Point
- **Fecha:** 2026-08-26T01:20
- **Branch:** develop
- **CI pendiente:** ninguno — build 2863 modules + 69 tests verde, fmt check verde
- **Decisiones:** Unions ya sincronizadas (PaletteSurface === Surface, 12 valores); `proxy` condicional respetado; duplicación intencional documentada (línea 24-26). Verify-only, sin edición de source.
- **Problemas conocidos:** ninguno — H-02 cerrado.
- **Próxima tarea:** DESKTOP-QW2 (Handler keydown F1/F2) — WorkspaceShell ya tiene handler F1/F2 (líneas 364-369), Task 2 verificará HelpPanel integration.
