---
title: "CRDT"
type: glossary-entry
status: stable
tags: [concept, distributed, convergence, crdt]
last_reviewed: 2026-09-15
links: "[[README.md]]"
aliases: [CRDT, Conflict-free Replicated Data Types]
description: "Conflict-free Replicated Data Types: data structures that converge without coordination; relevant only for future multi-node VantaDB."
---

# CRDT

## Definition

Data types whose concurrent updates merge deterministically toward the same state.

## VantaDB context

Distributed convergence is out of scope for embedded single-writer VantaDB; tracked for multi-node futures only.
