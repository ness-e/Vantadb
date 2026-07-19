# DEF-04: HNSW search_layer — HashSet alloc + hasher lento

## Metadata
- **Plan file:** `docs/plans/2026-07-19-deferred-fixes.md`
- **Fuente:** Investigación vanta-engine
- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🟡
- **Tipo:** Rust
- **Turns estimados:** 5-8
- **Creado:** 2026-07-19T14:00
- **last-synced:** 2026-07-19T14:00
- **Estado:** ⬜ PENDING

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `src/index/search.rs` (search_nearest llama search_layer max_layer+1 veces), `src/index/graph.rs` (insert_hnsw llama search_layer) |
| Callees | `src/index/search.rs:24-28` (HashSet allocation), `Cargo.toml` (add ahash dep) |
| Implicaciones | XxHash64 ~30-50ns/lookup vs ahash ~5-10ns (3-10x). ahash ya es dep transitiva via dashmap. Rank 2: pre-allocar HashSet en search_nearest, pasar &mut a search_layer. Elimina 87.5% de allocs. Sin cambios de API pública. |

## Contrato
```
cargo check -p vantadb && cargo nextest run --profile audit --workspace --build-jobs 2 pasa
```

## Herramientas necesarias
- cargo-mcp (check, clippy, fmt, test, add)
- rust-analyzer-mcp (goto def)
- codegraph_explore (blast radius)

## Investigation Notes
- `search_layer` crea `HashSet::with_capacity_and_hasher(ef * 2, BuildHasherDefault::<XxHash64>::default())`
- Llamado `max_layer + 1` veces por query (típico 7-9 para 1M docs)
- Layer0 avg size: 150-1200 nodos, Upper layers: 10-50
- Fix rank 1: reemplazar XxHash64 → ahash (2 líneas + Cargo.toml)
- Fix rank 2: pre-allocar HashSet en `search_nearest`, pasar `&mut` a `search_layer`, `clear()` entre capas

## Steps

### Step 1: Agregar ahash a Cargo.toml
- **Archivos:** `Cargo.toml`
- **Acción:** `cargo-mcp cargo_add` con `dependencies: ["ahash"]` o editar Cargo.toml
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 2: Reemplazar XxHash64 por ahash en search_layer
- **Archivos:** `src/index/search.rs:3,27`
- **Acción:** Cambiar import: `use twox_hash::XxHash64` → `use ahash::RandomState`. Cambiar `BuildHasherDefault::<XxHash64>::default()` → `ahash::RandomState::new()`
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 3: Pre-allocar HashSet en search_nearest
- **Archivos:** `src/index/search.rs` (search_nearest function)
- **Acción:** Crear HashSet una vez antes del layer loop, pasar `&mut` a cada `search_layer()`, `clear()` entre capas
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 4: Pre-allocar HashSet en insert_hnsw
- **Archivos:** `src/index/graph.rs` (insert_hnsw function)
- **Acción:** Crear HashSet una vez, pasar `&mut` a cada `search_layer()` call, `clear()` entre usos
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 5: Full verify
- **Acción:** `cargo check -p vantadb && cargo clippy -p vantadb -- -D warnings && cargo nextest run --profile audit --workspace --build-jobs 2`
- **Verify:** 0 errors, 0 warnings, 0 test failures
- **Estado:** ⬜ PENDING

### Step 6: Commit
- **Acción:** `git add -A && git commit -m "perf: replace XxHash64 with ahash + pre-alloc HashSet in HNSW search_layer"`
- **Verify:** Commit creado
- **Estado:** ⬜ PENDING

## Dependencias
- Ninguna
