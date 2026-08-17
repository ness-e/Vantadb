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
1. DISCOVERY: leer los 2 tests de FND-02 + cómo configurar max_nodes bajo + cómo dispara la evicción (watermark rss_threshold/eviction_ratio)
2. Diseñar: test de estrés con max_nodes bajo + N threads (insert/delete_batch/get_many) + assert de que la evicción ocurrió (contador/estado) y no hubo timeout/deadlock (deadline wall-clock generoso)
3. Implementar en tests/ops.rs (o archivo de tests nuevo si el tamaño lo justifica)
4. Verificar: `cargo test -p vantadb --lib storage::engine::tests::ops` (o el path del test nuevo) — pasa, determinístico (sin flakiness)
5. Task file + RESULTADO

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
RESULTADO: ✅ COMPLETO | 🟡 INCOMPLETO | ❌ FALLIDO
STEPS_OK: <n>/<M>
PROXIMO_STEP: <...>
COMMIT_HASH: ninguno (lead commitea)
ARCHIVOS: <paths tocados>
VERIFY_CONTRATO: <pasa | no-corrido | falla>
BLOQUEO: <ninguno | ...>
```