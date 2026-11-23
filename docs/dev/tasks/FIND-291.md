---
id: FIND-291
title: "Ejecutar shutdown_background_services antes del flush() final en serve_http_or_tls"
status: completed
type: fix
plan: docs/dev/plans/2026-11-07-consolidacion-tecnica-pre-v0100.md (Task 19)
---

# FIND-291: Ejecutar `shutdown_background_services` antes del `flush()` final en `serve_http_or_tls`

## Metadata
- **Plan file:** `docs/dev/plans/2026-11-07-consolidacion-tecnica-pre-v0100.md`
- **Fuente:** `docs/dev/Backlog.md` (L284 `FIND-291`) / Plan Task 19
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟠
- **Tipo:** Rust (`src/server/bootstrap.rs`, `Cargo.toml`)
- **Turns estimados:** 10
- **Creado:** 2026-11-18T10:00
- **last-synced:** 2026-11-18
- **Estado:** ✅ COMPLETED
- **Commits:** `eca33817b` (`fix(server): drain background workers before final storage flush on shutdown`), `a51dfcc1f` (O1-O4: docs de cotas de drenaje)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps de ejecución restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `serve_http_or_tls` (`src/server/bootstrap.rs`, rutas HTTP, TLS y error) |
| Callees | `drain_services_then_flush`, `shutdown_background_services`, `flush_on_shutdown_async`, `shutdown_telemetry` (`src/server/bootstrap.rs:190-194,489-564`) |
| Implicaciones | Garantiza que todos los servicios de fondo (`BackgroundService`, p. ej. `MemoryScheduler` y `MemoryTtlSweeper`) drenen sus últimas escrituras antes del `storage.flush()` final durante el apagado del servidor. |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** `src/server/bootstrap.rs`, `Cargo.toml`.
- **Archivos referenciados hacia dentro:** `BackgroundService::shutdown`, `StorageEngine::flush`, `tokio::time::timeout`.
- **Archivos que referencian a los editados:** `vantadb-server/src/main.rs`, `src/cli_server.rs`.
- **Veredicto impacto:** bajo (acotado a la secuencia de apagado de `src/server/bootstrap.rs`).

## Contrato
`cargo nextest run --profile audit -p vantadb --features server -E "test(shutdown) or test(bootstrap) or test(server)" --locked --build-jobs 2`

## Fase 1 — Evidencia de Debugging (GATE — tipo Bug)
- **Repro:** `src/server/bootstrap.rs:477,484` ejecutaba `flush_on_shutdown_async` del `StorageEngine` *antes* de invocar `shutdown_background_services`, dejando sin `flush()` las escrituras emitidas por workers de fondo durante su drenaje.
- **Hipótesis:** Unificar la salida en `drain_services_then_flush` que ejecute primero `shutdown_background_services` (con cada servicio aislado en `tokio::spawn` y `SERVICE_SHUTDOWN_TIMEOUT = 30s` para que un panic o cuelgue no impida llegar al flush) y luego `flush_on_shutdown_async`, en todas las ramas (HTTP, TLS y error).
- **1 variable controlada:** Inversión del orden a `shutdown_background_services` → `flush_on_shutdown_async` con aislamiento por servicio.
- **Test RED:** Un servicio de fondo que escribe en su `shutdown()` veía su escritura ocurrir después del `flush()` final; con el fix (`bootstrap.rs:947-1086`), el orden registrado es estrictamente `service_shutdown` → `storage_flush`, incluso si otro servicio hace panic o excede el timeout.

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** `flush_on_shutdown_async` se ejecuta siempre después de drenar los servicios de fondo, incluso si algún servicio hace panic o agota `SERVICE_SHUTDOWN_TIMEOUT` (30 s); `shutdown_telemetry()` se invoca una única vez al final (`bootstrap.rs:190-194`); la feature `server` en `Cargo.toml:105` incluye `tokio/time`.
- **Comandos de verificación:** `cargo nextest run --profile audit -p vantadb --features server -E "test(shutdown) or test(bootstrap) or test(server)" --locked --build-jobs 2`
- **Deuda pendiente:** ninguna.

## Deuda técnica (Regla 6 — MUST)
**Saldo neto de deuda por PR:** ≤ 0 (elimina duplicación en ramas HTTP/TLS/error centralizando en `drain_services_then_flush`).

## Definition of Done (contrato multi-nivel — P2-08)
| Nivel | Gate |
|-------|------|
| **Task** | Contrato mecánico + tests de orden, panic y timeout (`src/server/bootstrap.rs:947-1086`) |
| **Commit** | `eca33817b`, `a51dfcc1f` |
| **Release** | Apagado ordenado sin pérdida de escrituras de fondo en `v0.10.0` |

## Herramientas necesarias
- cargo-mcp / nextest

**Skills cargadas (SDP):** `security-and-hardening`, `source-driven-development`, `test-driven-development`.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03
| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)
- [x] **SECURITY** — Un servicio colgado o con panic durante el apagado no bloquea indefinidamente el `flush()` de durabilidad del motor gracias a `tokio::spawn` + `SERVICE_SHUTDOWN_TIMEOUT = 30s`.
- [x] **PERFORMANCE** — N/A (ruta de apagado del servidor).

## Steps
- [x] Step 1: Añadir `tokio/time` a la feature `server` en `Cargo.toml:105`.
- [x] Step 2: Implementar `drain_services_then_flush` y aislamiento con `SERVICE_SHUTDOWN_TIMEOUT = 30s` en `src/server/bootstrap.rs:489-564`, más `shutdown_telemetry()` único en `:190-194`.
- [x] Step 3: Añadir tests unitarios de orden de drenaje, aislamiento de panic y timeout en `src/server/bootstrap.rs:947-1086` y documentar cotas (`a51dfcc1f`).

## Dependencias
- Wave 0 completa.

## Review (GATE — agente distinto, P2-01)
- **Revisor:** `vanta-review` (`2047a71b`, fresh-context) + re-auditoría `audit-w1-durability` (#20).
- **Enfoque:** Correcto. Drena productores antes de `flush_on_shutdown_async` y aísla fallos por servicio con timeout de 30s.
- **Cómo se probó:** Tests unitarios `bootstrap.rs:947-1086` + verificación workspace check/clippy `-D warnings`.
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas de comandos/herramientas que no se ejecutaron.
  - [x] No saltarse la clarificación por "ya sé qué quiere".
  - [x] No declarar done sin verificar contra los acceptance criteria.
  - [x] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial.
  - [x] No hacer un solo intento de búsqueda y darlo por saturado.
  - [x] No copiar sin citar ni presentar supuestos propios como evidencia.
  - [x] No reintentar en bucle sin diagnóstico.
  - [x] No dejar huérfanos los pasos: cada paso conectado al objetivo.
  - [x] No degradar el chequeo de errores en paths de dinero/seguridad.
  - [x] No gastar presupuesto infinito; paradas explícitas.
  - [x] Verificar cobertura SDP (v3).
- **Veredicto:** ✅ approve

## Notas
- Creado y sincronizado en Pasada 1 de auditoría (`2026-11-18`).
