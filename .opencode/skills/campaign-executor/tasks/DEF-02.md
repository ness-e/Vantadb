# DEF-02: text_stats_cache — write-only memory leak + bounds

## Metadata
- **Plan file:** `docs/plans/2026-07-19-deferred-fixes.md`
- **Fuente:** Investigación vanta-tuner
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🔴
- **Tipo:** Rust
- **Turns estimados:** 10-15
- **Creado:** 2026-07-19T14:00
- **last-synced:** 2026-07-19T14:00
- **Estado:** ⬜ PENDING

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `src/sdk/api.rs` (apply_text_stats_deltas), `src/sdk/serialization/impl_index.rs` (replace_record), `src/sdk/serialization/impl_rebuild.rs` (rebuild_text_index_with_report) |
| Callees | `src/storage/engine/mod.rs` (cache definitions), `src/sdk/serialization/impl_text_index.rs` (load_text_term_stats, load_text_namespace_stats), `src/storage/engine/stats.rs` (initialize_cardinality_stats, get_estimated_selectivity), `src/config.rs` (max_entries settings) |
| Implicaciones | text_stats_cache y text_ns_cache son WRITE-ONLY: se pueblan en cada Put/Delete pero NUNCA se leen. Fix: conectar al read path (cache-aside) + watermark eviction. cardinality_stats necesita global cap. |

## Contrato
```
cargo check -p vantadb && cargo nextest run --profile audit --workspace --build-jobs 2 pasa
```
Adicional: verificar que load_text_term_stats() ahora retorna datos del cache cuando hay hit.

## Herramientas necesarias
- cargo-mcp (check, clippy, fmt, test)
- rust-analyzer-mcp (goto def, references)
- codegraph_explore (blast radius)

## Investigation Notes
- `text_stats_cache`: `RwLock<HashMap<(String, String), TextTermStats>>` — ~1.5GB en 10M tokens
- `text_ns_cache`: `RwLock<HashMap<String, TextNamespaceStats>>` — ~1GB en 10M
- `cardinality_stats`: `RwLock<HashMap<String, HashMap<String, usize>>>` — ~80MB en 10K fields
- NUNCA se leen: `load_text_term_stats()` siempre hace backend GET
- Fix strategy: cache-aside (check cache → miss → load from backend → populate cache) + watermark eviction (100K / 1K / 10K global pairs)
- Los read paths ya manejan `Option`/`None` — zero correctness risk

## Steps

### Step 1: Conectar text_stats_cache al read path
- **Archivos:** `src/sdk/serialization/impl_text_index.rs:162-173`
- **Acción:** En `load_text_term_stats()`, check cache first: `engine.text_stats_cache.read().get(&(ns, token))`. On miss, load from backend, write to cache.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 2: Conectar text_ns_cache al read path
- **Archivos:** `src/sdk/serialization/impl_text_index.rs:176-187`
- **Acción:** En `load_text_namespace_stats()`, check cache first. On miss, load from backend, write to cache.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 3: Agregar watermark eviction a text_stats_cache
- **Archivos:** `src/storage/engine/mod.rs:197-198`, `src/sdk/api.rs:531`, `src/sdk/serialization/impl_index.rs:176-177`
- **Acción:** En cada write path, check `cache.len() > MAX_TEXT_STATS_CACHE` (100K). Si excede, drain mitad. Mover lógica a helper fn.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 4: Agregar watermark eviction a text_ns_cache
- **Archivos:** `src/storage/engine/mod.rs:200-201`, `src/sdk/api.rs:538-539`, `src/sdk/serialization/impl_index.rs:183-184`
- **Acción:** En cada write path, check `cache.len() > MAX_NS_CACHE` (1K). Si excede, drain mitad.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 5: Agregar global cap a cardinality_stats
- **Archivos:** `src/storage/engine/ops.rs:141-181`, `src/storage/engine/stats.rs:176-196`
- **Acción:** Verificar total pairs antes de insert. Si > MAX_CARDINALITY_PAIRS (10K), dropear field con menos valores.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 6: Agregar constantes de configuración
- **Archivos:** `src/config.rs`
- **Acción:** Agregar `text_stats_cache_max_entries: 100_000`, `text_ns_cache_max_entries: 1_000`, `cardinality_stats_max_entries: 10_000` con defaults
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 7: Full verify
- **Acción:** `cargo check -p vantadb && cargo clippy -p vantadb -- -D warnings && cargo nextest run --profile audit --workspace --build-jobs 2`
- **Verify:** 0 errors, 0 warnings, 0 test failures
- **Estado:** ⬜ PENDING

### Step 8: Commit
- **Acción:** `git add -A && git commit -m "fix: wire text_stats_cache to read path + add watermark eviction bounds"`
- **Verify:** Commit creado
- **Estado:** ⬜ PENDING

## Dependencias
- Ninguna
