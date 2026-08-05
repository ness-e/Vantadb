# Legacy Docs — Investigación (Extracción 2026-07-16)

> **Propósito:** Extracción de la información valiosa de los análisis históricos de `VANTADB DOC OLD\investigacion\` (21 archivos, ~280 docs antiguos leídos por sub-agentes). Los originales fueron eliminados el 2026-08-05 tras verificar que el contenido técnico ya vive en la doc activa. Este archivo conserva SOLO lo que no está duplicado en la doc viva ni en el backlog.
>
> **Estado:** `historical` — referencia histórica, NO fuente de verdad actual (el repo está en v0.5.0, 2026-07-31; los análisis originales describen v0.3.0).

---

## 1. Análisis Competitivo — Tabla de 10 competidores (fuente: batch5)

La doc viva (`docs/web/standards/product-positioning.md`) solo compara 3 competidores. Esta tabla es la matriz completa; los deep-dives originales fueron eliminados y esta es la única fuente restante.

| Competidor | Tipo | Vector | Grafo | Stack | Diferenciador VantaDB |
|------------|------|--------|-------|-------|----------------------|
| **Qdrant** | Vector puro | Excelente | No | Rust | Graph+Vector nativo |
| **Pinecone** | Vector SaaS | Excelente | No | Propietario | Open source + graph |
| **Milvus** | Vector distribuido | Excelente | No | Go/C++ | Simplicidad + graph |
| **Chroma** | Embedding DB | Buena | No | Python | Producción + graph |
| **Weaviate** | Vector + híbrido | Buena | Básico | Go | Rust + graph nativo |
| **Neo4j** | Grafo puro | Básico | Excelente | Java | Vector nativo + Rust |
| **TigerGraph** | Grafo analítico | No | Excelente | C++ | Vector + deploy simple |
| **ArangoDB** | Multi-modelo | Básica | Buena | C++/JS | Vector-first + Rust |
| **SurrealDB** | Multi-modelo | Básica | Buena | Rust | Enfoque sin scope creep |
| **pgvector** | Extensión SQL | Básica | No | C (PG) | Especializado + graph |

### Conclusiones técnico-estratégicas por competidor (vigencia evaluada 2026-07-16)

- **Qdrant** (vigencia alta): benchmark técnico más cercano — Rust, HNSW personalizado (no HNSWlib), filtrado con `filterable mask` (>90% recall con ~10% overhead), quantization scalar+product, wal-delta merging. Debilidad: sin grafo nativo, clustering stateful complejo. → superarlo agregando grafo.
- **Neo4j** (alto en graph, bajo en vector): estándar index-free adjacency, Cypher como "SQL de grafos", Block Format v5 (40-70% mejora). Debilidad: soporte vectorial reciente/básico, licencia Enterprise costosa.
- **TigerGraph** (alto-medio): líder analytics de grafos pesados, MPP particionado, GSQL→C++ JIT, acumuladores. Debilidad: sin vector nativo, deploy pesado.
- **Milvus** (alto): feature set más completo (IVF_FLAT, IVF_SQ8, HNSW, DISKANN, GPU indexing). Debilidad: complejidad operacional severa (múltiples microservicios). → VantaDB: feature parity con deploy single-binary.
- **Pinecone** (alto): managed-only closed source, auto-scaling caro. → open source es el diferenciador.
- **Weaviate** (medio-alto): híbrido vector+keyword, stack Go. → la búsqueda híbrida es feature esencial.
- **Chroma** (medio): "pip install" developer-first, stack Python limita producción. → experiencia developer-first replicable.
- **ArangoDB** (medio): multi-modelo con AQL coherente, supera a Neo4j en inserciones; vectores limitados.
- **SurrealDB** (medio): multi-modelo con permisos a nivel fila; scope creep = lección de enfoque.
- **pgvector** (alto): "stay where your data lives" — omnipresente por PostgreSQL, básico en features vectoriales.

**Posicionamiento validado:** VantaDB compite en la intersección no cubierta — graph+vector nativo en Rust, single binary, embedded-first. Ningún competidor ofrece ambos con buen rendimiento. Los deep-dives de Qdrant, Neo4j, pgvector, SurrealDB, TigerGraph y ArangoDB no tienen INV propio (solo Weaviate→INV-018, Pinecone→INV-019, Milvus→INV-020, Chroma/LanceDB→INV-007) — candidatos si se requiere profundidad.

---

## 2. Ingeniería inversa — Conceptos de referencia técnica única (fuente: batch6)

Conceptos NO cubiertos en la doc viva ni en los reportes consolidados. Referencia técnica para futuros diseños de almacenamiento/grafo:

- **VelocyPack (ArangoDB)**: serialización binaria con acceso aleatorio a sub-objetos (zero-copy parcial). → intersección con COMP-019 (decisión WONTFIX de rkyv/FlatBuffers ya tomada).
- **SmartGraphs (ArangoDB)**: co-localización de vértices por comunidad → candidato a ampliar `backlog-futuro.md` FUT-03 (hoy solo lista Leiden/Louvain, sin co-localización ni Raft).
- **Block Format v5 (Neo4j)**: localidad de datos con estructuras compactas SIMD-friendly → referencia para diseño de almacenamiento en Rust.
- **Index-free adjacency (Neo4j/TigerGraph)**: punteros físicos para saltos O(1) → referencia en `STORAGE-TIERS.md`.
- **TigerGraph NPG**: GSE con compresión homomórfica 2-10x, MPP con paso de mensajes, GSQL JIT compilado a C++, acumuladores (→ operaciones atómicas lock-free en Rust).
- **Axiomatic Bandwidth Reservation (análisis Kimi)**: reservar 15% de I/O para validaciones críticas y evitar DoS por ráfagas STRICT → sin menciones en reportes; candidato a `backlog-futuro.md`.

### Riesgos técnicos identificados por los análisis (referencia)

- **"Muro de la Incertidumbre"**: disonancia cognitiva temporal (consulta responde con L2, validación L3 detecta error 2s después).
- **DoS de I/O**: ráfagas de queries STRICT degradan el sistema permanentemente a BALANCED.
- **HNSW mono-capa** (superado): el análisis de 2026-07-16 decía "solo capa 0" — hoy `src/index/graph.rs` tiene `max_layer`/`random_layer`/`search_layer` (multi-capa ✅).

---

## 3. Registro histórico de releases v0.1.0 → v0.1.5 (fuente: batch16)

**IMPORTANTE:** `docs/CHANGELOG.md` fue reseteado en v0.4.0 ("Workspace version reset — All previous tags removed", 54 líneas). Este registro detallado ya NO existe en la doc viva; solo sobrevive aquí y en `docs_backup_2026-06-30/CHANGELOG.md`. Conservado como evidencia histórica.

- **v0.1.0-rc1 → v0.1.1**: MVP inicial (Fjall, HNSW, WAL, PyO3), rename ConnectomeDB→VantaDB, purga de terminología biológica, text index, BM25, Hybrid Retrieval v1, Python wheels CI.
- **CLI-EPIC**: 7 comandos (backup, restore, doctor, inspect, stats, count, search-similar) + repl + tui.
- **Integraciones**: LangChain/LlamaIndex, advanced-tokenizer default, Python 3.13+, ARM64 wheels, Homebrew.
- **v0.1.2-v0.1.5**: SQ8 quantization, rkyv zero-copy HNSW, Grafana dashboard, WAL compaction, TTL, `put_batch` Rayon, AsyncVantaDB, type stubs .pyi, NumPy FFI, Prometheus histograms, panic hardening, pyo3 0.24→0.29, bincode 1.3→2.0, unsafe audit.

### Auditorías históricas de mayo 2026 (ausentes de doc viva, hallazgos resueltos)

- `2026-05-04`: cleanup-candidates (P1: `vantadb_data/` 64MB — ✅ limpiado), test-report (97 passed/4 skipped, pytest 17/17), total-review (hallazgos P1-P3 resueltos).
- `2026-05-19`: fase-5-certification (GIL 94.55%, SIFT1M 1535.80s debug, 10K nodes 340.78s debug), plan-accion-alto-rendimiento (15 tareas: GIL injection, telemetría, benchmark HNSW, Euclidean, ranking explicable, SIMD, stemming, backup/restore, OTel, refactor planner, zero-copy Arrow, chaos, PyPI, benchmark competitivo, pilotos — la mayoría ✅ implementadas según CHANGELOG v0.3.0+).

---

## 4. Políticas operativas vigentes confirmadas (fuente: batch17)

Las 11 políticas marcadas vigentes viven en `docs/operations/` con el mismo nombre (verificado): `BACKUP_POLICY`, `BENCHMARKS`, `CI_POLICY`, `COMMUNITY_GOVERNANCE`, `CONFIGURATION`, `DURABILITY_GUARANTEES`, `EXPERIMENTAL_FEATURES`, `FUZZING`, `GRAFANA_SETUP`, `MEMORY_TELEMETRY`, `PYTHON_RELEASE_POLICY` + `RELIABILITY_GATE` (parcial). **Dos correcciones al análisis original:**

- `EDITOR_INTEGRATIONS` NO es obsoleto: `query_lisp`, `inject_context`, `read_axioms` existen en `vantadb-mcp/src/lib.rs:864-1324` y siguen en `docs/api/MCP.md`. Única obsolescencia real: config `opencode.json` → `opencode.jsonc`.
- `SHOW_HN_PREP` SÍ está publicado: vive activo en `docs/strategy/SHOW_HN_PREP.md` (status: active, last_reviewed 2026-08-02).

---

## 5. Fidelidad MPTS y lecciones metodológicas (fuente: batch18, reporte)

- **Fidelidad del vault VantaDB-MPTS: ~82%** (2026-06-22, v0.1.4/v0.1.5). Core engine 95%, APIs 88%, Persistencia 100%, Testing 90%, GTM 60%, Seguridad 10%, Integraciones Tier 1 40%.
- **Reporte de auditoría cruzada MPTS (2026-06-14)**: la auditoría previa "alucinó" features como pendientes cuando ya estaban implementadas. Tras rectificar 4 archivos, la alineación subió de 45% a 85%. Se descartaron ~6 semanas de trabajo duplicado. Hallazgos: 12 corregidos (ERR-01..12), 6 tareas descartadas (HAZ-01..06), 10 features validadas (FEAT-01..10), ~4,500 líneas verificadas.
- **Lección metodológica**: "documentar solo lo que existe" — la auditoría previa alucinó features. Timeline adelantado de 2026-07-26 a 2026-06-21.

---

## 6. Lecciones del Legacy para el Futuro (del reporte consolidado)

1. **El exceso de abstracción biológica fue costoso.** ~50 specs cognitivas descartadas (ConnectomeDB). Validar conceptos contra código temprano.
2. **Los ADRs y walkthroughs fueron el formato más efectivo**: ADRs ~80% fidelidad, walkthroughs ~93% ejecutados; los planes de implementación solo ~20%.
3. **La documentación aspiracional (MPTS, glosario) creó expectativas no cumplidas.** Documentar solo lo que existe o está en desarrollo activo.
4. **El análisis competitivo fue temprano y preciso**: Qdrant como benchmark (sigue cierto), Pinecone vulnerable por closed-source, pgvector competidor por adopción.
5. **La brecha producto vs presencia pública era enorme** (2026-07-16): motor técnicamente sólido sin PyPI/comunidad/marketing. Superado en parte (vantadb-py v0.5.0 publicado, blog, Show HN en curso).

### Tasa de supervivencia por área (medida contra código real, 2026-07-16)

| Área | Supervivencia |
|------|--------------|
| Cognitive Specs 20-29 | ~30% |
| Cognitive Specs 30-36 | ~8-10% |
| Arquitectura y ADRs | ~80% |
| Planes de Implementación (12 planes, ~1955 líneas) | ~20% (la mayoría hoy ✅ implementados — análisis desactualizado) |
| Walkthroughs y Tasks | ~93% (14/15) |
| SDK/API planeado vs real | ~70% (Python 95%, PGWire 0%→ hoy OLD-01 en Backlog) |
| Marketing/GTM/Monetización | ~5% ejecutado |
| Operaciones/Config | ~80% vigentes |
| VantaDB-MPTS | ~82% |

---

## 7. Findings estructurados del JSON (trazabilidad de código)

`old_docs_extracted_findings.json` contenía ~100 findings con `type` (deficiency/todo/performance/security/other), `severity`, `summary`, `detail`, `code_reference` — cubriendo batches 10-16 (incluidos los que no están en los reportes). Valor de trazabilidad: los `code_reference` permiten re-auditar qué paths sobrevivieron al refactor.

**La mayoría ya resueltos** (verificado contra v0.5.0): PyPI publicado, PITR/WAL shipping, AES-GCM, migraciones, CLI/TUI (`src/tui/`), MVCC (VFY-011), auto-embedding (COMP-010 cerrado), snapshots hard-link (OLD-08 ✅).

**Findings aún plausibles o abiertos (referencia):**
- **PGWire** → único item realmente abierto: `docs/Backlog.md:280` (OLD-01, 🗺️ Roadmap).
- **RBAC standalone** → `src/rbac.rs` cableado en `src/cli_server.rs:116-117` (parcial: admin role + namespaces; no integrado al parser IQL).
- **Ollama proxy HTTP** (`/api/generate`, WebSockets, `/v1/points`) → sin tracker; candidato a `backlog-futuro.md` como I+D de bajo valor (LlmClient ya cubre embeddings/summary).
- **GcWorker sin loop background** → `src/gc.rs` sweep manual, sin tokio::spawn ni métricas Prometheus reales.
- **Tiered storage sin disco físico** → `Backlog.md:177` NUEVO-17 (LSM tiers hot/warm/cold) + `backlog-futuro.md` FUT-07.
- **Inference bridge env var** (`VANTA_LLM_URL` vs planeado `OLLAMA_HOST`) → desviación menor documentada.
- **Named benchmarks** (`bench_pure_vector`, `bench_graph_traversal`, `bench_hybrid_filtered`) → no existen con esos nombres exactos; no hay comparativa directa vs Qdrant/Neo4j en `benches/`.

---

## 8. Decisiones clave del legacy (timeline)

| Fecha | Decisión |
|-------|----------|
| 2026-04-12 | Plan Maestro de Redirección — pivot ConnectomeDB cognitivo → VantaDB vector DB |
| 2026-05-19 | Plan de Acción de Alto Rendimiento — SIMD, SQ8, zero-copy |
| 2026-06-19 | Auditoría Integral — 44 findings (7 críticos, 14 medium, 23 low) — todos resueltos |
| 2026-06-22 | FASE 4 kickoff — Ecosistema, TS SDK, integraciones |
| 2026-07-02 | v0.2.0 — Hero redesign, cleanup masivo |
| 2026-07-07 | v0.3.0 — Governance, encryption, WAL shipping |

---

*Fuentes: `old_docs_batch5/6/15/16/17/18.md`, `REPORTE_EVALUACION_COMPLETO.md`, `old_docs_extracted_findings.json` (VANTADB DOC OLD\investigacion, eliminados 2026-08-05). El resto de batches (4, 8-14, 19-21) y `REPORTE_OLD_DOCS_COMBINED.md` fueron eliminados por estar 100% duplicados en doc viva, ADRs, Backlog o este archivo.*
