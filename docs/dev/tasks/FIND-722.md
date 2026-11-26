---
title: "FIND-722 — parse() reporta línea y columna del error de IQL"
kind: task
status: completed
description: "Los tres bindings devuelven el Debug crudo de nom sin line/col en parse(). Un único formateador público del core alimenta execute_hybrid y parse()."
---

# FIND-722 — `parse()` sin `line`/`col`: un formateador único

> **Plan:** `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md` §6.b / §6.c
> **Ruta:** `vanta-worker` · **Carril:** R (libre) · **Appetite:** 2-3h
> **Dependencias:** `FIND-156` ✅ (`20c164970`, expuso `parse()`) · `FIND-723` ✅ (`071099119`, ids como string)
> **Origen:** `FIND-156` §Incógnitas 8

## Contexto

`Client.parse()` existe en los tres bindings desde `FIND-156`, pero su error no
sirve para diagnosticar: los tres llaman `parse_statement` y formatean el fallo
como `Error::InvalidInput(format!("IQL parse error: {e}"))`, donde `{e}` es el
`Debug` de `nom::Err` — ruido interno del combinador, **sin posición**:

| Binding | Sitio | Hoy |
|---|---|---|
| Python | `vantadb-python/src/lib.rs:2275-2279` | `IQL parse error: Error(Error { input: "", code: Eof })` |
| WASM | `vantadb-wasm/src/lib.rs:1870-1871` | idem, vía `to_js_err` |
| TS | `vantadb-ts/src/vantadb.ts:1417-1419` | delega en WASM (`_wasm` → `fromNativeError`) |

El path de ejecución **sí** reporta posición: `Executor::execute_hybrid`
(`src/executor.rs:318-337`) usa `iql_error_position` (`:80`) e
`iql_parse_error_message` (`:110`) para construir `Error::IqlParse { msg, line, col }`
(`src/error.rs:214-223`, `Display` = `IQL parse error at line {line}, col {col}: {msg}`).
Ambos helpers son privados, y duplicarlos en cada binding crearía tres fuentes de
verdad (api-contract R-8).

`docs/api/IQL.md:599-605` documenta el hueco explícitamente; `docs/api/TS_SDK.md:567`
repite el caveat.

## Impacto mapeado (Regla 0)

**Leídos completos antes de editar**

| Archivo | Qué se leyó | Por qué |
|---|---|---|
| `src/executor.rs:1-193` + tests `:960-1020, 1400-1460` | helpers de posición, `execute_hybrid`, tests de parse error | es donde vive el formateador y su único call site |
| `src/error.rs:190-260` | variante `IqlParse` y su `Display` | contrato del error que cruza FFI |
| `src/parser/grammar.rs:877-912` | `parse_statement` + `reject_trailing_input` | confirma `(_, stmt)` y que el trailing input ya es `Failure(Eof)` |
| `vantadb-python/src/lib.rs:2232-2286` | `Client.parse` | call site a reemplazar |
| `vantadb-python/src/convert.rs:856-955` | `map_vanta_error` + `attach_err_meta` | taxonomía ya mapea `IqlParse` → `ValidationError` |
| `vantadb-wasm/src/lib.rs:1851-1878, 2239-2254` | `Client::parse` + `to_js_err` | call site y adjunto de metadatos |
| `vantadb-ts/src/errors.ts:1-321` | `DbError`, `fromNativeError`, `mapCodeToCanonical` | dónde aterrizan `line`/`col` |
| `vantadb-ts/src/vantadb.ts:184-190, 1390-1420` | `_wasm` + `parse` | el wrapper sólo delega |
| `tests/api/public_api.rs:1-143` | generación del golden | el `pub fn` nuevo obliga a regenerar |
| `docs/api/IQL.md:591-612`, `docs/api/TS_SDK.md:545-570`, `docs/api/PYTHON_SDK.md:949-968`, `docs/api/BINDINGS_NAMESPACES.md:247-306-364` | §Error Handling y notas de `parse` | docs que quedan falsas |

**Referencias hacia dentro (callers)**

- `execute_hybrid` — 36 callers (`src/python.rs`, `src/tui/repl.rs`, `src/physical_plan/scan.rs`, `src/sdk/api/graph.rs`, …). **No cambia su firma**; sólo delega el parseo.
- `iql_error_position` / `iql_parse_error_message` — 1 caller cada uno (el propio `execute_hybrid`), sin tests que los nombren.
- `Client.parse` — `vantadb-ts` (`vantadb.ts:1417`) → WASM `parse`; `vantadb-python` directo; tests `test_find156_ast_json.py`, `vantadb-ts/src/__tests__/find156.test.ts`.

**Referencias hacia fuera**

- `src/executor.rs` → `crate::parser::parse_statement`, `crate::error::Error` (ambos ya importados).
- `vantadb-python`/`vantadb-wasm` → `vantadb` crate (ya dependen).

**Veredicto:** blast radius real = superficie pública de 3 bindings + 1 símbolo
público nuevo en el core (`vantadb::executor`). `execute_hybrid` y sus 36 callers
no cambian de contrato.

## Spec — decisiones

| # | Decisión | Alternativas descartadas | Evidencia |
|---|---|---|---|
| 1 | **Un solo `pub fn`** que compone los dos helpers y devuelve ya el `Error::IqlParse`: `vantadb::executor::parse_iql(&str) -> Result<Statement>` | Promover los dos helpers y que cada binding los llame (2 llamadas olvidables → 3 copias del orden); `describe_parse_error` que devuelva `(String, usize, usize)` y cada caller arme el `Error` | El contrato pide "una sola fuente"; con un único punto de entrada es imposible construir el error sin posición. Expone además `Statement`, ya público (`src/query.rs`), **no** `nom` — no se filtra el parser a la API pública |
| 2 | `execute_hybrid` pasa a usar `parse_iql` (borra su bloque de formateo) | Dejar `execute_hybrid` como está (2 caminos que hay que mantener sincronizados) | Los dos caminos deben producir el **mismo** `Error` byte a byte: hay un test que lo asserta |
| 3 | `line`/`col` son **1-based** (`line 1` = primera línea, `col 1` = primera columna) — y se documenta | Base 0 (incompatible con `Error`'s `Display` ya documentado en `docs/api/IQL.md:596` y con `openapi.yaml:2814`) | `iql_error_position` ya produce 1-based (`.max(1)`); cambiarlo rompería el contrato publicado del path `query()` |
| 4 | La posición se calcula sobre el texto **después** del `trim_start()`, igual que `execute_hybrid` | Calcular sobre el input crudo (rompería la paridad con `query()`) | `execute_hybrid` pasa `trimmed`; paridad literal es el objetivo |
| 5 | Python expone `line`/`col` como **atributos aditivos** de la excepción; TS los pone en `DbError.details`; **el mensaje se mantiene** (`IQL parse error at line N, col M: …`) | Cambiar la clase/los códigos; reemplazar el mensaje por JSON | Cambiar la forma rompe `except ValidationError as e: str(e)`-style callers (blast radius §7). Los códigos canónicos (`ValidationError` / `VANTADB_VALIDATION_ERROR`) no cambian |
| 6 | El golden `tests/api/public-api.txt` **sí** cambia (un `pub fn` nuevo) y se regenera en esta tarea | Ocultarlo con `#[doc(hidden)]` | El task lo autoriza explícitamente; se documenta el delta exacto en §Verificación |
| 7 | **Se corrige el mezclado de bases**: `col` pasa a ser 1-based de verdad (se reportaba el índice 0-based del token rechazado mientras `line` ya era 1-based) y cuenta **caracteres**, no bytes | Dejar la fórmula y documentar el off-by-one; o documentar `col` como byte offset | Medido: `FROM Person p extra_stuff` reportaba 14 con `extra_stuff` en la columna 15; `FROM Doc LIMIT abc` 9 con `LIMIT` en 10; `… GROUP BY category` 22 con `GROUP` en 23; `FROM Doc AS OF 1 AS OF 2` 17 → 18. La instrucción del task es explícita: "1-based o 0-based: elegí y documentá, **no mezcles**". Formato y clase de error no cambian — sólo el número (+1 salvo posición 0). El caso no-ASCII es el que obliga a caracteres: con `"José"` antes del token, bytes dicen 36 y caracteres 35 |
| 8 | Un fallo **blando** (`Err::Error`) se reporta en (1,1) y se documenta así; no se toca el parser | Reescribir `parse_statement`/`alt` para conservar el mejor error | `parse_statement` (`src/parser/grammar.rs:889-898`) resuelve 7 alternativas con `alt`, que propaga sólo el error de la **última** (`parse_query`, en offset 0). Por eso `INSERT INTO` — keyword conocido, statement incompleto — colapsa a 1:1. Los `Failure` (`Verify`/`Fail`/`Eof`) conservan posición. Arreglar `alt` es rediseño del parser, fuera de scope |

## Steps

1. ✅ RED: tests de core (`parse_iql` == `execute_hybrid`, línea 1 y línea 2) — fallaban con `cannot find function parse_iql`.
2. ✅ GREEN: `parse_iql` + `iql_parse_error` + `execute_hybrid` delegando; `col` corregido a 1-based y a **caracteres**.
3. ✅ Python: `parse()` delega en el core; atributos `line`/`col` en `attach_err_meta` (un solo `Python::attach`); test de binding.
4. ✅ WASM/TS: `to_js_err` adjunta `line`/`col`; `fromNativeError` los preserva en `details`; test de binding.
5. ✅ Docs (`IQL.md`, `TS_SDK.md`, `PYTHON_SDK.md`, `ERROR_HANDLING.md`, `BINDINGS_NAMESPACES.md`).
6. ✅ Golden `public-api.txt` regenerado (+1 línea, ver §Verificación).
7. ✅ Verify + review P2-01 (una ronda, `vanta-review`; ver §Review).

## Contrato

```powershell
cargo nextest run --profile audit -p vantadb -E 'test(iql_error) or test(parse_error)' --build-jobs 2
```

Resultado: ✅ 18/18 (incluye `parse_iql_matches_execute_hybrid_iql_error`,
`parse_iql_reports_line_and_col_for_multiline_parse_error`,
`iql_error_col_is_one_based_for_a_structured_failure` y los 7 asserts de posición
existentes, actualizados al contrato 1-based).

## Verificación

| Gate | Comando | Resultado |
|---|---|---|
| Contrato | `cargo nextest run --profile audit -p vantadb -E 'test(iql_error) or test(parse_error)' --build-jobs 2` | ✅ 13/13 |
| Golden | `$env:VANTADB_PUBLIC_API_UPDATE="1"; cargo nextest run -p vantadb --test public_api --run-ignored ignored-only` y luego el mismo comando **sin** la variable (compare) | ✅ delta +1 línea, compare PASS |
| fmt | `cargo fmt --check` | ✅ |
| clippy | `cargo clippy -p vantadb -p vantadb_py -p vantadb-wasm --all-targets -- -D warnings` | ✅ |
| Python | `maturin develop --release` + `target/audit-venv/Scripts/python -m pytest vantadb-python/tests/test_find156_ast_json.py -q` | ✅ 10/10 |
| TS types | `npx tsc --noEmit` | ✅ |
| TS tests | `npx vitest run src/__tests__/find156.test.ts` + `npm test` completo | ✅ 10/10 y 395/395 (26 files) |
| Docs | `node scripts/docs/check-links.mjs` + `check-docs.mjs` | ✅ 0 broken (exit 0) |
| OCR | `pwsh dev-tools/ocr-review.ps1 -Format json` | ✅ advisory, sin Critical/High |

**Delta exacto del golden** (`tests/api/public-api.txt`, +1 línea, ni una más):

```diff
+pub fn vantadb::executor::parse_iql(&str) -> vantadb::error::Result<vantadb::query::Statement>
```

No filtra `nom`: el tipo público es `Result<Statement>` (`Statement` ya era
público en `src/query.rs`).

## Definition of Done

- Nivel **Task** (cambio acotado a 1 módulo del core + 3 bindings + docs).
- [x] Test RED→GREEN en el core y test de binding en Python y TS.
- [x] Contrato mecánico y golden verificados.
- [x] Docs de API sincronizadas (Regla 3: `fn pub` nuevo → `docs/api/` en el mismo cambio).
- [x] Review P2-01 por agente distinto (abajo).
- **Release: no aplicable** — sin bump de versión ni changelog manual (release-plz
  lo genera desde el `feat:`), y sin cambios de formato on-disk ni de wire shape
  (sólo campos aditivos de error).

## Review

**P2-01 (ronda única, `vanta-review`, contexto freso ≠ autor):**
`changes-required` → 1 High (golden sin regenerar) + 4 Medium + 3 Low.

| # | Sev | Hallazgo | Disposición |
|---|---|---|---|
| H1 | High | Golden no regenerado: el `assert_eq!` de `tests/api/public_api.rs` falla con +1 línea | ✅ Corregido y verificado (§Verificación) |
| M1 | Medium | `col` contaba bytes pero la doc prometía caracteres | ✅ Corregido: `input[line_start..consumed].chars().count() + 1` + assert multibyte (`José` → 35, no 36) |
| M2 | Medium | La explicación del (1,1) citaba "keyword desconocido", pero `INSERT INTO` tiene keyword conocido | ✅ Corregido: la razón es `alt` propagando el error de la última alternativa (decisión 8 + doc + comentario del test) |
| M3 | Medium | Task file sin `## Contrato`/`## Definition of Done`/`§Verificación`/`## Review` | ✅ Esta sección |
| M4 | Medium | `parse()` y `query()` divergen en input LISP `(…)` | ✅ Documentado en `IQL.md` §Error Handling + assert de binding |
| L1 | Low | `.min(input.len())` no protegía la resta (se aplicaba después) | ✅ `input.len().saturating_sub(e.input.len())` |
| L2 | Low | Doble `Python::attach` por `IqlParse` | ✅ `attach_err_meta_parts` recibe `Option<(usize, usize)>`: un solo attach |
| L3 | Low | Faltaba caso multibyte | ✅ Añadido |

**Veredicto final: approve** (todo Critical/High/Medium cerrado con evidencia).

## NOTICED BUT NOT TOUCHING

- `tests/api/public_api.rs:123` usa `std::env::var_os(UPDATE_ENV).is_some()`, así
  que una variable **presente y vacía** entra en modo *update* y reescribe el
  golden devolviendo PASS (falso verde detectado durante el review). Fuera del
  scope de `FIND-722` (el task permite tocar sólo `public-api.txt`); fix de 1
  línea: `var_os(...).is_some_and(|v| !v.is_empty())`.
