---
title: "Avance — Desktop (Tauri)"
type: domain-log
status: active
tags: [vantadb, avance, desktop, tauri, rust, frontend]
last_reviewed: 2026-08-07
aliases: [DESKTOP]
---

# Avance — Desktop (Tauri)

> Registro consolidado de la Fase 12 — DESKTOP: cliente Tauri v2 con integración Rust nativa del crate `vantadb`. **IDs originales conservados (DESKTOP-01..11).**

## Cobertura rápida

- **Decisión de plataforma:** Tauri v2 (no Electron) — bundle 2-10MB vs 80-200MB, RAM idle ~50MB vs ~120MB+, backend Rust nativo + WebView.
- **Arquitectura:** `desktop/` workspace desacoplado; crate `vantadb` con `default-features=false`; trait `VantaConnection` object-safe (Native/Server); frontend React+Vite+TS con bridge tipado.
- **Estado:** 11/11 tareas ✅ (04-08 al 06-08). Cero cambios de código en el raíz.

---

## Investigación

### DESKTOP-01: Investigar Tauri como plataforma desktop
- **Fecha:** 2026-08-04
- **Resultado:** ✅ Doc `docs/Investigaciones/DESKTOP-01-tauri-plataforma-desktop.md` (20.9KB, 208 líneas). **Recomendación: SI — Tauri v2** con integración Rust nativa (`vantadb` en `src-tauri/`, `VantaEmbedded` en managed state, commands async `vanta_ingest`/`vanta_search`, SIN bridge WASM/OPFS). Tauri v2.11.5 vs Electron v43.2.0. Effort MVP: ~8-13 días hábiles. Solo investigación, cero cambios de código.

## Scaffold & core desktop

### DESKTOP-02: Scaffold Tauri v2 + workspace propio
- **Fecha:** 2026-08-06
- **Resultado:** ✅ `create-tauri-app` en `desktop/`; `src-tauri/Cargo.toml` con `[workspace] members=["."]` desacoplado del raíz; tauri.conf + capabilities mínimas; command `ping`. Tauri v2 (React+Vite+TS), capabilities `core:default`, `com.vantada.desktop`. Commit `9feefea7`.

### DESKTOP-03: Integrar crate `vantadb` + managed state + healthcheck
- **Fecha:** 2026-08-06
- **Resultado:** ✅ Dep `vantadb` con `default-features=false` + subset, `AppState { manager, config }` managed, command `vanta_health`. `vanta_health` abre `VantaEmbedded` en temp dir, devuelve `HealthReport{backend:"fjall"}`; doble open del path → `VantaError::Lock`. `HealthReport` ganó campo `backend`. 17 tests. Commit `759e2d3e`.

### DESKTOP-04: Trait `VantaConnection` + tipos + errores
- **Fecha:** 2026-08-06
- **Resultado:** ✅ `VantaConnection` async_trait object-safe + 9 tipos serde devueltos por todas las vías + `VantaError` `#[non_exhaustive]` (Native/Http/Mcp/... + Lock/Timeout). 17 tests serde roundtrip. Commits `dd7d25a1`, `363c3f8a7`.

### DESKTOP-05: NativeConnection
- **Fecha:** 2026-08-06
- **Resultado:** ✅ `VantaEmbedded` embebida, ops en `spawn_blocking`, lock de path, capabilities. `NativeConnection::open` con lock de path duplicado → `VantaError::Lock`, ops en `spawn_blocking`, health "fjall". 4 tests (trait roundtrip + lock). Commit `5cebcc29`.

### DESKTOP-06: Commands CRUD async
- **Fecha:** 2026-08-06
- **Resultado:** ✅ Commands Tauri `vanta_connect/disconnect/list_connections/set_active/ingest/ingest_batch/search/get/delete/list` delegando al adaptador activo. `ConnectionManager` (tokio RwLock, HashMap + active_id, 14 métodos) reemplaza placeholder `manager: ()`; 11 commands registrados. E2E connect→ingest→search ordenado. 24 tests lib total. Commit `9d2d5319`.

## Frontend

### DESKTOP-07: Frontend MVP
- **Fecha:** 2026-08-06
- **Resultado:** ✅ React+Vite MVP: ConnectionPanel, IngestForm, SearchBar, ResultsList, hook, bridge `vanta.ts`. Bridge tipado + 5 componentes + single-file CSS; `npm run build` (tsc+vite) exit 0. Commit `10c161aa`.

## Conexión server

### DESKTOP-08: Cliente IQL tipado
- **Fecha:** 2026-08-06
- **Resultado:** ✅ `ServerClient` wrapper reqwest (json) config url/port/token timeout; mapea 8 statements IQL (health, metrics, query POST `/api/v2/query`, put/get/delete/list/search) con auth Bearer; `success:false` → `VantaError::Http`. Validado contra `HTTP_API.md`/`cli_server.rs`. 28 tests (11 mock + 17 unit). Commit `b7aff3a0`.

### DESKTOP-09: ServerConnection
- **Fecha:** 2026-08-06
- **Resultado:** ✅ Implementa el trait sobre el client IQL; connect valida auth/health; timeouts; `success:false` como error de dominio. `ServerConnection` delegando a `ServerClient`, timeouts → `VantaError::Timeout`, capabilities [Http]; test e2e con server real gateado por `VANTADB_TEST_SERVER=1`. 21 tests lib + 2 e2e. Commit `a5f2da1b`.

### DESKTOP-10: Wire Server en commands + UI
- **Fecha:** 2026-08-06
- **Resultado:** ✅ `ConnectionSelector.tsx` (loopback-only url/port/token, bridge invoke sin fetch directo); `vanta_connect` ya soportaba `via:"server"`. `npm run build` + `cargo check` exit 0. Commit `7619c3cb`.

### DESKTOP-11: Spawn manager subproceso MCP
- **Fecha:** 2026-08-06
- **Resultado:** ✅ `McpSpawn` con `tokio::process::Command`, stderr→log temporal, timeout; spawn+kill limpio; test gateado si falta binario. Localiza `vantadb-server`; flag `--mcp`; stdio piped. Commit `d62c1c0c`.

---

## Fuentes
- `docs/progreso/README.md` §Detalle de Tareas Completadas (DESKTOP-01..11) / snapshot-2026-08-07.
- `docs/Investigaciones/DESKTOP-01-tauri-plataforma-desktop.md`.
