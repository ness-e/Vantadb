# WEB-06 — E2E Playwright contra server real (solo parte E2E)

> **Plan:** `docs/plans/2026-08-18-vanta-studio-fase3.md` · **Wave 3** · **Estado:** ✅ COMPLETO (parte E2E)
> **Contexto:** WEB-00 (0cccd326) transporte pluggable · WEB-01 (c81bc23a) + WEB-02 (c856b3bd) REST v2 · WEB-03 (62d63377) `vanta-cli server --dashboard-dir` sirve `/dashboard` + SPA fallback · WEB-04 (8b2bc14f) HttpBackend real + vanta-http-map · WEB-05 (42d2b26a) `vite build --mode web` → `desktop/dist-web/` (base `/dashboard/`).
> **Nota scope:** la parte ADR (`docs/architecture/`) + Backlog la hace el lead — NO tocar.

## Contrato (parte E2E)

1. Arranca server con DB temp: `vanta-cli server --http --dashboard-dir <dist-web>` (flags reales verificados en `src/cli_handlers/server.rs` + `src/cli.rs`: subcomando `server`; `-d/--db` global, `--http`, `-p/--port`, `--host`, `--dashboard-dir`).
2. Playwright navega `http://127.0.0.1:8080/dashboard/`.
3. Verifica HOME con datos (ingesta previa vía REST: `POST /api/v2/records/batch` con 3 registros incl. uno con vector para search).
4. En la UI: un registro aparece en el grid (MEMORIAS), se puede editar (Inspector → PAYLOAD → guardar), se borra (con undo vía Ctrl+Z si la UI lo expone — sí: `undoStore`).
5. Search híbrida devuelve hits (campo de búsqueda global de la Topbar).
6. Exit 0 solo si todo pasa; errores con mensajes claros.

## Impacto mapeado (Regla 0)

**Archivos leídos completos:**
- `docs/plans/2026-08-18-vanta-studio-fase3.md` (141L) — Task 7 WEB-06 contrato + Wave 3
- `src/cli_handlers/server.rs` (356L) — `cmd_server(db_path, http, mcp, port, host, require_auth, dashboard_dir, memory_limit, verbose)`; subcomando `server`; HTTP requiere feature `server`
- `src/cli.rs` (410L) — `Server { --http, --mcp, -p/--port, --host, --require-auth, --dashboard-dir }`; `-d/--db` global (env VANTA_DB, default ./db)
- `src/cli_server.rs` — rutas: `/api/v2/health`, `POST /api/v2/records` (VantaMemoryInput), `POST /api/v2/records/batch` (Vec<VantaMemoryInput>), `GET /api/v2/records/{ns}/{key}`, `DELETE ...`, `GET /api/v2/list`, `POST /api/v2/search`, `POST /api/v2/query`, `GET /api/v2/audit`; `mount_dashboard` sirve `/dashboard` con SPA fallback (404 hint sin dir)
- `desktop/package.json` (46L) — **sin playwright ni tsx**; scripts existentes usan node 24 type-stripping (`node scripts/selfcheck-*.ts`)
- `desktop/scripts/selfcheck-retrieval.ts` / `selfcheck-vs14.ts` — patrón selfcheck: asserts + exit code, corren con `node` (type-stripping nativo, sin deps)
- `desktop/src/vanta-http-map.ts` (407L) — wire shape REST exacto (VantaMemoryInput, metadata tagged `{"String": v}`, searchToRequest, list)
- `desktop/src/components/layout/WorkspaceShell.tsx` (835L) — selectores UI: sidebar `MEMORIAS`, Topbar search `aria-label="Búsqueda global"`, notice `role="alert"`, grid en section `aria-label="Memorias"`, Inspector `aria-label="Inspector de registro"`, Ctrl+Z global (skip inputs)
- `desktop/src/components/DataExplorer.tsx` (889L) — grid table, fila por `<tr>`, DeleteButton `aria-label="Mover {id} a papelera"` → confirmar `BORRAR`, `undoStore.softDelete`, "N loaded"
- `desktop/src/components/inspector/Inspector.tsx` (287L) — tabs GENERAL/METADATA/VECTOR/PAYLOAD/HISTORIAL, commit explícito `GUARDAR` (dirty), flash `✓ guardado v{n}`
- `desktop/src/components/inspector/PayloadTab.tsx` (80L) — modo preview default → botón `editar json` abre CodeMirror (`.cm-content` editable)
- `desktop/src/components/ResultsList.tsx` (35L) — `<ol class="results"><li>` con id/texto/score
- `desktop/src/components/home/HomeOverview.tsx` (369L) — HOME: heading `MEMORIA EN VISTA`, cards `Total records` / `Por namespace` (con nombre ns + count)
- `desktop/src/hooks/useConnectionState.ts` (175L) — modo embebido: conexión implícita `embedded` (HTTP), health probe
- `desktop/src/store/undo.ts` (193L) — `undo()` devuelve label `deshecho · restaurado {id}`; softDelete = remove backend + tombstone
- `desktop/dist-web/` — build web existente (WEB-05), `index.html` + assets

**Referencias hacia dentro (quién importa lo que cambio):**
- `desktop/package.json` → `playwright` devDep NUEVA (única dependencia agregada; `npm run build` no la usa)
- `desktop/scripts/selfcheck-web-e2e.ts` → archivo NUEVO, sin importers (script standalone)

**Referencias salientes:**
- Script → `playwright` (devDep), binary `target/{debug,release}/vanta-cli.exe`, `desktop/dist-web/`
- Script → REST del server embebido (`/api/v2/*`) — read/write sobre DB temp de prueba

**Veredicto de impacto:**
- Cambios SOLO en `desktop/scripts/` (nuevo) + `desktop/package.json` + `desktop/package-lock.json` (devDep playwright@1.61.1, browsers ya descargados chromium-1228 → 0 descarga)
- NO toca source React, `src-tauri/`, `web/`, `vantadb-wasm/`, `src/sdk/`, docs protegidas
- `npm run build` no se ve afectado (tsc/vite no importan scripts/)
- E2E usa DB temp descartable; server en puerto 8080 (contrato) — riesgo: puerto ocupado → el script falla con mensaje claro

## Fase 1 — DISCOVERY (✅)

- ✅ Leer plan file + pipeline-full.md
- ✅ Flags reales del server: `server --http -p <port> -d <db> --dashboard-dir <dir>` (cli.rs:295-321, server.rs:173-214); debug binary `target/debug/vanta-cli.exe` TIENE feature `server` (health 200 verificado)
- ✅ Wire REST verificado con smoke real: `POST /api/v2/records/batch` body `[{namespace,key,payload,metadata,vector,sparse_vector,ttl_ms}]` → 201; metadata `{"String": v}`; vector array plano
- ✅ Playwright: NO en package.json; disponible global (nested `@playwright/cli` alpha 1.61.0-alpha espera browsers 1226 — MISMATCH con 1228 descargados); `playwright-core@1.61.1` usa chromium **1228** → coincide con browsers ya en `%LOCALAPPDATA%\ms-playwright` → `npm i -D playwright@1.61.1` (smoke launch OK)
- ✅ Selectores UI mapeados (WorkspaceShell/DataExplorer/Inspector/PayloadTab/ResultsList/HomeOverview)
- ✅ `node --version` = v24.16.0 → type-stripping nativo (patrón selfcheck existente, sin tsx)

## Fase 2 — EJECUCIÓN

### Step 1: dependencia `playwright` devDep
- [x] `npm install --save-dev playwright@1.61.1` en `desktop/` → package.json + package-lock.json; browsers existentes (chromium-1228) → sin descarga; smoke launch OK

### Step 2: `desktop/scripts/selfcheck-web-e2e.ts`
- [x] Script E2E completo (arranque server + ingesta REST + flujo UI + cleanup + exit code)

### Step 3: fix de raíz en `src/cli_server.rs` (hallazgo E2E → bug real)
- [x] `ListParams.namespace` → `Option<String>` (antes String requerido → axum 400 antes del handler)
- [x] `records_list`: namespace vacío/ausente → `"default"` (alinea con bridge nativo `native.rs:34 DEFAULT_NAMESPACE`)
- [x] `records_search`: namespace vacío → `"default"` (la Topbar busca sin namespace)
- [x] Test `v2_errors_map_status` actualizado: list sin namespace 200 (era 400)

## Fase 3 — VERIFICACIÓN

- [x] `node scripts/selfcheck-web-e2e.ts` exit 0 (desde desktop/) contra server real + dist-web — PASS (11 checks)
- [x] Artifacts de prueba limpiados (DB temp, proceso server, e2e-*.png de corridas fallidas)
- [x] `npm run build` (desktop) ✅ verde — sin cambios de source React
- [x] `node --test src/vanta-deep-link.test.ts src/vanta-http-map.test.ts` → 22/22 pass
- [x] `cargo test --lib --features server` → 1827 passed / 0 failed (incl. v2_errors_map_status actualizado)

## Fase 4 — CIERRE

- [x] Actualizar task file (estado final)
- [x] NO commit (lo hace el lead)
- [x] Devolver bloque RESULTADO

## Notas / Decisiones

- **Por qué playwright@1.61.1 (no alpha global):** el playwright global instalado (1.61.0-alpha, nested bajo `@playwright/cli`/`@playwright/mcp`) espera browsers 1226; los browsers ya descargados son 1228 → alpha falla al lanzar. `playwright-core@1.61.1` y `1.61.0` usan chromium **1228** → coincidencia exacta, cero descarga. (Verificado contra browsers.json de los tarballs.)
- **Runner:** node 24 type-stripping nativo (`node scripts/selfcheck-web-e2e.ts`), mismo patrón que selfcheck-retrieval/vs14 — tsx no instalado, no hace falta.
- **BUG REAL cazado por el E2E (fix de raíz, 3 líneas en cli_server.rs):** la consola web lista/busca SIN namespace (HomeOverview `list({limit})`, sidebar `useNamespaceCounts list(500)`, DataExplorer `listPage({limit,cursor})`, Topbar `search({query})`) — el bridge nativo (src-tauri native.rs) defaulta vacío → `"default"`, pero `/api/v2/list` rechazaba con 400 (campo requerido + guard del handler) y `/api/v2/search` fallaba con ValidationError del SDK. Resultado en web: HOME "no se pudo leer el backend", sidebar sin namespaces, grid sin datos, search roto. WEB-05 no lo detectó (verificó solo estáticos + unit tests, nunca corrió en browser). Fix: `ListParams.namespace: Option<String>` + default `"default"` en list/search. El test `v2_errors_map_status` documentaba el 400 → actualizado.
- **Rate limiter (hallazgo para el lead, NO fixeado):** `protected` (toda la API v2) lleva governor con rpm=100 → burst 10 + 1 token/600ms. Un flujo UI normal (mount ~8 reqs + acciones en ráfaga) lo pisa: el E2E recibió `429 Too Many Requests` en un PUT de undo con ~12 requests en ventana corta. La consola local embebida es un caso legítimo de ráfaga; el harness E2E lo desactiva con `VANTADB_RATE_LIMIT_RPM=0`. Decisión de exponer/ajustar por defecto: lead (security posture).
- **Undo en la UI:** el borrado del grid usa `undoStore.softDelete` (remove backend + tombstone); Ctrl+Z global (WorkspaceShell keydown) → `undoStore.undo()` restaura vía `vantaPut` y muestra notice `deshecho · restaurado {id}` — el E2E verifica borrado (404 en REST + fila fuera del grid) y undo (notice + 200 en REST).
- **Search híbrida:** topbar `Búsqueda global` → `search({query, top_k:8})` → ResultsList; E2E verifica `POST /api/v2/search` con text_query + query_vector (híbrido BM25+vector) devuelve el hit con vector, y en UI el hit aparece en ResultsList.
- **Test preexistente roto (NO tocado, delegado al lead):** `cargo test --features server` falla al COMPILAR `cli_tests` — `src/cli_handlers/server.rs:1185` llama `cmd_server(...)` con 8 args (firma actual espera 9, dashboard_dir agregado en WEB-03). Preexistente a esta sesión (`git diff` vacío en server.rs). No bloquea lib tests ni el E2E (el binary compila OK).