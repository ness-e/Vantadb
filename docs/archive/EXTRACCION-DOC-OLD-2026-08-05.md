# Extracción Histórica — VANTADB DOC OLD (audit-reports) — 2026-08-05

> **Propósito:** Contenido valioso preservado de la limpieza de la carpeta `VANTADB DOC OLD/audit-reports/`. Solo se extrajo información **única** (no duplicada en el proyecto actual); los archivos originales fueron eliminados. Verificado contra `src/`, `docs/Backlog.md`, ROADMAP, ADRs e INV-018/019/020 el 2026-08-05.
>
> **Fuentes (eliminadas tras extracción):** `competitive-features-graph.json`, `competitive-features-consolidated-report.md`, `deep-analysis-{arch,graph,vector}.md`, `cross-ref-verified.json`, `cross-ref-wave3-final-report.md`.

---

## 1. DECISIONES NO TOMADAS (candidato ADR `DECISIONS-NOT-TAKEN.md`)

> Fuente: `deep-analysis-arch/graph/vector.md`. Estas decisiones negativas NO están documentadas en ningún ADR/Backlog actual (verificado 2026-08-05). Sugerencia: mover a `docs/architecture/adr/DECISIONS-NOT-TAKEN.md` cuando se decida formalizar.

| ID | Feature | Archivo fuente | Competidor/Origen | Justificación (del archivo) |
|----|---------|---------------|-------------------|------------------------------|
| QDR-001 | Gridstore (KV custom sin LSM/WAL) | deep-analysis-vector.md | Qdrant | «Reemplazar el storage engine es proyecto de meses. Migrar un LSM existente a segmentos es un costo alto para una ganancia marginal vs features de indexación. Sin WAL, power loss se maneja más manual.» |
| ARC-015 | Arrow IPC como formato de WAL (Fase 2) | deep-analysis-arch.md | — | «Utilidad baja, prematurísimo. postcard está battle-tested y el CRC32C + scan_forward_valid actual es robusto; Arrow no mejora eso, introduciría riesgo.» |
| ARC-005 | IRI (Implicit Relational Inference) | deep-analysis-arch.md | — | «Dificultad Baja, esencialmente inviable: VantaDB es schemaless y IRI requiere schema que no existe. Puede generar joins incorrectos. GraphRAG (OLD-02) es más útil y simple como prerequisito.» |
| WEV-011 | HFresh (índice en disco tipo SPFresh, particionado por freshness) | deep-analysis-vector.md | Weaviate | «Alta complejidad, casos de uso de nicho; paper SPFresh reciente con poca adopción. Latencia variable hot/cold; depende de segment architecture y CRUD básico primero.» |

### Otras diferidas por nicho / bajo retorno (agrupadas)

| Feature | Fuente | Competidor | Justificación |
|---------|--------|------------|---------------|
| GRF-011 — optimizer Cypher completo | graph | Neo4j | Queries simples (1-2 saltos + vector search); el overhead no se amortiza. Diferir CBO. |
| GRF-009 — WASM plugins | graph | SurrealDB | Utilidad media; overhead FFI superior al beneficio de cercanía. Depurar Wasm es difícil. |
| GRF-041 — Pipelined commit | graph | SurrealDB | Escrituras son batches poco frecuentes; no optimiza el bottleneck. Complejidad en recovery. |
| GRF-025 — VelocyPack serialization | graph | ArangoDB | Modelo plano, no anidado; rkyv es mejor inversión. |
| GRF-027 — SmartGraphs (colocation comunidades) | graph | ArangoDB | Solo relevante multi-nodo; no hay producto distribuido. |
| WEV-004 — Rotational Quantization (RQ) | vector | Weaviate | Sobre PQ; beneficio marginal, añade O(d²). |
| PGV-006 — Statistical BQ (SBQ) | vector | pgvector | Mejora incremental; menos útil si vectores normalizados. |
| GRF-011/016 — MPP vertex-centric | graph | TigerGraph | Solo analytics, no RAG transaccional. Diferir. |
| GRF-014 — GSQL/JIT | graph | TigerGraph | Beneficio solo hot paths pre-registrados; pausa 1-10ms. |
| PGV-002 — Iterative Index Scans | vector | pgvector | Obsoleto si in-filter implementado; fallback only. |
| PIN-003 — Single-stage filtering | vector | Pinecone | Redundante con in-filter; bitmaps no escalan alta cardinalidad. |

### Notas de métricas (de deep-analysis-arch.md)

- **ARC-002 (Bitset inline u128):** ahorro "24-56 B/nodo" + elimina indirección de heap.
- **ARC-004 (Edge Label Interning):** "80MB → 12MB para 1M nodos con 4 edges" (~20 B/edge); match label pasa O(n) string → O(1) int.
- **ARC-003 (Cuantización I8):** compresión 4x — "1544B vs 6144B para 1536d".

---

## 2. GRAPH COMPETITOR INSIGHTS (no cubiertos por el proyecto)

> Del dataset `competitive-features-graph.json`. ~20 features graph de Neo4j/TigerGraph/ArangoDB/SurrealDB **no implementadas** ni cubiertas por INV-018/019/020 ni COMPETITIVE_ANALYSIS. Excluídas las ya implementadas (GRF-003/005/006/009/015/020/029/034/042/049/055/056 → COMP-* ✅).

| ID | Feature | Competitor | Descripción técnica | Value for VantaDB |
|----|---------|------------|--------------------|-------------------|
| GRF-001 | Index-free adjacency (physical pointers) | Neo4j | Nodos guardan punteros físicos de disco directos a vecinos (records fijos 15B/34B); hops O(1) independientes del tamaño del grafo. | Eliminar edge lookup-by-ID; fixed-length records + offset pointers en disco en vez de scan de `Vec<Edge>`. |
| GRF-060 | Multi-hop cost = O(traversed subgraph) | Neo4j | Con index-free adjacency el costo escala con el subgrafo visitado, no con la DB; pointer hop ~100ns vs network ~500µs. | Diseñar storage para que el costo multi-hop escale con el resultado, no el dataset. |
| GRF-017 | Hybrid Graph+Vector búsqueda en un query | TigerGraph | vectorSearch + traversal en una unidad GSQL: ANN halla K similares, estos seed traversal multi-hop con filtros de negocio. | Pipeline híbrido: búsqueda vectorial alimenta navegación estructural en el mismo query. |
| GRF-002 | Block Format storage (Neo4j 5.0) | Neo4j | Formato por bloques alinea a líneas de cache CPU; 40–70% más rápido que records fijos legacy. | Block storage denso + CSR (Compressed Sparse Row) + SIMD vecinos. |
| GRF-004 | Protocolo binario Bolt + PackStream | Neo4j | Protocolo TCP/WebSocket stateful binario; PackStream (tipo MessagePack). | Reemplazar JSON por protocolo binario (serde + bincode); menor latencia. |
| GRF-008 | Cypher optimizer pipeline (AST→Logical→CBO→Execution) | Neo4j | Optimizer multi-etapa con estadísticas de selectividad. | Optimización multi-stage para queries LISP. |
| GRF-014 | GSQL queries compiladas (JIT a C++) | TigerGraph | GSQL compila a C++ nativo; elimina overhead de interpreter. | JIT de expresiones LISP vía Cranelift/LLVM. |
| GRF-016 | MPP con message-passing | TigerGraph | Vértices intercambian mensajes; particionado hash; computo se mueve a los datos. | Writer-pool de trabajo; hash-partition; MapReduce-style. |
| GRF-018 | Distributed HNSW (auto-construcción) | TigerGraph | HNSW distribuidos automáticamente en cluster. | Distributed/particionado HNSW; construcción paralela. |
| GRF-021 | Zero-copy binary serialization (Arrow/rkyv/FlatBuffers) | TigerGraph | Eliminar serialización costosa de hot path. | Zero-copy desde día 1; Arrow + gRPC/Flight. |
| GRF-026 | AQL traversal con cláusula PRUNE | ArangoDB | PRUNE detiene exploración en branch con condición; prevención de explosión combinatoria. | PRUNE como iteradores lazy + closures. |
| GRF-027 | SmartGraphs — colocation por comunidad | ArangoDB | Vértices conectados en mismo shard; Louvain; minimiza network hops. | Sharding basado en Louvain; Raft para topología. |
| GRF-033 | RocksDB LSM-tree integration | ArangoDB / SurrealDB | LSM KV con WAL, MemTable, block cache, SSTables. | Evaluar RocksDB/SurrealKV o LSM 100% Rust. |
| GRF-040 | WASM plugin system (Surrealism) | SurrealDB | Plugins Wasm sandboxed en el server a velocidad near-native. | Plugins wasm para lógica LISP compilada; sandbox in-engine. |
| GRF-041 | SurrealMX engine commit pipeline | SurrealDB | MVCC + lock-free; pipeline segmented commit (Analyze→Validate→Persist/Confirm). | Pipeline segmented commit para throughput; MVCC lock-free con atomics. |
| GRF-043 | Rich Edges (edges como full docs) | SurrealDB / ArangoDB | Edges con propiedades, timestamps, metadata, filtrables en traversal. | First-class edges con propiedades; pesos sinápticos. |
| GRF-046 | Cluster zero-config con Raft embebido | SurrealDB | Embeber Raft/Paxos en el binario; single-binary cluster. | Embebir Raft en Rust; cluster single-binary. |
| GRF-062 | Pre/post/in-index filtering vector | SurrealDB / Neo4j / Arango / TigerGraph | Tres estrategias de filtrado (pre, post, in-index bitset). | Implementar las tres; selector por selectividad + cardinalidad. |
| GRF-050 | Local/Global accumulators para algoritmos graph | TigerGraph | Accumulators local (`@`) y global (`@@`) durante traversals. | Accumulators Rust con atomics lock-free. |
| GRF-054 | Concurrent structures lock-free (atomics) | TigerGraph | AtomicU64/I64 fetch_add/CAS; crossbeam epoch reclamation. | Escala multi-core con atomics. |

---

## 3. CONSOLIDATED FEATURES (tracking crítico + alto — del reporte consolidado)

> `competitive-features-consolidated-report.md` — 172 features de 27 docs OLD. Solo se preservan las 17 críticas y las 48 altas como trazabilidad feature→competidor. ⚠️ Los GRF-IDs de esta tabla **no mapean 1:1** con los de la sección 2 (fuentes distintas). Tratar IDs independientes.

### Critical — 17

| ID | Feature | Categoría | Competidor |
|----|---------|-----------|------------|
| QDR-004 | In-filter traversal (bitset durante walk) | VEC | Qdrant |
| QDR-009 | Multiples tipos de cuantización (Scalar, Binary, Product) | VEC | Qdrant |
| WEV-001 | HNSW con custom CRUD (tombstones + async cleanup) | VEC | Weaviate |
| MLV-007 | Multiples index types (HNSW, IVF, DiskANN, SCANN) | VEC | Milvus |
| QDR-008 | HNSW multi-stage quant (PQ traverse + rerank) | VEC | Qdrant |
| QDR-002 | Segment architecture (appendable + non-appendable) | VEC | Qdrant |
| QDR-003 | Optimizers: Vacuum, Merge, Indexing | VEC | Qdrant |
| PIN-003 | Single-stage filtering | VEC | Pinecone |
| GRF-001 | Index-free adjacency (physical pointers) | GRF | Neo4j |
| GRF-005 | 205 HNSW tunable + bitset filter | GRF | Neo4j |
| GRF-011 | GSQL procedural query language | GRF | TigerGraph |
| GRF-025 | Hybrid Search & Multi-Model Joins (AQL) | GRF | ArangoDB |
| GRF-037 | Live queries WebSocket push | GRF | SurrealDB |
| GRF-048 | Full-text indexing (BM25 + FST) | GRF | SurrealDB |
| ARC-014 | HNSW Persistence (sin rebuild cada startup) | ARC | — |
| ARC-019 | Cuantización Escalar SQ8 / PQ | ARC | — |
| ARC-027 | Open-Core Partition: vantadb-core / vantadb-pro | ARC | — |

### High — Vector DB (22)

| ID | Feature | Competidor |
|----|---------|-----------|
| QDR-001 | Gridstore (custom LSM-free KV) | Qdrant |
| QDR-005 | ACORN algorithm (second-hop for filtered search) | Qdrant |
| QDR-007 | Inlined vectores cuantizados en HNSW nodes | Qdrant |
| QDR-008 | Asymmetric rescoring | Qdrant |
| QDR-016/017 | Hybrid RRF + WAL durability | Qdrant |
| QDR-018 | Sharding/replication (Raft) | Qdrant |
| CHR-004/011 | Auto-embedding / embedding provider abstraction | Chroma |
| CHR-009/012 | Hybrid RRF | Chroma |
| MLV-001 | Growing/Sealed segment architecture | Milvus |
| MLV-003 | JSON Shredding | Milvus |
| MLV-005 | Bitset filtering + soft deletes | Milvus |
| MLV-010 | MMap + Tiered Storage | Milvus |
| MLV-016 | Pluggable index engine (VecIndex) | Milvus |
| PIN-004 | IVF + PQFS | Pinecone |
| PIN-005 | Roaring Bitmaps para metadata | Pinecone |
| PIN-010 | Serverless architecture | Pinecone |
| PGV-002 | Iterative Index Scans | pgvector |
| PGV-008 | Binary COPY bulk | pgvector |
| WEV-004 | Rotational Quantization (RQ) | Weaviate |
| WEV-011 | HFresh disk-optimized index (freshness-aware) | Weaviate |
| QDR-022 | Batch upsert ordering guarantees | Qdrant |
| QDR-019 | Payload indexing | Qdrant |

### High — Graph DB (20)

| ID | Feature | Categoría | Competidor |
|----|---------|-----------|------------|
| GRF-002 | Block Format (cache-line aligned) | GRF | Neo4j |
| GRF-003 | Double-linked relationship chains | GRF | Neo4j |
| GRF-004 | Bolt protocol + PackStream | GRF | Neo4j |
| GRF-006 | In-index vector filtering | GRF | Neo4j |
| GRF-007 | Point-in-time recovery + causal clustering | GRF | Neo4j |
| GRF-012 | Distributed graph processing | GRF | TigerGraph |
| GRF-013 | SmartGraph / subgraph detection | GRF | TigerGraph |
| GRF-014 | Attribute-based partition  (hash/range) | GRF | TigerGraph |
| GRF-015 | Multi-hop parallel traversal | GRF | TigerGraph |
| GRF-016 | Graph-native RBAC | GRF | TigerGraph |
| GRF-024 | Multi-model query (doc+graph+vector) | GRF | ArangoDB |
| GRF-027 | ArangoSearch (ICUS tokenizer) | GRF | ArangoDB |
| GRF-028 | SmartJoins (colocated join opt) | GRF | ArangoDB |
| GRF-029 | Change Data Capture (CDC) streams | GRF | ArangoDB |
| GRF-036 | Schemafull + Schemaless hybrid | GRF | SurrealDB |
| GRF-039 | SurrealQL (SQL+GraphQL+script) | GRF | SurrealDB |
| GRF-040 | DEFINE TABLE/FIELD (strict schema) | GRF | SurrealDB |
| GRF-043 | Record links + graph traversal | GRF | SurrealDB |
| GRF-044 | LQ/RLQ subquerías | GRF | SurrealDB |
| GRF-045 | Built-in auth + RBAC per scope | GRF | SurrealDB |

### High — Arquitectura (12)

| ID | Feature | Fuente |
|----|---------|--------|
| ARC-001 | Semantic Cost Estimator (SCE) | cbo_design.md |
| ARC-004 | Edge Label Interning (u32 labels) | unified_node.md |
| ARC-018 | Lock-Free con sharded-slab | Doc Maestro |
| ARC-020 | Leiden/Louvain community detection | Doc Maestro |
| ARC-021 | Edge Temporales (timestamp) | Doc Maestro |
| ARC-022 | FreshHNSW (background repair) | Doc Maestro |
| ARC-023 | Binary Quantization Int4 + Hamming | Doc Maestro |
| ARC-024 | Matryoshka Embeddings (MRL) | Doc Maestro |
| ARC-025 | Node.js/TS bindings via napi-rs | Doc Maestro |
| ARC-026 | Go bindings via cbindgen+cgo | Doc Maestro |
| ARC-028 | Survival Mode + Safe Cap 10% | Doc Actualizada |
| ARC-029 | HardwareScout con ajuste dinámico | Doc Actualizada |

---

## 4. TRIAGE CROSS-REF WAVE3 (hallazgos aún no trackeados en Backlog)

> Origen: `cross-ref-verified.json` + `cross-ref-wave3-final-report.md` (dataset 2026-07-16). Encontraron que estos son los hallazgos high de CI/release **no trackeados** (los demás están resueltos o ya en Backlog). Sugerencia: convertirlos en items Backlog con IDs del esquema actual (no NEW-*). **Referencia de autoridad:** el proyecto tiene auditoría más reciente en `docs/audit-reports/audit-full-2026-08-04T174544.md` (IDs AUDIT-01..08).

### Críticos (3 items)

**1. ROOT1-007 — release-binaries catch-22 (bloqueante real de release)**
- `release-binaries-63.yml` dispara en `release: [published]`, pero **ningún workflow crea un GitHub Release**: release-plz solo crea tags. El workflow nunca corre → no produce binarios.
- Ref: `.github/workflows/release-binaries-63.yml:5-7`. Estado 2026-08-05: sigue así. **Prioridad alta.**

**2. EXT-35/EXT-191 — insert_lock global (~EXT-84 parallel insert)**
- `insert_lock: FairMutex<()>` sigue global en `src/engine/mod.rs:313`. Micro-batching Rayon (`pending_hnsw_batch`) reduce contención pero el lock no se elimina. DELETE/delete_batch ya lo adquieren (parcialmente abordado).
- Acción: fusionar EXT-35 + EXT-191 + EXT-84 (Parallel HNSW insert) en UN item Backlog.

**3. WF1-001 — RUSTSEC-2026-0176/0177 ignorados en CI**
- `cargo audit` ignora estos advisories. Re-chequeo 2026-08-05: `rustls-pemfile` salió de Cargo.lock (quitar ignore); `atomic-polyfill` y `paste` siguen `unmaintained` (mantener ignore supeditado a remediación).
- Acción: reconciliar `.cargo/audit.toml` con Cargo.lock actual.

### Batch CI-01 — Housekeeping Workflows Wave3 (26 hallazgos)

**Sanitizers con `continue-on-error: true`** (`ci-rust-10.yml:298,423,457`)
- WF1-002 Miri, WF1-003 ASan, WF1-004 TSan — detección de UB/leaks/races no bloquea CI.

**Jobs CI fallando** (`ci-rust-10.yml`)
- PLAN2-017/028: macOS `librocksdb-sys` SIGABRT (build script, l.163-166)
- PLAN2-018: Windows `test_consume_io_accumulates` falla
- PLAN2-019: MSRV/minimal-versions: `missing field 'namespace'` PyO3 + `Option<Vec<Option<&Bound>>>` invalid
- PLAN2-020: Clippy `-D warnings` falla
- PLAN2-021: MSRV check (1.94.1) falla
- PLAN2-022: Miri `setup` falla
- PLAN2-023: Tests Linux fallan perfil Audit
- PLAN2-014 e2e: 26 tests fallan (design-audit-pipeline.spec.ts, falsos positivos `ci-web-11.yml`)

**Gaps de calidad CI**
- WF1-016: fuzzing solo semanal (`fuzz-40.yml`), no en PRs → bugs persistencia
- WF1-027: benchmarks sin baseline fijo (regresión compara 2 commits consecutivos) — no detecta regresión en dos commits
- WF1-029: sin performance budget bloqueante (solo crea issue)
- WF1-033: `sift_validation` SIFT-1M opcional/manual
- WF1-036: certificación semanal (regresiones 7 días)
- WF2-007: release-binaries no ejecuta tests (solamente build)
- WF2-010: release-npm no ejecuta tests
- WF2-015: SBOM solo Rust (`cargo-cyclonedx`), huecos Python/TS
- WF2-020: CodeQL solo `languages: rust` — excluye Python/TS
- PLAN2-038: release-npm `publish-ts` no declara `needs: [publish-wasm]`
- PLAN2-042: SHA pinning de acciones sin verificar (13 workflows)
- EXT-133: Authenticode signing inexistente (Windows enterprise, baja)
- EXT-27: sin differential fuzzing vs SQLite (baja)

---

## 5. NOTA DE CALIDAD DEL DATASET cross-ref

- **Dataset:** 2026-07-16, cross-ref wave3. 1.241 findings (59 high, ~140 medium, ~55 low). Fuente original: 482 "still_present".
- ⚠️ **IDs corruptos:** 194 IDs llevan el literal `.PadLeft(3,'0')` (ej. `"EXT-1.PadLeft(3,'0')"`). Eliminar el sufijo al matcheante.
- ⚠️ **verification_notes reutilizadas** sin re-chequeo tras iteración — no todas validadas contra el código actual.
- ⚠️ **Contradicción PERF-23/21/22** resuelta a favor de `docs/progreso/README.md`: los fixes están en `src/index/graph.rs:441`, `distance.rs:32-35`, `node.rs:469` (verificado 2026-08-05). NO incluir como pendientes.
- ✅ **Ya trackeados, no duplicar:** NUEVO-16/17/18 (PQ/LSM), DEVOPS-HOMEBREW.
- ✅ **No re-verificar a ciegas:** usar `docs/audit-reports/audit-full-2026-08-04T174544.md` (AUDIT-01..08) como fuente de autoridad para nuevos Backlog items.
- **Provenance competitive:** dataset ~abril-2026, consolidado 2026-07-16. Re-verificar antes de adoptar (puede ya estar shipped). Los GRF-IDs JSON y consolidated NO mapean 1:1.

---

## Referencias rotas corregibles (mejora opcional)
- `docs/Backlog.md` líneas ~300 y ~443 aún apuntan a `docs/audit-reports/competitive-features-consolidated-report.md` y `deep-analysis-{vector,graph,arch}.md` (archivos que ya no existen). Considerar apuntar a las secciones 2-4 de este archivo o eliminar la línea.