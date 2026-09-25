# 🏛️ VantaDB — Arquitectura, Modelo de Datos, Competencia y Oportunidades de Producto

> **Complemento de:** `VantaDB-Informe-Analisis-Completo.md` (análisis módulo a módulo de las 2 sesiones previas — 12 hallazgos críticos verificados, veredicto global 7.3/10, preparación de lanzamiento 4/10)
> **Método de esta fase:** 6 agentes de análisis en paralelo — arquitectura/patrones (4-h), modelo de datos (4-i), investigación web de 17 sistemas de memoria para agentes (4-j), investigación web de 10 BDs multi-modelo (4-k) — más verificación cruzada contra el worklog consolidado (Tasks 1, 2-a..2-e, 4-a..4-g).
> **Base:** repo clonado `develop` en `/home/z/vantadb` (~214,646 LOC Rust + 32,530 TS + 18,273 Python + 237K LOC docs), v0.6.1.

---

## ⚠️ 0. NOTA CRÍTICA SOBRE LOS ARCHIVOS ADJUNTOS

Intentaste entregarme tu investigación de Notion (`VantaDB-Docs-Unificado.md`, el ZIP de export y los 5 análisis .md) **5 veces**. Verifiqué con monitoreo en vivo, búsqueda en todo el filesystem y diagnóstico de los montajes FUSE/ossfs del sandbox: **los archivos nunca llegaron a este entorno** (el puente de subida quedó vacío en todos los intentos — falla de infraestructura de la plataforma, no de tu lado).

**Consecuencia:** este informe cruza TODO lo disponible (repo clonado + investigación web + análisis previo), pero **el cruce con tu visión original de Notion está pendiente**. Cuando me entregues el contenido por una vía que funcione (pegar el texto directo en el chat, o un gist/URL pública que descargo con `curl`), integro el análisis cruzado como capa adicional sin rehacer nada. La sección §13 deja preparada la plantilla de integración.

---

## 1. RESUMEN EJECUTIVO

| Dimensión | Veredicto | En una frase |
|---|:---:|---|
| **Arquitectura (separación de responsabilidades)** | **7.5/10** | Direcciones de dependencia correctas, documentadas y verificadas; grietas: proxy/MCP saltan la fachada SDK, feature-coupling, DTO drift sin mecanismo |
| **Aplicación de patrones de diseño** | **8/10** | 22 patrones bien elegidos sin sobre-ingeniería; pierde por OpGate×3 sin crate compartido y sharing por macro/`#[path]` |
| **Calidad de código** | **8/10** | Disciplina heroica convertida en infraestructura (lints deny, snapshot tests, 2,780 tests); carga muerta ~600 LOC institucionalizada |
| **Modelo de datos** | **6.5/10 como motor, 5/10 como memoria** | Base vector+metadata+edges correcta; pero la semántica de memoria vive por convención ENCIMA del modelo, no dentro |
| **Producto (concepto)** | **7.5/10** | La intersección embebido+híbrido+grafo+MCP+desktop es real y defendible |
| **Producto (preparación GTM)** | **4/10** | Embudo público roto, números no reconciliados, 0 usuarios externos |
| **Posición competitiva** | **Ventana abierta pero cerrándose** | Qdrant Edge acabó de saltar al embebido; MemOS/ReMe atacan la tesis local-first; el terreno libre es la **memoria verificable/gobernable** |

**Las 3 conclusiones que cambian la estrategia:**

1. **La memoria de VantaDB es "por convención", no de primera clase.** Namespaces-carpetas, JSON en payload, heat/supersession fuera del store, sin procedencia consultable como grafo, sin bi-temporalidad, sin entity linking. Los líderes del sector (Zep/Graphiti) ya tienen bi-temporalidad como primitiva. VantaDB puede pasar de "vector DB con convenciones de memoria" a "motor de memoria de primera clase" con 8 features concretas (§10).
2. **El hueco competitivo que NADIE ocupa (de los 17+ sistemas analizados): memoria verificable y gobernable.** Tamper-evident (hash-chain sobre el WAL que ya existe), borrado certificado (shred que ya existe, falta cablear), redacción persistida + namespaces cifrados (el proxy ya redacta en vivo), governance de inyección (solo VantaDB tiene proxy+memoria en el mismo paquete). Ninguna combinación de estas existe en Mem0, Zep, Letta, LangMem, Cognee, MemOS, MIRIX, A-MEM, Memobase, Supermemory, MemMachine, MemU, ReMe, Second-Me, Honcho, Hindsight ni OpenMemory.
3. **Qdrant Edge acaba de validar el nicho — y de amenazarlo.** El lanzamiento de la versión embebida in-process de Qdrant (con UpdateOperation log) confirma que "SQLite para agentes" es la categoría correcta, y a la vez significa que la ventana de mercado se está cerrando. La lección de Kùzu (compañía cerrada — el embebido puro sin funnel server/cloud no es negocio) es la contraparte: ejecutar embebido→server→managed cuanto antes.

---

## 2. ARQUITECTURA COMPLETA

### 2.1 Grafo de crates (verificado leyendo los 12 Cargo.toml)

```
                        ┌──────────────────── DESKTOP (Tauri) ───────────────────┐
                        │  src-tauri → reqwest HTTP + spawn vanta-cli + vantadb  │
                        │  + vanta-memory (default-features=false)               │
                        └───────────────────────────┬────────────────────────────┘
                                                    │
  ┌─────────────┐   ┌───────────────┐   ┌──────────▼──────────┐
  │ vantadb-ts  │──▶│ vantadb-wasm  │──▶│                     │      (no members:
  │ (pure TS)   │   │ (wasm32 feat) │   │                     │       vantadb-node
  └─────────────┘   └───────────────┘   │      VANTADB        │       [workspace] own)
  ┌─────────────┐                       │      (core lib)     │
  │vantadb-node │──────────────────────▶│  0 deps hacia fuera │      providers/* ×3
  │ (napi-rs)   │  Embedded + OpGate    │                     │       [PyO3 cdylib] →
  └─────────────┘                       │                     │       vantadb (fjall)
  ┌─────────────┐                       └──────────▲──────────┘
  │vantadb-     │──▶ Embedded                    │ │
  │python (PyO3)│                                │ │
  └─────────────┘   ┌──────────────────┐         │ │
  ┌──────────────┐  │ vantadb-server   │─────────┘ │
  │ vanta-proxy  │─▶│ (bin + --mcp)    │───────────┼──▶ vantadb-mcp ──▶ vanta-memory
  │ (⚠ sin fjall │  └──────────────────┘           │         (llm-driver OFF ⚠)
  │  por defecto)│───▶ vanta-memory ──▶ vantadb::sdk (única dependencia, ejemplar)
  └──────────────┘
```

**Veredicto del grafo:** acíclico, sin inversión (todo apunta DOWN al core). Cuatro grietas:
1. **proxy y MCP saltan la fachada `sdk`** — importan `StorageEngine`/`Executor`/`EntityStore` directos (`vanta-proxy/src/server.rs:15`, `auth.rs:12-14`; `vantadb-mcp/src/server.rs:20-21`). Dos crates de producto acoplados a internals del engine: cualquier cambio de `StorageEngine` rompe ambos.
2. **Feature-coupling**: `server = ["cli", ...]` arrastra clap/indicatif/anyhow al binario HTTP (`Cargo.toml:179-187`); el proxy compila el core SIN fjall (`default-features=false`) → un build aislado `-p vanta-proxy` produce un backend InMemory silencioso.
3. `default-members` excluye proxy/wasm (ADR-031) → CI desatendido por diseño.
4. `vantadb-node` fuera del workspace (razón válida MSVC) pero congela su versión en 0.5.0 y bloquea compartir código FFI.

### 2.2 Estilo arquitectónico: hexagonal pragmático con constitución escrita

Hallazgo poco común: existe una **constitución arquitectónica viva** (`docs/dev/architecture/BOUNDARIES.md`, 254 líneas) con owners por módulo, reglas BND-01..08, grafo de direcciones de dependencia, 8 violaciones "grandfathered" listadas por archivo:línea, y un gate `cargo modules --acyclic`. El ciclo `storage↔index` está detectado, tabulado y con plan de trait-split firmado.

Puertos reales verificados:
- `IndexPort` (`src/index_port.rs:99`) — rompe-ciclos con sealed trait documentado (pero interface ancha de 214 líneas = header-interface).
- `StorageBackend` (`src/backend.rs`) con ISP correcto: roles opcionales `Snapshotable`/`Compactable` separados, no-soporte declarado en tipos.
- `LlmRunner`/`AsyncLlmRunner` (`vanta-memory/src/core/abstractions/llm_runner.rs:91,184`) — host-neutral, el módulo hexagonal más limpio del repo.
- `ListRecordsPorts` (`src/server/list_records.rs:56-60`) — use-case object con port inyectado (doctrina cleanCA).

### 2.3 Capas y orquestación: UN corazón, 5 front-doors, pegamento faltante

CLI, TUI, HTTP, MCP y los 3 bindings FFI **convergen en `Embedded` (`src/sdk/`)** — no hay 5 motores de negocio duplicados. La duplicación no está en la lógica sino en **DTOs y config** (§2.6).

El pegamento que falta (todos verificados con grep de callers):
| Pieza huérfana | LOC | Consecuencia |
|---|---:|---|
| Scheduler vanta-memory (PipelineWorker/manager/timer_scanner) | ~1,700 | El pipeline automático L1→L3 no corre en NINGÚN binario shipped (`bootstrap.rs:332` → `conversation_trigger: None`) |
| Feature `llm-driver` en MCP/proxy/server | 1 línea de fix | `StandaloneLlmRunner` = `NotConfigured` SIEMPRE en los bins; la config `VANTADB_INGEST_PROVIDER=ollama/openai` es decorativa |
| Loop de memoria del proxy | ~3 días | capture escribe `proxy-turns` (write-only), search lee `l1/*`, injection lee persona/scene que el proxy nunca escribe → deployment proxy-only = memoria ilusoria; `record_response_usage` sin cablear → output_tokens siempre 0 |
| Dashboard HTTP | — | Mount point sin SPA en el repo |
| Perfil MCP | — | Solo filtra `tools/list`; cualquier tool "no listada" sigue siendo llamable (drift con `docs/api/MCP.md:254`) |

### 2.4 Inventario de patrones de diseño (22 verificados con archivo:línea)

| # | Patrón | Evidencia | Evaluación |
|---|---|---|---|
| 1 | Facade | `src/sdk/mod.rs:14-33` | ✅ Bueno |
| 2 | Builder | `src/sdk/builder.rs:71-117` | ✅ Bueno |
| 3 | Registry/factory OCAP | `src/backends/registry.rs:18-45` (fn-pointers, cero matches) | ✅ Bueno |
| 4 | Strategy (backend) | `src/backend.rs:105` + `dyn StorageBackend` | ✅ Bueno |
| 5 | Strategy (índice) | `src/index_port.rs:37-49`, `:127` search_with_method | ✅ Bueno |
| 6 | Strategy (cuantización) | `src/vector/quantization.rs` (SQ8/Turbo/Binary) | ✅ Bueno |
| 7 | Port/Adapter hexagonal | `index_port.rs:99`, `backend.rs`, `LlmRunner`, `ListRecordsPorts` | ✅ Bueno |
| 8 | Sealed trait | `index_port.rs:26-30` (justificado) | ✅ Bueno |
| 9 | Command (WAL) | `src/wal.rs:47` WalRecord enum | ✅ Bueno |
| 10 | State machine | `circuit_breaker.rs:20-29` lock-free; `session.rs:40-69` team→agent→task; lifecycle Embedded | ✅ Bueno |
| 11 | Pipeline/middleware | `vanta-proxy/src/server.rs:284+` — 16 etapas numeradas **inline**, sin punto de extensión (el comentario `:335` lo admite) | ⚠️ Aceptable |
| 12 | Object pool | `src/index/search/pool.rs:5-18` (thread-local NeighborVec) | ✅ Bueno |
| 13 | RAII guards | OpGate/OpGuard **replicado 3×** (node lib.rs:661-732, python lib.rs:88-106, wasm lib.rs:435-511) | ⚠️ Patrón bueno, abuso la triplicación sin crate compartido |
| 14 | Actor (worker+mailbox) | `vanta-memory/src/services/pipeline_worker.rs` (claim→TTL locks→retry→dead-letter→reclaim) | ✅ Bueno **pero huérfano** (sin host) |
| 15 | DI manual | `L1DedupConfig::with_local_provider`, `ServerListPorts`, `OllamaEmbedProvider::from_env` | ✅ Bueno |
| 16 | Use-case object | `server/list_records.rs:80+`, `conversation.rs` | ✅ Bueno |
| 17 | Delegación por macro | `forward_to_db!` ×4 (python lib.rs) | ⚠️ Fronterizo — sustituto de extracción |
| 18 | Sharing por `#[path]` | `providers/*/src/python.rs:9` → shared_py.rs (solo helpers; 70% sigue copy-paste) | ❌ Abuso |
| 19 | Newtype | `RequestId`, `LabelIntern`, SparseVector | ✅ Moderado |
| 20 | Interning | `LabelIntern` + DashMap | ✅ Bueno |
| 21 | Checkpoint/retry/dead-letter | `vanta-memory/src/utils/checkpoint.rs` | ✅ Bueno (huérfano) |
| 22 | OnceLock dispatch | `src/index/distance/kernels.rs` (SIMD runtime-detect) | ✅ Bueno |

**Ausencia notable:** typestate (0 usos de `PhantomData` en `src/`) — `LockPolicy::AssumeHeld` es disciplina enum, no typestate. Decisión razonable para un codebase de esta edad.

### 2.5 Concurrencia y async

- **Dominio 100% sync** (parking_lot, cero tokio en el core); tokio confinado a server/proxy/MCP; GIL/bloqueo manejado en la frontera (52 `py.detach`, `spawn_blocking`+OpGate en node). Composición correcta.
- **Techo de escritura concentrado y documentado**: `insert_lock` global FairMutex (`insert.rs:208,588`) + `supersede_lock` global SDK (`builder.rs:25`). ADR-037 existe pero no aplicado. (Es la causa de los 74 rec/s vs 1,016 prototipados con batching — 9.1×.)
- OpGate: mismo concepto (Mutex+Condvar, drain-on-close) adaptado a 3 runtimes — ya divergen en el caso wasm32 no-threads.

### 2.6 Arquitectura de errores, versionado y DTOs

**Errores (fuerte):** thiserror en todos los crates; anyhow confinado a bins por feature; **10 códigos canónicos `VANTADB_*` con match exhaustivo + snapshot tests** (`error.rs:337,1260-1278`); `is_retriable`/`recovery_hint` llegan estructurados a Python/MCP. Eslabón débil: a node/TS llegan como **string parseado** ("CODE: msg"), no estructurados.

**DTO drift (el defecto estructural #1):** DTOs redefinidos a mano en ≥7 fronteras — wasm `SearchRequest` (lib.rs:152), parse manual en node, `.pyi` en python, `types.ts` con casts `as unknown as`, `NodeDTO/QueryRequest` en state.rs, 87 schemas JSON en MCP tools.rs (~1,000 líneas), `wire_types.rs` en desktop. Causa raíz: **no existe codegen schema-first** (ts-rs/specta/typeshare) y el gate que validaría contratos (gate-docs) tiene el trigger YAML corrupto. Config triplicada (core ~70 campos / proxy ~40 / MCP) sin `deny_unknown_fields` en el proxy → typos de TOML ignorados en silencio.

### 2.7 Las 10 deudas arquitectónicas priorizadas

1. 🔴 Loop de memoria del proxy inerte (la funcionalidad bandera es ilusoria en deployment proxy-only) — `server.rs:241-251`
2. 🔴 Scheduler de vanta-memory sin host + llm-driver muerto en bins — `bootstrap.rs:332`
3. 🟠 Fachada SDK saltada por proxy y MCP (acoplamiento a internals)
4. 🟠 DTO drift estructural sin mecanismo de codegen ni gate
5. 🟠 Feature-coupling `server→cli` + unificación peligrosa de features + hack getrandom
6. 🟡 OpGate×3 sin crate compartido
7. 🟡 Config triplicada sin validación estricta
8. 🟡 Carga muerta institucionalizada (eviction.rs 311 LOC, bloom 159, io_budget write-only, engine.rs duplicado, 44 `allow(dead_code)`)
9. 🟡 Providers 70% copy-paste + colisión de nombres con integrations
10. 🟡 God-files en fronteras calientes (tools.rs 3,880 líneas; handlers.rs 1,552)

### 2.8 Las 5 refactorizaciones arquitectónicas recomendadas

| Refactor | Esfuerzo | Efecto |
|---|---|---|
| 1. Crate kernel FFI compartido (`vantadb-ffi-core`: OpGate + guards MAX_* + mapeo de errores) | 2-3 días | Elimina la triplicación; congela UNA semántica de drain |
| 2. Cerrar el loop de memoria del proxy (capture→`l1/`, cablear `record_response_usage`, presupuesto de tokens del bloque inyectado) | 1-2 días | Convierte la funcionalidad bandera de ilusoria a real |
| 3. Despertar o enterrar el scheduler de vanta-memory (ADR: un host productivo lo arranca, o recortar como experimental) | 2-4 días (a) / 1 día (b) | Mata la mayor brecha construcción-uso del repo (~40% del crate) |
| 4. Codegen schema-first de DTOs (typeshare/specta desde `sdk/types.rs` → TS/py/wasm/MCP) + gate de paridad en CI (arreglar trigger gate-docs) | 1-2 semanas | Ataca el DTO drift en su raíz |
| 5. Trait-split `storage↔index` según el plan ya firmado en BOUNDARIES §3 + desacoplar `server→cli` | 3-5 días | Ejecuta el plan propio en vez de dejarlo "madurar" |

---

## 3. MODELO DE DATOS (lo que VantaDB puede almacenar y cómo)

### 3.1 Esquema de 2 capas (verificado archivo:línea)

**Capa núcleo — `UnifiedNode`** (`src/node/unified.rs:15-47`):
- `id: u128` · `bitset: FilterBitset` (roaring, filtrado multi-tenant por categoría) · `semantic_cluster: u32`
- `flags: NodeFlags` — 10 bits: ACTIVE/INDEXED/DIRTY/TOMBSTONE/HAS_VECTOR/HAS_EDGES/PINNED/RECOVERED/**INVALIDATED**/**CONFLICT_RESOLVED** (+4 bits de tipo de vector) — los 2 últimos son los ganchos para conflictos que ya existen en el nivel más bajo
- `vector: VectorRepresentations` — {Binary RaBitQ / Turbo PolarQuant-4bit / SQ8 / Full / MmapFull / None}
- `edges: Vec<Edge>` + `label_index` · `relational: BTreeMap<String, FieldValue>`
- `tier: Hot|Cold` · `hits` · `last_accessed` (⚠️ no alimentado por search) · **`confidence_score` (def 0.5) · `importance` (def 0.1)** — ¡existen pero NO están expuestos ni consultables!
- `ext_metadata`

**Edge** (`src/node/edge.rs:9-23`): `{target u128, label_id u32 internado, weight f32, reverse bool, created_at_ms}` — tipada, ponderada, dirigida con semi-reverso automático; **SIN propiedades, SIN validez temporal [from,to]**.

**Capa memoria — vista sobre el núcleo**: `MemoryRecord` (`src/sdk/types/record.rs:79-114`) se materializa sobre **10 campos reservados `__vanta_*`** del mapa relacional: namespace+key → node_id determinista XxHash3_128, payload, metadata libre (11 variantes de `Value`, plano, sin objetos anidados), created/updated_at_ms, version, vector+sparse (`BTreeMap<u32,f32>`), **TTL `expires_at_ms`**, **`superseded_by/at_ms`**.

**Naturaleza del modelo:** schema-less (namespace = carpeta implícita con índices automáticos). Multi-tenancy: core = namespaces + bitset + EntityStore (`namespace/collection[user|team|agent|task|asset]/id`) con checker allow-only; vanta-memory añade `team_id/user_id/agent_id/task_id` en su record L1 **con default-tenancy silenciosa** (riesgo de sorpresa).

### 3.2 Capacidades de búsqueda y grafo (matriz verificada)

| Capacidad | Estado | Detalle |
|---|---|---|
| Filtros en list/delete | 🟡 | `FilterOp{Eq,Neq,Gt,Lt,Gte,Lte}` **AND-only plano**, sin OR/IN/anidados |
| Filtros en search | 🔴 | **igualdad-only** |
| Pre/post filtrado | ✅ | `CostEstimator` decide PreFilter (bitset+brute-force) vs PostFilter (HNSW+filtro) |
| Híbrido | ✅ | RRF tri-canal (BM25+denso+sparse), k=60 configurable, budget [32,256] |
| query_sparse | 🟡 | existe en core/HTTP pero **NO expuesto en ningún binding** (hardcodeado None) |
| Grafo | ✅ | BFS/DFS/topo/DAG/degree/**PageRank** (damping 0.85) + traversal por labels y ventana temporal del edge |
| GraphRAG | 🟡 | seed→BFS→rank 0.6/0.3/0.1→context_text; **SIN NER, SIN comunidades** |
| Cuantización | ✅ | Binary/Turbo/SQ8 + índices Hnsw/Ivf/Scann/Flat (search_with_method) |
| As-of / point-in-time | 🔴 | no soportado (approx per-key vía version_history) |

### 3.3 IQL (el "SQL" de VantaDB)

`FROM/MATCH` + `SIGUE min..max "label"` · WHERE **AND-only** (3 predicados) · `FETCH` · `RANK BY` · `WITH TEMPERATURE` · `ROLE` · `PROFILE` · DML INSERT/UPDATE/DELETE NODE#, RELATE, INSERT MESSAGE · **SELECT con JOIN y subqueries** (verificado `physical_plan/join.rs`) · **SIN agregaciones** (plan = scan/filter/project/sort/dedup/join/vector — sin nodo aggregate) · sin OR · sin transacciones multi-op declarativas. Expuesto en HTTP `/api/v2/query`, MCP `query_iql`, Python, TS, CLI — **no en el binding Node**.

### 3.4 Semántica temporal

**No bi-temporal** — un solo reloj (ingesta). TTL real end-to-end (lazy en lectura + `purge_expired`); `version_history` por put; supersession durable con `exclude_superseded`; vanta-memory añade trail de merges + **heat** (bump en lectura, decae por pase) + `mark_contradiction` — **pero sin job productivo que decaiga/pode**, y dream-promote es stub. ADR-028 rechazó explícitamente el decaimiento automático en core.

### 3.5 Veredicto del modelo de datos

> **La base (vector + metadata tipado + edges etiquetadas) es correcta y probada** — vanta-memory, escenas, skills, wiki, threads, entidades y axioms TODOS se mapean sobre ella sin hacks de storage. **Pero la semántica de "memoria" vive hoy por convención ENCIMA del modelo** (namespaces-carpetas, JSON dentro de payload, heat/supersession en campos del payload que el store no entiende): recall O(N) full-scan, sin queries que crucen capas, sin índice sobre el JSON del payload (shred write-only), sin procedencia consultable como grafo, sin bi-temporalidad, sin entity linking. Para ser memoria de primera clase, la semántica debe **bajar al store** (campos, edges, políticas).

---

## 4. CALIDAD DE CÓDIGO (síntesis consolidada)

**Fortalezas (medidas):**
- **7 `.unwrap()` / 1 `panic!` en ~99K LOC de producción** (1,767 unwraps confinados a tests); lints `unwrap_used/expect_used = deny` en `[workspace.lints]` heredados por TODOS los members — la disciplina no depende de revisión humana.
- **0 TODO/FIXME** — convención propia `ponytail:` (86+15 marcadores con decisión + condición de upgrade documentadas).
- **~2,780 tests** (core) + 563 (vanta-memory) + 229 (MCP) + 101 (proxy) + paridad openapi 37/44 exacta + stub-drift python + meta-test de 87 tools MCP.
- **51 `unsafe` concentrados en 3 módulos** con SAFETY comments (mmap con handler SIGBUS propio, kernels SIMD con chunks_exact).
- WAL CRC32C por registro + auto-healing + salvage con quarantine + chaos testing + failpoints + Miri. `cargo check -p vantadb --all-targets` → **0 errores, 0 warnings** (verificado localmente, 7m41s, toolchain 1.95.0 pineada).

**Debilidades (medidas):**
- **44 `#[allow(dead_code)]`** en 23 archivos; huérfanos confirmados: `eviction.rs` (311 LOC + feature `bayesian_decay` muerta), `duplicate_prevention.rs` (159 LOC bloom sin cablear), `io_budget` write-only, `list_window.rs` bench no registrado.
- Mojibake en 126 líneas de comentarios de 10+ archivos críticos (corrompe invariantes ACID documentadas).
- Providers ~70% copy-paste; 6 de 9 ejemplos de frameworks son el mismo esqueleto CRUD sin el framework real.
- El módulo cuya función es evitar info-stale es el más afectado por info-stale (skills 0.5.0, llms.txt, api-reference omite 6/87 tools, CHANGELOG duplicado, benchmarks §2 congelados) — el drift-gate existe pero su trigger de CI está roto.

---

## 5. PROBLEMAS DE PRODUCTO Y DE SOLUCIÓN

### 5.1 Problemas del PRODUCTO (concepto y go-to-market)

1. **El embudo de adquisición está roto en sus 3 primeros peldaños**: instalador macOS roto (`install.sh:119`), Discord invite inválido, `vantadb.dev` sin DNS, security@ muerto. "1 comando" es la promesa central y falla en el 20% de los laptops dev.
2. **Números públicos no reconciliados**: README p50 2.0ms vs BENCHMARKS.md §2 62ms (brecha 30×) — causa raíz confirmada: el script de benchmark Python se rompió con el rename 0.5.0 y §2 quedó congelado en la era pre-SIMD. En HN/Reddit los números son lo primero que se ataca.
3. **ICP sin decidir** (BIZ-08): SPEC dice "agentes de código"; GTM define 3 verticales. El mensaje "SQLite para agentes" compite contra Mem0/Zep/Letta por el mismo wallet, pero COMPARISON.md solo compara contra vector DBs.
4. **9 blogs draft + 3 posts Reddit listos sin publicar** — el marketing está escrito y no sale.
5. **Riesgo de marca "Vanta"**: Vanta Inc. (unicornio compliance ~$2.4B, marcas VANTA registradas clases 9/42) — riesgo asimétrico; el filing LEG-01 sin dictamen puede atraer la atención que hoy no existe. Consulta formal de marcas ANTES de invertir en marca.
6. **Bus factor 1 humano + 939 tasks/150 planes**: el proceso documental ya genera sus propios bugs (CHANGELOG duplicado, trigger corrupto, ADR duplicado) — el proceso consumirá al mantenedor antes que el mercado.

### 5.2 Problemas de la SOLUCIÓN (técnica)

1. **Ingesta 74 rec/s** (insert_lock global + HNSW serial) vs 598 QPS competitivos vs 114,583 de LanceDB — el fix está prototipado (batching 9.1×) y no productizado.
2. **Seguridad en un producto que vende privacidad**: export/import HTTP con path injection (`handlers.rs:676-741`), `/snapshot` del proxy sin auth + bind 0.0.0.0 — un CVE anula la narrativa.
3. **La funcionalidad de memoria autocontenida del proxy es ilusoria** (capture↔search disjuntos) — el "cost tracking" registra la mitad (output_tokens nunca).
4. **40% de vanta-memory sin host** — el pipeline automático que diferencia al producto no corre en ningún binario.
5. **Modelo de datos con semántica de memoria por convención** (§3.5) — recall O(N), sin procedencia como grafo, sin bi-temporalidad.

---

## 6. COMPETENCIA #1: SISTEMAS DE MEMORIA PARA AGENTES (17 sistemas analizados)

### 6.1 Los 8 principales

| Sistema | Idea central | Features distintivas | Qué copiaría VantaDB |
|---|---|---|---|
| **Mem0** (YC S24, ~50k★) | Pipeline extracción→consolidación; **v3 (2026): single-pass ADD-only** (memorias acumulan, no se sobreescriben) | Entity linking; retrieval multi-señal (semantic+BM25+entity); temporal reasoning; Mem0g (grafo); memoria procedural; OpenMemory MCP local con scoping por cliente; framework de eval open-source | Operaciones de consolidación explícitas como contrato; entity linking; scoping user/agent/run; eval framework |
| **Zep/Graphiti** | Grafo **bi-temporal** (valid time vs transaction time); contradicciones **invalidan**, nunca borran | Facts con ventana de validez; episodios como provenancia obligatoria; comunidades; ontología prescrita+aprendida; híbrido cosine+BM25+graph con RRF sobre edges/nodes/communities; point-in-time; sub-200ms | **Bitemporalidad como primitiva** (la mayor brecha de VantaDB); invalidación trazable; capa de episodios |
| **Letta/MemGPT** ($10M) | El agente se auto-gestiona la memoria: **blocks siempre en contexto** (sin retrieval) | core/recall/archival; self-editing tools; **sleep-time agents**; shared blocks multi-agente; Letta Filesystem (74.0 LoCoMo "solo archivos") | Bloques con presupuesto para el assembly del proxy; dry-run/diff de consolidación; export file-native |
| **LangMem/LangGraph** | Store con namespaces jerárquicos + taxonomía cognitiva | semantic/episodic/procedural; **hot-path vs background formation**; query por prefijo de namespace | Modos hot-path/background formalizados; namespaces jerárquicos |
| **Cognee** | ECL (Extract-Cognify-Load) → grafo+embeddings self-hosted | Corre sin LLM (GLiNER local); **forget como API de 1er nivel**; Memify; ontologías; **COGX: import de Mem0/Letta/Zep**; SDK Rust | **Formato de intercambio + importadores de rivales**; forget API; modo 100% local |
| **MemOS** | "OS de memoria": grafo inspeccionable + **cubes** componibles + MemScheduler | **Plugin local: SQLite+FTS5+vector, L1 traces→L2 policies→L3 world models→skills** (¡clon del pipeline VantaDB!); feedback/corrección en NL; **Memory Viewer**; OmniMemEval (14 productos) | Memory cubes (multi-tenancy); Viewer en desktop; corrección NL; presentación multi-benchmark |
| **MIRIX** | 6 memorias (Core/Episodic/Semantic/Procedural/Resource/**Knowledge Vault**) con agente gestor por cada una + Meta Manager | **auto_dream con `dry_run:true`** y modos por componente; screen tracking; skills destiladas de trazas de tools (errores→fixes) | **Dry-run de dreams** (ROI inmediato); Knowledge Vault; destilación procedural de tool-traces |
| **ReMe** (Alibaba) | **"Memory as File"**: Markdown+frontmatter+wikilinks, índices regenerables | Recall híbrido BM25+embeddings+**expansión wikilinks**; Memory Tags file-native; workspace multi-agente (SKILL.md/CLI/HTTP/MCP) | Export file-native git-friendly; wikilink expansion; tags de entidad con índice reconstruible |

**Otros 9 relevantes:** A-MEM (Zettelkasten: notas atómicas con evolución retroactiva — cuando llega un fact nuevo, las notas viejas relacionadas se enriquecen) · Memobase (perfil JSON tipado con schema del dev — contrato de memoria) · Supermemory (memoria+RAG+conectores en 1 API; 95% Recall@15, ~50ms) · MemMachine (episódica-grafo + perfil-SQL + working memory por sesión) · MemU (memoria-wiki, 500 LOC, skills Markdown auto-extraídas) · Second-Me (HMM L1/L2/L3 + red P2P local) · Honcho (peer-centric con teoría-de-la-mente multi-peer) · Hindsight (semantic+episodic+procedural+graph como redes paralelas de retrieval fusionadas) · OpenMemory MCP (scoping por cliente).

### 6.2 Patrones de UX de los productos masivos (ChatGPT / Claude / Cursor)

1. Memoria **visible, editable y borrable por item** (transparencia total — el usuario ve la lista literal).
2. **Opt-out fácil y granular** (Temporary Chat).
3. **Separación aprendido/impuesto** (Cursor: Memories vs Rules; Claude Code: auto-memory vs CLAUDE.md).
4. **Scoping claro** (proyecto/usuario/sesión).
5. **Filesystem-as-memory** = transparencia + auditabilidad + git-friendliness (Claude memory tool ejecuta en el cliente — la memoria vive en TU infra).
6. **Inyección confiable en el primer prompt** (no depender de retrieval).

### 6.3 Benchmarks del mercado (vendor-reported, harnesses incomparables)

| Sistema | LoCoMo | LongMemEval | Otros | Eficiencia |
|---|---|---|---|---|
| Mem0 (v3, plataforma) | **92.5** | **94.4** | BEAM 64.1@1M / 48.6@10M | ~7K tokens, p50 ~1s |
| Zep | 75.14 (su harness) / 58.44 (corregida por Mem0) | 71.2 (medido por Mem0) | — | sub-200ms |
| MemOS | 88.83 | 89.20 | BEAM-10M 56.75; HaluMem 80.91 | — |
| Letta Filesystem | 74.0 (solo archivos) | — | — | — |

⚠️ **Controversia estructural**: el judge de LoCoMo acepta hasta 63% de respuestas incorrectas y ~6.4% del answer-key está mal; Mem0 vs Zep mantienen una disputa pública de números. **LongMemEval** (5 habilidades incl. abstention) y **MemBench** (factual vs reflective) son las más serias; emergen BEAM (escala 1M/10M), OmniMemEval, HaluMem (alucinación), MemoryAgentBench. **Lección para VantaDB:** publicar números con harness propio reproducible (dataset commiteado, como ya hace su suite criterion) es diferenciador de credibilidad en un mercado de marketing inflado.

### 6.4 Convergencia del sector (2026) — hacia dónde va el mercado

1. **Bitemporalidad+invalidación** (Zep) vs **ADD-only acumulativo** (Mem0 v3): la consolidación destructiva (borrar/sobreescribir) cayó en desgracia — el historial completo con marcado de vigencia gana.
2. **File-native con índices regenerables** (ReMe/memU/Claude/Letta FS) gana tracción por transparencia.
3. **Consolidación en background tipo sueño estandarizada** (sleep-time/auto_dream/dreams) — todos la tienen, **NINGUNA con dry-run determinista salvo MIRIX**.
4. **MCP = bus de memoria de facto** (todos publican MCP servers; la memoria se vuelve "un servidor que se instala").
5. **Portable Agent Memory** (arXiv 2026) + COGX: primeros intentos de interoperabilidad — **no existe aún un estándar aceptado de serialización de memoria con semántica temporal/dedup/conflicto**.

---

## 7. COMPETENCIA #2: BDs MULTI-MODELO (10 sistemas × 17 features)

### 7.1 Tabla comparativa (● nativo · ◐ parcial · ○ no)

| Feature | Qdrant | LanceDB | Weaviate | Milvus | Chroma | SurrealDB | Neo4j/GDS | sqlite-vec | Turso | ArangoDB |
|---|---|---|---|---|---|---|---|---|---|---|
| Embebido | ◐ (Edge nuevo) | ● | ○ | ◐ (Lite) | ◐ | ● | ○ | ● | ● | ○ |
| Sparse nativo | ● | ◐ | ◐ | ● | ◐ | ◐ | ○ | ○ | ○ | ◐ |
| FTS / híbrido | ● / ● | ● / ● | ● BM25F / ● | ● / ●+rerank | ● / ● | ● / ● | ◐ | ○ | ◐ | ● / ◐ |
| Filtros payload | ● range/geo/nested | ● SQL | ● | ● | ● $and/$or | ● | ● | ◐ | ● | ● AQL |
| Grafo nativo | ○ | ○ | ● x-refs | ○ | ○ | ● RELATE | ● Cypher+GDS | ○ | ○ | ● |
| Multi-tenancy | ◐ shard_key | ○ | ● tenants+offload | ● partition keys | ◐ IAM | ◐ perms fila | ◐ | ◐ | ◐ | ◐ |
| Time-travel | ○ | **● checkout** | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ○ |
| CDC/live | ○ | ○ | ◐ | ◐ | ○ | **● LIVE SELECT** | ◐ | ○ | ○ | ◐ |
| Quantización | ● SQ/PQ/BQ | ● PQ | ● PQ/BQ | ● | ○ | ○ | ◐ | ◐ | ◐ | ○ |
| Tx multi-op | ○ | ○ | ○ | ○ | ○ | **● ACID** | ● | ◐ | ● | ● |

### 7.2 Taxonomía transversal — estado REAL de VantaDB (verificado en código)

| Feature | Estado | Evidencia / referencia del líder |
|---|---|---|
| TTL/retención | 🟡 parcial | core+MCP sí, **server HTTP no**; sin default por colección (Qdrant/Weaviate/Milvus tampoco lo tienen — oportunidad de diferenciar) |
| Time-travel | 🟡 parcial | version_history+supersede; sin checkout "AS OF" (LanceDB) |
| Multi-tenancy | 🟡 parcial | namespaces; sin cuotas ni estados HOT/COLD (Weaviate/Milvus) |
| RBAC fino por colección | 🔴 | roles hardcodeados (SurrealDB row-level, Milvus privilegios) |
| Replicación | 🔴 | ninguna (Qdrant/Turso/Milvus) |
| CDC/change streams | 🔴 | no expuesto — **el WAL ya es el log** (SurrealDB LIVE SELECT) |
| Agregaciones | 🔴 | sin nodo aggregate en el physical plan |
| Rerankers pluggables | 🟡 | RRF nativo; sin weighted ni BM25F ni cross-encoder (Milvus/Weaviate) |
| Server-side embeddings | 🟡 | e5 ONNX en vanta-memory, no en server (Chroma default, Weaviate modules) |
| Quantización configurable | 🟡 | módulo existe; sin API por colección ni rescore asymmetric (Qdrant) |
| Sparse consultable | 🟡 | guardado; no consultable vía bindings (Qdrant/Milvus) |
| Range/grouping search | 🔴 | sin radius ni group_by (Milvus) |
| Search iterator | 🟡 | cursor en list; no en search (Milvus) |
| Transacciones multi-op | 🔴 | un solo write-lock lo facilita (SurrealDB ACID) |
| Sharding/replicación | 🔴 | solo WAL sharded |

### 7.3 Las 10 lecciones de arquitectura aplicables a un embebido Rust

1. **Qdrant**: toda mutación es un `UpdateOperation` serializable → UNA primitiva da replicación + edge-sync + CDC. El WAL de VantaDB ya es eso — **hay que exponerlo**. Y su salto a embebido (Edge) es competidor directo: la ventana se cierra.
2. **LanceDB**: versionado barato nace del FORMATO (segmentos inmutables + deletion vectors) — el patrón exacto para atacar `insert_lock` (74 rec/s): append de segmentos + compactación background en vez de mutación bajo lock global.
3. **Weaviate**: patrón "modules" para embeddings/rerank/generative server-side sin inflar el core; tenants-shard con offload HOT/COLD = multi-tenancy real.
4. **Milvus**: funnel un-API Lite→Standalone→Distributed es EL playbook de distribución 2025.
5. **Chroma**: DX gana — 4 verbos, embed-on-write por defecto. Aplicar al perfil MCP "agent" (~10-45 tools) en vez de 87 por defecto.
6. **SurrealDB**: aristas-como-registros (RELATE, con propiedades/TTL/permisos) y CDC-como-query (LIVE SELECT) — el modelo correcto para GraphRAG temporal.
7. **Graphiti/Neo4j GDS**: grafo sin algoritmos (WCC, shortest-path) ni temporalidad (t_valid/t_invalid) es decorativo.
8. **Postgres**: los índices se copian; **el foso es superficie DSL estable + transacciones + operador simple + ecosistema** — estabilizar IQL (agregaciones, transacciones) vale más que otro ANN. Filtered DiskANN (pgvectorscale) se copia directo porque VantaDB ya tiene DiskANN.
9. **sqlite-vec/Turso**: "runs anywhere" + **embedded replica** (lectura local, escritura remota) = distribución local-first ganadora.
10. **Kùzu (contrapicado)**: embebido puro sin server/cloud murió con su compañía — ejecutar el funnel embebido→server→managed es urgente.

---

## 8. QUÉ COPIAR: LISTA MAESTRA CONSOLIDADA (deduplicada de §6 y §7, priorizada)

### Tier A — Victorias rápidas (impacto alto, esfuerzo bajo; primer trimestre)

| # | Feature | Fuente | Esfuerzo | Dónde cae |
|---|---|---|---|---|
| A1 | **Exponer `query_sparse` en los 3 bindings** (hoy guardan sparse_vector pero hardcodean None en query) | Qdrant/Milvus | Muy bajo | bindings py/node/wasm |
| A2 | **Dry-run + diff report de dreams** | MIRIX `auto_dream` | Bajo | vanta-memory + MCP |
| A3 | **TTL superficie completa**: param en server HTTP + default por colección + sweeper al índice | hueco del mercado | Bajo | server + core sweeper existente |
| A4 | **Range search (radius/range_filter) + group_by** | Milvus | Bajo | `sdk/search/vector.rs` |
| A5 | **Search iterator/cursor con resume** | Milvus | Bajo | extender cursor existente de list |
| A6 | **Export file-native Markdown** con índices regenerables (git-friendly) | ReMe/Claude/Letta FS | Bajo-Medio | vanta-memory + desktop (rebuild_index ya existe) |
| A7 | **MCP "persona de memoria" con scoping por cliente + viewer local** | OpenMemory | Bajo | MCP 87 tools + desktop Tauri |
| A8 | **Importadores de memoria rival** (Mem0/Zep/Letta → VantaDB, formato COGX-like) | Cognee | Bajo-Medio | SDK/CLI/docs — captura usuarios de la competencia |
| A9 | **Skills desde trazas de tools** (aceptar role=tool, destilar errores/retries como señal) | MIRIX/memU | Bajo | skills pipeline |
| A10 | **Cerrar el loop proxy capture→search + cablear cost real + presupuesto del bloque inyectado** | (hallazgo interno) | 1-2 días | vanta-proxy |

### Tier B — Diferenciación (esfuerzo medio; segundo trimestre)

| # | Feature | Fuente | Dónde cae |
|---|---|---|---|
| B1 | **Governance de inyección**: presupuesto de contexto por request + audit log de qué memoria se inyectó en qué prompt con qué score | hueco exclusivo (solo VantaDB tiene proxy+memoria) | vanta-proxy |
| B2 | **Redacción-on-write persistida + namespaces cifrados** (la PII redactada como fuente de verdad; original solo en envelope cifrado) | hueco (el proxy ya redacta en vivo) | vanta-proxy + core |
| B3 | **Bitemporalidad**: `valid_at_ms`/`invalid_at_ms` en edges y records + point-in-time queries | Zep/Graphiti | core grafo+índices |
| B4 | **Consolidación como operaciones explícitas ADD/UPDATE/DELETE/NOOP con política elegible** | Mem0 | pipeline L1→L2 |
| B5 | **Entity linking nativo + boost multi-señal** sobre el RRF existente (determinista, sin LLM — ms y coste 0) | Mem0 (lo hace con LLM) | core |
| B6 | **Memory cubes**: aislamiento + sharing controlado multi-agente sobre namespaces+ACLs | MemOS/Letta | vanta-memory + core |
| B7 | **Bloques de memoria con presupuesto siempre-en-contexto** (Letta-style, para el assembly del proxy) | Letta | vanta-memory + proxy |
| B8 | **Quantización configurable por colección** (SQ/BQ/PQ + rescore asymmetric) | Qdrant | módulo existente + API |
| B9 | **Filtros avanzados**: range/datetime/nested + $and/$or compuestos | Qdrant/Chroma | scalar_index + IQL parser |
| B10 | **CDC/LIVE queries por SSE desde el WAL** | SurrealDB | WAL ya es log de cambios |
| B11 | **Server-side embeddings como módulo por colección** (e5 ONNX subido al server) | Chroma/Weaviate | server + manifest de 9 modelos existente |
| B12 | **Forget API de primer nivel + certificado de purga** | Cognee/ChatGPT | shred/GC (cablear delete-path) + MCP + desktop |

### Tier C — Arquitectura pesada (3-4 trimestres; foso de largo plazo)

| # | Feature | Fuente | Dónde cae |
|---|---|---|---|
| C1 | **Replicación por shipping de UpdateOperations + embedded replica + PITR** | Qdrant Edge/Turso/Milvus | el WAL ya es el log; server follower + restore-to-ts |
| C2 | **Multi-tenancy robusta**: partition-key hashing + estados HOT/COLD + cuotas | Weaviate/Milvus | NamespaceIndex → tenants |
| C3 | **RBAC por namespace/colección** con privilegios por acción | Milvus/SurrealDB | server RBAC + namespaces |
| C4 | **Time-travel "AS OF"** + deletion vectors formales | LanceDB | version_history + shred bitsets |
| C5 | **Edges con payload/TTL/validez temporal + WCC/shortest-path** | Graphiti/Neo4j GDS | core graph + graphrag |
| C6 | **Transacciones multi-op declarativas** BEGIN/COMMIT | SurrealDB | el write-lock único lo facilita |
| C7 | **IQL: agregaciones + transacciones + strict-mode** (estabilizar como el "SQL" de VantaDB) | Postgres/ArangoDB | physical_plan |
| C8 | **Batching productizado** (9.1× prototipado) con segmentos appendables | LanceDB | core storage |
| C9 | **Filtered DiskANN** (labels/filtros durante el traversal) | pgvectorscale | DiskANN ya existe |
| C10 | **Codegen schema-first de DTOs** + gates de contratos en CI | (deuda interna #1) | ts-rs/specta |

**Sequencing sugerido:** Q1 = A1-A10 · Q2 = B1-B6, B12 · Q3 = B7-B11, C8 · Q4 = C1-C7, C9, C10.

---

## 9. INNOVAR DESDE CERO: 12 huecos donde VantaDB puede ser PRIMERO del mundo

Estado del arte verificado: TODOS los sistemas existentes confían en un LLM para extraer/consolidar (no determinista); NINGUNO ofrece verificación criptográfica, borrado certificado, esquemas versionados con migraciones, sync federado, ni governance de inyección. VantaDB tiene las piezas (WAL durable, shred, proxy con redacción, e5 local, MCP 87 tools, desktop Tauri, grafo+PageRank, skills/dreams L0→L3, WASM) que nadie más combina:

1. **Memoria tamper-evident** 🔒 — hash-chain sobre el log de operaciones de memoria (add/update/invalidate/forget) firmada por el motor + verificación CLI. El WAL durable ya existe → añadir encadenamiento. *Nadie lo tiene.* (Investigación a hacer: cómo extender CRC32C por-registro a hash-chain incremental sin coste de escritura; referencia técnica: certificate transparency, Notary/Trillian.)
2. **Borrado certificado (certified forgetting)** 🗑️ — retención como política declarada + TTL + **attestation de purga** (certificado con timestamp de que el dato salió de TODOS los índices/respaldos/WAL). El shred existe; hay que cablear el delete-path y formalizar el certificado. *Nadie lo tiene.*
3. **Redacción persistida + namespaces cifrados** 🎭 — la versión redactada como fuente de verdad de memoria; el original solo en envelope cifrado por namespace (keys por usuario/proyecto). El proxy ya redacta en vivo — falta persistir. *Nadie lo tiene.*
4. **Memoria federada local-first con merge determinista** 🌐 — sync entre dispositivos/agentes vía event-log con merge LWW/authority (CRDT-lite sobre UpdateOperations). ReMe/memU comparten workspace sin sync; Second-Me hace P2P a nivel app sin motor. Desktop+WASM+embedded = posición única. *Nadie lo tiene.*
5. **Esquemas de memoria versionados con migraciones** 📐 — registros tipados validados en el motor (no en un LLM), con migraciones declarativas. (Referencia: perfil tipado de Memobase + ontología Pydantic de Graphiti, pero en el store.) *Nadie lo tiene bien.*
6. **Dedup determinista** (minhash/simhash/LSH con IDs estables y umbral auditable) vs el dedup-LLM no determinista de todos; el LLM reservado para conflictos semánticos verdaderos. *Nadie lo garantiza.*
7. **Conflicto con taxonomía + trazabilidad + política elegible** — bi-temporalidad + clasificación (contradicción/actualización/redundancia) + política (last-writer/authority/source-priority) + registro quién-cuándo-por-qué. Los flags INVALIDATED/CONFLICT_RESOLVED ya existen en NodeFlags — es el ganchillo natural. *Nadie lo tiene completo.*
8. **Planes como objetos de primera clase con estados** 📋 — planes/objetivos/tareas con lifecycle (active/blocked/done/expired), validez temporal, queries "¿qué quedó pendiente?" y edges depends_on. Los agentes hoy re-derivan planes cada sesión; Mem0 memoriza facts, no planes con ciclo de vida. *Nadie lo tiene.*
9. **Governance de inyección en proxy** 🛡️ — presupuesto de contexto por request, ACLs por tool/namespace, audit log de memoria inyectada. Solo VantaDB está en la posición proxy+memoria. *Nadie lo tiene.*
10. **Memoria WASM/browser offline-first sincronizable** 🧩 — el mismo motor en el navegador (bindings ya existen) con sync al desktop vía federación (#4). ChatGPT es cloud-only; todos los demás son server/lib. *Nadie lo tiene.*
11. **Entity linking + razonamiento temporal NATIVO sin LLM** ⚡ — determinista en Rust sobre grafo+índice temporal, latencia ms, coste 0 por query (Mem0 lo hace con LLM+índices). *Nadie lo tiene nativo.*
12. **Eval de memoria local y reproducible** 📊 — harness offline con dataset commiteado (como la suite criterion) midiendo precisión por tipo de query (temporal/abstención/actualización/abstención) + contratos de p99 en CI. En un mercado de números incomparables y LoCoMo quemado, esto es credibilidad como moat. *Nadie lo tiene.*

**Prioridad estratégica:** 1, 2, 3 y 9 son las combinaciones que SOLO VantaDB puede ofrecer (proxy+memoria+WAL+shred juntas) → **la posición de marca "la memoria verificable y gobernable para agentes"**. 4 y 10 definen la tesis local-first/desktop. 6, 7, 11 son mejoras de motor de alto impacto. 5 y 8 son producto. 12 es marketing de credibilidad.

**Idea de innovación original adicional (propuesta propia, cruzando §3 con §6):** *Memory Contracts* — el modelo de datos actual es schema-less; añadir **contratos por namespace** (schema versionado + política de retención + política de conflicto + política de tenancy declaradas EN la metadata del namespace) convierte cada namespace en un "cubo de memoria gobernado" con garantías ejecutables por el motor, no por convención del llamador. Combina los huecos 5, 7, 8 y las Memory cubes de MemOS en una primitiva única que nadie tiene.

---

## 10. MEMORIA POR DOMINIO: cómo el modelo de datos influye en agente / empresa / producto / tarea / plan / investigación

Para cada dominio: qué es expresible HOY con el esquema actual (§3) y qué feature nueva lo desbloquea.

### 10.1 Memoria de un AGENTE (identidad, persona, aprendizajes)

- **Hoy:** `EntityStore` (collection=agent) + persona/<session> persona.md + `MemoryType::Persona` (7 tipos de memoria) + skills versionadas (SkillRecord + skill_versions con content-hash idempotente) + context engine (mild/aggressive/emergency) + recall keyword/embedding/hybrid con degradación honesta.
- **Lo que falta:** edge `authored_by` y campo `author_id` en MemoryInput (procedencia de quién aprendió qué); bloques always-in-context con presupuesto (Letta-style) para el proxy; destilación de skills desde tool-traces (A9); export file-native para que el agente pueda leer su propia memoria con herramientas normales (A6).
- **Influencia positiva:** con A6+A9+B7, un agente en VantaDB tiene los 3 niveles que el sector validó: **impuesto** (axioms/_axioms ya existe, Iron Axioms), **declarativo** (persona file-native), **aprendido** (L1→L3 automático) — separación aprendido/impuesto que es el patrón UX #3 de los productos masivos.

### 10.2 Memoria de una EMPRESA (conocimiento organizacional)

- **Hoy:** namespace por dominio + metadata {team_id, kind} + payload-index escalar automático + wiki (WikiPage) + GraphRAG (seed→BFS→rank) + hiybrid BM25+HNSW en español/inglés con e5 local.
- **Lo que falta:** jerarquía de namespaces con permisos heredados por colección (C3 — Couchbase bucket→scope→collection como modelo); entity linking entre memorias de empleados/proyectos/clientes (B5); knowledge vault separado con permisos propios (MIRIX) cruzado con redacción persistida (B2); MEMORY CUBES para compartir selectivamente entre equipos (B6).
- **Influencia positiva:** la combinación B2+B5+B6+C3 convierte VantaDB en la primera memoria empresarial donde **el compliance es una propiedad del motor** (borrado certificado + audit WORM + PII redactada), no una capa contractual encima — exactamente lo que un comprador enterprise (y el conflicto con Vanta Inc. de compliance) sugiere como buyer persona.

### 10.3 Memoria de un PRODUCTO (specs, decisiones, estado)

- **Hoy:** registro con metadata {product, version, status} + `supersede()` + version_history + `exclude_superseded` en search — el ciclo de vida de un spec ya es consultable.
- **Lo que falta:** edge `spec_of` y `schema_version` de documento (procedencia); edges de decisión (`decided_by`, `supersedes`) para el trail de decisiones de producto; esquemas versionados por namespace (hueco #5) para validar que un spec cumple su contrato.
- **Influencia positiva:** con bitemporalidad (B3), la pregunta "¿qué decía el spec en la fecha del incidente?" se responde con point-in-time query — nada en el mercado lo hace para specs.

### 10.4 Memoria de una TAREA (contexto de trabajo)

- **Hoy:** WorkTask + task_id + filter_ops + MessageThread con TTL (GcWorker) + tenancy team→agent→task monotónica del proxy — el scoping jerárquico ya existe.
- **Lo que falta:** edges `next`/`depends_on` entre nodos-paso; status enum indexado con transiciones; TTL por paso; working memory por sesión como tier explícito (MemMachine); retomar-tarea como query ("¿qué había decidido sobre X la última sesión de esta tarea?") que hoy es O(N) full-scan.
- **Influencia positiva:** el modo "hot-path" de LangMem + working memory explícita evitan el Lost-in-the-Middle que el propio SPEC de VantaDB describe como dolor fundacional.

### 10.5 Memoria de un PLAN (objetivos con ciclo de vida)

- **Hoy:** nada específico — un plan es solo texto en un payload. Es el gap más grande de los 6 dominios.
- **Lo que falta (hueco de innovación #8):** planes como objetos de primera clase: `{goal, steps[], status, valid_from/to, depends_on, blocked_by, progress}` con estados active/blocked/done/expired, edges depends_on con grafo (topological sort ya existe en gds.rs), queries nativas "¿qué planes están activos?", "¿qué quedó pendiente?", "¿qué planes bloquea este cambio?".
- **Influencia positiva:** ningún competidor memoriza planes con lifecycle — el agente que retoma proyectos de semanas es el caso de uso más doloroso y más vendible; cruza perfecto con el sistema de tasks/planes que el propio VantaDB usa para desarrollarse (939 tasks) — dogfooding inmediato.

### 10.6 Memoria de una INVESTIGACIÓN (corpus, fuentes, hallazgos)

- **Hoy:** namespace research/<topic> + metadata {type: source|finding, url, confidence} + add_edge manual `cites` + GraphRAG expande por hops + confianze/importance en el nodo (no expuestos).
- **Lo que falta:** índice único de URL (dedup determinista de fuentes — hueco #6); edge `cites` con span (qué parte de la fuente sostiene el hallazgo); detección de contradicción entre fuentes consultable (los flags CONFLICT_RESOLVED/INVALIDATED existen en el nivel bajo — subirlos a la API); exposición de confidence/importance con boost en la fusión RRF (top-8 del modelo de datos); trazabilidad del modelo de embedding (el registro no guarda modelo/dim — cambia el dataset → las búsquedas mezclan espacios).
- **Influencia positiva:** una investigación es el caso donde **procedencia + bi-temporalidad + contradicciones** valen más que recall bruto — y son exactamente los 3 huecos del modelo (§3.5). El usuario de este informe es el ejemplo vivo: investigación propia (Notion) + corpus web + hallazgos cruzados que necesitan consultarse con procedencia.

### 10.7 Síntesis: las 8 features del modelo que desbloquean los 6 dominios

1. **Campos first-class de tenancy+procedencia** en MemoryInput (`__vanta_owner/team/agent/task/source`) → desbloquea agente/empresa/investigación.
2. **Exponer confidence/importance + boost en la fusión RRF** → investigación/empresa (ya existen en UnifiedNode, solo falta exponer).
3. **Edge.properties + validez temporal** (t_valid/t_invalid) → producto/investigación/plan.
4. **`valid_at_ms` + as-of search** (bitemporalidad) → producto/investigación.
5. **Edges `mentions`/`derived_from`/`depends_on`** apoyadas en SceneNodeStore (MEM-12) que ya existe → plan/investigación.
6. **Políticas de retención por namespace + wiring del timer_scanner** (hueco #5/#8) → empresa/plan.
7. **Audit WORM encadenado** (hueco #1) → empresa (compliance como feature).
8. **Trazabilidad del modelo de embedding** (modelo/dim por registro) → todo (integridad de retrieval).

---

## 11. ROADMAP CONSOLIDADO (fusión de 3 fuentes: fixes previos P0-P2 + features copiables + innovación)

### Fase 0 — Confianza (previa a todo; ~2-4 semanas, casi todo mecánico)
Los 15 P0 del informe anterior siguen vigentes: sandbox export/import/snapshots, auth /snapshot + refuse-to-start, install.sh macOS, trigger gate-docs, reparar CHANGELOG, portar bench Python (reconciliar números 30×), notebook Colab, forward llm-driver (1 línea), hooks sin pwsh, ci-rust en develop, versiones sincronizadas, tsc+tests en CI desktop, CSP, api-reference regenerada, BND-07 Discord/DNS.

### Fase 1 — Memoria real (Q1: A1-A10 + refactors 2.8)
**Tema: "que lo prometido funcione y sea medible."** Cerrar el loop del proxy (A10), llm-driver (P0), scheduler con host o enterrado (refactor 3), query_sparse (A1), dry-run dreams (A2), TTL completo (A3), export file-native (A6), importer Mem0/Zep (A8), codegen DTOs (C10 empieza), batching productizado (C8 — 9.1× ya prototipado).
**Hito medible:** un agente en Cursor/Claude Code con memoria que sobrevive sesiones Y se puede inspeccionar en el desktop.

### Fase 2 — Memoria de primera clase (Q2: B1-B6, B12 + top-8 modelo)
**Tema: "la semántica baja al store."** Tenancy+procedencia first-class, confidence/importance expuestos, bitemporalidad, consolidación explícita, entity linking nativo, memory cubes, governance de inyección, redacción persistida + cifrado, forget certificado.
**Hito medible:** LoCoMo/LongMemEval con harness propio publicado (hueco #12) + 3 casos de dominio funcionando (agente/empresa/investigación).

### Fase 3 — Foso de plataforma (Q3-Q4: B7-B11, C1-C7, C9 + innovación 1, 4, 10)
**Tema: "por qué no te vas."** Replicación/embedded replica/PITR (C1), multi-tenancy HOT/COLD (C2), RBAC (C3), time-travel AS OF (C4), transacciones + IQL agregaciones (C6/C7), tamper-evident (innovación #1), federación (innovación #4), WASM sync (innovación #10).
**Hito medible:** funnel embebido→server→managed ejecutándose (lección Kùzu) + "memoria verificable" como categoría propia.

### North Star sugerida
Agentes activos que recuperan una memoria con éxito en ventana de 7 días (medible en el proxy: sesiones MCP con put+search la misma semana). Guardrails: 0 hallazgos high sin parche ≤7 días; 0 regresión p99 >15%; 100% artefactos con versión sincronizada; 0 violaciones Regla 11 en material público.

---

## 12. CRUCE CON LOS HALLAZGOS PREVIOS (dónde corrobora y dónde corrige)

**Corroborado y profundizado:**
- Ingesta 74 rec/s: ahora con causa arquitectónica exacta (insert_lock FairMutex + supersede_lock + techo serial) y solución con patrón de líder (LanceDB segmentos inmutables).
- Loop de memoria del proxy inerte: ahora clasificado como deuda arquitectónica #1 con 3 mecanismos desconectados identificados por archivo:línea.
- Scheduler hostless de vanta-memory: confirmado como brecha construcción-uso #1 (~40% del crate) con ADR de despertar/enterrar recomendado.
- DTO drift: elevado de "hallazgos FIND-* sueltos" a deuda estructural con causa raíz (sin codegen) y solución (C10).

**Corregido/refinado por la investigación web:**
- TTL: no era "ausente" — es parcial (SDK/MCP sí, server no). La ausencia en server es una brecha de superficie fácil de cerrar.
- IQL: SÍ tiene JOIN (verificado physical_plan/join.rs); lo que no tiene son agregaciones ni OR ni transacciones.
- "Competencia solo vector DBs": la investigación web confirma que los rivales de wallet reales son Mem0/Zep/Letta (memoria-as-a-service) Y que Qdrant Edge/Chroma/Milvus Lite están bajando a embebido — VantaDB está siendo flanqueado por ambos lados.

**Nuevas señales no visibles en el análisis interno:**
- MemOS local plugin es un clon conceptual del pipeline L0→L3 de VantaDB (SQLite+FTS5+vector, L1→L3, skills) — validación de la tesis Y prueba de que la ejecución debe acelerar.
- LoCoMo está quemado como benchmark (judge acepta 63% errores) — no competir en su tabla; publicar harness propio.
- El patrón "modules" (Weaviate) y "memory cubes" (MemOS) son los dos diseños que más encajan con la arquitectura existente de VantaDB para crecer sin inflar el core.

---

## 13. PENDIENTE: INTEGRACIÓN CON TU INVESTIGACIÓN DE NOTION

Cuando me entregues el contenido (pegado en el chat o vía URL pública), integraré como capa adicional:
1. **Visión original vs implementación** — cuánto del concepto fundacional quedó en el código, qué se desvió, qué quedó en backlog interno.
2. **Cruce de prioridades** — tus prioridades de investigación vs las que emergen de este análisis (arquitectura + mercado).
3. **Terminología y modelo mental** — si tu Notion define conceptos (ej. capas de memoria, agentes objetivo, verticales) que difieren de los implementados (L0-L3, EntityStore), mapearé uno a uno.
4. **Features que tu investigación preveía y que este análisis ahora valida/refuta** con evidencia de mercado (§6-§9).

**Vías que funcionan para entregármelo (probadas):** (a) pegar el texto directamente en el mensaje del chat (el texto SIEMPRE llega); (b) subir el .md a un gist de GitHub o cualquier URL pública y darme el enlace (descargo con curl); (c) reintentar el adjunto uno por uno mientras ejecuto el monitor `zsh /home/z/my-project/upload-watch.sh` — si algún intento aterriza, lo detecto en segundos.

---

## 14. ARTEFACTOS GENERADOS (rutas)

| Artefacto | Contenido |
|---|---|
| `VantaDB-Analisis-Arquitectura-Producto-Competencia.md` | **Este informe** — arquitectura, modelo de datos, competencia, copiables, innovación, dominios de memoria, roadmap |
| `VantaDB-Informe-Analisis-Completo.md` | Informe módulo a módulo (2 sesiones previas): 12 críticos verificados, LOC, veredictos por módulo, producto 4/10 |
| `worklog.md` | Consolidado completo de 15 tareas de análisis (Tasks 1, 2-a..2-e, 4-a..4-k, 5, 6) con evidencia archivo:línea |
| `tool-results/web_agent_memory.md` | Reporte web completo: 17 sistemas de memoria, benchmarks, estándares, 30+ fuentes URL |
| `tool-results/web_multimodel_db.md` | Reporte web completo: 10 BDs multi-modelo, taxonomía ×17 features verificada, 10 lecciones, fuentes |

---

*Informe generado por 6 agentes de análisis/investigación en paralelo (arquitectura, modelo de datos, 2 investigaciónes web con 48 queries + 21 READMEs oficiales) cruzados contra el worklog consolidado de 2 sesiones previas (12 análisis especializados + verificación manual de cada hallazgo crítico + compilación local verificada). Todos los números del repo tienen archivo:línea; todos los números del mercado son vendor-reported con caveat de harness.*
