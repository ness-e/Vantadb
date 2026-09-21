# DESKTOP-30: Estado de usuario durable — favoritos, historial, undo, papelera

## Metadata
- **Plan file:** docs/Backlog.md (Phase 12)
- **Creado:** 2026-08-24
- **last-synced:** 2026-08-24
- **Estado:** ✅ COMPLETED

## Impacto mapeado (Regla 0)
- **Leídos completos:** `desktop/src/store/undo.ts`, `desktop/src/store/undo.test.ts`, `desktop/src/store/favorites.ts` (patrón a copiar), `desktop/src/store/persisted-stores.test.ts`, `desktop/vitest.config.ts`.
- **Referencias entrantes a undo.ts:** `TrashLens.tsx`, `WorkspaceShell.tsx`, `DataExplorer.tsx`, `SpaceLens.tsx`, `SelectionBar.tsx`, `CommandPalette.tsx` — todas vía API pública (`undoStore.getTrash/softDelete/restore/purge/undo/subscribe`). Cambio interno de persistencia no rompe ninguna.
- **Referencias salientes:** undo.ts → `../vanta` (remove, vantaPut) — sin cambios.
- **Grep de gaps:** favoritos/historial/prefs ya persisten (DESKTOP-23/26). Único gap: trash/tombstones del UndoStore (session-only, declarado en su header).
- **Veredicto:** cambio aditivo y encapsulado en undo.ts + tests. Cero cambios Rust/Tauri.

## Corrección de scope (2026-08-24)
Los Steps originales (config.rs + comandos Tauri `get_user_state`/`set_user_state`) quedan **OBSOLETOS**: DESKTOP-23 decidió localStorage inyectable como mecanismo único de persistencia (el WebView de Tauri ya lo persiste entre sesiones). Introducir app_config_dir duplicaría la fuente de verdad. Se reutiliza el patrón exacto de `preferences.ts`/`favorites.ts`.

## Spec
Persistir SOLO la papelera (`trash`) en storage inyectable:
- Storage default `localStorage` con guard `typeof`; inyectable por constructor para tests (patrón favorites.ts).
- Key: `vanta.trash.v1`. Hidratación en constructor con validación mínima de shape; corrupto → papelera vacía.
- Persistencia write-through en cada mutación de trash (hook único en `notify()`); fallos de quota → solo sesión, sin crash.
- El stack de undo NO se persiste (sus reverses referencian estado del backend de la sesión; DoD solo exige papelera).

## Contrato
`cd desktop && npm run build && cd desktop && npm test`; reiniciar app conserva favoritos, historial y papelera.

## Herramientas
- codegraph, terminal (npm)

## Steps
### Step 1: Persistir trash del UndoStore ✅
- **Archivos:** `desktop/src/store/undo.ts`
- **Acción:** storage inyectable + `loadTrash()` en constructor + `persistTrash()` dentro de `notify()`. Exportar clase `UndoStore`. Actualizar header obsoleto.
- **Verify:** `cd desktop && npm run build`

### Step 2: Tests de persistencia ✅
- **Archivos:** `desktop/src/store/undo.test.ts`
- **Acción:** fakeStorage inyectable; round-trip delete→hidratación en instancia nueva; corrupto→vacío. `localStorage.clear()` en beforeEach (jsdom comparte storage entre freshStore()).
- **Verify:** `cd desktop && npm test`

## Dependencias
- DESKTOP-23 / DESKTOP-26 (patrón localStorage inyectable) — completadas
- DESKTOP-08 (undo + papelera base) — completada

## Notas
- DoD: reiniciar conserva favoritos, historial y papelera (favoritos+historial ya cubiertos por DESKTOP-26)
- Save atómico temp+rename: N/A — mecanismo único es localStorage (decisión DESKTOP-23)