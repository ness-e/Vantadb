# FND-02-M3: Race delete-vs-consolidate (maintenance.rs:311-312 + delete.rs:68-69)

## Metadata
- **Plan:** docs/plans/2026-08-16-wave-followups.md (W1)
- **Fuente:** minor 3 del audit P2-01 de FND-02 (vanta-review, commit c104f1f2)
- **Estado:** ✅ COMPLETED (stale — plan previo archivado, cerrada por vanta-lead 2026-08-20)
- **Prioridad:** 🟡

## Objetivo
Race pre-existente entre `consolidate_node` (maintenance.rs:311-312 — re-aplica entrada HNSW antes de release mmap) y `delete` (delete.rs:68-69 — elimina nodo del índice). Se volvió más frecuente desde que la evicción dejó de ser no-op (FND-02). Determinar el interleaving exacto y aplicar el fix mínimo (orden de locks o version check), siguiendo la Regla 8 (orden global de locks de `.opencode/rules/concurrency-async.md`).

## Archivos clave
- `src/storage/engine/maintenance.rs` (consolidate_node, eviction), `src/storage/engine/delete.rs`, `src/storage/engine/ops.rs` (refresh_index), `src/edge_index.rs`, `src/scalar_index.rs`, `.opencode/rules/concurrency-async.md` (regla 8: orden de locks)

## Steps
1. DISCOVERY: leer maintenance.rs (consolidate/evict, ~1200L) + delete.rs paths; mapear interleaving delete↔consolidate (¿qué lock protege cada uno? ¿version check existe?)
2. Decidir fix: lock order / version check / serialización — el mínimo que elimina el race sin cambiar firma pública
3. Implementar + test: test que ejerza delete concurrente con consolidate (deadlock-free, resultado consistente)
4. Verificar: `cargo check -p vantadb` + test nuevo + tests de ops existentes (test_evict_cold_nodes_locked_no_reentrant_timeout, test_multi_index_write_paths_no_deadlock siguen pasando)
5. Reporte breve del fix en `docs/Investigaciones/FND-02-multi-index-locks.md` (sección de cierre) + task file + RESULTADO

## Contrato (verify mecánico)
- `cargo check -p vantadb` pasa
- Test de race nuevo compila y pasa
- Los 2 tests de FND-02 siguen pasando (no regresión de los fixes de reentrancia)
- Regla 8 de concurrency-async.md respetada (sin inversión de lock order)

## Invariantes (handoff)
- NO tocar: docs/Backlog.md, AUD-024.md, verify-log.jsonl, _vanta-cli.ps1, plan file, .opencode/AGENTS.md, .opencode/agents/*
- NO git add/commit; NO campaign_update_task_state
- No cambiar firma pública de consolidate_node/evict_cold_nodes_with_reason (FND-02 invariante 5)
- Invariante mmap: re-add HNSW antes de release (nunca skippear el refresh)

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:** maintenance.rs (1278L), delete.rs (342L), ops.rs (363L), txn.rs (commit path, 393L), insert.rs (eviction call sites, 905L), tests/ops.rs (1368L), tests/mod.rs, concurrency-async.md (Regla 8)
- **Referencias entrantes (callers):** `consolidate_node` → solo evict_cold_nodes_inner + tests; `consolidate_node_locked` → evict_cold_nodes_inner (lock_held=true); `evict_cold_nodes_with_reason` → stats.rs (periodic/OOM); `evict_cold_nodes_with_reason_locked` → insert.rs (apply_insert, batch_insert) — todos internos al crate
- **Referencias salientes:** consolidate_node_inner → backend.put, refresh_index/apply_index_entry_unlocked, release_mmap_vector, volatile_cache — sin API externa
- **Veredicto:** cambio localizado en el cuerpo de `consolidate_node_inner` (privado); no toca firmas públicas (invariante FND-02). `refresh_index` público queda intacto (usado por insert_to_cf + tests). Sin blast radius externo.
- **Race mapeado:** delete (todo su critical section bajo insert_lock) vs consolidate eviction pública (backend.put FUERA de lock; HNSW re-add bajo lock adquirido después) → zombie (A) o resurrección (B). Fix: retener insert_lock toda la sección + version check HNSW bajo lock.

## Fases
- SECURITY: n/a (locks internos, sin input externo)
- PERFORMANCE: no optimización — solo corrección de race; sin benchmark requerido (Regla 9: no es cambio de rendimiento)

## Resultado
RESULTADO: ✅ COMPLETO
STEPS_OK: 5/5
PROXIMO_STEP: ninguno (lead commitea + gate de cierre: fmt/clippy + auditoría concurrente P2-01 con vanta-review)
COMMIT_HASH: ninguno (lead commitea)
ARCHIVOS: src/storage/engine/maintenance.rs (fix consolidate_node_inner), src/storage/engine/tests/ops.rs (2 tests race), docs/Investigaciones/FND-02-multi-index-locks.md (sección 9 cierre), este task file
VERIFY_CONTRATO: pasa (cargo check -p vantadb ✅; 4 tests ✅ — 2 nuevos race + 2 de regresión FND-02)
BLOQUEO: ninguno