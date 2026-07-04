---
title: "Compaction"
type: glossary-entry
status: stable
tags: [glosario, storage, mantenimiento, rendimiento, lsm]
last_reviewed: 2026-07-03
aliases: [compaction, compactación, layout-compaction, compact]
description: "Proceso de reorganización del almacenamiento para recuperar espacio, reducir fragmentación y mantener rendimiento de lectura"
---

# Compaction

## Definición

La **compaction** (compactación) es el proceso de reorganizar los datos en disco para eliminar registros obsoletos, fusionar archivos fragmentados y optimizar el rendimiento de lectura. En VantaDB existen dos tipos principales: compactación de almacenamiento LSM y compactación de layout del vector store.

## Tipos de Compaction en VantaDB

### 1. Compaction de Layout (Vector Store)

Reorganiza los nodos del grafo HNSW en disco siguiendo un orden BFS desde el entry point del índice, agrupando nodos vecinos en regiones contiguas para minimizar page faults durante búsquedas **[[mmap]]**:

```rust
// src/storage/engine.rs
pub fn trigger_compaction(&self) -> Result<()> {
    // Reordena nodos en disco en orden BFS desde HNSW entry point
    // Agrupa vecinos cercanos físicamente para reducir page faults
}

pub fn compact_layout_bfs(&self) -> Result<u64> {
    // Compactación que itera el grafo HNSW en BFS
    // y reescribe los nodos en ese orden
}
```

### 2. Compaction de WAL

Archiva el WAL actual y comienza uno nuevo, eliminando registros de mutación ya aplicados al almacenamiento canónico:

```rust
// src/sdk/api.rs
pub fn compact_wal(&self) -> Result<()> {
    self.check_read_only()?;
    self.engine_handle()?.compact_wal()
}
```

### 3. Compaction LSM (Backend)

Manejo interno del motor de almacenamiento:

| Backend | Estrategia | Manual |
|---------|-----------|--------|
| **[[fjall]]** | Automática (background threads) | No soportada |
| **[[rocksdb]]** | Automática + manual | `request_compaction()` |

```rust
pub fn request_compaction(&self) {
    if !self.supports_manual_compaction() {
        warn!("Backend manages compaction automatically");
        return;
    }
    // RocksDB: trigger manual compaction
}
```

## Cuándo Ejecutar Compaction

| Señal | Acción |
|-------|--------|
| Fragmentación >20% | `trigger_compaction()` |
| WAL crece sin límite | `compact_wal()` |
| Post-import masivo | `compact_layout_bfs()` |
| Mantenimiento programado | `request_compaction()` |

## Véase También

- [[lsm-tree]] — Estructura de almacenamiento subyacente
- [[wal]] — Write-Ahead Log
- [[fjall]] — Backend con compaction automática
- [[rocksdb]] — Backend con compaction manual
- [[mmap]] — Memory-mapped I/O
- [[persistence]] — Persistencia general
