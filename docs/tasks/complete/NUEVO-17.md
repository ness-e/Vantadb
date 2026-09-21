# NUEVO-17: Segment LSM-style tiers (hot/warm/cold) con política de archive real

## Metadata
- **Plan file:** no hay plan activo (tarea directa desde backlog)
- **Fuente:** `docs/Backlog.md:169` (Phase 8 — Post-Launch & Enterprise)
- **Esfuerzo:** 🔴 2-3d (effort base "Muy Alto"; esfuerzo real menor tras discovery)
- **Prioridad:** 🔵
- **Tipo:** Rust (storage/engine)
- **Turns estimados:** 30-60
- **Creado:** 2026-08-02T17:29
- **last-synced:** 2026-08-02T17:29
- **Estado:** ✅ COMPLETADO

## Resumen de la tarea

La descripción del backlog dice "Segment LSM-style — hot/warm/cold tiers. Fjall tiene LSM interno, tiers no".
**Discovery (codegraph) revela que la infraestructura de niveles YA EXISTE parcialmente:**

- `src/lsm.rs` (`SegmentLevel` L0-L3, `SegmentRegistry`, `SegmentInfo`, `LsmConfig`, `pack_offset`/`unpack_offset`).
- `src/lsm.rs:7` comentario de diseño: "ponytail: L0 + L1 compaction only — L3 archive tier skipped".
- `SegmentLevel` ya etiqueta: L0=hot, L1=warm, L2=cold, L3=archive (file_name: `vstore_L0.vanta`, …, `vstore_L3.vanta`).
- `SegmentRegistry::open_or_create` YA pre-asigna los 4 niveles (L0..=L3) con `VantaFile` mmap.
- `src/storage/engine/maintenance.rs` ya tiene `compact_level` (por-nivel) + `LsmReport`.
- `LsmConfig` tiene thresholds: `l0_max_size=64MB`, `l1_max_size=512MB`, `l2_max_size=4GB`, tombstone thresholds, `min_segment_size=64KB`, pero **L3/archive NO tiene config ni policy** (SÓLO L0/L1/L2).

**Lo que FALTA (gap real):**
1. Política de tier explícita: qué promueve un nodo de hot→warm→cold (actualmente el "nivel" es solo el archivo destino, no hay policy de acceso/frecuencia).
2. Soporte de archive tier L3: `LsmConfig` sin l3_max_size/l3 threshold, `run_pipeline`/`compact_level` promoción a L3 ausente.
3. Métricas/monitoreo del tier actual por nodo.
4. Config o heurística para backing (warm/cold en disco barato vs hot en memoria).
5. Tests de política de tier.

**Objetivo:** convertir los niveles actuales (que son solo "destino de compactación por tamaño") en una política de tiers real (hot/warm/cold + archive opcional) verificable, sin romper backward-compat de `vstore_l0/l1/l2`.

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `src/storage/engine/mod.rs` (importa `pack_offset`, `SegmentRegistry`), `src/storage/engine/init.rs`, `src/storage/engine/ops.rs`, `src/storage/engine/maintenance.rs` (compact_level), `src/backends/fjall_backend.rs` (Fjall ya hace LSM interno separado) |
| Callees | `src/storage/vfile.rs` (VantaFile/mmap), `src/lsm.rs` (SegmentLevel/LsmConfig/SegmentRegistry), `src/index.rs`/`src/index/*` (hnsw), `BackendPartition` |
| Implicaciones | **No rompe contrato público** (es interna de storage). Backend ya existen: Fjall (LSM nativo a nivel KV), el LSM de levels es alterno. RMBA: no tocar `pack_offset` (6-bit segment_id limita a 64 segmentos; 4 niveles encaja). Afecta performance/memoria (¿quita mmap residente de capas frias?). |

## Contrato

"Cargo.nextest run --profile audit --workspace --build-jobs 2 pasa **y** existe una política de tier verificable: un nodit cold/archive se mueve entre niveles según criterio definido (edad/frecuencia/tamaño) y hay test que lo demuestra."

> ⚠️ El contrato NO debe ser vago: debe existir un test que pruebe la promoción entre tiers (p.ej. `test_tier_promotion_hot_to_cold`).

## Herramientas necesarias
- cargo-mcp (check, clopp, fmt, nextest)
- rust-analyzer-mcp (diagnostics)
- codegraph_explore (blast radius ya hecho)
- Web research: LSM-tiering patterns (Cassandra/FDB hot-warm-cold) si se necesita

## Investigation Notes
- Ya existe `SegmentLevel L0-L3` + `SegmentRegistry` con 4 mmap pre-asignados (112).
- `LsmConfig` solo configura L0/L1/L2; L3 (archive) sin tamaño/umbral → agregar.
- Fjall ya maneja compación LSM internamente en su KV; el tiers-vfile es independiente (vector segments). No confundir.

## Steps

### Step 1: Diseñar política de tier (doc primero, RFC intra-task)
- Archivos: `docs/architecture/STORAGE-TIERS.md` (nuevo, inglés)
- Consultar: definir criterios (edad vs tamaño vs frecuencia de acceso), niveles hot/warm/cold(+archive), valores default del `LsmConfig` para hot/warm/cold, y regla de promoción/democión.
- Verify: el doc existe y lista los 3 criterios posibles.
- Estado: ⬜ PENDING

### Step 2: Extender `LsmConfig` con archive/cold tier
- Archivos: `src/lsm.rs` (struct LsmConfig ≈ 128~200)
- Agregar `l3_max_size`, `l3_tombstone_threshold`, `cold_min_frequency`, `tier_policy` enum (por edad/frecuencia/tamaño); definir struct de política `TierPolicy`.
- Verify: `cargo check -p vantadb`
- Estado: ⬜ PENDING

### Step 3: Implementar promoción hot→warm→cold en `compact_level`
- Archivos: `src/storage/engine/maintenance.rs` (compact_level ≈ 885)
- pipe la política: que un segment lleno pase de L0→L1→L2 y opcionalmente L3 según umbral. Usar niveles vfile ya existentes.
- Verify: `cargo check -p vantadb` + nuevo unit test en `src/lsm`/`tests/storage/`.
- Estado: ⬜ PENDING

### Step 4: Test de promoción de tier
- Archivos: `src/storage/engine/tests/maintenance.rs` (registro de tests) + `tests/storage/tier*.test`
- `test_tier_promotion_hot_to_cold`: nodo de baja frecuencia/sea viejo migrado a nivel frío y queda consultable.
- Verify: `cargo nextest run -p vantadb --test torture...` / `cargo nextest run --profile audit --workspace`.
- Estado: ⬜ PENDING

### Step 5: Métricas + doc coverage + commit
- Métricas: agregar a `tracking`/`metrics` (per_otol) map de tier. Actualizar `docs/api/` no público (internal) pero registrar en CHANGELOG interno si es funcional.
- `just verify`: fmt + clippy + nextest + deny.
- commit `feat: segment tier policy hot/warm/cold + archive tier (NUEVO-17)`
- Estado: ⬜ PENDING

## Dependencias
- No requiere tareas previas del backlog (es Phase 8 post-launch, independiente).
- Depende de la infra ya existente (SegmentRegistry/LsmConfig) — ya cumplido.

## Notas
- El hallazgo CLAVE del discovery: no partir de cero; la infra de niveles ya está en `src/lsm.rs`. El trabajo es: (1) definir policy, (2) añadir archive L3 config, (3) encadenar promoción, (4) tests + métricas.
- Recordar: NUNCA tocar `pack_offset` semántica (6 bits segment id). Underscore `LsmReport` no tiene covering tests — agregar con la promoción.
- Backward-compat: el nombre de archivos `vstore_L0/L1/L2/L3.v` se mantiene. Migración legacy vector_store.v vanta → vstore_L0 ya funciona en `open_or_create`.
