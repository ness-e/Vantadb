# Plan de Implementación VantaDB

## Resumen Ejecutivo

Este documento consolida el plan técnico para evolucionar VantaDB desde un motor HNSW validado hasta una base de datos vectorial de grado empresarial con persistencia Zero-Copy.

---

## Phase 1: Estabilización de Compilación (Prioridad: Crítica)

### Objetivos

- Eliminar errores OOM del linker en Windows
- Habilitar ejecución de tests

### Acciones

1. **Limpiar tests biológicos de `Cargo.toml`:**
   - Eliminar de la sección `[[test]]`:
     - `neural_summarization`
     - `circadian_cycle`
     - `lisp_logic`
     - `immunology`
     - `cognitive_sovereignty`
     - `memory_promotion`
     - `memory_rehydration`
     - `thrashing_prevention`

2. **Eliminar archivos físicos de tests:**
   - Borrar los `.rs` correspondientes en `tests/`

3. **Crear `.cargo/config.toml`:**

   ```toml
   [target.x86_64-pc-windows-msvc]
   linker = "rust-lld.exe"
   ```

4. **Actualizar CI (`.github/workflows/`):**
   - Agregar configuración de linker para entornos de CI
   - Asegurar que el workflow use `rust-lld` en Windows

### Validación

- `cargo test --lib` corre sin errores de OOM

---

## Phase 2: Verificación y Limpieza (Prioridad: Baja)

### Objetivos

- Confirmar que el código no tiene metáforas biológicas

### Acciones

1. **Verificar renombrados existentes:**
   - Ejecutar: `grep -r "NeuronType" src/` → debe retornar 0
   - Ejecutar: `grep -r "semantic_valence" src/` → debe retornar 0

2. **Si hay resultados, aplicar renombrados:**
   - `NeuronType` → `NodeTier`
   - `semantic_valence` → `importance`

### Validación

- Tests deGrep limpios

---

## Phase 3: Zero-Copy Layout + MMap Persistente (Prioridad: Alta)

### Objetivos

- Arquitectura de memoria Zero-Copy para máximo rendimiento
- Persistencia híbrida con WAL
- Warm-up strategy para evitar page faults

### 3.1 Refactorización de UnifiedNode

```rust
#[repr(C, align(64))]
pub struct DiskNodeHeader {
    pub id: u64,
    pub bitset: u128,
    pub vector_offset: u64,  // Offset desde inicio del mmap
    pub vector_len: u32,
    pub edge_count: u16,
    pub relational_len: u32,
    pub tier: u8,
    pub flags: u32,
}
```

#### Validación de Alineación

```rust
impl DiskNodeHeader {
    /// Verifica que el nodo esté alineado a 64 bytes
    pub fn validate_alignment(&self) -> bool {
        // El tamaño total debe ser múltiplo de 64
        std::mem::size_of::<Self>() % 64 == 0
    }
}
```

**Razón**: Asegura que el siguiente nodo también-start alineado correctamente.

### 3.2 Arquitectura de Almacenamiento Híbrida

```
┌─────────────────────────────────────────────────────────────┐
│                    VantaDB Storage Layer                    │
├─────────────────────────────────────────────────────────────┤
│  Vector Store (mmap)           │  Metadata Store (RocksDB)   │
│  ─────────────────────────    │  ───────────────────────── │
│  • Fixed-size vectors         │  • Variable JSON/relational │
│  • 64-byte aligned (SIMD)    │  • Flexible schema          │
│  • Zero-Copy acceso           │  • Key-value lookups        │
│  • HNSW Index completo        │  • Filtrado de queries      │
└─────────────────────────────────────────────────────────────┘
```

### 3.3 Implementación de VantaFile

- Wrapper sobre `memmap2` con pointers basados en offsets (`u64`)
- Portabilidad entre ejecuciones
- No usa punteros de memoria (`*const`), solo offsets relativos
- **Thread-Safety**: Implementa `Send` + `Sync` para uso multi-thread (FastAPI/Gunicorn)

```rust
// VantaFile debe implementar traits seguros para uso multi-thread
unsafe impl Send for VantaFile {}
unsafe impl Sync for VantaFile {}
```

**Razón**: El SDK Python puede usarse con FastAPI/Gunicorn donde múltiples hilos ejecutan queries simultáneamente.

### 3.4 Warm-up Strategy (Prefaulting)

```rust
impl VantaFile {
    /// Protege capas superiores del HNSW con madvise(MADV_WILLNEED)
    pub fn warmup_top_layers(&mut self, layer_count: usize) {
        // Leer secuencialmente niveles superiores
        // El OS cacheará automáticamente estas páginas
    }
}
```

### 3.5 Write-Ahead Log (WAL)

- Archivo append-only para durabilidad
- Registra inserciones antes de actualizar el grafo
- Replay en caso de fallo de energía

### Validación

- Cold-start < 100ms para 50k vectores
- Recall@10 se mantiene intacto (≥ 0.87)
- Test de corrupción: recovery sin segfaults

- **Test de corrupción específico:**
  1. Insertar 10k vectores
  2. Forzar cierre inesperado (signal kill)
  3. Reiniciar motor
  4. Validar: datos intactos + índice HNSW válido + sin segfaults
  5. Verificar: tombstones reconocidos + estado consistente

### 3.6 Gestión de Fragmentación

#### El Problema

Al usar MMap + Append-Only, el archivo de datos puede crecer con huecos después de muchas operaciones de borrado o actualización. El OS maneja páginas individualmente, no compactará el archivo automáticamente.

#### Estrategia Implementada

1. **Tombstones (ya implementado):**
   - Los nodos no se eliminan físicamente, se marcan con flag `TOMBSTONE`
   - El search filtra automáticamente nodos muertos
   - No hay "huecos" en el archivo mientras no se compacte

2. **Compacción Periódica (Background Thread):**
   - Trigger: Cuando >20% de nodos son tombstones
   - Proceso: Reescribe el archivo sin los nodos muertos
   - Timing: Solo cuando el sistema está idle (sleep worker)
   - Similar a RocksDB compaction pero offline

3. **Copy-on-Write para Actualizaciones:**
   - Si un nodo se modifica, escribir en nueva posición (append)
   - No sobreescribir en el mismo lugar (evita corruption en caso de crash)
   - El vieja posición queda como tombstone

#### Métricas de Monitoreo

- `tombstone_ratio`: Porcentaje de nodos muertos
- `fragmentation_threshold`: 20% → activa compacción
- `compaction_cost`: Tiempo de reescritura vs. espacio recuperado

#### Validación

- Test de 10k inserciones + 5k borrados + 5k nuevas → archivo no crece más de 10%
- Test de crash durante compactación → recovery sin pérdida de datos

---

## Phase 4: Python SDK + Parser IQL (Prioridad: Media)

### Objetivos

- SDK Python funcional con queries reales
- Parser completo para sintaxis IQL

### Acciones

1. **Conectar PyO3 con queries reales:**
   - Reemplazar strings hardcoded en `src/python.rs`
   - Propagar queries a través del parser y executor

2. **Mapear errores Rust → Python:**
   - `VantaError` → `PyException` (evita panics en Python)

3. **Soporte para cierre explícito:**
   - Liberar handles de mmap al cerrar el motor

4. **Extender parser para:**
   - `INSERT` commands completos
   - `MATCH` queries con filtros

### Validación

- `python test_sdk.py` ejecuta inserción + búsqueda real
- Wheel (.whl) generado y funcional

---

## Niveles de Persistencia (Decisión de Arquitectura)

| Nivel | Estrategia | Recomendación |
|-------|------------|---------------|
| **1** | Snapshot manual | ❌ No - muy lento para DB en crecimiento |
| **2** | MMap directo + msync | ✅ Sí - estándar alto rendimiento |
| **3** | LSM-Tree | ⚠️ Solo para metadatos, no para HNSW |

### Estrategia Híbrida Elegida

1. **HNSW Index**: MMap total con offsets
2. **Vector Store**: MMap con alineación 64-byte
3. **WAL**: Append-only para durabilidad

---

## Métricas de Éxito

| Métrica | Target |
|---------|--------|
| Binary Footprint | ~40% reducción |
| Cold-start (50k vectors) | < 100ms |
| Recall@10 (50k) | ≥ 0.87 |
| Throughput de ingesta | Constante (sin picos de rehash) |
| Latencia búsqueda | < 20ms para 50k |
| Alineación SIMD | 64-byte verificada |

---

## Orden de Ejecución Sugerido

1. Phase 1 (Limpieza tests + rust-lld)
2. Phase 2 (Verificar limpieza - probablemente skip)
3. Phase 3 (Zero-Copy + MMap - el diferenciador)
4. Phase 4 (Python SDK + Parser)

---

## Notas Técnicas Importantes

- **No usar `Vec<f32>`** en estructuras disk - son punteros al heap
- **Separar Vector Store de Metadata Store** para mantener eficiencia SIMD
- **MMap solo para datos densos** - RocksDB para datos sparse
- **Prefaulting de capas superiores** del HNSW en startup
