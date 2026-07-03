# Reporte de Revisión y Evaluación del Proyecto — VantaDB

Este documento presenta una evaluación de arquitectura, seguridad, concurrencia, calidad de código y estado del roadmap de **VantaDB** a fecha del **2 de julio de 2026**.

---

## 1. Arquitectura y Estructura del Proyecto

VantaDB es un motor de base de datos vectorial y persistencia embebida de alto rendimiento. El proyecto se organiza como un workspace de Cargo con la siguiente estructura de componentes:

```mermaid
graph TD
    subgraph Core Engine [vantadb (Rust Core)]
        Storage[StorageBackend]
        HNSW[HNSW Vector Index]
        BM25[BM25 Lexical Index]
        RRF[RRF Query Planner]
        WAL[WAL + Recovery]
    end

    subgraph Adapters & Bindings
        Py[vantadb-python / PyO3]
        Wasm[vantadb-wasm]
        TS[vantadb-ts]
    end

    subgraph Deployment Wrappers
        Server[vantadb-server / Axum]
        MCP[vantadb-mcp / Protocolo de Contexto de Modelos]
    end

    Py --> Core Engine
    Wasm --> Core Engine
    TS --> Wasm
    Server --> Core Engine
    MCP --> Core Engine
```

### Tabla de Componentes y Estado

| Componente | Descripción Técnica | Estado |
| :--- | :--- | :--- |
| **`src/` (Core)** | Motor base, WAL con CRC32C, planificador RRF, integraciones con Fjall y RocksDB. | 🟢 Estable |
| **`vantadb-python`** | Extensión nativa en Rust con PyO3. Soporta asyncio, stubs de tipado y conversión mediante buffer protocol. | 🟢 Estable |
| **`vantadb-server`** | API REST/gRPC basada en Axum, TLS mediante Rustls y límites de tasa con Tower. | 🟢 Completo |
| **`vantadb-mcp`** | Interfaz MCP para comunicación nativa con agentes de inteligencia artificial. | 🟢 Completo |
| **`vantadb-wasm`** | Compilación WASM del motor, limitado temporalmente a persistencia en memoria (`InMemory`). | 🟡 Limitado |
| **`vantadb-ts`** | Wrapper de TypeScript que consume la compilación WASM o el servidor. | 🟢 Completo |
| **`web/`** | Interfaz del sitio web en React 19 + Tailwind v4 + GSAP/Motion para documentación e interacciones. | 🟢 Completo |

---

## 2. Diagnóstico del Core Engine y Persistencia

### Abstracción de Persistencia
El motor implementa el trait `StorageBackend`, permitiendo desacoplar la base de datos del motor LSM subyacente.
- **Fjall** (por defecto): Excelente rendimiento embebido, escrito en Rust nativo.
- **RocksDB**: Proporciona compatibilidad heredada y alta tolerancia bajo cargas extremas.
- **In-Memory**: Usado para pruebas y en entornos WASM.

### Análisis de la serialización y Riesgos de Formato en Disco
> [!WARNING]
> **Riesgo Crítico de Schema Evolution:**
> Actualmente, las estructuras serializadas en disco (como metadatos, WAL e índices) utilizan `bincode v2.0`. Dado que `bincode` produce salidas compactas pero acopladas a la representación binaria exacta en memoria de las structs de Rust, **cualquier refactorización o cambio de versión en las structs romperá la compatibilidad con bases de datos antiguas**.
>
> VantaDB carece de un subsistema de migraciones físicas de almacenamiento (Physical Storage Migrations). Un salto de versión de v0.1.5 a v0.2.0 sin migración de datos corromperá el almacenamiento de los usuarios.

---

## 3. Vulnerabilidades de Seguridad y Dependencias Inseguras

El backlog del proyecto destaca dos dependencias prioritarias por motivos de seguridad:

1. **`bincode` (RUSTSEC-2025-0141):**
   - **Estado actual:** Se migró de 1.3 a 2.0-rc para solventar vulnerabilidades directas de pánicos. Sin embargo, al ser considerado *unmaintained*, representa una deuda técnica y un riesgo de seguridad a mediano plazo.
   - **Acción propuesta:** Evaluar la migración a `postcard` (seguro, compacto y diseñado para entornos integrados) o `rkyv` (acceso zero-copy, rendimiento extremo).
2. **`rustls-pemfile` (RUSTSEC-2025-0134):**
   - **Uso:** Importado condicionalmente en `src/cli_server.rs` para configurar certificados TLS en el servidor HTTP Axum.
   - **Acción propuesta:** Actualizar las llamadas de parsing a `rustls-pki-types` o encapsular un parser manual simplificado para eliminar la dependencia obsoleta.

---

## 4. Concurrencia y Cuellos de Botella de Rendimiento

El motor implementa protección mediante exclusión mutua en puntos calientes de datos:
- **`insert_lock` en HNSW:** El proceso de inserción requiere bloquear capas de HNSW mediante `RwLock`. En cargas de inserción concurrente masiva, esto crea contención y degrada la latencia.
- **Sugerencia de Optimización:** Implementar una estructura de bloques distribuidos (`sharded-slab`) o un esquema lock-free parcial para los enlaces de HNSW (como se plantea en `TSK-122`), mitigando la contención de escritura en el grafo.

---

## 5. Diagnóstico de los Bindings de Clientes

### Python SDK (`vantadb-python` / PyO3)
- **Ventajas:** Excelente uso del buffer protocol de Python 3.11+ para evitar copias de memoria en la transferencia de arrays NumPy de alta dimensionalidad (`extract_vector`).
- **Desventajas:** La conversión dinámica de payloads JSON o metadatos escalares mediante `py_any_to_value` introduce sobrecarga de paso de mensajes, manteniendo la latencia p50 en ~62ms.
- **Acción:** Optimizar la serialización FFI delegando conversiones masivas directamente a deserializadores de alta velocidad en Rust (como `serde_json::value::Value` o parseadores nativos directos).

### WebAssembly (`vantadb-wasm`)
- **Limitación principal:** Limitado exclusivamente a `BackendKind::InMemory`. Esto imposibilita su uso como base de datos persistente en navegadores.
- **Solución propuesta:** Integrar compatibilidad con Origin Private File System (OPFS) de HTML5 para proporcionar persistencia nativa en disco en el navegador a través de `vantadb-wasm`.

---

## 6. Suite de Pruebas e Infraestructura de CI/CD

El proyecto cuenta con una cobertura admirable: **265 tests funcionales**. La auditoría realizada en junio resolvió la mayoría de las incidencias críticas (como fallos en el runner de Windows y tests desorganizados en CI).

Sin embargo, persisten problemas operativos en la infraestructura de pruebas:

- **`test-threads = 2` global en nextest:**
  Establecido en `.config/nextest.toml` debido a un problema ambiental en Windows con el archivo de paginación (`os error 1455`). Forzar este límite globalmente perjudica los tiempos de ejecución del pipeline en entornos Linux/macOS, donde el paralelismo nativo podría ser sustancialmente mayor.
  * **Solución:** Crear configuraciones de perfiles de nextest específicos del sistema operativo o eliminar la limitación global, delegándola al script de CI de Windows de manera exclusiva.

---

## 7. Recomendaciones y Siguientes Pasos

Para preparar a VantaDB para el lanzamiento formal (v0.2.0) y hacerlo apto para producción, se recomiendan las siguientes prioridades técnicas:

```
┌─────────────────────────────────────────────────────────────┐
│ 1. RESOLVER DEPENDENCIAS SEGURAS (rustls-pemfile, bincode)   │
└──────────────────────────────┬──────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────┐
│ 2. DISEÑAR ESTRATEGIA DE EVOLUCIÓN DE FORMATOS EN DISCO     │
└──────────────────────────────┬──────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────┐
│ 3. IMPLEMENTAR PERSISTENCIA OPFS EN WASM                    │
└─────────────────────────────────────────────────────────────┘
```

1. **Resolución de Dependencias Vulnerables:** Migrar `rustls-pemfile` a `rustls-pki-types`.
2. **Definición de Formato de Intercambio Físico:** Implementar un sistema de versión de cabeceras en el almacenamiento que permita actualizar bases de datos creadas con versiones anteriores de `bincode` o reemplazar el motor de serialización por `postcard`.
3. **Persistencia WASM (OPFS):** Investigar e implementar persistencia con OPFS para `vantadb-wasm`, completando un hito único para su uso en aplicaciones web cliente.
4. **Optimización FFI en Python:** Abordar `DX-02` para bajar la latencia p50 de ~62ms a <20ms mediante FFI rápido y deserialización optimizada.
5. **Ajuste de Nextest:** Remover `test-threads = 2` global en nextest para optimizar el paralelismo en entornos Unix.

# Auditoría de Workflows, Tests y Relaciones — VantaDB

**Fecha:** 2026-06-20
**Última actualización:** 2026-06-20
**Tipo:** Revisión estática de configuración CI/CD y declaraciones de tests
**Alcance:** `.github/workflows/*.yml`, `.config/nextest.toml`, `Cargo.toml` (`[[test]]`), `docs/operations/CI_POLICY.md`

---

## Estado Actual — 7/9 Hallazgos Corregidos

| # | Hallazgo | Severidad | Estado |
|---|----------|-----------|--------|
| 1 | `hnsw_recall` ID mismatch | 🔴 Crítico | ✅ CORREGIDO |
| 2 | Tests sin `[[test]]` explícito | 🟡 Alto | ✅ CORREGIDO |
| 3 | `multilingual_tokenizer_integration` no excluido | 🟡 Alto | ✅ CORREGIDO |
| 4 | `mcp_tests` sin clasificación CI | 🟡 Alto | ✅ CORREGIDO |
| 5 | `--features cli` implícito en storage-persistence | 🟠 Medio | ✅ CORREGIDO |
| 6 | `chaos_integrity` sin `required-features` | 🟠 Medio | ❌ PENDIENTE |
| 7 | `test-threads = 2` global | 🔵 Bajo | ❌ PENDIENTE |
| 8 | `columnar` nunca corre en CI | 🔵 Bajo | ✅ CORREGIDO |
| 9 | Filtro `not test(...)` frágil | 🔵 Bajo | ✅ CORREGIDO |

---

## ✅ Hallazgos Corregidos

### 1. 🔴 `hnsw_recall` → `hnsw_recall_certification`
- `.config/nextest.toml:28` ahora usa `not binary(hnsw_recall_certification)`

### 2. 🟡 Tests con `[[test]]` explícito agregados
- `Cargo.toml:400-414` — se agregaron entradas para `fjall_cold_copy_restore`, `property_durability`, `fuzz_proptest`, `multilingual_tokenizer_integration`

### 3. 🟡 `multilingual_tokenizer_integration` excluido y clasificado
- `.config/nextest.toml:54` → `not binary(multilingual_tokenizer_integration)`
- `heavy_certification.yml:198` → agregado al job `other-heavy`
- `CI_POLICY.md:60` → documentado

### 4. 🟡 `mcp_tests` clasificado
- `.config/nextest.toml:53` → `not binary(mcp_tests)`
- `heavy_certification.yml:210-214` → nuevo bloque `--package vantadb-mcp --test mcp_tests`
- `CI_POLICY.md:60` → documentado

### 5. 🟠 Documentación de features implícitos
- `heavy_certification.yml:115` → comentario que `prefetch_benchmark` y `file_locking_stress` requieren `cli`

### 8. 🔵 `columnar` ahora corre en CI
- `heavy_certification.yml:186` → `--features cli,arrow` (antes solo `cli`)
- `heavy_certification.yml:199` → `--test columnar` agregado

### 9. 🔵 Filtro frágil eliminado
- `not test(integrations_certification)` ya no existe en el filter

### Otros cambios adicionales detectados
- `binary_id(...)` → `binary(...)` en todo `.config/nextest.toml`
- `integration` agregado al filtro audit (`nextest.toml:38`)
- `memory_telemetry` y `concurrent_insert_preserves_hnsw_invariants` agregados al filtro audit y a `heavy_certification.yml`

---

## ❌ Hallazgos Pendientes (2)

### 6. 🟠 `chaos_integrity` sin `required-features = ["failpoints"]`

| Campo | Valor |
|-------|-------|
| **Archivo** | `Cargo.toml:199` |
| **Problema** | Corre en `heavy_certification.yml` con `--features failpoints`, pero el `[[test]]` en Cargo.toml no declara el requirement. |
| **Impacto** | Compila sin failpoints (pasa vacío o falla distinto) |

### 7. 🔵 `test-threads = 2` global en nextest (no OS-específico)

| Campo | Valor |
|-------|-------|
| **Archivo** | `.config/nextest.toml:64` |
| **Problema** | El límite aplica también a Linux, donde podría usar más paralelismo. El comentario dice que es por Windows MSVC. |

---

## 📊 Resumen Final

| Estado | Cantidad |
|--------|----------|
| ✅ Corregidos | 7 |
| ❌ Pendientes | 2 |
| **Total hallazgos** | **9** |

---

# ═══════════════════════════════════════════════════════════════
# ANÁLISIS MULTI-AGENTE PROFUNDO — Julio 2026
# ═══════════════════════════════════════════════════════════════

> Este análisis fue generado el 2 de julio de 2026 mediante 6 subagentes especializados que exploraron en paralelo: estructura del proyecto, backend Rust, frontend web, base de datos, documentación y tests.

---

## A. ARQUITECTURA GENERAL DEL PROYECTO

### A.1 Estructura de Directorios (Top 3 niveles)

```
vantadb/
├── src/                    # Core engine Rust
│   ├── api/                # API routes
│   ├── backends/           # RocksDB, Fjall, InMemory
│   ├── bin/                # vanta-cli, lock_helper, crash_helper
│   ├── hardware/           # CPU/SIMD detection
│   ├── parser/             # IQL parser (nom)
│   ├── sdk/                # Public SDK (api, builder, types, search, graph)
│   ├── serialization/      # rkyv archives
│   ├── utils/              # confidence_metrics, duplicate_prevention
│   ├── vector/             # quantization, transform
│   ├── cli_server.rs       # Axum HTTP server + middleware
│   ├── engine.rs           # InMemoryEngine + WAL
│   ├── storage.rs          # StorageEngine (2624 lines)
│   ├── index.rs            # CPIndex HNSW (2044 lines)
│   ├── metrics.rs          # Prometheus metrics (1300 lines)
│   ├── wal.rs              # WAL writer/reader
│   ├── config.rs           # VantaConfig builder
│   ├── node.rs             # UnifiedNode, Edge, FieldValue
│   ├── ... (~54 .rs files)
├── vantadb-python/         # PyO3 bindings
├── vantadb-server/         # Axum HTTP server binary
├── vantadb-mcp/            # MCP stdio server
├── vantadb-wasm/           # WASM bindings
├── vantadb-ts/             # TypeScript SDK (WASM wrapper)
├── web/                    # React 19 marketing site
│   ├── src/
│   │   ├── components/     # 13 components
│   │   ├── routes/         # 23 pages (TanStack Router)
│   │   ├── hooks/          # 2 custom hooks
│   │   ├── lib/            # API layer (empty), blog, gsap, utils
│   │   └── styles/         # 26 CSS files + tokens.css
│   ├── content/blog/       # 4 markdown blog posts
│   └── public/             # static assets
├── docs/                   # Obsidian vault (~140 files)
│   ├── api/                # EMBEDDED_SDK, PYTHON_SDK, HTTP_API, MCP
│   ├── architecture/       # ARCHITECTURE, ADRs (3), TEXT_INDEX, WAL
│   ├── glosario/           # 54 glossary terms
│   ├── operations/         # 22 documents
│   ├── strategy/           # ROADMAP, GO_TO_MARKET
│   ├── migration/          # FROM_CHROMADB, FROM_LANCEDB
│   └── ...
├── tests/                  # 61 integration test files
├── benches/                # 6 Criterion benchmarks
├── fuzz/                   # 2 fuzz targets
├── integrations/           # LangChain + LlamaIndex adapters
├── .github/workflows/      # 8 CI/CD workflow files
└── packages/               # langchain + llamaindex Python packages
```

### A.2 Stack Tecnológico Completo

| Capa | Tecnología |
|------|-----------|
| **Lenguaje principal** | Rust 1.94+ (edition 2021) |
| **Async runtime** | Tokio (multi-threaded) |
| **HTTP framework** | Axum 0.8 (feature `server`) |
| **Serialización** | bincode 2.0, serde, serde_json, rkyv 0.8 (opt) |
| **Storage backends** | RocksDB 0.22 (LZ4), Fjall 3.1 (default), InMemory |
| **Vector index** | CPIndex custom (HNSW-like, mmap-backed) |
| **Text index** | BM25 (custom) + tantivy 0.22 (opt) |
| **Query language** | IQL (nom 7) + Executor + Planner |
| **Auth** | Bearer token (env `VANTADB_API_KEY`) |
| **Rate limiting** | tower_governor |
| **TLS** | rustls 0.23 (opt) |
| **Metrics** | Prometheus 0.14 (opt) + atomic fallback |
| **Tracing** | tracing + OpenTelemetry (opt) |
| **Python bindings** | PyO3 0.29 + maturin |
| **TypeScript SDK** | WASM (vantadb-wasm) |
| **MCP** | Custom stdio JSON-RPC |
| **Frontend** | React 19, Vite 8, Tailwind v4, TanStack Router |
| **Animaciones frontend** | GSAP 3.15 + Motion 12.42 + AnimeJS 4.5 |
| **Concurrencia** | parking_lot, dashmap 6, arc-swap 1.7 |
| **Memory allocation** | mimalloc (opt), jemalloc (opt, Unix) |
| **Error handling** | thiserror 2.0 |

### A.3 Workspace Members (Cargo.toml)

| Crate | Propósito |
|-------|-----------|
| `vantadb` (root) | Core engine, CLI, HTTP server feature, SDK |
| `vantadb-python` | Python native extension via PyO3 |
| `vantadb-server` | Axum HTTP server binary |
| `vantadb-mcp` | Model Context Protocol stdio server |
| `vantadb-wasm` | WASM compilation target |

---

## B. ANÁLISIS BACKEND RUST

### B.1 API Endpoints

| Method | Route | Auth | Handler | Description |
|--------|-------|------|---------|-------------|
| GET | `/health` | No | `health_check` | Health check |
| GET | `/metrics` | No | `metrics_endpoint` | Prometheus metrics |
| POST | `/api/v2/query` | Bearer | `execute_query` | IQL query execution |

> Solo 3 endpoints. Todo (CRUD, search, graph) pasa por el endpoint único de IQL.

### B.2 Middleware Stack

```
TraceLayer (tower_http tracing)
  → request_metrics_middleware (latency, method, route, status)
    → Branch: public vs protected
      → [protected] GovernorLayer (rate limit, default 100 req/min)
        → [protected] auth_middleware (Bearer token)
          → handler
```

### B.3 Error Handling (VantaError)

| Variant | Uso |
|---------|-----|
| `NodeNotFound(u64)` | Node no existe |
| `DuplicateNode(u64)` | ID duplicado en insert |
| `DimensionMismatch { expected, got }` | Dimensión vector incorrecta |
| `WalError(String)` | Falla genérica de WAL |
| `WALVersionMismatch { expected, found, hint }` | Migración de formato |
| `SerializationError(String)` | Falla de bincode/serde |
| `IoError(std::io::Error)` | Error de I/O (auto-from) |
| `IncompatibleFormat { ... }` | Magic/version mismatch |
| `NotInitialized` | Engine no abierto |
| `ResourceLimit(String)` | OOM / backpressure |
| `Execution(String)` | **Catch-all con TODO para refactorizar** |
| `DatabaseBusy(String)` | Lock contention |

### B.4 Cuellos de Botella Críticos (Backend)

#### B.4.1 N+1 en Graph Traversal
- **Archivo:** `src/graph.rs:30-80`
- **Problema:** BFS/DFS llama `storage.get(curr_id)` **uno a uno** por cada nodo descubierto. Sin batch loading.
- **Impacto:** Graph traversal escala O(n) en latencia de red.

#### B.4.2 N+1 en PhysicalScan
- **Archivo:** `src/physical_plan.rs:58-86`
- **Problema:** Por cada key del backend scan, llama `storage.get(id)` individualmente.
- **Impacto:** 10K keys = 10K get() individuales.

#### B.4.3 N+1 en Vector Search
- **Archivo:** `src/physical_plan.rs:243-255`
- **Problema:** Después de obtener hits de HNSW, itera cada resultado llamando `storage.get(id)`.

#### B.4.4 N+1 en Hybrid Search Explain
- **Archivo:** `src/sdk/search.rs:79-92`
- **Problema:** Por cada hit llama `debug_explain_hit()` que hace text index lookups.

#### B.4.5 WAL Mutex Contention
- **Archivo:** `src/storage.rs`
- **Problema:** `Mutex<Option<WalWriter>>` serializa todas las escrituras al WAL.
- **Impacto:** Bajo write throughput, cuello de botella en inserción concurrente.

#### B.4.6 spawn_blocking Cap
- **Archivo:** `src/cli_server.rs`
- **Problema:** Todas las queries se ejecutan en `spawn_blocking` con semaphore cap default 16.
- **Impacto:** Límite duro de concurrencia.

#### B.4.7 Archivos Monolito
| Archivo | Líneas | Problema |
|---------|--------|----------|
| `src/storage.rs` | ~2624 | Difícil de navegar, probar, mantener |
| `src/index.rs` | ~2044 | Mezcla lógica HNSW + serialización + E/S |
| `src/metrics.rs` | ~1300 | Boilerplate repetitivo por métrica |
| `src/cli_server.rs` | ~687 | init_telemetry repetitivo (280-438) |

### B.5 Code Smells Backend

1. **`Execution(String)` catch-all** — Variante con TODO en fuente para refactorizar en tipadas.
2. **Lógica duplicada `append_to_vstore` / `write_node_to_vstore`** (`storage.rs:1170-1257`) — ~40 líneas casi idénticas.
3. **Patrón WAL repetido en engine.rs** — `if let Some(ref mut wal) = *self.wal.lock() { wal.append(...) }` repetido en insert, update, delete.
4. **`read_only` check repetido** — 5 veces en `sdk/api.rs` (rebuild_index, compact_layout, flush, compact_wal, purge_expired).
5. **init_telemetry massivo** (`cli_server.rs:280-438`) — Bloques if/else repetidos para json/full/compact x mcp/no-mcp.
6. **Magic numbers** — `1024` capacity, `64` byte alignment, `0x8` tombstone, `0.80` RSS threshold.
7. **Comentarios mezclados español/inglés** en `storage.rs`, `wal.rs`.
8. **Sin `#![warn(missing_docs)]`** en ningún crate del workspace.
9. **init_telemetry function** — ~160 líneas de configuración tracing_subscriber repetitiva.

### B.6 Auth Analysis

- **Mecanismo:** Bearer token desde env `VANTADB_API_KEY`
- **Problemas:**
  - Sin RBAC (un solo token para todo)
  - Sin scoped permissions
  - Sin token rotation
  - Comparación string NO constant-time (`==` en vez de `subtle::ConstantEq`)
  - Sin rate limiting en auth failures
  - `/metrics` es público (sin auth)

---

## C. ANÁLISIS FRONTEND WEB

### C.1 Stack Frontend

| Aspecto | Detalle |
|---------|---------|
| Framework | React 19 |
| Build tool | Vite 8 |
| CSS | Tailwind v4 (CSS-first, tokens.css) |
| Router | TanStack Router (file-based, v1.168) |
| Data fetching | TanStack React Query v5 (configurado pero **sin uso**) |
| Estado global | Ninguno (solo React Query context) |
| Animaciones | GSAP 3.15 + Motion 12.42 + AnimeJS 4.5 |
| Fonts | Space Grotesk (display), Outfit (body), JetBrains Mono (mono) |
| Icons | Lucide (declarado en components.json, no en package.json) |
| UI Library | shadcn/ui New York style (configurado, **0 componentes instalados**) |

### C.2 Páginas / Rutas (23 totales)

| Ruta | Archivo | Props |
|------|---------|-------|
| `/` | `index.tsx` | Homepage (8 secciones) |
| `/architecture` | `architecture.tsx` | Engine deep-dive |
| `/changelog` | `changelog.tsx` | Release timeline |
| `/config` | `config.tsx` | Zero-config comparison |
| `/cost` | `cost.tsx` | Cost comparison ($0 vs $200/mo) |
| `/docs` | `docs.tsx` | Documentation examples |
| `/docs-api` | `docs-api.tsx` | API reference |
| `/engine` | `engine.tsx` | Core engine (1085 lines) |
| `/integrations` | `integrations.tsx` | Framework selector |
| `/latency` | `latency.tsx` | Latency pipeline simulator |
| `/maint` | `maint.tsx` | Ops comparison |
| `/pricing` | `pricing.tsx` | Pricing tiers + FAQ |
| `/security` | `security.tsx` | Security posture |
| `/storage` | `storage.tsx` | Storage architecture |
| `/use-cases` | `use-cases.tsx` | 8 production patterns |
| `/about/` | `about/index.tsx` | About hub |
| `/about/company` | `about/company.tsx` | Company values |
| `/about/community` | `about/community.tsx` | Community |
| `/about/contact` | `about/contact.tsx` | Contact + security |
| `/about/roadmap` | `about/roadmap.tsx` | Strategic roadmap |
| `/blog/` | `blog/index.tsx` | Blog list |
| `/blog/$slug` | `blog/$slug.tsx` | Single blog post |
| `/solutions/ai-agents` | `solutions/ai-agents.tsx` | AI agents use case |
| `/solutions/ai-ide-tooling` | `solutions/ai-ide-tooling.tsx` | IDE tooling |
| `/solutions/local-rag` | `solutions/local-rag.tsx` | Local RAG |
| `/product/benchmarks` | `product/benchmarks.tsx` | Benchmarks table |

### C.3 Problemas Críticos Frontend

#### C.3.1 Triplicación de Librerías de Animación

| Librería | Propósito | Bundle size (est.) |
|----------|-----------|-------------------|
| GSAP 3.15 | Scroll animations, typewriter, count-up | ~45KB |
| Motion 12.42 | Route transitions (AnimatePresence) | ~80KB |
| AnimeJS 4.5 | Text scramble effect | ~30KB |

> GSAP maneja el 95% de las animaciones. Motion solo route transitions. AnimeJS solo text scramble. Las 3 juntas añaden ~155KB+ al bundle.

#### C.3.2 Estilos Inline Masivos

- ~80% de los estilos son inline `style={{}}`
- `engine.tsx`: 1085 líneas (la mayoría inline styles)
- `architecture.tsx`: 557 líneas
- Problemas:
  - Nuevos objetos en cada render (garbage collection pressure)
  - Sin cacheo de CSS
  - Imposible de sobrescribir con temas
  - Mala DX de mantenimiento

#### C.3.3 Over-engineering de Routing

TanStack Router con file-based routing + auto-generated route tree para 23 páginas mayormente estáticas es excesivo. React Router plano sería:
- Más simple
- Menos dependencias (react-router vs @tanstack/react-router + @tanstack/router-plugin)
- Sin necesidad de `routeTree.gen.ts` con `@ts-nocheck`

#### C.3.4 Patrón de Comparación Duplicado

El mismo layout "Legacy vs VantaDB" está repetido manualmente en 7+ archivos:
- `config.tsx`, `cost.tsx`, `latency.tsx`, `maint.tsx`, `storage.tsx`, `company.tsx`
- `solutions/ai-agents.tsx`, `solutions/ai-ide-tooling.tsx`, `solutions/local-rag.tsx`

#### C.3.5 Sin Lazy Loading

- `React.lazy()` no usado en ninguna ruta
- Todas las páginas se cargan eager en el bundle principal
- Potencial de mejora: code splitting automático con Vite

#### C.3.6 Sin Memoization

- 0 usos de `React.memo`
- 0 usos de `useMemo`
- 0 usos de `useCallback`
- Componentes como Nav, SwissFooter, SwissSubpageHero se rerenderizan en cada navegación

#### C.3.7 Mutación Directa del DOM

- `onMouseEnter`/`onMouseLeave` handlers mutan `element.style` directamente
- Ejemplos: `about/index.tsx`, `about/community.tsx`, `SwissArchSection.tsx`
- Antipatrón en React: rompe el virtual DOM, causa inconsistencias

#### C.3.8 Sin Tests Frontend

- **0 tests** para el frontend
- No hay Vitest configurado para componentes
- No hay React Testing Library
- No hay Playwright para E2E

### C.4 Componentes Actuales

```
Layout:
├── __root.tsx -> Nav + AnimatePresence + Outlet + SwissFooter + SwissBackToTop

Homepage (index.tsx):
├── SwissHero (GSAP animated)
├── SwissBenchmarkGrid (bento + count-up)
├── SwissQuickstart (interactive terminal + typewriter)
├── SwissCoreEngine (pinned scroll + feature cards)
├── SwissArchSection (interactive layer diagram)
├── SwissUseCases (hover cards)
├── SwissEcosystem (integration grid)
└── SwissMonolith (CTA section)

Reusables:
├── SwissSubpageHero (num + eyebrow + title + sub)
├── VantaDBLogo (4 variants, 5 sizes)
├── Nav (fixed, scroll-aware glass)
├── SwissFooter (4-column dark grid)
└── SwissBackToTop (scroll-triggered)

Inline page components:
├── GraphTopology (engine.tsx)
├── RRFWeightsSlider (engine.tsx)
├── WALSimulator (engine.tsx)
├── ArchitecturePipeline (engine.tsx)
├── PerformanceProfiler (architecture.tsx)
└── SpecRow (architecture.tsx)
```

### C.5 Dependencias Frontend (package.json)

**Runtime (21):**
`react`, `react-dom`, `@tanstack/react-router`, `@tanstack/react-query`, `@tanstack/router-plugin`, `gsap`, `@gsap/react`, `motion`, `animejs`, `@types/animejs`, `tailwindcss`, `@tailwindcss/vite`, `tailwind-merge`, `clsx`, `tw-animate-css`, `vite-tsconfig-paths`, `@fontsource-variable/space-grotesk`, `@fontsource-variable/outfit`, `@fontsource-variable/jetbrains-mono`, `marked`

**Dev (12):**
`typescript`, `vite`, `@vitejs/plugin-react`, `@types/react`, `@types/react-dom`, `@types/node`, `eslint`, `@eslint/js`, `eslint-plugin-react-hooks`, `eslint-plugin-react-refresh`, `eslint-plugin-prettier`, `prettier`, `typescript-eslint`

**Notablemente ausente:**
- `lucide-react` (declarado en components.json pero no instalado)
- `@radix-ui/*` (dependencias de shadcn/ui, no instaladas)
- Testing: `vitest`, `@testing-library/react`, `@playwright/test`
- Estado global: `zustand`, `jotai` (innecesario para sitio estático)

---

## D. ANÁLISIS DE BASE DE DATOS Y ALMACENAMIENTO

### D.1 Entidades Core

#### UnifiedNode (src/node.rs:416-445)
| Campo | Tipo | Descripción |
|-------|------|-------------|
| `id` | `u64` | Identificador único global |
| `bitset` | `u128` | Filtro rápido de 128 bits (país, rol, activo, etc.) |
| `semantic_cluster` | `u32` | Cluster semántico para super-node routing |
| `flags` | `NodeFlags` | Bitfield: ACTIVE, INDEXED, DIRTY, TOMBSTONE, HAS_VECTOR, HAS_EDGES, PINNED, RECOVERED, INVALIDATED, CONFLICT_RESOLVED |
| `vector` | `VectorRepresentations` | Vector multi-tier: Binary (L1), Turbo/3-bit (L2), SQ8 (L2.5), Full f32 (L3), MmapFull |
| `epoch` | `u32` | Versión de lineage |
| `edges` | `Vec<Edge>` | Aristas dirigidas label+weight |
| `relational` | `RelFields` (BTreeMap) | Campos dinámicos schema-less |
| `tier` | `NodeTier` | Hot (RAM) o Cold (disk) |
| `hits` | `u32` | Frecuencia de acceso |
| `last_accessed` | `u64` | Unix ms |
| `confidence_score` | `f32` | 0.0-1.0 |
| `importance` | `f32` | 0.0-1.0 |
| `ext_metadata` | `HashMap<String, Vec<u8>>` | Extensión forward-compatible |

#### Edge (src/node.rs:187-222)
| Campo | Tipo | Descripción |
|-------|------|-------------|
| `target` | `u64` | Node ID destino |
| `label` | `String` | Tipo de relación (e.g. "knows") |
| `weight` | `f32` | Peso float |

#### FieldValue (src/node.rs:226-240)
String, Int(i64), Float(f64), Bool, DateTime, List<T> de cada uno, y Null.

### D.2 Particiones Lógicas (BackendPartition, 8 totales)

| Partition | Propósito | Rebuildable |
|-----------|-----------|-------------|
| Default | Node metadata + relational fields | — |
| TombstoneStorage | Tombstone archive | ❌ No |
| CompressedArchive | Semantic summaries | ❌ No |
| Tombstones | is_deleted checks | ❌ No |
| NamespaceIndex | namespace/key lookup | ✅ Sí |
| PayloadIndex | Metadata equality filter | ✅ Sí |
| TextIndex | BM25 inverted index | ✅ Sí |
| InternalMetadata | Checkpoints, schema versions | ❌ No |

> Las 3 derived indexes (NamespaceIndex, PayloadIndex, TextIndex) son **no autoritativas** — pueden ser reconstruidas desde Default.

### D.3 Formatos Binarios Versionados

| Componente | Magic Bytes | Versión |
|------------|-------------|---------|
| VantaFile | `b"VFLE"` | 1 |
| Vector Index | — | 4 |
| Text Index | — | 3 (basic) / 4 (advanced-tokenizer) |
| WAL | `b"VWAL"` | 1 |
| VantaHeader | — | 1 |

### D.4 Índices

| Tipo | Archivo | Descripción |
|------|---------|-------------|
| HNSW (ANN) | `src/index.rs` | CPIndex, DashMap, SIMD f32x8 cosine, BFS serialization |
| BM25 (Text) | `src/text_index.rs` | k1=1.2, b=0.75, tantivy multilingual optional |
| Bitset 128-bit | `node.rs` | Fast mask matching (single AND) |
| Derived Namespace | `backend.rs` | namespace/key lookups |
| Derived Payload | `backend.rs` | metadata equality filters |
| Cardinality Stats | `storage.rs` | HashMap<String, HashMap<String, usize>> |

### D.5 Problemas de Base de Datos

#### D.5.1 Sin Índices Secundarios Escalares
- `filter_field()` en InMemoryEngine hace **full table scan**
- No hay índices hash/B-tree para campos relacionales comunes

#### D.5.2 Sin Índice de Aristas Global
- Graph edges almacenados localmente en cada nodo
- No hay tabla de adyacencia global o inversa
- Traversal requiere cargar nodo por nodo

#### D.5.3 Sin Restricciones de Integridad Referencial
- `Edge.target: u64` no valida que el nodo exista
- Borrar nodo no remueve edges entrantes (dangling edges)
- No hay `ON DELETE CASCADE`

#### D.5.4 Sin ACID Transaccional
- WAL: durabilidad pero sin atomicidad multi-operación
- No hay two-phase commit entre VantaFile + KV backend + WAL
- No hay `BEGIN TRANSACTION` / `COMMIT` / `ROLLBACK`

#### D.5.5 Bitset Limitado a 128 bits
- Máximo 128 filtros categóricos distintos
- Para sistemas con muchos tenants/categorías, se queda corto

#### D.5.6 N+1 Query Patterns (Detalle)

| Ubicación | Patrón | Impacto |
|-----------|--------|---------|
| `graph.rs:bfs_traverse()` | `storage.get()` por nodo en BFS loop | Graph traversal O(n*m) |
| `graph.rs:dfs_traverse()` | `storage.get()` por nodo en DFS loop | Idem |
| `graph.rs:topological_sort()` | `storage.get()` por nodo visitado | Idem |
| `physical_plan.rs:PhysicalScan::next()` | `storage.get(id)` por cada key de backend scan | 10K keys = 10K gets |
| `physical_plan.rs:243-255` | `storage.get(id)` por cada HNSW hit | Search O(k) gets |
| `sdk/search.rs:79-92` | Per-hit text index lookup en explain | Explain lento |

---

## E. ANÁLISIS DE TESTING

### E.1 Frameworks y Configuraciones

| Lenguaje | Framework | Runner | Config |
|----------|-----------|--------|--------|
| Rust | `#[test]` + `#[tokio::test]` | cargo nextest (CI), cargo test (local) | `.config/nextest.toml` |
| Rust (property) | proptest | Via cargo test | `tests/fuzz_proptest.rs` |
| Rust (fuzz) | cargo-fuzz + libfuzzer-sys | `cargo +nightly fuzz run` | `fuzz/Cargo.toml` |
| Rust (bench) | criterion | `cargo bench` | `benches/*.rs` |
| TypeScript | Vitest v4.1.9 | `vitest run` | `vantadb-ts/vitest.config.ts` |
| Python | pytest v9.x | `python -m pytest` | Sin archivo de configuración |
| Coverage | cargo-llvm-cov | CI (LCOV) | En `rust_ci.yml` |

### E.2 Conteo de Tests

| Categoría | Count | Calidad |
|-----------|-------|---------|
| Rust unit tests (inline `#[cfg(test)]`) | ~328 | ✅ Excelente cobertura |
| Rust integration tests (`tests/`) | ~228 (61 files) | ✅ Muy completo |
| Rust heavy certification | ~38 | ✅ Profesional (SIFT, stress) |
| Rust server tests | ~28 | ✅ E2E HTTP + auth |
| Rust MCP tests | 9 | ✅ Protocolo |
| Rust fuzz targets | 2 | ⚠️ Básico (parser + deserialize) |
| Rust benchmark files | 6 | ✅ Criterion |
| **WASM tests** | **0** | 🔴 **ARCHIVO VACÍO** (`vantadb-wasm/tests/wasm_tests.rs`) |
| TypeScript tests | 18 | ⚠️ Mínimo (2 test files) |
| Python SDK tests | 46 | ✅ Bueno para binding layer |
| Python integration tests | 10 | ⚠️ Básico |
| **Frontend tests** | **0** | 🔴 **CRÍTICO** |
| **Security tests** | **0** | 🔴 **CRÍTICO** |

### E.3 Patrones de Testing

- **Mocking:** Casi nulo. Tests usan instancias reales (tempdir, real StorageEngine, real WASM). Solo 1 mock: `FakeEmbeddings` en LangChain test.
- **Failpoints:** `fail` crate para fault injection testing (en vez de mocking tradicional).
- **E2E:** `vantadb-server/tests/e2e.rs` — TCP listener real con axum, reqwest HTTP client.
- **Property-based:** `tests/fuzz_proptest.rs` — proptest! para WAL, Node, IDs, delete idempotency.

### E.4 CI Profiles (nextest.toml)

| Profile | Filter | test-threads | Uso |
|---------|--------|-------------|-----|
| `audit` | Excluye ~40 heavy tests | 2 | CI push/PR (rápido) |
| `experimental` | parser, executor, governor | default | Experimental |
| `chaos` | chaos_integrity_failpoints | 1 | Weekly certification |

### E.5 Gaps de Testing

1. **WASM: 0 tests** — Archivo `vantadb-wasm/tests/wasm_tests.rs` vacío
2. **Frontend: 0 tests** — Sin Vitest, RTL, ni Playwright
3. **Security: 0 tests** — Sin SQL injection en IQL, auth bypass, input validation fuzzing
4. **TypeScript SDK: solo 18 tests** — Debería tener ~50+ con edge cases
5. **Sin regression suite** — No hay tests específicos para bugs ya corregidos
6. **Sin snapshot testing** — Para certificación HNSW recall, export/import format
7. **Sin load/stress en TS/Python** — Solo Rust tiene stress tests

---

## F. ANÁLISIS DE DOCUMENTACIÓN

### F.1 Inventario Completo (~155 archivos)

#### Root (6)
| Archivo | Idioma | Estado |
|---------|--------|--------|
| `README.md` | EN | Bueno, broken links a `.github/` |
| `README_ES.md` | ES | Traducción completa, mismos broken links |
| `CONTRIBUTING.md` | EN | 135 líneas, buena calidad |
| `LICENSE` | EN | Apache 2.0 |
| `cliff.toml` | — | Changelog generator config |
| `auditoria-workflows.md` | ES | Audit findings (merged here) |

#### docs/ (Obsidian Vault, ~140 archivos)

| Subdirectorio | Archivos | Propósito |
|---------------|----------|-----------|
| `docs/` root | 3 | README, master-index, QUICKSTART |
| `docs/api/` | 4 | EMBEDDED_SDK (428l), PYTHON_SDK (378l), HTTP_API (149l), MCP (257l) |
| `docs/architecture/` | 4 | ARCHITECTURE (459l), WAL, TEXT_INDEX, ADVANCED_TOKENIZER |
| `docs/architecture/adr/` | 3 | ADR 001-003 (solo 3!) |
| `docs/operations/` | 22 | CONFIGURATION, BENCHMARKS, CI_POLICY, DURABILITY, FUZZING, GRAFANA, etc. |
| `docs/glosario/` | 54 | README + 53 términos técnicos |
| `docs/migration/` | 2 | FROM_CHROMADB, FROM_LANCEDB |
| `docs/strategy/` | 2 | ROADMAP (213l), GO_TO_MARKET (461l) |
| `docs/case_studies/` | 2 | RAG edge device, Agent + Ollama |
| `docs/experimental/` | 1 | IQL historical |
| `docs/archive/` | 4 | Closeouts, reports |
| `docs/research/` | 1 | SQL_ANALYSIS |
| `docs/web/` | 4 | PLAYWRIGHT_CLI, QA report, strategy, product |
| `docs/progreso/` | 1 | Progress dashboard |
| `docs/_templates/` | 4 | ADR, note, glossary-term, devlog-entry |

#### Sub-project READMEs (5)
`vantadb-python/README.md` (ES), `vantadb-ts/README.md`, `Formula/README.md`, `integrations/langchain/README.md`, `integrations/llamaindex/README.md`

#### Blog posts (4 in web/content/blog)
`introducing-vantadb.md`, `how-hybrid-search-works.md`, `sqlite-for-ai-agents.md`, `why-i-built-vantadb-local-memory-engine.md`

### F.2 Problemas de Documentación

#### 🔴 Críticos

1. **SECURITY.md / SUPPORT.md / CODE_OF_CONDUCT.md no existen** — Aunque README.md los referencia en `.github/`
   - El directorio `.github/` **no existe** en el repo
   - Todos los links son 404

2. **Broken links en README.md y README_ES.md**
   - `.github/CONTRIBUTING.md`, `.github/SECURITY.md`, `.github/SUPPORT.md` no existen
   - `docs/vision/VISION.md` no existe (referenciado por ROADMAP.md y GO_TO_MARKET.md)

3. **Blog post con error factual**
   - `introducing-vantadb.md`: "License: MIT" → **es Apache 2.0**
   - GitHub link apunta a `vantadb/vantadb` → **es `ness-e/Vantadb`**

4. **`llms.txt` desactualizado**
   - Menciona "v0.4.0 → v0.6.0" pero el proyecto está en v0.2.0

#### 🟡 Altos

5. **Solo 3 ADRs para todo el proyecto** — Faltan decisiones clave:
   - Fjall vs RocksDB (criterios de selección)
   - HNSW params (M=16, ef_construction=200, ef_search=100)
   - RRF constant (k=60)
   - PyO3 binding architecture
   - WASM support strategy
   - Community governance model

6. **Mezcla de idiomas inconsistente**
   - Glosario con títulos en español (`busqueda-hibrida` en vez de `hybrid-search`)
   - `vantadb-python/README.md` completamente en español en proyecto inglés
   - Comentarios español/inglés mezclados en código Rust

7. **Sin tutoriales prácticos**
   - No hay tutorial de "AI Agent Memory con VantaDB"
   - No hay "Local RAG Pipeline" walkthrough
   - No hay "Migrating from ChromaDB" paso a paso (solo documentación técnica)

8. **Sin performance tuning guide**
   - No hay guía para ajustar HNSW params, memory limits, backend selection, sync modes

#### 🔵 Medios

9. **HTTP_API.md escueto** — Solo 3 endpoints documentados (149 líneas vs 428 de EMBEDDED_SDK)
10. **Sin OpenAPI/Swagger spec**
11. **ADN de documentación:** No hay workflow de contribución de docs en CONTRIBUTING.md
12. **Sin VISION.md** — Referenciado pero no existe
13. **Sin "Getting Started" tutorial series**
14. **Sin architecture diagrams formales** (solo ASCII art en ARCHITECTURE.md)
15. **54 términos en glosario pero faltan:** `similar_to_key`, `put_batch`, `compaction`, `serialization`, `heuristic_search`

### F.3 CHANGELOG Analysis

| Aspecto | Detalle |
|---------|---------|
| Archivo | `docs/CHANGELOG.md` (552 líneas) |
| Formato | Keep a Changelog + SemVer |
| Versiones | v0.1.0-rc1 → v0.2.0 |
| Estilo | Bilingüe (EN changelog + ES task logs) |
| Fortaleza | Extremadamente detallado con task IDs (AUD-*, TSK-*) |
| Debilidad | Muy largo, secciones español al final duplican info |

---

## G. ANÁLISIS CI/CD Y DEVOPS

### G.1 Workflows CI/CD (8 archivos)

| Workflow | Trigger | Jobs |
|----------|---------|------|
| `rust_ci.yml` | Push/PR a main | build (fmt+clippy+nextest+cargo-audit+deny), windows-check, coverage |
| `release.yml` | Tag v*.*.* | gate, build-and-deploy (matrix ubuntu/macos/windows) |
| `python_wheels.yml` | PR python paths, tags, manual | build-wheels, publish-pypi, verify-pypi |
| `heavy_certification.yml` | Weekly/manual | 10 jobs (stress, HNSW, SIFT, failpoints, storage, text-index, etc.) |
| `web-ci.yml` | Push/PR a main on web/ | lint, tsc, build |
| `bench.yml` | Push/PR on src/ | Benchmark + BENCHMARKS.md update |
| `nightly_bench.yml` | Daily 03:00 UTC | 5 Criterion benchmarks + regression detection |
| `dependabot.yml` | Weekly | Cargo + pip + GHA updates |

### G.2 Dependabot Configuration

- **Cargo:** Weekly, grouped, ignore sysinfo >=0.31
- **pip:** Weekly (vantadb-python)
- **GitHub Actions:** Weekly

### G.3 Problemas CI/CD

1. **No Dockerfile** — El proyecto no tiene Dockerfile ni docker-compose.yml
2. **test-threads = 2 global** — Renderiza lento en Linux/macOS
3. **chaos_integrity** sin `required-features = ["failpoints"]` en Cargo.toml
4. **Sin CodeQL analysis** en CI
5. **Sin verificación de building de docs** (mdbook, docs.rs)
6. **Sin deploy automático de web** a Vercel en CI (aunque hay config de Vercel)
7. **Sin signed releases** — Windows SmartScreen warning documentado

### G.4 Entorno y Configuración

| Variable | Default | Descripción |
|----------|---------|-------------|
| `VANTADB_STORAGE_PATH` | `vantadb_data` | Directorio de datos |
| `VANTADB_HOST` | `127.0.0.1` | Bind address |
| `VANTADB_PORT` | `8080` | Puerto servidor |
| `VANTADB_API_KEY` | (ninguno) | Bearer token |
| `VANTADB_RATE_LIMIT_RPM` | `100` | Rate limit |
| `VANTADB_MAX_BLOCKING_THREADS` | `16` | Semaphore cap |
| `VANTA_BACKEND` | `fjall` | `rocksdb`, `memory`, o `fjall` |
| `VANTADB_MEMORY_LIMIT` | (ninguno) | Memoria máxima |
| `VANTADB_LOG_FORMAT` | `compact` | Formato de log |

---

## H. RECOMENDACIONES CONSOLIDADAS

### FASE 1 — Inmediato (1-2 semanas)

| # | Acción | Área | Impacto | Archivos/Directorios Involucrados |
|---|--------|------|---------|----------------------------------|
| 1 | Fix broken links en README.md y README_ES.md | Docs | 🔴 Alto | `README.md`, `README_ES.md` |
| 2 | Crear `.github/` con SECURITY.md, SUPPORT.md, CODE_OF_CONDUCT.md, templates | Docs/DevOps | 🔴 Alto | `.github/` (nuevo directorio) |
| 3 | Eliminar redundancia de animation libraries (quitar Motion o AnimeJS) | Frontend | 🟡 Medio | `web/src/__root.tsx`, `web/package.json` |
| 4 | Fix blog factual errors (License: MIT → Apache 2.0, GitHub URL) | Docs | 🟡 Medio | `web/content/blog/introducing-vantadb.md` |
| 5 | Implementar tests WASM reales | Testing | 🔴 Alto | `vantadb-wasm/tests/wasm_tests.rs` |
| 6 | Refactor `Execution(String)` → variantes tipadas | Backend | 🟡 Medio | `src/error.rs` |
| 7 | Fix `chaos_integrity` required-features | CI/CD | 🟠 Medio | `Cargo.toml` |
| 8 | Hacer `/metrics` endpoint auth-required o documentar que es público | Backend/Security | 🟠 Medio | `src/cli_server.rs` |

### FASE 2 — Corto Plazo (2-4 semanas)

| # | Acción | Área | Impacto | Archivos/Directorios Involucrados |
|---|--------|------|---------|----------------------------------|
| 9 | Batch KV loader (`get_many`) para eliminar N+1 en graph/scan/search | Backend | 🔴 Alto | `src/backend.rs`, `src/storage.rs`, `src/graph.rs`, `src/physical_plan.rs` |
| 10 | Refactor inline styles → Tailwind classes (empezar con Homepage) | Frontend | 🔴 Alto | `web/src/routes/index.tsx`, `web/src/styles/` |
| 11 | Crear 5-7 ADRs faltantes | Docs | 🟡 Medio | `docs/architecture/adr/` |
| 12 | Componente `<VsTable data={...} />` reusable | Frontend | 🟡 Medio | `web/src/components/VsTable.tsx` |
| 13 | Implementar `React.lazy()` por ruta | Frontend | 🟡 Medio | `web/src/routes/` |
| 14 | Crear Dockerfile multi-stage + docker-compose | DevOps | 🟡 Medio | `Dockerfile`, `docker-compose.yml` |
| 15 | Update `llms.txt` con versión actual | Docs/AI-SEO | 🟠 Medio | `web/public/llms.txt` |
| 16 | Corregir `llms.txt` version (v0.4.0→v0.6.0 → v0.2.0) | Docs | 🟠 Medio | `web/public/llms.txt` |

### FASE 3 — Medio Plazo (1-2 meses)

| # | Acción | Área | Impacto | Archivos/Directorios Involucrados |
|---|--------|------|---------|----------------------------------|
| 17 | Split archivos monolito (storage.rs, metrics.rs, index.rs) | Backend | 🟡 Medio | `src/storage.rs` → `src/storage/mod.rs`, `src/storage/vanta_file.rs`, etc. |
| 18 | Global edge index + cascade delete | Database | 🟡 Medio | `src/node.rs`, `src/backend.rs` |
| 19 | Performance tuning guide oficial | Docs | 🟡 Medio | `docs/operations/PERFORMANCE_TUNING.md` |
| 20 | Tutorial series (AI Agent, RAG, Migration walkthrough) | Docs | 🟡 Medio | `docs/tutorials/` |
| 21 | Configurar Vitest + React Testing Library para frontend | Testing | 🟡 Medio | `web/vitest.config.ts`, `web/src/**/*.test.tsx` |
| 22 | Memory governor con métricas de eviction visibles | Backend | 🟠 Medio | `src/governor.rs`, `src/metrics.rs` |
| 23 | OpenAPI/Swagger spec para HTTP API | Docs | 🟠 Medio | `docs/api/openapi.yaml` |

### FASE 4 — Largo Plazo (3+ meses)

| # | Acción | Área | Impacto |
|---|--------|------|---------|
| 24 | Índices secundarios escalares (filter_field_index) | Database | 🔴 Alto |
| 25 | Migration runner (`vanta-cli migrate`) | Backend/Database | 🔴 Alto |
| 26 | RBAC en API server (multi-usuario, scoped tokens) | Backend/Security | 🔴 Alto |
| 27 | Web dashboard (React admin panel para monitoreo) | Frontend | 🟡 Medio |
| 28 | Plugin system para storage backends custom | Backend/Architecture | 🟡 Medio |
| 29 | OPFS persistence para WASM | Backend/WASM | 🟡 Medio |
| 30 | reemplazar bincode por postcard o rkyv | Backend/Security | 🟡 Medio |

---

## I. NOTAS TÉCNICAS ADICIONALES

### I.1 Configuración de Rust

- `rust-toolchain.toml`: stable, componentes: rustfmt, clippy, rust-src
- Targets: `x86_64-pc-windows-msvc`, `x86_64-unknown-linux-gnu`
- `.cargo/config.toml`: linker = "link.exe" (Win), rustflags: `-C target-cpu=native`
- Build profiles: release (LTO=thin, opt-level=3), CI (release sin LTO), dev (opt-level=1), test (opt-level=0)

### I.2 Versiones de Formato de Archivos

```
VantaFile   → v1 (magic: b"VFLE")
VectorIndex → v4
TextIndex   → v3 (basic), v4 (advanced-tokenizer)
WAL         → v1 (magic: b"VWAL")
```

### I.3 Configuración HNSW

| Parámetro | Valor | Descripción |
|-----------|-------|-------------|
| M | 32 | Conexiones por nodo |
| M_max0 | 64 | Máx conexiones en capa 0 |
| ef_construction | 200 | Tamaño de búsqueda durante construcción |
| ef_search | 100 | Tamaño de búsqueda durante query |
| ml | 1.0/ln(32) | Factor de distribución de capas |
| Métrica | Cosine (default) o Euclidean | Distancia vectorial |

### I.4 BM25 Configuration

| Parámetro | Valor |
|-----------|-------|
| k1 | 1.2 |
| b | 0.75 |
| Tokenizer default | lowercase-ascii-alnum |
| Tokenizer advanced | tantivy-multilingual (feature) |

### I.5 Dependencias de Seguridad a Monitorear

| Crate | Issue | Tipo | Estado |
|-------|-------|------|--------|
| `bincode` 2.0 | RUSTSEC-2025-0141 | Unmaintained | ⚠️ Monitorear |
| `rustls-pemfile` | RUSTSEC-2025-0134 | Vulnerability | 🔴 Reemplazar |
| `instant` (via tantivy) | RUSTSEC-2024-0384 | Unmaintained | Ignorado en deny.toml |
| `lru` (via tantivy) | RUSTSEC-2026-0002 | Unsoundness | Ignorado en deny.toml |

### I.6 Hardware Capabilities

```
.vanta_profile:
  - AVX2: sí
  - Cores: 12 logical
  - RAM: ~34GB
  - Resource Score: 79
```

---

## J. SUMMARY SCORE CARD

| Categoría | Nota | Fundamento |
|-----------|------|------------|
| **Backend Architecture** | A | Sólido, modular, pero monolito y N+1 |
| **Backend Security** | B- | Auth básico, sin RBAC, timing attack |
| **Backend Performance** | B | N+1 queries, WAL mutex, spawn_blocking cap |
| **Frontend Architecture** | C+ | Over-engineered routing, inline styles, sin tests |
| **Frontend Performance** | C | 3 anim libs, sin lazy loading, sin memo |
| **Frontend Design** | A- | Swiss design system hermoso pero mal implementado |
| **Database Design** | B+ | Schema-less potente pero sin secondary indexes |
| **Database Constraints** | C | Sin FK, sin cascade, sin ACID transactions |
| **Testing Coverage** | A (Rust) / D (WASM, Frontend) | 667 tests en Rust pero 0 en WASM y Frontend |
| **Documentation Volume** | A | 155+ archivos |
| **Documentation Quality** | B | ADRs faltantes, broken links, errors factuales |
| **CI/CD** | A- | Profesional pero sin Docker |
| **DevOps** | C | Sin Docker, sin signed releases |

**Puntaje General: B+** (Sólido con áreas de mejora significativas)

---

## K. GLOSARIO DE TÉRMINOS USADOS EN ESTE REPORTE

| Término | Definición |
|---------|------------|
| **WAL** | Write-Ahead Log — log de escritura anticipada para durabilidad |
| **HNSW** | Hierarchical Navigable Small World — índice de vectores ANN |
| **BM25** | Best Match 25 — algoritmo de ranking de texto |
| **RRF** | Reciprocal Rank Fusion — fusión de rankings híbridos |
| **CPIndex** | Custom CP Index — implementación HNSW propia de VantaDB |
| **N+1** | Patrón donde se hace 1 query + N queries por cada resultado |
| **IQL** | Infinite Query Language — lenguaje de consulta propio |
| **LSM-tree** | Log-Structured Merge-Tree — estructura de datos de almacenamiento |
| **OPFS** | Origin Private File System — API de archivos en navegador |
| **MCP** | Model Context Protocol — protocolo de comunicación con LLMs |
| **SLSA** | Supply-chain Levels for Software Artifacts — nivel de seguridad de build |
| **RBAC** | Role-Based Access Control — control de acceso por roles |
| **ADR** | Architecture Decision Record — registro de decisiones arquitectónicas |
| **FFI** | Foreign Function Interface — interfaz entre lenguajes |

---

*Documento generado el 2 de julio de 2026 mediante análisis multi-agente automatizado.*
*Fuentes: 6 subagentes explorando en paralelo estructura, backend, frontend, base de datos, documentación y tests.*
