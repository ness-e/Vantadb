# Eviction Policy & Garbage Collection

> **Fase 13 · Carpeta 15_TTL_GC · Semana 29-30**

Este documento detalla la política de expiración de tiempo de vida (TTL) y el colector de basura (GC) en segundo plano del IADBMS, diseñado para liberar memoria de nodos obsoletos (por ejemplo, interacciones antiguas de agentes).

## 1. Arquitectura de TTL

El motor rastrea el tiempo de vida (TTL) de nodos individuales usando un árbol optimizado para rangos temporales:

```rust
BTreeMap<u64, Vec<u64>>  // Timestamp -> [NodeID1, NodeID2, ...]
```
El uso de un `BTreeMap` permite encontrar todos los nodos expirados (donde `timestamp < now`) con complejidad `O(log N)` y recorrerlos en un escaneo secuencial súper rápido.

## 2. Ciclo de Vida de Eviction (Protocolo de Purga)

El proceso de recolección de basura sigue una política conservadora y segura para evitar *race conditions* con el motor de queries:

1. **Marcado (Soft Deletion):** El `GcWorker` despierta cada configuración predeterminada de tiempo y busca en el `BTreeMap` los nodos expirados y los marca para borrado. Si el nodo aún tiene referencias ativas en caché, no se borra de inmediato.
2. **Purga Física (Hard Deletion):** El motor instruye al `StorageEngine` para realizar la eliminación en disco (RocksDB). Esto incluye borrar la data relacional y las aristas (generando cascada en el grafo cuando aplique, según reglas estrictas).
3. **Escritura en WAL:** El proceso `StorageEngine::delete(id)` emite un `WalRecord::Delete { id }` para garantizar que la base de datos no recupere datos muertos tras un crasheo.

## 3. Worker Background con Tokio

El proceso corre en segundo plano usando un `tokio::spawn`:
- **Intervalo de Purga:** 60 segundos por defecto (baja latencia, poco impacto).
- **Control concurrente:** Requiere `Arc<RwLock<GcState>>` o compartir la instancia del `StorageEngine` y mantener sincronización interna de las colas de borrado.

## 4. Métricas a reportar
El módulo de GC expone contadores que deberían anexarse al registro general de Prometheus (Fase 12):
- `gadbage_collection_sweeps_total`
- `nodes_purged_total`
- `gc_latency_ms` (histograma)
