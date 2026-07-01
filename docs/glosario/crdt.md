---
title: CRDT
type: glossary-entry
status: active
tags: [distributed, synchronization, future]
aliases: [CRDT, crdt, CRDTs]
description: "Conflict-free Replicated Data Types, used for state synchronization in multi-node and edge environments."
links: "[[README.md]]"
---

# Conflict-free Replicated Data Types (CRDTs)

CRDTs are data structures that can be replicated across multiple computers in a network, where replicas can be updated independently and concurrently without coordination, and it is mathematically guaranteed that all replicas will eventually converge.

## Role in VantaDB

While VantaDB focuses initially on an [[embedded]] architecture, the roadmap for multi-node scaling and edge federation relies on CRDTs to synchronize metadata, delete tombstones, and index states without requiring a heavy distributed consensus protocol like Raft or Paxos.

## See Also
- [[ROADMAP.md]]
