---
title: "FIND-158: deuda API-03 - cuatro items de superficie, cuatro decisiones, un tipo alineado"
kind: task
status: in-progress
description: "El barrido de parciales de 2026-11-27 dejo cuatro items vivos en la fila: versions sin paginar, export/import verbales sin nota, next_cursor del nodo que excluye la forma string del contrato compartido, y el cursor de fan-out multi-namespace. Se decide uno por uno y se documenta; solo el tipo se toca en codigo."
id: FIND-158
---

# FIND-158 - deuda API-03: decidir es el trabajo, no corregir una nota

> **Carril:** docs + un tipo de binding, **sin compilar el core** (dos carriles Rust vivos,
> `--build-jobs 2`, disco ~16 GB). Escritura: `src/server/handlers.rs` (solo doc comments) ·
> `docs/api/openapi.yaml` · `docs/api/HTTP_API.md` · `vantadb-node/index.d.ts` ·
> `vantadb-node/dts-header.d.ts` · `vantadb-node/tests/api.test.ts` · este task file ·
> `docs/dev/Backlog.md` (solo la fila `FIND-158`) · `docs/dev/avance/activo/bindings.md`.
> **Prohibido y respetado:** `src/text_index.rs`, `src/storage/engine/get.rs` (carril 513/812) ·
> `vantadb-mcp/src/memory_scheduler.rs` y `vantadb-{python,node,wasm}/src/**`
> (carril WIRE-23/EGO-15) · `src/sdk/api/**` · `vanta-memory/**` · `.github/**` ·
> `dev-tools/**` · `.opencode/**` · `docs/dev/plans/**` · `docs/index.md`.

## Premisa re-medida en HEAD (2026-11-27)

La fila trae cuatro items y **dos de ellos ya no dicen lo que decian**. Lo que se midio:

| Item | Estado medido | Evidencia |
| --- | --- | --- |
| (a) `GET /records/{ns}/{key}/versions` sin paginar | **VIVA** | `src/server/handlers.rs:286-289` — `RecordsVersionsParams` solo tiene `version: Option<u64>` |
| (b) "nota YAML vs REST plurales" | **NO EXISTE** | 0 hits de `naming`/`REST convention` en `openapi.yaml`/`HTTP_API.md`; lo que queda es decidir la excepcion |
| (c) `next_cursor` inconsistente en el artefacto publicado | **VIVA, con el signo al reves de lo que decia la fila** | `index.d.ts:120` acepta `number \| string`; `:148` declara `number`. **Medido en runtime**: el addon nativo emite un **number** (`typeof page.next_cursor === 'number'`, ver abajo) — el tipo no miente sobre el nodo, pero **excluye** la forma decimal que el contrato compartido documenta (`vantadb-ts/src/types.ts:159-161`) y que el backend WASM emite |
| (d) cursor de fan-out multi-namespace | **VIVA con mitigacion** | `src/server/list_records.rs:107,116` usa un cursor global y pide cada namespace con `cursor: None`; los capados se reportan en `truncated_namespaces` (`:126`, `handlers.rs:449-453`) |

Medicion propia del item (c), sobre el addon precompilado (no heredada de la fila):

```
node --input-type=module -e "<putBatch 3 records>; db.list('n', {limit: 2})"
→ records: 2  next_cursor: 2  typeof: number
```

## Decisiones (una por item, con su razon)

### (a) `versions` — **se documenta el limite, no se pagina**

El endpoint devuelve el historial **retenido** de una clave, y ese historial tiene techo por
configuracion: `version_history_limit` (`src/config.rs:505` → **Some(32)**) con eviccion FIFO
en cada escritura (`src/sdk/version_history.rs:310,335,393-417`). Sin techo queda solo si el
operador lo apaga explicitamente (`VANTADB_VERSION_HISTORY_LIMIT=0` → `None`,
`src/config.rs:1730-1737`).

Paginar exigiria cambiar la forma del wire de **array** a **objeto de pagina** (el esquema unico
`limit`+`cursor`+`has_more` de `src/server/pagination.rs:20-41`), lo que rompe a todo consumidor
del array actual — incluido el fallback HTTP de MCP (`memory_versions`) — para proteger una
configuracion que el operador eligio al desactivar el techo. **Se declara el contrato** (ventana
fija por clave + `?version=N` como lectura puntual) en `openapi.yaml` y `HTTP_API.md`, y se
deja el porque en el codigo, junto al struct. Si alguna vez hace falta paginar de verdad, el
disparador esta escrito: techo desactivado + claves con historial grande.

### (b) `export`/`import` verbales — **se documenta la excepcion**

`POST /api/v2/export` y `/import` no son colecciones: son acciones RPC sobre datos que el
servidor produce/consume fuera del arbol de recursos (un fichero en su disco). Un
`POST /exports` implicaria un recurso "export" almacenado que se puede listar y recuperar, y
no existe. Renombrar sin crear ese recurso seria cambiar el nombre por el nombre. **Se declara
la excepcion** en `openapi.yaml` y en `HTTP_API.md`, con la regla de cuando aplica
(colecciones que se pueden `GET` → plural; acciones de un solo disparo → verbo).

### (c) `next_cursor` — **se alinea el tipo a `number | string`**

Es el unico cambio de codigo de la fila y el mas barato. `MemoryListResult.next_cursor` pasa a
`number | string` en el header que genera el `.d.ts` y en el artefacto publicado, con el
comentario que dice cual backend emite cual. No se cambia el runtime: el addon nativo sigue
emitiendo un number (pinnado por test), y el tipo deja de excluir la forma decimal que el
contrato compartido (`vantadb-ts/src/types.ts:159-161`) ya documenta.

> **Direccion del cambio:** la fila pedia "corregir la salida a `string`". Eso habria sido
> **mentir en el otro sentido** (el nodo emite `2`, no `"2"`). Se ensancha a la union, que es
> lo unico verdadero para ambos backends.

El docstring del binding nativo (`vantadb-node/src/lib.rs:499`) dice `next_cursor?: number`;
**no se toca**: `vantadb-node/src/**` es carril WIRE-23/EGO-15. Queda declarado abajo como
delta, con el texto exacto.

### (d) cursor de fan-out — **el contrato es el cursor global + `truncated_namespaces`**

Un cursor compuesto (posicion por namespace, codificada opaca) obligaria a versionar una
estructura nueva en el wire y a estabilizar la lista de namespaces entre paginas — para
resolver un caso que **ya esta señalizado**: la respuesta lleva `truncated_namespaces` con los
namespaces cuyo listado quedo capado en esta pagina. Se declara ese campo como el contrato
("request these namespaces individually for complete results") y se corrige el texto de
`HTTP_API.md:411-413`, que seguia citando un `NS_CAP = 10_000` que **ya no existe en el codigo**
(la ventana por namespace hoy es `cursor + limit + 1`, `src/server/list_records.rs:108` + el probe
de `handlers.rs:487`).

## Impacto mapeado (Regla 0)

- **Archivos leidos completos:** `src/server/handlers.rs:240-558`, `src/server/pagination.rs`,
  `src/server/list_records.rs`, `src/sdk/version_history.rs:340-470`,
  `src/config.rs:495-520` y `:1730-1739`, `vantadb-node/index.d.ts:95-169`,
  `vantadb-node/dts-header.d.ts` (region `MemoryList*`), `vantadb-node/src/lib.rs:480-520`,
  `vantadb-node/tests/api.test.ts:141-200`, `docs/api/openapi.yaml:455-514` y `:697-739`,
  `docs/api/HTTP_API.md:380-449`, `.opencode/rules/api-contract.md`,
  `.opencode/rules/server-mcp.md`, `vantadb-ts/src/types.ts:157-162`.
- **Referencias hacia dentro:** `get_version`/`versions` del SDK solo los llama
  `src/server/handlers.rs` en la superficie HTTP; `MemoryListResult` lo consume
  `index.d.ts`/`index.js` (el `d.ts` se genera concatenando `dts-header.d.ts` + declaraciones).
- **Referencias hacia fuera:** el `d.ts` lo publica npm (`files: ["index.d.ts", ...]`); los
  consumidores HTTP son la consola (`web/`, fuera de este arbol) y el fallback MCP
  (`docs/dev/architecture/mcp-35-http-fallback-spec.md`), ninguno de los cuales pierde nada:
  no se cambia ninguna forma del wire.
- **Veredicto:** riesgo = 0 de ruptura de contrato (solo se documenta y se ensancha un tipo).
  El gate de `cargo doc` se corre igual porque `RecordsVersionsParams` es `pub`.

## Steps

| # | Step | Estado |
| --- | --- | --- |
| 1 | Re-medir los cuatro items contra HEAD y descartar el item (b) tal como lo describia la fila | ✅ |
| 2 | (a) declarar el techo de `versions` en el doc comment del handler + `openapi.yaml` + `HTTP_API.md` | ✅ |
| 3 | (b) declarar la excepcion verbal `export`/`import` en `openapi.yaml` + `HTTP_API.md` | ✅ |
| 4 | (c) `next_cursor: number \| string` en `dts-header.d.ts` + `index.d.ts` + test que pinnea el runtime | ✅ |
| 5 | (d) declarar el contrato de fan-out y corregir el `NS_CAP` stale de `HTTP_API.md` | ✅ |
| 6 | Verificacion mecanica + commit, Backlog y avance | ✅ |

## Verificacion

| Comando | Resultado |
| --- | --- |
| `cargo fmt --check` | sin diff en mis archivos (los 7 diffs son de `impl_rebuild.rs`, `storage/engine/get.rs` y `text_index.rs`, carriles ajenos) |
| `cargo public-api` / `tests/api/public-api.txt` | **no aplica**: sólo doc comments, ninguna firma publica cambia; el snapshot compara firmas, no texto de doc |
| `npx vitest run tests/api.test.ts -t "FIND-158"` | **1 passed / 33 skipped** (tras el fix B1 del review: `toBeNull()`) |
| `python -c "yaml.safe_load(open('docs/api/openapi.yaml'))"` | OK, 38 paths |
| `pwsh scripts/validate-docs-coverage.ps1` | **0 gaps** |

## Review P2-01

Tier **adversarial** (`docs/api/**`). Ronda 1 por `vanta-review` en contexto fresco:
**`changes-required` — 1 Blocking / 5 Required / 4 Optional**. Todo aplicado:

- **B1 (Blocking) — el comentario nuevo del `.d.ts` mentía igual que el tipo viejo.** Decía
  "absent on the last page" y la unión excluía `null`. **Falso, medido:** `MemoryListPage.next_cursor:
  Option<usize>` no lleva `skip_serializing_if` (`src/sdk/types/record.rs:555-560`), así que el
  addon nativo emite `next_cursor: null` con la clave **presente** (el WASM sí la omite,
  `vantadb-wasm/src/lib.rs:1350-1356`). Un cliente que creyera el comentario haría
  `=== undefined`, realimentaría `null` y `parse_list_options` lo trataría como primera página →
  bucle. Corregido: `number | string | null`, texto que dice qué backend hace qué, y el test pasa
  de `toBeFalsy()` a **`toBeNull()`** (era la aserción laxa la que dejaba pasar B1).
- **R1:** `### 1.1 Canonical codes (10 …)` → `(11 …)`.
- **R2:** las frases absolutas sobre `-32602` eran falsas — `validation()` se construye sobre
  `invalid_params` y **siempre** fija `data.code` (`vantadb-mcp/src/error.rs:46-53`, 46 call
  sites). Acotadas en `ERROR_HANDLING.md` §6.1 y en la tabla de `MCP.md`.
- **R3:** "Every … carries `data.code`" era falso para el path proxy (`proxy.rs:306,335` sin
  `data`; `:319,795` sin `data.code`). Reescrita nombrando la excepción como deuda declarada.
- **R4:** `retriable: false` contra §6.4 "retry once": nota nueva que ata la retriabilidad de
  `-32603` a `data.retriable` y declara que `server.rs:719,743` mandan un timeout por esa puerta.
- **R5:** `cursor + limit` → **`cursor + limit + 1`** (el probe de `has_more` de `handlers.rs:487`).
- **O2/O4 aplicados.** **O1** corregido en `docs/api/EMBEDDED_SDK.md` (fuera del carril
  declarado; se declara): decía que Python/TS/MCP/HTTP "normalize to: 10 codes" y ahora hay un
  11.º **de transporte** que `code()` no devuelve. **O3** aplicado: el test de compat pasa de
  `j["data"].is_null()` (pasa con clave ausente **o** null) a `get("data").is_none()`.
- **Drive-by en la misma sección:** el ejemplo de §6.3 usaba `"code": "VALIDATION_ERROR"` sin
  prefijo; §1.2 exige los valores `VANTADB_*` en el wire.
- **No aplicado, con razón:** la sugerencia de `?limit=&offset=` manteniendo el array (el
  reviewer la ofrece como alternativa a (a)). No cambia la decisión —el techo sigue acotando el
  payload— y añadiría dos parámetros que ningún consumidor pide hoy; queda anotada como la
  salida si el techo se desactiva en un despliegue real.

## Observaciones fuera de carril (NOTICED BUT NOT TOUCHING)

- `vantadb-node/src/lib.rs:499` — el docstring del binding nativo sigue diciendo
  `next_cursor?: number`. **No se toca**: `vantadb-node/src/**` es carril WIRE-23/EGO-15.
  Delta exacto para ese carril: cambiar el tipo del docstring a `number | string | null` y
  aclarar que el addon emite un number (o `null` en la última página) mientras el backend WASM
  emite un string decimal y omite la clave.
- La fila original de `PARCIALES-verificacion-A.md` y las de `docs/dev/plans/**` citan las
  cifras viejas; los planes estan prohibidos en este carril y el informe de parciales es
  historico (no se reescribe).
