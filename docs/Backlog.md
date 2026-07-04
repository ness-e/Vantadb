---
title: "Active Backlog — VantaDB"
type: backlog-tracking
status: active
tags: [vantadb, backlog, engineering, phases, priorities]
links: "[[master-index]]"
last_reviewed: 2026-07-04
aliases: []
---

# Active Backlog — VantaDB

> **Purpose:** Single source of truth for all project tasks, active and postponed.
> **Completed features:** `docs/CHANGELOG.md`
> **Total items:** 154 (62 original + 91 code review + 1 governance redesign)

---

## TIER 0 — 🔴 Bloqueantes de Release (Semana 1, Jul 4-11)

> Items que bloquean cualquier release seguro o publicación pública.

### 🩹 Data Loss & Crash Prevention

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `CODE-001` | **WAL replay no escribe backend metadata** — `recover_state()` reaplica Insert/Update en vstore+HNSW pero NUNCA en StorageBackend. Tras crash, nodos en HNSW pero `get()` retorna nada | `engine.rs:395-398` | 🔴 2-3d | 🔴 | ❌ |
| `CODE-002` | **WAL append antes de validación** — `insert()` escribe WAL (L132) ANTES de check duplicado (L135). Si falla, WAL tiene registro fantasma. Mismo bug en `update()` (L154→L159) y `delete()` (L168→L170) | `engine.rs:132-173` | 🔴 2-3d | 🔴 | ❌ |
| `CODE-003` | **6 puntos de `process::exit(1)` sin flush WAL** — Salta todos los Drop. BufWriter pierde records buffered. File lock `vanta.lock` nunca se libera | `cli_server.rs:595-767` | 🟡 1-2d | 🔴 | ❌ |
| `CODE-009` | **`save_vector_index()` traga errores de persistencia** — Retorna `()`, no `Result`. `persist_to_file()` falla → solo warn log. Caller cree que salvó OK | `engine.rs:1374` | 🟡 1d | 🔴 | ❌ |
| `CODE-026` | **BFS order vacío destruye DB en compact** — Si `bfs_order` está vacío, compact reemplaza DB real con archivo vacío de 64 bytes | `archive.rs:15-107` | 🟡 1d | 🔴 | ❌ |
| `CODE-027` | **`.expect()` panic en `get_many()` con backend corrupto** — Crash producido en lugar de error. Mata el server completo | `engine.rs:1090-1093` | 🟢 2-4h | 🔴 | ❌ |

### 🛡️ Seguridad & Data Integrity

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `CODE-020` | **CSP permite `unsafe-eval` + `unsafe-inline`** — Anula toda protección XSS. GSAP necesita `unsafe-inline` pero `unsafe-eval` probablemente no | `vercel.json:18` | 🟢 1-2h | 🔴 | ❌ |
| `CODE-021` | **`dangerouslySetInnerHTML` en blog sin DOMPurify** — XSS si atacante escribe blog post. `marked()` permite raw HTML por defecto | `$slug.lazy.tsx:82` | 🟢 2h | 🔴 | ❌ |
| `CODE-012` | **Path traversal en Python SDK export/import/constructor** — `../../etc/passwd` pasa sin validación | `lib.rs:676,974,988,1000` | 🟡 1d | 🔴 | ❌ |
| `SEC-08` | Migrar `rustls-pemfile` → `rustls-pki-types` (RUSTSEC activa) | — | 🟢 2-4h | 🔴 | ✅ |
| `SEC-09` | Eliminar `bincode` de archive + actualizar docs | — | 🟢 2h | 🔴 | ✅ |
| `SEC-10` | Security test suite: IQL injection, auth bypass, fuzzing | — | 🟡 1-2d | 🔴 | ✅ |

### ⚡ Migration Runner

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DB-01` | **Migration runner operativo (`vanta-cli migrate`):** Sincronizar migration.rs con vfile.rs (rango v1-v2), usar `VECTOR_INDEX_VERSION`, añadir `WAL_POSTCARD_VERSION` | `migration.rs`, `vfile.rs`, `wal.rs` | 🔴 2-3d | 🔴 | ⏳ |
| `DB-02` | Documentar estrategia de versionado de formatos físicos | `docs/architecture/STORAGE_VERSIONING.md` | 🟡 1d | 🔴 | ✅ |
| `—` | Snapshot tests: WAL integrity, VantaFile, HNSW, export/import (reemplazar TEST-05) | — | 🟡 1-2d | 🔴 | ❌ |

### 💥 Crash / Deadlock / OOM Fixes

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `CODE-018` | **`expect()` panic en serialización WASM vectors NaN/Inf** — Mata instancia WASM completa. Un nodo corrupto → DB inaccesible | `lib.rs:48-51` | 🟢 4h | 🔴 | ❌ |
| `CODE-015` | **`search_batch` usa rayon thread pool dentro de `py.detach`** — Deadlock por GIL si hilo re-entra Python | `lib.rs:1126-1143` | 🟡 1d | 🔴 | ❌ |
| `CODE-019` | **TS `close()` llama `free()` no `close()` del Rust** — Puede saltar shutdown completo del engine. Sin flush WAL | `vantadb.ts:49-51` | 🟢 4h | 🔴 | ❌ |

### 🐛 Python SDK Data Bugs

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `CODE-004` | **`hardware_profile()` muta dict de `capabilities()`** — `PyDict::clone()` es shallow ref. `merged_dict` y `caps_dict` apuntan al MISMO objeto | `lib.rs:1204-1231` | 🟡 1d | 🔴 | ❌ |
| `CODE-005` | **WASM `delete_file()` nunca hace await de la Promise** — `removeEntry()` retorna Promise que se pierde. Errores silenciosos | `opfs.rs:86-90` | 🟡 1d | 🔴 | ❌ |
| `CODE-011` | **100% errores Rust → `PyRuntimeError`** — Sin KeyError, ValueError, FileNotFoundError. Backend indescifrable | `lib.rs:700-846` (~40 sites) | 🟡 2-3d | 🔴 | ❌ |

### 📦 Publicación de Integraciones (BLOQUEA ADOPCIÓN)

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `INT-01` | **LangChain adapter → PyPI + PR upstream** | 🟡 1-2d | 🔴 | ❌ |
| `INT-02` | **LlamaIndex adapter → PyPI + PR upstream** | 🟡 1-2d | 🔴 | ❌ |
| `INT-03` | **Mem0 adapter → PyPI** | 🟡 1d | 🔴 | ❌ |
| `INT-04` | **CrewAI adapter → PyPI** | 🟡 1d | 🟠 | ❌ |
| `INT-05` | **DSPy adapter → PyPI** | 🟡 1d | 🟠 | ❌ |
| `INT-06` | **Haystack adapter → PyPI** | 🟡 1d | 🟠 | ❌ |
| `INT-07` | **Letta adapter → PyPI** | 🟡 1d | 🟠 | ❌ |
| `INT-08` | **OpenAI adapter → PyPI** | 🟡 1d | 🟠 | ❌ |
| `INT-09` | **Ollama adapter → PyPI** | 🟡 1d | 🟠 | ❌ |
| `INT-10` | **LiteLLM adapter → PyPI** | 🟡 1d | 🟢 | ❌ |
| `DEVOPS-05` | Pipeline CI unificado para publicar los 10 adapters a PyPI | 🟡 1-2d | 🔴 | ❌ |
| `REL-02` | **Publicar `vantadb-ts` en npm** (WASM build) | 🟡 1-2d | 🔴 | ❌ |

### 🧪 Testing Crítico

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `TEST-09` | Implementar tests WASM reales (39 tests, 11 categorías) | 🔴 2-3d | 🔴 | ✅ |
| `TEST-10` | Configurar Vitest + React Testing Library para frontend | 🔴 2-3d | 🔴 | ✅ |
| `TEST-06` | Load/stress tests Python (9) y TypeScript (6) | 🟡 2-3d | 🟡 | ✅ |

---

## TIER 1 — 🟠 Pre-Lanzamiento (Semanas 1-3, Jul 4-25)

> Items necesarios ANTES del Show HN para que el producto sea creíble.

### 🎯 Corrección de Marketing vs Realidad

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `MKT-11` | **Corregir `llms.txt`:** SQL (deferido), IVF (no implementado), latencia real | 🟢 1h | 🔴 | ❌ |
| `MKT-12` | **Auditar claims de performance** contra benchmarks reales. Publicar metodología | 🟡 1-2d | 🔴 | ❌ |
| `CODE-091` | **`hit.distance` etiquetado como `"score"` en JS** — Semantic confusion. consumer espera higher=better pero es distance | `lib.rs:488-490` | 🟢 2h | 🟡 | ❌ |
| `DX-02` | **Reducir p50 hybrid search de 62ms a <20ms** | 🟡 2-3d | 🔴 | ❌ |
| `—` | Eliminar `OldSerializationError` deprecated del enum | 🟢 1h | 🟡 | ❌ |

### 🏗️ Index & Storage Quality

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `CODE-007` | **Tombstone check bypass durante HNSW insert** — `search_layer` con `vector_store: None` marca todos como elegibles. Nodos eliminados usados como nearest neighbors. Degradación monótona del grafo | `core.rs:758-770` | 🟡 2-3d | 🔴 | ❌ |
| `CODE-008` | **HNSW nunca elimina nodos de `CPIndex`** — `delete()` no tiene `remove()` en DashMap. Crecimiento ilimitado. Solo full rebuild recupera | `engine.rs:1161-1202` | 🟡 1-2d | 🔴 | ❌ |
| `CODE-010` | **Compact layout en InMemory orfana tmp file** — `replace_backing_file()` retorna sin hacer nada. Archivos temporales huérfanos | `archive.rs:102-106` | 🟢 4h | 🟡 | ❌ |
| `CODE-024` | **`scan_nodes()` carga TODAS las KV pairs a RAM** — OOM en datasets medianos. Llamado desde 5 code paths distintos | `engine.rs:1431` | 🟡 2-3d | 🔴 | ❌ |
| `CODE-029` | **Read lock held durante todo search pipeline** — Bloquea writes en datasets >100K. Mismo patrón en scan_bitset, traverse, filter_field, hybrid_search | `engine.rs:196-343` | 🟡 2-3d | 🔴 | ❌ |
| `CODE-030` | **NaN en cosine_similarity → sort indefinido** — `partial_cmp.unwrap_or(Equal)` silencia el problema | `engine.rs:213,329` | 🟢 2h | 🟡 | ❌ |

### 🌐 Presencia Web y Landing Page

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `MKT-13` | **Integrar demo WASM interactiva en la hero** (botón "Try in browser") | 🟡 1-2d | 🔴 | ❌ |
| `MKT-14` | **Publicar 2 case studies** + ruta `/case-studies/` | 🟡 1-2d | 🔴 | ❌ |
| `WEB-06` | Migrar 637 inline styles a Tailwind classes | 🟡 3-5d | 🟡 | ✅ |
| `WEB-07` | Unificar animation libraries: mantener solo GSAP | 🟡 1-2d | 🟡 | ✅ |
| `WEB-18` | Componente `<VsTable>` reusable | 🟢 4-6h | 🟢 | ✅ |
| `WEB-19` | `React.lazy()` / code splitting por ruta | 🟢 2-4h | 🟢 | ✅ |

### 📚 Documentación Pre-Lanzamiento

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `DOC-13` | ADRs faltantes (6 de 11 creados) | 🟡 2-3d | 🟡 | ✅ |
| `DOC-14` | Performance Tuning Guide (479L) | 🟡 2-3d | 🟡 | ✅ |
| `DOC-16` | Tutorial series (3 creados) | 🟡 2-3d | 🟡 | ✅ |
| `DOC-17` | Diagramas Mermaid (5) | 🟡 1-2d | 🟡 | ✅ |
| `DOC-18` | Expandir HTTP_API.md (149L→504L) | 🟡 1d | 🟡 | ✅ |
| `—` | Docs de setup MCP por IDE (Cursor, Claude Code, Windsurf) | 🟡 1-2d | 🔴 | ❌ |
| `CODE-085` | **README Python documenta APIs que no existen** (`put_memory`, `search_hybrid`) | `README.md:33,48,59` | 🟢 1h | 🟡 | ❌ |

### 🧪 WASM y MCP

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `MCP-03` | Benchmarks WASM vs EdgeVec/minimemory/altor-vec/lattice-db | 🟡 2-3d | 🔴 | ❌ |
| `MCP-05` | Integration test suite MCP (9→25+) | 🟡 1-2d | 🟡 | ✅ |
| `WASM-03` | Demo AI Agent in browser (Transformers.js + OPFS) | 🟡 2-3d | 🟡 | ✅ |
| `WASM-04` | WASM bundle size optimization (<500KB gzip) | 🟡 1-2d | 🟡 | ✅ |
| `WASM-05` | SIMD acceleration for WASM build | 🟡 1-2d | 🟡 | ✅ |
| `CODE-059` | **`wasm-opt = false` en release** — Bundle 2-3x más grande de lo necesario | `Cargo.toml:13-14` | 🟢 1h | 🟡 | ❌ |
| `CODE-060` | **Demo WASM llama `put()`/`search()` sin `await`** — Si WASM se vuelve async, demo roto | `app.js:76-77` | 🟢 1h | 🟢 | ❌ |

### 📦 Distribución

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `DEVOPS-02` | ARM64 wheels (Apple Silicon, Graviton, RPi) | 🟡 2-3d | 🟠 | ❌ |
| `DEVOPS-06` | Homebrew formula para `vanta-cli` | 🟢 4-6h | 🟢 | ❌ |
| `DEVOPS-10` | **Firma de binarios Windows (SmartScreen)** — Research ✅, implementar | 🟡 2-3d | 🟡 | ❌ |
| `TSK-121` | SHA256 hash verification del wheel en tests | 🟢 2-4h | 🟢 | ❌ |
| `DEVOPS-07` | Dockerfile multi-stage mejorado | 🟡 2-4h | 🟡 | ✅ |
| `DEVOPS-11` | CodeQL analysis en CI | 🟢 2h | 🟡 | ✅ |

### 🧹 Code Health Core

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `PERF-13` | Refactor `read_only` check → helper method | — | 🟢 1h | 🟢 | ✅ |
| `PERF-14` | Refactor `init_telemetry` masivo | — | 🟡 1d | 🟡 | ✅ |
| `DOC-01` | Unit tests (91 nuevos) | — | 🟡 2-3d | 🟡 | ✅ |
| `DOC-02` | Refactor `insert_hnsw()` (177L→3 funciones) | — | 🟡 1d | 🟡 | ✅ |
| `CODE-014` | **LRU cache Python completamente muerto** — Cachea pero nunca lee. 100% overhead | `lib.rs:615-641` | 🟡 1d | 🟡 | ❌ |
| `CODE-067` | **Hash 64-bit XxHash: colisión bloquea ambos records** — Con 2^32 keys, ~0.5 colisiones esperadas | `serialization.rs:39-45` | 🟡 1-2d | 🟡 | ❌ |
| `CODE-089` | **`VantaConfig.storage_path` sin efecto en WASM** — Siempre InMemory, path ignorado. Usuarios engañados | `types.rs:142-147` | 🟢 4h | 🟡 | ❌ |
| `CODE-090` | **`insertNode(id: number)` hace `BigInt(id)` — overflow > 2^53** | `vantadb.ts:210-217` | 🟢 2h | 🟡 | ❌ |

### 🧪 CI/CD Web Quality

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `CODE-023` | **0 tests ejecutados en CI web** — Solo lint+typecheck+build. Sin vitest ni playwright | `web-ci.yml` | 🟡 1d | 🔴 | ❌ |
| `CODE-022` | **Three.js 600KB+ no usado en bundle** — Cero imports en todo web/src | `package.json:32,40` | 🟢 1h | 🟡 | ❌ |
| `CODE-070` | **Sin bundle analysis** — Ni visualizer ni size budget. Three.js pasó desapercibido | `vite.config.ts` | 🟢 2h | 🟡 | ❌ |
| `CODE-073` | **Cero e2e tests reales** — 2 tests, 11 líneas, solo homepage title check | `smoke.spec.ts` | 🟡 2-3d | 🟡 | ❌ |
| `CODE-078` | **Sin `playwright install` en CI** — Si se agregan e2e, van a fallar | `web-ci.yml` | 🟢 1h | 🟢 | ❌ |
| `CODE-079` | **`VERCEL_TOKEN` expuesto en CLI** — Mejor usar vercel-action | `web-deploy.yml:33-35` | 🟢 1h | 🟡 | ❌ |
| `CODE-080` | **Dependabot sin npm ecosystem** — Frontend sin update automático | `dependabot.yml` | 🟢 1h | 🟢 | ❌ |

---

## TIER 2 — 🟡 Launch Campaign (Semanas 3-6, Jul 18 - Ago 15)

> Items para el Show HN + Reddit + lanzamiento público.

### 🚀 Launch Campaign

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `LEG-01` | **Registrar trademark "VantaDB" (USPTO + EUIPO)** | 🟡 2-4h paper | 🔴 | ❌ |
| `LEG-02` | CLA para contribuciones | 🟢 1-2h | 🟠 | ✅ |
| `MKT-03` | **Show HN post** | 🟢 2h | 🔴 | ❌ |
| `MKT-04` | Reddit posts (r/rust, r/MachineLearning, r/LocalLLaMA) | 🟢 2-4h | 🟠 | ❌ |
| `MKT-05` | Technical blog posts (5+ pre-launch) | 🟡 2-3d | 🟠 | ❌ |
| `MKT-10` | "AI Agent Memory" campaign | 🟡 2-3d | 🟠 | ❌ |
| `MKT-15` | **Página de benchmarks competitivos** (`/product/benchmarks`) | 🟡 2-3d | 🔴 | ❌ |
| `MKT-16` | **Publicar metodología de benchmark GraphRAG** | 🟡 1-2d | 🔴 | ❌ |
| `TSK-103` | Public benchmark site | 🟡 2-3d | 🟠 | ❌ |
| `TSK-104` | Demo agent: LangChain + Ollama + VantaDB | 🟡 1-2d | 🟠 | ❌ |

### 🌐 Conversión y SEO

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `MKT-17` | Página de comparación competitiva interactiva | 🟡 2-3d | 🟡 | ❌ |
| `MKT-07` | Pricing page | 🟡 1-2d | 🔴 | ✅ |
| `WEB-08` | Anti-Slop Audit, Performance Budget, SEO Final Review | 🟢 1d | 🟢 | ✅ |
| `WEB-17` | TanStack Router vs React Router (✅ mantener) | 🟡 2-3d | 🟡 | ✅ |

### 🗄️ Database Evolution

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `DB-01` | Migration runner completo (ver TIER 0) | 🔴 3-5d | 🔴 | ⏳ |
| `DB-03` | ACID transactions research + prototipo | 🟡 3-5d | 🟡 | ✅ |
| `DB-04` | Expandir bitset 128→256 o dinámico (✅ dinámico) | 🟢 1-2d | 🟢 | ✅ |

### 🐛 GC & Background Tasks

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `CODE-031` | **GC delete failure silencioso en sweep** — Si `storage.delete()` falla, TTL entry se elimina igual. Nodo expirado sobrevive para siempre | `gc.rs:47-51` | 🟡 1d | 🟡 | ❌ |
| `CODE-032` | **TTL map crece sin límite en deletes pre-expiry** — Nodos con TTL borrados manualmente nunca se limpian del map | `gc.rs:26-28` | 🟡 1d | 🟡 | ❌ |
| `CODE-037` | **AuthRateLimiter HashMap unbounded** — Crecimiento por IP en ataque distribuido | `cli_server.rs:127-129` | 🟡 1d | 🟡 | ❌ |
| `CODE-064` | **`serialize_to_bytes` aloca Vec gigante** — ~2.5GB para 10M nodos de una | `core.rs:1401-1510` | 🟡 1d | 🟡 | ❌ |
| `CODE-065` | **`estimate_memory_bytes` O(n) en cada insert** — Itera todos los nodos. Debería ser cached counter | `core.rs:604-624` | 🟡 1-2d | 🟡 | ❌ |
| `CODE-066` | **WAL `recover_state()` muerto con `#[allow(dead_code)]`** — Y encima difiere del vivo (sí escribía backend). Peligro de confusión | `wal.rs:21` | 🟢 2h | 🟢 | ❌ |

### 👥 Comunidad

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `COM-01` | **Discord server** | 🟢 2-4h | 🔴 | ❌ |
| `TSK-106` | **Habilitar GitHub Discussions** | 🟢 1h | 🟡 | ❌ |
| `TSK-107` | Community showcase page | 🟢 4-6h | 🟡 | ❌ |
| `TSK-108` | Newsletter setup | 🟢 2-4h | 🟢 | ❌ |
| `—` | Good first issues (20+ tagged) | 🟢 2-4h | 🟠 | ❌ |

### 🎨 SDK Mejoras

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `—` | TypeScript SDK hardening: type safety, error wrapping, JSDoc, tests | — | 🟡 2-3d | 🔴 | ❌ |
| `—` | Python SDK: `put_batch` → keyword arguments | — | 🟢 1d | 🟡 | ❌ |
| `—` | Python SDK: eliminar LRU cache home-grown | — | 🟢 1d | 🟢 | ❌ |
| `CODE-045` | **`OperationalMetrics` TS 70% incompleto** — 11 de 37 campos mapeados | `types.ts:120-132` | 🟡 1d | 🟡 | ❌ |
| `CODE-046` | **`_mapRecord` es identity lie** — `any → T` sin validación alguna | `vantadb.ts:18-20` | 🟢 2h | 🟡 | ❌ |
| `CODE-047` | **Tests TS con `catch {}` vacío** — 4 tests que pasan SIEMPRE. No testean nada | `dx04.test.ts:107-112` | 🟢 2h | 🟢 | ❌ |
| `CODE-081` | **Python `put_batch` API posicional frágil** — 5-tuple sin nombres. Si orden cambia en Rust, Python se rompe | `lib.rs:765-789` | 🟢 4h | 🟡 | ❌ |
| `CODE-083` | **Sin `.pyi` type stubs** — IDEs sin autocompletado | — | 🟡 1d | 🟢 | ❌ |
| `CODE-084` | **`connect()` sin `memory_limit`** — Potencialmente unbounded vs constructor | `lib.rs:1426-1433` | 🟢 2h | 🟢 | ❌ |
| `CODE-086` | **Métodos TS `async` sin async real** — Promise overhead innecesario | `vantadb.ts` | 🟢 2h | 🟢 | ❌ |
| `CODE-087` | **`_mapRecord` O(n) copy en `putBatch()`/`list()`** — Sin propósito | `vantadb.ts:87,109` | 🟢 1h | 🟢 | ❌ |
| `CODE-088` | **Object reconstruction duplicada en `search()`/`explainSearch()`** — 7 líneas duplicadas | `vantadb.ts:115-151` | 🟢 1h | 🟢 | ❌ |
| `DX-01` | Refactor API: `VantaDB()` → `connect()` | 🟠 1-2d | 🟠 | ✅ |
| `DX-04` | TS SDK: mejorar de 18 tests a 50+ | 🟡 2-3d | 🟡 | ✅ |

### 🔧 Accesibilidad Web

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `CODE-048` | **Skip link después de `<Nav />`** — Usuario de teclado tabula toda nav antes de verlo | `__root.tsx:140-143` | 🟢 1h | 🟡 | ❌ |
| `CODE-049` | **Sin focus trapping en drawer mobile** — Foco escapa detrás del overlay. No retorna al cerrar | `Nav.tsx` | 🟡 1d | 🟡 | ❌ |

---

## TIER 3 — 🔵 Post-Lanzamiento (Semanas 6-12, Ago 15 - Sep 30)

> Items post-Show HN, previo a Phase 5.

### 📦 Distribución Avanzada

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `DEVOPS-06` | Homebrew formula | 🟢 4-6h | 🟢 | ❌ |
| `DEVOPS-09` | Auto-deploy web a Vercel en push a main | 🟡 1d | 🟡 | ✅ |
| `DEVOPS-08` | Docs build verification en CI | 🟢 2-4h | 🟢 | ✅ |
| `—` | Publicar 8 workspace members en crates.io | 🟡 2-3d | 🟡 | ❌ |

### 🧪 Testing Post-Launch

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `TEST-04` | Regression test suite (12 tests) | 🟡 1-2d | 🟡 | ✅ |
| `TEST-05` | Snapshot testing (7 tests) | 🟡 1-2d | 🟡 | ✅ |
| `TEST-07` | Fix test-threads: Windows 2, Linux/macOS paralelismo | 🟢 2h | 🟢 | ✅ |
| `TEST-08` | Fix `chaos_integrity` required-features | 🟠 1h | 🟠 | ✅ |
| `CODE-033` | **Tests GC usan `Box::leak`** — Leaks file handles. Windows TempDir cleanup falla | `gc.rs:88-159` | 🟡 1d | 🟢 | ❌ |
| `CODE-035` | **Test config asume CPU 8-core** — `assert_eq!(..., 16)` falla en 4/16/32 cores | `config.rs:602` | 🟢 1h | 🟢 | ❌ |
| `CODE-043` | **`Cargo_test.toml` stale duplicate** — Features diferentes al real. Time bomb | `Cargo_test.toml` | 🟢 1h | 🟢 | ❌ |
| `CODE-044` | **`test_search_batch` skipeado pero API ya existe** — Test muerto | `tests/test_sdk.py:144` | 🟢 1h | 🟢 | ❌ |
| `CODE-057` | **`debug = 0` en profile.test** — Backtraces sin line numbers. Debug imposible | `Cargo.toml:508-510` | 🟢 1h | 🟡 | ❌ |
| `CODE-074` | **Cero visual regression tests** — Sin Percy/Chromatic/Playwright screenshots | — | 🟡 2-3d | 🟡 | ❌ |
| `CODE-075` | **Sin coverage provider en vitest** — No hay métricas de cobertura | `vitest.config.ts` | 🟢 1h | 🟢 | ❌ |

### 🛡️ Seguridad Post-Launch

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `SEC-04` | Auth hardening: constant-time, rate limiting, `/metrics` auth | 🟠 2-3d | 🟠 | ✅ |
| `SEC-05` | RBAC design | 🟡 1-2d | 🟡 | ✅ |
| `SEC-06` | SBOM generation | 🟡 1-2d | 🟡 | ✅ |
| `SEC-07` | CodeQL + cargo-deny en CI | 🟡 1d | 🟡 | ✅ |
| `CODE-036` | **TLS 1.3 only** — Rechaza TLS 1.2 (curl legacy, .NET, Java 8) | `cli_server.rs:671-673` | 🟢 2h | 🟢 | ❌ |
| `CODE-061` | **Signal handler SIGBUS llama `warn!()`** — No signal-safe. UB potencial | `vfile.rs:141-167` | 🟡 1d | 🟡 | ❌ |
| `CODE-058` | **Ignored advisories en deny.toml sin rationale** — Sin plan de resolución | `deny.toml:3-4` | 🟢 1h | 🟢 | ❌ |

### 🧹 Code Health General

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `CODE-034` | **`VANTA_BACKEND=fjall` triggers warning falso** — Valor válido no en match | `config.rs:271-281` | 🟢 1h | 🟢 | ❌ |
| `CODE-038` | **LRU Python no refresca orden en update** — Item updated se evicta prematuro | `lib.rs:60-71` | 🟢 2h | 🟢 | ❌ |
| `CODE-039` | **Empty list `[]` siempre `ListString`** — Ambiguo semánticamente | `lib.rs:87-89` | 🟢 1h | 🟢 | ❌ |
| `CODE-040` | **List type inference del primer elemento** — `[42,"hello"]` error confuso | `lib.rs:91-151` | 🟢 2h | 🟢 | ❌ |
| `CODE-041` | **`operational_metrics()` sin `allow_threads()`** — GIL retenido innecesario | `lib.rs:1045-1048` | 🟢 1h | 🟢 | ❌ |
| `CODE-042` | **`BUFFER_CACHE` thread-local declarado, NUNCA usado** | `lib.rs:24-26` | 🟢 1h | 🟢 | ❌ |
| `CODE-050` | **Date sorting produce NaN** — `new Date("").getTime()` cuando falta frontmatter | `blog.ts:67` | 🟢 1h | 🟢 | ❌ |
| `CODE-051` | **`motion` chunk config para dep no instalado** — Dead config | `vite.config.ts:18` | 🟢 1h | 🟢 | ❌ |
| `CODE-052` | **`marked.parse()` en import time** — Parse eager de todos los posts | `blog.ts:53` | 🟡 1d | 🟢 | ❌ |
| `CODE-053` | **docs-api: 130 líneas dead code, nunca renderizado** — Redirect antes del lazy | `docs-api.*` | 🟢 1h | 🟢 | ❌ |
| `CODE-054` | **`QueryClient` recreado en cada `getRouter()`** — Cache loss frágil | `router.tsx:5-16` | 🟢 1h | 🟢 | ❌ |
| `CODE-055` | **Sin `rust-version.workspace` en miembros** — MSRV no enforced | Todos los member `Cargo.toml` | 🟢 1h | 🟢 | ❌ |
| `CODE-056` | **Duplicate `reqwest` 0.12 + 0.13** — Compila ambos | Múltiples `Cargo.toml` | 🟢 1h | 🟢 | ❌ |
| `CODE-062` | **Cursor reset en archivo corrupto sin zero-fill** — Garbage data holes | `vfile.rs:446-453` | 🟢 2h | 🟢 | ❌ |
| `CODE-063` | **`grow_to` puede shrink sin validación** — Potencial DB truncation | `vfile.rs:550` | 🟢 1h | 🟢 | ❌ |
| `CODE-068` | **33+ imágenes diseño (~20-50MB) commiteadas** — Fuera de source code | `web/src/SourceDesign/` | 🟢 1h | 🟢 | ❌ |
| `CODE-069` | **`.tanstack/** ignorado pero `routeTree.gen.ts` committed** — CI inconsistency | `.gitignore` | 🟢 1h | 🟢 | ❌ |
| `CODE-071` | **`getAllPosts()` sin memo** — Parse en cada render | `index.lazy.tsx:11` | 🟢 1h | 🟢 | ❌ |
| `CODE-072` | **Array index como `key` en ~20+ listas** — Reconciliation bug si se filtra | Múltiples `.lazy.tsx` | 🟡 1d | 🟢 | ❌ |
| `CODE-076` | **GSAP ScrollTrigger sin cleanup** — Duplicados en remounts | `SwissBackToTop.tsx:7-48` | 🟢 2h | 🟢 | ❌ |
| `CODE-077` | **`useState<number>` para hover en vez de CSS `:hover`** — Re-renders | Múltiples `.lazy.tsx` | 🟡 1d | 🟢 | ❌ |
| `CODE-082` | **Python f64→f32 silent precision loss** — Sin warning al usuario | `lib.rs:195-206` | 🟢 1h | 🟢 | ❌ |
| `CODE-016` | **Python `__aexit__` bloquea event loop** — Llama `close()` sync | `__init__.py:40-41` | 🟢 2h | 🟡 | ❌ |
| `CODE-017` | **`hardware_profile` property bloquea event loop** — Sin asyncio.to_thread | `__init__.py:231-233` | 🟢 2h | 🟡 | ❌ |

---

## PHASE 5 — ⬜ Enterprise / Pre-Seed (Q4 2026)

> Items post-lanzamiento público. No bloquean v0.2.0.

### 5.A Enterprise Readiness

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `TSK-72` | AES-256-GCM at-rest encryption | 🟡 3-5d | 🟡 | ❌ |
| `TSK-107b` | Audit logging enterprise (JSONL, timestamp + op) | 🟡 2-3d | 🟡 | ❌ |
| `TSK-110` | SBOM en cada release (vía SEC-06) | 🟡 1d | 🟡 | ✅ |
| `BIZ-02` | WAL shipping asíncrono (replication sin Raft) | 🟡 3-5d | 🟡 | ❌ |
| `TSK-122` | Sharded-slab para HNSW lock-free | 🟡 2-3d | 🟡 | ❌ |
| `TSK-131` | PITR via archival WAL | 🟡 3-5d | 🟡 | ❌ |
| `TSK-133` | Incremental backup (snapshot + WAL deltas) | 🟢 2-3d | 🟢 | ❌ |
| `TSK-142` | WASM persistence via OPFS + Web Workers | 🟡 2-3d | 🟡 | ❌ |
| `ENT-01` | SOC 2 prep (access controls, audit trails, retention) | 🟡 3-5d | 🟡 | ❌ |
| `ENT-02` | HIPAA assessment + BAA readiness | 🟡 2-3d | 🟡 | ❌ |
| `ENT-03` | Multi-tenant isolation (RAM, IOPS, storage quotas) | 🟡 3-5d | 🟡 | ❌ |
| `ENT-04` | Connection pooling + circuit breaker | 🟡 2-3d | 🟡 | ❌ |
| `GOV-01` | **Governance redesign** — Rediseñar admission control, conflict resolution, y consistency buffer basado en el design doc de experimental-governance. 12 bugs conocidos (Bloom saturation, friction invertido, death spiral, etc.). Ver `docs/architecture/EXPERIMENTAL_GOVERNANCE_DESIGN.md` | 🟠 3-5d | 🟡 | ❌ |
| `LOW-01` | TLS 1.3 on vantadb-server | 🟢 1-2d | 🟢 | ✅ |

### 5.B VantaDB Cloud & Business

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `CLD-01` | VantaDB Cloud Beta (Fly.io, NVMe, Bearer auth) | 🟡 3-5d | 🟡 | ❌ |
| `CLD-02` | Pitch Deck + one-pager | 🟡 2-3d | 🟡 | ❌ |
| `CLD-03` | Enterprise pilot program (3-5 early adopters) | 🟡 2-3d | 🟡 | ❌ |
| `CLD-04` | Case Studies (mínimo 2) | 🟡 2-3d | 🟡 | ❌ |
| `CLD-06` | Stripe billing integration | 🟡 2-3d | 🟡 | ❌ |
| `CLD-07` | Web dashboard (admin panel) | 🟡 3-5d | 🟡 | ❌ |
| `BIZ-01` | Enterprise crate (encryption, audit, RBAC, replication) | 🟡 3-5d | 🟡 | ⏳ |
| `BIZ-03` | Pricing page (ver MKT-07) | 🟡 1-2d | 🟡 | ✅ |
| `BIZ-04` | Cloud architecture design doc | 🟡 2-3d | 🟡 | ❌ |
| `BIZ-05` | Competitive pricing analysis | 🟡 1-2d | 🟡 | ❌ |
| `BIZ-06` | Pitch Deck (10 slides) | 🟡 2-3d | 🟡 | ❌ |

---

## 📊 Matriz de Impacto vs Esfuerzo (Priorización)

```
                    Alta Impacto
                        │
    🔴  DB-01           │   🔴  CODE-001 (WAL replay backend)
    🔴  INT-01/02       │   🔴  CODE-002 (WAL before validation)
    🔴  REL-02 (npm)    │   🔴  CODE-003 (process::exit flush)
    🔴  MKT-15 (bench)  │   🔴  CODE-007 (tombstone bypass)
    🔴  TS SDK hardening│   🔴  CODE-008 (HNSW never removes)
    🔴  Python errors    │   🔴  CODE-020/021 (XSS)
    🔴  MKT-11 (llms.txt)│   🔴  CODE-011 (PyRuntimeError)
    🟡  DX-02 (62ms)    │   🟡  CODE-024 (scan_nodes OOM)
                        │   🟡  CODE-029 (read lock search)
                        │
Bajo ───────────────────┼────────────────── Alto
Esfuerzo                │   Esfuerzo
                        │
    🟢  DEVOPS-06       │   🟡  DEVOPS-02 (ARM64)
    🟢  TSK-108         │   🟡  DEVOPS-10 (signing)
    🟢  COM-01          │   🟡  MCP-03 (WASM bench)
    🟢  TSK-106         │   🟡  CODE-045 (OperationalMetrics 70%)
    🟢  CODE-085        │   🟡  CODE-064/065 (index perf)
    🟢  CODE-022        │   🟡  CODE-073 (e2e tests)
    🟢  CODE-048/049    │
                        │
                    Bajo Impacto
```

### 🎯 Quick Wins (Alto Impacto, Bajo Esfuerzo) — HACER PRIMERO

| ID | Tarea | Tiempo | Dependencia |
|----|-------|--------|-------------|
| `MKT-11` | Corregir `llms.txt` (SQL, IVF, latency) | 🟢 1h | — |
| `COM-01` | Abrir Discord | 🟢 2-4h | — |
| `TSK-106` | Activar GitHub Discussions | 🟢 1h | — |
| `MKT-13` | Botón "Try in browser" WASM en hero | 🟡 1-2d | WASM-03 ✅ |
| `MKT-14` | Case studies en landing page | 🟡 1-2d | Docs exist |
| `CODE-020` | CSP: sacar unsafe-eval | 🟢 1-2h | — |
| `CODE-021` | Agregar DOMPurify al blog | 🟢 2h | — |
| `CODE-022` | Sacar Three.js de deps | 🟢 1h | — |
| `CODE-027` | Reemplazar expect por error en get_many | 🟢 2-4h | — |
| `CODE-048` | Mover skip link antes de Nav | 🟢 1h | — |
| `CODE-085` | Actualizar README Python | 🟢 1h | — |
| `CODE-091` | Renombrar distance→score en JS bindings | 🟢 2h | — |

### 💎 High-Investment (Alto Impacto, Alto Esfuerzo) — PLANEAR BIEN

| ID | Tarea | Tiempo | Riesgo |
|----|-------|--------|--------|
| `DB-01` | Migration runner completo | 2-3d | ⚠️ Crítico para release |
| `CODE-001` | WAL replay escriba backend metadata | 2-3d | ⚠️ Data-loss real |
| `CODE-002` | WAL append después de validación | 2-3d | ⚠️ Phantom records |
| `CODE-007` | Tombstone check en HNSW insert | 2-3d | 🟡 Degradación calidad |
| `CODE-008` | Implementar HNSW remove() | 1-2d | 🟡 Memory leak |
| `CODE-011` | Mapeo VantaError→Python exceptions | 2-3d | 🟢 Adopción SDK |
| `CODE-024` | scan_nodes paginado o streaming | 2-3d | 🟡 OOM |
| `CODE-029` | Read lock acotado en search | 2-3d | 🟡 Write starvation |
| `INT-01/02` | LangChain + LlamaIndex → PyPI | 1-2d | ⚠️ Bloquea adopción |
| `DX-02` | Reducir latency 62ms→20ms | 2-3d | ⚠️ Puede requerir re-arquitectura |

---

## ⚠️ Riesgos y Bloqueadores

| Riesgo | Probabilidad | Impacto | Mitigación |
|--------|-------------|---------|------------|
| WAL replay no escribe backend | 🔴 Alta | 🔴 Data-loss post-crash | **CODE-001** TIER 0 |
| WAL append antes de validación | 🟡 Media | 🔴 Phantom recovery | **CODE-002** TIER 0 |
| `process::exit()` sin flush | 🟡 Media | 🔴 Lost records | **CODE-003** TIER 0 |
| save_vector_index traga errores | 🟡 Media | 🔴 Persistencia falsa | **CODE-009** TIER 0 |
| BFS order vacío destruye DB | 🟢 Baja | 🔴 Data-loss total | **CODE-026** TIER 0 |
| Crash por expect() en backend corrupto | 🟢 Baja | 🔴 Server caído | **CODE-027** TIER 0 |
| XSS via CSP unsafe-eval + blog | 🔴 Alta | 🔴 Ejecución remota | **CODE-020/021** TIER 0 |
| Path traversal Python SDK | 🟡 Media | 🔴 File system access | **CODE-012** TIER 0 |
| HNSW sin remove + tombstone bypass | 🔴 Alta | 🟡 Degradación calidad | **CODE-007/008** TIER 1 |
| scan_nodes OOM | 🟡 Media | 🟡 Server crash | **CODE-024** TIER 1 |
| Read lock en search bloquea writes | 🟡 Media | 🟡 Write starvation | **CODE-029** TIER 1 |
| Python 100% RuntimeError | 🔴 Alta | 🟡 Sin diagnóstico | **CODE-011** TIER 0 |
| Migration runner roto | 🟡 Media | 🔴 Data loss | DB-01 TIER 0 |
| LangChain/LlamaIndex no publicados | 🔴 Alta | 🔴 Sin adopción | INT-01/02 TIER 0 |
| Latencia 62ms vs target 20ms | 🟡 Media | 🟡 Claims engañosos | DX-02 TIER 1 |
| Trademark no registrado | 🟡 Media | 🔴 Name squatting | LEG-01 TIER 2 |
| Sin ARM64 wheels | 🟡 Media | 🟡 Pierde edge/RPi | DEVOPS-02 TIER 1 |
| `llms.txt` con datos falsos | 🔴 Alta | 🟡 AI crawlers mienten | MKT-11 TIER 1 |
| Sin tests web en CI | 🔴 Alta | 🟡 Regresiones no detectadas | CODE-023 TIER 1 |

---

## 📋 Resumen de Carga de Trabajo por Categoría

| Categoría | TIER 0 ❌ | TIER 1 ❌ | TIER 2 ❌ | TIER 3 ❌ | PHASE 5 ❌ | Total |
|-----------|----------|----------|----------|----------|-----------|-------|
| 🩹 Data Loss & Crash Prev | 6 | 0 | 0 | 0 | 0 | 6 |
| 🛡️ Seguridad & Integrity | 4 | 0 | 0 | 3 | 0 | 7 |
| ⚡ Migration Runner | 3 | 0 | 0 | 0 | 0 | 3 |
| 💥 Crash/Deadlock Fixes | 3 | 0 | 0 | 0 | 0 | 3 |
| 🐛 Python SDK Data Bugs | 3 | 0 | 0 | 0 | 0 | 3 |
| 📦 Integraciones & Release | 12 | 0 | 0 | 0 | 0 | 12 |
| 🧪 Testing | 0 | 0 | 0 | 2 | 0 | 2 |
| 🎯 Marketing vs Realidad | 0 | 4 | 0 | 0 | 0 | 4 |
| 🏗️ Index & Storage Quality | 0 | 6 | 0 | 0 | 0 | 6 |
| 🌐 Web & Landing Page | 0 | 2 | 0 | 0 | 0 | 2 |
| 📚 Documentación | 0 | 4 | 0 | 0 | 0 | 4 |
| 🧪 WASM & MCP | 0 | 6 | 0 | 0 | 0 | 6 |
| 📦 Distribución | 0 | 1 | 0 | 1 | 0 | 2 |
| 🧹 Code Health Core | 0 | 4 | 0 | 0 | 0 | 4 |
| 🧪 CI/CD Web Quality | 0 | 7 | 0 | 0 | 0 | 7 |
| 🚀 Launch Campaign | 0 | 0 | 10 | 0 | 0 | 10 |
| 🌐 Conversión & SEO | 0 | 0 | 2 | 0 | 0 | 2 |
| 🗄️ Database Evolution | 0 | 0 | 1 | 0 | 0 | 1 |
| 🐛 GC & Background Tasks | 0 | 0 | 6 | 0 | 0 | 6 |
| 👥 Comunidad | 0 | 0 | 5 | 0 | 0 | 5 |
| 🎨 SDK Mejoras | 0 | 0 | 12 | 0 | 0 | 12 |
| 🔧 Accesibilidad Web | 0 | 0 | 2 | 0 | 0 | 2 |
| 📦 Distribución Avanzada | 0 | 0 | 0 | 1 | 0 | 1 |
| 🧪 Testing Post-Launch | 0 | 0 | 0 | 8 | 0 | 8 |
| 🛡️ Seguridad Post-Launch | 0 | 0 | 0 | 3 | 0 | 3 |
| 🧹 Code Health General | 0 | 0 | 0 | 23 | 0 | 23 |
| 🏢 Enterprise Readiness | 0 | 0 | 0 | 0 | 10 | 10 |
| ☁️ VantaDB Cloud & Biz | 0 | 0 | 0 | 0 | 10 | 10 |
| **Total** | **31** | **34** | **38** | **41** | **20** | **164** |

Nota: La diferencia de 10 items respecto al total de 154 (vs 164 en tabla) se debe a subtareas ✅ completadas dentro de categorías que igual listamos para tracking.

---

## 📈 Timeline Consolidado

```
Jul 4-11   TIER 0 (🔴 31 items):
           ─ Data loss: CODE-001/002/003/009/026/027
           ─ Security: CODE-012/020/021, SEC-08/09/10
           ─ Migration: DB-01/02
           ─ Crash: CODE-015/018/019
           ─ Python bugs: CODE-004/005/011/014
           ─ Integrations: INT-01→10, DEVOPS-05, REL-02, tests
Jul 11-18  TIER 1 (🟠 34 items):
           ─ Marketing: MKT-11/12, DX-02, CODE-091
           ─ Index: CODE-007/008/010/024/029/030
           ─ Web: MKT-13/14, CODE-022/023/070/073/078/079/080
           ─ Docs: MCP per-IDE, CODE-085
           ─ WASM: MCP-03, CODE-059/060
           ─ Distribución: DEVOPS-02/06/10
           �─ Code health: CODE-014/067/089/090
Jul 18-25  TIER 2 (🟡 38 items):
           ─ Launch: LEG-01, MKT-03→05/10/15/16, TSK-103/104
           ─ GC: CODE-031/032/037/064/065/066
           ─ Comunidad: COM-01, TSK-106/107/108, Good first issues
           ─ SDK: CODE-045/046/047/081/083/084/086/087/088, DX-01/04
           ─ Accesibilidad: CODE-048/049
           ─ SEO/Conversion: MKT-17, WEB-08
Ago-Sep    TIER 3 (🔵 41 items):
           ─ Testing: CODE-033/035/043/044/057/074/075
           ─ Seguridad: CODE-036/058/061
           ─ Code health general: CODE-034/038→042/050→056/062/063/068/069/071/072/076/077/082
           ─ Distribución: Homebrew, crates.io
           ─ Post-launch: SEC-04→07, TEST-04/05/07/08
Oct+       PHASE 5 (⬜ 20 items):
            ─ Enterprise: encryption, RBAC, audit, SOC2, HIPAA
            ─ Governance: GOV-01 redesign (admission, conflict, consistency)
            ─ Cloud: WAL shipping, billing, dashboard, pitch deck
```

---

## ✅ Definition of Ready (DoR)

- [ ] ID único asignado
- [ ] Prioridad definida (🔴🟠🟡🟢)
- [ ] Dependencias identificadas
- [ ] Archivos/directorios involucrados conocidos
- [ ] Esfuerzo estimado

## ✅ Definition of Done (DoD)

- [ ] Código compila (`cargo check` / `tsc --noEmit`)
- [ ] Tests pasan (`cargo test` / `vitest run`)
- [ ] Linters pasan (`cargo clippy` / `eslint`)
- [ ] Docs afectados actualizados
- [ ] Tarea movida de Backlog.md a progreso/README.md
- [ ] Changelog actualizado si es cambio visible al usuario
- [ ] `scripts/validate-docs-coverage.ps1` pasa

---

## See Also

- [[master-index]] — Central navigation
- [[docs/strategy/ACTION_PLAN.md]] — Detailed execution plan
- [[docs/strategy/ROADMAP.md]] — Phase definitions
- [[CHANGELOG.md]] — Release history
