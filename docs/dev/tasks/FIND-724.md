---
title: "FIND-724: `test_snapshot_quiesce_capture_waits_for_held_writer` es un assert de scheduling (no de comportamiento)"
kind: task
status: active
description: "El test del quiesce de snapshot medía el scheduling, no el producto: el spin de 10000 yields sobrevivía la paciencia acotada y reportaba el Timeout documentado, retriable, como si el capture hubiera fallado. Ahora prueba el wait con señal de progreso + una ventana completa, y fija la paciencia acotada como contrato"
---

# FIND-724: `test_snapshot_quiesce_capture_waits_for_held_writer` es un assert de scheduling (no de comportamiento)

## Metadata
- **Plan file:** `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md` (§6.b, residuales del run)
- **Fuente:** `docs/dev/Backlog.md:757` (monitor CI `ec1bf9d58`, 2026-11-24)
- **Esfuerzo:** 🟢 2-3h
- **Prioridad:** 🟡 Media (bloquea el required `Tests (macOS)` del ruleset `develop`)
- **Tipo:** Rust (test-only)
- **Turns estimados:** 8
- **Creado:** 2026-11-24T00:00
- **last-synced:** 2026-11-24T00:00
- **Estado:** ⏳ IN PROGRESS
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 5 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `src/storage/engine/tests/mod.rs` (declara el módulo); `create_snapshot` (`src/storage/engine/mod.rs:1649`) es el sujeto bajo prueba |
| Callees | `create_snapshot_inner` → `acquire_snapshot_quiesce` (`mod.rs:1515`, 5 intentos × `insert_lock_timeout_ms`) → `flush_locked` (`maintenance.rs:56`) |
| Implicaciones | Ningún cambio de producto ni de contrato público. El assert del test cambia de "spin temporal" a "señal". Riesgo real: volverlo siempre-verde (pre-mortem b). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/storage/engine/tests/snapshot_quiesce.rs` (391 L) · `src/storage/engine/init.rs:430-474` (`data_dir = base_path/data`) · `.opencode/rules/concurrency-async.md` · `.opencode/rules/durability.md`
- **Leídos verbatim vía `codegraph_codegraph_explore` (tratados como ya leídos):** `create_snapshot` / `create_snapshot_inner` (`src/storage/engine/mod.rs:1612-1702`), `acquire_snapshot_quiesce` (`:1505-1545`), `acquire_insert_lock` (`:1490-1503`), `flush_locked` (`src/storage/engine/maintenance.rs:33-100`), `write_snapshot_format` (`mod.rs:1316-1320`)
- **Referencias hacia dentro:** `use super::*` (helpers `sample_node`, `storage_engine!` del módulo `tests`), `std::sync::mpsc`, `tempfile`, feature gate `fjall`
- **Referencias entrantes:** `grep -n FIND-724` → `docs/dev/Backlog.md:757`, `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md:188`
- **Veredicto impacto:** bajo — test-only; no toca producto, ni firmas, ni wire format, ni WAL.

## Contrato

"`cargo nextest run --profile audit -p vantadb -E 'test(snapshot_quiesce)' --build-jobs 2` pasa, **más 10 corridas seguidas** del mismo filtro sin un fallo, más `cargo nextest run --profile audit -p vantadb -E 'test(storage) or test(snapshot)' --build-jobs 2` verde. Prohibido subir el spin, añadir `sleep`/`#[ignore]`/retries o relajar el assert. El test DEBE caer cuando se quita el wait del producto (mutante verificado). El contrato del producto (`acquire_snapshot_quiesce` espera, pero la paciencia está **acotada** a 5 ventanas) queda fijado por `test_snapshot_quiesce_bounded_patience_surfaces_timeout`."

## Veredicto (a) producto defectuoso vs (b) test mal construido

**(b) — el producto está bien; el escenario del test era inválido por construcción.**

| Evidencia | Dato |
|---|---|
| Error que devuelve el capture con el guard tomado (Windows, guard en el thread principal) | `Timeout { operation: "acquire insert_lock in create_snapshot (FIND-651 quiesce)", duration_ms: 200 }`, `retriable=true`, `elapsed=1.0498578s` = exactamente 5 ventanas × 200 ms |
| Mismo escenario en macOS/CI | `FAIL [1.856s]` = `open` (~0.5 s) + 5 ventanas × 250 ms (1.25 s): el capture **esperó** y luego, por el límite documentado, devolvió `Timeout`. Un fallo temprano (I/O, formato) habría tardado ~0.5 s |
| ¿Es específico de macOS? | No. El mismo `Timeout` de quiesce aparece en Windows con el guard tomado → comportamiento del producto, no plataforma |
| ¿El test viejo pedía algo imposible? | Sí: retenía el guard un tiempo **no acotado** (10 000 `yield_now()`) y exigía paciencia **ilimitada**; `acquire_snapshot_quiesce` la acota a 5 ventanas por diseño (`mod.rs:1505-1537`: "a genuinely stuck critical section still surfaces Timeout rather than hanging the caller forever") |
| ¿El assert viejo medía lo que decía? | No: `is_finished()` es `true` tanto por `Ok` como por `Err`; el mensaje "instead of failing" era prosa |
| Second panic `:110:31` | Es `release_rx.recv().expect("wait for release")` en el **closure del writer** (líneas 105-112): cascada del panic del test (dropea `release_tx` → `RecvError`). No es una segunda causa, y el thread del snapshot no paniqueó |

## Spec (SDD — Phase 1b)

No aplica: la solución **no** agrega símbolos públicos, endpoints, tool MCP ni métodos de binding. Es un test-only fix (Phase 1b: ninguna de las 5 señales matchea — el diff se limita a `src/storage/engine/tests/snapshot_quiesce.rs` + este task file).

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. `create_snapshot` **espera** al quiesce guard (`insert_lock`) en vez de fallar: si un writer lo retiene, el capture bloquea y completa al liberarse (producto: `acquire_snapshot_quiesce`, 5 intentos × `insert_lock_timeout_ms`).
  2. El test no puede volverse siempre-verde: debe seguir observando que el capture *no* completó mientras el guard está tomado.
  3. No se modifica código de producto (`src/storage/**` ≠ tests) en el commit.
- **Comandos de verificación:** ver §Contrato (filtro `snapshot_quiesce` ×11 + suite `storage|snapshot`).
- **Deuda pendiente:** el test no assertea el *número* de ventanas de paciencia (5 intentos) — observarlo requiere un subscriber de `tracing` para el `debug!` de `acquire_snapshot_quiesce`. Queda como nota, no como deuda nueva (la paciencia multi-ventana ya está cubierta por comportamiento: el test falla si el capture falla en vez de esperar).

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda (test-only; elimina el flake sin introducir dependencias ni código de producto).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato §Contrato cumplido (11 corridas del filtro + suite del área) + mutante cae |
| **Commit** | Commit atómico `test(storage): ...`, diff limpio (solo test + task file), verificación mecánica |
| **Release** | N/A — no toca producto ni packaging; se justifica en Notas |

## Herramientas necesarias
- `codegraph_codegraph_explore` (blast radius de `create_snapshot` / `acquire_snapshot_quiesce`)
- `cargo nextest ... -p vantadb` (filtro acotado, nunca suite completa)

**Skills cargadas (SDP):**
- `test-driven-development` [pinned: tests/fix] — el mutante es el RED; sin él el fix no prueba nada.
- `systematic-debugging` [pinned: tests/fix] — Iron Law: causa raíz (assert de scheduling) antes de tocar el test; `condition-based-waiting.md` es la técnica exacta (reemplazar timeouts/ventanas arbitrarias por polling de condición).
- `incremental-implementation` [lifecycle BUILD] — un slice: reescribir un test, verificar, commit.
- `context-engineering` [lifecycle BUILD] — cargar regla de área (`concurrency-async.md`) + fuente verbatim antes de editar.
- `source-driven-development` / `doubt-driven-development` [base Rust core] — `is_finished()`/`JoinHandle` y `mpsc` contra std docs; verificación adversarial del blast radius.
- `deprecation-and-migration` [pinned: storage/schema] — evaluada y **no aplica**: no hay remoción/migración de sistema ni de formato; ningún símbolo deprecado entra o sale.

## Investigation Notes

**Fase 1 — Evidencia de Debugging**

- **Repro:** CI `Tests (macOS)`, run `45510840907`, commit `ec1bf9d58`:
  `panicked at src/storage/engine/tests/snapshot_quiesce.rs:128:5: snapshot must wait for the quiesce guard instead of failing` → 4120 passed, 1 failed. Rerun: **2 de 2**, misma firma, `FAIL [1.856s]`. `git diff --name-only aebc68dab..ec1bf9d58` **no** toca `src/storage/**` → no es regresión de producto.
- **Hipótesis inicial (descartada como incompleta):** "el spin de 10 000 `yield_now()` tarda más que la ventana de paciencia (1.25 s)". Correcta en el mecanismo pero insuficiente: no explicaba *por qué* el capture terminaba (¿`Ok`? ¿`Err`? ¿qué variante?).
- **Hipótesis final (confirmada):** el capture terminaba porque la paciencia del producto es **acotada** (5 ventanas) y el test retenía el guard más que eso → `Err(Timeout)`, el variante documentado y retriable. El assert `is_finished()` no distingue `Ok` de `Err`, así que reportó "el producto falla" cuando el producto hizo exactamente lo diseñado.
- **1 variable controlada:** la sincronización del test (spin por tiempo → dos señales + una ventana de paciencia). El producto no se toca.
- **Test RED (mutante):** `acquire_snapshot_quiesce` sin wait (`try_lock` inmediato) → el test nuevo debe caer. Resultado en §Steps.
- **Técnica:** `systematic-debugging` → `condition-based-waiting.md` (sustituir ventanas arbitrarias por polling de condición).

**Observabilidad elegida (sin ganchos de test en producto):**

1. `create_snapshot_inner` escribe `data/snapshots/<name>/snapshot.format` **inmediatamente antes** de `acquire_snapshot_quiesce()` (`mod.rs:1659-1675`). Ese marker prueba que el capture *llegó* al guard (sin él, "liberá y esperá a que pase" sería vacuo: el thread podría arrancar después del release).
2. Estando en el guard y con el guard tomado, el capture debe **sobrevivir una ventana completa** (`insert_lock_timeout_ms`): solo esperar sobrevive una ventana, y un capture que dejó de esperar completa en microsegundos. Esa es la aserción que mata al mutante.
3. Recién entonces se libera; el capture debe devolver `Ok` y un snapshot completo.

**Por qué no alcanzó con (1) solo:** la primera versión del rewrite verificaba el mutante y **pasaba** — el `try_lock` del mutante podía ejecutarse *después* del release (el marker se escribe antes del acquire, y el thread podía ser despreemptado entre ambos). Ese hueco es exactamente el pre-mortem (b) "volverlo siempre-verde"; lo cerró la observación (2), verificada contra el mutante.

**Hallazgo colateral:** `TIMEOUT_MS` sube 250 → 1000 ms: la observación consume 1.5 ventanas y deja 3.5 de slack antes de que el producto pueda agotar legítimamente sus 5. El test pasa en ~1.5 s; con `is_finished()` inmediato (sin ventana) el margen era µs y el mutante se escapaba.

**Higiene de cierre:** el task file lleva frontmatter canónico (`docs/_schema/frontmatter.schema.json`: `title`/`kind`/`status`/`description`, sin `id` — `additionalProperties: false`); `node scripts/docs/check-docs.mjs` → `GATING: all clear`.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 5 |
| % completado | 0% |

## Steps

### Step 1: Reescribir el test para observar por señal
- **Archivos:** `src/storage/engine/tests/snapshot_quiesce.rs`
- **Acción:** sustituir el spin de 10 000 `yield_now()` por: (1) espera del marker `data/snapshots/patient/snapshot.format` (señal de "el capture llegó al guard"), abortando si el thread retorna antes; (2) `assert!(!snapshot.is_finished())` en ese instante; (3) release + join + `Ok`. `TIMEOUT_MS` 250 → 1000.
  **Corregido tras el mutante:** (2) es `recv_timeout` de **una ventana completa** de paciencia (`insert_lock_timeout_ms`), no un `is_finished()` inmediato — el inmediato dejaba pasar al mutante. Además se añadió `test_snapshot_quiesce_bounded_patience_surfaces_timeout`, que fija la paciencia **acotada** (5 ventanas → `Timeout` retriable) como contrato del producto.
- **Verify:** `cargo nextest run --profile audit -p vantadb -E 'test(snapshot_quiesce)' --build-jobs 2`
- **Estado:** ✅ DONE (8/8 verde; ver Step 2)

### Step 2: 10 corridas consecutivas del filtro
- **Acción:** 10× el mismo filtro; 0 fallos. Una pasada no es evidencia.
- **Verify:** loop de 10 con exit code agregado
- **Estado:** ✅ DONE — `FAILURES: 0 / 10`, cada corrida `8 tests run: 8 passed` (2.6 s – 5.2 s por corrida)

### Step 3: Mutante — quitar el wait del producto (temporal, revertido)
- **Acción:** parchear `acquire_snapshot_quiesce` a `try_lock` inmediato, correr el filtro, confirmar que el test **falla**, revertir con `git checkout -- src/storage/engine/mod.rs` y verificar `git diff` limpio.
- **Resultado:** 1er intento del rewrite → el test **pasaba** con el mutante (hueco: el `try_lock` del mutante podía ocurrir después del release, porque el marker se escribe antes del acquire). Tras añadir la observación de una ventana completa → el test cae con:
  `create_snapshot completed within one 1000 ms quiesce window while the writer held the guard: it must wait for it instead of failing`.
  Mutante revertido; `git diff -- src/storage/engine/mod.rs` vacío.
- **Estado:** ✅ DONE

### Step 4: Suite del área storage/snapshot
- **Verify:** `cargo nextest run --profile audit -p vantadb -E 'test(storage) or test(snapshot)' --build-jobs 2`
- **Estado:** ⬜ PENDING

### Step 5: Cierre (fmt/clippy del paquete, OCR, review, commit local)
- **Verify:** `rustfmt --check` del archivo (no `cargo fmt`: el worktree tiene cambios de otros agentes) · `cargo clippy -p vantadb --all-targets -- -D warnings` · `pwsh dev-tools/ocr-review.ps1` · review `vanta-review`
- **Estado:** ⬜ PENDING

## Dependencias
- Ninguna (carril R, cupo 2 — comparte `target/` con `FIND-721`, `-j 2`).

## Review (GATE — agente distinto, P2-01)

- **Revisor:** vanta-review (tier adversarial: el diff toca `src/storage/**` → review por agente distinto)
- **Enfoque:** ¿el test sigue probando el invariante (y cae con el mutante) o se volvió siempre-verde?
- **Cómo se probó:** mutante verificado + 10 corridas + suite del área
- **Veredicto:** ⬜ pendiente

## Notas
- Prohibido explícito: subir el número de yields, `sleep`, `#[ignore]`, retries, relajar el assert.
- No se usa `loom` (interleavings deterministas): el invariante no necesita explorar el espacio de interleavings — la exclusión mutua del guard ya lo garantiza; añadir la dependencia sería coste sin ganancia (Regla 6 / ponytail).
