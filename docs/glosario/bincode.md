---
title: "Bincode"
type: glossary-entry
status: stable
tags: [concept, serialization, rust, bincode]
last_reviewed: 2026-09-15
links: "[[README.md]]"
aliases: [Bincode]
description: "Compact binary serialization format for Rust; used with serde for WAL and index-state persistence."
---

# Bincode

## Definition

Dense binary encoding for Rust values, paired with serde derives.

## VantaDB context

Serialization pair `[[serde]]` + `[[bincode]]` persists WAL records, index metadata and storage structures (see `[[serialization]]`).
