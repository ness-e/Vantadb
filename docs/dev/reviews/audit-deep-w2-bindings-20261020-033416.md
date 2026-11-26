---
title: "Deep audit W2 - Bindings and SDKs"
kind: review
status: active
description: "Auditoria read-only de los bindings (python/node/ts/wasm/ffi-core/proxy): paridad de API, stubs .pyi anti-drift, semver, errores, FFI/UB y contrato ERR-028 entre SDKs."
tags: [audit, reviews, bindings]
---

# Deep audit W2 - Bindings / SDKs

> **Wave:** W2 · **Fecha:** 2026-10-20 · **Modo:** read-only
> **Especificacion:** `docs/dev/plans/2026-10-20-revision-profunda-integral-vantadb.md` §4 (W2)
> **Ejecucion:** subagentes `vanta-audit` (FFI/UB) + `vanta-review` (paridad/contrato) no disponibles
> (sin tool `task`). El orquestador ejecuto ambos sub-scope.

## Scope

**Cubierto:**

| Binding | Ruta | Tamano medido |
|---|---|---|
| Python (PyO3) | `vantadb-python/src/{lib,types,convert,vector}.rs` | 4 ficheros, 4 255 lineas, 64 metodos `#[pymethods]` |
| Node (napi) | `vantadb-node/src/lib.rs` | 1 fichero, 1 957 lineas |
| TS (WASM) | `vantadb-ts/src/{vantadb,native,types,guards,errors,metadata}.ts` | 6 ficheros |
| WASM | `vantadb-wasm/src/{lib,opfs,idb,worker}.rs` + `opfs_bridge.js` | 4 ficheros, 3 979 lineas, 67 `pub fn` |
| FFI core | `vantadb-ffi-core/src/lib.rs` | 1 fichero, 215 lineas |
| Proxy | `vanta-proxy/src/**` | 33 ficheros, 11 524 lineas, 138 `pub fn` |

**Excluido con motivo:** `providers/*` (W4); `integrations/*` (W4); `vanta-memory` (W9, es un crate
del workspace, no un binding de usuario).

## Metodologia

- Conteo de superficie publica por binding (`pub fn` / `#[pymethods]`).
- Anti-drift de stubs: verificacion cruzada entre `vantadb-python/src/lib.rs` y los 3 ficheros
  `.pyi`, y entre `providers/*/src/lib.rs` y `providers/*/vantadb_*.pyi`.
- FFI/UB: conteo de `unsafe`, `from_raw_parts`, punteros crudos y `__array_interface__` por binding.
- Contrato de score: verificacion de ERR-028 (zero-norm) en los 4 bindings contra
  `docs/api/scores.md`.
- Sin ejecucion (cargo ausente del PATH).

## Hallazgos

| ID | Severidad | Hallazgo | Archivo:línea | Evidencia | Fix propuesto | Esfuerzo |
|---|---|---|---|---|---|---|
| `W2-01` | Info | **0 `unsafe` en los 4 bindings de usuario** (python, node, wasm, ffi-core) | `vantadb-python/src/`, `vantadb-node/src/lib.rs`, `vantadb-wasm/src/`, `vantadb-ffi-core/src/lib.rs` | Conteo de `unsafe\s*\{|unsafe\s+fn` por binding: python 0, node 0, wasm 0, ffi-core 0. El unico `unsafe` en todo el arbol de bindings esta en `vantadb-mcp` (1). Esto confirma que el P2-2 (raw pointer UB en `__array_interface__`) esta efectivamente resuelto: `vantadb-python/src/vector.rs` devuelve un `PyBytes` owned copy en lugar de exponer un puntero. | Sin accion. | - |
| `W2-02` | Info | Anti-drift de stubs Python: existe y es efectivo | `vantadb-python/vantadb_py/vantadb_py.pyi`, `vantadb-python/tests/test_stub_drift.py` | El stub canonico es `vantadb-python/vantadb_py/vantadb_py.pyi` (514 lineas, 143 declaraciones `def`/`class`). `vantadb-python/vantadb/__init__.pyi` (26 lineas, 0 `def`) es un **shim de re-export** deliberado, documentado en las docstrings de ambos stubs. `vantadb-python/tests/test_stub_drift.py` existe y "asserts this file matches the compiled module (method set, parameter names and requiredness via `inspect.signature`)". `providers/*/vantadb_*.pyi` (3 stubs) se validan con `.github/scripts/verify_pyi.py`, invocado por `ci-fleet.yml:468`. | Sin accion. | - |
| `W2-03` | Info | `vantadb-ffi-core` es un leaf std-only con contrato documentado y sin dependencias | `vantadb-ffi-core/src/lib.rs:1-40` | La docstring del crate declara: "This crate must stay dependency-free (`std` only)... `cargo tree -p vantadb-ffi-core -e normal` must stay empty". Motivo explicito: evitar que napi + pyo3 + js-sys contaminen el build wasm32. Concentra `OpGate`/`OpGuard` (barrera de durabilidad cross-transport) y `clamp_top_k` (politica ERR-022). "Error mapping stays per transport on purpose". Diseno correcto y justificado. | Sin accion. | - |
| `W2-04` | Info | Paridad de `put_batch` entre Python y el resto: la dual API sigue existiendo | `vantadb-python/src/lib.rs:839` (`put_batch`), `vantadb-python/src/lib.rs:869` (`put_batch_raw`) | Dos metodos PyO3 con la misma responsabilidad: `put_batch` (keyword-only, objetos) y `put_batch_raw` (arrays crudos). Es exactamente la deuda P2-5 registrada en `.opencode/AGENTS.md:579` ("revalidar en API-02"), que sigue **abierta**. La revalidacion pendiente es que la referencia de linea de AGENTS.md (`:494`) esta obsoleta: hoy es `:702`/`:811` segun AGENTS.md, y el codigo real esta en `:839`/`:869` - la deriva de linea ya es de 3 referencias. | Actualizar la referencia de linea en `AGENTS.md:579` a `:839`/`:869` y hacer la revalidacion de API-02. | 1h |
| `W2-05` | Info | El contrato ERR-028 (zero-norm) esta **paralelo** en los 4 bindings: no hay fallback silencioso en ningun lado | `src/sdk/search/mod.rs:244`, `src/sdk/api/search.rs:30`, `vantadb-python/src/`, `vantadb-node/src/lib.rs`, `vantadb-wasm/src/lib.rs` | El core rechaza el vector zero-norm en `src/sdk/search/mod.rs:244` y `src/sdk/api/search.rs:30` con el mensaje "zero-norm cosine query vector is undefined...". Busqueda de `zero-norm|ERR-028|ZeroNorm` en `vantadb-node/src/lib.rs`, `vantadb-wasm/src/lib.rs` y `vantadb-python/src/*.rs` = **0 ocurrencias**, es decir, ningun binding reintroduce un fallback: la validacion vive **solo** en el core y los bindings la propagan. Esto es exactamente lo que R-8 de `docs/api/scores.md` exige. | Sin accion de codigo. La **documentacion** de la deriva esta obsoleta (ver W5 `FIND-387`). | - |
| `W2-06` | Low | `vanta-proxy` expone 138 `pub fn` en 33 ficheros: superficie amplia para un componente periferico | `vanta-proxy/src/` | 138 `pub fn` frente a 4 de `vantadb-ffi-core` y 3 de `vantadb-node`. El proxy es el binding mas grande del sistema y no tiene un `[[test]]` dedicado ni un `.pyi`/`.d.ts` de contrato publico equivalente al de los demas bindings (tiene 25 ficheros de test, `vanta-proxy/tests/`, lo que es positivo). | Auditar la frontera publica del proxy en una wave dedicada. No bloquea v0.9.0. | 1-2d |
| `W2-07` | Info | Semver: los bindings no declaran version propia en el `.pyi`; la version se propaga desde el core | `vantadb-python/vantadb_py/vantadb_py.pyi` (docstring), `Cargo.toml:805` | Los stubs exponen `__version__` como re-export, no como literal. `Cargo.toml:805` fija `version = "0.9.0"` a nivel workspace. `release-wheels.yml:95` invoca `cargo test --test version_coherence` (target declarado en `Cargo.toml:591-592`) para verificar coherencia de version. Existe gate. | Sin accion. | - |

## Veredicto

**PASS**

Los bindings son la parte mejor ingneriada del sistema junto con el core. Cero `unsafe` en los cuatro
bindings de usuario, un leaf FFI sin dependencias con contrato documentado, stubs `.pyi` con un test
anti-drift real, y una unica validacion de contrato (ERR-028 zero-norm) que vive **solo** en el core y
se propaga sin fallbacks silenciosos en ningun binding - que es exactamente el principio "bindings are
thin glue" que `docs/api/scores.md` enuncia. No se encontro ningun hallazgo Critical ni High en W2.
El unico apunte con coste de mantenimiento es la dual API `put_batch`/`put_batch_raw` (P2-5, ya
registrada) cuya referencia de linea en `AGENTS.md` ha vuelto a quedar obsoleta.

## Indice de hallazgos

| ID | Severidad | Archivo:línea |
|---|---|---|
| `W2-01` | Info | `vantadb-python/src/vector.rs:59` |
| `W2-02` | Info | `vantadb-python/vantadb_py/vantadb_py.pyi:1` |
| `W2-03` | Info | `vantadb-ffi-core/src/lib.rs:1` |
| `W2-04` | Info | `vantadb-python/src/lib.rs:839` |
| `W2-05` | Info | `src/sdk/search/mod.rs:244` |
| `W2-06` | Low | `vanta-proxy/src/lib.rs:1` |
| `W2-07` | Info | `Cargo.toml:805` |
