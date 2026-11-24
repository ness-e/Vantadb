---
title: "FIND-156 — exponer el AST JSON del parser en los bindings Python y TS"
kind: task
status: in-progress
---

# FIND-156 — AST JSON en los bindings (Python + TS)

> **Plan:** `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md` §6, Fase 2, wave 2.1
> **Ruta:** `vanta-worker` · **Carril:** R (Rust, cupo 2) · **Appetite:** 3h
> **Origen de la fila (Backlog):** `FIND-156` — *"Exponer el AST JSON en bindings Py/TS (el core ya serializa)"*
> **Contrato del plan:** test de binding con el mismo shape que `test_ast_json_projection_shape`.

## Contexto

API-06 dejó el `DEFER` *"prohibición de tocar bindings"*; esa restricción ya no
aplica. El core **ya** serializa el AST: los tipos `Statement`/`Query`/`SelectStatement`
derivan `Serialize` y el shape está pinneado por
`src/parser/mod.rs::tests::test_ast_json_projection_shape` y documentado en
`docs/api/IQL.md` §AST JSON. Lo que falta es la superficie de binding: no existe
ningún `parse` en `vantadb-python`, `vantadb-wasm` ni `vantadb-ts`.

Evidencia de la ausencia (`rg -n "fn parse"` sobre los 4 bindings → 0 hits;
`rg -n "parse" vantadb-ts/src/vantadb.ts` → 0 hits de un método `parse`).

## Spec — decisiones (gate spec-first)

| # | Decisión | Alternativas descartadas | Evidencia |
|---|---|---|---|
| 1 | El método se llama `parse(query)` en las 3 superficies (Python, WASM, TS), no `ast()`/`explain()`/`query_ast()` | `ast()` — colisiona con el módulo `ast` de Python en docs/ejemplos; `query_ast()` — stutter (`query`+`ast`) | El plan lo nombra `parse`; `docs/api/BINDINGS_NAMESPACES.md` agrupa por dominio con nombres cortos (AST-012 anti-stutter) |
| 2 | La proyección es **la del core**: `serde_json::to_string(&Statement)` donde `Statement` viene de `vantadb::parser::parse_statement` | Proyección propia (walk del AST en el binding) | `docs/api/IQL.md:526-528` — *"Rust consumers serialize the `Statement` returned by `vantadb::parser::parse_statement` with `serde_json`"*; R-8 `api-contract.md` prohíbe reimplementar lógica del core |
| 3 | Devuelve un **valor nativo** (dict en Python, object en TS), no un `str` | Devolver el JSON como string (cero glue) | Paridad Py/TS: un `str` obligaría a `json.loads` en Python y `JSON.parse` en JS, y rompería la comparación estructural del test de binding. Python usa `json.loads` (stdlib) y WASM `js_sys::JSON::parse` — ambos parsean el texto que produjo el core |
| 4 | `parse` es **puro**: no entra al `op_gate` | Envolver con `enter(&self.op_gate)` como `query()` | El parser no toca el engine; gatearlo haría fallar la inspección de queries sobre un `Client` cerrado, un modo de fallo inventado. Documentado en el docstring |
| 5 | Error de parseo = `Error::InvalidInput("IQL parse error: …")` en el binding, mapeado por `map_vanta_error` / `to_js_err` → `ValidationError` / `DbError(VANTADB_VALIDATION_ERROR)` | Reusar `Error::IqlParse` con line/col | `iql_error_position`/`iql_parse_error_message` son **privadas** en `src/executor.rs:80,110` y ese fichero está prohibido (`FIND-710` en vuelo). Duplicarlas en 2 bindings violaría R-8 (fuente única). `ValidationError` es la clase que `map_vanta_error` ya asigna a `IqlParse` (`convert.rs:890`), así que la taxonomía pública no cambia |
| 6 | El TS delega en WASM (`vantadb.ts` importa `Client` de `vantadb-wasm`) | Implementar parse en TS | `vantadb.ts:1` — `import { Client as WasmClient } from "vantadb-wasm"`. Sin el método en `vantadb-wasm/pkg` el binding TS no compila; el slice vertical exige las 3 capas |

## Impacto mapeado (Regla 0)

**Archivos leídos completos / por sección antes de editar:**

| Archivo | Qué se leyó | Por qué |
|---|---|---|
| `src/parser/grammar.rs:889` | firma `pub fn parse_statement(i: &str) -> IResult<&str, Statement>` | es la fuente del AST |
| `src/parser/mod.rs:1559-1576` | `test_ast_json_projection_shape` | shape pinneado (snapshot de referencia) |
| `src/parser/mod.rs:5-9` | `pub mod grammar; pub use grammar::*;` | confirma que `vantadb::parser::parse_statement` es superficie pública |
| `src/lib.rs:133` | `pub mod parser;` | idem |
| `src/executor.rs:315-337, 80-135` | `execute_hybrid`, `iql_error_position`, `iql_parse_error_message` (privadas) | decisión 5 |
| `vantadb-python/src/lib.rs:743-744, 2191-2295` | bloque `#[pymethods] impl Client`, `query`, `query_structured` | patrón a seguir + punto de inserción |
| `vantadb-python/src/convert.rs:866-891` | `map_vanta_error` (IqlParse/InvalidInput → `ValidationError`) | taxonomía de error |
| `vantadb-wasm/src/lib.rs:1820-1849` | `query`, `to_js`, `to_js_err` | patrón a seguir |
| `vantadb-ts/src/vantadb.ts:1-46, 1365-1388, 184` | imports, `query()`, `_wasm<T>` | patrón a seguir |
| `vantadb-wasm/pkg/vantadb_wasm.d.ts:842-845` | declaración de `query` | el `pkg/` es artefacto gitignored |
| `docs/api/IQL.md:512-559` | §AST JSON (la fuente del shape) | snapshot de referencia |
| `docs/api/BINDINGS_NAMESPACES.md:320-400` | tabla Python + totals | dónde registrar el método |
| `vantadb-python/tests/test_stub_drift.py` | gate `.pyi` vs runtime | obliga a tocar el `.pyi` y a rebuildear el `.pyd` |

**Referencias hacia dentro (quién depende de lo que edito):**

- `Client` de Python: `vantadb-python/vantadb_py/__init__.py` (wrapper + sub-clients
  `db.system`), `.pyi` (2), `tests/test_stub_drift.py`, `tests/test_subclients.py`,
  `tests/test_w1_surface.py`.
- `Client` de WASM: `vantadb-ts/src/vantadb.ts` (único import TS del paquete),
  `vantadb-wasm/tests/*`, `vantadb-ts/src/__tests__/parity.test.ts`.
- **Un método nuevo es aditivo**: nada existente cambia de firma ni de shape.

**Veredicto de impacto:** ADITIVO en 3 capas + docs. Riesgo real = shape divergente
(pre-mortem a), `.pyi` sin espejar (pre-mortem b) y TS declarado verde sin typecheck
(pre-mortem c). Los 3 tienen gate mecánico en §Contrato.

## Steps

| # | Step | Verify | Resultado |
|---|---|---|---|
| 1 | Baseline: `cargo nextest run --profile audit -p vantadb -E 'test(ast_json)'` | verde | ✅ 3 passed (el core no se tocó) |
| 2 | `vantadb-python`: `Client::parse` + `.pyi` + `tests/test_find156_ast_json.py` | `cargo check -p vantadb_py --offline` + pytest + `test_stub_drift.py` | ✅ check 0 · pytest 8/8 · stub drift 7/7 |
| 3 | `vantadb-wasm`: `Client::parse` + `src/vantadb_wasm.d.ts` (fuente del `pkg/`) | `cargo check -p vantadb-wasm --target wasm32-unknown-unknown` | ✅ check 0 · `wasm-pack build --release` 5m31s · `build-wasm-types.mjs --check` in sync |
| 4 | `vantadb-ts`: `parse()` + `AstJson` + `src/__tests__/find156.test.ts` | `npx tsc --noEmit` + `npx vitest run src/__tests__/find156.test.ts` | ✅ tsc 0 · vitest 7/7 |
| 5 | Docs: `BINDINGS_NAMESPACES.md` (3 tablas + totals) + `PYTHON_SDK.md` §`parse()` | gates de docs | ✅ `check-links` 0 · `check-docs` 0 · `validate-docs-coverage` 0 gaps |
| 6 | Verify full scoped + review P2-01 + commit local | ver §Contrato | ver §Review |

## Contrato

```bash
cargo nextest run --profile audit -p vantadb -E 'test(ast_json)' --build-jobs 2
cargo check -p vantadb_py --offline -j 2
cargo check -p vantadb-wasm --target wasm32-unknown-unknown
cd vantadb-ts && npx tsc --noEmit
cd vantadb-ts && npx vitest run src/__tests__/find156.test.ts
python -m pytest vantadb-python/tests/test_find156_ast_json.py -v
```

Test de binding obligatorio en ambas plataformas: misma query representativa
(`FROM Person p WHERE edad == 28 FETCH name`) y misma aserción que
`test_ast_json_projection_shape` (nodo `Query`, `from_entity`, `target_alias`,
`where_clause[0].Relational`, `fetch`, y los nulos aditivos).

## Verificación medida

| Comando | Resultado |
|---|---|
| `cargo nextest run --profile audit -p vantadb -E 'test(ast_json)'` | ✅ 3 passed, 2974 skipped |
| `cargo check -p vantadb_py --offline -j 2` | ✅ exit 0 |
| `cargo check -p vantadb-wasm --target wasm32-unknown-unknown` | ✅ exit 0 |
| `cargo fmt --check -p vantadb_py -p vantadb-wasm` | ✅ exit 0 |
| `cargo clippy -p vantadb_py --all-targets --offline -- -D warnings` | ✅ exit 0 (0 warnings) |
| `cargo clippy -p vantadb-wasm --target wasm32-unknown-unknown -- -D warnings` | ❌ **pre-existente, fuera de scope**: 3 errores en `src/wal.rs:1347` (`drop_non_drop` — `std::fs::File` no implementa `Drop` en wasm32). Fichero no tocado por esta tarea y sin modificar en el árbol |
| `pytest tests/test_find156_ast_json.py` (venv `.venv`, py3.11.9) | ✅ 8/8 (incluye `parse` sobre un `Client` cerrado) |
| `pytest tests/test_stub_drift.py` | ✅ 7/7 (el `.pyi` espeja el runtime) |
| `npx tsc --noEmit` (+ typecheck explícito del test excluido por `tsconfig`) | ✅ exit 0 |
| `npx vitest run src/__tests__/find156.test.ts` | ✅ 7/7 |
| `node dev-tools/build-wasm-types.mjs --check` | ✅ in sync |
| `node scripts/docs/check-links.mjs` · `check-docs.mjs` · `validate-docs-coverage.ps1` | ✅ 0 broken · 0 · 0 gaps |
| `pwsh dev-tools/ocr-review.ps1` | ✅ spec generado (15 reviewable / 28 total; docs excluidos por `unsupported_ext`) |

## Invariantes

- El shape del AST **no se redefine** en ningún binding: sale de `Serialize` del core.
- `.pyi` y runtime no pueden divergir (`test_stub_drift.py` es el gate).
- Nada de `unwrap()`/`expect()` en el código nuevo.
- `pkg/` (gitignored) se regenera desde `vantadb-wasm/src/vantadb_wasm.d.ts`; nunca se edita a mano.

## Incógnitas / deuda

1. **`cargo clippy -p vantadb-wasm --target wasm32-unknown-unknown -- -D warnings`
   está rojo en `src/wal.rs:1347`** (`drop_non_drop`) con rust 1.95 — pre-existente,
   no lo introduce esta tarea (`git status -- src/wal.rs` vacío) y es dominio de
   Engine/Arch. Candidato a fila `FIND-*`: la lane wasm32 no tiene clippy verde hoy.
   Comando exacto del verde parcial alcanzado:
   `cargo clippy -p vantadb-wasm --target wasm32-unknown-unknown --offline -j 2 -- -D warnings`.
2. **Suite completa `vantadb-ts` (26 ficheros): 386/390** — 4 rojos en
   `portability.test.ts` (2) y `hardening.test.ts` (2), todos de mantenimiento
   (`rebuildIndex` IO en DB no persistente, `reindexHnswFromText` error de plataforma,
   zero-norm cosine). Aparecieron al reconstruir `pkg/` desde un árbol de trabajo que
   arrastra el WIP de otra sesión (`src/executor.rs`, FIND-710). El diff de esta tarea
   es **puramente aditivo** (`+20/-0` en `vantadb-wasm/src/lib.rs`), así que no puede
   cambiar el comportamiento de esas rutas; re-verificar tras aterrizar FIND-710.
   El test acotado de esta tarea pasa 8/8.
3. **Suite completa Python bajo `vantadb-python/.venv`: 242 passed / 19 failed**, todos
   en ficheros ajenos a `parse` (`test_step22_migrations`, `test_step23_*`,
   `test_typed_errors`, `test_wire_u128`) y **todos verdes en aislamiento**. Los 19 rojos
   se reproducen con una selección de 5 ficheros que **no incluye** `test_find156_ast_json.py`
   (→ no es esta tarea). Causa: contaminación entre tests por RSS de proceso (lo describe
   el docstring de `tests/conftest.py`) en un venv que no es el hermético.
   **El venv canónico `target/audit-venv` (`.opencode/references/python-sdk.md`) no existe
   en esta máquina**; recrearlo exige `dev-tools/setup_venv.ps1` (crea venv + `pip install`
   con red). El binding se compiló con `maturin develop` (debug) en `vantadb-python/.venv`
   (py3.11.9), que es el venv presente.
4. **Posiciones de error de parseo**: el mensaje es `IQL parse error: <nom>` sin
   `line`/`col` (vs. el formato de `docs/api/IQL.md` §Error Handling, que ahora está
   acotado explícitamente a `query()`), y filtra el `Debug` de `nom::error::Error`
   (review R3). Decisión 5 de §Spec: los helpers que calculan posición y mensaje
   curado son privados en `src/executor.rs` (prohibido, FIND-710). Limpieza futura:
   promoverlos a `pub` en el core (p.ej. `parser::describe_parse_error(input, &err)`)
   y que `execute_hybrid` + los 3 bindings llamen a esa única fuente — una fila `FIND-*`.
6. **`vantadb-ts/src/native.ts` (backend napi) no expone `parse`**: requiere una función
   en `vantadb-node/src/lib.rs` (fuera del alcance declarado). Gap de paridad TS
   wasm-vs-native, candidato a fila `FIND-*`.
7. **`u128`/`i64` en el AST JSON no son exactos en JS** (review C1): `serde_json`
   emite `node_id: u128` (`src/query.rs:39,63,70-72`) y `Value::Int(i64)`
   (`src/sdk/types.rs:128`) como números JSON planos; `js_sys::JSON::parse` los
   decodifica como double → pérdida de precisión por encima de 2^53 (probado:
   `2^53+1` → `2^53`). Python no está afectado (`json.loads` es arbitrario). El
   core ya tiene el precedente de `u128_serde` (API-01) que serializa ids como
   string; aplicarlo al AST **cambia el shape público** (IQL.md §AST JSON +
   `test_ast_json_projection_shape`) → requiere decisión de API/Engine. Hasta
   entonces el límite está documentado y **fijado por test**
   (`find156.test.ts > pins the documented >2^53 precision limitation`).
   Candidato a fila `FIND-*`.
8. **`parse` es flat-only** (no `db.system.*`): si se quiere paridad con `query`
   (que sí está en el sub-client), es 1 línea en `forward_to_db!` + 1 en el `.pyi`
   + rebuild del `.pyd`. Declarado como flat-only en las 3 filas de
   `BINDINGS_NAMESPACES.md`.

## Review

**P2-01 adversaria (agente distinto: `vanta-review`, contexto fresco) — ronda 1:
🔴 CHANGES REQUIRED (1 Critical, 3 Required, 4 Optional).** El reviewer no confió
en los verdes reportados: ejerció el **artefacto construido** (`vantadb-wasm/pkg`)
con `node --experimental-wasm-modules`. Resolución de cada punto:

| # | Hallazgo | Resolución en este commit |
|---|---|---|
| **C1** | `js_sys::JSON::parse` convierte todo número a IEEE-754 → `u128` node ids y `Int` >2^53 pierden precisión en WASM/TS, mientras la doc decía "exacto". Probe del reviewer: `DELETE NODE#123456789012345678901234` → `1.2345678901234569e+23`; `== 9007199254740993` → `9007199254740992` | **Documentado, no silenciado**: caveat en `AstJson` (`vantadb_wasm.d.ts` + `types.ts`), en el docstring del `parse` de WASM (el "match exactly" queda acotado a *shape*), en `TS_SDK.md` y en `PYTHON_SDK.md` (Python sí es exacto). **Test nuevo que lo fija**: `pins the documented >2^53 precision limitation` (8/8) — cuando el core serialice `u128`/`i64` como string, el test falla y obliga a quitar el caveat. El fix real (serde del AST en el core) es cambio de shape de API pública → fuera de alcance de un binding (Gate D); queda como follow-up en §Incógnitas |
| **R2** | Quitar `_assertOpen()` fue correcto, pero contradice `vantadb.ts:414` y `TS_SDK.md:764` ("todos los métodos lanzan tras `close()`") | Ambas frases acotadas: "todo método que toca el engine … `parse()` es la excepción (puro)". Se mantiene sin guard (el reviewer validó la decisión) |
| **R3** | El error de parseo no lleva `line`/`col` y filtra el `Debug` de `nom::error::Error` (`Parsing Failure: Error { input: "WHERE", code: Verify }`), perdiendo los mensajes curados que `execute_hybrid` sí produce | Imposible reusar: `iql_error_position`/`iql_parse_error_message` son privados en `src/executor.rs` (prohibido, FIND-710) y duplicarlos viola R-8. **`IQL.md` §Error Handling ahora acota el formato posicionado a `query()`** y declara que `parse()` devuelve el texto del parser sin posición; el follow-up (promover el helper a `pub` en el core) queda registrado en §Incógnitas 4 |
| **R4** | `parse` falta en el sub-client TS `SystemClient` (y en el `db.system` de Python) | Se deja **flat-only** de forma explícita en las 3 filas de la tabla (no se añade a los sub-clients: 2 líneas en TS + rebuild del `.pyd` en Python, sin valor en este slice) |
| O1 | `TS_SDK.md` sin sección `parse()` | ✅ añadida (`#### parse()` bajo *IQL Queries*) con el caveat numérico y el contrato de error |
| O2 | Total WASM del doc desviado (`forget` real y no listado → 55, no 54) | ✅ fila `forget` añadida y total corregido a **55** (verificado contra `impl Client`: 55 `pub fn`) |
| O3 | `AstJson` declarado dos veces (`vantadb_wasm.d.ts` y `types.ts`) → deriva silenciosa que `tsc` no caza | ✅ `types.ts` ahora **re-exporta** `AstJson` desde `vantadb-wasm` |
| O4 | `parse` era de los pocos métodos Python sin línea `GIL Policy:` | ✅ añadida (`HELD` + razón: cómputo en memoria de µs + `json` de CPython; no hay I/O que justifique soltar el GIL) |
| Nit | `match="IQL parse error"` demasiado laxo; ningún test cubría >2^53 | ✅ el test Python ahora asserta también `code == "VANTADB_VALIDATION_ERROR"`; el test TS fija el límite de 2^53 |

Verificado el artefacto tras los cambios: `pytest` 15/15 (8 del fichero + 7 de
`test_stub_drift`) · `npx vitest run src/__tests__/find156.test.ts` **8/8** ·
`npx tsc --noEmit` 0 · `node dev-tools/build-wasm-types.mjs --check` in sync ·
`cargo fmt --check -p vantadb_py -p vantadb-wasm` 0 · docs gates 0 broken / 0 gaps.

## SDP

`SDP:` campaign-executor, progreso, security-and-hardening, documentation-and-adrs,
api-and-interface-design, source-driven-development, incremental-implementation,
test-driven-development.

