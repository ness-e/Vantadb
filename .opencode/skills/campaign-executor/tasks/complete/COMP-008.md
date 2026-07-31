# COMP-008: Pluggable Index Engine (VecIndex trait)

## Metadata
- **Plan file:** `docs/strategy/ROADMAP.md` (Sem 15-16, item 52)
- **Fuente:** `docs/Backlog.md` línea 202
- **Esfuerzo:** 🟡 1-2 sem
- **Prioridad:** 🟡 Media-Alta
- **Tipo:** Rust core (refactor + trait abstraction)
- **Turns estimados:** 20-40
- **Creado:** 2026-07-27
- **last-synced:** 2026-07-27
- **Estado:** ⬜ PENDING

## Contexto Actual

### Código existente
| Componente | Archivo | Descripción |
|---|---|---|
| `IndexBackend` (enum) | `src/index/graph.rs:144` | Storage backend (InMemory/MMapFile) — **NO** es trait de index |
| `IndexType` (enum) | `src/index/mod.rs:21` | `Hnsw` \| `Ivf` |
| `CPIndex` (struct) | `src/index/graph.rs:290` | HNSW graph index concreto. 45 callers en todo el código |
| `IvfIndex` (struct) | `src/index/ivf.rs:51` | IVF index concreto |
| `flat_search()` (fn) | `src/index/flat.rs:7` | Free function, brute-force O(n) scan |
| `search_nearest()` | `src/index/search.rs:454` | Método en CPIndex que internamente dispatches: IVF → IvfIndex, flat → flat_search(), HNSW → search_layer() |
| `vector_memory_search()` | `src/sdk/search/mod.rs:292` | Accede directamente a `engine.hnsw.load()` — acoplado a HNSW |

### Problema
No existe un trait `VecIndex` que abstraiga operaciones de index. `search_nearest()` es un método de `CPIndex` que maneja 3 paths con if/else internos. `vector_memory_search()` está acoplado a `engine.hnsw`. Nuevos index types (COMP-027: IVF, DiskANN, SCANN) no tienen un contrato común.

## Blast Radius

| Dirección | Módulos |
|---|---|
| Callers de `search_nearest` | 51 callers en benches + `src/sdk/search/mod.rs` + tests de certificación |
| Callers de `CPIndex` | 45 callers en `src/index/search.rs`, `src/index/serialize.rs`, `src/storage/vfile.rs`, `src/storage/archive.rs` + benches/tests |
| Callers de `flat_search` | 1 caller en `src/index/search.rs:search_nearest()` |
| SDK surface | `src/sdk/search/mod.rs:vector_memory_search()` usa `engine.hnsw.load()` directamente |
| Serialización | `src/index/serialize.rs` serializa CPIndex directamente |
| Tests | `tests/certification/hnsw_recall.rs`, `tests/certification/hnsw_validation.rs`, `tests/certification/competitive_bench.rs`, `tests/certification/stress_protocol.rs`, `tests/storage/mmap_index.rs` |

### Implicaciones
- **API pública no cambia** — `VantaEmbedded::search()` sigue igual. El trait es interno.
- **Serialización no cambia** — solo se abstrae, no se modifica formato de disco.
- **vector_memory_search** deja de llamar a `engine.hnsw.load()` directamente y pasa a usar `engine.vec_index()` o similar.
- **COMP-027** (multiple index types) depende de esto — desbloquea IVF, DiskANN, SCANN.

## Contrato
```
cargo nextest run --profile audit --workspace --build-jobs 2 pasa
  && cargo check --workspace pasa sin warnings
  && cargo clippy --workspace --all-targets -- -D warnings pasa
  && Existe trait VecIndex con métodos search/add/len/memory_estimate
  && CPIndex implementa VecIndex
  && IvfIndex implementa VecIndex
  && flat_search se integra vía VecIndex (FlatIndex wrapper)
  && vector_memory_search usa VecIndex trait object
  && benchmarks existentes siguen compilando y pasando
  && No hay cambios en serialización de disco
```

## Herramientas necesarias
- cargo-mcp (check, clippy, fmt, test, bench)
- rust-analyzer-mcp (diagnostics, go-to-def)
- codegraph_explore (blast radius)

## Investigation Notes
- **Ponytail approach**: No crear abstracciones innecesarias. `VecIndex` debe tener solo los métodos que realmente se necesitan hoy: `search`, `add`, `len`, `memory_estimate`. No generalizar para casos de uso futuros inciertos.
- **FlatIndex wrapper**: `flat_search` es una función libre. Crear `FlatIndex` struct mínimo que implemente `VecIndex` para ser tratado uniformemente.
- **IvfIndex**: Ya tiene `search`, `build`, `serialize_to_bytes`, `deserialize_from_bytes`. Hacer que implemente `VecIndex`.
- **CPIndex**: Ya tiene `search_nearest`, `add`, etc. Hacer que implemente `VecIndex` (ya sea directamente o vía un wrapper).
- **StorageEngine**: Agregar `vec_index: Arc<dyn VecIndex>` que se construye según `HnswConfig.index_type`.
- **Thread safety**: `VecIndex` trait requiere `Send + Sync` para ser usable con `Arc<dyn VecIndex>`.

## Steps

### Step 1: Definir VecIndex trait
- **Archivos:** `src/index/mod.rs` (nuevo archivo o agregar al existente)
- **Acción:** (ponytail: mínimo viable)
  ```rust
  /// Trait for pluggable vector index backends.
  pub(crate) trait VecIndex: Send + Sync {
      /// Search nearest neighbors for a query vector.
      fn search(
          &self,
          query_vec: &[f32],
          query_mask: &FilterBitset,
          top_k: usize,
          vector_store: Option<&VantaFile>,
          distance_metric: DistanceMetric,
      ) -> Vec<(u128, f32)>;

      /// Add a node to the index.
      fn add(&self, id: u128, bitset: FilterBitset, vec_data: VectorRepresentations, storage_offset: u64);

      /// Estimate memory usage in bytes.
      fn estimate_memory_bytes(&self) -> usize;

      /// Number of indexed nodes.
      fn len(&self) -> usize;

      /// Returns true if the index is empty.
      fn is_empty(&self) -> bool { self.len() == 0 }
  }
  ```
- **Verify:** `cargo check -p vantadb`

### Step 2: Implementar VecIndex para CPIndex (wrapper o impl directo)
- **Archivos:** `src/index/graph.rs` o `src/index/search.rs`
- **Acción:**
  - CPIndex ya tiene `search_nearest`, `add`, `estimate_memory_bytes`, `total_nodes`
  - Implementar `VecIndex for CPIndex` mapeando:
    - `search()` → `self.search_nearest(...)`
    - `add()` → `self.add(...)`
    - `estimate_memory_bytes()` → `self.estimate_memory_bytes()`
    - `len()` → `self.nodes.len()`
- **Verify:** `cargo check -p vantadb`

### Step 3: Implementar VecIndex para IvfIndex
- **Archivos:** `src/index/ivf.rs`
- **Acción:**
  - `search()` → `self.search(query_vec, top_k, query_mask)`
  - `add()` → `panic!("IvfIndex is read-only after build; rebuild via IvfIndex::build()")`
  - `estimate_memory_bytes()` → calcular de centroids + inverted_lists
  - `len()` → sum de inverted_lists lengths
- **Verify:** `cargo check -p vantadb`

### Step 4: Crear FlatIndex wrapper (opcional — si se quiere tratar flat como VecIndex)
- **Archivos:** `src/index/flat.rs`
- **Acción:**
  ```rust
  pub(crate) struct FlatIndex {
      nodes: Arc<DashMap<u128, HnswNode>>,
      distance_metric: DistanceMetric,
  }
  ```
  Implementar `VecIndex for FlatIndex` que llama a `flat_search()` internamente.
- **Verify:** `cargo check -p vantadb`

### Step 5: Integrar VecIndex en StorageEngine
- **Archivos:** `src/engine.rs` o `src/storage/mod.rs`
- **Acción:**
  - Agregar `vec_index: Arc<dyn VecIndex>` en `StorageEngine`
  - En inicialización, construir el index apropiado según `HnswConfig.index_type`:
    - `Hnsw` → `CPIndex` como `Arc<dyn VecIndex>`
    - `Ivf` → `CPIndex` (con lazy-build, como hoy) — o wrapper
    - Flat threshold → `CPIndex` internamente decide flat/HNSW como antes
  - Agregar método `pub fn vec_index(&self) -> &dyn VecIndex`
- **Verify:** `cargo check -p vantadb`

### Step 6: Actualizar vector_memory_search
- **Archivos:** `src/sdk/search/mod.rs`
- **Acción:**
  - Cambiar `let hnsw = engine.hnsw.load();` por `let index = engine.vec_index();`
  - Llamar `index.search(query_vector, query_mask, budget, vector_store, distance_metric)`
- **Verify:** `cargo check -p vantadb`

### Step 7: Verificar que benches compilan
- **Archivos:** `benches/high_density.rs`, `benches/hnsw_pure.rs`
- **Acción:** Revisar que `search_nearest()` sigue siendo accesible (CPIndex todavía existe como tipo concreto, los benches pueden usar CPIndex directamente o vía VecIndex)
- **Verify:** `cargo check --benches -p vantadb`

### Step 8: Tests
- **Archivos:** tests relevantes
- **Acción:** Asegurar que todos los tests pasan
- **Verify:** `cargo nextest run --profile audit --workspace --build-jobs 2`

### Step 9: fmt + clippy final
- **Acción:** `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings`
- **Verify:** Pasa sin errores

## Dependencias
- Pre-COMP-027 (desbloquea múltiples index types)

## Notas
- `CPIndex` sigue existiendo como tipo concreto (los benches y tests de certificación lo usan directamente). `VecIndex` es una capa adicional.
- `search_nearest()` en `CPIndex` puede seguir existiendo como método concreto — `VecIndex::search()` delega a él.
- La serialización/disco no cambia — `CPIndex` sigue serializándose igual.
- Ponytail: no abstraer de más. Solo los métodos que se necesitan para que `vector_memory_search` funcione sin conocer el tipo concreto.
