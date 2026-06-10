# Experimental Feature Archive: Quarantine 2024-06

**Date Archived:** June 10, 2026  
**Reason:** Runtime governance approach abandoned due to architectural incompatibility  

---

## Context

These crates were originally developed to provide runtime policy evaluation and governance capabilities for VantaDB. However, this approach proved fundamentally incompatible with the project's direction toward compile-time governance via IQL AST Pass.

### Why Runtime Governance Failed

**Technical Issues:**
1. **Borrow Checker Panics:** Runtime mutation of graph views in `MmapMut` caused Rust borrow checker panics
2. **GIL Blocking:** LISP runtime introduced thread contention that blocked Python's GIL
3. **Performance Overhead:** Dynamic policy evaluation added unpredictable latency in embedded systems
4. **Complexity vs Benefit:** Runtime policy languages added configuration complexity without clear user demand

**Solution Adopted:**
- Compile-time governance via IQL AST Pass
- Static policy validation before memory operations
- Zero runtime overhead for policy enforcement
- Predictable performance in embedded scenarios

---

## Archived Crates

### `experimental-lisp`
- **Purpose:** LISP query language and VM for dynamic policy evaluation
- **Status:** PoC (Proof of Concept) - Only `INSERT` implemented, `MATCH` pending
- **Issues:** 
  - Designed for runtime evaluation (incompatible with compile-time governance)
  - Fuel limits are hacks, not real solutions
  - Random node generation not deterministic
- **Components:**
  - LISP Parser (nom-based)
  - Stack-based VM with bytecodes
  - Sandbox with fuel limits (MAX_FUEL=1000)

### `experimental-governance`
- **Purpose:** Runtime admission control, conflict resolution, and maintenance workers
- **Status:** Functional but architecturally incompatible
- **Issues:**
  - Direct dependency on StorageEngine internals (tight coupling)
  - Thread-dedicated maintenance worker (anti-pattern for embedded systems)
  - Assumes runtime conflict resolution (incompatible with AST-based governance)
- **Components:**
  - `AdmissionFilter`: Bloom filter for duplicate prevention
  - `ConflictResolver`: Confidence arbitration with friction metrics
  - `ConsistencyBuffer`: Runtime conflict resolution buffer
  - `MaintenanceWorker`: Background thread for eviction/consolidation
  - `InvalidationDispatcher`: MPSC channel for invalidation events

---

## Extracted & Preserved Utilities

The following useful components were extracted and integrated into the core:

### `src/utils/duplicate_prevention.rs`
- Extracted from: `experimental-governance/src/admission_filter.rs`
- Purpose: Bloom filter for duplicate prevention in multi-writer scenarios
- Status: ✅ Integrated into core as `DuplicatePreventionFilter`

### `src/utils/confidence_metrics.rs`
- Extracted from: `experimental-governance/src/conflict_resolver.rs`
- Purpose: Collision tracking and friction metrics for multi-agent coordination
- Status: ✅ Integrated into core as `OriginCollisionTracker` and `compute_confidence_friction`

---

## References

- **Walkthrough:** `docs/progreso/cuarentena-experimental/walkthrough.md`
- **Implementation Plan:** `docs/progreso/cuarentena-experimental/implementation_plan.md`
- **Plan Maestro:** `VantaDB_Plan_Maestro_Unificado.md` (sections on experimental features)
- **Experimental Features Doc:** `docs/operations/EXPERIMENTAL_FEATURES.md`

---

## Future Considerations

If runtime governance is reconsidered in the future:
1. Start from scratch with event-driven architecture (no dedicated threads)
2. Design for compile-time integration, not runtime evaluation
3. Use interface-based design to avoid tight coupling with StorageEngine
4. Prioritize simplicity and predictability over expressiveness
5. Validate against embedded systems constraints (no blocking operations, deterministic behavior)

---

**Archived by:** Automated cleanup process  
**Preservation:** Historical reference only - not intended for reuse