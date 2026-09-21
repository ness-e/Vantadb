# TBH-17 — Evaluar `loom` para nuevas primitivas de concurrencia (DOC-ONLY)

## Estado
- ✅ Plan: `docs/plans/2026-08-30-testing-bench-harden.md`
- 🔄 In progress
- Pendiente commit

## Fase DISCOVERY ✅

- `git grep "loom" Cargo.toml` → 0 hits ✅
- `git grep "shuttle" Cargo.toml` → 0 hits ✅ (alternativa evaluada también descartada)
- `tests/concurrency_parity.rs` existe y cubre:
  - `test_triple_backend_parity_validation` (RocksDB ↔ Fjall ↔ InMemory con 1000 nodos)
  - `test_high_concurrency_fjall_stress` (10 writers × 100 ops, valida integridad)
  - `test_interleaved_read_write_parity` (reader + writer simultáneos)
  - `test_concurrency_rebuild_rcu` (RCU lock-free swap durante rebuild de HNSW)
- `dev-dependencies` activas para testing de concurrencia: `tokio`, `serial_test`, `criterion`, `proptest`. **`loom` ausente.**

## Decisión

**NO agregar `loom` al workspace.** Justificación:

1. **YAGNI + decisión D5** ("justificar cada dep antes de añadir"): el audit multi-agente del
   2026-08-30 no encontró necesidad inmediata de nuevas primitivas de concurrencia que
   requieran model checking exhaustivo.

2. **El testing actual cubre las superficies de riesgo reales:**
   - Parity entre backends (RocksDB/Fjall/InMemory)
   - Write stress (10 threads concurrentes)
   - R/W interleaving bajo contención
   - RCU lock-free rebuild (la única primitiva verdaderamente nueva en el periodo)

3. **Coste de `loom`:**
   - Recompilación sustancial (proc-macros que interceptan `std::sync::*`).
   - Model checking exhaustive = state explosion; el harness actual ya valida comportamiento
     bajo contención real.
   - El codebase usa primitivas probadas (`parking_lot`, `arc-swap`, `dashmap`, `portable-atomic`),
     no locks caseros nuevos que necesiten verificación exhaustiva.

4. **Si en el futuro se introduce una primitiva no trivial** (lock-free queue custom, epoch
   reclamation manual, custom RCU), se reevaluará `loom` puntualmente para esa superficie.
   Documentado en el ADR-like doc: `docs/research/concurrency-testing-2026-08-30.md`.

## Fase EJECUCIÓN (pendiente)

1. Crear `docs/research/concurrency-testing-2026-08-30.md` con la decisión y razonamiento.
2. Commit: `docs(TBH-17): record loom evaluation decision (not introduced; current concurrency_parity.rs sufficient)`.

## Cierre (pendiente)

- Verify `docs/research/concurrency-testing-2026-08-30.md` existe y matchea "loom".
- `git add` + commit.
- `campaign_update_task_state` → `completed`.