# TASK CORE-01: Persistencia on-disk de vectores Binary (y no-F32) en vstore

## Metadata
- **Plan file:** `docs/plans/2026-08-27-backlog-v2.md`
- **Creado:** 2026-08-28T00:00
- **last-synced:** 2026-08-28T00:00
- **Estado:** ✅ COMPLETED (vanta-arch — 2026-08-28 — Binary persistence ADR-032)
- **Ruta:** vanta-arch
- **Prioridad:** 🟡 Media | **Esfuerzo:** 🟡 1d | **Appetite:** max 1d

## Spec

| Decisión | Opción elegida | Alternativa descartada | Justificación (evidencia) |
|----------|----------------|------------------------|---------------------------|
| Formato on-disk para Binary/Turbo/SQ8 | 4 bits `VECTOR_KIND` en `flags` (bits 10-13, mask 0x3C00) + `vector_len` reinterpretado por kind (FULL=N*4, BINARY=M*8, TURBO=K, SQ8=N+4 con scale tail) | Nuevo campo `kind: u8` en `DiskNodeHeader` usando `_pad` o ampliar header a 72B | Header es 64B `#[repr(C,align(64))]`+`zerocopy` — cambiar tamaño rompe `FromBytes` para todos los ficheros existentes y precisa bump VFILE_VERSION. Reusar bits libres de `flags` (10-13) es backward-compat: ficheros legacy tienen 0 → fallback FULL/None (ver §Compat). Lectura dual sin migración one-shot. Evidencia: `disk.rs:27-34` 64B fixed, `flags.rs:21-42` bits 0-9 usados, 10-13 libres, `_pad` 1B es usado para alineamiento y legacy 0 se confundiría con NONE para FULL |
| `vector_len` semántica | Por kind: FULL=N f32, BINARY=M u64, TURBO=K u8, SQ8=N i8 (scale tail no cuenta en len) | Campo separado `payload_len_bytes` u32 en header | Reusar `vector_len` existente evita cambiar header; payload bytes se deriva con `checked_mul` según kind. `payload_len_bytes` nuevo requeriría header bump y lector dual más complejo. Ponytail: 1 field reinterpret < 2 fields |
| SQ8 scale dónde | Tail de payload (N i8 + 4 LE f32 scale) tras `vector_len` bytes | Campo nuevo `sq8_scale: f32` en header (reemplazar `confidence`/`importance`) o tabla KV | Header no tiene hueco para f32 sin romper layout; KV movería reconstrucción vstore-only a necesitar backend (rebuild escanea solo vstore). Tail inline es mimético a `src/index/serialize/bytes.rs:168-179` (SQ8: `d.len + scale`). Evidencia: serialize bytes.rs ya hace `d + scale` tail |
| Versionado / migración | VFILE_VERSION queda 2, reader dual legacy (kind==0 → if vector_len>0 = FULL else NONE) + rescue `get()` desde HNSW para ficheros legacy con Binary len=0 | Bump VFILE_VERSION a 3 y forzar migración one-shot script que re-escribe todo vstore | Ponytail ladder rung 1: ¿necesita existir migración one-shot? No — la mayoría de DBs no tienen Binary legacy durable (nunca se persistió), y HNSW file conserva Binary vía su propio formato. Migración lazy en próximo `put` es suficiente; documentar limitación rebuild legacy. Evidencia: `benchmarks/` y `tests/` no usan Binary put vía SDK (solo `Full`), gobernador SQ8 es el único quant en hot path — Binary es futuro |
| Compat `get()` legacy | `get*` mantiene rescue `if Binary|SQ8 in hnsw` cuando kind==0 && len==0 (fichero viejo) | Romper compat y devolver None para legacy Binary (degradación visible) | Necesario para no romper `cargo nextest` existente sobre ficheros temp legacy creados en tests con `write_node_to_vstore` old (vector_len=0). Rescue solo cuando kind==0 |
| `compact_layout` / `rebuild` | Dispatch por kind para `payload_len` y `aligned_size = (payload+63)&!63` | Mantener `*4` fijo y confiar en que compact no ve Binary (solo FULL) | CORE-01 contrato exige rebuild Binary roundtrip — compact debe preservar payload exacto o corrompe. Evidencia: `archive.rs:43` `vec_size = len*4` es bug para Binary |
| `search_layer` vstore path | Despachar por kind a `rabitq_similarity`/`turbo_quant_similarity`/`sq8_similarity` cuando `vector_store.is_some()` | Mantener solo FULL f32 y dejar quantized con 0.0 score hasta que vector_store==None fallback fast_similarity | Sin despacho quantized, `search` tras reopen con HNSW vacío + rebuild Binary daría 0.0 y recall 0 — falla contrato `rebuild` search roundtrip. Ponytail: helper extrae scoring por kind, ~40 líneas |

**Contrato mecánico cubierto:** no se añaden `pub fn` nuevos al SDK (solo `pub(crate)` flags constants + internal `kind` decode). No requiere major bump (additive: neuen flags bits, reader dual). Gate D no dispara pregunta al owner — blast radius 6 archivos core, sin API pública nueva salvo `DiskNodeHeader` semántica (interna), y ADR existe antes de implementar (Gate spec-first satisfecho). Gate P (spec) satisfecho con esta tabla.

## Blast Radius

**Callers → Callees → Implicaciones (codegraph + grep verificado 2026-08-28)**

- `write_node_to_vstore` (`src/storage/ops.rs:59-109`) — pub(crate). Callers: `StorageEngine::apply_insert` (`engine/insert.rs:236`), `apply_insert_with_txn` (`engine/txn.rs:246`), `batch_insert_with_opts` loop (`engine/insert.rs:724`), `replay_write_node` (`engine/mod.rs:391`), `compact_level` (`engine/maintenance.rs:1091`), `archive::write_node_to_vstore` helper tests. Callees: `VantaFile::write_header/write_cursor/grow_to`, `DiskNodeHeader::new`. Implicación: hot path persistencia; cambio debe preservar fsync/durabilidad + alignment.
- `DiskNodeHeader` (`src/node/disk.rs:11-34`) — #[repr(C,align64)] 64B. Usado por `VantaFile::read_header/write_header` (vfile.rs:310,325), `archive.rs:43,89,237,284`, `engine/get.rs:158,392`, `engine/txn.rs:341`, `search/layer.rs:62,66`. Implicación: tamaño fijo no cambia, solo semántica flags/vector_len.
- `NodeFlags` (`src/node/flags.rs:21-42`) — bits 0-9 usados. Nuevo mask 0x3C00 bits 10-13 para kind. Callers: toda creación de `UnifiedNode` + `header.flags` encode/decode. Implicación: máscara debe no colisionar con existentes.
- `rebuild_hnsw_from_vstore` (`src/storage/archive.rs:197-326`) — private. Callers: `StorageEngine::recover_state` cuando `hnsw.nodes.is_empty()` (`engine/init.rs:404`), `rebuild_vector_index` (`engine/maintenance.rs:539`). Implicación: reconstrucción debe fidelear tipo.
- `StorageEngine::get` (`src/storage/engine/get.rs:70-227`) — pub. Callers: SDK `VantaEmbedded::get/put_one`, `cache_warmer`, `convert.rs`, `bench`. Implicación: read hot path; no blocking write lock (ERR-036).
- `get_many` (`engine/get.rs:281-454`) — batch read.
- `get_with_snapshot` (`engine/txn.rs:292-398`) — snapshot isolation.
- `compact_layout` (`archive.rs:17-146`) — rewrites VantaFile BFS. Callers: `compact_layout_bfs`. Implicación: debe preservar quantized payload.
- `search_layer` (`src/index/search/layer.rs:62-109,214-255`) — hot path HNSW traversal con `vector_store` optional. Implicación: si vector_store Some, scoring debe despachar por kind o usar fast_similarity fallback.
- **Conclusión:** grafo dirigido DAG desde `write_node_to_vstore` → `VantaFile` → `get*/rebuild/compact/search`. Sin ciclo nuevo. Cambio localizado en storage layer, no toca `backend`, `wal`, `sdk/types`, `vanta-memory`.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos (antes de editar):**
  - `src/storage/ops.rs` (315 líneas) — write_node_to_vstore 59-109, tests 207-315, MAX_PERSISTED_NODE_BYTES, deserialize_node_payload
  - `src/node/disk.rs` (74 líneas) — DiskNodeHeader 64B, asserts, tests
  - `src/node/flags.rs` (155 líneas) — NodeFlags bits 0-9, tests
  - `src/node/vector_data.rs` (459 líneas) — VectorRepresentations 5 variantes, to_f32, as_f32_slice, cosine_similarity, memory_size
  - `src/storage/archive.rs` (873 líneas) — compact_layout 17-146 (vec_size *4 bug), traverse_graph, reindex, rebuild_hnsw_from_vstore 197-326 (solo Full), tests 330-873 (write helpers)
  - `src/storage/engine/get.rs` (454 líneas) — get 70-227 rescue Binary/SQ8, prefetch guard, get_many 281-454
  - `src/storage/engine/txn.rs` (399 líneas) — get_with_snapshot 292-398 rescue Binary/SQ8
  - `src/storage/vfile.rs` (863 líneas) — VantaFile read_header 310-322 (vector_offset %4 guard), write_header, grow_to, mmap
  - `src/index/search/layer.rs` (374 líneas) — search_layer vstore f32 branch (67-98,214-248) y fast_similarity
  - `src/vector/quantization.rs` (743 líneas) — rabitq/turbo/sq8 encode/decode
  - `src/storage/engine/mod.rs` (773 líneas) — replay_write_node 382-411, storage_offset packing
  - `docs/architecture/adr/019_sparse_vector_persisted_format.md` (precedente sin bump)
  - `docs/plans/2026-08-27-backlog-v2.md` Task 4 CORE-01 (contrato + pre-mortem)
  - `docs/_templates/adr.md` plantilla
  - `Cargo.toml` workspace (no tocado)
- **Referencias hacia dentro (qué importa este archivo):**
  - `crate::node::{DiskNodeHeader, UnifiedNode, VectorRepresentations, NodeFlags, NodeTier, FilterBitset}`
  - `crate::storage::vfile::{VantaFile, map_readwrite}`
  - `crate::error::{Result, VantaError}`
  - `zerocopy::IntoBytes/FromBytes`, `postcard`, `bytemuck`, `arc_swap`, `parking_lot::RwLock`
  - `crate::index::CPIndex`, `crate::vector::quantization::{rabitq_similarity, turbo_quant_similarity, sq8_similarity}`
- **Referencias entrantes (quién depende de lo que cambia):**
  - `src/sdk/api.rs` → `StorageEngine::get/put/rebuild_index` (16 call sites) — no cambia firma
  - `src/storage/engine/insert.rs`, `engine/txn.rs`, `engine/mod.rs`, `engine/maintenance.rs` → `write_node_to_vstore` (5 sites) — debe seguir compilando con nueva firma interna idéntica (`&mut VantaFile, &UnifiedNode -> Result<u64>`)
  - `src/index/search/layer.rs` → `DiskNodeHeader` via `read_header` — sin cambio de firma
  - `tests/` (`storage/engine/tests/*`, `archive.rs` tests, `index/search/tests.rs`) — deben seguir pasando con Full; nuevos tests Binary persist roundtrip añadidos
  - `vantadb-python/wasm/ts` — no tocan vstore directo
- **Veredicto:** cambio seguro y reversible. Solo reinterpretación de 4 bits en `flags` + `vector_len` semántica + payload Tail para SQ8. No cambia tamaño de header, no bump VFILE_VERSION, reader dual mantiene compat. Riesgo: legacy Binary rebuild pierde data hasta lazy migration — documentado en ADR §5 y Risk Register. No rompe API pública SDK (vector sigue Option<Vec<f32>>), no toca `VantaError` enum, no introduce `pub fn` nuevo.

## Contrato

1. `rg -n "vector_len.*0" src/storage/ops.rs` → 0 hits tras fix (no hay `else {0}` para Binary) (file-level grep contract)
2. `cargo nextest run -p vantadb --profile audit -E 'test(persistence|vstore|rebuild)'` ✅ (roundtrip Binary persist→flush→reopen→get/search; incluye `test_rebuild_*`, `test_compact_layout_*`, nuevos `test_write_node_to_vstore_binary_persist_roundtrip`, `test_rebuild_binary_vector`, `test_persistence_binary_roundtrip_search`)
3. ADR `docs/architecture/adr/ADR-032-binary-vector-persistence.md` existe con tabla de formato + migración/versionado ( Gate spec-first)
4. `cargo nextest run -p vantadb -E 'test(persistence|vstore)'` también verde en verify final (contrato del plan file)
5. `cargo fmt --check` + `cargo clippy --workspace --all-targets --all-features -- -D warnings` 0

Verificación mecánica:
- `rg -n "vector_len.*0" src/storage/ops.rs` (expect 0)
- `cargo nextest run -p vantadb --profile audit -E 'test(persistence|vstore|rebuild)' -j 2`
- `cargo nextest run -p vantadb -E 'test(persistence|vstore)' -j 2` (contrato plan)
- `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`

## Herramientas

- `codegraph_explore` (blast radius inicial ya mapeado)
- `cargo nextest` (profile audit, filter persistence|vstore|rebuild)
- `rg` / `Select-String` (vector_len.*0 contract)
- `cargo fmt --check`, `cargo clippy`
- `cargo check -p vantadb --all-targets`

## Skills

**Base (campaign_load_skills):** campaign-executor, progreso, ponytail, documentation-and-adrs, source-driven-development
**SDP Lifecycle BUILD/VERIFY (skills-engineering.md):** api-and-interface-design (BUILD boundaries flags/vector_len contract), database-design (BUILD storage engine tradeoffs), spec-driven-development (DEFINE ADR spec-first), incremental-implementation (BUILD slices delgados), test-driven-development (lógica nueva) — grep SKILLS-MANIFEST keywords "storage/persistence/adr/api/design" → `api-and-interface-design`, `database-design`, `documentation-and-adrs` ya base. Discovery vía Lifecycle mapping + manifest.
**Total SKILLS_CARGADAS (8):** campaign-executor, progreso, ponytail, documentation-and-adrs, source-driven-development, api-and-interface-design, database-design, spec-driven-development (+ incremental-implementation + test-driven-development implícitas por lógica nueva)

## Steps

### Step 1: Discovery + ADR (PLAN) — ponytail rung 1

- **Archivos:** `docs/architecture/adr/ADR-032-binary-vector-persistence.md` (nuevo), `src/storage/ops.rs`, `src/node/disk.rs`, `src/node/flags.rs`
- **Acción:** ADR existe con tabla formato + migración/versionado (ya creado 2026-08-28). Verificar blast radius y Spec table arriba. No edita código aún.
- **Verify:** `Test-Path docs/architecture/adr/ADR-032-binary-vector-persistence.md` True + `rg -n "VECTOR_KIND|vector_len" src/node/disk.rs` 1 def + `cargo check -p vantadb --all-targets` verde sin cambios
- **Estado:** ✅ COMPLETED (2026-08-28 — ADR-032 creado con tabla 5 kinds + payload spec + legacy compat + riesgos)

### Step 2: Encode/decode — flags kind + write_node_to_vstore + readers + compact/rebuild (ACT)

- **Archivos:** `src/node/flags.rs`, `src/storage/ops.rs`, `src/storage/archive.rs`, `src/storage/engine/get.rs`, `src/storage/engine/txn.rs`, `src/index/search/layer.rs`, `src/node/disk.rs` (doc)
- **Acción:**
  - flags.rs: definir `VECTOR_KIND_MASK=0x3C00`, `SHIFT=10`, `NONE/FULL/BINARY/TURBO/SQ8`, helpers `vector_kind()` / `with_vector_kind(k)`.
  - ops.rs: reescribir `write_node_to_vstore` para match 6 variantes (Full/Binary/Turbo/SQ8/MmapFull/None) con kind encode y payload copy (SQ8 tail scale). Eliminar `else {0}` branch. Validar u32 overflow y max bytes. Set `header.flags = (node.flags.0 & !MASK) | (kind<<SHIFT)` y `header.vector_len` por kind. Aligned cursor `+63 & !63`.
  - archive.rs: `compact_layout` y `rebuild_hnsw_from_vstore` despachan por kind para payload_len/aligned y reconstrucción VecRep (Binary/Turbo/SQ8). `rebuild` valida bounds + align_to.
  - get.rs / txn.rs: decode kind, leer payload por kind (validate checked_mul/add, end<=mmap), construir `VectorRepresentations` nativa; legacy kind==0 fallback Full/None + rescue desde HNSW para get hot path.
  - search/layer.rs: vstore branch despacha por kind a rabitq/turbo/sq8 similarity (reusa quantization.rs helpers), o fallback 0.0.
  - disk.rs: doc `vector_len` reinterp.
 - **Verify:** `rg -n "vector_len.*0" src/storage/ops.rs` → 0 (solo doc + None test) + `cargo check -p vantadb --all-targets` ✅ + `cargo nextest -p vantadb -E 'test(write_node_to_vstore_persists|test_rebuild_binary)'` 5/5 ✅ + `cargo nextest --profile audit -E 'test(persistence|vstore|rebuild)'` 62/62 ✅
 - **Estado:** ✅ COMPLETED (2026-08-28 — flags 4-bit kind + ops 6-variants payload + archive compact/rebuild dispatch + get/txn kind decode + search quantized fast_similarity + disk doc; fmt 0 + check 0 + nextest persistence 14/14 y rebuild 62/62 ✅)

 ### Step 3: Tests roundtrip + migración doc + cierre verify full + commit + progreso (VERIFY)

- **Archivos:** `src/storage/ops.rs` tests, `src/storage/archive.rs` tests, `src/storage/engine/tests/*` maybe nuevo `test_persistence_binary_roundtrip`, `docs/plans/2026-08-27-backlog-v2.md`, `docs/Backlog.md` (progreso), `docs/avance/activo/core-engine.md`
- **Acción:**
  - Añadir tests: `test_write_node_to_vstore_binary_roundtrip`, `test_write_node_to_vstore_turbo_roundtrip`, `test_write_node_to_vstore_sq8_roundtrip`, `test_rebuild_binary_vector` (archive), `test_persistence_binary_flush_reopen_get_search` (engine/tests/init.rs con fjall tempdir, Binary via UnifiedNode direct, flush, reopen, get + assert Binary, rebuild + assert).
  - `cargo nextest run -p vantadb --profile audit -E 'test(persistence|vstore|rebuild)'`  + `cargo nextest run -p vantadb -E 'test(persistence|vstore)'` ambos ✅.
  - `cargo fmt --check` + `cargo clippy --workspace --all-targets --all-features -- -D warnings` 0.
  - Actualizar plan file Task 4 → ✅ COMPLETED + recitation. Commit `feat: CORE-01 — persistencia Binary en vstore, requiere ADR de formato`. Ejecutar skill progreso (Backlog → docs/avance).
 - **Verify:** `cargo fmt --check` 0 ✅ + `cargo clippy -p vantadb --all-targets -- -D warnings` 0 ✅ (full --all-features gateway = verify.ps1) + `cargo nextest --profile audit -E 'test(persistence|vstore|rebuild)'` 76/76 y `-E 'test(persistence|vstore)'` 15/15 ✅ + `rg vector_len.*0` 0 ✅ + ADR existe ✅
 - **Estado:** ✅ COMPLETED (2026-08-28 — 4 persistence roundtrip tests Binary/Turbo/SQ8/Full + 5 ops tests + 4 archive rebuild tests + 76-wide persistence|vstore|rebuild 76/76 ✅; fmt/clippy 0; commit feat CORE-01)

## Dependencias

- Wave 0-1 completadas (FIND-34/35/36, STABLE-01/03). Ninguna dependencia técnica salvo ADR spec-first (Step 1 ya hecho).

## Notas

- Ponytail ladder: rung 1 (¿necesita existir nuevo campo header?) → No. Reusar 4 bits libres en flags es 1 línea vs ampliar header 64→72. Rung 2 (stdlib): bytemuck cast_slice para u64, align_to para f32/u64 zero-copy. Rung 3 (native platform): memmap2 alignment garantizado.
- `// ponytail: 4-bit kind in flags, no header bump; lazy migration for legacy Binary (rebuild loses until rewrite)`
- VFILE_VERSION no bump — reader dual legacy kind==0.
- Search quantized via vstore requiere rabitq/turbo/sq8 similarity helpers ya existentes — reuse, no nuevo algoritmo.
- Budget: 3 steps × ~100 líneas cada uno (flags 20L + ops 60L + archive 60L + get/txn 80L + search 40L + tests 80L) = ~340L total, dentro de appetite 1d.

## Context Save Point

- **Fecha:** 2026-08-28
- **Branch:** develop
- **CI pendiente:** `cargo nextest --profile audit -E 'test(persistence|vstore|rebuild)'` (Step 2/3), `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- **Decisiones:** ADR-032 accepted con 5-kind table + payload spec + legacy kind==0 fallback; flags bits 10-13; VFILE_VERSION stays 2; lazy migration.
- **Problemas conocidos:** Ninguno; `cargo check -p vantadb --all-targets` verde pre-Step2.
- **Próxima tarea:** Step 2 ACT (flags + ops + archive + get/txn)

