# Backlog Guide — Explicación y Justificación de Cada Tarea

> Documento companion al `Backlog.md`. Explica el **por qué** de cada tarea, su impacto en el producto y cómo se conecta con la estrategia general de VantaDB.
> **Fecha:** 2026-07-02

---

## PHASE 3 — Pre-Launch (Completada)

### 3.C Core Engine

| ID | Tarea | Explicación |
|----|-------|-------------|
| `TSK-09` ✅ | OpenTelemetry traces | Trace distribuido para debugging en producción. Se pospuso porque sin métricas básicas (Prometheus) no tiene sentido. Ya completada. |

---

## PHASE 4 — Launch (Activa)

---

### 4.0 Foundational — Tareas Bloqueantes

*Estas 4 tareas deben completarse PRIMERO porque todo lo demás depende de ellas.*

| ID | Tarea | Justificación |
|----|-------|---------------|
| `MKT-06` | **Logo y branding** (SVG, palette, favicon) | Sin identidad visual no se puede lanzar. El logo aparece en GitHub, npm, PyPI, docs, web. Es la primera impresión del proyecto. |
| `REL-01` | **Bump v0.1.5 → v0.2.0** | 340+ commits desde v0.1.5, nuevas APIs públicas, 4 plataformas de build. SemVer exige major bump cuando hay cambios de API. Sin esto no se puede publicar a producción. |
| `LEG-01` | **Registrar trademark "VantaDB"** (USPTO + EUIPO) | Riesgo real: alguien puede reclamar el nombre antes del Show HN. Perder el nombre destruiría el proyecto. |
| `LEG-02` | **Contributor License Agreement (CLA)** | Sin CLA no se puede relicenciar ni usar contribuciones comercialmente. Necesario antes de aceptar PRs externos post-lanzamiento. |

---

### 4.B Framework Integrations — Canales de Distribución

*Cada integración es un canal de distribución: los usuarios te descubren donde ya están.*

| ID | Tarea | Justificación |
|----|-------|---------------|
| `INT-01` 🔴 | **LangChain adapter → PyPI + PR upstream** | LangChain es el framework de AI agents más usado. Tener VantaDB como vector store nativo en LangChain es el canal de adquisición más importante. El adapter ya existe en el repo — falta publicarlo y hacer el PR. |
| `INT-02` 🔴 | **LlamaIndex adapter → PyPI + PR upstream** | Segundo framework más importante para RAG. Misma lógica: el adapter existe, hay que publicarlo. |
| `MEM-01` 🔴 | **Mem0: VantaDB como VectorStoreBackend** | **LA INTEGRACIÓN MÁS IMPORTANTE.** Mem0 tiene 57K GitHub stars y soporta 20 backends vectoriales — pero VantaDB NO está entre ellos. Ser el backend #21 conectaría a VantaDB con la comunidad de memoria para agentes más grande del mercado. |
| `MEM-02` 🟡 | **Letta (fka MemGPT): backend de memoria** | Letta tiene 23K stars y su modelo OS-style de memoria es complementario. Segunda integración de memoria más importante. |
| `TSK-89` 🟡 | **CrewAI: VantaDBMemory** | CrewAI (28K stars) usa LanceDB por defecto para memoria de agentes multi-agente. Integrar VantaDB como alternativa. |
| `TSK-91` 🟡 | **DSPy: VantaDBRM** | DSPy (20K stars) es el framework de programación de LLMs. Tener un Retrieval Module nativo abre uso en pipelines avanzados. |
| `TSK-92` 🟡 | **Haystack: VantaDBDocumentStore** | Haystack (18K stars) es el framework de search+RAG más maduro. |
| `TSK-116` 🟡 | **vantadb-openai** | Package helper para generar embeddings con OpenAI API. Reduce fricción de onboarding. |
| `TSK-117` 🟡 | **vantadb-ollama** | Package helper para embeddings locales con Ollama. Clave para el caso de uso "local-first AI". |
| `TSK-95` 🟡 | **vantadb-litellm** | Gateway universal para cualquier proveedor de embeddings (OpenAI, Anthropic, Cohere, etc.). |

---

### 4.C MCP & WASM Differentiation — Ventaja Competitiva Única

*Esta sección es lo que NADIE más en el mercado tiene. Es la verdadera ventaja competitiva de VantaDB.*

| ID | Tarea | Justificación |
|----|-------|---------------|
| `MCP-02` 🔴 | **Estabilizar MCP server a GA** | Weaviate ya tiene MCP nativo en v1.38. Pinecone también. VantaDB tiene MCP experimental. Si no lo estabilizamos ahora, perdemos la ventana. Diferenciador clave: VantaDB es el **único MCP server embebido** (sin cloud). Incluye: docs per-IDE (Cursor, Claude Code, Windsurf, OpenCode, Cline), tests de integración, manejo de errores, connection pooling. |
| `MCP-03` 🔴 | **Benchmarks WASM vs competidores** | El espacio WASM vector DB está fragmentado: EdgeVec (~500 stars), minimemory (~400), altor-vec (~300), ninguno >1K. VantaDB técnicamente es el más completo (HNSW+BM25+RRF+WAL+MCP+IQL) pero NADIE lo sabe. Publicar benchmarks nos establece como líder WASM. |
| `WASM-02` 🔴 | **OPFS persistence para WASM** | WASM actualmente solo funciona en memoria (InMemory). Sin persistencia, es solo una demo. Los competidores (EdgeVec, minimemory) ya tienen IndexedDB/OPFS. Sin esto, el caso de uso "agente en browser" es imposible. |
| `WASM-03` 🟡 | **Demo: AI Agent en browser** | Demo de agente corriendo 100% en browser con Transformers.js + VantaDB WASM + OPFS. Ningún competidor puede hacer esto. Es material viral para Show HN / Twitter. |
| `WASM-04` 🟡 | **Optimizar tamaño WASM bundle** | Target: <500KB gzip. Actualmente sin medir. El tamaño del bundle es crítico para adopción web. |
| `WASM-05` 🟡 | **SIMD acceleration en WASM** | Exponer f32x8 cosine distance en browser. Mejora latencia de búsqueda en WASM significativamente. |
| `MCP-04` 🟡 | **MCP: tools de collection management** | Añadir tools para listar, borrar colecciones, stats, y streaming de resultados. Hace el MCP server funcionalmente completo. |
| `MCP-05` 🟡 | **MCP: integration test suite** | Actualmente 9 tests. Target: 25+. Sin tests no podemos declarar GA. |

---

### 4.D Launch Campaign — Salir al Mundo

| ID | Tarea | Justificación |
|----|-------|---------------|
| `MKT-01` 🔴 | **Landing page (vantadb.dev)** | Hero con benchmarks, comparaciones, install command. Es la cara del proyecto. Sin landing page no hay lanzamiento. |
| `MKT-02` 🔴 | **Blog post "Introducing VantaDB"** | Post técnico con benchmarks reales. Debe mostrar: latencia, recall, comparación con Chroma/Qdrant, casos de uso. |
| `MKT-03` 🔴 | **Show HN post** | El momento más importante del lanzamiento. Timing, título, prepared responses para preguntas comunes. Una hora de atención de Hacker News. |
| `MKT-04` 🟠 | **Reddit posts** | r/rust (comunidad técnica), r/MachineLearning (AI engineers), r/LocalLLaMA (local AI users). Post diferente para cada uno. |
| `MKT-05` 🟠 | **5+ technical blog posts** | Contenido indexed por Google: "How VantaDB works", "Hybrid search explained", "Building AI agents with persistent memory". SEO a largo plazo. |
| `MKT-10` 🟠 | **Campaña "AI Agent Memory"** | Crear la narrativa de que VantaDB es "la SQLite de la memoria para agentes". Blog posts sobre reducción de tokens, benchmarks vs full-context, demos. El mercado de memoria para agentes es el killer use case del momento. |
| `COM-01` 🔴 | **Discord server** | Canales: announcements, general, help, showcase, dev. Es donde la comunidad vive. Sin Discord no hay comunidad. |
| `TSK-106` 🟡 | **GitHub Discussions** | Q&A, Ideas, Show & Tell. Menos urgente que Discord pero importante para el registro público. |
| `TSK-107` 🟡 | **Community showcase** | Proyectos hechos con VantaDB en docs/showcase.md. Pr Figma. |
| `TSK-108` 🟢 | **Newsletter mensual** | Substack/Beehiiv. Bajo esfuerzo, mantiene engaged a early adopters. |
| `—` 🟠 | **20+ Good First Issues** | Para atraer contribuidores externos. Issues etiquetadas, documentadas, con mentoring. |

---

### 4.E Backend Performance — Arreglar lo que Duele

*7 patrones N+1 identificados en el análisis de código. Cada uno representa latencia evitable.*

| ID | Tarea | Justificación |
|----|-------|---------------|
| `PERF-01` 🔴 | **Batch KV loader (`get_many`)** | **EL problema de performance #1.** 7 ubicaciones donde se hace `storage.get(id)` individualmente: BFS/DFS en graph.rs, PhysicalScan en physical_plan.rs, post-filter de HNSW, hybrid search explain. Con `get_many`, 10K keys pasan de 10K get() individuales a 1 batch call. Impacto directo en latencia p50. |
| `PERF-02` 🟡 | **WAL Mutex contention** | `Mutex<Option<WalWriter>>` serializa TODAS las escrituras al WAL. Bajo write throughput, cuello de botella en inserción concurrente. Evaluar `async-lock` o sharded WAL segments. |
| `PERF-03` 🟠 | **spawn_blocking cap dinámico** | Default 16 threads en semaphore. Límite duro de concurrencia para queries. Hacerlo configurable y dinámico basado en CPU cores. |
| `PERF-04` 🟡 | **`Execution(String)` catch-all → variantes tipadas** | TODO en el código fuente. Actualmente es un String catch-all que traga errores. Refactorizar a variantes tipadas mejora debugging y reliability. |
| `PERF-05` 🟡 | **Split archivos monolito** | `storage.rs` (2624L), `index.rs` (2044L), `metrics.rs` (1300L), `cli_server.rs` (687L) son difíciles de navegar, testear y mantener. Split en módulos. |
| `PERF-06` 🟢 | **Eliminar código duplicado** | `append_to_vstore` / `write_node_to_vstore` son ~40L casi idénticas en storage.rs:1170-1257. DRY. |
| `PERF-07` 🟡 | **Global edge index + integridad referencial** | Aristas almacenadas localmente en cada nodo. No hay tabla de adyacencia global. Borrar un nodo deja edges huérfanos (dangling edges). Implementar ON DELETE CASCADE. |
| `PERF-08` 🟡 | **Secondary scalar indexes** | `filter_field()` hace full table scan. Sin índices hash/B-tree para metadata relacional. Necesario para queries eficientes con filtros. |
| `PERF-09` 🟢 | **Dynamic quantization governor** | Auto-transicionar nodos f32→SQ8 basado en frecuencia de acceso. Ahorra RAM sin perder recall en nodos calientes. |
| `PERF-10` 🟠 | **Memory governor con métricas de eviction** | Actualmente no hay visibilidad de cuándo/por qué se evictan nodos. Añadir métricas a `/metrics` y un memory governor configurable. |

---

### 4.F Distribution — Cómo Llega el Software a los Usuarios

| ID | Tarea | Justificación |
|----|-------|---------------|
| `TSK-121` 🟢 | **SHA256 verification en tests** | Verificar integridad de wheels en tests. Bajo esfuerzo, alta confianza. |
| `REL-02` 🔴 | **Publicar `vantadb-ts` npm package** | TypeScript SDK listo (26/26 tests, examples). No publicado en npm. Sin npm package, los devs de JS/TS no pueden usarlo. |
| `DEVOPS-05` 🔴 | **Publicar adapters LangChain + LlamaIndex a PyPI + PR upstream** | Los adapters existen en el repo pero no están en los repos oficiales de LangChain/LlamaIndex. La gente busca "langchain vector store" no "vantadb". |
| `DEVOPS-02` 🟠 | **ARM64 wheels** | Apple Silicon, AWS Graviton, Raspberry Pi. Sin esto, usuarios de M1/M2 no pueden `pip install`. |
| `DEVOPS-06` 🟢 | **Homebrew formula** | `brew install vantadb`. Estándar para herramientas CLI en macOS. Bajo esfuerzo. |

---

### 4.G Developer Experience — Que Sea Fácil Decir que Sí

| ID | Tarea | Justificación |
|----|-------|---------------|
| `TSK-104` 🟠 | **Demo agent: LangChain + Ollama + VantaDB** | Showcase funcional que cualquiera puede clonar y ejecutar. Reduce el tiempo de "entiendo el concepto" a "lo estoy usando". |
| `TSK-103` 🟠 | **Public benchmark site** | `compare.py` vs Chroma/LanceDB/Qdrant. Los devs eligen con datos. Sin benchmarks, las claims son marketing. |
| `DX-01` 🟠 | **Refactor API: `VantaDB()` → `connect()`** | Alinear con SQLite3, LanceDB, DuckDB. La API actual es redundante. `connect()` es el estándar de la industria. |
| `DX-02` 🟠 | **Python SDK: reducir p50 de ~62ms a <20ms** | Sobrecarga FFI en PyO3. La latencia actual es 3x más alta de lo necesario. Optimizar serialización y buffer protocol. |
| `DX-03` 🔴 | **Docker Compose "Local LLM Stack"** | **CRÍTICO.** Todos los competidores (Qdrant, Weaviate, Chroma) tienen Docker impecable. VantaDB tiene 0. Sin Docker, la barrera de entrada para evaluar el producto es enorme. `docker compose up` con VantaDB + Ollama + Open WebUI. |
| `DX-04` 🟡 | **TypeScript SDK: 18 → 50+ tests** | TS SDK necesita más cobertura para ser production-grade. Edge cases, error handling, concurrent access. |

---

### 4.H Code Health & Security — Lo que Duele Ignorar

| ID | Tarea | Justificación |
|----|-------|---------------|
| `SEC-01` 🔴 | **Migrar `bincode`** | RUSTSEC-2025-0141: unmaintained. Usado en serialización de índices, WAL, estado. Alternativas: `postcard` (seguro, compacto) o `rkyv` (zero-copy, extremo). |
| `SEC-02` 🔴 | **Migrar `rustls-pemfile`** | RUSTSEC-2025-0134: vulnerability. Usado para TLS en vantadb-server. Reemplazar con `rustls-pki-types`. |
| `SEC-03` 🔴 | **Schema evolution para formato en disco** | **RIESGO CRÍTICO.** Actualmente bincode serializa structs de Rust directamente. Cualquier refactor rompe DBs existentes. Implementar versioned headers + migration runner en vanta-cli. |
| `SEC-04` 🟠 | **Auth hardening** | La comparación de tokens NO es constant-time (`==` en vez de `subtle::ConstantEq`). Sin rate limiting en auth failures. `/metrics` es público. |
| `SEC-05` 🟡 | **RBAC design** | Scoped API tokens (read-only, namespace-scoped, time-limited). Para deploys multi-usuario. Post-MVP pero necesario para enterprise. |
| `SEC-06` 🟡 | **SBOM en cada release** | Software Bill of Materials (SPDX/CycloneDX). Requisito enterprise y de compliance. |
| `SEC-07` 🟡 | **CodeQL + cargo-deny en CI** | Escaneo de vulnerabilidades en cada PR. Previene que dependencias maliciosas lleguen a producción. |
| `DOC-01` 🟡 | **Unit tests en 34/48 módulos** | Módulos sin `#[cfg(test)]`: config.rs, engine.rs, executor.rs, gc.rs, metrics.rs, storage.rs, graph.rs, backends/. |
| `DOC-02` ✅ | **Refactor `insert_hnsw()`** | 177L → 3 funciones. Completada. |
| `DOC-03` ✅ | **Normalizar filenames Unicode → ASCII** | 6 archivos. Completada. |

---

### 4.I Documentation Consolidation — Tu Mejor Vendedor

| ID | Tarea | Justificación |
|----|-------|---------------|
| `DOC-04` ✅ | **Restaurar contenido archivado** | ~2,900 líneas únicas sin equivalente EN: Vision/UVP, GTM, Roadmap, design principles, risk register. Creado VISION.md, ROADMAP.md, GO_TO_MARKET.md. |
| `DOC-05` ✅ | **Traducir 10 docs + 3 ADRs de ES→EN** | Completada. |
| `DOC-06` 🟡 | **Unified frontmatter schema** | 117 .md files con distintos formatos de metadatos. Unificar a schema común (title, status, tags, last_reviewed, aliases). En progreso. |
| `DOC-07` ✅ | **kebab-case sin acentos** | Completada. |
| `DOC-08` ✅ | **Archivar docs históricos** | TEXT_INDEX_PHASE_1_CLOSEOUT, RELEASE_V0.1.1, MILESTONE_V0.2.0. |
| `DOC-09` 🔴 | **Crear `.github/` con SECURITY.md, SUPPORT.md, CODE_OF_CONDUCT.md** | README.md referencea estos archivos pero `.github/` NO EXISTE. Todos los links devuelven 404. Esto se ve en GitHub como "proyecto abandonado". |
| `DOC-10` 🔴 | **Fix broken links en README.md y README_ES.md** | Múltiples 404s. Mala primera impresión. |
| `DOC-11` 🟡 | **Fix errores factuales en blog** | License: MIT→Apache 2.0, GitHub URL: `vantadb/vantadb`→`ness-e/Vantadb`. El blog es la cara pública del proyecto. |
| `DOC-12` 🟡 | **Update `llms.txt`** | Dice v0.4.0→v0.6.0, el proyecto está en v0.2.0. Importante para AI-SEO (cómo nos ven los LLMs). |
| `DOC-13` 🟡 | **Crear ADRs faltantes** | Solo 3 ADRs para todo el proyecto. Faltan: Fjall vs RocksDB, HNSW params, RRF k, PyO3 architecture, WASM strategy, governance. |
| `DOC-14` 🟡 | **Performance Tuning Guide** | Guía oficial para ajustar HNSW params, memory limits, backend selection, sync modes. Los devs esperan esto. |
| `DOC-15` 🟡 | **OpenAPI/Swagger spec** | HTTP_API.md tiene 149L (vs 428L de EMBEDDED_SDK). Sin spec OpenAPI no hay tooling, no hay client generation. |
| `DOC-16` 🟡 | **Tutorial series** | `docs/tutorials/` con: AI Agent Memory, Local RAG Pipeline, Migrating from ChromaDB. Contenido indexed por Google que atrae usuarios. |

---

### 4.J Web Frontend — La Primera Impresión

| ID | Tarea | Justificación |
|----|-------|---------------|
| `WEB-06` 🟡 | **Migrar 637 inline styles a CSS Modules** | ~80% de los estilos son inline `style={{}}`. engine.tsx: 1085L, architecture.tsx: 557L. Nuevos objetos en cada render (GC pressure), sin cacheo, imposible de sobrescribir. |
| `WEB-07` 🔴 | **Vitest + RTL + Playwright** | **0 tests en frontend.** No hay testing de componentes, no hay E2E. Roturas no detectadas hasta producción. |
| `WEB-08` 🟢 | **Anti-Slop Audit + Performance Budget + SEO** | Revisión final pre-lanzamiento. Bajo esfuerzo, checklist. |
| `WEB-09` 🟡 | **Consolidar animation libraries** | GSAP + Motion 12.42 + AnimeJS 4.5 = ~155KB+ innecesarios. GSAP maneja el 95% de las animaciones. Elegir una y eliminar las otras 2. |
| `WEB-10` 🟡 | **React.lazy() code splitting** | 0 lazy loading actualmente. Todas las páginas en bundle principal. Code splitting automático con Vite. |
| `WEB-11` 🟡 | **React.memo + useMemo + useCallback** | 0 memoization. Componentes como Nav, SwissFooter se rerenderizan en cada navegación. |
| `WEB-12` 🟡 | **Componente reusable `<VsTable>`** | Mismo layout "Legacy vs VantaDB" repetido manualmente en 7+ archivos. DRY. |
| `WEB-13` 🔴 | **SEO: OG tags, canonical, JSON-LD** | **0 OG tags, 0 structured data.** Compartir en Twitter/LinkedIn muestra un link genérico. Sin JSON-LD no hay rich snippets en Google. |
| `WEB-14` 🟡 | **Implementar GSAP animations** | DiseñoNuevo.md especifica: scroll-trigger reveals, count-up numbers, stroke-dashoffset grid, typewriter terminal. No implementado. |
| `WEB-15` 🟢 | **Fix Nav background** | Usa `rgba(10,10,10,0.85)` (dark), debe ser `--surface-glass` (warm paper). |
| `WEB-16` 🟢 | **Fix H1 weight y text-align** | H1 weight 800→700. 9 elementos con text-align:center → left. Detalles de QA. |
| `WEB-17` 🟡 | **Evaluar TanStack Router** | 23 páginas mayormente estáticas. TanStack Router con file-based routing + `routeTree.gen.ts` (`@ts-nocheck`) es overkill. React Router sería más simple. |

---

### 4.K Testing Gaps — Lo que Falta Validar

| ID | Tarea | Justificación |
|----|-------|---------------|
| `TEST-01` 🔴 | **WASM tests** | `vantadb-wasm/tests/wasm_tests.rs` está VACÍO. 0 tests para el módulo WASM. Hay que escribirlos (20+). |
| `TEST-02` 🔴 | **Frontend tests** | 0 tests en toda la web. Vitest + RTL para componentes, Playwright para E2E. Sin tests, no hay confianza en el deploy. |
| `TEST-03` 🔴 | **Security tests** | 0 tests de seguridad. IQL injection fuzzing, auth bypass attempts, input validation, payloads malformed. |
| `TEST-04` 🟡 | **Regression test suite** | Tests específicos para bugs ya corregidos. Previene regressions. |
| `TEST-05` 🟡 | **Snapshot testing** | HNSW recall certification snapshots, export/import format versioning, WAL format integrity. |
| `TEST-06` 🟡 | **Load/stress tests en TS/Python** | Solo Rust tiene stress tests. Los SDKs pueden tener memory leaks o performance issues no detectados. |
| `TEST-07` 🟢 | **test-threads OS-specific** | `test-threads = 2` global perjudica a Linux/macOS. Hacer config por OS. |
| `TEST-08` 🟠 | **Fix chaos_integrity required-features** | `Cargo.toml:199` — el `[[test]]` no declara `required-features = ["failpoints"]`. Sin esto compila sin failpoints y el test pasa vacío. |

---

### 4.L Pricing & Monetization — Preparar el Negocio

| ID | Tarea | Justificación |
|----|-------|---------------|
| `MKT-07` 🔴 | **Pricing page** | Aunque cloud no esté listo, señalar el modelo de precios. Los competidores (Pinecone $50/mo, Qdrant $57/GB, Chroma $0+$5) tienen pricing transparente. Sin pricing page la gente asume que es "too expensive" o "abandoned". |
| `MKT-08` 🔴 | **Trademark registration** | Proteger el nombre antes del Show HN. La investigación muestra que es un riesgo real. |
| `MKT-09` 🟠 | **CLA** | Necesario antes de aceptar contribuciones externas. |
| `BIZ-01` 🟡 | **Enterprise crate structure** | Separar features pagas a `vantadb-enterprise/` (crate propietario). El core Apache 2.0 se mantiene gratuito. Modelo open-core estándar (como GitLab, Mattermost). |
| `BIZ-04` 🟡 | **Cloud architecture design doc** | WAL shipping a S3/R2, serverless read replicas, usage-based pricing. Documento de diseño para cuando toque implementar. |
| `BIZ-05` 🟡 | **Competitive pricing analysis** | Modelar: $0 self-hosted → $29/mo Pro (1M vectors, 10GB) → $149/mo Business (10M) → $499/mo Enterprise (unlimited). Under-cutear el mínimo del mercado (~$50/mo). |
| `BIZ-06` 🟡 | **Pitch Deck + one-pager** | 10 slides para pre-seed fundraising. MRR target, TAM, competitive landscape, team, traction. |

---

## PHASE 5 — Post-Launch / Pre-Seed

### 5.A Enterprise Readiness

| ID | Tarea | Justificación |
|----|-------|---------------|
| `TSK-72` 🟡 | **AES-256-GCM at-rest encryption** | Requisito enterprise. Sin encryptación, datos en reposo son accesibles. |
| `TSK-107b` 🟡 | **Audit logging enterprise** | JSONL con timestamp + operación. Para compliance y debugging en producción. |
| `TSK-110` 🟡 | **SBOM en releases** | Software Bill of Materials. Cada release debe declarar sus dependencias. |
| `BIZ-02` 🟡 | **WAL Shipping asíncrono** | Replicación sin Raft. Stream de WAL a réplicas. Paso intermedio antes de multi-node completo. |
| `TSK-122` 🟡 | **Sharded-slab para HNSW lock-free** | Mitiga el bottleneck de `insert_lock` en HNSW. Escala inserción concurrente. |
| `TSK-131` 🟡 | **PITR via archival WAL** | Point-in-time recovery. Archivar WAL + replay a timestamp específico. |
| `TSK-132` 🟢 | **Research checkpoint API en Fjall** | Investigar si Fjall upstream tiene checkpoint API o si hay que contribuirlo. |
| `TSK-133` 🟢 | **Incremental backup** | Full snapshot + WAL deltas. Backup sin downtime. |
| `TSK-48` 🟢 | **Dynamic quantization f32→SQ8** | Auto-transicionar nodos fríos a SQ8 (4x RAM reduction). |
| `LOW-01` ✅ | **TLS 1.3 en vantadb-server** | Completada. |
| `TSK-142` 🟡 | **WASM OPFS research** | Investigar y prototipar persistencia WASM con OPFS + Web Workers. |
| `TSK-143` 🟡 | **Fjall vs RocksDB benchmark** | Performance parity benchmark. Para depreciar RocksDB si Fjall es suficiente. |
| `TSK-144` 🟠 | **HNSW recall vs latency benchmark** | Comparación cuantitativa del HNSW custom de VantaDB vs hnswlib. Para validación en papers. |
| `ENT-01` 🟡 | **SOC 2 compliance prep** | Access controls, audit trails, data retention policies. Prepararse antes de que clientes lo pidan. |
| `ENT-02` 🟡 | **HIPAA assessment** | Documentación para cumplimiento en datos médicos. Oportunidad de mercado (Chroma no lo tiene). |
| `ENT-03` 🟡 | **Multi-tenant isolation** | Resource quotas: RAM, IOPS, storage por tenant. Para cloud multi-tenant. |
| `ENT-04` 🟡 | **Connection pooling + circuit breaker** | Para server-mode clients. Conexiones reusables con backoff. |

---

### 5.B VantaDB Cloud and Business

| ID | Tarea | Justificación |
|----|-------|---------------|
| `CLD-01` 🟡 | **Cloud Beta (Fly.io)** | Managed cloud. El revenue está en cloud. Open source es marketing, cloud es negocio. |
| `CLD-02` 🟡 | **Pitch Deck** | 10 slides pre-seed. Necesario para fundraising. |
| `CLD-03` 🟡 | **Enterprise pilot program** | 3-5 early adopters que usen VantaDB en producción. Feedback, case studies, revenue. |
| `CLD-04` 🟡 | **Case Studies** | Mínimo 2: AI agent memory + local RAG. Pr Figma. |
| `CLD-05` 🟡 | **Cloud architecture** | WAL shipping a S3/R2, serverless read replicas, usage-based billing. Diseño previo a implementación. |
| `CLD-06` 🟡 | **Stripe integration** | Self-service signup + billing. Sin Stripe no hay cloud. |
| `CLD-07` 🟡 | **Web dashboard** | Admin panel: collections, usage, billing, team management. |
| `BIZ-03` 🟡 | **Pricing page** | Ya en 4.L pero también aquí como parte del cloud. |

---

## Icebox — Ideas para Después

### Roadmap v2

| ID | Tarea | Por qué está aquí |
|----|-------|-------------------|
| `ROAD-02` | Backup/Restore a S3 | Exportar snapshots a cloud storage. No urgente vs features core. |
| `ROAD-03` | Web UI Explorer | Visualizar HNSW topology + UMAP/t-SNE. Nice-to-have, no bloqueante. |
| `ROAD-04` | Bulk Import CLI | Import optimizado de millones de nodos desde JSON/CSV. Para casos enterprise. |
| `ROAD-05` | Multi-model Hooks | Integración con Ollama/OpenAI para embeddings automáticos. Ya hay packages separados (TSK-116/117). |
| `ROAD-07` | Connection Pooling | Queue con circuit breaker. Más relevante cuando cloud esté listo. |
| `ROAD-08` | Schema Validation | Validaciones opcionales por namespace. Post-MVP. |
| `ROAD-09` | Query Caching | LRU cache con TTL. Útil pero no crítico hoy. |
| `ROAD-12` | BM25 v2 | Mejorar phrase positions, stemming, relevance scoring. Para competir con Weaviate en calidad de hybrid search. |
| `ROAD-13` | Query analytics dashboard | Track slow queries, popular collections. Para cloud. |
| `ROAD-14` | Built-in embedding models | Feature opcional lightweight. Reduce fricción pero añade 500MB+ al bundle. |
| `ROAD-15` | TTL scheduler | Auto-mantenimiento para server deployments. Útil pero no urgente. |

### Distributed y Multi-Node (v2.0+)

Raft, Sharding, Zero-Downtime, etc. — TODO pospuesto porque VantaDB es embebido first. La filosofía es "single binary, zero servers". Distribuido es para otra fase (post-seed con equipo).

### VantaLISP / VantaScript

Cognitive primitives: bytecode JIT, multimodal unification, metacognition, CRDTs, multi-hop reasoning. Esto es I+D para v3.0+. No tocar hasta tener product-market fit sólido.

---

## ❌ Do Not Do (hasta post-seed con equipo)

| Feature | Razón |
|---------|-------|
| **Full SQL** | 3-6 meses de trabajo. pgvector ya lo tiene. El ICP no lo necesita. |
| **Distributed / Raft** | 6-12 meses. Contradice la filosofía embedded. |
| **IVF-PQ disk-based** | LanceDB lo hace mejor. No es el mercado de VantaDB. |
| **GPU acceleration** | Rompe zero-config. No resuelve bottleneck real (I/O, no compute). |
| **RBAC / SSO in core** | Cloud managed only. No en el core open source. |
| **Embedding models bundled** | Destruye zero-config (500MB+ wheel). |
| **GraphQL API** | ICP prefiere REST API. MCP ya disponible para queries complejas. |
| **Git-style versioning** | LanceDB ya lo tiene. No es pain point del ICP. |
| **Time-series mode** | Producto diferente. Fuera de scope. |
| **1.5/2-bit Quantization** | Retornos marginales para datasets <1M. |

---

## Top 10 Prioridades Absolutas

Basado en el análisis de 4 subagentes de investigación (competencia, industria, producto, código):

| # | ID | Tarea | Por qué es #1 en su categoría |
|---|----|-------|-------------------------------|
| 1 | `MEM-01` | **Mem0 VectorStoreBackend** | 57K stars, 20 backends soportados, VantaDB no está. Canal de distribución más grande disponible. |
| 2 | `MCP-02` | **Estabilizar MCP server a GA** | Único MCP server embebido del mercado. Weaviate ya tiene MCP nativo. Ventana cerrándose. |
| 3 | `DX-03` | **Docker Compose "Local LLM Stack"** | Todos los competidores tienen Docker impecable. VantaDB tiene 0. Barrera de entrada enorme. |
| 4 | `SEC-01/02` | **Migrar bincode + rustls-pemfile** | Vulnerabilidades activas RUSTSEC. bincode es unmaintained, riesgo de seguridad real. |
| 5 | `SEC-03` | **Schema evolution para formato en disco** | Cualquier refactor rompe DBs existentes. Riesgo crítico de compatibilidad. |
| 6 | `WASM-02` | **OPFS persistence para WASM** | Sin persistencia, WASM es solo demo. Competidores WASM (EdgeVec, minimemory) ya tienen. |
| 7 | `PERF-01` | **Batch KV loader (get_many)** | 7 patrones N+1 que causan latencia evitable. Impacto directo en p50. |
| 8 | `TEST-01` | **WASM tests** | Archivo de tests vacío. Riesgo de regression no detectada. |
| 9 | `MKT-07` | **Pricing page** | Señalizar modelo de negocio antes del Show HN. Sin pricing, el proyecto se percibe como no-serio. |
| 10 | `DEVOPS-05` | **Publicar npm + PRs LangChain/LlamaIndex upstream** | Canales de distribución bloqueados. Los adapters existen pero no llegan a los usuarios. |

---

## Resumen Estratégico

VantaDB ocupa un **nicho que NADIE más ocupa**: embebido + WASM + hybrid search + MCP + IQL. Pero tiene gaps importantes en:

1. **Distribución** — Sin npm, sin PRs upstream, sin Docker, sin Mem0. El mejor producto no sirve si nadie lo encuentra.
2. **Seguridad y Deuda Técnica** — Vulnerabilidades activas, schema evolution no existe, 7 N+1 patterns.
3. **Testing** — 0 tests en WASM, Frontend, Security. El core Rust está bien (667+ tests) pero los bindings no.
4. **Documentación** — Links rotos, ADRs faltantes, sin tutoriales, sin OpenAPI spec.
5. **Web** — Sin SEO, sin tests, inline styles masivos, 3 animation libraries.
6. **Monetización** — Sin pricing page, sin trademark, sin cloud architecture.

**La ventana de oportunidad es real pero limitada.** El mercado de vector DBs es $3-4B en 2026 con 20%+ CAGR. El espacio WASM vector DB está fragmentado (<1K stars todos). Mem0, Weaviate MCP, y LanceDB están avanzando rápido. Cada mes que pasa sin actuar, la ventana se cierra un poco más.

---

*Documento generado el 2 de julio de 2026.*
*Fuentes: `analisis_proyecto.md` (1067L), `docs/` (155+ archivos), 4 subagentes de investigación (competencia, industria, producto, documentación).*
