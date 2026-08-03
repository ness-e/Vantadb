---
title: "Backlog Validation Report — 2026-07-28"
type: audit-report
status: completed
tags: [vantadb, audit, backlog-validation, cross-check]
verified_by: "5 sub-agentes explore (vanta-lead orchestrated), codegraph_explore, grep, glob, read directa"
methodology: "Cada item del backlog activo verificado contra código real. 0 modificaciones."
---

# 🏗️ VantaDB — Validación Completa de Task Files

**Fecha:** 2026-07-28
**Metodología:** 5 sub-agentes paralelos (explore), verificando cada item contra código real vía `codegraph_explore`, `grep`, `glob`, y lectura directa. **0 archivos modificados.**

---

## Resumen General

| Origen | Total | ✅ Implementado | ⚠️ Parcial | ❌ No Implementado | 🔵 Scaffolding/Deferido |
|--------|-------|-----------------|-------------|---------------------|------------------------|
| **Task file formal** (`.opencode/tasks/`) | 1 | 1 | 0 | 0 | 0 |
| **Plan files** (`docs/plans/`) | 3 planes, 15 tasks | 14 | 0 | 0 | 1 (WONTFIX) |
| **Backlog P0** (Release Blockers) | 1 | 0 | 0 | **1** | 0 |
| **Backlog P5** (Docs & Community) | 8 | 2 | 5 | 1 | 0 |
| **Backlog P6** (Launch Campaign) | 8 | 3 | 3 | **2** | 0 |
| **Backlog P8** (Post-Launch) | 8 | 0 | 2 | 5 | 1 |
| **Backlog P9** (Old Docs Rescue) | 20 | 17 | 1 | 1 | 1 |
| **Backlog P10** (Competitive Features) | 20 | 13 | 2 | 3 | 2 |
| **TOTAL** | **69** | **50** | **13** | **10** | **5** |

> *Nota: Tasks en `progreso/README.md` (completadas históricamente, ~95 items) no reverificadas — se asumen correctas a menos que el backlog activo las contradiga.*

---

## 1. Task File Formal: `.opencode/tasks/COMP-010.md`

Archivo único de definición formal de tarea en el proyecto.

| Claim | Verificación | Estado |
|-------|-------------|--------|
| `EmbeddingProvider` trait en `src/llm.rs` | Línea 26-29: `pub trait EmbeddingProvider: Send + Sync { fn embed(...) }` | ✅ |
| `OllamaProvider` implementa trait | Líneas 66-126: POST `/api/embeddings`, env vars `VANTA_LLM_URL`, `VANTA_LLM_MODEL` | ✅ |
| `OpenAIProvider` | Líneas 134-219: POST `api.openai.com/v1/embeddings`, `text-embedding-3-small` | ✅ |
| `get_embedding_provider()` factory | Línea 39-47: lee `VANTA_EMBEDDING_PROVIDER`, default Ollama | ✅ |
| 4 call sites actualizados | `executor.rs:237,369`, `physical_plan.rs:235,746` | ✅ |
| `summarize_context()` en `LlmClient` | Líneas 258-344 — solo generación de texto | ✅ |
| 6 tests de auto-embedding | 6 tests gated `#[cfg(feature = "remote-inference")]` | ✅ |

**Veredicto:** ✅ IMPLEMENTADO — el task file describe exactamente el código existente.

---

## 2. Plan Files (`docs/plans/`)

### 2a. `2026-07-25-p4-engineering-health.md` — claims 6/6 completados

| ID | Claim | Archivos verificados | Estado |
|----|-------|---------------------|--------|
| WEB-04 | Storage format versioning | `src/migration.rs` — MigrationEngine, plan_all, migrate_format, check_integrity | ✅ |
| DRV-121 | CBO optimize_and_compile | `src/planner.rs:195` — 5 tests: scan-only, scan+filter, filter-no-match, identity elimination, sort+limit+project | ✅ |
| DRV-123 | Auto-embedding on INSERT | `src/llm.rs` EmbeddingProvider, `executor.rs` + `physical_plan.rs` call sites | ✅ |
| DOC-20 | mdBook docs site | `docs/book/book.toml`, `SUMMARY.md` (8 secciones vs claimed 9 — discrepancia menor) | ✅ |
| VFY-011 | MVCC implementation | `src/storage/engine/ops.rs` — snapshot isolation, concurrent txns, write-write conflict, MVCC GC | ✅ |
| DRV-122 | IQL JOINs/subqueries | Parser → LogicalPlan → Planner → PhysicalPlan — `FromClause::Join`, `SubqueryCondition`, `PhysicalNestedLoopJoin` | ✅ |
| DRV-131 | IVF Flat index | `src/index/ivf.rs` (872L), 16 tests, k-means, VecIndex impl, serialization v8 | ✅ |

**Veredicto:** ✅ 6/6 completados (1 discrepancia menor: SUMMARY.md tiene 8 secciones, plan claims 9).

### 2b. `2026-07-25-p4-drv130-sift-bottleneck.md`

| Claim | Estado |
|-------|--------|
| T1: SearchProfile con `cfg(debug_assertions)` gate | ✅ `src/index/search.rs` — ZST no-op en release |
| T2: Prefetch batching WONTFIX (ya existe) | ✅ `PrefetchMode` enum, `prefetch_mmap_vector` en graph.rs |
| T3: Node reordering WONTFIX (~9% mejora, <20% threshold) | ✅ Decisión documentada, benchmark data consistente |

**Veredicto:** ✅ 3/3 documentado correctamente.

### 2c. `2026-07-25-p4-drv130-t3-node-reordering.md`

| Claim | Estado |
|-------|--------|
| WONTFIX decision | ✅ `compact_layout_bfs` benchmark: in_memory 783ms, with_vfile 2440ms, compacted 2221ms (~9%) |

**Veredicto:** ✅ Documentado.

---

## 3. Backlog P0 — Release Blockers

| ID | Título | Código verificado | Estado Real |
|----|--------|-------------------|-------------|
| **DEVOPS-15** | Reducir default features de 7 a 3 | `Cargo.toml:93`: `default = ["cli", "arrow", "fjall", "advanced-tokenizer", "memmap2", "fs2", "sysinfo"]` — **aún 7 features** | **🔴 PENDIENTE** |

---

## 4. Backlog P5 — Docs & Community

| ID | Título | Estado Real | Evidencia |
|----|--------|-------------|-----------|
| **MKT-14** | Case studies + `/case-studies/` | ✅ | 3 CS en `vanta-data.ts:993-1042`, ruta listing + `[slug]` detail |
| **NUEVO-01** | README hero (readme-aura + benchmark + WASM GIF) | **❌** | README estático, PNG sola, no readme-aura, no GIF |
| **NUEVO-07** | Migration tools Chroma→Vanta, LanceDB→Vanta | ✅ | `vantadb_py/migrate/chroma.py` + `lancedb.py` + CLI, 2 tutoriales |
| **NUEVO-08** | Learning path (5-7 tutoriales) | ⚠️ | 4 tutoriales (3 draft, 1 active) — faltan 1-3 |
| **NUEVO-10** | Benchmark suite pública reproducible | ⚠️ | Scripts existen pero requieren build local; no standalone |
| **COM-02** | Discord: reaction roles, autorole, etc. | ⚠️ | Server existe (3 miembros), docs listas, Carl-bot **no configurado** |
| **COM-03** | Discord: AutoMod, stickers, forums | ⚠️ | Forums seedeados (9 threads ✅). AutoMod/stickers sin configurar ❌ |
| **COM-04** | Discord: ticketing, stage, Canny.io | ⚠️ | Stage channel creado. Ticketing, Discovery, Canny sin configurar |

---

## 5. Backlog P6 — Launch Campaign

| ID | Título | Estado Real | Evidencia |
|----|--------|-------------|-----------|
| **LEG-01** | Trademark USPTO + EUIPO | **❌** | Sin docs, receipt, serial numbers. Solo menciones en backlog. |
| **MKT-05** | Blog posts (5+) | ⚠️ | 3 archivos en `docs/blog/` (backlog dice 4/5). Faltan 2. |
| **MKT-10** | AI Agent Memory campaign | **❌** | Sin materiales de campaña. Solo mención estratégica en GO_TO_MARKET.md |
| **MKT-15** | Benchmarks competitivos `/benchmarks` | ⚠️ | Ruta existe, **`vs-table.tsx` existe** (backlog dice que no). `BenchmarkRace` renderiza comparación. |
| **MKT-16** | Metodología benchmark GraphRAG | **❌** | No existe documento de metodología. GraphRAG docs existen pero no metodología. |
| **MKT-17** | Página comparación interactiva `/compare` | **❌** | No existe ruta `/compare`. Datos comparativos parciales en `vs-table.tsx` y `vanta-data.ts` |
| **TSK-103** | Public benchmark site `/benchmarks` | ⚠️ | Ruta + UI completas. Sin script público reproducible standalone. |
| **TSK-104** | Demo LangChain + Ollama | ⚠️ | `examples/python/langchain_rag.py` + `agent_memory.py` existen pero son experimentales (mock embeddings, no Ollama real) |

---

## 6. Backlog P8 — Post-Launch & Enterprise

| ID | Título | Estado Real | Evidencia |
|----|--------|-------------|-----------|
| **NUEVO-16** | Product Quantization 96x | **❌** | SQ8 y TurboQuant existen. PQ real (sub-vector codebooks) NO. |
| **NUEVO-17** | Segment LSM (hot/warm/cold) | ⚠️ | **Hot/Cold existen** con eviction y promoción. Backlog dice "tiers no" — esto es incorrecto. Falta "Warm" tier y LSM compaction real. |
| **NUEVO-18** | Sparse vectors nativos | **❌** | No existe `SparseVector`. Solo mención de "sparse" en bitset context. |
| **TSK-107b** | Audit logging enterprise (JSONL) | **❌** | Sin módulo audit. JSONL solo para export/import, no para operaciones. Placeholder en `ops.rs:233`. |
| **ENT-04** | Connection pooling + circuit breaker | **❌** | Una métrica existe (`circuit_breaker`). Sin ConnectionPool, sin state machine. *(Snapshot 2026-07-28 — **RESUELTO 2026-08-02**: `src/connection_pool.rs` + `src/circuit_breaker.rs`, cableado en `cli_server.rs`, commit `f0c76768`, unit 9/9 + e2e 2/2.)* |
| **BIZ-01** | Enterprise (encryption + RBAC + audit) | ⚠️ | Crypto (`src/crypto.rs` AES-256-GCM) ✅. RBAC (`src/rbac.rs`) ✅. Enterprise crate separado ❌. Audit ❌. Replication ❌. |
| **WEB-001** | WASM demo en `/playground` | **❌** | `CodePlayground` es **simulador pattern-matching**, no WASM real. Demo WASM de `web_old/` eliminada. |
| **WEB-018** | Pricing strategy | ⚠️ | **2 modelos incompatibles**: `vanta-data.ts` ($49/seat Team) vs `GO_TO_MARKET.md` ($99/mo Pro, $499 Business). Sin decisión estratégica. |

---

## 7. Backlog P9 — Old Docs Rescue

| ID | Claim en backlog | Estado Real | Archivos clave |
|----|------------------|-------------|----------------|
| **OLD-01** | PGWire PostgreSQL wire protocol | **❌ No implementado** | — |
| **OLD-02** | GraphRAG Pipeline formal | ✅ | `src/graphrag/` (6 archivos), pipeline.rs, seed.rs, expand.rs, retrieve.rs, context.rs |
| **OLD-03** | Chaos testing (Jepsen/Maelstrom) | ✅ | `src/testing/chaos.rs`, `docs/chaos-testing.md` |
| **OLD-04** | OpenTelemetry | ✅ | Cargo.toml (4 crates opentelemetry), `src/cli_server.rs`, 19 refs OTel |
| **OLD-07** | AutoHot/Cold tiering | ✅ | `src/storage/engine/maintenance.rs` — evict_cold_nodes, consolidate_node |
| **OLD-08** | Snapshots hard-link | ✅ | `src/storage/engine/mod.rs` — create_snapshot, list_snapshots, CLI handler |
| **OLD-09** | Bayesian Decay (olvido bayesiano) | ✅ | `src/eviction.rs` — BayesianDecay, Beta-Binomial score, 10 tests |
| **OLD-10** | Index-free adjacency (sinapsis eléctrica) | ✅ | `UnifiedNode.edges: Vec<Edge>` es index-free adjacency nativa |
| **OLD-11** | CLI/TUI interactivo | ✅ | `src/tui/` — dashboard, monitor, repl, 3 modos, ratatui + crossterm |
| **OLD-12** | Pilot Program formal | ✅ | `docs/operations/PILOT_PROGRAM.md` — 275L, 9+ secciones |
| **OLD-13** | Explainable ranking | ✅ | `src/sdk/search/debug.rs` — BM25 terms, snippet, phrases, RRF |
| **OLD-14** | MessageThread / GcWorker | ✅ | `src/agentic/thread.rs`, `src/gc.rs` — CRUD + TTL |
| **OLD-15** | Euclidean SIMD | ✅ | `src/index/distance.rs` — 3 kernels f32x8 (dot, euclidean, SQ8) |
| **OLD-16** | WAL Rotation 256MB | ✅ | `src/wal.rs:393` try_auto_rotate, 3 tests |
| **OLD-17** | Migration guides | ✅ | `docs/tutorials/migration-from-lancedb.md` + `03-migrating-from-chromadb.md` |
| **OLD-18** | TEMPERATURE param | ✅ | `src/query.rs`, `src/parser/mod.rs`, `src/governor.rs` — 43 refs, 8 archivos |
| **OLD-19** | Rehidratación desde shadow archive | ✅ | `src/storage/engine/maintenance.rs:903`, SDK binding, 6 tests |
| **OLD-20** | Contextual Priming (cache warming predictivo) | ⚠️ Parcial | `src/cache_warner.rs` (305L, 9 tests) — `decay()` dead_code, no conectado a hot path, métricas sin exportar |
| **OLD-21** | CP-Index routing (query routing inteligente) | 🔵 Deferido | Routing en planner (`classify()`), multi-index en `search_nearest()`. CPIndex es HNSW puro. Depende de COMP-028 |
| **OLD-22** | Arrow columnar export | ✅ | `src/columnar.rs` (136L), nodes_to_record_batch, 6 tests |

---

## 8. Backlog P10 — Competitive Features

| ID | Claim en backlog | Estado Real | Archivos clave |
|----|------------------|-------------|----------------|
| **COMP-006** | Edge Label Interning | ✅ | `Edge.label_id: u32` + LabelIntern map |
| **COMP-008** | Pluggable index engine (VecIndex trait) | ✅ | 5 impls: Hnsw, Flat, DiskAnn, Scann, Ivf |
| **COMP-009** | Binary bulk import | ✅ | `bulk_import_stream()`, `bulk_import_file()` |
| **COMP-010** | Auto-embedding abstraction | ✅ | `EmbeddingProvider` trait, OllamaProvider, OpenAIProvider |
| **COMP-012** | RoaringBitmaps | ✅ | `FilterBitset` backed by `croaring::Bitmap` |
| **COMP-013** | Segment optimizer pipeline | ✅ | Vacuum → FreshHNSW → Merge → Reindex |
| **COMP-014** | FreshHNSW link repair | ✅ | `repair_orphan_links()` en pipeline |
| **COMP-016** | Supernode mitigation | ✅ | `label_index: HashMap<u32, Vec<u128>>` |
| **COMP-017** | Parallel accumulators | ✅ | `GraphAccumulator` lock-free DashMap |
| **COMP-022** | Graph Data Science library | ✅ | `GraphDataScience::page_rank()`, degree_centrality |
| **COMP-023** | 3 filtering strategies | ✅ | PreFilter (<1%), InFilter (1-10%), PostFilter (≥10%) |
| **COMP-024** | ACORN algorithm | ✅ | `acorn_expansion` en search_layer, 2-hop |
| **COMP-027** | Multiple index types | ✅ | Hnsw, Ivf, Flat, DiskAnn, Scann |
| **COMP-018** | Double-linked relationship chains | ✅ | `Edge.reverse: bool`, mirrored en source+target. **Backlog dice ❌ pero está ✅** |
| **COMP-019** | Binary protocol (rkyv/FlatBuffers over gRPC) | **❌** | Sin gRPC, rkyv, FlatBuffers. Solo REST/JSON. |
| **COMP-021** | Temporal edges | **❌** | Edge struct: `target, label_id, weight, reverse` — sin timestamp |
| **COMP-025** | JSON shredding | ⚠️ Phase 1 ✅ | `src/shred/mod.rs` — ShreddedRowStore, schema inference, filter pushdown. Phase 2-3: ❌ delete path, nested JSON, GC |
| **COMP-026** | LSM multi-level compaction | 🔵 Scaffolding | L0-L3 definidos, `trigger_compaction()` es stub (solo log). `ponytail: Phase 2` |
| **COMP-028** | Semantic Cost Estimator (SCE) | ⚠️ Parcial | No hay módulo SCE. Cost estimation distribuido: ResourceGovernor, CBO, `select_filter_strategy()`. Sin modelo per-operator. |
| **COMP-029** | napi-rs bindings | **❌** | TS SDK usa WASM (`wasm-bindgen`), no napi-rs. No depende de napi/napi-derive. |

---

## 9. Discrepancias Backlog vs Realidad

Se detectaron **5 items donde el backlog reporta estado peor/incorrecto** comparado con el código real:

| Item | Backlog dice | Realidad | Corrección |
|------|-------------|----------|-----------|
| **MKT-05** | "4/5 blog posts" | 3 archivos en `docs/blog/` | ⚠️ Backlog **infla** conteo |
| **MKT-15** | "sin tabla competitiva" | `vs-table.tsx` existe con Pinecone/Weaviate/Chroma | ✅ **Mejor** de lo reportado |
| **NUEVO-17** | "tiers no" | Hot/Cold 2-tier existe con eviction y promoción | ⚠️ Parcial implementado, no "no" |
| **MKT-14** | "falta pulir copy y métricas" | 3 CS completos con métricas, listing + detail pages | ✅ **Más completo** de lo reportado |
| **NUEVO-07** | "scripts ejecutables faltan" | `vantadb_py/migrate/chroma.py` + `lancedb.py` existen y son ejecutables | ✅ **Mejor** de lo reportado |
| **COMP-018** | "❌ No implementado" | `Edge.reverse: bool` + mirrored edges en SDK API | ✅ **Ya implementado** — falso negativo en backlog |

---

## 10. Análisis de Inconsistencias No Triviales

### 10.1 DEVOPS-15: 7 Features en lugar de 3
- **Síntoma:** `Cargo.toml:93` tiene `default = ["cli", "arrow", "fjall", "advanced-tokenizer", "memmap2", "fs2", "sysinfo"]`.
- **Historial:** La tarea original pedía 9→3. `prometheus` y `rayon` fueron removidos exitosamente, pero `cli, memmap2, fs2, sysinfo` no.
- **Posible causa raíz:** Dependencias transitivas — `cli` probablemente usado en el binary principal sin feature gate, `memmap2`/`fs2`/`sysinfo` probablemente referenciados directamente en código sin `#[cfg(feature = "...")]`. Sin un `cargo check --no-default-features` que falle, es fácil dejarlos.
- **Riesgo:** Bloqueante para release — más features = más superficie de compilación, más dependencias, más riesgo de seguridad.

### 10.2 LEG-01: Trademark nunca iniciado
- **Síntoma:** Sin documentos, receipts, serial numbers USPTO/EUIPO.
- **Posible causa raíz:** Tarea legal/administrativa fuera del expertise del equipo técnico. Sin budget para abogado. USPTO requiere ~$250-750, EUIPO ~€850. Proceso de 6-12 meses.
- **Riesgo:** Show HN con nombre sin trademark = riesgo de cease & desist post-lanzamiento.

### 10.3 WEB-001: WASM playground es simulador
- **Síntoma:** `CodePlayground` usa pattern-matching de strings, no invoca WASM real.
- **Posible causa raíz:** La demo WASM real estaba en `web_old/` (eliminada). Reconstruir requiere integrar `@vantadb/wasm` (bundle 394KB gzip) en React con worker thread. Más complejo que un simulador.
- **Riesgo:** Percepción de falta de producto funcional en landing page.

### 10.4 COMP-019: gRPC/Binary Protocol nunca iniciado
- **Síntoma:** Sin rkyv, FlatBuffers, ni gRPC server. Solo REST/JSON.
- **Posible causa raíz:** El proyecto es embedded-first (librería embebida). gRPC es útil para modo servidor remoto, pero no es core del producto. Decision arquitectónica no documentada.
- **Riesgo:** Competidores (Pinecone, Weaviate) tienen binary protocol. Percepción de inmadurez.

### 10.5 COMP-028: Semantic Cost Estimator distribuido
- **Síntoma:** No hay módulo SCE unificado. Cost estimation está en ResourceGovernor + CBO + `select_filter_strategy()`.
- **Posible causa raíz:** La funcionalidad existe pero está acoplada a otros componentes. Extraerla a módulo separado requiere refactor sin cambio de comportamiento.
- **Riesgo:** Bajo — la funcionalidad existe, solo no está modularizada.

### 10.6 Discrepancias de reporte (falsos negativos en backlog)
- **Patrón:** Múltiples items que el backlog marca como "no implementado" y están implementados (COMP-018, MKT-15, NUEVO-17).
- **Posible causa raíz:** El backlog se actualiza manualmente, no hay validación automática. Items implementados por sub-agentes no siempre actualizan el backlog.
- **Riesgo:** Medio — desconfianza en el backlog como source of truth.

---

## 11. Mayorías de Riesgo (No Implementado pero Prioritario)

| # | ID | Prioridad | Impacto | Depende de |
|---|----|-----------|---------|------------|
| 1 | **DEVOPS-15** | 🔴 Release Blocker | Seguridad, compilación | Solamente edición de Cargo.toml + feature gates |
| 2 | **LEG-01** | 🔴 Show HN Risk | Legal, marca | Recursos externos, $$$ |
| 3 | **WEB-001** | 🔴 Percepción Producto | Demo funcional | `@vantadb/wasm` bundle + worker |
| 4 | **WEB-018** | 🔴 Decisión Estratégica | Pricing, GTM | Decisión de negocio |

## 12. Mayorías Sorpresa (Mejor de lo Esperado)

| # | ID | Backlog decía | Realidad |
|---|----|--------------|----------|
| 1 | **COMP-018** | ❌ No implementado | ✅ Double-linked edges funcionando |
| 2 | **MKT-15** | ❌ Sin tabla competitiva | ✅ `vs-table.tsx` con Pinecone/Weaviate/Chroma |
| 3 | **NUEVO-17** | ❌ Tiers no existen | ⚠️ Hot/Cold 2-tier ya implementado |
| 4 | **NUEVO-07** | ❌ Scripts faltan | ✅ Scripts de migración ejecutables |

## 13. Items que Requieren Decisión

| Item | Decisión pendiente | Recomendación |
|------|-------------------|---------------|
| **WEB-018** | Elegir pricing: per-seat ($49/seat) vs usage-based ($99/mo) | Unificar en `GO_TO_MARKET.md`, eliminar del site hasta decisión |
| **COMP-025** | Continuar Phase 2 (delete path, nested JSON) o cerrar | Phase 1 es usable. Phase 2 si hay demanda |
| **COMP-019** | gRPC/binary protocol: necesario o WONTFIX | WONTFIX recomendado — proyecto es embedded-first |

---

## 14. Metadatos de la Validación

| Campo | Valor |
|-------|-------|
| **Fecha** | 2026-07-28 |
| **Duración** | ~45min (5 sub-agentes paralelos) |
| **Sub-agentes** | 5 × `explore` |
| **Herramientas** | `codegraph_explore`, `grep`, `glob`, `read`, `bash` |
| **Modificaciones** | 0 |
| **Items validados** | 69 (1 task file + 3 planes + 65 backlog items) |
| **Discrepancias detectadas** | 6 |
| **Backlog reviewer** | `vanta-lead` (Release Orchestrator) |
