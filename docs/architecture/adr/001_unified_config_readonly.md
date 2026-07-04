---
title: "ADR 001: Unified Configuration Architecture and Read-Only Barrier"
type: adr
status: active
tags: [vantadb, architecture, adr]
last_reviewed: 2026-07-01
aliases: []
---

# ADR 001: Unified Configuration Architecture and Read-Only Barrier

## Status

Status: Approved

## Context

In the MVP versions of VantaDB, configuration options were fragmented across multiple mutable interfaces (e.g., `VantaOpenOptions`), allowing structural engine parameters to be modified at runtime. This inconsistent mutability caused internal consistency problems and corruption risks.
Additionally, write operations (such as inserts, updates, and deletes) would proceed deep into the storage engine pipeline before validating the operational state of the database (e.g., whether the engine was opened in `read_only` mode), incurring unnecessary CPU costs, memory allocation, and lock contention before failing.

## Decision

1. **Structural Consolidation:** Unify all initialization options into a single, immutable, strongly-typed structure called `VantaConfig`. All SDK and server entry points consume this consolidated configuration at engine open time (`open_with_config`).

2. **Early Entry Barrier (Fail-Fast):** Implement the `guard_write_allowed(&self.config)` guard function in the public storage API. This routine immediately evaluates the write-protection flag:

   ```rust
   pub fn guard_write_allowed(config: &VantaConfig) -> Result<()> {
       if config.read_only {
           return Err(VantaError::ReadOnlyViolation);
       }
       Ok(())
   }
   ```

3. **Inject into Mutators:** Invoke this barrier as the first line of execution in all methods that alter the logical or physical state of the engine: `insert`, `put`, `delete`, `add_edge`, and `flush`.

## Consequences

### Benefits

* **Predictable Isolation:** The engine aborts unauthorized operations in $O(1)$ time without acquiring read/write locks or opening transactions in underlying engines (RocksDB, Fjall), preventing resource degradation.
* **Thread Safety:** Since `VantaConfig` is completely immutable after initialization, any possibility of data races or inconsistencies from hot reconfiguration in highly concurrent environments is eliminated.
* **API Simplification:** Redundant APIs and deprecated methods are removed, improving maintainability for clients integrating VantaDB as an embedded engine.

### Technical Debt / Costs

* **Mandatory Re-initialization:** If a node needs to change operational state (from read-only replica to read-write primary), the engine instance must be explicitly closed and reopened with a new `VantaConfig` structure. This is considered desirable behavior in industrial distributed system architectures.
