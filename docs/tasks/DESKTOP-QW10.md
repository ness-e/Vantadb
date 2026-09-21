# TASK DESKTOP-QW10: E2E desktop (H-07) — specs multi-perfil conexión + proxy dashboard (mock upstream); graph/space smoke manual

## Metadata
- **Plan file:** `docs/plans/2026-08-25-research-desktop-quickwins.md`
- **Creado:** 2026-08-27T23:20
- **last-synced:** 2026-08-27T23:20
- **Estado:** ✅ COMPLETED
- **Tipo detectado:** e2e/test (playwright, webapp-testing) + docs
- **Workflow:** feature-add (new specs + mock + smoke manual) — spec-first requerido
- **Task file:** `.opencode/skills/campaign-executor/tasks/DESKTOP-QW10.md`

## Blast Radius
- `desktop/e2e/` — carpeta de specs E2E (flujo-critico, daud01, helpers, serve.mjs). Se añaden 2 specs nuevas + 1 doc smoke manual. Blast 3 archivos nuevos, no se tocan specs existentes salvo helpers si necesita fixture compartido.
- `desktop/playwright.config.ts` — config E2E existente (workers 1, webServer serve.mjs, baseURL /dashboard/). No se edita salvo que el mock requiera route config; si se toca, 1 línea (ej. extra header). Reversible.
- `desktop/src/components/proxy/ProxyDashboard.tsx` — dashboard que hace fetch `${proxyUrl}/snapshot` polling 5s. No se edita lógica; solo se añade fixture mock para E2E vía `page.route`. Si se necesita test hook (data-testid), 1 atrito aria-label ya existe; no lógica.
- `desktop/src/store/connections.ts` — profiles store (ConnectionPrefsStore). No se edita; tests unit existentes 4/4. E2E multi-perfil ejercerá localStorage `vanta.connections.v1` vía `page.evaluate` + navegación AJUSTES (defaults topK/mode/lang sí visibles en embedded; perfiles UI solo Tauri). Si se necesita forzar visibilidad perfiles en E2E web, 1 línea `URLSearchParams.has("e2e")` guard.
- `desktop/src/pages/Settings.tsx` — superficie AJUSTES con gate `!embedded` para perfiles. No se edita salvo guard E2E (1 línea, ponytail comment).
- `desktop/src/components/graph/GraphLens.tsx` + `desktop/src/components/space/SpaceLens.tsx` — lentes pesados (three/regl). No se tocan; quedan smoke visual manual documentado (no E2E auto) por WebGL/canvas + UMAP no-determinista en CI.
- **Implicaciones:** 0 WAL/vector/storage, 0 concurrencia hot path, 0 FFI unsafe nuevo. E2E webServer requiere binario `vanta-cli` (cargo build) o PATH fallback; sin binario, `npx playwright test` falla en timeout health. Verify full debe incluir `cargo fmt --check` + `npm --prefix desktop run build` + `npx playwright test` (con binary). Gate D: sí feature-add (nuevos specs = símbolos públicos nuevos desde punto vista plan) → Spec tabla obligatoria.

## Impacto mapeado (Regla 0)
- **Archivos leídos completos (antes de editar):**
  - `desktop/playwright.config.ts` (39 líneas, HEAD d2c82cd7) — testDir e2e, webServer `node e2e/serve.mjs --port 8091`, baseURL `${HOST}/dashboard/`, timeout 60s, workers 1, reuseExistingServer !CI.
  - `desktop/e2e/flujo-critico.spec.ts` (115 líneas) — ingest→grid→Inspector Enter→borrado→papelera RESTORE→palette AJUSTES, seedRecords 3 keys, wait Vanta Studio, grid 3 cargados,_ingest UI, row focus Enter, borrado 2 pasos, papelera RESTORE, palette Ctrl+K→AJUSTES→Defaults de búsqueda.
  - `desktop/e2e/daud01-temas.spec.ts` (134 líneas) — light/dark computed styles (bodyBg CREAM/VANTA_BLACK, sidebar, input, shadow), screenshots.
  - `desktop/e2e/helpers.ts` (46 líneas) — APP_BASE, API_BASE, SCREENSHOTS_DIR, seedRecords POST /api/v2/records/batch.
  - `desktop/e2e/serve.mjs` (85 líneas) — rebuild dist-web via vite build --mode web, resolveBinary target/debug|release .exe + PATH fallback, mkdtemp + spawn `server --http -p PORT -d dbDir --dashboard-dir DIST_WEB`, health wait /api/v2/health.
  - `desktop/src/components/proxy/ProxyDashboard.tsx` (288 líneas) — LS_KEY vanta.proxy.url, PROXY_URL_EVENT, proxyUrl(), ttlLabel(), fetchSnapshot(base+"/snapshot"), polling 5s, form CONECTAR, turn reports table, sesiones stageGlyph, writeback, rate_limit, conexión + cambiar URL.
  - `desktop/src/components/proxy/ProxyDashboard.test.tsx` (35 líneas) — ttlLabel + proxyUrl roundtrip.
  - `desktop/src/components/layout/WorkspaceShell.tsx` (1147 líneas, excerpt proxy/surface) — Surface union 12 valores inc proxy/memoria/ajustes, proxyConfigured state via PROXY_URL_EVENT, SideButton PROXY condicional, palette proxyConfigured gate.
  - `desktop/src/store/connections.ts` (128 líneas) — STORAGE_KEY vanta.connections.v1, ProfileKind native|server, ConnectionPrefsStore load/sanitize, upsertProfile, removeProfile, profileTarget.
  - `desktop/src/store/connections.test.ts` (74 líneas) — 4 tests fakeStorage round-trip.
  - `desktop/src/pages/Settings.tsx` (195 líneas) — Section, form add profile (name/kind/path/url/port/token), list profiles con conectar/eliminar, Defaults búsqueda topK/mode, Idioma es/en, gate `!embedded` para perfiles.
  - `desktop/src/transport.ts` (140 líneas) — isEmbedded = !(transport instanceof TauriBackend), HttpBackend fetch mapping, isWasm.
  - `desktop/src/App.tsx` (63 líneas) — isEmbedded prop a WorkspaceShell, TitleBar solo !isEmbedded.
  - `desktop/package.json` (56 líneas) — test:e2e `playwright test`, build `tsc && vite build`, 2863 modules expectativa.
  - `desktop/src/components/graph/GraphLens.tsx` (151 líneas) — Canvas R3F + IQL console, MAX_NODES cap 500, toolbar fit/reset/labels.
  - `desktop/src/components/space/SpaceLens.tsx` (317 líneas) — regl-scatterplot + useProjection, palette categorical, lasso SHIFT+drag, SelectionBar export/delete.
  - `docs/plans/2026-08-25-research-desktop-quickwins.md` (130 líneas) — Wave3 Task10 H-07, contrato multi-perfil + proxy mock, graph/space smoke manual, verify `npx playwright test` verde.
  - `SKILLS-MANIFEST.md` grep `playwright|e2e|proxy|graph|space` → hits `playwright-cli`, `playwright-e2e`, `browser-testing-with-devtools`, `webapp-testing`, `performance-optimization` (bundle), `incremental-implementation`.
  - `docs/operations/BENCHMARKS.md` §9 Desktop (QW9) — baseline 2.71MB, build 24.59s; no E2E section, no gap.
  - `desktop/dist` — 24 files 2.71MB (build 22.97s vite). No editado, artefacto.
- **Referencias hacia dentro (qué importa este archivo):**
  - `playwright.config.ts` → usado por `npx playwright test` + CI `desktop.yml` (si existe). testDir e2e descubre `*.spec.ts`. webServer lifecycle depende de `serve.mjs` + `vanta-cli` binary.
  - `e2e/*.spec.ts` → corren contra `APP_BASE` (embedded web build). flujo-critico es guard permanente; daud01 es visual FIX-D1. Nuevos specs deben mantener workers 1 serial (DB temp compartida) y no romper `seedRecords` idempotente.
  - `ProxyDashboard.tsx` → fetch a `proxyUrl()/snapshot` (origin distinto a APP_BASE). E2E mock vía `page.route("**/snapshot", ...)` intercepta sin necesidad de proxy real. `PROXY_URL_EVENT` + `proxyConfigured` controlan visibilidad botón sidebar/palette.
  - `connections.ts` → localStorage `vanta.connections.v1` compartido entre Settings y ConnectionPanel/WorkspaceShell. E2E puede ejercer via `page.evaluate` + reload; no requiere Tauri IPC.
  - `Settings.tsx` → gate `!embedded` oculta perfiles en web E2E. Para ejercitar UI de perfiles sin Tauri, se puede addInitScript `__TAURI_INTERNALS__` mock o query param `?e2e=1` guard — decisión Spec.
- **Referencias entrantes (qué depende de lo que cambio):**
  - Plan file Wave3 Task10 → gating final quickwins (QW10 cierra Wave3 + plan completo 10/10). Si specs fallan, plan queda IN PROGRESS.
  - `desktop/dist` + `dist-web` → artefactos de serve.mjs rebuild; tamaño ~2.7MB estable.
  - `docs/avance/activo/desktop.md` → registro de progreso desktop; recibirá §QW10 recitation.
  - CI: `npx playwright test` en local y (futuro) `desktop.yml` — debe pasar verde; sin binary `vanta-cli`, health timeout 120s → FAIL.
  - No hot path Rust, no WAL/vector/storage, no concurrencia nueva. Seguridad: ProxyDashboard fetch a URL user-controlled (http://localhost:*) ya permitida por CSP H-01 (connect-src localhost:* https://*); mock no introduce nuevo fetch externo.
- **Veredicto de impacto:** MEDIO-BAJO (3 archivos nuevos e2e + 1 doc smoke manual + posible 1 línea guard Settings/transport para E2E visibility). Reversible (delete specs + revert guard). No runtime prod afectado salvo guard E2E detrás de `URLSearchParams.has("e2e")` (ponytail comment). Verify: build + 69/69 unit + `npx playwright test` (requiere binary) + fmt.

## Contrato
E2E desktop specs multi-perfil conexión + proxy dashboard (mock upstream); graph/space quedan smoke visual manual documentado; `npx playwright test` verde

Verificación mecánica:
1. `desktop/e2e/multi-perfil.spec.ts` existe y ejercita store `vanta.connections.v1` (native + server con token) + Settings defaults (topK/mode/lang) persistencia + active embedded health; al menos 3 tests con `expect` y `page.evaluate` localStorage round-trip ✅
2. `desktop/e2e/proxy-dashboard.spec.ts` existe y ejercita ProxyDashboard con `page.route("**/snapshot", mock)` — fixture TurnReports/sesiones/writeback/rate_limit, verifica form CONECTAR, turn table, sesiones TTL, cambiar URL limpia, PROXY surface/palette visibility condicional ✅
3. `desktop/e2e/SMOKE-MANUAL.md` (o `docs/operations/...`) documenta pasos manuales graph/space (GraphLens 3D + SpaceLens UMAP) con checklist visual y comandos reproducibles ✅
4. `npx playwright test` — todos los specs (flujo-critico + daud01 + 2 nuevos) verde (workers 1, timeout 60s, webServer serve.mjs + binary) ✅
5. `npm --prefix desktop run build` — tsc && vite verde (2863 modules, dist assets) ✅
6. `cargo fmt --check` — verde ✅
7. Cierre full: `git add` solo tocados + commit `feat(desktop): DESKTOP-QW10 — ...` + plan recitation + memory_write + progreso Trigger 1

## Herramientas
- Read (playwright.config, e2e/*.ts, ProxyDashboard, connections, Settings, transport, App, GraphLens, SpaceLens, plan, BENCHMARKS)
- Grep (proxy, isEmbedded, vanta.connections, PROXY_URL_EVENT, SMOKE, graph/space)
- Bash (npm run build, npm test, cargo fmt --check, npx playwright test, git status/log/diff, cargo build -p vanta-cli si hace falta binary)
- Edit/Write (e2e/multi-perfil.spec.ts, e2e/proxy-dashboard.spec.ts, e2e/SMOKE-MANUAL.md, posible Settings guard 1 línea)
- campaign_memory_write, campaign_diagnose_pipeline

## Skills
- campaign-executor, progreso, ponytail (base obligatoria)
- playwright-cli (detectado: playwright.config.ts e2e, webServer serve.mjs)
- browser-testing-with-devtools (candidate: verifica computed styles ya en daud01, pero no necesario para mock)
- webapp-testing (detectado: E2E flow con fetch mock)
- incremental-implementation (detectado: 3 archivos nuevos, slice vertical test→code→verify)
- SDP discovery (lifecycle BUILD/VERIFY): keywords `e2e/playwright/mock upstream/proxy dashboard/multi-perfil/graph space smoke/test` → grep SKILLS-MANIFEST hits `playwright-cli` (ya), `webapp-testing` (ya), `browser-testing-with-devtools` (visual, ya via daud01), `incremental-implementation` (ya), `test-driven-development` (lógica nueva E2E? unit tests 69/69 ya verdes, pero E2E es nueva lógica — candidate pero ponytail: E2E es slice test, no TDD puro). **SDP: 2 candidatos adicionales cargados (playwright-cli, webapp-testing) + incremental-implementation si >2 archivos. Total 6.** **SKILLS_CARGADAS: campaign-executor, progreso, ponytail, playwright-cli, webapp-testing, incremental-implementation**

## Spec
N/A — feature-add E2E specs (sí `pub fn`? no Rust, pero sí nuevos test symbols `test()` públicos desde vista plan: agregan `e2e/*.spec.ts` con nuevos `test()` suites). Gate spec-first: feature-add con lógica nueva SÍ requiere Spec tabla (question-gates.md). Contrata E2E multi-perfil + proxy mock + smoke manual. Justificación ponytail: 2 specs + 1 doc, 0 deps nuevas, mock via `page.route` (stdlib Playwright), graph/space manual por WebGL no-automatizable.

| Decisión | Opciones | Elegido | Por qué |
|---|---|---|---|
| Multi-perfil E2E en web embedded (Settings oculta perfiles) | (a) mock `__TAURI_INTERNALS__` + invoke para forzar `!embedded` vs (b) ejercitar store vía `page.evaluate` + testear Settings defaults visibles (topK/mode/lang) + localStorage round-trip native/server | (b) store + defaults | (a) requiere mock Tauri invoke para todos los `vanta_*` (14 métodos) + fragile. (b) reusa storage inyectable ya testeado unit (4/4) y valida persistencia real via E2E `page.evaluate` + reload + UI defaults que SÍ están en embedded (topK/mode/lang = mismo store `vanta.connections.v1`). Perfiles UI Tauri queda smoke manual breve + unit coverage; E2E valida contrato "multi-perfil conexión" a nivel store + defaults, no regresa flujo-critico. Ponytail: 0 mock Tauri, 0 binary extra. |
| Proxy dashboard mock | (a) Node HTTP server mock separado (puerto 8096) vs (b) `page.route("**/snapshot", fixture)` | (b) page.route | (a) requiere spawn proceso + lifecycle + port conflict + CI setup. (b) es 1 línea Playwright nativo, determinista, sin proceso extra, mismo origin mock. Fixture SnapshotWire deterministico. |
| Graph/space automation | (a) E2E auto con canvas assertions vs (b) smoke visual manual documentado | (b) manual | GraphLens three+Canvas R3F y SpaceLens regl-scatterplot WebGL no son assertables sin screenshots flaky + UMAP es O(n²) y no-determinista por data. Contrato explícito: "graph/space quedan smoke visual manual documentado". |
| Smoke manual dónde | (a) `desktop/e2e/SMOKE-MANUAL.md` vs (b) `docs/operations/...` | (a) e2e/ | (a) colocada junto a specs para que `npx playwright test` y dev que abre e2e la encuentren sin buscar docs/operations. Link desde docs/avance si hace falta. |
| Para forzar perfiles UI en E2E (si se decide) | (a) `URLSearchParams.has("e2e")` guard en Settings vs (b) mantener oculto | (b) mantener oculto | YAGNI: store test + defaults test cubren contrato sin cambiar prod code. Si owner pide UI perfiles en web E2E, añadir guard en QW10 follow-up con `// ponytail: e2e-only` 1 línea. |

Evidencia por ítem: Read e2e/ 4 files + ProxyDashboard 288L + connections 128L + Settings 195L + transport isEmbedded + plan H-07 + playwright config webServer serve.mjs + BENCHMARKS §9.

## Steps

### Step 1: Auditoría E2E existente + Spec + task file ✅ DONE
- **Archivos:** `desktop/e2e/`, `desktop/playwright.config.ts`, `desktop/src/components/proxy/ProxyDashboard.tsx`, `desktop/src/store/connections.ts`, `desktop/src/pages/Settings.tsx`, `docs/plans/2026-08-25-research-desktop-quickwins.md`
- **Acción:** Verificar 2 specs existentes (flujo-critico 115L, daud01 134L) + helpers + serve.mjs + contrato H-07. Poblar Blast Radius + Regla 0 + Spec tabla (5 decisiones). Crear task file.
- **Verify:** `Select-String "proxyConfigured" WorkspaceShell` hit + `Select-String "isEmbedded" transport` hit + `Select-String "H-07" plan` hit + task file con Regla 0 completa + Spec tabla llena ✅
- **Estado:** ✅ DONE

### Step 2: Crear specs E2E multi-perfil + proxy mock + smoke manual ✅ DONE
- **Archivos:** `desktop/e2e/multi-perfil.spec.ts` (nuevo), `desktop/e2e/proxy-dashboard.spec.ts` (nuevo), `desktop/e2e/SMOKE-MANUAL.md` (nuevo)
- **Acción:** multi-perfil: 4 tests (embedded health + Settings defaults persist topK/mode/lang + localStorage vanta.connections.v1 native/server round-trip + reload + sanitize). proxy-dashboard: `page.route("**/snapshot", fixture)` 5 tests con TurnReports/sesiones/writeback/rate_limit, form CONECTAR, tabla turns, TTL labels, cambiar URL, PROXY surface condicional. SMOKE-MANUAL.md: checklist GraphLens (3D, fit/reset/labels, IQL) + SpaceLens (UMAP, lasso, export/delete) + comando `npx playwright test` + prereq binary.
- **Verify:** `Test-Path e2e/multi-perfil.spec.ts` 193L ✅ + `Test-Path e2e/proxy-dashboard.spec.ts` 210L ✅ + `Test-Path e2e/SMOKE-MANUAL.md` 83L ✅ + `Select-String "page.route" proxy-dashboard` 5 hits ✅ + `Select-String "vanta.connections.v1" multi-perfil` 4 hits ✅ + `npx playwright test --list` 12 tests ✅
- **Estado:** ✅ DONE

### Step 3: Verificación mecánica build + playwright verde ✅ DONE
- **Archivos:** `desktop/package.json`, `desktop/dist`, `desktop/e2e/*.spec.ts`, `target/debug/vanta-cli.exe`
- **Acción:** `npm --prefix desktop run build` (10.35s, 2863 modules) + `npm --prefix desktop test` (69/69, 24.81s) + `cargo fmt --check` + `cargo build --bin vanta-cli --features server` (16MB, 9m13s) + `npx playwright test --config desktop/playwright.config.ts` (requiere binary; si falta, cargo build). Si playwright falla por binary, documentar deuda + fallback. Fix 3 locators (embedded .first, 429 .first, 5→dd exact, TTL drift 600s/300s + title TTL).
- **Verify:** `npm --prefix desktop run build` exit 0 10.35s 2863 modules ✅ + `npm --prefix desktop test` 69/69 ✅ + `cargo fmt --check` 0 ✅ + `target/debug/vanta-cli.exe` 16MB --features server ✅ + `npx playwright test --config desktop/playwright.config.ts` 12 tests 45.4s 12 passed ✅ + `.\desktop\node_modules\.bin\playwright test --config desktop/playwright.config.ts` 12 passed ✅ + `npx playwright test --list` 12 tests 4 files ✅ + disk 120GB free (WMI) ✅
- **Estado:** ✅ DONE

### Step 4: Cierre — plan + commit + memoria + progreso ✅ DONE
- **Archivos:** `docs/plans/2026-08-25-research-desktop-quickwins.md`, `.opencode/skills/campaign-executor/tasks/DESKTOP-QW10.md`, `desktop/e2e/*`
- **Acción:** Actualizar plan con `=== RECITATION DESKTOP-QW10 ===`, `git add` solo tocados, commit `feat(desktop): DESKTOP-QW10 — E2E multi-perfil + proxy mock (H-07)`, `campaign_memory_write` lesson, `campaign_diagnose_pipeline`, `skill progreso` Trigger 1 (elimina fila Backlog si aplica — pero QW10 es H-07 del plan, no backlog row, así que solo avanza dominio desktop.md).
- **Verify:** `git log --oneline -1` contiene DESKTOP-QW10 ✅ + plan recitation presente ✅ + `campaign_memory_write` entry ✅
- **Estado:** ✅ DONE

## Context Save Point
- **Fecha:** 2026-08-27T23:20
- **Branch:** develop
- **CI pendiente:** `npx playwright test` requiere binary `vanta-cli` (target/debug/vanta-cli.exe) — si no existe, build cargo (~2-5min) antes de verify
- **Decisiones:** (b) store+defaults sobre Tauri mock, (b) page.route sobre HTTP server, (b) manual sobre auto canvas, (a) e2e/SMOKE-MANUAL.md sobre docs/
- **Problemas conocidos:** ninguno — auditoría confirma 2 specs existentes verdes (cuando binary presente), ProxyDashboard polling 5s, connections store 4/4 unit
- **Próxima tarea:** Step 2 — crear 3 archivos nuevos

## Dependencias
- DESKTOP-QW9 ✅ COMPLETED (d2c82cd7, BENCHMARKS §Desktop) — Wave3 Task9 desbloquea QW10
- DESKTOP-QW8 ✅ (release-plz exclude), QW7 ✅ (sparse_vector), QW6 ✅ (CSP) — quickwins 7/10 → QW10 final
- `desktop/src/components/proxy/ProxyDashboard.tsx` + `desktop/src/store/connections.ts` ya en main (no bloqueante)

## Notas
- Ponytail: 2 specs + 1 md, 0 deps nuevas, 0 lógica prod salvo posible 1 línea guard e2e (YAGNI, no añadido). page.route es stdlib Playwright, storage test es 5 líneas page.evaluate. Skipped: Tauri mock invoke (14 métodos), Node mock server (port lifecycle), canvas screenshot assertions (flaky), ADR humano (owner escribe si quiere formal).
- Regla 11: smoke manual sin claims numéricos sin fuente; E2E specs citan fixture determinista (SnapshotWire mock) con comandos reproducibles `npx playwright test`.
- Gate D disparado (feature-add specs) → Spec tabla llena con 5 decisiones + justificación ponytail.

