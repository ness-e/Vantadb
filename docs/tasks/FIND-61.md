# FIND-61: Spike medición insert_lock vs fsync — desglose + decisión batch (vanta-tuner)

## Metadata
- **Backlog:** `docs/Backlog.md` L220 (fila FIND-61, Baja, spike ≤1d, 0 código prod)
- **Creado:** 2026-09-04
- **Estado:** ✅ COMPLETO (medido ×2 corridas + §13.1 + decisión CERRAR, 0 código prod)
- **SDP:** `performance-optimization` + `observability-and-instrumentation` + `source-driven-development` + `doubt-driven-development` cargadas vía `skill` (base fija del agente vanta-tuner §9). Lifecycle mapping + grep manifest: `campaign_discover_skills_v2` (phase BUILD, keywords benchmark/throughput/fsync/insert_lock/batching/WAL/tracing/metrics) devolvió 8 (campaign-executor, source-driven-development, doubt-driven-development, incremental-implementation, test-driven-development, context-engineering, frontend-ui-engineering, api-and-interface-design) — filtradas a las 4 del agente + `performance-optimization`/`observability-and-instrumentation` por keywords (las de frontend/TDD/incremental no aplican a spike 0-prod-code; TDD N/A con rationale: el spike mide, no implementa lógica). Keywords: `insert_lock`, `fsync`, `SyncMode`, `batching`, `WAL ordering`, `ERR-010`, `bench §13`.
- **Sub-agente / Ruta:** vanta-tuner (spike MEDICIÓN — NO implementa, NO optimiza sin bench A/B Regla 9)
- **Área:** `benches/ingestion_concurrent.rs` (solo lectura + extensión bench-only), `docs/operations/BENCHMARKS.md` §13 (subsección spike), `docs/architecture/adr/ADR-037-insert-lock-granularity.md` (contexto), `src/storage/engine/txn.rs:119-213` (solo lectura ERR-010). NADA en `src/` se escribe. NADA en `Cargo.toml`.
- **Paralelo declarado:** FUT-12 (mitad WAL, P24 ❌ Sin implementar — gate 2/2 pendiente, ver Gate de salida). NO tocar archivos de otros agentes. NO stagear `completions/`, `Cargo.lock`, `.opencode`; NO tocar `stash@{0}` (14 stashes en develop, verificado `git stash list`).

## Gate D (question-gates.md) — NO dispara
- Blast radius: 1 bench (`benches/ingestion_concurrent.rs` + flag bench-only) + 2 docs (BENCHMARKS §13, avance) + task file. <10 archivos, 0 hot path productivo tocado, 0 API pública nueva, 0 `pub fn` nuevos (el prototype vive en el harness `benches/`, no en `src/ingestion.rs`), contrato no ambiguo (tabla Always/Never/batch-N + decisión ABRIR/CERRAR explícita). Spike research con métricas, no feature-add → sin `## Spec`, sin `question` al usuario. Procede a EJECUCIÓN.

## Blast Radius (Discovery — lectura completa)
- `benches/ingestion_concurrent.rs` (completo 123L): harness RES-03 — `AsyncIngestionPipeline::new(engine, Some(workers))` + `engine.insert` por tarea vía `process` (`ingestion.rs:107-116`); `StorageEngine::open(path)` default (Periodic threshold 1 = fsync por op); matriz p{1,4}×w{1,2,4}, BATCH=400 DIM=16, criterion `iter_custom`, DB fjall fresca por lote. Baseline post-FIND-57: p1/1 **111.5 ops/s** (114/109).
- `src/config.rs:87-103` `SyncMode::{Always, Periodic(default), Never}` + `with_sync_mode` (L979) + `with_flush_threshold` (L1053) + `flush_threshold: Option<usize>` (L163, default None L178).
- `src/wal.rs:225-299,304-397`: `WalWriter { sync_mode, flush_threshold, records_since_sync }`; `maybe_sync()` (L376-389): `if Always → sync(); else { threshold = flush_threshold or DEFAULT_PERIODIC_THRESHOLD=1; if records_since_sync >= threshold → sync() }`. **HALLAZGO DISCOVERY (crítico para el diseño del spike): `SyncMode::Never` NO tiene rama propia — cae al `else` y con `flush_threshold=None` usa threshold 1 = fsync por escritura, IDÉNTICO a Periodic-default.** `rg "Never" src/` = solo `config.rs:102` (definición) + `config.rs:1308` (parsing) — 0 branches de comportamiento. Por tanto `Never` solo aísla lock+HNSW si se combina con `flush_threshold=Some(HUGE)` bench-only (config del harness, 0 prod). Documentar en §13 como `Never*` con asterisco + rationale (no se toca `src/wal.rs` por contrato PROHIBIDO).
- `src/storage/wal.rs:8-25` `init_wal`: `flush_threshold = config.flush_threshold` → `ShardedWal::new_with_buffer(path, shards, sync_mode, buf, threshold)` — plumbing confirmado.
- `src/wal_sharded.rs:133-235`: `batch_append(Vec<WalRecord>)` agrupa por shard (moved, no clones) → 1 lock + 1 write_all + ≤1 maybe_sync por shard. Por op individual (`append`) = 1 lock + 1 write + maybe_sync por op.
- `src/storage/engine/insert.rs:36-117` `insert()`: WAL `append` + `apply_insert` + `drain_hnsw_batch_locked` bajo UN `insert_lock` (ERR-010 L92-116). `batch_insert_with_opts()` (L517-805): fases 3-4 (WAL `batch_append` + KV `write_batch` + HNSW bulk `add_with_level` con RNG local seed 42) bajo UN guard (L750); `skip_wal` (L753) y `InsertMode::{Incremental,Rebuild,Auto}` + `BatchInsertOptions { skip_existing_check, skip_wal, insert_mode, incremental_threshold }` (`ops.rs:11-60`).
- `src/storage/engine/txn.rs:119-213` `commit_transaction()`: WAL `batch_append([Begin+ops+Prepare])` (L159-161) + apply loop (`apply_insert_with_txn`/`apply_delete`, L167-195) + `Commit` marker (L208-210) — **SIN `acquire_insert_lock` en ningún punto** (solo `active_txns`/`txn_buffers` locks + `try_push` oportunista dentro de apply). Contraste: todos los demás paths mutantes lo toman (insert, batch_insert, delete, flush, consolidate, rebuild...). Hueco ERR-010 colateral de ADR-037 confirmado por lectura.
- `src/storage/engine/maintenance.rs:36-93` `flush()`: guard ERR-010 `[drain → serialize → count checkpoint_seq → write]` (L47-48); `checkpoint_seq` DESPUÉS de serializar (L67-93). `ops.rs:124-204` `flush_pending_hnsw` / `drain_hnsw_batch_locked` / `try_push_pending_hnsw` (oportunista, nunca bloquea).
- `src/backends/fjall_backend.rs:234-243` `flush()`: `persist(PersistMode::SyncAll)` = fsync datos+metadata; pero `insert()` per-op NO llama `backend.flush()` — el único fsync per-op es el WAL `maybe_sync`. Aislar WAL-fsync aísla el techo.
- `docs/operations/BENCHMARKS.md` §13 (matriz + post-FIND-57 111.5 + nota FIND-59) y `ADR-037` (matriz a/b/c/d, decisión (d), gate FIND-61: batch ≥2× + FUT-12 para abrir slice).
- `docs/backlog-futuro.md` P24 FUT-12: ❌ Sin implementar (WAL fsync-batching, requiere decisión de política de sync antes de implementar). Gate 2/2 YA pendiente al abrir el spike → salvo decisión FUT-12 durante el timebox, la salida es CERRAR con números.

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:** benches/ingestion_concurrent.rs, ingestion.rs, txn.rs, insert.rs (parcial 30-120/495-840), ops.rs (parcial 1-80/120-210), maintenance.rs (parcial 30-100), wal.rs (parcial 210-440), wal_sharded.rs (parcial 130-260), storage/wal.rs, config.rs (parcial SyncMode/flush_threshold/builders), fjall_backend.rs (parcial flush), BENCHMARKS §13, ADR-037, Backlog L220 + P24 FUT-12, performance-checklist.md, observability-checklist.md, definition-of-done.md.
- **Referencias hacia dentro:** el prototype batch bench-only llamará `batch_insert_with_opts` (ya ERR-010-conforme, UN guard) o acumulará N tasks y hará 1 `batch_append` lógico — NO tocará `commit_transaction` (path sin lock). El bench `Never*` usará `open_with_config` con `VantaConfig::default().with_sync_mode(Never).with_flush_threshold(HUGE)` — solo harness.
- **Referencias entrantes:** ningún caller productivo depende del bench; `AsyncIngestionPipeline::process` sigue `engine.insert` (default intacto por PROHIBIDO cambiar defaults).
- **Veredicto:** BAJO y contenido. 0 `src/`, 0 `Cargo.toml`, 0 defaults. Reversible por construcción (docs + bench). Gate Regla 0 ✅ — se puede entrar a ACT (solo bench/docs).

## Contrato
BENCHMARKS §13 (o subsección spike `§13.1 FIND-61`) con tabla Always/Never*/batch-N (ops/s + p50-ack-latency + ventana de pérdida explícita) + decisión explícita (ABRIR slice con números ≥2× Y FUT-12 decidido, o CERRAR con números) Y fila FIND-61 eliminada del Backlog Y `cargo test` del bench 0 failed Y clippy/fmt del scope tocado 0 (bench-only: si no se tocó `src/`, N/A con rationale) Y avance core-engine.md u operaciones.md Y memory decision con los números.

## Steps
### Step 1: A/B Always vs Never* (desglose lock vs fsync) — bench-only
- **Acción:** en `benches/ingestion_concurrent.rs` (o harness auxiliar bench-only si el flag lo requiere) abrir el engine con `open_with_config`: (A) `SyncMode::Always` (≈ baseline §13, revalidar ~111.5 ops/s p1/w1) vs (B) `SyncMode::Never + flush_threshold=Some(1_000_000)` (`Never*`, WAL bytes sin fsync = solo lock+HNSW+memcpy). Misma matriz mínima p1/w1 (+ p1/w4 como testigo convoy). ×2 corridas + mediana (regla §13). Registrar ops/s por celda. Desglose: `t_fsync ≈ 1/opsA − 1/opsB` por op.
- **Verify:** tabla A/B con 2 corridas + mediana; `cargo bench -p vantadb --bench ingestion_concurrent --features async-ingestion -- "p1/1"` compila y corre.
- **Estado:** ✅ DONE (2026-09-04 — Run B: Always 97 / Never* p1w1 103 / Never* p1w4 63; Run C: Always 96 / Never* p1w1 93 / Never* p1w4 60; medianas: Always **96.5**, Never* **98.0** (+1.6%), Never* p1w4 **61.5** (−37% convoy sin fsync). `t_fsync ≈ 0.16 ms/op ~1.5%` — lock+HNSW-dominado, revisa "fsync-dominado" acotado de ADR-037 con evidencia; decisión (d) intacta).

### Step 2: Prototype micro-batching bench-only N={8,16,32} + p50-ack-latency + ventana de pérdida
- **Acción:** SOLO en el pipeline del harness (flag bench-only, nunca `src/ingestion.rs`): acumular N `IngestionTask` → 1 `batch_insert_with_opts` (`skip_existing_check=true` IDs frescos, `skip_wal=false`, `InsertMode::Incremental`) bajo UN guard. Medir por N: ops/s end-to-end + p50-ack-latency (tiempo submit→ack por task dentro del batch: el primero espera al batch completo) + ventana de pérdida explícita (N writes ante crash entre batches; con `skip_wal=false` la ventana es N ops en memoria no-acked, NO durables). Comparar contra §13 (111.5). Gate ≥2× = ≥223 ops/s.
- **Verify:** tabla batch-N con las 3 métricas + ventana declarada por N.
- **Estado:** ✅ DONE (2026-09-04 — Run B vs Run C Collecting-medianas → final: N=8 **433 ops/s / 18.6 ms / 8 writes (3.9×)**, N=16 **713 / 22.2 ms / 16 (6.4×)**, N=32 **1016 / 31.6 ms / 32 (9.1×)** sobre §13 111.5. Gate ① CUMPLE con margen; p50-ack crece con N — tradeoff explícito).

### Step 3: Verificar ERR-010 del path commit_transaction (txn.rs:119-213)
- **Acción:** revisión por interleaving (0 código): ¿puede `flush()` (guard ERR-010) intercalarse entre el `batch_append` (L160) y el apply loop (L167-195) de `commit_transaction` y contar WAL records cuya mutación HNSW sigue encolada vía `try_push` oportunista? Documentar RESPETA o VIOLA + escenario concreto + si el prototype del Step 2 lo respeta (usa `batch_insert` con lock → respeta) o lo viola (si usara commit path → viola).
- **Verify:** veredicto explícito con file:línea por cada afirmación.
- **Estado:** ✅ DONE (2026-09-04 — **VIOLA**: interleaving commit-WAL-durable → flush-drain-vacío → serialize → count-incluye-commit → checkpoint → commit-pushea-tarde = invisible record; `txn.rs:119-213` vs `maintenance.rs:47-93` vs `ops.rs:162-204`. Prototype **RESPETA** vía `batch_insert_with_opts` UN guard `insert.rs:750`. Fix = `src/` prod → fuera del spike, colateral para el lead).

### Step 4: BENCHMARKS §13.1 + decisión gate + Backlog + avance + memory
- **Acción:** escribir subsección spike en BENCHMARKS §13 (tabla Always/Never*/batch-N + entorno Regla 11 + nota `Never*` + decisión ABRIR/CERRAR con los 2 gates evaluados por separado: ① batch ≥2× sobre 111.5, ② FUT-12 decidió ventana de pérdida). Eliminar fila FIND-61 del Backlog con motivo. Registrar en `docs/avance/activo/core-engine.md` (o `operaciones.md`) + `campaign_memory_write(file="decisions")` con los números.
- **Verify:** `rg -c "FIND-61" docs/Backlog.md` == 0 (salvo historial/avance); §13.1 presente con tabla + decisión.
- **Estado:** ✅ DONE (2026-09-04 — §13.1 con scorecard + Tablas 1-2 + desglose + gate ① CUMPLE/② NO → CERRAR; Backlog sin fila; avance core-engine.md; memory decisions pendiente al cierre).

### Step 5: Cierre (verify + commit scoped)
- **Acción:** `cargo test` del bench 0 failed + clippy/fmt del scope tocado 0 (si 0 `src/`, N/A con rationale) + markdownlint 0 en docs tocados + `git status` confirma 0 `src/` + 0 `Cargo.toml` + 0 defaults cambiados. Commit scoped docs/bench: `perf(spike): desglose insert_lock vs fsync + decisión batch (FIND-61)`. Si gate NO se cumple: commit igual con decisión de NO abrir slice + fila eliminada con motivo. NO stagear `completions/`, `Cargo.lock`, `.opencode`; NO tocar `stash@{0}`.
- **Verify:** commit hash + `git status` limpio en scope prohibido.
- **Estado:** ✅ DONE (2026-09-04 — bench-test 12/12 Success exit 0 + lib ingestion 1/0 + clippy bench-scope 0 + fmt 0 + Backlog sin fila (header 104) + memory decisions + commit scoped docs/bench `perf(spike): ... (FIND-61)`; `.opencode` (task file) sin stagear por precedente FIND-59; stash@{0} intacto; completions/Cargo.lock no tocados)

## Dependencias
- FIND-59/ADR-037 (decisión (d), baseline §13 111.5) — landed. FUT-12 (política ventana de pérdida) — ❌ Sin implementar, gate 2/2 pendiente. RES-03/FIND-57 (harness + matriz) — landed.

## Notas
- Metric-honesty (Regla 2b): sin artifacts el scorecard es `not measured`; cada hallazgo es `potential impact` hasta que el bench lo mida. Bench ≠ live: los números son sintéticos (BATCH=400 DIM=16 fjall tempdir Win11), no usuarios reales.
- `SyncMode::Never` sin rama en `maybe_sync` NO es un fix de este spike (PROHIBIDO `src/`) — es evidencia para §13.1 + posible fila FIND futura (fuera de este contrato).
- Timebox ≤1d: si se agota, devolver estado exacto + próximo step, nunca silencio (RESULTADO 🟡 INCOMPLETO con PROXIMO_STEP).
