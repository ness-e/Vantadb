# Telemetría de Memoria, Baselines y Contratos de Datos

Este documento define el contrato de observabilidad de memoria de VantaDB, las métricas operacionales expuestas por Prometheus y el baseline histórico de la persistencia de datos en memoria.

---

## 📊 1. Desglose de Telemetría por Subsistema

VantaDB reporta el uso de memoria a nivel del host (hardware total) y a nivel del proceso (internos del motor y RSS del sistema operativo).

| Métrica | Origen en Código | Unidad | Propósito / Alcance |
| :--- | :--- | :--- | :--- |
| `HardwareCapabilities::total_memory` | `sysinfo::System::total_memory()` | Bytes | Capacidad de memoria del host. |
| `process_rss_bytes` | `sysinfo::Process::memory()` | Bytes | Memoria física residente del proceso. |
| `process_virtual_bytes` | `sysinfo::Process::virtual_memory()` | Bytes | Memoria virtual asignada al proceso. |
| `hnsw_nodes_count` | `CPIndex::nodes.len()` | Conteo | Nodos cargados en el índice vectorial. |
| `hnsw_logical_bytes` | `CPIndex::estimate_memory_bytes()` | Bytes | Estimación lógica determinista del grafo. |
| `mmap_resident_bytes` | Syscalls `mincore` (Unix) / `QueryWorkingSetEx` (Win) | Bytes | Páginas residentes de archivos mapeados. |
| `volatile_cache_entries` | `volatile_cache.len()` | Conteo | Entradas activas en la caché LRU. |

### Métricas de Prometheus

Los siguientes gauges son registrados en `METRICS_REGISTRY` y se exponen en `/metrics` en `vantadb-server`:
* `vanta_process_rss_bytes`
* `vanta_process_virtual_bytes`
* `vanta_hnsw_nodes_count`
* `vanta_hnsw_logical_bytes`
* `vanta_mmap_resident_bytes`

---

## 🏗️ 2. Contrato de Datos y Estado Derivado

VantaDB opera con un contrato estructurado en memoria y disco para asegurar consistencia:

* **Identidad (Keys):** Generada a partir de `namespace + "\0" + key`.
* **Payload:** Carga útil en formato UTF-8 serializado.
* **Metadata:** Atributos planos compuestos únicamente por valores del enum `FieldValue` (Strings, Enteros, Flotantes, Booleanos). No se admiten objetos JSON anidados para preservar la eficiencia.
* **Vectores:** Almacenados opcionalmente en formato contiguo de precisión `f32` dentro de `vector_store.vanta`.

### Índices Derivados en Memoria
El motor lee el almacenamiento de clave-valor canónico y materializa en frío las siguientes estructuras que pueden ser reconstruidas en su totalidad mediante `rebuild_index`:
1. **`NamespaceIndex`:** Mapea el prefijo de namespace a identificadores de nodos lógicos.
2. **`PayloadIndex`:** Mapea campos escalares para búsquedas rápidas con filtrado relacional.
3. **`TextIndex`:** Índices invertidos del motor BM25 para búsquedas léxicas FTS.

---

## 🧪 3. Verificación de Telemetría de Memoria

Para realizar mediciones de estabilidad locales y perfilado de consumo de memoria bajo inserciones continuas, ejecuta el arnés de control:

```powershell
# Setear reporte en archivo local
$env:VANTA_CERT_REPORT="target/memory_telemetry.json"
cargo test --test memory_telemetry -- --nocapture
```
Esto valida que la memoria RSS no sufra fugas y que las páginas MMap se liberen correctamente al vaciar el índice a disco.
