# OLD-20: Contextual Priming (cache warming predictivo)

## Metadata
- **Plan file:** Ninguno (desde Backlog.md)
- **Fuente:** `docs/Backlog.md:181` (Phase 9 — Old Docs Rescue)
- **Esfuerzo:** 🟢 2-3d
- **Prioridad:** 🟢
- **Tipo:** Rust
- **Turns estimados:** 15-25
- **Creado:** 2026-07-28
- **Estado:** ✅ COMPLETED (2026-07-26, verificado batch 6: `src/cache_warmer.rs` + métricas + co-access en hot path; Backlog confirma "auto-decay cada 1000 eventos, métricas exportables")

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `src/storage/engine/ops.rs` (get, get_many), `src/storage/engine/init.rs`, `src/storage/engine/mod.rs` |
| Callees | `src/cache_warmer.rs` (CacheWarmer), `src/metrics/`, `src/storage/engine/ops.rs` (prefetch_related) |
| Implicaciones | No cambia API pública ni SDK. No rompe tests existentes. |

## Current State (real, verified vía codegraph 2026-07-28)

La implementación está ~90% completa. `CacheWarmer` ya existe y está conectado:

### ✅ Ya implementado

- ✅ `CacheWarmer` struct en `src/cache_warmer.rs` — co-access tracking + prefetch
- ✅ `record_co_access()` llamado desde `get_many()` (ops.rs:1227)
- ✅ `prefetch_related()` → `suggest_warm_ids()` llamado desde `get()` (ops.rs:1051)
- ✅ `warm_hnsw_top_layer()` llamado al init (init.rs:129)
- ✅ `record_prefetch_hit()` llamado (ops.rs:1090)
- ✅ 230 líneas de tests de integración en `tests/storage/cache_warming.rs`
- ✅ 7 tests unitarios en `src/cache_warmer.rs`
- ✅ **Gap 1 (decay scheduler) ya resuelto** — `record_co_access()` (line 73) llama `self.decay()` cada 1000 eventos (line 77-81). Pero `#[allow(dead_code)]` en `decay()` (line 131) es un atributo stale que ya no es necesario.

### 2 gaps reales a cerrar:

**Gap 2: Métricas no exportadas** — `CacheWarmerMetrics` struct (line 36) tiene `#[allow(dead_code)]` stale. No conectado a Prometheus counters. Acción:
  - Quitar `#[allow(dead_code)]` de `CacheWarmerMetrics` (line 36) y `clear()` (line 183)
  - Quitar `#[allow(dead_code)]` de `decay()` (line 131) — ya llamado, atributo stale
  - Agregar counters Prometheus en `src/metrics/` para tracked_nodes, total_pairs, total_events, prefetch_hits
  - Opcional: incluir cache warmer stats en `get_memory_stats()`

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

### Step 1: [✅ YA HECHO] Decay scheduler
- **Estado:** ✅ COMPLETED (ya implementado en `record_co_access()`, line 77-81)
- **Pendiente:** Solo remover `#[allow(dead_code)]` stale de `decay()` (line 131)

### Step 2: Remover dead_code stale + conectar métricas Prometheus
- **Archivos:** `src/cache_warmer.rs`, `src/metrics/`, `src/storage/engine/mod.rs`
- **Acción:** 
  - Quitar `#[allow(dead_code)]` de `CacheWarmerMetrics` (line 36), `decay()` (line 131), `clear()` (line 183)
  - Revisar `src/metrics/mod.rs` para ver patrón Prometheus existente
  - Agregar counters: `cache_warmer_tracked_nodes`, `cache_warmer_total_pairs`, `cache_warmer_total_events`, `cache_warmer_prefetch_hits`
  - Llamar actualización de métricas desde `metrics()` del CacheWarmer
- **Verify:** `cargo check -p vantadb` sin dead_code warnings, `cargo clippy -p vantadb`
- **Estado:** ⬜ PENDING

### Step 3: Conectar record_co_access en search paths
- **Archivos:** `src/sdk/search/mod.rs`, `src/storage/engine/mod.rs`
- **Acción:**  
  - En `lexical_search()`, después de obtener hits (line 347): llamar `engine.cache_warmer.record_co_access()` con los node_ids de resultados
  - Misma lógica para `vector_memory_search()` (necesita `cache_warmer` accesible)
  - `hybrid_search()` no necesita duplicado porque llama a lexical+vector internamente
  - NOTA: `record_co_access()` necesita `ids.len() >= 2` para hacer algo (early return si < 2)
- **Verify:** `cargo check -p vantadb`, `cargo nextest run --profile audit`
- **Estado:** ⬜ PENDING

### Step 4: Integration tests para los gaps restantes
- **Archivos:** `tests/storage/cache_warming.rs`, `src/cache_warmer.rs`
- **Acción:** Agregar tests:
  - Test que metrics() se llaman sin dead_code
  - Test que search result IDs se registran en co-access
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
