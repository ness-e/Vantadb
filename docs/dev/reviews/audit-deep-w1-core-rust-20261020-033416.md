---
title: "Deep audit W1 - Core Rust src/"
kind: review
status: active
description: "Auditoria read-only del core Rust (271 ficheros .rs): unsafe, disciplina de unwrap, #[non_exhaustive], dead code, complejidad, hotspots y Findings de la architecture de modulos."
tags: [audit, reviews, core-engine]
---

# Deep audit W1 - Core Rust `src/`

> **Wave:** W1 · **Fecha:** 2026-10-20 · **Modo:** read-only
> **Especificacion:** `docs/dev/plans/2026-10-20-revision-profunda-integral-vantadb.md` §4 (W1)
> **Ejecucion:** subagentes `vanta-arch` + `vanta-engine` + `vanta-audit` no disponibles (sin tool
> `task` en la sesion). El orquestador ejecuto los tres sub-scope como secciones de este reporte.

## Scope

**Cubierto:** `src/` - 271 ficheros `.rs` en 23 subdirectorios (`agentic`, `api`, `backends`, `bin`,
`cli_handlers`, `entity`, `graphrag`, `hardware`, `index`, `metrics`, `node`, `parser`,
`physical_plan`, `sdk`, `serialization`, `server`, `shred`, `skills`, `storage`, `testing`, `tui`,
`utils`, `vector`, `wiki`) + 56 ficheros `.rs` en la raiz de `src/`.

**Sub-scope W1a (arquitectura / coupling / dead code)** - seccion "Arquitectura" y "Dead code".
**Sub-scope W1b (indices / algoritmos)** - seccion "Indices".
**Sub-scope W1c (correctness / seguridad)** - seccion "Correctness y seguridad".

**Excluido con motivo:** `src/**/tests*.rs` y los modulos `#[cfg(test)]` se tratan en W6; `fuzz/` no
es miembro del workspace (verificado en `Cargo.toml` members) y se trata en W6/W8.

## Metodologia

- `codebase-memory-mcp_get_architecture` (project `C-Users-Eros-VantaDB-Proyect-VantaDB`, 50 365
  nodos / 188 831 aristas) para estructura, capas, clusters (Leiden) y hotspots.
- `codebase-memory-mcp_query_graph` para complejidad (`complexity`, `cognitive`, `loop_depth`,
  `linear_scan_in_loop`) y para candidatos a dead code.
- Analisis estatico con PowerShell sobre `Get-ChildItem src -Recurse -Filter *.rs`:
  inventario de `unsafe`, separacion prod/test de `unwrap()`/`expect()`, `panic!`, `todo!`,
  `#[non_exhaustive]`.
- Verificacion manual de cada hallazgo con `Read`/`Select-String` sobre la linea exacta.
- **Sin ejecucion de cargo**: `cargo`/`rustc` **no estan en PATH** en este entorno (ver bloque
  BLOCKED en el reporte de sintesis). Ninguna afirmacion de este reporte depende de un runtime.

## Hallazgos

| ID | Severidad | Hallazgo | Archivo:línea | Evidencia | Fix propuesto | Esfuerzo |
|---|---|---|---|---|---|---|
| `FIND-396` | Medium | `unsafe` sin invariante `// SAFETY:` documentado en codigo de produccion | `src/storage/engine/txn.rs:612` | `Select-String` sobre `src/**.rs` de `unsafe\s*\{|unsafe\s+fn|unsafe\s+impl` = **83 ocurrencias en 24 ficheros**. Al cruzar con la presencia de la cadena `SAFETY` en el mismo fichero, solo **2 ficheros** quedan sin invariante documentado: `src/storage/engine/txn.rs` (produccion) y `src/index/search/tests.rs` (solo tests). En `txn.rs:612`: `let (_, u64_slice, _) = unsafe { slice.align_to::<u64>() };` precedido por `let slice = &vstore.mmap_bytes()[vec_start..vec_end as usize];` (linea 611) sin ninguna nota que documente por que el slice es valido. `AGENTS.md` Regla 4: "`unsafe` sin `// SAFETY:` ... Exigir documentacion del invariante de seguridad". | Anadir el bloque `// SAFETY:` con el invariante explicito (longitud del slice, alineacion, `align_to` es safe pero la longitud del prefijo de bytes no alineados debe quedar documentada porque el check `u64_slice.len() != header.vector_len` (linea 613) es la unica garantia de que no se leen bytes de mas). | 1h |
| `W1-02` | Info | Disciplina de `unwrap()`: Excelente en produccion (14 ocurrencias reales), el volumen bruto de 2 098 es casi todo codigo de test | `src/shred/mod.rs` (4), `src/sdk/search/mod.rs` (3), `src/sdk/api/namespaces.rs` (3), `src/index/serialize/bytes.rs` (1), `src/llm.rs` (1), `src/server/state.rs` (1) | Conteo bruto en `src/`: 2 098 `.unwrap()` + 1 521 `.expect()` + 168 `panic!` + 0 `todo!`/`unimplemented!`. Separando por el primer `#[cfg(test)]` de cada fichero, el total en rutas de produccion es **~14 `unwrap()`**: el resto (2 084) esta en modulos `#[cfg(test)]` o en ficheros que son modulos de test incluidos desde un `#[cfg(test)] mod tests;` (p.ej. `src/storage/engine/tests/ops.rs`, `src/entity/checker_tests.rs`, `src/cli_server_auth_tests.rs`). Esto **cumple** `RULES.md` ("sin `unwrap()` en produccion con datos de usuario o E/S"). | Sin accion. Se documenta para evitar que un reporte futuro cite el numero bruto como hallazgo. | - |
| `W1-03` | Info | `#[non_exhaustive]` correctamente aplicado a los enums publicos que pueden crecer | `src/error.rs:136-137`, `src/entity/checker.rs:35,46,57`, `src/entity/linking.rs:42,70,261,286`, `src/sdk/merge.rs:41,59`, `src/sdk/api/sharing.rs:52`, `src/sdk/serialization/vector_types.rs:235`, `src/sdk/types/record.rs:91,140`, `src/server/jwt.rs:33`, `src/wiki/state.rs:16` | 17 usos sobre enums publicos. `src/error.rs:137` es `pub enum Error {` bajo `#[non_exhaustive]`. Cubre el caso que `AGENTS.md` Regla 4 marca (P2-6, ya resuelto). | Sin accion. | - |
| `W1-04` | Info | Sin `todo!()` ni `unimplemented!()` en produccion | `src/**` | `Select-String` de `todo!\(|unimplemented!\(` sobre los 271 ficheros = 0 ocurrencias. No hay stubs de funcionalidad declarados como pendientes. | Sin accion. | - |
| `W1-05` | Info | 5 ficheros de `src/` superan 1 300 lineas y 4 superan 2 000 (riesgo de mantenibilidad, no de correctitud) | `src/sdk/search/tests.rs` (2 567), `src/sdk/api/memory.rs` (2 421), `src/config.rs` (2 364), `src/sdk/serialization/mod.rs` (2 072), `src/wal.rs` (1 870), `src/sdk/api.rs` (1 811), `src/parser/mod.rs` (1 714), `src/executor.rs` (1 653), `src/server/handlers.rs` (1 581) | Conteo de lineas por fichero. `documentation-skill` §5 (Nygard) aplica al codigo: un modulo de 2 400 lineas con API publica y logica de negocio concentrates el riesgo de drift. Nota: `src/sdk/search/tests.rs` (2 567) es el fichero mas grande y es **puro test**, lo que infla la metricica de `src/`. | Partir `src/config.rs` (2 364) y `src/sdk/api/memory.rs` (2 421) por subdominio. Es deuda de mantenibilidad, no bloquea v0.9.0. | 2-3d |
| `W1-06` | Info | Dead code: no se confirma ninguna funcion de produccion realmente muerta | varios | `codebase-memory-mcp_query_graph` devolvio 40 candidatos sin `CALLS` entrantes ni `USAGE`; **todos son funciones de test** (`test_*`, `miri_*`, `*_is_sane`, `*_rejects_*`) o funciones feature-gated verificadas por grep. Los 3 candidatos no-test se verificaron a mano: `src/cli_handlers/search.rs:319 hit_to_json` -> 3 call-sites (104, 228, 343); `src/server/router.rs:54 rate_limit_error_response` -> 1 call-site (309); `src/config.rs:1892 apply_hot_reload_from_value` -> 1 call-site (1853). **Ninguno muerto.** | Sin accion. Nota metodologica: el grafo no modela el harness de test, asi que "sin CALLS entrantes" NO es evidencia de dead code para funciones `#[test]`. | - |

### Sub-scope W1b - Indices y algoritmos

| ID | Severidad | Hallazgo | Archivo:línea | Evidencia | Fix propuesto | Esfuerzo |
|---|---|---|---|---|---|---|
| `W1-07` | Info | La concentracion de `unsafe` en la capa de mmap/kernels es coherente con el diseno y esta documentada | `src/storage/vfile_mmap.rs` (24 bloques), `src/index/distance/kernels.rs` (12), `src/storage/vfile.rs` (5), `src/storage/engine/get.rs` (5), `src/storage/ops.rs` (4), `src/storage/lock.rs` (4) | 24 ficheros con `unsafe`; 21 de 24 contienen la cadena `SAFETY` (invariante documentado). El patron es el esperado: `align_to`/`from_raw_parts` sobre buffers mmap para SIMD. `src/index/distance/kernels.rs` es el unico con 12 bloques y todos con invariante. | Sin accion. | - |
| `W1-08` | Info | `EntityBoostReport::iter` es el simbolo con mayor fan-in del core (786) | `src/sdk/search/fusion.rs` | `codebase-memory-mcp_get_architecture` hotspots: `src.sdk.search.fusion.EntityBoostReport.iter` fan_in 786. Un iterador de hot path con ese fan-in merece atencion en cualquier trabajo futuro de rendimiento (Regla 9: cualquier cambio aqui exige before/after contra `benches/canonical_p99.rs`). | Sin accion (no es un defecto). Anotar como zona sensible para Regla 9. | - |

### Sub-scope W1c - Correctness y seguridad

| ID | Severidad | Hallazgo | Archivo:línea | Evidencia | Fix propuesto | Esfuerzo |
|---|---|---|---|---|---|---|
| `W1-09` | Info | La superficie `unsafe` del core es acotada y auditable (83 bloques, 24 ficheros, 0 `todo!`) | `src/**` | Ver W1-07. Ratio 83 bloques / 271 ficheros. 21/24 ficheros con invariante `SAFETY` documentado. `src/index/search/tests.rs:856-868` documenta explicitamente por que los bloques `from_raw_parts` de `search_layer` **no** son cubiertos por Miri (requieren mmap real) y que el camino Miri se cubre via `fast_similarity` con `vector_store = None`. Es una justificacion honesta y trazable. | Sin accion. | - |
| `W1-10` | Info | El grafo de llamadas muestra un limite de capa limpio: `src` -> `sdk` (312 edges), `sdk` no llama a `src` (0 out) | `src/sdk/**` | `get_architecture` boundaries: `src -> sdk` 312 llamadas; `sdk` aparece como capa "core" con fan-in 517 y **fan-out 0**. El SDK es una capa superior que solo consume el core, nunca al reves. Coherente con la regla R-8 de `docs/api/scores.md` ("bindings are thin glue"). | Sin accion. Fortaleza arquitectonica a destacar en la sintesis (FODA). | - |

## Veredicto

**PASS CON OBSERVACIONES**

El core Rust es la parte mas solida del sistema. La disciplina de `unwrap()` en produccion es
estricta (~14 ocurrencias reales, no 2 098), no hay `todo!()`/`unimplemented!()`, `#[non_exhaustive]`
esta aplicado a los 17 enums publicos que pueden crecer, el limite de capa `src -> sdk` es limpio
(sdk no depende de vuelta del core) y **no se confirma dead code de produccion**. El unico hallazgo
Medium es documental: **un bloque `unsafe` de produccion sin invariante `SAFETY:` documentado**
(`src/storage/engine/txn.rs:612`), en contra de la Regla 4 del propio proyecto. No se ejecuto codigo
(cargo ausente del PATH), por lo que ninguna afirmacion de este reporte es un claim de runtime.

## Indice de hallazgos

| ID | Severidad | Archivo:línea |
|---|---|---|
| `FIND-396` | Medium | `src/storage/engine/txn.rs:612` |
| `W1-02` | Info | `src/shred/mod.rs` |
| `W1-03` | Info | `src/error.rs:136` |
| `W1-04` | Info | `src/**` |
| `W1-05` | Info | `src/config.rs:1` |
| `W1-06` | Info | `src/cli_handlers/search.rs:319` |
| `W1-07` | Info | `src/storage/vfile_mmap.rs:1` |
| `W1-08` | Info | `src/sdk/search/fusion.rs` |
| `W1-09` | Info | `src/**` |
| `W1-10` | Info | `src/sdk/**` |
