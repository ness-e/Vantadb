# FND-01-F1: Wire RSS real en check_memory_pressure (cierra riesgo OOM)

## Metadata
- **Plan:** docs/plans/2026-08-16-wave-followups.md (W1)
- **Fuente:** follow-up del audit FND-01 (bench memory_budget confirmó subestimación 6.5×)
- **Estado:** ✅ COMPLETO · **Sub-agente:** vanta-worker
- **Prioridad:** 🔴

## Objetivo
El guard `check_memory_pressure` (`src/storage/engine/stats.rs:98`) compara contra `physical_rss` (solo mmap, 54 MiB) o estimación lógica — nunca el RSS real del proceso (354 MiB a 20k nodos). Wire el RSS real (`get_native_memory()` vía `src/metrics/core/mod.rs:471` `_get_rss_virt`, con fallback sysinfo) para que el guard detecte el riesgo OOM real antes de que el SO mate el proceso.

## Archivos clave
- `src/storage/engine/stats.rs` (guard), `src/metrics/core/mod.rs:471` (RSS real ya disponible), `src/config.rs` (memory_limit/rss_threshold), `.opencode/rules/memory-budget.md` (regla a actualizar si aplica), `benches/memory_budget.rs` (evidencia), `docs/Investigaciones/FND-01-memory-budget.md` (reporte a cerrar con el fix)

## Steps
1. ✅ DISCOVERY: stats.rs completo, `_get_rss_virt`/`get_native_memory` (mod.rs:471/306), `record_memory_breakdown`, callers (insert.rs:34/542, delete.rs:22/198), tests stats.rs, bench memory_budget, regla memory-budget.md
2. ✅ Implementar: `check_memory_pressure` usa `crate::metrics::core::_get_rss_virt()` (RSS real, fallback sysinfo interno) con fallback a `stats.effective_bytes()` si la medición da 0 (Miri/plataforma sin soporte). Sin cambio de firma pública. `_get_rss_virt` pasó a `pub(crate)` (única visibilidad nueva). Tests con límites artificialmente chicos (64 MiB/80 MB/128 MiB) ajustados a 8 GiB porque el RSS real del binary de test los excede.
3. ✅ Verificar: `cargo check -p vantadb` ✅; `cargo nextest run -p vantadb -E 'test(check_memory_pressure) or test(memory_stats) or test(backpressure) or test(hot_node_eviction) or test(resource_limit) or test(pressure_ratio) or test(record_memory_breakdown) or test(open_with_config)'` → 30/30 PASS ✅; `MEMORY_BUDGET_SCALE=lite cargo bench -p vantadb --bench memory_budget` re-corrido → pressure_ratio a 20k: 0.002 (mmap) → **0.011** (RSS real 354.62 MiB) ✅
4. ✅ Actualizar: `.opencode/rules/memory-budget.md` regla 2 reescrita (mecanismo descrito → medición en vivo `_get_rss_virt()` con fallback, no snapshot); `docs/Investigaciones/FND-01-memory-budget.md` sección "Fix aplicado" (§8) + §6 actualizado.
5. ✅ Task file + RESULTADO.

## Contrato (verify mecánico)
- ✅ `cargo check -p vantadb` pasa
- ✅ El guard usa RSS real (grep `_get_rss_virt` en stats.rs) con fallback documentado
- ✅ Bench lite re-corrido y evidencia en el reporte (§8: 0.002 → 0.011)
- ✅ Sin cambio de firma pública de `check_memory_pressure`

## Invariantes (handoff)
- NO tocar: docs/Backlog.md, AUD-024.md, verify-log.jsonl, _vanta-cli.ps1, plan file, .opencode/AGENTS.md, .opencode/agents/*
- NO git add/commit; NO campaign_update_task_state
- Preservar comportamiento: si la medición de RSS falla → fallback a la estimación actual (no panic)
- Regla 8: toca storage engine — sin locks nuevos, pero revisar lock order si toca algún lock

## Fases
- SECURITY: n/a (no trust boundary; no input externo)
- PERFORMANCE: aplica — es un fix de guard de memoria; el costo de medir RSS ya existe (record_memory_breakdown lo usa)

## Resultado
```
RESULTADO: ✅ COMPLETO
STEPS_OK: 5/5
PROXIMO_STEP: ninguno (FND-01-F2/F3 quedan en el reporte como pendientes, no bloquean)
COMMIT_HASH: ninguno (lead commitea)
ARCHIVOS: src/storage/engine/stats.rs, src/metrics/core/mod.rs, src/storage/engine/tests/stats.rs, src/storage/engine/tests/engine.rs, tests/api/python.rs, benches/memory_budget.rs, .opencode/rules/memory-budget.md, docs/Investigaciones/FND-01-memory-budget.md, .opencode/skills/campaign-executor/tasks/FND-01-F1.md
VERIFY_CONTRATO: pasa
BLOQUEO: ninguno
```