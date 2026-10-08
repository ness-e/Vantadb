---
title: "EGO-02: query_vector opcional con text_query/query_sparse en TS"
kind: task
description: "buildSearchRequestBase acepta query_vector ausente (default []) cuando hay text_query o query_sparse; tipo opcional en types.ts; sin cambio de comportamiento en el resto."
---

# EGO-02: `query_vector` opcional con `text_query`/`query_sparse`

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-08-ego-feedback-0.9.0.md` (Task 2, Ola 1)
- **Fuente:** Backlog `EGO-02` ← Ego `VDB-REQ-02` (2026-10-07)
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟡 Baja
- **Tipo:** TypeScript
- **Turns estimados:** 5-8
- **Creado:** 2026-10-08T00:00
- **last-synced:** 2026-10-08T00:00
- **Estado:** ✅ COMPLETED (contrato 5/5 + suite 346/346; review degradado — ver §Review)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `native.ts` (`NativeVantaDB.search`, spread del base), `vantadb.ts` (`Client.search`, spread del base) — ambos pasan el objeto sin tocar `query_vector` |
| Callees | Ninguno (función pura de validación) |
| Implicaciones | Relajación compatible: todo caller que pasa array sigue igual; solo el caso `undefined`/`null` cambia (de throw a default `[]` con texto/sparse). `[]` sin texto/sparse sigue lanzando (test wire03 pinneado). Sin cambio wire (Rust ya acepta vector vacío). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vantadb-ts/src/guards.ts` (función `buildSearchRequestBase`, L204-286), `vantadb-ts/src/types.ts` (`SearchRequest` L193-225), `vantadb-ts/src/__tests__/d5a-validation.test.ts` (L84-178), `vantadb-ts/src/__tests__/wire03.test.ts` (L12-61)
- **Archivos referenciados hacia dentro (imports/deps de los editados):** `guards.ts` ← `native.ts:2`, `vantadb.ts:6` (`buildSearchRequestBase`); `types.ts` ← ambos + tests
- **Archivos que referencian a los editados (referencias entrantes):** grep `query_vector` en `vantadb-ts/src` = `types.ts`, `guards.ts`, `native.ts` (vía spread), `vantadb.ts` (vía spread), tests. Ningún test aserta el mensaje `must be an array` para `undefined` (d5a L134 pinnea `[]` sin texto → sigue throw; wire03 usa `[]` explícito)
- **Veredicto impacto:** bajo — 2 archivos fuente + 1 de tests; sin callers que pasen `undefined` hoy que dependan del throw

## Contrato

"`buildSearchRequestBase({namespace, text_query})` retorna base con `query_vector: []`; sin `text_query`/`query_sparse` y sin vector sigue lanzando `DbError`; `npx tsc --noEmit` + `vitest run` (suite completa TS) verdes."

## Spec

Sin feature-add (Phase 1b: no agrega `pub fn`/tool/endpoint/símbolo público — relaja un campo existente de requerido a opcional). Sin Gate P/D.

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** `[]` sin `text_query`/`sparse` sigue siendo error (intención explícita, wire03 L46); request `null`/no-objeto sigue lanzando; elementos no-finitos siguen lanzando
- **Comandos de verificación:** `cd vantadb-ts && npx tsc --noEmit` (0 errors) + `npx vitest run src/__tests__/d5a-validation.test.ts src/__tests__/wire03.test.ts` + suite completa `npx vitest run`
- **Deuda pendiente:** ninguna

## Recitation

| Campo recitation (MCP) | Valor |
|------------------------|-------|
| `activeGoal` | EGO-02: query_vector opcional |
| `lastAction` | (se actualiza al cerrar steps) |
| `result` | OK (contrato + suite + lint verificados mecánicamente) |
| `nextAction` | Review P2-01 por agente distinto (pendiente) |
| `contract` | ver §Contrato + §Invariantes |
| `nextTask` | EGO-01 (Ola 1) |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda — relajación compatible sin nuevos `any`, sin suppressions.

## Definition of Done

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable verde + `tsc` + vitest suite completa |
| **Commit** | Commit atómico, conventional commit `fix(ts):` + task ID, `git diff` limpio |
| **Release** | `eslint` en archivos tocados; changelog vía release-plz (no manual) |

## Herramientas necesarias

- tsc, vitest, eslint (TS SDK)

**Skills cargadas (SDP):** documentation-skill (task file + plan sync) · base-only + SDP sin candidatos adicionales (cambio de 3 líneas, sin dominio nuevo)

## Investigation Notes

- WIRE-03 ya acepta `[]` con texto pero no ausente (`guards.ts:226-243`); este task completa esa intención.
- Rust acepta vector vacío en search (verificado en análisis previo) — ningún cambio de wire.

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 4 (editar, test nuevo, verify full, commit) |
| % completado | 0% |

## Fases explícitas — SECURITY | PERFORMANCE

- [x] **SECURITY** — no aplica: validación de input ya existente, sin trust boundary nuevo; el default `[]` no amplía superficie (justificación: misma data que el caller podía pasar explícita)
- [x] **PERFORMANCE** — no aplica: sin hot path (validación O(n) sobre vector ya existente)

## Steps

### Step 1: Relajar guard + tipo
- **Archivos:** `vantadb-ts/src/guards.ts`, `vantadb-ts/src/types.ts`
- **Acción:** `query_vector?: number[]` en `SearchRequest` (+ doc); en `buildSearchRequestBase`: `const query_vector = request.query_vector ?? []`, validar array sobre la resuelta, retornar la resuelta
- **Verify:** `cd vantadb-ts && npx tsc --noEmit` → exit 0 ✅
- **Estado:** ✅ COMPLETED

### Step 2: Tests del nuevo comportamiento
- **Archivos:** `vantadb-ts/src/__tests__/d5a-validation.test.ts`
- **Acción:** casos: ausente+texto → `[]`; ausente+sparse → `[]`; ausente sin texto/sparse → throw; `null`+texto → `[]`; no-array (`123`) → throw `must be an array`
- **Verify:** `npx vitest run src/__tests__/d5a-validation.test.ts` → incluido en 27/27 ✅
- **Estado:** ✅ COMPLETED

### Step 3: Verify full + lint
- **Archivos:** —
- **Acción:** suite completa + eslint en tocados
- **Verify:** `npx vitest run` → 20 files, 346/346 ✅ + `npx eslint` exit 0 ✅
- **Estado:** ✅ COMPLETED

### Step 4: Commit local
- **Archivos:** —
- **Acción:** commit conventional `fix(ts): EGO-02 ...` (local, sin push)
- **Verify:** `git log --oneline -1` → `9f6bc7ab` ✅ (pre-commit hook verde)
- **Estado:** ✅ COMPLETED

## Dependencias

- Ninguna (Ola 1, archivos disjuntos)

## Review (GATE — agente distinto, P2-01)

> Fallback degradado (sin subagente disponible en esta sesión): auto-revisión adversarial + escalado al owner. Sin review externo u OK del owner, la tarea no se da por certificada.

- **Revisor:** self-review degradado (implementador) — pendiente agente distinto / owner
- **Enfoque:** alternativa considerada: defaultear en cada caller (`native.ts`, `vantadb.ts`) vs en el guard compartido. Elegido el guard: un solo choke point, cubre native+wasm, precedente WIRE-03. Sin alternativas mejores a la vista
- **Cómo se probó:** evidencia mecánica real (no auto-reporte): `tsc --noEmit` exit 0; `vitest` 20 files 346/346 (incluye 5 casos nuevos EGO-02 + pinnes wire03/d5a previos); `eslint` exit 0 en los 3 archivos
- **Checklist anti-hábitos tóxicos:**
  - [x] Salidas ejecutadas de verdad (logs en esta sesión)
  - [x] Sin done sin acceptance criteria (contrato 5/5 verificado)
  - [x] Sin fallos ignorados (suite completa, no solo focused)
  - [x] Búsqueda saturada: grep de `must be an array` en tests (0 asserts del mensaje viejo para `undefined`); callers vía spread (`native.ts`, `vantadb.ts`)
  - [x] Sin supuestos como evidencia; sin reintentos en bucle (verde al primer run)
- **Veredicto:** ⚠️ degradado — implementación + verify mecánico OK; falta review externo (owner o `vanta-review`)

## Notas

- Plan Ola 1: EGO-01 (Rust) y EGO-04 (text_index) corren en paralelo sin colisión de archivos.
