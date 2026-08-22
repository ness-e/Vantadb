# Auditoría — INV-003: Tokio Blocking Audit (uso de spawn_blocking)

> **ID:** `INV-003`  
> **Categoría:** Phase 4 — Engineering Health & Architecture  
> **Fecha:** 2026-07-31  
> **Actualizado:** 2026-08-04 — verificación de líneas de código, corrección de `build_tls13_config` y cobertura de `vantadb-mcp`  
> **Estado:** ✅ Auditoría Completada — Propuesta Lista  

---

## 1. Contexto y Objetivos

Las llamadas sincrónicas al sistema de archivos (`std::fs::*`) y los mutexes sincrónicos de largo bloqueo (`std::sync::Mutex` / `parking_lot::Mutex`) ejecutados dentro del context del runtime asincrónico de Tokio pueden provocar la **inanición de los hilos de trabajo (worker threads)**. Esto causa picos impredecibles de latencia (*tail latencies*) y degrada la capacidad del servidor bajo carga concurrente.

Esta auditoría evaluó todo el espacio de código en `src/`, `vantadb-server/` y `vantadb-mcp/` para detectar llamadas sincrónicas en funciones `async` y proponer su mitigación mediante `spawn_blocking` o desacoplamiento.

---

## 2. Hallazgos Principales

### 2.1 Uso de Async vs Sync en el Motor Principal (`src/`)

1. **Protección mediante `spawn_blocking` ya existente en API Async:**
   - **`execute_query` en `src/cli_server.rs:429`**: Invocaciones pesadas al motor mediante `Executor::execute_hybrid` corren aisladas dentro de `tokio::task::spawn_blocking`.
   - **`flush_on_shutdown_async` en `src/cli_server.rs:858`**: `storage.flush()` corre mediante `spawn_blocking`.
   - **`AsyncIngestionPipeline` en `src/ingestion.rs:85`**: El procesamiento del task `Self::process` corre en `spawn_blocking`.

2. **Manejo Dual en `src/transcript.rs`:**
   - El módulo utiliza compilación condicional `#[cfg(feature = "async-io")]` con `tokio::fs` para lectura y escritura asincrónica de transcripciones, y `#[cfg(not(feature = "async-io"))]` con `std::fs`. Está adecuadamente aislado.

3. **Llamadas a `std::fs` en Contextos Sincrónicos:**
   - Módulos como `src/wal_shipping.rs`, `src/wal_archiver.rs`, `src/wal.rs` y `src/storage/engine/mod.rs` utilizan `std::fs` en hilos de background dedicados o durante operaciones de inicio (`init`/`snapshot`). **No bloquean el event loop de Tokio directamente.**

### 2.2 Mutexes y Estado Concurrente

- En `vantadb-server/`, los manejadores usan estado compartido de Tokio (`Arc<ServerState>`) con un semáforo de concurrencia (`tokio::sync::Semaphore`). El semáforo vive en `ConnectionPool` (`src/connection_pool.rs:15`) como campo de `ServerState` (`src/cli_server.rs:101`); `vantadb-server/src/server.rs` re-exporta el router de `vantadb::cli_server`.
- `vantadb-mcp/` aplica el mismo patrón explícitamente: `tokio::sync::Semaphore` + `tokio::task::spawn_blocking` con `timeout` en `tools/call` y `resources/read` (`vantadb-mcp/src/lib.rs:505-561`).
- No se detectaron cierres o guardas de `std::sync::Mutex` mantenidas a través de puntos de suspensión (`.await`).
- Los mutexes sincrónicos en la capa de índices (`src/index/scann.rs`, `src/index/flat.rs`, `src/index/diskann.rs`) se utilizan exclusivamente en llamadas sincrónicas aisladas dentro de `spawn_blocking`.

---

## 3. Matriz de Riesgos Identificados

| Ubicación | Operación | Contexto | Nivel de Riesgo | Solución Propuesta |
|---|---|---|---|---|
| `src/cli_server.rs:810` | `build_tls13_config` (lectura de certificados SSL) | Startup Async | 🟢 Bajo | Ya usa `tokio::fs::read` — sin bloqueo real del event loop. |
| `src/wal_shipping.rs` | `std::fs::read_dir` en envío de segmentos WAL | Background thread | 🟢 Bajo | Corre fuera del loop de Tokio. |
| `src/ingestion.rs:77` | `tokio::sync::Mutex` en `worker_loop` | Async Worker | 🟢 Bajo | Lock liviano de canal Tokio (no bloquea hilo). |

---

## 4. Plan de Acción Recomendado

1. **Mantener la Disciplina en Endpoints HTTP/gRPC:**
   - Toda llamada al `StorageEngine` o `Executor` en handlers futuros debe envolverse en `tokio::task::spawn_blocking`.
2. **Sin cambios inmediatos requeridos:**
   - La arquitectura actual respeta la separación entre hilos I/O de Tokio y tareas intensivas en CPU/Disco.

---
*Reporte generado automáticamente como parte de INV-003. Actualizado el 2026-08-04 tras verificación contra el código fuente.*
