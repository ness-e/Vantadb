# Walkthrough: Fase 2 — Mecanismo RCU / Double-Buffer en Memoria para Reconstrucción de Índices (AUD-03)

## Objetivo

Eliminar la contención en el camino de lectura (*hot path*) del índice vectorial HNSW (`CPIndex`) de VantaDB durante las operaciones de reconstrucción y compactación (`rebuild_vector_index` / `compact_layout_bfs`). Esto se logra migrando de un esquema tradicional basado en bloqueos de exclusión mutua (`RwLock<CPIndex>`) a un esquema de concurrencia optimista tipo **RCU (Read-Copy-Update)** en memoria usando `ArcSwap`.

---

## Cambios Realizados

### 1. Integración de Dependencias
- **[Cargo.toml](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/Cargo.toml)**
  - Agregada la dependencia `arc-swap = "1.7"`.

### 2. Migración del Core de HNSW a RCU
- **[storage.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/storage.rs)**
  - Migrada la propiedad `pub hnsw: RwLock<CPIndex>` a `pub hnsw: ArcSwap<CPIndex>`.
  - Inicializado `self.hnsw` usando `ArcSwap::from_pointee(...)` en `StorageEngine::open_with_config`.
  - Reemplazadas todas las llamadas internas de `.hnsw.read()` y `.hnsw.write()` por `.hnsw.load()`.

### 3. Implementación de Mitigaciones Críticas de Concurrencia

#### Mitigación A-01: Prevención de Pérdida de Mutaciones
- Adquirida la protección exclusiva `insert_lock.lock()` al inicio de `rebuild_vector_index` y `compact_layout_bfs`. Esto bloquea nuevos escritores de forma cooperativa durante la construcción de la copia del índice, asegurando que ninguna inserción concurrente se pierda o sea sobrescrita por el swap del Arc pointer.

#### Mitigación A-02: Scopes de Guarda Acotados (Descarte Rápido)
- **[sdk.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/sdk.rs)** y **[physical_plan.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/physical_plan.rs)**:
  - Modificadas las consultas para limitar el tiempo de vida del `ArcSwapGuard` a un scope local mínimo. Los lectores obtienen un puntero rápido al índice y liberan el guard en milisegundos, previniendo fugas de memoria o retención excesiva de copias antiguas.

#### Mitigación A-03: Persistencia en Disco antes de Swap
- En `save_vector_index` (usado por `rebuild_vector_index`), la serialización y mmap del índice en disco se realiza de forma atómica antes de actualizar la referencia en memoria:
  1. Guardar bytes en `.bin.tmp`.
  2. Mapear en memoria e inicializar el nuevo backend.
  3. Renombrar en disco atómicamente a `vector_index.bin`.
  4. Realizar el swap en memoria: `self.hnsw.store(Arc::new(rebuilt))`.

---

## Migración de la Suite de Pruebas de Integración

Se actualizaron los archivos de prueba para reemplazar las llamadas legacy `.hnsw.read()` por `.hnsw.load()`, solucionando también los problemas de inferencia de tipo asociados al iterador de nodos de DashMap con `ArcSwap`:
- [antilocality_layout.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/tests/storage/antilocality_layout.rs)
- [wal_resilience.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/tests/storage/wal_resilience.rs)
- [tombstone_ann_vstore.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/tests/storage/tombstone_ann_vstore.rs)
- [memory_telemetry.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/tests/memory_telemetry.rs)
- [index_reconstruction.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/tests/index_reconstruction.rs)
- [durability_recovery.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/tests/durability_recovery.rs)
- [vector_scale_check.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/tests/core/vector_scale_check.rs)

---

## Verificación

Se añadieron pruebas de estrés de concurrencia y se validó la suite completa.

### 1. Test Específico de Estrés RCU
Se implementó `test_concurrency_rebuild_rcu` en [concurrency_parity.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/tests/concurrency_parity.rs). Este test:
- Lanza 100 hilos concurrentes que realizan búsquedas vectoriales ininterrumpidas sobre el HNSW.
- De manera simultánea, ejecuta ciclos repetidos de `rebuild_vector_index`.
- Valida que **no se produce ningún pánico, bloqueo (*deadlock*) ni inconsistencia de datos**, demostrando que los lectores operan 100% libres de locks sin verse afectados por el swap atómico del índice.

### 2. Resultados de las Pruebas

| Suite de Test | Estado | Resultado / Tiempo | Detalle |
|---|---|---|---|
| **concurrency_parity.rs** | ✅ PASADO | `ok` (~4.23s) | Certificó `test_concurrency_rebuild_rcu` libre de locks y pánicos |
| **index_reconstruction.rs** | ✅ PASADO | `ok` (~1.61s) | Reconstrucción en frío del índice ante pérdida del archivo binario |
| **durability_recovery.rs** | ✅ PASADO | `ok` (~1.39s) | Recuperación correcta desde el WAL tras apagado no estructurado |
| **memory_telemetry.rs** | ✅ PASADO | `ok` (~29.57s) | Validación de telemetría de memoria de HNSW |
| **tombstone_ann_vstore.rs** | ✅ PASADO | `ok` (~0.65s) | Exclusión de registros lógicos eliminados del índice HNSW |
| **wal_resilience.rs** | ✅ PASADO | `ok` (~0.63s) | Auto-sanado y scan-forward en presencia de corrupción del WAL |
| **vector_scale_check.rs** | ✅ PASADO | `ok` (~228.93s) | Estabilidad y navegación del HNSW con 1,000 nodos (en modo Debug) |
| **antilocality_layout.rs** | ✅ PASADO | `ok` (~5.11s) | Reorganización BFS del layout físico en mmap |
