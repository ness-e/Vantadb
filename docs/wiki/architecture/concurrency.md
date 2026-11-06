---
title: "Modelo de concurrencia de VantaDB"
kind: concept
status: active
description: "Locks, estructuras concurrentes y semántica multi-proceso del motor embebido: qué está verificado y dónde."
---

# Modelo de concurrencia de VantaDB

Audiencia: usuarios que ejecutan el engine multi-hilo y contribuidores que tocan rutas de escritura.

## Principios verificados

`[HECHO]` El crate usa `parking_lot`, `dashmap`, `arc-swap` y `portable-atomic` como primitivas de concurrencia (`Cargo.toml:74-78`). La documentación del crate declara `#![deny(unsafe_op_in_unsafe_fn)]` (`src/lib.rs:2`).

`[HECHO]` La barrera de cierre compartida por bindings es `OpGate`/`OpGuard` en [`vantadb-ffi-core/src/lib.rs`](../../../vantadb-ffi-core/src/lib.rs): rechaza operaciones nuevas tras `drain()` y espera a que las en vuelo terminen; el contrato prohíbe bloquear manteniendo locks externos y recupera poison con `into_inner`. En `wasm32-unknown-unknown` `drain()` no puede bloquear — el barrier se activa igualmente y las operaciones async en vuelo completan en el event loop (`vantadb-ffi-core/src/lib.rs:44-50`). PyO3 exige llamar `drain` con el GIL liberado para no deadlock (`vantadb-ffi-core/src/lib.rs:52-58`).

`[HECHO]` Un único proceso puede escribir por base de datos: el lock de fichero `.vanta.lock` impone single-writer y hay test dedicado `tests/storage/multi_process_lock.rs` y `tests/file_locking_stress.rs` (registrados en `Cargo.toml:604-607`, `:619-623`); el caso MCP multi-instancia se resuelve con fallback HTTP (ADR-0036).

`[HECHO]` La decisión sobre granularidad del lock de inserción está registrada: ADR-0037 mantiene un lock global de escritura (`insert_lock`) tras medir la alternativa. `[INFERENCIA]` Por tanto la escritura concurrente intra-proceso serializa en ese punto; lectores y estructuras lock-free (`dashmap`, `arc-swap`) absorben el paralelismo de lectura — la afirmación exacta del orden de locks debe verificarse en `src/storage/engine/` antes de citarse como contrato.

## Concurrencia asíncrona

`[HECHO]` Hay tres vías async opt-in: `async-ingestion` (pool de workers para inserción, `src/ingestion.rs`, `src/lib.rs:183-184`), `async-io` (I/O de transcripts, `src/transcript.rs`, `src/lib.rs:185-187`) y el runtime `tokio` tras la feature `server` (`Cargo.toml:105`, `:184-193`). El SDK embebido síncrono es la superficie primaria; `AsyncClient` en Python envuelve el engine en un pool de hilos con el GIL liberado ([PYTHON_SDK](../../api/PYTHON_SDK.md)).

## WASM

`[HECHO]` El target `wasm32-unknown-unknown` es single-threaded en la build actual: `OpGate::drain` no bloquea ahí por diseño (fuente citada arriba). `[HECHO]` Existe un crate `vantadb-wasm/threads-kernel` separado (fuera del grafo público por defecto) y documentación de diseño en `docs/dev/architecture/WASM_THREADS.md` — su estado es experimental; no asumir SharedArrayBuffer activo.

## Qué NO está verificado aquí

`[INFERENCIA]` El orden exacto de adquisición de locks, la vida de guards a través de `await` y el comportamiento ante poison dentro del engine no se auditaron para esta página — verificar en `src/storage/engine/{mod,txn,insert}.rs` antes de afirmar invariantes.

## Fuentes

`vantadb-ffi-core/src/lib.rs` · `src/lib.rs` · `src/ingestion.rs` · `src/transcript.rs` · `Cargo.toml` · ADR-0036/0037 en `docs/dev/architecture/adr/` · `tests/storage/multi_process_lock.rs` · [WASM_THREADS](../../dev/architecture/WASM_THREADS.md)
