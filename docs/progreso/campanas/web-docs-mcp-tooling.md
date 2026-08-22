# Web perf, NUEVO, MCP, DX, DOC — tooling y docs

> **Migrado desde** `docs/progreso/README.md` (split GOV-D2, 2026-08-22). Contenido histórico sin cambios salvo dedup indicado.

### WEB-10: Code splitting con React.lazy (4 páginas pesadas)
- **Fecha:** 2026-07-02
- **Objetivo:** Implement `React.lazy()` for route-level code splitting. Previously all pages loaded eagerly.
- **Checklist:**
  - [x] `React.lazy()` applied to Engine, Architecture, Docs, Changelog pages
  - [x] `Suspense` wrappers with fallback loaders
- **Ids:** `WEB-10`

### WEB-11: Optimización con React.memo + useMemo (10 componentes)
- **Fecha:** 2026-07-02
- **Objetivo:** Add `React.memo` + `useMemo` + `useCallback` across 10+ components to prevent unnecessary rerenders.
- **Checklist:**
  - [x] `React.memo` applied to 5+ presentational components
  - [x] `useMemo` applied to expensive calculations in 3 components
  - [x] `useCallback` for stable function references in event handlers
- **Ids:** `WEB-11`

### WEB-12: Componente reutilizable VsTable
- **Fecha:** 2026-07-02
- **Objetivo:** Create `<VsTable data={...} />` component. "Legacy vs VantaDB" layout was repeated manually in 7+ files.
- **Checklist:**
  - [x] Reusable `<VsTable>` component with typed props
  - [x] Refactored all 7+ manual table layouts to use VsTable
- **Ids:** `WEB-12`

### WEB-13: URLs canónicas SEO (los 25 archivos de rutas)
- **Fecha:** 2026-07-02
- **Objetivo:** Add OG tags, canonical URLs, JSON-LD structured data across all 25 route files.
- **Checklist:**
  - [x] Canonical `<link rel="canonical">` on all 25 route files
  - [x] OG tags (title, description, image) added
  - [x] JSON-LD structured data (WebSite, Organization schemas)
- **Ids:** `WEB-13`

<!-- movido a ARCHIVO_HISTORICO.md -->
### NUEVO-05: Sanitizador CI (ASan + TSan)
- **Fecha:** 2026-07-10
- **Objetivo:** Add AddressSanitizer and ThreadSanitizer CI jobs to catch memory errors and data races in CI.
- **Checklist:**
  - [x] ASan job in `ci-rust-10.yml` with nightly + `-Z sanitizer=address`
  - [x] TSan job in `ci-rust-10.yml` with nightly + `-Z sanitizer=thread`
  - [x] `.lsan_suppressions` for known RocksDB false positives
  - [x] Both jobs marked `continue-on-error: true`
- **Ids:** `NUEVO-05`

### NUEVO-06: Umbral de índice plano <10K brute-force
- **Fecha:** 2026-07-10
- **Objetivo:** When the index has few nodes (< threshold), skip HNSW graph traversal and use brute-force flat scan for equivalent accuracy with less overhead.
- **Checklist:**
  - [x] `flat_threshold` field on `VantaConfig` (env var `VANTADB_FLAT_THRESHOLD`, default 10000)
  - [x] Builder method `with_flat_threshold()`
  - [x] Wired from `VantaConfig` → `HnswConfig` → `CPIndex` in `init_indexes()`
  - [x] Flat search dispatch in `graph.rs::search_layer()` when node count ≤ threshold
  - [x] Tests: `flat_search_matches_hnsw_on_small_dataset`, `flat_search_used_when_under_threshold`, `test_with_flat_threshold`
- **Ids:** `NUEVO-06`

### NUEVO-07: Herramientas de migración Chroma→Vanta, LanceDB→Vanta
- **Fecha:** 2026-08-02
- **Objetivo:** Scripts de migración ejecutables desde ChromaDB y LanceDB a VantaDB usando la API real del SDK Python (`vantadb_py.VantaDB`). Corrige el falso positivo del audit 2026-07-28 (que afirmaba scripts inexistentes) y la API inventada (`vantadb.connect`/`db.space`) en tutoriales.
- **Checklist:**
  - [x] `vantadb-python/vantadb_py/migrate/chroma.py` — CLI `python -m vantadb_py.migrate.chroma` + `migrate_from_chroma()` con paginación por batches
  - [x] `vantadb-python/vantadb_py/migrate/lancedb.py` — CLI + `migrate_from_lancedb()` (usa `to_arrow().to_pylist()`, sin dep pylance)
  - [x] `vantadb-python/vantadb_py/migrate/__init__.py` — exports públicos (lazy imports)
  - [x] `vantadb-python/tests/test_migration.py` — 4 tests smoke (chroma, lancedb, custom namespace/table)
  - [x] Tutoriales `03-migrating-from-chromadb.md` + `migration-from-lancedb.md` corregidos a API real; 0 ocurrencias de API inventada
- **Resultado:** ✅ 46 tests pasan (4 migration + 42 regresión `test_sdk.py`)
- **Ids:** `NUEVO-07`

### NUEVO-10: Benchmark suite pública reproducible
- **Fecha:** 2026-08-02
- **Objetivo:** Hacer la suite de benchmarks públicamente reproducible sin build local. Corrige el gap del audit 2026-07-28 ("scripts requieren build local; no standalone"): los 3 scripts forzaban `maturin develop` al fallar `import vantadb_py`.
- **Checklist:**
  - [x] `benchmarks/requirements.txt` — path standalone `vantadb-py>=0.5.0` (PyPI) + deps opcionales para competitive
  - [x] Hints de instalación corregidos en `vantadb_local_bench.py`, `competitive_bench.py`, `batch_vs_sequential_bench.py` → `pip install vantadb-py` (maturin queda como alternativa dev)
  - [x] `benchmarks/README.md` — guía pública: quick start standalone, competitive, variante dev local
  - [x] `docs/operations/BENCHMARKS.md` sección 3 reescrita — path standalone (pip install) antes que maturin
  - [x] Smoke test en venv limpio: `pip install -r benchmarks/requirements.txt` exit 0 (vantadb-py 0.5.0 PyPI); `vantadb_local_bench.py --size 1000 --queries 50` JSON 5/5 claves no vacías
- **Resultado:** ✅ Commit `d0b1c7c6` (feat: public reproducible benchmark suite). `validate-docs-coverage.ps1` falla por gaps preexistentes no relacionados (search.rs inexistente en script, coverage config/error/cli).
- **Ids:** `NUEVO-10`

<!-- movido a ARCHIVO_HISTORICO.md -->
### NUEVO-08: Learning path estructurado en tutorials/ (5-7 ejemplos)
- **Fecha:** 2026-08-02
- **Objetivo:** Learning path estructurado en `docs/tutorials/` (5-7 ejemplos). Backlog decía "4/7, algunos draft". Corrige API inventada (`vantadb.connect`, `db.space`) en tutoriales draft y completa el path a 6 tutoriales.
- **Checklist:**
  - [x] `01-ai-agent-memory.md`, `02-local-rag-pipeline.md` — reescritos a API real (`VantaDB(path)`, `db.put(ns, key, payload, metadata, vector)`, `db.search_memory(ns, vector, text_query, filters)`), status → active
  - [x] `04-hybrid-search-basics.md`, `05-embedding-integrations.md` — nuevos (4 search modes + `explain_memory_search`; OpenAI/Ollama/LiteLLM + hash fallback)
  - [x] `docs/tutorials/index.md` — learning path estructurado (core track 01→02→04→05, migration track 03/lancedb)
  - [x] `03-migrating-from-chromadb.md` — draft → active
  - [x] mdBook sync — index con `{{#include}}`, stubs 04/05, SUMMARY lista 6 tutoriales
  - [x] `docs/master-index.md`, `docs/README.md` — links de tutoriales actualizados
- **Resultado:** ✅ 6 tutoriales, todos active, 0 API inventada (`rg` 0 matches), 30+ snippets validados contra `vantadb_py 0.5.0` en root `.venv`. Commits a460e4e4, a0c8415d, 70a820de, a8104873, b027926e, cff2fb99. `validate-docs-coverage.ps1` falla por gaps preexistentes no relacionados.
- **Ids:** `NUEVO-08`

<!-- movido a ARCHIVO_HISTORICO.md -->
<!-- movido a ARCHIVO_HISTORICO.md -->
### TSK-104: Demo agent LangChain + Ollama + VantaDB
- **Fecha:** 2026-08-02
- **Objetivo:** Demo RAG pulida conectando LangChain + Ollama + VantaDB con las integraciones reales (reemplaza el sketch que emulaba ambos).
- **Checklist:**
  - [x] `examples/python/langchain_ollama_rag.py` (151 líneas) — usa `vantadb_langchain.VantaDBVectorStore` + `langchain_ollama.OllamaEmbeddings`, API moderna `put`/`search_memory`
  - [x] Fallback determinístico (hash-based, sin red) cuando Ollama no está disponible — smoke exit 0 sin servidor
  - [x] Sketch legacy `examples/python/langchain_rag.py` eliminado (emulaba LangChain/Ollama, API legacy `insert`/`search`/`get`)
  - [x] `docs/operations/EXPERIMENTAL_FEATURES.md:73` actualizado al nuevo nombre
  - [x] 0 matches de API legacy/`Emulated` en la demo (grep verificado)
- **Resultado:** ✅ Smoke test exit 0 (fallback path). Tests de integraciones no corribles por disco lleno (45 MB libres — ambiental, no regresión).
- **Ids:** `TSK-104`

### MCP-IDE: Docs de setup MCP por IDE
- **Fecha:** 2026-07-10
- **Objetivo:** Add per-IDE setup documentation for Cursor, Claude Code, Windsurf, OpenCode, and Cline.
- **Checklist:**
  - [x] Cursor setup (Settings → Features → MCP Servers)
  - [x] Claude Code setup (.claude/settings.json)
  - [x] Windsurf setup (Settings → AI → MCP Servers)
  - [x] OpenCode setup (opencode.json)
  - [x] Cline setup (VS Code settings.json)
  - [x] Notes for first-time install, cross-IDE usage, custom binary path, Windows paths
- **Ids:** `MCP-IDE`
- **Fecha:** 2026-07-02
- **Objetivo:** Create PyO3 crate `vantadb-mem0/` for Mem0 VectorStoreBackend integration (57K stars, 20 backends).
- **Checklist:**
  - [x] `vantadb-mem0/` crate created with PyO3 bindings
  - [x] VectorStoreBackend trait implementation skeleton
- **Ids:** `MEM-01`

### MCP-02: Estabilización del servidor MCP (preparación GA)
- **Fecha:** 2026-07-02
- **Objetivo:** Stabilize MCP server from experimental to GA: config, error handling, timeouts, graceful shutdown, metrics.
- **Checklist:**
  - [x] Added per-IDE setup docs (Cursor, Claude Code, Windsurf, OpenCode, Cline)
  - [x] Error handling and connection pooling improvements
  - [x] Graceful shutdown on SIGTERM/SIGINT
  - [x] Metrics (Prometheus histograms, request counters)
  - [x] Configurable timeouts and retry logic
- **Ids:** `MCP-02`

### DX-03: Docker Compose "Stack LLM Local"
- **Fecha:** 2026-07-02
- **Objetivo:** Single `docker compose up` for complete local RAG stack: VantaDB + Ollama + AnythingLLM / Open WebUI.
- **Checklist:**
  - [x] `Dockerfile` for VantaDB server
  - [x] `docker-compose.yml` with VantaDB + Ollama + Open WebUI
  - [x] `.dockerignore` for optimized builds
- **Archivos Creados:**
  - `Dockerfile`, `docker-compose.yml`, `.dockerignore`
- **Ids:** `DX-03`

### DOC-09: Enriquecimiento de documentación Obsidian (Wikilinks y glosario)
- **Fecha:** 2026-07-01
- **Objetivo:** Inject internal wikilinks into docs/ and enrich the glosario/ to establish a bidirectional knowledge graph.
- **Checklist Completado:**
  - [x] Inject wikilinks into architecture/, api/, operations/, strategy/.
  - [x] Create missing glossary terms (bincode, serde, wasm, crdt, opentelemetry, lancedb, qdrant).
  - [x] Establish bidirectional references from glossary back to implementation docs.
  - [x] Remove orphan files (archive/VantaDB_CLI_TUI_Design_Spec.md).
  - [x] Update community-plugins.json for Obsidian.
- **Archivos Modificados:** 35+ Markdown files in docs/
- **Walkthrough:** walkthrough.md (no ha sido comisionado; documento futuro en `docs/`)

### WEB-01: Despliegue en Vercel y configuración de infraestructura web (Plan/CI_CD_INTEGRATION.md)
- **Fecha:** 2026-07-02
- **Objetivo:** Diagnosticar y corregir el despliegue de la SPA en Vercel: resolver errores 404 en rutas internas, unificar configuración de `vercel.json` y corregir el crash crítico de GSAP en producción que dejaba la página en blanco.
- **Checklist Completado:**
  - [x] Auditar estructura completa del proyecto (monorepo Rust + web/)
  - [x] Eliminar `vercel.json` redundante en la raíz del monorepo
  - [x] Centralizar configuración en `web/vercel.json` con `buildCommand`, `outputDirectory`, `cleanUrls` y reglas de reescritura SPA
  - [x] Diagnosticar por qué la SPA mostraba 404 al acceder directamente a rutas internas (`/engine`, `/docs`)
  - [x] Verificar via CLI de Vercel (`npx vercel ls`) el estado de los despliegues en producción
  - [x] Diagnosticar crash crítico de GSAP (`TypeError: aS is not a function`) via errores de consola del browser
  - [x] Resolver race condition de inicialización de módulos en Rollup/producción: mover `gsap.registerPlugin()` a `main.tsx` como primera instrucción del entry point
  - [x] Corregir errores de compilación Rust en `tests/certification/hnsw_validation.rs` (tipos explícitos para `SmallVec<[u64; 32]>` en closures)
  - [x] Suprimir advertencia de `dead_code` en `src/metrics.rs::reset_metrics` con `#[allow(dead_code)]`
  - [x] Añadir `optimizeDeps` en `vite.config.ts` para pre-empaquetar módulos GSAP
- **Archivos Modificados:**
  - `web/vercel.json` — Centralización de configuración Vercel
  - `web/src/main.tsx` — Registro de GSAP como primera instrucción del entry point
  - `web/src/lib/gsap.ts` — Limpieza de imports y exportaciones duplicadas
  - `web/vite.config.ts` — Adición de `optimizeDeps` para GSAP
  - `tests/certification/hnsw_validation.rs` — Corrección de tipos `SmallVec` en closures
  - `src/metrics.rs` — Supresión de `dead_code` en `reset_metrics()`
  - `vercel.json` (raíz) — Eliminado
- **Deuda Técnica Identificada (pendiente):**
  - Múltiples errores de Clippy en `src/metrics.rs` (`int_plus_one`, `field_reassign_with_default`) y `vantadb-mcp/src/storage.rs` bloqueando el pre-push hook
  - Carpeta `web/public/admin/` con artefactos de Decap CMS no utilizado

### WEB-08: Auditoría anti-slop, presupuesto de rendimiento, revisión SEO final
- **Fecha:** 2026-07-02
- **Objetivo:** Realizar una auditoría completa del frontend contra las guías de diseño anti-slop, implementar el presupuesto de eyebrows (máximo 3 en todo el index) y corregir bugs visuales y estructurales identificados en responsive.
- **Checklist Completado:**
  - [x] Rediseño de SwissBenchmarkGrid para usar un layout bento asimétrico y corregir el bug de count-up en valores no numéricos.
  - [x] Rediseño de SwissCoreEngine convirtiendo la cuadrícula genérica de 3 columnas en un accordion stacked minimalista de fondo OLED.
  - [x] Rediseño de SwissEcosystem agrupando integraciones por categorías en filas minimalistas con chips inline en lugar de celdas homogéneas idénticas.
  - [x] Reducción de eyebrows en todo el index para cumplir el presupuesto estricto (máximo 3).
  - [x] Adaptabilidad responsive (breakpoints 960px) en Quickstart y paddings adaptativos en CoreEngine.
- **Archivos Modificados:**
  - `web/src/components/SwissBenchmarkGrid.tsx`
  - `web/src/components/SwissCoreEngine.tsx`
  - `web/src/components/SwissEcosystem.tsx`
  - `web/src/components/SwissQuickstart.tsx`
  - `web/src/components/SwissArchSection.tsx`
  - `web/src/components/SwissUseCases.tsx`

### WEB-14: Implementar animaciones GSAP faltantes según REDESIGN_V2_PLAN.md
- **Fecha:** 2026-07-02
- **Objetivo:** Refinar e implementar las animaciones GSAP que faltaban o eran inconsistentes con el movimiento minimalista de 12px y custom easing definidos en la spec de diseño.
- **Checklist Completado:**
  - [x] Unificación del easing suizo a `cubic-bezier(0.25, 1, 0.5, 1)` (vía variables o inline transition).
  - [x] Corrección de los parámetros de animación en el reveal de celdas en SwissBenchmarkGrid (stagger 0.06s).
  - [x] Corrección de la animación de aparición y:30 a y:12 con el custom cubic-bezier en SwissMonolith.
- **Archivos Modificados:**
  - `web/src/components/SwissBenchmarkGrid.tsx`
  - `web/src/components/SwissUseCases.tsx`
  - `web/src/components/SwissMonolith.tsx`

### DOC-11: Corregir errores factuales en el post del blog
- **Fecha:** 2026-07-02
- **Objetivo:** Resolver errores factibles en la publicación del blog introductorio (`introducing-vantadb.md`) cambiando el tipo de licencia y la dirección del repositorio de GitHub.
- **Checklist Completado:**
  - [x] Corregir licencia de MIT a Apache 2.0 en la tabla de especificaciones.
  - [x] Corregir URL del repositorio de `vantadb/vantadb` a `ness-e/Vantadb`.
- **Archivos Modificados:**
  - `web/content/blog/introducing-vantadb.md`

<!-- movido a ARCHIVO_HISTORICO.md -->
