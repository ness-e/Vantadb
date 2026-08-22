---
title: "ADR-026: Vanta Studio Fase 3 — REST completo del SDK + dashboard embebido (server as primary boundary re-considerado)"
type: architecture
status: active
tags: [vantadb, architecture, rest, http, dashboard, web, embedded]
last_reviewed: 2026-08-19
aliases: []
---

# ADR-026: REST completo del SDK + dashboard embebido

> Decisión documentada al cerrar la Fase 3 de Vanta Studio (plan
> `docs/plans/2026-08-18-vanta-studio-fase3.md`, campaña `9d4f2b8e-7c1a-4e6f-9d3b-5a8c2f7e1d60`).
> Decisiones del usuario 2026-08-19: **D11** (REST completo) y **D12** (local-first
> sin auth en loopback).

## Contexto

- Hasta Fase 3, el HTTP server era deliberadamente mínimo (`src/cli_server.rs`:
  solo `/health`, `/api/v2/query`, `/metrics`) — decisión documentada en P25
  ("server as primary boundary" diferido).
- El SDK (`src/sdk/api.rs`, `VantaEmbedded`) expone ~35 métodos públicos; la
  consola desktop (`desktop/`) hablaba 100% vía Tauri `invoke`.
- Objetivo Fase 3: servir la MISMA consola React desde el proceso embebido
  (`:puerto/dashboard`, patrón Qdrant Web UI) sin reescribir componentes.

## Decisión

1. **D11 — REST completo del SDK vía `/api/v2/*`** (re-considera "server as
   primary boundary" de P25): todos los métodos del SDK se exponen vía REST —
   no solo los 15 que la consola usa. Mapeo 1:1, mismo shape de errores
   `{success:false,error}` + status HTTP coherente (400/404/409/422/500).
2. **D12 — Local-first sin auth en loopback**: el dashboard y los endpoints
   REST no exigen token por defecto en `127.0.0.1`. Si `require_auth`/`api_key`
   está configurado, se aplica Bearer a `/api/v2/*` (comportamiento existente
   preservado). Auth fuerte 3 capas llega con el memory engine (MEM-05, F2).
3. **WASM/OPFS diferido a Fase 4** (D10): la abstracción de transporte
   (`VantaTransport`, WEB-00) deja el WASM enchufable sin tocar componentes.

## Consecuencias

**Positivas:**
- La consola web (`vite build --mode web` → `dist-web/`, base `/dashboard/`)
  funciona contra el server embebido con `HttpBackend` (fetch REST) — mismo
  código React, cero duplicación.
- El REST completo habilita integraciones externas (curl, scripts, agentes)
  además de la UI.
- Transporte pluggable (`TauriBackend` | `HttpBackend`) prepara Fase 4 (WASM).

**Negativas / riesgos:**
- Superficie REST grande (~35 endpoints) = más código de mantenimiento y más
  superficie de ataque (mitigado por D12 loopback y rate limiter governor).
- Divergencias wire documentadas: `vanta_connect/disconnect/list_connections/
  set_active` (multi-conexión Tauri-only) y `vanta_metrics`/`vanta_graph_*`
  (REST no devuelve el DTO desktop) rechazan en web con error descriptivo.
- Rate limiter default (governor rpm=100, burst 10) puede pisar ráfagas UI
  normales (~12 reqs) — el E2E usa `VANTADB_RATE_LIMIT_RPM=0`; evaluar el
  default para consola embebida local.
- `bulk_import_stream` no expuesto (requiere lector binario); `create_snapshot`
  no funciona con backend InMemory (requiere fjall).

## Referencias

- Plan: `docs/plans/2026-08-18-vanta-studio-fase3.md` (7 tareas WEB-00..06, 4 waves).
- Commits: `0cccd326` (WEB-00), `c81bc23a` (WEB-01), `c856b3bd` (WEB-02),
  `62d63377` (WEB-03), `8b2bc14f` (WEB-04), `42d2b26a` (WEB-05), `583dad9a` (WEB-06).
- Task files: `.opencode/skills/campaign-executor/tasks/WEB-00..06.md`.
- E2E: `desktop/scripts/selfcheck-web-e2e.ts` (Playwright, 11 checks).