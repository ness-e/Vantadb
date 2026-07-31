# COMP-013 — Segment Optimizer Pipeline (Vacuum/Merge/Index)

**Estado:** ⚠️ Parcial → **Objetivo:** ✅ Completa
**Esfuerzo:** 🟡 1-2 semanas
**Dependencias:** COMP-004 (bitset soft deletes), COMP-011 (HNSW CRUD tombstones) — ✅ ya implementadas

## Qué existe hoy

En `src/storage/engine/maintenance.rs`:
- `compact_layout_bfs()` — reescribe VantaFile en orden BFS, skips tombstones
- `trigger_compaction()` — chequea fragmentación >20%, solo log warning
- `rebuild_vector_index()` — rebuild completo del HNSW desde VantaFile
- `run_quantization_maintenance()` — mantenimiento periódico f32 ↔ SQ8
- `compact_wal()` — rota WAL + checkpoint

## Qué implementar

### 1. Tipos en `src/storage/engine/mod.rs`
```rust
pub enum PipelineMode { Full, VacuumOnly, MergeOnly, IndexOnly }

pub struct VacuumReport {
    pub scanned_nodes: u64,
    pub removed_tombstones: u64,
    pub reclaimed_bytes: u64,
    pub duration_ms: u64,
}

pub struct MergeReport {
    pub nodes_before: u64,
    pub nodes_after: u64,
    pub saved_bytes: u64,
    pub duration_ms: u64,
}

pub struct PipelineReport {
    pub mode: PipelineMode,
    pub vacuum: Option<VacuumReport>,
    pub merge: Option<MergeReport>,
    pub index: Option<IndexRebuildReport>,
    pub total_duration_ms: u64,
    pub success: bool,
}

pub struct SegmentOptimizerConfig {
    pub enabled: bool,
    pub vacuum_threshold_pct: f32,  // default 15.0
    pub max_duration_secs: u64,     // default 300
}
```

### 2. Métodos en `src/storage/engine/maintenance.rs`
- `vacuum(&self) -> Result<VacuumReport>` — escanea HNSW, remueve nodos FLAG_TOMBSTONE
- `merge_segments(&self) -> Result<MergeReport>` — si hay fragmentación, llama compact_layout_bfs()
- `run_pipeline(&self, mode: PipelineMode) -> Result<PipelineReport>` — orquesta Vacuum→Merge→Index

### 3. `src/config.rs` — añadir `SegmentOptimizerConfig` a `VantaConfig`

### 4. `src/sdk/api.rs` — exponer `vacuum()`, mejorar `compact()` para usar pipeline

### 5. Tests en `src/storage/engine/tests/maintenance.rs`

## NO hacer
- ❌ NO LSM multi-tier (COMP-026)
- ❌ NO WAL archiver
- ❌ NO cambiar compact_layout_bfs interno
