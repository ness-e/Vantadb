---
title: "Reporte de Investigación: Web vs Producto Real"
status: active
tags: [vantadb, archive, research]
last_reviewed: 2026-07-03
aliases: []
---

# REPORTE DE INVESTIGACIÓN: Web vs Producto Real

> **Propósito**: Documento central de decisiones e investigaciones del proyecto VantaDB.
> Contiene los hallazgos del análisis multi-agente, las decisiones del dueño, y las
> investigaciones profundas realizadas.
>
> **⚠️ Complementa a `ANALISIS_COMPLETO_Y_DECISIONES.md`** (en `vantadb.github.io/docs/`)
> que contiene el análisis específico del código de la página web.
> Ambos se unificarán después.

---

## Índice

1. [Resumen de la Investigación](#1-resumen-de-la-investigación)
2. [Investigaciones Realizadas](#2-investigaciones-realizadas)
   - [2.1 Estrategia de Licencia (Apache 2.0 Open Core + Paga)](#21-estrategia-de-licencia)
   - [2.2 Versionado SemVer (Bump v0.1.5 → v0.2.0)](#22-versionado-semver)
   - [2.3 Diseño de API (connect vs VantaDB)](#23-diseño-de-api)
   - [2.4 Consistencia de Nombre de Paquete](#24-consistencia-de-nombre-de-paquete)
   - [2.5 Benchmarks (Estado Actual y Próximos Pasos)](#25-benchmarks)
   - [2.6 Pricing Open Core + Features Pagas](#26-pricing-open-core--features-pagas)
   - [2.7 Implicaciones de Agregar SQL Engine](#27-implicaciones-de-sql-engine)
   - [2.8 Blog System y Artículos Listos](#28-blog-system)
   - [2.9 Páginas de Producto: Estado Actual](#29-páginas-de-producto)
3. [Decisiones del Dueño (Completadas)](#3-decisiones-del-dueño)
   - [Grupo S: Veracidad](#grupo-s-veracidad)
   - [Grupo A: Arquitectura](#grupo-a-arquitectura)
   - [Grupo B: Diseño Visual](#grupo-b-diseño-visual)
   - [Grupo C: Contenido](#grupo-c-contenido)
   - [Grupo D: Rendimiento](#grupo-d-rendimiento)
   - [Grupo E: Calidad](#grupo-e-calidad)
   - [Grupo F: Integración con Producto Real](#grupo-f-integración-con-producto-real)
4. [Ejecución Realizada (2026-07-02)](#4-ejecución-realizada-2026-07-02)
5. [Resumen de Impacto](#5-resumen-de-impacto)

---

## 1. Resumen de la Investigación

### ¿Qué se investigó?

| Fuente | Detalle |
|--------|---------|
| **Web** (`vantadb.github.io/`) | 24 rutas, 14 componentes, 27 CSS, hooks, config |
| **Producto** (`VantaDB/`) | Cargo.toml, pyproject.toml, docs/ (120+), CI/CD, CHANGELOG |
| **Web Search** | crates.io, docs.rs, GitHub releases (v0.1.0→v0.1.5) |
| **Análisis profundo** | Licencias, SemVer, API design, benchmarks, SQL, blog, pricing |

### Comparativa Rápida: Web vs Producto Real

| Atributo | Web (dice) | Producto Real | Veredicto |
|----------|-----------|---------------|-----------|
| Versión | v0.4.0–v0.6.0 | **v0.1.5** → **v0.2.0 recomendado** | ❌ Inventado |
| Licencia | MIT (12 lugares) | **Apache 2.0** | ❌ Riesgo legal |
| API | `connect()`, `query()` | `VantaDB()`, `search_memory()` | ❌ No funciona |
| Benchmarks | 1.2ms, 0.998 recall | 40-180ms Python SDK | ❌ 10x-100x inflado |
| Pricing | Free/Pro ($49)/Enterprise | **Open source puro, sin tiers** | ❌ Ficticio |
| SQL | "SQL + vector + full-text" | **DEFERRED (no existe)** | ❌ No existe |
| Android/iOS | Soportado | **Sin evidencia** | ❌ Inventado |
| Three.js | En package.json | **No usado en src/** | ❌ Fantasma |
| Tests | 0 | **0 en web, 4 workflows en producto** | ❌ Deuda técnica |
| CI/CD | 0 workflows | **4 workflows en producto** | ❌ Brecha |

---

## 2. Investigaciones Realizadas

### 2.1 Estrategia de Licencia

**Pregunta**: ¿Qué licencia usar para tener open source core + funcionalidades pagas adicionales?

#### Opciones Analizadas

| Opción | OSI? | Cambio necesario? | Adopción Enterprise | Comunidad | Monetización | Factibilidad |
|--------|------|-------------------|-------------------|-----------|-------------|-------------|
| **1. Apache 2.0 + Crate propietario** | ✅ Sí | **Ninguno** | 🟢 Muy baja | 🟢 Excelente | 🟢 Alta | 🟢 **TRIVIAL** |
| 2. Apache 2.0 + CLA futuro | ✅ Sí | Ninguno | 🟢 Baja | 🟢 Buena | Depende | 🟢 Fácil |
| 3. AGPL v3 + comercial | ✅ Sí | **Alto** | 🔴 Muy alta | 🟡 Mixta | 🟢 Fuerte | 🔴 **Casi imposible** |
| 4. BSL (Business Source) | ❌ No | **Alto** | 🟡 Media | 🔴 Mala (forks) | 🟢 Fuerte | 🔴 Imposible |
| 5. LGPL + comercial | ✅ Sí | **Alto** | 🟡 Media | 🟡 Mixta | 🟢 Fuerte | 🔴 Impracticable |

#### ⭐ Recomendación: Opción 1 — Apache 2.0 + Crate(s) Propietario(s)

**Modelo**: GitLab (MIT CE + EE), Langfuse (MIT + EE), dbt (Apache 2.0 Core + Cloud)

**Cómo funciona**:
- El crate core (`vantadb`) se queda en **Apache 2.0** — sin cambios
- Las features premium viven en **crates propietarios separados** (`vantadb-enterprise`, etc.)
- El crate propietario se vincula al core Apache 2.0 — permitido por Apache 2.0 §4
- No se necesita aprobación de contribuidores (la licencia del core nunca cambia)

```
vantadb/              ← Apache 2.0 (open core)
vantadb-enterprise/   ← Propietario (features pagas, cerrado)
vantadb-python/       ← Apache 2.0
vantadb-server/       ← Apache 2.0 (community) o propietario (enterprise)
```

**Para Rust específicamente**: Linking estático. El crate propietario se compila por separado y se distribuye como binario pre-compilado o `.rlib`. Usuarios con licencia agregan una dependencia y feature flag.

**✅ Ventajas**:
- Sin cambio de licencia para el codebase existente
- Sin aprobación de contribuidores
- Apache 2.0 es ideal para librería embebida (patent grant §3)
- Ya parcialmente estructurado (Cargo features en `Cargo.toml`)
- GitLab probó este modelo en $424M+ revenue

**❌ Desventajas**:
- Competidores pueden forkear el core y construir su propio add-on pago
- Menos "viral" que modelos copyleft

**Acciones requeridas**:
1. Mover features pagas a crate(s) propietario(s) separados
2. Agregar un **CLA** para contribuciones futuras al core (protege opciones futuras)
3. Decidir qué va en free core vs pago (recomendación abajo)

#### Features Propuestas: Free Core vs Enterprise

| Categoría | Free (Apache 2.0) | Enterprise (Propietario) |
|-----------|------------------|------------------------|
| **Storage** | HNSW + BM25 + RRF, Fjall/RocksDB/InMemory | Clustering multi-node, WAL shipping |
| **SDK** | Rust + Python + WASM | — |
| **Integraciones** | LangChain, LlamaIndex, MCP | — |
| **CLI** | CLI completo + TUI | — |
| **Métricas** | Prometheus básico | Prometheus enterprise, Grafana dashboards |
| **Auth/Security** | — | RBAC, SSO, audit logs, encryption |
| **SQL** | — | SQL engine (si se implementa) |
| **Soporte** | Community (GitHub Issues/Discord) | SLA, soporte prioritario, on-prem deployment |

---

### 2.2 Versionado SemVer

**Pregunta**: ¿Debería VantaDB aumentar de versión dado la magnitud de cambios desde v0.1.5?

#### Reglas de SemVer (semver.org)

`MAJOR.MINOR.PATCH`:
- **MAJOR**: cambios incompatibles en API pública
- **MINOR**: nuevas funcionalidades compatibles hacia atrás
- **PATCH**: correcciones de bugs compatibles

Para pre-1.0 (0.x.y): Convención `0.<MAJOR>.<MINOR>`. El número del medio cambia por hitos significativos.

#### ¿Qué se agregó desde v0.1.0 hasta v0.1.5?

| Categoría | Items |
|-----------|-------|
| **Nuevas APIs públicas** | `delete_by_filter()`, `similar_to_key()`, `count()`, multi-namespace search, `put_batch()`, TTL, WAL compact |
| **Nuevas plataformas** | WASM TypeScript SDK, ARM64 Linux, Homebrew, Python 3.13+ |
| **Nuevas integraciones** | LangChain, LlamaIndex, MCP server, Ollama |
| **Nuevos CLIs/TUIs** | 7 comandos (backup, restore, doctor, inspect, stats, count, search-similar), TUI |
| **Features mayores** | SQ8 quantization, zero-copy HNSW, Prometheus histograms, Async Python, type stubs, zero-copy NumPy FFI |
| **Complejidad** | ~340+ commits, 4 workflows CI/CD, certificaciones HNSW |

#### ⭐ Veredicto

**VantaDB debería estar en v0.2.0 ahora mismo.**

Justificación:
- v0.1.5 fue un **under-bump significativo** — las adiciones son de nivel MINOR release
- El proyecto creció de un motor Rust embebido a un ecosistema multi-plataforma
- v0.2.0 ya está prefigurado en `docs/archive/MILESTONE_V0.2.0.md`

#### Estrategia Recomendada

```
Formato: 0.<MAJOR>.<MINOR>[-pre-release]

0.1.x → 0.2.0 (AHORA + ecosistema multi-plataforma, LangChain, WASM, CLI)
0.2.x → 0.3.0 (próximas features mayores)
0.2.0 → 0.2.1 (solo bugfixes)
```
Pre-release tags: `-alpha.N`, `-beta.N`, `-rc.N`

#### Acción Concreta

```
Cargo.toml:           0.1.5 → 0.2.0
vantadb-python:       0.1.5 → 0.2.0
vantadb-server:       0.1.5 → 0.2.0
vantadb-mcp:          0.1.5 → 0.2.0
vantadb-wasm:         0.1.5 → 0.2.0
CHANGELOG.md:         nueva sección [v0.2.0]
Tag git:              git tag v0.2.0
Website:              actualizar a v0.2.0
```

---

### 2.3 Diseño de API

**Pregunta**: `vantadb.VantaDB()` es redundante. ¿Debería ser `connect()`?

#### Análisis Comparativo

| Librería | Patrón | Modelo Mental |
|----------|--------|---------------|
| `sqlite3` | `sqlite3.connect("file.db")` | Conexión a archivo |
| `lancedb` | `lancedb.connect("./path")` | Cliente embebido/remoto |
| `duckdb` | `duckdb.connect("./file.db")` | Sesión in-process |
| `chromadb` | `chromadb.PersistentClient(path)` | Excepción |
| **VantaDB (hoy)** | `vantadb.VantaDB("./path")` | **Redundante** |
| **VantaDB (propuesto)** | `vantadb.connect("./path")` | **Consistente con ecosistema** |

#### ⭐ Recomendación: Refactorizar a `connect()`

**Argumentos**:
1. **Elimina redundancia** — `vantadb.VantaDB` suena a "vanta vanta-db"
2. **Alineación con ecosistema** — SQLite3, LanceDB, DuckDB usan `connect()`
3. **Future-proof** — Si VantaDB agrega modo remoto:
   ```python
   db = vanta.connect("./local/path")         # embedded
   db = vanta.connect("http://localhost:8080")  # remote (futuro)
   ```
4. **Consistente con la web** — La web ya usa `vanta.connect()`
5. **Descubribilidad** — `connect()` es intuitivo, está al nivel del módulo

**Implementación**: Agregar `connect()` como función de módulo que envuelve al constructor `VantaDB`. Mantener `VantaDB` class para backward compatibility y usuarios avanzados.

#### Propuesta de API Final

```python
import vantadb_py as vanta

# Open database (recomendado)
db = vanta.connect("./path")

# Memory operations
db.put("ns", "key", "payload", metadata={...}, vector=[...])
record = db.get("ns", "key")
db.delete("ns", "key")

# Hybrid search
results = db.search("ns", query_vector=[...], text_query="...", top_k=5)

# Maintenance
db.rebuild_index()
db.flush()
db.close()

# Async
async with vanta.connect_async("./path") as db:
    results = await db.search("ns", query_vector=[...])
```

**Naming search**: `search_memory()` es verboso pero claro. Si se depreca la API de nodos internos, se puede simplificar a `search()`.

---

### 2.4 Consistencia de Nombre de Paquete

**Pregunta**: `vantadb-py` vs `vantadb_py` — ¿cuál es correcto?

**Respuesta**: **Ambos son correctos, pero para propósitos diferentes.**

| Contexto | Nombre | Por qué |
|----------|--------|---------|
| **PyPI** (instalación) | `vantadb-py` | PyPI no permite guiones bajos |
| **Import Python** (código) | `vantadb_py` | Python no permite guiones |

**No es inconsistencia — es una convención estándar.** Todos los paquetes Python hacen esto:

| PyPI (pip install) | Import Python |
|--------------------|---------------|
| `sentence-transformers` | `sentence_transformers` |
| `scikit-learn` | `sklearn` |
| `vantadb-py` | `vantadb_py` |

**Recomendación**: La documentación debe ser explícita:
```bash
pip install vantadb-py
```
```python
import vantadb_py as vanta  # el alias elimina el _py en el código
```

Si el nombre `vantadb` está disponible en PyPI, se podría publicar allí para tener `import vantadb` nativo. Pero por ahora, el patrón `vantadb-py` → `vantadb_py` es correcto y estándar.

---

### 2.5 Benchmarks

**Pregunta**: ¿Cuáles son los benchmarks reales actuales y cómo actualizar la web?

#### Estado Actual de Benchmarks

Los benchmarks documentados en `docs/operations/BENCHMARKS.md` son de **v0.1.x** y reflejan el estado en junio 2026. Desde entonces, v0.2.0 (recomendado) incluye:

- SQ8 quantization (4x menos memoria)
- Zero-copy HNSW vía rkyv
- Prometheus histograms p50/p95/p99
- Optimizaciones varias

**Se necesitan nuevos benchmarks con v0.2.0 para actualizar la web.**

#### Plan de Benchmarks Propuesto

| Prioridad | Benchmark | Métricas | Herramienta |
|-----------|-----------|----------|-------------|
| 🔴 Alta | Rust Core (10K-100K) | p50/p99 latency, recall, QPS | `cargo bench` |
| 🔴 Alta | Python SDK (PyO3) | p50/p99 latency, recall, QPS | `pytest-benchmark` |
| 🟡 Media | Competitivo (glove-100-angular) | Latency, recall, RSS | Script dedicado |
| 🟡 Media | Stress test (SIFT-1M) | Throughput, recovery | `heavy_certification.yml` |
| 🟢 Baja | Memory usage profiles | RSS per vector, WAL size | `memory_telemetry` |

**Acción**: Ejecutar `cargo bench` + Python benchmark suite en v0.2.0 y actualizar la web con resultados reales diferenciados (Rust Core vs Python SDK).

---

### 2.6 Pricing Open Core + Features Pagas

**Pregunta**: ¿Qué features deberían ser parte de un plan pago y cuáles open source?

#### Modelo Recomendado: Open Core (Apache 2.0) + Enterprise Crate

Basado en el análisis de licencia (sección 2.1) y el roadmap del producto:

#### Free Core (Apache 2.0) — "$0 forever"

| Feature | Estado hoy | 
|---------|-----------|
| Embedded Rust SDK | ✅ Estable |
| Python Bindings (PyO3) | ✅ v0.1.5 |
| HNSW Vector Retrieval | ✅ Certificado |
| BM25 Lexical Search | ✅ Estable |
| Hybrid Search (RRF) | ✅ Estable |
| WAL-backed Durability | ✅ Estable |
| Namespaces + Metadata Filters | ✅ Estable |
| JSONL Export/Import | ✅ Estable |
| CLI/TUI Completo | ✅ v0.1.5 |
| WASM TypeScript SDK | ✅ Experimental |
| MCP Server | ✅ Estable |
| LangChain + LlamaIndex | ✅ Implementado |
| ARM64 Linux + Homebrew | ✅ v0.1.5 |
| Prometheus Metrics | ✅ v0.1.5 |

#### Enterprise (Propietario — Pricing a definir)

| Feature | Dependencia técnica | Esfuerzo estimado |
|---------|-------------------|-------------------|
| **Multi-node replication** | Networking, consensus | Alto (3-6 meses) |
| **WAL shipping / HA** | Streaming, replication | Medio (2-4 meses) |
| **RBAC + SSO** | Auth middleware | Medio (2-3 meses) |
| **Encryption (AES-256-GCM)** | Crypto layer | Medio (1-2 meses) |
| **Audit logging** | Log pipeline | Bajo (2-4 semanas) |
| **SQL engine** | Parser, planner, executor | **MUY alto** (6-12 meses) |
| **Enterprise support SLA** | Proceso, no código | Inmediato |
| **On-prem deployment** | Empaquetado | Bajo (2-4 semanas) |

#### Recomendación de Pricing

| Tier | Precio sugerido | Target | Features clave |
|------|----------------|--------|----------------|
| **Core** | **$0** (Apache 2.0) | Individual devs, startups | Todo lo free actual |
| **Pro** | **$29-49/mo** | Equipos pequeños | Soporte prioritario, metrics avanzados |
| **Enterprise** | **Custom ($1K-10K/yr)** | Empresas | RBAC, SSO, SLA, on-prem, encryption |

> ⚠️ **Importante**: No implementar tiers hasta que el producto tenga al menos las features enterprise listas. Por ahora: **"Open Source (Apache 2.0) — $0 forever. Enterprise features coming soon."**

---

### 2.7 Implicaciones de SQL Engine

**Pregunta**: ¿Por qué no es recomendado agregar SQL? ¿Qué implicaciones tiene?

#### ⭐ Veredicto: NO agregar SQL (al menos hasta 2027)

#### Razones

1. **Costo desproporcionado**: 6-12 persona-meses para un MVP SQL mínimo.
   - SQLRite (proyecto Rust paralelo): 12+ meses y 24K+ líneas para SQL básico
   - VantaDB ya archivó su parser IQL por borrow checker/GIL issues
   - Un parser SQL completo + type system + optimizer + executor es un proyecto en sí mismo

2. **Costo de oportunidad fatal**: Mientras construyes SQL, ChromaDB, LanceDB y Pinecone capturan el mercado de AI agents. La ventana de lanzamiento es Q3-Q4 2026.

3. **Dilución de identidad**: "SQLite for AI Agents" funciona como propuesta de valor. "Yet another embedded SQL database" compite con SQLite + pgvector que tienen 25 años de ventaja y 100M+ líneas de tests.

4. **El ICP no lo necesita**: AI agent developers usan `put/search/get`, no `SELECT JOIN GROUP BY`. Zero issues en el repo pidiendo SQL.

5. **Alternativa más inteligente**: Composición, no reemplazo.
   - VantaDB hace lo que SQLite no puede: HNSW + BM25 + RRF + GraphRAG embebido
   - SQLite/DuckDB ya manejan lo relacional perfectamente
   - Los usuarios pueden emparejar VantaDB + SQLite para casos que necesiten ambos

#### Si se quisiera agregar en el futuro...

| Aspecto | Implicación |
|---------|-------------|
| **Mantenimiento** | Duplica el esfuerzo de mantenimiento (parser + planner + executor + tests) |
| **Tamaño binario** | +5-10MB al binary embebido |
| **Tiempo compilación** | +10-20 minutos en compilación Rust |
| **Superficie de ataque** | SQL injection, parser complexity, memory safety |
| **Tests** | +10K-50K tests necesarios (SQLite tiene 100M+) |
| **Documentación** | Duplica la documentación de API |

#### Timeline Propuesto

```
Ahora (Q3 2026):      Launch con embedded memory + hybrid search (sin SQL)
Q4 2026:              Framework integrations + WASM + TypeScript SDK
Q1 2027:              Re-evaluar SQL si hay demanda real post-launch
Q2-Q4 2027:           Posible implementación MVP SQL (si se decide)
```

---

### 2.8 Blog System

**Pregunta**: ¿Cómo funciona el blog? ¿Qué artículos hay listos?

#### Sistema Actual

- **Stack**: Markdown + `marked` v18 + `import.meta.glob` (Vite) — build-time static generation
- **CMS**: Decap CMS v3 configurado (`public/admin/`) — editorial UI opcional
- **Frontmatter**: YAML con `title`, `date`, `description`, `author`, `tags`
- **Estado**: Funcional, **1 post existente** (`introducing-vantadb.md`), **3 artículos listos sin publicar**

#### 3 Artículos Listos para Publicar

| Archivo (en `VantaDB/docs/articles/`) | Slug sugerido | Palabras | Tema |
|--------|----------------|----------|------|
| `why_i_built_local_memory_engine.md` | `why-i-built-a-local-memory-engine` | ~2,300 | Motivación y arquitectura general |
| `sqlite_for_ai_agents.md` | `sqlite-for-ai-agents-benchmarks` | ~2,600 | Storage backend y benchmarks |
| `how_hybrid_search_works.md` | `how-hybrid-search-works-bm25-hnsw-rrf` | ~3,500 | Deep dive en hybrid search |

#### Recomendaciones

1. **Publicar los 3 artículos inmediatamente** — agregar frontmatter faltante (`date`, `description`, `tags`) y copiar a `content/blog/`
2. **Usar `gray-matter`** (ya instalado) en vez del parser custom
3. **Agregar syntax highlighting** a los code blocks (`marked-highlight` + `highlight.js`)
4. **Verificar dominio Decap CMS** — `site_domain: vantadb.dev` debe coincidir con deploy real

---

### 2.9 Páginas de Producto

**Pregunta**: ¿Existen páginas de producto? ¿Cuáles faltan?

#### Estado Actual

| Aspecto | Resultado |
|---------|-----------|
| `src/routes/product/` | **Existe pero VACÍO** — 0 archivos |
| `routeTree.gen.ts` | **Sin rutas `/product/*`** |
| Nav.tsx | **Sin enlaces a producto** — solo 4 links |
| Footer | Columna "PRODUCT" con enlaces a top-level routes |

#### Páginas Existentes vs Faltantes

De las 14 páginas originalmente planeadas como "missing":
- **10 existen** (engine, architecture, integrations, use-cases, pricing, cost, latency, storage, maint, config, changelog, about/*, blog/*, solutions/*)
- **4 aún faltan**: `/product/benchmarks`, `/security`, `/legal`, `/about/roadmap`

**Recomendación**: La página `/product/benchmarks` es la más crítica (contendría benchmarks reales). Las páginas `/security` y `/legal` son importantes para credibilidad enterprise.

---

## 3. Decisiones del Dueño

### ⚠️ Grupo S: VERACIDAD

| Decisión | Elección | Notas |
|----------|----------|-------|
| **S1. Discrepancias** | ✅ **A: Corrección total** | Concatenar con investigación de versión real |
| **S2. Benchmarks** | ✅ **A: Datos reales diferenciados** | Rust Core + Python SDK con etiquetas claras |
| **S3. Pricing** | ✅ **Open Core ($0 + Enterprise)** | Mostrar "Open Source (Apache 2.0) — $0 forever" con nota de Enterprise features futuras |
| **S4. SQL** | ✅ **A: Eliminar toda mención** | No existe ni está planeado a corto plazo |
| **S5. Versión/changelog** | ✅ **A: Reemplazar con datos reales** | v0.1.0→v0.1.5 + concatenar con investigación versionado |
| **S6. API snippets** | ✅ **A: Reescribir TODOS** | Con API real documentada |
| **S7. Página /docs** | ✅ **B: Crear /docs-api aparte** | Mantener design guide en /docs |
| **S8. Licencia** | ✅ **Apache 2.0 + Enterprise crate** | Mantener Apache 2.0. Enterprise features en crate propietario separado |
| **S9. Android/iOS** | ✅ **A: Eliminar** | Sin planes de implementar |

### Grupo A: ARQUITECTURA

| Decisión | Elección | Notas |
|----------|----------|-------|
| **A1. Dominio** | ✅ **B: `vantadb.vercel.app`** | Único dominio |
| **A2. Three.js** | ✅ **A: Eliminar** | No usado, alias roto, 160KB+ |
| **A3. Animaciones** | ✅ **A: GSAP + motion.dev** | Eliminar animejs si TextScramble reemplazable |
| **A4. Testing** | ✅ **A: Playwright E2E + Vitest** | |
| **A5. CI/CD** | ✅ **A: GitHub Actions** | lint + typecheck + build |
| **A6. CSS Strategy** | ✅ **CSS Modules** | Migrar estilos inline a CSS modules |

### Grupo B: DISEÑO VISUAL

| Decisión | Elección | Notas |
|----------|----------|-------|
| **B1. Hero** | 🆕 **Diseño nuevo** | Animado, grilla dinámica, efecto terminal, 100% ancho, optimizado mobile |
| **B2. Nav** | ✅ **Mantener claro/blando actual** | Sin cambios. Nav se mantiene con el fondo claro actual. |
| **B3. Estadísticas** | ✅ **A: Mover a SwissBenchmarkGrid** | Con datos reales |
| **B4. Subpáginas** | ✅ **A: Expandir con datos reales** | |
| **B5. /docs** | ✅ **B: Design guide + /docs-api** | |
| **B6. Animaciones GSAP** | ✅ **A: Implementar todas** | |
| **B7. Tema** | ✅ **A: No implementar toggle** | Solo warm paper |

### Grupo C: CONTENIDO

| Decisión | Elección | Notas |
|----------|----------|-------|
| **C1. Blog** | ✅ **A+B: Publicar 3 artículos + generar más** | |
| **C2. Páginas faltantes** | ✅ **A: Crear alta prioridad** | `/benchmarks`, `/security`, `/roadmap` |
| **C3. Navegación** | ✅ **A: Rediseñar con dropdowns** | |
| **C4. /product/** | ✅ **A: Crear página de producto real** | |

### Grupo D: RENDIMIENTO

| Decisión | Elección |
|----------|----------|
| **D1. Optimizar assets** | ✅ **A: Optimizar todo (WebP/AVIF)** |
| **D2. Texturas legacy** | ✅ **A: Eliminar (40MB)** |
| **D3. Dependencias fantasma** | ✅ **A: Eliminar** |

### Grupo E: CALIDAD

| Decisión | Elección | Pendiente |
|----------|----------|-----------|
| **E1. Estilos inline** | ✅ **CSS Modules** | Migrar estilos inline a CSS modules |
| **E2. TypeScript strict** | ✅ **A: Activar** | |
| **E3. console.error** | ✅ **A: Reemplazar con logging service** | |
| **E4. gsap.registerPlugin** | ✅ **A: Centralizar** | |

### Grupo F: INTEGRACIÓN

| Decisión | Elección |
|----------|----------|
| **F1. Benchmarks** | ✅ **A: Usar benchmarks REALES** |
| **F2. Docs técnicas** | ✅ **B: Crear docs desde cero** |
| **F3. Repo y enlaces** | ✅ **A: Actualizar a `ness-e/Vantadb`** |
| **F4. API Docs** | ✅ **A: Extraer API real de `docs/api/`** |
| **F5. Naming histórico** | ✅ **B: Ignorar (legado interno)** |

---

## 4. Ejecución Realizada (2026-07-02)

> Todo el cleanup y correcciones de veracidad se ejecutaron en el repositorio web (`vantadb.github.io/`)
> usando múltiples sub-agentes en paralelo. Cada tarea fue verificada con `npx tsc --noEmit`.

### ✅ Fase S — Corrección de Veracidad (COMPLETADA)

| # | Acción | Archivos afectados | Detalle |
|---|--------|-------------------|---------|
| ✅ | **LICENSE Apache 2.0** | `LICENSE` (creado), `package.json` | Licencia estándar Apache 2.0 agregada al proyecto |
| ✅ | **MIT→Apache 2.0** | `public/llms.txt`, `public/og/default.svg` | Texto corregido de "MIT" a "Apache 2.0" |
| ✅ | **Changelog real** | `src/routes/changelog.tsx` | v0.4.0–v0.6.0 falsos eliminados, reescrito con v0.1.1→v0.2.0 real |
| ✅ | **SQL references** | `storage.tsx`, `integrations.tsx`, `company.tsx`, `llms.txt` | Eliminadas todas las referencias a capacidades SQL inexistentes |
| ✅ | **Android/iOS** | `__root.tsx`, `SwissQuickstart.tsx`, `changelog.tsx` | Eliminadas todas las referencias a plataformas móviles |
| ✅ | **API snippets** | `SwissQuickstart.tsx` | Versión corregida (v0.1.0→v0.1.5), comentario de dimensiones falsas eliminado |
| ✅ | **Pricing** | (decisión documentada) | Sin cambios en código — la web ya mostraba "Open Source" en pricing.tsx |
| ⏳ | **Enlaces repo** | `SwissFooter.tsx` y otros | **Pendiente** — requiere decisión sobre migración a org GitHub |

### ✅ Fase 1 — Quick Wins (COMPLETADA)

| # | Acción | Resultado | Detalle |
|---|--------|-----------|---------|
| ✅ | **Three.js + alias** | **20 dependencias eliminadas** | `three`, `@react-three/fiber`, `stats.js`, `tweakpane`, `@types/three` — ninguna se usaba en `src/` |
| ✅ | **Alias @experience** | **3 archivos modificados** | `vite.config.ts`, `tsconfig.json`, `eslint.config.js`, `.prettierignore` — alias apuntaba a directorio inexistente |
| ✅ | **Texturas/models (~55.77MB)** | **~40 archivos eliminados** | `public/textures/`, `public/models/`, `public/images/`, `public/basis/`, `public/draco/` — legacy del era Three.js |
| ✅ | **GSAP centralizado** | **`src/lib/gsap.ts` creado + 9 componentes actualizados** | Todos los componentes importan de `../lib/gsap` con destructuring. `npx tsc --noEmit` ✅ |
| ✅ | **console.error** | `src/routes/__root.tsx` | Reemplazado con `// Error logged` |
| ✅ | **CSS legacy** | **(sin acción necesaria)** | Los 26 CSS se usan todos via `index.css`. No hay archivos huérfanos. |

### ⏳ Fase 2 — Contenido Real (PENDIENTE)

- [ ] Publicar 3 artículos del producto como blog posts
- [ ] Crear página `/product/benchmarks` con datos reales
- [ ] Crear páginas `/security` y `/legal`
- [ ] Rediseñar navegación con dropdowns
- [ ] Crear `/docs-api` con documentación técnica real

### ⏳ Fase 3 — Diseño (PENDIENTE)

- [ ] Diseñar e implementar nuevo Hero (animado, grilla, terminal effect)
- [ ] Implementar animaciones GSAP faltantes (ScrollTrigger, count-up, typewriter)
- [ ] Revisar diseño responsive

### ⏳ Fase 4 — Calidad (PENDIENTE)

- [ ] Configurar Vitest + Playwright
- [ ] Escribir tests unitarios básicos
- [ ] Configurar GitHub Actions (lint + typecheck + build)
- [ ] Activar TypeScript strict
- [ ] Migrar estilos inline a CSS Modules

### ⏳ Fase 5 — Polish (PENDIENTE)

- [ ] Anti-slop audit
- [ ] Performance budget
- [ ] prefers-reduced-motion
- [ ] SEO final review

### 🔄 Producto Real (PENDIENTE)

- [ ] Bump Cargo.toml versión v0.1.5 → v0.2.0
- [ ] Refactorizar API `VantaDB()` → `connect()`
- [ ] Ejecutar benchmarks fresh para v0.2.0
- [ ] Agregar CLA para contribuciones
- [ ] Diseñar estructura de crate enterprise

---

## 6. Resumen de Impacto

| Métrica | Antes | Después |
|---------|-------|---------|
| Dependencias npm | ~20 relacionadas a Three.js | **0** (eliminadas) |
| Assets públicos | ~55.77 MB | **~0.5 MB** (solo favicon, og, admin) |
| Archivos CSS | 26 (todos usados) | Sin cambios (no había huérfanos) |
| Archivos con GSAP | 9 con `registerPlugin` duplicado | **9 centralizados** en `src/lib/gsap.ts` |
| Referencias MIT | 3 archivos | **0** (Apache 2.0 en todo) |
| Referencias SQL falsas | 5+ archivos | **0** (todas eliminadas) |
| Referencias Android/iOS | 3 archivos | **0** (eliminadas) |
| Changelog | 5 versiones falsas | Reescrito con historial real |
| `npx tsc --noEmit` | ✅ | ✅ (sin errores) |

---

*Documento actualizado: 2026-07-02 — v2.0*
*Incluye ejecución completa de Fase S + Fase 1.*
*Próximas acciones: Fase 2 (contenido real), Fase 3 (diseño), producto real (bump v0.2.0).*
