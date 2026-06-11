# 📋 Lista Maestra Consolidada de Tareas Pendientes — VantaDB
**Fecha:** Junio 2026 | **Versión:** 1.0 | **Fuentes:** Análisis de repo, 3 auditorías técnicas, Plan Maestro, Informe Estratégico, Plan Post-MVP, Roadmap v2, PlanDeAccion, recomendaciones, y documentación histórica completa.

> **Leyenda de símbolos:**
> - 🔴 = Bloqueante (riesgo de data-loss, corrupción o seguridad crítica)
> - 🟠 = Alto (impacto en confiabilidad, adopción o competitividad)
> - 🟡 = Medio (mejora de producto, DX o performance)
> - 🟢 = Bajo (cosmético, documentación, gobernanza)
> - ✅ = Ya implementado verificado
> - ❌ = Pendiente
> - ⚠️ = Estado contradictorio (verificar en código)
> - 🆕 = Tarea NUEVA descubierta en esta revisión histórica

---

## 🎯 RESUMEN EJECUTIVO

**Total de tareas consolidadas:** 110+ (organizadas en 11 categorías)
**Bloqueantes:** 6 | **Altas:** 18 | **Medias:** 35 | **Bajas/Deferred:** 50+

**Decisión estratégica cero (a tomar esta semana):**
¿VantaDB es librería embebida (SQLite-style) o plataforma de BD (Qdrant-style)?
**Recomendación:** Embedded-first. Archivar todo el Plan Enterprise hasta tener 1,000 usuarios reales.

---

## 🔴 CATEGORÍA 1: BLOQUEANTES CRÍTICOS (Riesgo de data-loss o corrupción)

| ID | Tarea | Estado | Evidencia | Criterio de Éxito |
|---|---|:---:|---|---|
| AUD-03 | Mecanismo RCU / Double-Buffer para `rebuild_index` | ❌ | `src/sdk.rs` sin swap atómico ni epoch | `ArcSwap<HnswIndex>` + 100 queries concurrentes durante rebuild → 0 resultados inconsistentes |
| AUD-06 | File locking multi-proceso (advisory lock) | ❌ | Sin `flock`/`O_EXCL` en snapshot | `fs2::FileExt::try_lock_exclusive()` + error claro si ya hay proceso |
| AUD-09 ⚠️ | Checksums CRC32C **por registro** en WAL | ⚠️ | Plan Maestro dice "integrado"; auditoría dice "ausente" | Verificar en `src/wal.rs` si cada registro tiene CRC32C y replay lo valida |
| AUD-02 | Crash-injection tests automatizados | ❌ | Post-MVP Roadmap lo marca prerrequisito | Job CI con `SIGKILL` aleatorio → 100/100 recoveries consistentes |
| AUD-01 | py.allow_threads consistente en PyO3 | ✅ | Verificado en `vantadb-python/src/lib.rs` | — (cerrado) |
| AUD-08 | fsync síncrono configurable en WAL | ✅ | `SyncMode` en `src/config.rs` | — (verificar si default es Always) |

---

## 🟠 CATEGORÍA 2: HARDENING DEL TEXT INDEX & HYBRID RETRIEVAL

| ID | Tarea | Estado | Origen |
|---|---|:---:|---|
| 🆕 TEX-01 | **Auditoría estructural del text index** como capacidad operativa first-class (no solo debug) | ❌ | Informe Analítico 3 |
| 🆕 TEX-02 | **Benchmark de calidad híbrida** con NDCG/MRR/Recall@k | ❌ | Estado actual después de Hybrid v1 |
| 🆕 TEX-03 | **Corpus interno de evaluación** (queries oro con expectativas) | ❌ | Estado actual después de Hybrid v1 |
| 🆕 TEX-04 | **Positions y phrase queries** (schema v3 de postings con posiciones) | ❌ | Informe Analítico 3 |
| 🆕 TEX-05 | **Snippets/highlighting** (guardar offsets o re-analizar payload) | ❌ | Informe Analítico 3 |
| 🆕 TEX-06 | **Tokenizer v3** (Unicode folding, stopwords, stemming versionado) | ❌ | Informe Analítico 3 |
| 🆕 TEX-07 | **Explainability/debug del ranking** (planner report + contadores por ruta) | ❌ | Estado actual después de Hybrid v1 |
| 🆕 TEX-08 | **BM25 TF/DF/doc length persistentes** (deduplicación actual rompe TF) | ❌ | Auditoría fase text index |
| 🆕 TEX-09 | **RRF con budgets explícitos y métricas de candidate explosion** | ❌ | Auditoría previa |

---

## 🟠 CATEGORÍA 3: PERFORMANCE & ESCALABILIDAD DEL CORE

| ID | Tarea | Estado | Origen |
|---|---|:---:|---|
| 🆕 PERF-01 | **Auditoría y corrección de HNSW multi-layer** (verificar navegación por todas las capas) | ❌ | Plan Maestro Ejecutivo T1.1 |
| 🆕 PERF-02 | **Soporte nativo de distancia Euclidiana (L2)** con SIMD | ❌ | Plan Maestro Ejecutivo T1.2 |
| 🆕 PERF-03 | **Layout antilocatario en mmap** (BFS re-ordering de nodos HNSW) | ❌ | Plan Maestro Ejecutivo T1.3 + PlanDeAccion |
| 🆕 PERF-04 | **Optimización boundary Python-Rust** (API batch queries con Rayon) | ❌ | Plan Maestro Ejecutivo T1.4 |
| 🆕 PERF-05 | **Planner con pipeline AST / LogicalPlan / PhysicalPlan** (predicate pushdown) | ❌ | Plan Maestro Ejecutivo T2.3 + Informe Estratégico §9 |
| 🆕 PERF-06 | **Versionado de formato de serialización** (header magic bytes + version) | ❌ | Plan Maestro Ejecutivo T2.4 |
| 🆕 PERF-07 | **Asignador global mimalloc v3** | ❌ | Estudio Arquitectónico, PlanDeAccion |
| 🆕 PERF-08 | **Telemetría de 3 dominios** (RSS vs HNSW lógico vs mmap mincore) | ⚠️ | Implementación existe pero cifras inconsistentes |
| 🆕 PERF-09 | **Filtro de admisión (backpressure) al 80% RAM** | ❌ | Estudio Arquitectónico |
| 🆕 PERF-10 | **MMap-backed HNSW** para datasets >RAM (SCALE-01) | ❌ | Post-MVP Roadmap |
| 🆕 PERF-11 | **Sharded-slab para concurrencia lock-free** en indexación | ❌ | PlanDeAccion FASE-05 |
| 🆕 PERF-12 | **Tests equivalencia SIMD vs escalar** (proptest, tolerancia 1e-6) | ❌ | Auditoría AUD-02 |
| 🆕 PERF-13 | **Cuántización SQ8** (f32 → int8 con SIMD) | ❌ | Estudio Arquitectónico |
| 🆕 PERF-14 | **Cuántización dinámica por ciclo de vida** (32→8→4→2 bits estilo Ebbinghaus) | ❌ | Evolución y Mejora Propuesta |
| 🆕 PERF-15 | **Deserialización zero-copy con rkyv** | ❌ | recomendaciones.md |

---

## 🟠 CATEGORÍA 4: CONCURRENCIA & RUNTIME

| ID | Tarea | Estado | Origen |
|---|---|:---:|---|
| CONC-01 | **Eliminar bloqueos síncronos en Tokio** (spawn_blocking audit) | ❌ | Plan Maestro Ejecutivo T2.1 |
| CONC-02 | **Semáforo max_blocking_threads** en VantaConfig | ❌ | ADR-003 |
| CONC-03 | **Lecturas wait-free con aarc/arc-swap** | ❌ | Evolución y Mejora Propuesta |
| CONC-04 | **Shutdown handler (SIGTERM)** en vantadb-server | ❌ | Auditoría AUD-07 |
| CONC-05 | **Validación estricta metadata FFI** (NaN, Inf, nested dicts) | ⚠️ | AUD-03/AUD-11 |

---

## 🟠 CATEGORÍA 5: RELEASE ENGINEERING & DISTRIBUCIÓN

| ID | Tarea | Estado | Origen |
|---|---|:---:|---|
| REL-01 | **Publicar core en crates.io** (no solo PyPI) | ❌ | Auditoría AUD-05 |
| REL-02 | **Pipeline wheels cibuildwheel + Sigstore + OIDC** Trusted Publishing | ❌ | Plan Maestro Ejecutivo T3.3 |
| REL-03 | **Chaos testing expandido** (1,000 iteraciones kill -9) | ❌ | Plan Maestro Ejecutivo T3.1 |
| REL-04 | **Benchmark competitivo vs LanceDB/Chroma** (ann-benchmarks) | ❌ | Plan Maestro Ejecutivo T3.2 |
| REL-05 | **Job CI nocturno de benchmarks** (artifacts JSON históricos) | ❌ | Post-MVP Roadmap BENCH-01b |
| REL-06 | **Tabla pública en README + docs/benchmarks.md** | ❌ | Post-MVP Roadmap BENCH-01c |
| REL-07 | **Datasets reales** (GloVe cosine, NQ texto) | ❌ | Auditoría AUD-03 |
| REL-08 | **Corregir `pyproject.toml`** (URLs apuntan a `ness-e/VantaDB`) | ❌ | Informe de Estado |
| REL-09 | **Hermetización del entorno Python** (target/audit-venv) | ❌ | PlanDeAccion ENV-01a |
| REL-10 | **Limpieza lints FFI** (Clippy warnings en python.rs/sdk.rs) | ❌ | PlanDeAccion ENV-01b |
| REL-11 | **Fix runner Windows** (`windows-2025-vs2026` no existe) | ❌ | Auditoría AUD-01 |
| REL-12 | **Verificación hash wheel antes de pip install** en CI | ❌ | Auditoría AUD-04 |

---

## 🟡 CATEGORÍA 6: INTEGRACIONES & ECOSISTEMA

| ID | Tarea | Estado | Origen |
|---|---|:---:|---|
| INT-01 | **Adapter LangChain** (`langchain-vantadb` → PyPI) | ❌ | Post-MVP FEAT-01a |
| INT-02 | **Adapter LlamaIndex** (`llamaindex-vantadb` → PyPI) | ❌ | Post-MVP FEAT-01b |
| INT-03 | **Adapter Mem0** (memory for AI agents, 20K stars) | ❌ | Plan Maestro Ejecutivo |
| INT-04 | **Adapter CrewAI** (multi-agent framework) | ❌ | Plan Maestro Ejecutivo |
| INT-05 | **Adapter AutoGen** (Microsoft) | ❌ | Plan Maestro Ejecutivo |
| INT-06 | **Adapter Haystack** (deepset RAG) | ❌ | Plan Maestro Ejecutivo |
| INT-07 | **Adapter LangGraph** (checkpoint store) | ❌ | Plan Maestro Ejecutivo |
| INT-08 | **Adapter Semantic Kernel** (.NET + Python) | ❌ | Plan Maestro Ejecutivo |
| INT-09 | **Adapter DSPy** (Stanford) | ❌ | Plan Maestro Ejecutivo |
| INT-10 | **Go SDK** (vía C-ABI/cbindgen) | ❌ | Plan Maestro Ejecutivo Mejora P5 |
| INT-11 | **Guía `docs/agent-integration.md`** | ❌ | Post-MVP FEAT-01d |
| INT-12 | **Servidor MCP ligero** (feature-gated) | ⚠️ | Plan Maestro dice "diferido" pero existe crate |

---

## 🟡 CATEGORÍA 7: OBSERVABILIDAD & SEGURIDAD

| ID | Tarea | Estado | Origen |
|---|---|:---:|---|
| OBS-01 | **OpenTelemetry + logs estructurados JSON** (tracing-opentelemetry) | ❌ | Plan Maestro Ejecutivo Mejora P2 |
| OBS-02 | **Trace IDs** desde Python SDK hasta LSM tree | ❌ | Plan Maestro SRE-001 |
| OBS-03 | **Backpressure / Admission filter** funcional | ⚠️ | Código existe pero AUD-03 lo marca no funcional |
| SEC-01 | **Bearer token auth** en vantadb-server | ❌ | Plan Maestro Ejecutivo T5.2 |
| SEC-02 | **TLS con rustls** (Let's Encrypt + auto-firmado) | ❌ | Plan Maestro Ejecutivo |
| SEC-03 | **Rate limiting** (tower-governor) | ❌ | Plan Maestro Ejecutivo |
| SEC-04 | **Cifrado AES-256-GCM en reposo** (Pro tier) | ❌ | Plan Maestro Ejecutivo |
| SEC-05 | **RBAC granular** por namespace | ❌ | Plan Maestro Ejecutivo |
| SEC-06 | **Audit log inmutable** con firma HMAC | ❌ | Plan Maestro Ejecutivo |
| SEC-07 | **cargo-audit + cargo-deny + SBOM** en CI | ✅ | Verificado en rust_ci.yml |

---

## 🟡 CATEGORÍA 8: DOCUMENTACIÓN & GOBERNANZA

| ID | Tarea | Estado | Origen |
|---|---|:---:|---|
| DOC-01 | **ADRs completos** (ADR-004 a ADR-007: StorageBackend, HNSW propio, Planner, Concurrencia) | ❌ | Plan Maestro Ejecutivo Mejora P6 |
| DOC-02 | **Carpeta `examples/`** con 5+ casos reales (RAG, LangChain, MCP, multi-agente, Ollama) | ❌ | Post-MVP Roadmap |
| DOC-03 | **Política Semver documentada** en README + CONTRIBUTING | ❌ | Estudio Arquitectónico |
| DOC-04 | **Corregir inconsistencia `connectome-server` vs `vanta-server`** | ❌ | Informe Estratégico Apéndice A |
| DOC-05 | **Eliminar `vantadb_data/` (64MB) trackeado en Git** | ❌ | Plan Maestro Día 1 |
| DOC-06 | **Alinear README/arquitectura/changelog** con estado Hybrid actual | ❌ | Estado actual después de Hybrid v1 |
| DOC-07 | **Regenerar tracker fuente-de-verdad** (CSV o equivalente con owner/priority/fecha) | ❌ | Auditoría analítica |
| DOC-08 | **Política Semver real del formato de índices** | ❌ | Auditoría AUD-09 |

---

## 🟡 CATEGORÍA 9: FEATURES ESTRUCTURALES AVANZADOS

| ID | Tarea | Estado | Origen |
|---|---|:---:|---|
| FEAT-01 | **Bucle AUDN (Add/Update/Delete/None)** en ingesta para curación proactiva | ❌ | Evolución y Mejora Propuesta |
| FEAT-02 | **Grafo MAGMA** (Semántica, Causal, Temporal, Entidades) | ❌ | Evolución y Mejora Propuesta |
| FEAT-03 | **Decaimiento Ebbinghaus** (curva de olvido adaptativa) | ❌ | Evolución y Mejora Propuesta |
| FEAT-04 | **Motor REM** de consolidación nocturna | ❌ | Evolución y Mejora Propuesta |
| FEAT-05 | **Hot Cache tiered storage** (LRU → immutable → persistent) | ❌ | recomendaciones.md |
| FEAT-06 | **Learned Indexes (RMI)** para metadatos escalares | ❌ | recomendaciones.md |
| FEAT-07 | **WAL batching** (checkpointing agrupado estilo OkayWAL) | ❌ | recomendaciones.md |
| FEAT-08 | **CRDTs (Observed-Remove Sets)** para sync multi-agente | ❌ | recomendaciones.md |
| FEAT-09 | **Versionado de documentos e invalidación** (política explícita) | ❌ | Informe Estratégico §9 |
| FEAT-10 | **Planner adaptativo inicial** con cardinalidad | ❌ | Informe Estratégico §19 |

---

## 🟡 CATEGORÍA 10: VANTALISP / IQL AVANZADO

| ID | Tarea | Estado | Origen |
|---|---|:---:|---|
| LISP-01 | **Transformación a Bytecode/JIT zero-copy** (opcodes sobre mmap) | ❌ | recomendaciones.md |
| LISP-02 | **Unificación multimodal en IQL** (operadores `~`, `SIGUE`) | ❌ | recomendaciones.md |
| LISP-03 | **Fuel 2.0** (vinculado a métricas hardware reales) | ❌ | recomendaciones.md |
| LISP-04 | **Lógica de rehidratación de contexto** (metacognición) | ❌ | recomendaciones.md |
| LISP-05 | **Monotonic Logic** (estilo Bloom, coordination-free) | ❌ | recomendaciones.md |
| LISP-06 | **Sandbox estricto** (sin unsafe, solo StorageEngine) | ❌ | recomendaciones.md |
| LISP-07 | **CRDTs definibles en LISP** | ❌ | recomendaciones.md |
| LISP-08 | **Primitivas de razonamiento multi-salto** | ❌ | recomendaciones.md |
| LISP-09 | **Fuzzing específico para parser LISP** | ❌ | recomendaciones.md |
| LISP-10 | **Renombrar a "VantaScript" / "Inference Logic"** | ❌ | recomendaciones.md |

---

## 🟢 CATEGORÍA 11: EXPANSIÓN DE PRODUCTO (DIFERIR hasta validación de demanda)

### VantaDB Cloud & Enterprise
| ID | Tarea | Estado | Origen |
|---|---|:---:|---|
| CLD-01 | **VantaDB Cloud Beta** (Fly.io + Bearer auth) | ❌ | Plan Maestro Ejecutivo T5.2 |
| CLD-02 | **Deck de inversores + one-pager** | ❌ | Plan Maestro Ejecutivo T5.1 |
| CLD-03 | **Programa de pilotos controlados** (3-5 usuarios reales) | ❌ | Plan Maestro Ejecutivo T3.4 |
| CLD-04 | **Case studies** (al menos 2 documentados) | ❌ | Plan Maestro Ejecutivo |

### Distribución v2.0+ (Roadmap_v2.md)
| ID | Tarea | Estado | Origen |
|---|---|:---:|---|
| ROAD-01 | **WASM Build** (wasm32-wasi browser playground) | ❌ | roadmap_v2.md v1.5 |
| ROAD-02 | **Backup/Restore** (archivo `.vantadb` + S3) | ❌ | roadmap_v2.md v1.5 |
| ROAD-03 | **Web UI Visualizador** (graph explorer + vector scatter) | ❌ | roadmap_v2.md v1.5 |
| ROAD-04 | **Bulk Import** CSV/JSON con progress bar (100k nodes/sec) | ❌ | roadmap_v2.md v1.5 |
| ROAD-05 | **Multi-model Hooks** (Ollama, vLLM, OpenAI) | ❌ | roadmap_v2.md v1.5 |
| ROAD-06 | **Monitoring Dashboard** Grafana preconfigurado | ❌ | roadmap_v2.md v1.5 |
| ROAD-07 | **Connection Pooling** con circuit breaker | ❌ | roadmap_v2.md v1.5 |
| ROAD-08 | **Schema Validation** (strict mode) | ❌ | roadmap_v2.md v1.5 |
| ROAD-09 | **Query Caching** LRU + TTL | ❌ | roadmap_v2.md v1.5 |
| ROAD-10 | **CLI syntax highlighting + `.explain`** | ❌ | roadmap_v2.md v1.0 |
| ROAD-11 | **Docker Compose** con Ollama y UI | ❌ | roadmap_v2.md v1.0 |

### Distribuido v2.0+ (roadmap_v2.md v2.0-v3.0)
| ID | Tarea | Estado | Origen |
|---|---|:---:|---|
| DIST-01 | **Raft Consensus** (openraft) | ❌ | roadmap_v2.md v2.0 |
| DIST-02 | **Hash Sharding + Cross-Shard Queries** | ❌ | roadmap_v2.md v2.0 |
| DIST-03 | **Zero-Downtime Upgrades** (rolling restart) | ❌ | roadmap_v2.md v2.0 |
| DIST-04 | **ML Cost-Based Optimizer** (micro decision tree) | ❌ | roadmap_v2.md v2.5 |
| DIST-05 | **Auto-Indexing** (detectar queries frecuentes) | ❌ | roadmap_v2.md v2.5 |
| DIST-06 | **Adaptive TEMPERATURE** | ❌ | roadmap_v2.md v2.5 |
| DIST-07 | **Query Recommendations** ("Did you mean?") | ❌ | roadmap_v2.md v2.5 |
| DIST-08 | **Anomaly Detection** (spike detection) | ❌ | roadmap_v2.md v2.5 |
| DIST-09 | **Multi-Tenant** con aislamiento real | ❌ | roadmap_v2.md v3.0 |
| DIST-10 | **Plugin Marketplace** (WASM plugins, 70/30 split) | ❌ | roadmap_v2.md v3.0 |
| DIST-11 | **Edge Federation** (sync eventual) | ❌ | roadmap_v2.md v3.0 |
| DIST-12 | **Time-Series Mode** (window functions) | ❌ | roadmap_v2.md v3.0 |
| DIST-13 | **GraphQL API** | ❌ | roadmap_v2.md v3.0 |
| DIST-14 | **CDC (Change Data Capture)** vía WebSocket | ❌ | roadmap_v2.md v3.0 |

### Open Core & Monetización
| ID | Tarea | Estado | Origen |
|---|---|:---:|---|
| BIZ-01 | **Bifurcación del workspace** (vantadb-core vs vantadb-pro) | ❌ | PlanDeAccion FASE-06 |
| BIZ-02 | **WAL Shipping P2P** (replicación descentralizada) | ❌ | PlanDeAccion FASE-09 |
| BIZ-03 | **Modelo pricing** ($49/$299/$1000+ tiers) | ❌ | Plan Maestro Ejecutivo |

### Marketing & Comunidad
| ID | Tarea | Estado | Origen |
|---|---|:---:|---|
| MKT-01 | **Demo content** (asciinema + GIF animado) | ❌ | Plan Maestro Ejecutivo T4.1 |
| MKT-02 | **Serie 3 artículos técnicos** (dev.to/Medium) | ❌ | Plan Maestro Ejecutivo T4.2 |
| MKT-03 | **HackerNews Show HN** (timing + respuestas preparadas) | ❌ | Plan Maestro Ejecutivo T4.3 |
| MKT-04 | **Servidor Discord + Good First Issues** | ❌ | Plan Maestro Ejecutivo T4.4 |

---

## ⚠️ TAREAS CON CONTRADICCIONES (Verificar en código)

| ID | Tarea | Conflicto Detectado |
|---|---|---|
| AUD-08 | fsync síncrono configurable | Yo verifiqué `SyncMode` ✅, pero auditoría lo marca bloqueante. **Verificar default.** |
| AUD-09 | CRC32C por registro en WAL | Plan Maestro dice "integrado"; auditoría dice "ausente". **Leer `src/wal.rs`.** |
| PERF-08 | Telemetría RSS/mmap/HNSW | Implementación existe ✅, pero Informe §11 dice "225GB en 34GB". **Verificar bug resuelto.** |
| CONC-05 | Validación metadata FFI | `validate_key()` existe, pero falta validación de tipos Python complejos. |

---

## 🎯 PRIORIZACIÓN RECOMENDADA (Orden de ejecución)

### Semana 1-2: Cerrar bloqueantes
1. AUD-03 (RCU rebuild) — **máxima prioridad**
2. AUD-06 (file locking)
3. AUD-09 (verificar CRC32C por registro)
4. AUD-02 (crash-injection CI)

### Semana 3-4: STAB-01 completo
5. PERF-07 (mimalloc)
6. PERF-12 (tests equivalencia SIMD)
7. REL-01 (publicar en crates.io)
8. REL-09, REL-10 (entorno Python hermetizado)
9. DEBT (limpieza código experimental)

### Mes 2: BENCH-01 + Hardening Hybrid
10. REL-04, REL-05, REL-06 (benchmark formal + CI + README)
11. REL-07 (datasets reales)
12. TEX-01 (auditoría estructural text index)
13. TEX-02, TEX-03 (benchmark calidad híbrida + corpus)

### Mes 3: FEAT-01 (Integraciones)
14. INT-01, INT-02 (LangChain + LlamaIndex → PyPI)
15. INT-03 (Mem0)
16. DOC-02 (examples)
17. DOC-06, DOC-07 (docs alineadas + tracker)

### Mes 4-6: Performance & Scale (solo si hay demanda validada)
18. PERF-01, PERF-02, PERF-03 (HNSW multi-layer + L2 + layout)
19. PERF-04 (batch queries Python)
20. PERF-10 (MMap-backed HNSW)

### Mes 6+: Diferir hasta validación de mercado
21. Roadmap v2.0+ completo (DIST-*)
22. Enterprise features (CLD-*, SEC-04/05/06)
23. VantaLISP avanzado (LISP-*)

---

## 📊 MÉTRICAS DE ÉXITO (Próximos 90 días)

| Métrica | Objetivo | Por qué importa |
|---|---|---|
| Tests de crash-injection WAL | 100/100 pasan | Sin esto, eres "cache con pretensiones" |
| Recall@10 certificado | ≥0.95 en SIFT1M | Sin esto, no compites con nadie |
| Latencia Python SDK p50 | <20ms (hoy ~62ms) | Cuello de botella documentado |
| Publicación en crates.io | Hecha | Desbloquea adopción Rust nativa |
| LangChain + LlamaIndex publicadas | En PyPI | Desbloquea adopción Python AI |
| GitHub stars | 500+ en 90 días post-lanzamiento | Validación de mercado mínima |
| Issues de data-loss reportados | 0 | Si aparece uno, pierdes la narrativa |

---

## 🚫 LO QUE NO DEBES HACER

1. **NO añadas features enterprise** (RBAC, mTLS, Raft, multi-tenancy) antes de tener 1,000 usuarios reales.
2. **NO publiques benchmarks sin metodología reproducible.** La comunidad técnica destruye claims no verificables en horas.
3. **NO mantengas identidad dual** (embedded + server simultáneamente). Elige uno; el otro va en `/experimental`.
4. **NO refactorices el core completo** (como sugiere el Plan Maestro con "Desacoplamiento Compute-Storage"). Es premature optimization.
5. **NO ignores los bloqueantes de auditoría** esperando que "se resuelvan solos". WAL sin fsync y rebuild sin exclusión son **data-loss bugs**.
6. **NO ejecutes el Plan Enterprise en paralelo** al Roadmap post-MVP. Son incompatibles.
7. **NO inicies features biológicas/cognitivas** (Ebbinghaus, MAGMA, AUDN) sin haber cerrado Hybrid v1 hardening.

---

## 🔑 RECOMENDACIÓN FINAL (en una frase)

**Congela el alcance, resuelve los 4 bloqueantes de auditoría esta quincena, publica en crates.io, ejecuta BENCH-01 y STAB-01 en ese orden, y no toques enterprise hasta tener 1,000 usuarios embedded reales.**

Cualquier desviación de este orden —añadir features, escalar, o pivotar a SaaS— antes de cerrar la durabilidad y la coherencia es deuda técnica que te costará 10x más caro en 6 meses.

---

## 📎 Fuentes analizadas para este consolidado

**Documentación de auditoría:**
- `investigacion multiple (2).txt` (snapshot audit)
- `Informe_estrategico_VantaDB_2026-04-12.md`
- `VantaDB Post-MVP Roadmap_ A Risk-Managed Strategy...pdf`
- `Auditoría analítica del estado actual de VantaDB.md`
- `Estado actual de VantaDB después de Hybrid Retrieval v1.md`
- `Informe analítico del estado actual de VantaDB 3.md`
- `Informe de estado y siguiente fase de VantaDB.md`
- `Auditoría de la fase de índice de texto persistente derivado.md`

**Planes estratégicos:**
- `PlanDeAccion.md`
- `VantaDB_Plan_Maestro_Ejecutivo.md`
- `roadmap_v2.md`
- `VantaDB_ Evolución y Mejora Propuesta.md`
- `recomendaciones.md`

**Código fuente (verificación directa):**
- `src/wal.rs`, `src/config.rs`, `src/sdk.rs`
- `vantadb-python/src/lib.rs`
- `.github/workflows/rust_ci.yml`
- `src/storage.rs`

---
*Documento generado: 11 de junio de 2026. Próxima revisión recomendada: al completar Fase 1 (bloqueantes) o en 4 semanas.*
