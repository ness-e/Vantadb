# FIND-59: Serialización global `insert_lock` — DISCOVERY granularidad (vanta-arch)

## Metadata
- **Backlog:** `docs/Backlog.md` L219 (fila FIND-59, Media, análisis 1-2d)
- **Creado:** 2026-09-04
- **Estado:** ✅ COMPLETO (análisis, 0 código producción)
- **SDP (manual — sin MCP campaign en este entorno):** `documentation-and-adrs` + `api-and-interface-design` + `database-design` cargadas vía `skill`. Lifecycle: DEFINE/análisis (no BUILD — la tarea prohíbe código). Keywords del contrato: `insert_lock`, `concurrency`, `WAL ordering`, `HNSW atomicity`, `bench §13`. Sin candidatas extra: es análisis puro, no fix ni feature.
- **Sub-agente / Ruta:** vanta-arch (análisis/DISCOVERY — NO implementa)
- **Área:** `src/storage/engine/insert.rs`, `src/storage/engine/mod.rs` (solo lectura), `docs/operations/BENCHMARKS.md` §13, `docs/architecture/adr/`. NADA en `src/` se escribe.
- **Paralelo declarado:** FUT-12 (mitad WAL, roadmap P24) — esta tarea cubre SOLO la mitad insert_lock. No tocar archivos de otros agentes. NO stagear `completions/`, `Cargo.lock`, `.opencode`; NO tocar `stash@{0}`.

## Blast Radius (Discovery — lectura completa)
- `src/storage/engine/mod.rs` (completo 856L): `insert_lock: FairMutex<()>` (L318) + `acquire_insert_lock(op)` con degradación wasm→`try_lock` (L570-594).
- `src/storage/engine/insert.rs` (completo 881L): `insert()` WAL+apply bajo un guard (L99-116); `batch_insert_with_opts()` fases 3-4 bajo UN guard (L750-759) + HNSW bulk `add_with_level` con RNG local (L781-805).
- `src/storage/engine/delete.rs` (completo 318L): `delete()` (L54-62), `delete_batch()` (L234-240) bajo el mismo lock.
- `src/storage/engine/ops.rs` (parcial): `flush_pending_hnsw()` (L124), `drain_hnsw_batch_locked()` (L135), `try_push_pending_hnsw()` oportunista sin bloqueo (L162-204).
- `src/storage/engine/maintenance.rs` (parcial): `flush()` checkpoint ERR-010 (L36-48), `refresh_index`/`consolidate_node_inner`/`rebuild_vector_index`/`compact_layout_bfs`/quantization — todos adquieren el lock.
- `src/storage/engine/txn.rs` (completo 514L): `commit_transaction()` hace WAL `batch_append` + apply SIN `insert_lock` (solo `try_push` oportunista) — hueco ERR-010 en path txn, observación colateral no verificada → dentro del scope del spike FIND-61.
- `src/storage/engine/init.rs` `recover_state()` (L396+): single-thread en open, sin lock; replay ordenado por `global_seq` con skip `checkpoint_seq` (round-robin por shard).
- `src/index/graph.rs` vía codegraph: `CPIndex { nodes: DashMap, entry_point: AtomicU128 }` (L330/332); `add()` = `random_layer()` (lock `rng`) + `insert_hnsw()` multi-paso (entry-point read, `search_layer`, `connect_layer_neighbors` bidireccional, `update_metadata`) — NO atómico sin el lock externo.
- `src/storage/engine/get.rs` `get()` (L70+): SIN `insert_lock` (solo txn-buffer locks + `volatile_cache.try_write/read`) — lectores ya lock-free vía RCU (`ArcSwap<CPIndex>`) + DashMap.
- `src/wal.rs` (L304-397): `append`/`batch_append` → `maybe_sync()` con `DEFAULT_PERIODIC_THRESHOLD = 1` (sync por escritura por default); `src/wal_sharded.rs`: shards con Mutex por shard, round-robin.
- `src/config.rs`: `SyncMode::{Always, Periodic(default), Never}` (L87-103), `insert_lock_timeout_ms: 5000` (L165).
- `.opencode/rules/concurrency-async.md` R1/R2/R8 (orden global `cardinality_stats → insert_lock → {wal, pending, hnsw, vstore, cache, backend}`).
- `benches/ingestion_concurrent.rs` + BENCHMARKS §13 (matriz p{1,4}×w{1,2,4}, techo 111.5-113.5 ops/s, −43% con w=4).
- Clave arquitectónica: el core NO tiene `namespace` (`UnifiedNode` = id/bitset/vector/relational/edges; namespace vive en SDK/memory layer) → sharding "por namespace" exigiría plumbear clave nueva; el sharding viable sería por hash-de-id.

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:** mod.rs, insert.rs, delete.rs, txn.rs, ingestion.rs, BENCHMARKS §13, concurrency-async.md, config.rs (parcial), wal.rs (parcial), wal_sharded.rs (parcial), get.rs (parcial), init.rs (parcial), maintenance.rs (parcial), ops.rs (parcial).
- **Referencias hacia dentro:** `insert_lock` lo toman insert / batch_insert / delete / delete_batch / flush / flush_pending_hnsw / refresh_index / consolidate / evict-locked / rebuild / compact / quantization. Lectores (`get`/search) NO lo toman. Recovery NO lo necesita (single-thread).
- **Referencias entrantes:** `AsyncIngestionPipeline::process` → `engine.insert` por tarea (un fsync por op).
- **Veredicto:** cambio de granularidad tocaría el invariante ERR-010 + protocolo FND-02 + orden Regla 8 en 6+ paths; blast radius alto, ganancia acotada por fsync (mitad FUT-12). Recomendación (d) + spike FIND-61. 0 código en esta tarea.

## Contrato
El ADR `docs/architecture/adr/ADR-037-insert-lock-granularity.md` existe con invariantes + matriz alternativas×riesgo×ganancia + recomendación explícita (d) Y fila FIND-59 ELIMINada de Backlog con registro en `docs/avance/activo/core-engine.md` Y nota de 2 líneas en BENCHMARKS §13 Y (spike follow-up) fila FIND-61 creada Y `cargo fmt` N/A (0 código) + markdownlint 0 en archivos tocados + `git status` sin `src/` modificado.

## Steps
### Step 1: Discovery completo (codegraph + lectura)
- **Acción:** localizar locks, holders, invariantes, alternativas (a/b/c/d), riesgos R1/R2/Regla 8/durabilidad/WASM, baseline §13.
- **Verify:** matriz poblada con evidencia file:línea.
- **Estado:** ✅ DONE (2026-09-04 — ver ADR-037 § Invariantes y § Matriz).

### Step 2: Redactar ADR-037
- **Archivos:** `docs/architecture/adr/ADR-037-insert-lock-granularity.md` (nuevo; sigue frontmatter de ADR-036 + plantilla `docs/_templates/adr.md`).
- **Verify:** existe + matriz + recomendación explícita (d) + assessment de contrato (api-and-interface-design).
- **Estado:** ✅ DONE (2026-09-04).

### Step 3: Backlog — eliminar FIND-59 + crear FIND-61 (spike timeboxeado)
- **Archivos:** `docs/Backlog.md`.
- **Acción:** eliminar fila FIND-59 (análisis completo); añadir FIND-61 (spike medición ≤1d, 0 código prod: desglose fsync-vs-lock con SyncMode Never/Always + prototype batch en pipeline; gate >2× para abrir slice de implementación; incluye verificar cobertura ERR-010 del path commit_transaction).
- **Verify:** `rg -c "FIND-59" docs/Backlog.md` == 0 (salvo historial) + fila FIND-61 presente.
- **Estado:** ✅ DONE (2026-09-04).

### Step 4: BENCHMARKS §13 nota + avance
- **Archivos:** `docs/operations/BENCHMARKS.md` (§13, nota 2 líneas), `docs/avance/activo/core-engine.md` (entrada FIND-59).
- **Verify:** nota presente; entrada avance presente.
- **Estado:** ✅ DONE (2026-09-04).

### Step 5: Cierre
- **Acción:** markdownlint en los 4 archivos docs; `git status` confirma 0 `src/`; commit scoped solo-docs (NO stagear completions/Cargo.lock/.opencode; NO tocar stash@{0}).
- **Estado:** ✅ DONE (2026-09-04 — markdownlint 0 en 4 archivos; `git status` sin `src/` ni `Cargo.lock`; commit `f38f3316` scoped 4 docs; stash@{0} intacto; pre-existentes completions/ci-rustdoc.yml/.opencode no tocados).

## Dependencias
- RES-03 + FIND-57 (baselines §13) — landed. FUT-12 (mitad WAL) — prerequisite de política de durabilidad para cualquier batching con ventana de pérdida; NO bloquea (d).

## Notas
- El análisis NO pudo cuantificar el desglose fsync-vs-lock sin medir — declarado explícitamente en ADR-037 § Ganancia estimada; el spike FIND-61 lo resuelve (no se implementó acá por contrato de la tarea).
- Sin empate: (d) converge con rationale; (c) queda como follow-up condicionado a FUT-12, no como alternativa empatada.
