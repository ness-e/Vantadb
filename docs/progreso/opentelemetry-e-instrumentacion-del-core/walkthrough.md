# Walkthrough: Fase 2 — MP2: OpenTelemetry e Instrumentación del Core

## Objetivo

Completar la integración e instrumentación fina de OpenTelemetry (OTLP) y el logging estructurado en los hot-paths de la base de datos (`vantadb`) de forma opcional (detrás de una feature flag). Esto permite trazas distribuidas detalladas en los hot-paths para diagnosticar latencias en sistemas de observabilidad externos.

---

## Cambios Implementados

### 1. Configuración de dependencias opcionales en el Core
#### `Cargo.toml`
* Se agregaron las dependencias de observabilidad de manera opcional:
  * `opentelemetry = { version = "0.32.0", optional = true }`
  * `tracing-opentelemetry = { version = "0.33.0", optional = true }`
* Se introdujo el feature flag `opentelemetry` en la sección `[features]`:
  * `opentelemetry = ["dep:opentelemetry", "dep:tracing-opentelemetry"]`

### 2. Instrumentación de Hot-Paths
Se añadió la macro de instrumentación a métodos críticos del motor, asegurando evitar el overhead de serialización omitiendo campos pesados en los spans.

#### `src/storage.rs`
Se instrumentaron las operaciones críticas de almacenamiento y E/S de datos persistentes:
* `get` 👉 `#[tracing::instrument(skip(self), level = "debug", err)]`
* `insert` 👉 `#[tracing::instrument(skip(self, node), level = "debug", err)]`
* `delete` 👉 `#[tracing::instrument(skip(self), level = "debug", err)]`
* `flush` 👉 `#[tracing::instrument(skip(self), level = "info", err)]`

#### `src/index.rs`
Se instrumentaron los hot-paths de recorrido y mutaciones del grafo HNSW:
* `search_nearest` 👉 `#[tracing::instrument(skip(self, query_vector, vector_store), level = "debug")]`
* `insert` 👉 `#[tracing::instrument(skip(self, vector, vector_store), level = "debug")]`

Se incluyó también el fix para el manejo de vectores nulos (zero-norm) con métrica Cosine en HNSW, redirigiéndolo correctamente a Euclidean para evitar fallas lógicas.

---

## Resultados de Verificación

### 1. Compilación
* Compilación básica sin features:
  ```powershell
  cargo check --workspace --release
  ```
  👉 **Resultado:** Exitosa (0 errores).
* Compilación de producción con feature flag de telemetría:
  ```powershell
  cargo check --workspace --features opentelemetry --release
  ```
  👉 **Resultado:** Exitosa (0 errores).

### 2. Tests Unitarios e Integración en Rust
* Ejecución de las pruebas de integridad:
  ```powershell
  cargo test --package vantadb --release
  ```
  👉 **Resultado:** Pasando al 100%.

### 3. Tests del SDK de Python
Después de reconstruir la extensión nativa del SDK de Python usando el ejecutable de `maturin` ubicado en el entorno virtual del proyecto:
```powershell
& "vantadb-python/.venv/Scripts/maturin.exe" develop --release --manifest-path vantadb-python/Cargo.toml
```
Se ejecutó la suite de tests de Python:
```powershell
& "vantadb-python/.venv/Scripts/python.exe" -m pytest vantadb-python/tests/test_sdk.py -v
```
👉 **Resultado:** **18 passed in 4.37s** (100% de éxito).
* Se verificó el funcionamiento correcto de `search_batch` en paralelo con Rayon (GIL liberado).
* Se validó que `process_rss_bytes` se reporta correctamente en el diccionario devuelto por `hardware_profile`.

---

## Snapshot Guardado
Los artefactos correspondientes a este hito han sido preparados para archivado histórico.
