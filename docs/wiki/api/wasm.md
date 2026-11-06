---
title: "API de WASM (`vantadb-wasm`)"
kind: reference
status: active
description: "Binding wasm-bindgen: persistencia OPFS/IDB por runtime, contadores de saneamiento observables y límites del target wasm32."
---

# API de WASM (`vantadb-wasm`)

Audiencia: usuarios del runtime browser/WASM. Referencias canónicas: [WASM_API](../../api/WASM_API.md), [WASM_PERSISTENCE](../../api/WASM_PERSISTENCE.md), [WASM_STANDALONE](../../api/WASM_STANDALONE.md).

## Superficie

`[HECHO]` El crate expone sus operaciones vía `wasm_bindgen` en `vantadb-wasm/src/lib.rs`; el inventario normativo por dominio (memory/graph/wiki/system, incl. `recover_archived_nodes`) está en [BINDINGS_NAMESPACES](../../api/BINDINGS_NAMESPACES.md). Canal declarado: npm `vantadb-wasm` compilado con wasm-pack (`release-npm-wasm.yml`); consumo habitual a través del wrapper [TypeScript](typescript.md).

`[HECHO]` Estructura del crate: `lib.rs` (binding), `opfs.rs`, `idb.rs`, `worker.rs`, más el crate separado `threads-kernel/` (experimental).

## Persistencia por runtime

`[HECHO]` El crate incluye capa de persistencia OPFS (Origin Private File System) e IndexedDB (`src/opfs.rs`, `src/idb.rs`); los límites y la matriz por runtime están en WASM_PERSISTENCE. `[INFERENCIA]` La persistencia real depende del runtime JS: browser (OPFS/IDB) vs Node ESM wasm (filesystem del host o memoria según config) — verificar el modo antes de prometer durabilidad en un entorno concreto.

## Límites del target wasm32

`[HECHO]` `OpGate::drain` no puede bloquear en `wasm32-unknown-unknown` (single-thread): la barrera se activa y las ops async completan después en el event loop (`vantadb-ffi-core/src/lib.rs:44-50`). `[HECHO]` La capa cognitiva `vanta-memory` NO compila hoy para wasm32 — `SystemTime::now()` panics, `std::fs` siempre error, `reqwest::blocking` no existe en wasm; gap list verificado en FIND-255 (BINDINGS_NAMESPACES §Cognitive layer scope).

`[HECHO]` El binding hace observables dos alteraciones silenciosas de datos: `nan_sanitization_count` (NaN/Inf→0.0 al serializar hacia JS/JSON) y `metadata_drop_count` (fallos de serialización de metadata), ambos acumulados y expuestos vía `operational_metrics()` ([`vantadb-wasm/src/lib.rs:15-46`](../../../vantadb-wasm/src/lib.rs#L15-L46)).

## Semántica

`[HECHO]` `SearchHit.score` mayor-es-mejor en híbrida; `search_vector()` devuelve `distance` (WSM-10 alineado al core). `u128` ids como string decimal; traversals `bigint[]`.

## Fuentes

`vantadb-wasm/src/` · `vantadb-wasm/threads-kernel/` · [WASM_API](../../api/WASM_API.md) · [WASM_PERSISTENCE](../../api/WASM_PERSISTENCE.md) · [CRASH_MODEL](../../dev/wasm/CRASH_MODEL.md) · [WASM_THREADS](../../dev/architecture/WASM_THREADS.md)
