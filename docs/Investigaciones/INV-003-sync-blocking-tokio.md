# Reporte de Auditoría y Propuesta — INV-003: Sync Blocking en Tokio

> **ID:** `INV-003`  
> **Categoría:** Phase 4 — Engineering Health & Architecture  
> **Fecha:** 2026-07-31  
> **Estado:** ✅ Auditoría Completada — Propuesta Lista  

---

## 1. Contexto y Objetivos

Las llamadas sincrónicas al sistema de archivos (`std::fs::*`) y los mutexes sincrónicos de largo bloqueo (`std::sync::Mutex` / `parking_lot::Mutex`) ejecutados dentro del context del runtime asincrónico de Tokio pueden provocar la **inanición de los hilos de trabajo (worker threads)**. Esto causa picos impredecibles de latencia (*tail latencies*) y degrada la capacidad del servidor bajo carga concurrente.

Esta auditoría evaluó todo el espacio de código en `src/`, `vantadb-server/` y `vantadb-mcp/` para detectar llamadas sincrónicas en funciones `async` y proponer su mitigación mediante `spawn_blocking` o desacoplamiento.

---

## 2. Hallazgos Principales

### 2.1 Uso de Async vs Sync en el Motor Principal (`src/`)

1. **Protección mediante `spawn_blocking` ya existente en API Async:**
   - **`execute_query` en `src/cli_server.rs:409`**: Invocaciones pesadas al motor mediante `Executor::execute_hybrid` corren aisladas dentro de `tokio::task::spawn_blocking`.
   - **`flush_on_shutdown_async` en `src/cli_server.rs:794`**: `storage.flush()` corre mediante `spawn_blocking`.
   - **`AsyncIngestionPipeline` en `src/ingestion.rs:85`**: El procesamiento del task `Self::process` corre en `spawn_blocking`.

2. **Manejo Dual en `src/transcript.rs`:**
   - El módulo utiliza compilación condicional `#[cfg(feature = "async-io")]` con `tokio::fs` para lectura y escritura asincrónica de transcripciones, y `#[cfg(not(feature = "async-io"))]` con `std::fs`. Está adecuadamente aislado.

3. **Llamadas a `std::fs` en Contextos Sincrónicos:**
   - Módulos como `src/wal_shipping.rs`, `src/wal_archiver.rs`, `src/wal.rs` y `src/storage/engine/mod.rs` utilizan `std::fs` en hilos de background dedicados o durante operaciones de inicio (`init`/`snapshot`). **No bloquean el event loop de Tokio directamente.**

### 2.2 Mutexes y Estado Concurrente

- En `vantadb-server/`, los manejadores usan estado compartido de Tokio (`Arc<ServerState>`) con un semáforo de concurrencia (`tokio::sync::Semaphore`).
- No se detectaron cierres o guardas de `std::sync::Mutex` mantenidas a través de puntos de suspensión (`.await`).
- Los mutexes sincrónicos en la capa de índices (`src/index/scann.rs`, `src/index/flat.rs`, `src/index/diskann.rs`) se utilizan exclusivamente en llamadas sincrónicas aisladas dentro de `spawn_blocking`.

---

## 3. Matriz de Riesgos Identificados

| Ubicación | Operación | Contexto | Nivel de Riesgo | Solución Propuesta |
|---|---|---|---|---|
| `src/cli_server.rs:744` | `build_tls13_config` (lectura de certificados SSL en inicio) | Startup Async | 🟢 Bajo | Aceptable en fase de arranque (single-pass). |
| `src/wal_shipping.rs` | `std::fs::read_dir` en envío de segmentos WAL | Background thread | 🟢 Bajo | Corre fuera del loop de Tokio. |
| `src/ingestion.rs:77` | `tokio::sync::Mutex` en `worker_loop` | Async Worker | 🟢 Bajo | Lock liviano de canal Tokio (no bloquea hilo). |

---

## 4. Plan de Acción Recomendado

1. **Mantener la Disciplina en Endpoints HTTP/gRPC:**
   - Toda llamada al `StorageEngine` o `Executor` en handlers futuros debe envolverse en `tokio::task::spawn_blocking`.
2. **Sin cambios inmediatos requeridos:**
   - La arquitectura actual respeta la separación entre hilos I/O de Tokio y tareas intensivas en CPU/Disco.

---
*Reporte generado automáticamente como parte de INV-003.*
