# C2S3b: extraer `CacheLayer` de `StorageEngine` (slice 2 de S3)

## Metadata
- **Plan file:** docs/plans/2026-09-13-cleanCA-fase2.md (Wave 3.5, Task 13b: S3b)
- **Creado:** 2026-09-13
- **Estado:** ✅ COMPLETED (implementación + verify contrato completos 2026-09-13; commit pendiente del lead — adaptador §10: NO commitear)
- **Ruta:** vanta-worker · **Appetite:** 1d · 🟡 · 🟠
- **Depende de:** S3-txn (cdb15b3e) + Wave 3 (suite rápida como red) · **Bloquea:** Wave 5 (M1-full exige D final) · **nextTask:** C2C2
- **SDP (phase=BUILD, keywords SRP/cache/manager):** incremental-implementation (1 slice vertical CacheLayer) | test-driven-development (RED=suites storage verdes antes/después) | context-engineering (Rules→Spec→Source→Error) | systematic-debugging (root-cause si verify falla) | performance-optimization (bench en CI al mergear, NO local — hot path) | planning-and-task-breakdown (alcance acotado 5 campos) | source-driven-development (C-SEALED pub(crate) precedente C2S1/S3) | code-simplification (delegación 1:1, sin abstracciones) | api-and-interface-design (contract-first CacheLayer) | code-review-and-quality (5 ejes pre-cierre) | doubt-driven-development (revisión adversarial split cache) | observability-and-instrumentation (n/a, sin cambio runtime)

## Gate D: GO
- Blast radius: engine interno 7 archivos (mod/cache/get/insert/delete/maintenance/stats/init/txn) + externos SDK 5 archivos (impl_text_index, impl_index, impl_rebuild, api/memory, cost_estimator) + tests engine. >10 archivos pero cambio 100% mecánico (1-token `self.X` → `self.cache.X`), sin símbolos `pub` nuevos (`CacheLayer` + métodos `pub(crate)` = sellados, precedente C2S1/S3-txn, sin ADR).
- Sin cambio semántico: delegación 1:1, mismo orden de locks, misma política de evicción, mismos guards ERR-036/ERR-037.
- Patrón obligatorio (slice txn): `pub(crate)` sellado + delegación 1:1 + batch paths preservan ERR-037 (one-lock-pass por chunk, sin clonar cache entera).
- Bench: NO local (tantivy H2); delta esperado cero; comando CI al mergear: `cargo bench -p vantadb --bench canonical_p99`.
- txn.rs CERRADO salvo 2 toques mecánicos `self.volatile_cache` → `self.cache.volatile_cache` (no se re-abre lógica txn).

## Blast Radius (grep + Read)
- `src/storage/engine/mod.rs:332-379` `StorageEngine`: 24 campos (tras S3-txn). Campos cache: `volatile_cache:332`, `text_stats_cache:370`, `text_ns_cache:373`, `cardinality_stats:376`, `cache_warmer:379`.
- Nuevo `src/storage/engine/cache.rs` (nombre libre — verificado: no existe `cache.rs` en `src/storage/engine/`).
- Usos internos: get.rs (~10: lookup_volatile_cache:212 + get_many:477/497 + prefetch:435/447 + record_co_access:723), insert.rs (~8: apply_insert_stats:237/269 + cache_batch:375 + probes:762/823 + stats serial/rayon:534/570), delete.rs (~4: 63/139/180/266), maintenance.rs (~4: flush:121 + consolidate:293/354 + evict:403), stats.rs (stats:56 + initialize:220), txn.rs commit/apply (335/461), init.rs construcción (103/111/120-122/132).
- Usos externos SDK (solo lectura/escritura directa de los 2 text caches + cardinality): `impl_text_index.rs:169/182/193/204`, `impl_index.rs:186/201/221/226`, `impl_rebuild.rs:79/83`, `api/memory.rs:787/802/822/827`, `cost_estimator.rs:101`.
- Tests como red: `cargo nextest run -p vantadb --lib storage` + suites engine/ops/maintenance; accesos directos `engine.volatile_cache/cardinality_stats/cache_warmer` en tests/engine.rs, tests/ops.rs, tests/maintenance.rs (migración mecánica a `engine.cache.*`).

## Impacto mapeado (Regla 0)
- **Leídos completos:** mod.rs:316-383 (struct), get.rs:197-230 + 383-455 + 456-727, init.rs:103-136, maintenance.rs:116-125 + 268-483, stats.rs:54-95 + 220-254, txn.rs:318-465 (commit/apply toques), insert.rs:228-304 + 373-400 + 740-848, delete.rs:54-142 + 178-271, impl_text_index.rs:162-208, cost_estimator.rs:100-102.
- **Referencias hacia dentro (nuevas):** `CacheLayer` (cache.rs) + `lookup_volatile_cache` movido + `CacheLayer::new` + delegación `StorageEngine::lookup_volatile_cache`; `StorageEngine::cache` delega. Campos con NOMBRES IDÉNTICOS dentro de CacheLayer (`volatile_cache`, `text_stats_cache`, `text_ns_cache`, `cardinality_stats`, `cache_warmer`) → call sites solo añaden `.cache`.
- **Referencias entrantes:** engine/* + sdk/* vía `engine.cache.*` — firmas públicas intactas (`get/get_many/evict/stats` sin cambio), solo cambia el path del campo interno. Tests migran a `engine.cache.*`.
- **Veredicto:** refactor de tipos sin cambio semántico; reversible por revert; 1 slice (tamaño ≈ txn); hot path NO se optimiza.

## Spec (decisiones)
| # | Decision | Opción elegida | Evidencia |
|---|----------|----------------|-----------|
| 1 | Qué posee CacheLayer | Los 5 campos cache puros (volatile + 2 text + cardinality + warmer) | Step 2 C2S3.md alcance; text caches viven en engine aunque sus lectores estén en sdk/ |
| 2 | Nombres de campos | IDÉNTICOS dentro de CacheLayer | diff mecánico 1-token, cero renames semánticos |
| 3 | Probe lectura | `CacheLayer::lookup_volatile_cache` movido 1:1 desde get.rs (ERR-036 try_write) | mismo lock order, misma política |
| 4 | Batch/rayon | Sin `cloned_*` nuevo: probes batch ya amortizan por chunk (ERR-037 preservado tal cual) | insert.rs:785-848 intacto salvo path `.cache.` |
| 5 | Sealed | `pub(crate)` sin trait Sealed extra | precedente C2S1/S3, Rust API Guidelines C-SEALED |
| 6 | Bench | NO local (tantivy H2); delta esperado cero + comando CI | decisión humana bench-en-CI-al-mergear |
| 7 | Stats init | `initialize_cardinality_stats` se queda en stats.rs (necesita backend); CacheLayer::new recibe el mapa ya construido | manager puro estado, sin I/O |

## Contrato
`CacheLayer` extraído con tests propios + `StorageEngine` delega + suites storage verdes + `acyclic` sin ciclos nuevos + M1 post-números (24→~19 campos, D debe bajar) + bench en CI al mergear.

## Steps
### Step 1: CacheLayer + delegación + migración call sites + tests propios
- **Archivos:** nuevo cache.rs; mod.rs (struct 5→1 campo); init.rs; get/insert/delete/maintenance/stats/txn (mecánico `.cache.`); sdk externos (5 files); tests engine (mecánico `engine.cache.*`).
- **Verify:** cargo check + clippy -D warnings + fmt + nextest storage + backend + acyclic + conteo campos.
- **Estado:** ✅ COMPLETED

### Step 2: verify contrato + tabla después + cierre
- **Acción:** nextest storage completo + fmt/clippy + conteo campos después (24→~19) + comando bench CI documentado; Gates D/V/C.
- **Estado:** ✅ COMPLETED

## Cierre (2026-09-13, vanta-worker)
- **Nuevo:** `src/storage/engine/cache.rs` — `CacheLayer` `pub(crate)` sellado (5 campos NOMBRES IDÉNTICOS) + `new(map)` + `lookup_volatile_cache` movido 1:1 (ERR-036) + 4 tests propios verdes.
- **Delegación:** `StorageEngine::lookup_volatile_cache` (get.rs) delega a `self.cache`; resto call sites solo añaden `.cache`. txn.rs: 2 toques mecánicos. Batch/rayon ERR-037 intactos.
- **Verify contrato:**
  - `cargo check -p vantadb` ✅ · `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` ✅ · `cargo fmt --check` ✅
  - `cargo nextest run -p vantadb --lib storage` ✅ 456 passed · `cargo nextest run -p vantadb --lib` ✅ 2078 passed, 2 skipped
  - `cargo test -p vantadb --test structured_api_v2` ✅ 1 passed, 1 ignored
  - `cargo modules dependencies -p vantadb --lib --acyclic`: falla SOLO por ciclo pre-existente `accumulator::GraphAccumulator↔new` (idéntico en baseline con stash; NO introducido) — cache.rs edges leaf-only, sin ciclos nuevos ✅
  - Conteo campos `StorageEngine`: 24→20 (5 fuera + `cache`) ✅ (~19 esperado)
  - Bench: NO local (decisión humana bench-en-CI-al-mergear); comando CI: `cargo bench -p vantadb --bench canonical_p99`
- **HALLAZGOS (fuera del Blast Radius declarado):**
  - H1: `src/sdk/search/vector.rs:51/217` (`cw_engine/engine.cache_warmer`) — migrado mecánico igual que lexical.rs.
  - H2: `tests/api/structured_api_v2.rs:33` leía `storage.volatile_cache` (campo `pub` externo) — incompatible con sellado `pub(crate)`; reescrito a IDs desde `ExecutionResult::Write { node_id }` (API pública, test verde).
- **Gates:** D: GO (mecánico 1-token, sin `pub` nuevos) · V: no-disparado (verify pasó al 1er intento; H2 resuelto por diseño sellado, no por reintento) · C: pasa (colaterales H1/H2 incluidos en el diff, sin scope extra).
- **Commit:** pendiente del lead (adaptador §10). Diff: 19 files M + `src/storage/engine/cache.rs` nuevo. Pre-existentes NO tocar: `m .opencode`, `M desktop/src-tauri/Cargo.lock`.

## Context Save Point
- **Fecha:** 2026-09-13 · **Branch:** (la del lead) · **CI pendiente:** bench `canonical_p99` al mergear
- **Decisiones:** sellado `pub(crate)` estricto (sin `pub` nuevos) → H2 por API pública; `now_ms` duplicado privado en cache.rs (cero dependencia entre siblings)
- **Problemas conocidos:** ninguno — suites verdes
- **Próxima tarea:** C2C2 (bloquea Wave 5)
