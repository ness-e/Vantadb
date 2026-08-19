# Plan de Ejecución: Vanta Studio — Fase 3 (web/embebido)

> **Campaign ID:** 9d4f2b8e-7c1a-4e6f-9d3b-5a8c2f7e1d60
> **Inicio:** 2026-08-19
> **Estado:** ✅ FASE 3 COMPLETA (2026-08-19) — 7/7 tareas (WEB-00..06), 7 commits en develop (0cccd326, c81bc23a, c856b3bd, 62d63377, 8b2bc14f, 42d2b26a, 583dad9a + 0da6d33c completions/Cargo.lock). Archivo movido a archive/ al cierre.
> **Fuente:** `docs/research/human-facing-db-ui/06-synthesis/SYNTHESIS.md` (§7 Fase 3: "Servir la misma consola desde el proceso embebido `:puerto/dashboard` estilo Qdrant") + `01-vector-db-consoles/RESEARCH.md` (patrón Qdrant Web UI `localhost:6333/dashboard`, SPA servida por el propio binary) + backlog P25 (server "deliberadamente mínimo", embedded-first) + decisiones del usuario 2026-08-19 (ver Decisiones).
> **Modo:** secuencial con paralelismo en Wave 0/1 (archivos disjuntos).

## Decisiones del usuario (2026-08-19)

| # | Decisión | Valor |
|---|----------|-------|
| D10 | Alcance Fase 3 | **Server-embebido primero, WASM/OPFS después (Fase 4)** — la consola se sirve desde el proceso embebido vía REST + estáticos. WASM/OPFS queda diferido (Fase 4) para no duplicar superficie. |
| D11 | Nivel REST | **REST completo del SDK** — exponer los ~35 métodos públicos del SDK vía REST (NO solo los 15 de la consola). Re-considera la decisión "server as primary boundary" documentada en P25. |
| D12 | Auth dashboard | **Local-first sin auth, bind 127.0.0.1** — coherente con P10 (local-first sin login). El dashboard y los endpoints REST no exigen token por defecto en loopback; si `require_auth`/`api_key` está configurado, se aplica Bearer a `/api/v2/*` (comportamiento existente preservado). MEM-05 (auth 3 capas) llegará con el memory engine. |

## Resumen

| DO | DEFER | SKIP | BLOQUEADO |
|----|-------|------|-----------|
| 1 abstracción de transporte + 3 REST + 1 estáticos + 1 HttpBackend + 1 build web + 1 E2E/docs | WASM/OPFS (Fase 4, D10) · auth fuerte (MEM-05) · adapter Node/Python en web · streaming | web/ Next.js marketing (no tocar) · MCP stdio en web (D10) | — |

## Orden de ejecución

1. **Wave 0 — Transporte (fundamento):** abstraer `vanta.ts` de Tauri invoke → interface de transporte (`TauriBackend` + `HttpBackend` stub). Los componentes React NO cambian. Sin esto, el build web no puede reutilizar la consola.
2. **Wave 1 — Server (Rust, disjunto de Wave 0):** REST completo del SDK en `cli_server.rs` + handlers; servir estáticos `/dashboard` con SPA fallback + flag CLI `--dashboard-dir`.
3. **Wave 2 — Web:** `HttpBackend` real (fetch REST) + build web de la consola (Vite base `/dashboard/`).
4. **Wave 3 — Verificación + cierre:** E2E con Playwright contra server real; docs + ADR (D11/D12).

## Archivos protegidos (NO tocar por sub-agentes)

- `docs/Backlog.md` — migración la hace el lead
- `web/` (Next.js marketing site) — NO es la consola; no tocar
- `vantadb-wasm/` — solo lectura (referencia OPFS/IdbStorage para Fase 4)
- `src/sdk/` (tipos públicos) — cambios solo vía task file con contrato
- `desktop/src-tauri/` — no requiere cambios en Fase 3 (HttpBackend no toca Tauri); tocar solo si una tarea lo exige explícitamente
- Otros workstreams (web/remotion, completions, assets, README.md) — nunca tocar/commitear

---

## Wave 0 — Abstracción de transporte (fundamento)

### Task 1: WEB-00 — Abstraer `vanta.ts` de Tauri invoke (transporte pluggable)
- **Archivos clave:** `desktop/src/vanta.ts` (493L, 35+ funciones con `invoke` directo de `@tauri-apps/api/core`), `desktop/src/transport.ts` (nuevo)
- **Gate Justificación:** reto central de la Fase 3 — hoy TODA la consola habla con Tauri `invoke`. Para servir la misma consola vía HTTP (o WASM en Fase 4) sin reescribir componentes, el transporte debe ser pluggable: `TauriBackend` (invoke) | `HttpBackend` (fetch, stub en WEB-00).
- **Contrato:** interface `VantaTransport { call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> }`; `TauriBackend` implementa `invoke`; factory `getTransport()` que selecciona por entorno (Tauri vs browser); **TODAS** las funciones de `vanta.ts` delegan a `transport.call(cmd, args)` — refactor mecánico 1:1, sin cambios de firma ni de tipos exportados. El stub `HttpBackend` puede lanzar "not implemented" (se completa en WEB-04). Deep-link helpers (`takeDeepLink`) quedan en TauriBackend (no-op en web).
- **Verificación:** `npm run build` verde (tsc + vite); `npx vitest run vanta-deep-link` verde; diff mecánico: 0 cambios de firma en exports.
- **Estado:** ✅ COMPLETO (commit 0cccd326)

## Wave 1 — Server: REST completo del SDK + estáticos

### Task 2: WEB-01 — REST: superficie de la consola (CRUD + search + list + IQL + health/metrics/audit)
- **Archivos clave:** `src/cli_server.rs` (nuevos endpoints en `app()`/`app_with_cors()`), `src/cli_handlers/` (reuso de lógica existente), `src/sdk/api.rs` (fuente de operaciones)
- **Gate Justificación:** D11 (REST completo); la consola web no puede existir sin estos endpoints. Primer tramo: los que la UI consume directamente.
- **Contrato (mapeo 1:1 con el SDK, siguiendo el patrón existente `/api/v2/query`):**
  - `GET  /api/v2/health` → health
  - `POST /api/v2/records` (put) · `POST /api/v2/records/batch` (put_batch) · `GET  /api/v2/records/{ns}/{key}` (get) · `GET  /api/v2/records/{ns}/{key}/versions` (versions + get_version) · `DELETE /api/v2/records/{ns}/{key}` (delete) · `DELETE /api/v2/records?filter=...` (delete_by_filter)
  - `GET  /api/v2/list?namespace=&limit=&cursor=&filter_ops=` (list/listPage, con cursor real — VS-CORE-01)
  - `POST /api/v2/search` (search) · `POST /api/v2/query` (IQL, ya existe) · `GET  /api/v2/autocomplete` (iql_autocomplete)
  - `GET  /api/v2/metrics` (ya existe) · `GET  /api/v2/audit` (audit events)
  - Errores: mismos `VantaError` con status HTTP coherente (400/404/409/500), shape JSON documentado en el contrato del task file.
- **Verificación:** `cargo check --features server` + `cargo test` verde; curl contract por endpoint (script de humo contra `vanta serve` con DB temp).
- **Estado:** ✅ COMPLETO (commit c81bc23a; ojo: test real es `node --test src/vanta-deep-link.test.ts`, vitest da falso negativo pre-existente)

### Task 3: WEB-02 — REST: resto del SDK (export/import, graph avanzado, mantenimiento, threads, snapshots)
- **Archivos clave:** `src/cli_server.rs`, `src/sdk/serialization/impl_export.rs` (export/import), `src/sdk/gds.rs` (GDS), `src/agentic/` (threads), `src/storage/` (snapshots, purge/compact/flush)
- **Gate Justificación:** D11 — "REST completo" incluye lo que la consola aún no usa pero el SDK expone (P25 gaps MCP-16..26 son los mismos métodos sin exponer).
- **Contrato:** `POST /api/v2/export` (export_namespace/export_all) · `POST /api/v2/import` (import_records/import_file/bulk_import) · `POST /api/v2/graph/bfs|dfs|degree|pagerank|centrality` (GDS + traversal) · `POST /api/v2/maintenance/purge|compact|flush|rebuild-index` · `POST /api/v2/threads` + `GET/POST/DELETE /api/v2/threads/{id}` · `GET /api/v2/snapshots` + `POST /api/v2/snapshots/{name}`. Mismo contrato de errores que WEB-01.
- **Verificación:** `cargo check --features server` + `cargo test` verde; curl contract por endpoint.
- **Estado:** ✅ COMPLETO (commit c856b3bd; divergencias: graph/centrality→degree_centrality, compact→compact_layout, thread_id string en wire)

### Task 4: WEB-03 — Servir estáticos `/dashboard` + SPA fallback + flag CLI
- **Archivos clave:** `src/cli_server.rs` (router + fallback), `src/cli_handlers/server.rs` (flag `--dashboard-dir`), `Cargo.toml` (dep `tower-http` `fs` feature si no está)
- **Gate Justificación:** patrón Qdrant Web UI (`localhost:6333/dashboard`) documentado en 01; P10 local-first sin login.
- **Contrato:** flag CLI `vanta serve --dashboard-dir <path>` (default: embebido no activo → 404 con hint). `GET /dashboard` y `/dashboard/*` sirven estáticos vía `ServeDir`; rutas sin extensión de archivo → fallback `index.html` (SPA deep links); `/health` permanece público; D12: sin auth en loopback por defecto (middleware existente preservado si `require_auth` configurado). Sin CORS extra (mismo origin `:port`).
- **Verificación:** `cargo check --features server`; curl: `GET /dashboard/` → index.html; `GET /dashboard/alguna-ruta-spa` → index.html; `GET /dashboard/assets/x.js` → archivo si existe.
- **Estado:** ✅ COMPLETO (commit 62d63377; nota: subcomando real es `vanta-cli server`, axum 0.8 no re-exporta service_fn → tower directa)

## Wave 2 — Web: HttpBackend + build de la consola

### Task 5: WEB-04 — `HttpBackend` real (fetch REST) + factory por entorno
- **Archivos clave:** `desktop/src/transport.ts` (completar `HttpBackend`), `desktop/src/vanta.ts` (factory), `desktop/src/vanta-http-map.ts` (nuevo: mapeo cmd → método/URL REST)
- **Gate Justificación:** WEB-00 dejó el stub; la consola web necesita el fetch real contra los endpoints de WEB-01/02.
- **Contrato:** `HttpBackend.call(cmd, args)` → fetch al endpoint REST correspondiente (tabla de mapeo `vanta_*` → `/api/v2/...`); errores `VantaError` deserializados del shape JSON del server; factory `getTransport()` detecta `window.__TAURI_INTERNALS__` → TauriBackend, sino HttpBackend con base `""` (mismo origin) o `VITE_VANTA_API_BASE` opcional. `vanta.ts` no cambia firma.
- **Verificación:** `npm run build` verde; `npx vitest run` verde (si hay tests de transporte, crearlos mínimos); selfcheck con server real de humo.
- **Estado:** ✅ COMPLETO (commit 8b2bc14f; nota: recuento real 23 comandos vanta_*, vitest NO instalado en desktop — runner real es node:test; 8 comandos rechazados en web por divergencia wire documentada en task file)

### Task 6: WEB-05 — Build web de la consola (Vite base `/dashboard/`, sin Tauri)
- **Archivos clave:** `desktop/vite.config.ts` (modo `web`: `base: "/dashboard/"`, sin plugin Tauri), `desktop/src/main.tsx` (guard runtime Tauri-only), `desktop/src/App.tsx` (ocultar selector de conexiones Tauri-only en modo web; modo "embebido" = HTTP activo por defecto)
- **Gate Justificación:** reutilizar los componentes React de la consola (D10 — server-embebido primero); el build web es el artefacto que sirve WEB-03.
- **Contrato:** `vite build --mode web` produce `desktop/dist-web/` (base `/dashboard/`, assets relativos); la app arranca en browser sin Tauri: transport = HttpBackend contra mismo origin; surfaces HOME/MEMORIAS/ACTIVITY/ÍNDICES/IQL funcionales con datos del server; surfaces que dependen de Tauri (deep links) degradan con aviso, no crash. `npm run build` (modo Tauri) sigue verde sin cambios.
- **Verificación:** `npm run build` (modo desktop) y `vite build --mode web` verdes; servir dist-web con `vite preview --base /dashboard/` y navegar.
- **Estado:** ✅ COMPLETO (sin commit — lead; cambios en 7 archivos desktop/: vite.config.ts mode-web base/outDir, transport.ts isEmbedded, useConnectionState conexión embedded implícita, useDeepLink guard Tauri-only, App/WorkspaceShell ocultan ConnectionPanel, .gitignore dist-web. Mecanismo de modo: `--mode web` → `defineConfig(({ mode }) => …)`, sin .env.web. Imports @tauri-apps NO rompen el build — guards runtime. dist-web agregado a desktop/.gitignore. main.tsx sin cambios: nada Tauri-only)

## Wave 3 — Verificación E2E + cierre

### Task 7: WEB-06 — E2E Playwright contra server real + docs/ADR
- **Archivos clave:** `desktop/scripts/selfcheck-web-e2e.ts` (nuevo, Playwright), `docs/architecture/` (ADR), `docs/Backlog.md` (estado P25/DESKTOP)
- **Gate Justificación:** cierre de fase — probar el flujo completo `vanta serve --dashboard-dir dist-web` → browser `:8080/dashboard` → CRUD/search/graph/export funcionales; documentar D11 (server as primary boundary re-considerado) y D12.
- **Contrato:** e2e: arranca server con DB temp + dist-web, Playwright navega `http://127.0.0.1:8080/dashboard/`, verifica HOME con datos, ingesta un registro, aparece en grid, se edita, se borra (con undo), search híbrida devuelve hits; ADR en `docs/architecture/` (o `docs/plans/` si no existe dir) documentando REST completo + sin-auth-loopback + WASM diferido a Fase 4; Backlog: P25 nota actualizada, tasks WEB-00..06 registradas como completadas.
- **Verificación:** `npx tsx scripts/selfcheck-web-e2e.ts` exit 0; docs presentes.
- **Estado:** ⏳ PENDING

---

## DEFER table

| Item | A cuándo | Motivo |
|------|----------|--------|
| WASM/OPFS (consola 100% browser con `vantadb-wasm` + OPFS/IndexedDB) | Fase 4 | D10; `vantadb-wasm` ya tiene `OpfsStorage`/`IdbStorage`/`OpfsWorkerProxy` — la abstracción de transporte (WEB-00) lo deja enchufable |
| Auth fuerte 3 capas en server | memory engine (MEM-05, F2) | D12; hoy local-first loopback |
| Adapters Node/Python/MCP en la consola web | — | D10; la web solo expone HTTP (y WASM en Fase 4); el desktop sigue multi-connection |
| Streaming (SSE) en `/api/v2/query` | — | no requerido por la consola actual |
| `web/` Next.js marketing → consola | — | no es la consola; el dashboard se sirve desde el proceso embebido |

## Riesgos

| Riesgo | Impacto | Mitigación |
|--------|---------|------------|
| Refactor de `vanta.ts` (35+ funciones invoke → transporte) rompe desktop | Alto | Refactor mecánico 1:1 sin cambio de firmas; `npm run build` + tests existentes después de WEB-00; verify del lead |
| REST completo = superficie grande (35+ endpoints) | Med | 2 waves server (WEB-01 consola / WEB-02 resto); contrato curl por endpoint; verify mecánico por task |
| SPA fallback con ServeDir rompe deep links | Bajo | Fallback solo para rutas sin extensión de archivo (patrón estándar); verificado en WEB-03 |
| Imports Tauri-only rompen build web | Med | Guard runtime en `main.tsx` + `takeDeepLink` no-op en HttpBackend; verificado en WEB-05 |
| CORS en dev (vite 1420 → server 8080) | Bajo | `app_with_cors` ya existe; en prod mismo origin `:8080/dashboard` + `:8080/api/v2` → sin CORS |
| WASM/OPFS queda sin probar hasta Fase 4 | Bajo | D10 explícito; transporte abstracto ya permite enchufarlo |

## RECITATION (progreso — patrón lead)

- **Estado:** ✅ FASE 3 COMPLETA — 7/7 tareas (WEB-00..06), campaña `9d4f2b8e-7c1a-4e6f-9d3b-5a8c2f7e1d60` cerrada 2026-08-19. ADR-026 en `docs/architecture/`. Registro en `docs/progreso/README.md` (WEB-00..06) + mirror `docs/avance/activo/desktop.md`. Backlog P25/P26 actualizados.
- **Próximo (fuera de scope):** Fase 4 — WASM/OPFS (D10); auth fuerte 3 capas (MEM-05); rate limiter default a evaluar para ráfagas UI; endpoints `/api/v2/metrics` JSON + graph DTO desktop para cerrar los 8 rechazos de vanta-http-map.
