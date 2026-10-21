---
title: "FIND-518 - El WHERE entero de IQL se pierde si la primera condicion no parsea"
kind: task
status: review
description: "Corregir el parser: un WHERE mal formado debe ser un error de parseo, no un escaneo sin filtro. Incluye FIND-519 (el trailing input se descarta en silencio)."
tags: [parser, iql, critica, correctness]
---

# FIND-518: El WHERE entero se pierde si la primera condicion no parsea

El parser de IQL envuelve la clausula `WHERE` en `opt(...)`: en `nom`, un `opt` ante `Err::Error` **restaura el input** y devuelve `None`. Con `FROM Doc WHERE n =` (una condicion mal formada) el resultado es `FROM Doc` **sin filtro y sin error**: la query se ejecuta como un escaneo completo y `PhysicalScan::open` materializa todos los nodos en RAM antes de filtrar (viola `memory-budget.md` R-3/R-4). Su hermana `FIND-519` es el mismo patron: el *trailing input* de cualquier statement se descarta en silencio (HTTP, MCP y CLI), asi que `WHERE a = 1, b = 2` aplica un filtro y **pierde el otro**, `min >= 0.88` degrada el brazo vectorial a match lexico, `PROFILE bogus` corre sin perfil e `INSERT ... LINK ...` inserta el nodo sin arista.

**El fix estructural de ambas es el mismo:** que `reject_trailing_input` sea estricto para todas las statements (tolerando solo `;`) y que el `WHERE` propague el error cuando su keyword ya se consumio.

> **Estado:** ✅ IMPLEMENTADO Y VERIFICADO — falta la review P2-01 de un agente DISTINTO
> (no se marca COMPLETED sin ella).

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-21-fix-critical-backlog.md`
- **Fuente:** `docs/dev/Backlog.md` (FIND-518 (critica) + FIND-519 (alta, su hermana))
- **Evidencia de la auditoria:** `docs/dev/reviews/audit-grammar-iql-20261021.md` (F-01 = FIND-518, F-02 = FIND-519, F-14 = cobertura de tests)
- **Esfuerzo:** 🟠 1-2sem
- **Prioridad:** 🔴 Alta
- **Tipo:** Rust
- **Turns estimados:** 6-8
- **Creado:** 2026-10-21T12:00
- **last-synced:** 2026-10-21T18:40
- **Estado:** 🟡 EN REVISION (implementado y verificado; falta la review P2-01 de otro agente)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps de ejecución (4/4 hechos) — queda el gate P2-01

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | src/executor.rs, src/server/handlers.rs, vantadb-mcp/src/handlers/tools.rs, src/bin/vanta-cli.rs |
| Callees | src/parser/grammar.rs, src/parser/lexer.rs |
| Implicaciones | el contrato de error cambia: un query mal formado pasa a fallar en vez de ejecutarse sin filtro. Es el comportamiento que la doc ya promete. |

## Impacto mapeado (Regla 0)

> **GATE ANTES DE CUALQUIER EDICIÓN (MUST — AGENTS.md Regla 0):** todo archivo
> que se vaya a modificar/eliminar se lee completo y se mapea su impacto ANTES
> del primer step de edición. Sin este bloque poblado, NO se escribe ni se
> ejecuta ningún step que edite archivos.

- **Archivos leídos (completos):** src/parser/grammar.rs (832 lineas), src/parser/lexer.rs (180), src/parser/mod.rs (1914), src/executor.rs (la parte de parseo: 80-280 y los tests del modulo)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** src/parser/mod.rs -> grammar.rs + lexer.rs; grammar.rs -> lexer.rs (tokens); executor.rs -> parser::parse_statement
- **Archivos que referencian a los editados (referencias entrantes):** src/executor.rs (parse_statement), src/server/handlers.rs, vantadb-mcp/src/handlers/tools.rs (query_iql), src/sdk (IQL), src/cli_handlers/data.rs (`query_is_mutating`), fuzz/fuzz_targets/fuzz_parser.rs, tests/api/openapi_yaml_parity.rs
- **Veredicto impacto:** alto: cambia el contrato observable del parser (un trailing pasa a ser error). Los tres llamadores (HTTP/MCP/CLI) delegan en `Executor::execute_hybrid` -> `parse_statement`, asi que el cambio es **estructural y central**: no hay ningun caller que dependa del descarte silencioso (todos le pasan la query tal cual). El unico consumidor que mira el `rest` es `query_is_mutating` (que solo clasifica el tipo de statement) y `tests/api/openapi_yaml_parity.rs` (que ya exigia consumo total).

## Contrato

"`FROM Doc WHERE n =` devuelve **error de parseo** (no un escaneo sin filtro) y `FROM Doc WHERE a = 1, b = 2` tambien; `FROM Doc` (sin WHERE) sigue funcionando; `FROM Doc;` (con `;`) sigue funcionando; el resto del trailing input (`FROM Doc garbage`, `PROFILE bogus`, `INSERT ... LINK ...`) devuelve error. Tests: hay un test que fija cada caso."

### Matiz del contrato (verificado contra la gramática) — leer antes de revisar

- **`FROM Doc garbage` NO es trailing input**: `garbage` es un `<alias>` valido de la
  superficie `FROM`/`MATCH` (`docs/api/IQL.md`: `| <alias> | Target alias for result
  nodes (defaults to "target"). |`). El parser lo consume como alias, la statement es
  valida y el resultado es el mismo que `FROM Doc`. El trailing junk real es lo que va
  detras de un alias explicito: **`FROM Doc p garbage`**. El ejemplo literal del
  contrato/backlog es impreciso en este punto (hereda la lectura del auditor, que
  tomo el test `assert!(parse_statement("FROM Doc garbage").is_ok())` por un
  silent-drop sin mirar que `garbage` es el alias). **Rechazar `FROM Doc garbage`
  exigiriaabolir el `<alias>` documentado** ⇒ violaria el invariante "un statement
  valido no puede pasar a fallar". Por eso el test fija las dos mitades:
  `FROM Doc p garbage` -> error (trailing junk) y `FROM Doc garbage` -> statement
  valida con alias `garbage`.
- Los casos de trailing input que la auditoria lista de verdad (F-02) estan TODOS
  cubiertos por tests: `WHERE a = 1, b = 2`, `WHERE bio ~ "x", min >= 0.88`,
  `WHERE bio ~ "x", minimum = 0.5`, `PROFILE bogus`, `PROFILE KEYWORD`,
  `INSERT ... LINK ...`, mas el subcase de FIND-312 (trailing INTERNO en subquery).

## Spec (SDD)

N/A — no es feature-add ni symbolos publicos nuevos: es la correccion de un
comportamiento existente que ya tiene contrato documentado (el del backlog).

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** 1) un statement **valido** no puede pasar a fallar (la suite de 2493 tests del lib debe seguir verde); 2) el `;` final (REPL/CLI) se sigue tolerando; 3) los limites de longitud de query no cambian; 4) nada de panics: todo fallo de parseo es un `Error` con mensaje util.
- **Comandos de verificación:** `cargo nextest run -p vantadb --profile audit --lib parser`
- **Deuda pendiente:** ninguna

### Verificacion de los invariantes (2026-10-21)

| # | Invariante | Estado | Como se verifica |
|---|-----------|--------|-------------------|
| 1 | Un statement valido no pasa a fallar | ✅ | `cargo nextest run -p vantadb --profile audit --lib` -> **2517 tests, 2517 pass, 0 fail** (2 skipped); + test de control `test_parse_statement_valid_statements_still_parse` (14 statements) y `test_parse_statement_keeps_the_documented_alias` |
| 2 | El `;` final se sigue tolerando | ✅ | `test_parse_statement_tolerates_trailing_semicolon` (5 spellings: `FROM Doc;`, `... WHERE n = 1;`, `SELECT * FROM Doc;`, `DELETE NODE#1;`, `  FROM Doc ;  `) + el `;` ya tolerado por el filtro de pagination/aggregacion |
| 3 | Los limites de longitud de query no cambian | ✅ | No se toco ningun limite: `max_query_length` (MCP), `query_string.len()` y el resto de guardas viven fuera de `src/parser/**`; el cambio es de *aceptacion de input*, no de longitud |
| 4 | Nada de panics; todo fallo es un `Error` con mensaje util | ✅ | `Err::Failure` (nunca `unwrap`/`expect` en el parser); dos mensajes estables nuevos: `"malformed WHERE clause: expected at least one condition (\`WHERE <cond> [AND <cond>...]\`)"` y `"unexpected trailing input after the statement: \`<resto>\`"` |

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | Encabezado `# FIND-518: El WHERE entero se pierde si la primera condicion no parsea` |
| `lastAction` | (step 4) Verificacion completa: fmt + clippy + suite del lib (2517/2517) + integracion (17 + 81) + MCP (115) |
| `result` | ✅ IMPLEMENTADO Y VERIFICADO — pendiente la review P2-01 |
| `nextAction` | Review P2-01 por un agente DISTINTO (ver `## Review`) |
| `contract` | `## Contrato` + `## Invariantes de dominio` |
| `nextTask` | Task 2 del plan (FIND-552, ya en curso) |

## Deuda técnica (Regla 6 — MUST)

Sin deuda. Todo lo que se hizo cabe en 3 ficheros (`src/parser/grammar.rs`,
`src/parser/mod.rs`, `src/executor.rs`).

## Que se cambio (diff, 3 ficheros)

### `src/parser/grammar.rs`

1. **Nuevo `parse_where_clause` (generico sobre el parser de item)** — sustituye al
   `opt(tuple((ws(tag("WHERE")), separated_list1(...))))` de `parse_query` y de
   `parse_select`. Reglas:
   - el keyword `WHERE` se consume **una vez** y a partir de ahi la clausula es
     obligatoria: si la primera condicion no parsea -> `Err::Failure(Verify)` en la
     posicion del keyword (nunca `None` ⇒ nunca filtro descartado);
   - `separated_list1` se traga el error de la condicion que va **despues** de un
     `AND` (deja `AND <condicion-rota>` en el resto) ⇒ se detecta explicitamente y
     se convierte tambien en `Failure` (cierra la asimetria "dos gramaticas de fallo
     para el mismo error de tipeo" que cita la auditoria);
   - `Err::Failure` de un subquery (p.ej. `AS OF` dentro de un subquery) se propaga
     tal cual.
2. **`reject_trailing_input` estricto para TODAS las statements** (antes solo para
   `LIMIT`/`OFFSET` y agregados). Tolera **solo** `;` (REPL/CLI). El fallo se emite
   con `ErrorKind::Eof` (el kind de nom para "queda input") y el `input` del error es
   el resto sin espacios ⇒ la posicion (linea/columna) apunta al texto sobrante.
3. **`parse_statement`**: el check de `GROUP BY` en la superficie `FROM`/`MATCH` se
   mueve **antes** del rechazo generico de trailing input, para que su mensaje
   especifico ("aggregation ... is only supported in SELECT queries") no se degrade a
   "unexpected trailing input" (test existente en `src/executor.rs` lo fija).

### `src/executor.rs`

4. **`iql_parse_error_message`**: dos ramas nuevas.
   - `ErrorKind::Eof` -> `"unexpected trailing input after the statement: `<resto>`"`
     (resto truncado a 80 chars).
   - `ErrorKind::Verify` cuyo input empieza por `WHERE` -> `"malformed WHERE clause:
     expected at least one condition (\`WHERE <cond> [AND <cond>...]\`)"`.
   Las ramas existentes (`AS OF` duplicado, `LIMIT`/`OFFSET` malformados, y las de
   agregacion con `ErrorKind::Fail`) no se tocan ⇒ sus mensajes y sus posiciones
   siguen identicos (tests existentes en verde).

### Contrato observable (nuevo)

| Query | Antes | Ahora |
|-------|-------|-------|
| `FROM Doc WHERE n =` | `Ok(Read(todos los nodos))` — escaneo sin filtro | `Error::IqlParse` ("malformed WHERE clause: ..."), linea 1 col 9 |
| `FROM Doc WHERE a = 1, b = 2` | `Ok(Read(...))` con solo `a = 1` | `Error::IqlParse` ("unexpected trailing input after the statement: `, b = 2`") |
| `FROM Doc WHERE bio ~ "x", min >= 0.88` | degrada a match lexico, sin error | `Error::IqlParse` |
| `FROM Doc PROFILE bogus` | corre sin perfil, sin error | `Error::IqlParse` |
| `INSERT NODE#1 TYPE X { a: 1 } LINK NODE#2` | inserta el nodo sin arista | `Error::IqlParse` (no inserta nada) |
| `FROM Doc p garbage` | descarta el `garbage` | `Error::IqlParse` |
| `FROM Doc;` / `FROM Doc` / `FROM Doc garbage` (alias) | ok | ok (sin cambios) |

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable cumplido + `cargo fmt --check` + `cargo clippy -p vantadb --all-targets -- -D warnings` + los tests del cambio en verde |
| **Commit** | Commit atomico, conventional commit, `git diff` limpio, verificacion mecanica (nunca auto-reporte) — **lo commitea el orquestador, no este agente** |
| **Release** | No aplica (sin release en esta iteracion) |

## Verificacion ejecutada (2026-10-21)

```
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"

cargo fmt --check                                   # limpio en los 3 ficheros tocados
                                                     # (el unico diff restante es
                                                     #  tests/encryption_chaos.rs, de otro agente)
cargo clippy -p vantadb --all-targets -j 2 -- -D warnings      # exit 0
cargo nextest run -p vantadb --profile audit --lib -j 2         # 2517 tests, 2517 pass, 0 fail, 2 skipped
cargo nextest run -p vantadb --profile audit -j 2 \
  --test parser --test executor --test openapi_yaml_parity --test public_api --test edge_cases
                                                             # 17 tests, 17 pass
cargo nextest run -p vantadb --profile audit -j 2 \
  --test time_travel --test schema_v2_migration --test graph --test integration --test joins \
  --test query_result_basic --test query_result_advanced --test structured_api_v2 \
  --test message_thread_test --test graphrag_test --test sdk_serialization
                                                             # 81 tests, 81 pass
cargo test -p vantadb-mcp --test mcp_tests -j 2                 # 115 tests, 115 pass (superficie MCP)
```

> Nota operativa: la maquina tiene el **archivo de paginacion en 8 MB** y 32 GB de
> RAM ⇒ con 3+ agentes compilando a la vez aparecen `E0463` / `LNK1104`
> (documentados en el plan). Se reintenta con backoff hasta que el build pasa; no se
> fuerza nada.

## Tests nuevos (RED primero)

Todos escritos **antes** del fix y verificados **fallando** con el codigo viejo
(`cargo nextest run -p vantadb --profile audit --lib -E 'test(~parser::)'` -> 4 FAIL
y el executor -> `unwrap_err()` sobre `Ok(Read([UnifiedNode { id: 1, .. }]))`, la
prueba del bug).

### `src/parser/mod.rs` (modulo `tests`)

| Test | Que fija |
|------|----------|
| `test_parse_query_rejects_malformed_where_clause` | `WHERE` mal formado en la superficie `FROM`/`MATCH` -> error (5 casos: `n =`, `WHERE` solo, `n = AND m = 2`, `a = 1 AND b =`, `MATCH`) |
| `test_parse_select_rejects_malformed_where_clause` | lo mismo en la superficie `SELECT` (3 casos) |
| `test_parse_statement_rejects_malformed_where_clause` | a nivel de statement: `FROM Doc WHERE n =` y `FROM Doc WHERE a = 1, b = 2` |
| `test_parse_statement_rejects_trailing_input_on_every_statement` | los 9 casos reales de trailing input de la auditoria F-02 + `PROFILE KEYWORD` |
| `test_parse_statement_keeps_the_documented_alias` | control: `FROM Doc garbage` sigue siendo una statement valida con alias `garbage` |
| `test_parse_statement_tolerates_trailing_semicolon` | el `;` final se sigue tolerando (5 spellings) |
| `test_parse_statement_valid_statements_still_parse` | control: 14 statements validas siguen parseando |
| `test_parse_statement_rejects_trailing_input_inside_a_subquery` | subcase de FIND-312: trailing INTERNO en subquery -> error; y el subquery bien formado sigue parseando |

### `src/executor.rs` (modulo `tests`)

| Test | Que fija |
|------|----------|
| `malformed_where_is_a_parse_error_not_an_unfiltered_scan` | `FROM Doc WHERE n =` -> `Error::IqlParse` con mensaje que menciona `WHERE` y posicion (1, 9) — **no** un `Read` con todos los nodos |
| `trailing_input_is_a_parse_error` | `FROM Doc p garbage` -> `Error::IqlParse` con mensaje que menciona "trailing" y posicion (1, 11) |

### Test existente actualizado (no nuevo)

| Test | Cambio |
|------|--------|
| `test_parse_statement_rejects_trailing_clause_after_pagination` | el ultimo assert era `assert!(parse_statement("FROM Doc garbage").is_ok())` (fijaba el silent-drop como esperado — FIND-529) → ahora `is_err()` con `FROM Doc p garbage` + `FROM Doc WHERE a = 1, b = 2` |

## Herramientas necesarias
- cargo (check, clippy, fmt) — con `$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"`
- codegraph_codegraph_explore (blast radius)

## Skills cargadas (SDP):** test-driven-development (el fix necesita un test que fije el
comportamiento roto antes del fix) + systematic-debugging (causa raiz, no sintoma) +
context-engineering (jerarquia Rules -> Spec -> Source -> Error) +
incremental-implementation (slices finos: tests -> fix -> verify) +
source-driven-development (nom 7.1.3: `separated_list1` se traga el error del
elemento posterior al separador; `ErrorKind::Eof` es el kind de "queda input").

## Investigation Notes
- El hallazgo viene de la auditoria profunda del 2026-10-20/21 y su evidencia (archivo:línea)
  esta en la fila del backlog. Verificar la evidencia ANTES de tocar nada: si el codigo ya
  esta corregido, documentarlo y cerrar la fila (sin editar).
- **Evidencia verificada (no estaba ya corregido)**: `src/parser/grammar.rs:207-210`
  (`opt(tuple((ws(tag("WHERE")), separated_list1(...))))`) y `:737-753`
  (`reject_trailing_input` con la rama `strict`) se comportaban exactamente como
  describe la fila. Traza confirmada en nom 7.1.3 (`src/multi/mod.rs`:
  `separated_list1` devuelve `Err(Err::Error)` si falla el primer elemento pero `Ok`
  con lista parcial si falla uno posterior; `opt` ante `Err::Error` restaura el input).
- **Causa raiz**: dos, no tres. (a) el `opt` del `WHERE` restaura el input y devuelve
  `None` ⇒ el filtro desaparece sin error; (b) `reject_trailing_input` solo era
  estricto para `LIMIT`/`OFFSET` y agregados ⇒ el resto de statements perdiaban el
  input en silencio. El fix estructural es el mismo para las dos (y para FIND-519).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 (los 4 steps ejecutados) |
| % completado | 100% de la implementación · pendiente el gate P2-01 |

## Steps

1. **Verificar la evidencia** (sin editar): ✅ `opt(...)` alrededor del
   `WHERE` y el `reject_trailing_input` no estricto se comportan como dice la fila
   (leidos `grammar.rs:196-269` y `:737-753`; traza de nom 7.1.3 verificada).
2. **Test primero**: ✅ 8 tests nuevos en `src/parser/mod.rs` + 2 en `src/executor.rs`,
   verificados **fallando** (4 FAIL en el modulo parser; el executor devolvia
   `Ok(Read([UnifiedNode]))` en vez de error).
3. **Fix**: ✅ (a) `parse_where_clause` propaga `Err::Failure` cuando el keyword
   `WHERE` ya se consumio; (b) `reject_trailing_input` estricto para todas las
   statements (tolerando solo `;`); (c) el check de `GROUP BY` reordenado para
   conservar su mensaje; (d) dos mensajes nuevos en `iql_parse_error_message`.
   "Ajustar los llamadores que dependian del descarte silencioso": **no hacia
   falta** — HTTP/MCP/CLI delegan todos en `Executor::execute_hybrid` ->
   `parse_statement` (verificado leyendo `src/server/handlers.rs:126`,
   `vantadb-mcp/src/handlers/tools.rs:1907` y `src/cli_handlers/data.rs:289`), y
   ningun caller consumia el `rest` como statement valido.
4. **Verificar**: ✅ `cargo fmt --check` (limpio en mis 3 ficheros), `cargo clippy -p
   vantadb --all-targets -- -D warnings` (exit 0), suite del lib completa
   (**2517/2517**), integracion (17 + 81 pass) y MCP (115 pass).

## Review (P2-01)

- **Reviewer:** (pendiente — agente DISTINTO del implementador; la tarea NO se marca
  COMPLETED sin este review registrado aqui)
- **Veredicto:** (pendiente)
- **Fecha:** (pendiente)

### Checklist para el revisor (P2-01)

- [ ] **Reproducir el bug original**: con el codigo viejo, `FROM Doc WHERE n =`
      devolvia `Ok(Read([...]))` (escaneo sin filtro). Verificar que ahora es
      `Error::IqlParse` y que **no** se ejecuta nada.
- [ ] **Contrato del task file**: los 9 casos de trailing input de F-02 + los 5 de
      `WHERE` mal formado + el subcase de subquery de FIND-312 devuelven error de
      parseo (tests nuevos listados arriba).
- [ ] **`;` intacto**: `FROM Doc;`, `  FROM Doc ;  `, `SELECT * FROM Doc;`,
      `DELETE NODE#1;` siguen parseando (REPL/CLI).
- [ ] **Ningun statement valido pasa a fallar**: suite del lib verde (2517) + el
      control `test_parse_statement_valid_statements_still_parse` (14 statements).
- [ ] **El `<alias>` documentado no se rompio**: `FROM Doc garbage` sigue siendo una
      statement valida con alias `garbage` (test
      `test_parse_statement_keeps_the_documented_alias`). **Este es el punto donde
      el fix se aparta del ejemplo literal del contrato** — ver el matiz de
      `## Contrato`: rechazar `FROM Doc garbage` exigiria abolir el `<alias>`
      documentado en `docs/api/IQL.md`.
- [ ] **Mensajes utiles y estables**: `"malformed WHERE clause: ..."` y
      `"unexpected trailing input after the statement: ..."`; las posiciones
      (linea/columna) apuntan al `WHERE` / al texto sobrante.
- [ ] **No hay panics**: el parser no introduce `unwrap`/`expect`; los fallos son
      `Err::Failure` (ver `src/parser/grammar.rs`).
- [ ] **Sin cambios fuera de scope**: solo `src/parser/grammar.rs`,
      `src/parser/mod.rs` (tests) y `src/executor.rs` (parseo + tests). Ningun
      fichero de `src/physical_plan/**`, `src/planner.rs`, `src/cost_estimator.rs`,
      `src/index/**`, `src/crypto*`, `src/storage/**`, `benches/**` ni
      `tests/encryption_chaos.rs` fue tocado por este trabajo.
- [ ] **Sin commits**: el commit lo hace el orquestador.

### Follow-ups que salen de este trabajo (NO en el scope de FIND-518)

- **Docs**: `docs/api/IQL.md` documenta que un `;` final se tolera en la superficie
  de agregacion, pero no que **cualquier** trailing input es ahora un error de
  parseo. Actualizar la seccion de estanqueidad (tarea de `vanta-docs`; este
  trabajo no toca `docs/api/**` por scope).
- **FIND-525** (`SIGUE 1..5000000000` se descarta en silencio) y **FIND-526**
  (`FROM Doc where x = 1` se come el `where` como alias) siguen abiertos: son
  *clausulas* que el `opt` traga en `parse_query`, no trailing input. El fix de
  este task los deja mas cerca (el trailing que produzcan ahora es error), pero
  necesitan su propio cambio (`opt` -> estricto tambien para `SIGUE` y alias en
  minusculas).
- **FIND-529** (cobertura de tests del parser): este task cierra los casos de
  F-01/F-02; siguen sin test el limite de profundidad (`min = nan`, `FIND-522`),
  `FROM NODE#abc` (`FIND-524`) y el limite de profundidad de `SIGUE` (`FIND-523`).

## Notas
- Regla de git: el push requiere instruccion explicita del usuario (AGENTS.md Regla 7).
- Este agente **no ha hecho commit** (lo commitea el orquestador).
- Coexistencia: `src/executor.rs` lo editan en paralelo la task FIND-552 (mismo
  fichero, otro bloque: la proyeccion). Los bloques de este trabajo son
  `iql_parse_error_message` (2 ramas) y los 2 tests nuevos al final del modulo
  `tests`; no se toco el codigo de la proyeccion.
