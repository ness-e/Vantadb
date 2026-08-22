# Plan de Ejecución: Desktop MVP — DESKTOP-02..11 (Tauri v2 multi-connection)

> **Campaign ID: 6d527f85-4943-453d-bb9e-4b95d31cb0ea
> **Campaign ID:** 6d527f85-4943-453d-bb9e-4b95d31cb0ea
> **Inicio:** 2026-08-06
> **Estado: completed
> **Fuente:** `docs/Backlog.md` → Phase 12 DESKTOP (`DESKTOP-02..27`), scoping Task 54 (2026-08-05)
> **Arquitectura base:** `docs/research/DESKTOP-01-tauri-plataforma-desktop.md` + `DESKTOP-01b-investigacion-6-integraciones-arquitectura.md`
> **Scoping previo:** Task 54 — incluir en MVP los 13 (02/03/04/05/08-10/11-14/19/20/24/26); defiere Node/Python (15-18). Este plan cubre el arranque Fase 0→3 (DESKTOP-02..11).

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 10 (DESKTOP-02..11) |
| 🟡 DEFER | 0 |
| ❌ SKIP | 0 |
| 🔴 BLOQUEADO | 0 |
| (y tratado aparte) DESKTOP-01 ya completada = fuente | 1 |

**Nota de scoping heredado (Task 54):** DESKTOP-06 + 07 son "commands + UI = mismo demo"; en este plan se listan separadas para preservar los IDs del backlog, pero se ejecutan en la misma wave y el contrato de 07 valida el demo completo de 06.

## Reglas transversales (todas las tareas)

- **`desktop/` es un workspace PROPIO y desacoplado**: `src-tauri/Cargo.toml` con `[workspace]` vacío de 1 miembro. Depende de `vantadb` por ruta `../..` **nunca** del workspace raíz (`cargo check` raíz queda invariante).
- **Deps de `vantadb`** (todas las tareas Rust): `default-features = false` + `fjall, fs2, memmap2, roaring, advanced-tokenizer`. **Nunca** `cli`, `server`, `prometheus` (evita pull de axum/tokio heavy).
- **Verificación base:** `cargo check` dentro de `desktop/src-tauri` + `cargo check -p vantadb` (raíz) invariante + `npm run build` en `desktop` para UI.
- **Regla 1 (pre-push gate):** `dev-tools/verify.ps1` antes de push; root no se toca.

---

## Tasks

### Task 1: DESKTOP-02 — Scaffold Tauri v2 + propio workspace

- **Esfuerzo:** 🟢 | **Prioridad:** 🔴 (fundación de todo)
- **Archivos clave:** `desktop/src-tauri/*`, `desktop/package.json`, `desktop/src/*`
- **Gate Justificación:** no existe `desktop/`; es el scaffolding sin el que ninguna tarea compila ni se verifica.
- **Gate Result:** ✅ DO
- **Contrato: cargo check -p vantadb --features server ✅, desktop/ existe y commiteado ✅, migración a progreso ya hecha (947403f1) ✅
  - **Commit:** `9feefea7`

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**
  - `create-tauri-app` en `desktop/` (React+Vite+TS template), luego `src-tauri/Cargo.toml` con `[workspace]` vacío para desacoplar del root.
  - `tauri.conf.json` + capabilities mínimas (`core:default`) + un command `ping` de prueba.
  - Validar contra Tauri v2 docs oficiales (no asumir): scaffolding v2, no v1.

### Task 2: DESKTOP-03 — Integrar crate `vantadb` + managed state + healthcheck

- **Esfuerzo:** 🟢 | **Prioridad:** 🔴
- **Archivos clave:** `desktop/src-tauri/Cargo.toml`, `desktop/src-tauri/src/lib.rs`, `src/commands/connection.rs`
- **Gate:** ✅ corazón de la app — la integración nativa es la vía recomendada por DESKTOP-01.
- **Gate Result:** ✅ DO
- **Contrato:**
  - Dep `vantadb` con `default-features=false` + `fjall,fs2,memmap2,roaring,advanced-tokenizer`
  - `AppState { manager, config }` en managed state (`tauri::Builder::manage`)
  - command `vanta_health` abre `VantaEmbedded` en temp dir y reporta capabilities → `HealthReport { backend: "fjall" }`
  - Abrir dos veces el mismo path → error de lock
  - `cargo check` en `desktop/src-tauri` → exit 0
- **Task file:** `skills/campaign-executor/tasks/DESKTOP-03.md`
- **Agente:** `vanta-worker`
- **Estado:** ✅ COMPLETED
- **Branch:**
- **Commit:** `759e2d3e`

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | 1 | Dep `vantadb` subset + AppState managed + `vanta_health` (temp dir, backend fjall) + HealthReport.backend | ✅ | edit/write |
  | 2 | `cargo check -p vantadb` raíz (subset features) + `cargo check` desktop (aislado de WIP paralelo DESK-05/09) + `cargo test --lib` 17 passed | ✅ | bash/cargo |

  **Notas:**
  - Usar `tauri::State<'_, AppState>`; guard sobre `manager`.
  - Temp-dir para healthcheck (nunca crear DB persistente en el healthcheck).

### Task 3: DESKTOP-04 — Trait `VantaConnection` + tipos + errores

- **Esfuerzo:** 🟢 | **Prioridad:** 🔴 (contrato de todo el multi-connection)
- **Archivos clave:** `desktop/src-tauri/src/connections/{trait,types}.rs`, `desktop/src-tauri/src/error.rs`
- **Gate Justificación (Spec-Driven):** el trait es el contrato que DESKTOP-04..14 implementan; definirlo antes de los adaptadores evita romper contratos.
- **Gate Result:** ✅ DO
- **Contrato:**
  - `async_trait` object-safe: `trait VantaConnection { ... }`
  - Tipos compartidos serde: `IngestItem`, `SearchQuery`, `SearchResult`, `MemoryRecord`, `HealthReport`, `ConnectionInfo`, `Capability`
  - `VantaError` unificado `#[non_exhaustive]`, variants por vía `Native/Http/Mcp/Node/Python/Wasm` + `Lock/Timeout/Unsupported`
  - Tests unitarios de serde roundtrip de todos los tipos pasan
  - `cargo nextest run` en `desktop/src-tauri` → exit 0
- **Task file:** `desktop-tauri`→`skills/campaign-executor/tasks/DESKTOP-04.md`
- **Agente:** `vanta-arch` (contract) con impl de `vanta-worker` — trait diseñado por arch (arquitectura multi-connection), implementado con worker
- **Estado:** ✅ COMPLETED
- **Branch:**
  - **Commit:** `dd7d25a1, 363c3f8a7`

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**
  - Este es el contrato de la arquitectura multi-connection diseñada por vanta-arch en DESKTOP-01b (trait `VantaConnection` + `ConnectionManager`).
  - Regla de diseño: "un escritor por path de DB" se modela en error `Lock`.

### Task 4: DESKTOP-05 — `NativeConnection`

- **Esfuerzo:** 🟢 | **Prioridad:** 🔴
- **Archivos clave:** `desktop/src-tauri/src/connections/native.rs`
- **Gate Justificación:** primer adaptador funcional que valida el trait en la práctica.
- **Gate Result:** ✅ DO
- **Contrato:**
  - `VantaEmbedded` embebida; ops síncronas en `spawn_blocking`; mapeo de errores a `VantaError`
  - `capabilities()`
  - Lock del path: segunda conexión mismo path → `VantaError::Lock`
  - Test integración: `put/search/get/delete` en temp dir vía `&dyn VantaConnection` pasa
- **Task file:** `skills/campaign-executor/tasks/DESKTOP-05.md`
- **Agente:** `vanta-worker`
- **Estado:** ✅ COMPLETED
- **Branch:**
  - **Commit:** `5cebcc29`

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**
  - El lock de path usa fs2 (feature ya incluida).

### Task 5: DESKTOP-06 — Commands CRUD async

- **Esfuerzo:** 🟡 | **Prioridad:** 🟠
- **Archivos clave:** `desktop/src-tauri/src/commands/{connection,data}.rs`
- **Gate Justificación:** hace usable el adaptador nativo desde la UI; demo vertical.
- **Gate Result:** ✅ DO
- **Contrato:**
  - `vanta_connect / disconnect / list_connections / set_active / ingest / ingest_batch / search / get / delete / list`
  - delegando al adaptador activo (solo nativo por ahora)
  - Keys/namespaces como `String` (limitación `&str` en async)
  - E2E manual: conectar nativo, ingest 3, search → resultados ordenados
  - `cargo check` en `desktop/src-tauri` → exit 0
- **Task file:** `skills/campaign-executor/tasks/DESKTOP-06.md`
- **Agente:** `vanta-worker`
- **Estado:** ✅ COMPLETED
- **Branch:**
  - **Commit:** `9d2d5319`

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**
  - fusionable con DESK-07 (demo). El E2E de 06 es el contrato de 07.

### Task 6: DESKTOP-07 — Frontend MVP

- **Esfuerzo:** 🟡 | **Prioridad:** 🟠
- **Archivos clave:** `desktop/src/*` (React+Vite, reusando tokens de `web/`)
- **Gate Justificación:** UI mínima que valida health + ingest + search.
- **Gate Result:** ✅ DO
- **Contrato:**
  - `ConnectionPanel`, `IngestForm`, `SearchBar`, `ResultsList`, hook `useConnectionState`
  - bridge `vanta.ts` (wrapper tipado de `invoke`)
  - UI permite conectar nativo, ingresar y buscar; badge de health
  - `npm run build` en `desktop` → exit 0
- **Task file:** `skills/campaign-executor/tasks/DESKTOP-07.md`
- **Agente:** `vanta-worker` (con skills de frontend/design si aplica)
- **Estado:** ✅ COMPLETED
- **Branch:**
  - **Commit:** `10c161aa`

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**
  - Stack: React+Vite en `desktop/` (scaffold propio; NO Next/Vite de web/ — web/ es Next.js 16).
  - Validar ruta de tokens/design-system de web/ antes de reusar.

### Task 7: DESKTOP-08 — CN client IQL tipado

- **Esfuerzo:** 🟢 | **Prioridad:** 🟢
- **Archivos clave:** `desktop/src-tauri/src/connections/server_client.rs`
- **Gate Justificación:** premisa ya corregida 2026-08-05: la API real tiene 3 endpoints (`/health`, `/metrics`, `/api/v2/query` IQL); put/get/delete/list/search van como statements IQL → cliente tipado, NO REST por-op.
- **Gate Result:** ✅ DO
- **Contrato:**
  - Wrapper reqwest (json): config `url/port/token/timeout`
  - Statements IQL mapeados y autenticados (token en header)
  - Tests contra mock HTTP server (axum en dev-deps) pasan
  - `cargo nextest run` en `desktop/src-tauri` → exit 0
- **Task file:** `skills/campaign-executor/tasks/DESKTOP-08.md`
- **Agente:** `vanta-worker` (validar rutas/auth contra `docs/api/HTTP_API.md` + `src/cli_server.rs`)
- **Estado:** ✅ COMPLETED
- **Branch:**
  - **Commit:** `b7aff3a0`

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**
  - No asumir endpoints: validar contra `docs/api/HTTP_API.md` y `src/cli_server.rs` (src/cli_server.rs: `/health`, `/metrics`, `/api/v2/query`).
  - Auth: header (token) + `success:false` en body 200 es error de dominio.

### Task 8: DESKTOP-09 — `ServerConnection`

- **Esfuerzo:** 🟢 | **Prioridad:** 🟢
- **Archivos clave:** `desktop/src-tauri/src/connections/server.rs`
- **Gate Justificación:** 2ª vía de conexión implementada sobre el trait + client.
- **Gate Result:** ✅ DO
- **Contrato:**
  - Implementa `VantaConnection` sobre el client; `connect` valida auth/health
  - Mapeo a `VantaError::Http`; timeouts; `success:false` → error de dominio
  - Integración contra `vantadb-server` real (spawn con `VANTADB_API_KEY` + `--require-auth`): health/put/search OK; server caído → error `Http` limpio
  - `cargo nextest run` en `desktop/src-tauri` → exit 0
- **Task file:** `skills/campaign-executor/tasks/DESKTOP-09.md`
- **Agente:** `vanta-worker`
- **Estado:** ✅ COMPLETED
- **Branch:**
  - **Commit:** `a5f2da1b`

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**
  - El server devuelve HTTP 200 con `success:false` en fallos de dominio → tratarlo como error (no confundir con 4xx).

### Task 9: DESKTOP-10 — Wire Server en commands + UI

- **Esfuerzo:** 🟢 | **Prioridad:** 🟢
- **Archivos clave:** `desktop/src-tauri/src/commands/connection.rs`, `desktop/src/components/ConnectionSelector.tsx`
- **Gate Justificación:** hace usable la vía server desde la UI.
- **Gate Result:** ✅ DO
- **Contrato:**
  - Selector muestra vía "Server" con campos url/puerto/token
  - Conexión entra al registry (`ConnectionManager`) y puede ser activa
  - Desde la UI, conectar a server real, ingest + search por HTTP
  - `cargo check` + `npm run build` → exit 0
- **Task file:** `skills/campaign-executor/tasks/DESKTOP-10.md`
- **Agente:** `vanta-worker`
- **Estado:** ✅ COMPLETED
- **Branch:**
  - **Commit:** `7619c3cb`

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**
  - CORS/seguridad: no exponer el puerto a interfaces no-loopback (validar modelo auth real de `HTTP_API.md`).

### Task 10: DESKTOP-11 — Spawn manager subproceso MCP

- **Esfuerzo:** 🟢 | **Prioridad:** 🟢
- **Archivos clave:** `desktop/src-tauri/src/connections/child_process.rs`
- **Gate Justificación:** inicio de la Fase 3 (adaptador MCP stdio); aísla el sidecar, safety de sub-proceso.
- **Gate Result:** ✅ DO
- **Contrato:**
  - Localizar binario `vantadb-server` (dev: `target/debug/`; release: bundled)
  - Confirmar flag `--mcp` en `vantadb-server/src/main.rs`
  - `tokio::process::Command` con stdio piped, stderr a log, timeout de arranque
  - Test: spawn + kill limpio; `--mcp` confirmado; stderr capturado
  - `cargo nextest run` → exit 0
- **Task file:** `skills/campaign-executor/tasks/DESKTOP-11.md`
- **Agente:** `vanta-worker` con review de `vanta-audit` (manejo de sub-procesos/unsafe de sidecars — ver DESKTOP-01b:1189)
- **Estado:** ✅ COMPLETED
- **Branch:**
  - **Commit:** `d62c1c0c`

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**
  - Fase 3 en el grafo de DESKTOP-01b (§Fasearrivo). Sub-proceso es trust boundary — review de seguridad por vanta-audit.

---

## Fases / Waves (orden de ejecución)

- **Wave 0 (Fase 0 — paralelo, independientes):** DESK-02, DESK-04, DESK-08, DESK-11
- **Wave 1 (Fase 1 — depende de 02/04/08):** DESK-03 (←02), DESK-05 (←04), DESK-09 (←08)
- **Wave 2 (Fase 2):** DESK-06 (←03,05)
- **Wave 3 (Fase 3):** DESK-07 (←06), DESK-10 (←06,09)
- **Checkpoint final:** corazón nativo fúrico + vía server usable + MCP spawn listo; `cargo check` raíz invariante.

## Checklist (gates de calidad entre fases)

- [x] Wave 0: `cargo check -p vantadb` raíz invariante antes y después
- [x] **Checkpoint Wave 0→1:** 4 crates/UI base scaffold compilan ✓
- [x] **Checkpoint Wave 1→2:** trait+types con serde roundtrip ✓; NativeConnection put/get ✓; Server IQL y ServerConnection integrado ✓
- [x] **Checkpoint Wave 2→3:** CRUD commands E2E nativo ✓
- [x] **Checkpoint final:** build de demo completa (Tauri + server) ✓; `scripts/validate-docs-coverage.ps1` ✓
- [ ] `dev-tools/verify.ps1` al cerrar el plan (Regla 1 pre-push)

## Riesgos / Mitigaciones

| Riesgo | Impacto | Mitigación |
|--------|---------|------------|
| Tauri scaffolding con la versión equivocada (v1 vs v2) | Alto | Validar docs oficiales Tauri v2 (v2.11.x) antes de scaffold; `create-tauri-app` actual |
| Dependencia `vantadb` arrastra features pesadas (axum/server) | Alto | `default-features=false` + subset explícito; nunca `server`/`prometheus`/`cli` |
| Desacople de workspace incompleto (root se rompe) | Alto | `[workspace]` vacío en src-tauri; `cargo check -p vantadb` invariante como contrato |
| API de server asumida (endpoints/auth) | Med | DESK-08 valida contra `docs/api/HTTP_API.md` + `src/cli_server.rs` (no asumir) |
| Sidecar MCP (sub-proceso) | Med | Spawn/kill limpio + stderr log; review de vanta-audit |
| Shapes de tipos no comparten la API pública de `vantadb` | Med | Reuso de tipos `VantaEmbedded`/`HealthReport` de la crate core; no redefinir tipos del SDK |

## Fuente
- `docs/Backlog.md` → Phase 12 DESKTOP (`DESKTOP-02..27`)
- `docs/research/DESKTOP-01-tauri-plataforma-desktop.md`
- `docs/research/DESKTOP-01b-investigacion-6-integraciones-arquitectura.md`
- Scoping previo: `docs/plans/2026-08-05-backlog-validation-actions.md` (Task 54)

=== RECITATION ===
Campaign ID: d83a2a1e-5b96-4ee9-a4d7-23afa648cf91
Objetivo activo: Completar backlog desktop-MVP (pipeline run)