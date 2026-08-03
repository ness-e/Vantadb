---
title: "Active Backlog — VantaDB"
type: backlog-tracking
status: active
tags: [vantadb, backlog, engineering, phases, priorities]
last_reviewed: 2026-07-29
verified_by: "2026-07-27: vanta-lead ejecutó 8 tareas de P5/P6/P8. 2026-07-28: 5 sub-agentes explore validaron 69 items contra código real — ver docs/audit-reports/backlog-validation-2026-07-28.md. 2026-07-29: 19 items INVESTIGACION agregados (INV-001 a INV-017) tras verificación de consolidación de 4 sub-agentes vs código real."
---

# Active Backlog — VantaDB

> **Purpose:** Single source of truth for all project tasks — organized by execution order.
> **Execution state lives in:** `docs/plans/YYYY-MM-DD-<campaign>.md` (plan file) + task files — per campaign-executor RULES.md §2. This file is the task catalog; the plan file is the execution state.
> **Completed tasks moved to:** `docs/progreso/README.md`
> **Verification method:** All items cross-checked against actual codebase (Jul 27, 2026). 8 tareas ejecutadas en sesión: TSK-106, MKT-03, NUEVO-21, MKT-04, TSK-107, COM-03, COM-04, Good first issues (18 creadas).
> **Total open items:** ~100 (59 anteriores + 19 investigaciones INV-001..INV-017 + INV-024, -6 items migrados a completado REC-001/REC-010/INV-002/NUEVO-07/INV-019/TSK-104 + 15 GitHub issues GH-119..GH-144 convertidos a backlog Phase 11)
> **Origen docs-audit:** `docs/strategy/ROADMAP.md`, `docs/progreso/bitacora.md`, `docs/reviews/FULL_CODEBASE_AUDIT_2026-07-11.md`, `docs/reviews/analisis_proyecto.md`, `docs/operations/PERFORMANCE_TUNING.md`, `docs/operations/REPO_CHECKLIST.md`, `docs/architecture/STORAGE_VERSIONING.md`, `docs/plans/2026-07-13-workflow-repair-campaign.md`, `docs/Investigaciones/cargo-check-optimizacion.md`, `docs/discord/todo.md`

---

## Exec Summary

| Phase | Items | Est. Effort | Priority |
|-------|-------|-------------|----------|
| **P0** 🚀 Release Blockers | 1 (+6 ✅ completados, +7 removidos, 1 WONTFIX) | ~2-3d | 🔴 Bloqueante |
| **P1** 🛡️ Security & Critical | 0 (+2 INV investigación ✅) | ~3-5d | 🟡 Media |
| **P2** ⚡ Quick Wins Técnicos | 0 (7 ✅ + 24 stale removidos) | — | ✅ Cerrado |
| **P3** 🧪 Test Coverage (adapters) | 0 (7 ✅ + 7 stale removidos) | — | ✅ Cerrado |
| **P4** 🔧 Engineering Health | 4 (+4 INV investigación) | ~1-2 semanas | 🟡 Media |
| **P5** 📖 Docs & Community | 9 (+1 INV investigación) | ~1-2 semanas | 🟡 Media |
| **P6** 🚀 Launch Campaign | 9 (+1 INV investigación) | ~1-2 semanas | 🟡 Media |
| **P7** 🌐 WASM & Performance | 0 (todos completados) | — | ✅ Cerrado |
| **P8** 🔮 Post-Launch & Enterprise | 28 (+10 INV investigación) | ~3-5 semanas | 🔵 Futuro |
| **P9** 📚 Old Docs Rescue (reference) | 13 (7 ✅ progreso) | — | 📖 Referencia |
| **P10** 🏗️ Competitive Features (catalog) | 20 (11 ✅ progreso) | — | 🗺️ Roadmap |

> **Items removidos (71+):** ~25 originales + 6 P0 stale + 9 P1 resueltos + 24 P2 stale + 7 P3 stale + 10 P4 completados + 7 P9 completados + 11 P10 completados + 1 P7 completado + 24 crates de integración nunca implementados

---

## ✅ Definition of Ready (DoR)

- [ ] ID único asignado
- [ ] Prioridad definida (🔴🟠🟡🟢🔵⬜)
- [ ] Archivos involucrados conocidos
- [ ] Esfuerzo estimado
- [ ] Verificado contra código real (no asumido)

## ✅ Definition of Done (DoD)

- [ ] Código compila (`cargo check` / `tsc --noEmit`)
- [ ] Tests pasan (`cargo nextest run` / `pytest`)
- [ ] Linters pasan (`cargo clippy` / `eslint`)
- [ ] Docs actualizados si aplica
- [ ] Tarea movida a `progreso/README.md`
- [ ] Changelog actualizado si es cambio visible al usuario

---

## Phase 0: 🚀 Release Blockers

> Items que bloquean un release público seguro. Resolver antes de cualquier publicación.

| ID | Descripción | Archivos | Esfuerzo | Prio |
| ~~`DEVOPS-15`~~ | ~~**Reducir default features de 7 a 3** — ❌ **WONTFIX**. Analizado: remover `cli, memmap2, fs2, sysinfo` rompe UX "it just works". Las 7 features mantienen experiencia completa. | `Cargo.toml:89` | 🟡 — | ✅ Cerrado |~~
| ~~`META-001`~~ | ~~**🔍 Root Cause Analysis: Inconsistencias del Backlog** — Auditoría profunda. **Entregable:** Reporte de hallazgos.~~ | ~~`docs/audit-reports/meta-001-root-cause-analysis.md`~~ | ~~🟠 2-3d~~ | ✅ |

> **Items removidos (7):** DEVOPS-10 (deferido), DEVOPS-12 (PyPI signing), DEVOPS-14 ✅, NUEVO-09 ✅, NUEVO-10 ✅, ~~DEVOPS-15 re-opened~~ → ❌ **WONTFIX** (7 features necesarias para UX completo), META-001 queda como único P0 activo.

---

## Phase 1: 🛡️ Security & Critical

> Investigaciones de seguridad y dependencias críticas.

| ID | Descripción | Archivos | Esfuerzo | Prio |
|----|-------------|----------|----------|------|
| ~~`INV-001`~~ | ~~**🔍 Investigar dependencias con RUSTSEC activas** — Auditar `Cargo.lock` contra advisories conocidos. 3 crates reportadas: `atomic-polyfill` (RUSTSEC-2023-0089), `paste` (RUSTSEC-2024-0436), `rustls-pemfile` (RUSTSEC-2025-0134). **✅ COMPLETADA 2026-07-29 — Las 3 están gestionadas o son stale. Reporte: `docs/audit-reports/inv-001-rustsec-2026-07-29.md`. Cargo deny pasa limpio.**~~ | ~~`Cargo.lock`, `deny.toml`~~ | ~~🟢 2-4h~~ | ✅ |
| ~~`INV-024`~~ | ~~**🔍 Auditar bloqueos `unsafe` sin SAFETY docs** — Revisar todos los bloques `unsafe` en el código Rust (`src/node.rs`, `src/index/graph.rs`, `src/storage/vfile.rs`) que carecen de invariantes documentados. Verificar si hay UB potencial o si son seguros pero sin docs. Proponer: (a) agregar SAFETY comments documentando invariantes, o (b) reemplazar con alternativas seguras. **Sin implementación — solo auditoría + propuesta.** **✅ COMPLETADA 2026-07-30 — 39 bloques auditados (28 SAFE, 4 SAFE_BUT_UNDOCUMENTED, 7 UB_POTENTIAL). 1 High (panic-DoS sq8_similarity) + 1 Medium (UB alineación). Reporte: `docs/audit-reports/inv-024-unsafe-audit-2026-07-30.md`. cargo deny PASSED, cargo audit 0 vulns.**~~ | ~~`src/node.rs`, `src/index/graph.rs`, `src/storage/vfile.rs`, `src/index/search.rs`~~ | ~~🟡 3-5d~~ | ✅ |

> **Items previos resueltos (9):** Todos los items P1 originales resueltos/deferidos en campañas anteriores.

---

> **Phase 2: ⚡ Quick Wins Técnicos** — **31 items removidos:** DRV-014 ✅, DRV-028 ✅, DRV-041 ✅, VFY-006 ✅, VFY-007 ✅, REV-012 ✅, DRV-136 ✅ + 24 stale items de la auditoría original. No quedan items activos en P2.
> ⚠️ **Nota DRV-014 (2026-07-31):** El fix fue **REVERTIDO** por `cae92db3` "perf(engine): Phase 1 optimizations — WAL batch". El clon de WalRecords (`Vec<Vec<WalRecord>>` + `record.clone()`) se reintrodujo deliberadamente para agrupar por shard y usar `WalWriter::batch_append()` (1 lock + 1 write_all + 1 maybe_sync por shard, 3-5× speedup en WAL writes). La tarea quedó cerrada ✅ como completada en su momento (`3bdfc93e`), pero el código actual NO refleja ese fix — es un tradeoff de performance posterior, no deuda pendiente.

---

> **Phase 3: 🧪 Test Coverage (Adapters & Engine)** — **14 items removidos:** DRV-013 ✅, DRV-017 ✅, DRV-061 ✅, DRV-067 ✅, DRV-073 ✅, TEST-11 ✅, TEST-12 ✅ + 7 stale de auditoría original. No quedan items activos en P3.

---

## Phase 4: 🔧 Engineering Health & Architecture

> Investigaciones de salud de ingeniería — rendimiento, concurrencia, arquitectura.

| ID | Descripción | Archivos | Esfuerzo | Prio |
|----|-------------|----------|----------|------|
| ~~`INV-002`~~ | ~~**🔍 Memory Telemetry Correction — investigación** — El reporte de RAM actual es inconsistente (mezcla core RAM, index RAM, page cache, mmap, ingest buffers). Audit dice "hasta que arregles la telemetría, no hables de eficiencia de memoria". **Alcance:** (1) Mapear qué mide cada métrica actual vs qué debería medir, (2) Diseñar esquema de telemetría con categorías separadas (core, index, page cache, mmap, ingest), (3) Identificar qué estructuras contribuyen a cada categoría, (4) Proponer implementación con `tracing::metrics` + labels. **Sin implementación — solo diseño + propuesta.** **✅ COMPLETADA 2026-07-30 — Esquema 5 categorías diseñado (core/index/page_cache/mmap/ingest) en `docs/operations/MEMORY_TELEMETRY.md`; IntGaugeVec con label `category` validado contra API oficial prometheus; contrato `OperationalMetrics` preservado.**~~ | ~~`src/metrics/`, `docs/operations/MEMORY_TELEMETRY.md`, `docs/operations/PERFORMANCE_TUNING.md`~~ | ~~🟡 3-5d~~ | ✅ |
| ~~`INV-003`~~ | ~~**🔍 Sync Blocking en Tokio — auditoría** — Auditoría de `std::fs::*` y `std::sync::Mutex::lock()` en contextos `async`. **✅ COMPLETADA 2026-07-31 — Reporte: `docs/Investigaciones/INV-003-sync-blocking-tokio.md`. Se verificó que llamadas bloqueantes usan `spawn_blocking` correctamente.**~~ | ~~`src/`, `vantadb-server/`, `vantadb-mcp/`~~ | ~~🟡 2-3d~~ | ✅ |
| ~~`INV-004`~~ | ~~**🔍 mimalloc como Global Allocator — investigación** — Evaluación de `mimalloc` y allocators globales por plataforma. **✅ COMPLETADA 2026-07-31 — Reporte: `docs/Investigaciones/INV-004-mimalloc-global-allocator.md`. Feature `custom-allocator` y `mimalloc` validados y configurados.**~~ | ~~`Cargo.toml`, `src/bin/vanta-cli.rs`~~ | ~~🟢 4-6h~~ | ✅ |
| ~~`INV-005`~~ | ~~**🔍 ErrorBoundary en web frontend — investigación** — Auditoría de `react-error-boundary` y manejo de errores Next.js. **✅ COMPLETADA 2026-07-31 — Reporte: `docs/Investigaciones/INV-005-error-boundary-web.md`. Se propone adoptar `error.tsx` nativo de App Router.**~~ | ~~`web/src/app/layout.tsx`, `web/package.json`~~ | ~~🟢 2-4h~~ | ✅ |

> **Items previos completados (10):** WEB-03 ✅, WEB-04 ✅, VFY-004 ✅, VFY-011 ✅, DRV-121 ✅, DRV-122 ✅, DRV-123 ✅, DRV-130 ✅, DRV-131 ✅, DOC-20 ✅ — movidos a `docs/progreso/README.md`.

---

## Phase 5: 📖 Docs & Community

> Preparación de documentación pública, comunidad, y onboarding.

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado Real |
|----|-------------|----------|----------|------|-------------|
| ~~`MKT-14`~~ | ~~**Publicar 3 case studies** + ruta `/case-studies/` + `/case-studies/[slug]`~~ | ~~`web/src/components/vanta/vanta-data.ts` (CASE_STUDIES, 3 items), `web/src/app/case-studies/`~~ | ~~🟡 1-2d~~ | ~~🔴~~ | ✅ COMPLETADA 2026-08-02 — 3 CS con métricas, listing + detail pages, i18n (62 keys), rutas en navbar + LIVE_ROUTES. Validado por audit 2026-07-28. |
| ~~`TSK-106`~~ | ~~**Habilitar GitHub Discussions**~~ | — | ~~🟢 1h~~ | ~~🟠~~ | ✅ Ya estaba habilitado (`has_discussions: true`). 0 discussions creadas. |
| `NUEVO-01` | **README hero** con readme-aura + benchmark gráfico + GIF demo WASM | `README.md` (PNG estática actual) | 🟡 2-3d | 🟠 | ❌ Desde cero |
| ~~`NUEVO-07`~~ | ~~**Migration tools: Chroma→Vanta, LanceDB→Vanta** — **✅ COMPLETADA 2026-08-02** — `vantadb_py/migrate/chroma.py` + `lancedb.py` + CLI, tests (4 migration + 42 regresión), tutoriales corregidos a API real `vantadb_py.VantaDB` (el audit 2026-07-28 reportó scripts inexistentes — falso positivo).~~ | ~~`vantadb-python/vantadb_py/migrate/`, `docs/tutorials/`~~ | ~~🟡 3-5d~~ | ~~🟠~~ | ✅ |
| ~~`NUEVO-08`~~ | ~~**Learning path estructurado** en tutorials/ (5-7 ejemplos)~~ — **✅ COMPLETADA 2026-08-02** — 6 tutoriales active con API real (`vantadb_py`), índice learning path (`docs/tutorials/index.md`), mdBook sync (SUMMARY 6). API inventada corregida en 01/02. Commits a460e4e4→cff2fb99. | ~~`docs/tutorials/`, `docs/book/src/tutorials/`~~ | ~~🟡 2-3d~~ | ✅ |
| ~~`NUEVO-10`~~ | ~~**Benchmark suite pública reproducible** — **✅ COMPLETADA 2026-08-02** — `benchmarks/requirements.txt` (path standalone `pip install vantadb-py` desde PyPI 0.5.0), hints corregidos en los 3 scripts (vantadb_local_bench, competitive_bench, batch_vs_sequential), `benchmarks/README.md` (guía pública), `docs/operations/BENCHMARKS.md` sección 3 reescrita (standalone antes que maturin). Smoke test exitoso (JSON 5/5 claves). Commit d0b1c7c6.~~ | ~~`benchmarks/`, `docs/operations/BENCHMARKS.md`~~ | ~~🟡 3-5d~~ | ✅ |
| ~~`TSK-107`~~ | ~~Community showcase page (`/showcase`, `/about/community`)~~ | ~~`web/src/app/showcase/page.tsx` (6 items mock)~~ | ~~🟢 4-6h~~ | ~~🟡~~ | ✅ 6 items actualizados a ejemplos reales (LangGraph, AutoGen, Haystack, CrewAI, Rust hybrid, GraphRAG) |
| `—` | Good first issues (18 open en GitHub) | GitHub Issues (#118-#145) | 🟢 2-4h | 🟠 | ✅ 18 issues creados (22 en total, 3 duplicados cerrados) |
| ~~`INV-006`~~ | ~~**🔍 Blog series completion — plan de finalización**~~ — **✅ COMPLETADA 2026-08-02** — `docs/strategy/BLOG_SERIES_PLAN.md`: inventario 4 web vs 3 docs/blog (6 mismatches M1-M6), revisión drafts, audiencia + keyword research (5 segmentos), calendario Show HN + cadencia 2/mes. Sin implementación (solo plan). Commit 042e8e50. | ~~`docs/blog/`, `docs/strategy/SHOW_HN_PREP.md`~~ | ~~🟢 2-4h~~ | ✅ |
| `COM-02` | **Configurar Discord: reaction roles, autorole, logging, welcome DM, onboarding** | `docs/discord/todo.md` + assets SVG + server activo | 🟡 2-3d | 🟢 | ⚠️ Docs + assets OK. Config pendiente |
| `COM-03` | **Discord: AutoMod, stickers/emojis, forums seed** | — | 🟢 4-6h | 🟢 | ⚠️ Forums seedeado (9 threads: FAQ/Showcase/Ideas/Bug). AutoMod/stickers/emojis requieren Discord UI manual — no API-accessible |
| `COM-04` | **Discord: ticketing system, stage channel, Server Discovery, Canny.io** | — | 🟢 4-6h | 🟢 | ⚠️ Stage channel creado. Ticketing requiere bot auth (Ticket Tool/Helper.gg), Server Discovery necesita 1000+ miembros, Canny.io requiere cuenta externa — documentado en `docs/discord/todo.md` |

---

## Phase 6: 🚀 Launch Campaign

> Todo lo necesario para el Show HN y marketing de lanzamiento.

| ID | Descripción | Esfuerzo | Prio | Estado Real |
|----|-------------|----------|------|-------------|
| `LEG-01` | **Registrar trademark "VantaDB" (USPTO + EUIPO)** | 🟡 2-4h | 🔴 | ❌ Desde cero |
| ~~`MKT-03`~~ | ~~**Show HN post**~~ | ~~🟢 2h~~ | ~~🔴~~ | ✅ Draft actualizado a v0.4.0 en `docs/strategy/SHOW_HN_PREP.md` |
| ~~`MKT-04`~~ | ~~Reddit posts (r/rust, r/MachineLearning, r/LocalLLaMA)~~ | ~~🟢 2-4h~~ | ~~🟠~~ | ✅ 3 drafts en `docs/strategy/REDDIT_POSTS.md` |
| `MKT-05` | Technical blog posts (5+ pre-launch) — 4/5 posts escritos | 🟡 2-3d | 🟠 | ⚠️ 4/5 completados |
| `MKT-10` | "AI Agent Memory" campaign | 🟡 2-3d | 🟠 | ❌ Desde cero |
| `MKT-15` | **Página de benchmarks competitivos** (`/benchmarks`) — Ruta existe (BenchmarksView + BenchmarkRace, 444L), sin tabla competitiva VantaDB vs Pinecone/Weaviate/Chroma | 🟡 2-3d | 🔴 | ⚠️ BenchmarksView OK (BENCH01, SIFT1M, LatencyComparator). Falta tabla competitiva |
| `MKT-16` | **Publicar metodología de benchmark GraphRAG** — Sin doc específico | 🟡 1-2d | 🟡 | ❌ Desde cero |
| `MKT-17` | Página de comparación competitiva interactiva — Sin ruta `/compare` ni archivos | 🟡 2-3d | 🟢 | ❌ Desde cero |
| `TSK-103` | Public benchmark site (`/benchmarks`) — BenchmarksView + BenchmarkRace existen (BENCH01, SIFT1M). Falta script público reproducible | 🟡 2-3d | 🟠 | ⚠️ `/benchmarks` existe con datos benchmark reales. Sin script standalone público |
| ~~`TSK-104`~~ | ~~**Demo agent: LangChain + Ollama + VantaDB** — **✅ COMPLETADA 2026-08-02** — `examples/python/langchain_ollama_rag.py` (151 líneas) con integraciones reales (`VantaDBVectorStore` + `OllamaEmbeddings`), fallback determinístico sin Ollama, smoke exit 0. Sketch legacy `langchain_rag.py` eliminado.~~ | ~~`examples/python/langchain_ollama_rag.py`, `docs/operations/EXPERIMENTAL_FEATURES.md`~~ | ~~🟡 1-2d~~ | ~~🟠~~ | ✅ |
| ~~`INV-007`~~ | ~~**🔍 Competitive benchmark vs LanceDB/Chroma — investigación y diseño** — El asset marketing #1 para audiencia técnica. **Alcance:** (1) Investigar `ann-benchmarks` y su conector para VantaDB, (2) Definir datasets: glove-100-angular + sift-128-euclidean, (3) Diseñar metodología: throughput, latencia p50/p95/p99, Recall@10, RAM, (4) Evaluar si benchmarks internos existentes (`/benchmarks`) pueden extenderse o si se necesita script standalone, (5) Proponer implementación mínima para página pública. **Sin implementación — solo diseño + propuesta.** **✅ COMPLETADA 2026-08-03** (Doc: `docs/Investigaciones/INV-007-competitive-benchmark-lancedb-chroma.md`. Propuesta: harness standalone reproducible Python con glove-100-angular + sift-128-euclidean, protocolo Recall@10/QPS/latencia p50-p95-p99/RSS, 3 slices verticales, rechazado ann-benchmarks por desuso.)~~ | ~~🟡 2-3d~~ | ~~🟠~~ | ✅ |

---

## Phase 7: 🌐 WASM & Performance

| ID | Descripción | Archivos | Esfuerzo | Prio |
|----|-------------|----------|----------|------|
> **Items removidos (5):** NUEVO-11/12 (WASM IndexedDB + multi-tab coordinación — ✅ implementados), NUEVO-14 (bundle 394KB gzip < 500KB — ✅ en WASM-04), NUEVO-19 (SourceDesign/ no existe), BENCH-01 (solo mención en backlog, sin script ni dataset)

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
| ~~`NUEVO-17`~~ | ~~**Segment LSM-style** — hot/warm/cold tiers. Fjall tiene LSM interno, tiers no~~ — **✅ COMPLETADA 2026-08-02** (TierPolicy SizeBased/FrequencyBased/AgeBased + TierPolicyConfig + L3 archive configurable en `LsmConfig`, promoción L0(hot)→L1(warm)→L2(cold)→L3(archive) en `compact_level`, 3 tests de promoción. `docs/architecture/STORAGE-TIERS.md`.) | ~~Muy alto~~ | ✅ |
| `NUEVO-18` | **Sparse vectors nativos** — hybrid search real. Solo mención en test | Alto | 🔵 |
| ~~`NUEVO-21`~~ | ~~**Vectara competitive research**~~ | ~~🟢 2-4h~~ | ✅ Hallazgo clave: Vectara cerró self-service tier → gap de mercado para soluciones local-first. Reporte en `docs/audit-reports/vectara-competitive-research-2026-07-27.md` |
| ~~`TSK-107b`~~ | ~~**✅ Audit logging enterprise (JSONL, timestamp + op)** — módulo `src/audit.rs` append-only JSONL (ISO 8601 + op + outcome + reason), opt-in via `audit_log_path`/`VANTADB_AUDIT_LOG_PATH`, hooks en put/put_batch/delete/delete_by_filter/export/import, no-op sin config. Migrada a progreso.~~ | ~~🟡 2-3d~~ | ~~🟡~~ | ~~✅ Hecho~~ |
| ~~`ENT-04`~~ | ~~**✅ Connection pooling + circuit breaker** — módulos `src/connection_pool.rs` + `src/circuit_breaker.rs` (feature-gated `server`), `ServerState` con breaker+pool en `cli_server.rs`, middleware breaker como capa más externa, 503 + `Retry-After` al abrir, probe half-open, config `server.*` (pool/min_connections/max_connections/breaker threshold/timeout). Migrada a progreso.~~ | ~~🟡 2-3d~~ | ~~🟡~~ | ~~✅ Hecho~~ |
| `BIZ-01` | **Enterprise features: encryption + RBAC ya en crate principal. Audit/replication/enterprise crate separado no existen** | 🟡 3-5d | 🟡 ⏳ |
| `WEB-001` | **Implementar WASM demo en `/playground`** — El CodePlayground actual es un simulador, no corre WASM real. La demo WASM anterior estaba en `web_old/` (eliminada). Reconstruir requiere integrar `@vantadb/wasm` en el componente. | 🟡 1-2d | 🟡 | ❌ Desde cero — CodePlayground existe pero sin WASM real |
| `WEB-18` | **⚠️ Definir pricing y estrategia de monetización** — El archivo `docs/web/standards/product-positioning.md` y `vanta-data.ts` tienen un plan "Team $49/mes por seat" que NO existe en `docs/strategy/GO_TO_MARKET.md`. Decidir: (a) agregar Team $49 a la estrategia GTM, (b) alinear vanta-data.ts con los planes reales de GO_TO_MARKET.md, o (c) eliminar pricing del sitio hasta definir. | 🟡 2-4h | 🔴 |

| `DESKTOP-01` | **🔍 Investigar Tauri como plataforma desktop para VantaDB** — Análisis de propuesta: integración nativa Rust (`vantadb` como dependency directa en Cargo.toml de Tauri), casos de uso (desktop AI app privada con memoria local), comparativa vs Electron (requiere TS SDK), effort estimate para MVP desktop, y recomendación de arquitectura. **Origen:** investigación previa en `docs_backup_2026-06-30/Investigaciones/VantaDB_Investigacion_Contexto_GTM.md` (líneas 966-976, prioridad 🔴). | 🟡 3-5d | 🔵 |
| ~~`SDK-01`~~ | ~~**`delete_by_filter()` — Implementar desde cero**~~ — **✅ COMPLETADA 2026-07-31** (Implementado `VantaEmbedded::delete_by_filter` en SDK + PyO3/WASM + CLI `delete-by-filter`) | ~~🟡 3-5d~~ | ✅ |
| ~~`SDK-02`~~ | ~~**`similar_to_key()` — Implementar desde cero**~~ — **✅ COMPLETADA 2026-07-31** (Implementado `VantaEmbedded::similar_to_key` en SDK + CLI `similar-to-key`) | ~~🟡 3-5d~~ | ✅ |
| ~~`SDK-03`~~ | ~~**`count()` con filtros — Implementar desde cero**~~ — **✅ COMPLETADA 2026-07-31** (Implementado `VantaEmbedded::count` con soporte de metadatos en SDK + CLI `count`) | ~~🟡 1-2d~~ | ✅ |
| ~~`SDK-04`~~ | ~~**Multi-namespace search — Implementar desde cero**~~ — **✅ COMPLETADA 2026-07-31** (Implementado `search_multi` y `search_all` en SDK + CLI `search-multi`/`search-all`) | ~~🟡 4-7d~~ | ✅ |
| ~~`SDK-05`~~ | ~~**Expanded metadata filters en SDK — Exponer 6 operadores**~~ — **✅ COMPLETADA 2026-07-31** (Implementado `matches_advanced_filters` con operadores Eq, Neq, Gt, Lt, Gte, Lte en listados y paginación) | ~~🟡 5-10d~~ | ✅ |
| ~~`REC-001`~~ | ~~**[Foundation] Definir `VantaFilterOp` + `VantaMemoryFilter` types** — `VantaFilterOp` enum con 6 variantes (Eq, Neq, Gt, Lt, Gte, Lte), `VantaMemoryFilterItem` struct, `VantaMemoryFilter = Vec<...>`. Creado en `src/sdk/types.rs:106-126`, re-exportado desde `src/sdk/mod.rs`. `cargo check + clippy` ✅. **✅ COMPLETADA 2026-07-29.**~~ | ~~🟢 2-3h~~ | ✅ |
| ~~`REC-007`~~ | ~~**WAL compaction + vacuum CLI** — Binding directo de funciones existentes (`VantaEmbedded::compact_wal()`, `VantaEmbedded::vacuum()`) a CLI. `vanta-cli wal compact` + `vanta-cli wal vacuum`. Sin lógica nueva. **Nuevo — identificado en auditoría 2026-07-28.** **✅ COMPLETADA 2026-07-29.**~~ | ~~🟢 1-2h~~ | ✅ |
| ~~`REC-008`~~ | ~~**[Diseño] Incremental backup + PITR CLI**~~ — **✅ COMPLETADA 2026-07-31** (Diseño documentado, Fase A implementada agregando `MANIFEST.json` con integridad CRC32C a backups) | ~~🟡 1d~~ | ✅ |
| ~~`REC-009`~~ | ~~**[Investigación] Analizar viabilidad PQ (Product Quantization)**~~ — **✅ COMPLETADA 2026-07-31** (Análisis completo documentado, viabilidad deferida) | ~~🟢 2-4h~~ | ✅ |
| ~~`REC-010`~~ | ~~**py.typed marker + config maturin** — 2 `.pyi` stubs existían pero `py.typed` no. PEP 561 non-compliant. Creado `py.typed` + configurado `[tool.maturin] include` en `pyproject.toml`. **✅ COMPLETADA 2026-07-29.**~~ | ~~🟢 30min~~ | ✅ |
| ~~`REC-999`~~ | ~~**Corregir `docs/progreso/README.md`**~~ — **✅ COMPLETADA 2026-07-31** (README de progreso actualizado y corregido con los hitos reales) | ~~🟢 30min~~ | ✅ |

### 🔍 Investigaciones Post-Consolidación

> Items agregados 2026-07-29 tras verificación de 19 hallazgos de 4 sub-agentes contra código actual. **Sin implementación — solo investigación + propuesta.**

| ID | Descripción | Archivos | Esfuerzo | Prio |
|----|-------------|----------|----------|------|
| ~~`INV-008`~~ | ~~**🔍 Batch Queries Python SDK — diseño** — `VantaDB.search_batch()` para ejecutar múltiples queries en paralelo vía Rayon. **Alcance:** (1) Definir API: `search_batch(queries: List[SearchRequest]) -> List[SearchResult]`, (2) Diseñar conversión eager a Rust types + `py.allow_threads` + Rayon parallel map, (3) Evaluar target: batch 10 queries < 3× single query latency, (4) Identificar si `VantaEmbedded` necesita método nuevo o wrapper. **Sin implementación — solo diseño.** **✅ COMPLETADA 2026-08-03** (Doc: `docs/Investigaciones/INV-008-batch-queries-python-sdk.md`. `search_batch(vectors, top_k)` vector-only ya existía (lib.rs:1181, Rayon + GIL release). Propuesta: `search_batch_requests` con dataclass SearchRequest completo, patrón GIL+Rayon existente, errores fail-fast v1, target batch 10 < 3× single.)~~ | ~~`vantadb-python/src/lib.rs`, `src/sdk/api.rs`~~ | ~~🟡 1-2d~~ | ~~🟡~~ | ✅ |
| ~~`INV-009`~~ | ~~**🔍 Phrase Queries + Term Positions — diseño** — Implementar phrase query operator con almacenamiento de term positions para snippets destacados. **Alcance:** (1) Diseñar extensión del text index para almacenar term positions por documento, (2) Evaluar `tantivy` como backend vs storage custom en VantaFile, (3) Definir sintaxis de phrase query (comillas dobles en IQL), (4) Evaluar integración con `generate_snippet_with_highlighting` existente. **Sin implementación — solo diseño.** **✅ COMPLETADA 2026-08-03** (Doc: `docs/Investigaciones/INV-009-phrase-queries-term-positions.md`. Infra phrase-ready YA existía: `TextQueryPlan.phrases`, `token_positions`, `posting_value`, test v3. Propuesta: sintaxis IQL comillas dobles vía `Condition::TextMatch`, matching con `text_positions_match_phrase`, rechazado tantivy (YAGNI), highlight de frase en snippets.)~~ | ~~`src/text_index.rs`, `src/sdk/search/snippet.rs`, `src/parser/`~~ | ~~🟡 1-2d~~ | ~~🟡~~ | ✅ |
| ~~`INV-010`~~ | ~~**🔍 ACID rollback multi-capa completo — diseño** — VFY-010 y TASK-30 implementaron ACID Phase 1-2 (buffered writes + BEGIN/COMMIT/ABORT). Falta rollback coordinado entre WAL, VantaFile, HNSW y KV store. **Alcance:** (1) Mapear estado actual (qué capas tienen rollback y cuáles no), (2) Diseñar protocolo de two-phase rollback entre capas, (3) Evaluar Approach B del research `ACID_TRANSACTIONS.md` vs implementación actual, (4) Proponer plan de implementación por fases. **Sin implementación — solo diseño.** **✅ COMPLETADA 2026-08-03** (Doc: `docs/Investigaciones/ACID_ROLLBACK_DESIGN.md`. Research recuperado de git, protocolo 2-phase + crash-windows, plan 4 fases.)~~ | ~~`src/storage/engine/ops.rs`, `src/index/graph.rs`, `src/wal.rs`~~ | ~~🟡 2-3d~~ | ✅ |
| `INV-011` | **🔍 Core-Server Separation — auditoría** — Verificar si el core embebido (`VantaEmbedded`) tiene dependencias no deseadas del modo servidor (axum, tower, MCP). **Alcance:** (1) Escanear features de `Cargo.toml` que mezclan core vs server, (2) Identificar imports de server-only en `src/` (no en `vantadb-server/`), (3) Verificar si `default` features incluyen server deps, (4) Proponer separación limpia con feature gates. **Sin implementación — solo auditoría.** | `Cargo.toml`, `src/lib.rs`, `vantadb-server/Cargo.toml` | 🟡 1-2d | 🟡 |
| `INV-012` | **🔍 Anti-Locality Disk Layout — re-evaluación** — DRV-130 T3 concluyó WONTFIX (~9% mejora, <20% threshold). Revisar si con los cambios recientes (LSM compaction, multi-level storage) el BFS relabeling tiene más impacto. **Alcance:** (1) Re-ejecutar benchmark con dataset SIFT 1M en la arquitectura actual (LSM + multi-level), (2) Comparar resultados con benchmark original de DRV-130 (2,440ms vs 2,221ms, ~9%), (3) Si mejora >15%, recomendar re-apertura. **Sin implementación — solo benchmark + recomendación.** | `src/index/graph.rs`, `tests/certification/`, `benches/` | 🟡 1-2d | 🟢 |
| ~~`INV-019`~~ | ~~**🔍 Advanced Tokenizer (Unicode + Stopwords) — investigación** — **✅ SKIP 2026-08-02 — YA IMPLEMENTADA.** Verificado vs código: `src/tokenizer.rs` (tokenize_advanced, stemming, stopwords, Unicode folding), feature `advanced-tokenizer` en default (Cargo.toml:94,108), wiring en `src/config.rs`, tests multilingües (ES/FR/DE), bench `benches/tokenizer_bench.rs`. Commits: `1a7c4d04`, `7459a558`. Único gap: `docs/api/ADVANCED_TOKENIZER.md` no existe — doc de API, ticket separado.**~~ | ~~`src/text_index.rs`, `src/tokenizer.rs`, `Cargo.toml`, `src/config.rs`~~ | ~~🟡 1-2d~~ | ~~🟡~~ | ✅ SKIP (ya implementada) |

### 🌐 Web Frontend — Auditorías

| ID | Descripción | Archivos | Esfuerzo | Prio |
|----|-------------|----------|----------|------|
| ~~`INV-013`~~ | **🔍 JSON-LD structured data — auditoría** — Verificar si el sitio web tiene JSON-LD structured data para SEO. **Alcance:** (1) Revisar `layout.tsx` y `page.tsx` para `<script type="application/ld+json">`, (2) Verificar si Next.js Metadata API genera JSON-LD automáticamente, (3) Si falta: proponer schema.org/SoftwareApplication con keywords, description, author, (4) Evaluar herramientas de validación (Google Rich Results Test). **Sin implementación — solo auditoría + propuesta.** — **📋 AUDITORÍA 2026-08-03** — **Hallazgo: AUSENTE.** `layout.tsx` exporta metadata rico (title, description, keywords, authors, openGraph, twitter, icons, manifest) pero **cero** `<script type="application/ld+json">` / `jsonLd`. `page.tsx` no exporta metadata ni JSON-LD. **Veredicto Metadata API:** Next.js 16 Metadata API NO genera JSON-LD — solo tags `<head>` (title/description/OG/Twitter viewport). No existe campo `jsonLd` en el tipo `Metadata`; el JSON-LD debe emitirse manualmente con un `<script type="application/ld+json">` dentro del JSX (Server Component). **Propuesta (schema.org/SoftwareApplication):** ```json {"@context":"https://schema.org","@type":"SoftwareApplication","name":"VantaDB","description":"Local-first, embedded Rust database engine for AI agents and local RAG. Persistent memory, crash-safe WAL (CRC32C), and native hybrid search (BM25 + HNSW via RRF) — zero network, in-process, 1.2ms latency.","applicationCategory":"DatabaseApplication","applicationSubCategory":"Vector Database","operatingSystem":"Windows, macOS, Linux","softwareVersion":"0.2.0","offers":{"@type":"Offer","price":"0","priceCurrency":"USD"},"author":{"@type":"Person","name":"ness-e"},"keywords":["vector database","Rust","embedded database","local-first","HNSW","BM25","RRF","hybrid search","RAG"]} ``` **Validación ✅ Google Rich Results Test (search.google.com/test/rich-results) o validator.schema.org. Falta de JSON-LD = sin rich snippets en resultados de búsqueda.** | `web/src/app/layout.tsx`, `web/src/app/page.tsx` | 🟢 2-4h | ✅ |
| ~~`INV-014`~~ | ~~**🔍 Light mode (CSS muerto) — auditoría** — **✅ COMPLETADA 2026-08-03** (Hallazgo: **NO existe CSS light muerto — el sitio es LIGHT-ONLY por diseño** (paleta manga/linocut cream `#FBF9F5` / ink `#000000` / neon `#FF5500`). `globals.css`: `@theme inline` + `:root` definen SOLO tokens light; **cero bloque `.dark`, cero variantes `light:`, cero `@media (prefers-color-scheme: light)`** (único media query: `prefers-reduced-motion`); sin duplicación de variables `:root` vs `.dark` porque `.dark` no existe. Wiring real: `ThemeProvider` (attribute="class", defaultTheme="light", enableSystem=false, disableTransitionOnChange) **NO está montado** — `layout.tsx` envuelve solo en `LanguageProvider`; `ThemeToggle` es funcional (useTheme + mounted guard + aria) pero su único consumer es `navbar.tsx:396`, **código muerto** (reemplazado por `site-navbar.tsx`, que NO tiene toggle — grep confirma que `navbar.tsx` no se importa en ningún lado); `next-themes` lo importan únicamente los 2 componentes huérfanos. Recomendación: la deuda real es la **plomería DARK inerte**, no CSS light. Eliminar `theme-provider.tsx` + `theme-toggle.tsx` + dep `next-themes` de `package.json` (YAGNI, sigue el patrón dead-code ya documentado en `web/AUDIT.md`). Dark mode real = feature aparte: requeriría bloque `.dark` con nueva paleta + sobreescribir decenas de utilities hardcodeadas a light — contradice la estética manga/linocut; NO reactivar. Además corregir nota stale en `web/AGENTS.md` ("class-based theme switching (light default)") — es aspiracional, next-themes no está montado.)~~ | ~~`web/src/app/globals.css`, `web/src/components/vanta/theme-toggle.tsx`, `web/src/components/vanta/theme-provider.tsx`~~ | ~~🟢 1-2h~~ | ~~🟢~~ | ✅ |
| ~~`INV-015`~~ | **🔍 Touch targets < 44px — auditoría** — Verificar accesibilidad mobile: botones y enlaces deben tener mínimo 44×44px de área táctil. **Alcance:** (1) Inspeccionar componentes interactivos del sitio (botones, enlaces, icon buttons), (2) Medir tamaños actuales vs estándar WCAG 2.5.8 (44px), (3) Identificar componentes que no cumplen, (4) Proponer fixes con Tailwind `min-h-[44px] min-w-[44px]`. **Sin implementación — solo auditoría + propuesta.** **📋 AUDITORÍA 2026-08-03** — Resultado: **~23 componentes interactivos no cumplen 44×44px** (WCAG 2.5.8 Target Size Minimum: 24×24 mínimo obligatorio, 44×44 recomendado). Inventario por componente → tamaño actual → cumple → fix:

**P0 · Navbar (todos los viewports):** `site-navbar.tsx:378` hamburger `h-9 w-9` = 36×36 ❌ → `size-11`; `site-navbar.tsx:356` search ⌘K `h-9 w-9` = 36×36 ❌ → `size-11`; `lang-toggle.tsx:14` `px-2 py-1.5 text-xs` ≈ 32×40 ❌ → `min-h-[44px] min-w-[44px] px-3`; `theme-toggle.tsx:19` `h-9 w-9` = 36×36 ❌ → `size-11`.

**P1 · Close buttons de modales/overlays:** `command-palette.tsx:228` `h-7 w-7` = 28×28 ❌ → `size-11`; `shortcut-overlay.tsx:101` `h-7 w-7` = 28×28 ❌ → `size-11`; `tutorial-modal.tsx:106` `h-8 w-8` = 32×32 ❌ → `size-11`; `easter-egg.tsx:97` `h-8 w-8` = 32×32 ❌ → `size-11`; `tutorial-modal.tsx:118` step segments `h-1.5` = 6px alto ❌ → `min-h-[44px]`.

**P2 · Copy buttons:** `docs-view.tsx:563` CodeBlock `h-7 w-7` = 28×28 + `opacity-0` hover-only ❌ → `size-11`; `docs-view.tsx:648` CliCard `h-7 w-7` = 28×28 ❌ → `size-11`; `tutorial-modal.tsx:257` StepCodeBlock `h-7 w-7` = 28×28 ❌ → `size-11`; `docs-view.tsx:596` CopyButton `px-2 py-0.5 text-[10px]` ≈ 22px alto ❌ → `min-h-[44px] min-w-[44px]`; `code-terminal.tsx:196-213` Run/Copy `px-2 py-0.5 text-[10px]` ≈ 22px ❌ → `min-h-[44px] min-w-[44px]`.

**P3 · Nav links text-only:** `footer.tsx:155` 31 nav buttons `text-[11px]` sin padding ≈ 16-18px ❌ → `min-h-[44px] flex items-center`; `footer.tsx:119-142` community links `text-xs` ≈ 18px ❌ → `min-h-[44px] flex items-center`; `site-navbar.tsx:398` mobile group headers `px-1 py-2.5` ≈ 36px ❌ → `min-h-[44px]`; `site-navbar.tsx:420` mobile group items `px-2 py-2` ≈ 31px ❌ → `min-h-[44px]`; `site-navbar.tsx:441` mobile flat links `px-3 py-2` ≈ 35px ❌ → `min-h-[44px]`; `docs-view.tsx:167` sidebar sections `px-2 py-1.5` ≈ 32px ❌ → `min-h-[44px]`.

**P4 · Clear/filter icon buttons:** `changelog-section.tsx:81` clear-search icon 14px sin padding ❌ → `size-11`; `docs-view.tsx:148` search clear icon 14px ❌ → `size-11`; `changelog-section.tsx:93` filters `px-2 py-1` ≈ 26px ❌ → `min-h-[44px] min-w-[44px]`; `latency-comparator.tsx:267` dataset `py-2` ≈ 35px ❌ → `min-h-[44px]`.

**CUMPLEN ✅:** back-to-top 48×48; hero CTAs ≈52px; cta-final buttons ≈48px; benchmark-race ≈52px; FAQ accordion row `p-4` ≈76px; FAQ CTA links ≈44px; architecture/benchmarks-view CTA ≈52px. **Dead code excluido (web/AUDIT.md):** navbar.tsx, hero-mark-interactive.tsx, ecosystem.tsx, metrics-bar.tsx. **Propuesta priorizada:** P0 navbar → P1 close → P2 copy → P3 nav links → P4 clear/filter. Patrón Tailwind: `size-11` (=44px) para icon-only; `min-h-[44px] min-w-[44px]` para targets de texto. | `web/src/components/vanta/*.tsx` | 🟢 2-4h | ✅ |
| ~~`INV-016`~~ | ~~**🔍 Motion-duration tokens — auditoría** — Verificar si existe un sistema de tokens de animación consistente. **✅ COMPLETADA 2026-08-03** (Auditoría + propuesta, sin código). **Hallazgos:** (1) **NO existen tokens de duración/easing.** `globals.css` `@theme inline{}` solo define colores + fuentes; no hay `--duration-*` ni `--ease-*`. Las únicas `transition|animation-duration` son el override `prefers-reduced-motion` (0.01ms, líneas 435-437). El easing `cubic-bezier(0.2,0.8,0.2,1)` se repite hardcodeado en 4 lugares (page-transition, reveal, latency-comparator, faq) — candidato a token `--ease-default`. (2) **Inventario de duraciones hardcodeadas:** *framer-motion* (2 files): `page-transition.tsx:26`→0.28s, `latency-comparator.tsx:323`→0.4s, `:367`→0.5s. *`Reveal` (NO framer-motion — CSS transition vía IntersectionObserver en `useReveal`): default `duration=600ms` + `delay` (default 0, callers pasan 40/60/80/100/120/150/180/200/240, staggers `i*40/60/80`). *animejs* (mark-classic:70→400, :87→2400 loop; mark-cta:250,300,350,400,400,500,500). *Tailwind `duration-*`: 75(1),150(1),200(13),300(6),500(1),1000(1) — parte en dead code (navbar.tsx, hero-mark-interactive.tsx). (3) **Esquema propuesto:** CSS vars en `@theme`: `--duration-fast:150ms; --duration-normal:300ms; --duration-slow:500ms` + `--ease-default:cubic-bezier(0.2,0.8,0.2,1)`. framer-motion/animejs NO consumen CSS vars en `duration` — exportar **mapa JS** `web/src/lib/motion.ts`: `export const MOTION={duration:{fast:.15,normal:.3,slow:.5},ease:[.2,.8,.2,1]}`. `Reveal` SÍ puede leer CSS vars directamente vía `transitionDuration`/`transitionDelay`. **Tabla reemplazo:** page-transition .28→normal(.3); latency .4/→.5 y .5→slow; Reveal delay→`--duration-fast`; stagger `i*80`→`motion.delay(i)` con base fast; Tailwind `duration-200`→`duration-fast`(13), `duration-300`→`duration-normal`(6, matchean token), `duration-500`→`duration-slow`; animejs settle ms→`.slow`/`.normal`, loop 2400 queda (ambient intencional). Recomendación: centralizar en `motion.ts` y dejar CSS vars solo para `Reveal`.~~ | ~~`web/src/app/globals.css`, `web/src/components/vanta/page-transition.tsx`, `web/src/components/vanta/reveal.tsx`~~ | ~~🟢 1-2h~~ | ✅ |

### ⚙️ CI & Tooling — Auditorías

| ID | Descripción | Archivos | Esfuerzo | Prio |
|----|-------------|----------|----------|------|
| ~~`INV-017`~~ | **✅ 🔍 sccache en CI — investigación** — Los builds Rust en CI compilan desde cero cada vez (~8-10 min). sccache podría cachear compilaciones entre runs. **Alcance:** (1) Investigar compatibilidad de `sccache` con GitHub Actions + `Swatinem/rust-cache`, (2) Evaluar si son complementarios o redundantes, (3) Diseñar integración mínima (instalar sccache + configurar `RUSTC_WRAPPER`), (4) Medir impacto estimado en tiempo de CI. **Sin implementación — solo investigación + propuesta.** | `docs/Investigaciones/INV-017-sccache-ci.md` | 🟢 2-4h | 🟡 | ✅ Hecho |

> **Items removidos (1):** NUEVO-20 (Dockerfile ya existe en raíz del repo — multi-stage, Rust 1.94)

---

## Phase 9: 📚 Old Docs Rescue — Reference Catalog

> Recuperado de `VANTADB DOC OLD` (~280 archivos .md analizados vía 21 sub-agentes).
> **Total:** 21 items, **13 activos** (8 ✅ removidos a progreso). **Estado:** 1 ⚠️ parcial, 1 ❌ pendiente, 2 ❌ justificado.
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
| ~~`OLD-03`~~ | ~~**Chaos testing (Jepsen/Maelstrom)** — `ChaosTestHarness` reutilizable, 6 failpoints (wal_append, storage_insert, mmap_flush, hnsw_serialize, edge_write, snapshot_serialize), docs `docs/chaos-testing.md`.~~ | ~~🟡 2-3 sem~~ | ~~✅ COMPLETADA~~ | Docker. WAL shipping existente | 🗺️ Roadmap |
| ~~`OLD-08`~~ | ~~Life Insurance / snapshots hard-link — `SnapshotManager` + `FsSnapshot` con hard-link POSIX, `StorageEngine::create_snapshot()`/`list_snapshots()`, CLI, VantaEmbedded API. +failpoint `snapshot_create_fail`. Tests: instant, multiple, independence.~~ | ~~🟡 3-4d~~ | ~~✅ COMPLETADA~~ | Ninguna. Solo syscalls POSIX |
| ~~`OLD-09`~~ | ~~Olvido Bayesiano (hit decay) — `EvictionPolicy` ahora soporta `BayesianDecay`: score Beta-Binomial α/(α+β), threshold configurable, 31 tests. Feature-gated `bayesian_decay`.~~ | ~~🟡 3-4d~~ | ~~✅ COMPLETADA~~ | Ninguna. `EvictionPolicy` existe |
| ~~`OLD-10`~~ | ~~Sinapsis eléctrica (index-free adjacency) — `edge_index.rs` usa DashSet, no index-free adjacency nativa~~ | ~~🟡 1 sem~~ | ~~❌ Ya existe: `UnifiedNode.edges: Vec<Edge>` es index-free adjacency nativa. EdgeIndex es auxiliar para cascade delete~~ | Post-HNSW multi-capa |
| ~~`OLD-11`~~ | ~~**CLI/TUI interactivo** — `vantadb tui` con ratatui + crossterm. 3 modos: Dashboard (stats engine), Monitor (queries live), REPL (queries interactivas con historial). Feature-gated `tui`.~~ | ~~🟡 1-2 sem~~ | ~~✅ COMPLETADA~~ | Ninguna. Proyecto aparte |
| ~~`OLD-12`~~ | ~~**Pilot program formal** — `docs/operations/PILOT_PROGRAM.md` actualizado (9 secciones), +3 templates: agreement, feedback, onboarding checklist.~~ | ~~🟡 1 sem~~ | ~~✅ COMPLETADA~~ | PyPI publicado |
| ~~`OLD-14`~~ | ~~MessageThread / GcWorker para agentic chat — `GcWorker` en `src/gc.rs` existe, MessageThread no~~ | ~~🟡 1 sem~~ | ~~✅ COMPLETADA~~ | Ninguna. `GcWorker` existe |
| ~~`OLD-16`~~ | ~~**WAL rotation a 256MB** — `WalWriter::try_auto_rotate()` en `append()`/`batch_append()`. 3 tests (trigger, no-trigger, data preservation). 52/52 WAL tests pass.~~ | ~~🟢 1d~~ | ~~✅ COMPLETADA~~ |
| ~~`OLD-19`~~ | ~~**Rehidratación desde shadow archive** — `VantaEmbedded::recover_archived_nodes()`, MCP tool `rehydrate`, Python binding. Conecta `StorageEngine::recover_archived_nodes()` (6 tests existentes) a SDK público.~~ | ~~🟡 1 sem~~ | ~~✅ COMPLETADA~~ | OLD-07 (AutoHot/Cold tiering) |
| ~~`OLD-20`~~ | ~~Contextual Priming (cache warming predictivo) — ✅ COMPLETADA. Auto-decay cada 1000 eventos, métricas exportables, co-access tracking ya conectado en hot path.~~ | ~~🟢 2-3d~~ | ~~✅ COMPLETADA~~ | ~~Ninguna~~ |
| ~~`OLD-21`~~ | ~~**CP-Index formal (query routing inteligente)** — `CostEstimator::select_index_strategy()` (Flat ≤ flat_threshold, IVF ≥ 10K, HNSW default; respeta config explícita). Conectado en `vector_memory_search` (métrica `record_vector_index_routing`). Admission budget de `Executor::execute_plan` ahora usa `ResourceGovernor::estimate_plan_cost` (removido dead_code) en vez de MIB fijo. 6 tests nuevos, 1816 tests ✅.~~ | ~~🟡 1 sem~~ | ~~✅ COMPLETADA~~ | COMP-028 ✅ |

---

## Phase 10: 🏗️ Competitive Features — Catalog

> **Fuente:** Análisis de 27 archivos de `VANTADB DOC OLD/` (9 vector DBs + 8 graph DBs + 10 arquitectura).
> **Total:** 30 items, **18 activos.** 12 ✅ implementados removidos a progreso: COMP-001 (SQ8/PQ), COMP-002 (HNSW persist), COMP-003 (in-filter), COMP-004 (bitset), COMP-005 (params), COMP-006 (Edge Label Interning), COMP-007 (inline u128), COMP-010 (auto-embedding), COMP-011 (CRUD tombstones), COMP-015 (hybrid pipeline), COMP-018 (Double-linked chains), COMP-020 (RRF fusion), COMP-030 (survival mode).
> **Reportes completos:** `docs/audit-reports/competitive-features-consolidated-report.md`, `docs/audit-reports/deep-analysis-{vector,graph,arch}.md`

### 🔴 Alta — Features competitivas críticas para adopción

| ID | Feature | Esfuerzo | Estado | Dependencias |
|----|---------|----------|--------|--------------|
| ~~`COMP-006`~~ | ~~**Edge Label Interning (u32 label_id)** — `edge_label` es `String`, no u32 internado~~ | ~~🟢 ~2d~~ | ~~✅ COMPLETADA~~ | ~~Ninguna~~ |

### 🟠 Media-Alta — Features competitivas importantes

| ID | Feature | Esfuerzo | Estado | Dependencias |
|----|---------|----------|--------|--------------|
| ~~`COMP-008`~~ | ~~**Pluggable index engine (VecIndex trait)** — `VecIndex` trait definido con `search`/`add`/`len`/`estimate_memory_bytes`. Implementado para CPIndex (HNSW) e IvfIndex. `vector_memory_search` usa `engine.vec_index()`. 1679 tests ✅. 🟡 1-2 sem~~ | ~~✅ COMPLETADA~~ | ~~Pre-COMP-027~~ |
| ~~`COMP-009`~~ | ~~**Binary bulk import (5-10x faster than INSERT)** — `bulk_import_stream()` + `bulk_import_file()` + `bulk_import_bytes()` en core Rust, Python, WASM. Formato `VDBJSON\n` header + serde_json body. Bypass per-record validation, batch commit, 3 tests. ✅ COMPLETADA~~ | ~~🟢 3-4d~~ | ✅ | Ninguna |
| ~~`COMP-010`~~ | ~~**Auto-embedding (embedding function abstraction)** — Trait `EmbeddingProvider` + `OllamaProvider` + `OpenAIProvider`. Factory `get_embedding_provider()` vía `VANTA_EMBEDDING_PROVIDER`. ✅ COMPLETADA | 🟡 1-2 sem~~ | ~~✅ COMPLETADA~~ | ~~DRV-123~~ |
| ~~`COMP-012`~~ | ~~**RoaringBitmaps for metadata indexing** — `FilterBitset` migrado a `croaring::Bitmap`. 19/19 tests ✅, serializado ~10× más pequeño. `DiskNodeHeader` intacto (u128).~~ | ~~🟡 1 sem~~ | ~~✅ COMPLETADA~~ | Pre-COMP-003 ✅ |
| ~~`COMP-013`~~ | ~~**Segment optimizer pipeline (Vacuum/Merge/Index)** — `PipelineMode`, `VacuumReport`, `MergeReport`, `PipelineReport`, `SegmentOptimizerConfig`, `vacuum()`, `merge_segments()`, `run_pipeline()` + SDK API + 77 tests. ✅ COMPLETADA~~ | ~~🟡 1-2 sem~~ | ~~✅ COMPLETADA~~ | ~~COMP-004, COMP-011~~ |
| ~~`COMP-014`~~ | ~~**FreshHNSW (background repair de enlaces huérfanos)** — `repair_orphan_links()` en CPIndex (three-phase: snapshot → scan → repair). `FreshHnswReport`, `PipelineMode::FreshHnswOnly`, pipeline phase, 4 tests. Fix: deadlock DashMap. ✅ COMPLETADA~~ | ~~🟡 1 sem~~ | ~~✅ COMPLETADA~~ | ~~COMP-004, COMP-011~~ |
| ~~`COMP-016`~~ | ~~**Supernode mitigation (indexed relationships)** — `label_index: HashMap<u32, Vec<u128>>` en `UnifiedNode`. `bfs_traverse_filtered`/`dfs_traverse_filtered` en `GraphTraverser`. `graph_bfs_filtered`/`graph_dfs_filtered` en SDK, WASM, Python. 6 tests. ✅ COMPLETADA~~ | ~~🟢 3-5d~~ | ~~✅ COMPLETADA~~ | ~~COMP-006~~ |
| ~~`COMP-017`~~ | ~~**Accumulators for parallel graph algorithms** — `GraphAccumulator` con `AtomicU64` lock-free, `traverse_with_accumulator` en `GraphTraverser`, SDK API, 6 tests. ✅ COMPLETADA~~ | ~~🟡 1-2 sem~~ | ~~✅ COMPLETADA~~ | Ninguna |
| ~~`COMP-018`~~ | ~~Double-linked relationship chains — Relaciones dirigidas simples, sin doble enlace — **✅ COMPLETADA** — Rust SDK (4 métodos con direction param), bindings WASM + Python, 33 graph tests pasan~~ | ~~🟡 1-2 sem~~ | ~~✅~~ | COMP-006 |
| ~~`COMP-019`~~ | ~~**Binary protocol (rkyv/FlatBuffers over gRPC)** — ❌ **WONTFIX** (ADR `COMP-019-binary-protocol-wontfix.md`). gRPC contradice posicionamiento embedded-first. rkyv ya cubre serialización interna. Sin demanda ni dependencias.~~ | ~~🟡 1-2 sem~~ | ~~✅ Cerrado — WONTFIX~~ | Ninguna |
| ~~`COMP-021`~~ | ~~**Temporal edges (timestamp-aware relationships)** — `Edge.created_at_ms` con custom `Deserialize` para backward-compat postcard, `bfs_traverse_filtered`/`dfs_traverse_filtered` con `time_range`, `add_edge` con `created_at_ms`, bindings Python/WASM/TS, 13 tests (7 unit + 6 certification). ✅ COMPLETADA~~ | ~~🟡 1 sem~~ | ~~✅ COMPLETADA~~ | ~~Ninguna~~ |
| ~~`COMP-022`~~ | ~~**Graph Data Science library (PageRank, centrality)** — `GraphDataScience` con `page_rank()` + `degree_centrality()`, SDK API, Python bindings, 7 tests. ✅ COMPLETADA~~ | ~~🟡 2-3 sem~~ | ~~✅ COMPLETADA~~ | ~~COMP-017~~ ✅ |
| ~~`COMP-023`~~ | ~~**3 filtering strategies (pre/post/in-index)** — `FilterStrategy::PreFilter/InFilter/PostFilter` con selector por joint selectivity. PreFilter (< 1%): scan+brute-force. InFilter (1-10%): query_mask via HNSW. PostFilter (≥ 10%): vector→filter. 1589 tests ✅.~~ | ~~🟡 1-2 sem~~ | ~~✅ COMPLETADA~~ | COMP-003 ✅, COMP-012 ✅, COMP-028 (diferida) |
| ~~`COMP-024`~~ | ~~**ACORN algorithm (second-hop filtered search)** — `acorn_expansion` param en + `search_layer()` con 2-hop expansion. 3 tests: exp | ~~🟡 1-2 sem~~ | ~~✅ COMPLETADA~~ | COMP-003 ✅, COMP-023 ✅, COMP-012 ✅ |

### 🟡 Medio — Features de madurez y ecosistema

| ID | Feature | Esfuerzo | Estado | Dependencias |
|----|---------|----------|--------|--------------|
| ~~`COMP-025`~~ | ~~**JSON shredding (dynamic schema to columns)** — Phase 1: schema inference + columnar storage + filter integration. Phase 2: typed comparison filters (Gt/Lt/Gte/Lte/Neq). `matches_shredded` con 6 operadores, 13 tests.~~ | ~~🟡 2-3 sem~~ | ~~✅ COMPLETADA~~ | Ninguna |
| ~~`COMP-026`~~ | ~~**Multi-level LSM compaction (L0→L1→L2→L3)** — SegmentRegistry, compact_level(), PipelineMode::CompactOnly/L0Only in run_pipeline. 13+ archivos modificados. `cargo check -p vantadb` ✅~~  | ~~🟡 1-2 sem~~ | ~~✅ COMPLETADA~~ | COMP-013 |
| ~~`COMP-027`~~ | ~~**Multiple index types (IVF, DiskANN, SCANN)** — FlatIndex (brute-force), DiskAnnIndex (Vamana graph + robust pruning), ScannIndex (SQ8 scalar quantization). IndexType enum extendido: `Flat`, `DiskAnn`, `Scann`. `create_index()` factory. 15 tests pasan.~~ | ~~🟠 5-10d~~ | ~~✅ COMPLETADA~~ | ~~COMP-008~~ |
| ~~`COMP-028`~~ | ~~**Semantic Cost Estimator (SCE)** — módulo `src/cost_estimator.rs` unificado: `CostEstimator::selectivity`/`estimate_operator`/`estimate_plan`, `FilterStrategy` movido de search, `select_filter_strategy` delegada. CBO + `ResourceGovernor::estimate_plan_cost` (unwired, consumido por OLD-21). `get_estimated_selectivity` delega (firma pública intacta, 21 callers). 4 tests nuevos, 1776 tests ✅. Commit `f7cb46e4`.~~ | ~~🟡 2 sem~~ | ~~✅ COMPLETADA~~ | DRV-121/122 ✅ |
| ~~`COMP-029`~~ | ~~**Node.js/TS bindings via napi-rs** — **Implementado (2026-08-02):** crate `vantadb-node` (standalone, `vantadb_native` cdylib) como backend ADICIONAL a WASM. API isomórfica con `vantadb-ts`. Persistencia real (fjall/WAL/fsync) — WASM no puede. Tests vitest 3/3: put/get, persistencia cross-reconnect, search ordenado por score. Browser se queda con WASM.~~ | ~~🟡 2-3 sem~~ | ~~✅ Implementado~~ | ~~Ninguna~~ |

---

## Phase 11: 🐙 GitHub Issues — Backlog de Issues Abiertos

> Issues abiertos en `ness-e/Vantadb` convertidos a tareas del backlog (2026-08-01).
> **Total:** 14 tareas (issues #119–#144). Todos son `good first issue`.
> ⚠️ **Recordatorio obligatorio:** al completar cada tarea, **revisar el resultado contra el issue y cerrarlo en GitHub** (`gh issue close <NUM> --repo ness-e/Vantadb`), confirmando que se cumple su Definition of Done.
> Estado real se actualiza en la columna **Estado**.

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `GH-144` | **i18n: Traducciones ES para showcase page** — Auditar `web/src/app/showcase/page.tsx` y completar claves de diccionario i18n sin traducción al español. **Context:** la página showcase tiene claves i18n que pueden faltar en ES. **Task:** auditar y agregar traducciones ES para las claves de showcasePage. **DoD:** todas las claves `showcasePage.*` con traducción EN y ES; sin warnings de traducción faltante. **Cierre:** revisar y cerrar issue #144. | `web/src/lib/dictionaries.ts` | 🟢 2-3h | 🟡 | ❌ Desde cero |
| ~~`GH-143`~~ | **✅ ci: Acelerar CI con sccache y paralelización** — El pipeline CI corre build/test/clippy pero puede optimizarse. **Task:** (1) habilitar `sccache` para cachear compilación Rust, (2) evaluar reemplazar `cargo build` por `cargo check` donde baste, (3) paralelizar jobs independientes. **DoD:** CI ≥20% más rápido; todos los checks existentes pasan. **Cierre:** revisar y cerrar issue #143. Relacionado con `INV-017` (sccache investigación). | `.github/actions/rust-setup/action.yml`, `.github/workflows/ci-rust-10.yml` | 🟡 2-4h | 🟡 | ✅ Hecho |
| ~~`GH-142`~~ | **✅ ci: Smoke tests de examples en CI** — Workflow `ci-examples-12.yml`: 4 examples Rust (`cargo run --example`) + 10 Python (wheel maturin local, 1 step por example, sin `continue-on-error`). Se repararon 7 examples Python con drift de API (`db.list`→`list_memory`, `query_vector=None`→`[]`). Todos pasan. Migrada a progreso. | `.github/workflows/` (nuevo job) | 🟡 2-4h | 🟠 | ✅ Hecho |
| `GH-141` | **docs: Documentar integración webhook GitHub→Discord** — El webhook de Discord envía eventos push/PR/release a #announcements pero no está documentado. **Task:** (1) qué eventos se trackean, (2) cómo agregar nuevos tipos de evento, (3) dónde se configura (server settings). **DoD:** integración documentada con tipos de evento y mapeo de canales; instrucciones para agregar eventos. **Cierre:** revisar y cerrar issue #141. | `docs/discord/server-config.md` (sección Integrations) | 🟢 1-2h | 🟢 | ❌ Desde cero |
| `GH-140` | **chore: Auditar y eliminar CSS no usado** — `web/src/app/globals.css` puede tener clases sin uso. **Task:** (1) encontrar clases no referenciadas en componentes, (2) eliminar CSS muerto, (3) verificar sin regresiones visuales. **DoD:** ≥10% reducción de tamaño CSS; sin cambios visuales; método de auditoría documentado. **Cierre:** revisar y cerrar issue #140. | `web/src/app/globals.css` | 🟡 2-4h | 🟢 | ❌ Desde cero |
| `GH-139` | **feat: GIF demo animado para README** — El README tiene PNG estático. **Task:** crear GIF animado mostrando (1) `pip install vantadb-py`, (2) REPL Python con CRUD, (3) resultado de hybrid search. Herramientas: vhs/asciinema/terminal-to-gif. **DoD:** GIF <5MB; muestra workflow realista; README lo muestra correctamente. **Cierre:** revisar y cerrar issue #139. | `README.md` (hero), `assets/` (nuevo gif) | 🟡 2-3h | 🟢 | ❌ Desde cero |
| `GH-132` | **feat: Badge Google Colab + notebook quickstart** — Muchos devs descubren proyectos vía Colab. **Task:** (1) crear `examples/colab/vantadb_quickstart.ipynb` (install, CRUD básico, hybrid search), (2) agregar badge "Run in Colab" a README. **DoD:** notebook corre end-to-end en Colab; badge renderiza en README. **Cierre:** revisar y cerrar issue #132. | `examples/colab/vantadb_quickstart.ipynb` (nuevo), `README.md` | 🟡 2-4h | 🟢 | ❌ Desde cero |
| `GH-131` | **docs: Documentar integración mem0 en README** — Existe `examples/python/mem0_integration.py` sin documentar. **Task:** sección mem0 en README: descripción breve, snippet de código, link al ejemplo completo. **DoD:** integración mem0 documentada; snippet verificado funcionando. **Cierre:** revisar y cerrar issue #131. | `README.md` | 🟢 1-2h | 🟢 | ❌ Desde cero |
| `GH-129` | **feat: Ejemplo de integración Semantic Kernel** — Existe `examples/python/semantic_kernel_memory.py` sin conectar a docs/README. **Task:** (1) sección SK en README con snippet, (2) verificar con último SDK de Semantic Kernel, (3) agregar imports/setup faltantes. **DoD:** integración documentada; ejemplo verificado; CI corre smoke test del ejemplo. **Cierre:** revisar y cerrar issue #129. | `README.md`, `examples/python/semantic_kernel_memory.py` | 🟡 2-3h | 🟢 | ❌ Desde cero |
| `GH-128` | **docs: Ejemplo retriever DSPy en README** — Existe `examples/python/dspy_retriever.py` sin mencionar en README. **Task:** sección DSPy: descripción breve, snippet de uso básico, link al ejemplo completo. **DoD:** integración DSPy documentada; snippet preciso y testeado. **Cierre:** revisar y cerrar issue #128. | `README.md`, referencia `examples/python/dspy_retriever.py` | 🟢 1-2h | 🟢 | ❌ Desde cero |
| ~~`GH-124`~~ | **✅ docs: Ejemplos doc-test para API pública Rust** — 7 doc-tests agregados (`open`, `open_with_config`, `put`, `get`, `delete`, `search`, `VantaConfig`) + 2 doc-tests rotos reparados. `cargo test --doc` 11/11. Migrada a progreso. | `src/lib.rs` o módulos relevantes | 🟡 3-5h | 🟡 | ✅ Hecho |
| `GH-123` | **docs: Corregir typos y links rotos en docs** — `docs/` acumuló typos, links rotos y referencias desactualizadas en 167+ archivos. **Task:** (1) correr spell checker en `.md` de `docs/`, (2) verificar links internos, (3) corregir versiones desactualizadas. **DoD:** typos corregidos; links internos resuelven; referencias de versión actualizadas. **Cierre:** revisar y cerrar issue #123. | `docs/**` | 🟡 2-4h | 🟢 | ❌ Desde cero |
| ~~`GH-122`~~ | **✅ docs: Docstrings en API pública del Python SDK** — 12/12 métodos de `VantaDB` documentados (Args/Returns/Raises/ejemplo runnable ` ```python `) en `vantadb-python/src/lib.rs`, visibles vía PyO3. Docstring de clase con constructor. check/fmt/clippy clean. Migrada a progreso. | `vantadb-python/src/lib.rs` | 🟡 3-5h | 🟡 | ✅ Hecho |
| `GH-119` | **docs: Guía de migración Vectara → VantaDB** — Vectara cerró su tier self-service en 2026; muchos equipos buscan alternativas local-first. **Task:** crear `docs/tutorials/migrate-from-vectara.md` cubriendo: diferencias de arquitectura (hosted vs embedded), exportar corpus (endpoint `corpus-export`), re-embedding (vectores Boomerang no portables), mapeo de API (corpus Vectara → namespace VantaDB). **DoD:** guía cubre workflow completo de migración; incluye ejemplos Python funcionales. **Cierre:** revisar y cerrar issue #119. Material de research: `docs/audit-reports/vectara-competitive-research-2026-07-27.md`. | `docs/tutorials/migrate-from-vectara.md` (nuevo) | 🟡 1-2d | 🟠 | ❌ Desde cero |

---

## Referencias Cruzadas

- **RC items:** `docs/reviews/PROJECT_FULL_REVIEW_2026-07-13.md` (generado por `vantadb-full-review` skill)
- **REV items:** `docs/reviews/2026-07-13-full-review.md`
- **DRV findings:** `docs/plans/2026-07-15-cross-ref-docs-vs-code.md` + `docs/audit-reports/cross-ref-wave3-final-report.md`
- **OLD items:** `docs/REPORTE_EVALUACION_COMPLETO.md` secciones 6 y 7 — ~280 archivos VANTADB DOC OLD analizados
- **COMP items:** `docs/audit-reports/competitive-features-consolidated-report.md` + `docs/audit-reports/deep-analysis-{vector,graph,arch}.md` — 27 archivos, 172 features, top 30 priorizados

=== RECITATION ===
Session: 2026-07-28 — SDK Gap Audit + Recovery Plan
Campaign: auditoría-sdk-2026-07-28
Objetivo: Investigar 12 puntos sobre SDK/CLI missing features contra código real y git history.
Estado: ✅ COMPLETED
Hallazgo clave: delete_by_filter(), count(), similar_to_key() NUNCA existieron como SDK — solo CLI handlers eliminados en AUD-09 (e9371ea8). Multi-namespace solo en tipos de output report.
Outputs:
- `docs/plans/2026-07-28-recovery-plan.md` — plan detallado (37KB, 11 tasks, 4 fases)
- `docs/Backlog.md` Phase 8 — SDK-01 a SDK-05 existentes + REC-001, REC-007–010, REC-999 agregados (líneas 152-162)
Corrección al plan: SDK-01/02/03/04/05 YA estaban en backlog (no REC-002/003/004/005/006).
IDs finales: REC-001 (foundation types), REC-007 (WAL CLI), REC-008 (backup design), REC-009 (PQ analysis), REC-010 (py.typed), REC-999 (progreso fix)
Próxima acción sugerida: Ejecutar REC-010 primero (🟢 30min), después REC-001 (foundation types)
Contrato: Plan recuperación + backlog actualizados. Plan referencias REC IDs internas, backlog usa SDK-XX + REC-XX.
=== END RECITATION ===

