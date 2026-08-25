# E2E-VISUAL — smoke E2E guard (UX-19) + verificación visual runtime (DAUD-01)

> Plan: `docs/plans/2026-08-25-batch-desktop-ux-core.md` · Task 4 · Campaign `6a6c322a-6a6a-4d17-9b34-3166181cbc4a`
> Estado: ⬜ PENDING → ⏳ IN PROGRESS (2026-08-25)
> Contrato: test Playwright en `desktop/e2e/` corre (`npx playwright test`) y pasa; capturas light/dark confirmando sin marco crema en dark

## Objetivo

Convertir el smoke E2E manual (ingest→teclado→borrar→papelera→restore→paleta) en test
Playwright permanente (`desktop/e2e/`) que corra sin Tauri contra el web build `embedded`
(igual que el smoke original, evidencia `smoke-0*.png`), + verificación visual runtime del
FIX-D1 (body padding 24px→0, tokens theme-flipping) con capturas light/dark.

## Spec (decisiones)

| # | Decisión | Justificación / evidencia |
|---|----------|---------------------------|
| 1 | Runner: `@playwright/test` (nuevo devDep, `^1.61.1` = versión de `playwright` ya instalada) | `desktop/package.json` ya declara `playwright ^1.61.1` (usado por `scripts/selfcheck-web-e2e.ts`) pero NO `@playwright/test` (verificado `node_modules/@playwright/test` = False). El runner es lo que permite `npx playwright test` con config (docs oficiales: playwright.dev/docs/test-webserver). |
| 2 | Servir la app: `vanta-cli server --http -p <puerto> -d <tmp> --dashboard-dir dist-web` (patrón exacto de `scripts/selfcheck-web-e2e.ts:126`), NO `npm run dev` | El smoke original usó el web build `embedded` (plan Task 4, Regla crítica). El e2e queda self-contained: 1 server, same-origin API, sin proxy ni Tauri. `E2E_BASE_URL` permite correr contra `npm run dev` (vite proxya `/api` → 8090, `vite.config.ts:82`) como alternativa documentada. |
| 3 | `serve.mjs` (webServer command) SIEMPRE rebuilda `dist-web` (`npx vite build --mode web`) | `dist-web/` está STALE (2026-08-19) vs `src/` (2026-08-24) → un guard sobre bundle viejo no protege nada. Rebuild determinista antes de cada run (`E2E_SKIP_BUILD=1` para iterar). `build:web` NO existe en package.json — usar `npx vite build --mode web` directo. |
| 4 | webServer: `url = http://127.0.0.1:<PORT>/api/v2/health`, `timeout 120s`, `reuseExistingServer: !CI` | Patrón oficial Playwright (docs test-webserver): `url` espera 2xx/3xx/4xx; timeout 120s cubre el rebuild. Puerto default 8091 (evita 8080/8090). |
| 5 | `workers: 1` + `fullyParallel: false` | DB temp compartida + papelera es por-sesión de página (store en memoria). Serial = determinista. Los ~2 specs corren en <1 min. |
| 6 | Seed de datos vía REST `POST /api/v2/records/batch` (fetch de Node 24, sin fixture request) | Contrato del plan: "sembrar datos (vía el bridge/vanta.ts o API)". El batch REST es idempotente por key (upsert). Node 24 tiene fetch global. |
| 7 | Assert del flujo crítico por estados UI (roles/labels existentes), no por implementation details | Patrón del selfcheck ya verde: `region Memorias`, filas `role=row`, `Inspector de registro`, botón `Mover <key> a papelera` + `BORRAR`, `↩ RESTORE`, palette Ctrl+K → AJUSTES (`Nombre del perfil`). Enter-keyboard vía `row.focus()` + `keyboard.press("Enter")` (UX-02 `handleRowKey` DataExplorer.tsx:513). |
| 8 | DAUD-01: computed styles programáticos (body bg, padding, input bg, button box-shadow) en ambos temas + screenshots `e2e/screenshots/daud01-{light,dark}.png` | "Sin marco crema" = `body` bg `#0a0a0a` en dark + `padding: 0px` (App.css:20-25); inputs desnudos = regla `@layer base input { background: var(--background) }` (App.css:32-47) verificada en checkbox del grid + radios TTL del Inspector + textarea ImportPaste; sombra dura = `var(--ink-shadow)` (negro light / crema dark). |
| 9 | NO tocar `desktop/src/**` (lógica de la app intacta) | Contrato: "NO toques la lógica de la app — solo tests + config de test". Cambios: `desktop/e2e/*` (nuevo), `package.json`, `vitest.config.ts` (excluir `e2e/**` para que `npx vitest run` no descubra specs Playwright), `.gitignore`. |
| 10 | Tsconfig sin cambios | `desktop/tsconfig.json` incluye solo `src/` → `npm run build` (tsc) no toca `e2e/`. Playwright transpila TS propio. |
| 11 | No commit (regla del batch) | El lead verifica mecánico y commitea por tarea. |

## Impacto mapeado (Regla 0)

- **Archivos a tocar:** `desktop/e2e/` (nuevo: playwright.config.ts, serve.mjs, helpers.ts, flujo-critico.spec.ts, daud01-temas.spec.ts), `desktop/package.json`, `desktop/package-lock.json`, `desktop/vitest.config.ts`, `desktop/.gitignore`
- **Leídos completos:** package.json, vite.config.ts, tsconfig.json, vitest.config.ts, index.html, .gitignore, App.css, index.css, App.tsx, main.tsx, transport.ts, useConnectionState.ts, WorkspaceShell.tsx, DataExplorer.tsx, TrashLens.tsx, CommandPalette.tsx, IngestForm.tsx, GeneralTab.tsx, Inspector.tsx (parcial), ImportPaste.tsx (parcial), Settings.tsx (parcial), selfcheck-web-e2e.ts (patrón de referencia), docs/Backlog.md (DAUD-01), plan file
- **Referencias hacia dentro:** `package.json` (npm scripts/deps — aditivo); `vitest.config.ts` (exclude aditivo); `.gitignore` (aditivo). `e2e/*` es nuevo, nadie lo referencia.
- **Referencias salientes:** `serve.mjs` → binario `vanta-cli` (existe debug+release ✅) + `npx vite build --mode web`; specs → HTTP API `/api/v2/*` + UI del bundle; helpers.ts no importa `src/**` (usa fetch + env) → cero acoplamiento de imports.
- **Veredicto:** aditivo y aislado. Cero cambios en `src/**` — la lógica de la app no se rompe. Dependencia nueva dev-only (`@playwright/test`) — sin cambio de runtime.

## Steps

1. ✅ Crear task file (este) con Spec + Impacto mapeado.
2. ✅ Agregar `@playwright/test ^1.61.1` a devDependencies + script `test:e2e` en `desktop/package.json`; excluir `e2e/**` en `vitest.config.ts`; gitignore `test-results` + `e2e/screenshots`.
3. ✅ Crear `desktop/e2e/serve.mjs` (rebuild dist-web + spawn vanta-cli server con DB temp).
4. ✅ Crear `desktop/playwright.config.ts` (raíz desktop — Playwright busca config en CWD) + `desktop/e2e/helpers.ts`.
5. ✅ Crear `desktop/e2e/flujo-critico.spec.ts` (UX-19).
6. ✅ Crear `desktop/e2e/daud01-temas.spec.ts` (DAUD-01).
7. ✅ `npm install` en desktop (+ `npx playwright install chromium` — browsers 1.62.1 faltaban).
8. ✅ VERIFY: `cd desktop && npx playwright test` — **3/3 passed (30.9s)** (contrato).
9. ✅ VERIFY colateral: `npx vitest run` 11 files/68 tests ✅ · `npm run build` exit 0 ✅ · píxeles screenshots: dark TL/TR/BL = rgb(10,10,10), light = rgb(251,249,245) ✅

## Context Save Point (2026-08-25, CIERRE)

- **Contrato cumplido:** `cd desktop && npx playwright test` → 3 passed (flujo crítico + light + dark). Capturas: `desktop/e2e/screenshots/daud01-{light,dark}.png` (gitignored, regenerables).
- **Servidor e2e:** `e2e/serve.mjs` rebuilda `dist-web` (`npx vite build --mode web`) y spawnea `vanta-cli server --http -p 8091 -d <tmp> --dashboard-dir dist-web` — mismo patrón que el smoke original (embedded web build, sin Tauri). `E2E_BASE_URL` permite correr contra `npm run dev` (vite proxya /api → 8090). Rebuild del binario: `cargo build --bin vanta-cli --features server` (los binarios previos NO tenían la feature server — hallazgo de infra, ya resuelto local).
- **Hallazgos durante implementación (routing findings):**
  - `FIND-23` (Backlog): `vanta-http-map.ts:93` manda `namespace: ""` en ingest/get HTTP con namespace omitido → server rechaza; WASM mapping sí defaulta `DEFAULT_NS`. Workaround en test: el spec de flujo explicita namespace "default" en IngestForm.
  - Config en `e2e/` no era descubierta por Playwright (busca en CWD) → movido a `desktop/playwright.config.ts` (testDir "e2e").
  - Preflight Tailwind pisa el fallback `@layer base input` de App.css → inputs desnudos computan `transparent` (el fondo del tema se ve detrás); DAUD-01 asserta "no-crema en dark" en lugar de valor token.
  - Settings embebido oculta la sección perfiles ("Nombre del perfil" solo con !embedded) → assert de AJUSTES usa "Defaults de búsqueda".
- **Invariantes:** cero cambios en `desktop/src/**` (lógica de la app intacta); vitest 68/68; `npm run build` OK; NO commit (lead commitea solo los archivos de esta tarea: `desktop/package.json`, `desktop/package-lock.json`, `desktop/vitest.config.ts`, `desktop/.gitignore`, `desktop/e2e/*`, `desktop/playwright.config.ts`, `.opencode/skills/campaign-executor/tasks/E2E-VISUAL.md`, `docs/Backlog.md` [solo la fila FIND-23]).
- **WIP guard:** claim in-progress bloqueado por FIND-11/UX-POLISH stale (tareas ya completadas que quedaron in-progress) — mismo patrón que DAUD-LIMPI/FIND-11. El lead debe limpiar esos estados.