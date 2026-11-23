---
id: FIND-594
title: "Detectar truncamiento de 1 registro en verify_shard_counts (src/wal_sharded.rs)"
status: completed
type: test
plan: docs/dev/plans/2026-11-07-consolidacion-tecnica-pre-v0100.md (Task 17)
---

# FIND-594: Detectar truncamiento de 1 registro en `verify_shard_counts` (`src/wal_sharded.rs`)

## Metadata
- **Plan file:** `docs/dev/plans/2026-11-07-consolidacion-tecnica-pre-v0100.md`
- **Fuente:** `docs/dev/Backlog.md` (Phase 60 Caos L647) / Plan Task 17
- **Esfuerzo:** 🟡 2h
- **Prioridad:** 🟠
- **Tipo:** Rust (`src/wal_sharded.rs`, `tests/storage/wal_resilience.rs`)
- **Turns estimados:** 10
- **Creado:** 2026-11-16T12:00
- **last-synced:** 2026-11-18
- **Estado:** ✅ COMPLETED
- **Commits:** `9e9bc446c` (test-only: la detección ya existía desde `44bebc567`, premisa invalidada en Paso 0; fijada con tests unitarios y E2E)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps de ejecución restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `ShardedWal::recover_and_verify` (`src/wal_sharded.rs`) |
| Callees | `verify_shard_counts` (`src/wal_sharded.rs:69-94`) |
| Implicaciones | Sin cambio en código de producción (el guard de monotonía `shard_counts[i] > shard_counts[i-1]` ya existía desde `44bebc567`); blindado con tests unitarios y E2E plain+cifrado. |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** `src/wal_sharded.rs`, `tests/storage/wal_resilience.rs`.
- **Archivos referenciados hacia dentro:** `verify_shard_counts` (`src/wal_sharded.rs:69-94`), `ShardedWal` (`:498-516`).
- **Archivos que referencian a los editados:** `tests/storage/wal_resilience.rs:413-513`.
- **Veredicto impacto:** bajo (test-only tras invalidación de premisa en Paso 0).

## Contrato
`cargo nextest run --profile audit -p vantadb -E "test(wal_sharded) or test(shard)" --locked --build-jobs 2`

## Fase 1 — Evidencia de Debugging (GATE — Paso 0)
- **Repro / Paso 0:** Al inspeccionar `src/wal_sharded.rs:69-94` en `HEAD`, `verify_shard_counts` ya validaba tanto `count + 1 < max_count` como la monotonía del prefijo round-robin (`shard_counts[i] > shard_counts[i - 1]`, introducido en `44bebc567`), además de recalcular `next_shard = durable_records % num_shards` (`:498-516`).
- **Hipótesis:** La premisa de que `[2, 1, 2, 2]` pasaba sin error estaba obsoleta en `HEAD`; debía fijarse con tests de regresión unitarios y E2E (plaintext, cifrado y truncamiento legítimo de cola).
- **1 variable controlada:** Adición de tests de regresión cubriendo huecos de 1 registro en shard intermedio (`[2, 1, 2, 2]`) frente a prefijos round-robin válidos (`[2, 2, 1, 1]`).
- **Test RED/GREEN:** Verificado que `verify_shard_counts(&[2, 1, 2, 2])` devuelve `Err(Error::Corrupt(...))` y `[2, 2, 1, 1]` devuelve `Ok(())`.

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** `verify_shard_counts` rechaza cualquier distribución no monótona (`shard_counts[i] > shard_counts[i-1]`) o con brecha `> 1` entre shards, y acepta prefijos round-robin legítimos tras truncamiento de cola.
- **Comandos de verificación:** `cargo nextest run --profile audit -p vantadb -E "test(wal_sharded) or test(shard)" --locked --build-jobs 2`
- **Deuda pendiente:** ninguna.

## Deuda técnica (Regla 6 — MUST)
**Saldo neto de deuda por PR:** Sin deuda.

## Definition of Done (contrato multi-nivel — P2-08)
| Nivel | Gate |
|-------|------|
| **Task** | Contrato mecánico + tests unitarios (`src/wal_sharded.rs:1006-1039`) y E2E (`tests/storage/wal_resilience.rs:413-513`) |
| **Commit** | `9e9bc446c` |
| **Release** | Integridad de WAL shardeado certificada para `v0.10.0` |

## Herramientas necesarias
- cargo-mcp / nextest

**Skills cargadas (SDP):** `systematic-debugging`, `test-driven-development`, `doubt-driven-development`.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03
| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)
- [x] **SECURITY** — Detecta pérdida o truncamiento de 1 registro en un shard intermedio del WAL (tanto en claro como cifrado).
- [x] **PERFORMANCE** — N/A (cambio test-only; la validación en `open()` es $O(\text{num\_shards})$).

## Steps
- [x] Step 1: Paso 0 — verificar `verify_shard_counts` en `src/wal_sharded.rs:69-94` y confirmar que el guard monótono existe desde `44bebc567`.
- [x] Step 2: Añadir tests unitarios de regresión en `src/wal_sharded.rs:1006-1039` y E2E plain+cifrado+prefijo cola en `tests/storage/wal_resilience.rs:413-513`.

## Dependencias
- Wave 0 completa.

## Review (GATE — agente distinto, P2-01)
- **Revisor:** `vanta-review` + re-auditoría `audit-w1-durability` (#20).
- **Enfoque:** Correcto. Premisa invalidada con evidencia (`44bebc567`); sin cambios innecesarios en producción (Ponytail) y cobertura añadida en `wal_sharded.rs:1006-1039` y `wal_resilience.rs:413-513`.
- **Cómo se probó:** Ejecución del contrato nextest + suite `wal_resilience`.
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
