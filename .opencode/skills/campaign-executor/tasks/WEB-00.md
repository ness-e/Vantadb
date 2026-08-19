# Task WEB-00 — Abstraer `vanta.ts` de Tauri invoke (transporte pluggable)

> **Plan:** `docs/plans/2026-08-18-vanta-studio-fase3.md` — Wave 0, Task 1
> **Tipo:** refactor (TS puro, mecánico 1:1) — detect_task_type devolvió "rust" por default del detector; el cambio real es TypeScript en `desktop/`
> **Campaña:** 9d4f2b8e-7c1a-4e6f-9d3b-5a8c2f7e1d60
> **Estado:** ✅ COMPLETO — 2026-08-19 (vanta-worker)

## Contexto

Toda la consola desktop habla con Tauri `invoke` vía `desktop/src/vanta.ts` (35+ funciones, 24 con `invoke`). Para servir la misma consola vía HTTP (o WASM en Fase 4) sin reescribir componentes, el transporte debe ser pluggable: interface `VantaTransport` + `TauriBackend` (invoke) | `HttpBackend` (fetch, stub en esta task → WEB-04).

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `desktop/src/vanta.ts` (552L), `desktop/src/vanta-deep-link.test.ts` (63L), `desktop/src/hooks/useDeepLink.ts` (43L), `desktop/package.json`, `desktop/vite.config.ts`, `desktop/node_modules/@tauri-apps/api/core.d.ts` (firma `invoke` v2).
- **Referencias hacia dentro (quién importa de `vanta.ts`):** `hooks/useConnectionState.ts` (health, connectNative, disconnect, setActive, listConnections, vantaErrorMessage), `components/DataExplorer.tsx` (listPage), `components/lens/retrieval/RetrievalLens.tsx` (listPage), `components/space/useProjection.ts` (listPage), `components/graph/IqlConsole.tsx` (queryIql), `hooks/useDeepLink.ts` (DEEP_LINK_EVENT, parseVantaUrl, takeDeepLink), más componentes de UI que consumen `search/ingest/get/versions/remove/metrics/auditEvents/exportNamespace/graphBfs/...` — mapeado completo por codegraph (47 símbolos, 6 archivos).
- **Referencias hacia fuera de `vanta.ts`:** `@tauri-apps/api/core` (`invoke`) — único import de Tauri en el archivo. El hook `useDeepLink.ts` importa `listen` de `@tauri-apps/api/event` directamente (fuera de scope; lo degrada WEB-05).
- **Veredicto de impacto:** refactor mecánico sin cambio de firmas ni de tipos exportados → 0 cambios en callers. Verificación: `npm run build` (tsc strict) + tests existentes.

## Fase 1 — DISCOVERY ✅

- [x] Leer plan file (`docs/plans/2026-08-18-vanta-studio-fase3.md`, Task 1) y pipeline (`pipeline-full.md`)
- [x] `codegraph_explore` sobre `vanta.ts` — blast radius (callers) y fuentes de DTOs (`desktop/src-tauri/src/connections/types.rs`)
- [x] Verificar firma oficial de `invoke` en `core.d.ts:127`: `invoke<T>(cmd, args?: InvokeArgs, options?: InvokeOptions)` con `InvokeArgs = Record<string, unknown> | number[] | ArrayBuffer | Uint8Array` — los wrappers actuales pasan solo objetos → `call<T>(cmd, args?: Record<string, unknown>)` compatible
- [x] Confirmar que `invoke` solo se usa en `vanta.ts` dentro de `desktop/src/` (grep: 1 match de import)
- [x] Baseline: `npm run build` ✅ (tsc + vite 7.3.6), `node --test src/vanta-deep-link.test.ts` ✅ 8/8 (el archivo usa `node:test`, NO vitest — `npx vitest run vanta-deep-link` es falso negativo pre-existente: suite fail pero 8/8 tests pass)
- [x] Capturar firmas exportadas actuales (55 exports) en `C:\Users\Eros\AppData\Local\Temp\opencode\web00-exports-before.txt` para diff mecánico

## Fase 2 — EJECUCIÓN ✅

- [x] Crear `desktop/src/transport.ts`: interface `VantaTransport { call<T>(cmd, args?): Promise<T> }`, `TauriBackend` (delega a `invoke`), `HttpBackend` stub (rechaza "not implemented", WEB-04), factory `getTransport()` (detecta `window.__TAURI_INTERNALS__` → TauriBackend; sino HttpBackend), singleton `transport = getTransport()`
- [x] Refactor `desktop/src/vanta.ts`:
  - [x] Header comment: bridge → transporte pluggable
  - [x] Import: `invoke` de `@tauri-apps/api/core` → `transport` de `./transport`
  - [x] 24 llamadas `invoke<T>(...)` → `transport.call<T>(...)` (args idénticos, sin reordenar)
  - [x] `takeDeepLink`: `instanceof TauriBackend` → `call("vanta_deep_link_take")`; web → `[]` no-op (contrato punto 6)
  - [x] `parseVantaUrl` / `DEEP_LINK_EVENT` / tipos DTOs: intactos (sin invoke)
- [x] NO tocar: `docs/Backlog.md`, `web/`, `vantadb-wasm/`, `src/sdk/`, `desktop/src-tauri/`, README.md

## Fase 3 — VERIFICACIÓN ✅

- [x] `npm run build` (desktop/) — tsc + vite ✅ exit 0
- [x] `npx vitest run vanta-deep-link` — 8/8 tests pass (suite reportado fail es falso negativo pre-existente: el archivo usa `node:test`, no vitest; igual que baseline)
- [x] `node --test src/vanta-deep-link.test.ts` (comando real del repo) ✅ 8/8
- [x] Diff mecánico de firmas: 55 exports antes == 55 exports después (nombre y tipo intactos); `git diff` muestra solo cuerpo delegando a `transport.call` + import + header
- [x] `cargo` no aplica (cambio TS puro)

## Fase 4 — CIERRE ✅

- [x] Task file actualizado con checkboxes
- [x] Sin commit — el lead commitea tras verify (instrucción del orquestador)
- [x] Bloque RESULTADO devuelto al orquestador

## Contrato (verificable)

1. ✅ Interface `VantaTransport { call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> }` en `desktop/src/transport.ts`
2. ✅ `TauriBackend` implementa `call` delegando a `invoke`
3. ✅ Factory `getTransport()` selecciona por entorno (Tauri vs browser) + singleton `transport`
4. ✅ TODAS las funciones de `vanta.ts` delegan a `transport.call(cmd, args)` — 1:1, sin cambios de firma ni tipos exportados
5. ✅ Stub `HttpBackend` rechaza "not implemented" (se completa en WEB-04)
6. ✅ `takeDeepLink` queda en TauriBackend (no-op `[]` en web)
7. ✅ Archivos protegidos intactos

## Deuda / pendientes

- `HttpBackend` real (fetch + mapeo cmd→REST) → WEB-04
- Guard runtime de imports Tauri-only en build web → WEB-05
- Test de transporte mínimo (factory selecciona por entorno) → WEB-04
- `npx vitest run vanta-deep-link` reporta suite failed aunque los 8 tests pasan (pre-existente: el test usa `node:test`); el verify canónico del repo es `node --test`
