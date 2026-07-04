---
title: Serde
type: glossary-entry
status: active
tags: [serialization, rust, core]
last_reviewed: 2026-07-03
aliases: [Serde, serde]
description: "Rust's premier serialization and deserialization framework used across VantaDB."
links: "[[README.md]]"
---

# Serde

Serde is a powerful framework for serializing and deserializing Rust data structures efficiently and generically. 

## Usage in VantaDB

VantaDB derives Serde traits (`Serialize`, `Deserialize`) for almost all of its public and internal data structures. This allows these structures to be easily converted into JSON for the HTTP API, or into [[bincode|Bincode]] for the [[wal|Write-Ahead Log]] and disk storage.

Serde's zero-copy deserialization is heavily utilized to minimize memory allocations when reading massive amounts of vectors from disk.

## See Also
- [[bincode|Bincode]]
- [[ARCHITECTURE.md]]
