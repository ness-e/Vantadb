# Walkthrough — Completación de T2.2 (mimalloc + RSS) y T2.4 (Headers Binarios)

Hemos completado la integración y verificación de las tareas **T2.2** y **T2.4** del Plan Maestro Unificado. Este documento resume los cambios realizados, las estrategias de mitigación de fragmentación y los resultados de las pruebas de estabilidad de memoria.

## Cambios Realizados

### 1. Versionado de Formato Binario (T2.4)
- **Implementación**: Integrado un header estructurado uniforme `VantaHeader` (16 bytes) para validar la compatibilidad y formato en los archivos:
  - `vector_index.bin`
  - Snapshots de la base de datos
  - Archivos del Write-Ahead Log (WAL) (`WalHeader` con firma CRC32C).
- **Alineación Zero-Copy**: Modificado `VantaFile` en `src/storage.rs` para reservar los primeros 16 bytes para el header y alinear el primer descriptor de nodo a los **64 bytes** (alineación requerida para casting directo zero-copy en memoria).
- **Control de Errores**: Se lanza la excepción explícita `VantaError::IncompatibleFormat` si los magic bytes o la versión del formato leída no coinciden con los esperados.

### 2. Integración de mimalloc y Telemetría de Memoria (T2.2)
- **Global Allocator**: Configurado condicionalmente el asignador `mimalloc` en `vanta-cli` and `vantadb-server` a través de la feature flag `custom-allocator` para mitigar la fragmentación a largo plazo.
- **Unificación de Métricas**: Modificado `operational_metrics` en `src/sdk.rs` para que antes de tomar el snapshot refresque las métricas de memoria real consultando al sistema operativo.
- **Python Bindings**: Actualizado el método `hardware_profile()` en el Python SDK para que combine las capacidades básicas con el desglose de métricas operacionales de memoria, retornando un JSON estructurado con:
  - `process_rss_bytes` (RSS físico real detectado por el OS).
  - `process_virtual_bytes` (Memoria virtual del proceso).
  - `hnsw_logical_bytes` (Footprint lógico estimado de la estructura HNSW).
  - `hnsw_nodes_count` (Cantidad de nodos en el índice).
  - `mmap_resident_bytes` (Páginas residentes reales mapeadas en memoria).
  - `volatile_cache_entries` y `volatile_cache_cap_bytes`.

### 3. Test de Estrés de RSS (CI + Manual)
- **Test de CI**: Añadido el bloque `"RSS Stability Under Load"` en `tests/certification/hardware_profiles.rs` que calienta el motor con 20K inserciones y añade otras 80K bajo una aserción de deriva de RSS controlada (drift ratio <= 4.0, tolerando el working set de Windows y el caché de páginas de archivos mapeados).
- **Test Manual**: Creado el documento operacional [RELIABILITY_GATE.md](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/docs/operations/RELIABILITY_GATE.md) con un script en Python para medir la deriva a largo plazo durante 30 minutos de estrés y validar un incremento residual final de RSS < 10% (estabilidad de mimalloc contra fugas del heap).

---

## Resultados de Verificación y Pruebas

### 1. Test de Certificación de Rust (`hardware_profiles`)
- **Ejecución**: `cargo test --test hardware_profiles --release -- --nocapture`
- **Métricas de Estabilidad**:
  - **RSS Warmup (20K)**: ~66.76 MB
  - **RSS Final (100K)**: ~216.61 MB
  - **Ratio de Deriva**: ~3.24 (224% de incremento físico de RAM, dentro del límite de working set de Windows para el caché del sistema de archivos mapeado de 40MB netos de vectores).
  - **Resultado**: Pasa la validación de deriva del working set y estabilidad física del core.

### 2. Tests de Integración Python SDK
- **Test**: `pytest tests/test_sdk.py -v -k "hardware_profile"`
- **Resultado**: El alias `hardware_profile()` mantiene compatibilidad regresiva completa y retorna exitosamente las nuevas métricas físicas del motor (como `process_rss_bytes` > 0).
