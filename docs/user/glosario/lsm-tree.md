---
title: LSM-Tree—Log-Structured Merge-Tree
kind: glossary
status: stable
description: "##Definition"
aliases: [Log-Structured Merge-Tree, LSM Tree]
tags: [storage, lsm, estructura-datos, write-optimized]
links: "[[README.md]]"
---

# LSM-Tree—Log-Structured Merge-Tree

##Definition

A **LSM-Tree** (Log-Structured Merge-Tree) is a data structure optimized for **sequential writes**, which maintains data in memory (MemTable) and periodically dumps it to disk in immutable files (SSTables), with background compaction to maintain read performance.

## How It Works

### General Architecture

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

## Advantages and Disadvantages

### Advantages
- **Fast writes** (append-only, sequential)
- **Write amplification low** vs B-trees
- **Efficient compression** (immutable SSTables)
- **Scalability** for datasets > RAM

### Disadvantages
- **Read amplification** (read searches at multiple levels)
- **Write amplification** by compaction
- **Variable latency** during compaction

## Usage in VantaDB

VantaDB uses LSM-trees via:
- **[fjall](./fjall.md)** — Backend default (100% Rust)
- **[rocksdb](./rocksdb.md)** — Alternative backend (C++)

## See Also

- [fjall](./fjall.md) — LSM-tree implementation in Rust
- [rocksdb](./rocksdb.md) — LSM-tree implementation in C++
- [wal](./wal.md) — Durability for writes
- [mvcc](./mvcc.md) — Concurrency in LSM-trees

---

*LSM-trees are the underlying storage structure of VantaDB.*

