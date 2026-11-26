---
title: "MEMG-28: Serializar `CheckpointStore::mutate` para eliminar carreras lost-update sobre `checkpoint.json`"
kind: task
description: "cargo nextest run --profile audit -p vanta-memory -E \"test(checkpoint)\" --locked --build-jobs 2 pasa con 0 pérdidas de actualización bajo mutaciones concurrentes"
---

# MEMG-28: Serializar `CheckpointStore::mutate` para eliminar carreras lost-update sobre `checkpoint.json`

## Metadata
- **Plan file:** `docs/dev/plans/2026-11-07-consolidacion-tecnica-pre-v0100.md`
- **Fuente:** `docs/dev/Backlog.md` (Alta 2026-11-07 L852 · origen `Backlog-OpenViking-VantaDB.md` Prereq #2) / Plan Task 21
- **Esfuerzo:** 🟡 2h
- **Prioridad:** 🟠
- **Tipo:** Rust
- **Turns estimados:** 15
- **Creado:** 2026-11-18T12:00
- **last-synced:** 2026-11-18
- **Estado:** ✅ COMPLETED
- **Commits:** `4a6c69f6b`, `93fae5b5f`
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps de ejecución restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vanta-memory::pipeline_worker`, `vanta-memory::services::scheduler`, `vanta-memory::utils::task_checkpoint` |
| Callees | `vanta-memory::utils::checkpoint`, `std::sync::Mutex` |
| Implicaciones | Contrato público de `CheckpointManager` preservado; `write()` restringido a `pub(crate)` para impedir bypasses de la barrera RMW |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vanta-memory/src/utils/checkpoint.rs`, `vanta-memory/tests/task_checkpoint.rs`
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `std::sync::Mutex`, `vantadb::Embedded`
- **Archivos que referencian a los editados (referencias entrantes):** `vanta-memory/src/pipeline_worker.rs`, `vanta-memory/src/services/scheduler.rs`, `vanta-memory/src/utils/task_checkpoint.rs`
- **Veredicto impacto:** bajo — serializa las mutaciones RMW de `checkpoint.json` a nivel de proceso sin cambiar la API pública consumida por los callers externos.

## Contrato
`cargo nextest run --profile audit -p vanta-memory -E "test(checkpoint)" --locked --build-jobs 2` pasa con 0 pérdidas de actualización bajo mutaciones concurrentes.

## Spec (SDD)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Alcance del lock RMW de `checkpoint.json` | A: `CHECKPOINT_RMW_LOCK` estático de proceso con poison-recovery (protege contra múltiples instancias efímeras de `CheckpointManager` por call-site) / B: `Mutex` por instancia (inútil cuando cada caller construye su propio manager) | A | ✅ decidido-por-evidencia (ref: `vanta-memory/src/utils/checkpoint.rs:47-55`) |
| 2 | Visibilidad de `CheckpointManager::write` | A: `pub(crate)` para cerrar bypasses directos fuera de `mutate()` / B: `pub` | A | ✅ decidido-por-evidencia (ref: `vanta-memory/src/utils/checkpoint.rs:139-164`, commit `93fae5b5f`) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** Toda actualización sobre `checkpoint.json` (`mutate`, `update_runner_state` y mutadores derivados) debe ejecutarse bajo `CHECKPOINT_RMW_LOCK` con recuperación ante `PoisonError`; ningún caller externo puede invocar `write()` directamente sorteando `mutate()`.
- **Comandos de verificación:** `cargo nextest run --profile audit -p vanta-memory -E "test(checkpoint)" --locked --build-jobs 2`
- **Deuda pendiente:** ninguna

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | `# MEMG-28: Serializar CheckpointStore::mutate para eliminar carreras lost-update sobre checkpoint.json` |
| `lastAction` | Step 3 ✅ — visibilidad de `write()` cerrada a `pub(crate)` (`93fae5b5f`) y auditoría de verificación #22 PASS |
| `result` | `OK` ↔ ✅ COMPLETED |
| `nextAction` | Ninguna (tarea completada) |
| `contract` | `cargo nextest run --profile audit -p vanta-memory -E "test(checkpoint)" --locked --build-jobs 2` → PASS |
| `nextTask` | `FIND-281` / `FIND-277` |

```yaml
contract:
  verificacion: "cargo nextest run --profile audit -p vanta-memory -E \"test(checkpoint)\" --locked --build-jobs 2 -> PASS (727/727 vanta-memory suite)"
  evidencia:
    - claim: "CHECKPOINT_RMW_LOCK estático de proceso serializa todo RMW en checkpoint.rs con poison-recovery y 0 bypasses"
      evidencia: "vanta-memory/src/utils/checkpoint.rs:47-55,139-164,247-272; vanta-memory/tests/task_checkpoint.rs:283-330"
      confianza: alta
  artefactos:
    - "vanta-memory/src/utils/checkpoint.rs"
    - "vanta-memory/tests/task_checkpoint.rs"
  invariantes: "Toda mutación de checkpoint.json pasa por CheckpointManager::mutate bajo CHECKPOINT_RMW_LOCK; write() es pub(crate)"
  deuda: "ninguna"
  queda_pendiente: "ninguno"
```

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda (reduce deuda de concurrencia eliminando lost-updates y cerrando visibilidad de `write()`).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable cumplido + `cargo fmt --check` + `cargo clippy -p vanta-memory -- -D warnings` + test determinista con channel rendezvous |
| **Commit** | Commits atómicos `4a6c69f6b` y `93fae5b5f`, conventional commits, verificación mecánica |
| **Release** | Incluido en tren `v0.10.0` |

## Herramientas necesarias
- cargo-mcp (check, clippy, fmt, nextest)
- codegraph_codegraph_explore (blast radius)

**Skills cargadas (SDP):** `security-and-hardening`, `doubt-driven-development`, `test-driven-development`, `systematic-debugging`

## Investigation Notes
- Cada call-site de `vanta-memory` construye un `CheckpointManager` efímero sobre `&Embedded`, por lo que un lock alojado en el struct no serializa llamadas concurrentes; se requiere un lock estático de proceso (`CHECKPOINT_RMW_LOCK`).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Fase 1 — Evidencia de Debugging (GATE — solo tipo Bug)

- **Repro:** Dos hilos invocando `CheckpointManager::mutate` o `update_runner_state` en paralelo sobre distintas instancias de `CheckpointManager` leían el mismo snapshot de `checkpoint.json` y el segundo `write()` sobrescribía la mutación del primero.
- **Hipótesis:** `CheckpointManager::mutate` ejecutaba `read() -> f(&mut state) -> write()` sin sección crítica compartida entre instancias.
- **1 variable controlada:** Introducir `CHECKPOINT_RMW_LOCK: Mutex<()>` estático en `vanta-memory/src/utils/checkpoint.rs` envolviendo el ciclo completo `read -> f -> write` en `mutate()` y encaminar `update_runner_state` por `mutate()`.
- **Test RED:** Test concurrente con sincronización por canales (`vanta-memory/tests/task_checkpoint.rs:283-330`) pierde actualizaciones sin el lock estático y pasa a GREEN preservando ambas mutaciones con `CHECKPOINT_RMW_LOCK`.

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — Evita corrupción de estado de cursores L0/L1/L2 y pérdida de progreso por condiciones de carrera; maneja `PoisonError` con `unwrap_or_else(|e| e.into_inner())` para evitar DoS por panic previo.
- [x] **PERFORMANCE** — No aplica hot path vectorial/HNSW; `checkpoint.json` se actualiza únicamente al finalizar pasadas del pipeline L0→L1→L2.

## Steps

### Step 1: Test RED de concurrencia RMW sobre `CheckpointManager`
- **Archivos:** `vanta-memory/tests/task_checkpoint.rs`
- **Acción:** Añadir test determinista con rendezvous por canales para verificar que dos mutaciones concurrentes sobre `CheckpointManager` no pierden estado.
- **Verify:** `cargo nextest run --profile audit -p vanta-memory -E "test(checkpoint)" --locked --build-jobs 2`
- **Estado:** ✅ COMPLETED

### Step 2: Serializar `mutate()` y `update_runner_state` con `CHECKPOINT_RMW_LOCK`
- **Archivos:** `vanta-memory/src/utils/checkpoint.rs`
- **Acción:** Declarar `CHECKPOINT_RMW_LOCK` estático con recuperación de poison y asegurar que todos los mutadores pasen por `mutate()`.
- **Verify:** `cargo nextest run --profile audit -p vanta-memory -E "test(checkpoint)" --locked --build-jobs 2`
- **Estado:** ✅ COMPLETED (`4a6c69f6b`)

### Step 3: Restringir `CheckpointManager::write` a `pub(crate)` y documentar precondición
- **Archivos:** `vanta-memory/src/utils/checkpoint.rs`, `vanta-memory/src/utils/task_checkpoint.rs`
- **Acción:** Cerrar `write()` a `pub(crate)` (hallazgo O5/O6 del review P2-01) para impedir escrituras directas fuera del lock RMW.
- **Verify:** `cargo clippy -p vanta-memory --all-targets --locked -- -D warnings`
- **Estado:** ✅ COMPLETED (`93fae5b5f`)

## Dependencias
- Wave 0 completa (`FIND-466`, `FIND-392`, `FIND-468`, `FIND-467`, `FIND-473`, `FIND-455`, `FIND-550`, `FIND-257`).

## Review (GATE — agente distinto, P2-01)

- **Revisor:** `vanta-review` (`2047a71b`) + re-verificación independiente en auditoría `audit-w2-w3-mem-iql` (#22)
- **Enfoque:** Correcto. El lock a nivel de proceso (`CHECKPOINT_RMW_LOCK`) resuelve la raíz del problema (instancias `CheckpointManager` efímeras por sitio de llamada) y el cierre de `write()` a `pub(crate)` en `93fae5b5f` garantiza 0 bypasses en el crate.
- **Cómo se probó:** Suite `vanta-memory` 727/727 PASS + inspección estática y verificación de test determinista con channel rendezvous en `vanta-memory/tests/task_checkpoint.rs:283-330`.
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
- Commits aterrizados en `develop`: `4a6c69f6b` (implementación principal) y `93fae5b5f` (endurecimiento O5-O6 de review P2-01).
