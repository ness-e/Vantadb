# FIND-31 — purge_expired tras reopen falla "text index df would go negative"

- **Plan:** `docs/plans/2026-08-25-batch-colaterales-deuda-desktop.md`
- **Estado:** ✅ COMPLETO (pasos implementados y verificados; commit lo ejecuta el lead)
- **Esfuerzo:** 🟡 · **Appetite:** max 2h
- **Tipo:** Bug (Rust core — storage/text index durability consistency)
- **Contrato:** test put→flush→reopen→purge_expired pasa sin error; `cargo nextest run -p vantadb` verde

## DISCOVERY

### Blast radius (codegraph_explore + Read)
- `purge_expired` (`src/sdk/api.rs:911`) — computa deltas de term-stats/namespace-stats/doc-stats y los
  aplica con `checked_stats_value` (api.rs:1041-1098) → `ValidationError "text index {label} would go negative"`.
- `load_text_term_stats` (`impl_text_index.rs:162`) — cache-aside sobre `BackendPartition::TextIndex`.
- `recover_state` (`init.rs:391`) — reconstruye HNSW + replay WAL; NO toca text index.
- `ensure_indexes_current` (`impl_index.rs:22`) — corre en CADA `VantaEmbedded::open_with_config`; llama
  `ensure_text_index_current_with` → `rebuild_text_index_with_report`.
- `memory_record_from_node` vs `memory_record_from_node_include_expired` (`serialization/mod.rs:309/316`).

### Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** api.rs (purge_expired 908-1187, put 112-187, tests 1798+),
  impl_text_index.rs (162-209, 246-374, 11-66, 233-244), impl_rebuild.rs (71-181, 183-260),
  init.rs (391-618), mod.rs (382-411, 448-492), partition.rs, impl_index.rs (22-77, 154-240),
  backend.rs, fjall_backend.rs (35-274), ops.rs (230-363), serialization/mod.rs (309-425),
  text_index.rs.
- **Referencias hacia dentro del cambio:** `checked_stats_value` (purge_expired + text_index_ops_for_replace),
  `load_text_term_stats` (4 callers), `memory_record_from_node_include_expired` (2 callers nuevos).
- **Referencias entrantes a los editados:** `src/sdk/api.rs` — 178 callers de VantaEmbedded; `purge_expired`
  público lo llama MCP (tools.rs). `impl_text_index.rs`/`impl_rebuild.rs` — módulos privados del SDK.
- **Veredicto impacto:** medio. No cambia API pública, no cambia semántica de purge para records no indexados,
  no toca WAL/vector/. Cambia: el rebuild/reconcile del text index ahora incluye records expirados-no-purgados.

## Fase 1 — Evidencia de Debugging (GATE)

- **Repro:** `test_purge_expired_after_reopen_with_indexed_payload` (put ttl=1ms → flush → close → reopen →
  sleep 5ms → purge_expired). FALLÓ RED con `ValidationError "text index df would go negative"` (api.rs:2042).
- **Hipótesis inicial (task):** tras reopen replay_write_node no reconstruye text index → stats ausentes.
- **Evidencia de diagnóstico (desechó la hipótesis inicial):**
  - Session 1 (put+flush): `BackendPartition::TextIndex` = **10** entradas.
  - Raw reopen (`StorageEngine::open_with_config`, SIN SDK): = **10** entradas, scan_nodes=1 → el keyspace
    **SÍ persiste** en Fjall; `recover_state` no lo borra.
  - SDK reopen (`ensure_indexes_current` corre): = **0** entradas, scan_nodes=1, Default=1.
  - `rebuild_text_index_with_report()` manual tras reopen: report **record=0 posting=0** → `scan_nodes()`
    devuelve 1 pero `memory_record_from_node` filtra el record → 0.
- **CAUSA RAÍZ REAL:** `memory_record_from_node` aplica **lazy TTL eviction** (`serialization/mod.rs:359-368`:
  `if now > deadline return None`). `rebuild_text_index_with_report`, `expected_text_index_counts_from` y
  `expected_text_index_entries` usan `memory_record_from_node`, así que un record **expirado pero no purgado**
  queda FUERA del rebuild del text index. En reopen, `ensure_text_index_current_with` ve state ausente/mismatch
  (primer put nunca persiste state) → rebuild → borra las 10 entradas → re-indexa solo records no expirados →
  las term-stats del record expirado se pierden. Luego `purge_expired` decrementa esas stats ausentes → negativo.
- **1 variable controlada:** el test RED es la única variable; no se tocó producción antes de confirmar RED.

## EJECUCIÓN

### Step 1 — Test RED ✅
- **Archivos:** `src/sdk/api.rs`
- **Acción:** agregar `test_purge_expired_after_reopen_with_indexed_payload`.
- **Verify:** FALLÓ RED (ValidationError df negative) — confirmado en árbol limpio (worktree HEAD).

### Step 2 — Root cause + fix mínimo ✅
- **Archivos:** `impl_rebuild.rs` (rebuild_text_index_with_report + expected_text_index_entries),
  `impl_text_index.rs` (expected_text_index_counts_from).
- **Acción:** cambiar `memory_record_from_node` → `memory_record_from_node_include_expired` en los 3 paths
  del text index (rebuild + reconcile + audit). El text index debe reflejar records durables (hasta que purge
  los borre como unidad), no la vista TTL-evicteada de lectura.
- **Por qué esta opción:** es la causa raíz. Descarté (a) guard de deltas (saturación) porque **enmascara** la
  inconsistencia real del índice y requeriría saturar también doc_count/total_doc_len; descarté (b) rebuild en
  recover_state por costo de cold-start (stop-condition del plan). (c) "asegurar text index antes de purge" es
  redundante: el rebuild correcto (include_expired) ya deja el text index consistente con lo que purge decrementa.
- **Verify:** RED→GREEN ✅ (`test_purge_expired_after_reopen` PASS).

### Step 3 — Suite completa ✅
- **Acción:** correr tests de purge/TTL/text index/rebuild + suite completa.
- **Verify:** `cargo nextest run -p vantadb` → 2061 passed, 1 skipped. Related 110/110 pass.
  `cargo check -p vantadb --all-targets` ✅ · `cargo fmt --check` ✅ · `cargo clippy -p vantadb --all-targets -- -D warnings` ✅

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** (1) records sin payload indexado siguen purgados; (2) firma `purge_expired`
  intacta (`Result<u64>`); (3) no tocar wal.rs/storage/vector/; (4) conteo devuelto intacto.
- **Comandos de verificación:** `cargo nextest run -p vantadb test_purge_expired_after_reopen` → ✅;
  `cargo nextest run -p vantadb` → ✅ 2061 pass; `cargo check -p vantadb --all-targets` → ✅;
  `cargo fmt --check` → ✅; `cargo clippy -p vantadb --all-targets -- -D warnings` → ✅.
- **Deuda pendiente:** el índice derivado (namespace/payload) y sparse también usan `memory_record_from_node`
  en sus rebuilds → expirados-no-purgados se omiten de esos índices. NO causa crash (deletes idempotentes, sin
  stats df/doc_count), por eso quedó fuera de este fix acotado. Evaluar alinear a include_expired en un
  follow-up de consistencia (no bloquea esta task).

## Review (GATE — agente distinto, P2-01)
- **Revisor:** `vanta-review` (PENDIENTE — el lead lo delega antes de commit; fallback: doubt-driven-development).
- **Enfoque:** fix de causa raíz (include_expired en text index rebuild/reconcile) vs guard saturante en purge.
  El guard enmascara la inconsistencia; include_expired la resuelve manteniendo el índice consistente con los
  records durables que purge elimina como unidad.
- **Cómo se probó:** RED falla pre-fix (ValidationError df negative) → GREEN post-fix + suite 2061 pass +
  check/fmt/clippy limpios en worktree limpio (HEAD, sin WIP de otros workers).

## Notas
- **Concurrencia:** el worktree HEAD se usó para verificar porque `src/wal_sharded.rs` y `engine/insert.rs/
  delete.rs/txn.rs` tienen WIP sin commitear de MOD-06 (otro worker, MOD-06 toca wal_sharded.rs) que rompe
  `vantadb (lib test)`. Mis 3 archivos NO chocan con el blast radius de MOD-06. El lead debe verificar el estado
  combinado al commitear.
- FASE SECURITY: N/A (consistencia interna de índice derivado; no trust boundary/input/deps).
- FASE PERFORMANCE: sin impacto (no loops nuevos de reconstrucción; el rebuild solo corre cuando hay drift).
- Archivos tocados: `src/sdk/api.rs`, `src/sdk/serialization/impl_rebuild.rs`,
  `src/sdk/serialization/impl_text_index.rs` (+ task file FIND-31.md).
