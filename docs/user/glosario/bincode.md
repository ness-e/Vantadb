---
title: Bincode
kind: glossary
status: stable
description: "Legacy dense binary encoding for Rust values, paired with serde derives. VantaDB"
aliases: [Bincode]
tags: [concept, serialization, rust, bincode]
links: "[[README.md]]"
---

# Bincode

## Definition

Legacy dense binary encoding for Rust values, paired with serde derives. **VantaDB
no longer depends on it** — the `bincode` crate was replaced by
[postcard](./postcard.md) (declared in `Cargo.toml` as `postcard = "1.1"`) and
appears nowhere in the current dependency graph or source.

## VantaDB context

Historically the serialization pair was `serde` + `bincode` for WAL records,
index metadata and storage structures. The current pair is `serde` +
`[[postcard]]` (see `[[serialization]]`), and the wire format version is tracked
by `WAL_POSTCARD_VERSION` / `WAL_FORMAT_VERSION` (`src/wal.rs`, re-exported from
the crate root).

This page is kept as a stub so existing `[[bincode]]` links resolve; the
authoritative format reference is [serialization](./serialization.md).
