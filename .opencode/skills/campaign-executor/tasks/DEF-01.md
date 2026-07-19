# DEF-01: SendPtr — fuga de lifetime en mmap

## Metadata
- **Plan file:** `docs/plans/2026-07-19-deferred-fixes.md`
- **Fuente:** Investigación vanta-worker + vanta-audit
- **Esfuerzo:** 🟡 1d (12+ archivos, requiere refactor mediano)
- **Prioridad:** 🔴
- **Tipo:** Rust
- **Turns estimados:** 20-30
- **Creado:** 2026-07-19T14:00
- **last-synced:** 2026-07-19T14:00
- **Estado:** ⬜ PENDING

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `src/index/search.rs` (search_nearest, search_layer), `src/index/graph.rs` (insert_hnsw), `src/index/flat.rs` (flat_search), `src/index/serialize.rs` (serialize/deserialize) |
| Callees | `src/node.rs` (SendPtr, VectorRepresentations), `src/storage/engine/mod.rs` (StorageEngine), `src/vector/transform.rs` |
| Implicaciones | SendPtr es `*const f32` dentro de `VectorRepresentations::MmapFull` → `HnswNode` → `DashMap<CPIndex, HnswNode>` → `ArcSwap<DashMap>`. Si mmap se re-mappea (serialize.rs:614), el puntero crudo queda Dangling. Fix requiere cambiar a `Arc<Mmap>` + accessor con read-lock gated. |

## Contrato
```
cargo check -p vantadb && cargo nextest run --profile audit --workspace --build-jobs 2 pasa
```

## Herramientas necesarias
- cargo-mcp (check, clippy, fmt, test)
- rust-analyzer-mcp (goto def, references, diagnostics)
- codegraph_explore (blast radius)

## Investigation Notes
- SendPtr = wrapper de `*const f32` que implementa `Send + Sync`
- Almacenado dentro de `VectorRepresentations::MmapFull(SendPtr)` en `HnswNode`
- 8 sitios de construcción (asignación de SendPtr)
- 8 sitios de dereference (lectura del puntero)
- serialize.rs:614 hace `drop(mmap)` → `rename()` → re-mmap, lo que INVALIDA todos los SendPtr existentes
- Fix: cambiar `MmapFull(SendPtr)` → `MmapFull(Arc<Mmap>)`, agregar accessor `vector_slice()` en HnswNode que adquiere read-lock

## Steps

### Step 1: Mapear todos los sitios de SendPtr
- **Archivos:** `src/node.rs`, `src/index/graph.rs`, `src/index/serialize.rs`, `src/index/search.rs`, `src/index/flat.rs`, `src/vector/transform.rs`
- **Acción:** Usar codegraph_explore y grep para listar cada construcción de SendPtr (new, from_raw_parts, as_ptr, etc.) y cada dereference (calls a .as_ptr(), .deref(), etc.)
- **Verify:** `grep -rn "SendPtr\|MmapFull\|as_ptr" src/` — output documentado en Notas
- **Estado:** ⬜ PENDING

### Step 2: Cambiar VectorRepresentations::MmapFull a Arc<Mmap>
- **Archivos:** `src/node.rs`
- **Acción:** Cambiar `MmapFull(SendPtr)` → `MmapFull(Arc<Mmap>)`. Eliminar SendPtr o mantenerlo como wrapper de Arc<Mmap>
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 3: Actualizar 8 sitios de construcción
- **Archivos:** `src/index/graph.rs`, `src/index/serialize.rs`, `src/index/flat.rs`, `src/vector/transform.rs`
- **Acción:** Cambiar cada `SendPtr::new(mmap.as_ptr())` → `MmapFull(Arc::new(mmap))`
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 4: Actualizar accessor en HnswNode
- **Archivos:** `src/node.rs` (HnswNode)
- **Acción:** Agregar método `vector_slice(&self) -> &[f32]` que extrae el slice del Arc<Mmap> o de la representación inline
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 5: Actualizar 8 sitios de dereference
- **Archivos:** `src/index/search.rs`, `src/index/graph.rs`, `src/index/flat.rs`
- **Acción:** Reemplazar cada `node.send_ptr.as_ptr()` (o similar) con `node.vector_slice()`
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 6: Full verify
- **Archivos:** Todos
- **Acción:** `cargo check -p vantadb && cargo clippy -p vantadb -- -D warnings && cargo nextest run --profile audit --workspace --build-jobs 2`
- **Verify:** 0 errors, 0 warnings, 0 test failures
- **Estado:** ⬜ PENDING

### Step 7: Commit
- **Acción:** `git add -A && git commit -m "fix: SendPtr lifetime — replace *const f32 with Arc<Mmap>"`
- **Verify:** Commit creado con mensaje conventional commit
- **Estado:** ⬜ PENDING

## Dependencias
- Ninguna
