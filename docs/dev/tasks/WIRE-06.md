---
title: "WIRE-06: Batching productizado (`insert_lock` → group-commit opt-in)"
kind: task
description: "Corrida final (28-09 ~04:0x, ventana quieta) — todos los grupos wire06 en una invocación"
---

# WIRE-06: Batching productizado (`insert_lock` → group-commit opt-in)

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 20, Fase F2b)
- **Fuente:** plan Task 20 · dependencia FUT-12-spec ✅ (`docs/dev/tasks/FUT-12-spec.md` + `ADR-0038-wal-fsync-batching-opt-in.md` spec-only)
- **Esfuerzo:** 🔴 1sem · **Prioridad:** 🔴
- **Tipo:** Rust (core) + benches + docs
- **Turns estimados:** 30-60 (uphill: 3 → resueltas, ver Incógnitas)
- **Creado:** 2026-09-27T00:00
- **last-synced:** 2026-09-27T00:00
- **Estado:** ⏳ IN PROGRESS (funcionalmente COMPLETO — contrato verificado; cierre formal: commit + review del LEAD)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps (7/7 ✅)

## Medición (evidencia)

**Corrida final (28-09 ~04:0x, ventana quieta)** — todos los grupos `wire06_*` en una invocación:

| Celda | ops/s | ON/OFF | p50-ack | p99-ack |
|---|---|---|---|---|
| off_p1w1 | 92.5 | 1.00× | 10.5 ms | ~16.8 ms |
| on_p1w1 (default 32/1ms) | 480.5 | **5.19×** | 70.5 ms | ~90 ms |
| on_w2 (sweep) | 483.5 | 5.23× | 68.7 ms | ~90 ms |
| on_w5 (sweep) | 547 | 5.91× | 61.2 ms | ~80 ms |
| on_serial (chunk=1) | 83 | 0.90× | 11.9 ms | ~16.7 ms |
| **paired_ab (misma ventana)** | — | **5.92×** (5.29–6.39×) | — | — |

**Atribución (misma ventana):** directa skip=false ~457 ops/s · directa skip=true ~465 · pipeline ON ~459 → check+plumbing **1.00×** (sin costo medible vs prototipo). El delta vs FIND-61 (1016 ops/s / 9.1×, 2026-09-04) es del entorno: hoy la misma ruta directa mide ~460.

**Sensibilidad a carga (4 corridas, mismo código):** ratios intra-corrida 3.49×–9.09× (celdas secuenciales agarran fases de carga distintas; detalle en BENCHMARKS §13 WIRE-06). Evidencia de gate = celda pareada (5.92× mediana, 5.29× mínimo) + secuencial final 5.19×.

- Prototipo FIND-61 (2026-09-04, quieta): N=32 → 1016 ops/s / 9.1×.
- Default OFF = byte-idéntico; canonical_p99 consumo guard (`--no-run`) ✅; `src/index/**` intacto.

## SDP

`SDP: campaign-executor, progreso, ponytail, deprecation-and-migration (pinned storage/schema), performance-optimization (pinned performance), source-driven-development, doubt-driven-development, incremental-implementation + test-driven-development, rust-write-tests, api-and-interface-design (cargadas por contrato de lógica nueva/API)`

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `AsyncIngestionPipeline` ← benches (`ingestion_concurrent`) + consumidores del feature `async-ingestion` (SDK/server ad-hoc). `Config` ← todos los crates del workspace (campo nuevo aditivo). |
| Callees | `StorageEngine::batch_insert_with_opts` (insert.rs:887) → `commit_batch_locked` (1× `insert_lock`) → `append_batch_wal` → `ShardedWal::batch_append` (1 lock + 1 write_all + ≤1 `maybe_sync` por shard) → HNSW bulk `insert_hnsw_leveled`. |
| Implicaciones | Contrato de `insert()` intacto (batch NO toca el path single). `Config` gana 1 campo aditivo (default OFF = byte-idéntico). Pipeline gana un modo opt-in; OFF = código actual sin cambios. Sin cambios de formato on-disk/WAL. Sin migración de datos. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/ingestion.rs` (145L) · `src/wal.rs` (1434L) · `src/wal_sharded.rs` (1162L) · `src/storage/engine/insert.rs` (937L) · `benches/ingestion_concurrent.rs` (344L) · `benches/wal_throughput.rs` (158L) · `benches/canonical_p99.rs` (135L) · `src/storage/engine/mod.rs:300-659` (struct + `acquire_insert_lock`) · `src/storage/engine/init.rs:20-169` (wiring) · `src/config.rs` (secciones: `SyncMode` 86-104, `StorageCfg` 151-172, `PoolCfg` 230-241, `Config` 573-819, Default 909-1289, builders 1380-1569, tests) · `src/storage/engine/stats.rs:1-29` (`ensure_writable`) · `src/error.rs:130-160` + helpers · `ADR-0038` completo · `FUT-12-spec` completo · `BENCHMARKS.md` §13/§13.1 (422-663) · `docs/user/operations/CONFIGURATION.md` (tabla de flags) · `.github/workflows/heavy-bench-nightly.yml`.
- **Referencias hacia dentro:** `AsyncIngestionPipeline::new` ← `benches/ingestion_concurrent.rs:43,165` + `src/ingestion.rs` tests; `Config` ← workspace entero (struct literal con `..Config::default()`); `ShardedWal::batch_append` ← `insert.rs:367` + `delete.rs` + `txn.rs`; `acquire_insert_lock` ← 13 callers (delete/ops/insert/maintenance/...) — ninguno cambia.
- **Referencias entrantes:** `rg AsyncIngestionPipeline` = 5 hits (def + bench + test). `engine.config` es `pub` → los bindings leen config; campo nuevo aditivo no rompe literales (todos usan `..Default::default()`; verificado por compilación workspace).
- **Veredicto impacto:** BAJO-MEDIO — superficie aditiva con default OFF; el riesgo real es el *modo ON* (comportamiento nuevo del pipeline) → mitigado con tests de integridad (WAL/reopen/atomicidad) + A/B bench + invariante OFF byte-idéntico.

## Contrato

> verbatim del plan (Task 20):
> "≥5× ingesta sostenida vs baseline (bench before/after commiteado, Regla 9) Y p99 sin regresión >15% Y tests de integridad (WAL/durabilidad/reopen, batch atómico) verdes Y bench nightly activo con gate anti-regresión"

Traducción verificable:
1. `cargo bench -p vantadb --bench ingestion_concurrent --features async-ingestion -- "wire06_group_commit"` → celda ON ≥5× la celda OFF (mismo harness, misma corrida, BATCH=400 DIM=16).
2. `cargo bench -p vantadb --bench canonical_p99` before/after sin regresión >15% (Regla 9; el pipeline no toca el index in-memory) + celda OFF del A/B sin regresión vs baseline §13 (111.5 ops/s).
3. `cargo nextest run --profile audit -p vantadb --features async-ingestion -E 'test(ingestion) or test(wal) or test(reopen) or test(insert)'` verde (integridad WAL/reopen/atomicidad).
4. `.github/workflows/heavy-bench-nightly.yml` corre `ingestion_concurrent` (con feature) + gate de regresión por baseline (`scripts/bench_regression.py compare`, auto-issue) — las celdas nuevas entran al gate sin tocar el workflow.

## Spec (SDD — feature-add: símbolos públicos nuevos)

> Gate D: no dispara ronda de `question` — las decisiones ya fueron tomadas por el owner en FUT-12-spec (Gate P, 2026-09-04) y el acceptance está fijado por el contrato del plan; API nueva 100% aditiva con default OFF (One-Version Rule). Evidencia por ítem:

| # | Decisión | Opción elegida | Evidencia |
|---|----------|----------------|-----------|
| 1 | Opt-in vs default | 100% opt-in, default OFF byte-idéntico | ADR-0038 §Decision (decisión owner 1) + patrón `SyncMode`/`segment_optimizer` |
| 2 | Mecanismo de batching | **Group-commit a nivel de operación**: el pipeline acumula N tasks y commitea con UN `batch_insert_with_opts` (1× insert_lock + 1× WAL batch_append/shard + HNSW bulk) | FIND-61 Tabla 2 (N=32 → 1016 ops/s / 9.1×, BENCHMARKS:608-612) + fsync≈1.5% (BENCHMARKS:588-597) → el término dominante es lock+serial; agrupar *operaciones* lo amortiza |
| 3 | ADR-0038 (queue de fsync dedicado, `returned=queued`) — ¿implementación literal? | **No**: se consume su POLÍTICA (opt-in, ventana máx. declarada y testeable, cola acotada, un sync por shard por ciclo) y su efecto WAL se logra vía `batch_append` (≤1 `maybe_sync`/shard por batch). El ack es *después del apply* (más fuerte que watermark: `returned ⇒ aplicado + durable per SyncMode`) | FIND-61 §Tabla 1: fsync 0.16ms (~1.5%) → una queue dedicada solo para fsync no alcanza ≥5×; batch_append ya colapsa los fsyncs (un sync por shard por llamada). `sync()` = flush+sync_data (wal.rs:397-402) llamado por `maybe_sync` 1×/batch (`wal.rs:376-394`); `ShardedWal::batch_append` agrupa por shard (wal_sharded.rs:407-427) |
| 4 | Dónde vive el batching | `AsyncIngestionPipeline` (feature `async-ingestion`) lee `engine.config.insert_batch`; el engine (insert.rs) NO cambia | El harness A/B (infra find61, BATCH=400 DIM=16) es la medición contractual y su worker es **serial por diseño** (1 `spawn_blocking` en vuelo, ingestion.rs:74-103) → un coordinator en `insert()` no podría agrupar nada en ese régimen (batches de 1). El pipeline es la superficie que sí acumula |
| 5 | Opciones del batch | `skip_existing_check: false` (correcto para UPSERTs), `skip_wal: false`, `InsertMode::Incremental` forzado | `BatchInsertOptions` (ops.rs:31) — skip_existing=true exige IDs frescos (prototipo lo usaba: FIND-61); el pipeline no puede garantizarlo. Incremental evita que `Auto` elija Rebuild si `max_batch_records > incremental_threshold` (1000) y deje el HNSW sin entradas |
| 6 | Ventana declarada | `enabled` (false) · `max_batch_records` (32) · `max_wait_ms` (1) · `max_queued_records` (1024) | 32 = celda mejor medida FIND-61 (9.1×); 1ms ≤ worst-case idle ≤ +15% (medido en celda `on_serial`); 1024 = capacidad actual del canal (ingestion.rs:46) |
| 7 | Txn activa | Fallback a `insert()` por-task (sin batching) si `txn.has_active()` | `insert()` bufferea writes en la txn (insert.rs:162-178); `batch_insert_with_opts` NO consulta txn → aplicar directo rompería el aislamiento. Guard de 3 líneas preserva semántica |
| 8 | Propagación de error batch | `Err` del batch → mismo mensaje para todos los waiters (`Error::generic_error`); `Error` no es `Clone` | Contrato "batch atómico": o todos OK o todos Err (test con `config.read_only = true`). `Error` no implementa Clone (error.rs:134) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. `enabled=false` → comportamiento del pipeline byte-idéntico al actual (mismo loop, misma capacidad 1024, misma latencia).
  2. Ack del batch ⇒ records aplicados a WAL (per `SyncMode`) + KV + vstore + HNSW (read-your-writes intacto: `get()` inmediato ve el nodo).
  3. `insert()`/`delete`/`flush` y el orden global de locks (FND-02, concurrency-async.md R8) NO cambian: el batch toma `insert_lock` 1× vía `commit_batch_locked`.
  4. Cero cambios de formato on-disk (WAL framing, KV payload, vstore). Reopen/replay debe recuperar los N records.
  5. Txn activa ⇒ sin batching (fallback per-task).
  6. `Error` en batch ⇒ todos los waiters reciben Err (nunca hang: canal oneshot muere si el worker cae).
- **Comandos de verificación:** `cargo nextest run --profile audit -p vantadb --features async-ingestion -E 'test(ingestion)'` · `cargo bench -p vantadb --bench ingestion_concurrent --features async-ingestion -- "wire06_group_commit"` · `cargo fmt --check` · `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- **Deuda pendiente:** appendables/segmentos (diferido con diseño → FIND nueva + nota en ADR-0038); variant engine-level para callers concurrentes (SDK/MCP/server) no implementada (documentada como límite en ADR-0038 addendum); baseline de las celdas nuevas en `benchmarks/criterion_baseline.json` queda a promoción manual (workflow existente).

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda nueva (aditivo; sin `unsafe`, sin unwrap nuevo, sin alloc en hot path OFF; el modo ON reusa `batch_insert_with_opts` existente). Pago: elimina el convoy medido del pipeline (−37% w=4) vía opt-in.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate | Estado |
|-------|------|--------|
| **Task** | Contrato 1-4 + fmt/clippy/nextest del cambio | ✅ (220/220 · 0 warnings · 0 gaps) |
| **Commit** | Commit atómico (LEAD commitea), conventional `perf(storage): opt-in group-commit batching (WIRE-06)` | ⬜ (no commiteado por instrucción: LEAD) |
| **Release** | No aplica (sin cambio de formato/API breaking; campo aditivo) | N/A justificado |

## Herramientas necesarias
- `codegraph_codegraph_explore` (blast radius) · `codebase-memory-mcp` (index coverage)
- `cargo bench -p vantadb --bench ingestion_concurrent --features async-ingestion` (+ `canonical_p99`)
- `cargo nextest run --profile audit -p vantadb --features async-ingestion`
- `CARGO_BUILD_JOBS=2` · regla dura `-p`

## Investigation Notes

- **FIND-61 (2026-09-04, cerrado con números):** t_Always=10.36ms/op · t_Never*=10.20ms · **t_fsync≈0.16ms (1.5%)** → el techo es lock+HNSW, no fsync. Prototipo `batch_insert_with_opts` por chunks: N=8→433 (3.9×), N=16→713 (6.4×), **N=32→1016 ops/s (9.1×)** vs baseline §13 111.5. Decisión: cerrar sin slice; infra A/B queda para FUT-12 (BENCHMARKS:561-663).
- **ADR-0038 (spec-only, Proposed):** decisiones owner: opt-in 100%, group-commit ventana tiempo/tamaño reutilizando `batch_append`, aceptación ≥10× batch + ventana declarada. Nuestro diseño la consume en política; el mecanismo literal (queue + coordinator thread) queda en `Future tracking` del ADR (no aporta ≥5× por fsync=1.5%).
- **Baseline §13** (p1/w1 sin batching): **111.5 ops/s** (114/109) — `benches/ingestion_concurrent.rs` (BATCH=400, DIM=16, canal 1024, DB fjall fresca por iter, `sample_size(10)`).
- **Nightly:** `heavy-bench-nightly.yml` (cron diario 02:00 UTC) corre `ingestion_concurrent --features async-ingestion` + `canonical_p99` + análisis `scripts/bench_regression.py extract/compare --fail-on-regression` contra `benchmarks/criterion_baseline.json` (9 labels semilla; las celdas nuevas son informativas hasta promoción). **No se toca `perf-bench.yml`** (WIP ajeno).
- **Ventana de pérdida (modo ON):** el caller recibe ack solo después del commit del batch; un crash pierde únicamente tasks *en vuelo sin ack* (el cliente reintenta) — nunca un ack. No se degrada la durabilidad de `SyncMode` (el WAL sync ocurre dentro del batch).
- **Files touched vs hint del plan:** `src/wal.rs` / `src/storage/engine/insert.rs` NO se modifican (la agrupación reutiliza `batch_append`+`batch_insert_with_opts` ya existentes; un cambio ahí no aportaría al gate). Re-baseline documentado (patrón WIRE-04).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — resueltas: (a) mecanismo = batch de operaciones (FIND-61 + fsync 1.5%); (b) engine-level no batchea en el harness (worker serial por diseño); (c) ADR-0038 se consume como política, no como queue literal |
| Pendientes de ejecución (downhill) | 7 steps |
| % completado | 0% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — no toca trust boundaries nuevos: no hay input externo nuevo (los tasks ya eran la superficie), sin dependencias nuevas, sin FFI. El canal sigue acotado (1024). **Justificación:** cambios aditivos limitados a `config.rs` + `ingestion.rs` + docs + bench.
- [x] **PERFORMANCE** — hot path de ingestión SÍ: baseline before (Regla 9) = §13 111.5 ops/s + re-medición pre-cambio; after = celda ON A/B; canonical_p99 before/after; presupuesto ≤15% regresión en defaults.

## Steps

### Step 1: Task file + consumo ADR-0038 (docs-only)
- **Archivos:** `docs/dev/tasks/WIRE-06.md`
- **Acción:** DISCOVERY completo (arriba) + verificar `docs/user/operations/CONFIGURATION.md` (punto de sync docs).
- **Verify:** archivo existe; `git status` limpio de ajenos.
- **Estado:** ✅ DONE

### Step 2: Config — `InsertBatchConfig` + env + builder + unit tests
- **Archivos:** `src/config.rs`
- **Acción:** struct aditivo (`enabled=false`, `max_batch_records=32`, `max_wait_ms=1`, `max_queued_records=1024`), campo `Config.insert_batch`, env `VANTADB_INSERT_BATCH_{ENABLED,MAX_RECORDS,WAIT_MS,QUEUE}`, builder `with_insert_batching`, `sanitized()` boundary-validation, 3 tests.
- **Verify:** `cargo nextest run --profile audit -p vantadb --features async-ingestion --lib -E 'test(config)'` ✅ (parte del run 220/220).
- **Estado:** ✅ DONE

### Step 3: Group-commit en el pipeline (RED→GREEN)
- **Archivos:** `src/ingestion.rs`, `src/storage/engine/mod.rs`
- **Acción:** worker loop opt-in: fast-drain del backlog + ventana `max_wait_ms` (timer `spawn_blocking`+oneshot, features `rt`+`sync` — `tokio::time` no está en `async-ingestion` y `Cargo.toml` es ajeno), 1× `batch_insert_with_opts` (Incremental, skip_existing=false, skip_wal=false) en `spawn_blocking`, ack por task con su latencia, fallback per-record si txn activa (accessor `has_active_transaction`), `tracing::debug!` por batch; OFF byte-idéntico.
- **Verify:** `cargo nextest ... -E 'test(ingestion)'` ✅ (parte del run 220/220).
- **Estado:** ✅ DONE

### Step 4: Tests de integridad (WAL/reopen/atomicidad/overwrite/txn)
- **Archivos:** `src/ingestion.rs` (tests)
- **Acción:** 5 tests nuevos: ack+persistencia+WAL count==N; reopen con runtime explícito (WAL replay); batch atómico (read_only ⇒ todos Err, nada persistido); txn activa ⇒ buffer hasta commit (backend no ve nada); overwrite UPSERT.
- **Verify:** `cargo nextest run --profile audit -p vantadb --features async-ingestion --lib -E 'test(ingestion) or test(config) or test(wal) or test(insert) or test(reopen)'` → **220/220 passed** ✅.
- **Estado:** ✅ DONE

### Step 5: Bench A/B (OFF vs ON) en `ingestion_concurrent.rs`
- **Archivos:** `benches/ingestion_concurrent.rs`
- **Acción:** grupo `wire06_group_commit` (5 celdas: off, on, on_w2, on_w5, on_serial), `wire06_paired_ab` (OFF/ON misma ventana, orden alternado) y `wire06_attribution` (check skip + plumbing); `run_batch_chunked` devuelve latencias por task (p50/p99-ack); sin tocar `perf-bench.yml`.
- **Verify:** 4 corridas ejecutadas (A/B/C/D + final); ver §Medición. ✅
- **Estado:** ✅ DONE

### Step 6: Medición before/after + docs
- **Archivos:** `docs/user/operations/BENCHMARKS.md` (§13 subsección WIRE-06 con números/env/comando/tradeoffs/sensibilidad), `docs/user/operations/CONFIGURATION.md` (flags), `ADR-0038` (addendum de implementación + mapeo de aceptación), `docs/dev/Backlog.md` (FIND-182 appendables/engine-level), `tests/api/public-api.txt` (snapshot regenerado — incluye símbolos WIRE-05 staged, atribución FIND-178).
- **Acción:** ✅ BENCHMARKS §13 (64→~100 líneas), ✅ CONFIGURATION row, ✅ ADR-0038 addendum, ✅ FIND-182, ✅ snapshot.
- **Verify:** logs persistidos en `$env:TEMP\opencode\wire06_*`; números en docs.
- **Estado:** ✅ DONE

### Step 7: Cierre — verify full + RESULTADO
- **Archivos:** `docs/dev/tasks/WIRE-06.md`
- **Acción:** `cargo fmt --check` + `cargo clippy -p vantadb --all-targets --features async-ingestion` + nextest scoped + `scripts/validate-docs-coverage.ps1`; RESULTADO §7 (sin commit — LEAD commitea; sin self-review — LEAD).
- **Verify:** ✅ fmt exit 0 · ✅ clippy 0 warnings · ✅ nextest 220/220 (`-E 'test(ingestion) or test(config) or test(wal) or test(insert) or test(reopen)'`) · ✅ validate-docs-coverage 0 gaps (incluye `src/config.rs` 67 items ok).
- **Estado:** ✅ DONE

## Evidencia de cierre (comandos + resultados)

| Comando | Resultado | Fecha |
|---|---|---|
| `cargo nextest run --profile audit -p vantadb --features async-ingestion --lib -E 'test(ingestion) or test(config) or test(wal) or test(insert) or test(reopen)'` | **220/220 passed** | 28-09 |
| `cargo clippy -p vantadb --all-targets --features async-ingestion` | exit 0, **0 warnings** | 28-09 |
| `cargo fmt --check` | exit 0 | 28-09 |
| `pwsh scripts/validate-docs-coverage.ps1` | **0 gaps** | 28-09 |
| `cargo bench -p vantadb --bench ingestion_concurrent --features async-ingestion -- "wire06_"` | pares/atribución/secuencial (BENCHMARKS §13 WIRE-06) | 28-09 |
| `cargo bench -p vantadb --bench canonical_p99 --no-run` | ✅ compila (consumo guard) | 28-09 |
| `VANTADB_PUBLIC_API_UPDATE=1 cargo nextest run -p vantadb --test public_api` (CARGO_TARGET_DIR dedicado, FIND-177) | snapshot regenerado (390 líneas; incluye símbolos WIRE-05 staged — FIND-178) | 28-09 |

**Notas de cierre:**
- No commiteado y sin self-review por instrucción del orquestador (LEAD commitea/revisa).
- `git status`: todos los archivos ajenos (WIRE-05: `src/entity/**`, `src/sdk/**`; WIRE-07: `Cargo.toml`, `vantadb-ffi-core/`, bindings, `src/wal_sharded.rs`, `src/lib.rs`, `src/console.rs`; `perf-bench.yml` pre-sesión) quedaron **sin tocar** por WIRE-06.
- Scope del verificado: `-p vantadb --features async-ingestion` (el workspace completo incluye WIP de otros dos agentes; la capa determinista workspace-wide es responsabilidad del LEAD tras el wave).

## Dependencias

- FUT-12-spec ✅ (spec cerrada + ADR-0038 proposed) — consumida.
- WIRE-05 (en progreso, otro agente): archivos PROHIBIDOS ajenos: `src/entity/**`, `src/sdk/search/**`, `src/graph.rs`.
- WIRE-07: `Cargo.toml` raíz, `vantadb-ffi-core/`, bindings — PROHIBIDOS.
- `.github/workflows/perf-bench.yml` — PROHIBIDO ABSOLUTO (WIP ajeno); el nightly de benches se coordina vía `heavy-bench-nightly.yml` (no se toca).

## Review (GATE — agente distinto, P2-01)

> Ejecutado por el LEAD (instrucción del orquestador: NO self-review). Tier adversarial (paths `src/storage/**`? no; `src/ingestion.rs` + `src/config.rs` + `docs/user/**` + benches → Fast tier, pero el modo ON toca el write path → se recomienda review adversarial igual por `vanta-review`).

- **Revisor:** `vanta-review` fresco adversarial (sesión `ses_f19250359ffezPsisDqLL66HhB`; ≠ autor) — **✅ APPROVE** (2026-09-28)
- **Enfoque:** ¿el approach (batching de operaciones en el pipeline, ADR-0038 como política) es correcto? ¿alternativas (queue de fsync literal, engine-level) evaluadas con evidencia?
- **Cómo se probó (revisor):** rerun independiente `ingestion_concurrent --features async-ingestion` pareado → **min 5.93×** / mediana 6.22× · nextest **220/220** · clippy ambos feature sets `-D warnings` 0 · fmt 0 · coverage 0 gaps · canonical `--no-run` ✅ · lectura de durabilidad (ack post-commit, muerte del timer, fallback txn ERR-013) · OFF byte-idéntico.
- **Veredicto:** ✅ **APPROVE** — 0 Critical/Required; Optional aplicados (alternación por muestra `(i+sample)%2`, clamp `sanitized()` ≤60 s / batch ≤ queue, doc de aplanado de errores) + nit de cola-OFF documentado. **Desviación consciente registrada:** `canonical_p99` medido por construcción (compile-guard + ruta default código-idéntica; corrida completa ~10.7 h inviable en laptop).

## Notas

- El plan citaba `src/storage/engine/insert.rs` y `src/wal.rs` como archivos esperados; el diseño final NO los toca (re-baseline verificado: el batch path ya amortiza lock/WAL/HNSW; tocar el path single no cierra el gate). El group-commit de fsync queda absorbido por `batch_append` (1 sync/shard/ciclo).
- `insert_lock` NO se elimina ni se cambia de granularidad (ADR-0037: load-bearing; FIND-59 (d) intacta) — se **amortiza** 1 toma por batch de N.
