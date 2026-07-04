---
title: Bincode
type: glossary-entry
status: active
tags: [serialization, storage, core]
last_reviewed: 2026-07-03
aliases: [Bincode, bincode]
description: "Binary serialization format used internally by VantaDB for efficient state and WAL persistence."
links: "[[README.md]]"
---

# Bincode

Bincode is a highly efficient binary serialization format used throughout VantaDB's core, especially in the [[wal|WAL]] (Write-Ahead Log) and when flushing index state to disk.

## Why Bincode?

Unlike JSON or YAML, Bincode stores data in a compact binary representation without field names or metadata, making it exceptionally fast to serialize and deserialize. This is critical for database operations where memory layout and disk I/O are the primary bottlenecks.

In VantaDB, Bincode is used in conjunction with [[serde|Serde]] to persist structs like `VantaMemoryRecord` and index metadata safely and deterministically.

## See Also
- [[serde|Serde]]
- [[wal|Write-Ahead Log]]
- [[ARCHITECTURE.md]]
