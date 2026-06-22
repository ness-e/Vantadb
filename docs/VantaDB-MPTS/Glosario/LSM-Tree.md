---
type: glosario-entry
status: stable
tags: [storage, lsm, estructura-datos, write-optimized]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
aliases: [Log-Structured Merge-Tree, LSM Tree]
description: "Estructura de datos optimizada para escrituras secuenciales que mantiene datos en memoria (MemTable) y los vuelca periódicamente a disco en archivos inmutables (SSTables)"
---

# LSM-Tree — Log-Structured Merge-Tree

## Definición

Un **LSM-Tree** (Log-Structured Merge-Tree) es una estructura de datos optimizada para **escrituras secuenciales**, que mantiene datos en memoria (MemTable) y los vuelca periódicamente a disco en archivos inmutables (SSTables), con compactación en background para mantener performance de lectura.

## Cómo Funciona

### Arquitectura General

```
┌─────────────────────────────────────┐
│         Escrituras                   │
│  (append-only, secuencial)          │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│         MemTable (en RAM)            │
│  - Estructura ordenada (SkipList)   │
│  - Writes rápidos (O(log N))        │
│  - Tamaño límite: 16-64 MB          │
└──────────────┬──────────────────────┘
               │
               │ flush (cuando MemTable está llena)
               ▼
┌─────────────────────────────────────┐
│         SSTables (en disco)          │
│  Level 0: [SST1] [SST2] [SST3]      │
│           (overlapping keys)        │
│                                     │
│  Level 1: [SST4] [SST5] [SST6]      │
│           (non-overlapping)         │
│                                     │
│  Level 2: [SST7] ... [SST20]        │
│           (non-overlapping)         │
└─────────────────────────────────────┘
```

## Ventajas y Desventajas

### Ventajas
- **Escrituras rápidas** (append-only, secuencial)
- **Write amplification baja** vs B-trees
- **Compresión eficiente** (SSTables inmutables)
- **Escalabilidad** para datasets > RAM

### Desventajas
- **Read amplification** (lectura busca en múltiples niveles)
- **Write amplification** por compactación
- **Latencia variable** durante compactación

## Uso en VantaDB

VantaDB usa LSM-trees a través de:
- **[Fjall](Fjall.md)** — Backend default (100% Rust)
- **[RocksDB](RocksDB.md)** — Backend alternativo (C++)

## Véase También

- [Fjall](Fjall.md) — Implementación LSM-tree en Rust
- [RocksDB](RocksDB.md) — Implementación LSM-tree en C++
- [WAL](WAL.md) — Durabilidad para writes
- [MVCC](MVCC.md) — Concurrencia en LSM-trees

---

*LSM-trees son la estructura de almacenamiento subyacente de VantaDB.*

