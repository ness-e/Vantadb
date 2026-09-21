# DESKTOP-32: CRUD de namespaces — crear/renombrar/borrar con confirmación + undo

## Metadata
- **Plan file:** docs/Backlog.md (Phase 12)
- **Creado:** 2026-08-24
- **last-synced:** 2026-08-24T00:00:00
- **Estado:** ✅ COMPLETO

## Impacto mapeado (Regla 0)
- **Leídos completos:** WorkspaceShell.tsx (963L), vanta.ts (725L), store/undo.ts (245L), store/undo.test.ts (155L), src-tauri/src/commands/data.rs (250L).
- **Referencias entrantes:** undo.ts ← WorkspaceShell/DataExplorer/TrashLens/SpaceLens/favorites; vanta.ts ← todos los componentes; tests que mockean ../vanta: undo.test.ts, useMetricsPoll.test.tsx, HomeOverview.test.tsx (estos dos últimos NO importan undo → no se rompen).
- **Referencias salientes:** vantaPut/listPage/remove/ingestBatch/namespaceStats → comandos Tauri existentes (data.rs).
- **Veredicto:** cero cambios Rust — el bridge ya expone todo lo necesario. Rename usa `ingestBatch` (preserva embedding) + `remove` por key; delete usa `softDeleteBatch` (papelera + undo gratis). Sonner NO instalado → feedback con onNotice/onError existente.

## Blast Radius
Callers: desktop/src/components/layout/WorkspaceShell.tsx (sidebar), desktop/src/vanta.ts
Callees: src/sdk/api.rs (count, delete_by_filter), Tauri commands
Implicaciones: Sidebar lista namespaces con acciones: crear vacío, renombrar (copy+delete batch), borrar con confirmación 2 pasos + papelera

## Spec
N/A — feature CRUD con contrato mecánico

## Contrato
`cd desktop && npm run build`; crear/renombrar/borrar ns desde sidebar con feedback y undo donde aplique

## Herramientas
- cargo-mcp, rust-analyzer-mcp, codegraph

## Steps
### Step 1: Comando Tauri para CRUD namespaces — ✅ (resuelto SIN cambios Rust)
- **Archivos:** `desktop/src/vanta.ts`
- **Acción real:** el bridge ya exponía todo — rename usa `ingestBatch` (preserva embedding/metadata/ttl) + `remove` por key; delete reusa `softDeleteBatch` del undo store. Se agregó `listAll(namespace)` (paginación completa) y `createNamespace()` (put key reservada `NS_META_KEY`). `delete_by_filter` NO se usó: sin filtro que matchee todo el ns, y el undo necesita los records completos igualmente.
- **Verify:** `cd desktop && npm run build` ✅

### Step 2: UI en Sidebar (WorkspaceShell) — ✅
- **Archivos:** `desktop/src/components/layout/WorkspaceShell.tsx`, `desktop/src/components/layout/NamespaceDialog.tsx` (nuevo)
- **Acción:** header Namespaces con botón "+" (disabled sin backend). Por namespace: ✎ renombrar / 🗑 borrar. Modal único 3 modos; borrado = 2 pasos (aviso → tipear nombre exacto, input arranca vacío). Feedback con onNotice/onError (patrón existente — Sonner NO estaba instalado).
- **Verify:** `npm run build` ✅

### Step 3: Undo para rename/delete — ✅
- **Archivos:** `desktop/src/store/undo.ts`, `desktop/src/store/undo.test.ts`
- **Acción:** `renameNamespace(from,to)` (copia→borra→entry reverse `move`) y `deleteNamespace(ns)` (reusa `applySoftDelete`, extraído de softDeleteBatch). Ctrl+Z revierte ambos. 4 tests nuevos.
- **Verify:** `npm test` — 57 passed (9 files)

## Dependencias
- VS-CORE-02 (contadores por namespace) — ya completada
- DESKTOP-08 (undo + papelera base) — ya completada
- DESKTOP-30 (papelera durable) — complementaria

## Notas
- DoD: crear/renombrar/borrar ns desde sidebar con feedback y undo donde aplique
- Core no soporta rename atómico → implementar como copy+delete batch vía `delete_by_filter`
- Namespaces se crean implícitos por primer put → crear vacío = put key reservada `__vanta_namespace_meta`