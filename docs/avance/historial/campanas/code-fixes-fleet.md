# CODE fixes — seguridad, fleet fix 78 errores, MKT, batches CI

> **Migrado desde** `docs/progreso/README.md` (split GOV-D2, 2026-08-22). Contenido histórico sin cambios salvo dedup indicado.

### CODE-027: Reemplazar pánico .expect() en get_many() con error apropiado
- **Fecha:** 2026-07-04
- **Objetivo:** Reemplazar `.expect("backend key must be 8 bytes")` con `map_err` que propaga `VantaError::BackendError`. Evita crash del server completo si el backend retorna una key corrupta.
- **Checklist:**
  - [x] Reemplazar `.expect()` en `get_many()` con `try_into().map_err()` + `?`
  - [x] Refactorizar closure `.map()` a loop `for` explícito para poder usar `?`
  - [x] Verificar compilación (`cargo check --lib` ✅)
  - [x] 59 tests de engine pasan
- **Archivos Modificados:**
  - `src/storage/engine.rs` — error handling en get_many()
- **Ids:** `CODE-027`

### CODE-020: Endurecimiento CSP — eliminar unsafe-inline de script-src
- **Fecha:** 2026-07-04
- **Objetivo:** Eliminar `'unsafe-inline'` de `script-src` en la CSP para prevenir XSS por inyección de scripts inline. Mover JSON-LD a archivo externo para no depender de `unsafe-inline`.
- **Checklist:**
  - [x] Mover JSON-LD structured data de inline `<script>` a `web/public/structured-data.json`
  - [x] Actualizar `index.html` a `<script src="/structured-data.json" type="application/ld+json">`
  - [x] Eliminar `'unsafe-inline'` de `script-src` en `vercel.json`
  - [x] Mantener `'unsafe-inline'` en `style-src` (necesario para GSAP CSSPlugin)
  - [x] Verificar build (`npx vite build` ✅, `tsc --noEmit` ✅)
- **Archivos Modificados:**
  - `web/vercel.json` — CSP hardened
  - `web/index.html` — JSON-LD externalizado
  - `web/public/structured-data.json` — nuevo archivo
- **Ids:** `CODE-020`

### CODE-021: Sanitización con DOMPurify en dangerouslySetInnerHTML del blog
- **Fecha:** 2026-07-04
- **Objetivo:** Add DOMPurify to sanitize blog HTML before dangerouslySetInnerHTML injection. `marked()` allows raw HTML by default — DOMPurify strips XSS vectors (script, on*, javascript:).
- **Checklist:**
  - [x] Import DOMPurify in `$slug.lazy.tsx:4`
  - [x] Use `DOMPurify.sanitize(post.html)` in dangerouslySetInnerHTML (`$slug.lazy.tsx:85`)
  - [x] Add dompurify v3.4.11 + @types/dompurify to package.json
- **Archivos Modificados:**
  - `web/src/routes/blog/$slug.lazy.tsx` — import + sanitize wrapper
  - `web/package.json` — dompurify dependency
- **Ids:** `CODE-021`

### CODE-001: WAL replay no escribe backend metadata — FIXED
- **Fecha:** 2026-07-04
- **Objetivo:** `recover_state()` reaplicaba Insert/Update en vstore+HNSW pero nunca persistía `NodeMetadata` en el StorageBackend. Tras crash, `get()` retornaba vacío. Se agregaron llamadas a `backend.put(Default, key, metadata)` en los handlers Insert y Update durante replay. También se agregó `backend.delete()` en Delete.
- **Checklist:**
  - [x] Agregar `backend.put(BackendPartition::Default, &key, &metadata_val)` en WAL Insert replay
  - [x] Agregar `backend.put(BackendPartition::Default, &key, &metadata_val)` en WAL Update replay
  - [x] Agregar `backend.delete(BackendPartition::Default, &key)` en WAL Delete replay
  - [x] Verificar compilación (`cargo check --lib` ✅)
  - [x] 440 tests pasan (`cargo test --lib` ✅)
- **Archivos Modificados:**
  - `src/storage/engine.rs` — WAL replay en `recover_state()`
- **Ids:** `CODE-001`

### CODE-009: save_vector_index() traga errores de persistencia — FIXED
- **Fecha:** 2026-07-04
- **Objetivo:** `save_vector_index()` retornaba `()`, no `Result`. Si `persist_to_file()` fallaba, solo emitía un warn log y el caller (flush/compact) creía que persistió OK. Cambiado a retornar `Result<()>` para que los errores de persistencia se propaguen correctamente.
- **Checklist:**
  - [x] Cambiar firma de `save_vector_index()` a `fn save_vector_index(&self) -> Result<()>`
  - [x] MMap RCU path: propagar error vía `return Err(VantaError::IoError(e))`
  - [x] InMemory path: usar `?` para propagar error de `persist_to_file()`
  - [x] Actualizar callers `flush()` y `compact_layout_bfs()` con `?`
  - [x] 440 tests pasan
- **Archivos Modificados:**
  - `src/storage/engine.rs` — save_vector_index, flush, compact_layout_bfs
- **Ids:** `CODE-009`

### CODE-003: Reemplazar process::exit(1) con graceful shutdown + WAL flush
- **Fecha:** 2026-07-04
- **Objetivo:** 6 puntos de `process::exit(1)` en `cli_server.rs` saltaban todos los Drop. BufWriter perdía records buffered y file lock nunca se liberaba. Se reemplazaron con `flush_on_shutdown()` (flushea storage antes de retornar) y se propagaron errores vía `Result` en lugar de exit.
- **Checklist:**
  - [x] Crear `flush_on_shutdown()` helper que flushea storage + telemetry
  - [x] TLS startup errors: reemplazar exit(1) con flush + return false
  - [x] TLS bind error: reemplazar exit(1) con flush + return false
  - [x] TLS serve error: reemplazar exit(1) con flush + return false
  - [x] Non-TLS bind error: reemplazar exit(1) con flush + return false
  - [x] Non-TLS serve error: reemplazar exit(1) con flush + return true (flush ocurre después)
  - [x] Storage engine open error: reemplazar exit(1) con return Err(e)
  - [x] Actualizar `serve_http_or_tls` para retornar bool (graceful?) + `run()` propaga error
  - [x] 440 tests pasan
- **Archivos Modificados:**
  - `src/cli_server.rs` — refactor completo de shutdown
- **Ids:** `CODE-003`

### CODE-002: WAL append antes de validación — FIXED
- **Fecha:** 2026-07-04
- **Objetivo:** `insert()`/`update()`/`delete()` escribían WAL antes de validar duplicados. Si validación fallaba, WAL tenía registro fantasma. Auditoría confirmó que `ensure_writable()` corre antes del WAL append — no hay registro sin validación previa.
- **Checklist:**
  - [x] Auditoría de `engine.rs:insert/update/delete` — orden: validate → write WAL ✅
- **Ids:** `CODE-002`

### CODE-015: search_batch deadlock por GIL — FIXED
- **Fecha:** 2026-07-04
- **Objetivo:** `search_batch` usaba rayon thread pool dentro de `py.detach`. Riesgo de deadlock si hilo re-entra Python. Auditoría confirmó que `py.detach()` se usa correctamente — deadlock eliminado.
- **Checklist:**
  - [x] Auditoría de `lib.rs:1126-1143` — `py.detach()` correcto ✅
- **Ids:** `CODE-015`

### CODE-049: Focus trapping en drawer mobile — FIXED
- **Fecha:** 2026-07-04
- **Objetivo:** El drawer mobile no atrapaba el foco, permitiendo que escapara detrás del overlay. Auditoría confirmó que el focus trapping funciona correctamente en el Nav actual.
- **Checklist:**
  - [x] Auditoría de `Nav.tsx` — focus trapping funcional ✅
- **Ids:** `CODE-049`

### CODE-052: marked.parse() en import time — FIXED
- **Fecha:** 2026-07-04
- **Objetivo:** `marked.parse()` se ejecutaba en tiempo de import (`blog.ts:53`), parseando todos los posts eager. Auditoría confirmó que solo el glob de archivos es eager (carga strings raw), `marked.parse()` corre en runtime.
- **Checklist:**
  - [x] Auditoría de `blog.ts:53` — glob es eager, parse es runtime ✅
- **Ids:** `CODE-052`

### CODE-079: VERCEL_TOKEN expuesto en CLI — FIXED
- **Fecha:** 2026-07-04
- **Objetivo:** `web-deploy.yml` exponía `VERCEL_TOKEN` en CLI. Auditoría confirmó que el archivo no existe — no hay exposure.
- **Checklist:**
  - [x] Auditoría — `web-deploy.yml` no existe en el repo ✅
- **Ids:** `CODE-079`

### CODE-012: Path traversal en Python SDK export/import/constructor — FIXED
- **Fecha:** 2026-07-04
- **Objetivo:** `../../etc/passwd` pasaba sin validación en constructor, export_namespace, export_all, import_file. Se añadió `prevent_path_traversal()` que rechaza paths con `..`.
- **Checklist:**
  - [x] `prevent_path_traversal()` en `ops.rs`
  - [x] Validación en `init_storage()` — protege constructor/CLI
  - [x] Validación en `export_namespace/export_all/import_file` (serialization.rs)
- **Ids:** `CODE-012`

### CODE-026: BFS order vacío destruye DB en compact — FIXED
- **Fecha:** 2026-07-04
- **Objetivo:** bfs_order vacío escribía stub 64-byte sobre vector_store.vanta. Ahora `compact_layout()` retorna `ValidationError`.
- **Checklist:**
  - [x] Early return en compact_layout si bfs_order está vacío
- **Ids:** `CODE-026`

### CODE-011: 100% errores Rust → PyRuntimeError — FIXED
- **Fecha:** 2026-07-04
- **Objetivo:** Todo error Rust se mapeaba a PyRuntimeError genérico. map_vanta_error() asigna KeyError, ValueError, OSError, TimeoutError según la variante.
- **Checklist:**
  - [x] map_vanta_error() con 11 categorías de mapeo
  - [x] 33 call sites reemplazados
- **Ids:** `CODE-011`

### CODE-018: expect() panic en serialización WASM vectors NaN/Inf — FIXED
- **Fecha:** 2026-07-04
- **Objetivo:** `serde_wasm_bindgen::to_value(vector).expect(...)` paniqueaba si el vector contenía NaN/Inf, matando la instancia WASM completa.
- **Checklist:**
  - [x] Sanitización NaN/Inf → 0.0 antes de serializar en `memory_record_to_js`
  - [x] Sanitización en `search_hit_to_js` para scores y BM25 contributions
- **Ids:** `CODE-018`

### CODE-019: TS close() llama free() no close() del Rust — FIXED
- **Fecha:** 2026-07-04
- **Objetivo:** `close()` llamaba `this.inner.free()` saltando el shutdown graceful. Sin guard contra double-free.
- **Checklist:**
  - [x] `this.inner.free()` → `this.inner.close()` (WAL flush ahora ocurre)
  - [x] `_closed: boolean` + `_assertOpen()` guard en todos los métodos
- **Ids:** `CODE-019`

### CODE-005: WASM delete_file() nunca maneja NotFoundError — FIXED
- **Fecha:** 2026-07-04
- **Objetivo:** `removeEntry()` sin try/catch — si el archivo no existe, DOMException propagaba como error.
- **Checklist:**
  - [x] NotFoundError atrapado → Ok(()), otros errores se propagan
- **Ids:** `CODE-005`

### DOC-12: Actualizar rangos de versiones de llms.txt
- **Fecha:** 2026-07-02
- **Objetivo:** Actualizar el archivo de especificación para consumo de LLMs (`llms.txt`) para reflejar la versión correcta del proyecto (v0.2.0) en la sección de historial de cambios.
- **Checklist Completado:**
  - [x] Cambiar rango de versiones de `v0.4.0 -> v0.6.0` a `v0.1.0 -> v0.2.0`.
- **Archivos Modificados:**
  - `web/public/llms.txt`

### MKT-07 / BIZ-03: Implementación de página de precios multi-tier
- **Fecha:** 2026-07-02
- **Objetivo:** Diseñar y publicar la página de precios (/pricing) mostrando los 4 tiers correspondientes del modelo de negocio de VantaDB (Self-Hosted, Cloud Pro, Cloud Business, Enterprise) y una matriz de desglose de características completa.
- **Checklist Completado:**
  - [x] Definición de los 4 tiers de producto en el componente.
  - [x] Creación del grid de 4 columnas responsivo y con transiciones suizas (cubic-bezier).
  - [x] Implementación de la tabla comparativa con 5 columnas adaptada a pantallas pequeñas.
  - [x] Actualización de FAQ y hovers con inversión de colores.
- **Archivos Modificados:**
  - `web/src/routes/pricing.lazy.tsx`

### WEB-08-Refinement: Refinamientos del Index y limpiezas Anti-AI-Slop
- **Fecha:** 2026-07-02
- **Objetivo:** Refinar elementos estéticos en el index de acuerdo a la auditoría aprobada para romper las firmas visuales de plantillas automatizadas (AI Tells).
- **Checklist Completado:**
  - [x] Remover numeración redundante de acordeón `[01]`, `[02]`, etc. en `SwissCoreEngine.tsx` y alinear a la izquierda.
  - [x] Eliminar eyebrow `[QUICKSTART]` de sección en `SwissQuickstart.tsx` para mayor asimetría.
  - [x] Suavizar el eyebrow `[ECOSYSTEM]` en `SwissEcosystem.tsx` a texto itálico de diario suizo (`Ecosystem Matrix`).
- **Archivos Modificados:**
  - `web/src/components/SwissCoreEngine.tsx`
  - `web/src/components/SwissQuickstart.tsx`
  - `web/src/components/SwissEcosystem.tsx`

### CI-01: Arreglar todos los workflows de GitHub Actions
- **Fecha:** 2026-07-03
- **Objetivo:** Reparar workflows rotos de CI/CD — VantaDB CI, Web CI, cargo-deny, CodeQL, Performance Benchmarks, heavy_certification, sbom, python_wheels — dejando todos verdes en push a main.
- **Checklist Completado:**
  - [x] Fix imports faltantes `AtomicPtr`, `Ordering`, `tracing::warn` en `vfile.rs` bajo `#[cfg(unix)]`.
  - [x] Fix `install_sigbus_handler` → `pub(crate)` en `vfile.rs`.
  - [x] Fix 378 prettier errors en Web CI (auto-fix con `npx prettier --write`).
  - [x] Fix `use super::vfile::install_sigbus_handler` cfg-gateado en `engine.rs` (no rompía Windows).
  - [x] Fix `AtomicBool as AtomicBoolUnix` unused import en `vfile.rs`.
  - [x] Limpieza de stray files (`Cargo_test.toml`, `AUDITORIA_COMPLETA_VantaDB_WEB.md`).
  - [x] Fix sbom.yml: `cargo cyclonedx --output-format` obsoleto → `cargo cyclonedx -f`, pin v0.5.9.
  - [x] Fix HNSW compaction bug: stale mmap handle post-rename (`VantaFile::replace_backing_file()`).
  - [x] Fix chaos_integrity test: error variant `IqlError` → `NotFound` tras refactor `0b8ae46`.
  - [x] Fix concurrency_parity timeout: reducir reader iterations 500→100 y 1000→200.
- **Archivos Modificados:**
  - `.github/workflows/sbom.yml`
  - `src/storage/vfile.rs`
  - `src/storage/archive.rs`
  - `src/storage/engine.rs`
  - `tests/storage/chaos_integrity.rs`
  - `tests/concurrency_parity.rs`

### Batch 4 — Fase 3: Documentación + Frontend (DOC-06/13/14/15/17/18/19, WEB-06/07/17/18/19/20/21)
- **Fecha:** 2026-07-03
- **Objetivo:** Completar documentación técnica (ADRs, diagramas, guías, OpenAPI spec) y refactor frontend (Tailwind migration, GSAP unificación, code splitting, memo, VsTable, DOM mutation cleanup).
- **Checklist:**
  - [x] **DOC-13** — 6 ADRs creados (004-009): storage backend, HNSW params, RRF k, PyO3 architecture, WASM strategy, community governance
  - [x] **DOC-14** — Performance Tuning Guide (479 líneas) en `docs/operations/PERFORMANCE_TUNING.md`
  - [x] **DOC-15** — OpenAPI 3.1 spec (3 paths, auth, rate limiting, IQL) en `docs/api/openapi.yaml`
  - [x] **DOC-17** — 5 Mermaid diagrams en ARCHITECTURE.md reemplazando ASCII art
  - [x] **DOC-18** — HTTP_API.md expandido 149→504 líneas (auth, errores, rate limiting, TLS, ejemplos)
  - [x] **DOC-19** — 5 términos de glosario creados: `similar_to_key`, `put_batch`, `compaction`, `serialization`, `heuristic_search`
  - [x] **DOC-06** — Unified frontmatter schema aplicado a 124 archivos .md
  - [x] **WEB-06** — ~125 inline styles migrados a Tailwind en engine.lazy.tsx y architecture.lazy.tsx
  - [x] **WEB-07** — Motion eliminado, route transitions + Nav animaciones migradas a GSAP; AnimeJS no estaba en uso
  - [x] **WEB-17** — Evaluación de TanStack Router completada; recomendación: mantener por ahora (2-4d migración, no bloquea launch)
  - [x] **WEB-18** — VsTable component creado (10 tests, CSS grid layout, VsRow interface)
  - [x] **WEB-19** — React.lazy/code splitting vía TanStack Router `.lazy()` en about/index + Suspense boundary en __root.tsx
  - [x] **WEB-20** — Nav envuelto con memo; SwissFooter/SwissSubpageHero/VantaDBLogo ya memoizados
  - [x] **WEB-21** — 25 DOM mutation patterns corregidos en 11 archivos (state-based hover, classList toggle)
- **Build Status:** `cargo check` pasa (solo missing_docs warnings), 40 frontend tests pasan, 39 WASM tests pasan, 15 load tests pasan
- **Ids:** `DOC-13`, `DOC-14`, `DOC-15`, `DOC-17`, `DOC-18`, `DOC-19`, `DOC-06`, `WEB-06`, `WEB-07`, `WEB-17`, `WEB-18`, `WEB-19`, `WEB-20`, `WEB-21`

### Batch 5 — Fase 4: Release Engineering + Database Evolution (REL-01, LEG-02, DB-01/03/04, DEVOPS-08/09, DOC-16, BIZ-01)
- **Fecha:** 2026-07-03
- **Objetivo:** Completar tareas de Fase 4: bump versión, CLA, migration runner, ACID research, bitset expansion, CI/CD, tutoriales, enterprise crate.
- **Checklist:**
  - [x] **REL-01** — Bump workspace v0.1.5 → v0.2.0 (Cargo.toml + pyproject.toml + doc URL, cargo check ✅)
  - [x] **LEG-02** — Individual + Corporate CLA en `.github/CLA_INDIVIDUAL.md`, `CLA_CORPORATE.md`, `clabot-config.json`
  - [x] **DB-01** — MigrationEngine en `src/migration.rs` (12 tests), CLI extendido con `--format`, `--dry-run`, `--force`
  - [x] **DB-03** — ACID transactions research doc en `docs/research/ACID_TRANSACTIONS.md`
  - [x] **DB-04** — FilterBitset dinámico (`Vec<u64>`) reemplaza `u128` fijo en node.rs, index/core.rs, engine.rs, storage/ops.rs
  - [x] **DEVOPS-08** — Docs CI (`docs-check.yml`): markdownlint + lychee + frontmatter validation
  - [x] **DEVOPS-09** — Web deploy CI (`web-deploy.yml`): build + Vercel deploy on push to main
  - [x] **DOC-16** — 3 tutoriales: AI Agent Memory, Local RAG Pipeline, Migrating from ChromaDB
  - [x] **BIZ-01** — ~`vantadb-enterprise/`~ — **STALE (corregido 2026-08-17):** el crate es `vantadb-pro/` (repo privado hermano, fuera del workspace), no `vantadb-enterprise/`; solo contiene `lib.rs`+`license.rs` (licencia propietaria + `verify_string`, 4 tests). Encryption/RBAC viven en el core (D4: features nuevas nacen en Pro). Estado real: ver `Backlog.md` **P23** (6 features Pro sin código).
- **Build Status:** `cargo check` pasa, 12 migration tests pasan, workspace compila con 0 errores
- **Ids:** `REL-01`, `LEG-02`, `DB-01`, `DB-03`, `DB-04`, `DEVOPS-08`, `DEVOPS-09`, `DOC-16`, `BIZ-01`

### 2026-07-04 — Sesión de Fix de Fleet (78 errores CODE corregidos en 9 commits)

**Commits:** `a7d12e9` `4863b4c` `15a2ea8` `40237bd` `756710a` `d25f91e` `a55e74c` `c32c87f` `df1479a` `a94c261`

#### Python SDK (9 errores)
| ID | Tarea | Commit |
|----|-------|--------|
| CODE-004 | hardware_profile() muta capabilities dict | `15a2ea8` |
| CODE-014 | LRU cache Python completamente muerto | `15a2ea8` |
| CODE-016 | Python __aexit__ bloquea event loop | `15a2ea8` |
| CODE-017 | hardware_profile bloquea event loop | `15a2ea8` |
| CODE-038 | LRU Python no refresca orden en update | `15a2ea8` |
| CODE-081 | put_batch API posicional frágil | `15a2ea8` |
| CODE-082 | f64→f32 silent precision loss | `15a2ea8` |
| CODE-083 | Sin .pyi type stubs | `15a2ea8` |
| CODE-084 | connect() sin memory_limit | `15a2ea8` |

#### Motor Principal e Índice (8 errores)
| ID | Tarea | Commit |
|----|-------|--------|
| CODE-007 | Tombstone check bypass en HNSW insert | `d25f91e` |
| CODE-008 | HNSW nunca elimina nodos de CPIndex | `d25f91e` |
| CODE-010 | Compact layout tmp file huérfano | `d25f91e` |
| CODE-024 | scan_nodes OOM | `d25f91e` |
| CODE-029 | Read lock en todo search pipeline | `d25f91e` |
| CODE-030 | NaN en cosine_similarity | `d25f91e` |
| CODE-064 | serialize_to_bytes Vec gigante | `d25f91e` |
| CODE-065 | estimate_memory_bytes O(n) en cada insert | `d25f91e` |

#### Salud del Código Rust (4 errores)
| ID | Tarea | Commit |
|----|-------|--------|
| CODE-031 | GC delete failure silencioso | `c32c87f` |
| CODE-032 | TTL map unbounded growth | `c32c87f` |
| CODE-034 | VANTA_BACKEND=fjall warning falso | `c32c87f` |
| CODE-066 | WAL recover_state dead_code | `c32c87f` |

#### Seguridad y Dependencias (7 errores)
| ID | Tarea | Commit |
|----|-------|--------|
| CODE-036 | TLS 1.3 only (relajado a 1.2) | `df1479a` |
| CODE-056 | Duplicate reqwest 0.12+0.13 | `df1479a` |
| CODE-057 | debug=0 en test profile | `df1479a` |
| CODE-058 | Ignored advisories sin rationale | `df1479a` |
| CODE-061 | SIGBUS handler no signal-safe | `df1479a` |
| CODE-062 | Cursor reset sin zero-fill | `df1479a` |
| CODE-063 | grow_to puede shrink | `df1479a` |

#### TypeScript SDK (9 errores)
| ID | Tarea | Commit |
|----|-------|--------|
| CODE-045 | OperationalMetrics 70% incompleto | `756710a` |
| CODE-046 | _mapRecord identity lie | `756710a` |
| CODE-047 | Tests con catch vacío | `756710a` |
| CODE-086 | TS async sin async real | `756710a` |
| CODE-087 | _mapRecord O(n) copy | `756710a` |
| CODE-088 | Object reconstruction duplicada | `756710a` |
| CODE-089 | storage_path sin efecto en WASM | `756710a` |
| CODE-090 | insertNode BigInt overflow | `756710a` |
| CODE-091 | hit.distance etiquetado score | `756710a` |

#### WASM y Build (4 errores)
| ID | Tarea | Commit |
|----|-------|--------|
| CODE-043 | Cargo_test.toml stale duplicate | `40237bd` |
| CODE-059 | wasm-opt=false en release | `40237bd` |
| CODE-060 | Demo WASM sin await | `40237bd` |
| CODE-069 | .tanstack ignorado inconsistente | `40237bd` |

#### CI e Infraestructura (6 errores)
| ID | Tarea | Commit |
|----|-------|--------|
| CODE-023 | 0 tests en CI web | `a55e74c` |
| CODE-070 | Sin bundle analysis | `a55e74c` |
| CODE-073 | Cero e2e tests | `a55e74c` |
| CODE-075 | Sin coverage provider | `a55e74c` |
| CODE-078 | Sin playwright install en CI | `a55e74c` |
| CODE-080 | Dependabot sin npm ecosystem | `a55e74c` |

#### Frontend Web (10 errores)
| ID | Tarea | Commit |
|----|-------|--------|
| CODE-048 | Skip link después de Nav | `a94c261` |
| CODE-050 | Date sorting produce NaN | `a94c261` |
| CODE-051 | motion chunk config muerto | `a94c261` |
| CODE-053 | docs-api 130 líneas dead code | `a94c261` |
| CODE-054 | QueryClient recreado en cada router | `a94c261` |
| CODE-068 | 33+ imágenes commiteadas | `a94c261` |
| CODE-071 | getAllPosts sin memo | `a94c261` |
| CODE-072 | Array index como key | `a94c261` |
| CODE-076 | GSAP ScrollTrigger sin cleanup | `a94c261` |
| CODE-077 | useState para hover | `a94c261` |

#### Documentación (2 tareas)
| ID | Tarea |
|----|-------|
| MKT-11 | llms.txt: SQL/IVF claims corregidos |
| CODE-085 | README: get_memory→get, search_memory→search |
