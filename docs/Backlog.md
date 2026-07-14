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
> **Total open items:** 91 (66 previos + 22 del docs-audit Jul 13 + 3 del full review Jul 13)
> **Origen docs-audit:** `docs/strategy/ROADMAP.md`, `docs/bitacora.md`, `docs/reviews/FULL_CODEBASE_AUDIT_2026-07-11.md`, `docs/reviews/analisis_proyecto.md`, `docs/operations/PERFORMANCE_TUNING.md`, `docs/operations/REPO_CHECKLIST.md`, `docs/architecture/STORAGE_VERSIONING.md`, `docs/plans/2026-07-13-workflow-repair-campaign.md`, `docs/Investigaciones/cargo-check-optimizacion.md`, `docs/discord/todo.md`

---

## TIER 0 — 🔴 Bloqueantes de Release

> Items que bloquean cualquier release seguro.

### 📦 Publicación de Integraciones

| ID | Tarea | Esfuerzo | Prioridad | Estado | Verificación |
|----|-------|----------|-----------|--------|-------------|
| `INT-01` | **LangChain adapter → PyPI** | 🟡 1-2d | 🔴 | ✅ | Código existe, CI configurado (`release-adapters-62.yml`), 5/5 tests pasan. Push tag `adapters-v0.3.0` para publicar. |
| `INT-02` | **LlamaIndex adapter → PyPI** | 🟡 1-2d | 🔴 | ✅ | Código existe, CI configurado (`release-adapters-62.yml`), 5/5 tests pasan. Push tag `adapters-v0.3.0` para publicar. |
| `DEVOPS-05` | Pipeline CI unificado para publicar los 9 adapters a PyPI | 🟡 1-2d | 🔴 | ✅ | `release-adapters-62.yml` existente: test → build → TestPyPI (dispatch) → PyPI (tag `adapters-v*`). Los 9 adapters en `integrations/` build correctos. |
| `REL-02` | **Publicar `vantadb-ts` en npm** (WASM build) | 🟡 1-2d | 🔴 | ⏳ | 3 cambios aplicados: (1) `impl_text_index.rs` visibility fix (`fn` → `pub(crate)` en 2 métodos), (2) `wasm-opt = false` en `vantadb-wasm/Cargo.toml` (toolchain local no soporta bulk-memory), (3) CI `release-npm-61.yml` fix: `ts-v*` tag ahora ejecuta `publish-wasm`. Build WASM ✅, Build TS ✅, npm dry-run ✅. Tests: ⚠️ 80/219 fail (pre-existing WASM panics `unreachable!` en Node.js, pasan 113, 26 skip — bug no relacionado a REL-02). npm names `vantadb` + `vantadb-wasm` disponibles. |

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
| `SEC-13` | **CSP unsafe-inline en prod + HSTS + nonce system** — Sin nonce, `style-src 'unsafe-inline'`, `/metrics` endpoint público sin auth | bitacora P12, CSP2/CSP3, W6 | 🟡 1-2d | 🔴 | ✅ |
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

> Items descubiertos durante `vantadb-full-review`. Referencia: `docs/reviews/PROJECT_FULL_REVIEW_2026-07-13.md`.

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `RC1-RC4` | **23 `.expect("RwLock/Mutex poisoned")` en governance/ + vector/governor.rs** — Ya resuelto: helpers `RwLockExt`/`MutexExt` existen en `src/sync_ext.rs`, governance/ ya los usa. No requiere acción | `src/sync_ext.rs`, `src/governance/*`, `src/vector/governor.rs` | 🟢 N/A | ✅ | ✅ |
| `RC5` | **Mejorar mensaje de `Aes256Gcm::new_from_slice().expect()`** — Mensaje genérico, incluir razón técnica en panic | `src/crypto.rs:104` | 🟢 15min | 🟡 | ❌ |
| `RC6` | **Evaluar propagar `CryptoError` desde `encrypt()` vs mantener expect** — 45+ call sites vía `EncryptionStream`. Decidir si vale la ruptura | `src/crypto.rs:126` | 🟡 1d | 🟡 | ❌ |
| `RC7` | **`.expect("GovernorConfig build failed")`** — Startup path fatal, abort intencional. **Ponytail: keep as-is** | `src/cli_server.rs:139` | 🟢 N/A | ℹ️ | — |
| `RC8` | **`auth_middleware` .expect("keys")** — Middleware debe devolver 401 en vez de panic cuando el invariante se viola | `src/cli_server.rs:758` | 🟢 2h | 🟡 | ❌ |
| `RC9` | **SystemTime::duration_since** — Ya tiene `.unwrap_or_default()` en L32. **Ya resuelto** | `src/binary_header.rs:32` | 🟢 N/A | ✅ | ✅ |
| `RC10` | **`.expect("reqwest blocking client")`** — Startup-only, fatal abort aceptable. **Ponytail: keep as-is** | `src/wal_shipping.rs:78` | 🟢 N/A | ℹ️ | — |
| `RC11` | **`ClientEngine::default().expect()`** — Sin engine = sin bindings, abort intencional. **Ponytail: keep as-is** | `src/python.rs:21` | 🟢 N/A | ℹ️ | — |

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
| `DRV-040` | **`unsafe` en simd.rs sin `// SAFETY:` comment** — L34-77: bloque `unsafe` con `v128_load` requiere punteros alineados a 16 bytes. `Vec<f32>` garantiza alineación, pero sin SAFETY docs el invariante no es verificable en code review. 1 bloque unsafe (L34) sin documentación | `vantadb-wasm/src/simd.rs:34-77` | 🟢 30min | 🟡 | ❌ |
| `DRV-041` | **`worker.rs` Promise constructor con inline JS string** — L201-209: `js_sys::Function::new_no_args` con string JS crudo + `arguments[0]`/`arguments[1]`. El callback `_reject` nunca se invoca: el Promise cuelga para siempre si el mensaje nunca llega. Response parsing vía `serde_json::from_str` (L229) agrega round-trip JSON innecesario vs `serde_wasm_bindgen::from_value` | `vantadb-wasm/src/worker.rs:201-229` | 🟢 2h | 🔵 | ❌ |
| `DRV-042` | **Test duplicación entre `lib.rs` mod tests y `tests/wasm_tests.rs`** — ~15 tests idénticos (put/get, delete, batch, capabilities, flush/compact, list_namespaces, search_without_results, large_metadata, concurrent_put_get) repartidos en 207L de tests en `lib.rs` + 751L en `wasm_tests.rs`. Mantenimiento duplicado | `vantadb-wasm/src/lib.rs:901-1107`, `vantadb-wasm/tests/wasm_tests.rs` | 🟢 1h | ⚪ | ❌ |
| `DRV-043` | **Core crate compilation errors bloquean `cargo check -p vantadb-wasm`** — `ensure_text_index_current_with` y `adjust_text_index_state_after_replace` son privados en `impl_text_index.rs` pero llamados desde `impl_index.rs`. 2 errores E0624. No afecta al wasm module per se, pero impide CI check del wasm crate | `vantadb/src/sdk/serialization/impl_index.rs:20,210` | 🟢 30min | 🟡 | ❌ |

### 🔍 Hallazgos del Review Deep — Server Binary (DRV)

> Items descubiertos durante `review-deep` del módulo `vantadb-server` (Wave 0). Binary wrapper thin crate (78L source, 4 files) — toda la lógica del servidor HTTP vive en `vantadb::cli_server`. Referencia: `.opencode/skills/review-deep/`.

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-044` | **MCP shutdown via `std::process::exit(0)` antes de que `run_stdio_server` reciba señal de parada** — `main.rs:46-55`: SIGTERM handler flushes storage y llama `exit(0)`, matando el proceso inmediatamente. `vantadb_mcp::run_stdio_server(storage)` (L57) nunca recibe señal de shutdown; in-flight JSON-RPC requests en stdin se pierden sin respuesta. Fix: usar `CancellationToken` para señalizar `run_stdio_server` primero, luego `return` de main en vez de `exit(0)` | `vantadb-server/src/main.rs:46-57` | 🟢 2h | 🔵 | ❌ |
| `DRV-045` | **Test setup factory duplicado en 3 test files** — `tests/server.rs::build_context`, `tests/e2e.rs::build_e2e_context`, `tests/benchmarks.rs::setup_bench` implementan el mismo patrón (tempdir + StorageEngine + ServerState) con ~15L cada uno. Algunos retornan `TestContext` struct, otros tuplas `(TempDir, Arc<ServerState>)`. Refactor a helper compartido reduciría ~40L de duplicación | `vantadb-server/tests/server.rs:26-39, tests/e2e.rs:52-66, tests/benchmarks.rs:23-55` | 🟢 30min | ⚪ | ❌ |

### 🔍 Hallazgos del Review Deep — MCP Protocol Interface (DRV)

> Items descubiertos durante `review-deep` del módulo `vantadb-mcp` (Wave 0). Custom MCP protocol sobre stdio sin librería externa. 1 source file `src/lib.rs` (1309L) + 1 test file `tests/mcp_tests.rs` (956L). `cargo fmt` OK; `cargo check` bloqueado por 2 errores de visibilidad en `vantadb::sdk::serialization` (DRV-043). Web research no disponible (timeouts). Referencia: `.opencode/skills/review-deep/`.

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-046` | **Blocking stdio I/O en tokio runtime impide graceful shutdown** — `run_stdio_server` (async fn) ejecuta `stdin.lock().lines()` sincrónicamente en el worker de tokio, bloqueando la ejecución de otras tareas async (incluyendo el handler de SIGINT). Ctrl+C termina el proceso via señal OS sin ejecutar shutdown graceful ni responder in-flight JSON-RPC requests. Fix: `spawn_blocking` para el loop de I/O o `tokio::io::AsyncBufReadExt::lines()` | `vantadb-mcp/src/lib.rs:320-384` | 🟢 2h | 🟡 | ❌ |
| `DRV-047` | **Hardcoded validation limits en handle_resources_read** — Líneas 549,553,575 usan literales `256` y `512` en vez de `config.max_namespace_length` / `config.max_key_length`. Consistencia: otros handlers usan `config.*`. Bajo impacto porque los default coinciden | `vantadb-mcp/src/lib.rs:549,553,575` | 🟢 15min | ⚪ | ❌ |
| `DRV-048` | **JSON-RPC 2.0 spec — versión no-2.0 descartada silenciosamente** — L359-363: si `req.jsonrpc != "2.0"`, se loggea warning y se hace `continue` sin enviar respuesta de error. La especificación (§7) dice que el servidor DEBE responder con un error de tipo invalid-request (-32600). El cliente nunca sabe que su request fue rechazado | `vantadb-mcp/src/lib.rs:359-363` | 🟢 30min | 🔵 | ❌ |
| `DRV-049` | **collection_delete no atómico** — Fetch all records via `collect_all_records`, luego delete one-by-one. Si el proceso crashea a mitad, el namespace queda parcialmente borrado. Sin transacción ni batch delete | `vantadb-mcp/src/lib.rs:1269-1305` | 🟢 1h | 🔵 | ❌ |
| `DRV-050` | **inject_context construye LISP query via string interpolation** — Naive escaping (`content.replace('\\', "\\\\").replace('"', "\\\"")`) antes de interpolación en query LISP via `format!`. No escapa paréntesis, newlines u otros metacaracteres LISP. Potencial injection vector si content no es confiable | `vantadb-mcp/src/lib.rs:1154-1187` | 🟢 1h | 🟡 | ❌ |
| `DRV-051` | **search_semantic N+1 query pattern** — Por cada hit de HNSW, llama `embedded.get_node()` individualmente (L1114-1122). Para top_k=1000, 1000 queries separadas. Optimización: batch get | `vantadb-mcp/src/lib.rs:1114-1122` | 🟢 1h | ⚪ | ❌ |
| `DRV-052` | **McpMetrics trackeadas pero nunca reportadas** — Struct `McpMetrics` (requests_total, errors_total, active_requests) solo se loggea una vez en shutdown. Sin endpoint `/metrics`, sin log periódico. Datos operacionales no visibles en runtime | `vantadb-mcp/src/lib.rs:161-166,386-390` | 🟢 1h | ℹ️ | ❌ |
| `DRV-053` | ❌ **DESCARTADO** — Duplicado de DRV-047 | — | — | — | — |
| `DRV-054` | **read_axioms hardcoded como JSON literal** — 4 axioms definidos inline en L1191-1198 como array JSON hardcoded. Si los axioms se actualizan en el metadata module/database, la copia MCP deriva. DRV: leer del metadata module o storage | `vantadb-mcp/src/lib.rs:1190-1198` | 🟢 30min | 🔵 | ❌ |
| `DRV-055` | **Test test_mcp_invalid_json testea serde_json no MCP** — L697-701: `serde_json::from_str::<Value>(malformed)` verifica que serde_json rechace JSON inválido. Esto testea la librería third-party, no la lógica del MCP server. La porción McpError y handle_tools_call(None) del test es válida | `vantadb-mcp/tests/mcp_tests.rs:697-721` | 🟢 15min | ⚪ | ❌ |
| `DRV-056` | **stdout write errors silenciosamente ignorados** — `write_json` y el main loop usan `let _ = writeln!(...)` y `let _ = stdout.flush()`, ignorando errores de I/O. Si stdout se cierra (proceso padre termina), los errores se tragan sin feedback | `vantadb-mcp/src/lib.rs:394-399,378-383` | 🟢 30min | ⚪ | ❌ |

### 🔍 Hallazgos del Review Deep — OpenAI Adapter (DRV)

> Items descubiertos durante `review-deep` del módulo `vantadb-openai` (Wave 0, DEPTH=quick). Thin PyO3 wrapper crate (10L lib.rs + 170L python.rs + 32L tests). 1 Python class `VantaDBOpenAI` con 4 métodos. `cargo fmt` OK; `cargo check` bloqueado por 2 errores de visibilidad en `vantadb::sdk::serialization` (DRV-043). Referencia: `.opencode/skills/review-deep/`.

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-057` | **OpenAI client recreado en cada llamada `embed()`** — Sin caching: `openai.OpenAI(api_key=...)` se instancia en cada llamada (L63-69). El cliente interno maneja connection pooling + TLS; recrearlo por request evita reuso de conexión y añade handshake TLS por llamada. Fix: cachear `Py<PyAny>` del cliente en el struct | `vantadb-openai/src/python.rs:63-69` | 🟢 1h | 🔵 | ❌ |
| `DRV-058` | **Metadata no-string values silenciosamente ignorados** — L149-155: `v.extract::<String>()` descarta bool/int/float. `if let (Ok(key), Ok(val))` silencia el error. Usuario pasa `metadata={"count": 5}` y el valor desaparece sin warning | `vantadb-openai/src/python.rs:149-155` | 🟢 30min | 🔵 | ❌ |
| `DRV-059` | **RwLock<String> namespace con overhead concurrente innecesario** — `RwLock` en L39 sugiere mutabilidad, pero nunca se escribe (0 `.write()` calls). Únicas operaciones: 2 `.read().unwrap().clone()`. Podría ser `String` plano, ahorrando 2 allocs + lock acquisition por operación | `vantadb-openai/src/python.rs:39,109,142` | 🟢 15min | ⚪ | ❌ |
| `DRV-060` | **Sin método para cambiar namespace runtime** — RwLock sugiere que namespace sería mutable, pero no hay setter expuesto. YAGNI candidate o feature incompleta | `vantadb-openai/src/python.rs:43-163` | 🟢 30min | ℹ️ | ❌ |
| `DRV-061` | **Test coverage mínima: 5 tests/32L** — Sin tests para: error de API key inválida, network timeout, results vacíos, metadatos edge cases, delete/update operations. Solo happy path | `vantadb-openai/tests/test_openai.py:1-32` | 🟢 1h | ⚪ | ❌ |

### 🔍 Hallazgos del Review Deep — Ollama Adapter (DRV)

> Items descubiertos durante `review-deep` del módulo `vantadb-ollama` (Wave 0, DEPTH=quick). Thin PyO3 wrapper crate (10L lib.rs + 161L python.rs + 32L tests). 1 Python class `VantaDBOllama` con 4 métodos. `cargo fmt` OK; `cargo check` bloqueado por 2 errores de visibilidad en `vantadb::sdk::serialization` (DRV-043). Comparte estructura y patrones con `vantadb-openai`. Referencia: `.opencode/skills/review-deep/`.

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-062` | **Ollama client recreado en cada llamada `embed()`** — Sin caching: `ollama.Client(host=...)` se instancia en cada llamada (L63-70). El cliente interno maneja connection pooling; recrearlo evita reuso de conexión. Fix: cachear `Py<PyAny>` del cliente en el struct | `vantadb-ollama/src/python.rs:63-70` | 🟢 1h | 🔵 | ❌ |
| `DRV-063` | **Metadata no-string values silenciosamente ignorados** — L141: `v.extract::<String>()` descarta bool/int/float igual que en DRV-058. Bug duplicado por copy-paste del adapter openai | `vantadb-ollama/src/python.rs:139-145` | 🟢 30min | 🔵 | ❌ |
| `DRV-064` | **embed() llama API secuencialmente por texto** — L73-90: cada texto hace una llamada RPC individual. Ollama soporta `client.embed(model=..., input=[...])` para batch embedding. N textos = N RPCs vs 1 batch | `vantadb-ollama/src/python.rs:73-90` | 🟢 1h | ⚪ | ❌ |
| `DRV-065` | **RwLock<String> namespace con overhead concurrente innecesario** — Ídem DRV-059. Nunca escrito, solo 2 `.read().unwrap().clone()`. Podría ser `String` plano | `vantadb-ollama/src/python.rs:39,100,133` | 🟢 15min | ⚪ | ❌ |
| `DRV-066` | **Sin método para cambiar namespace runtime** — Ídem DRV-060. YAGNI candidate o feature incompleta | `vantadb-ollama/src/python.rs:43-154` | 🟢 30min | ℹ️ | ❌ |
| `DRV-067` | **Test coverage mínima: 5 tests/32L** — Ídem DRV-061. Sin tests para error paths, delete, edge cases | `vantadb-ollama/tests/test_ollama.py:1-32` | 🟢 1h | ⚪ | ❌ |

### 🔍 Hallazgos del Review Deep — LiteLLM Adapter (DRV)

> Items descubiertos durante `review-deep` del módulo `vantadb-litellm` (Wave 0, DEPTH=quick). Thin PyO3 wrapper crate (156L python.rs + 31L tests). 1 Python class `VantaDBLiteLLM`. `cargo fmt` OK. **Regresión: no libera GIL en search/store, a diferencia de openai/ollama**. Referencia: `.opencode/skills/review-deep/`.

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-068` | **GIL no liberado en search()** — `search()` recibe `py: Python` (L96) pero no usa `py.detach()` para la búsqueda vectorial (L111). openai/ollama sí lo hacen. Bloquea GIL durante `engine.search()` impidiendo ejecución concurrente de Python threads | `vantadb-litellm/src/python.rs:94-122` | 🟢 15min | 🔵 | ❌ |
| `DRV-069` | **store() sin parámetro py — no puede liberar GIL** — `fn store(&self, text: &str, ...)` (L124) no acepta `py: Python`, a diferencia de openai/ollama que sí lo hacen y usan `py.detach()` para GIL release. Para liberar GIL necesita cambio de firma + caller update | `vantadb-litellm/src/python.rs:124-148` | 🟢 30min | 🔵 | ❌ |
| `DRV-070` | **Metadata no-string values silenciosamente ignorados** — L138: `v.extract::<String>()` descarta bool/int/float, igual que DRV-058/063. Tercera copia del mismo bug | `vantadb-litellm/src/python.rs:136-142` | 🟢 30min | 🔵 | ❌ |
| `DRV-071` | **RwLock<String> namespace con overhead concurrente innecesario** — Ídem DRV-059/065. Nunca escrito, solo 2 `.read().unwrap().clone()` | `vantadb-litellm/src/python.rs:38,100,130` | 🟢 15min | ⚪ | ❌ |
| `DRV-072` | **Sin método para cambiar namespace runtime** — Ídem DRV-060/066 | `vantadb-litellm/src/python.rs:43-148` | 🟢 30min | ℹ️ | ❌ |
| `DRV-073` | **Test coverage mínima: 4 tests/31L** — Ni siquiera test de store con metadata. Sin tests para error paths | `vantadb-litellm/tests/test_litellm.py:1-31` | 🟢 1h | ⚪ | ❌ |

### vantadb-mem0 (quick — 5 findings)

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-074` | **`delete_col()` solo pagina 1 — data loss** — `VantaMemoryListOptions::default()` tiene limit=100 (L326). Si collection tiene >100 registros, los de páginas posteriores sobreviven al delete. No es atómico ni completo | `vantadb-mem0/src/python.rs:324-344` | 🟡 2h | 🟠 | ❌ |
| `DRV-075` | **`search()` ignora text_query** — `let _ = query;` (L202) descarta el texto de búsqueda de Mem0. Solo vector search, sin hybrid/text-reranking. Gap de feature | `vantadb-mem0/src/python.rs:201-202` | 🟢 30min | ⚪ | ❌ |
| `DRV-076` | **`update()` TOCTOU entre get() y put()** — Dos `py.detach()` separados (L268, L289) permiten modificación concurrente entre lectura y escritura del registro | `vantadb-mem0/src/python.rs:254-291` | 🟢 1h | ⚪ | ❌ |
| `DRV-077` | **Collection name sanitización lazy/late** — `create_col()` guarda el raw name (L144), sanitización solo ocurre al usar como namespace. Namespace efectivo ≠ collection_name si contiene chars inválidos | `vantadb-mem0/src/python.rs:88-93,137-146` | 🟢 30min | ⚪ | ❌ |
| `DRV-078` | **Test coverage: 8 tests/60L** — Sin tests para: `delete_col()` multi-page, `update()` con vector, `create_col()`, `list_cols()`, `reset()`, edge cases (empty vectors, empty strings) | `vantadb-mem0/tests/test_mem0.py:1-60` | 🟢 1h | ⚪ | ❌ |

### vantadb-letta (quick — 6 findings)

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-079` | **`list_memories` solo pagina 1 — truncación** — `VantaMemoryListOptions::default()` (L128) limit=100. Si user/agent tiene >100 memorias, las extra no aparecen. Mismo patrón que DRV-074 | `vantadb-letta/src/python.rs:122-139` | 🟡 2h | 🔵 | ❌ |
| `DRV-080` | **`retrieve_memory` expone distancia VantaDB raw** — `hit.score` (L116) pasa distancia Cosine (0→2, 0=idéntico) directa. Consumidores Letta esperan score 0-1. Sin normalización vs `vanta_distance_to_mem0_score` en mem0 | `vantadb-letta/src/python.rs:111-118` | 🟢 30min | ⚪ | ❌ |
| `DRV-081` | **`AtomicU64 counter` reset en restart** — Counter inicializado en 0 (L64). Tras cerrar/reabrir store, nuevos inserts pueden colisionar con memorias existentes. Para agentes efímeros, impacto bajo | `vantadb-letta/src/python.rs:64,76-77` | 🟢 30min | ⚪ | ❌ |
| `DRV-082` | **Test coverage: 5 tests/42L** — Sin tests para: retrieve vacío, múltiples agentes, invalid memory_id, memory_id con `:` extra, payloads grandes | `vantadb-letta/tests/test_letta.py:1-42` | 🟢 1h | ⚪ | ❌ |
| `DRV-083` | **`store_memory` sin dedup** — Mismo user/agent/content almacenado dos veces → dos entradas separadas. Sin idempotencia | `vantadb-letta/src/python.rs:67-85` | 🟢 30min | ℹ️ | ❌ |
| `DRV-084` | **No `delete_col` ni `reset`** — No hay API para limpiar todas las memorias de un user/agent. Letta consumers no pueden resetear estado | `vantadb-letta/src/python.rs:42-163` | 🟢 1h | ℹ️ | ❌ |

### vantadb-crewai (quick — 6 findings)

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-085` | **`clear()` solo pagina 1 — data loss** — `Default::default()` limit=100 (L126). Si namespace tiene >100 records, el resto sobrevive al clear. Mismo bug que DRV-074/079 | `vantadb-crewai/src/python.rs:120-138` | 🟡 2h | 🔵 | ❌ |
| `DRV-086` | **Metadata no-string values silenciosamente ignorados** — `py_dict_to_string_map` (L144): `v.extract::<String>()` descarta Bool/Int/Float. Mismo bug que DRV-058/063/070. A diferencia de openai/ollama/litellm, metadata se serializa a JSON con serde_json, pero los valores no-string se pierden antes | `vantadb-crewai/src/python.rs:141-148` | 🟢 30min | 🔵 | ❌ |
| `DRV-087` | **RwLock\<String\> namespace con overhead innecesario** — 0 `.write()` calls, solo 3 `.read().unwrap().clone()` (L65, L90, L121). Dead lock. Mismo que DRV-059/065/071 | `vantadb-crewai/src/python.rs:37,65,90,121` | 🟢 15min | ⚪ | ❌ |
| `DRV-088` | **serde_json dependency extra** — Único adapter que añade serde_json para convertir BTreeMap<String,String> a String. Podría ser format!/join sin dependencia | `vantadb-crewai/Cargo.toml:20` | 🟢 15min | ⚪ | ❌ |
| `DRV-089` | **Test coverage: 5 tests/35L** — Sin tests para: metadata con non-string values, empty embedding, threshold filtering, clear con >100 records | `vantadb-crewai/tests/test_crewai.py:1-35` | 🟢 1h | ⚪ | ❌ |
| `DRV-090` | **search() threshold filtering post-GIL** — Score normalization (L107) ocurre tras `py.detach()`, después de recibir todos los hits. Para top_k=1000 con threshold alto, se traen 1000 results para posiblemente devolver 0 | `vantadb-crewai/src/python.rs:105-116` | 🟢 1h | ⚪ | ❌ |

### vantadb-dspy (quick — 5 findings)

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-091` | **RwLock\<String\> collection dead overhead** — `collection` nunca escrito (0 `.write()`), solo 2 `.read().unwrap().clone()`. Mismo patrón que DRV-059/065/071/087 | `vantadb-dspy/src/python.rs:37,60,94` | 🟢 15min | ⚪ | ❌ |
| `DRV-092` | **Metadata no-string values silenciosamente ignorados** — `add_passage()` L102: `v.extract::<String>()` descarta Bool/Int/Float. Mismo bug que DRV-058/063/070/086 (pero superficie menor — DSPy raramente usa metadata rica) | `vantadb-dspy/src/python.rs:100-107` | 🟢 30min | ⚪ | ❌ |
| `DRV-093` | **forward() expone distancia VantaDB raw** — `hit.score` (L79) pasa Cosine distance (0→2) directa. Sin normalización como crewai. Mismo que DRV-080 | `vantadb-dspy/src/python.rs:76-81` | 🟢 30min | ⚪ | ❌ |
| `DRV-094` | **AtomicU64 counter reset en restart** — Mismo que DRV-081. Counter en 0 al abrir store; nuevos inserts pueden colisionar con existentes | `vantadb-dspy/src/python.rs:38,95-96` | 🟢 30min | ℹ️ | ❌ |
| `DRV-095` | **Test coverage: 6 tests/39L** — Cubre init, add+forward, metadata, empty forward. Sin tests para: múltiples passages, metadata con non-string, large payloads | `vantadb-dspy/tests/test_dspy.py:1-39` | 🟢 1h | ⚪ | ❌ |

### vantadb-haystack (quick — 6 findings)

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-096` | **RwLock\<String\> namespace dead overhead** — `namespace` nunca escrito (0 `.write()`), solo 5 `.read().unwrap().clone()` (L59/123/152/164). Dead lock. Mismo que DRV-059/065/071/087/091 | `vantadb-haystack/src/python.rs:37,59,123,152,164` | 🟢 15min | ⚪ | ❌ |
| `DRV-097` | **count_documents() truncates at 100** — Usa `Default::default()` como VantaMemoryListOptions, que tiene `limit: Some(100)`. Si namespace tiene >100 docs, `page.records.len()` devuelve 100. Mismo bug que DRV-074/079/085. Carga además todos los records en memoria solo para contar | `vantadb-haystack/src/python.rs:167-171` | 🟢 1h | 🔵 | ❌ |
| `DRV-098` | **Metadata no-string values silenciosamente ignorados en write_documents** — `write_documents` L92: `entry.1.extract::<String>()` descarta Bool/Int/Float. Pero `py_dict_to_vanta_metadata` (usado por `filter_documents`) maneja los 4 tipos correctamente (L181-189). Inconsistencia intra-archivo: metadata no-string se pierde al escribir pero se parsea al filtrar. Documentos escritos con metadata no-string no son encontrables por filtro | `vantadb-haystack/src/python.rs:88-98` | 🟢 30min | 🔵 | ❌ |
| `DRV-099` | **No implementa protocolo Haystack Document real** — Métodos aceptan/retornan `list[dict]` en vez de `list[haystack.dataclasses.Document]`. No maneja `DuplicatePolicy`. No retorna `int` de `write_documents`. No es compatible con pipelines Haystack reales sin conversión manual. La .pyi solo declara dicts — no importa `haystack` en absoluto | `vantadb-haystack/src/python.rs:42-173`, `vantadb-haystack/vantadb_haystack.pyi:1-8` | 🟡 4h | 🔴 | ✅ |
| `DRV-100` | **Test coverage: 7 tests/48L** — Cubre init (2), write+filter, metadata write, count, delete. Sin tests para: filter con filtros reales, empty results, count >100 docs, metadata no-string, auto-generated IDs, error paths (path inválido, namespace vacío) | `vantadb-haystack/tests/test_haystack.py:1-48` | 🟢 1h | ⚪ | ❌ |
| `DRV-101` | **AtomicU64 doc_counter no persistido** — Counter se reinicia a 0 al abrir store. Nuevos inserts pueden generar IDs que colisionan con docs existentes (mismo que DRV-081/094) | `vantadb-haystack/src/python.rs:38,68-70` | 🟢 30min | ℹ️ | ❌ |

### vantadb-langchain (quick — 7 findings)

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-102` | **Missing GIL release en TODOS los métodos** — `add_texts` (L82-85), `similarity_search_by_vector` (L109-112), `delete` (L126-133) corren sin `py.detach()`/`py.allow_threads()`. Parámetro `py: Python` se recibe en 2/3 métodos pero nunca se usa. Contraste: los otros 7 adapters (haystack, dspy, crewai, mem0, letta, openai, ollama, litellm) usan `py.detach()` correctamente | `vantadb-langchain/src/python.rs:82-85,109-112,126-133` | 🟢 1h | 🔴 | ✅ |
| `DRV-103` | **Metadata no-string values silenciosamente ignorados** — `v.extract::<String>()` (L73) descarta Bool/Int/Float. Mismo bug que DRV-058/063/070/086/092/098 | `vantadb-langchain/src/python.rs:70-78` | 🟢 30min | 🔵 | ❌ |
| `DRV-104` | **similarity_search_by_vector no retorna metadata** — Solo id/text/score (L116-120). Metadata almacenada via `add_texts` se pierde en la respuesta. LangChain espera metadata para filtering en pipeline | `vantadb-langchain/src/python.rs:114-121` | 🟢 30min | 🔵 | ❌ |
| `DRV-105` | **delete() silenciosamente no-op en IDs malformados** — `id.split(':')` (L127) con `parts.len() != 2` (L128) ignora el delete sin error ni warning. Si un ID externo no tiene formato `namespace:key`, el delete falla silenciosamente | `vantadb-langchain/src/python.rs:125-133` | 🟢 30min | 🔵 | ❌ |
| `DRV-106` | **from_texts class method no implementado pese a docstring** — Docstring (L11) afirma implementar VectorStore protocol incluyendo `from_texts`, pero el método no existe en código. LangChain usa `from_texts` como entry point principal para crear stores | `vantadb-langchain/src/python.rs:11` | 🟡 2h | 🟡 | ❌ |
| `DRV-107` | **Test coverage: 5 tests/43L** — Cubre init, add+search, metadata, delete, unique keys. Sin tests para: delete con IDs malformados, empty texts/embeddings, metadata no-string, metadata return en search, large payloads, error paths | `vantadb-langchain/tests/test_langchain.py:1-43` | 🟢 1h | ⚪ | ❌ |
| `DRV-108` | **AtomicU64 counter no persistido** — Counter se reinicia a 0 al abrir store (mismo que DRV-081/094/101) | `vantadb-langchain/src/python.rs:23,59,63` | 🟢 30min | ℹ️ | ❌ |

### vantadb-llamaindex (quick — 6 findings)

Nota: El código es casi byte-for-byte idéntico a `vantadb-langchain`. Los hallazgos duplican DRV-102→108. Solo cambian nombres de métodos (`add`/`query` vs `add_texts`/`similarity_search_by_vector`).

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `DRV-109` | **Missing GIL release en TODOS los métodos** — `add` (L82-85), `query` (L104-107), `delete` (L124-126) corren sin `py.detach()`. Mismo bug que DRV-102 | `vantadb-llamaindex/src/python.rs:82-85,104-107,124-126` | 🟢 1h | 🔴 | ✅ |
| `DRV-110` | **Metadata no-string values silenciosamente ignorados** — `v.extract::<String>()` (L73). Mismo bug que DRV-103 | `vantadb-llamaindex/src/python.rs:70-78` | 🟢 30min | 🔵 | ❌ |
| `DRV-111` | **query() no retorna metadata** — Solo id/text/score (L112-114). Metadata almacenada via `add` se pierde. Mismo bug que DRV-104 | `vantadb-llamaindex/src/python.rs:109-116` | 🟢 30min | 🔵 | ❌ |
| `DRV-112` | **delete() silenciosamente no-op en IDs malformados** — `split(':')` con `parts.len() != 2` ignora error. Mismo bug que DRV-105 | `vantadb-llamaindex/src/python.rs:120-128` | 🟢 30min | 🔵 | ❌ |
| `DRV-113` | **Test coverage: 5 tests/43L** — Idéntico a langchain. Sin tests para: delete con IDs malformados, empty, metadata return, no-string metadata | `vantadb-llamaindex/tests/test_llamaindex.py:1-43` | 🟢 1h | ⚪ | ❌ |
| `DRV-114` | **AtomicU64 counter no persistido** — Mismo que DRV-108 | `vantadb-llamaindex/src/python.rs:23,59,63` | 🟢 30min | ℹ️ | ❌ |

| ID | Tarea | Archivo | Esfuerzo | Prioridad | Estado |
|----|-------|---------|----------|-----------|--------|
| `REV-001` | **CI: Rust fails on main — TSan ABI mismatch** — `-Zsanitizer=thread` incompatible con toolchain 1.94.1 | `.github/workflows/ci-rust-10.yml` → H05-ERROR-001 | 🟢 2h | 🔴 | ✅ |
| `REV-002` | **CI: Web fails on main — 21 lint issues** — 14 ESLint errors + 7 warnings rompen build | `.github/workflows/ci-web-11.yml` → H05-ERROR-002 | 🟢 2h | 🔴 | ✅ |
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
| `REV-017` | **`why-vantadb.tsx` prettier error** — Trailing newline rompe formateo | `web/src/routes/why-vantadb.tsx:43` → H03-CODE-003 | 🟢 5min | 🟢 | ✅ |
| `REV-018` | **`NbToast.tsx` react-refresh warning** — Archivo exporta más que solo componentes | `web/src/components/nb/NbToast.tsx:15` → H03-CODE-004 | 🟢 5min | 🟢 | ✅ |

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
                ─ Code health: VFY-001→012, **REV-003→018**, **DRV-001→045**, **DRV-046→056**, **DRV-057→061**, **DRV-062→067**, **DRV-068→073**, **DRV-074→078**, **DRV-079→084**, **DRV-085→090**, **DRV-091→095**, **DRV-096→101**, **DRV-102→108**, **DRV-109→114**
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
