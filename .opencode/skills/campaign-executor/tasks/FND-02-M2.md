# FND-02-M2: Stress test de evicción *_locked bajo contención

## Metadata
- **Plan:** docs/plans/2026-08-16-wave-followups.md (W2)
- **Fuente:** minor 2 del audit P2-01 de FND-02 (el stress test existente nunca dispara la evicción: 192 nodos vs max_nodes ~2.7M)
- **Estado:** ⏳ IN PROGRESS · **Sub-agente:** vanta-chaos
- **Prioridad:** 🟡

## Objetivo
Test de estrés que EJERZA la evicción `*_locked` bajo contención real: config con `max_nodes` bajo (p.ej. 5-10k) + watermark que dispare + threads concurrentes de insert/delete_batch/get_many. Validar que `evict_cold_nodes_with_reason_locked` y `consolidate_node_locked` corren bajo contención sin deadlock ni timeout (los fixes de FND-02 no eran ejercitados).

## Archivos clave
- `src/storage/engine/tests/ops.rs` (tests existentes de FND-02: test_evict_cold_nodes_locked_no_reentrant_timeout, test_multi_index_write_paths_no_deadlock), `src/storage/engine/maintenance.rs` (eviction), `src/config.rs` (max_nodes/memory_limit/rss_threshold)

## Steps
1. ✅ DISCOVERY: leídos test_evict_cold_nodes_locked_no_reentrant_timeout + test_multi_index_write_paths_no_deadlock (ops.rs:1253-1368), evict_cold_nodes_with_reason_locked/consolidate_node_locked (maintenance.rs:410-522), watermark real: NO existe config max_nodes — deriva de hardware `total_memory/4/1536` (insert.rs:294-323, 831-857). El trigger *_locked solo corre en apply_insert/batch_insert al superar el watermark (~2.7M nodos).
2. ✅ Diseñado: test con umbral bajo local (EVICTOR_THRESHOLD=64) que simula el watermark; threads evictor toman insert_lock (como apply_insert) y llaman evict_cold_nodes_with_reason_locked(0.5, Watermark); 4 writers (insert+get_many), 1 deleter (delete_batch), 2 evictors; watchdog 60s deadline; assert acumulado total_evicted > 0.
3. ✅ Implementado `test_evict_locked_under_contention_no_deadlock` en src/storage/engine/tests/ops.rs (tras test_multi_index_write_paths_no_deadlock, +133 líneas). Solo tests; ningún archivo de producción tocado.
4. ✅ Verificado: `cargo test -p vantadb --lib storage::engine::tests::ops` — 74 passed, 0 failed ×2 corridas; test nuevo aislado 3.23-3.60s; `cargo fmt --check` ✅; `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` ✅ (exit 0).
5. ✅ Task file + RESULTADO actualizado.

## Contrato (verify mecánico)
- Test nuevo compila y pasa de forma determinista (correr ≥2 veces)
- El test demuestra que la evicción `*_locked` se ejecutó (assert sobre evict count o estado) — NO un no-op
- Los 2 tests de FND-02 siguen pasando

## Invariantes (handoff)
- NO tocar: código de producción (solo tests), docs/Backlog.md, AUD-024.md, verify-log.jsonl, _vanta-cli.ps1, plan file, .opencode/AGENTS.md, .opencode/agents/*
- NO git add/commit; NO campaign_update_task_state
- Sin flakiness: timeouts generosos, no assert <100ms frágiles
- No depende de timing exacto (deadline, no sleep fijo)

## Fases
- SECURITY: n/a
- PERFORMANCE: n/a (test)

## Resultado
```
RESULTADO: ✅ COMPLETO
STEPS_OK: 5/5
PROXIMO_STEP: ninguno
COMMIT_HASH: ninguno (lead commitea)
ARCHIVOS: src/storage/engine/tests/ops.rs (solo tests — +133 líneas: test_evict_locked_under_contention_no_deadlock)
VERIFY_CONTRATO: pasa
BLOQUEO: ninguno
```