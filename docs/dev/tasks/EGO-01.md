---
title: "EGO-01: cursor undefined/Null + string en db.list() (napi)"
kind: task
description: "parse_list_options acepta cursor ausente/Null (sin paginar) y string decimal (WASM); cursor inválido sigue fallando; contrato d.ts y comentario FIND-125 actualizados."
---

# EGO-01: cursor `undefined`/`Null` + string en `db.list()`

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-08-ego-feedback-0.9.0.md` (Task 1, Ola 1)
- **Fuente:** Backlog `EGO-01` ← Ego `VDB-BUG-01` (2026-10-07)
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🔴 Alta
- **Tipo:** Rust (napi binding) + d.ts + tests TS
- **Turns estimados:** 8-12
- **Creado:** 2026-10-08T00:00
- **last-synced:** 2026-10-08T00:00
- **Estado:** ✅ COMPLETED (contrato + unit Rust 8/8 + napi 29/29; review degradado — ver §Review)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `parse_list_options` ← `list` napi (`lib.rs:163`); JS: `NativeVantaDB.list` (`native.ts:326`, pasa cursor tal cual); `db.list(ns, opts)` en `api.test.ts` |
| Callees | Nuevo `get_opt_cursor` (autocontenido); `get_opt_u64` **no se toca** (as_of_ms/valid_window conservan contrato byte-idéntico) |
| Implicaciones | Solo el parse de `cursor` cambia: `Null`/ausente → `None` (antes: throw); string decimal → `Some` (antes: throw). Números: misma aceptación que hoy (int + float integral, espejo de `get_opt_u64`). `dts-header.d.ts` + comentario FIND-125 en `native.ts` se alinean al nuevo contrato. Sin cambio de wire hacia el core (`MemoryListOptions.cursor` intacto). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vantadb-node/src/lib.rs` (`parse_list_options` L803-851, `get_opt_u64` L906-928, tests L1231+, `list` L163-166), `vantadb-node/dts-header.d.ts` (L106-135), `vantadb-node/tests/api.test.ts` (L115-137), `vantadb-ts/src/native.ts` (L326-353), `vantadb-ts/src/types.ts` (L109/134: ya `string | number`)
- **Archivos referenciados hacia dentro:** `lib.rs` ← `MemoryListOptions` (core `src/sdk/types/record.rs`), `Error::from_reason` (napi)
- **Archivos que referencian a los editados:** grep `parse_list_options` = 1 call site (`list`); grep `cursor` en `vantadb-node/tests` = `api.test.ts` paginación; `index.d.ts` se regenera desde `dts-header.d.ts` en el build
- **Veredicto impacto:** bajo — 1 fn Rust nueva + 3 líneas en `parse_list_options` + d.ts + tests; `get_opt_u64` y el resto de params intactos

## Contrato

"`db.list(ns, {cursor: undefined})` y `db.list(ns, {})` → primera página OK; cursor string decimal (`String(next_cursor)`) → misma página que el numérico; `cursor: 'abc'`/`-5`/`5.5` → `cursor must be a number`; `cargo test -p vantadb-node` + `vitest run` (napi, tras rebuild) verdes."

## Spec

Sin feature-add (Phase 1b: no agrega símbolo público — relaja la aceptación de un campo existente). Sin Gate P/D.

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** `get_opt_u64`/`limit`/`as_of_ms`/resto de params sin cambio; cursor float no-integral, negativo, string no-numérico y tipos no-numéricos siguen fallando; paginación numérica existente idéntica (test L117 pinneado)
- **Comandos de verificación:** `cargo test -p vantadb-node` + rebuild `npm run build` en `vantadb-node` + `npx vitest run tests/api.test.ts` + `cargo clippy -p vantadb-node -- -D warnings` + `cargo fmt --check`
- **Deuda pendiente:** ninguna

## Recitation

| Campo recitation (MCP) | Valor |
|------------------------|-------|
| `activeGoal` | EGO-01: cursor Null + string en list() |
| `lastAction` | (se actualiza al cerrar steps) |
| `result` | OK (contrato + unit + napi + clippy/fmt/tsc verificados) |
| `nextAction` | Review P2-01 por agente distinto (pendiente) |
| `contract` | ver §Contrato + §Invariantes |
| `nextTask` | EGO-04 (Ola 1) |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda — ~15 líneas nuevas con tests; sin `unsafe`, sin suppressions.

## Definition of Done

| Nivel | Gate |
|-------|------|
| **Task** | Contrato + `cargo test -p vantadb-node` + vitest napi (tras rebuild) verdes |
| **Commit** | Commit atómico, `fix(node):` + task ID, diff limpio |
| **Release** | `index.d.ts` regenerado en el build con `cursor?: number \| string`; changelog vía release-plz |

## Herramientas necesarias

- cargo (test/clippy/fmt -p vantadb-node), napi build, vitest (vantadb-node)

**Skills cargadas (SDP):** documentation-skill · systematic-debugging (Iron Law: repro primero — el bug se reproduce con test RED: `cursor: undefined` → throw antes del fix) · base-only, sin candidatos adicionales

## Investigation Notes

- TS ya tipa `cursor?: string | number` (FIND-125) y `native.ts` pasa el cursor tal cual con cast borrado — el único que rechaza strings es Rust. El comentario FIND-125 (`native.ts:333-336`, "napi only takes numbers") queda stale con este fix y se actualiza.
- `get_opt_u64` acepta float integral (`5.0`) — el cursor debe espejarlo porque napi entrega números JS como f64.

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 5 (Rust+test, d.ts+comment, rebuild, vitest napi, commit) |
| % completado | 0% |

## Fase 1 — Evidencia de Debugging (GATE — tipo Bug)

- **Repro:** `db.list('ns', { limit: 2, cursor: undefined })` → `VantaError: list: cursor must be a number` (reporte Ego VDB-BUG-01; causa raíz: `obj.get('cursor')` = `Some(Null)`, `as_u64()` = `None`)
- **Hipótesis:** el parse no distingue ausente/`Null` de presente-inválido, y no contempla strings (WASM los emite)
- **1 variable controlada:** solo el parse de `cursor` (resto de `parse_list_options` intacto)
- **Test RED:** test napi `cursor: undefined` debe FALLAR antes del fix y pasar después; idem string decimal

## Fases explícitas — SECURITY | PERFORMANCE

- [x] **SECURITY** — no aplica: parse de input ya validado; strings se validan con `parse::<u64>` estricto (sin coerciones raras, sin trim que oculte); sin trust boundary nuevo
- [x] **PERFORMANCE** — no aplica: parse O(1) por llamada, fuera de hot path

## Steps

### Step 1: `get_opt_cursor` + tests Rust
- **Archivos:** `vantadb-node/src/lib.rs`
- **Acción:** fn nueva (ausente/Null→None; Number int o float integral finita ≥0→Some; String trim+parse u64→Some; resto→`cursor must be a number`) + usarla en `parse_list_options`; tests unitarios espejo del módulo (None/num/float/string/negativo/float-no-integral/basura)
- **Verify:** `cd vantadb-node && cargo test --lib` → 8/8 ✅ (nota: `-p vantadb-node` no resuelve — crate standalone fuera del workspace)
- **Estado:** ✅ COMPLETED

### Step 2: d.ts + comentario TS
- **Archivos:** `vantadb-node/dts-header.d.ts`, `vantadb-ts/src/native.ts`
- **Acción:** `cursor?: number | string` (+ doc); actualizar comentario FIND-125 (napi ahora acepta ambas)
- **Verify:** `npx tsc --noEmit -p vantadb-ts` → exit 0 ✅
- **Estado:** ✅ COMPLETED

### Step 3: Rebuild del binding
- **Archivos:** —
- **Acción:** `npm run build` en `vantadb-node` (regenera `.node` + `index.d.ts`); confirmar que `index.d.ts` lleva `cursor?: number | string`
- **Verify:** `index.d.ts:112` ✅ + `.node` 2026-10-08 ✅ (build release ~4.5min)
- **Estado:** ✅ COMPLETED

### Step 4: Tests napi + clippy/fmt
- **Archivos:** `vantadb-node/tests/api.test.ts`
- **Acción:** casos: `cursor: undefined` → OK; `{}` → OK; `String(next_cursor)` → misma página que numérico; `'abc'`/`-1`/`1.5` → throw; paginación existente intacta
- **Verify:** `npx vitest run tests/api.test.ts` → 29/29 ✅ + `cargo clippy --all-targets -- -D warnings` ✅ + `cargo fmt --check` ✅ (tras `cargo fmt`)
- **Estado:** ✅ COMPLETED

### Step 5: Commit local
- **Archivos:** —
- **Acción:** commit `fix(node):` + task ID (local, sin push)
- **Verify:** `git log --oneline -1`
- **Estado:** ⬜ PENDING

## Dependencias

- Ninguna (Ola 1; `lib.rs` serializado con EGO-05/06 — esos van en Olas 2-3)

## Review (GATE — agente distinto, P2-01)

> Fallback degradado (sin subagente disponible en esta sesión): auto-revisión adversarial + escalado al owner. Sin review externo u OK del owner, la tarea no se da por certificada.

- **Revisor:** self-review degradado (implementador) — pendiente agente distinto / owner
- **Enfoque:** alternativa considerada: extender `get_opt_u64` con brazo string vs bloque dedicado. Elegido bloque inline en el único call site: `get_opt_u64` conserva contrato byte-idéntico para `as_of_ms`/`valid_window` (cero blast radius colateral). Desviación del task file (decía "fn nueva") documentada: inline es menor diff con un solo caller
- **Cómo se probó:** evidencia mecánica: `cargo test --lib` 8/8; rebuild release + `vitest` napi 29/29 (RED previo: reporte VDB-BUG-01 + lectura determinista `Some(Null)→as_u64→None→Err`); `clippy --all-targets -D warnings` ✅; `fmt --check` ✅; `tsc` ✅
- **Checklist anti-hábitos tóxicos:**
  - [x] Salidas ejecutadas de verdad
  - [x] Contrato 6/6 verificado (undefined, omitido, numérico, string, inválidos + paginación previa)
  - [x] Sin fallos ignorados (suite napi completa 29/29, no solo el test nuevo)
  - [x] Saturación: 1 call site de `parse_list_options` (grep); `index.d.ts` regenerado confirma el contrato público
  - [x] Sin bucles sin diagnóstico
- **Veredicto:** ⚠️ degradado — implementación + verify OK; falta review externo (owner o `vanta-review`)

## Notas

- Plan Ola 1: EGO-02 ✅ (`c82f4b2d`). EGO-04 pendiente en paralelo (archivos disjuntos).
