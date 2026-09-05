# DESKTOP-31: Pantalla SETTINGS — perfiles conexión, auth Bearer, defaults búsqueda, idioma

## Metadata
- **Plan file:** docs/Backlog.md (Phase 12)
- **Creado:** 2026-08-24
- **last-synced:** 2026-08-24T00:00:00
- **Estado:** ✅ COMPLETADA (2026-08-24)

## Blast Radius
Callers: desktop/src/pages/Settings.tsx (nuevo), desktop/src/components/layout/WorkspaceShell.tsx (topbar), desktop/src/vanta.ts
Callees: desktop/src-tauri/src/commands/transport.rs, src/config.rs
Implicaciones: Permite conectar a `vanta-cli server` remoto con token desde UI; perfiles persistidos y seleccionables

## Spec
N/A — feature UI con contrato mecánico

## Contrato
`cd desktop && npm run build`; conectar a `vanta-cli server` remoto con token desde UI; perfiles persistidos y seleccionables

## Herramientas
- cargo-mcp, rust-analyzer-mcp, codegraph

## Steps
### Step 1: Crear página Settings.tsx — ✅
- **Archivos:** `desktop/src/pages/Settings.tsx` (nuevo), `desktop/src/components/layout/WorkspaceShell.tsx` (surface "ajustes": tipo, SideButton sidebar, render, ítem en CommandPalette)
- **Acción:** Página con secciones: (1) Conexiones guardadas multi-perfil (path nativo / URL server), (2) Auth Bearer token para server remoto, (3) Defaults búsqueda (top_k, modo híbrido/vectorial), (4) Idioma (ES/EN). Design system Studio (Tailwind manga/linocut).
- **Verify:** `cd desktop && npm run build` ✅

### Step 2: Transporte server con Bearer — ✅ (SIN CAMBIOS RUST)
- **Desvío justificado:** ServerClientConfig con Bearer YA existe en Rust (`connections/server_client.rs`, bridge `connectServer` en vanta.ts) — el gap era solo UI/persistencia. No se crearon comandos Tauri ni se tocó app_config_dir (decisión DESKTOP-23: localStorage inyectable).
- **Archivos:** `desktop/src/hooks/useConnectionState.ts` (+`connectServerCfg`), `desktop/src/store/connections.ts` (nuevo)
- **Verify:** `cd desktop && npm run build` ✅

### Step 3: Conectar ConnectionPanel a perfiles — ✅
- **Archivos:** `desktop/src/components/ConnectionPanel.tsx` (dropdown perfiles guardados + botón Connect profile), `desktop/src/components/layout/WorkspaceShell.tsx` (handler `useProfile`)
- **Verify:** build ✅; flujo: perfil server+token → conectar → topbar muestra conexión activa

### Step 4: Persistencia y selección de perfiles — ✅
- **Archivos:** `desktop/src/store/connections.ts` (nuevo), `desktop/src/store/connections.test.ts` (nuevo)
- **Acción:** ConnectionPrefsStore (storage inyectable, patrón preferences.ts): profiles[], activeProfileId, topK/mode/lang. Defaults búsqueda alimentan runSearch del topbar.
- **Verify:** `cd desktop && npm test` ✅ 48/48 (4 tests nuevos)

## Impacto mapeado (Regla 0)
Leídos completos: WorkspaceShell.tsx, ConnectionPanel.tsx, useConnectionState.ts, vanta.ts, preferences.ts, persisted-stores.test.ts, CommandPalette.tsx (parcial), App.tsx. Entrantes: Surface union consumida por palette (PaletteSurface duplicado actualizado); ConnectionActions extendida solo aditivamente. Salientes: connectServer ya existente (sin cambios wire). Veredicto: cambios aditivos, ningún contrato roto.

## Context Save Point
- Contrato mecánico verde: `cd desktop && npm run build` ✅ + `cd desktop && npm test` ✅ (48/48)
- Sin commit (regla explícita del orquestador)

## Dependencias
- DESKTOP-23 (config.rs save atómico)
- DESKTOP-10 (put bridge) — ya completada

## Notas
- DoD: conectar a `vanta-cli server` remoto con token desde UI; perfiles persistidos y seleccionables
- Hoy imposible conectar a server con auth (sin UI para token) → esta tarea lo desbloquea
- Transporte pluggable: nativa embebida / HTTP `/api/v2/*` / WASM-OPFS