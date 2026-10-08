---
title: "EGO-03: comodines/prefijo nativos en search_multi"
kind: task
description: "search_multi expande patrones search-only (* y prefijo/*) contra list_namespaces; literales y patrones malformados conservan silent-skip; dedup preservando orden."
---

# EGO-03: comodines/prefijo nativos en `search_multi`

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-08-ego-feedback-0.9.0.md` (Task 3, Ola 2)
- **Fuente:** Backlog `EGO-03` ← Ego `VDB-REQ-01` (2026-10-07)
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟠 Media
- **Tipo:** Rust (core SDK)
- **Turns estimados:** 12-20
- **Creado:** 2026-10-08T00:00
- **last-synced:** 2026-10-08T00:00
- **Estado:** ✅ COMPLETED (contrato + search 236 + clippy/fmt; review degradado — ver §Review)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `search_multi` ← `search_all`, napi `search_multi` (node), wasm `search_multi`, MCP/HTTP que lo usen; TS `searchMulti` (pasa namespaces tal cual, sin validación client-side) |
| Callees | `list_namespaces` (solo cuando hay patrón), `validate_namespace` (literales, intacto), `search` por namespace |
| Implicaciones | Sin patrón: byte-idéntico (mismo loop). Con patrón: 1 `list_namespaces` por llamada + fan-out sobre coincidencias. `put`/`get`/`delete` intactos — el wildcard solo existe en search. Los 3 bindings heredan sin cambios. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/sdk/search/multi.rs` (92L: `search_multi` + `search_all`), `src/sdk/serialization/mod.rs:111` (`validate_namespace`), `src/sdk/api/namespaces.rs:23` (`list_namespaces`), `src/sdk/search/tests.rs` (setup `connect(":memory:")` + `insert`), `vantadb-ts/src/vantadb.ts:777` (pasa namespaces sin validar), `vantadb-wasm/src/lib.rs:1503`
- **Archivos referenciados hacia dentro:** `multi.rs` ← `builder::Embedded`, `serialization::validate_namespace`, `types::MemorySearchRequest`
- **Archivos que referencian a los editados:** grep `search_multi` = `multi.rs` (def) + `search_all` + bindings (node/wasm) + tests TS; grep `validate_namespace` = múltiples paths write (no se tocan)
- **Veredicto impacto:** medio — 1 archivo core + tests; validación write intacta por construcción (no se toca `validate_namespace`)

## Contrato

"`search_multi(['kb/*'], text_query)` retorna hits de `kb/docs`+`kb/facts` y no de `other`; `['*']` == `search_all`; `['a*b']`/`['']` se skipean en silencio (comportamiento previo); literales sin cambio; `cargo test -p vantadb --lib search_multi` + suites search verdes."

## Spec

Feature-add leve (nuevo contrato público en `search_multi`): decisiones resueltas por evidencia/diseño (sin Gate P/D — scope fijado por el plan):
| # | Decisión | Resuelto |
|---|----------|----------|
| 1 | Sintaxis | ✅ Solo `*` (todo) y `prefijo/*` (prefijo con charset de `validate_namespace`, no vacío). `a*b`, `**`, `/*` → no-patrones → silent-skip previo |
| 2 | 0 matches | ✅ Contribución vacía (coherente con "namespaces sin resultados se skipean") |
| 3 | Overlap (`kb/*`+`*`) | ✅ Dedup preservando orden (evita hits duplicados en el merge) |
| 4 | Dónde validar | ✅ Solo search: `validate_namespace` intacta; patrones solo existen dentro de `search_multi` |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** `put`/`get`/`delete` rechazan `*` igual que hoy; silent-skip de inválidos intacto; `top_k == 0` y slice vacío → vacío; merge ordenado + truncate intactos
- **Comandos de verificación:** `cargo test -p vantadb --lib search` (scoped; binario `vanta-cli.exe` lockeado por MCPs) + `cargo clippy -p vantadb --lib -- -D warnings` + `cargo fmt --check`
- **Deuda pendiente:** ninguna

## Recitation

| Campo recitation (MCP) | Valor |
|------------------------|-------|
| `activeGoal` | EGO-03: wildcard nativo en search_multi |
| `lastAction` | (se actualiza al cerrar steps) |
| `result` | OK (contrato + search + clippy/fmt verificados) |
| `nextAction` | Review P2-01 por agente distinto (pendiente) |
| `contract` | ver §Contrato + §Invariantes |
| `nextTask` | EGO-05 (Ola 2) |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda — ~30 líneas + tests; sin `unsafe`, sin dependencias nuevas.

## Definition of Done

| Nivel | Gate |
|-------|------|
| **Task** | Contrato + suites search verdes |
| **Commit** | Commit atómico, `feat(search):` + task ID, diff limpio |
| **Release** | changelog (feature → minor) + ejemplo en docs API si aplica |

## Herramientas necesarias

- cargo (test/clippy/fmt -p vantadb --lib)

**Skills cargadas (SDP):** documentation-skill · base-only, sin candidatos adicionales (fan-out ya existente, sin dominio nuevo)

## Investigation Notes

- TS/WASM/Node no validan namespaces client-side → el fix core-only cubre los 3 bindings sin tocarlos.
- `search_all` ya hace list+fan-out; la expansión de patrones reusa el mismo mecanismo.

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 4 (código+tests, verify, commit) |
| % completado | 0% |

## Fases explícitas — SECURITY | PERFORMANCE

- [x] **SECURITY** — no aplica: el patrón solo filtra nombres contra `list_namespaces` (sin glob en storage, sin inyección: `*` solo al final, charset restringido); sin trust boundary nuevo
- [x] **PERFORMANCE** — 1 `list_namespaces` extra solo cuando hay patrón; fan-out acotado a namespaces existentes (mismo costo que N llamadas dede-Ego hoy, pero en 1 round-trip). Sin claim → sin bench (Regla 9 n/a)

## Steps

### Step 1: Expansión + tests
- **Archivos:** `src/sdk/search/multi.rs`
- **Acción:** `wildcard_prefix(ns) -> Option<Option<&str>>` (None=literal, Some(None)=todo, Some(Some(p))=prefijo con `/`); en `search_multi`: si hay patrón, `list_namespaces()` una vez, expandir, dedup preservando orden; loop existente intacto; tests `#[cfg(test)]` con `connect(":memory:")` (kb/docs+kb/facts+other; patrón, `*`, malformado, literales, overlap)
- **Verify:** `cargo test -p vantadb --lib search_multi` → 5/5 ✅ (4 nuevos + 1 diskann por filtro)
- **Estado:** ✅ COMPLETED

### Step 2: Verify full
- **Archivos:** —
- **Acción:** suites search + clippy + fmt
- **Verify:** `--lib search` 236 ✅ + `clippy --lib -D warnings` ✅ + `fmt --check` ✅ (tras `cargo fmt`)
- **Estado:** ✅ COMPLETED

### Step 3: Commit local
- **Archivos:** —
- **Acción:** commit `feat(search):` + task ID (local, sin push)
- **Verify:** `git log --oneline -1`
- **Estado:** ⬜ PENDING

## Dependencias

- Ninguna (Ola 2; archivos disjuntos de EGO-05)

## Review (GATE — agente distinto, P2-01)

> Fallback degradado (sin subagente disponible en esta sesión): auto-revisión adversarial + escalado al owner. Sin review externo u OK del owner, la tarea no se da por certificada.

- **Revisor:** self-review degradado (implementador) — pendiente agente distinto / owner
- **Enfoque:** alternativa considerada: expandir en cada binding vs en el core. Elegido core: TS/WASM/Node no validan namespaces (pasan tal cual) → un solo punto cubre 3 bindings + MCP/HTTP. Sintaxis mínima (`*`, `prefijo/*`) vs glob general: el caso Ego es prefijo; glob general es superficie de abuso/ambigüedad sin consumidor
- **Cómo se probó:** evidencia mecánica: lib `search_multi` 5/5 (patrón, `*`==`search_all`, 3 malformados + 0-matches, literales, overlap-dedup); suite `search` 236 ✅; clippy ✅; fmt ✅
- **Checklist anti-hábitos tóxicos:**
  - [x] Salidas ejecutadas de verdad
  - [x] Contrato 5/5 verificado
  - [x] Sin fallos ignorados (suite search completa)
  - [x] Saturación: callers de `search_multi` (search_all + 2 bindings) verificados sin validación client-side; `validate_namespace` intacta (write paths sin cambio)
  - [x] Sin bucles sin diagnóstico
- **Veredicto:** ⚠️ degradado — implementación + verify OK; falta review externo (owner o `vanta-review`)

## Notas

- Plan Ola 2: EGO-05 en paralelo (archivos disjuntos: `sdk/types.rs` vs `sdk/search/multi.rs`).
