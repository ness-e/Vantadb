---
title: "Active Backlog — VantaDB"
type: backlog-tracking
status: active
tags: [vantadb, backlog, engineering, phases, priorities]
last_reviewed: 2026-07-26
verified_by: "6 sub-agentes: P0+P1 (vanta-lead), P2 (vanta-worker), P3+P7 (general), P4+P8 (general), P5+P6 (vanta-docs), P9+P10 (vanta-worker)"
---

# Active Backlog — VantaDB

> **Purpose:** Single source of truth for all project tasks — organized by execution order.
> **Completed tasks moved to:** `docs/progreso/README.md`
> **Verification method:** All items cross-checked against actual codebase (Jul 26, 2026). 167 items verified across 11 phases. Completed items moved out of tables into progreso. P9/P10 statuses reflect real implementation state.
> **Total open items:** ~65
> **Origen docs-audit:** `docs/strategy/ROADMAP.md`, `docs/progreso/bitacora.md`, `docs/reviews/FULL_CODEBASE_AUDIT_2026-07-11.md`, `docs/reviews/analisis_proyecto.md`, `docs/operations/PERFORMANCE_TUNING.md`, `docs/operations/REPO_CHECKLIST.md`, `docs/architecture/STORAGE_VERSIONING.md`, `docs/plans/2026-07-13-workflow-repair-campaign.md`, `docs/Investigaciones/cargo-check-optimizacion.md`, `docs/discord/todo.md`

---

## Exec Summary

| Phase | Items | Est. Effort | Priority |
|-------|-------|-------------|----------|
| **P0** 🚀 Release Blockers | 1 (+6 ✅ completados, +6 removidos) | ~2-3d | 🔴 Bloqueante |
| **P1** 🛡️ Security & Critical | 0 (todos resueltos/deferidos) | — | ✅ Cerrado |
| **P2** ⚡ Quick Wins Técnicos | 0 (7 ✅ + 24 stale removidos) | — | ✅ Cerrado |
| **P3** 🧪 Test Coverage (adapters) | 0 (7 ✅ + 7 stale removidos) | — | ✅ Cerrado |
| **P4** 🔧 Engineering Health | 0 (10 ✅ removidos a progreso) | — | ✅ Cerrado |
| **P5** 📖 Docs & Community | 11 | ~1-2 semanas | 🟡 Media |
| **P6** 🚀 Launch Campaign | 10 | ~1-2 semanas | 🟡 Media |
| **P7** 🌐 WASM & Performance | 1 (NUEVO-14) | ~1 semana | 🟡 Media |
| **P8** 🔮 Post-Launch & Enterprise | 8 | ~3-5 semanas | 🔵 Futuro |
| **P9** 📚 Old Docs Rescue (reference) | 13 (7 ✅ progreso) | — | 📖 Referencia |
| **P10** 🏗️ Competitive Features (catalog) | 20 (10 ✅ progreso) | — | 🗺️ Roadmap |

> **Items removidos (70+):** ~25 originales + 6 P0 stale + 9 P1 resueltos + 24 P2 stale + 7 P3 stale + 10 P4 completados + 7 P9 completados + 10 P10 completados + 1 P7 completado + 24 crates de integración nunca implementados

---

## ✅ Definition of Ready (DoR)

- [ ] ID único asignado
- [ ] Prioridad definida (🔴🟠🟡🟢🔵⬜)
- [ ] Archivos involucrados conocidos
- [ ] Esfuerzo estimado
- [ ] Verificado contra código real (no asumido)

## ✅ Definition of Done (DoD)

- [ ] Código compila (`cargo check` / `tsc --noEmit`)
- [ ] Tests pasan (`cargo test` / `vitest run`)
- [ ] Linters pasan (`cargo clippy` / `eslint`)
- [ ] Docs actualizados si aplica
- [ ] Tarea movida a `progreso/README.md`
- [ ] Changelog actualizado si es cambio visible al usuario

---

## Phase 0: 🚀 Release Blockers

> Items que bloquean un release público seguro. Resolver antes de cualquier publicación.

| ID | Descripción | Archivos | Esfuerzo | Prio |
|----|-------------|----------|----------|------|
| `DEVOPS-15` | **Reducir default features de 7 a 3** — Código actual tiene 7 (`cli, arrow, fjall, advanced-tokenizer, memmap2, fs2, sysinfo`). La tarea original decía 9→3, pero solo `prometheus`/`rayon` fueron removidos. Pendiente: `cli, memmap2, fs2, sysinfo`. | `Cargo.toml:89` | 🟡 1d | 🔴 Bloqueante |

> **Items removidos (6):** DEVOPS-10 (deferido, no bloqueante), DEVOPS-12 (PyPI signing), DEVOPS-14 ✅, NUEVO-09 ✅, NUEVO-10 ✅, DEVOPS-15 re-opened tras verificación (código tiene 7 defaults, no 3).

---

> **Phase 2: ⚡ Quick Wins Técnicos** — **31 items removidos:** DRV-014 ✅, DRV-028 ✅, DRV-041 ✅, VFY-006 ✅, VFY-007 ✅, REV-012 ✅, DRV-136 ✅ + 24 stale items de la auditoría original. No quedan items activos en P2.

---

> **Phase 3: 🧪 Test Coverage (Adapters & Engine)** — **14 items removidos:** DRV-013 ✅, DRV-017 ✅, DRV-061 ✅, DRV-067 ✅, DRV-073 ✅, TEST-11 ✅, TEST-12 ✅ + 7 stale de auditoría original. No quedan items activos en P3.

---

> **Phase 4: 🔧 Engineering Health & Architecture** — **10 items removidos:** WEB-03 ✅, WEB-04 ✅, VFY-004 ✅, VFY-011 ✅, DRV-121 ✅, DRV-122 ✅, DRV-123 ✅, DRV-130 ✅, DRV-131 ✅, DOC-20 ✅. No quedan items activos en P4.

---

## Phase 5: 📖 Docs & Community

> Preparación de documentación pública, comunidad, y onboarding.

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado Real |
|----|-------------|----------|----------|------|-------------|
| `MKT-14` | **Publicar 2 case studies** + ruta `/case-studies/` | `docs/case_studies/` drafts + ruta web montada | 🟡 1-2d | 🔴 | ⚠️ Drafts + ruta OK, falta pulir |
| `TSK-106` | **Habilitar GitHub Discussions** | — | 🟢 1h | 🟠 | ❌ Desde cero |
| `NUEVO-01` | **README hero** con readme-aura + benchmark gráfico + GIF demo WASM | `README.md` (PNG estática actual) | 🟡 2-3d | 🟠 | ❌ Desde cero |
| `NUEVO-07` | **Migration tools: Chroma→Vanta, LanceDB→Vanta** | `docs/tutorials/` (guías existen), `src/migration.rs` (formatos internos) | 🟡 3-5d | 🟠 | ⚠️ Tutoriales OK, scripts ejecutables faltan |
| `NUEVO-08` | **Learning path estructurado** en tutorials/ (5-7 ejemplos) | 4 tutoriales (2 draft, 1 active, 1 migration) | 🟡 2-3d | 🟠 | ⚠️ 4/7, algunos draft |
| `NUEVO-10` | **Benchmark suite pública reproducible** | Benchmarks internos existen, sin script público standalone | 🟡 3-5d | 🟠 | ⚠️ Benchmarks OK, reproducibilidad no |
| `TSK-107` | Community showcase page | Ruta web montada, probablemente vacía de proyectos reales | 🟢 4-6h | 🟡 | ⚠️ Página existe, sin data |
| `—` | Good first issues (20+ tagged) | GitHub Issues + drafts en `PUBLIC_ISSUE_DRAFTS.md` | 🟢 2-4h | 🟠 | 🎯 Estratégico (no verificable local) |
| `COM-02` | **Configurar Discord: reaction roles, autorole, logging, welcome DM, onboarding** | `docs/discord/todo.md` + assets SVG + server activo | 🟡 2-3d | 🟢 | ⚠️ Docs + assets OK. Config pendiente |
| `COM-03` | **Discord: AutoMod, stickers/emojis, forums seed** | — | 🟢 4-6h | 🟢 | ❌ Documentado, sin implementar |
| `COM-04` | **Discord: ticketing system, stage channel, Server Discovery, Canny.io** | — | 🟢 4-6h | 🟢 | ❌ Documentado, sin implementar |

---

## Phase 6: 🚀 Launch Campaign

> Todo lo necesario para el Show HN y marketing de lanzamiento.

| ID | Descripción | Esfuerzo | Prio | Estado Real |
|----|-------------|----------|------|-------------|
| `LEG-01` | **Registrar trademark "VantaDB" (USPTO + EUIPO)** | 🟡 2-4h | 🔴 | ❌ Desde cero |
| `MKT-03` | **Show HN post** | 🟢 2h | 🔴 | ⚠️ Draft 184L en `docs/strategy/SHOW_HN_PREP.md` |
| `MKT-04` | Reddit posts (r/rust, r/MachineLearning, r/LocalLLaMA) | 🟢 2-4h | 🟠 | ❌ Desde cero |
| `MKT-05` | Technical blog posts (5+ pre-launch) — 4/5 posts escritos | 🟡 2-3d | 🟠 | ⚠️ 4/5 completados |
| `MKT-10` | "AI Agent Memory" campaign | 🟡 2-3d | 🟠 | ❌ Desde cero |
| `MKT-15` | **Página de benchmarks competitivos** (`/product/benchmarks`) — Ruta existe, sin comparación competitiva explícita | 🟡 2-3d | 🔴 | ⚠️ Página OK, contenido competitivo no |
| `MKT-16` | **Publicar metodología de benchmark GraphRAG** — Sin doc específico | 🟡 1-2d | 🟡 | ❌ Desde cero |
| `MKT-17` | Página de comparación competitiva interactiva — Sin ruta `/compare` ni archivos | 🟡 2-3d | 🟢 | ❌ Desde cero |
| `TSK-103` | Public benchmark site | 🟡 2-3d | 🟠 | ⚠️ `/product/benchmarks` existe |
| `TSK-104` | Demo agent: LangChain + Ollama + VantaDB — Ejemplo experimental existe | 🟡 1-2d | 🟠 | ⚠️ Ejemplo OK, no demo pulido |

---

## Phase 7: 🌐 WASM & Performance

| ID | Descripción | Archivos | Esfuerzo | Prio |
|----|-------------|----------|----------|------|
| `NUEVO-14` | **WASM bundle size <500KB gzip** — Sin medición de bundle actual ni flags de optimización en Cargo.toml más allá de `opt-level = "s"` | `vantadb-wasm/Cargo.toml` | 🟡 1-2d | 🟡 |

> **Items removidos (4):** NUEVO-11/12 (WASM IndexedDB + multi-tab coordinación — ✅ implementados), NUEVO-19 (SourceDesign/ no existe), BENCH-01 (solo mención en backlog, sin script ni dataset)

---

## Phase 8: 🔮 Post-Launch & Enterprise

> Features para después del lanzamiento público.

| ID | Descripción | Esfuerzo | Prio |
|----|-------------|----------|------|
| ~~`CLI-01`~~ | ~~**CLI polish: handlers backup/restore/doctor/stats/inspect existen pero no conectados al binary. REPL/TUI no existen**~~ — **✅ COMPLETADA** (5 handlers conectados: backup, restore, doctor, inspect, stats. 46 tests CLI pasan) | ~~🟡 2-3d~~ | ✅ |
| ~~`DEVOPS-HOMEBREW`~~ | ~~**Homebrew formula**~~ — **✅ COMPLETADA** (`Formula/vantadb.rb` existe con livecheck, 4 plataformas, install + test. Placeholder SHA256 — actualizar antes de publish.) | ~~🟢 4h~~ | ✅ |
| ~~`DEVOPS-PY313`~~ | ~~**Python 3.13 wheels en CI matrix**~~ — **✅ COMPLETADA** (`pyproject.toml` ya incluye `>=3.11` + classifier 3.13. CI verify jobs actualizados a Python 3.13 + build mantiene 3.11 con abi3) | ~~🟢 2h~~ | ✅ |
| ~~`DEVEX-DEMO`~~ | ~~**Demo app (Rust + Python)** — Phase 4.G~~ — **✅ COMPLETADA** (`examples/demo/demo.py` 239L con create → insert → search → delete, README, requirements.txt) | ~~🟡 2-3d~~ | ✅ |
| ~~`DEVEX-EXAMPLES`~~ | ~~**Rust examples en `examples/rust/`** (no `docs/examples/`)~~ — **✅ COMPLETADA** (4 ejemplos: basic, hybrid, graphrag, concurrent. Compilan clean.) | ~~🟢 4-6h~~ | ✅ |
| `NUEVO-16` | **Product Quantization (PQ) 96x** — compresión para datasets >RAM. RabitQ + TurboQuant + SQ8 existen, PQ real no | Alto | 🔵 |
| `NUEVO-17` | **Segment LSM-style** — hot/warm/cold tiers. Fjall tiene LSM interno, tiers no | Muy alto | 🔵 |
| `NUEVO-18` | **Sparse vectors nativos** — hybrid search real. Solo mención en test | Alto | 🔵 |
| `NUEVO-21` | **Vectara competitive research** | 🟢 2-4h | ⬜ |
| `TSK-107b` | Audit logging enterprise (JSONL, timestamp + op) | 🟡 2-3d | 🟡 |
| `ENT-04` | Connection pooling + circuit breaker (métrica existe, implementación no) | 🟡 2-3d | 🟡 |
| `BIZ-01` | **Enterprise features: encryption + RBAC ya en crate principal. Audit/replication/enterprise crate separado no existen** | 🟡 3-5d | 🟡 ⏳ |
| `WEB-001` | **Re-add interactive WASM demo page** — Tras publicar `@vantadb/wasm` | 🟢 30min | 🟡 |

> **Items removidos (1):** NUEVO-20 (Dockerfile ya existe en raíz del repo — multi-stage, Rust 1.94)

---

## Phase 9: 📚 Old Docs Rescue — Reference Catalog

> Recuperado de `VANTADB DOC OLD` (~280 archivos .md analizados vía 21 sub-agentes).
> **Total:** 21 items, **13 activos** (8 ✅ removidos a progreso). **Estado:** 7 ⚠️ parcial, 4 ❌ pendiente.
> **Referencia completa:** `docs/REPORTE_EVALUACION_COMPLETO.md` secciones 6 y 7.
> **Batch file map:** ver `docs/Backlog.md` sección Tier 5 original para archivos por batch.

### 🔴 Alta — Features perdidas con alto valor de mercado

| ID | Feature | Esfuerzo | Estado | Dependencias | Prioridad |
|----|---------|----------|--------|--------------|-----------|
> **8 items ✅ removidos a progreso:** OLD-04 (OpenTelemetry), OLD-07 (AutoHot/Cold tiering), OLD-13 (Explainable ranking), OLD-15 (Euclidean SIMD), OLD-16 (WAL rotation 256MB), OLD-17 (Migration guides), OLD-18 (TEMPERATURE param), OLD-22 (Arrow columnar export).

| ID | Feature | Esfuerzo | Estado | Dependencias |
|----|---------|----------|--------|--------------|
| `OLD-01` | **PGWire (PostgreSQL wire protocol)** — Compatibilidad con psql, pgAdmin, ecosistema PG | 🟠 2-3 sem | ❌ No implementado | Ninguna | 🗺️ Roadmap |
| ~~`OLD-02`~~ | ~~**GraphRAG pipeline formal** — seed → expand → retrieve → generate context. Ejemplo en `examples/rust/graphrag.rs`, no pipeline formal~~ | ~~🟡 1-2 sem~~ | ~~✅ COMPLETADA~~ | DRV-123 (auto-embedding) recomendado | 🗺️ Roadmap |
| `OLD-03` | **Chaos testing (Jepsen/Maelstrom)** — `chaos_test_wal.sh` + failpoint tests CI existen, no Jepsen formal | 🟡 2-3 sem | ⚠️ Parcial (scripts existen) | Docker. WAL shipping existente | 🗺️ Roadmap |
| `OLD-08` | Life Insurance / snapshots hard-link — `snapshot_certification.rs` existe, hard-link pattern no | 🟡 3-4d | ⚠️ Parcial | Ninguna. Solo syscalls POSIX |
| `OLD-09` | Olvido Bayesiano (hit decay) — `EvictionPolicy` con hit counts + recency weights, sin decay bayesiano formal | 🟡 3-4d | ⚠️ Parcial | Ninguna. `EvictionPolicy` existe |
| `OLD-10` | Sinapsis eléctrica (index-free adjacency) — `edge_index.rs` usa DashSet, no index-free adjacency nativa | 🟡 1 sem | ❌ No implementado | Post-HNSW multi-capa |
| `OLD-11` | CLI/TUI interactivo (spec 1106 líneas escrito) — CLI completo, TUI no implementado | 🟡 1-2 sem | ⚠️ Parcial (CLI OK, TUI no) | Ninguna. Proyecto aparte |
| `OLD-12` | Pilot program formal (early adopters) — `docs/operations/PILOT_PROGRAM.md` existe (solo spec) | 🟡 1 sem | ⚠️ Parcial (doc existe) | PyPI publicado |
| ~~`OLD-14`~~ | ~~MessageThread / GcWorker para agentic chat — `GcWorker` en `src/gc.rs` existe, MessageThread no~~ | ~~🟡 1 sem~~ | ~~✅ COMPLETADA~~ | Ninguna. `GcWorker` existe |
| ~~`OLD-16`~~ | ~~**WAL rotation a 256MB** — `WalWriter::try_auto_rotate()` en `append()`/`batch_append()`. 3 tests (trigger, no-trigger, data preservation). 52/52 WAL tests pass.~~ | ~~🟢 1d~~ | ~~✅ COMPLETADA~~ |
| ~~`OLD-19`~~ | ~~**Rehidratación desde shadow archive** — `VantaEmbedded::recover_archived_nodes()`, MCP tool `rehydrate`, Python binding. Conecta `StorageEngine::recover_archived_nodes()` (6 tests existentes) a SDK público.~~ | ~~🟡 1 sem~~ | ~~✅ COMPLETADA~~ | OLD-07 (AutoHot/Cold tiering) |
| `OLD-20` | Contextual Priming (cache warming predictivo) — Sin código de warming predictivo | 🟢 2-3d | ❌ No implementado | Ninguna |
| `OLD-21` | CP-Index formal (query routing inteligente) — `CPIndex` existe como struct HNSW, no query routing formal | 🟡 1 sem | ❌ No implementado | DRV-121/122 (Planner AST + IQL) |

---

## Phase 10: 🏗️ Competitive Features — Catalog

> **Fuente:** Análisis de 27 archivos de `VANTADB DOC OLD/` (9 vector DBs + 8 graph DBs + 10 arquitectura).
> **Total:** 30 items, **20 activos.** 10 ✅ implementados removidos a progreso: COMP-001 (SQ8/PQ), COMP-002 (HNSW persist), COMP-003 (in-filter), COMP-004 (bitset), COMP-005 (params), COMP-007 (inline u128), COMP-011 (CRUD tombstones), COMP-015 (hybrid pipeline), COMP-020 (RRF fusion), COMP-030 (survival mode).
> **Reportes completos:** `docs/audit-reports/competitive-features-consolidated-report.md`, `docs/audit-reports/deep-analysis-{vector,graph,arch}.md`

### 🔴 Alta — Features competitivas críticas para adopción

| ID | Feature | Esfuerzo | Estado | Dependencias |
|----|---------|----------|--------|--------------|
| `COMP-006` | **Edge Label Interning (u32 label_id)** — `edge_label` es `String`, no u32 internado | 🟢 ~2d | ❌ No implementado | Ninguna |

### 🟠 Media-Alta — Features competitivas importantes

| ID | Feature | Esfuerzo | Estado | Dependencias |
|----|---------|----------|--------|--------------|
| `COMP-008` | Pluggable index engine (VecIndex trait) — `IndexBackend` trait existe, `VecIndex` formal no | 🟡 1-2 sem | ⚠️ Parcial | Pre-COMP-027 |
| `COMP-009` | Binary bulk import (5-10x faster than INSERT) — Solo `put_batch()`, no protocolo binario | 🟢 3-4d | ❌ No implementado | Ninguna |
| `COMP-010` | Auto-embedding (embedding function abstraction) — `remote-inference` feature con Ollama, sin `EmbeddingFunction` abstracto | 🟡 1-2 sem | ⚠️ Parcial | DRV-123 |
| `COMP-012` | RoaringBitmaps for metadata indexing — `FilterBitset` custom, no `croaring` | 🟡 1 sem | ❌ No implementado | Pre-COMP-003 |
| `COMP-013` | Segment optimizer pipeline (Vacuum/Merge/Index) — `compact_layout_bfs` + vacío existe, pipeline formal no | 🟡 1-2 sem | ⚠️ Parcial | COMP-004, COMP-011 |
| `COMP-014` | FreshHNSW (background repair de enlaces huérfanos) — Sin repair background | 🟡 1 sem | ❌ No implementado | COMP-004, COMP-011 |
| `COMP-016` | Supernode mitigation (indexed relationships) — Sin indexed relationships | 🟢 3-5d | ❌ No implementado | COMP-006 |
| `COMP-017` | Accumulators for parallel graph algorithms — Sin accumulators | 🟡 1-2 sem | ❌ No implementado | Ninguna |
| `COMP-018` | Double-linked relationship chains — Relaciones dirigidas simples, sin doble enlace | 🟡 1-2 sem | ❌ No implementado | COMP-006 |
| `COMP-019` | Binary protocol (rkyv/FlatBuffers over gRPC) — Solo HTTP JSON. rkyv usado internamente en serialización | 🟡 1-2 sem | ⚠️ Parcial (rkyv interno sí) | Ninguna |
| `COMP-021` | Temporal edges (timestamp-aware relationships) — Sin timestamp-aware edges | 🟡 1 sem | ❌ No implementado | Ninguna |
| `COMP-022` | Graph Data Science library (PageRank, centrality) — Solo BFS/DFS traversal | 🟡 2-3 sem | ❌ No implementado | COMP-017 |
| `COMP-023` | 3 filtering strategies (pre/post/in-index) — Filtros en cost order (bitset → relational → vector), pre/post/in no formalizados | 🟡 1-2 sem | ⚠️ Parcial | COMP-003, COMP-012, COMP-028 |
| `COMP-024` | ACORN algorithm (second-hop filtered search) — Sin second-hop search | 🟡 1-2 sem | ❌ No implementado | COMP-003 |

### 🟡 Medio — Features de madurez y ecosistema

| ID | Feature | Esfuerzo | Estado | Dependencias |
|----|---------|----------|--------|--------------|
| `COMP-025` | JSON shredding (dynamic schema to columns) — Sin schema dinámico | 🟡 2-3 sem | ❌ No implementado | Ninguna |
| `COMP-026` | Multi-level LSM compaction (L0→L1→L2→L3) — Sin tiers múltiples | 🟡 1-2 sem | ❌ No implementado | COMP-013 |
| `COMP-027` | Multiple index types (IVF, DiskANN, SCANN) — Solo HNSW + brute-force flat | 🟠 5-10d | ❌ No implementado | COMP-008 |
| `COMP-028` | Semantic Cost Estimator (SCE) — `governor.rs` tiene rate limiting, sin cost estimator | 🟡 2 sem | ❌ No implementado | DRV-121/122 |
| `COMP-029` | Node.js/TS bindings via napi-rs — `vantadb-ts` usa WASM, no napi-rs nativo | 🟡 2-3 sem | ❌ No implementado | Ninguna |

---

## Referencias Cruzadas

- **RC items:** `docs/reviews/PROJECT_FULL_REVIEW_2026-07-13.md` (generado por `vantadb-full-review` skill)
- **REV items:** `docs/reviews/2026-07-13-full-review.md`
- **DRV findings:** `docs/plans/2026-07-15-cross-ref-docs-vs-code.md` + `docs/audit-reports/cross-ref-wave3-final-report.md`
- **OLD items:** `docs/REPORTE_EVALUACION_COMPLETO.md` secciones 6 y 7 — ~280 archivos VANTADB DOC OLD analizados
- **COMP items:** `docs/audit-reports/competitive-features-consolidated-report.md` + `docs/audit-reports/deep-analysis-{vector,graph,arch}.md` — 27 archivos, 172 features, top 30 priorizados
