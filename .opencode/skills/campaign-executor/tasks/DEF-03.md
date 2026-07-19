# DEF-03: scan_prefix — streaming en vez de materialización

## Metadata
- **Plan file:** `docs/plans/2026-07-19-deferred-fixes.md`
- **Fuente:** Investigación vanta-tuner
- **Esfuerzo:** 🟡 1d (trait change + 3 backends + 3 call sites)
- **Prioridad:** 🟡
- **Tipo:** Rust
- **Turns estimados:** 15-20
- **Creado:** 2026-07-19T14:00
- **last-synced:** 2026-07-19T14:00
- **Estado:** ⬜ PENDING

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `src/sdk/serialization/impl_export.rs:24` (indexed_ids_by_namespace), `impl_export.rs:47` (indexed_ids_by_filter), `src/sdk/search.rs:191` (BM25 scoring text query) |
| Callees | `src/backend.rs:169-173` (StorageBackend trait), `src/backends/fjall_backend.rs:193-210`, `src/backends/rocksdb_backend.rs:278-297`, `src/backends/in_memory.rs:115-133`, `src/storage/engine/partition.rs:34-40` (scan_partition_prefix) |
| Implicaciones | Los 3 call sites hacen 1 solo for loop y dropean el Vec. Agregar `scan_prefix_iter()` al trait con `Box<dyn Iterator>`. `scan_prefix()` existente se convierte en default que hace collect. Sin breakage de API pública (método nuevo, old sigue funcionando). |

## Contrato
```
cargo check --workspace && cargo nextest run --profile audit --workspace --build-jobs 2 pasa
```

## Herramientas necesarias
- cargo-mcp (check, clippy, fmt, test)
- rust-analyzer-mcp (goto def, diagnostics)
- codegraph_explore (blast radius)

## Investigation Notes
- 3 backends implementan scan_prefix: Fjall, RocksDB, InMemory
- Los 3 itenan internamente con un iterador nativo (range, iterator_cf, btree.range) y coleccionan a Vec
- Fix: `scan_prefix_iter()` → `Box<dyn Iterator<Item=Result<(Vec<u8>, Vec<u8>)>> + 'a>` en el trait
- `scan_prefix()` se convierte en default que hace `.collect()`
- Cada backend implementa scan_prefix_iter devolviendo su iterador nativo con prefix-suffix break
- 3 call sites se actualizan a usar scan_prefix_iter

## Steps

### Step 1: Agregar scan_prefix_iter al StorageBackend trait
- **Archivos:** `src/backend.rs:169-173`
- **Acción:** Agregar `fn scan_prefix_iter<'a>(&'a self, partition: BackendPartition, prefix: &'a [u8]) -> Result<Box<dyn Iterator<Item=Result<(Vec<u8>, Vec<u8>)>> + 'a>>;` al trait. Convertir `scan_prefix()` existente a default que hace collect.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 2: Implementar en FjallBackend
- **Archivos:** `src/backends/fjall_backend.rs:193-210`
- **Acción:** Agregar `scan_prefix_iter()` que envuelve `ks.range(prefix..)` con prefix-suffix break, retorna Box<dyn Iterator>
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 3: Implementar en RocksDBBackend
- **Archivos:** `src/backends/rocksdb_backend.rs:278-297`
- **Acción:** Agregar `scan_prefix_iter()` que envuelve `self.db.iterator_cf(...)` con prefix-suffix break
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 4: Implementar en InMemoryBackend
- **Archivos:** `src/backends/in_memory.rs:115-133`
- **Acción:** Agregar `scan_prefix_iter()` que envuelve `btree.range(prefix.to_vec()..)` con prefix-suffix break
- **Verify:** `cargo check --workspace`
- **Estado:** ⬜ PENDING

### Step 5: Agregar scan_partition_prefix_iter en StorageEngine
- **Archivos:** `src/storage/engine/partition.rs:34-40`
- **Acción:** Agregar `scan_partition_prefix_iter()` que delega al backend. Mantener `scan_partition_prefix()` existente.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 6: Actualizar callers a usar streaming
- **Archivos:** `src/sdk/serialization/impl_export.rs:24,47`, `src/sdk/search.rs:191`
- **Acción:** Cambiar de `scan_partition_prefix()` a `scan_partition_prefix_iter()`. El for loop existente funciona igual porque `Box<dyn Iterator>` itera igual que Vec.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 7: Full verify
- **Acción:** `cargo check --workspace && cargo clippy -p vantadb -- -D warnings && cargo nextest run --profile audit --workspace --build-jobs 2`
- **Verify:** 0 errors, 0 warnings, 0 test failures
- **Estado:** ⬜ PENDING

### Step 8: Commit
- **Acción:** `git add -A && git commit -m "perf: add streaming scan_prefix_iter to avoid full materialization"`
- **Verify:** Commit creado
- **Estado:** ⬜ PENDING

## Dependencias
- Ninguna
