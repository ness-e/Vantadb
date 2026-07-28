# OLD-20: Contextual Priming (cache warming predictivo)

## Metadata
- **Plan file:** Ninguno (desde Backlog.md)
- **Fuente:** `docs/Backlog.md:181` (Phase 9 — Old Docs Rescue)
- **Esfuerzo:** 🟢 2-3d
- **Prioridad:** 🟢
- **Tipo:** Rust
- **Turns estimados:** 15-25
- **Creado:** 2026-07-28
- **Estado:** ⬜ PENDING

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `src/storage/engine/ops.rs` (get, get_many), `src/storage/engine/init.rs`, `src/storage/engine/mod.rs` |
| Callees | `src/cache_warmer.rs` (CacheWarmer), `src/metrics/`, `src/storage/engine/ops.rs` (prefetch_related) |
| Implicaciones | No cambia API pública ni SDK. No rompe tests existentes. |

## Current State (real, verified)

La implementación está ~80% completa. `CacheWarmer` ya existe y está conectado:

- ✅ `CacheWarmer` struct en `src/cache_warmer.rs` — co-access tracking + prefetch
- ✅ `record_co_access()` llamado desde `get_many()` (ops.rs:1227)
- ✅ `prefetch_related()` → `suggest_warm_ids()` llamado desde `get()` (ops.rs:1051)
- ✅ `warm_hnsw_top_layer()` llamado al init (init.rs:129)
- ✅ `record_prefetch_hit()` llamado (ops.rs:1090)
- ✅ 230 líneas de tests de integración en `tests/storage/cache_warming.rs`
- ✅ 7 tests unitarios en `src/cache_warmer.rs`

### 3 gaps reales a cerrar:

**Gap 1: `decay()` nunca llamado** — `decay()` existe (cache_warmer.rs:127) pero es `#[allow(dead_code)]`. Co-access counts crecen sin límite, memory leak a largo plazo. Necesita un scheduler (ej: background thread que llame `decay()` cada N eventos o cada M minutos).

**Gap 2: Métricas no exportadas** — `metrics()` devuelve `CacheWarmerMetrics` pero tiene `#[allow(dead_code)]`. No está conectado a:
  - Prometheus counters en `crate::metrics`
  - `get_memory_stats()` del engine
  No necesita nueva struct de métricas — solo conectar lo que ya existe.

**Gap 3: `record_co_access()` no llamado desde search paths** — Search paths (lexical, vector, hybrid) devuelven hits de búsqueda pero NO llaman `record_co_access()`. Solo `get_many()` lo llama. Esto omite co-access tracking para queries que vienen por búsqueda en vez de fetch directo.

## Contrato
```
cargo nextest run --profile audit -p vantadb --test cache_warming pasa
cargo nextest run --profile audit --workspace --build-jobs 2 pasa
cargo check -p vantadb sin warnings
cargo clippy -p vantadb sin warnings nuevos
```

## Herramientas necesarias
- cargo-mcp (check, clippy, fmt, nextest)
- codegraph_explore (blast radius)

## Steps

### Step 1: Conectar decay scheduler
- **Archivos:** `src/cache_warmer.rs`, `src/storage/engine/ops.rs`
- **Acción:** Implementar un mecanismo que llame `decay()` periódicamente:
  - Opción A: después de cada N eventos de co-access (ej: cada 1000 `record_co_access`)
  - Opción B: background thread con timer (ej: cada 5 minutos)
  - Elegir la más simple (ponytail: Opción A es una línea en `record_co_access`)
- **Verify:** `cargo check -p vantadb`, test de decay existente pasa
- **Estado:** ⬜ PENDING

### Step 2: Exportar métricas a Prometheus
- **Archivos:** `src/cache_warmer.rs`, `src/metrics/`, `src/storage/engine/mod.rs`
- **Acción:** 
  - Quitar `#[allow(dead_code)]` de `metrics()` y `CacheWarmerMetrics`
  - Agregar counters Prometheus en `src/metrics/` para tracked_nodes, total_pairs, total_events, prefetch_hits
  - Opcional: incluir cache warmer stats en `get_memory_stats()`
- **Verify:** `cargo check -p vantadb`, `cargo clippy -p vantadb`
- **Estado:** ⬜ PENDING

### Step 3: Conectar record_co_access en search paths
- **Archivos:** `src/sdk/search/mod.rs`, `src/storage/engine/ops.rs`
- **Acción:** Llamar `cache_warmer.record_co_access()` después de que search paths devuelvan resultados con IDs. Esto asegura que las búsquedas también alimenten el co-access table.
- **Verify:** `cargo check -p vantadb`, `cargo nextest run --profile audit`
- **Estado:** ⬜ PENDING

### Step 4: Integration tests para los 3 gaps
- **Archivos:** `tests/storage/cache_warming.rs`, `src/cache_warmer.rs`
- **Acción:** Agregar tests:
  - Test que decay se llama automáticamente después de N eventos
  - Test que prefetch hits incrementan
  - Test que search paths graban co-access patterns
- **Verify:** `cargo nextest run --profile audit -p vantadb --test cache_warming`
- **Estado:** ⬜ PENDING

### Step 5: Verificación final
- **Acción:** `cargo check -p vantadb && cargo clippy -p vantadb && cargo nextest run --profile audit --workspace --build-jobs 2`
- **Verify:** Todo pasa, sin warnings nuevos
- **Estado:** ⬜ PENDING

## Dependencias
- Ninguna (task independiente)

## Notas
- La implementación base ya está functional (~80%). Son 3 gaps pequeños y localizados.
- No cambiar API pública ni contratos existentes.
- El decay scheduler con contador de eventos es la opción más lazy (una línea en `record_co_access`).
