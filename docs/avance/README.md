---
title: "Avance — Índice Maestro"
type: index
status: active
tags: [vantadb, avance, index, progreso]
last_reviewed: 2026-08-07
aliases: [docs/avance]
---

# Avance — Índice Maestro

> Registro vivo del trabajo completado en VantaDB, organizado por dominio en lugar de cronología plana. Este índice es el **único punto de entrada** del árbol `docs/avance/`; cada archivo cubre un área y cada entrada conserva su ID original (`DRV-*`, `COMP-*`, `SEC-*`, etc.) para trazabilidad.

## Convenciones

- Los **IDs** se conservan tal cual en los fuentes (`docs/progreso/*`).
- Convención de commits: Conventional Commits estricto (`feat:`, `fix:`, `docs:`, `perf:`, `ci:`, `refactor:`, `chore:`).
- Estado ⚠️ = parcial/deferido; ❌ = WONTFIX o no implementado; ✅ = completado verificado contra código.

## Correspondencias — Bloque 1 (README → dominio)

| Sección origen (`docs/progreso/README.md`) | Destino (`docs/avance/`) |
|---|---|
| Exec Summary & tabla de correspondencia | `README.md` (este archivo) |
| Bloques CORE ENGINE / storage / WAL / HNSW / ACID / IQL | `activo/core-engine.md` |
| Bindings Python / WASM / TS / MCP / adapters | `activo/bindings.md` |
| Frontend Web / SEO / UX / docs de la web | `activo/web-frontend.md` |
| CI/CD, GitHub Actions, release, docker, wheels | `activo/ci-cd.md` |
| Ops: backup, restore, docs API, ejemplo, enterprise | `activo/operaciones.md` |
| Seguridad (SEC / NVs / fuzz / Miri / FFI) | `auditoria/seguridad.md` |
| Dependencias (cargo-deny, dependabot, advisories) | `auditoria/dependencias.md` |
| WONTFIX / decisiones (RELEASE-DEFERRED) | `decisiones/wontfix.md` |
| No-ops / SKIPs / overwrite-ya-existente | `historial/no-ops.md` |
| Cambios de proceso y obediencia | `meta.md` |

## El contenido — Bloque 2 (histórico)

| Fuente | Destino (`docs/avance/historial/`) |
|---|---|
| `BACKLOG_HISTORY.md` | `historial/backlog-history.md` ✅ (copia directa) |
| `README.md` completo congelado 2026-08-03 | `historial/snapshot-2026-08-03.md` ✅ (copia directa) |
| `README.md` completo congelado 2026-08-07 (post-migración) | `historial/snapshot-2026-08-07.md` ✅ (copia directa) |
| `bitacora.md` (julio) | `historial/sesiones/2026-07.md` |
| Sessiones auditoría / razonamientos de 19-jun | `historial/autopsias-2026-06-19.md` |

## Contenido procesado (fase migración 2026-08-07)

| Archivo | Fuentes procesadas | Estado |
|---|---|---|
| `activo/core-engine.md` | README §core, bitacora §CORE P1-P8, ARCHIVO_HISTORICO §engine | ✅ creado |
| `activo/bindings.md` | README §bindings, bitacora §BINDINGS, ARCHIVO_HISTORICO §bindings | ✅ creado |
| `activo/web-frontend.md` | README §web, bitacora §WEB, ARCHIVO_HISTORICO §web | ✅ creado |
| `activo/ci-cd.md` | README §ci-cd, bitacora §CI/CD & DOCKER | ✅ creado |
| `activo/operaciones.md` | README §ops/api, bitacora §DOCUMENTACIÓN parcial | ✅ creado |
| `activo/desktop.md` | README §DESKTOP-01..11 (Fase 12, Tauri) | ✅ creado |
| `auditoria/seguridad.md` | README §SEC/AUD, ARCHIVO_HISTORICO §INV-024, bitacora §TESTING/security | ✅ creado |
| `auditoria/dependencias.md` | README §SEC-01/02, bitacora §C7/DEPENDABOT, ARCHIVO_HISTORICO | ✅ creado |
| `investigaciones.md` | `docs/research/` catálogo + ARCHIVO_HISTORICO §INV | ✅ creado |
| `decisiones/wontfix.md` | bitacora §WONTFIX, ARCHIVO_HISTORICO §WONTFIX, README §DECISIONES | ✅ creado |
| `meta.md` | bitacora §META/PROCESO, ARCHIVO_HISTORICO §Meta, BACKLOG_HISTORY | ✅ creado |
| `historial/no-ops.md` | ARCHIVO_HISTORICO §No-ops, README §no-code | ✅ creado |
| `historial/sesiones/2026-07.md` | `bitacora.md` | ✅ creado |
| `historial/autopsias-2026-06-19.md` | `ARCHIVO_HISTORICO.md` + `historial/snapshot-2026-08-03.md` | ✅ creado |

## Fuentes vivas referenciadas (fuera de `docs/avance`)

Estas 4 carpetas **no se mueven físicamente**: son escritas por pipelines activos (task system MCP, `audit-all.ps1`, `unified-review`) que las buscan por ruta fija. Se integran por catálogo — ver `fuentes-vivas.md` (índice + estado de cada archivo, con rutas directas):

| Carpeta externa | Propietario del pipeline | Catálogo en `docs/avance` |
|---|---|---|
| `docs/plans/` | Sistema de tareas (campaign-server.mjs / pipeline-run.md) | `fuentes-vivas.md` §Planes |
| `docs/reviews/` | `audit-all.ps1`, `/audit` + `unified-review` / `/review` (escritura de reportes) | `fuentes-vivas.md` §Auditorías |
| `docs/research/` | Research (destino de INV-*) | `investigaciones.md` (ya existente, 1:1) |

## Cruz de referencia con `docs/Backlog.md`

- `docs/Backlog.md` mantiene el **catálogo activo** (lo que queda por hacer) — ver `docs/avance/README.md` → `docs/Backlog.md`.
- La limpieza 2026-08-07 (225 filas / 221 IDs eliminados) está documentada en `historial/backlog-history.md`.
- Los completados viven aquí; el *por qué* de cada cierre quedó en la propia fila del backlog antes de eliminarla.

## Atajos rápidos

| Quiero… | Ver |
|---|---|
| Estado del core engine (HNSW/WAL/ACID/IVF/… breve) | `activo/core-engine.md` |
| SDKs y bindings | `activo/bindings.md` |
| Frontend y credibilidad web | `activo/web-frontend.md` |
| Desktop (Tauri, Fase 12) | `activo/desktop.md` |
| Pipeline de release y CI | `activo/ci-cd.md` |
| Operaciones/API/enterprise | `activo/operaciones.md` |
| Auditoría y seguridad | `auditoria/seguridad.md`, `auditoria/dependencias.md` |
| Investigaciones | `investigaciones.md` |
| Planes/auditorías/reviews en curso (carpetas vivas) | `fuentes-vivas.md` |
| Decisiones WONTFIX | `decisiones/wontfix.md` |
| Lo que NO se hizo (con razón) | `historial/no-ops.md` |
| Sesiones de julio 2026 | `historial/sesiones/2026-07.md` |