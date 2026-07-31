# COMP-026: Multi-level LSM Compaction (L0→L1→L2→L3)

**Prioridad:** 🟡 Media | **Esfuerzo:** 🟠 1-2 sem | **Dependencia:** COMP-013 ✅

---

## Objetivo

Extender el sistema de almacenamiento de VantaDB para soportar **múltiples niveles de segmentos** con compactación estilo LSM (Log-Structured Merge-tree), donde los datos migran automáticamente entre niveles según edad/tamaño.

## Contexto técnico

- **VantaFile** actual: un único archivo mapeado en memoria para todos los nodos de un índice. `merge_segments()` compacta tombstones pero no divide en niveles.
- **Backends (Fjall/RocksDB):** ya tienen LSM interno para sus keyspaces (metadata, text index, tombstones). No se tocan.
- **COMP-013:** `SegmentOptimizer` pipeline con Vacuum→FreshHNSW→Merge→Reindex. `PipelineMode::Full/VacuumOnly/MergeOnly/IndexOnly/FreshHnswOnly`.
- **El gap:** No hay tiering de datos vectoriales/graph. El VantaFile crece sin límite, y al hacer merge se reescribe completo.

## Diseño arquitectónico — ✅ COMPLETADO (2026-07-28)

ADR en `docs/adr/COMP-026-lsm-compaction-design.md` (545 líneas). Ver sección "Decisiones clave" abajo.

## Decisiones clave (del ADR)

1. **Arquitectura:** Archivos `.vanta` separados por nivel (L0: vstore_L0.vanta, L1: vstore_L1.vanta, etc.)
2. **Direccionamiento HNSW:** `storage_offset: u64` = 6 bits segment_id + 58 bits offset local. Los 6 bits bajos ya estaban en 0 por el alignment a 64 bytes → overhead 0, retrocompatible.
3. **Compactación independiente:** Solo se reescribe el nivel fuente, no todos. L0→L1: reescribe ~64MB en vez de ~10GB (156x mejora).
4. **Pipeline:** `Vacuum → CompactL0 → CompactL1 → CompactL2 → FreshHNSW → Reindex`
5. **Concurrencia:** `Vec<RwLock<VantaFile>>` (un lock por nivel). Lecturas no bloquean durante compactación.
6. **Retrocompatibilidad:** En startup con VantaFile legacy → renombrar a `vstore_L0.vanta`, crear niveles vacíos.

## Archivos clave para implementación

1. **¿LSM a nivel VantaFile o particionado lógico?**
   - Opción A: Dividir VantaFile en segmentos físicos múltiples (varios archivos mmap)
   - Opción B: Usar los backends (Fjall/RocksDB) también para datos vectoriales
   - Opción C: Mantener VantaFile único pero añadir archivos delta/parche (niveles como "capas" de escritura)

2. **Estructura de niveles**
   - L0: Writes recientes, buffered en memoria + pequeño archivo delta
   - L1: Segmento compactado de size mediano
   - L2: Segmento grande, altamente compactado
   - L3: Archive/read-only (opcional, para cold data)

3. **Política de promoción**
   - ¿Por tamaño del nivel? (ej: L0 < 64MB → L1 < 512MB → L2)
   - ¿Por edad? (ej: datos > 7 días → L2)
   - ¿Por ratio de tombstones? (ej: > 30% tombstones → compactar y promover)

4. **Integración con pipeline existente**
   - Nuevo `PipelineMode` variant: `CompactOnly` o `CompactLevel { target: Level }`
   - ¿Fase separada o extender Merge existente?

5. **HNSW + segmentos múltiples**
   - HNSW actualmente apunta a `storage_offset` dentro del VantaFile
   - Si hay múltiples segmentos, los offsets cambian al promover/compactar
   - Necesito capa de indirección (offset → (segment_id, offset_in_segment))

6. **Concurrencia**
   - Lecturas durante compactación
   - Writes bloqueantes o no-bloqueantes durante promoción

## Plan de implementación (después del diseño)

### Phase 1: Split + Merge inteligente
- Implementar `SegmentLevel` enum
- Modificar `merge_segments()` para aceptar level target
- Si VantaFile > threshold, dividir en 2 segmentos L0→L1

### Phase 2: Política de promoción
- `CompactPolicy` trait/trait object con `should_compact(source_level, stats) -> bool`
- Default policy: size-based + tombstone-ratio
- Configurable via `SegmentOptimizerConfig`

### Phase 3: Pipeline extendido
- Nuevo `PipelineMode::CompactLevel { target: SegmentLevel }`
- Phase entre Vacuum y Merge en el pipeline

### Phase 4: SDK/API
- Exponer `compact(level)` en SDK
- CLI `vantadb compact --level L2`

## Criterios de éxito

- [ ] 3+ niveles funcionales (L0, L1, L2 como mínimo)
- [ ] Datos promovidos L0→L1→L2 sin pérdida ni duplicación
- [ ] HNSW searches funcionan correctamente post-compactación
- [ ] Pipeline integrado: `run_pipeline(PipelineMode::CompactLevel { level: SegmentLevel::L0 })`
- [ ] Tests: promoción, lectura cross-segment, rollback en fallo
- [ ] Benchmarks: write throughput mejora al tener L0 pequeño

## Archivos clave

- `src/storage/engine/maintenance.rs` — merge_segments(), run_pipeline()
- `src/storage/engine/mod.rs` — PipelineMode, MergeReport, SegmentOptimizerConfig
- `src/storage/vantafile.rs` — VantaFile (el archivo único a segmentar)
- `src/config.rs` — configuración de niveles
- `src/sdk/api.rs` — API pública
- `src/console.rs` — CLI

## Notas

- **Ponytail:** No implementar compresión LZ4/zstd por nivel (eso es OLD-07). No implementar archive tier L3 a menos que haya demanda. El mínimo viable es L0 (hot) + L1 (warm).
- **Compatibilidad:** Los formatos de VantaFile existentes deben seguir siendo legibles.
- **Fjall/RocksDB:** Sus LSM internos son independientes — no acoplarlos al tiering de VantaDB.
