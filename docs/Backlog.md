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
> **Completed tasks:** `docs/CHANGELOG.md` + `docs/progreso/README.md`
> **Verification method:** All items cross-checked against actual codebase (Jul 24, 2026). 167 items verified, ~25 items removed (stale/resolved), ~25 descriptions corrected, P9/P10 statuses updated to reflect real implementation state.
> **Total open items:** ~120
> **Origen docs-audit:** `docs/strategy/ROADMAP.md`, `docs/progreso/bitacora.md`, `docs/reviews/FULL_CODEBASE_AUDIT_2026-07-11.md`, `docs/reviews/analisis_proyecto.md`, `docs/operations/PERFORMANCE_TUNING.md`, `docs/operations/REPO_CHECKLIST.md`, `docs/architecture/STORAGE_VERSIONING.md`, `docs/plans/2026-07-13-workflow-repair-campaign.md`, `docs/Investigaciones/cargo-check-optimizacion.md`, `docs/discord/todo.md`

---

## Exec Summary

| Phase | Items | Est. Effort | Priority |
|-------|-------|-------------|----------|
| **P0** 🚀 Release Blockers | 1 (+6 ✅) | ~2-3d | 🔴 Bloqueante |
| **P1** 🛡️ Security & Critical | 0 (+1✅ 2🔵 1❌) | ~4-6d | 🔴 Bloqueante |
| **P2** ⚡ Quick Wins Técnicos | 15 | ~1-2d (paralelo) | 🟠 Alta |
| **P3** 🧪 Test Coverage (adapters) | 7 | ~4-6h c/u | 🟠 Alta |
| **P4** 🔧 Engineering Health | 10 | ~2-4 semanas | 🟡 Media |
| **P5** 📖 Docs & Community | 11 | ~1-2 semanas | 🟡 Media |
| **P6** 🚀 Launch Campaign | 10 | ~1-2 semanas | 🟡 Media |
| **P7** 🌐 WASM & Performance | 2 | ~1 semana | 🟡 Media |
| **P8** 🔮 Post-Launch & Enterprise | 13 | ~3-5 semanas | 🔵 Futuro |
| **P9** 📚 Old Docs Rescue (reference) | 21 | — | 📖 Referencia |
| **P10** 🏗️ Competitive Features (catalog) | 30 | — | 🗺️ Roadmap |

> **Items removidos (25):** VFY-012, NUEVO-15, DRV-126, DRV-129, VFY-002, SEC-14, DRV-039, VFY-005, VFY-008, VFY-009, REV-013, DRV-060/064/066/072/075-077/080/081/083/084/088/090/093/094/097/101/108/114 (crates de integración nunca implementados), DRV-078/082/089/095/100/113/128, NUEVO-11/12/19, BENCH-01, NUEVO-20, OLD-06

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
| ~~`DEVOPS-15`~~ | ~~**✅ COMPLETADA.** Default features reducidas de 9 a 3: `["arrow", "fjall", "advanced-tokenizer"]`. Se removieron `cli`, `sysinfo`, `memmap2`, `fs2`, `prometheus`, `rayon` del default.~~ | ~~`Cargo.toml:89`~~ | 🟡 1d | ✅ |
| ~~`DEVOPS-12`~~ | ~~**Production PyPI signing pipeline** — **✅ COMPLETADA.** OIDC Trusted Publishing + actions/attest-build-provenance + gh attestation verify.~~ | ~~CI config~~ | 🟡 1-2d | ✅ |
| `DEVOPS-10` | **🔵 DEFERIDO (ponytail: nice-to-have pre-1.0). Firma de binarios Windows (SmartScreen)** — Sin signtool ni Azure. SHA256 + .zip ya dan integridad básica. Agregar cuando el release público lo requiera. Step YAML preparado en task file. | `release-binaries-63.yml` | 🟡 2-3d | 🔵 |
| ~~`REV-014`~~ | ~~**✅ COMPLETADA.** `target-branch: develop` agregado a los 4 ecosystems (cargo, npm, github-actions, docker) en dependabot.yml.~~ | ~~`.github/dependabot.yml`~~ | 🟢 15min | ✅ |
| ~~`DRV-045`~~ | ~~**Test setup factory duplicado** — **✅ COMPLETADA.**~~ | ~~`vantadb-server/tests/`~~ | 🟢 30min | ✅ |
| ~~`DRV-125`~~ | ~~**✅ COMPLETADA (pre-existente).** 21 tests Miri cubren los ~30 unsafe blocks en src/index/: 5 en distance.rs (f32x8/16 kernels + SQ8 + dispatches), 3 en graph.rs (HNSW build/search), 6 en search.rs (search_layer + select_neighbors), 7 en serialize.rs (roundtrips). Job Miri en CI ya no es no-op.~~ | ~~`src/index/*.rs`~~ | 🟡 1-2d | ✅ |

---

## Phase 1: 🛡️ Security & Critical Infra

> Items que protegen la integridad del sistema y permiten despliegue seguro en producción.

| ~~`DRV-054`~~ | ~~**✅ COMPLETADA.** read_axioms extraído a const + `resolve_axioms()` con fallback a storage.~~ | ~~`vantadb-mcp/src/lib.rs:77-82`~~ | 🟢 30min | ✅ |
| ~~`DRV-124`~~ | ~~**🔵 DEFERIDO** (triage: bloqueado por Apple Developer Account $99/yr, no verificable). macOS code signing/notarization missing~~ | CI config | 🟡 2-3d | 🔵 |
| ~~`DRV-127`~~ | ~~**🔵 DEFERIDO** (ponytail: WAL funciona sin encrypt, enterprise feature pre-1.0). WAL encryption~~ | ~~`src/storage/wal.rs`, `src/storage/vfile.rs`~~ | 🟡 2-3d | 🔵 |
| ~~`RC6`~~ | ~~**❌ SKIP** (triage: diseño intencional documentado como infalible L122-146). CryptoError propagation~~ | ~~`src/crypto.rs:124-146`~~ | 🟡 1d | ❌ |

---

## Phase 2: ⚡ Quick Wins Técnicos

> Items de 15min-4h que mejoran calidad de código, performance, y DX. Ejecutables en paralelo.

| ID | Descripción | Archivos | Esfuerzo | Prio |
|----|-------------|----------|----------|------|
| `DRV-014` | ~~**ShardedWal::batch_append() clona todos los records por shard**~~ — **✅ COMPLETADA** (`wal_sharded.rs`: reemplazado `Vec<Vec<WalRecord>>` + `record.clone()` por `append()` directo round-robin, -10 lines, 0 allocs intermedios) | `src/wal_sharded.rs:85-89` | 🟢 2h | ✅ |
| `DRV-028` | ~~**Hand-rolled LRU cache con O(n) por operación**~~ — **✅ COMPLETADA** (`convert.rs:21-77` optimizada de O(n) Vec<String> a O(1) HashMap + u64 tick) | `vantadb-python/src/convert.rs:21-70` | 🟢 30min | ✅ |
| `DRV-041` | **worker.rs Promise con serde_wasm_bindgen** — **Corregido:** _reject SÍ se invoca (línea 254). No hay serde_json round-trip (usa serde_wasm_bindgen). Descripción original no coincide | `vantadb-wasm/src/worker.rs:201-254` | 🟢 1h | 🔵 |
| `VFY-006` | **`add_node` y `remove_node` — lock contention** — **Corregido:** usa `DashMap` (locking por shard) + `AtomicUsize`/`AtomicU128` (lock-free). El único `Mutex` es `rng` para random. No bloquea lecturas como describía originalmente | `src/index/graph.rs:476-490` | 🟡 1-2d | 🟡 |
| `VFY-007` | **`remove_node` O(n²) neighbor fixup** — **Corregido:** archivo real `src/index/graph.rs` (no `core.rs`) | `src/index/graph.rs` | 🟡 1-2d | 🟢 |
| `REV-012` | ~~**HNSW `insert_lock` contention**~~ — **✅ COMPLETADA** (ponytail: no contention real medida. DashMap adecuado, Mutex<Rng> <5µs, micro-batching 64 ops/acq. thread_local RNG documentado como upgrade path si profiling lo requiere) | `src/index/graph.rs:283-291` | 🟡 1-2d | ✅ |
| `DRV-136` | ~~**vantadb-wasm monolítico — sin tree-shaking WASM**~~ — **✅ COMPLETADA** (bundle 433KB gzipped — rango normal. Fix: removido `-C lto=yes` de rustflags que rompía build WASM. Todos los levers ya activos: opt-level=s, wasm-opt -Oz, lto=thin) | `vantadb-wasm/Cargo.toml`, `.cargo/config.toml` | 🟡 2-3d | ✅ |

> **Items removidos (24):** 19 items referenciando crates nunca implementados (openai/ollama/litellm/mem0/letta/crewai/dspy/haystack/langchain/llamaindex) + 5 stale (DRV-039 ESLint ya existe, VFY-005 OperationalMetrics completo, VFY-008 WAL fsync controlado, VFY-009 ~40 inline styles no 637, REV-013 spin 0.9.9 no yanked)

---

## Phase 3: 🧪 Test Coverage (Adapters & Engine)

> Cobertura de tests para todos los adaptadores Python y módulos core sin tests unitarios.

| ID | Descripción | Archivos | Esfuerzo | Prio |
|----|-------------|----------|----------|------|
| `DRV-013` | ~~**ShardedWal — 556 líneas con 22+ tests existentes**~~ — **✅ COMPLETADA** (25 tests, ~90%+ line coverage. 1 gap medio: concurrent access no testeado explícitamente. Document-only.) | `src/wal_sharded.rs` | 🟢 2h | ✅ |
| `DRV-017` | ~~**`search.rs` / `serialize.rs`**~~ — **✅ COMPLETADA** (33+29 tests. Gap: mmap zero-copy `unsafe` path en search_layer no testeado. `MmapFull(Some)` round-trip faltante. Document-only.) | `src/index/search.rs`, `src/index/serialize.rs` | 🟢 2h | ✅ |
| `DRV-061` | ~~**OpenAI test coverage**~~ — **✅ COMPLETADA** (10 tests, happy path sólido. Error paths dependen de API externa. Document-only.) | `vantadb-openai/tests/test_openai.py:1-119` | 🟢 1h | ✅ |
| `DRV-067` | ~~**Ollama test coverage**~~ — **✅ COMPLETADA** (8 tests, adapter 1-line delegate a engine. Document-only.) | `vantadb-ollama/tests/test_ollama.py:1-79` | 🟢 1h | ✅ |
| `DRV-073` | ~~**LiteLLM test coverage**~~ — **✅ COMPLETADA** (10 tests, mejor coverage de los 3 adapters. Document-only.) | `vantadb-litellm/tests/test_litellm.py:1-78` | 🟢 1h | ✅ |
| `TEST-11` | ~~**Frontend tests**~~ — **✅ COMPLETADA** (38 Vitest + 54 Playwright. Sin cross-browser WASM — demo es "Coming Soon". Agregar cuando /demo esté vivo. Document-only.) | `web/src/` | 🟡 2-3d | ✅ |
| `TEST-12` | ~~**Security fuzzing**~~ — **✅ COMPLETADA** (4 fuzz targets + proptest cubren superficies críticas. Sin corpus guardado, sin storage API fuzz target. Document-only.) | fuzz targets en `fuzz/` | 🟡 2-3d | ✅ |

> **Items removidos (7):** DRV-078/082/089/095/100/113/128 — crates de integración o directorios nunca implementados (mem0, letta, crewai, dspy, haystack, llamaindex, governance)

---

## Phase 4: 🔧 Engineering Health & Architecture

> Items de mayor esfuerzo que mejoran la arquitectura, performance y mantenibilidad a largo plazo.

| ID | Descripción | Archivos | Esfuerzo | Prio |
|----|-------------|----------|----------|------|
| ✅ `WEB-03` | **Async WAL batching fsyncs** — `flush_all` spawns one thread per shard. **Completado** `c59e0f80` | `src/wal_sharded.rs` | ✅ 1d | 🟡 |
| ✅ `WEB-04` | **Storage format versioning (draft→implement)** — `validate_compat()` range-based check for VantaFile/HNSW/WAL. Constants made pub. **Completado** `21432104` | `docs/architecture/STORAGE_VERSIONING.md` | ✅ 3d | 🔵 |
| ✅ `VFY-004` | **`flat.rs` O(n²) en filter** — By design (DashMap scan bounded by `flat_threshold`). Comment-only. **Completado** `dd13b67d` | `src/index/flat.rs:32` | ✅ 1h | 🟡 |
| ✅ `VFY-011` | **ACID Phase 3: Snapshot isolation / MVCC** — MVCC con snapshot isolation, write-write conflict detection, concurrent txns. **Completado** (working tree) | `src/storage/engine/ops.rs` | ✅ 3-5d | 🔵 |
| ✅ `DRV-121` | **Planner CBO optimization** — Predicate pushdown (sort by selectivity) + filter elimination (identity filter sel≥1.0 skipped). **Completado** `21432104` | `src/planner.rs` | ✅ 3d | 🟠 |
| ✅ `DRV-122` | **IQL JOINs/subqueries/SQL compatibility** — SELECT/JOIN/subquery parser, NestedLoopJoin, subquery filter, planner integration. **Completado** `6449469f` | `src/query.rs`, `src/parser/mod.rs`, `src/executor.rs`, `src/planner.rs`, `tests/logic/joins.rs` | ✅ 5-10d | 🟠 |
| ✅ `DRV-123` | **Auto-embedding on INSERT (remote-inference)** — Error handling polish: `match` instead of `if let Ok`, empty text guard, `tracing::warn!` on failure. Test added. **Completado** `21432104` | `src/llm.rs`, `src/executor.rs` | ✅ 2d | 🟠 |
| ~~`DRV-130`~~ | ~~**SIFT 1M high-recall 127s bottleneck** — **✅ COMPLETADA.** T1 (SearchProfile) ✅ + fix cfg-gate. T2 (prefetch) ✅ WONTFIX. T3 (node reordering) ❌ WONTFIX.~~ | ~~`src/index/search.rs`, `benches/vfile_search.rs`~~ | ~~🟡 2-3d~~ | ✅ |
| ✅ `DRV-131` | **Missing index types beyond HNSW** — Implementado IVF Flat index con k-means. **Completado** | `src/index/ivf.rs`, `src/index/search.rs`, `src/index/serialize.rs`, `src/index/graph.rs`, `src/index/mod.rs` | 🟠 5-10d | 🔵 |
| ✅ `DOC-20` | **mdBook adoption for docs site** — Docs fragmentados, sin search unificado. `docs/book/` creado con `book.toml`, `SUMMARY.md`, `{{#include}}` stubs. **Completado** `1f9f681d` | bitacora D1, D6 | ✅ 1d | 🟡 |

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
| ~~`NUEVO-13`~~ | ~~**✅ COMPLETADA.** HNSW ef_search auto-tuning con dampening 1.5x, gauge `vantadb_auto_tune_ef`, integration test `repeated_fallbacks_increase_ef`.~~ | ~~`src/index/auto_tune.rs`, `src/metrics/core/{mod,registry}.rs`~~ | ~~🟡 3-5d~~ | ✅ |
| `NUEVO-14` | **WASM bundle size <500KB gzip** — Sin medición de bundle actual ni flags de optimización en Cargo.toml más allá de `opt-level = "s"` | `vantadb-wasm/Cargo.toml` | 🟡 1-2d | 🟡 |

> **Items removidos (4):** NUEVO-11/12 (WASM IndexedDB + multi-tab coordinación — ✅ implementados), NUEVO-19 (SourceDesign/ no existe), BENCH-01 (solo mención en backlog, sin script ni dataset)

---

## Phase 8: 🔮 Post-Launch & Enterprise

> Features para después del lanzamiento público.

| ID | Descripción | Esfuerzo | Prio |
|----|-------------|----------|------|
| `CLI-01` | **CLI polish: handlers backup/restore/doctor/stats/inspect existen pero no conectados al binary. REPL/TUI no existen** | 🟡 2-3d | 🟡 |
| `DEVOPS-HOMEBREW` | **Homebrew formula** | 🟢 4h | 🟡 |
| `DEVOPS-PY313` | **Python 3.13 wheels en CI matrix** | 🟢 2h | 🟡 |
| `DEVEX-DEMO` | **Demo app (Rust + Python)** — Phase 4.G | 🟡 2-3d | 🟡 |
| `DEVEX-EXAMPLES` | **Rust examples en `examples/rust/`** (no `docs/examples/`) | 🟢 4-6h | 🟡 |
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
> **Total:** 21 items. **Estado real tras verificación:** 10 ✅ implementados, 7 ⚠️ parcial, 4 ❌ pendiente.
> **Referencia completa:** `docs/REPORTE_EVALUACION_COMPLETO.md` secciones 6 y 7.
> **Batch file map:** ver `docs/Backlog.md` sección Tier 5 original para archivos por batch.

### 🔴 Alta — Features perdidas con alto valor de mercado

| ID | Feature | Esfuerzo | Estado | Dependencias | Prioridad |
|----|---------|----------|--------|--------------|-----------|
| `OLD-01` | **PGWire (PostgreSQL wire protocol)** — Compatibilidad con psql, pgAdmin, ecosistema PG | 🟠 2-3 sem | ❌ No implementado | Ninguna | 🗺️ Roadmap |
| `OLD-02` | **GraphRAG pipeline formal** — seed → expand → retrieve → generate context. Ejemplo en `examples/rust/graphrag.rs`, no pipeline formal | 🟡 1-2 sem | ⚠️ Parcial (ejemplo existe) | DRV-123 (auto-embedding) recomendado | 🗺️ Roadmap |
| `OLD-03` | **Chaos testing (Jepsen/Maelstrom)** — `chaos_test_wal.sh` + failpoint tests CI existen, no Jepsen formal | 🟡 2-3 sem | ⚠️ Parcial (scripts existen) | Docker. WAL shipping existente | 🗺️ Roadmap |
| `OLD-04` | **OpenTelemetry tracing** — ✅ Implementado. `src/cli_server.rs` con feature flag `opentelemetry`, OTLP exporter | 🟡 1 sem | ✅ Implementado | Feature flag independiente | 🗺️ Roadmap |

### 🟡 Medio — Valor moderado

| ID | Feature | Esfuerzo | Estado | Dependencias |
|----|---------|----------|--------|--------------|
| `OLD-07` | **AutoHot/Cold tiering (STN/LTN simplificado)** — ✅ `VantaStorageTier::Hot/Cold`, `NodeTier::Cold`, `maintenance.rs` cold migration existen | 🟡 1 sem | ✅ Implementado | Ninguna. `QuantizationGovernor` existe |
| `OLD-08` | Life Insurance / snapshots hard-link — `snapshot_certification.rs` existe, hard-link pattern no | 🟡 3-4d | ⚠️ Parcial | Ninguna. Solo syscalls POSIX |
| `OLD-09` | Olvido Bayesiano (hit decay) — `EvictionPolicy` con hit counts + recency weights, sin decay bayesiano formal | 🟡 3-4d | ⚠️ Parcial | Ninguna. `EvictionPolicy` existe |
| `OLD-10` | Sinapsis eléctrica (index-free adjacency) — `edge_index.rs` usa DashSet, no index-free adjacency nativa | 🟡 1 sem | ❌ No implementado | Post-HNSW multi-capa |
| `OLD-11` | CLI/TUI interactivo (spec 1106 líneas escrito) — CLI completo, TUI no implementado | 🟡 1-2 sem | ⚠️ Parcial (CLI OK, TUI no) | Ninguna. Proyecto aparte |
| `OLD-12` | Pilot program formal (early adopters) — `docs/operations/PILOT_PROGRAM.md` existe (solo spec) | 🟡 1 sem | ⚠️ Parcial (doc existe) | PyPI publicado |
| `OLD-13` | **Explainable ranking (explain flag)** — ✅ `debug::explain_hit()`, test `memory_euclidean_and_explainable_ranking` | 🟢 2-3d | ✅ Implementado | Ninguna |
| `OLD-14` | MessageThread / GcWorker para agentic chat — `GcWorker` en `src/gc.rs` existe, MessageThread no | 🟡 1 sem | ⚠️ Parcial (GcWorker OK) | Ninguna. `GcWorker` existe |

### 🟢 Bajo — Quick wins ~1 día

| ID | Feature | Esfuerzo | Estado |
|----|---------|----------|--------|
| `OLD-15` | **Distancia Euclidiana L2 (código SIMD ya existe)** — ✅ `DistanceMetric::Euclidean` con SIMD | 🟢 2d | ✅ Implementado |
| `OLD-16` | WAL rotation a 256MB — WAL segments existen (`wal_archiver.rs`), rotation por tamaño no | 🟢 1d | ❌ No implementado |
| `OLD-17` | **Migration guides públicos (FROM_CHROMADB, FROM_LANCEDB)** — ✅ `docs/tutorials/` con ambas guías | 🟢 1d | ✅ Implementado |
| `OLD-18` | **Query TEMPERATURE parameter (diversidad controlada)** — ✅ Parser soporta `WITH TEMPERATURE`, `governor.rs` aplica límites | 🟢 1d | ✅ Implementado |

### ⚪ Futuro / Con Dependencias

| ID | Feature | Esfuerzo | Estado | Dependencias |
|----|---------|----------|--------|--------------|
| `OLD-19` | Rehidratación desde shadow archive — `rehydration_available` en MCP, `rehydration_required` en Python SDK | 🟡 1 sem | ⚠️ Parcial | OLD-07 (AutoHot/Cold tiering) |
| `OLD-20` | Contextual Priming (cache warming predictivo) — Sin código de warming predictivo | 🟢 2-3d | ❌ No implementado | Ninguna |
| `OLD-21` | CP-Index formal (query routing inteligente) — `CPIndex` existe como struct HNSW, no query routing formal | 🟡 1 sem | ❌ No implementado | DRV-121/122 (Planner AST + IQL) |
| `OLD-22` | **Apache Arrow columnar export** — ✅ `src/columnar.rs`, `tests/logic/columnar.rs` con certificación | 🟡 3-4d | ✅ Implementado | Ninguna. `columnar.rs` existe |

---

## Phase 10: 🏗️ Competitive Features — Catalog

> **Fuente:** Análisis de 27 archivos de `VANTADB DOC OLD/` (9 vector DBs + 8 graph DBs + 10 arquitectura).
> **Total:** 30 items. **Estado real tras verificación:** 10 ✅ implementados, 5 ⚠️ parcial, 15 ❌ pendiente.
> **Reportes completos:** `docs/audit-reports/competitive-features-consolidated-report.md`, `docs/audit-reports/deep-analysis-{vector,graph,arch}.md`

### 🔴 Alta — Features competitivas críticas para adopción

| ID | Feature | Esfuerzo | Estado | Dependencias |
|----|---------|----------|--------|--------------|
| `COMP-001` | **SQ8/PQ Quantization (4x-16x compression)** — ✅ `VectorRepresentations::SQ8`, `QuantizationGovernor`, SIMD fast path | 🟡 2-3 sem | ✅ Implementado | ARC-014 (HNSW persistence) recomendado |
| `COMP-002` | **HNSW Persistence (no rebuild en startup)** — ✅ `persist_to_file()` / `load_from_file()` en `serialize.rs` | 🟡 1-2 sem | ✅ Implementado | Ninguna |
| `COMP-003` | **In-filter traversal (bitset durante HNSW walk)** — ✅ `query_mask` en `search.rs:97,233` | 🟢 ~50 líneas | ✅ Implementado | COMP-012 (RoaringBitmaps) |
| `COMP-004` | **Bitset-based filtering + soft deletes** — ✅ `FilterBitset`, `scan_bitset()`, tombstones completos | 🟢 3-5d | ✅ Implementado | Pre-ComP-011 |
| `COMP-005` | **HNSW params configurables (M, ef_construction, ef_search)** — ✅ `HnswConfig` + `auto_tune.rs` | 🟢 2-3d | ✅ Implementado | Ninguna |
| `COMP-006` | **Edge Label Interning (u32 label_id)** — `edge_label` es `String`, no u32 internado | 🟢 ~2d | ❌ No implementado | Ninguna |
| `COMP-007` | **Bitset inline u128 en UnifiedNode** — ✅ `FilterBitset` con `to_u128()`, `UnifiedNode.header.bitset: u128` | 🟡 1 sem | ✅ Implementado | Ninguna |

### 🟠 Media-Alta — Features competitivas importantes

| ID | Feature | Esfuerzo | Estado | Dependencias |
|----|---------|----------|--------|--------------|
| `COMP-008` | Pluggable index engine (VecIndex trait) — `IndexBackend` trait existe, `VecIndex` formal no | 🟡 1-2 sem | ⚠️ Parcial | Pre-COMP-027 |
| `COMP-009` | Binary bulk import (5-10x faster than INSERT) — Solo `put_batch()`, no protocolo binario | 🟢 3-4d | ❌ No implementado | Ninguna |
| `COMP-010` | Auto-embedding (embedding function abstraction) — `remote-inference` feature con Ollama, sin `EmbeddingFunction` abstracto | 🟡 1-2 sem | ⚠️ Parcial | DRV-123 |
| `COMP-011` | **HNSW CRUD con tombstones + async cleanup** — ✅ Tombstones completos + `compact_layout_bfs` | 🟡 2-3 sem | ✅ Implementado | COMP-004, COMP-014 |
| `COMP-012` | RoaringBitmaps for metadata indexing — `FilterBitset` custom, no `croaring` | 🟡 1 sem | ❌ No implementado | Pre-COMP-003 |
| `COMP-013` | Segment optimizer pipeline (Vacuum/Merge/Index) — `compact_layout_bfs` + vacío existe, pipeline formal no | 🟡 1-2 sem | ⚠️ Parcial | COMP-004, COMP-011 |
| `COMP-014` | FreshHNSW (background repair de enlaces huérfanos) — Sin repair background | 🟡 1 sem | ❌ No implementado | COMP-004, COMP-011 |
| `COMP-015` | **Hybrid Graph+Vector search pipeline** — ✅ `engine.hybrid_search()`, search routes en SDK | 🟡 2-3 sem | ✅ Implementado | COMP-005, COMP-003 |
| `COMP-016` | Supernode mitigation (indexed relationships) — Sin indexed relationships | 🟢 3-5d | ❌ No implementado | COMP-006 |
| `COMP-017` | Accumulators for parallel graph algorithms — Sin accumulators | 🟡 1-2 sem | ❌ No implementado | Ninguna |
| `COMP-018` | Double-linked relationship chains — Relaciones dirigidas simples, sin doble enlace | 🟡 1-2 sem | ❌ No implementado | COMP-006 |
| `COMP-019` | Binary protocol (rkyv/FlatBuffers over gRPC) — Solo HTTP JSON. rkyv usado internamente en serialización | 🟡 1-2 sem | ⚠️ Parcial (rkyv interno sí) | Ninguna |
| `COMP-020` | **Hybrid search with RRF (Reciprocal Rank Fusion)** — ✅ `fuse_rrf()` / `fuse_rrf_with_report()` en `planner.rs` | 🟡 1 sem | ✅ Implementado | Ninguna (BM25 existe) |
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
| `COMP-030` | **Survival Mode (backpressure + Docker OOM prevention)** — ✅ Backpressure, OOM circuit breaker, memory pressure checks, eviction | 🟡 1-2 sem | ✅ Implementado | Ninguna |

---

## Referencias Cruzadas

- **RC items:** `docs/reviews/PROJECT_FULL_REVIEW_2026-07-13.md` (generado por `vantadb-full-review` skill)
- **REV items:** `docs/reviews/2026-07-13-full-review.md`
- **DRV findings:** `docs/plans/2026-07-15-cross-ref-docs-vs-code.md` + `docs/audit-reports/cross-ref-wave3-final-report.md`
- **OLD items:** `docs/REPORTE_EVALUACION_COMPLETO.md` secciones 6 y 7 — ~280 archivos VANTADB DOC OLD analizados
- **COMP items:** `docs/audit-reports/competitive-features-consolidated-report.md` + `docs/audit-reports/deep-analysis-{vector,graph,arch}.md` — 27 archivos, 172 features, top 30 priorizados
