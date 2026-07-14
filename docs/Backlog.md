---
title: "Active Backlog — VantaDB"
type: backlog-tracking
status: active
tags: [vantadb, backlog, engineering, phases, priorities]
last_reviewed: 2026-07-13
---

# Active Backlog — VantaDB

> **Purpose:** Single source of truth for all project tasks.
> **Completed tasks:** `docs/CHANGELOG.md` + `docs/progreso/README.md`
> **Verification method:** All claims cross-checked against actual codebase via 4 sub-agents (Jul 13). See `docs/archive/` for superseded audit reports.
> **Total open items:** 88 (66 previos + 22 del docs-audit Jul 13)
> **Origen docs-audit:** `docs/strategy/ROADMAP.md`, `docs/bitacora.md`, `docs/reviews/FULL_CODEBASE_AUDIT_2026-07-11.md`, `docs/reviews/analisis_proyecto.md`, `docs/operations/PERFORMANCE_TUNING.md`, `docs/operations/REPO_CHECKLIST.md`, `docs/architecture/STORAGE_VERSIONING.md`, `docs/plans/2026-07-13-workflow-repair-campaign.md`, `docs/Investigaciones/cargo-check-optimizacion.md`, `docs/discord/todo.md`

---

## TIER 0 — 🔴 Bloqueantes de Release

> Items que bloquean cualquier release seguro.

### 📦 Publicación de Integraciones

| ID | Tarea | Esfuerzo | Prioridad | Estado | Verificación |
|----|-------|----------|-----------|--------|-------------|
| `INT-01` | **LangChain adapter → PyPI** | 🟡 1-2d | 🔴 | ❌ | Código existe en `vantadb-langchain/` + `integrations/langchain/`, no publicado |
| `INT-02` | **LlamaIndex adapter → PyPI** | 🟡 1-2d | 🔴 | ❌ | Código existe en `vantadb-llamaindex/` + `integrations/llamaindex/`, no publicado |
| `DEVOPS-05` | Pipeline CI unificado para publicar los 10 adapters a PyPI | 🟡 1-2d | 🔴 | ❌ | No existe pipeline integrado |
| `REL-02` | **Publicar `vantadb-ts` en npm** (WASM build) | 🟡 1-2d | 🔴 | ❌ | Código listo, `package.json` presente, no publicado |

### 🌐 Web & Landing

| ID | Tarea | Esfuerzo | Prioridad | Estado | Verificación |
|----|-------|----------|-----------|--------|-------------|
| `MKT-13` | **Enlazar demo WASM desde la hero** — Ruta `/demo` existe, demo funcional. Falta botón "Try in browser" en `NbTerminalHero` | 🟡 1-2h | 🔴 | ⏳ | `NbTerminalHero.tsx` no tiene link a `/demo`. Verificado. |

---

## TIER 1 — 🟠 Pre-Lanzamiento

> Necesario ANTES del Show HN.

### 📖 Documentación & Community

| ID | Tarea | Esfuerzo | Prioridad | Estado | Verificación |
|----|-------|----------|-----------|--------|-------------|
| `MKT-14` | **Publicar 2 case studies** + ruta `/case-studies/` | 🟡 1-2d | 🔴 | ❌ | `docs/case_studies/` drafts existen, no desplegados |
| `TSK-106` | **Habilitar GitHub Discussions** | 🟢 1h | 🟠 | ❌ | No verificable desde repo local |
| `NUEVO-01` | **README hero** con readme-aura + benchmark gráfico + GIF demo WASM | 🟡 2-3d | 🟠 | ❌ | No implementado |
| `NUEVO-07` | **Migration tools: Chroma→Vanta, LanceDB→Vanta** | 🟡 3-5d | 🟠 | ❌ | No existen scripts de migración automatizados |
| `NUEVO-08` | **Learning path estructurado** en tutorials/ (5-7 ejemplos) | 🟡 2-3d | 🟠 | ❌ | Tutorials existen (3) pero sin progresión clara |
| `NUEVO-10` | **Benchmark suite pública reproducible** | 🟡 3-5d | 🟠 | ❌ | Benchmarks internos existen, no hay publish |
| `TSK-107` | Community showcase page | 🟢 4-6h | 🟡 | ❌ | No existe |
| `—` | Good first issues (20+ tagged) | 🟢 2-4h | 🟠 | ❌ | No hay issues etiquetados |

### 🚀 Launch Campaign

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `LEG-01` | **Registrar trademark "VantaDB" (USPTO + EUIPO)** | 🟡 2-4h | 🔴 | ❌ |
| `MKT-03` | **Show HN post** | 🟢 2h | 🔴 | ❌ |
| `MKT-04` | Reddit posts (r/rust, r/MachineLearning, r/LocalLLaMA) | 🟢 2-4h | 🟠 | ❌ |
| `MKT-05` | Technical blog posts (5+ pre-launch) | 🟡 2-3d | 🟠 | ❌ |
| `MKT-10` | "AI Agent Memory" campaign | 🟡 2-3d | 🟠 | ❌ |
| `MKT-15` | **Página de benchmarks competitivos** (`/product/benchmarks`) | 🟡 2-3d | 🔴 | ❌ |
| `MKT-16` | **Publicar metodología de benchmark GraphRAG** | 🟡 1-2d | 🟡 | ❌ |
| `TSK-103` | Public benchmark site | 🟡 2-3d | 🟠 | ❌ |
| `TSK-104` | Demo agent: LangChain + Ollama + VantaDB | 🟡 1-2d | 🟠 | ❌ |
| `DEVOPS-12` | **Production PyPI signing pipeline** (OIDC + Sigstore) | 🟡 1-2d | 🟡 | ❌ |
| `DEVOPS-10` | **Firma de binarios Windows (SmartScreen)** | 🟡 2-3d | 🟢 | ❌ |

### 🌐 Conversión y SEO

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `MKT-17` | Página de comparación competitiva interactiva | 🟡 2-3d | 🟢 | ❌ |

### 🚨 Hallazgos de Docs Audit (nuevos)

> Items verificados como pendientes en docs/ pero no trackeados previamente. Referencia: `docs/bitacora.md`, `docs/reviews/FULL_CODEBASE_AUDIT_2026-07-11.md`.

| ID | Tarea | Origen | Esfuerzo | Prioridad | Estado |
|----|-------|--------|----------|-----------|--------|
| `SEC-13` | **CSP unsafe-inline en prod + HSTS + nonce system** — Sin nonce, `style-src 'unsafe-inline'`, `/metrics` endpoint público sin auth | bitacora P12, CSP2/CSP3, W6 | 🟡 1-2d | 🔴 | ❌ |
| `SEC-14` | **Evaluar migrar bincode → postcard/rkyv** — Crate no mantenido desde 2021, propuesto en STORAGE_VERSIONING.md | `docs/architecture/STORAGE_VERSIONING.md:100` | 🟡 1d | 🟠 | ❌ |
| `WEB-02` | **Corregir claims falsos en landing** — Benchmarks web (50x vs real 40x), mención "SQL support", "auto-embeddings", "cloud tiers" sin infraestructura | bitacora W1–W4 | 🟡 2-3d | 🔴 | ❌ |
| `WEB-03` | **Async WAL batching fsyncs** — Recomendado en PERFORMANCE_TUNING.md para alta throughput | `docs/operations/PERFORMANCE_TUNING.md:264` | 🟡 2-3d | 🟡 | ❌ |
| `WEB-04` | **Storage format versioning (draft→implement)** — STORAGE_VERSIONING.md Phases 1-3, sin migration path para VantaFile/HNSW/WAL | `docs/architecture/STORAGE_VERSIONING.md` | 🟠 3-5d | 🔵 | ❌ |
| `DEVOPS-13` | **Pin all workflow actions a SHA + Node 22** — 11 workflows sin SHA pinning, Node 20 deprecated | bitacora C1, plan repair campaign | 🟡 1-2d | 🟡 | ❌ |
| `DEVOPS-14` | **Extract composite action para Rust setup** — 5+ workflows duplican inline | bitacora C1 | 🟢 4h | 🟡 | ❌ |
| `DEVOPS-15` | **Mover features heavies fuera de default + consolidar deps duplicadas** — Optimización compilación workspace | `docs/Investigaciones/cargo-check-optimizacion.md` T5/T7 | 🟡 1-2d | 🟡 | ❌ |
| `TEST-11` | **Frontend tests (Vitest + Playwright)** + cross-browser WASM testing | bitacora T1/T4 | 🟡 2-3d | 🟡 | ❌ |
| `TEST-12` | **Security testing: fuzzing expand + regression/snapshot suite** — Solo parser fuzzed, sin regression gates | bitacora T2/T3 | 🟡 2-3d | 🟡 | ❌ |
| `DOC-20` | **mdBook adoption for docs site** — Docs fragmentados, sin search unificado, sin versioning | bitacora D1, D6 | 🟡 2-3d | 🟡 | ❌ |

### 🧪 Issues Técnicos Verificados (Nuevos)

> Items descubiertos durante la verificación cross-code del backlog. No estaban registrados previamente.

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `VFY-001` | **TS SDK `catch {}` silencia errores** — 4+ bloques catch vacíos | `vantadb-ts/src/vantadb.ts:176,215,249` | 🟢 2h | 🟡 | ❌ |
| `VFY-002` | **`get_nns_by_id` spawn por llamada** — Sin batching | `vantadb-ts/src/vantadb.ts:325` | 🟢 2h | 🟢 | ❌ |
| `VFY-003` | **`reindex_hnsw_from_text` riesgo OOM** — Sin batch processing | `vantadb-python/src/lib.rs:1584` | 🟡 1d | 🟡 | ❌ |
| `VFY-004` | **`flat.rs` O(n²) en filter** — Sin índice para filtros | `src/index/flat.rs:32` | 🟡 1-2d | 🟡 | ❌ |
| `VFY-005` | **TS `OperationalMetrics` 70% incompleto** — 3 de 10+ métricas mapeadas | `vantadb-ts/src/types.ts:148-168` | 🟢 4h | 🟢 | ❌ |
| `VFY-006` | **`add_node` escribe lock durante toda inserción** | `src/index/graph.rs:476-490` | 🟡 1-2d | 🟡 | ❌ |
| `VFY-007` | **`remove_node` O(n²) neighbor fixup** — Deletes costosos | `src/index/core.rs` | 🟡 1-2d | 🟢 | ❌ |
| `VFY-008` | **WAL fsync por escritura** — Write amplification | `src/storage/wal.rs` | 🟡 1-2d | 🟡 | ❌ |
| `VFY-009` | **637 inline styles no migrados a Tailwind** | `web/src/` | 🟡 3-5d | 🟢 | ❌ |
| `VFY-010` | **ACID Phase 2: Buffered write transactions** — No implementado | `src/wal.rs` | 🟡 2-3d | 🔵 | ❌ |
| `VFY-011` | **ACID Phase 3: Snapshot isolation / MVCC** | — | 🟠 3-5d | 🔵 | ❌ |
| `VFY-012` | **DEVOPS-03: musllinux target gap** — Algunos targets sin soporte | CI config | 🟢 4h | 🟢 | ❌ |

### 🔍 Hallazgos del Full Review (2026-07-13)

> Items descubiertos durante `vantadb-full-review`. Referencia: `docs/reviews/2026-07-13-full-review.md`.

### 🔍 Hallazgos del Review Deep — SDK (DRV)

> Items descubiertos durante `review-deep` del módulo `vantadb-sdk` (Wave 0). Referencia: `.opencode/skills/review-deep/`.

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-001` | **search.rs: 1085L god file** — Contiene BM25 scoring, snippet generation, hybrid fusion, RRF, debug explain. 4+ responsabilidades en un solo impl block. `#[allow(clippy::type_complexity)]` en L642 | `src/sdk/search.rs:1-1085` | 🟡 2-3d | 🟡 | ❌ |
| `DRV-002` | **put_batch duplica lógica de put()** — ~40 líneas idénticas (validación, node_id collision, timestamp, version). DRY violation | `src/sdk/api.rs:117-193` | 🟢 1d | 🟢 | ❌ |
| `DRV-003` | **purge_expired llama replace_derived_indexes por nodo** — O(n) index rebuilds en loop. Si purga 10K registros, hace 10K rebuilds | `src/sdk/api.rs:380-383` | 🟢 2h | 🟡 | ❌ |
| `DRV-004` | **list() carga ALL records a memoria antes de paginar** — `records_for_namespace()` devuelve todo. Namespace con 100K+ registros → OOM | `src/sdk/api.rs:296-315` | 🟡 1d | 🟡 | ❌ |
| `DRV-005` | **SDK sin unit tests** — No hay `#[cfg(test)]` en api.rs, search.rs, types.rs. Solo integration tests en `tests/`. Edge cases de validación sin cobertura | `src/sdk/` | 🟡 1-2d | 🟡 | ❌ |

### 🔍 Hallazgos del Review Deep — Engine (DRV)

> Items descubiertos durante `review-deep` del módulo `vantadb-engine` (Wave 0). Referencia: `.opencode/skills/review-deep/`.

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-006` | **Race condition en `delete()`: write lock dropped antes de index cleanup** — `drop(nodes)` libera `nodes.write()` L241, luego actualiza `edge_index` y `scalar_index` sin protección. Ventana donde un `insert` concurrente con mismo ID target puede interleaver, corrompiendo índices | `src/engine.rs:235-248` | 🟢 30min | 🔴 | ❌ |
| `DRV-007` | **Data race en `filter_field()`: accede `scalar_index` sin lock** — No adquiere el `nodes` RwLock. Mutaciones concurrentes (insert/update/delete) acceden a `scalar_index` bajo `nodes.write()`, pero `filter_field` no. Comportamiento indefinido | `src/engine.rs:354` | 🟢 30min | 🟡 | ❌ |
| `DRV-008` | **Duplicate scoring pipeline en `vector_search()` y `hybrid_search()`** — ~25 líneas idénticas (sort_by, truncate, collect, QueryResult build). DRY violation entre L288-305 y L399-413 | `src/engine.rs:288-305,399-413` | 🟢 1h | 🔵 | ❌ |
| `DRV-009` | **`node_count()` O(n) full scan bajo read lock** — itera todos los nodos vivos cada vez. 1M nodos → 1M iteraciones por cada `node_count()`. Sin contador cacheado | `src/engine.rs:424-426` | 🟢 1h | ⚪ | ❌ |
| `DRV-010` | **63 `unwrap()` en tests** — Todos en `#[cfg(test)]`, aceptable para test helpers. Solo documentar como deuda de estilo | `src/engine.rs:460-932` | 🟢 N/A | ℹ️ | ❌ |

### 🔍 Hallazgos del Review Deep — WAL (DRV)

> Items descubiertos durante `review-deep` del módulo `vantadb-wal` (Wave 0). Referencia: `.opencode/skills/review-deep/`.

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-011` | **Scan-forward recovery duplicado en WalWriter y WalReader** — ~40 líneas de algoritmo byte-scan idéntico para localizar el siguiente registro válido tras corrupción. DRY violation entre open_with_buffer L287-332 y next_record L593-630 | `src/wal.rs:287-332,593-630` | 🟢 2h | 🔵 | ❌ |
| `DRV-012` | **`append()` y `batch_append()` duplican lógica de sync** — Las líneas L390-396 y L430-436 son idénticas (check de sync_mode + flush_threshold). Podría extraerse a fn `maybe_sync()` | `src/wal.rs:390-396,430-436` | 🟢 30min | ⚪ | ❌ |
| `DRV-013` | **ShardedWal sin unit tests** — `src/wal_sharded.rs` no tiene `#[cfg(test)]`. 168 líneas de lógica concurrente (locks, round-robin, batch_append, rotate_all) sin cobertura directa | `src/wal_sharded.rs` | 🟡 4h | ⚪ | ❌ |
| `DRV-014` | **ShardedWal::batch_append() clona todos los records por shard** — Llama `record.clone()` en L88 para cada elemento, clonando `UnifiedNode`. Para batches grandes (>1000), overhead de alloc significativo | `src/wal_sharded.rs:85-89` | 🟢 2h | ℹ️ | ❌ |
| `DRV-015` | **`WalWriter::open_with_buffer()` función monolítica de 170L** — Mezcla apertura de archivo + validación de header + recovery scanning + truncation. 2+ responsabilidades, dificulta testeo unitario | `src/wal.rs:201-367` | 🟢 1d | ℹ️ | ❌ |

### 🔍 Hallazgos del Review Deep — Vector (DRV)

> Items descubiertos durante `review-deep` del módulo `vantadb-vector` (Wave 0). Referencia: `.opencode/skills/review-deep/`.

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-016` | **Inconsistencia de Mutex: `governor.rs` usa `std::sync::Mutex` en vez de `parking_lot`** — Todo el codebase usa parking_lot (no poison, no unwrap). governor.rs usa std Mutex con `MutexExt` que hace `.lock().unwrap()`. Si el lock se envenena, paniquea. Refactor trivial | `src/vector/governor.rs:94,111,147,160` | 🟢 30min | 🔵 | ❌ |
| `DRV-017` | **`search.rs` (416L) y `serialize.rs` (615L) sin tests unitarios** — Lógica de búsqueda HNSW con mmap zero-copy y serialización/deserialización no tienen cobertura directa. Solo tests de integración en `core.rs` | `src/index/search.rs`, `src/index/serialize.rs` | 🟡 1d | ⚪ | ❌ |
| `DRV-018` | **`refresh.rs` es stub vacío (4L)** — Archivo planeado para background refresh pero solo contiene un comment. Sin implementación, sin tests | `src/index/refresh.rs` | 🟢 N/A | ℹ️ | ❌ |
| `DRV-019` | **14 `.expect()` en hot-path SIMD loops en `distance.rs`** — En `cosine_sim_f32`, `euclidean_distance_squared_f32`, etc. Correctos (chunks_exact garantiza tamaño) pero overhead en cada chunk del loop. Preferir `unreachable_unchecked` para 0 overhead | `src/index/distance.rs:97,100,129,132,204,207,238,241,268,271,297,300,379,420` | 🟢 1h | ℹ️ | ❌ |
| `DRV-020` | **`serialize.rs:21` — `unwrap()` en producción en `serialize_to_bytes()`** — Write a Vec, no puede fallar realmente, pero rompe convención del proyecto de no usar unwrap en prod. Refactor a `.expect("Vec write")` o usar `?</* ! */` | `src/index/serialize.rs:21` | 🟢 5min | ℹ️ | ❌ |

### 🔍 Hallazgos del Review Deep — Index (DRV)

> Items descubiertos durante `review-deep` del módulo `vantadb-index` (Wave 0). Referencia: `.opencode/skills/review-deep/`.

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-021` | **`#[allow(dead_code)]` en `tokenize()` y `tokenize_with_spec()`** — 2 funciones convenience envueltas que no se usan directamente (el código llama `*_with_config`). Sin impacto, pero muerto | `src/text_index.rs:153,159` | 🟢 N/A | ℹ️ | ❌ |

### 🔍 Hallazgos del Review Deep — Governance (DRV)

> Items descubiertos durante `review-deep` del módulo `vantadb-governance` (Wave 0). Referencia: `.opencode/skills/review-deep/`.

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-022` | **`governance/` completo (1235L) gated tras feature no-default sin consumidores** — 4 módulos (admission, conflict, consistency, worker) bajo `#[cfg(feature = "governance")]` pero ningún crate externo ni módulo interno importa `AdmissionFilter`, `ConflictResolver`, etc. Feature existe como flag vacío, nunca activado. Adicionalmente depende de `sync_ext` que hace compilación inviable incluso si se activara | `src/governance/` | 🟢 30min | 🔵 | ❌ |
| `DRV-023` | **`ResourceGovernor` + `ALLOCATED_BYTES` sin callers** — `governor.rs` exporta struct y static global, pero cero referencias fuera del archivo. Ni query engine ni execution path lo usan | `src/governor.rs` | 🟢 15min | 🔵 | ❌ |
| `DRV-024` | **`memory_governor.rs` muerto con `#![allow(dead_code)]`** — Todo el archivo tiene dead_code explicitamente permitido. `pub(crate)` pero nada en el crate lo invoca | `src/memory_governor.rs:1` | 🟢 N/A | ℹ️ | ❌ |
| `DRV-025` | **TOCTOU race en `ResourceGovernor::request_allocation()`** — `ALLOCATED_BYTES.load(Ordering::Relaxed)` L41 + `fetch_add` L57 sin CAS. Dos threads concurrentes pueden pasar el check OOM y sobre-asignar `2x` del límite. El OOM guard no protege bajo concurrencia | `src/governor.rs:41-57` | 🟢 30min | 🟡 | ❌ |
| `DRV-026` | **Redundant `unwrap()` en `three_way_merge()`** — `ours_val.unwrap()` y `theirs_val.unwrap()` en L272-273 cuando el match ya garantiza `Some`. Código correcto pero redundante | `src/governance/conflict.rs:272-273` | 🟢 5min | ℹ️ | ❌ |

### 🔍 Hallazgos del Review Deep — Python SDK (DRV)

> Items descubiertos durante `review-deep` del módulo `vantadb-python` (Wave 0). PyO3 bindings, 3077L total (src/ + tests/). Referencia: `.opencode/skills/review-deep/`.

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-027` | **God module en lib.rs (1978L)** — 90 funciones en un archivo: VantaDB pyclass (~35 métodos), 19 conversores report-to-PyDict, error mapping, LRU cache, extract_vector, py_any_to_value, módulo setup. 6 pyclasses exportadas. Todos los helpers de conversión (`*_to_pydict`) son funciones libres en el mismo archivo — candidatos a módulo separado `src/convert.rs` | `vantadb-python/src/lib.rs` | 🟡 1d | 🟡 | ❌ |
| `DRV-028` | **Hand-rolled LRU cache con O(n) por operación** — `LruCache` usa `Vec<String>` para tracking de orden (L38-88). Cada `get()` y `put()` recorre el vector con `.position()` (O(n)). Para capacity=64 es ~32 comparaciones promedio. `indexmap` o `linked_hash_map` darían O(1). `ponytail:` — funciona a escala actual, reemplazar si throughput requiere | `vantadb-python/src/lib.rs:38-88` | 🟢 30min | ⚪ | ❌ |
| `DRV-029` | **Cache-key overhead en py_dict_to_metadata** — Construye string serializado + sorted de todo el dict (L609-635) solo para verificar cache hit. Para dicts ≤4 entradas, la serialización + sort probablemente cuesta más que construir el BTreeMap directamente. Hit solo en llamadas subsecuentes con contenido idéntico | `vantadb-python/src/lib.rs:602-662` | 🟢 15min | ℹ️ | ❌ |
| `DRV-030` | **19 conversores report-to-PyDict duplicados (~280L)** — Funciones como `operational_metrics_to_pydict`, `search_explanation_to_pydict`, etc. (L306-599) iteran campos manualmente uno a uno con `dict.set_item()`. Boilerplate mecánico refactorizable vía macro con `#[derive(IntoPyDict)]` | `vantadb-python/src/lib.rs:306-599` | 🟡 1d | ℹ️ | ❌ |
| `DRV-031` | **Comentario doc duplicado** — `/// Put or update a namespace-scoped persistent memory record.` aparece dos veces (L1140-1142) | `vantadb-python/src/lib.rs:1140-1142` | 🟢 2min | ℹ️ | ❌ |
| `DRV-032` | **4 métodos con #[allow(clippy::too_many_arguments)]** — `put_batch` (8 params), `put_batch_raw` (6), `put` (6), `explain_memory_search` (6). `put_batch` además implementa dos API (tuple-list legacy + keyword moderna) en 130L (L835-966) duplicando lógica batch | `vantadb-python/src/lib.rs:976,1143,1234,1652` | 🟢 2h | ℹ️ | ❌ |
| `DRV-033` | **✅ Sin unsafe, unwrap ni expect en producción** — 0 `unsafe`, 0 `unwrap()`, 0 `.expect()` en producción. 34/34 `py.detach()` envuelven GIL release correctamente. Navegación segura del GIL | `vantadb-python/src/lib.rs` | 🟢 N/A | ✅ | ❌ |

### 🔍 Hallazgos del Review Deep — TypeScript SDK (DRV)

> Items descubiertos durante `review-deep` del módulo `vantadb-ts` (Wave 0). WASM-powered TypeScript SDK, 4 src files + 6 test files (~92KB). Referencia: `.opencode/skills/review-deep/`.

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-034` | **30+ try-catch blocks repetidos siguiendo el mismo patrón** — Cada método público sigue el molde `_assertOpen() → try { this.inner.X() } catch { wrapWasmError(e, "X") }`. Defensiva pero repetitiva: 30 instancias de lógica idéntica que podría abstraerse via decorador/función wrapper. `ponytail:` — funciona, refactor si la tax de catch crece | `vantadb-ts/src/vantadb.ts` (todo el archivo) | 🟡 1d | ⚪ | ❌ |
| `DRV-035` | **Type mismatch: metadata usa formato shorthand en tests vs tipo definido** — Los tests usan `metadata: { source: { String: "test" } }` (ej: L355 vanta.test.ts, L210 hardening.test.ts), pero `VantaValue` define formato `{ type: "String", value: "test" }`. Los tests pasan (solo verifican `r.payload`, no `r.metadata`), y `tsconfig` excluye `__tests__/` del typecheck. Bug dormido: metadatos no se serializan correctamente a través del bridge WASM | `vantadb-ts/src/__tests__/*.test.ts` + `src/types.ts:1-10` | 🟢 30min | 🟡 | ❌ |
| `DRV-036` | **`_mapRecord` valida 3 campos pero retorna `as MemoryRecord` sin validar el resto** — Verifica namespace, key, payload son strings (L25-51) pero no valida metadata, vector, timestamps, node_id del objeto devuelto por la capa WASM. `as MemoryRecord` omite cualquier verificación en los campos restantes | `vantadb-ts/src/vantadb.ts:25-52` | 🟢 1h | ⚪ | ❌ |
| `DRV-037` | **`types.test.ts` usa types incorrectos que tsc no detecta** — `created_at_ms: 1000` (number) vs type `string`, `node_id: 42` (number) vs type `string`. Pasa porque `tsconfig` excluye `__tests__/`. Si se incluyeran tests en el typecheck, romperían compilación | `vantadb-ts/src/__tests__/types.test.ts:11-14` | 🟢 15min | ⚪ | ❌ |
| `DRV-038` | **TypeScript numeric fields tipados como `string` inconsistentes con otros SDKs** — `created_at_ms`, `updated_at_ms`, `version`, `node_id` son `string` en TS pero `u64` en Rust/Python. El bridge WASM convierte a string para JSON, pero obliga a conversión `Number()` en el consumidor TS | `vantadb-ts/src/types.ts:28-31` | 🟢 1h | ℹ️ | ❌ |
| `DRV-039` | **No ESLint config presente** — `vantadb-ts/` no tiene `.eslintrc.*`. Sin linting estático de TS. Tests usan `any` (dx04.test.ts, load.test.ts) sin detección | `vantadb-ts/` | 🟢 30min | ℹ️ | ❌ |

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `REV-001` | **CI: Rust fails on main — TSan ABI mismatch** — `-Zsanitizer=thread` incompatible con toolchain 1.94.1 | `.github/workflows/ci-rust-10.yml` → H05-ERROR-001 | 🟢 2h | 🔴 | ❌ |
| `REV-002` | **CI: Web fails on main — 21 lint issues** — 14 ESLint errors + 7 warnings rompen build | `.github/workflows/ci-web-11.yml` → H05-ERROR-002 | 🟢 2h | 🔴 | ❌ |
| `REV-003` | **No code coverage measurement** — CII Silver requiere ≥80%, sin herramienta configurada | CI config → H05-MISSING-001 | 🟡 1d | 🔴 | ✅ |
| `REV-004` | **`tantivy` rlib not found** — Test builds de `vantadb-openai` fallan por dependencia faltante | `vantadb-openai/Cargo.toml` → H08-ARCH-001 | 🟡 1d | 🟡 | ✅ |
| `REV-005` | **14 ESLint errors en web frontend** — 6x `no-explicit-any`, 8x prettier en `demo.lazy.tsx` + `why-vantadb.tsx` | `web/src/routes/demo.lazy.tsx`, `web/src/routes/why-vantadb.tsx` → H03-CODE-001 | 🟢 1h | 🟡 | ✅ |
| `REV-006` | **No workspace-level clippy en CI** — Solo `-p vantadb`, adapters excluded | `.github/workflows/ci-rust-10.yml` → H05-MISSING-002 | 🟢 2h | 🟡 | ✅ |
| `REV-007` | **`reducedMotion` missing from useEffect deps** — Stale closure risk en 3 componentes | `NbMonolith.tsx:61`, `NbVectorNebula.tsx:239`, `__root.tsx:181` → H03-CODE-002 | 🟢 30min | 🟡 | ✅ |
| `REV-008` | **Node 20 actions deprecated** — `actions/checkout` y `setup-node` usan Node 20, runner usa Node 24 | `.github/workflows/*.yml` → H05-CODE-005 | 🟢 30min | 🟡 | ✅ |
| `REV-009` | **19 workspace crates compilation overhead** — Adaptadores (10 crates) dependen de pyo3, rebuild en cascada | `Cargo.toml` → H08-ARCH-002 | 🟡 2-3d | 🟡 | ✅ |
| `REV-010` | **`serialization.rs` god module 1827L** — Candidato a split, documentado pero no ejecutado | `src/sdk/serialization/mod.rs` → H08-PATTERN-001 | 🟡 1d | 🟡 | ❌ |
| `REV-011` | **`insert_hnsw` monolithic 177L** — Función sin descomponer en sub-operaciones | `src/index/graph.rs` → H08-PATTERN-002 | 🟡 4h | 🟡 | ✅ |
| `REV-012` | **HNSW `insert_lock` contention** — Micro-batching implementado (P1), posible bottleneck bajo alta concurrencia | `src/index/graph.rs` → H08-ALGO-001 | 🟡 1-2d | 🟡 | ❌ |
| `REV-013` | **`spin 0.9.8` yanked dependency** — Usado transitivamente vía fjall/flume, monitoreado | `deny.toml` → H08-LOGIC-001 | 🟢 1h | 🟡 | ❌ |
| `REV-014` | **24 stale dependabot branches** — No auto-delete después de merge | `origin/dependabot/*` → H05-DIRECTION-001 | 🟢 30min | 🔵 | ❌ |
| `REV-015` | **6 `any` types sin justificación** — `demo.lazy.tsx` usa `any` sin `eslint-disable` ni type alias | `web/src/routes/demo.lazy.tsx` → H03-CLARITY-001 | 🟢 1h | 🟡 | ✅ |
| `REV-016` | **`vantadb-enterprise` abstracción prematura** — Crate existe pero features no definidos públicamente | `vantadb-enterprise/` → H08-ARCH-003 | 🟢 2h | 🔵 | ✅ |
| `REV-017` | **`why-vantadb.tsx` prettier error** — Trailing newline rompe formateo | `web/src/routes/why-vantadb.tsx:43` → H03-CODE-003 | 🟢 5min | 🟢 | ❌ |
| `REV-018` | **`NbToast.tsx` react-refresh warning** — Archivo exporta más que solo componentes | `web/src/components/nb/NbToast.tsx:15` → H03-CODE-004 | 🟢 5min | 🟢 | ❌ |

### WASM & Performance

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `NUEVO-11` | **WASM IndexedDB fallback** | 🟡 2-3d | 🟡 | ❌ |
| `NUEVO-12` | **WASM multi-tab coordination** (Web Locks + BroadcastChannel) | 🟡 2-3d | 🟡 | ❌ |
| `NUEVO-13` | **HNSW auto-tuning PID loop** (ef_search dinámico) | 🟡 3-5d | 🟡 | ❌ |
| `NUEVO-14` | **WASM bundle size <500KB gzip** | 🟡 1-2d | 🟡 | ❌ |
| `NUEVO-15` | **Code coverage report en CI** + upload | 🟢 1d | 🟡 | ❌ |
| `NUEVO-19` | **Mover SourceDesign/ fuera de web/src/** | 🟢 1h | 🔵 | ❌ |

---

## TIER 2 — 🟡 Launch Campaign

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `CLI-01` | **CLI polish: backup/restore/doctor/stats/inspect/REPL/TUI** — Phase 4.E del roadmap, no implementado | 🟡 3-5d | 🟡 | ❌ |
| `DEVOPS-HOMEBREW` | **Homebrew formula** — Phase 4.F, no existe | 🟢 4h | 🟡 | ❌ |
| `DEVOPS-PY313` | **Python 3.13 wheels en CI matrix** — Phase 4.F, no configurado | 🟢 2h | 🟡 | ❌ |
| `DEVEX-DEMO` | **Demo app (Rust + Python)** — Phase 4.G, no existe | 🟡 2-3d | 🟡 | ❌ |
| `DEVEX-EXAMPLES` | **Rust examples en `docs/examples/`** — Phase 4.G, no existe | 🟢 4-6h | 🟡 | ❌ |
| `NUEVO-16` | **Product Quantization (PQ) 96x** — compresión para datasets >RAM | Alto | 🔵 | ❌ |
| `NUEVO-17` | **Segment LSM-style** — hot/warm/cold tiers | Muy alto | 🔵 | ❌ |
| `NUEVO-18` | **Sparse vectors nativos** — hybrid search real | Alto | 🔵 | ❌ |
| `NUEVO-20` | **Server Docker image** | 🟡 1-2d | 🔵 | ❌ |
| `NUEVO-21` | **Vectara competitive research** | 🟢 2-4h | ⬜ | ❌ |
| `TSK-107b` | Audit logging enterprise (JSONL, timestamp + op) | 🟡 2-3d | 🟡 | ❌ |
| `ENT-04` | Connection pooling + circuit breaker | 🟡 2-3d | 🟡 | ❌ |
| `BIZ-01` | Enterprise crate (encryption, audit, RBAC, replication) | 🟡 3-5d | 🟡 | ⏳ |

---

## TIER 3 — 🟢 Comunidad y Operaciones

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `COM-02` | **Configurar Discord: reaction roles, autorole, logging, welcome DM, onboarding** — ~10 items operacionales de `docs/discord/todo.md` | 🟡 2-3d | 🟢 | ❌ |
| `COM-03` | **Discord: AutoMod (spam, mass-mention, invites), stickers/emojis, forums seed** | 🟢 4-6h | 🟢 | ❌ |
| `COM-04` | **Discord: ticketing system, stage channel, Server Discovery, Canny.io roadmap, cross-promotion** | 🟢 4-6h | 🟢 | ❌ |

---

## TIER 4 — 🔵 Post-Lanzamiento

| ID | Tarea | Esfuerzo | Prioridad | Estado |
|----|-------|----------|-----------|--------|
| `—` | Publicar 8 workspace members en crates.io | 🟡 2-3d | 🟡 | ❌ |
| `WEB-001` | **Re-add interactive WASM demo page** — Restaurar demo después de publicar `@vantadb/wasm` en npm | 🟢 30min | 🟡 | ❌ |

---

## ✅ Resumen de Verificación (Jul 13, 2026)

### Documentos archivados (13 → `docs/archive/`)

| Archivo | Razón |
|---------|-------|
| `reviews/agent-01-local-AK.md` | Raw agent output, consolidado en FINAL-REVIEW |
| `reviews/agent-02-local-LZ.md` | Raw agent output, consolidado en FINAL-REVIEW |
| `reviews/agent-03-global-agents.md` | Raw agent output, consolidado en FINAL-REVIEW |
| `reviews/agent-04-global-claude.md` | Raw agent output, consolidado en FINAL-REVIEW |
| `reviews/agent-05-internet-research.md` | Raw agent output, consolidado en FINAL-REVIEW |
| `reviews/EXECUTIVE_TECHNICAL_AUDIT.md` | Superseded por audits Jul 11/13 |
| `reviews/AUDITORIA_COMPLETA_VantaDB_WEB.md` | Web-only, superseded |
| `reviews/FULL_CODEBASE_AUDIT_2026-07-09.md` | Superseded por 2026-07-11 |
| `reviews/web-audit-report.md` | Superseded |
| `research/DOCS_TOOLS_RESEARCH.md` | Cold research, no tool adopted |
| `research/SQL_ANALYSIS.md` | Decisión tomada (no SQL), sin acción pendiente |
| `research/COGNEE_EVALUATION.md` | Pure research, cero implementación |
| `research/DOCS_AUDIT_REPORT.md` | Issues tracked en bitácora/backlog |

### Claims verificados contra código: 100% precisos

De ~150 claims de estado en el backlog anterior, todos fueron verificados contra el código real usando `codegraph_explore`, `grep`, y lectura directa. Ver `docs/reviews/FULL_CODEBASE_AUDIT_2026-07-11.md` y `docs/reviews/2026-07-13-full-review.md` para el detalle.

### Documentos de investigación aún vigentes

| Documento | Estado | Nota |
|-----------|--------|------|
| `research/ACID_TRANSACTIONS.md` | ⚠️ Parcial | Phase 1 implementada; Phase 2-3 no; WAL shipping + PITR existen |
| `research/SIGNED_RELEASES.md` | ⚠️ Parcial | Attestations + Windows builds OK; GPG/sigstore no |
| `research/VantaDB_RESEARCH_UNIFIED.md` | ✅ Vigente | Mejor referencia de arquitectura |
| `research/VantaDB_RESEARCH_VALIDADO.md` | ✅ Vigente | Meta-validación precisa |
| `research/VantaDB_ANALISIS_COMPLETO.md` | ⚠️ Parcial | Version sync ya resuelto |

---

## 📈 Timeline Consolidado

```
Jul 14-18  TIER 0 (🔴 6+ items):
               ─ INT-01/02: LangChain + LlamaIndex → PyPI
               ─ DEVOPS-05: Pipeline CI unificado
               ─ REL-02: vantadb-ts → npm
               ─ MKT-13: Link WASM demo en hero ⏳
               ─ **REV-001/002: Fix CI (TSan + ESLint)**
               ─ **SEC-13: CSP unsafe-inline + HSTS + nonce** ← docs-audit
               ─ **WEB-02: Fix false claims landing** ← docs-audit
Jul 18-25  TIER 1 (🟠 35+ items):
               ─ Docs: MKT-14 (case studies), TSK-106 (Discussions)
               ─ NUEVO-01/07/08/10: README, migrations, learning, benchmarks
               ─ Launch: LEG-01, MKT-03/04/05/10/15/16
               ─ Code health: VFY-001→012, **REV-003→018**, **DRV-001→005**, **DRV-022→026**
               ─ WASM: NUEVO-11→15
               ─ Docs-audit: SEC-14, WEB-03/04, DEVOPS-13/14/15, TEST-11/12, DOC-20
Ago+       TIER 2-4:
               ─ CLI-01, DEVOPS-HOMEBREW, DEVOPS-PY313, DEVEX-DEMO, DEVEX-EXAMPLES
               ─ NUEVO-16/17/18: PQ, LSM, sparse vectors
               ─ Enterprise: ENT-04, BIZ-01
               ─ COM-02/03/04: Discord setup
               ─ Publishing: crates.io, WEB-001
```

---

## ✅ Definition of Ready (DoR)

- [ ] ID único asignado
- [ ] Prioridad definida (🔴🟠🟡🟢🔵⬜)
- [ ] Archivos involucrados conocidos
- [ ] Esfuerzo estimado
- [ ] Verificado contra código real (no asumido)

## ✅ Definition of Done (DoD)

- [ ] Código compila (`cargo check` / `tsc --noEmit`)
- [ ] Tests pasan (`cargo test` / `vitest run`)
- [ ] Linters pasan (`cargo clippy` / `eslint`)
- [ ] Docs actualizados si aplica
- [ ] Tarea movida a `progreso/README.md`
- [ ] Changelog actualizado si es cambio visible al usuario

---
**Fuente de REV-001→018:** `docs/reviews/2026-07-13-full-review.md` — generado por `vantadb-full-review` skill.
