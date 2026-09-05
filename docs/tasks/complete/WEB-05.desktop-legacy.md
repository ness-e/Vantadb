# WEB-05 — Build web de la consola (Vite base `/dashboard/`, sin Tauri)

> **Plan:** `docs/plans/2026-08-18-vanta-studio-fase3.md` · **Wave 2** · **Estado:** ✅ COMPLETO
> **Contexto:** WEB-00 (0cccd326) abstrajo transporte; WEB-04 (8b2bc14f) implementó HttpBackend real (factory detecta `window.__TAURI_INTERNALS__` → Tauri, sino HttpBackend base `""`/`VITE_VANTA_API_BASE`). Server sirve estáticos `/dashboard` (WEB-03, 62d63377).

## Contrato

1. `vite build --mode web` produce `desktop/dist-web/` (base `/dashboard/`) — sin plugin Tauri en ese modo.
2. App arranca en browser sin Tauri: transport = HttpBackend contra mismo origin (ya hecho WEB-04 — no duplicar).
3. Surfaces HOME/MEMORIAS/ACTIVITY/ÍNDICES/IQL funcionales con datos del server.
4. Surfaces que dependen de Tauri (deep links) degradan con aviso, no crash.
5. `npm run build` (modo desktop/Tauri) sigue verde sin cambios.

## Impacto mapeado (Regla 0)

**Archivos leídos completos:**
- `desktop/vite.config.ts` (33L) — sin base/outDir por modo; sin plugin Tauri (solo react + tailwindcss)
- `desktop/package.json` — `build: tsc && vite build` (mode default production → dist/)
- `desktop/src/main.tsx` (15L) — sin imports Tauri-only; solo theme init + render App
- `desktop/src/App.tsx` (42L) — contenedor fino: useConnectionState + WorkspaceShell
- `desktop/src/transport.ts` (87L) — TauriBackend/HttpBackend; factory `getTransport()` ya detecta `__TAURI_INTERNALS__`; `transport` module-level
- `desktop/src/vanta.ts` (559L) — bridge tipado; `takeDeepLink()` ya es no-op fuera de Tauri (line 554-558)
- `desktop/src/hooks/useConnectionState.ts` (157L) — `refresh()` llama `listConnections()` → THROWS en web (unsupported) → app queda "sin backend activo"
- `desktop/src/hooks/useDeepLink.ts` (43L) — `listen()` de `@tauri-apps/api/event` → promise rechazada en browser (unhandled rejection)
- `desktop/src/components/layout/WorkspaceShell.tsx` (828L) — renderiza ConnectionPanel en RESUMEN (682-692); `state.active` gatea superficies
- `desktop/src/components/ConnectionPanel.tsx` — selector Tauri-only (connect/disconnect/activate)
- `desktop/src/components/MetricsGrid.tsx` / `KpiCards.tsx` / `ExportPanel.tsx` / `graph/useGraphData.ts` — ya manejan errores con catch (degradan, no crash)
- `desktop/src/vanta-http-map.ts` — vanta_list_connections/connect/disconnect/set_active/metrics/graph_* → unsupported con mensaje descriptivo
- `desktop/src-tauri/tauri.conf.json` — `frontendDist: "../dist"` → el build desktop DEBE seguir saliendo a `dist`
- `desktop/.gitignore` — cubre `dist` pero NO `dist-web`

**Referencias hacia dentro (quién importa lo que cambio):**
- `transport.ts` → importado por `vanta.ts` (transport, TauriBackend) y `useDeepLink.ts` (indirecto)
- `useConnectionState.ts` → importado por `App.tsx`
- `useDeepLink.ts` → importado por `WorkspaceShell.tsx`
- `WorkspaceShell.tsx` → importado por `App.tsx` (solo App)

**Referencias salientes:**
- `useDeepLink.ts` → `@tauri-apps/api/event` (listen) — el único import Tauri-only en runtime que rompe en web
- `transport.ts` → `@tauri-apps/api/core` (invoke) — import-safe en browser; solo se llama en TauriBackend

**Veredicto de impacto:**
- Cambios acotados a 6 archivos: `vite.config.ts`, `transport.ts`, `useConnectionState.ts`, `useDeepLink.ts`, `App.tsx`, `WorkspaceShell.tsx` + `.gitignore`
- `main.tsx` NO requiere cambios (nada Tauri-only)
- Riesgo principal (plan §Riesgos "Imports Tauri-only rompen build web"): NO rompen el BUILD (vite/tsc bundlean `@tauri-apps/api` sin ejecutar); romperían RUNTIME → guards runtime (no import dinámico) en useDeepLink + useConnectionState
- Sin cambios en `src-tauri/`, `web/`, `vantadb-wasm/`, `src/sdk/` (protegidos)

## Fase 1 — DISCOVERY (✅)

- ✅ Leer plan file + pipeline-full.md
- ✅ Leer vite.config.ts, package.json, tsconfigs, index.html, main.tsx, App.tsx, transport.ts, vanta.ts, useConnectionState.ts, useDeepLink.ts, WorkspaceShell.tsx, vanta-http-map.ts, tauri.conf.json
- ✅ Grep imports `@tauri-apps/*` → solo transport.ts (invoke) + useDeepLink.ts (listen)
- ✅ Mapear comandos por componente: solo `listConnections` (mount, useConnectionState) y `listen` (mount, useDeepLink) rompen en web; metrics/graph ya degradan con catch
- ✅ Baseline `npm run build` → ✅ verde (7.44s, dist/)

## Fase 2 — EJECUCIÓN

### Step 1: `vite.config.ts` — modo web
- [x] `defineConfig(({ mode }) => ...)` — `mode === "web"` → `base: "/dashboard/"`, `build.outDir: "dist-web"`; default → undefined (comportamiento actual idéntico)
- [x] Verify: `npx vite build --mode web` produce `desktop/dist-web/` con assets referenciados `/dashboard/assets/...` (✅ index.html: `/dashboard/assets/index-b4HdIPxm.js`)

### Step 2: `transport.ts` — flag de modo embebido
- [x] Exportar `isEmbedded = !(transport instanceof TauriBackend)` (runtime truth, misma base que factory WEB-04)

### Step 3: `useConnectionState.ts` — conexión implícita embedded
- [x] `refresh()`: si `isEmbedded` → sintetizar `[["embedded", {id, name:"embedded", via:"http", status:"connected"}]]` en lugar de `listConnections()` (que throws en web)
- [x] Verify: tsc pasa; `state.active` seteado en web → superficies activas por defecto (npm run build ✅)

### Step 4: `useDeepLink.ts` — guard Tauri-only
- [x] Si no Tauri (isEmbedded) → no-op con `console.info` aviso (deep links son OS-scheme, no llegan en web); skip `listen()`

### Step 5: `App.tsx` + `WorkspaceShell.tsx` — ocultar selector Tauri-only
- [x] App: `embedded={isEmbedded}` prop a WorkspaceShell
- [x] WorkspaceShell: `{!embedded && <ConnectionPanel …/>}` (modo embebido = HTTP activo por defecto)

### Step 6: `.gitignore` — dist-web
- [x] Agregar `dist-web` en `desktop/.gitignore` (build output, no commitear)

## Fase 3 — VERIFICACIÓN

- [x] `npm run build` (desktop) ✅ verde (7.04s) → `dist/` (frontendDist `../dist` intacto; `dist/index.html` existe)
- [x] `npx vite build --mode web` ✅ verde (6.93s) → `desktop/dist-web/`
- [x] `npx vite preview --mode web --base /dashboard/ --port 4173` + Invoke-WebRequest: `GET /dashboard/` → 200 index.html con `src="/dashboard/assets/index-b4HdIPxm.js"` + `href="/dashboard/assets/index-DcGNFv4F.css"`; `GET /dashboard/assets/index-b4HdIPxm.js` → 200 (379664 bytes)
- [x] `node --test src/vanta-deep-link.test.ts src/vanta-http-map.test.ts` → 22/22 pass
- [x] `git status --short -- desktop/` → solo los 7 archivos esperados (sin dist/, dist-web/, src-tauri/, node_modules) — `dist-web` ignorado ✅

## Fase 4 — CIERRE

- [x] Actualizar plan file WEB-05 → ✅ COMPLETO + nota mecanismo modo + dist-web gitignore
- [x] NO commit (lo hace el lead)
- [x] Devolver bloque RESULTADO

## Notas / Decisiones

- **Mecanismo de modo:** `--mode web` de Vite → `defineConfig(({ mode }) => …)`; `mode === "web"` en config. Sin archivos `.env.web` (no hacen falta — no hay variables de entorno por modo). En client code no se usa `import.meta.env.MODE`: el flag runtime `isEmbedded` (transport) es la fuente de verdad (cubre también el caso de correr el build web dentro de Tauri, que no aplica pero es robusto).
- **Guard runtime vs import dinámico:** los imports `@tauri-apps/api/*` NO rompen el build (vite/tsc los bundlean sin ejecutar). Romperían runtime. Guards runtime en useDeepLink + useConnectionState — sin import dinámico (innecesario).
- **main.tsx sin cambios:** no contiene lógica Tauri-only (solo theme init + render). El guard que anticipaba el plan vive en useDeepLink (donde está el `listen()`).
- **IQL surface:** GraphLens usa `useGraphData` que llama graphDegree/graphBfs (unsupported en web) → degrada con `onError` aviso, no crash. Requiere server con endpoint graph_v2 DTO para funcionalidad completa (follow-up documentado en vanta-http-map).
- **Metrics:** MetricsGrid/KpiCards/ExportPanel llaman `vanta_metrics` (unsupported en web) → degradan con "metrics unavailable" (no crash). Follow-up: endpoint JSON `/api/v2/metrics`.
- **dist-web en gitignore:** ✅ agregado en `desktop/.gitignore` (antes NO estaba cubierto — solo `dist`).