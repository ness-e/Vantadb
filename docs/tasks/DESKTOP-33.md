# DESKTOP-33: CONSOLIDAR — merge + delete reales (detectar→revisar→merge/delete sin salir de la lente)

## Metadata
- **Plan file:** docs/Backlog.md (Phase 12)
- **Creado:** 2026-08-24
- **last-synced:** 2026-08-24T00:00:00
- **Estado:** ⬜ PENDING → ✅ COMPLETADA (2026-08-24)

## Impacto mapeado (Regla 0)
- Leídos completos: `ConsolidateLens.tsx`, `consolidate-core.ts`, `undo.ts`, `NamespaceDialog.tsx` (patrón 2 pasos), `vanta.ts` (get/remove/vantaPut), `consolidate-core.test.ts`.
- Referencias entrantes a ConsolidateLens: WorkspaceShell (lazy import) — props sin cambios.
- Referencias salientes nuevas: `store/undo` (softDelete durable DESKTOP-30), `vanta.get/remove`. Styling Tailwind preservado.
- Veredicto: cambio aditivo en `components/consolidate/` + tests; sin tocar core Rust ni otros lenses.

## Blast Radius
Callers: desktop/src/components/ConsolidateLens.tsx, desktop/src/components/consolidate-core.ts
Callees: src/sdk/api.rs (put, delete_by_filter), desktop/src/vanta.ts
Implicaciones: Flujo completo detectar duplicados → revisar lado a lado → merge campo a campo → delete del supersedido

## Spec
N/A — feature UX con contrato mecánico

## Contrato
`cd desktop && npm run build`; flujo detectar→revisar→merge/delete completo sin salir de la lente

## Herramientas
- cargo-mcp, rust-analyzer-mcp, codegraph

## Steps
### Step 1: ✅ Extender ConsolidateLens con merge manual campo a campo
- **Hecho:** `defaultSources()` + `mergeFields()` en consolidate-core.ts (pre-fill dominante, fuente A/B por campo, claves ausentes omitidas). MergeEditor inline en ConsolidateLens: dominante elegible (radio A/B), toggle por campo, preview, Guardar → `vantaPut` merged + papelera del supersedido vía `undoStore.softDelete` (snapshot completo con `get()` para restaurar vector/ttl). Batch: checkboxes por par + toolbar (superar a→b / b→a en lote + descartar superados de la selección).
- **Verify:** `cd desktop && npm run build` ✅

### Step 2: ✅ Confirmación delete del supersedido
- **Hecho:** `ConfirmDiscard.tsx` (patrón NamespaceDialog): paso 1 aviso + elección "Mover a papelera" (default) vs "Eliminar permanente", paso 2 tipear objetivo exacto (`ns/id` o CONFIRMAR en lote). Undo vía Ctrl+Z (undoStore).
- **Verify:** build ✅ — flujo merge → confirmar delete → undo disponible (soft-delete durable)

### Step 3: ✅ Feedback y estado visual
- **Hecho:** badges existentes preservados ("✕ superado por"), progress textual (`marcando i/n`, `eliminando i/n`) en barra de controles, notices por acción (mecanismo onNotice/onError del shell = toasts), lista reactiva tras merge/delete (pares y registros se filtran en el mismo setState).
- **Verify:** `cd desktop && npm test` ✅ 64/64

## Dependencias
- DESKTOP-32 (delete_by_filter Tauri command)
- DESKTOP-30 (papelera + undo durable)
- consolidate-core.ts ya existe (kNN textual)

## Notas
- DoD: flujo detectar→revisar→merge/delete completo sin salir de la lente
- Merge manual campo-a-campo (pre-fill desde dominante) → put merged → delete superseded
- Acción batch sobre selección múltiple