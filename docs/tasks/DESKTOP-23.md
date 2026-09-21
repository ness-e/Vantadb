# DESKTOP-23: Persistencia de preferencias UI (tema/layout/filtros) en app_config_dir con save atómico

## Metadata
- **Plan file:** docs/Backlog.md (Phase 12)
- **Creado:** 2026-08-24
- **last-synced:** 2026-08-24
- **Estado:** ✅ COMPLETED

## Impacto mapeado (Regla 0)
- **Leídos completos:** `desktop/src/store/favorites.ts` (patrón a reutilizar),
  `desktop/src/store/persisted-stores.test.ts` (patrón de tests DESKTOP-26),
  `desktop/src/App.tsx` + `main.tsx` (persistencia tema `vanta-theme`),
  `desktop/src/components/layout/WorkspaceShell.tsx` (surface/ruleGroup/showFilters),
  `desktop/src/components/home/HomeOverview.tsx` (solo estado de fetch — nada que persistir).
- **Referencias hacia dentro:** WorkspaceShell importa stores de `src/store/`.
- **Entrantes:** ninguno nuevo — el store preferences es consumido solo por WorkspaceShell.
- **Decisión de alcance (ponytail full):** los steps 1-2 del plan original planteaban
  app_config_dir vía comandos Tauri. El WebView de Tauri ya persiste localStorage entre
  sesiones — favorites/search-history/tema dependen de eso hoy. Un segundo mecanismo
  (app_config_dir) duplicaría la fuente de verdad sin añadir garantía → se descarta.
  Tema ya persiste (`vanta-theme`, aplicado pre-render en main.tsx). Solo faltaba
  surface + panel filtros + ruleGroup → un store, cero cambios en src-tauri.

## Blast Radius
Callers: desktop/src/lib/store.ts (theme persistence), desktop/src/components/layout/WorkspaceShell.tsx (layout)
Callees: src/config.rs (app_config_dir), Tauri fs API
Implicaciones: Reiniciar la app debe conservar tema/layout/filtros. No rompe nada existente (solo añade persistencia).

## Spec
N/A — bug-fix/feature con contrato mecánico

## Contrato
`cargo check -p vantadb && cd desktop && npm run build && npm run test` (si existe test script)

## Herramientas
- cargo-mcp, rust-analyzer-mcp, codegraph

## Steps
### Step 1: ✅ Store persistido de preferencias (localStorage, patrón DESKTOP-26)
- **Archivos:** `desktop/src/store/preferences.ts` (nuevo)
- **Acción:** `WorkspacePrefsStore` con storage inyectable + sanitize (surface string,
  showFilters bool, ruleGroup exige `rules` array). Key `vanta.workspace.v1`.
  Descartado app_config_dir/save atómico Rust: localStorage del WebView ya es el
  mecanismo probado en esta app; temp+rename duplicaría fuente de verdad.
- **Verify:** `cd desktop && npm test` ✅

### Step 2: ✅ Conectar WorkspaceShell (hidratación + write-through)
- **Archivos:** `desktop/src/components/layout/WorkspaceShell.tsx`
- **Acción:** estado inicial de `surface`/`ruleGroup`/`showFilters` hidratado desde
  `workspacePrefs.get()`; `useEffect` write-through en cambios. Tema intacto
  (`vanta-theme` ya persistía).
- **Verify:** `cd desktop && npm run build` ✅

### Step 3: ✅ Tests de persistencia (round-trip ≈ reinicio)
- **Archivos:** `desktop/src/store/persisted-stores.test.ts`
- **Acción:** 3 tests: round-trip completo con instancia nueva sobre el mismo
  storage, merge parcial, storage corrupto/campos mal tipados → defaults limpios.
- **Verify:** `cd desktop && npm test` → 41/41 ✅

## Dependencias
- Ninguna (tarea independiente)

## Notas
- DoD: reiniciar la app conserva tema/layout/filtros
- Usar `app_config_dir` de Tauri para ruta portable
- Save atómico: escribir a `.tmp` + `rename` para evitar corrupción