---
title: Compaction
kind: glossary
status: stable
description: "Proceso de reorganizaci├│n del almacenamiento para recuperar espacio, reducir fragmentaci├│n y mantener rendimiento de lectura"
aliases: [compaction, compactaci├│n, layout-compaction, compact]
tags: [glosario, storage, mantenimiento, rendimiento, lsm]
type: glossary-entry
last_reviewed: "2026-09-15"
---


# Compaction

## Definici├│n

La **compaction** (compactaci├│n) es el proceso de reorganizar los datos en disco para eliminar registros obsoletos, fusionar archivos fragmentados y optimizar el rendimiento de lectura. En VantaDB existen dos tipos principales: compactaci├│n de almacenamiento LSM y compactaci├│n de layout del vector store.

## Tipos de Compaction en VantaDB

### 1. Compaction de Layout (Vector Store)

Reorganiza los nodos del grafo HNSW en disco siguiendo un orden BFS desde el entry point del ├¡ndice, agrupando nodos vecinos en regiones contiguas para minimizar page faults durante b├║squedas **[mmap](./mmap.md)**:

```rust
// src/storage/engine/maintenance.rs
pub fn trigger_compaction(&self) -> Result<()> {
    // Mide la fragmentaci├│n por tombstones contra
    // segment_optimizer.vacuum_threshold_pct (default 15%) y delega a
    // merge_segments(), que compacta v├¡a compact_layout_bfs().
}

pub fn compact_layout_bfs(&self) -> Result<u64> {
    // Compactaci├│n que itera el grafo HNSW en BFS
    // y reescribe los nodos en ese orden
}
```

### 2. Compaction de WAL

Archiva el WAL actual y comienza uno nuevo, eliminando registros de mutaci├│n ya aplicados al almacenamiento can├│nico:

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
| **[fjall](./fjall.md)** | Autom├ítica (background threads) | No soportada |
| **[rocksdb](./rocksdb.md)** | Autom├ítica + manual | `request_compaction()` |

```rust
pub fn request_compaction(&self) {
    if !self.supports_manual_compaction() {
        warn!("Backend manages compaction automatically");
        return;
    }
    // RocksDB: trigger manual compaction
}
```

## Cu├índo Ejecutar Compaction

| Se├▒al | Acci├│n |
|-------|--------|
| Fragmentaci├│n > `vacuum_threshold_pct` (default 15%) | `trigger_compaction()` |
| WAL crece sin l├¡mite | `compact_wal()` |
| Post-import masivo | `compact_layout_bfs()` |
| Mantenimiento programado | `request_compaction()` |

## V├⌐ase Tambi├⌐n

- [lsm-tree](./lsm-tree.md) ΓÇö Estructura de almacenamiento subyacente
- [wal](./wal.md) ΓÇö Write-Ahead Log
- [fjall](./fjall.md) ΓÇö Backend con compaction autom├ítica
- [rocksdb](./rocksdb.md) ΓÇö Backend con compaction manual
- [mmap](./mmap.md) ΓÇö Memory-mapped I/O
- [persistence](./persistence.md) ΓÇö Persistencia general
