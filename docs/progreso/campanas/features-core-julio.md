# Features core julio — NUEVO/COMP/REC (LSM, napi, aristas temporales, recuperación)

> **Migrado desde** `docs/progreso/README.md` (split GOV-D2, 2026-08-22). Contenido histórico sin cambios salvo dedup indicado.

### 2026-08-02 — NUEVO-17: Segment LSM tiers hot/warm/cold + archive ✅

**Fuente:** Backlog (Phase 8 — Post-Launch) `NUEVO-17`

**Resuelto por (vanta-worker):**
- **Hallazgo clave:** la infra de niveles ya existía (`src/lsm.rs`: `SegmentLevel` L0-L3, `SegmentRegistry::open_or_create` pre-asigna 4 VantaFile mmap). El gap era la *política de tier* y el archive L3.
- `TierPolicy` enum (SizeBased | FrequencyBased | AgeBased) + `TierPolicyConfig` (archive on/off, cold_min_access, cold_age_days) en `src/lsm.rs`.
- `LsmConfig` extendido: `l3_max_size`, `l3_tombstone_threshold`, `tier` (defaults compatibles).
- Promoción encadenada en `compact_level`: L0(hot)→L1(warm)→L2(cold)→L3(archive). L3 terminal (no-op seguro) y solo cuando `tier.archive=true`; si está off, L2 es el tier más profundo.
- Tests: `test_tier_promotion_hot_to_cold`, `test_tier_promotion_cold_to_archive`, `test_tier_archive_disabled_stops_at_cold` — 3/3 ✅.
- Doc: `docs/architecture/STORAGE-TIERS.md` (EN inglés).
- Verify: `cargo check` ✅, `cargo test tier*` 3/3 ✅, fmt 0 diffs, clippy 0 warnings.

### 2026-08-02 — COMP-029: Bindings Node.js/TS mediante napi-rs (backend adicional) ✅

**Fuente:** Backlog `COMP-029`

**Resuelto por (vanta-worker + vanta-docs):**
- Crate standalone **`vantadb-node/`** (NO workspace member): `lib = "vantadb_native"` (cdylib), `napi 3` + `napi-derive` sobre `vantadb` (features `fjall, memmap2, rayon`). El aislamiento standalone evita el crash del linker MSVC con cdylib en workspace.
- API isomórfica con el wrapper WASM: `connect`, `flush`, `close`, `put`, `put_batch`, `get`, `delete`, `list`, `list_namespaces`, `search`, `capabilities`. Patrón `engine.clone()` + `tokio::task::spawn_blocking`.
- Persistencia real (fjall/WAL/fsync) en Node.js — WASM no puede. Browser se queda con WASM (`vantadb-wasm` intocado).
- Wrapper TS `vantadb-ts/src/native.ts` + dep `vantadb-node` en `vantadb-ts/package.json`.
- Verify: `cd vantadb-node && npm test` → vitest **3/3** (put/get, persistencia cross-reconnect, search ordenado).
- ADR: `docs/architecture/adr/COMP-029-napi-rs-node-bindings.md`.

### 2026-08-02 — COMP-021: Aristas temporales (relaciones con timestamp) ✅

**Fuente:** Backlog (Phase 10 — Competitive Features) `COMP-021`

**Resuelto por (vanta-lead + vanta-worker):**
- `Edge.created_at_ms: u64` en `src/node.rs`, seteado a wall-clock en `new`/`with_weight`/`reverse`; helper `Edge::with_timestamp`.
- **Custom `Deserialize` manual para `Edge`:** hallazgo — postcard 1.1.3 NO consulta `#[serde(default)]` (`deserialize_struct` → `deserialize_tuple(fields.len())`, `next_element_seed` devuelve `Err(DeserializeUnexpectedEnd)` al agotar el buffer con `len > 0`). Se implementó un visitor que trata el fin de buffer del campo nuevo como `0`, preservando lectura de datasets persistidos antes de esta feature.
- `bfs_traverse_filtered`/`dfs_traverse_filtered` con `time_range: Option<(u64,u64)>` (inclusive) en `GraphTraverser`.
- `add_edge(source, target, label, weight, created_at_ms)` en SDK + bindings Python/WASM/TS; timestamp compartido entre arista forward y reverse.
- `docs/api/PYTHON_SDK.md` documenta `created_at_ms: Optional[int] = None`.

**Verificación:** `cargo test -p vantadb --lib` 1672 passed ✅ | `cargo test -p vantadb --test temporal_edges` 6/6 ✅ (backward-compat postcard, roundtrip, window filtering, forward+reverse persistence)

**Ids:** `COMP-021`

### 2026-07-31 — Plan de Recuperación de VantaDB (REC-001 a REC-010, REC-999) ✅

**Fuente:** Recovery Plan (`docs/plans/2026-07-28-recovery-plan.md`)

**Resuelto por:**
- **REC-001:** Definición de `VantaFilterOp`, `VantaMemoryFilterItem` y `VantaMemoryFilter` en `src/sdk/types.rs`.
- **REC-002:** Implementación de `delete_by_filter()` en SDK + CLI.
- **REC-003:** Implementación de `count()` con filtros opcionales en SDK + CLI.
- **REC-004:** Implementación de `similar_to_key()` en SDK + CLI.
- **REC-005:** Multi-namespace search (`search_multi` y `search_all`) en SDK + CLI.
- **REC-006:** Implementación de coincidencia de predicados de metadatos avanzados (`matches_advanced_filters`) en listados del SDK.
- **REC-007:** Comandos de WAL compactación y vacuum en la CLI (completado previamente).
- **REC-008:** Diseño de incremental backup + PITR, e implementación de la Fase A (`MANIFEST.json` con integridad CRC32C de archivos en `cmd_backup`).
- **REC-009:** Análisis de viabilidad de Product Quantization (PQ) vs SQ8/TurboQuant/RaBitQ.
- **REC-010:** Empaquetado y tipados de Python (completado previamente).
- **REC-999:** Corrección e historial actualizado en `docs/progreso/README.md`.

**Verificación:** `cargo check -p vantadb` ✅ | `cargo check --bin vanta-cli` ✅

**Ids:** `REC-001`, `REC-002`, `REC-003`, `REC-004`, `REC-005`, `REC-006`, `REC-007`, `REC-008`, `REC-009`, `REC-010`, `REC-999`

### 2026-07-28 — COMP-018: Cadenas de Relaciones Doblemente Enlazadas ✅

**Fuente:** Backlog (Phase 10 — Competitive Features) `COMP-018`

**Resuelto por:**
- Rust SDK: `graph_bfs()`, `graph_dfs()`, `graph_bfs_filtered()`, `graph_dfs_filtered()` — añadido parámetro `direction: TraversalDirection`
- Python bindings: `graph_bfs()`, `graph_dfs()` — añadido `direction="Forward"` via PyO3 signature + `parse_direction()`
- WASM bindings: `graph_bfs()`, `graph_dfs()` — añadido `direction: String` con parse
- 5 archivos modificados: `src/sdk/graph.rs`, `vantadb-python/src/lib.rs`, `vantadb-python/src/convert.rs`, `vantadb-wasm/src/lib.rs`, `examples/rust/graphrag.rs`
- Edge.reverse + add_edge/remove_edge bidireccional ya existían

**Verificación:** `cargo check -p vantadb` ✅ | `cargo check -p vantadb_py` ✅ | `cargo check -p vantadb-wasm` ✅ | 33 tests graph ✅

**Ids:** `COMP-018`

<!-- movido a ARCHIVO_HISTORICO.md -->
<!-- movido a ARCHIVO_HISTORICO.md -->
<!-- movido a ARCHIVO_HISTORICO.md -->
### 2026-07-29 — REC-007: Compactación WAL + CLI Vacuum ✅

**Fuente:** Backlog (Phase 8 — Post-Launch & Enterprise) `REC-007`

**Resuelto por (vanta-worker, ponytail):**
- `src/cli.rs` — Nuevo `WalCommand` enum con variantes `Compact` / `Vacuum`
- `src/cli_handlers/wal.rs` — Handlers `cmd_wal_compact()` / `cmd_wal_vacuum()` con box-drawing output
- `src/cli_handlers/mod.rs` — `pub mod wal;` + `pub use wal::*;`
- `src/bin/vanta-cli.rs` — Dispatch match arm
- Binding directo de `VantaEmbedded::compact_wal()` y `VantaEmbedded::vacuum()` — sin lógica nueva

**Verificación:** `cargo check -p vantadb --features cli` ✅ | `cargo clippy` ✅ | 4 archivos modificados

**Ids:** `REC-007`

### 2026-07-29 — REC-001: Tipos de Filtro Base (VantaFilterOp + VantaMemoryFilterItem) ✅

**Fuente:** Backlog (Phase 8 — Post-Launch & Enterprise) `REC-001`

**Resuelto por (vanta-lead, vanta-worker, ponytail):**
- `src/sdk/types.rs:106-126` — Tres nuevos tipos agregados:
  - `VantaFilterOp` enum: `Eq`, `Neq`, `Gt`, `Lt`, `Gte`, `Lte`
  - `VantaMemoryFilterItem` struct: `field: String`, `op: VantaFilterOp`, `value: VantaValue`
  - `VantaMemoryFilter` type alias: `Vec<VantaMemoryFilterItem>` (AND semantics)
- `src/sdk/mod.rs` — Re-exportados `VantaFilterOp`, `VantaMemoryFilterItem`, `VantaMemoryFilter`
- Pure additive change — 0 existing types touched
- desbloquea: SDK-01 (delete_by_filter), SDK-03 (count_with_filters), SDK-05 (expanded metadata filters)

**Ponytail:** No implementar `evaluate_filter()` todavía — los tipos primero, el matching se añade con el primer consumidor.

**Verificación:** `cargo check -p vantadb` ✅ | `cargo clippy -p vantadb -- -D warnings` ✅ | 0 regresiones

**Ids:** `REC-001`

### 2026-07-28 — COMP-026: Compactación LSM Multi-nivel ✅

**Fuente:** Backlog (Phase 10 — Competitive Features) `COMP-026`

**Resuelto por (vanta-worker, vanta-lead, ponytail):**
- `SegmentRegistry` con `open_or_create()` — maneja legacy `vector_store.vanta` → `vstore_L0.vanta`
- `StorageEngine.vector_store` cambiado de `RwLock<VantaFile>` → `Vec<RwLock<VantaFile>>` (un lock por nivel)
- `read_header_from_segment()`, `read_vec_bytes_from_segment()`, `write_node_to_l0()` — lectura/escritura segment-aware
- `should_compact_level()` — decisión por tamaño + tombstone ratio
- `compact_level(level)` — promueve nodos vivos a nivel+1, actualiza HNSW offsets, trunca source
- `PipelineMode::CompactOnly` y `CompactL0Only` — nuevas variantes
- `run_pipeline()` extendido con fases LSM (CompactL0 → CompactL1 → CompactL2)
- `LsmReport` en `PipelineReport`
- 13+ archivos modificados: `lsm.rs`, `engine/mod.rs`, `engine/init.rs`, `engine/ops.rs`, `engine/maintenance.rs`, `engine/stats.rs`, `archive.rs`, `physical_plan.rs`, `sdk/api.rs`, `sdk/search/mod.rs`, `engine/tests/`

**Ponytail:** L0+L1 mínimo viable. L3 archive tier diferido.

**Verificación:** `cargo check -p vantadb` ✅ | `cargo nextest run -p vantadb --build-jobs 2` ✅

**Ids:** `COMP-026`
