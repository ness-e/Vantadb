# Plan de Implementación: MP2 — OpenTelemetry e Instrumentación del Core

## Goal Description

Completar la integración de OpenTelemetry (OTLP) en el core del motor de base de datos (`vantadb`). En la fase previa, se configuró el pipeline de exportación OTLP y el enrutador HTTP en `vantadb-server`, pero el core del motor carece de instrumentación fina.

Este cambio habilitará trazas distribuidas detalladas en los hot-paths de la base de datos (recorridos del HNSW, accesos al motor de almacenamiento persistente y ejecución en el planificador Volcano), permitiendo a los operadores diagnosticar cuellos de botella de latencia con precisión milimétrica en sistemas de observabilidad (Jaeger, Tempo, etc.).

---

## User Review Required

> [!IMPORTANT]
> **Enfoque de Dependencias Opcionales:**
> Para preservar la filosofía de VantaDB como un motor ultra-ligero de cero dependencias pesadas por defecto, se introducirá la feature flag `opentelemetry` en el Cargo.toml raíz (`vantadb`).
> 
> * Las dependencias de `opentelemetry` y `tracing-opentelemetry` se cargarán de forma opcional (`optional = true`).
> * Si el feature flag `opentelemetry` no está activo, las trazas del core se seguirán emitiendo a través de la crate estándar `tracing` (que es muy ligera y ya está en uso), pero sin importar los tipos ni el SDK pesado de OpenTelemetry en la compilación final del core.

---

## Proposed Changes

### 1. Configuración de dependencias en el Core

#### [MODIFY] [Cargo.toml](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/Cargo.toml)
* Agregar dependencias opcionales de observabilidad:
  * `opentelemetry = { version = "0.32.0", optional = true }`
  * `tracing-opentelemetry = { version = "0.33.0", optional = true }`
* Agregar el feature flag `opentelemetry` en la sección `[features]`:
  * `opentelemetry = ["dep:opentelemetry", "dep:tracing-opentelemetry"]`

---

### 2. Instrumentación de Hot-Paths

Añadiremos el macro `#[tracing::instrument]` a los métodos críticos del motor. Para evitar overhead innecesario, omitiremos la serialización de estructuras masivas (como vectores completos) utilizando `skip(...)` en los parámetros.

#### [MODIFY] [src/storage.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/storage.rs)
Instrumentar métodos de E/S y almacenamiento persistente:
* `pub fn get(&self, node_id: u64) -> Result<Option<UnifiedNode>>`  
  👉 `#[tracing::instrument(skip(self), level = "debug", err)]`
* `pub fn insert(&self, node: &UnifiedNode) -> Result<()>`  
  👉 `#[tracing::instrument(skip(self, node), level = "debug", err)]`
* `pub fn delete(&self, node_id: u64, reason: &str) -> Result<()>`  
  👉 `#[tracing::instrument(skip(self), level = "debug", err)]`
* `pub fn flush(&self) -> Result<()>`  
  👉 `#[tracing::instrument(skip(self), level = "info", err)]`

#### [MODIFY] [src/index.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/index.rs)
Instrumentar los hot-paths del índice HNSW (búsquedas vectoriales y mutaciones):
* `pub fn search_nearest(...)`  
  👉 `#[tracing::instrument(skip(self, query_vector, vector_store), level = "debug")]`
* `pub fn insert(...)`  
  👉 `#[tracing::instrument(skip(self, vector, vector_store), level = "debug")]`

---

## Verification Plan

### Automated Tests
* **Compilación sin feature flag:**
  ```powershell
  cargo check --workspace --release
  ```
* **Compilación con feature flag (Simulación de producción):**
  ```powershell
  cargo check --workspace --features opentelemetry --release
  ```
* **Pruebas de integridad:**
  ```powershell
  cargo test --package vantadb --release
  ```

### Manual Verification
1. Arrancar el servidor local con telemetría activa:
   ```powershell
   $env:VANTADB_LOG_JSON="1"
   cargo run -p vantadb-server --features opentelemetry
   ```
2. Realizar peticiones HTTP de búsqueda y validar que los logs estructurados reflejan los spans de `execute_hybrid`, `search_nearest` y `get` correlacionados bajo el mismo `trace_id`.
