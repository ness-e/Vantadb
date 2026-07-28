---
title: "ADR-COMP-026: Multi-level LSM Compaction (L0→L1→L2→L3)"
type: adr
status: proposed
tags: [vantadb, architecture, storage, compaction, lsm]
created: 2026-07-28
last_reviewed: 2026-07-28
---

# ADR-COMP-026: Multi-level LSM Compaction (L0→L1→L2→L3)

## Resumen Ejecutivo

Se introduce un modelo de compactación multi-nivel (L0→L1→L2→L3) dividiendo el VantaFile único en **múltiples archivos VantaFile por nivel**, con direccionamiento HNSW mediante **empaquetado de `(segment_id, offset)` en el campo `storage_offset: u64`** aprovechando que los offsets son siempre múltiplos de 64 (los 6 bits bajos están libres). Esto permite compactar niveles independientemente, reduce la amplificación de escritura de O(all data) a O(L0 size), y es 100% retrocompatible con VantaFiles existentes.

## Contexto

### Estado actual

- VantaDB almacena **todos los nodos (vectores + headers)** en un único archivo `VantaFile` mapeado en memoria.
- `merge_segments()` delega en `compact_layout_bfs()`, que **reescribe el archivo completo** en orden BFS, saltando tombstones.
- El `MergeReport` siempre reporta `segments_before: 1, segments_after: 1`.
- HNSW apunta a `storage_offset: u64` — offset absoluto de 64 bits dentro del VantaFile.
- Los backends (Fjall/RocksDB) gestionan metadata con su propio LSM interno, no se tocan.

### Problema

1. **Amplificación de escritura**: compactar un 5% de datos nuevos requiere reescribir el 100% del VantaFile.
2. **Sin aislamiento por nivel**: no se puede compactar datos recientes (con alta tasa de tombstones) sin tocar datos fríos.
3. **Sin gradualidad**: la compactación es un evento binario (se hace o no se hace) — no progresiva.
4. **Cola de escritura única**: la compactación bloquea inserts mediante `insert_lock` durante toda la operación.

### Restricciones

- `storage_offset` en HNSW debe seguir siendo `u64` (es un campo de estructura usado en hot path).
- Backends KV (Fjall/RocksDB) no se modifican — su LSM es independiente.
- El PipelineMode actual (COMP-013) debe coexistir o extenderse.
- La compatibilidad hacia atrás con VantaFiles existentes es obligatoria.

## Opciones Consideradas

### Opción A: Segmentación Física Múltiple (elegida)

Dividir el espacio de nodos en **múltiples archivos VantaFile**, uno por nivel (L0, L1, L2, L3). Cada nivel es un VantaFile completo con su propio mmap y write_cursor. Los nodos nuevos se escriben siempre en L0. La compactación promueve nodos de L0→L1, L1→L2, etc.

El direccionamiento HNSW usa **empaquetado en los 6 bits bajos de `storage_offset`**, que están siempre a 0 porque los offsets son múltiplos de `STORAGE_ALIGNMENT=64`:

```
bits 63..6: offset dentro del segmento (64-aligned, 58 bits → 288PB/segment)
bits  5..0: segment_id (6 bits → 64 segmentos posibles)
```

- **Escritura**: se escribe en L0 → se obtiene offset local → se empaqueta `(segment_id=0, offset)`.
- **Lectura**: se desempaqueta `segment_id` → se selecciona el VantaFile correspondiente → se lee en `offset_local`.
- **Promoción**: se copian nodos de L0→L1 → se empaqueta `(segment_id=1, nuevo_offset)` → se actualiza HNSW.

### Opción B: VantaFile Único con Capas Delta

Mantener un solo VantaFile, pero separar la región de datos en zonas "caliente" (L0) y "fría" (L1/L2) dentro del mismo archivo. La compactación mueve datos entre zonas en lugar de reescribir todo.

- **Pros**: sin cambios al modelo de direccionamiento.
- **Contras**: el archivo sigue siendo único → crece monoliticamente → no se puede truncar L0 independientemente → mmap de todo el archivo es costoso. La amplificación de escritura sigue siendo alta porque las zonas comparten el mismo archivo.

### Opción C: Usar Backend KV para Vectores (Fjall/RocksDB)

Mover los datos vectoriales al StorageBackend (Fjall/RocksDB) en lugar de VantaFile. Aprovechar el LSM nativo del backend KV.

- **Pros**: zero esfuerzo de implementación de LSM, escalado "gratis".
- **Contras**: pierde la eficiencia de mmap directo para lectura de vectores (el backend KV serializa/deserializa), overhead de serialización por nodo, el benchmark actual depende de mmap para latencias de 1.2ms. Reverse de una decisión arquitectónica fundamental.

### Opción D: VantaFile Shard + Registry Table (descartada)

Partir el VantaFile en fragmentos de tamaño fijo (ej: 64MB). Cada fragmento es un archivo mmap independiente pare a una "SST". Un `SegmentRegistry` (almacenado en metadata backend) mapea fragmentos a niveles.

- **Pros**: granularidad fina, paralelismo de compactación.
- **Contras**: complejidad alta de gestión de fragmentos, 6 bits de segment_id (64 segmentos) son muy limitados para fragmentación, overhead de abrir/cerrar muchos mmaps.

## Decisión: Opción A — Segmentación Física Múltiple con Offset Packing

| Aspecto | Decisión |
|---------|----------|
| Archivos | `vstore_L0.vanta`, `vstore_L1.vanta`, `vstore_L2.vanta`, `vstore_L3.vanta` |
| Direccionamiento HNSW | `pack_offset(segment_id, local_offset)` en u64 |
| Indirección | Desempaquetado inline (operación de bits) — 0 overhead |
| Backward compat | segment_id=0 = archivo legacy, sin cambios |
| Locking | Un `RwLock` por nivel (no un lock global para toda la compactación) |

## Diagrama de Niveles

```
                    ┌─────────────────────────────────────────────────┐
                    │               Writes / Inserts                  │
                    └─────────────────────┬───────────────────────────┘
                                          │
                                          ▼
                    ┌─────────────────────────────────────────────────┐
         L0 ◄───────│           vstore_L0.vanta   (≤ 64 MB)          │────── Hot tier
                    │     muchos tombstones, datos recientes          │    compactación
                    └─────────────────────┬───────────────────────────┘    frecuente
                                          │ compact L0 → L1
                                          ▼
                    ┌─────────────────────────────────────────────────┐
         L1 ◄───────│           vstore_L1.vanta   (≤ 512 MB)         │────── Warm tier
                    │     datos consolidados, pocos tombstones        │    compactación
                    └─────────────────────┬───────────────────────────┘    periódica
                                          │ compact L1 → L2
                                          ▼
                    ┌─────────────────────────────────────────────────┐
         L2 ◄───────│           vstore_L2.vanta   (≤ 4 GB)           │────── Cool tier
                    │     datos fríos, sin tombstones                 │    compactación
                    └─────────────────────┬───────────────────────────┘    infrecuente
                                          │ compact L2 → L3
                                          ▼
                    ┌─────────────────────────────────────────────────┐
         L3 ◄───────│           vstore_L3.vanta   (sin límite)       │────── Archive tier
                    │     datos de archivo, solo lectura              │    nunca se compacta
                    └─────────────────────────────────────────────────┘
```

### Flujo de búsqueda

```
Search(query) → HNSW.search(query) → [(node_id, score, storage_offset), ...]
                                         │
                                         ▼
                    segment_id = storage_offset & 0x3F
                    seg_offset = storage_offset & !0x3F
                                         │
                                         ▼
                    segment = registry[segment_id]
                    segment.vfile.read_header(seg_offset)  → DiskNodeHeader
                    segment.vfile.mmap_bytes()[seg_offset..]  → vector f32
```

## Estructura de Datos para Indirección HNSW→Segmento

### Empaquetado de offset (zero-overhead)

```rust
/// Número de bits para segment_id en storage_offset.
const SEGMENT_ID_BITS: u64 = 6;
const SEGMENT_ID_MASK: u64 = (1 << SEGMENT_ID_BITS) - 1;  // 0x3F

/// Empaqueta segment_id + offset local en un u64 compatible con storage_offset.
/// Precondición: `local_offset` es múltiplo de STORAGE_ALIGNMENT (64).
fn pack_offset(segment_id: u8, local_offset: u64) -> u64 {
    debug_assert!(
        local_offset.is_multiple_of(STORAGE_ALIGNMENT),
        "local_offset must be 64-aligned"
    );
    debug_assert!(
        (segment_id as u64) < SEGMENT_ID_MASK,
        "segment_id out of range"
    );
    (local_offset & !SEGMENT_ID_MASK) | (segment_id as u64 & SEGMENT_ID_MASK)
}

/// Desempaqueta segment_id y offset local.
fn unpack_offset(packed: u64) -> (u8, u64) {
    let segment_id = (packed & SEGMENT_ID_MASK) as u8;
    let local_offset = packed & !SEGMENT_ID_MASK;
    (segment_id, local_offset)
}
```

### SegmentRegistry

```rust
/// Información de un segmento VantaFile en el registry.
#[derive(Debug, Clone)]
struct SegmentInfo {
    /// ID del segmento (0-63).
    segment_id: u8,
    /// Nivel LSM (0=L0, 1=L1, 2=L2, 3=L3).
    level: u8,
    /// Path al archivo VantaFile.
    path: PathBuf,
    /// Tamaño actual del archivo en bytes.
    size: u64,
    /// Timestamp de última compactación.
    last_compacted: Option<Instant>,
    /// Ratio de tombstones estimado.
    tombstone_ratio: f32,
}

/// Registry global de segmentos, protegido por RwLock.
/// Persistido en BackendPartition::InternalMetadata.
struct SegmentRegistry {
    segments: Vec<SegmentInfo>,
    /// Mapeo rápido: segment_id → índice en segments.
    by_id: [Option<usize>; 64],
}

impl SegmentRegistry {
    /// Abre o crea los archivos VantaFile para cada nivel.
    fn open_or_create(data_dir: &Path) -> Result<(Self, Vec<VantaFile>)> { ... }

    /// Retorna el VantaFile para un segment_id dado.
    fn vfile(&self, segment_id: u8) -> &RwLock<VantaFile> { ... }

    /// Serializa el registry al backend de metadata.
    fn persist(&self, backend: &dyn StorageBackend) -> Result<()> { ... }
}
```

## Flujo de Compactación/Promoción

### Algoritmo general

```
compact(source_level, target_level):
  1. Lock source segment (write lock)
  2. Lock target segment (write lock)
  3. Scan source VantaFile secuencialmente
  4. Para cada nodo no-tombstone:
     a. Leer header + vector desde source VantaFile
     b. Escribir en target VantaFile → obtener local_offset
     c. Acumular en offset_map: node_id → (target_segment_id, local_offset)
  5. Cerrar source (write unlock)
  6. Actualizar HNSW offsets via reindex_nodes(offset_map)
  7. Truncar source VantaFile a tamaño inicial (~64KB)
  8. Persistir SegmentRegistry
  9. Liberar source/target locks
```

### Compactación L0→L1 (detalle)

```
compact_L0_to_L1():
  source = registry.vfile(segment_of(L0))
  target = registry.vfile(segment_of(L1))

  let offset_map: HashMap<u128, u64> = HashMap::new()

  // Scan secuencial de L0
  cursor = STORAGE_ALIGNMENT
  while cursor < source.write_cursor:
    header = source.read_header(cursor)
    if header is tombstone → skip
    if header.id == 0 → skip

    // Leer vector data
    vec_data = source.mmap_bytes()[header.vector_offset..]

    // Escribir en L1, obtener nuevo offset
    new_local_offset = append_node(target, header, vec_data)

    // Empaquetar (L1_segment_id, new_local_offset)
    packed = pack_offset(L1_SEGMENT_ID, new_local_offset)
    offset_map.insert(header.id, packed)

    cursor = next_node_boundary(cursor, header)

  // Actualizar HNSW
  hnsw = self.hnsw.load()
  reindex_nodes(&hnsw, &offset_map)

  // Truncar L0 a tamaño inicial
  source.truncate_to(MIN_SEGMENT_SIZE)

  // Guardar metadata de niveles
  registry.mark_compacted(L0_SEGMENT_ID)
  registry.persist(self.backend)
```

### Consideración de lecturas concurrentes

Durante la compactación L0→L1, las lecturas que caen en L0 pueden leer datos viejos (antes de la promoción). Para evitarlo:

1. Se completa la escritura a L1 **antes** de truncar L0.
2. Luego se actualizan los offsets HNSW atómicamente via `reindex_nodes`.
3. **Solo entonces** se trunca L0.

Esto significa que por una ventana de tiempo, L0 y L1 tienen los mismos datos. Los searches ven los offsets actualizados en HNSW casi inmediatamente (arc_swap Guard), y caen en L1. Las lecturas que aún tengan el offset viejo caerán en L0, que sigue intacto hasta el truncado. Es seguro.

### Manejo de fallos

Si el proceso crashea durante la compactación L0→L1:
- L1 tiene datos duplicados parciales → en recovery, se scannea L0 y L1, y se eligen los offsets más recientes (por write_cursor position).
- O más simple: en recovery, se detecta compactación incompleta (L0 truncado + L1 con datos + registry desactualizado) y se **rehace** la compactación desde L0.
- Si el crash es antes de truncar L0: L0 está intacto, L1 puede tener datos duplicados que se ignoran en recovery (se prefiere L0).
- Si el crash es después de truncar L0: L1 tiene todos los datos, registry tiene los nuevos offsets, no hay pérdida.

## Política de Promoción

### Gatillos configurables

La compactación se dispara cuando **cualquier** nivel supera sus umbrales:

| Nivel | Tamaño máx | Tombstone ratio | Frecuencia esperada |
|-------|------------|-----------------|---------------------|
| L0 | 64 MB | >20% | Cada ~segundo (writes intensivos) |
| L1 | 512 MB | >15% | Cada ~minuto |
| L2 | 4 GB | >10% | Cada ~hora |
| L3 | ∞ (archive) | N/A | Nunca |

### Algoritmo de decisión

```rust
fn should_compact_level(segment: &SegmentInfo, config: &LsmConfig) -> Option<u8> {
    let size_ratio = segment.size as f64 / config.max_size_for_level(segment.level);

    if segment.tombstone_ratio > config.vacuum_threshold_pct / 100.0 {
        return Some(segment.level); // compactar por tombstones
    }
    if size_ratio > 1.0 {
        return Some(segment.level); // compactar por tamaño
    }
    None
}
```

### Configuración (extensión de SegmentOptimizerConfig)

```rust
/// Configuración LSM, añadida a SegmentOptimizerConfig.
#[derive(Debug, Clone, Copy)]
pub struct LsmConfig {
    /// Tamaño máximo de L0 antes de forzar compactación (default: 64 MB).
    pub l0_max_size: u64,
    /// Tamaño máximo de L1 (default: 512 MB).
    pub l1_max_size: u64,
    /// Tamaño máximo de L2 (default: 4 GB).
    pub l2_max_size: u64,
    /// Umbral de tombstones para compactar L0 (default: 20.0%).
    pub l0_tombstone_threshold: f32,
    /// Umbral de tombstones para compactar L1 (default: 15.0%).
    pub l1_tombstone_threshold: f32,
    /// Umbral de tombstones para compactar L2 (default: 10.0%).
    pub l2_tombstone_threshold: f32,
    /// Tamaño inicial de un segmento vacío (default: 64 KB).
    pub min_segment_size: u64,
}
```

Los valores default se eligen para que L0 quepa cómodamente en L1/L2 cache de CPU, L1 quepa en RAM disponible, y L2 archive en disco.

## Integración con Pipeline (COMP-013)

### Nuevo PipelineMode

```rust
pub enum PipelineMode {
    Full,                               // Vacuum → CompactL0 → FreshHNSW → MergeAll → Reindex
    VacuumOnly,
    CompactOnly,          // NUEVO: solo compactación LSM multi-nivel
    CompactL0Only,        // NUEVO: compactar solo L0→L1
    MergeOnly,            // legacy: compactación BFS del VantaFile único
    IndexOnly,
    FreshHnswOnly,
}
```

### Orden del pipeline Full (modificado)

```
Full pipeline (nuevo):
Phase 1: Vacuum        — purgar tombstones de HNSW (como hoy)
Phase 2: CompactL0     — compactar L0→L1 si thresholds excedidos
Phase 3: CompactL1     — compactar L1→L2 si thresholds excedidos
Phase 4: CompactL2     — compactar L2→L3 si thresholds excedidos (opcional)
Phase 5: FreshHNSW     — reparar orphan links (después de compactación)
Phase 6: Reindex       — rebuild total del índice HNSW (opcional)
```

La compactación LSM ocurre después de vacuum (para evitar promover tombstones) y antes de FreshHNSW (porque los cambios de offset pueden crear orphan links temporales).

### RunPipeline modificado

```rust
fn run_pipeline(&self, mode: PipelineMode) -> Result<PipelineReport> {
    // Phase 1: Vacuum (unchanged) ...
    // Phase 2-4: LSM Compaction (nuevo)
    if matches!(mode, Full | CompactOnly) {
        for level in 0..=2 {
            if let Some(info) = self.compact_level(level) {
                compact_reports.push(info);
            }
        }
    }
    // Phase 5: FreshHNSW (unchanged) ...
    // Phase 6: Reindex (unchanged) ...
}
```

## Concurrencia

### Modelo de locks

```
StorageEngine {
    vector_store: Vec<RwLock<VantaFile>>,   // Antes: RwLock<VantaFile>
    //                                ^^^ Array de VantaFile, uno por segmento activo
    segment_registry: RwLock<SegmentRegistry>,
    insert_lock: FairMutex<()>,              // Sigue protegiendo HNSW inserts
    hnsw: ArcSwap<CPIndex>,                 // Sin cambios
    ...
}
```

### Reglas de concurrencia

| Operación | Locks adquiridos |
|-----------|-----------------|
| **Insert** (write node to L0) | `vector_store[L0].write()` → `insert_lock` (HNSW) |
| **Search** (read from any level) | `vector_store[seg].read()` para cada nivel buscado |
| **Compact L0→L1** | `vector_store[L0].write()` + `vector_store[L1].write()` |
| **Read segment registry** | `segment_registry.read()` |
| **Write segment registry** | `segment_registry.write()` |

### Safety

- **Lecturas durante compactación**: permitidas. Los VantaFile por nivel son independientes. Mientras L0 se compacta a L1, las lecturas a L1 siguen funcionando (L1 está completo antes de truncar L0). Las lecturas a L0 que caen en un offset ya promovido ven datos consistentes hasta el truncado.
- **Writes durante compactación**: bloqueados para el nivel source (L0). Los inserts normalmente escriben a L0; durante compactación, se detienen hasta que se libere L0. Alternativa futura: buffer de writes en L0.5 durante compactación.
- **Rollback**: si la compactación falla antes de truncar L0, no hay daño (L0 intacto). Si falla después de truncar L0 pero antes de persistir registry, recovery detecta L0 truncado + L1 con datos y rehace el mapeo.

### Snapshots (arc_swap)

HNSW se actualiza via `arc_swap::ArcSwap<CPIndex>`. Los offsets actualizados por compactación se ven inmediatamente para nuevos searches. Los searches en progreso con el viejo Arc ven los offsets viejos y leen de L0 (que sigue ahí hasta el truncado). No hay carrera.

## Compatibilidad Hacia Atrás

### VantaFiles existentes

Un VantaFile legacy `vector_store.vanta` se maneja así:

1. **En startup**, `SegmentRegistry` no encuentra metadata LSM. Detecta `vector_store.vanta` existente.
2. Lo trata como **segment 0, L0**, con todos los nodos apuntando a offsets sin empaquetar (segment_id = 0).
3. Todos los `storage_offset` existentes tienen `segment_id=0` porque los 6 bits bajos ya son 0 (64-aligned) — **cero cambios en los valores en memoria**.
4. En el primer ciclo de compactación, se promueven nodos a L1→L2 con offsets empaquetados.

### Migración

```
Startup con VantaFile legacy:
  1. Renombrar vector_store.vanta → vstore_L0.vanta
  2. Crear vstore_L1.vanta, vstore_L2.vanta, vstore_L3.vanta (vacíos)
  3. Inicializar SegmentRegistry: segment[0] = L0 (existente)
  4. Primer CompactL0 no hace nada (L0 no ha crecido)
  5. Primer CompactL1 mueve datos de L1→L2 (L1 está vacío, no-op)
  
  6. Eventualmente, umbral de L0 se excede → compact L0→L1
     - Todos los nodos legacy se mueven a L1
     - HNSW offsets se actualizan con segment_id=1 empaquetado
     - L0 se trunca a tamaño mínimo
     - El sistema ahora es completamente multi-nivel
```

### Formato de archivo

No hay cambios en el formato binario de VantaFile (magic `VFLE`). Cada archivo de nivel es un VantaFile estándar. El `VFILE_VERSION` sigue siendo 2. Solo cambia el nombre del archivo y cómo se interpreta `storage_offset`.

## Criterios de Éxito

### Funcionales

- [ ] Ingest: inserts escriben en L0, no tocan L1/L2/L3
- [ ] Compaction L0→L1: datos promovidos correctamente, tombstones excluidos
- [ ] Compaction L1→L2: mismo patrón, niveles separados
- [ ] Search conoffsets multi-nivel: lectura desde cualquier nivel funciona correctamente
- [ ] Búsqueda híbrida HNSW + BM25: funciona sin cambios (la metadata sigue en backend KV)

### De rendimiento

- [ ] Amplificación de escritura en compactación: de O(all data) a O(L0 size). Con L0=64MB y DB=10GB, la compactación reescribe 64MB en lugar de 10GB (~156x mejora)
- [ ] Compactación concurrente con lecturas: las queries no se bloquean durante compactación
- [ ] Compactación L0→L1 completada en <1s para L0=64MB típico
- [ ] Overhead de desempaquetado de offset: <1ns por acceso (operación de bits)

### De robustez

- [ ] Crash recovery: si el proceso falla durante compactación, recovery restaura estado consistente
- [ ] Error handling: si la compactación falla (disk full, IO error), el sistema degrada gracefulmente (sigue sirviendo lecturas desde L0)
- [ ] Backward compat: bases de datos existentes abren sin errores y migran en background

### De observabilidad

- [ ] Métrica: `vantadb_lsm_level_size_bytes{level="L0"}` por nivel
- [ ] Métrica: `vantadb_lsm_compaction_duration_ms` por operación de compactación
- [ ] Métrica: `vantadb_lsm_tombstone_ratio` por nivel
- [ ] Report ampliado: `LsmReport` con detalles por nivel en `PipelineReport`

## Plan de Implementación

### Fase 1: Infraestructura (Riesgo: bajo)

1. Crear `SegmentRegistry` struct con load/save desde metadata backend
2. Modificar `StorageEngine.vector_store` de `RwLock<VantaFile>` a `Vec<RwLock<VantaFile>>`
3. Añadir `LsmConfig` a `SegmentOptimizerConfig`
4. Crear funciones `pack_offset()` / `unpack_offset()` con tests

### Fase 2: Writes multi-nivel (Riesgo: medio)

1. Modificar `write_node_to_vstore()` para escribir en L0 siempre
2. Modificar `compact_layout()` y `traverse_graph()` para operar sobre un nivel específico
3. Implementar `compact_level(level)` que promueve nodos de level→level+1
4. Modificar `merge_segments()` para usar el nuevo sistema

### Fase 3: Pipeline integración (Riesgo: bajo)

1. Añadir `PipelineMode::CompactOnly` y `PipelineMode::CompactL0Only`
2. Modificar `run_pipeline()` para ejecutar fases de compactación LSM
3. Modificar `PipelineReport` para incluir `LsmReport`

### Fase 4: Recovery y robustez (Riesgo: medio-alto)

1. Implementar detección de compactación incompleta en startup
2. Tests de crash durante compactación
3. Tests de recovery con datos parcialmente promovidos

### Fase 5: Limpieza de legacy (Riesgo: bajo)

1. Deprecar `MergeOnly` mode legacy (mantener como alias de `CompactOnly`)
2. Migración automática de VantaFile único a multi-nivel en startup

## Alternativas Rechazadas (detalle)

### Opción B (Capas Delta) — Rechazada por:

- Un solo mmap gigante → presión en TLB y page cache
- No se puede truncar/compactar L0 independientemente
- La fragmentación interna hace que "promover" datos sea tan costoso como reescribir todo

### Opción C (Backend KV) — Rechazada por:

- Pérdida de la ventaja de mmap: lectura directa de vectores sin serialización
- Los benchmarks actuales (1.2ms latencia híbrida) dependen de mmap de vectores
- Fjall/RocksDB añaden overhead de serialización por nodo
- Sería un redesign completo del storage layer

### Opción D (Shard + Registry Table) — Rechazada por:

- 64 segmentos (6 bits) insuficientes para sharding fino
- Gestión de muchos archivos mmap abre complejidad en lifetime de mappings
- La ganancia sobre la Opción A es marginal para el caso de uso actual

## Referencias

- COMP-013: SegmentOptimizer pipeline
- VantaFile: `src/storage/vfile.rs` (formato VFLE, mmap, write_cursor)
- CompactLayout: `src/storage/archive.rs`
- HNSW storage_offset: `src/index/graph.rs` (HnswNode.storage_offset)
- MergeSegment: `src/storage/engine/maintenance.rs` (merge_segments, compact_layout_bfs)
- Pipeline: `src/storage/engine/maintenance.rs` (run_pipeline)
- Glosario LSM: `docs/glosario/lsm-tree.md`
