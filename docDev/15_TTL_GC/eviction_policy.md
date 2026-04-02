# TTL & Garbage Collection (State Eviction)
> **Status**: 🟡 In Progress — FASE 13

## 1. Context Accumulation 
Agent workflows (like continuous autonomous chains) generate thousands of context vectors over a span of days. Without eviction, memory usage breaches the 16GB RAM limit, even though historical vectors lose relevance.

## 2. Ephemeral Nodes
IADBMS supports assigning a `Time-To-Live (TTL)` value to `UnifiedNode` properties. During insertion, the exact expiration timestamp is recorded.

## 3. Passive vs Active Eviction
- **Active (Background Sweeper):** A background Tokio task `GcWorker` awakes every 60 seconds. It iterates through an index mapping `Expiration -> NodeId`, proactively purging RocksDB and memory buffers.
- **Passive (Lazy):** If a query traverses a node past its expiry time, the Engine actively refuses to include it in the physical result and triggers an immediate localized deletion.
