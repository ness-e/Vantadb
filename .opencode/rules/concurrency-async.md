# Concurrency & Async (Tokio) — Reglas

> **Scope:** uso de async/await en `src/` y bindings, `tokio::task::spawn_blocking`, `std::sync::Mutex` vs `tokio::sync::Mutex`, `parking_lot`, semáforos, atomics, `wal_sharded`, ingestion async (`ingestion.rs`), I/O async (`transcript.rs`)
> **No tocar aquí:** durabilidad/WAL (`durability.md`), índices (`indexes.md`)
> **Status:** 🟢 Vigente
> **Fuentes:** INV-003 (Tokio Blocking Audit)

## Reglas

### 1 — Todo trabajo pesado del motor en handlers async va en `spawn_blocking`

- **Must:** toda llamada a `StorageEngine` o `Executor` desde una función `async` debe envolverse en `tokio::task::spawn_blocking` (patrón de `execute_query` en `cli_server.rs`).
- **Must not:** ejecutar `std::fs::*`, `execute_hybrid`, `flush()` u otras operaciones CPU/disco directamente dentro de tareas Tokio.
- **Por qué:** una llamada síncrona larga bloquea un worker thread de Tokio → inanición del event loop y picos de tail latency bajo carga concurrente (INV-003 §1).

### 2 — Nunca mantener un guard de mutex síncrono a través de `.await`

- **Must not:** mantener un `std::sync::Mutex` / `parking_lot::Mutex` guard mientras se suspende la tarea con `.await`.
- **Must:** para estado compartido en contexto async usar `tokio::sync::Mutex`; para estado síncrono interno, mantener el guard y el `lock()` dentro de la misma llamada síncrona (sin `.await` intermedio).
- **Por qué:** un guard síncrono cruzando un `.await` bloquea el hilo ajeno al runtime y puede causar deadlock con el task scheduling (INV-003 §2.2 — no se detectaron casos actuales; la regla es preventiva).

### 3 — Concurrencia de servidor: usar `tokio::sync::Semaphore` + pool

- **Must:** limitar concurrencia de endpoints con `tokio::sync::Semaphore` (patrón `ConnectionPool` en `src/connection_pool.rs`, o `acquire_owned()` en `vantadb-mcp`).
- **Must not:** inventar límites con mutex síncronos alrededor del handler.
- **Por qué:** el semáforo Tokio no bloquea hilos; un mutex síncrono sí (INV-003 §2.2 y §3).

### 4 — Mutexes síncronos en capa de índices: solo dentro de llamadas síncronas aisladas

- **Must:** mantener el uso de `std::sync::Mutex` en `src/index/*` (`scann.rs`, `flat.rs`, `diskann.rs`) exclusivamente en métodos síncronos invocados desde `spawn_blocking`.
- **Must not:** exponer esos métodos como `async` directos ni mantener guards entre `.await`.
- **Por qué:** la auditoría INV-003 confirmó que el patrón actual es seguro; cambiarlo rompería la separación hilos I/O vs CPU/disco.

### 5 — I/O de archivos en módulos async: dual mode `#[cfg(feature = "async-io")]`

- **Must:** mantener el patrón de `src/transcript.rs`: `tokio::fs` con `#[cfg(feature = "async-io")]` y `std::fs` con `#[cfg(not(feature = "async-io"))]`, con variantes `async fn` / `fn` respectivamente.
- **Por qué:** permite build sin Tokio (WASM/embed) y con Tokio (server) sin duplicar lógica.

### 6 — `std::fs` en background threads: permitido pero NO en event loop

- **Must:** las operaciones `std::fs` de `wal_shipping.rs`, `wal_archiver.rs`, `wal.rs` corren en hilos de background dedicados o en init/snapshot — **no** en el event loop de Tokio.
- **Must not:** mover esas llamadas a un contexto async sin envolverlas en `spawn_blocking` o sin un hilo dedicado.
- **Por qué:** INV-003 §2.1.3 confirmó que no bloquean el runtime porque ya están aisladas.
- **Extensión INV-003:** los handlers MCP (`vantadb-mcp`) y del server deben seguir el mismo patrón: trabajo del motor en `spawn_blocking` (ver R-7), con límite de concurrencia vía semáforo (ver R-3) para no saturar `spawn_blocking` con requests ilimitados.

### 7 — Handlers MCP/HTTP: trabajo del motor SIEMPRE en `spawn_blocking` (INV-003 R7)

- **Must:** en `vantadb-mcp` y `vantadb-server`, envolver toda llamada a `StorageEngine`/`Executor`/I/O en `tokio::task::spawn_blocking` (patrón `execute_query` de `cli_server.rs`; en `vantadb-mcp` usar `spawn_blocking` con `join` en el handler stdio).
- **Must not:** ejecutar búsquedas híbridas, `flush()` o `std::fs` directamente en el handler async del servidor.
- **Por qué:** INV-003 verificó que el motor es síncrono y CPU/disco-bound; correrlo en el event loop de Tokio (o en el thread del runtime MCP) causa inanición y tail latency bajo carga concurrente.

<!-- Referencias cruzadas: → ver durability.md, core-engine.md, server-mcp.md -->
