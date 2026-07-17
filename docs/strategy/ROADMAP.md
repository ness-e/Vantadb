---
title: "VantaDB — Roadmap de Ejecución"
type: strategy
status: active
tags: [vantadb, roadmap, execution, timeline, priorities]
version: 2.0
created: 2026-07-16
supersedes: 2026-07-01
aliases: [Roadmap, Milestones, Engineering Plan, Timeline, Plan de Acción]
---

# VantaDB — Roadmap de Ejecución

> **Backlog fuente:** [`docs/Backlog.md`](../Backlog.md) — 165 items abiertos
> **Última revisión del proyecto:** 2026-07-16 (537 commits desde el roadmap anterior)
> **Estas fuentes fueron analizadas para este roadmap:**
> - Auditorías: 10 reports en `docs/audit-reports/`
> - Deep analysis: Vector DB (372L), Graph DB (392L), Arquitectura (306L)
> - Competitive consolidation: 172 features de 27 documentos OLD
> - Cross-ref wave 3: `docs/audit-reports/cross-ref-wave3-final-report.md`
> - Backlog OLD rescue: `docs/REPORTE_EVALUACION_COMPLETO.md` (280+ archivos)

---

## Índice

1. [Estado Actual](#1-estado-actual)
2. [Riesgos Bloqueantes](#2-riesgos-bloqueantes)
3. [Fases de Ejecución](#3-fases-de-ejecución)
   - [Fase 0 — Estabilizar y Publicar (Sem 1-2)](#fase-0--estabilizar-y-publicar-sem-1-2)
   - [Fase 1 — Competitividad Core (Sem 3-5)](#fase-1--competitividad-core-sem-3-5)
   - [Fase 2 — Features de Release (Sem 6-8)](#fase-2--features-de-release-sem-6-8)
   - [Fase 3 — Diferenciación Graph+Vector (Sem 9-12)](#fase-3--diferenciación-graphvector-sem-9-12)
   - [Fase 4 — Ecosystem & Madurez (Sem 13-16)](#fase-4--ecosystem--madurez-sem-13-16)
4. [Mapa de Dependencias](#4-mapa-de-dependencias)
5. [Items del Backlog por Fase](#5-items-del-backlog-por-fase)

---

## 1. Estado Actual

### Métricas del proyecto

| Dimensión | Valor |
|-----------|-------|
| **Código** | ~42.500 LOC (32.440 Rust core + 6.100 WASM + 2.000 Python + 2.000 adapters) |
| **Commits totales** | 1.075 |
| **Commits desde Jul 1** | 537 (~33/día) |
| **Core Rust files** | 116 |
| **Tests** | 444 Rust + tests de adapters |
| **PyPI adapters** | 9 (LangChain, LlamaIndex, Haystack, Mem0, Letta, CrewAI, DSPy, OpenAI, LiteLLM) |
| **npm package** | `vantadb` + `vantadb-wasm` (80/219 WASM tests fallan pre-existing) |

### Backlog

| Categoría | Items | Estado |
|-----------|-------|--------|
| TIER 0 🔴 Bloqueantes de Release | ~5 | Mayoría ⏳ |
| TIER 1 🟠 Pre-Lanzamiento | ~15 | Mayoría ❌ |
| TIER 2 🔵 ACID/Persistencia | ~8 | Mayoría ❌ |
| TIER 3 🟡 Optimizaciones | ~25 | Mixto |
| DRV Items SDK/Adapter/Engine | ~20 | Varios ✅, resto ❌ |
| Hallazgos (VFY, SEC, WEB, TEST, DOC) | ~12 | Mixto |
| COMP-001→030 (Competitive Features) | 30 (7🔴+17🟠+6🟡) | Todos ❌ |
| **Total abiertos** | **165** | — |

### Estado vs Competidores

| Feature | VantaDB | Qdrant | Milvus | Chroma | Pinecone |
|---------|---------|--------|--------|--------|----------|
| HNSW | ✅ | ✅ | ✅ | ✅ | ❌ propio |
| Cuantización (SQ8/PQ) | ❌ SQ8 sin exponer | ✅ SQ8+PQ | ✅ SQ8+PQ | ❌ | ✅ PQ |
| In-filter filtering | ❌ | ✅ | ✅ | ❌ | ✅ |
| HNSW Persistencia | ❌ rebuild cada startup | ✅ segmentos | ✅ LSM | N/A | N/A |
| Graph+Vector híbrido | ✅ parcial | ❌ | ❌ | ❌ | ❌ |
| Auto-embedding | ❌ (DRV-123) | ❌ | ❌ | ✅ | ❌ |
| Node.js bindings nativos | ❌ WASM limitado | ✅ gRPC | ✅ gRPC | ✅ HTTP | ✅ gRPC |
| CRUD en HNSW | ❌ rebuild completo | ✅ | ✅ growing/sealed | ✅ simple | N/A |

---

## 2. Riesgos Bloqueantes

Estos riesgos **no tienen item dedicado en el backlog** o están subestimados. Son condiciones necesarias para cualquier release.

| # | Riesgo | Impacto | Mitigación | Tracking |
|---|--------|---------|------------|----------|
| **R1** | **CI certification inestable** — 20+ commits de fix (ASan, TSan, coverage thresholds) en 2 días. Pipeline certifica falso-negativos | 🔴 Bloquea todo release | Dedicar semana 1 a CI hardening con métricas de estabilidad | ❌ No hay item. Agregar como bloqueante |
| **R2** | **WASM demo placeholder** — 80/219 tests fallan en Node.js. `/demo` sin build funcional | 🔴 Bloquea Show HN | Fijar demo WASM antes de cualquier marketing | MKT-13 (parcial) |
| **R3** | **bincode deprecated** — Crate no mantenido desde 2021. Toda serialización del engine depende de él | 🟠 Migración forzosa eventual | Evaluar rkyv (ya existe para archive) como reemplazo | SEC-14 |
| **R4** | **DRV-115: MSVC linker overflow** — No se puede build workspace completo en Windows con MSVC | 🟠 Bloquea Windows build en CI | Excluir adaptadores PyO3 de workspace build o usar rust-lld | DRV-115 |
| **R5** | **165 items abiertos, persona-equipo 1-2** — Sin priorización estricta, el backlog es months | 🔴 Parálisis por analysis-paralysis | Congelar nuevos items hasta reducir a ≤100. No agregar COMP-031+ | ❌ No hay item |
| **R6** | **SQ8 no expuesto en query path** — Existe como `VectorRepresentations::SQ8` pero el hot path de búsqueda solo usa f32 full precision. SIFT 1M tarda 127s | 🔴 Benchmarks no competitivos vs Qdrant/Milvus | Exponer SQ8 en `distance.rs` hot path (sem 3) | COMP-001 |
| **R7** | **HNSW rebuild en cada startup** — 30-60s para 1M vectores. Impide uso en serverless/edge | 🟠 Cold start inaceptable para AI agents | Serializar neighbor lists con bincode + load condicional (sem 4) | COMP-002 |
| **R8** | **Claims falsos en landing** — "50x" vs 40x real, "SQL support" sin implementar, "auto-embeddings" sin feature, "cloud tiers" sin infra | 🟠 Riesgo reputacional en Show HN | Corregir WEB-02 antes de cualquier campaign | WEB-02 |

### Resolución de riesgos por fase

```
R1 (CI)     ─── Fase 0
R2 (WASM)   ─── Fase 0
R3 (bincode)─── Fase 1 (evaluación)
R4 (MSVC)   ─── Fase 0
R5 (backlog)─── Fase 0 (freeze + triage)
R6 (SQ8)    ─── Fase 1 (COMP-001)
R7 (rebuild)─── Fase 2 (COMP-002)
R8 (claims) ─── Fase 0 (WEB-02)
```

---

## 3. Fases de Ejecución

### Fase 0 — Estabilizar y Publicar (Sem 1-2)

> **Objetivo:** Pipeline CI estable, adapters publicados, demo WASM funcional, claims corregidos.
> **Riesgos resueltos:** R1, R2, R4, R5, R8

#### Semana 1 — CI Hardening + Freeze

| Orden | Item | Descripción | Esfuerzo | Dependencias |
|-------|------|-------------|----------|-------------|
| 1 | **R1 (nuevo)** | CI certification: estabilizar ASan/TSan/coverage thresholds. Agregar métrica de "builds verdes consecutivos" como gate | 🟡 2-3d | — |
| 2 | **R5 (nuevo)** | Freeze backlog: congelar nuevos items. Hacer triage de 165 → ≤100 items. Mover diferidos a `docs/archive/backlog-futuro.md` | 🟢 1d | — |
| 3 | **DRV-115** | Fix MSVC linker overflow: excluir adapters PyO3 de workspace build o usar rust-lld | 🟡 4h | — |
| 4 | **DRV-116** | 10 warnings: `unnecessary unsafe` (9) + dead code (4) en vfile.rs, graph.rs, serialize.rs, archive.rs, maintenance.rs | 🟢 30min | — |
| 5 | **DRV-117** | Stale advisory ignores: limpiar RUSTSEC-2024-0436 y RUSTSEC-2025-0134 de deny.toml | 🟢 5min | — |
| 6 | **DRV-118** | Windows CI: agregar Windows a release matrix (blocker para adopción Windows) | 🟡 1d | 3 (DRV-115) |
| 7 | **WEB-02** | Corregir claims falsos en landing: benchmarks (50x→40x), SQL support, auto-embeddings, cloud tiers | 🟡 2-3d | — |

#### Semana 2 — Publicar + Demo

| Orden | Item | Descripción | Esfuerzo | Dependencias |
|-------|------|-------------|----------|-------------|
| 8 | **SEC-13** | CSP unsafe-inline en prod + HSTS + nonce system | 🟡 1-2d | — |
| 9 | **DEVOPS-13** | Pin actions a SHA + Node 22 (✅ listo, verificar) | 🟢 1h | — |
| 10 | **DEVOPS-14** | Extraer composite action para Rust setup (5+ workflows duplicados) | 🟢 4h | — |
| 11 | **DEVOPS-15** | Mover features heavies fuera de default + consolidar deps | 🟡 1-2d | — |
| 12 | **MKT-13** | WASM demo funcional: link hero→/demo fix + build WASM funcional | 🟡 1-2d | — |
| 13 | **INT-01 + INT-02 + DEVOPS-05** | Publicar adapters LangChain + LlamaIndex + pipeline CI a PyPI | 🟡 1-2d | — |
| 14 | **REL-02** | Publicar `vantadb-ts` en npm (WASM build) | 🟡 1-2d | 12 (WASM demo) |
| 15 | **REL-01** | Bump workspace v0.1.5 → v0.2.0 | 🟢 1h | 13, 14 |
| 16 | **DEVOPS-12** | Production PyPI signing pipeline (OIDC + Sigstore) | 🟡 1-2d | 13 |

---

### Fase 1 — Competitividad Core (Sem 3-5)

> **Objetivo:** Cerrar gaps críticos vs Qdrant/Milvus en filtering y metadata indexing.
> **Riesgos resueltos:** R3 (evaluación), R6 (SQ8 en query path)
> **Items del backlog:** COMP-003, 004, 005, 007, 012, 006 + SEC-14 + VFY-004, 006, 007

#### Semana 3 — Filtering Foundation

| Orden | Item | Descripción | Esfuerzo | Dependencias |
|-------|------|-------------|----------|-------------|
| 17 | **COMP-012** | RoaringBitmaps metadata index (`croaring` crate). Construir bitsets por valor de metadata. Base para COMP-003 | 🟡 1 sem | — |
| 18 | **COMP-005** | HNSW params configurables (M, ef_construction, ef_search). Extender `HnswConfig` + exponer en API | 🟢 2-3d | — |
| 19 | **COMP-003** | In-filter traversal: intersectar `FilterBitset` durante HNSW walk en `graph.rs:search_nearest()`. ~50 líneas | 🟢 3-5d | 17 (COMP-012) |
| 20 | **SEC-14** | Evaluar migración bincode → postcard/rkyv. Si es viable, hacerla ahora antes de COMP-002 (serialización HNSW) | 🟡 1d | — |

#### Semana 4 — Bitset + Performance

| Orden | Item | Descripción | Esfuerzo | Dependencias |
|-------|------|-------------|----------|-------------|
| 21 | **COMP-004** | Bitset-based filtering + soft deletes con RoaringBitmaps. Mask de deleted nodes + compact periódico | 🟢 3-5d | 17 (COMP-012) |
| 22 | **COMP-007** | Bitset inline u128 en UnifiedNode. Reemplazar `FilterBitset` (Vec<u64>) por u128 inline. -24-56 bytes/nodo | 🟡 1 sem | — |
| 23 | **COMP-006** | Edge Label Interning: `Edge.label: String` → `Edge.label_id: u32`. -80MB para 1M nodos | 🟢 2d | — |
| 24 | **VFY-004** | `flat.rs` O(n²) en filter: agregar índice para filtros | 🟡 1-2d | 21 |
| 25 | **VFY-006** | `add_node` escribe lock durante toda inserción: reducir granularidad | 🟡 1-2d | — |

#### Semana 5 — Consolidación

| Orden | Item | Descripción | Esfuerzo | Dependencias |
|-------|------|-------------|----------|-------------|
| 26 | **VFY-007** | `remove_node` O(n²) neighbor fixup: optimizar deletes | 🟡 1-2d | 21 (soft deletes) |
| 27 | **VFY-008** | WAL fsync por escritura: batching fsyncs | 🟡 1-2d | — |
| 28 | **WEB-03** | Async WAL batching fsyncs (PERFORMANCE_TUNING.md) | 🟡 2-3d | — |
| 29 | **DRV-001** | `search.rs:1085L` god file: split BM25 scoring, snippet gen, hybrid fusion, RRF | 🟡 2-3d | — |
| 30 | **DRV-002** | `put_batch` duplica `put()`: DRY refactor | 🟢 1d | — |
| 31 | **DRV-003** | `purge_expired` O(n) index rebuilds | 🟢 2h | — |

---

### Fase 2 — Features de Release (Sem 6-8)

> **Objetivo:** Cuantización, persistencia, CRUD en HNSW. Performance competitiva en benchmarks.
> **Riesgos resueltos:** R7 (rebuild startup)
> **Items del backlog:** COMP-001, 002, 009, 010, 011, 014, 020

#### Semana 6 — Cuantización

| Orden | Item | Descripción | Esfuerzo | Dependencias |
|-------|------|-------------|----------|-------------|
| 32 | **COMP-001** | SQ8 quantization en query path: exponer `VectorRepresentations::SQ8` en `distance.rs` hot path + HNSW insert/search | 🟡 1-2 sem | 20 (bincode decision) |
| 33 | **COMP-010** | Auto-embedding function abstraction: trait `EmbeddingProvider` + OpenAI/Ollama providers | 🟡 1-2 sem | DRV-123 (existe) |

#### Semana 7 — Persistencia + CRUD

| Orden | Item | Descripción | Esfuerzo | Dependencias |
|-------|------|-------------|----------|-------------|
| 34 | **COMP-002** | HNSW Persistence: serializar neighbor lists con bincode. Load condicional. Eliminar rebuild en startup | 🟡 1-2 sem | 20 (bincode/rkyv) |
| 35 | **COMP-011** | HNSW CRUD con tombstones: custom HNSW con updates/deletes sin rebuild completo | 🟡 2-3 sem | 21 (COMP-004 soft deletes) |
| 36 | **COMP-014** | FreshHNSW: background repair de enlaces huérfanos por borrados masivos | 🟡 1 sem | 35 |

#### Semana 8 — Búsqueda Híbrida

| Orden | Item | Descripción | Esfuerzo | Dependencias |
|-------|------|-------------|----------|-------------|
| 37 | **COMP-020** | Hybrid search con RRF: unificar BM25 + vector search con Reciprocal Rank Fusion | 🟡 1 sem | — |
| 38 | **COMP-009** | Binary bulk import: formato binario FlatBuffer/bincode para import masivo. 5-10x INSERT | 🟢 3-4d | — |
| 39 | **COMP-024** | ACORN algorithm: second-hop exploration para filtered search de alta selectividad | 🟡 1-2 sem | 19 (COMP-003) |

---

### Fase 3 — Diferenciación Graph+Vector (Sem 9-12)

> **Objetivo:** Activar el diferenciador único de VantaDB: graph+vector nativo.
> **Items del backlog:** COMP-015, 016, 017, 018, 022, 028, 029

#### Semana 9-10 — Graph Pipeline

| Orden | Item | Descripción | Esfuerzo | Dependencias |
|-------|------|-------------|----------|-------------|
| 40 | **COMP-016** | Supernode mitigation: HashMap<label_id, Vec<VantaEdgeRecord>> para nodos con millones de edges | 🟢 3-5d | 23 (COMP-006) |
| 41 | **COMP-018** | Double-linked relationship chains: AdjacencyList con array contiguo, navegación O(k) | 🟡 1-2 sem | 23 (COMP-006) |
| 42 | **COMP-017** | Accumulators para graph algorithms: AtomicU64/fetch_add para PageRank paralelo | 🟡 1-2 sem | — |
| 43 | **COMP-015** | Hybrid Graph+Vector search: vector search → graph traversal en misma query | 🟡 2-3 sem | 18 (HNSW params), 19 (COMP-003) |

#### Semana 11-12 — GDS + Bindings

| Orden | Item | Descripción | Esfuerzo | Dependencias |
|-------|------|-------------|----------|-------------|
| 44 | **COMP-022** | Graph Data Science: PageRank nativo en Rust (primero), Louvain después | 🟡 2-3 sem | 42 (accumulators) |
| 45 | **COMP-028** | Semantic Cost Estimator: metadata collector + cost model + planner integration | 🟡 2 sem | DRV-121/122 (planner) |
| 46 | **COMP-029** | Node.js/TS bindings via napi-rs: bindings nativos para ecosistema JS | 🟡 2-3 sem | — |
| 47 | **COMP-025** | JSON shredding: dynamic JSON → typed columns, SQL-like filtering | 🟡 2-3 sem | — |

---

### Fase 4 — Ecosystem & Madurez (Sem 13-16)

> **Objetivo:** Production readiness, storage optimization, survival mode.
> **Items del backlog:** COMP-008, 013, 019, 021, 023, 026, 027, 030 + ACID items

#### Semana 13-14 — Production Hardening

| Orden | Item | Descripción | Esfuerzo | Dependencias |
|-------|------|-------------|----------|-------------|
| 48 | **COMP-030** | Survival Mode: backpressure + Docker OOM prevention. Integrar memory_governor con cgroups | 🟡 1-2 sem | — |
| 49 | **COMP-019** | Binary protocol (rkyv/FlatBuffers): reemplazar JSON por binario zero-copy | 🟡 1-2 sem | 20 (rkyv) |
| 50 | **COMP-013** | Segment optimizer: Vacuum/Merge/Index optimizadores background | 🟡 1-2 sem | 35 (tombstones) |
| 51 | **COMP-026** | Multi-level LSM compaction: L0→L1→L2→L3, spread compaction cost | 🟡 1-2 sem | 50 |

#### Semana 15-16 — Advanced Features

| Orden | Item | Descripción | Esfuerzo | Dependencias |
|-------|------|-------------|----------|-------------|
| 52 | **COMP-008** | VecIndex trait: abstraer index operations para múltiples backends | 🟡 1-2 sem | — |
| 53 | **COMP-027** | Multiple index types: IVF, DiskANN, SCANN además de HNSW | 🟠 5-10d | 52 |
| 54 | **COMP-021** | Temporal edges: timestamp en edges para time-travel queries | 🟡 1 sem | — |
| 55 | **COMP-023** | 3 filtering strategies (pre/post/in-index) con optimizador por selectividad | 🟡 1-2 sem | 19 (COMP-003), 17 (COMP-012), 45 (SCE) |
| 56 | **DRV-119→122** | ACID Phase 0-3: WAL/VantaFile/HNSW/KV coordination, HNSW multi-layer, Planner AST, IQL JOINs | 🟠 3-10d c/u | — |

---

## 4. Mapa de Dependencias

```
Sem 1-2: FASE 0 — ESTABILIZAR
┌─────────────────────────────────────────────────────┐
│ R1 (CI) ─── DRV-115 (MSVC) ─── DRV-118 (Win CI)    │
│ R5 (freeze backlog)                                 │
│ MKT-13 (WASM demo) ─── REL-02 (npm publish)        │
│ INT-01/02 + DEVOPS-05 (adapters PyPI)               │
│ WEB-02 (claims falsos)                              │
└──────────────────────┬──────────────────────────────┘
                       │
Sem 3-5: FASE 1 — COMPETITIVIDAD CORE
┌──────────────────────┴──────────────────────────────┐
│ COMP-012 (RoaringBitmaps) ──┬── COMP-003 (in-filter)│
│ SEC-14 (bincode eval) ──┬──┐                        │
│                          │  COMP-005 (HNSW params)  │
│ COMP-004 (soft deletes) ─┤                          │
│ COMP-007 (bitset u128)   │                          │
│ COMP-006 (label interning)│                         │
│ VFY-004/006/007          │                          │
└──────────────────────┬───┴──────────────────────────┘
                       │
Sem 6-8: FASE 2 — RELEASE FEATURES
┌──────────────────────┴──────────────────────────────┐
│ COMP-001 (SQ8/PQ) ─── depende de SEC-14             │
│ COMP-010 (auto-embedding)                            │
│ COMP-002 (HNSW persistence) ─── depende de SEC-14   │
│ COMP-011 (HNSW CRUD) ─── depende de COMP-004        │
│ COMP-014 (FreshHNSW) ─── depende de COMP-011        │
│ COMP-020 (RRF hybrid)                                │
│ COMP-009 (binary import)                             │
│ COMP-024 (ACORN) ─── depende de COMP-003            │
└──────────────────────┬──────────────────────────────┘
                       │
Sem 9-12: FASE 3 — GRAPH+VECTOR
┌──────────────────────┴──────────────────────────────┐
│ COMP-016 (supernode) ─── depende de COMP-006        │
│ COMP-018 (double-linked chains) ─── COMP-006        │
│ COMP-017 (accumulators)                              │
│ COMP-015 (hybrid graph+vector) ─── COMP-003, 005   │
│ COMP-022 (GDS) ─── depende de COMP-017              │
│ COMP-028 (SCE) ─── depende de DRV-121/122           │
│ COMP-029 (napi-rs)                                   │
│ COMP-025 (JSON shredding)                            │
└──────────────────────┬──────────────────────────────┘
                       │
Sem 13-16: FASE 4 — MADUREZ
┌──────────────────────┴──────────────────────────────┐
│ COMP-030 (Survival Mode)                             │
│ COMP-019 (binary protocol) ─── SEC-14 (rkyv)        │
│ COMP-013 (segment optimizer) ─── COMP-004/011       │
│ COMP-026 (LSM compaction) ─── COMP-013              │
│ COMP-008 (VecIndex trait)                            │
│ COMP-027 (multiple index types) ─── COMP-008        │
│ COMP-021 (temporal edges)                            │
│ COMP-023 (3 filtering strategies) ─── COMP-003/012/028│
│ DRV-119→122 (ACID)                                   │
└─────────────────────────────────────────────────────┘
```

---

## 5. Items del Backlog por Fase

### Fase 0 (Sem 1-2) — 16 items

| Item | Descripción | Prioridad |
|------|-------------|-----------|
| R1 (nuevo) | CI certification: estabilizar ASan/TSan/coverage | 🔴 |
| R5 (nuevo) | Freeze + triage backlog 165→100 | 🔴 |
| DRV-115 | MSVC linker overflow fix | 🔴 |
| DRV-116 | 10 warnings de compilación | 🟢 |
| DRV-117 | Stale advisory ignores | 🟢 |
| DRV-118 | Windows CI release matrix | 🔴 |
| WEB-02 | Corregir claims falsos landing | 🔴 |
| SEC-13 | CSP + HSTS + nonce system | 🔴 |
| DEVOPS-13 | Pin actions a SHA + Node 22 | 🟡 |
| DEVOPS-14 | Composite action para Rust setup | 🟡 |
| DEVOPS-15 | Features heavies fuera de default | 🟡 |
| MKT-13 | WASM demo funcional | 🔴 |
| INT-01/02 + DEVOPS-05 | Publicar adapters PyPI | 🔴 |
| REL-02 | Publicar vantadb-ts npm | 🔴 |
| REL-01 | Bump v0.2.0 | 🟢 |
| DEVOPS-12 | PyPI signing pipeline | 🟡 |

### Fase 1 (Sem 3-5) — 15 items

| Item | Descripción | Prioridad |
|------|-------------|-----------|
| COMP-012 | RoaringBitmaps metadata index | 🟡 |
| COMP-005 | HNSW params configurables | 🟢 |
| COMP-003 | In-filter traversal (bitset HNSW walk) | 🟢 |
| SEC-14 | Evaluar migración bincode→rkyv | 🟡 |
| COMP-004 | Bitset soft deletes | 🟢 |
| COMP-007 | Bitset inline u128 | 🟡 |
| COMP-006 | Edge Label Interning | 🟢 |
| VFY-004 | flat.rs O(n²) filter | 🟡 |
| VFY-006 | add_node lock granularidad | 🟡 |
| VFY-007 | remove_node O(n²) fixup | 🟡 |
| VFY-008 | WAL fsync batching | 🟡 |
| WEB-03 | Async WAL batching | 🟡 |
| DRV-001 | search.rs god file split | 🟡 |
| DRV-002 | put_batch duplica put() | 🟢 |
| DRV-003 | purge_expired O(n) | 🟢 |

### Fase 2 (Sem 6-8) — 8 items

| Item | Descripción | Prioridad |
|------|-------------|-----------|
| COMP-001 | SQ8 quantization en query path | 🟡 |
| COMP-010 | Auto-embedding function | 🟡 |
| COMP-002 | HNSW Persistence (no rebuild) | 🟡 |
| COMP-011 | HNSW CRUD con tombstones | 🟡 |
| COMP-014 | FreshHNSW (background repair) | 🟡 |
| COMP-020 | Hybrid search con RRF | 🟡 |
| COMP-009 | Binary bulk import | 🟢 |
| COMP-024 | ACORN algorithm (second-hop) | 🟡 |

### Fase 3 (Sem 9-12) — 9 items

| Item | Descripción | Prioridad |
|------|-------------|-----------|
| COMP-016 | Supernode mitigation | 🟢 |
| COMP-018 | Double-linked chains | 🟡 |
| COMP-017 | Accumulators | 🟡 |
| COMP-015 | Hybrid Graph+Vector pipeline | 🟡 |
| COMP-022 | GDS library (PageRank) | 🟡 |
| COMP-028 | Semantic Cost Estimator | 🟡 |
| COMP-029 | Node.js/TS bindings napi-rs | 🟡 |
| COMP-025 | JSON shredding | 🟡 |

### Fase 4 (Sem 13-16) — 10 items

| Item | Descripción | Prioridad |
|------|-------------|-----------|
| COMP-030 | Survival Mode (OOM prevention) | 🟡 |
| COMP-019 | Binary protocol (rkyv) | 🟡 |
| COMP-013 | Segment optimizer pipeline | 🟡 |
| COMP-026 | Multi-level LSM compaction | 🟡 |
| COMP-008 | VecIndex trait | 🟡 |
| COMP-027 | Multiple index types | 🟠 |
| COMP-021 | Temporal edges | 🟡 |
| COMP-023 | 3 filtering strategies | 🟡 |
| DRV-119→122 | ACID Phases 0-3 | 🟠 |

### Items sin asignación de fase (backlog general)

| Item | Prioridad | Nota |
|------|-----------|------|
| OLD-001→022 (22 items) | Varias | Mover a `archive/` tras triage R5. Mayoría son futuribles sin implementación |
| TEST-11 | 🟡 | Frontend tests (Vitest + Playwright) — postergar post-Show HN |
| TEST-12 | 🟡 | Security testing fuzzing — postergar post-Show HN |
| DOC-20 | 🟡 | mdBook adoption — postergar |
| MKT-17 | 🟢 | Comparación competitiva — post-Show HN |
| LEG-01 | 🔴 | Trademark — iniciar ahora (proceso legal lento) |
| MKT-03→05 | 🔴🟠 | Show HN + Reddit + Blog posts — Fase 0/1 |

---

## 6. Recomendaciones Inmediatas (Checklist)

- [ ] **Hoy:** Freeze backlog. No agregar COMP-031+ ni nuevos items hasta reducir a ≤100
- [ ] **Sem 1:** Arreglar CI (R1) — es la tarea más importante, todo lo demás depende
- [ ] **Sem 1:** Publicar adapters a PyPI — están listos, solo falta pipeline CI
- [ ] **Sem 2:** Tener demo WASM funcional en `/demo`
- [ ] **Sem 2:** Corregir WEB-02 (claims falsos) antes de cualquier marketing
- [ ] **Sem 3-5:** In-filter filtering (COMP-003, ~50 líneas) + HNSW params (COMP-005, ~2d) son los items de mayor impacto por menor esfuerzo
- [ ] **Sem 6:** SQ8 quantization (COMP-001) + HNSW persistence (COMP-002) son habilitadores de benchmarks competitivos
- [ ] **Sem 8:** Decidir si Show HN se hace en Sem 4 (con Fase 0-1 completo) o se espera a Sem 8 (con Fase 2)
- [ ] **Continuo:** No abrir issues nuevos sin milestone asignado en este roadmap

---

> **Próxima revisión:** 2026-07-23 o al completar Fase 0, lo que ocurra primero.
> **Ver también:** [`docs/Backlog.md`](../Backlog.md) para detalle de cada item, [`docs/strategy/ACTION_PLAN.md`](ACTION_PLAN.md) (plan de acción v1.0, Jul 3 — supercedido por este documento).
