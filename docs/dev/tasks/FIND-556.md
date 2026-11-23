# FIND-556: Corregir filtros con alias, coerción numérica `Int`/`Float` y subqueries escalares en Physical Plan (absorbe FIND-157, FIND-557, FIND-558)

## Metadata
- **Plan file:** `docs/dev/plans/2026-11-07-consolidacion-tecnica-pre-v0100.md`
- **Fuente:** `docs/dev/Backlog.md` (Hallazgos L190 `FIND-157` + Phase 60 Physical Plan L603–605 `FIND-556..558`) / Plan Task 26
- **Esfuerzo:** 🟡 4h
- **Prioridad:** 🔴
- **Tipo:** Rust
- **Turns estimados:** 20
- **Creado:** 2026-11-18T12:00
- **last-synced:** 2026-11-18
- **Estado:** ✅ COMPLETED
- **Commits:** `7bdcebaa1`, `539fc7887`
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps de ejecución restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vantadb::planner`, `vantadb::executor` |
| Callees | `vantadb::physical_plan` (`mod.rs`, `filter.rs`, `join.rs`, `project.rs`) |
| Implicaciones | Resolución consistente exact-then-stripped para referencias de campo con alias, comparación numérica exacta `(i64, f64)` en filtros y joins, y semántica relacional correcta en subqueries escalares (vacía → `None`, columna = primer campo proyectado) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/physical_plan/mod.rs`, `src/physical_plan/filter.rs`, `src/physical_plan/join.rs`, `src/physical_plan/project.rs`, `src/planner.rs`
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `crate::node::FieldValue`, `std::cmp::Ordering`
- **Archivos que referencian a los editados (referencias entrantes):** `src/planner.rs`, `src/executor.rs`, `tests/`
- **Veredicto impacto:** medio — corrige 4 defectos de evaluación relacional en `physical_plan` y `planner` manteniendo compatibilidad hacia atrás con claves que contienen `.` literal (lookup exacto primero, `strip_alias` como fallback).

## Contrato
`cargo nextest run --profile audit -p vantadb -E "test(filter) or test(join) or test(subquery) or test(iql)" --locked --build-jobs 2` pasa con tests unitarios para alias en `WHERE`, coerción exacta `Int`/`Float`, subquery escalar vacía y selección del primer campo proyectado.

## Spec (SDD)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Resolución de campos con `.` (`FIND-556`) | A: Lookup exacto primero en `node.relational` y fallback a `strip_alias(field)` / B: `strip_alias` incondicional (rompe claves con `.` literal) | A | ✅ decidido-por-evidencia (ref: `src/physical_plan/filter.rs:69-123`, `project.rs:62-77`, `join.rs:306-318`, commit `539fc7887`) |
| 2 | Comparación mixta `Int(i64)` vs `Float(f64)` (`FIND-157`) | A: Helper exacto `cmp_int_float(i64, f64) -> Option<Ordering>` sin pérdida de precisión en `|n| > 2^53` / B: Cast `i as f64` (pierde precisión en enteros grandes) | A | ✅ decidido-por-evidencia (ref: `src/physical_plan/mod.rs:53-107`) |
| 3 | Subquery escalar vacía y columna extraída (`FIND-557`, `FIND-558`) | A: Conjunto vacío → `Ok(None)` (toda comparación evalúa a `false`); extraer `scalar_field` del primer campo proyectado por el `SELECT` / B: `values().next()` del `BTreeMap` | A | ✅ decidido-por-evidencia (ref: `src/physical_plan/join.rs:193-254`, `src/planner.rs:438-446`) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) Todo lookup de campo en `filter`, `project` y `subquery` busca primero la clave exacta y luego `strip_alias`; (2) `cmp_int_float` es exacto en todo el dominio `(i64, f64)` (`±2^63`, `>2^53`, fracciones negativas, `-0.0`, y non-finite → `None`/`false`); (3) Una subquery escalar con 0 filas produce `Ok(None)` y nunca deja pasar filas sin filtrar; (4) El valor escalar de una subquery extrae el primer campo proyectado (`scalar_field`), no la primera clave alfabética del `BTreeMap`.
- **Comandos de verificación:** `cargo nextest run --profile audit -p vantadb -E "test(filter) or test(join) or test(subquery) or test(iql)" --locked --build-jobs 2`
- **Deuda pendiente:** `FIND-657` (colisión de referencias cualificadas en el lado perdedor de un join, aislada y documentada en `src/physical_plan/join.rs:116-122`).

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | `# FIND-556: Corregir filtros con alias, coerción numérica Int/Float y subqueries escalares en Physical Plan` |
| `lastAction` | Step 4 ✅ — Fix r2 `539fc7887` (lookup exact-then-stripped en `scalar_field` + doc `FIND-657`) verificado por `audit-w2-w3-mem-iql` (#22) |
| `result` | `OK` ↔ ✅ COMPLETED |
| `nextAction` | Ninguna (tarea completada) |
| `contract` | `cargo nextest run --profile audit -p vantadb -E "test(filter) or test(join) or test(subquery) or test(iql)" --locked --build-jobs 2` → 19/19 joins PASS |
| `nextTask` | `FIND-563` |

```yaml
contract:
  verificacion: "cargo nextest run --profile audit -p vantadb -E \"test(filter) or test(join) or test(subquery) or test(iql)\" --locked --build-jobs 2 -> 19/19 joins PASS"
  evidencia:
    - claim: "Lookup exact-then-stripped, cmp_int_float exacto, subquery vacía -> Ok(None) y primer campo proyectado verificados en HEAD"
      evidencia: "src/physical_plan/mod.rs:53-107; src/physical_plan/filter.rs:69-123; src/physical_plan/join.rs:116-122,193-254,306-318; src/physical_plan/project.rs:62-77; src/planner.rs:438-446"
      confianza: alta
  artefactos:
    - "src/physical_plan/mod.rs"
    - "src/physical_plan/filter.rs"
    - "src/physical_plan/join.rs"
    - "src/physical_plan/project.rs"
    - "src/planner.rs"
  invariantes: "cmp_int_float sin pérdida de precisión en >2^53; lookup exact-then-stripped; subquery vacía -> None; scalar_field del primer campo proyectado"
  deuda: "FIND-657 (referencias cualificadas del lado perdedor del join, documentado en join.rs:116-122)"
  queda_pendiente: "ninguno"
```

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda (resuelve 4 bugs semánticos del Physical Plan y documenta explícitamente el límite residual en `FIND-657`).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato mecánico cumplido (19/19 joins + parser/iql) + `cargo fmt --check` + `cargo clippy -p vantadb -- -D warnings` |
| **Commit** | Commits atómicos `7bdcebaa1` y `539fc7887`, conventional commits |
| **Release** | Incluido en tren `v0.10.0` |

## Herramientas necesarias
- cargo-mcp (check, clippy, fmt, nextest)
- codegraph_codegraph_explore (blast radius en `physical_plan` y `planner`)

**Skills cargadas (SDP):** `systematic-debugging`, `test-driven-development`, `doubt-driven-development`

## Investigation Notes
- En la revisión P2-01 (`396f4246`), se detectó que `scalar_field` en `join.rs` aplicaba `strip_alias` antes del match exacto; en `539fc7887` se unificó al orden canónico `exact -> stripped` en `filter`, `project` y `subquery`, y se documentó `FIND-657` en `join.rs:116-122`.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Fase 1 — Evidencia de Debugging (GATE — solo tipo Bug)

- **Repro:** (1) `WHERE u.age = 28` fallaba porque `filter.rs` buscaba `"u.age"` literal sin fallback a `"age"`; (2) comparar `Int(28)` con `Float(28.0)` devolvía `false`; (3) una subquery escalar sin resultados delegaba en `child.next()` devolviendo todas las filas sin filtrar; (4) una subquery escalar con varias columnas tomaba `values().next()` (orden alfabético del `BTreeMap`) en vez de la primera columna proyectada.
- **Hipótesis:** Ausencia de normalización de alias en `filter.rs`, falta de rama mixta `(Int, Float)` / `(Float, Int)` en comparadores, y manejo defectuoso de `scalar_value: Option<FieldValue>` en `PhysicalSubqueryJoin`.
- **1 variable controlada:** Implementar `cmp_int_float` exacto en `physical_plan/mod.rs`, resolución `exact -> stripped` en `filter.rs`/`project.rs`/`join.rs`, y propagar `scalar_field` desde `planner.rs:438-446` con `Ok(None)` cuando la subquery está vacía.
- **Test RED:** 4 tests unitarios (filtro con alias, coerción `Int`/`Float` incluyendo bordes `> 2^53`, subquery vacía y orden de proyección en subquery escalar) fallaban antes del fix y pasan en `HEAD` (19/19 en `joins`).

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — Evita exposición de filas no autorizadas cuando una subquery escalar de filtrado devuelve 0 filas (`FIND-557`).
- [x] **PERFORMANCE** — `cmp_int_float` opera en aritmética entera/bit-level O(1) sin alocaciones; el fallback `strip_alias` es un slice `&str` zero-copy.

## Steps

### Step 1: Implementar `cmp_int_float` exacto en `src/physical_plan/mod.rs` (`FIND-157`)
- **Archivos:** `src/physical_plan/mod.rs`, `src/physical_plan/filter.rs`, `src/physical_plan/join.rs`
- **Acción:** Crear comparador exacto `(i64, f64)` que maneje `NaN`/`Inf`, bordes `±2^63`, enteros `> 2^53` y partes fraccionarias, e integrarlo en `filter.rs` y `join.rs`.
- **Verify:** `cargo nextest run --profile audit -p vantadb -E "test(filter) or test(join)" --locked --build-jobs 2`
- **Estado:** ✅ COMPLETED (`7bdcebaa1`)

### Step 2: Resolver alias con lookup `exact -> stripped` en `filter.rs`, `project.rs` y `join.rs` (`FIND-556`)
- **Archivos:** `src/physical_plan/filter.rs`, `src/physical_plan/project.rs`, `src/physical_plan/join.rs`
- **Acción:** Buscar primero la clave exacta en `node.relational` y usar `strip_alias` como fallback cuando no exista.
- **Verify:** `cargo nextest run --profile audit -p vantadb -E "test(filter) or test(join)" --locked --build-jobs 2`
- **Estado:** ✅ COMPLETED (`7bdcebaa1`, `539fc7887`)

### Step 3: Corregir subquery escalar vacía y selección del primer campo proyectado (`FIND-557`, `FIND-558`)
- **Archivos:** `src/physical_plan/join.rs`, `src/planner.rs`
- **Acción:** Devolver `Ok(None)` cuando la subquery escalar no produce filas y extraer el campo `scalar_field` correspondiente al primer elemento proyectado por el `SELECT`.
- **Verify:** `cargo nextest run --profile audit -p vantadb -E "test(join) or test(subquery)" --locked --build-jobs 2`
- **Estado:** ✅ COMPLETED (`7bdcebaa1`, `539fc7887`)

## Dependencias
- Wave 1 completa.

## Review (GATE — agente distinto, P2-01)

- **Revisor:** `vanta-review` (`396f4246` r1 + r2) + re-verificación independiente en auditoría `audit-w2-w3-mem-iql` (#22)
- **Enfoque:** Correcto. Tras ajustar en r2 (`539fc7887`) la precedencia `exact -> stripped` en `scalar_field` y aislar `FIND-657`, los 4 defectos quedaron cerrados con aritmética exacta en `cmp_int_float`.
- **Cómo se probó:** 19/19 tests de `joins` + 202/202 parser/iql + inspección matemática exhaustiva de `cmp_int_float` en todos los bordes (`±2^63`, `>2^53`, fracciones negativas, `-0.0`, non-finite) por `audit-w2-w3-mem-iql` (#22).
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
- Absorbe `FIND-157`, `FIND-557` y `FIND-558`.
- Commits en `develop`: `7bdcebaa1` y `539fc7887`.
