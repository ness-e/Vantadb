# FND-02: Regla de coordinación multi-índice + auditoría de deadlocks y contención

## Metadata
- **Plan file:** `docs/plans/2026-08-16-wave-p20-tsys.md` (W4, vanta-arch)
- **Fuente:** docs/Backlog.md:484 (P20a)
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🔴
- **Tipo:** Rust
- **Turns estimados:** 20
- **Creado:** 2026-08-16T13:00
- **last-synced:** 2026-08-16T13:00
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 5 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `src/sdk/api.rs` (put_one/put_batch → engine.insert), `src/storage/engine/insert.rs`, `delete.rs`, `txn.rs`, `maintenance.rs`, `get.rs`, `ops.rs` |
| Callees | `src/index/graph.rs` (HNSW), `src/edge_index.rs` (DashSet), `src/scalar_index.rs` (DashMap), `src/text_index.rs`, `src/wal_sharded.rs`, `src/backend.rs` |
| Implicaciones | Sin cambio de API pública; fixes internos de reentrancia + contención; regla normativa nueva en `.opencode/rules/concurrency-async.md` |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/storage/engine/mod.rs` (550L), `src/storage/engine/insert.rs` (889L), `src/storage/engine/delete.rs` (315L+), `src/storage/engine/ops.rs` (362L), `src/storage/engine/txn.rs`, `src/storage/engine/maintenance.rs` (1216L), `src/storage/engine/get.rs` (366L), `src/edge_index.rs`, `src/scalar_index.rs`, `src/sdk/api.rs` (put_one), `src/sdk/serialization/impl_index.rs`, `.opencode/rules/concurrency-async.md`, `docs/Investigaciones/FND-19-arc-mutex-inventario.md`
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** maintenance.rs usa `crate::storage::ops::NodeMetadata`, `crate::index::release_mmap_vector`, `crate::node::NodeTier`; get.rs usa `crate::node::NodeFlags`, `FilterBitset`, `crate::lsm::unpack_offset`; insert.rs usa `self.evict_cold_nodes_with_reason`
- **Archivos que referencian a los editados (referencias entrantes):** `evict_cold_nodes_with_reason` es llamado por `insert.rs:306`, `insert.rs:834`, `stats.rs:144,166`, tests; `refresh_index` por `ops.rs:224`, `consolidate_node` (maintenance.rs:311) + tests; `get_many` por search paths (`src/engine.rs` query executor)
- **Veredicto impacto:** MEDIO — los fixes son internos (métodos `pub(crate)` nuevos + 2 call sites en insert.rs + patrón try_write en get_many); `evict_cold_nodes_with_reason`/`refresh_index`/`consolidate_node` públicos NO cambian de firma ni semántica para callers standalone

## Contrato
"`cargo check -p vantadb` pasa; regla nueva en `.opencode/rules/concurrency-async.md`; test de deadlock nuevo compila y pasa; reporte `docs/Investigaciones/FND-02-multi-index-locks.md` con orden de locks mapeado (archivo:línea)"

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. `insert_lock` (parking_lot::FairMutex) NO es reentrante — ninguna llamada dentro de una sección crítica puede re-adquirirlo (timeout 5000ms default → VantaError::Timeout).
  2. `consolidate_node` DEBE re-aplicar la entrada HNSW (MmapFull→Full owned) ANTES de liberar las páginas mmap — el refresh no es un no-op; un skip corrompería memoria.
  3. Orden global de locks en writers: `cardinality_stats → insert_lock → {wal, pending_hnsw_batch, hnsw, vstore, volatile_cache, backend}`.
  4. Read paths NUNCA toman `volatile_cache.write()` bloqueante (patrón ERR-036 try_write).
  5. Sin cambios de firma en API pública (`evict_cold_nodes_with_reason`, `refresh_index`, `consolidate_node`).
- **Comandos de verificación:** `cargo check -p vantadb` · `cargo test -p vantadb --lib storage::engine::tests::ops` · `cargo fmt --check`
- **Deuda pendiente:** fixes de rediseño documentados (scalar_index.remove_node/edge_index.remove_all_for_node full-shard sweep; get_many bajo write lock ya resuelto) en el reporte.

## Recitation (canónico)

- **activeGoal:** FND-02 — Regla de coordinación multi-índice + auditoría de deadlocks y contención
- **lastAction:** DISCOVERY completo (codegraph + lectura de todos los paths): mapa de locks en insert/delete/delete_batch/commit_transaction/flush/get/get_many/put_one; hallazgo 🔴 reentrancia `refresh_index` bajo `insert_lock` (eviction desde apply_insert/batch_insert), 🟡 contención get_many write lock, 🟡 dashmap full-shard sweep en delete paths
- **result:** OK
- **nextAction:** Implementar fix 1 (maintenance.rs `_locked` variants) + fix 2 (get_many try_write) + tests
- **contract:** verificacion: `cargo check -p vantadb` + `cargo test` del test nuevo; evidencia: claim="refresh_index re-adquiere insert_lock bajo lock retenido" → evidencia=`src/storage/engine/maintenance.rs:249-257` (try_lock_for) + `insert.rs:306,834` (eviction dentro de guard), confianza=alta; claim="get_many usa write() bloqueante en path read" → evidencia=`src/storage/engine/get.rs:228`, confianza=alta; artefactos: task file FND-02.md, reporte FND-02-multi-index-locks.md; invariantes: ver arriba; deuda: rediseños documentados en reporte; queda_pendiente: lead revisa + commitea
- **nextTask:** lead decide siguiente tarea del wave

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda — no se introduce deuda nueva; los 2 fixes REDUCEN deuda (reentrancia 5s×N y write lock en read path). Los rediseños documentados (dashmap sweep) son deuda pre-existente, no nueva.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato mecánico ✅ (cargo check -p vantadb, test nuevo pasa) + regla en concurrency-async.md + reporte con archivo:línea |
| **Commit** | Conventional commit (lead lo hace — NO commit en esta tarea), diff atómico, verificación mecánica |
| **Release** | No aplica (sin release en wave; lead ejecuta verify.ps1 al cerrar wave) |

## Herramientas necesarias
- cargo (check, test), codegraph_explore

## Investigation Notes
- Hallazgos detallados en `docs/Investigaciones/FND-02-multi-index-locks.md` (se crea en Step 5)
- FND-19 (inventario Arc<Mutex>) delega a FND-02 los Mutex de índices no-Arc: flat.rs:64, scann.rs:51-59, diskann.rs:48-51 → verificados: son estado interno de cada índice, serializados por insert_lock; no hay ratio lectura/escritura que justifique RwLock sin medir (documentado en reporte)

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — resueltas en DISCOVERY (orden de locks, reentrancia, mmap dependency) |
| Pendientes de ejecución (downhill) | 5 — steps abajo |
| % completado | 10% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — no aplica trust boundary (locks internos, sin input externo, sin dependencias nuevas). Se preserva el invariante de memoria (mmap release tras re-add HNSW).
- [x] **PERFORMANCE** — aplica: fixes de contención (get_many write→try_write, eviction reentrancia). No requiere benchmark: son fixes de corrección de lock, no optimización especulativa; el test de estrés con timeout valida ausencia de deadlock.

## Steps

### Step 1: Fix reentrancia — maintenance.rs (variantes `*_locked` + `apply_index_entry_unlocked`)
- **Archivos:** `src/storage/engine/maintenance.rs`, `src/storage/engine/insert.rs`
- **Acción:** extraer `apply_index_entry_unlocked` de `refresh_index`; `refresh_index` = lock + helper; `consolidate_node_inner(node, lock_held)`; `evict_cold_nodes_inner(ratio, reason, locked)`; públicas `consolidate_node`/`evict_cold_nodes_with_reason` (lock_held=false) + `pub(crate)` `consolidate_node_locked`/`evict_cold_nodes_with_reason_locked` (true); insert.rs:306,834 → `_locked`
- **Verify:** `cargo check -p vantadb`
- **Estado:** ✅ DONE

### Step 2: Fix contención — get_many try_write (patrón ERR-036)
- **Archivos:** `src/storage/engine/get.rs`
- **Acción:** reemplazar `volatile_cache.write()` bloqueante en get_many por `try_write()` con fallback a `read()` (mismo patrón que get())
- **Verify:** `cargo check -p vantadb`
- **Estado:** ✅ DONE

### Step 3: Tests de deadlock + reentrancia (extender tests/ops.rs)
- **Archivos:** `src/storage/engine/tests/ops.rs`
- **Acción:** test `test_evict_cold_nodes_locked_does_not_timeout` (consolidate bajo insert_lock retenido completa <500ms) + test `test_multi_index_write_paths_no_deadlock` (threads mixtos insert/delete_batch/get_many/evict con deadline wall-clock)
- **Verify:** `cargo test -p vantadb --lib storage::engine::tests::ops`
- **Estado:** ✅ DONE

### Step 4: Regla normativa en concurrency-async.md
- **Archivos:** `.opencode/rules/concurrency-async.md`
- **Acción:** regla 8 — orden global de locks multi-índice + prohibición de re-adquirir insert_lock + `*_locked` variants + try_write en read paths
- **Verify:** lectura del archivo (formato regla)
- **Estado:** ✅ DONE

### Step 5: Reporte FND-02
- **Archivos:** `docs/Investigaciones/FND-02-multi-index-locks.md` (nuevo)
- **Acción:** paths, orden de locks (archivo:línea), hallazgos clasificados (deadlock-riesgo/contención/ok), fixes aplicados/pendientes
- **Verify:** `cargo check -p vantadb` + fmt
- **Estado:** ✅ DONE

## Dependencias
- FND-19 (inventario Arc<Mutex>) — ✅ completado, entrada usada
- FND-09 (Regla 8 en AGENTS.md) — ✅ completado, esta tarea la materializa en regla técnica

## Review (GATE — agente distinto, P2-01)

- **Revisor:** vanta-audit o vanta-review (lead delega al cerrar la wave — el sub-agente implementador no puede auto-revisarse)
- **Enfoque:** ¿el fix de reentrancia preserva la dependencia mmap (re-add antes de release)? ¿los `_locked` variants no cambian semántica de los públicos?
- **Cómo se probó:** test nuevo con timeout (falla con deadlock, pasa con fix) + cargo check
- **Checklist anti-hábitos tóxicos:** (verifica lead)
- **Veredicto:** pendiente

## Notas
- No hay commit (lead commitea). No tocar Backlog.md, AUD-024.md, _vanta-cli.ps1, verify-log.jsonl, wave-p20-tsys.md, AGENTS.md, benches/, src/metrics, cli_server.
- CLAIM adversario resuelto: el refresh de consolidate_node NO es no-op (detacha MmapFull→Full antes de release_mmap_vector) → el fix aplica la entrada con `apply_index_entry_unlocked` bajo el lock ya retenido; nunca skippea.
