# DESKTOP-37: Lente MEMORIA (UI) — escenas/persona/skills/log con heat/diff/timeline

## Metadata
- **Plan file:** docs/Backlog.md (Phase 12)
- **Creado:** 2026-08-24
- **last-synced:** 2026-08-24T12:30:00
- **Estado:** ✅ COMPLETADA

## Impacto mapeado (Regla 0)
- **Leídos completos:** `desktop/src/vanta.ts` (sección Memory observability L635-748), `desktop/src/components/layout/WorkspaceShell.tsx` (1043L), `desktop/src/components/inspector/Inspector.tsx`, `desktop/src/components/home/HomeOverview.test.tsx` (patrón test bridge-mock), `package.json` (scripts/vitest).
- **Referencias hacia dentro:** WorkspaceShell importa lentes estáticas/lazy y monta Inspector vía `selected: InspectorSelection {record: MemoryRecord, score}`; `Surface` union en L73 alimenta sidebar/palette/prefs.
- **Referencias entrantes:** ninguna hacia `components/memory/` (directorio nuevo). CommandPalette navega surfaces por string → extender la union es compatible.
- **Hacia afuera:** consumo read-only de `memorySceneList/memorySceneRead/memoryPersonaGet/memorySkillList/memoryGenlogQuery/get`. NO se tocan `vanta.rs` ni src-tauri (sin cambios Rust → sin `cargo check`).
- **Decisiones:** (1) Sidebar es inline en WorkspaceShell (no existe Sidebar.tsx) → Step 2 solo toca WorkspaceShell. (2) Ponytail: 1 archivo `MemoryLens.tsx` con 4 sub-paneles internos en vez de 5 archivos. (3) Inspector solo para datos que SON records: genlog con `anchor_id` → `get()` real → `onOpenRecord`; escenas/skills/persona usan detalle inline (no son VantaMemoryRecord; sintéticos contaminarían vantaPut del Inspector).
- **Veredicto:** cambio aditivo de bajo riesgo; única superficie compartida es WorkspaceShell (preservar SETTINGS/palette/tooltips/CRUD namespaces sin tocar).

## Blast Radius
Callers: desktop/src/components/memory/* (nuevos), desktop/src/components/layout/WorkspaceShell.tsx (sidebar + lente), desktop/src/vanta.ts
Callees: DESKTOP-36 (bridge Tauri vanta-memory)
Implicaciones: Sexta superficie del Studio que visualiza el diferenciador del producto (hoy invisible)

## Spec
N/A — feature UI con contrato mecánico

## Contrato
`cd desktop && npm run build`; ver escenas/persona/skills/log de una sesión sembrada con `vanta-seed`; click → Inspector cuando el dato sea un record

## Herramientas
- cargo-mcp, rust-analyzer-mcp, codegraph

## Steps
### Step 1: Crear componentes lente MEMORIA
- **Archivos:** `desktop/src/components/memory/MemoryLens.tsx` (nuevo), `desktop/src/components/memory/ScenesPanel.tsx`, `desktop/src/components/memory/PersonaPanel.tsx`, `desktop/src/components/memory/SkillsPanel.tsx`, `desktop/src/components/memory/GenlogPanel.tsx`
- **Acción:** (1) ScenesPanel: lista escenas ordenadas por heat (soft-delete visible), click → detalle. (2) PersonaPanel: snapshot persona con diff entre versiones (selector versión). (3) SkillsPanel: skills versionadas (timeline por content-hash), click → ver código/versión. (4) GenlogPanel: timeline generation-log filtrable por capa L1/L2/L3. Usar design system Studio (Tailwind manga/linocut, TanStack Table/Virtual)
- **Verify:** `cd desktop && npm run build`

### Step 2: Integrar en WorkspaceShell (sidebar + superficie central)
- **Archivos:** `desktop/src/components/layout/WorkspaceShell.tsx`, `desktop/src/components/layout/Sidebar.tsx`
- **Acción:** Añadir "MEMORIA" como 6ta lente en sidebar (icono cerebro/neurona). Al click: carga MemoryLens en superficie central. Contexto: sesión activa (de vanta-seed o selector)
- **Verify:** Navegación sidebar → MemoryLens renderiza sin errores

### Step 3: Click → Inspector para records
- **Archivos:** `desktop/src/components/memory/*.tsx`, `desktop/src/components/inspector/Inspector.tsx`
- **Acción:** Cuando dato en MemoryLens sea un `VantaMemoryRecord` (scene node, skill version, genlog entry), click → abre Inspector (master-detail, DESKTOP-06) con tabs General/Payload/Metadata/Vector
- **Verify:** Click en record → Inspector abre con datos correctos

### Step 4: Test E2E con vanta-seed
- **Archivos:** `desktop/src/components/memory/*.test.tsx` (extender DESKTOP-26)
- **Acción:** Seed DB con `vanta-seed`. Verificar: escenas listadas con heat, persona diff visible, skills timeline por hash, genlog filtro L1/L2/L3 funcional, Inspector abre records
- **Verify:** Test manual completo funcional

## Dependencias
- DESKTOP-36 (bridge Tauri vanta-memory) — BLOQUEANTE
- DESKTOP-06 (Inspector master-detail) — ya completada
- DESKTOP-28 (design system unificado) — complementaria

## Notas
- DoD: ver escenas/persona/skills/log de una sesión sembrada con `vanta-seed`; click → Inspector cuando el dato sea un record ✅ (2026-08-24)
- Esfuerzo 🔴 (complejo UI nueva), Prio 🟠 (diferenciador producto)
- Depende de DESKTOP-36 completado

## Context Save Point (cierre)
- **Implementado:** `desktop/src/components/memory/MemoryLens.tsx` (1 archivo, 4 sub-paneles internos — ponytail) + `MemoryLens.test.tsx` (7 tests) + integración en `WorkspaceShell.tsx` (Surface `"memoria"`, SideButton ◉, bloque central, import estática). Sidebar inline en WorkspaceShell — NO existe Sidebar.tsx.
- **Inspector:** solo genlog entries con `anchor_id` abren el Inspector vía `get()` real (`onOpenRecord` del shell). Escenas/skills/persona = detalle inline (no son VantaMemoryRecord; un record sintético sería target basura del vantaPut del Inspector).
- **Diff persona:** sin API de historial (deuda DESKTOP-36) → diff contra última snapshot vista, persistida en `localStorage["vanta-persona-last:<session>"]`.
- **Skills timeline:** agrupadas por nombre desde `memorySkillList`, versiones asc por `updated_at_ms`, hash corto hex de `content_hash`; sin skill_versions/skill_restore (no invocados).
- **Verify:** `cd desktop && npm run build` ✅ · `cd desktop && npm test` ✅ 64/64 (57 previas + 7 nuevas). Rust intacto → cargo check n/a. Sin commit (regla de la tarea).
- **Deuda:** diff persona es local (se pierde al limpiar storage); compaction_report y restore de skills siguen sin backing API (DESKTOP-36).