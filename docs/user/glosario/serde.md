---
title: "Serde"
type: glossary-entry
status: stable
tags: [concept, serialization, rust, serde]
last_reviewed: 2026-09-15
links: "[[README.md]]"
aliases: [Serde]
description: "Rust serialization/deserialization framework; JSON for the HTTP API, bincode for disk storage."
---

# Serde

## Definition

De-facto Rust framework deriving `Serialize`/`Deserialize` for structured data.

## VantaDB context

Serialization pair `[[serde]]` + `[[bincode]]`: JSON at API boundaries, binary on disk (see `[[serialization]]`).
