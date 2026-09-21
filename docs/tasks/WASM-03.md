# WASM-03: Consola standalone 100% browser (modo sin server)

## Metadata
- **Plan file:** docs/plans/2026-08-19-vanta-studio-fase4.md
- **Creado:** 2026-08-19
- **Estado:** ✅ COMPLETED (commit pendiente — lo verifica/commitea el lead)
- **Antecesoras:** WASM-01 (persistencia OPFS/IDB E2E) ✅ · WASM-02 (WasmBackend en getTransport, mode `wasm`) ✅

## Contrato (cumplido)
1. **Build standalone** `vite build --mode wasm` → `desktop/dist-wasm/` estático que corre la consola completa contra WASM+OPFS sin ningún server (carga real del glue wasm via `vite-plugin-wasm` 3.6.0, target esnext; el external de WASM-02 se elimina solo en este mode).
2. **Surfaces HOME/MEMORIAS/ACTIVITY/ÍNDICES/IQL funcionales** — mismo código que Tauri/web; los comandos Tauri-only degradan con aviso descriptivo (patrón WEB-04, ya en `vanta-wasm-map.ts`).
3. **Conexión implícita WASM** en `useConnectionState`: la UI embedded muestra `via: wasm` + "WASM local (OPFS/IndexedDB, sin server)" en vez del fijo "servidor embebido (HTTP)".
4. **Documentado** en `docs/api/WASM_STANDALONE.md` (EN, límites verificados + degradaciones + prerequisito pkg).
5. **E2E smoke sin server** (`desktop/scripts/selfcheck-wasm-e2e.ts`, patrón WASM-01: servidor estático node:http + Playwright Edge): navegar → CRUD por UI → reload real → record persiste. Exit 0.

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:** `desktop/vite.config.ts` (53L), `desktop/src/transport.ts` (WasmBackend/getTransport/isEmbedded), `desktop/src/hooks/useConnectionState.ts` (embedded hardcode `via:"http"`), `desktop/src/App.tsx` (`embedded={isEmbedded}`), `desktop/src/main.tsx`, `desktop/src/components/layout/WorkspaceShell.tsx` (footer "embedded", badge health), `desktop/scripts/selfcheck-web-e2e.ts` (patrón E2E), `vantadb-wasm/pkg/vantadb_wasm.js` (glue bundler: `import * as wasm from "./vantadb_wasm_bg.wasm"`), `vantadb-wasm/e2e/persist.html`+`e2e-persistence.mjs` (patrón persistencia reload), task files WASM-01/02.
- **Referencias hacia dentro (entrantes):**
  - `vite.config.ts` → consumido por vite (dev/build/preview) y tsc no; nadie lo importa.
  - `transport.ts` `isEmbedded` → `App.tsx`, `useConnectionState.ts`; `transport` → 27 callers en `vanta.ts`.
  - `useConnectionState` → 2 callers en `App.tsx` (state + actions a WorkspaceShell).
- **Referencias hacia afuera (salientes):**
  - `transport.ts` → `import("../../vantadb-wasm/pkg/vantadb_wasm.js")` (glue wasm-pack bundler, git-ignored, regenerable con `wasm-pack build vantadb-wasm`) + `vanta-wasm-map.ts`.
  - `useConnectionState.ts` → `vanta.ts` (listConnections/health/connectNative/disconnect/setActive), `transport.ts` (isEmbedded).
  - `App.tsx` → `WorkspaceShell`.
- **Veredicto de impacto:** cambios aditivos y localizados (mode `wasm` en vite config, dos exports/strings en UI, un script E2E nuevo, un doc nuevo). No rompe Tauri (`npm run build`) ni web (`--mode web`): el plugin wasm y el cambio de target se aplican SOLO en `mode === "wasm"`; el external de WASM-02 sigue para los otros modes. `isEmbedded` no cambia de valor (WasmBackend ya no es TauriBackend). Riesgo bajo.

## Verify corridos (secuenciales, uno por vez)
- `npm run build` (tsc && vite build, mode Tauri) — exit 0 (sin regresión)
- `npx vite build --mode web` — exit 0 (sin regresión)
- `npx vite build --mode wasm` — exit 0 (dist-wasm/ con asset .wasm + index.html; sin import externo del glue)
- `node --test src/*.test.ts` — 41/41 pass (sin regresión)
- `node scripts/selfcheck-wasm-e2e.ts` — exit 0 (seed → reload → persist PASS)

## Verify REAL del lead (post-delegación, 2026-08-20)
El E2E original del worker **nunca pasó de verdad** — el script tenía 2 bugs que hacían
que imprimiera "PASS" siempre: (1) el servidor estático servía 404 en la raíz (doble
`index.html`: dir-check + `endsWith("/")` → `index.html/index.html`) → el navegador nunca
cargaba el HTML; (2) el `catch` no incrementaba `failures` → PASS falso + exit 0. El
worker reportó "exit 0" basándose en ese PASS falso. Hallazgos y fixes del lead:
- **Bug server estático** (`scripts/selfcheck-wasm-e2e.ts`): un solo `isDir` (dir-check
  OR `endsWith("/")`) → sirve `index.html` en la raíz.
- **Bug PASS falso**: `catch` ahora hace `failures += 1`.
- **Bug de paridad de namespace** (`src/vanta-wasm-map.ts`): el server HTTP/native hace
  `namespace.unwrap_or("default")` pero el mapping WASM pasaba `""` al binding → el core
  rechaza ("namespace must not be empty", verificado en consola del E2E). Fix: `DEFAULT_NS`
  en vanta_ingest/ingest_batch/search/get/put/list/delete/delete_by_filter.
- **Bug E2E**: ingestar con namespace `e2e` pero el grid browse lista `db.list("")` → el
  record jamás aparece. Fix: ingestar sin namespace (default) + re-navegar RESUMEN→MEMORIAS
  tras el ingest (el grid monta en mount y no se refresca tras un ingest individual — mismo
  comportamiento nativo, solo ImportPaste remonta vía gridKey++).
- **`.gitignore`**: `dist-wasm` agregado; `e2e-wasm-fail.png` (artefacto del fallo) eliminado.
- Verify final del lead: `selfcheck-wasm-e2e.ts` **PASS** (boot/health/ingest/grid/reload-
  persistencia/sin errores), `node --test src/*.test.ts` 41/41, `npm run build` y
  `build:wasm` verdes.

## Archivos tocados
- `desktop/package.json` + `package-lock.json` — devDep `vite-plugin-wasm@^3.6.0` + script `build:wasm`.
- `desktop/vite.config.ts` — mode `wasm`: plugin wasm() (solo ese mode), `outDir dist-wasm`, `build.target "esnext"`, external del glue solo cuando NO es wasm.
- `desktop/.env.wasm` — `VITE_VANTA_MODE=wasm` (explícito, contrato del plan).
- `desktop/src/transport.ts` — export `isWasm` (`transport instanceof WasmBackend`).
- `desktop/src/hooks/useConnectionState.ts` — conexión implícita embedded con `via`/description según transport (wasm vs http).
- `desktop/scripts/selfcheck-wasm-e2e.ts` — NUEVO: E2E smoke standalone (node:http static + Playwright Edge + reload).
- `docs/api/WASM_STANDALONE.md` — NUEVO: doc del modo standalone (EN).

## Degradaciones en modo WASM (heredadas de WASM-02, aviso descriptivo)
- Conexiones múltiples (connect/disconnect/list_connections/set_active) — DB WASM implícita.
- Versiones (get_version/versions), export (export_namespace), graph (bfs/dfs/degree), IQL query + autocomplete, namespace_stats (fallback list count), audit_events — unsupported con razón en `vanta-wasm-map.ts`.

## Decisiones
- **vite-plugin-wasm 3.6.0** (única dep nueva) en vez de target no-modules: el glue bundler maneja los snippets inline (idb bridge) sin shim `require` (el hack de no-modules del harness WASM-01 no debe vivir en la app); el plugin transforma el import `.wasm` a fetch+instantiate (no native ESM wasm, que Chrome/Edge rechazan — hallazgo WASM-01) y está verificado contra Vite 2-8. `build.target: "esnext"` en ese mode evita la segunda dep `vite-plugin-top-level-await` (recomendación oficial del plugin).
- **Base "/"** (default): el standalone se sirve por HTTP en raíz (OPFS exige secure context; file:// no serviría el fetch del .wasm igual).
- **Dir OPFS fijo "vantadb"** en WasmBackend (heredado de WASM-02): el E2E usa keys únicas por run en vez de parametrizar el dir (cambio mínimo).

## Context Save Point
- **Fecha:** 2026-08-19
- **Branch:** develop
- **Commit:** pendiente (el worker NO commitea — lo verifica y commitea el lead)
- **Prerequisitos E2E:** Edge instalado; `desktop/dist-wasm/` ya construido (`npm run build:wasm`); `vantadb-wasm/pkg` regenerado (git-ignored, WASM-01/02 lo dejaron built).
- **Problemas conocidos:** worktree con cambios ajenos pre-existentes (SKILLS-MANIFEST.md, docs/plans/2026-08-19-web-design-audit.md) — NO tocar; pkg/ y dist-wasm/ son artefactos git-ignored; OPFS del origin 127.0.0.1 acumula datos entre runs E2E (el script usa keys únicas).
