---
title: "FIND-522: Endurecer parser, lexer y errores de IQL ante `NaN`/`Inf`, recursión profunda y rangos inválidos"
kind: task
description: "cargo nextest run --profile audit -p vantadb -E \"test(parser) or test(lexer) or test(iql) or test(scan)\" --locked --build-jobs 2 pasa con los 6 sub-hallazgos (FIND-522, FIND-523, FIND-524, FIND-525, FIND-526, FIND-528) cubiertos por..."
---

# FIND-522: Endurecer parser, lexer y errores de IQL ante `NaN`/`Inf`, recursión profunda y rangos inválidos (absorbe FIND-523, FIND-524, FIND-525, FIND-526, FIND-528)

## Metadata
- **Plan file:** `docs/dev/plans/2026-11-07-consolidacion-tecnica-pre-v0100.md`
- **Fuente:** `docs/dev/Backlog.md` (Phase 60 IQL L565–571) / Plan Task 25
- **Esfuerzo:** 🟡 5h
- **Prioridad:** 🔴
- **Tipo:** Rust
- **Turns estimados:** 25
- **Creado:** 2026-11-18T12:00
- **last-synced:** 2026-11-18
- **Estado:** ✅ COMPLETED
- **Commits:** `35ced06fe`, `6ca7358da`, `7bdcebaa1`
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps de ejecución restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vantadb::executor`, `vantadb::planner`, `vantadb::sdk::api::query`, `vantadb-mcp` (`query_iql`), `src/server/` |
| Callees | `vantadb::parser::lexer`, `vantadb::parser::grammar`, `vantadb::physical_plan::scan` |
| Implicaciones | Rechazo fail-closed de entradas IQL malformadas o maliciosas (`NaN`/`Inf`, subqueries `> 8` niveles, `NODE#<no-numérico>`, `SIGUE` inválido, keywords en minúscula como alias) y truncado seguro UTF-8 a 120 chars en mensajes de error |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/parser/lexer.rs`, `src/parser/grammar.rs`, `src/physical_plan/scan.rs`, `src/executor.rs`
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `nom`, `crate::error::Error`
- **Archivos que referencian a los editados (referencias entrantes):** `src/planner.rs`, `src/executor.rs`, `src/sdk/api/query.rs`, `tests/`
- **Veredicto impacto:** medio — endurece la validación del front-end IQL sin romper ninguna consulta válida existente.

## Contrato
`cargo nextest run --profile audit -p vantadb -E "test(parser) or test(lexer) or test(iql) or test(scan)" --locked --build-jobs 2` pasa con los 6 sub-hallazgos (`FIND-522`, `FIND-523`, `FIND-524`, `FIND-525`, `FIND-526`, `FIND-528`) cubiertos por tests dedicados.

## Spec (SDD)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Límite de profundidad de subqueries (`FIND-523`) | A: `MAX_SUBQUERY_DEPTH = 8` pasado por valor en recursión descendente / B: Contador thread-local mutable | A | ✅ decidido-por-evidencia (ref: `src/parser/grammar.rs:39,292-313,683,748`) |
| 2 | Validación de floats y `min` score (`FIND-522`) | A: `parse_finite_f32`/`parse_finite_f64` rechazando `NaN`/`Inf` y exigiendo `min ∈ [0.0, 1.0]` / B: Clamping silencioso | A | ✅ decidido-por-evidencia (ref: `src/parser/grammar.rs:49-54,96-112`) |
| 3 | Truncado de payload en error de parseo (`FIND-528`) | A: Truncado UTF-8 seguro a 120 caracteres (`char_indices`) en `executor.rs:124-182` / B: Emitir query completa | A | ✅ decidido-por-evidencia (ref: `src/executor.rs:124-182`) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) Ningún literal float o umbral `min` en IQL acepta `NaN`/`Inf`, y `min` vectorial debe pertenecer a `[0.0, 1.0]`; (2) La profundidad de subqueries no supera `MAX_SUBQUERY_DEPTH = 8`; (3) `FROM NODE#<no-numérico>` falla con `Error::Validation` en `PhysicalSeqScan::open`; (4) `SIGUE min..max` falla ante `min > max` o overflow `u32`; (5) `RESERVED_KEYWORDS` se comparan con `eq_ignore_ascii_case` sin alocar y nunca se aceptan como alias/identificadores; (6) `iql_parse_error_message` trunca la entrada a 120 caracteres respetando fronteras UTF-8.
- **Comandos de verificación:** `cargo nextest run --profile audit -p vantadb -E "test(parser) or test(lexer) or test(iql) or test(scan)" --locked --build-jobs 2`
- **Deuda pendiente:** ninguna

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | `# FIND-522: Endurecer parser, lexer y errores de IQL ante NaN/Inf, recursión profunda y rangos inválidos` |
| `lastAction` | Step 4 ✅ — Verificación de los 6 sub-hallazgos en `HEAD` por `vanta-review` (`396f4246`) y `audit-w2-w3-mem-iql` (#22) |
| `result` | `OK` ↔ ✅ COMPLETED |
| `nextAction` | Ninguna (tarea completada) |
| `contract` | `cargo nextest run --profile audit -p vantadb -E "test(parser) or test(lexer) or test(iql) or test(scan)" --locked --build-jobs 2` → 202/202 parser lib PASS |
| `nextTask` | `FIND-556` |

```yaml
contract:
  verificacion: "cargo nextest run --profile audit -p vantadb -E \"test(parser) or test(lexer) or test(iql) or test(scan)\" --locked --build-jobs 2 -> 202/202 PASS"
  evidencia:
    - claim: "Los 6 sub-hallazgos (FIND-522/523/524/525/526/528) están implementados y verificados en HEAD"
      evidencia: "src/parser/lexer.rs:46-102,186-235; src/parser/grammar.rs:39,49-54,96-112,292-313,683,748; src/executor.rs:124-182; src/physical_plan/scan.rs:47-64"
      confianza: alta
  artefactos:
    - "src/parser/lexer.rs"
    - "src/parser/grammar.rs"
    - "src/physical_plan/scan.rs"
    - "src/executor.rs"
  invariantes: "Front-end IQL fail-closed ante NaN/Inf, subqueries > 8, NODE# no numérico, SIGUE inválido, keywords case-insensitive y errores truncados a 120 chars UTF-8"
  deuda: "ninguna"
  queda_pendiente: "ninguno"
```

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda (elimina 6 vectores de error silencioso/DoS en el parser y ejecutor IQL).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato mecánico cumplido (202/202 parser tests) + `cargo fmt --check` + `cargo clippy -p vantadb -- -D warnings` |
| **Commit** | Commits `35ced06fe`, `6ca7358da` y `7bdcebaa1`, conventional commits |
| **Release** | Incluido en tren `v0.10.0` |

## Herramientas necesarias
- cargo-mcp (check, clippy, fmt, nextest)
- codegraph_codegraph_explore (blast radius)

**Skills cargadas (SDP):** `security-and-hardening`, `systematic-debugging`, `test-driven-development`

## Investigation Notes
- La validación de `FROM NODE#<no-numérico>` (`src/physical_plan/scan.rs:47-64`) aterrizó junto con el lote de Physical Plan en `7bdcebaa1`, completando los 6 sub-hallazgos del contrato en `HEAD`.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Fase 1 — Evidencia de Debugging (GATE — solo tipo Bug)

- **Repro:** (1) `min = nan` / `min = inf` eran aceptados produciendo 0 hits silenciosos; (2) subqueries anidadas sin límite agotaban la pila; (3) `FROM NODE#abc` caía al fallback de escaneo devolviendo el primer nodo arbitrario; (4) `SIGUE 5..2` o `SIGUE 1..5000000000` descartaba la cláusula en silencio; (5) keywords en minúscula (`where`) se consumían como alias; (6) errores de parseo reflejaban el input completo sin truncar.
- **Hipótesis:** Uso permisivo de `opt(...)` y `float` de `nom` sin guardas de finitud/rango/profundidad, más comparación case-sensitive con alocación en `lexer.rs` y formateo sin truncar en `executor.rs`.
- **1 variable controlada:** Endurecer cada regla de gramática/lexer/scan/error con validación fail-closed (`nom::Err::Failure` / `Error::Validation`) y tope explícito.
- **Test RED:** 6 baterías de tests unitarios para `FIND-522`, `FIND-523`, `FIND-524`, `FIND-525`, `FIND-526` y `FIND-528` que fallaban antes del fix y pasan en `HEAD` (202/202).

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — Cierra DoS por stack overflow en subqueries anidadas (`MAX_SUBQUERY_DEPTH = 8`) y amplificación/fuga de payload en logs de error (`120` chars UTF-8 safe).
- [x] **PERFORMANCE** — `lexer.rs` reemplaza alocaciones por token (`to_uppercase()`) con comparaciones `eq_ignore_ascii_case` zero-allocation.

## Steps

### Step 1: Endurecer `lexer.rs` (`FIND-526`) y `grammar.rs` (`FIND-522`, `FIND-523`, `FIND-525`)
- **Archivos:** `src/parser/lexer.rs`, `src/parser/grammar.rs`
- **Acción:** Validar floats finitos y `min ∈ [0, 1]`, propagar `depth` con tope `MAX_SUBQUERY_DEPTH = 8`, fallar cerrado en `SIGUE` con `min > max` u overflow, y usar `eq_ignore_ascii_case` + `RESERVED_KEYWORDS`.
- **Verify:** `cargo nextest run --profile audit -p vantadb -E "test(parser) or test(lexer)" --locked --build-jobs 2`
- **Estado:** ✅ COMPLETED (`35ced06fe`, `6ca7358da`)

### Step 2: Truncar payload en `iql_parse_error_message` (`FIND-528`)
- **Archivos:** `src/executor.rs`
- **Acción:** Truncar el input reflejado en errores de parseo IQL a 120 caracteres respetando fronteras UTF-8.
- **Verify:** `cargo nextest run --profile audit -p vantadb -E "test(executor) or test(iql)" --locked --build-jobs 2`
- **Estado:** ✅ COMPLETED (`35ced06fe`)

### Step 3: Rechazar `FROM NODE#<no-numérico>` en `PhysicalSeqScan` (`FIND-524`)
- **Archivos:** `src/physical_plan/scan.rs`
- **Acción:** Devolver `Error::Validation` cuando el sufijo de `NODE#` no sea un entero `u128` válido.
- **Verify:** `cargo nextest run --profile audit -p vantadb -E "test(scan)" --locked --build-jobs 2`
- **Estado:** ✅ COMPLETED (`7bdcebaa1`)

## Dependencias
- Wave 1 completa.

## Review (GATE — agente distinto, P2-01)

- **Revisor:** `vanta-review` (`396f4246`) + re-verificación independiente en auditoría `audit-w2-w3-mem-iql` (#22)
- **Enfoque:** Correcto. Los 6 sub-hallazgos se resolvieron de raíz sin degradar el rendimiento del lexer (eliminando alocaciones con `eq_ignore_ascii_case`) y con truncado UTF-8 seguro.
- **Cómo se probó:** 202/202 tests de librería del parser en `vantadb` + inspección estática de los 4 archivos en #22 + fix de formato `grammar.rs:744` verificado en #23/#25.
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
- Absorbe `FIND-523`, `FIND-524`, `FIND-525`, `FIND-526` y `FIND-528`.
- Commits en `develop`: `35ced06fe`, `6ca7358da` y `7bdcebaa1` (donde aterrizó `src/physical_plan/scan.rs:47-64`).
