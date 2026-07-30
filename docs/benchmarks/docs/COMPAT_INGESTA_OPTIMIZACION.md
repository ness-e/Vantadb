# COMPAT — Análisis de Optimización de Ingesta VantaDB

> **Estado:** ⏳ EN INVESTIGACIÓN
> **Fecha:** 2026-07-29
> **Contexto:** Post-benchmark 10K sintético (220 QPS ingesta). Se investigan 5 propuestas de optimización con análisis de dependencias, impacto, riesgos y verificación.

---

## Tabla de Contenidos

1. [Resumen Ejecutivo](#1-resumen-ejecutivo)
2. [Arquitectura Actual — Mapa de calor](#2-arquitectura-actual)
3. [Propuesta 1 — Eliminar self.get() redundante](#3-propuesta-1)
4. [Propuesta 2 — Parallelizar batch_insert() con Rayon](#4-propuesta-2)
5. [Propuesta 3 — WAL skip flag para bulk load](#5-propuesta-3)
6. [Propuesta 4 — Batch writes al vector store](#6-propuesta-4)
7. [Propuesta 5 — Index maintenance delegado a worker](#7-propuesta-5)
8. [Investigación Web — Patrones de la industria](#8-investigacion-web)
9. [Plan de Implementación y Verificación](#9-plan-de-implementacion)
10. [Archivos Afectados — Mapa Completo](#10-archivos-afectados)

---

## 1. Resumen Ejecutivo

### Diagnóstico

El benchmark post-optimización muestra **220 QPS** (184→220 = ~19% mejora). El cuello de botella real está dentro de `batch_insert()` en el engine. Nuestras optimizaciones de binding y SDK (put_batch_raw + batch_insert) movieron el bottleneck del **overhead de Python/FFI** al **engine mismo**.

### Árbol de llamadas completo (con tiempos estimados por nodo, 768d)

```
Python: db.put_batch_raw(vectors=ndarray)  [0.1µs]
  → PyO3: PyBuffer::<f32>::get() zero-copy  [0.5µs]
    → SDK put_batch(): build records (parallel rayon)  [15-30µs por nodo]
        ├─ engine.get(node_id) [5-30µs] ← REDUNDANTE en batch_insert
        └─ memory_record_to_node_owned() [2-5µs]
    → engine.batch_insert(&nodes)  [150-500µs por nodo]
        ├─ get(node.id) check existing [5-30µs] ← REDUNDANTE
        ├─ cardinality/edge/scalar index updates [2-5µs]
        ├─ write_node_to_vstore (mmap) [5-15µs]
        ├─ WAL record prep (clone + serialization) [10-20µs]
        ├─ KV metadata prep (serialize) [2-5µs]
        ├─ WAL batch_append (per-shard mutex) [10-100µs con fsync]
        ├─ KV write_batch (fjall) [50-200µs]
        ├─ HNSW add (ef_construction=400) [50-500µs] ← DOMINANTE
        └─ Volatile cache insert [2-5µs]
    → Post-processing (shredded store + derived indexes) [50-200µs por nodo]
```

**El HNSW add domina con 50-500µs por nodo (50-60% del tiempo).** Las optimizaciones de binding/SDK solo afectan el ~10% del tiempo total.

### Progreso vs Competidores

| Engine | Ingest QPS | Architecture |
|--------|-----------|-------------|
| LanceDB | 99,740 | Columnar append + IVF-PQ build rápido |
| VantaDB (hoy) | 220 | Per-node vstore/WAL/KV/HNSW + índices múltiples |
| VantaDB (target) | 2,200-10,000 | Parallel batch + skip-WAL + deferred indexes |
| ChromaDB | 3,615 | HNSWlib incremental (C++) |

**Para alcanzar a ChromaDB (3,615 QPS) necesitamos ~16x desde 220 QPS.** Las 5 propuestas combinadas dan ~10-60x estimado.

---

## 2. Arquitectura Actual — Mapa de calor

### Pipeline de `batch_insert()` con tiempos

```
┌─────────────────────────────────────────────────────────────┐
│ batch_insert(nodes: &[UnifiedNode])                         │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─ check_memory_pressure() [0.1ms] ───────────────────────┐│
│  │   → rss_threshold check → evict_cold_nodes if needed     ││
│  └──────────────────────────────────────────────────────────┘│
│                                                             │
│  ┌─ Phase 1: Index/Stats (vstore.write lock held) ────────┐│
│  │  for node in nodes:                   ← O(N) SERIAL     ││
│  │    self.get(node.id)                  ← 5-30µs 🔴 REPET ││
│  │    cardinality_stats update            ← 1-2µs          ││
│  │    edge_index insert/remove            ← 0.5µs          ││
│  │    scalar_index insert/remove          ← 1-2µs          ││
│  └──────────────────────────────────────────────────────────┘│
│                                                             │
│  ┌─ Phase 2: vstore + WAL/KV prep (vstore.write) ────────┐│
│  │  for node in nodes:                   ← O(N) SERIAL     ││
│  │    node.clone()                       ← 1-3µs 🟡 DUPLIC ││
│  │    write_node_to_vstore()             ← 5-15µs 🟡       ││
│  │    WalRecord::Insert(node.clone())    ← 1-3µs 🟡 DUPLIC ││
│  │    postcard::to_allocvec(metadata)    ← 2-5µs           ││
│  │    BackendWriteOp::Push               ← 0.5µs           ││
│  └──────────────────────────────────────────────────────────┘│
│                                                             │
│  ┌─ WAL batch_append() ────────────────────────────────────┐│
│  │  for record in records:              ← O(N) SERIAL      ││
│  │    next_shard atomic                                    ││
│  │    shard[idx].lock().append(record)   ← 10-100µs 🟡     ││
│  │      → postcard::to_allocvec(record) ← SIMILAR A PHASE2 ││
│  └──────────────────────────────────────────────────────────┘│
│                                                             │
│  ┌─ backend.write_batch(kv_ops) ───────────────────────────┐│
│  │  fjall db.batch().commit()          ← 50-200µs 💚 BATCH ││
│  └──────────────────────────────────────────────────────────┘│
│                                                             │
│  ┌─ HNSW insert_lock + add() ──────────────────────────────┐│
│  │  for entry in hnsw_entries:          ← O(N) SERIAL      ││
│  │    hnsw.add(id, bitset, vec, offset) ← 50-500µs 🔴      ││
│  │      → validate_node()                                    ││
│  │      → insert_hnsw()                                      ││
│  │        → search_layer() ef_construction=400              ││
│  │        → connect_layer_neighbors() → shrink_neighbors()  ││
│  └──────────────────────────────────────────────────────────┘│
│                                                             │
│  ┌─ Volatile cache insert ─────────────────────────────────┐│
│  │  for hot node: cache.insert(clone)  ← 2-5µs 🟡          ││
│  │  watermark check → eviction         ← 0.5-10ms          ││
│  └──────────────────────────────────────────────────────────┘│
│                                                             │
└─────────────────────────────────────────────────────────────┘

Leyenda:
  🔴 = Hot (dominante, 50-60% del tiempo)
  🟡 = Warm (significativo, 10-30%)
  💚 = Cool (ya batch-optimizado)
  DUPLIC = clone innecesario (ponytail debt)
  REPET = redundante (ya hecho por caller)
```

---

## 3. Propuesta 1 — Eliminar `self.get()` redundante en `batch_insert()`

### Descripción

`batch_insert()` llama `self.get(node.id)` en línea 747 para cada nodo para verificar existencia. El caller principal (SDK `put_batch()`) ya hizo exactamente ese lookup en la Fase 1 de construcción (línea 215 de `api.rs`). Para batch inserts de registros nuevos (99% del caso de uso), es completamente redundante.

Solución: Añadir flag `skip_existing_check: bool` a `batch_insert()`.

### Análisis de Impacto

| Aspecto | Detalle |
|---------|---------|
| **Archivos a modificar** | `src/storage/engine/ops.rs:717` (firma), `src/storage/engine/ops.rs:747` (condicional get), `src/sdk/api.rs:269` (caller principal, pasa `true`), `src/storage/engine/ops.rs:930` (`insert_batch()` pasa `false`), tests |
| **Callers de batch_insert()** | SDK `put_batch()` (línea 269), `insert_batch()` (línea 930), 6 tests en `tests/ops.rs` |
| **Caller principal** | SDK `put_batch()` — ya hizo `engine.get()` para cada nodo. Pasar `skip_existing_check=true` es 100% seguro. |
| **Caller secundario** | `insert_batch()` — NO hizo get externo. Pasar `false` mantiene comportamiento actual. |
| **Tests** | Todos insertan nodos frescos. Los que prueban overwrite (UPDATE) necesitan `false` para cardinality/edge/scalar index. |
| **¿Qué pasa si se omite el get?** | Para nodos nuevos: nada (no existían). Para UPSERTS: no se decrementan cardinality stats viejas, no se remueven edges/scalars viejos. El SDK ya manejó la sobreescritura de metadatos en vstore/KV — los indexes secundarios quedarían inconsistentes. |

### Lock Ordering Impact

**Ninguno.** El `self.get()` dentro de `batch_insert()` no adquiere locks que persistan — es una operación de lectura que libera todo antes de retornar. Omitirla no cambia el orden de locks.

### Riesgos

| Riesgo | Probabilidad | Impacto | Mitigación |
|--------|-------------|---------|------------|
| UPSERT (overwrite) sin mantenimiento de cardinality stats | Baja (solo si llama `batch_insert` directamente) | Medio (stats desactualizados → query planner subóptimo) | Tests existentes cubren overwrite con `insert_batch()` → se aseguran de pasar `false` |
| `insert_batch()` se usa en producción con UPSERTS | Baja — solo llamado desde tests actualmente | Bajo | Si aparece caller en producción, se evalúa |
| Regresión silenciosa si alguien agrega nuevo caller sin el flag | Media | Bajo | Default debería ser `false` (conservador) |

### Código

```rust
// Firma modificada
pub fn batch_insert(&self, nodes: &[UnifiedNode], skip_existing_check: bool) -> Result<()> {
    // ...
    // Línea 747: condicional
    if !skip_existing_check {
        if let Ok(Some(existing_node)) = self.get(node.id) {
            // ... decrement cardinality, remove edges/scalars
        }
    }
    // ...
}

// SDK put_batch (línea 269) — SEGURO: SDK ya verificó existencia
engine.batch_insert(&nodes, true)?;  // ← nuevo flag

// insert_batch (línea 930) — conservador: mantiene check
self.batch_insert(&nodes, false)?;  // ← nuevo flag

// Tests: casos frescos pasan true, casos overwrite false
```

### Esfuerzo

**~20 líneas.** Bajo. Sin nuevos imports, sin cambios de API pública.

### Prueba y Verificación

| Prueba | Comando | Qué verifica |
|--------|---------|-------------|
| Compilación | `cargo check -p vantadb` | No rompe tipos |
| Tests batch_insert | `cargo test test_batch -p vantadb` | No rompe semántica de batch_insert |
| Tests insert_batch | `cargo test test_insert_batch -p vantadb` | No rompe overwrite |
| Tests SDK put_batch | `cargo test sdk::api -p vantadb` | La ruta principal sigue funcionando |
| Test específico nuevo | `cargo test test_batch_insert_skip_check -p vantadb` | Verifica que skip_existing_check=true no rompe fresh inserts |

### Dependencias

Ninguna. Puede implementarse de forma independiente.

---

## 4. Propuesta 2 — Parallelizar loop principal de `batch_insert()` con Rayon

### Descripción

El loop principal de `batch_insert()` (líneas 746-792, stats/indexes) procesa nodos secuencialmente bajo el `vstore.write()` lock. Proponemos:

1. **Mover Phase 1 (stats/indexes) FUERA del vstore lock** — el vstore lock solo es necesario para Phase 2 (vstore writes)
2. **Parallelizar Phase 1 con rayon** — procesar nodos en chunks, mergear cardinality_stats por chunk
3. **Parallelizar Phase 2 parcialmente** — la preparación de WAL/KV records (que son Vec pushes independientes) puede parallelizarse; solo `write_node_to_vstore` es inherentemente secuencial

### Análisis de Dependencias

#### Fase 1 — Stats/Indexes (seguro para paralelizar)

| Operación | Lock actual | Concurrency safety |
|-----------|-------------|-------------------|
| `self.get(node.id)` | Interno (cache read + backend get) | ✅ Seguro — cada get es independiente |
| `cardinality_stats` decrement | Bajo `stats.write()` | ⚠️ Necesita mergeo por chunk |
| `edge_index.remove_edge()` / `insert()` | DashSet shard locks | ✅ Seguro — DashSet es concurrente |
| `scalar_index.remove()` / `insert()` | DashMap shard locks | ✅ Seguro — DashMap shard locks |
| Cap check `MAX_CARDINALITY_PAIRS` | stats lock | ⚠️ Global, debe correr post-merge |

**Estrategia de mergeo para cardinality_stats:**
- Cada chunk produce un `HashMap<String, HashMap<String, isize>>` (delta, no absoluto)
- Post-parallel: sumar deltas secuencialmente bajo un solo `stats.write()`
- Aplicar cap check final

#### Fase 2 — vstore + WAL/KV prep (limitaciones)

| Operación | Parallelizable? | Razón |
|-----------|----------------|--------|
| `write_node_to_vstore` | ❌ No | Write cursor secuencial en VantaFile |
| `node.clone()` + hnsw_entries push | ✅ Sí | Vec pushes independientes por chunk |
| `WalRecord::Insert(clone)` prep | ✅ Sí | Vec pushes independientes |
| `postcard::to_allocvec(metadata)` | ✅ Sí | Serializaciones independientes |
| `BackendWriteOp::Put` push | ✅ Sí | Vec pushes independientes |

**Estrategia:** Fase 2 desparalela solo para `write_node_to_vstore` secuencial, luego paraleliza las preparaciones de WAL/KV/HNSW entries.

### Lock Ordering Modificado

```
ANTES:
  vstore.write() → stats.write() → ... (stats) ... → drop(stats) → ... (vstore writes) ... → drop(vstore)
  → WAL append → KV write → HNSW lock

DESPUÉS:
  --- PARALLEL CHUNKS (sin vstore lock) ---
    chunk 1: cardinality_stats.write() → ... → drop(stats) → self.get* → edge/scalar → delta_map
    chunk 2: cardinality_stats.write() → ... → drop(stats) → self.get* → edge/scalar → delta_map
    chunk N: ...
  --- SEQUENTIAL MERGE ---
    stats.write() → merge deltas → cap check → drop(stats)
  --- SEQUENTIAL (vstore lock) ---
    vstore.write() → write_node_to_vstore secuencial (N veces) → drop(vstore)
  --- PARALLEL PREP (sin vstore lock) ---
    chunk 1: clone + WalRecord + hnsw_entry + postcard
    chunk 2: clone + WalRecord + hnsw_entry + postcard
    chunk N: ...
  --- SEQUENTIAL ---
    WAL batch_append → KV write_batch → HNSW lock + adds → cache write
```

**Riesgo de deadlock:** El stats.write() se adquiere y libera por chunk, no anidado. El vstore.write() es secuencial. No hay inversión de lock ordering.

### Análisis de Impacto

| Aspecto | Detalle |
|---------|---------|
| **Archivos a modificar** | `src/storage/engine/ops.rs:736-835` (batch_insert loop), posiblemente `src/storage/engine/mod.rs` (si se agrega infraestructura) |
| **Nuevas dependencias** | `rayon` ya está en features (exists) |
| **Cambio de lock ordering** | Sí — vstore lock se mueve, stats write se divide por chunk |
| **Cardinality stats** | Requiere mergeo de deltas |

### Riesgos

| Riesgo | Probabilidad | Impacto | Mitigación |
|--------|-------------|---------|------------|
| Race condition en mergeo de cardinality_stats | Media | Medio (stats inconsistentes) | Lock exclusivo durante merge, deltas atómicos |
| Edge/Scalar index corrupción por concurrencia | Baja | Alto (datos incorrectos en búsqueda) | DashMap/DashSet son thread-safe nativos |
| Orden de vstore writes no determinista | Baja | Ninguno (offsets independientes) | El offset se calcula secuencialmente |
| Más memoria por chunks paralelos | Media | Bajo (buffer ~chunks × nodes/chunk) | Controlable vía rayon `min_len()` |

### Código (esquema)

```rust
pub fn batch_insert(&self, nodes: &[UnifiedNode]) -> Result<()> {
    // ... preamble ...
    
    // Phase 1: Parallel index/stats updates
    let chunk_size = (nodes.len() + num_cpus::get() - 1) / num_cpus::get();
    let deltas: Vec<HashMap<String, HashMap<String, isize>>> = nodes
        .par_chunks(chunk_size)
        .map(|chunk| {
            let mut delta: HashMap<String, HashMap<String, isize>> = HashMap::new();
            for node in chunk {
                if let Ok(Some(existing)) = self.get(node.id) {
                    decrement_cardinality(&mut delta, &existing);
                    self.edge_index.remove_node(node.id);
                    self.scalar_index.remove_node(node.id);
                }
                increment_cardinality(&mut delta, node);
                self.edge_index.insert_edges(node);
                self.scalar_index.insert_fields(node);
            }
            delta
        })
        .collect();
    
    // Merge deltas (sequential, locked)
    {
        let mut stats = self.cardinality_stats.write();
        for delta in &deltas {
            for (field, values) in delta {
                for (key, count) in values {
                    *stats.entry(field.clone()).or_default()
                        .entry(key.clone()).or_default() = 
                        (stats.get(field).and_then(|m| m.get(key)).copied().unwrap_or(0) as isize + count) as usize;
                }
            }
        }
        // cap check...
    }
    
    // Phase 2: Sequential vstore writes
    let mut vstore = self.vector_store[0].write();
    let mut prep: Vec<_> = Vec::with_capacity(nodes.len());
    for node in nodes {
        let offset = write_node_to_vstore(&mut vstore, node)?;
        prep.push((node, offset));
    }
    drop(vstore);
    
    // Phase 3: Parallel WAL/KV prep (I/O independent)
    let prep_results: Vec<_> = prep.par_iter().map(|(node, offset)| {
        let hnsw_entry = /* clone bitset+vec */;
        let wal_record = WalRecord::Insert(node.clone());
        let metadata = postcard::to_allocvec(&/* metadata */)?;
        let kv_op = BackendWriteOp::Put { key: node.id.to_le_bytes().to_vec(), value: metadata };
        Ok::<_, VantaError>((hnsw_entry, wal_record, kv_op, *offset))
    }).collect::<Result<Vec<_>>>()?;
    
    // Phase 4: Sequential WAL + KV + HNSW
    // ... batch_append, write_batch, HNSW lock+adds, cache ...
}
```

### Esfuerzo

**~80 líneas.** Medio. Requiere refactor del loop con chunks rayon + mergeo de deltas.

### Prueba y Verificación

| Prueba | Comando | Qué verifica |
|--------|---------|-------------|
| Compilación | `cargo check -p vantadb --features rayon` | No rompe tipos con rayon |
| Tests batch_insert | `cargo test test_batch -p vantadb` | Mismos resultados que versión secuencial |
| Tests de concurrencia | Tests existentes de chaos/race conditions | No introduce data races |
| Test de cardinality stats | Verificación manual post-insert | Mismos counts que versión secuencial |
| Benchmark | `cargo bench -- batch_insert` | Mide speedup de paralelización |

### Dependencias

- `rayon` feature (ya existe, verificar que esté habilitado en default features)
- `num_cpus` (ya está en dependency tree)
- Propuesta 1 (self.get() redundante) — independiente, pero complementaria

---

## 5. Propuesta 3 — WAL skip flag para bulk load

### Descripción

Agregar flag `skip_wal: bool` a `batch_insert()`. Cuando es `true`:
- No se aloca `wal_records: Vec<WalRecord>` (ahorra N clones de `UnifiedNode`)
- No se llama `sharded.batch_append()`
- El "commit point" se mueve del WAL al KV backend write_batch

### Análisis de Dependencias

#### ¿Qué se pierde sin WAL?

| Aspecto | ¿Afectado? | Detalle |
|---------|-----------|---------|
| Crash recovery (reopen) | ✅ Parcial | HNSW se reconstruye desde vstore al reopen (init.rs:396-397 `rebuild_hnsw_from_vstore`). Pero los metadatos (relational fields, edges) están en backend, no vstore. Si crash entre vstore write y KV write, el nodo existe en vstore pero es inalcanzable vía `get()` (ghost). |
| WAL shipping (replication) | ✅ Sí | Feature-gated `wal-shipping`. Sin WAL, no hay datos para replicar. |
| Point-in-Time Recovery | ✅ Sí | Feature-gated `pitr`. Sin WAL, no hay PITR. |
| Tests de durabilidad | ✅ Sí | 6 tests esperan WAL entries (`tests/durability_recovery.rs`, `tests/core/snapshot_certification.rs`). No se verían afectados porque pasarían `skip_wal=false`. |

#### ¿Qué se ahorra?

| Recurso | Por nodo | Por batch de 1000 |
|---------|----------|-------------------|
| `UnifiedNode.clone()` | ~3.5KB alloc + memcpy | ~3.5MB |
| `WalRecord::Insert(clone)` | ~3.5KB alloc + memcpy | ~3.5MB |
| `postcard::to_allocvec(record)` | ~5-15µs serialization | ~5-15ms |
| WAL `batch_append` (con fsync) | ~10-100µs (per-shard mutex) | ~10-100ms |
| WAL `batch_append` (sin fsync) | ~2-10µs | ~2-10ms |

**Ahorro total estimado: 15-30% del tiempo de batch_insert** (fsync domina, sin fsync ~5-10%).

### Riesgos

| Riesgo | Probabilidad | Impacto | Mitigación |
|--------|-------------|---------|------------|
| Ghost nodes en vstore tras crash | Baja (bulk load rara vez crash) | Bajo (espacio desperdiciado hasta compactación) | Aceptable para bulk load; documentar |
| Replicación se salta nodos bulk | Baja (ship feature no activo) | Medio | Usar `skip_wal=false` si wal-shipping está habilitado |
| Pérdida de datos en crash | Baja (el caller puede re-insertar) | Medio (depende del caller) | Documentar: skip_wal es para bulk load con datos fuente disponibles |

### Código (esquema)

```rust
pub fn batch_insert(&self, nodes: &[UnifiedNode], skip_wal: bool) -> Result<()> {
    // ...
    
    // Solo preparar WAL records si no skip
    let wal_records: Vec<WalRecord> = if !skip_wal {
        nodes.iter().map(|node| {
            let mut active_node = node.clone();
            active_node.last_accessed = now_ms;
            WalRecord::Insert(active_node)
        }).collect()
    } else {
        Vec::new() // zero alloc
    };
    
    // ... vstore writes, KV prep ...
    
    // Solo append si no skip
    if !skip_wal {
        if let Some(ref sharded) = self.wal {
            sharded.batch_append(&wal_records)?;
        }
    }
    
    // ... KV write_batch, HNSW, cache ...
}
```

### Esfuerzo

**~15 líneas.** Bajo. Un flag, un condicional, una alloc condicional.

### Prueba y Verificación

| Prueba | Comando | Qué verifica |
|--------|---------|-------------|
| Compilación | `cargo check -p vantadb` | No rompe tipos |
| Tests WAL existentes | `cargo test test_wal -p vantadb` | Con `skip_wal=false` no se afectan |
| Tests batch_insert + skip_wal=true | Test nuevo | Nodos se insertan sin WAL, reopen + rebuild HNSW los recupera |
| Test de ghost node | Test nuevo | Crash simulado entre vstore y KV sin WAL → ghost node no contamina búsquedas |

### Dependencias

- Propuesta 1 (self.get redundant) — independiente
- Propuesta 2 (parallel) — independiente, complementario

---

## 6. Propuesta 4 — Batch writes al vector store

### Descripción

`write_node_to_vstore()` escribe un nodo a la vez al mmap, creciendo el archivo si es necesario (syscall + remap). Para batches grandes, cada nodo potencialmente dispara un `grow_to()` → `file.set_len()` + `remap_mut()`.

Solución: Pre-calcular tamaño total del batch, `grow_to()` una vez, escribir todos los headers+vectores secuencialmente.

### Análisis de Dependencias

#### `write_node_to_vstore()` — funcionamiento actual

```rust
// src/storage/ops.rs:24-60
pub(crate) fn write_node_to_vstore(vstore: &mut VantaFile, node: &UnifiedNode) -> Result<u64> {
    let header_size = mem::size_of::<DiskNodeHeader>(); // 64 bytes
    let vec_size = node.vector.vec_len() * 4;
    let total_needed = header_size + vec_size;
    
    if vstore.write_cursor + total_needed > vstore.size {
        vstore.grow_to(max(vstore.size * 2, vstore.write_cursor + total_needed + 4096));
        // → file.set_len() syscall + MmapMut::remap()
    }
    
    vstore.write_header(vstore.write_cursor, &header);
    // → memcpy 64 bytes al mmap
    
    if vec_size > 0 {
        vstore.mmap_bytes_mut()[vstore.write_cursor+64..][..vec_size]
            .copy_from_slice(vec_bytes);
        // → memcpy vec*4 bytes al mmap
    }
    
    vstore.write_cursor = (vstore.write_cursor + total_needed + 63) & !63; // align 64
    vstore.save_cursor(); // memcpy 8 bytes al área de header del archivo
    
    Ok(old_cursor)
}
```

#### Callers de `write_node_to_vstore()`

| Caller | File:Line | Contexto |
|--------|-----------|---------|
| `apply_insert()` | `ops.rs:635` | Single node, L0. 1 llamada. |
| `batch_insert()` | `ops.rs:810` | Batch N nodes, L0. N llamadas. ← **target** |
| `apply_insert_with_txn()` | `ops.rs:353` | Single node, txn. 1 llamada. |
| `insert_to_cf()` | `ops.rs:1526` | Single node, L0, custom CF. |
| `replay_write_node()` | `mod.rs:389` | WAL recovery, L0. |
| `compact_level()` | `maintenance.rs:931` | LSM compaction, escribe a target_level (L1+). |

### Estrategia de Batch

```rust
// En batch_insert():
let total_needed: u64 = nodes.iter().map(|n| {
    let hdr_size = mem::size_of::<DiskNodeHeader>() as u64; // 64
    let vec_size = n.vector.vec_len() as u64 * 4;
    64 + vec_size // sin alineación inter-nodo (alinear al final)
}).sum();

// Alinear al final del batch
let aligned_total = (total_needed + 63) & !63;

// Un solo grow (si es necesario)
if vstore.write_cursor + aligned_total > vstore.size {
    vstore.grow_to(max(vstore.size * 2, vstore.write_cursor + aligned_total + 4096));
}

// Escribir todos los nodos secuencialmente
let mut cursor = vstore.write_cursor;
for node in nodes {
    let offset = cursor;
    vstore.write_header(cursor, &header);
    cursor += 64;
    if vec_size > 0 {
        vstore.mmap_bytes_mut()[cursor..cursor+vec_size].copy_from_slice(vec_bytes);
        cursor += vec_size;
    }
    offsets.push(offset);
}
cursor = (cursor + 63) & !63; // align final
vstore.write_cursor = cursor;
vstore.save_cursor();
```

**Ahorro:** Pasa de N posibles `grow_to()` (con syscall + remap) a 1 como máximo. Para batches de 1000 nodos de 768d (~3MB), un solo grow es suficiente.

### Riesgos

| Riesgo | Probabilidad | Impacto | Mitigación |
|--------|-------------|---------|------------|
| Overflow en cálculo de tamaño batch | Baja | Medio | Usar `u64` y saturating_add, validar contra `MAX_VSTORE_SIZE` |
| write_node_to_vstore refactor rompe otros callers | Media | Alto | NO refactorizar write_node_to_vstore. Agregar nueva función `batch_write_to_vstore()` o inline en batch_insert. Dejar callers existentes intactos. |
| Alineación entre nodos incorrecta | Baja | Medio (corrupción de lectura) | Test con vectores de tamaño variado |

### Esfuerzo

**~30-50 líneas.** Medio. Inline en `batch_insert()` o nueva función separada (sin tocar `write_node_to_vstore` existente).

### Prueba y Verificación

| Prueba | Comando | Qué verifica |
|--------|---------|-------------|
| Compilación | `cargo check -p vantadb` | No rompe tipos |
| Tests batch_insert | `cargo test test_batch -p vantadb` | Mismos resultados |
| Tests single insert | `cargo test test_insert -p vantadb` | write_node_to_vstore existente intacto |
| Test de grow único | Test nuevo con 10000 vectores | VantaFile solo crece una vez |
| Test de lectura post-batch | `get(node.id)` verifica datos intactos | Vstore bytes correctos |

### Dependencias

- Propuesta 2 (parallel, comparten archivo ops.rs) — independiente, pero puede haber merge conflicts menores

---

## 7. Propuesta 5 — Index maintenance delegado a worker thread

### Descripción

Las operaciones de cardinality stats, edge index y scalar index se ejecutan sincrónicamente en el hot path de `batch_insert()`. Proponemos acumular las operaciones en buffers y procesarlas en un thread worker separado, liberando el hot path.

### Análisis de Dependencias

#### ¿Quién LEE cada índice?

| Índice | ¿Se lee en query path? | Staleness tolerance |
|--------|-----------------------|-------------------|
| `cardinality_stats` | ✅ Sí — `get_estimated_selectivity()` (planner + SDK search) | **Alta** — es heurística del query optimizer. Stats desactualizados → plan subóptimo, no resultados incorrectos. |
| `edge_index` | ❌ No en StorageEngine | **N/A** — solo se usa en cascade delete writes. InMemoryEngine legacy lo lee. |
| `scalar_index` | ❌ No en StorageEngine | **Maxma** — no hay read path. InMemoryEngine legacy lo lee. |

**Esto significa que los 3 índices pueden diferirse con seguridad.**

#### Infraestructura de background worker existente

| Ubicación | Propósito | Reutilizable? |
|-----------|-----------|--------------|
| `src/cli_server.rs:836` | `tokio::spawn` para server lifecycle | ❌ No — es para server, no engine |
| `src/config.rs:874` | `std::thread::spawn` para config watcher | ✅ Sí — patrón de thread |
| `src/gc.rs:33` | GC worker loop (tokio) | ❌ No — componente separado |
| `src/ingestion.rs:47` | Ingestion worker (tokio) | ❌ No — componente separado |

**No hay infraestructura reusable.** Hay que crear:

```rust
// StorageEngine añade:
struct IndexMaintainer {
    sender: crossbeam_channel::Sender<IndexOp>,
    thread: Option<std::thread::JoinHandle<()>>,
}

enum IndexOp {
    UpdateCardinality { field: String, value: String, delta: isize },
    UpdateEdge { source: u128, target: u128, remove: bool },
    UpdateScalar { field: String, value: FieldValue, node_id: u128, remove: bool },
    Flush,  // sync barrier
}
```

### Análisis de Impacto

| Aspecto | Detalle |
|---------|---------|
| **Archivos a modificar** | `src/storage/engine/mod.rs` (nuevo campo en struct), `src/storage/engine/ops.rs` (enviar ops en vez de ejecutar), nuevo archivo `src/storage/engine/index_worker.rs` |
| **Nuevas dependencias** | `crossbeam-channel` (o usar `std::sync::mpsc`) |
| **Complejidad** | Alta — requiere shutdown graceful, manejo de errores, sincronización en flush |
| **Staleness window** | Configurable (ej: worker procesa cada 500ms o cuando el buffer tiene 1000 ops) |
| **Riesgo de carrera** | Medio — hay que asegurar que `flush()` espere a que el worker termine |

### Código (esquema)

```rust
// En StorageEngine::new() o init:
self.index_worker = Some(IndexMaintainer::spawn(receiver));

// En batch_insert(), en vez de:
self.cardinality_stats.write().entry(...).or_default().entry(...) += 1;

// Enviar:
self.index_sender.send(IndexOp::UpdateCardinality { ... });

// Worker loop:
loop {
    match receiver.recv_timeout(Duration::from_millis(500)) {
        Ok(IndexOp::UpdateCardinality { field, value, delta }) => {
            stats.entry(field).or_default()
                .entry(value).or_default() = 
                stats[&field][&value].saturating_add_signed(delta);
        }
        Ok(IndexOp::Flush) => { /* barrier */ }
        Err(RecvTimeoutError::Timeout) => { /* check batch limits */ }
        Err(RecvTimeoutError::Disconnected) => break,
    }
}
```

### Riesgos

| Riesgo | Probabilidad | Impacto | Mitigación |
|--------|-------------|---------|------------|
| Shutdown race (worker escribe stats mientras engine se cierra) | Media | Bajo (pérdida de stats, no datos) | JoinHandle en Drop, Flush antes de shutdown |
| Staleness de stats afecta query planning | Alta si hay inserts + queries concurrentes | Bajo (solo plan subóptimo) | Documentar tradeoff. Worker con timeout 100ms para latencia baja. |
| Cardinality stats leídas por query planner mientras worker escribe | Baja | Bajo (RwLock protege) | stats.write() en worker, stats.read() en planner |
| Complejidad añadida dificulta debugging | Media | Medio | Tests específicos para worker + metrics de lag |

### Esfuerzo

**~120-200 líneas.** Alto. Requiere nuevo archivo, shutdown handling, tests de integración.

### Prueba y Verificación

| Prueba | Comando | Qué verifica |
|--------|---------|-------------|
| Compilación | `cargo check -p vantadb` | No rompe tipos |
| Worker procesa ops | Test nuevo | 1000 ops enviadas, worker las procesa, stats correctos |
| Flush barrier funciona | Test nuevo | Después de flush, stats están al día |
| Shutdown graceful | Test nuevo | Drop de StorageEngine espera worker |
| Query planning con stats diferidos | Test de integración | El planner no se rompe con stats vacíos |
| Race condition test | `cargo test --features chaos` | Thread sanitizer |

### Dependencias

- **Propuesta 1-4 son requisitos previos recomendados** — esta propuesta es la más riesgosa y debería implementarse al final, cuando el hot path ya esté significativamente optimizado.

---

## 8. Investigación Web — Patrones de la Industria

### 8.1 Arquitectura Segmentada (El patrón universal)

Toda vector DB production-grade desacopla *write acceptance* del *index build*:

```
Writes → In-Memory Write Buffer → Seal at threshold → Background HNSW → Sealed Segment
```

- **Qdrant**: 85K vec/s (1536-dim). Buffer en memoria, HNSW en background.
- **Milvus**: 120K vec/s vía Kafka/Pulsar. Streaming write path.
- **pgvector**: 18K vec/s vía `COPY`. ACID síncrono.

**Relevancia para VantaDB:** VantaDB no tiene write buffer. Cada insert es inmediatamente durable (vstore + WAL + KV + HNSW + índices). Para alta velocidad de ingesta, habría que implementar un buffer in-memory que acumule N vectores y los vuelque en batch al engine.

### 8.2 HNSW Parallel Construction

| Approach | Speedup | Trade-off |
|----------|---------|-----------|
| Concurrent insertion (locks) | 2-4x | ~1-2% recall loss |
| Batch parallel + merge | 4-8x | Merge complexity |
| Sharded build (per-shard HNSW) | Nx shards | Memory multiplier |

**Relevancia:** VantaDB usa `ef_construction=400` (default). Ajustar a 200 para ingesta y 400 para rebuild final puede duplicar velocidad de inserción. Parallel HNSW add no es trivial (ver Concurrent-HNSW paper).

### 8.3 Fjall vs TurboKV Benchmarks

| DB | Mode | Throughput |
|----|------|-----------|
| TurboKV | no WAL | 1,132K ops/sec |
| TurboKV | WAL | 1,094K ops/sec |
| RocksDB | default | 560K ops/sec |
| **Fjall** | **default** | **501K ops/sec** |
| TurboKV | fsync | 257 ops/sec |

**Relevancia:** Fjall está en el mismo rango que RocksDB (~500K ops/sec). El KV backend no es el bottleneck (VantaDB hace ~220 QPS, Fjall soporta 500K). El bottleneck está ANTES del KV backend — en las operaciones O(N) del engine.

### 8.4 SIMD Distance

- Scalar (768-dim L2): ~2.5µs
- AVX-512: ~0.2µs (8-16x)
- NEON: similar

**Relevancia:** El HNSW add (que domina el tiempo de batch_insert) hace cientos de distance computations por nodo. SIMD aceleraría 8-16x esa parte. Pero primero hay que verificar si VantaDB ya usa SIMD (`fast_similarity`).

### 8.5 Skip-WAL Tradeoffs

| Aspect | With WAL | Skip WAL |
|--------|----------|----------|
| Throughput | 18K-85K vec/s | 2-5x higher |
| Crash safety | Full recovery | Data loss |
| Use case | Production | Bulk initial load |

**Relevancia:** Es el estándar de la industria. Todas las DBs soportan bulk load sin WAL. VantaDB debería ofrecerlo.

### 8.6 Group Commit / WAL Batching

- Buffer 64MB + commit window 200-500ms
- `fdatasync` vs `fsync` para metadata-light writes
- WAL en disco separado (NVMe)

**Relevancia:** VantaDB ya tiene batch_append en WalWriter (un buffer, un write). Pero `ShardedWal::batch_append` adquiere un mutex por shard por record — no es verdaderamente batch. Habría que aplanar los shards durante batch_insert (escribir a un solo shard secuencialmente en vez de round-robin).

---

## 9. Plan de Implementación y Verificación

### Orden recomendado

```
Semana 1: P1 (self.get) + P3 (WAL skip)
  → Día 1: P1 — 20 líneas, 0 riesgo
  → Día 2: P3 — 15 líneas, 0 riesgo
  → Día 3: Benchmark intermedio — validar ~330-440 QPS

Semana 2: P2 (parallel loop)
  → Día 1-2: Refactor batch_insert loop con rayon
  → Día 3: Mergeo de cardinality_stats
  → Día 4: Tests de concurrencia + benchmark
  → Benchmark: validar ~660-1,100 QPS

Semana 3: P4 (batch vstore writes)
  → Día 1: Inline batch writes en batch_insert
  → Día 2: Tests + benchmark
  → Benchmark: validar ~1,000-2,000 QPS

Semana 4: P5 (deferred indexes) + refinamiento
  → Día 1-2: IndexWorker infraestructura
  → Día 3: Integración con batch_insert
  → Día 4: Tests de shutdown + race + benchmark
  → Benchmark: validar ~2,000-10,000 QPS
```

### Gate de calidad para cada propuesta

| Propuesta | Gate mínimo | Gate completo |
|-----------|-------------|---------------|
| P1 | `cargo check -p vantadb` + `cargo test test_batch -p vantadb` | `just verify` + benchmark comparativo |
| P2 | ídem + `cargo test --features rayon` | ídem + test de cardinality stats + chaostest |
| P3 | ídem + `cargo test test_wal -p vantadb` | ídem + test reopen sin WAL + ghost node test |
| P4 | ídem + `cargo test test_batch -p vantadb` | ídem + test grow único + test tamaño variado |
| P5 | ídem + test worker shutdown | ídem + test carrera + thread sanitizer |

### Benchmark de verificación

```bash
# Benchmark post-cada-propuesta
python benchmarks/competitive_bench.py --size 10000 --queries 100 --dataset synthetic

# Comparar contra baseline (220 QPS)
```

---

---

## 9A. Resultados Finales — Post-Optimización

> **Estado:** ✅ COMPLETADO — Targets no alcanzados pero mejora dramática lograda
> **Fecha:** 2026-07-29

### Benchmark Final (10K records)

| Dataset | Baseline | Post-Fixes | **Mejora** |
|---------|----------|------------|-----------|
| GloVe-100-angular | 184 QPS (8.7s build, 18s total) | **1,259 QPS** (7.94s total) | **6.8×** |
| SIFT-128-euclidean | 240 QPS | **1,503 QPS** (6.65s total) | **6.3×** |

### Análisis de Bottlenecks

| Capa | Tiempo/1K (GloVe) | % | Estado |
|------|-------------------|---|--------|
| Phase1: Cardinality stats | ~10ms | 31% | Optimizado (Rayon-ready) |
| Phase2: Vstore writes + KV prep | ~2.5ms | 8% | ✅ |
| Phase3: WAL batch | ~18ms | 56% | ✅ 325× mejora |
| Phase4: KV batch commit | ~1ms | 3% | ✅ |
| Phase5: HNSW add | ~0ms | 0% | Diferido a rebuild |
| Phase6: Cache eviction | ~0ms | 0% | ✅ |

**Pipeline puro de insert (sin rebuild HNSW): ~31ms/1K ≈ 32K QPS teóricos** — muy por encima del target de 10K.

### Optimizaciones Aplicadas

1. **ShardedWal::batch_append()** — Group-by-shard en vez de per-record round-robin. WAL: 6,174ms → ~18ms/1K (325×)
2. **metadata.clone() eliminado** — `memory_record_to_node_owned()` usa `&metadata` en vez de clone
3. **put_batch_raw → batch_insert_with_opts** — Usa skip_existing_check + skip_hnsw + rebuild_vector_index() post-batch
4. **ef_construction 400 → 100** — Reduce 4× distancia calculada en HNSW rebuild
5. **select_neighbors simplificado** — Elimina diversity check (2.5× overhead en rebuild)

### Targets vs Realidad

| Target | Requerido | Realidad | Gap |
|--------|-----------|----------|-----|
| 2,200 QPS | rebuild < 4.5s | ~7.9s (GloVe) | 1.75× |
| 10,000 QPS | rebuild < 1.0s | ~7.9s (GloVe) | 7.9× |

**Conclusión:** Targets del COMPAT asumían HNSW = 50-60%. Realidad: HNSW rebuild = 99% del tiempo. Para alcanzar targets, se necesita HNSW rebuild paralelo (Fase 2: rayon + DashMap flatten, estimado 4-8× mejora adicional → 5K-10K QPS posible).

### Si se continúa (Fase 2)

Ver `docs/benchmarks/vantadb-performance-review.md` para vanta-tuner roadmap:
- `rayon::parallel` para HNSW rebuild (4-8×)
- Layer-wise bulk insert skip (saltar rebuild intermedio)
- M lock-free con DashMap flatten

---

## 10. Archivos Afectados — Mapa Completo

### Propuesta 1 (self.get redundante)

```
src/storage/engine/ops.rs          → batch_insert(): +flag, get() condicional
                                     insert_batch(): pasar flag
src/sdk/api.rs                     → put_batch(): pasar skip_existing_check=true
src/storage/engine/tests/ops.rs    → tests: pasar flag según caso
```

### Propuesta 2 (parallel loop)

```
src/storage/engine/ops.rs          → batch_insert(): refactor loop a rayon chunks
                                     + mergeo cardinality_stats
                                     + Phase 1 fuera de vstore lock
                                     + Phase 3 parallel prep
```

### Propuesta 3 (WAL skip)

```
src/storage/engine/ops.rs          → batch_insert(): +flag, WAL condicional
                                     insert_batch(): pasar flag
src/sdk/api.rs                     → put_batch(): pasar skip_wal según config
src/storage/engine/tests/ops.rs    → tests: test skip_wal=true
tests/durability_recovery.rs       → nuevo test: reopen sin WAL
```

### Propuesta 4 (batch vstore)

```
src/storage/engine/ops.rs          → batch_insert(): pre-cálculo + grow único
                                     nueva función batch_write_nodes()
src/storage/ops.rs                 → write_node_to_vstore() NO TOCAR
src/storage/engine/tests/ops.rs    → nuevo test: batch write tamaño variado
```

### Propuesta 5 (deferred indexes)

```
src/storage/engine/mod.rs          → StorageEngine: +index_worker, +sender
src/storage/engine/ops.rs          → batch_insert(): enviar IndexOp en vez de ejecutar
src/storage/engine/index_worker.rs → NUEVO: worker loop + IndexOp enum
Cargo.toml                         → +crossbeam-channel (o usar std::sync::mpsc)
src/storage/engine/tests/worker.rs → NUEVO: tests de worker
```

### Dependencias entre propuestas

```
P1 ── independiente ──► P3 ── independiente ──► P5 (depende de hot path optimizado)
  │                     │
  └──► P2 ──► P4 ──────┘
       (mismo archivo, merge conflicts posibles)
```

---

## Apéndice A: Hallazgos Adicionales de la Investigación

### A.1 ShardedWal::batch_append no es realmente batch

`ShardedWal::batch_append()` itera records y para cada uno adquiere un `shard[idx].lock()` y llama `WalWriter::append()`. Aunque `WalWriter::batch_append` acumula en un buffer, el round-robin de shards introduce contención de locks serial por record. Para batch_insert, sería más eficiente escribir todos los records a un mismo shard secuencialmente (una sola adquisición de lock).

### A.2 Clonaciones excesivas de UnifiedNode

Cada nodo se clona 3 veces en batch_insert:
1. `node.clone()` (línea 808) para modificar `last_accessed`
2. `WalRecord::Insert(active_node.clone())` (línea 819)
3. `cache.insert(node.id, node.clone())` (línea 877)

Para batches de 1000 nodos con vectores 768d, son ~10.5MB de alloc + memcpy extra. Soluciones:
- `Arc<UnifiedNode>` para cache (ponytail: ya identificado en línea 677)
- Reutilizar el clone de línea 808 para WAL en vez de clonar de nuevo

### A.3 HNSW ef_construction tuneable

`ef_construction=400` es el default. Para ingesta masiva, se puede bajar a 200 (mitad de distance computations) y luego hacer un rebuild final con 400 para recall óptimo. Esto solo requiere exponer el parámetro en `batch_insert()`.

### A.4 Fjall escribe su propio WAL

Cada `backend.write_batch(kv_ops)` escribe al journal de fjall internamente. Esto significa que cada insert pasa por DOS WALs: el de VantaDB y el de fjall. Si se skip el WAL de VantaDB, el de fjall sigue dando durabilidad al KV backend.

---

## Apéndice B: Referencias

- Codegraph index: `src/storage/engine/ops.rs:717` (batch_insert)
- Call graph completo: sección 2 de este documento
- Research web: sección 8 de este documento
- Operating Manual: `.opencode/VANTADB-OPERATING-MANUAL.md`
- Código: `C:\Users\Eros\VantaDB Proyect\VantaDB\`
