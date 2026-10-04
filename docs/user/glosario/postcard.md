---
title: Postcard
kind: glossary
status: stable
description: "Compact serde-based binary format used by VantaDB for WAL records, index metadata and SDK structures."
aliases: [Postcard]
tags: [concept, serialization, rust, postcard]
links: "[[README.md]]"
---

# Postcard

## Definition

[Postcard](https://docs.rs/postcard) is a compact binary serialization format
for Rust built on `serde`. It is the dense successor to `bincode`: same
derive-based ergonomics, smaller output and a stable wire specification.

## VantaDB context

`postcard` is the current serialization half of the on-disk pair, declared in
the workspace `Cargo.toml` as `postcard = { version = "1.1", features =
["alloc", "use-std"] }`. It replaced `bincode` for WAL records, index metadata
and storage structures:

- **WAL records and index state** - serialized with `postcard::to_allocvec`
  (`src/text_index.rs`, `src/node/edge.rs`).
- **Migration snapshots** - versioned reads/writes via `postcard` in
  `src/migration.rs`.
- **SDK structures** - the `serde` + `postcard` pair is the canonical encoding
  ([serialization](./serialization.md)).

Wire format changes are governed by
[STORAGE_VERSIONING](../../dev/architecture/STORAGE_VERSIONING.md).

## See Also

- [serde](./serde.md) - The derive framework both formats build on
- [bincode](./bincode.md) - The legacy dense format postcard replaced
- [serialization](./serialization.md) - VantaDB's serialization overview
- [wal](./wal.md) - Main consumer of the postcard encoding
