---
title: "FIND-710: una política de vector sintético y degradación explícita (absorbe FIND-711 y FIND-712)"
kind: task
description: "Un solo criterio para el vector sintético, limpieza de la marca obsoleta y `degraded` en el resultado de escritura"
---

# FIND-710 — Política única de vector sintético + degradación explícita

> **Esta tarea absorbe `FIND-711` y `FIND-712`.** Los tres residuos tocan el
> mismo fichero (`src/executor.rs`) y el mismo evento (una escritura que degrada);
> en paralelo habrían colisionado sobre `apply_explicit_vector` y sobre la
> variante `ExecutionResult::Write`. Un dueño, un commit, tres alcances.
>
> | ID absorbido | Alcance |
> |---|---|
> | `FIND-711` | `apply_explicit_vector` debe borrar `__vanta_embed_status` |
> | `FIND-712` | `ExecutionResult::Write::degraded: bool` |

## Metadata

- **Plan file:** `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md` (§6, Fase 2, wave 2.1; nota de fusión §3.c)
- **Depende de:** `FIND-485` ✅ (`d0df55c2b`)
- **Fuente:** tres residuos declarados en el cierre de `FIND-485` (§Notas + RESULTADO)
- **Esfuerzo:** ~4h (appetite del plan)
- **Prioridad:** media (corrige un diagnóstico que miente y cierra la clase "degradación sin superficie")
- **Tipo:** Rust (core engine) + superficie SDK/MCP
- **Estado:** ✅ COMPLETED

## Blast Radius

| Dirección | Módulos |
|---|---|
| Callers | `Executor::execute_statement` → `execute_insert` / `execute_insert_message` → `auto_embed_insert` / `auto_embed_message`; `execute_update` / `execute_insert` → `apply_explicit_vector` (2 callers, mismo archivo). `ExecutionResult` tiene **8 consumidores fuera de `src/executor.rs`**: `src/sdk/serialization/conversions.rs`, `src/sdk/api/graph.rs`, `src/server/handlers.rs`, `src/cli_handlers/data.rs`, `src/python.rs`, `src/tui/repl.rs`, `src/physical_plan/scan.rs` (`:161`), `vantadb-mcp/src/handlers/tools.rs`. |
| Callees | `crate::llm::get_embedding_provider`, `EmbeddingProvider::embed`, `UnifiedNode::set_field`, `UnifiedNode::relational` (BTreeMap público), `StorageEngine::insert`/`get`. |
| Implicaciones | **Cambio aditivo en API pública:** `QueryResult::Write` gana `degraded: bool` con `#[serde(default)]`, así que los payloads viejos siguen deserializando (test dedicado). `ExecutionResult::Write` gana el mismo campo: rompe *destructuring exhaustivo* dentro del workspace (10 sitios, todos actualizados) y en crates externos, pero no cambia el comportamiento de lectura. `apply_explicit_vector` ahora muta el `BTreeMap` relacional (una entrada borrada): invisible para el wire, visible para el diagnóstico. Ningún `VantaError` nuevo, ninguna migración, ningún cambio de schema. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/executor.rs:1-760` + `mod tests:825-2400`; `src/sdk/serialization/mod.rs:55-90`; `src/sdk/types/graph.rs:1-130`; `src/sdk/serialization/conversions.rs:100-145` + tests; `src/sdk/api/graph.rs:320-350`; `src/server/handlers.rs:140-165`; `src/cli_handlers/data.rs:385-415`; `src/python.rs:39-50`; `src/llm.rs:40-130`; `vantadb-mcp/src/handlers/tools.rs:1600-1910, 2295-2330, 2725-2765, 5110-5290`; `vantadb-mcp/tests/test_auto_embed.rs`; `tests/auto_embed_insert_mark.rs`; `docs/api/IQL.md:330-430`; `docs/api/EMBEDDINGS.md:40-70`; `tests/sdk_serialization.rs:515-570,740-790`; `tests/query_result_advanced.rs:350-360`.
- **Referencias hacia dentro:** `auto_embed_insert` ← `execute_insert` (1); `auto_embed_message` ← `execute_insert_message` (1); `apply_explicit_vector` ← `execute_insert` + `execute_update` (2). `ExecutionResult` ← 11 consumidores (arriba).
- **Referencias entrantes a los editados:** `tests/api/public_api.rs` (golden `tests/api/public-api.txt` — snapshot de superficie, actualizado); `tests/sdk_serialization.rs`, `tests/proptest_serialization_roundtrip.rs`, `tests/query_result_advanced.rs` (construyen `QueryResult::Write`); `vantadb-python/src/convert.rs` (mapea `QueryResult::Write` a dict/string).
- **Veredicto impacto:** **medio**, contenido. Un enum público gana un campo (aditivo con `default`); los sitios que construyen/destruyen exhaustivamente son finitos y están enumerados por el compilador (verificado en 3 pasadas de `cargo check --all-targets`). No hay cambio de wire, ni de schema, ni de semántica de lectura.

## Spec (SDD)

### Decisión 1 — política de vector sintético (FIND-710): **descartar en ambos caminos**

| Opción | Índice | Búsqueda | Diagnóstico | Riesgo |
|---|---|---|---|---|
| **(a) descartar en ambos** ✅ | solo vecinos reales | el nodo degradado no aparece por vector (sí por texto/filtros) | `__vanta_embed_status = "synthetic"` + `degraded: true` | un `INSERT MESSAGE` sintético deja de tener "algún vector" |
| (b) conservar en ambos + filtro de búsqueda | recibe $n$ vecinos falsos por escritura | exige filtrar `synthetic` en **cada** path de búsqueda (HNSW, híbrida, MCP, SDK, server) | igual | un filtro olvidado = resultados envenenados; y borrar la marca (FIND-711) dejaría sin forma de filtrar |

**Decidido: (a).** La razón es la que ya estaba escrita en `executor.rs` para el
camino `INSERT`: un vector sintético "ensucia el índice con vecinos falsos — peor
que un nodo sin vector". Un vector determinista no es semántico; persistirlo
convierte una degradación visible en un resultado de búsqueda incorrecto. Además
(a) **no cambia el índice**: no hay re-indexado ni migración de las bases que ya
tienen vectores sintéticos de mensajes (los que existan siguen ahí, marcados
`synthetic`, y FIND-711 garantiza que un vector real posterior limpia la marca).
El comentario de divergencia en `auto_embed_message` se elimina: era la
documentación de un bug.

Consecuencia en `auto_embed_message`: el vector ya no se asigna en la rama
sintética (`HAS_VECTOR` queda limpio); se mantiene el `tracing::warn!` y la marca
`synthetic`. `INSERT` y `INSERT MESSAGE` comparten ahora un solo helper implícito
(`mark_embed_status`) y una sola decisión.

### Decisión 2 — limpiar la marca al aplicar vector explícito (FIND-711)

| Opción | Consecuencia |
|---|---|
| **(a) `node.relational.remove(FIELD_EMBED_STATUS)` en `apply_explicit_vector`** ✅ | la marca describe el estado del dato, no su historia: con vector real, no hay degradación que reportar. 1 línea, cubre INSERT-con-VECTOR y UPDATE-SET-VECTOR (función compartida). |
| (b) dejarla y documentarla como "histórica" | el registro afirma "no tengo vector semántico" mientras tiene uno → el diagnóstico miente, que es exactamente el defecto que `FIND-485` vino a cerrar. |

**Decidido: (a).** No hay pérdida de información: el `warn!` sigue en el log y la
contramedida (`UPDATE ... SET VECTOR`) es precisamente el evento que la resuelve.

### Decisión 3 — exponer `degraded` en `ExecutionResult::Write` (FIND-712)

| Opción | Compatibilidad | Coste |
|---|---|---|
| **(a) campo `degraded: bool` en la variante + `#[serde(default)]` en `QueryResult::Write`** ✅ | aditiva en el wire; rompe solo *destructuring exhaustivo* (enumerado por el compilador, 8 sitios fuera del executor + 2 dentro) | 1 campo, 4 superficies (SDK, CLI `--json`, MCP, Python dict) |
| (b) derivarlo en cada consumidor leyendo el campo relacional | cero cambios de tipo | obliga a cada consumidor a conocer `__vanta_embed_status`; el resultado de la escritura no podría decir nada de un nodo que no devuelve (`INSERT MESSAGE` sí, pero `affected_nodes` no) |

**Decidido: (a).** `degraded` es el hecho *de esta escritura*, no un atributo
consultable del nodo: el resultado es el único lugar donde existe. Se propaga a
`QueryResult::Write` (SDK), `vanta-cli query --json`, el dict de
`query_structured` en Python, y a `vantadb-mcp` (`memory_put` como alias de
`fallback`; `memory_query` e `inject_context` en el JSON de respuesta).

**No propagado (declarado en §Deuda):** el server HTTP (`QueryResponse` no tiene
el campo y `src/server/state.rs` queda fuera del scope autorizado), y
`vantadb-node` / `vantadb-ts` / `vantadb-wasm` (sus DTOs no exponen el campo; el
wire serde sí lo lleva y `#[serde(default)]` los mantiene compatibles).

### Decisión 4 — dónde vive la verdad

Se mantiene una sola fuente en el **executor**: el campo relacional decide.
`degraded` es `node.get_field(FIELD_EMBED_STATUS).is_some()` calculado después de
`auto_embed_*` + `apply_explicit_vector`. No hay un segundo booleano que pueda
divergir del registro (Regla 4: una sola fuente de verdad). El campo es
**propiedad del executor**: `__vanta_embed_status` se ignora si llega en los
`fields` de un `INSERT`/`UPDATE` del usuario (un replay export→INSERT no puede
fabricar una degradación inexistente) — review M-06.

**Desviación conocida (review L-09):** en `vantadb-mcp`, `memory_put` emite
`degraded` como alias del `fallback` que devuelve su propio `auto_embed_one`.
Hoy ambos caminos coinciden (sintético ⇒ `Err` ⇒ sin vector), pero es una segunda
derivación del mismo hecho, no la misma fuente. Un test fija la coherencia
(`degraded == fallback` en toda configuración); unificarlo exigiría leer el
registro tras `put` y está declarado en §Deuda.

## Contrato

```
cargo nextest run --profile audit -p vantadb --features remote-inference -E 'test(auto_embed)' --build-jobs 2
cargo nextest run --profile audit -p vantadb-mcp -E 'test(memory_put)' --build-jobs 2
cargo clippy -p vantadb --all-targets --features remote-inference -- -D warnings
```

**Legs obligatorios del cambio** (el primer filtro NO selecciona ninguno de los
tests nuevos — review H-03: `-E 'test(auto_embed)'` lista 9 tests pre-existentes):

```
# (a) + (b) + (c) — los tres tests nuevos
cargo nextest run --profile audit -p vantadb --features remote-inference \
  -E 'test(explicit_vector_clears_stale_embed_status) or test(write_result_reports_degraded) or test(caller_supplied_embed_status_is_ignored)' --build-jobs 2
# (a) paridad con provider sintético real
cargo nextest run --profile audit -p vantadb --features "remote-inference,embed-local" \
  -E 'test(synthetic_vector_policy_is_uniform_across_write_paths)' --build-jobs 2
# (c) superficie CLI end-to-end
cargo nextest run --profile audit -p vantadb --test auto_embed_insert_mark --build-jobs 2
# (c) MCP: degraded == fallback, y degraded:false con vector provisto
cargo nextest run --profile audit -p vantadb-mcp --test test_auto_embed --build-jobs 2
# aditividad del wire
cargo nextest run --profile audit -p vantadb --test sdk_serialization --test proptest_serialization_roundtrip --test query_result_advanced --build-jobs 2
```

Tests que el contrato exige (a/b/c):

| # | Test | Cubre |
|---|---|---|
| a | `synthetic_vector_policy_is_uniform_across_write_paths` (`remote-inference,embed-local`) | mismo provider sintético → `INSERT` y `INSERT MESSAGE` con `!HAS_VECTOR`, `vector.is_none()`, status `synthetic` y `degraded: true` en ambos |
| b | `explicit_vector_clears_stale_embed_status` (`remote-inference`) | `failed` → `UPDATE SET VECTOR` → `HAS_VECTOR` + status ausente |
| c | `write_result_reports_degraded` (`remote-inference`) | `degraded: true` (provider caído) y `degraded: false` (vector explícito) |
| c' | `test_query_result_write_missing_node_id_defaults_none` (`tests/sdk_serialization.rs`) | payload viejo sin `degraded` → `false`, sin fallo de deserialización (aditividad probada) |
| c'' | `auto_embed_insert_mark_visible_via_cli` (`tests/auto_embed_insert_mark.rs`) | el binario real reporta `"degraded": true` en `query --json` de una escritura degradada |
| d | `caller_supplied_embed_status_is_ignored` (`src/executor.rs`) | un `__vanta_embed_status` provisto por el usuario en el `INSERT` no fabrica una degradación (review M-06) |
| e | `put_sin_vector_lleva_flag_y_nunca_error_duro` + `put_vector_provisto_se_respeta_byte_exact` (`vantadb-mcp/tests/test_auto_embed.rs`) | `degraded` presente y `degraded == fallback`; `degraded:false` con vector provisto (review M-04) |

**Legs adicionales ejecutados (cierre):** `--features "remote-inference,embed-local"` para el test (a);
`--test sdk_serialization --test proptest_serialization_roundtrip --test query_result_advanced`;
`cargo check -p vantadb --no-default-features --features fjall` (brazo `unavailable`);
`cargo fmt --all -- --check`; `pwsh dev-tools/ocr-review.ps1`.

## Invariantes de dominio (handoff - MUST)

1. Un `INSERT` nunca falla por culpa del provider (disponibilidad > vector).
2. `VECTOR` explícito es byte-exacto, nunca consulta el provider, y **no** deja marca (y ahora la limpia).
3. **Ningún path de escritura persiste un vector sintético.** Un vector no semántico en el índice es un vecino falso.
4. `__vanta_embed_status` describe el estado **actual** del dato: ausente ⇒ hay vector o nunca se intentó; presente ⇒ no hay vector semántico.
5. `degraded` es derivado del registro, nunca de un estado paralelo; ausencia de `degraded` en un payload ⇒ `false`.
6. Ningún `VantaError` nuevo.

## Deuda técnica (Regla 6)

**Saldo neto: negativo.** Se elimina una divergencia documentada (dos criterios
para un evento), una marca obsoleta que mentía y una inferencia implícita en cada
consumidor. No se añaden dependencias ni abstracciones; el único símbolo nuevo es
un campo `bool`.

## Definition of Done

| Nivel | Gate |
|---|---|
| **Task** | Contrato verde + legs de cierre + tests (a)(b)(c)(c') + docs actualizadas |
| **Commit** | `fix(core): unify the synthetic-vector policy and make degradation explicit (FIND-710/711/712)` — local, sin push |
| **Release** | No aplica (sin bump ni publish) |

## Steps

### Step 1: RED→GREEN (a) — política única
- **Archivos:** `src/executor.rs` (`auto_embed_message`, tests)
- **Acción:** rama sintética de `auto_embed_message` deja de asignar el vector; test de paridad con el mismo provider sintético (feature `embed-local`) que asserta `!HAS_VECTOR` + `vector.is_none()` + `synthetic` en ambos caminos.
- **Verify:** `cargo nextest ... --features "remote-inference,embed-local" -E 'test(synthetic_vector_policy_is_uniform_across_write_paths)'` → 1/1 ✅

### Step 2: GREEN (b) — limpiar la marca
- **Archivos:** `src/executor.rs` (`apply_explicit_vector`)
- **Acción:** `node.relational.remove(FIELD_EMBED_STATUS)` al aplicar vector; test `failed → UPDATE SET VECTOR → sin marca`.
- **Verify:** `-E 'test(explicit_vector_clears_stale_embed_status)'` → 1/1 ✅

### Step 3: GREEN (c) — `degraded`
- **Archivos:** `src/executor.rs` (enum + 5 constructores), `src/sdk/types/graph.rs`, `src/sdk/serialization/conversions.rs`, `src/sdk/api/graph.rs`, `src/cli_handlers/data.rs`, `src/python.rs` (sin cambio: usa `..`), `vantadb-python/src/convert.rs`, `vantadb-mcp/src/handlers/tools.rs`
- **Acción:** campo aditivo con `#[serde(default)]`; propagación a SDK/CLI/MCP; `tests/api/public-api.txt` actualizado (3 líneas: `ExecutionResult` + los dos bloques de `QueryResult`); `tests/query_result_advanced.rs` + snapshot insta.
- **Verify:** `cargo check -p vantadb --all-targets --features remote-inference` ✅; `-E 'test(write_result_reports_degraded)'` 1/1 ✅; `--test sdk_serialization --test proptest_serialization_roundtrip --test query_result_advanced` 59/59 ✅

### Step 4: Docs + cierre
- **Archivos:** `docs/api/IQL.md`, `docs/api/EMBEDDINGS.md`, este task file
- **Acción:** documentar la política única, la limpieza de la marca y `degraded` en cada superficie; eliminar la frase de "divergencia".
- **Verify:** contrato completo + `ocr-review` sin Critical/High + review P2-01.

## Review (GATE — agente distinto, P2-01)

- **Tier:** adversarial (`src/sdk/**` + `docs/api/**` en el diff).
- **Revisor:** `vanta-review`, sesión fresca `ses_df056a374ffe6bt179dF5XXiHH` (contexto distinto del implementador). Hunk set pineado por `git hash-object` mientras otra tarea (`FIND-156`) escribía en el mismo árbol.
- **Veredicto 1ª pasada:** ❌ **changes-required** — 3 high, 5 medium, 4 low, 2 nit. Todos resueltos:

| # | Sev | Hallazgo | Resolución |
|---|-----|----------|-----------|
| H-01 | high | `public-api.txt` lista `QueryResult` en **dos** paths (`vantadb::sdk::QueryResult` y re-export `vantadb::QueryResult`); `degraded` faltaba en el segundo → el job `Public API Snapshot` compara 11157 vs 11159 | tercera línea añadida al bloque raíz (orden alfabético) |
| H-02 | high | el task file afirmaba propagación al **server HTTP**, pero `handlers.rs` la descarta con `..` y `QueryResponse` no tiene el campo | claim eliminado de §Decisión 3/§Step 3; HTTP movido a §Deuda (requiere `src/server/state.rs`, fuera de scope) |
| H-03 | high | el contrato `-E 'test(auto_embed)'` **no selecciona** ninguno de los tests nuevos: revertir la política dejaba el gate verde | §Contrato ampliado con los 5 legs obligatorios (a/b/c, paridad `embed-local`, CLI, MCP, aditividad) |
| M-04 | medium | `degraded` en MCP sin ninguna aserción | `degraded` presente + `degraded == fallback` + `degraded:false` con vector provisto (`vantadb-mcp/tests/test_auto_embed.rs`) |
| M-05 | medium | `Estado: ✅ COMPLETED` con §Review prometiendo un veredicto inexistente (orden P2-01 invertido) | veredicto escrito acá; el estado se sostiene solo después de esta tabla |
| M-06 | medium | un `INSERT ... { __vanta_embed_status: "failed" }` del usuario producía `degraded: true` sin intento de embed (el campo reservado es un identificador IQL legal) | el executor ignora `FIELD_EMBED_STATUS` en los `fields` de `INSERT` y `UPDATE`; test `caller_supplied_embed_status_is_ignored` |
| M-07 | medium | `docs/api/EMBEDDED_SDK.md` imprimía `QueryResult::Write` sin `degraded` | snippet actualizado |
| M-08 | medium | `tests/auto_embed_insert_mark.rs` fuera de la lista autorizada y añadido durante el review; segunda tarea escribiendo en el mismo árbol | archivo autorizado en §Artefactos; el commit usa pathspec explícito (nunca `git add -A`) |
| L-09 | medium→bajo | `memory_put` deriva `degraded` de su propio `fallback`, no del registro | anotado en §Decisión 4 + test de coherencia; unificación en §Deuda |
| L-10 | low | docstring de `query_structured` sin `degraded` | docstring actualizado |
| L-11 | low | link rustdoc a un item privado (`[FIELD_EMBED_STATUS]` → dead link) | code span |
| L-12 | low | sustitución `—` → `-` en prosa reescrita y en una frase ajena | em dashes restaurados |
| N-13 | nit | "11 consumidores" y lista de 8 | inventario corregido: 8 fuera de `executor.rs` |
| N-14 | nit | falta un test de `degraded:false` con embed real exitoso | aceptado como deuda (requiere sesión ONNX real) |

- **Lo que el revisor confirmó bien:** `degraded` derivado del campo (sin booleano paralelo); `UPDATE`/`DELETE`/`RELATE` en `false`; el test de paridad **no es vacuo** (4 aserciones por camino, incluye el status exacto); aditividad probada con un payload que omite `node_id` y `degraded`; `fallback` preservado; política (a) defendible y (b) rechazada por la razón correcta.
- **Alternativa evaluada por el revisor:** "conservar el vector en un campo lateral no indexado" → descartada (no existe variante no-indexable en `VectorRepresentations`; el vector es ruido por construcción).
- **Veredicto final:** ✅ **approve** — los 14 hallazgos resueltos en el árbol; verificación re-ejecutada (contrato + 5 legs + clippy ×3 + fmt + `check --no-default-features` + docs gates).

## Context Save Point

- **Última acción:** implementación + verificación + docs.
- **Evidencia mecánica:** ver `## Verification` del RESULTADO (todas las filas son comandos ejecutados, no auto-reporte).
- **Artefactos:** `src/executor.rs`, `src/sdk/types/graph.rs`, `src/sdk/serialization/{mod,conversions}.rs`, `src/sdk/api/graph.rs`, `src/server/handlers.rs`, `src/cli_handlers/data.rs`, `vantadb-python/src/convert.rs`, `vantadb-python/src/lib.rs`, `vantadb-mcp/src/handlers/tools.rs`, `vantadb-mcp/tests/test_auto_embed.rs`, `tests/sdk_serialization.rs`, `tests/proptest_serialization_roundtrip.rs`, `tests/query_result_advanced.rs` + snapshot, `tests/auto_embed_insert_mark.rs`, `tests/api/public-api.txt`, `docs/api/IQL.md`, `docs/api/EMBEDDINGS.md`, `docs/api/EMBEDDED_SDK.md`.
- **Próxima acción:** ninguna. Diferidos → §Deuda.

## Deuda / Diferido

- **Server HTTP:** `QueryResponse` (`src/server/state.rs`) no expone `degraded` y `src/server/handlers.rs:149` lo descarta con `..`. Añadirlo es aditivo (4 líneas) pero `state.rs` estaba fuera del scope autorizado. `docs/api/IQL.md` / `EMBEDDINGS.md` documentan explícitamente las superficies que sí lo emiten.
- **Bindings JS:** `vantadb-node` / `vantadb-ts` / `vantadb-wasm` no exponen `degraded` en sus DTOs (el wire serde sí lo lleva y `#[serde(default)]` los mantiene compatibles). Coste bajo, fuera del alcance de 4h.
- **`docs/api/COMPATIBILITY.md` / `UPGRADE.md`:** el campo nuevo en `QueryResult::Write` merece una fila en la tabla de cambios de superficie del próximo release; esos ficheros no estaban en el scope autorizado.
- **Drift pre-existente en `tests/api/public-api.txt` (corregido):** el snapshot no contenía `pub fn vantadb::config::resolve_local_model_path` (`src/config.rs:620`), así que el job `Public API Snapshot` ya fallaba antes de este cambio. Se regeneró el snapshot (`VANTADB_PUBLIC_API_UPDATE=1`) y el diff resultante son **exactamente 4 inserciones**: las 3 líneas de `degraded` más esa línea ajena. El test queda verde; la línea ajena va declarada en el commit, no silenciada.
- **`memory_put.degraded` derivado de `fallback`** (no del registro): unificarlo exige leer el `MemoryRecord` tras `put` y leer el campo relacional, que la API del SDK filtra de `metadata`. Test de coherencia lo cubre mientras tanto.
- **`degraded: false` con embed real exitoso** no tiene test (necesita una sesión ONNX real); el inverso sí está cubierto.
- **Vectores sintéticos ya persistidos por `INSERT MESSAGE`** (antes de este cambio) siguen en el índice marcados `synthetic`. No se limpian: borrarlos requeriría un re-index destructivo. Un `UPDATE ... SET VECTOR` sobre cada uno los repara (y limpia la marca).

## Notas

- `ponytail:` `degraded` se **deriva** del campo relacional en vez de plumbear un `bool` por `auto_embed_*` — menos firmas, cero riesgo de divergencia. Techo: una escritura que quiera reportar "degradado por otra razón" necesitaría el booleano explícito.
- `ponytail:` la rama sintética deja de usar el `Vec<f32>` que devolvió el provider; se descarta en el sitio (sin `Vec::clear` ni buffer reutilizado). El ahorro de una asignación no paga la complejidad de un pool en un path degradado.
- El `tracing::warn!` se mantiene en las tres degradaciones: log para el operador, campo/marca para el dato.
