---
title: "FIND-723 — decisión de arquitectura: shape numérico del AST JSON (u128 / i64)"
kind: task
status: completed
---

# FIND-723 — shape numérico del AST JSON: `u128` e `i64`

> **Plan:** `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md` §6.b
> **Ruta:** `vanta-arch` (decisión) → `vanta-worker` (implementación, tarea aparte)
> **Carril:** A (no compila nada) · **Appetite:** 3-4h
> **Entregable:** spec + ADR. **Cero código de producto.**
> **ADR:** [ADR-0062](../architecture/adr/ADR-0062-ast-json-numeric-fidelity.md)
> **Origen:** `FIND-156` §Incógnitas 7 (review C1), commit `20c164970`
> **Dependencia:** `FIND-156` ✅ (`20c164970`)

## Contexto

`parse()` (FIND-156) publica el AST del parser como JSON en Python, WASM y TS.
La proyección es la del core: `serde_json::to_string(&Statement)` sobre tipos que
derivan **solo** `Serialize` (`src/query.rs:17,36,49,60,67,80`). Dos familias de
valores viajan como número JSON plano y no sobreviven a un decodificador
IEEE-754:

| Familia | Tipo | Campos |
|---|---|---|
| ids de nodo / thread | `u128` | `InsertStatement.node_id`, `UpdateStatement.node_id`, `DeleteStatement.node_id`, `RelateStatement.source_id`, `RelateStatement.target_id`, `InsertMessageStatement.thread_id` (`src/query.rs:39,52,63,70,72,87`) |
| literales enteros | `i64` (`FieldValue::Int`) | `Condition::Relational(_, _, FieldValue)` (`src/query.rs:148`) y los mapas `fields` de `Insert`/`Update` |

**Evidencia ya medida (no re-derivada):** `vantadb-wasm/src/lib.rs:1865-1874`
serializa a texto y decodifica con `js_sys::JSON::parse` → todo número es double.
Probado en FIND-156: `2^53+1` → `2^53`; `DELETE NODE#123…` →
`1.2345678901234569e+23`. Python es exacto (`json.loads`,
`docs/api/PYTHON_SDK.md:960-962`).

**Qué está guardado hoy, exactamente** (corregido tras la review P2-01, ver §Review
C1): la mitad `Int` está documentada en 4 sitios y fijada por
`vantadb-ts/src/__tests__/find156.test.ts:103-115`, que **solo** asertan un
literal `Int`. La mitad **ids no está fijada en ningún test** —
`rg -n 'node_id|thread_id|source_id' vantadb-ts/src/__tests__/find156.test.ts` →
**0 hits**, y el test Python solo comprueba que la clave `InsertMessage` exista
(`vantadb-python/tests/test_find156_ast_json.py:88-91`). La corrupción de ids está
medida y escrita, pero nada la cazaría si se mueve.

## Spec — decisiones (gate spec-first)

| # | Decisión | Alternativas descartadas | Evidencia |
|---|---|---|---|
| 1 | Los **6 campos `u128`** serializan como **string decimal** vía `#[serde(serialize_with = "crate::sdk::types::u128_serde::serialize")]` | Dejarlos número (O2); emitir ambos (O4); cambiar el tipo del id (O5); `serde_json/arbitrary_precision` (O6); accessor de texto crudo (O7); newtype `NodeId` (O8) | `u128_serde::serialize` emite `serialize_str(&val.to_string())` (`src/sdk/types.rs:42-47`); `pub(crate) mod types` (`src/sdk/mod.rs:13`) tras `pub mod sdk;` incondicional (`src/lib.rs:140`) → sin feature gate ni `pub` nuevo. El mismo path ya compila in-tree: `src/graphrag/pipeline.rs:26,35,37` |
| 2 | El tipo es **siempre string**, nunca número por debajo de 2^53 | String condicional al valor | Un tipo que depende del valor obliga a cada consumidor a ramificar, y la rama es el punto de fallo silencioso (nadie escribe el guard). Además sería el único sitio del producto donde un `u128` a veces es número: API-01 ya los emite string en todo el wire |
| 3 | `i64` (`Int`) **sigue siendo número**, con el límite fijado por el test que ya existe | Uniformar todo a string (O3) | `FieldValue` deriva `Serialize + Deserialize` (`src/node/field.rs:6`) y es el mapa de campos de `UnifiedNode`, que `src/storage/` persiste (postcard en `src/storage/`, `src/wal.rs`; `src/storage/engine/insert.rs:8,1189-1192`) → cambiar su serde es una decisión de **formato on-disk** (ADR-0060), no una edición del AST. Además es el tipo de los metadatos de `QueryResult`: tocaría los 4 bindings a la vez |
| 4 | Un id debe ser **bit-exacto para ser usable** (vuelve a `delete`/`relate`); un `Int` solo necesita ser **legible** (es el input del caller reflejado) | Tratarlos igual | Truncar un id corrompe una clave y no falla; truncar un eco de literal reporta mal un valor que el caller ya tiene. Nota: `Statement` no deriva `Deserialize`, así que un AST nunca se reingesta — el riesgo es el canal de tokens, no un round-trip |
| 5 | Se hace **ahora, en 0.10.0** | Esperar a 1.0 y documentar el límite | `git tag --contains 20c164970` vacío; último tag `v0.9.0` (2026-10-20); `[workspace.package] version = "0.9.0"` (`Cargo.toml:819-820`) → **la superficie `parse()` nunca se publicó**. Coste de migración = 0, y ningún test del árbol se rompe. Después de 1.0 es breaking bajo Hyrum |
| 6 | La cobertura de ids es **cobertura nueva**, no partir un test | "Partir el test de find156" | Ya corregido: ese test solo cubre `Int` (0 hits de `node_id`/`thread_id`/`source_id`). El test nuevo cubre los 6 campos, uno por variante: `Insert`/`Update`/`Delete` (`node_id`), `Relate` (`source_id`+`target_id`), `InsertMessage` (`thread_id`) |
| 7 | Sin símbolo público nuevo → el golden `tests/api/public-api.txt` **no debe cambiar** | Newtype `NodeId` (O8) | Un atributo serde no es una firma: hoy el golden lista `pub vantadb::query::*Statement::node_id: u128` en 6 líneas (5128, 5148, 5167, 5278, 5279, 5370). Si difea, se filtró un `pub` |
| 8 | Python **no pierde exactitud**: su trabajo es de pin, no de caveat | "quitar el caveat de Python" | `PYTHON_SDK.md:960-962` ya dice que la proyección es exacta en esta plataforma (`json.loads`), y `vantadb-python/src/lib.rs:2244-2246` también. Lo que falta es el pin del nuevo tipo en `test_find156_ast_json.py:88-91` (`thread_id` pasa a `"1"`) |

## Investigación — cómo lo resuelven las APIs conocidas

Digest (≤500 palabras), 4 fuentes verificadas con `webfetch` el 2026-11-24:

- **RFC 8259 §6 (el estándar JSON):** las implementaciones pueden limitar la
  precisión; "numbers that are integers and are in the range
  [-(2\*\*53)+1, (2\*\*53)-1] are interoperable in the sense that
  implementations will agree exactly on their numeric values". Fuente:
  <https://www.rfc-editor.org/rfc/rfc8259.html> ✅
- **ProtoJSON (Google):** `int64`/`uint64` → **string decimal**; `int32`/`uint32`
  → número. Justificación textual: *"the json.org spec does not speak to the
  intended precision limits of numbers. Many implementations … interpret all
  numbers as binary64 … and are silently lossy if a number is an integer larger
  than 2\*\*53"*. Fuente: <https://protobuf.dev/programming-guides/json/> ✅
- **ECMAScript / MDN:** `Number.MAX_SAFE_INTEGER = 9007199254740991` (2\*\*53−1);
  "for larger integers, consider using BigInt". Fuente:
  <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/MAX_SAFE_INTEGER> ✅
- **Discord:** *"Because Snowflake IDs are up to 64 bits in size (e.g. a
  uint64), they are always returned as strings in the HTTP API to prevent
  integer overflows in some languages"*. Fuente:
  <https://docs.discord.com/developers/reference> ✅

**Lectura:** los tres productores con ids opacos grandes (protobuf, Discord,
Twitter) eligen string; el estándar pone la frontera en 2^53; y el patrón "dual"
(Twitter `id_str` + `id`) solo se paga cuando ya hay consumidores rompiéndose.
VantaDB no los tiene en `parse()`, así que el dual es coste sin beneficio
(decisión 5).

## Impacto mapeado (Regla 0)

**Archivos leídos completos / por sección antes de decidir:**

| Archivo | Qué se leyó | Por qué |
|---|---|---|
| `src/query.rs:1-155` | `Statement` + 5 structs con `u128`, `Condition`, `Traversal` | inventario exacto de campos afectados (6; `Traversal` no tiene ids) |
| `src/node/field.rs:1-34` | `FieldValue` (derive `Serialize, Deserialize`) | por qué `Int` no entra en esta decisión |
| `src/sdk/types.rs:39-85,123-140` | `u128_serde` (precedente API-01), `Value::Int(i64)` | reuso sin símbolo nuevo |
| `src/sdk/mod.rs:13`, `src/lib.rs:140` | `pub(crate) mod types` / `pub mod sdk` incondicional | que el path resuelve sin feature gate |
| `src/graphrag/pipeline.rs:26,35,37` | `#[serde(with = "crate::sdk::types::u128_serde")]` | precedente in-tree que prueba que compila |
| `src/parser/grammar.rs:404-428` | `parse_insert` (`INSERT NODE#<u128>`) | IQL concreta para el test de proyección |
| `src/parser/mod.rs:1559-1576,1717-1722,2020-2034` | `test_ast_json_projection_shape` y los otros 3 `to_value(&stmt)` | qué está pinneado y qué no se rompe |
| `vantadb-wasm/src/lib.rs:1858-1874`, `src/vantadb_wasm.d.ts:144-146` | `parse` + `js_sys::JSON::parse` + caveat | el punto de pérdida |
| `vantadb-python/src/lib.rs:2242-2286`, `tests/test_find156_ast_json.py:88-91` | `parse` + `json.loads` + test de variantes | decisión 8 |
| `vantadb-ts/src/types.ts:1-55`, `vantadb.ts:160-168` | `NodeId`, `AstJson`, caveat, y el throw de entrada | la asimetría entrada/salida |
| `tests/api/public-api.txt` | 6 líneas de `query::*Statement::node_id` | verificar que un atributo no lo mueve |
| `docs/api/IQL.md:512-559`, `PYTHON_SDK.md:955-966`, `TS_SDK.md:550-557`, `VERSIONING.md:54-55`, `BINDINGS_NAMESPACES.md:48` | superficies documentadas | tabla de migración |
| `.opencode/rules/api-contract.md`, `js-ecosystem.md` | R-1/R-8; bindings | reglas aplicables |

**Referencias hacia dentro:** los 3 bindings que exponen `parse()` (TS delega en
WASM), `docs/api/IQL.md` §AST JSON, `TS_SDK.md`, `PYTHON_SDK.md`,
`BINDINGS_NAMESPACES.md`, el golden `tests/api/public-api.txt`, el snapshot
`test_ast_json_projection_shape` y los tests de binding de FIND-156 en Python y
TS. `vantadb-node` **no** expone `parse()` (FIND-156 §Incógnitas 6) → sin cambio.

**Veredicto de impacto:** superficie pública de 3 bindings + 1 golden + 2 tests +
4 docs. Breaking para quien hoy parsee el AST desde JS/TS, **pero sin
consumidores publicados** y sin ningún test que se rompa (decisión 5). El riesgo
real no es la migración: es el landing parcial (atributos sin `.d.ts` ni
`IQL.md`) y la falta de guard estructural sobre los 6 campos.

## Steps

| # | Step | Verify | Resultado |
|---|---|---|---|
| 1 | Inventario del blast radius | `rg -n 'u128' src/query.rs` → 6 hits (39,52,63,70,72,87) + lectura de los 14 archivos de §Impacto | ✅ |
| 2 | Prior art con URLs verificadas | 4 URLs resueltas con `webfetch` | ✅ 4/4 |
| 3 | `ADR-0062` (MADR: contexto, 8 opciones, decisión, consecuencias ±, migración de 11 artefactos, verificación) | los 6 `rg -c` del §Contrato | ✅ 1/1/1/1/1 + 8 opciones |
| 4 | Tabla de migración de 11 artefactos | revisión contra §Impacto | ✅ |
| 5 | Gates de docs | `check-docs` / `check-links` / `gen-index --write` | ✅ (ver §Verificación: `check-docs` rojo solo por archivo ajeno) |
| 6 | Review P2-01 (`vanta-review`, contexto fresco) + commit local | ver §Review | ✅ ronda 1 🔴 → corregido → ronda 2 ✅ (payload en §Review) |

## Migración (para la tarea de implementación)

| # | Artefacto | Hoy | Después | Consecuencia si se omite |
|---|---|---|---|---|
| 1 | `src/query.rs` (6 campos) | `node_id: u128` → número | `#[serde(serialize_with = "crate::sdk::types::u128_serde::serialize")]` → string decimal | el ADR queda inerte |
| 2 | `src/parser/mod.rs::tests::test_ast_json_projection_shape` (1559-1576) | sin caso de id; `Int == 28` intacto | **añadir** el test por variante (6 campos) | shape sin pin; el 7º `u128` regresa en silencio. Los otros 3 `to_value(&stmt)` (1719, 2025, 2031) no asertan ids → no se rompen |
| 3 | `tests/api/public-api.txt` (golden) | 6 líneas `node_id: u128` (5128, 5148, 5167, 5278, 5279, 5370) | **sin diff** | si difea, se filtró un `pub`: revertir |
| 4 | `docs/api/IQL.md` §AST JSON (512-559) | sin ejemplo con id; `:523` dice "(`i64`, exact)" sin matiz | regla de ids string + ejemplo `Insert`/`Delete`; matizar `:523` (exacto en Rust/Python, lossy por JS sobre 2^53) | doc contra código, y una inexactitud que se queda |
| 5 | `vantadb-python/tests/test_find156_ast_json.py:88-91` | `TO THREAD#1` solo comprueba que la clave exista | asertar `ast["InsertMessage"]["thread_id"] == "1"` | el pin de Python no se mueve con el shape |
| 6 | `docs/api/PYTHON_SDK.md:960-962` | ya dice que la proyección es exacta | **sin retracción** (era correcto); comprobar que ningún ejemplo muestre id numérico | — (fila de verificación) |
| 7 | `vantadb-wasm/src/lib.rs:1860-1862` + `src/vantadb_wasm.d.ts:144-146` + rebuild `pkg/` | ids **y** `Int` pierden | acotar a `Int` | el `.d.ts` publicado conserva un caveat falso |
| 8 | `vantadb-ts/src/__tests__/find156.test.ts` | fija el límite de `Int` (103-115); **sin test de ids** | **añadir** aserción de id (string + decimal exacto); dejar la de `Int` intacta | la mitad arreglada queda sin test, en el fichero que ya documenta el límite |
| 9 | `vantadb-ts/src/types.ts:48-54` | el caveat cubre ids **e** `Int` | acotar a `Int` | el tipo miente sobre los ids |
| 10 | `docs/api/TS_SDK.md:551-555` | ids e `Int` > 2^53 pierden | acotar a `Int` | doc publicado con caveat falso |
| 11 | `docs/api/BINDINGS_NAMESPACES.md:48` | regla "decimal strings (u128 > 2^53)" | **sin edición** — la regla sigue siendo cierta y ahora el AST la cumple | — (fila de verificación) |

`vantadb-node`: n/a (no expone `parse`).

## Contrato

Cada línea es un gate falsable (los 5 primeros son el contrato MADR del ADR):

```bash
A=docs/dev/architecture/adr/ADR-0062-ast-json-numeric-fidelity.md
rg -c '^## Context and problem statement$' "$A"   # 1
rg -c '^## Considered options$'              "$A"   # 1
rg -c '^## Decision outcome$'                "$A"   # 1
rg -c '^### Positive consequences$'          "$A"   # 1
rg -c '^### Negative consequences$'          "$A"   # 1
rg -c '^### O[0-9]+ '                        "$A"   # 8 — no es un ADR de una sola opción
node scripts/docs/check-docs.mjs
node scripts/docs/check-links.mjs
node scripts/docs/gen-index.mjs --write
```

## Verificación medida

| Comando | Resultado |
|---|---|
| 5 × `rg -c` de secciones MADR sobre `$A` | ✅ 1,1,1,1,1 |
| `rg -c '^### O[0-9]+ ' $A` | ✅ 8 (O1-O8) |
| `node scripts/docs/gen-index.mjs --write` | ✅ exit 0 · ADR-0062 registrado en `docs/index.md` y `adr/README.md` (+1 ADR) |
| `node scripts/docs/check-links.mjs` | ✅ exit 0 (0 broken) |
| `node scripts/docs/check-docs.mjs` | ⚠️ exit 1 — **causa única y ajena**: `docs/dev/tasks/FIND-724.md` (untracked, carril R paralelo, sin frontmatter). Ni el ADR ni este fichero aparecen en el informe de gating; el resto del árbol está limpio |
| `npx markdownlint-cli2 --config .github/markdownlint-cli2.yaml <ADR>` | ✅ 0 issues (el config del repo ignora `docs/dev/tasks/**`) |

**Corrección tras review (C2):** la primera versión de esta tabla registraba
`rg -n -e u128 -e '2\^53' -e precision ADR-0062` como ✅. Ese comando **no
corría** (exit 2: no existe la ruta relativa `ADR-0062`), y aunque corriera no
puede contar secciones ni opciones. Sustituido por los 6 `rg -c` de arriba, que
sí fallan si falta una sección o sobra/mengua el set de opciones.

## Invariantes

- **No se toca código de producto, tests ni bindings** en esta tarea.
- El shape del AST **no se redefine en ningún binding**: sale de `Serialize` del
  core (R-8 de `api-contract.md`).
- Ningún símbolo `pub` nuevo: `u128_serde` queda `pub(crate)` y el golden
  `public-api.txt` byte-idéntico.
- El tipo de un campo del AST **nunca depende del valor** (siempre string).
- El golden y el snapshot no se "actualizan para que pase": el golden no debe
  cambiar; el snapshot gana coverage nueva sin perder la de `Int`.
- Los 6 campos `u128` tienen guard de test **por variante**: el compilador no
  avisa si se olvida el atributo en el séptimo.

## Incógnitas / deuda

1. **R2 del ADR — `Int` > 2^53 sigue lossy en JS.** El arreglo caro (uniformar
   `FieldValue` a string) es una decisión de formato on-disk (ADR-0060) + los
   metadatos de `QueryResult` en 4 bindings. El arreglo **barato** es O7: un
   accessor de texto crudo (`parse_ast_json() -> str`, ≈5 líneas por binding, sin
   cambio de shape ni golden) — **no está bloqueado** por la decisión on-disk y
   es la mitigación recomendada. Candidato a fila `FIND-*` propia.
2. **`vantadb-node` no expone `parse()`** (gap de paridad wasm-vs-native, ya
   anotado en FIND-156 §Incógnitas 6). Esta decisión no lo cierra; cuando se
   implemente, debe nacer con el shape de este ADR.
3. **Coste por binding** (el pre-mortem lo pedía explícito): Python = 0 código
   (1 aserción de test), WASM = 0 código (docstring + `.d.ts` + rebuild `pkg/`),
   TS = 0 código de producción (caveat en `types.ts` + 1 aserción de test),
   Node = n/a. El coste real está en docs y en recompilar el `pkg/` de WASM.
4. **Release notes:** al ser superficie no publicada, no requiere nota de
   migración; conviene que 0.10.0 lo mencione como shape inicial, no breaking.

## Review

**P2-01 adversarial (agente distinto: `vanta-review`, contexto fresco) — ronda 1:
🔴 CHANGES REQUIRED.** Dirección aprobada (O1 + always-string); el *registro* no
era publicable. Hallazgos y resolución:

| # | Hallazgo | Resolución |
|---|---|---|
| **C1** | "un test fallará cuando se arregle" era falso: `find156.test.ts:103-115` solo fija un `Int`; `rg -n 'node_id\|thread_id\|source_id'` en ese fichero → 0 hits. "Partir el test" no era una instrucción ejecutable | §Contexto y decisión 6 reescritos: la mitad de ids **no está fijada en ningún test**; la cobertura de ids es **nueva** (6 campos, una variante cada uno) y la aserción de `Int` no se toca |
| **C2** | La verificación registrada no reproducía: `rg … ADR-0062` → exit 2 (ruta inexistente), y ese grep no podía contar secciones ni opciones | §Contrato reescrito con 6 `rg -c` sobre la ruta correcta; §Verificación deja constancia del comando roto y por qué el nuevo sí es gate |
| **C3** | El §Review estaba pre-rellenado con un 🟢 APPROVE antes de que la review corriera | Sustituido por este dictamen real (ronda 1 🔴). El patrón queda registrado: un veredicto escrito dentro del contexto que implementa no es un gate (P2-01) |
| **R1** | La fila de Python era un no-op disfrazado: `PYTHON_SDK.md:960-962` ya dice que es exacta, no hay caveat que quitar | Fila 5 = pin en `test_find156_ast_json.py:88-91`; fila 6 = verificación sin retracción; decisión 8 nueva |
| **R2** | `BINDINGS_NAMESPACES.md` no lleva ese caveat; su único hit (`:48`) es la regla de casing, que sigue cierta | Fila 11 = "sin edición (verificación)"; el caveat real está solo en `TS_SDK.md:551-555` |
| **R3** | "comportamiento sin cambios en Python" era falso: el atributo vive en el serde del core, así que el tipo de salida de Python también cambia (`thread_id` → `"1"`) | Verificación reescrita: Python sigue **exacto** y su **tipo cambia**; el pin prueba que el shape se movió, no que el comportamiento sea idéntico |
| **R4** | Sin guard sobre los 6 campos: D1 son 6 atributos a mano y el pin era de un solo caso | D1 exige el test por variante (6 campos); añadido a la fila 2 y a los invariantes |
| **R5** | Faltaban dos opciones: accessor de texto crudo (arreglo barato de `Int`) y newtype `NodeId` | Añadidas como **O7** (rechazada para este ADR, **adoptada como mitigación de R2**) y **O8** (rechazada: diff de `public-api` por una garantía que un test da gratis) |
| O1 | El argumento "un decoder no puede escribir schema" era el más débil (existe `oneOf`) | Reemplazado por evidencia citable: `VERSIONING.md:54-55` (regla normativa que el AST viola), `vantadb.ts:160-168` (la entrada ya lanza con ids no seguros → simetría de salida), `graphrag/pipeline.rs:26` (precedente in-tree) |
| O2 | `IQL.md:523` dice "(`i64`, exact)" y sigue siendo falso por JS | Fila 4 incluye matizar `:523` |
| O3 | La línea del caveat en `vantadb_wasm.d.ts` es 144-146, no 148 (148 es el alias `AstJson`) | Corregido en la fila 7 |
| O4 | `status: in-progress` con todos los steps ✅ | `status: completed` |
| O5 | La fila 2 decía "un caso de id" (singular) | Ahora son 5 variantes / 6 campos |

**Ronda 2 — P2-01 adversarial (`vanta-review`, contexto fresco): ✅ APPROVE.**
**Enfoque:** O1 (`u128` ids como string decimal vía `u128_serde`, siempre string,
nunca condicionado al valor; `i64 Int` sigue numérico con el límite del test
existente) + O7 como mitigación declarada de R2 + guard obligatorio por variante
sobre los 6 campos. Aprobado.
**Cómo se probó:** 6 gates del §Contrato re-ejecutados (1,1,1,1,1 secciones + 8
opciones, exit 0); `gen-index --check` exit 0; `check-links` exit 0 (0 broken);
`check-docs` exit 1 con `docs/dev/tasks/FIND-724.md` como **única** entrada de
gating (ajeno, untracked); markdownlint-cli2 0 issues. Verificadas contra el repo
todas las citas nuevas (`query.rs:148`, `sdk/types.rs:123-140`,
`grammar.rs:404-428`, `storage/engine/insert.rs:8,1189-1192` + postcard en
`wal.rs`/`storage`, `TS_SDK.md:551-555`, `api-contract.md:10/52`, `ADR-0060`,
`FIND-156:155,158`). Confirmado que los únicos sitios que serializan `Statement`
son `parser/mod.rs:1565,1719,2025,2031` (ninguno aserta ids) + los 2 bindings:
ningún test del árbol se rompe. Confirmado de forma independiente que
`find156.test.ts` **no** contiene aserción de id alguna y que
`test_find156_ast_json.py:88-91` solo comprueba la clave.
**Veredicto:** ✅ APPROVE. C1/C2/C3 y R1-R5 cerrados con evidencia re-ejecutada.
Nits aplicados: se quitó el numeral "(63→64)" de §Verificación medida (el conteo
del generador no coincide con el de filas: 62→63) y el step 6 ya no pre-registra
el veredicto. Opcional aplicado: nota en `ADR-0062` §Verification sobre
`check-docs` rojo por ficheros untracked ajenos.

## SDP

`SDP:` campaign-executor, progreso, ci-cd-and-automation,
git-workflow-and-versioning, documentation-and-adrs, api-and-interface-design,
deprecation-and-migration, writing-guidelines, writing-plans, documentation-skill.
