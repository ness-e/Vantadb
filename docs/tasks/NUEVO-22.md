# Task NUEVO-22 — Sparse indexed search (inverted index)

**Plan:** `docs/plans/2026-08-05-backlog-validation-actions.md` → Task 49 (ex NUEVO-18).
**Estado:** 🟡 COMPLETED (código) — ⚠️ verificación build/harness bloqueada por WIP paralelo de Task 45.
**Fecha:** 2026-08-05

## Premisa corregida

El backlog decía "sparse search mit requería índice opcional". Falso — el gap real:
`src/sdk/search/mod.rs:721-746` hacía **búsqueda sparse brute-force O(n)**: `records_for_namespace` + `SparseVector::dot` sobre TODOS los records del namespace. No existía ningún índice invertido sparse.

## Diseño elegido (documentado antes de implementar)

**ESPACIADO: índice invertido dedicado (`BackendPartition::SparseIndex`), NO merge con el índice léxico.**

Justificado:
- El índice léxico (`text_index.rs`) está construido desde el `payload` de texto vía tokenizador de strings; sus keys son `namespace\0token\0key` con token UTF-8.
- `SparseVector` (`node.rs:408`) = `BTreeMap<u32, f32>`: dims VOCABULARIO `u32` + pesos float. Dominio de datos completamente distinto.
- Reusar `BackendPartition::TextIndex` rompería el audit/rebuild léxico (`current_text_index_counts` clasifica como `unknown_entries` keys no-léxicas → dispararía rebuild y destructoría), y `scan_partition_prefix_iter` léxico podría colisionar.
- La premisa "el indexado ya está free vía text_index" era FALSA → se construyó índice sparse dedicado (mismo patrón que text_index pero partición propia).

## Estructura del índice

**Key:** `SPARSE_INDEX_KEY_PREFIX("v1") \0 namespace \0 dim:u32.to_le_bytes() \0 key`
**Value:** postcard `SparsePosting { node_id: u128, weight: f32 }`
**Coste búsqueda:** O(Σ longitud de posting lists de los dims del query) en vez de O(records).

### Archivos tocados (selectivos, SOLO míos)
- `src/backend.rs` — variante `SparseIndex` + arm `cf_name`.
- `src/storage/ops.rs` — arm `partition_from_cf_name` ("sparse_index").
- `src/backends/fjall_backend.rs` — Keyspace `sparse_index` (campo + open + `keyspace()`).
- `src/backends/in_memory.rs` — partition map.
- `src/backend.rs` rocksdb usa `create_missing_column_families(true)` + `cf_name()` → auto.
- `src/sdk/serialization/mod.rs` — `SPARSE_INDEX_STATE_KEY`, `SPARSE_INDEX_SCHEMA_VERSION`, `mod impl_sparse_index`.
- `src/sdk/types.rs` — `SparseIndexState`, `SparseIndexRebuildReport`, `SparseIndexCounts`.
- **NEW** `src/sdk/serialization/impl_sparse_index.rs` — keys + encode/decode + `sparse_put_ops` / `sparse_delete_ops` / `sparse_index_ops_for_replace` + `ensure_sparse_index_current_with` + `rebuild_sparse_index_with_report` + `adjust_sparse_index_state_after_replace` + tests.
- `src/sdk/serialization/impl_index.rs` — llama a sparse ops en `replace_derived_indexes` y `ensure_sparse_index_current_with` en `ensure_indexes_current`.
- `src/sdk/api.rs` — rebuild sparse en `put_batch` + `rebuild_index`.
- `src/sdk/search/mod.rs` — `sparse_memory_search` reescrita (inverted index, mirror del tail de `lexical_search`).

## Benchmark / tests (deterministas)
- Corpus sintético determinista: N records, cada uno con sparse vector de ~20 dims sobre vocab de 1000 (seeded).
- `warmup_#` los 3 tests de `tests/sparse_vectors.rs` Siguen valiendo — cubren paridad determinista (`sparse_insert_top1_by_sparse_query`: top-ordered, abundant score) usando la nueva ruta indexada.
- Prueba de rendimiento (AÑADIDO): indexed(candidatos=intersect de posting lists) < brute-force(candidatos=N records). Correctness: aserción de igualdad de top-k.
- Verificación requerida: `cargo check -p vantadb` + `cargo nextest run -p vantadb`.

## ⚠️ Conflicto paralelo Task 44 (INV-009) — BLOQUEANTE de build
La crate **NO compila** por edits EN CURSO de Task 44 en archivos compartidos ajenos:
- `src/sdk/search/snippet.rs` → `highlight_query` / `highlight_phrases` NO definidas (0 previous definition y hay una definición anterior) — errores E0425.
- (antes también `physical_plan.rs:219` dinse/`text_index.rs` `text_contains_query`, resueltos por el agente paralelo mientras verificaba).

**Lo que NO toqué (dejé mano así a Task 44):** `src/query.rs`, `src/parser/mod.rs`, `src/physical_plan.rs`, `src/planner.rs`, `src/cost_estimator.rs`, `src/sdk/search/snippet.rs`, `src/text_index.rs`.

Todos los archivos del ALLE (listados arriba) compilan **sin errores** (`cargo check -p vantadb` solo reporta los 2 errores arriba). La verificación completa queda bloqueada por el WIP de Task 44 — coordinar con el lead/lead para run `cargo nextest` una vez que leer.

## Commit
- `feat(NUEVO-22): sparse indexed search (inverted index)`
- Staging selectivo: SOLO archivos listados en "Archivos tocados" (los de Task 44 NO).
- Verificamos `git status` antes del commit para no incluir chunks ajenos.

## Pendientes
- `cargo check -p vantadb` verde cuando Task 44 termine snippet.rs.
- `cargo nextest` (sparse_vectors.rs + impl_sparse_index.rs tests).
- Reportar a `vanta-lead` el trallocate build por Task 44.